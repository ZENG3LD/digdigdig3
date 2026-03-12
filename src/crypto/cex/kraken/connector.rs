//! # Kraken Connector
//!
//! Implementation of all core traits for Kraken.
//!
//! ## Core traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - futures positions

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
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::DecayingRateLimiter;

use super::endpoints::{KrakenUrls, KrakenEndpoint, format_symbol, map_ohlc_interval};
use super::auth::KrakenAuth;
use super::parser::KrakenParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Kraken connector
pub struct KrakenConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<KrakenAuth>,
    /// URLs (mainnet/testnet)
    urls: KrakenUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (Kraken Spot Starter tier: max=15, decay=0.33/s)
    rate_limiter: Arc<Mutex<DecayingRateLimiter>>,
}

impl KrakenConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            KrakenUrls::TESTNET
        } else {
            KrakenUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(KrakenAuth::new)
            .transpose()?;

        // Initialize rate limiter: Kraken Spot Starter tier (max=15, decay=0.33/s)
        let rate_limiter = Arc::new(Mutex::new(
            DecayingRateLimiter::new(15.0, 0.33)
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(1.0) {
                    return;
                }
                limiter.time_until_ready(1.0)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: KrakenEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

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
        Ok(response)
    }

    /// POST request (Spot API uses POST for both public and private)
    ///
    /// Note: Kraken expects application/x-www-form-urlencoded, but our HttpClient
    /// always sends JSON. As a workaround, we send form params as query params
    /// since Kraken private endpoints accept parameters in either the body or URL.
    async fn post(
        &self,
        endpoint: KrakenEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            // Sign request to get headers and form body
            let (headers, _body_str) = auth.sign_request(path, &params);

            // Build URL with path
            let url = format!("{}{}", base_url, path);

            // Use post_with_params - sends params as query string
            // The signature covers the POST body, but Kraken also accepts params in URL
            self.http.post_with_params(&url, &params, &json!({}), &headers).await
        } else {
            // Public POST endpoints (rare for Kraken)
            let url = format!("{}{}", base_url, path);
            self.http.post_with_params(&url, &params, &json!({}), &HashMap::new()).await
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Kraken-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all asset pairs information
    pub async fn get_asset_pairs(&self) -> ExchangeResult<Value> {
        self.get(KrakenEndpoint::SpotAssetPairs, HashMap::new(), AccountType::Spot).await
    }

    /// Get WebSocket authentication token
    pub async fn get_ws_token(&self) -> ExchangeResult<String> {
        let response = self.post(
            KrakenEndpoint::SpotWebSocketToken,
            HashMap::new(),
            AccountType::Spot,
        ).await?;

        let result = KrakenParser::extract_result(&response)?;
        result.get("token")
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or_else(|| ExchangeError::Parse("Missing WebSocket token".to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for KrakenConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Kraken
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_level() as u32, lim.max_level() as u32)
        } else {
            (0, 0)
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups: Vec::new(),
            ws_ping_rtt_ms: 0,
        }
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
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
impl MarketData for KrakenConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());

        let response = self.get(KrakenEndpoint::SpotTicker, params, account_type).await?;

        // Response will use full format (XXBTZUSD), try both formats
        KrakenParser::parse_price(&response, &formatted)
            .or_else(|_| {
                // Try with XX prefix for BTC
                let full_format = if formatted.starts_with("XBT")
                    || formatted.starts_with("ETH")
                    || formatted.starts_with("LTC") {
                    format!("X{}", formatted)
                } else {
                    formatted.clone()
                };
                // Add Z prefix for USD
                let full_format = if full_format.ends_with("USD") {
                    format!("{}Z{}", &full_format[..full_format.len()-3], "USD")
                } else {
                    full_format
                };
                KrakenParser::parse_price(&response, &full_format)
            })
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());
        if let Some(d) = depth {
            params.insert("count".to_string(), d.to_string());
        }

        let response = self.get(KrakenEndpoint::SpotOrderbook, params, account_type).await?;

        // Try with different symbol formats
        KrakenParser::parse_orderbook(&response, &formatted)
            .or_else(|_| {
                let full_format = Self::to_full_format(&formatted);
                KrakenParser::parse_orderbook(&response, &full_format)
            })
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());
        params.insert("interval".to_string(), map_ohlc_interval(interval).to_string());

        let response = self.get(KrakenEndpoint::SpotOHLC, params, account_type).await?;

        KrakenParser::parse_klines(&response, &formatted)
            .or_else(|_| {
                let full_format = Self::to_full_format(&formatted);
                KrakenParser::parse_klines(&response, &full_format)
            })
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());

        let response = self.get(KrakenEndpoint::SpotTicker, params, account_type).await?;

        KrakenParser::parse_ticker(&response, &formatted)
            .or_else(|_| {
                let full_format = Self::to_full_format(&formatted);
                KrakenParser::parse_ticker(&response, &full_format)
            })
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(KrakenEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        KrakenParser::extract_result(&response)?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get_asset_pairs().await?;
        KrakenParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for KrakenConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                
                        let mut params = HashMap::new();
                        params.insert("pair".to_string(), formatted);
                        params.insert("type".to_string(), match side {
                            OrderSide::Buy => "buy".to_string(),
                            OrderSide::Sell => "sell".to_string(),
                        });
                        params.insert("ordertype".to_string(), "market".to_string());
                        params.insert("volume".to_string(), quantity.to_string());
                
                        let response = self.post(KrakenEndpoint::SpotAddOrder, params, account_type).await?;
                        let order_id = KrakenParser::parse_order_id(&response)?;
                
                        // Return minimal order info
                        Ok(PlaceOrderResponse::Simple(Order {
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
                        }))
            }
            OrderType::Limit { price } => {
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                
                        let mut params = HashMap::new();
                        params.insert("pair".to_string(), formatted);
                        params.insert("type".to_string(), match side {
                            OrderSide::Buy => "buy".to_string(),
                            OrderSide::Sell => "sell".to_string(),
                        });
                        params.insert("ordertype".to_string(), "limit".to_string());
                        params.insert("price".to_string(), price.to_string());
                        params.insert("volume".to_string(), quantity.to_string());
                
                        let response = self.post(KrakenEndpoint::SpotAddOrder, params, account_type).await?;
                        let order_id = KrakenParser::parse_order_id(&response)?;
                
                        Ok(PlaceOrderResponse::Simple(Order {
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
                        }))
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
            params.insert("txid".to_string(), order_id.to_string());

            let response = self.post(KrakenEndpoint::SpotCancelOrder, params, account_type).await?;
            KrakenParser::extract_result(&response)?;

            // Return cancelled order (minimal info)
            Ok(Order {
                id: order_id.to_string(),
                client_order_id: None,
                symbol: symbol.to_string(),
                side: OrderSide::Buy,
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
        params.insert("txid".to_string(), order_id.to_string());

        let response = self.post(KrakenEndpoint::SpotGetOrder, params, account_type).await?;
        KrakenParser::parse_order(&response, order_id)
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let _symbol_str = _symbol;
        let _symbol: Option<crate::core::Symbol> = _symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let params = HashMap::new();
        let response = self.post(KrakenEndpoint::SpotOpenOrders, params, account_type).await?;
        KrakenParser::parse_open_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for KrakenConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let params = HashMap::new();
        let response = self.post(KrakenEndpoint::SpotBalance, params, account_type).await?;
        KrakenParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.16, // Kraken default maker fee (varies by tier)
            taker_commission: 0.26, // Kraken default taker fee
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
impl Positions for KrakenConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let response = self.get(
            KrakenEndpoint::FuturesOpenPositions,
            HashMap::new(),
            account_type,
        ).await?;

        KrakenParser::parse_futures_positions(&response)
    
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let symbol_str = symbol;
        let symbol = {
            let parts: Vec<&str> = symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.to_string(), quote: String::new(), raw: Some(symbol_str.to_string()) }
            }
        };

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), formatted.clone());

        let response = self.get(
            KrakenEndpoint::FuturesHistoricalFunding,
            params,
            account_type,
        ).await?;

        KrakenParser::parse_funding_rate(&response, &formatted)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                "Leverage not supported for Spot/Margin".to_string()
                ));
                }
                _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("maxLeverage".to_string(), leverage.to_string());

                let response = self.post(
                KrakenEndpoint::FuturesSetLeverage,
                params,
                account_type,
                ).await?;

                KrakenParser::extract_futures_data(&response)?;
                Ok(())
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// Helper methods
impl KrakenConnector {
    /// Convert simplified symbol to full ISO format
    ///
    /// XBTUSD → XXBTZUSD
    /// ETHUSD → XETHZUSD
    fn to_full_format(symbol: &str) -> String {
        // Common conversions
        let mut result = symbol.to_string();

        // Add X prefix to crypto if not present
        if (result.starts_with("XBT") && !result.starts_with("XXBT"))
            || ((result.starts_with("ETH") || result.starts_with("LTC"))
                && !result.starts_with("XETH") && !result.starts_with("XLTC")) {
            result = format!("X{}", result);
        }

        // Add Z prefix to fiat if not present
        if result.ends_with("USD") && !result.ends_with("ZUSD") {
            let base = &result[..result.len() - 3];
            result = format!("{}ZUSD", base);
        } else if result.ends_with("EUR") && !result.ends_with("ZEUR") {
            let base = &result[..result.len() - 3];
            result = format!("{}ZEUR", base);
        }

        result
    }
}
