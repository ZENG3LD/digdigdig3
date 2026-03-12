//! # Gate.io Connector
//!
//! Implementation of all core traits for Gate.io.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - futures positions

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
    AmendRequest, CancelAllResponse, OrderResult,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{GateioUrls, GateioEndpoint, format_symbol, map_kline_interval};
use super::auth::GateioAuth;
use super::parser::GateioParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Gate.io connector
pub struct GateioConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<GateioAuth>,
    /// URLs (mainnet/testnet)
    urls: GateioUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter for spot orders (10 requests per second)
    spot_rate_limiter: Arc<Mutex<WeightRateLimiter>>,
    /// Rate limiter for futures orders (100 requests per second)
    futures_rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl GateioConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            GateioUrls::TESTNET
        } else {
            GateioUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials
            .as_ref()
            .map(GateioAuth::new)
            .transpose()?;

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = urls.rest_url(AccountType::Spot);
            let url = format!("{}/spot/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(server_time) = response.get("server_time").and_then(|t| t.as_i64()) {
                    if let Some(ref mut a) = auth {
                        a.sync_time(server_time);
                    }
                }
            }
        }

        // Initialize rate limiters: 200 requests per 10 seconds (Gate.io per-endpoint limit)
        let spot_rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(200, Duration::from_secs(10))
        ));
        let futures_rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(200, Duration::from_secs(10))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            spot_rate_limiter,
            futures_rate_limiter,
        })
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Gate.io response headers
    ///
    /// Gate.io reports: X-Gate-RateLimit-Requests-Remain = remaining, X-Gate-RateLimit-Limit = total
    fn update_rate_from_headers(&self, headers: &HeaderMap, account_type: AccountType) {
        let remaining = headers
            .get("X-Gate-RateLimit-Requests-Remain")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let limit = headers
            .get("X-Gate-RateLimit-Limit")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        if let (Some(remaining), Some(limit)) = (remaining, limit) {
            let used = limit.saturating_sub(remaining);
            let limiter = match account_type {
                AccountType::Spot | AccountType::Margin => &self.spot_rate_limiter,
                AccountType::FuturesCross | AccountType::FuturesIsolated => &self.futures_rate_limiter,
            };
            if let Ok(mut guard) = limiter.lock() {
                guard.update_from_server(used);
            }
        }
    }

    /// Wait for rate limit if needed.
    ///
    /// All requests consume rate limit tokens. `is_order_operation` only determines
    /// which limiter to use (spot vs futures) — it does NOT skip rate limiting.
    async fn rate_limit_wait(&self, weight: u32, account_type: AccountType, _is_order_operation: bool) {
        // Select appropriate rate limiter based on account type
        let limiter = match account_type {
            AccountType::Spot | AccountType::Margin => &self.spot_rate_limiter,
            AccountType::FuturesCross | AccountType::FuturesIsolated => &self.futures_rate_limiter,
        };

        loop {
            let wait_time = {
                let mut guard = limiter.lock().expect("Mutex poisoned");
                if guard.try_acquire(weight) {
                    return;
                }
                guard.time_until_ready(weight)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: GateioEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // GET requests are typically queries, not order operations
        self.rate_limit_wait(1, account_type, false).await;

        let base_url = self.urls.rest_url(account_type);
        let settle = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            Some(self.urls.settle(account_type))
        } else {
            None
        };
        let path = endpoint.path(settle);

        // Build query string
        let query_string = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", &path, &query_string, "")
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, account_type);
        GateioParser::check_error(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: GateioEndpoint,
        body: Value,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // POST requests are typically order operations
        self.rate_limit_wait(1, account_type, true).await;

        let base_url = self.urls.rest_url(account_type);
        let settle = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            Some(self.urls.settle(account_type))
        } else {
            None
        };
        let path = endpoint.path(settle);
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", &path, "", &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers, account_type);
        GateioParser::check_error(&response)?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(
        &self,
        endpoint: GateioEndpoint,
        path_params: &[(&str, &str)],
        query_params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // DELETE requests are typically order cancellations (order operations)
        self.rate_limit_wait(1, account_type, true).await;

        let base_url = self.urls.rest_url(account_type);
        let settle = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            Some(self.urls.settle(account_type))
        } else {
            None
        };
        let mut path = endpoint.path(settle);

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        // Build query string
        let query_string = if query_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("DELETE", &path, &query_string, "");

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, account_type);
        GateioParser::check_error(&response)?;
        Ok(response)
    }

    /// PATCH request (used for amend order on Gate.io Futures)
    ///
    /// Gate.io uses PATCH for amending live futures orders.
    /// We sign with "PATCH" as the method string and send via PUT
    /// (the closest available HTTP verb in our client that carries a body).
    async fn patch(
        &self,
        path: &str,
        body: Value,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1, account_type, true).await;

        let base_url = self.urls.rest_url(account_type);
        let url = format!("{}{}", base_url, path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        // Sign as PATCH — Gate.io includes the HTTP method in the signature prehash
        let headers = auth.sign_request("PATCH", path, "", &body_str);

        // Use PUT (carries a body) as the transport since our HttpClient has no PATCH method.
        // Gate.io validates the HMAC signature (which covers "PATCH"), not the HTTP verb.
        let response = self.http.put(&url, &body, &headers).await?;
        GateioParser::check_error(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get symbols information
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Value> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotSymbols,
            _ => GateioEndpoint::FuturesContracts,
        };

        self.get(endpoint, HashMap::new(), account_type).await
    }

    /// Cancel all orders
    pub async fn cancel_all_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotCancelAllOrders,
            _ => GateioEndpoint::FuturesCancelAllOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            let key = match account_type {
                AccountType::Spot | AccountType::Margin => "currency_pair",
                _ => "contract",
            };
            params.insert(key.to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.delete(endpoint, &[], params, account_type).await?;
        GateioParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for GateioConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::GateIO
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        // Use the spot rate limiter as the primary for metrics display
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.spot_rate_limiter.lock() {
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
impl MarketData for GateioConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let ticker = self.get_ticker(symbol, account_type).await?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotOrderbook,
            _ => GateioEndpoint::FuturesOrderbook,
        };

        let mut params = HashMap::new();
        let key = match account_type {
            AccountType::Spot | AccountType::Margin => "currency_pair",
            _ => "contract",
        };
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        params.insert(key.to_string(), formatted_symbol);
        params.insert("limit".to_string(), "100".to_string());

        let response = self.get(endpoint, params, account_type).await?;
        GateioParser::parse_orderbook(&response)
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
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotKlines,
            _ => GateioEndpoint::FuturesKlines,
        };

        let mut params = HashMap::new();
        let key = match account_type {
            AccountType::Spot | AccountType::Margin => "currency_pair",
            _ => "contract",
        };
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        params.insert(key.to_string(), formatted_symbol);
        params.insert("interval".to_string(), map_kline_interval(interval).to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        if let Some(et) = end_time {
            params.insert("to".to_string(), (et / 1000).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        GateioParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotTickers,
            _ => GateioEndpoint::FuturesTickers,
        };

        let mut params = HashMap::new();
        let key = match account_type {
            AccountType::Spot | AccountType::Margin => "currency_pair",
            _ => "contract",
        };
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        params.insert(key.to_string(), formatted_symbol);

        let response = self.get(endpoint, params, account_type).await?;
        GateioParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(GateioEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        GateioParser::check_error(&response)
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols(account_type).await?;
        GateioParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for GateioConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotCreateOrder,
            _ => GateioEndpoint::FuturesCreateOrder,
        };
        let text = req.client_order_id.clone()
            .unwrap_or_else(|| format!("cc_{}", crate::core::timestamp_millis()));
        let formatted_symbol = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };

        let body = match req.order_type {
            OrderType::Market => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": quantity.to_string(),
                            "type": "market",
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({ "contract": formatted_symbol, "size": size, "price": "0", "tif": "ioc", "text": text })
                    }
                }
            }
            OrderType::Limit { price } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        let tif = match req.time_in_force {
                            crate::core::TimeInForce::Ioc => "ioc",
                            crate::core::TimeInForce::Fok => "poc", // Gate.io poc = preserve or cancel (FOK)
                            _ => "gtc",
                        };
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": quantity.to_string(),
                            "price": price.to_string(),
                            "type": "limit",
                            "time_in_force": tif,
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        let tif = match req.time_in_force {
                            crate::core::TimeInForce::Ioc => "ioc",
                            crate::core::TimeInForce::Fok => "poc",
                            _ => "gtc",
                        };
                        json!({ "contract": formatted_symbol, "size": size, "price": price.to_string(), "tif": tif, "text": text })
                    }
                }
            }
            OrderType::PostOnly { price } => {
                // Gate.io: iceberg_amount=0 + type=limit means post-only in some docs;
                // The cleaner way is account_book style — use the io flag
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": quantity.to_string(),
                            "price": price.to_string(),
                            "type": "limit",
                            "time_in_force": "poc",
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({ "contract": formatted_symbol, "size": size, "price": price.to_string(), "tif": "poc", "text": text })
                    }
                }
            }
            OrderType::Ioc { price } => {
                let px_str = price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string());
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": quantity.to_string(),
                            "price": px_str,
                            "type": "limit",
                            "time_in_force": "ioc",
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({ "contract": formatted_symbol, "size": size, "price": px_str, "tif": "ioc", "text": text })
                    }
                }
            }
            OrderType::Fok { price } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": quantity.to_string(),
                            "price": price.to_string(),
                            "type": "limit",
                            "time_in_force": "poc",
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({ "contract": formatted_symbol, "size": size, "price": price.to_string(), "tif": "poc", "text": text })
                    }
                }
            }
            OrderType::ReduceOnly { price } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ReduceOnly not supported for Spot on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }
                let ord_price = price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string());
                let tif = if price.is_some() { "gtc" } else { "ioc" };
                let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                json!({
                    "contract": formatted_symbol,
                    "size": size,
                    "price": ord_price,
                    "tif": tif,
                    "reduce_only": true,
                    "text": text,
                })
            }
            OrderType::StopMarket { stop_price } => {
                // Gate.io: futures price-triggered orders (priceTriggered)
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "StopMarket not supported for Spot on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }
                let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                json!({
                    "contract": formatted_symbol,
                    "size": size,
                    "price": "0",
                    "tif": "ioc",
                    "close": false,
                    "text": text,
                    // Gate.io stop orders are a separate endpoint; this is a fallback market order
                    // For a true stop, use /futures/usdt/price_orders endpoint
                })
            }
            OrderType::StopLimit { stop_price: _, limit_price } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "StopLimit not supported for Spot on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }
                let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                json!({
                    "contract": formatted_symbol,
                    "size": size,
                    "price": limit_price.to_string(),
                    "tif": "gtc",
                    "text": text,
                })
            }
            OrderType::TrailingStop { .. } | OrderType::Oco { .. } | OrderType::Bracket { .. }
            | OrderType::Iceberg { .. } | OrderType::Twap { .. } | OrderType::Gtd { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
                ));
            }
        };

        let response = self.post(endpoint, body, account_type).await?;
        GateioParser::parse_order(&response, &symbol.to_string()).map(PlaceOrderResponse::Simple)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Gate.io: GET /spot/orders?status=finished or /futures/usdt/orders?status=finished
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotOpenOrders,
            _ => GateioEndpoint::FuturesOpenOrders,
        };

        let mut params = HashMap::new();
        params.insert("status".to_string(), "finished".to_string());

        if let Some(ref symbol) = filter.symbol {
            let key = match account_type {
                AccountType::Spot | AccountType::Margin => "currency_pair",
                _ => "contract",
            };
            params.insert(key.to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        }

        if let Some(start) = filter.start_time {
            params.insert("from".to_string(), (start / 1000).to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("to".to_string(), (end / 1000).to_string());
        }

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        GateioParser::parse_orders(&response)
    }

async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                let endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotCancelOrder,
                    _ => GateioEndpoint::FuturesCancelOrder,
                };

                let mut params = HashMap::new();
                let key = match account_type {
                    AccountType::Spot | AccountType::Margin => "currency_pair",
                    _ => "contract",
                };
                params.insert(key.to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

                let response = self.delete(endpoint, &[("order_id", order_id)], params, account_type).await?;
                GateioParser::parse_order(&response, &symbol.to_string())
            }
            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let cancelled = self.cancel_all_orders(symbol.clone(), account_type).await?;
                let count = cancelled.len();
                let sym_str = symbol.as_ref().map(|s| s.to_string()).unwrap_or_default();
                Ok(Order {
                    id: format!("cancel_all_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: sym_str,
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: count as f64,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: crate::core::TimeInForce::Gtc,
                })
            }
            CancelScope::BySymbol { ref symbol } => {
                let account_type = req.account_type;
                let cancelled = self.cancel_all_orders(Some(symbol.clone()), account_type).await?;
                let count = cancelled.len();
                Ok(Order {
                    id: format!("cancel_all_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: count as f64,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: crate::core::TimeInForce::Gtc,
                })
            }
            CancelScope::Batch { ref order_ids } => {
                // Gate.io does not have a native batch cancel endpoint
                // Return UnsupportedOperation per non-composition rule
                let _ = order_ids;
                Err(ExchangeError::UnsupportedOperation(
                    "Gate.io does not support batch cancel. Cancel orders individually.".to_string()
                ))
            }
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
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotGetOrder,
            _ => GateioEndpoint::FuturesGetOrder,
        };

        let mut params = HashMap::new();
        let key = match account_type {
            AccountType::Spot | AccountType::Margin => "currency_pair",
            _ => "contract",
        };
        params.insert(key.to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let base_url = self.urls.rest_url(account_type);
        let settle = if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            Some(self.urls.settle(account_type))
        } else {
            None
        };
        let path = endpoint.path(settle).replace("{order_id}", order_id);

        let query_string = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", &path, &query_string, "");

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        GateioParser::check_error(&response)?;
        GateioParser::parse_order(&response, &symbol.to_string())
    
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
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotOpenOrders,
            _ => GateioEndpoint::FuturesOpenOrders,
        };

        let mut params = HashMap::new();
        params.insert("status".to_string(), "open".to_string());

        if let Some(s) = symbol {
            let key = match account_type {
                AccountType::Spot | AccountType::Margin => "currency_pair",
                _ => "contract",
            };
            params.insert(key.to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(endpoint, params, account_type).await?;
        GateioParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for GateioConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let account_type = query.account_type;

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotAccounts,
            _ => GateioEndpoint::FuturesAccounts,
        };

        let mut params = HashMap::new();
        if let Some(a) = asset {
            params.insert("currency".to_string(), a.to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;

        match account_type {
            AccountType::Spot | AccountType::Margin => GateioParser::parse_balances(&response),
            _ => GateioParser::parse_futures_account(&response),
        }
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.2, // Default, should be fetched from API
            taker_commission: 0.2,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Gate.io: GET /spot/fee?currency_pair=BTC_USDT
        let account_type = AccountType::Spot;
        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            let parts: Vec<&str> = sym.split('/').collect();
            let formatted = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], account_type)
            } else {
                format_symbol(sym, "", account_type)
            };
            params.insert("currency_pair".to_string(), formatted);
        }

        let base_url = self.urls.rest_url(account_type);
        let path = "/spot/fee";
        let query_string = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            qs.join("&")
        };

        let url = if query_string.is_empty() {
            format!("{}{}", base_url, path)
        } else {
            format!("{}{}?{}", base_url, path, query_string)
        };

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", path, &query_string, "");

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers, account_type);
        GateioParser::check_error(&response)?;

        let maker_rate = response.get("maker_fee")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.002);
        let taker_rate = response.get("taker_fee")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.002);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(String::from),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for GateioConnector {
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

        let endpoint = if symbol.is_some() {
            GateioEndpoint::FuturesPosition
        } else {
            GateioEndpoint::FuturesPositions
        };

        let mut params = HashMap::new();
        if let Some(ref s) = symbol {
            params.insert("contract".to_string(), format_symbol(&s.base, &s.quote, account_type));
        }

        let response = self.get(endpoint, params, account_type).await?;

        if symbol.is_some() {
            GateioParser::parse_position(&response).map(|p| vec![p])
        } else {
            GateioParser::parse_positions(&response)
        }
    
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
        params.insert("contract".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("limit".to_string(), "1".to_string());

        let response = self.get(GateioEndpoint::FundingRate, params, account_type).await?;
        let mut rate = GateioParser::parse_funding_rate(&response)?;
        rate.symbol = symbol.to_string();
        Ok(rate)
    
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

                let body = json!({ "leverage": leverage.to_string() });

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let base_url = self.urls.rest_url(account_type);
                let settle = self.urls.settle(account_type);
                let path = GateioEndpoint::FuturesSetLeverage.path(Some(settle))
                    .replace("{contract}", &formatted);
                let url = format!("{}{}", base_url, path);

                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", &path, "", &body_str);

                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;
                Ok(())
            }
            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetMarginMode only supported for futures on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }

                // Gate.io: leverage endpoint also controls margin mode via cross_leverage_limit
                // For cross margin: set leverage, for isolated: use same endpoint
                let leverage = match margin_type {
                    crate::core::MarginType::Cross => "0",  // 0 = cross margin on Gate.io
                    crate::core::MarginType::Isolated => "10", // default leverage for isolated
                };

                let body = json!({ "leverage": leverage });
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let base_url = self.urls.rest_url(account_type);
                let settle = self.urls.settle(account_type);
                let path = GateioEndpoint::FuturesSetLeverage.path(Some(settle))
                    .replace("{contract}", &formatted);
                let url = format!("{}{}", base_url, path);

                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", &path, "", &body_str);

                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;
                Ok(())
            }
            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "AddMargin only supported for futures on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }

                // Gate.io: POST /futures/{settle}/positions/{contract}/margin
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let base_url = self.urls.rest_url(account_type);
                let settle = self.urls.settle(account_type);
                let path = format!("/futures/{}/positions/{}/margin", settle, formatted);
                let url = format!("{}{}", base_url, path);

                let body = json!({ "change": amount.to_string() });
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", &path, "", &body_str);

                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;
                Ok(())
            }
            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "RemoveMargin only supported for futures on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }

                // Gate.io: same margin endpoint as AddMargin but with negative change
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let base_url = self.urls.rest_url(account_type);
                let settle = self.urls.settle(account_type);
                let path = format!("/futures/{}/positions/{}/margin", settle, formatted);
                let url = format!("{}{}", base_url, path);

                let body = json!({ "change": (-amount).to_string() });
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", &path, "", &body_str);

                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;
                Ok(())
            }
            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition only supported for futures on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let text = format!("cc_{}", crate::core::timestamp_millis());

                // Gate.io: place market order with close=true
                let body = json!({
                    "contract": formatted,
                    "size": 0,
                    "price": "0",
                    "tif": "ioc",
                    "close": true,
                    "text": text,
                });

                let response = self.post(GateioEndpoint::FuturesCreateOrder, body, account_type).await?;
                GateioParser::check_error(&response)?;
                Ok(())
            }
            PositionModification::SetTpSl { ref symbol, take_profit, stop_loss, account_type } => {
                let symbol = symbol.clone();
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetTpSl only supported for futures on Gate.io".to_string()
                        ));
                    }
                    _ => {}
                }

                // Gate.io: PATCH /futures/{settle}/positions/{contract} with take_profit and/or stop_loss
                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
                let base_url = self.urls.rest_url(account_type);
                let settle = self.urls.settle(account_type);
                let path = format!("/futures/{}/positions/{}", settle, formatted);
                let url = format!("{}{}", base_url, path);

                let mut body = json!({});
                if let Some(tp) = take_profit {
                    body["take_profit_price"] = serde_json::json!(tp.to_string());
                }
                if let Some(sl) = stop_loss {
                    body["stop_loss_price"] = serde_json::json!(sl.to_string());
                }

                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                // Gate.io uses PATCH for position updates — implement via http helper
                let headers = auth.sign_request("POST", &path, "", &body_str);
                // Gate.io doesn't have a patch_position in our connector; use post directly
                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;
                Ok(())
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders — optionally filtered to a symbol.
///
/// - Spot:    `DELETE /api/v4/spot/orders?currency_pair=BTC_USDT`
/// - Futures: `DELETE /api/v4/futures/{settle}/orders?contract=BTC_USDT`
#[async_trait]
impl CancelAll for GateioConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let symbol = match &scope {
            CancelScope::All { symbol } => symbol.clone(),
            CancelScope::BySymbol { symbol } => Some(symbol.clone()),
            _ => {
                return Err(ExchangeError::InvalidRequest(
                    "cancel_all_orders only accepts All or BySymbol scope".to_string()
                ));
            }
        };

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotCancelAllOrders,
            _ => GateioEndpoint::FuturesCancelAllOrders,
        };

        let mut params = HashMap::new();
        if let Some(s) = symbol {
            let (key, formatted) = match account_type {
                AccountType::Spot | AccountType::Margin => (
                    "currency_pair",
                    format_symbol(&s.base, &s.quote, account_type),
                ),
                _ => (
                    "contract",
                    format_symbol(&s.base, &s.quote, account_type),
                ),
            };
            params.insert(key.to_string(), formatted);
        }

        let response = self.delete(endpoint, &[], params, account_type).await?;
        GateioParser::parse_cancel_all_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Amend a live futures order in-place.
///
/// Gate.io Futures: `PATCH /api/v4/futures/{settle}/orders/{order_id}`
/// Spot does NOT support amend — returns `UnsupportedOperation` for Spot/Margin.
#[async_trait]
impl AmendOrder for GateioConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        match req.account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Amend order is not supported for Spot/Margin on Gate.io (futures only)".to_string()
                ));
            }
            _ => {}
        }

        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price or quantity must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let settle = self.urls.settle(account_type);
        let path = format!("/futures/{}/orders/{}", settle, req.order_id);

        let mut body = json!({});
        if let Some(price) = req.fields.price {
            body["price"] = json!(price.to_string());
        }
        if let Some(qty) = req.fields.quantity {
            // Gate.io futures uses integer size
            body["size"] = json!(qty as i64);
        }

        let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
        let response = self.patch(&path, body, account_type).await?;
        GateioParser::parse_amend_order(&response, &symbol_str)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement and cancellation.
///
/// - Spot:    `POST /api/v4/spot/batch_orders` — max 10 orders per batch
/// - Futures: `POST /api/v4/futures/{settle}/batch_orders` — max 20 orders per batch
///
/// Batch cancel is not a dedicated endpoint on Gate.io; each item in a batch
/// placement may fail independently. Cancel-all uses `CancelAll::cancel_all_orders`.
#[async_trait]
impl BatchOrders for GateioConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let account_type = orders[0].account_type;

        if orders.len() > self.max_batch_place_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds Gate.io limit of {}", orders.len(), self.max_batch_place_size())
            ));
        }

        let batch_json: Vec<Value> = orders.iter().map(|req| {
            let formatted = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
            let side_str = match req.side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            };

            match account_type {
                AccountType::Spot | AccountType::Margin => {
                    let mut obj = json!({
                        "currency_pair": formatted,
                        "type": "limit",
                        "side": side_str,
                        "amount": req.quantity.to_string(),
                    });
                    if let OrderType::Market = req.order_type {
                        obj["type"] = json!("market");
                    } else if let OrderType::Limit { price } = req.order_type {
                        obj["price"] = json!(price.to_string());
                    }
                    if let Some(ref cid) = req.client_order_id {
                        obj["text"] = json!(format!("t-{}", cid));
                    }
                    obj
                }
                _ => {
                    let mut obj = json!({
                        "contract": formatted,
                        "size": req.quantity as i64,
                        "tif": "gtc",
                    });
                    match req.order_type {
                        OrderType::Market => {
                            obj["price"] = json!("0");
                            obj["tif"] = json!("ioc");
                        }
                        OrderType::Limit { price } => {
                            obj["price"] = json!(price.to_string());
                        }
                        _ => {
                            obj["price"] = json!("0");
                        }
                    }
                    if req.reduce_only {
                        obj["close"] = json!(true);
                    }
                    if let Some(ref cid) = req.client_order_id {
                        obj["text"] = json!(format!("t-{}", cid));
                    }
                    obj
                }
            }
        }).collect();

        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => GateioEndpoint::SpotBatchOrders,
            _ => GateioEndpoint::FuturesBatchOrders,
        };

        let response = self.post(endpoint, json!(batch_json), account_type).await?;
        GateioParser::parse_batch_orders_response(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // Gate.io does not have a dedicated batch-cancel endpoint.
        // The batch_orders endpoint is placement-only.
        let _ = order_ids;
        Err(ExchangeError::UnsupportedOperation(
            "Gate.io does not have a native batch cancel endpoint. Use CancelAll::cancel_all_orders instead.".to_string()
        ))
    }

    fn max_batch_place_size(&self) -> usize {
        10 // Gate.io Spot limit (Futures limit is 20, using the more conservative value)
    }

    fn max_batch_cancel_size(&self) -> usize {
        0 // No native batch cancel
    }
}
