//! # HTX Connector
//!
//! Implementation of all core traits for HTX API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//!
//! ## Extended Methods
//! Additional HTX-specific methods as struct methods.

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
    Position, FundingRate, Asset,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::{CancelAll, BatchOrders};
use crate::core::types::{ConnectorStats, CancelAllResponse, OrderResult};
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{HtxUrls, HtxEndpoint, format_symbol, map_kline_interval};
use super::auth::HtxAuth;
use super::parser::HtxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// HTX connector
pub struct HtxConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<HtxAuth>,
    /// Testnet mode (HTX doesn't have dedicated testnet)
    testnet: bool,
    /// Rate limiter (100 requests per second for trading)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
    /// Cached account ID for spot trading
    account_id: Arc<Mutex<Option<i64>>>,
}

impl HtxConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials.as_ref().map(HtxAuth::new);

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = HtxUrls::base_url(testnet);
            let url = format!("{}/v1/common/timestamp", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if response["status"] == "ok" {
                    if let Some(time_ms) = response["data"].as_i64() {
                        if let Some(ref mut a) = auth {
                            a.sync_time(time_ms);
                        }
                    }
                }
            }
        }

        // Initialize rate limiter: 100 requests per 10 seconds (HTX spot public limit)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(100, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            testnet,
            rate_limiter,
            account_id: Arc::new(Mutex::new(None)),
        })
    }

    /// Create connector only for public methods
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from HTX response headers
    ///
    /// HTX reports either spot headers (X-HB-RateLimit-*) or futures headers (ratelimit-*)
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        // Try spot headers first: X-HB-RateLimit-Requests-Remain / X-HB-RateLimit-Requests-Limit
        let remaining = headers
            .get("X-HB-RateLimit-Requests-Remain")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| {
                // Try futures headers: ratelimit-remaining
                headers
                    .get("ratelimit-remaining")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u32>().ok())
            });

        let limit = headers
            .get("X-HB-RateLimit-Requests-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| {
                headers
                    .get("ratelimit-limit")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|s| s.parse::<u32>().ok())
            });

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
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(weight) {
                    return;
                }
                limiter.time_until_ready(weight)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: HtxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        // Route to correct base URL based on endpoint
        let base_url = match endpoint {
            HtxEndpoint::FuturesTicker
            | HtxEndpoint::FuturesOrderbook
            | HtxEndpoint::FuturesKlines
            | HtxEndpoint::FuturesTrades => HtxUrls::futures_base_url(self.testnet),
            _ => HtxUrls::base_url(self.testnet),
        };
        let path = endpoint.path();

        // For private endpoints, build signed query string
        let query = if endpoint.is_private() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.build_signed_query("GET", "api.huobi.pro", path, &params)
        } else {
            // Public endpoints: simple query string
            if params.is_empty() {
                String::new()
            } else {
                let qs: Vec<String> = params.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                qs.join("&")
            }
        };

        let url = if query.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query)
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: HtxEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let base_url = HtxUrls::base_url(self.testnet);
        let path = endpoint.path();

        // HTX requires auth params in query string even for POST
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        // Empty params for signature (business params go in body)
        let query = auth.build_signed_query("POST", "api.huobi.pro", path, &HashMap::new());

        let url = format!("{}{}?{}", base_url, path, query);

        // Add Content-Type header
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// Get account ID for spot trading
    ///
    /// HTX requires account-id for most trading operations.
    /// This method caches the ID after first call.
    async fn get_account_id(&self) -> ExchangeResult<i64> {
        // Check cache first
        {
            let cached = self.account_id.lock().expect("Mutex poisoned");
            if let Some(id) = *cached {
                return Ok(id);
            }
        }

        // Fetch account list
        let response = self.get(HtxEndpoint::AccountList, HashMap::new()).await?;
        let accounts = HtxParser::parse_account_list(&response)?;

        // Find spot account
        let spot_account = accounts.iter()
            .find(|(_, account_type)| account_type == "spot")
            .ok_or_else(|| ExchangeError::Parse("No spot account found".to_string()))?;

        let id = spot_account.0;

        // Cache it
        {
            let mut cached = self.account_id.lock().expect("Mutex poisoned");
            *cached = Some(id);
        }

        Ok(id)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (HTX-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all symbols (raw Value)
    ///
    /// Uses the V1 endpoint (`/v1/common/symbols`) which returns the standard
    /// `{"status": "ok", "data": [...]}` envelope that `HtxParser::extract_result_v1`
    /// expects.
    pub async fn get_symbols(&self) -> ExchangeResult<Value> {
        self.get(HtxEndpoint::SymbolsV1, HashMap::new()).await
    }

    /// Get exchange info (parsed symbol list)
    pub async fn get_exchange_info_parsed(&self) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols().await?;
        HtxParser::parse_exchange_info(&response)
    }

    /// Cancel all orders (struct method — also available via CancelAll trait)
    pub async fn cancel_all_orders(&self, symbol: Option<Symbol>) -> ExchangeResult<Vec<String>> {
        let account_id = self.get_account_id().await?;

        let mut params = HashMap::new();
        params.insert("account-id".to_string(), account_id.to_string());

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s, AccountType::Spot));
        }

        // Get all open orders first
        let response = self.get(HtxEndpoint::OpenOrders, params.clone()).await?;

        let data = HtxParser::extract_result_v1(&response)?;
        let order_ids: Vec<String> = data.as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v["id"].as_i64().map(|id| id.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        // Cancel in batch
        if !order_ids.is_empty() {
            let body = json!({
                "order-ids": order_ids,
            });

            let _ = self.post(HtxEndpoint::CancelAllOrders, body).await?;
        }

        Ok(order_ids)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for HtxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::HTX
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
impl MarketData for HtxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();

        // Route to correct endpoint based on account type
        let (endpoint, param_name) = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                (HtxEndpoint::FuturesTicker, "contract_code")
            }
            _ => (HtxEndpoint::Ticker, "symbol"),
        };

        params.insert(param_name.to_string(), format_symbol(&symbol, account_type));

        let response = self.get(endpoint, params).await?;
        let ticker = HtxParser::parse_ticker(&response, &symbol.to_string())?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();

        // Route to correct endpoint based on account type
        let (endpoint, param_name) = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                (HtxEndpoint::FuturesOrderbook, "contract_code")
            }
            _ => (HtxEndpoint::Orderbook, "symbol"),
        };

        params.insert(param_name.to_string(), format_symbol(&symbol, account_type));
        params.insert("type".to_string(), "step0".to_string()); // step0 = best precision

        if let Some(d) = depth {
            // HTX supports depth 5, 10, 20
            let depth_str = match d {
                1..=5 => "5",
                6..=10 => "10",
                _ => "20",
            };
            params.insert("depth".to_string(), depth_str.to_string());
        }

        let response = self.get(endpoint, params).await?;
        HtxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();

        // Route to correct endpoint based on account type
        let (endpoint, param_name) = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                (HtxEndpoint::FuturesKlines, "contract_code")
            }
            _ => (HtxEndpoint::Klines, "symbol"),
        };

        params.insert(param_name.to_string(), format_symbol(&symbol, account_type));
        params.insert("period".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("size".to_string(), l.min(2000).to_string());
        }

        let response = self.get(endpoint, params).await?;
        HtxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();

        // Route to correct endpoint based on account type
        let (endpoint, param_name) = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                (HtxEndpoint::FuturesTicker, "contract_code")
            }
            _ => (HtxEndpoint::Ticker, "symbol"),
        };

        params.insert(param_name.to_string(), format_symbol(&symbol, account_type));

        let response = self.get(endpoint, params).await?;
        HtxParser::parse_ticker(&response, &symbol.to_string())
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(HtxEndpoint::ServerTime, HashMap::new()).await?;

        if response["status"] == "ok" {
            Ok(())
        } else {
            Err(ExchangeError::Api {
                code: 0,
                message: "Ping failed".to_string(),
            })
        }
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols().await?;
        HtxParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for HtxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let account_id = self.get_account_id().await?;
        let client_order_id = format!("cc_{}", crate::core::timestamp_millis());
        let htx_symbol = format_symbol(&symbol, account_type);

        // Helper to map side to HTX order type prefix
        let side_str = match side {
            OrderSide::Buy => "buy",
            OrderSide::Sell => "sell",
        };

        match req.order_type {
            OrderType::Market => {
                let order_type = format!("{}-market", side_str);
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                    "type": order_type,
                    "amount": quantity.to_string(),
                    "client-order-id": client_order_id,
                });

                let response = self.post(HtxEndpoint::PlaceOrder, body).await?;
                let order = HtxParser::parse_order(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order.id,
                    client_order_id: Some(client_order_id),
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
                let order_type = format!("{}-limit", side_str);
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                    "type": order_type,
                    "amount": quantity.to_string(),
                    "price": price.to_string(),
                    "client-order-id": client_order_id,
                });

                let response = self.post(HtxEndpoint::PlaceOrder, body).await?;
                let order = HtxParser::parse_order(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order.id,
                    client_order_id: Some(client_order_id),
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

            OrderType::StopLimit { stop_price, limit_price } => {
                // HTX: buy-stop-limit / sell-stop-limit
                // Requires: stop-price, operator (gte/lte), price
                let order_type = format!("{}-stop-limit", side_str);
                // For buy stop: trigger when price >= stop_price (gte)
                // For sell stop: trigger when price <= stop_price (lte)
                let operator = match side {
                    OrderSide::Buy => "gte",
                    OrderSide::Sell => "lte",
                };
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                    "type": order_type,
                    "amount": quantity.to_string(),
                    "stop-price": stop_price.to_string(),
                    "price": limit_price.to_string(),
                    "operator": operator,
                    "client-order-id": client_order_id,
                });

                let response = self.post(HtxEndpoint::PlaceOrder, body).await?;
                let order = HtxParser::parse_order(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order.id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::StopLimit { stop_price, limit_price },
                    status: crate::core::OrderStatus::New,
                    price: Some(limit_price),
                    stop_price: Some(stop_price),
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

            OrderType::PostOnly { price } => {
                // HTX: buy-limit-maker / sell-limit-maker (post-only)
                let order_type = format!("{}-limit-maker", side_str);
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                    "type": order_type,
                    "amount": quantity.to_string(),
                    "price": price.to_string(),
                    "client-order-id": client_order_id,
                });

                let response = self.post(HtxEndpoint::PlaceOrder, body).await?;
                let order = HtxParser::parse_order(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order.id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::PostOnly { price },
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

            OrderType::Ioc { price } => {
                // HTX: buy-ioc / sell-ioc
                let price_val = price.unwrap_or(0.0);
                let order_type = format!("{}-ioc", side_str);
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                    "type": order_type,
                    "amount": quantity.to_string(),
                    "price": price_val.to_string(),
                    "client-order-id": client_order_id,
                });

                let response = self.post(HtxEndpoint::PlaceOrder, body).await?;
                let order = HtxParser::parse_order(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order.id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Ioc { price },
                    status: crate::core::OrderStatus::New,
                    price,
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Ioc,
                }))
            }

            OrderType::Fok { price } => {
                // HTX: buy-limit-fok / sell-limit-fok
                let order_type = format!("{}-limit-fok", side_str);
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                    "type": order_type,
                    "amount": quantity.to_string(),
                    "price": price.to_string(),
                    "client-order-id": client_order_id,
                });

                let response = self.post(HtxEndpoint::PlaceOrder, body).await?;
                let order = HtxParser::parse_order(&response)?;

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order.id,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::Fok { price },
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
                    time_in_force: crate::core::TimeInForce::Fok,
                }))
            }

            // Trailing stop via POST /v2/algo-orders (API-only feature).
            // orderType: "trailing-stop-order"
            // trailingRate: callback rate (0 < rate <= 5%)
            // activationPrice: optional price at which trailing begins
            OrderType::TrailingStop { callback_rate, activation_price } => {
                let body = json!({
                    "accountId": account_id.to_string(),
                    "symbol": htx_symbol,
                    "orderSide": side_str,
                    "orderSize": quantity.to_string(),
                    "orderType": "trailing-stop-order",
                    // trailingRate must be > 0 and <= 5 (as percentage string)
                    "trailingRate": format!("{:.4}", callback_rate.clamp(0.0001, 5.0)),
                    // activationPrice is optional
                    "activationPrice": activation_price
                        .map(|p| p.to_string())
                        .unwrap_or_default(),
                });

                let response = self.post(HtxEndpoint::AlgoOrders, body).await?;

                // Algo orders return: { "code": 200, "data": { "clientOrderId": "...", "orderId": "..." } }
                let order_id_str = response
                    .pointer("/data/orderId")
                    .or_else(|| response.pointer("/data/clientOrderId"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id_str,
                    client_order_id: Some(client_order_id),
                    symbol: symbol.to_string(),
                    side,
                    order_type: OrderType::TrailingStop { callback_rate, activation_price },
                    status: crate::core::OrderStatus::New,
                    price: activation_price,
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

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                // HTX uses path variable for order ID
                let path = HtxEndpoint::CancelOrder.path_with_vars(&[("order-id", order_id)]);

                let base_url = HtxUrls::base_url(self.testnet);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let query = auth.build_signed_query("POST", "api.huobi.pro", &path, &HashMap::new());

                let url = format!("{}{}?{}", base_url, path, query);

                let body = json!({});
                let mut headers = HashMap::new();
                headers.insert("Content-Type".to_string(), "application/json".to_string());

                self.rate_limit_wait(1).await;
                let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
                self.update_rate_from_headers(&resp_headers);

                HtxParser::parse_order(&response)
            }

            CancelScope::Batch { ref order_ids } => {
                // HTX: POST /v1/order/orders/batchcancel with {"order-ids": [...]}
                // Max 50 IDs per request
                let body = json!({
                    "order-ids": order_ids,
                });

                let _response = self.post(HtxEndpoint::CancelAllOrders, body).await?;

                // Return placeholder for first order
                Ok(Order {
                    id: order_ids.first().cloned().unwrap_or_default(),
                    client_order_id: None,
                    symbol: req.symbol.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
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
                format!("{:?} cancel scope not supported — use CancelAll trait", req.scope)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();

        if let Some(sym) = &filter.symbol {
            // sym is already a Symbol struct
            params.insert("symbol".to_string(), format_symbol(sym, account_type));
        }

        // HTX requires states filter
        params.insert("states".to_string(), "filled,canceled".to_string());

        if let Some(start) = filter.start_time {
            params.insert("start-time".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end-time".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("size".to_string(), limit.min(100).to_string());
        }

        let response = self.get(HtxEndpoint::OrderHistory, params).await?;

        let data = HtxParser::extract_result_v1(&response)?;
        let orders = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Data is not an array".into()))?
            .iter()
            .filter_map(|order_json| {
                let wrapped = json!({"status": "ok", "data": order_json});
                HtxParser::parse_order(&wrapped).ok()
            })
            .collect();

        Ok(orders)
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let path = HtxEndpoint::OrderStatus.path_with_vars(&[("order-id", order_id)]);

        let base_url = HtxUrls::base_url(self.testnet);
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let query = auth.build_signed_query("GET", "api.huobi.pro", &path, &HashMap::new());

        let url = format!("{}{}?{}", base_url, path, query);

        self.rate_limit_wait(1).await;
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.update_rate_from_headers(&resp_headers);

        HtxParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let symbol: Option<crate::core::Symbol> = symbol.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let account_id = self.get_account_id().await?;

        let mut params = HashMap::new();
        params.insert("account-id".to_string(), account_id.to_string());

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), format_symbol(&s, account_type));
        }

        let response = self.get(HtxEndpoint::OpenOrders, params).await?;

        let data = HtxParser::extract_result_v1(&response)?;
        let orders = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Data is not an array".into()))?
            .iter()
            .filter_map(|order_json| {
                let wrapped = json!({"status": "ok", "data": order_json});
                HtxParser::parse_order(&wrapped).ok()
            })
            .collect();

        Ok(orders)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for HtxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;
        let account_id = self.get_account_id().await?;

        // Replace path variable
        let path = HtxEndpoint::Balance.path_with_vars(&[("account-id", &account_id.to_string())]);

        let base_url = HtxUrls::base_url(self.testnet);
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let query = auth.build_signed_query("GET", "api.huobi.pro", &path, &HashMap::new());

        let url = format!("{}{}?{}", base_url, path, query);

        self.rate_limit_wait(1).await;
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.update_rate_from_headers(&resp_headers);

        HtxParser::parse_balance(&response)
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type: _account_type }).await?;

        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.002, // 0.2% default
            taker_commission: 0.002,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // HTX: GET /v2/reference/transact-fee-rate?symbols=btcusdt
        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            let symbol_parts: Vec<&str> = sym.split('/').collect();
            let htx_symbol = if symbol_parts.len() == 2 {
                let s = crate::core::Symbol::new(symbol_parts[0], symbol_parts[1]);
                format_symbol(&s, AccountType::Spot)
            } else {
                sym.to_lowercase().replace('/', "")
            };
            params.insert("symbols".to_string(), htx_symbol);
        }

        let response = self.get(HtxEndpoint::TransactFee, params).await?;

        // HTX v2 response format: {"code": 200, "data": [{"symbol": "btcusdt", "makerFeeRate": "0.002", "takerFeeRate": "0.002"}]}
        let data = response.get("data")
            .and_then(|d| d.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("No fee data".to_string()))?;

        let maker_rate = data.get("makerFeeRate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.002);

        let taker_rate = data.get("takerFeeRate")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.002);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS (Spot has no positions)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for HtxConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        // Spot trading has no positions
        Ok(vec![])
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        Err(ExchangeError::NotSupported("Funding rate not available for spot trading".to_string()))
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { .. } => {
                Err(ExchangeError::NotSupported("Leverage not available for spot trading".to_string()))
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported on {:?}", req, self.exchange_id())
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for HtxConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let account_id = self.get_account_id().await?;

        // HTX batchCancelOpenOrders: POST /v1/order/orders/batchCancelOpenOrders
        // Optional fields: account-id, symbol, side
        // Without symbol: cancels ALL open orders across all pairs
        match scope {
            CancelScope::All { symbol: None } => {
                // Cancel all open orders — no symbol filter
                let body = json!({
                    "account-id": account_id.to_string(),
                });

                let response = self.post(HtxEndpoint::CancelOpenOrders, body).await?;

                // HTX returns {"status": "ok", "data": {"success-count": N, "failed-count": M, "next-id": -1}}
                let data = HtxParser::extract_result_v1(&response)?;
                let cancelled_count = data.get("success-count")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32)
                    .unwrap_or(0);
                let failed_count = data.get("failed-count")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32)
                    .unwrap_or(0);

                Ok(CancelAllResponse {
                    cancelled_count,
                    failed_count,
                    details: vec![],
                })
            }

            CancelScope::All { symbol: Some(sym) } | CancelScope::BySymbol { symbol: sym } => {
                // Cancel all open orders for a specific symbol
                let htx_symbol = format_symbol(&sym, account_type);
                let body = json!({
                    "account-id": account_id.to_string(),
                    "symbol": htx_symbol,
                });

                let response = self.post(HtxEndpoint::CancelOpenOrders, body).await?;

                let data = HtxParser::extract_result_v1(&response)?;
                let cancelled_count = data.get("success-count")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32)
                    .unwrap_or(0);
                let failed_count = data.get("failed-count")
                    .and_then(|v| v.as_u64())
                    .map(|n| n as u32)
                    .unwrap_or(0);

                Ok(CancelAllResponse {
                    cancelled_count,
                    failed_count,
                    details: vec![],
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported in cancel_all_orders", scope)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for HtxConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<crate::core::OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // HTX doesn't have a true batch place endpoint for spot
        // Return UnsupportedOperation
        Err(ExchangeError::UnsupportedOperation(
            "Batch order placement not available on HTX spot".to_string()
        ))
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // HTX: POST /v1/order/orders/batchcancel with {"order-ids": [...]}
        // Max 50 IDs per call
        let chunks: Vec<Vec<String>> = order_ids.chunks(50)
            .map(|chunk| chunk.to_vec())
            .collect();

        let mut results = Vec::new();

        for chunk in chunks {
            let body = json!({ "order-ids": chunk });
            match self.post(HtxEndpoint::CancelAllOrders, body).await {
                Ok(response) => {
                    // Check for success/failed in response
                    let data = HtxParser::extract_result_v1(&response)?;
                    if let Some(success_arr) = data.get("success").and_then(|v| v.as_array()) {
                        for _id_val in success_arr {
                            results.push(OrderResult {
                                order: None,
                                client_order_id: None,
                                success: true,
                                error: None,
                                error_code: None,
                            });
                        }
                    }
                    if let Some(failed_arr) = data.get("failed").and_then(|v| v.as_array()) {
                        for item in failed_arr {
                            let err_msg = item.get("err-msg")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Cancel failed")
                                .to_string();
                            results.push(OrderResult {
                                order: None,
                                client_order_id: None,
                                success: false,
                                error: Some(err_msg),
                                error_code: None,
                            });
                        }
                    }
                }
                Err(e) => {
                    for _ in &chunk {
                        results.push(OrderResult {
                            order: None,
                            client_order_id: None,
                            success: false,
                            error: Some(e.to_string()),
                            error_code: None,
                        });
                    }
                }
            }
        }

        Ok(results)
    }

    fn max_batch_place_size(&self) -> usize {
        0 // Not supported
    }

    fn max_batch_cancel_size(&self) -> usize {
        50
    }
}
