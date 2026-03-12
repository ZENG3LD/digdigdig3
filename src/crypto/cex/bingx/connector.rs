//! # BingX Connector
//!
//! Реализация всех core трейтов для BingX.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции

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
    Order, OrderSide, OrderType,Balance, AccountInfo,
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
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BingxUrls, BingxEndpoint, format_symbol, map_kline_interval};
use super::auth::BingxAuth;
use super::parser::BingxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// BingX коннектор
pub struct BingxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<BingxAuth>,
    /// URL'ы
    urls: BingxUrls,
    /// Rate limiter для market data (100 req/10s)
    market_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BingxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, _testnet: bool) -> ExchangeResult<Self> {
        // BingX doesn't have a public testnet, always use mainnet
        let urls = BingxUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(BingxAuth::new)
            .transpose()?;

        // BingX rate limit: 100 requests per 10 seconds (shared pool)
        let market_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            market_limiter,
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(_testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, false).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.market_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire() {
                    return;
                }
                limiter.time_until_ready()
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request(&mut params)
        } else {
            HashMap::new()
        };

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

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        // Build form body
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        let response = self.http.post(&url, &json!({}), &headers).await?;
        Ok(response)
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: BingxEndpoint,
        mut params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        // Add auth signature
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(&mut params);

        // Build query string
        let query = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let url = format!("{}{}?{}", base_url, path, query);

        let response = self.http.delete(&url, &HashMap::new(), &headers).await?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BingxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::BingX
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.market_limiter.lock() {
            (lim.current_count(), lim.max_requests())
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
        false // BingX doesn't have public testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,
            AccountType::FuturesCross,
            AccountType::FuturesIsolated,
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
impl MarketData for BingxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Use get_ticker and extract the last_price
        let ticker = self.get_ticker(symbol, account_type).await?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotDepth,
            _ => BingxEndpoint::SwapDepth,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("limit".to_string(), d.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotKlines,
            _ => BingxEndpoint::SwapKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("interval".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1440).to_string());
        }

        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotTickerBookTicker,
            _ => BingxEndpoint::SwapTicker,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // BingX doesn't have dedicated ping endpoint, use symbols as health check
        let response = self.get(BingxEndpoint::SpotSymbols, HashMap::new(), AccountType::Spot).await?;

        // Check response is valid
        if response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) == 0 {
            Ok(())
        } else {
            Err(ExchangeError::Network("Ping failed".to_string()))
        }
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let response = self.get(BingxEndpoint::SpotSymbols, HashMap::new(), AccountType::Spot).await?;
                BingxParser::parse_spot_exchange_info(&response)
            }
            _ => {
                let response = self.get(BingxEndpoint::SwapContracts, HashMap::new(), AccountType::FuturesCross).await?;
                BingxParser::parse_swap_exchange_info(&response)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BingxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let endpoint = match account_type {
                            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOrder,
                            _ => BingxEndpoint::SwapOrder,
                        };
                
                        let mut params = HashMap::new();
                        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                        params.insert("side".to_string(), match side {
                            OrderSide::Buy => "BUY".to_string(),
                            OrderSide::Sell => "SELL".to_string(),
                        });
                        params.insert("type".to_string(), "MARKET".to_string());
                
                        // BingX Spot uses quoteOrderQty for market orders
                        // Swap uses quantity
                        match account_type {
                            AccountType::Spot | AccountType::Margin => {
                                params.insert("quoteOrderQty".to_string(), quantity.to_string());
                            }
                            _ => {
                                params.insert("quantity".to_string(), quantity.to_string());
                            }
                        }
                
                        let response = self.post(endpoint, params, account_type).await?;
                        BingxParser::parse_order(&response, &symbol.to_string()).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let endpoint = match account_type {
                            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOrder,
                            _ => BingxEndpoint::SwapOrder,
                        };
                
                        let mut params = HashMap::new();
                        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                        params.insert("side".to_string(), match side {
                            OrderSide::Buy => "BUY".to_string(),
                            OrderSide::Sell => "SELL".to_string(),
                        });
                        params.insert("type".to_string(), "LIMIT".to_string());
                        params.insert("quantity".to_string(), quantity.to_string());
                        params.insert("price".to_string(), price.to_string());
                
                        let response = self.post(endpoint, params, account_type).await?;
                        BingxParser::parse_order(&response, &symbol.to_string()).map(PlaceOrderResponse::Simple)
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

            let endpoint = match account_type {
                AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOrder,
                _ => BingxEndpoint::SwapOrder,
            };

            let mut params = HashMap::new();
            params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
            params.insert("orderId".to_string(), order_id.to_string());

            let response = self.delete(endpoint, params, account_type).await?;
            BingxParser::parse_order(&response, &symbol.to_string())
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOrder,
            _ => BingxEndpoint::SwapOrder,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_order(&response, &symbol.to_string())
    
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

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotOpenOrders,
            _ => BingxEndpoint::SwapOpenOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(endpoint, params, account_type).await?;
        BingxParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BingxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => BingxEndpoint::SpotBalance,
            _ => BingxEndpoint::SwapBalance,
        };

        let params = HashMap::new();
        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => BingxParser::parse_balances(&response),
            _ => BingxParser::parse_swap_balance(&response),
        }
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1, // Default BingX fees, should query from API
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
impl Positions for BingxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(BingxEndpoint::SwapPositions, params, account_type).await?;
        BingxParser::parse_positions(&response)
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
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

        // BingX doesn't have a dedicated funding rate endpoint accessible via REST
        // Would need to implement via WebSocket or parse from contract info
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate not available via REST API".to_string()
        ))
    
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

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
                params.insert("side".to_string(), "LONG".to_string()); // BingX requires side
                params.insert("leverage".to_string(), leverage.to_string());

                let response = self.post(BingxEndpoint::SwapLeverage, params, account_type).await?;

                // Check response for errors
                if response.get("code").and_then(|c| c.as_i64()).unwrap_or(-1) != 0 {
                let msg = response.get("msg")
                .and_then(|m| m.as_str())
                .unwrap_or("Failed to set leverage");
                return Err(ExchangeError::Api {
                code: -1,
                message: msg.to_string(),
                });
                }

                Ok(())
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}
