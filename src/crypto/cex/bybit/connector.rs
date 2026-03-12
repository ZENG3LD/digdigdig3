//! # Bybit Connector
//!
//! Implementation of all core traits for Bybit V5 API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - futures positions
//!
//! ## Extended Methods
//! Additional Bybit-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
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
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{BybitUrls, BybitEndpoint, format_symbol, account_type_to_category, map_kline_interval};
use super::auth::BybitAuth;
use super::parser::BybitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bybit connector
pub struct BybitConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<BybitAuth>,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (120 requests per second)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl BybitConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials.as_ref().map(BybitAuth::new);

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = BybitUrls::base_url(testnet);
            let url = format!("{}/v5/market/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(time_sec) = response.get("result")
                    .and_then(|r| r.get("timeSecond"))
                    .and_then(|t| t.as_str())
                    .and_then(|s| s.parse::<i64>().ok())
                {
                    if let Some(ref mut a) = auth {
                        a.sync_time(time_sec * 1000); // Convert to milliseconds
                    }
                }
            }
        }

        // Initialize rate limiter: 600 requests per 5 seconds (Bybit IP global limit)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(600, Duration::from_secs(5))
        ));

        Ok(Self {
            http,
            auth,
            testnet,
            rate_limiter,
        })
    }

    /// Create connector only for public methods
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Bybit response headers
    ///
    /// Bybit reports: X-Bapi-Limit-Status = remaining, X-Bapi-Limit = total limit
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let remaining = headers
            .get("X-Bapi-Limit-Status")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let limit = headers
            .get("X-Bapi-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        if let (Some(remaining), Some(limit)) = (remaining, limit) {
            let used = limit.saturating_sub(remaining);
            if let Ok(mut limiter) = self.rate_limiter.lock() {
                limiter.update_from_server(used);
            }
        }
    }

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, weight: u32) {
        loop {
            // Scope the lock to ensure it's dropped before await
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return; // Successfully acquired, exit early
                }
                limiter.time_until_ready(weight)
            }; // Lock is dropped here

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: BybitEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit (weight 1 for most GET requests)
        self.rate_limit_wait(1).await;

        let base_url = BybitUrls::base_url(self.testnet);
        let path = endpoint.path();

        // Build query string
        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        // Add auth headers if needed
        let headers = if endpoint.is_private() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", &query)
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: BybitEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        // Wait for rate limit (weight 1 for most POST requests)
        self.rate_limit_wait(1).await;

        let base_url = BybitUrls::base_url(self.testnet);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        self.check_response(&response)?;
        Ok(response)
    }

    /// Check response for errors
    fn check_response(&self, response: &Value) -> ExchangeResult<()> {
        let ret_code = response.get("retCode")
            .and_then(|c| c.as_i64())
            .unwrap_or(-1);

        if ret_code != 0 {
            let msg = response.get("retMsg")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: ret_code as i32,
                message: msg.to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Bybit-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all tickers
    pub async fn get_all_tickers(&self, account_type: AccountType) -> ExchangeResult<Vec<Ticker>> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());

        let response = self.get(BybitEndpoint::Ticker, params).await?;
        // TODO: parse all tickers
        let _ = response;
        Ok(vec![])
    }

    /// Get symbols
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());

        self.get(BybitEndpoint::Symbols, params).await
    }

    /// Cancel all orders
    pub async fn cancel_all_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<String>> {
        let mut body = json!({
            "category": account_type_to_category(account_type),
        });

        if let Some(s) = symbol {
            body["symbol"] = json!(format_symbol(&s, account_type));
        }

        let response = self.post(BybitEndpoint::CancelAllOrders, body).await?;

        // Parse cancelled order IDs
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let ids = result.get("list")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.get("orderId").and_then(|id| id.as_str()).map(String::from))
                    .collect()
            })
            .unwrap_or_default();

        Ok(ids)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BybitConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bybit
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            (limiter.current_weight(), limiter.max_weight())
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
impl MarketData for BybitConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

        let response = self.get(BybitEndpoint::Ticker, params).await?;
        let ticker = BybitParser::parse_ticker(&response)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

        if let Some(d) = depth {
            params.insert("limit".to_string(), d.to_string());
        }

        let response = self.get(BybitEndpoint::Orderbook, params).await?;
        BybitParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("interval".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }

        if let Some(et) = end_time {
            params.insert("end".to_string(), et.to_string());
        }

        let response = self.get(BybitEndpoint::Klines, params).await?;
        BybitParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

        let response = self.get(BybitEndpoint::Ticker, params).await?;
        BybitParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(BybitEndpoint::ServerTime, HashMap::new()).await?;
        self.check_response(&response)
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols(account_type).await?;
        BybitParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BybitConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let order_link_id = format!("cc_{}", crate::core::timestamp_millis());
                
                        let body = json!({
                            "category": account_type_to_category(account_type),
                            "symbol": format_symbol(&symbol, account_type),
                            "side": match side {
                                OrderSide::Buy => "Buy",
                                OrderSide::Sell => "Sell",
                            },
                            "orderType": "Market",
                            "qty": quantity.to_string(),
                            "orderLinkId": order_link_id,
                        });
                
                        let response = self.post(BybitEndpoint::PlaceOrder, body).await?;
                
                        // Extract order ID from response
                        let result = response.get("result")
                            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;
                
                        let order_id = result.get("orderId")
                            .and_then(|id| id.as_str())
                            .ok_or_else(|| ExchangeError::Parse("Missing orderId".to_string()))?
                            .to_string();
                
                        // Return minimal order info (can fetch full info with get_order)
                        Ok(PlaceOrderResponse::Simple(Order {
                            id: order_id,
                            client_order_id: Some(order_link_id),
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
                let order_link_id = format!("cc_{}", crate::core::timestamp_millis());
                
                        let body = json!({
                            "category": account_type_to_category(account_type),
                            "symbol": format_symbol(&symbol, account_type),
                            "side": match side {
                                OrderSide::Buy => "Buy",
                                OrderSide::Sell => "Sell",
                            },
                            "orderType": "Limit",
                            "qty": quantity.to_string(),
                            "price": price.to_string(),
                            "orderLinkId": order_link_id,
                        });
                
                        let response = self.post(BybitEndpoint::PlaceOrder, body).await?;
                
                        let result = response.get("result")
                            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;
                
                        let order_id = result.get("orderId")
                            .and_then(|id| id.as_str())
                            .ok_or_else(|| ExchangeError::Parse("Missing orderId".to_string()))?
                            .to_string();
                
                        Ok(PlaceOrderResponse::Simple(Order {
                            id: order_id,
                            client_order_id: Some(order_link_id),
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

            let body = json!({
                "category": account_type_to_category(account_type),
                "symbol": format_symbol(&symbol, account_type),
                "orderId": order_id,
            });

            let response = self.post(BybitEndpoint::CancelOrder, body).await?;
            self.check_response(&response)?;

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

        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(BybitEndpoint::OrderStatus, params).await?;
        BybitParser::parse_order(&response)
    
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
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s, account_type));
        }

        let response = self.get(BybitEndpoint::OpenOrders, params).await?;

        // Parse all orders from result.list
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let list = result.get("list")
            .and_then(|l| l.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing list".to_string()))?;

        let mut orders = Vec::new();
        for order_data in list {
            // Create a wrapper to match parser expectations
            let wrapper = json!({
                "retCode": 0,
                "retMsg": "OK",
                "result": order_data,
            });

            if let Ok(order) = BybitParser::parse_order(&wrapper) {
                orders.push(order);
            }
        }

        Ok(orders)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BybitConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let mut params = HashMap::new();
        params.insert("accountType".to_string(), match account_type {
            AccountType::Spot | AccountType::Margin => "UNIFIED",
            AccountType::FuturesCross | AccountType::FuturesIsolated => "CONTRACT",
        }.to_string());

        let response = self.get(BybitEndpoint::Balance, params).await?;
        BybitParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(BybitEndpoint::AccountInfo, HashMap::new()).await?;

        // Get balances
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        // Parse account info from response
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let can_trade = result.get("unifiedMarginStatus")
            .and_then(|s| s.as_i64())
            .map(|s| s == 1)
            .unwrap_or(true);

        Ok(AccountInfo {
            account_type,
            can_trade,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1, // Default, should be fetched from API
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
impl Positions for BybitConnector {
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
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());

        if let Some(ref s) = symbol {
            params.insert("symbol".to_string(), format_symbol(s, account_type));
        }

        let response = self.get(BybitEndpoint::Positions, params).await?;

        // Parse positions from result.list
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let list = result.get("list")
            .and_then(|l| l.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing list".to_string()))?;

        let mut positions = Vec::new();
        for pos_data in list {
            let symbol_str = pos_data.get("symbol")
                .and_then(|s| s.as_str())
                .unwrap_or("");

            let quantity = pos_data.get("size")
                .and_then(|s| s.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            // Skip zero positions
            if quantity == 0.0 {
                continue;
            }

            let side = pos_data.get("side")
                .and_then(|s| s.as_str())
                .map(|s| match s {
                    "Buy" => crate::core::PositionSide::Long,
                    "Sell" => crate::core::PositionSide::Short,
                    _ => crate::core::PositionSide::Long,
                })
                .unwrap_or(crate::core::PositionSide::Long);

            let entry_price = pos_data.get("avgPrice")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let unrealized_pnl = pos_data.get("unrealisedPnl")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);

            let leverage = pos_data.get("leverage")
                .and_then(|l| l.as_str())
                .and_then(|s| s.parse::<u32>().ok())
                .unwrap_or(1);

            let liquidation_price = pos_data.get("liqPrice")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let mark_price = pos_data.get("markPrice")
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse::<f64>().ok());

            let margin_type = match account_type {
                AccountType::FuturesCross => crate::core::MarginType::Cross,
                AccountType::FuturesIsolated => crate::core::MarginType::Isolated,
                _ => crate::core::MarginType::Cross,
            };

            positions.push(Position {
                symbol: symbol_str.to_string(),
                side,
                quantity,
                entry_price,
                mark_price,
                unrealized_pnl,
                realized_pnl: None,
                liquidation_price,
                leverage,
                margin_type,
                margin: None,
                take_profit: None,
                stop_loss: None,
            });
        }

        Ok(positions)
    
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

        let mut params = HashMap::new();
        params.insert("category".to_string(), account_type_to_category(account_type).to_string());
        params.insert("symbol".to_string(), format_symbol(&symbol, account_type));

        let response = self.get(BybitEndpoint::FundingRate, params).await?;
        BybitParser::parse_funding_rate(&response)
    
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

                let body = json!({
                "category": account_type_to_category(account_type),
                "symbol": format_symbol(&symbol, account_type),
                "buyLeverage": leverage.to_string(),
                "sellLeverage": leverage.to_string(),
                });

                let response = self.post(BybitEndpoint::SetLeverage, body).await?;
                self.check_response(&response)?;

                Ok(())
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}
