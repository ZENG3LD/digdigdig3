//! # Bithumb Connector
//!
//! Реализация всех core трейтов для Bithumb Pro.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции (limited support)

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::http::RetryConfig;
use crate::core::types::{
    WithdrawRequest, WithdrawResponse, DepositAddress,
    FundsHistoryFilter, FundsRecord, FundsRecordType,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions, CustodialFunds,
};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BithumbUrls, BithumbEndpoint, format_symbol, map_kline_interval};
use super::auth::BithumbAuth;
use super::parser::BithumbParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bithumb коннектор
pub struct BithumbConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BithumbAuth>,
    /// URL'ы (mainnet/testnet)
    urls: BithumbUrls,
    /// Testnet mode (note: Bithumb Pro doesn't have testnet)
    testnet: bool,
    /// Rate limiter для всех запросов (2 req/s - очень консервативно из-за проблем с инфраструктурой)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BithumbConnector {
    /// Создать новый коннектор
    ///
    /// Returns `ExchangeError::UnsupportedOperation` when `testnet = true` because
    /// Bithumb does not offer a testnet or sandbox environment.
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        if testnet {
            return Err(ExchangeError::UnsupportedOperation(
                "Bithumb does not support testnet mode — no sandbox environment is available.".to_string(),
            ));
        }
        let urls = BithumbUrls::MAINNET;

        // Bithumb API имеет известные проблемы с инфраструктурой (~20% запросов получают 504 Gateway Timeout)
        // Используем специальную конфигурацию retry с:
        // - 7 попыток (вместо 3)
        // - Более короткий таймаут (10s вместо 30s) с более быстрым exponential backoff
        // - Jitter для избежания thundering herd
        let retry_config = RetryConfig::unreliable_api();
        let http = HttpClient::with_config(10_000, retry_config)?; // 10 sec timeout

        let auth = credentials
            .as_ref()
            .map(BithumbAuth::new)
            .transpose()?;

        // Bithumb has poor documentation and flaky infrastructure
        // Use VERY conservative rate limit: 120 requests per 60 seconds
        // This prevents overwhelming their servers and triggering 504 errors
        // Slower requests = fewer retries = faster overall
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(120, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self) {
        let wait_time = {
            let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
            if !limiter.try_acquire() {
                limiter.time_until_ready()
            } else {
                Duration::ZERO
            }
        };

        if !wait_time.is_zero() {
            tokio::time::sleep(wait_time).await;
            // Try again after waiting
            let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
            limiter.try_acquire();
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: BithumbEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Apply rate limiting BEFORE making the request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth params if needed
        if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            params = auth.sign_request(&mut params);
        }

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        let response = self.http.get(&url, &HashMap::new()).await?;
        BithumbParser::check_response(&response)?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BithumbEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Apply rate limiting BEFORE making the request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth params
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let signed_params = auth.sign_request(&mut params);

        // Convert to JSON
        let body = json!(signed_params);

        let response = self.http.post(&url, &body, &HashMap::new()).await?;
        BithumbParser::check_response(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Bithumb-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить информацию о символах
    pub async fn get_config(&self) -> ExchangeResult<Value> {
        self.get(BithumbEndpoint::SpotConfig, HashMap::new(), AccountType::Spot).await
    }

    /// Получить server time
    pub async fn get_server_time(&self) -> ExchangeResult<i64> {
        let response = self.get(BithumbEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        let data = BithumbParser::extract_data(&response)?;
        data.as_i64()
            .ok_or_else(|| ExchangeError::Parse("Invalid server time".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // C3 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Query a single order by order ID.
    ///
    /// `POST /spot/singleOrder`
    /// Required parameter: `ordId` (order ID string).
    pub async fn get_single_order(&self, order_id: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("ordId".to_string(), order_id.to_string());
        self.post(BithumbEndpoint::SingleOrder, params, AccountType::Spot).await
    }

    /// List all available assets with their trading/deposit/withdrawal status.
    ///
    /// `POST /spot/assetList`
    /// Optional parameter: `assetType` ("spot" or "futures").
    pub async fn get_asset_list(&self, asset_type: Option<&str>) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(t) = asset_type {
            params.insert("assetType".to_string(), t.to_string());
        }
        self.post(BithumbEndpoint::AssetList, params, AccountType::Spot).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BithumbConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bithumb
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::FuturesCross,
            // Bithumb has separate platforms:
            // - Bithumb Pro: spot trading
            // - Bithumb Futures: perpetual futures
        ]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BithumbConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => BithumbEndpoint::FuturesTicker,
            _ => BithumbEndpoint::SpotTicker,
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => BithumbEndpoint::FuturesOrderbook,
            _ => BithumbEndpoint::SpotOrderbook,
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                // Bithumb Futures uses "interval" parameter
                params.insert("interval".to_string(), map_kline_interval(interval, account_type));
                BithumbEndpoint::FuturesKlines
            }
            _ => {
                // Bithumb Pro Spot uses "type" parameter
                params.insert("type".to_string(), map_kline_interval(interval, account_type));

                // Bithumb Pro requires start and end timestamps
                // Use last 24 hours as default
                let end = crate::core::timestamp_millis() / 1000; // seconds
                let start = end - 86400; // 24 hours ago
                params.insert("start".to_string(), start.to_string());
                params.insert("end".to_string(), end.to_string());

                BithumbEndpoint::SpotKlines
            }
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let endpoint = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => BithumbEndpoint::FuturesTicker,
            _ => BithumbEndpoint::SpotTicker,
        };

        let response = self.get(endpoint, params, account_type).await?;
        BithumbParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _ = self.get_server_time().await?;
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BithumbConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let mut params = HashMap::new();
                        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                        params.insert("side".to_string(), match side {
                            OrderSide::Buy => "buy".to_string(),
                            OrderSide::Sell => "sell".to_string(),
                        });
                        params.insert("type".to_string(), "market".to_string());
                        params.insert("quantity".to_string(), quantity.to_string());
                
                        let response = self.post(BithumbEndpoint::SpotCreateOrder, params, account_type).await?;
                        let order_id = BithumbParser::parse_order_id(&response)?;
                
                        // Return minimal order info
                        Ok(Order {
                            id: order_id,
                            client_order_id: None,
                            symbol: symbol.to_string(),
                            side,
                            order_type: OrderType::Market,
                            status: crate::core::OrderStatus::New,
                            price: None,
                            stop_price: None,
                            quantity,
                            filled_quantity: 0.0,
                            average_price: None,
                            commission: None,
                            commission_asset: None,
                            created_at: crate::core::timestamp_millis() as i64,
                            updated_at: None,
                            time_in_force: crate::core::TimeInForce::Gtc,
                        })
            }
            OrderType::Limit { price } => {
                let mut params = HashMap::new();
                        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                        params.insert("side".to_string(), match side {
                            OrderSide::Buy => "buy".to_string(),
                            OrderSide::Sell => "sell".to_string(),
                        });
                        params.insert("type".to_string(), "limit".to_string());
                        params.insert("quantity".to_string(), quantity.to_string());
                        params.insert("price".to_string(), price.to_string());
                
                        let response = self.post(BithumbEndpoint::SpotCreateOrder, params, account_type).await?;
                        let order_id = BithumbParser::parse_order_id(&response)?;
                
                        Ok(Order {
                            id: order_id,
                            client_order_id: None,
                            symbol: symbol.to_string(),
                            side,
                            order_type: OrderType::Limit { price: 0.0 },
                            status: crate::core::OrderStatus::New,
                            price: Some(price),
                            stop_price: None,
                            quantity,
                            filled_quantity: 0.0,
                            average_price: None,
                            commission: None,
                            commission_asset: None,
                            created_at: crate::core::timestamp_millis() as i64,
                            updated_at: None,
                            time_in_force: crate::core::TimeInForce::Gtc,
                        })
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::UnsupportedOperation(
            "get_order_history not yet implemented".to_string()
        ))
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

            let mut params = HashMap::new();
            params.insert("orderId".to_string(), order_id.to_string());

            let response = self.post(BithumbEndpoint::SpotCancelOrder, params, account_type).await?;
            BithumbParser::check_response(&response)?;

            // Return cancelled order (minimal info)
            Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: OrderSide::Buy, // Unknown
                order_type: OrderType::Limit { price: 0.0 },
                status: crate::core::OrderStatus::Canceled,
                price: None,
                stop_price: None,
                quantity: 0.0,
                filled_quantity: 0.0,
                average_price: None,
                commission: None,
                commission_asset: None,
                created_at: 0,
                updated_at: Some(crate::core::timestamp_millis() as i64),
                time_in_force: crate::core::TimeInForce::Gtc,
            })
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let mut params = HashMap::new();
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.post(BithumbEndpoint::SpotOrderDetail, params, account_type).await?;
        BithumbParser::parse_order(&response, "")
    
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let symbol_str = symbol;
        let symbol: Option<crate::core::Symbol> = symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.post(BithumbEndpoint::SpotOpenOrders, params, account_type).await?;
        BithumbParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BithumbConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let params = HashMap::new();
        let response = self.post(BithumbEndpoint::SpotAccount, params, account_type).await?;
        BithumbParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1, // Default, Bithumb Pro uses tiered fees
            taker_commission: 0.1,
            balances,
        })
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::UnsupportedOperation(
            "get_fees not yet implemented".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BithumbConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let account_type = query.account_type;

        // Bithumb Pro primarily supports spot trading
        // Futures API is limited and not well documented
        Err(ExchangeError::UnsupportedOperation(
            format!("Positions not supported for {:?}", account_type)
        ))
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let _symbol_str = _symbol;
        let _symbol = {
            let parts: Vec<&str> = _symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: _symbol_str.to_string(), quote: String::new(), raw: Some(_symbol_str.to_string()) }
            }
        };

        Err(ExchangeError::UnsupportedOperation(
            format!("Funding rate not supported for {:?}", account_type)
        ))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: account_type } => {
                let _symbol = _symbol.clone();

                Err(ExchangeError::UnsupportedOperation(
                format!("Set leverage not supported for {:?}", account_type)
                ))

            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bithumb Pro supports custodial deposit/withdrawal operations.
///
/// - Deposit address: `POST /wallet/depositAddress`
/// - Withdraw: `POST /withdraw`
/// - Deposit history: `POST /wallet/depositHistory`
/// - Withdrawal history: `POST /wallet/withdrawHistory`
#[async_trait]
impl CustodialFunds for BithumbConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let mut params = HashMap::new();
        params.insert("coinType".to_string(), asset.to_uppercase());
        if let Some(net) = network {
            params.insert("chain".to_string(), net.to_string());
        }

        let response = self.post(
            BithumbEndpoint::SpotDepositAddress,
            params,
            AccountType::Spot,
        ).await?;

        let data = BithumbParser::extract_data(&response)?;

        let address = data.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'address' field".to_string()))?
            .to_string();

        let tag = data.get("tag")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let returned_network = data.get("chain")
            .and_then(|v| v.as_str())
            .map(String::from)
            .or_else(|| network.map(String::from));

        Ok(DepositAddress {
            address,
            tag,
            network: returned_network,
            asset: asset.to_uppercase(),
            created_at: None,
        })
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let mut params = HashMap::new();
        params.insert("coinType".to_string(), req.asset.to_uppercase());
        params.insert("quantity".to_string(), req.amount.to_string());
        params.insert("addr".to_string(), req.address.clone());

        if let Some(tag) = &req.tag {
            params.insert("destination".to_string(), tag.clone());
        }

        if let Some(network) = &req.network {
            params.insert("chain".to_string(), network.clone());
        }

        let response = self.post(
            BithumbEndpoint::SpotWithdraw,
            params,
            AccountType::Spot,
        ).await?;

        let data = BithumbParser::extract_data(&response)?;

        let withdraw_id = data.get("orderId")
            .or_else(|| data.get("withdrawId"))
            .and_then(|v| v.as_str())
            .map(String::from)
            .unwrap_or_else(|| "submitted".to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        match filter.record_type {
            FundsRecordType::Deposit => {
                let mut params = HashMap::new();
                if let Some(asset) = &filter.asset {
                    params.insert("coinType".to_string(), asset.to_uppercase());
                }
                if let Some(limit) = filter.limit {
                    params.insert("limit".to_string(), limit.to_string());
                }

                let response = self.post(
                    BithumbEndpoint::SpotDepositHistory,
                    params,
                    AccountType::Spot,
                ).await?;

                parse_deposit_history(&response)
            }
            FundsRecordType::Withdrawal => {
                let mut params = HashMap::new();
                if let Some(asset) = &filter.asset {
                    params.insert("coinType".to_string(), asset.to_uppercase());
                }
                if let Some(limit) = filter.limit {
                    params.insert("limit".to_string(), limit.to_string());
                }

                let response = self.post(
                    BithumbEndpoint::SpotWithdrawHistory,
                    params,
                    AccountType::Spot,
                ).await?;

                parse_withdrawal_history(&response)
            }
            FundsRecordType::Both => {
                let asset_upper = filter.asset.as_deref().map(str::to_uppercase);

                let mut dep_params = HashMap::new();
                let mut wit_params = HashMap::new();
                if let Some(ref asset) = asset_upper {
                    dep_params.insert("coinType".to_string(), asset.clone());
                    wit_params.insert("coinType".to_string(), asset.clone());
                }
                if let Some(limit) = filter.limit {
                    dep_params.insert("limit".to_string(), limit.to_string());
                    wit_params.insert("limit".to_string(), limit.to_string());
                }

                let dep_response = self.post(
                    BithumbEndpoint::SpotDepositHistory,
                    dep_params,
                    AccountType::Spot,
                ).await?;
                let wit_response = self.post(
                    BithumbEndpoint::SpotWithdrawHistory,
                    wit_params,
                    AccountType::Spot,
                ).await?;

                let mut records = parse_deposit_history(&dep_response)?;
                records.extend(parse_withdrawal_history(&wit_response)?);
                Ok(records)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDS HISTORY PARSING HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse deposit history response from Bithumb Pro.
///
/// Expected format:
/// ```json
/// {
///   "code": "0",
///   "data": {
///     "list": [
///       {
///         "coinType": "BTC",
///         "quantity": "0.5",
///         "txId": "abc123",
///         "chain": "BTC",
///         "status": "success",
///         "createTime": 1650000000000
///       }
///     ]
///   }
/// }
/// ```
fn parse_deposit_history(response: &serde_json::Value) -> ExchangeResult<Vec<FundsRecord>> {
    BithumbParser::check_response(response)?;
    let data = BithumbParser::extract_data(response)?;

    let items = data.get("list")
        .and_then(|v| v.as_array())
        .or_else(|| data.as_array())
        .ok_or_else(|| ExchangeError::Parse("Expected array in deposit history".to_string()))?;

    let mut records = Vec::with_capacity(items.len());
    for item in items {
        let id = item.get("id")
            .or_else(|| item.get("txId"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let asset = item.get("coinType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let amount = item.get("quantity")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| item.get("quantity").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);

        let tx_hash = item.get("txId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let network = item.get("chain")
            .and_then(|v| v.as_str())
            .map(String::from);

        let status = item.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let timestamp = item.get("createTime")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        records.push(FundsRecord::Deposit {
            id,
            asset,
            amount,
            tx_hash,
            network,
            status,
            timestamp,
        });
    }

    Ok(records)
}

/// Parse withdrawal history response from Bithumb Pro.
///
/// Expected format:
/// ```json
/// {
///   "code": "0",
///   "data": {
///     "list": [
///       {
///         "coinType": "BTC",
///         "quantity": "0.1",
///         "fee": "0.0005",
///         "addr": "1A2B3C...",
///         "destination": "",
///         "txId": "def456",
///         "chain": "BTC",
///         "status": "success",
///         "createTime": 1650000000000
///       }
///     ]
///   }
/// }
/// ```
fn parse_withdrawal_history(response: &serde_json::Value) -> ExchangeResult<Vec<FundsRecord>> {
    BithumbParser::check_response(response)?;
    let data = BithumbParser::extract_data(response)?;

    let items = data.get("list")
        .and_then(|v| v.as_array())
        .or_else(|| data.as_array())
        .ok_or_else(|| ExchangeError::Parse("Expected array in withdrawal history".to_string()))?;

    let mut records = Vec::with_capacity(items.len());
    for item in items {
        let id = item.get("id")
            .or_else(|| item.get("orderId"))
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let asset = item.get("coinType")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let amount = item.get("quantity")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| item.get("quantity").and_then(|v| v.as_f64()))
            .unwrap_or(0.0);

        let fee = item.get("fee")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse().ok())
            .or_else(|| item.get("fee").and_then(|v| v.as_f64()));

        let address = item.get("addr")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        let tag = item.get("destination")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let tx_hash = item.get("txId")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        let network = item.get("chain")
            .and_then(|v| v.as_str())
            .map(String::from);

        let status = item.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let timestamp = item.get("createTime")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        records.push(FundsRecord::Withdrawal {
            id,
            asset,
            amount,
            fee,
            address,
            tag,
            tx_hash,
            network,
            status,
            timestamp,
        });
    }

    Ok(records)
}
