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
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendRequest, CancelAllResponse, OrderResult,
    TransferResponse, DepositAddress, WithdrawResponse, FundsRecord,
    UserTrade, UserTradeFilter,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};
use crate::core::types::{
    TransferRequest, TransferHistoryFilter, WithdrawRequest,
    FundsHistoryFilter, FundsRecordType, SubAccountOperation, SubAccountResult,
    SubAccount, ConnectorStats,
    FundingPayment, FundingFilter, LedgerEntry, LedgerFilter,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::utils::WeightRateLimiter;
use crate::core::utils::precision::PrecisionCache;

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
    /// Per-symbol precision cache (populated after get_exchange_info)
    precision: PrecisionCache,
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
            precision: PrecisionCache::new(),
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
                AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => &self.spot_rate_limiter,
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
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => &self.spot_rate_limiter,
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
        let symbols = GateioParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&symbols);
        Ok(symbols)
    }

    fn market_data_capabilities(&self, _account_type: AccountType) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            // get_recent_trades is a struct method only — not exposed via MarketData trait
            has_recent_trades: false,
            // Native intervals: 10s, 1m, 5m, 15m, 30m, 1h, 4h, 8h, 1d, 7d (1w), 30d (1M)
            // 3m/2h/6h/12h map to nearest supported interval in map_kline_interval()
            supported_intervals: &[
                "10s", "1m", "5m", "15m", "30m",
                "1h", "4h", "8h",
                "1d", "1w", "1M",
            ],
            // get_klines caps limit at .min(1000)
            max_kline_limit: Some(1000),
        }
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
        let sym = &formatted_symbol;

        let body = match req.order_type {
            OrderType::Market => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": self.precision.qty(sym, quantity),
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
                            "amount": self.precision.qty(sym, quantity),
                            "price": self.precision.price(sym, price),
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
                        json!({ "contract": formatted_symbol, "size": size, "price": self.precision.price(sym, price), "tif": tif, "text": text })
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
                            "amount": self.precision.qty(sym, quantity),
                            "price": self.precision.price(sym, price),
                            "type": "limit",
                            "time_in_force": "poc",
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({ "contract": formatted_symbol, "size": size, "price": self.precision.price(sym, price), "tif": "poc", "text": text })
                    }
                }
            }
            OrderType::Ioc { price } => {
                let px_str = price.map(|p| self.precision.price(sym, p)).unwrap_or_else(|| "0".to_string());
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": self.precision.qty(sym, quantity),
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
                            "amount": self.precision.qty(sym, quantity),
                            "price": self.precision.price(sym, price),
                            "type": "limit",
                            "time_in_force": "poc",
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({ "contract": formatted_symbol, "size": size, "price": self.precision.price(sym, price), "tif": "poc", "text": text })
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
                let ord_price = price.map(|p| self.precision.price(sym, p)).unwrap_or_else(|| "0".to_string());
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
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        // Spot: POST /spot/price_orders with order_type: "market"
                        // Rule: trigger >= stop_price for buy, <= stop_price for sell
                        let trigger_rule = match side {
                            OrderSide::Buy => ">=",
                            OrderSide::Sell => "<=",
                        };
                        let body = json!({
                            "trigger": {
                                "price": self.precision.price(sym, stop_price),
                                "rule": trigger_rule,
                                "expiration": 86400,  // 24h expiration
                            },
                            "put": {
                                "type": "market",
                                "side": side_str,
                                "amount": self.precision.qty(sym, quantity),
                                "account": "spot",
                            },
                            "market": formatted_symbol,
                            "text": text,
                        });
                        let response = self.post(GateioEndpoint::SpotPriceOrders, body, account_type).await?;
                        return GateioParser::parse_order(&response, &symbol.to_string())
                            .map(PlaceOrderResponse::Simple);
                    }
                    _ => {
                        // Futures: POST /futures/{settle}/price_orders with market trigger
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        let trigger_rule = match side {
                            OrderSide::Buy => ">=",
                            OrderSide::Sell => "<=",
                        };
                        let body = json!({
                            "trigger": {
                                "strategy_type": 0,
                                "price_type": 0,
                                "price": self.precision.price(sym, stop_price),
                                "rule": 1,  // 1 = >= for buy, 2 = <= for sell
                                "expiration": 86400,
                            },
                            "initial": {
                                "contract": formatted_symbol,
                                "size": size,
                                "price": "0",
                                "tif": "ioc",
                                "text": text.clone(),
                            },
                        });
                        let _ = trigger_rule;
                        let base_url = self.urls.rest_url(account_type);
                        let settle = self.urls.settle(account_type);
                        let path = format!("/futures/{}/price_orders", settle);
                        let url = format!("{}{}", base_url, path);
                        let auth = self.auth.as_ref()
                            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                        let body_str = body.to_string();
                        let headers = auth.sign_request("POST", &path, "", &body_str);
                        let response = self.http.post(&url, &body, &headers).await?;
                        GateioParser::check_error(&response)?;
                        return GateioParser::parse_order(&response, &symbol.to_string())
                            .map(PlaceOrderResponse::Simple);
                    }
                }
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        // Spot: POST /spot/price_orders with order_type: "limit"
                        let trigger_rule = match side {
                            OrderSide::Buy => ">=",
                            OrderSide::Sell => "<=",
                        };
                        let body = json!({
                            "trigger": {
                                "price": self.precision.price(sym, stop_price),
                                "rule": trigger_rule,
                                "expiration": 86400,
                            },
                            "put": {
                                "type": "limit",
                                "side": side_str,
                                "amount": self.precision.qty(sym, quantity),
                                "price": self.precision.price(sym, limit_price),
                                "account": "spot",
                                "time_in_force": "gtc",
                            },
                            "market": formatted_symbol,
                            "text": text,
                        });
                        let response = self.post(GateioEndpoint::SpotPriceOrders, body, account_type).await?;
                        return GateioParser::parse_order(&response, &symbol.to_string())
                            .map(PlaceOrderResponse::Simple);
                    }
                    _ => {
                        // Futures: POST /futures/{settle}/price_orders with limit trigger
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        let body = json!({
                            "trigger": {
                                "strategy_type": 0,
                                "price_type": 0,
                                "price": self.precision.price(sym, stop_price),
                                "rule": 1,
                                "expiration": 86400,
                            },
                            "initial": {
                                "contract": formatted_symbol,
                                "size": size,
                                "price": self.precision.price(sym, limit_price),
                                "tif": "gtc",
                                "text": text.clone(),
                            },
                        });
                        let base_url = self.urls.rest_url(account_type);
                        let settle = self.urls.settle(account_type);
                        let path = format!("/futures/{}/price_orders", settle);
                        let url = format!("{}{}", base_url, path);
                        let auth = self.auth.as_ref()
                            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                        let body_str = body.to_string();
                        let headers = auth.sign_request("POST", &path, "", &body_str);
                        let response = self.http.post(&url, &body, &headers).await?;
                        GateioParser::check_error(&response)?;
                        return GateioParser::parse_order(&response, &symbol.to_string())
                            .map(PlaceOrderResponse::Simple);
                    }
                }
            }
            OrderType::Iceberg { price, display_quantity } => {
                // Gate.io Spot + Futures: set `iceberg` flag on regular limit order.
                // `iceberg` field = amount to show on orderbook (the visible slice size).
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        json!({
                            "currency_pair": formatted_symbol,
                            "side": side_str,
                            "amount": self.precision.qty(sym, quantity),
                            "price": self.precision.price(sym, price),
                            "type": "limit",
                            "time_in_force": "gtc",
                            "iceberg": self.precision.qty(sym, display_quantity),
                            "text": text,
                        })
                    }
                    _ => {
                        let size = match side { OrderSide::Buy => quantity as i64, OrderSide::Sell => -(quantity as i64) };
                        json!({
                            "contract": formatted_symbol,
                            "size": size,
                            "price": self.precision.price(sym, price),
                            "tif": "gtc",
                            "iceberg": display_quantity as i64,
                            "text": text,
                        })
                    }
                }
            }
            OrderType::TrailingStop { .. } | OrderType::Oco { .. } | OrderType::Bracket { .. }
            | OrderType::Twap { .. } | OrderType::Gtd { .. }
            | OrderType::Oto { .. } | OrderType::ConditionalPlan { .. } | OrderType::DcaRecurring { .. } => {
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
            CancelScope::ByLabel(_)
            | CancelScope::ByCurrencyKind { .. }
            | CancelScope::ScheduledAt(_) => Err(ExchangeError::UnsupportedOperation(
                "Gate.io does not support this cancel scope".to_string()
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

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let is_futures = matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated);

        let endpoint = if is_futures {
            GateioEndpoint::FuturesMyTrades
        } else {
            GateioEndpoint::SpotMyTrades
        };

        let mut params = HashMap::new();

        if let Some(ref sym) = filter.symbol {
            // Gate.io uses underscore-separated pairs (BTC_USDT)
            // filter.symbol may arrive as "BTC/USDT" or already "BTC_USDT"
            let formatted = if sym.contains('/') {
                sym.replace('/', "_").to_uppercase()
            } else {
                sym.to_uppercase()
            };
            let key = if is_futures { "contract" } else { "currency_pair" };
            params.insert(key.to_string(), formatted);
        }

        if let Some(ref oid) = filter.order_id {
            let key = if is_futures { "order" } else { "order_id" };
            params.insert(key.to_string(), oid.clone());
        }

        // Gate.io time params are in seconds; filter provides milliseconds
        if let Some(st) = filter.start_time {
            params.insert("from".to_string(), (st / 1000).to_string());
        }
        if let Some(et) = filter.end_time {
            params.insert("to".to_string(), (et / 1000).to_string());
        }

        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        let response = self.get(endpoint, params, account_type).await?;
        GateioParser::parse_user_trades(&response, is_futures)
    }

    fn trading_capabilities(&self, account_type: AccountType) -> TradingCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            // StopMarket: POST /spot/price_orders (spot) and /futures/{settle}/price_orders (futures)
            has_stop_market: true,
            // StopLimit: same price_orders endpoint with limit put order
            has_stop_limit: true,
            // TrailingStop: returns UnsupportedOperation in place_order match
            has_trailing_stop: false,
            // Bracket: not supported (UnsupportedOperation)
            has_bracket: false,
            // OCO: not supported (UnsupportedOperation)
            has_oco: false,
            // AmendOrder trait implemented: /spot/orders/{id} PATCH and /futures/{settle}/orders/{id}
            has_amend: true,
            // BatchOrders trait implemented: POST /spot/batch_orders and /futures/{settle}/batch_orders
            has_batch: true,
            // Spot: max 10 per batch; Futures: max 20 per batch (Gate.io API limits)
            max_batch_size: if is_futures { Some(20) } else { Some(10) },
            // CancelAll trait implemented: DELETE /spot/orders and /futures/{settle}/orders
            has_cancel_all: true,
            // get_user_trades: GET /spot/my_trades and /futures/{settle}/my_trades
            has_user_trades: true,
            // get_order_history: GET /spot/orders?status=finished and /futures/{settle}/orders?status=finished
            has_order_history: true,
        }
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

    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            // get_fees uses GET /spot/fee — only valid for spot pairs, not futures contracts
            has_fees: !is_futures,
            // AccountTransfers trait: POST /wallet/transfers — wallet-level, same for both
            has_transfers: true,
            // SubAccounts trait: /sub_accounts endpoints — wallet-level, same for both
            has_sub_accounts: true,
            // CustodialFunds trait: /wallet/deposit_address, /withdrawals — wallet-level, same for both
            has_deposit_withdraw: true,
            // No margin borrow/repay endpoints implemented
            has_margin: false,
            // No earn/staking endpoints implemented
            has_earn_staking: false,
            // FundingHistory trait: GET /futures/{settle}/funding_payments — futures-only
            has_funding_history: is_futures,
            // AccountLedger trait: GET /wallet/ledger (spot) or /futures/{settle}/account_book (futures)
            has_ledger: true,
            // No coin conversion endpoint implemented
            has_convert: false,
        }
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
            PositionModification::SwitchPositionMode { .. } => Err(ExchangeError::UnsupportedOperation(
                "SwitchPositionMode not supported on Gate.io".to_string()
            )),
            PositionModification::MovePositions { .. } => Err(ExchangeError::UnsupportedOperation(
                "MovePositions not supported on Gate.io".to_string()
            )),
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

/// Amend a live order in-place.
///
/// Gate.io Spot:    `PATCH /api/v4/spot/orders/{order_id}` (introduced v4.35.0)
/// Gate.io Futures: `PATCH /api/v4/futures/{settle}/orders/{order_id}`
#[async_trait]
impl AmendOrder for GateioConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price or quantity must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                // Spot amend: PATCH /spot/orders/{order_id}
                // Gate.io Spot v4.35.0: supports amend_text, price (amount for market)
                let path = format!("/spot/orders/{}", req.order_id);
                let mut body = json!({
                    "currency_pair": symbol_str,
                });
                if let Some(price) = req.fields.price {
                    body["price"] = json!(self.precision.price(&symbol_str, price));
                }
                if let Some(qty) = req.fields.quantity {
                    // Spot uses `amount` for quantity
                    body["amount"] = json!(self.precision.qty(&symbol_str, qty));
                }
                let response = self.patch(&path, body, account_type).await?;
                GateioParser::parse_amend_order(&response, &symbol_str)
            }
            _ => {
                // Futures amend: PATCH /futures/{settle}/orders/{order_id}
                let settle = self.urls.settle(account_type);
                let path = format!("/futures/{}/orders/{}", settle, req.order_id);

                let mut body = json!({});
                if let Some(price) = req.fields.price {
                    body["price"] = json!(self.precision.price(&symbol_str, price));
                }
                if let Some(qty) = req.fields.quantity {
                    // Gate.io futures uses integer size
                    body["size"] = json!(qty as i64);
                }

                let response = self.patch(&path, body, account_type).await?;
                GateioParser::parse_amend_order(&response, &symbol_str)
            }
        }
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

        let limit = if !matches!(account_type, AccountType::Spot | AccountType::Margin) { 20 } else { 10 };
        if orders.len() > limit {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds Gate.io {} limit of {}", orders.len(),
                    if limit == 20 { "Futures" } else { "Spot" }, limit)
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
                        "amount": self.precision.qty(&formatted, req.quantity),
                    });
                    if let OrderType::Market = req.order_type {
                        obj["type"] = json!("market");
                    } else if let OrderType::Limit { price } = req.order_type {
                        obj["price"] = json!(self.precision.price(&formatted, price));
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
                            obj["price"] = json!(self.precision.price(&formatted, price));
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
        // This method has no account_type; returns Spot limit as safe default.
        // Actual per-type limits are enforced in place_orders_batch (Spot=10, Futures=20)
        // and reported correctly via trading_capabilities(account_type).max_batch_size.
        10
    }

    fn max_batch_cancel_size(&self) -> usize {
        0 // No native batch cancel
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH AMEND
// ═══════════════════════════════════════════════════════════════════════════════

impl GateioConnector {
    /// Batch amend multiple futures orders via `POST /api/v4/futures/{settle}/batch_amend_orders`.
    ///
    /// Each entry in `amends` must be a JSON object with `order_id` and at least
    /// one of `price` or `size`.
    ///
    /// Max 20 orders per batch (Gate.io Futures limit).
    ///
    /// Returns the raw JSON response from Gate.io.
    pub async fn batch_amend_orders(
        &self,
        amends: Vec<serde_json::Value>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        if amends.is_empty() {
            return Ok(serde_json::Value::Array(vec![]));
        }
        if amends.len() > 20 {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch amend size {} exceeds Gate.io Futures limit of 20", amends.len())
            ));
        }

        self.post(GateioEndpoint::FuturesBatchAmend, json!(amends), account_type).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal transfers between Gate.io account types.
///
/// - Transfer: `POST /api/v4/wallet/transfers`
/// - History:  `GET  /api/v4/wallet/transfers`
///
/// AccountType mapping:
/// - `Spot`           → `"spot"`
/// - `FuturesCross`   → `"futures"`
/// - `Margin`         → `"margin"`
#[async_trait]
impl AccountTransfers for GateioConnector {
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        fn map_account(at: AccountType) -> &'static str {
            match at {
                AccountType::Spot => "spot",
                AccountType::FuturesCross | AccountType::FuturesIsolated => "futures",
                AccountType::Margin => "margin",
                AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => "spot",
            }
        }

        let account_type = AccountType::Spot; // transfers use spot base URL
        let base_url = self.urls.rest_url(account_type);
        let path = GateioEndpoint::WalletTransfer.path(None);
        let url = format!("{}{}", base_url, path);

        let body = json!({
            "currency": req.asset,
            "from": map_account(req.from_account),
            "to": map_account(req.to_account),
            "amount": req.amount.to_string(),
        });

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", &path, "", &body_str);

        let response = self.http.post(&url, &body, &headers).await?;
        GateioParser::check_error(&response)?;

        // Gate.io transfer response is typically empty on success; generate synthetic ID
        let transfer_id = format!("t_{}", crate::core::timestamp_millis());

        Ok(TransferResponse {
            transfer_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: Some(crate::core::timestamp_millis() as i64),
        })
    }

    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);
        let path = GateioEndpoint::WalletTransferHistory.path(None);

        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(start) = filter.start_time {
            params.insert("from".to_string(), (start / 1000).to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("to".to_string(), (end / 1000).to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

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

        let items = response.as_array().cloned().unwrap_or_default();
        let mut records = Vec::with_capacity(items.len());

        for item in items {
            let transfer_id = item.get("id")
                .and_then(|v| v.as_i64())
                .map(|v| v.to_string())
                .unwrap_or_default();
            let asset = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = item.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let status = "Successful".to_string();
            let timestamp = item.get("timestamp")
                .and_then(|v| v.as_i64())
                .map(|t| t * 1000); // Gate.io uses seconds, convert to ms
            records.push(TransferResponse {
                transfer_id,
                status,
                asset,
                amount,
                timestamp,
            });
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management for Gate.io.
///
/// - Deposit address: `GET  /api/v4/wallet/deposit_address`
/// - Withdraw:        `POST /api/v4/withdrawals`
/// - Deposit history: `GET  /api/v4/wallet/deposits`
/// - Withdrawal hist: `GET  /api/v4/wallet/withdrawals`
#[async_trait]
impl CustodialFunds for GateioConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);
        let path = GateioEndpoint::DepositAddress.path(None);

        let mut params = HashMap::new();
        params.insert("currency".to_string(), asset.to_string());
        if let Some(chain) = network {
            params.insert("chain".to_string(), chain.to_string());
        }

        let query_string: Vec<String> = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let qs = query_string.join("&");
        let url = format!("{}{}?{}", base_url, path, qs);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", &path, &qs, "");

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
        GateioParser::check_error(&response)?;

        let address = response.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing address field".to_string()))?
            .to_string();
        let tag = response.get("payment_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);
        let network_out = response.get("chain")
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(DepositAddress {
            address,
            tag,
            network: network_out,
            asset: asset.to_string(),
            created_at: None,
        })
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);
        let path = GateioEndpoint::Withdraw.path(None);
        let url = format!("{}{}", base_url, path);

        let mut body = json!({
            "currency": req.asset,
            "address": req.address,
            "amount": req.amount.to_string(),
        });
        if let Some(chain) = req.network {
            body["chain"] = json!(chain);
        }
        if let Some(memo) = req.tag {
            body["memo"] = json!(memo);
        }

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request("POST", &path, "", &body_str);

        let response = self.http.post(&url, &body, &headers).await?;
        GateioParser::check_error(&response)?;

        let withdraw_id = response.get("id")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let status = response.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("PENDING")
            .to_string();
        let tx_hash = response.get("txid")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(String::from);

        Ok(WithdrawResponse {
            withdraw_id,
            status,
            tx_hash,
        })
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);

        let endpoint = match filter.record_type {
            FundsRecordType::Deposit => GateioEndpoint::DepositHistory,
            FundsRecordType::Withdrawal => GateioEndpoint::WithdrawalHistory,
            FundsRecordType::Both => GateioEndpoint::DepositHistory,
        };

        let path = endpoint.path(None);

        let mut params: HashMap<String, String> = HashMap::new();
        if let Some(ref asset) = filter.asset {
            params.insert("currency".to_string(), asset.clone());
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

        let items = response.as_array().cloned().unwrap_or_default();
        let is_deposit = matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both);
        let mut records = Vec::with_capacity(items.len());

        for item in items {
            let id = item.get("id")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let asset = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            let amount = item.get("amount")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<f64>().ok())
                .unwrap_or(0.0);
            let status = item.get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("UNKNOWN")
                .to_string();
            let timestamp = item.get("timestamp")
                .and_then(|v| v.as_i64())
                .map(|t| t * 1000)  // Gate.io uses seconds
                .unwrap_or(0);
            let tx_hash = item.get("txid")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);
            let network = item.get("chain")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
                .map(String::from);

            if is_deposit {
                records.push(FundsRecord::Deposit {
                    id,
                    asset,
                    amount,
                    tx_hash,
                    network,
                    status,
                    timestamp,
                });
            } else {
                let fee = item.get("fee")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok());
                let address = item.get("withdraw_address")
                    .or_else(|| item.get("address"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tag = item.get("memo")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(String::from);
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
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB-ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Sub-account management for Gate.io.
///
/// - Create:   `POST /api/v4/sub_accounts`
/// - List:     `GET  /api/v4/sub_accounts`
/// - Transfer: `POST /api/v4/sub_accounts/transfers`
/// - Balance:  `GET  /api/v4/sub_accounts/{user_id}/balances`
#[async_trait]
impl SubAccounts for GateioConnector {
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        let account_type = AccountType::Spot;
        let base_url = self.urls.rest_url(account_type);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        match op {
            SubAccountOperation::Create { label } => {
                let path = GateioEndpoint::SubAccountCreate.path(None);
                let url = format!("{}{}", base_url, path);
                let body = json!({ "login_name": label });
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", &path, "", &body_str);
                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;

                let user_id = response.get("user_id")
                    .and_then(|v| v.as_i64())
                    .map(|v| v.to_string())
                    .unwrap_or_default();
                let name = response.get("login_name")
                    .and_then(|v| v.as_str())
                    .unwrap_or(&label)
                    .to_string();

                Ok(SubAccountResult {
                    id: Some(user_id),
                    name: Some(name),
                    accounts: vec![],
                    transaction_id: None,
                })
            }

            SubAccountOperation::List => {
                let path = GateioEndpoint::SubAccountList.path(None);
                let url = format!("{}{}", base_url, path);
                let headers = auth.sign_request("GET", &path, "", "");
                let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
                GateioParser::check_error(&response)?;

                let items = response.as_array().cloned().unwrap_or_default();
                let accounts: Vec<SubAccount> = items.iter().map(|item| {
                    SubAccount {
                        id: item.get("user_id")
                            .and_then(|v| v.as_i64())
                            .map(|v| v.to_string())
                            .unwrap_or_default(),
                        name: item.get("login_name")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        status: item.get("state")
                            .and_then(|v| v.as_i64())
                            .map(|s| if s == 1 { "Normal" } else { "Locked" })
                            .unwrap_or("Normal")
                            .to_string(),
                    }
                }).collect();

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                let path = GateioEndpoint::SubAccountTransfer.path(None);
                let url = format!("{}{}", base_url, path);
                let direction = if to_sub { "to" } else { "from" };
                let body = json!({
                    "currency": asset,
                    "sub_account": sub_account_id,
                    "direction": direction,
                    "amount": amount.to_string(),
                    "sub_account_type": "spot",
                });
                let body_str = body.to_string();
                let headers = auth.sign_request("POST", &path, "", &body_str);
                let response = self.http.post(&url, &body, &headers).await?;
                GateioParser::check_error(&response)?;

                // Gate.io returns empty on success; generate a synthetic transaction ID
                let tx_id = format!("sub_tx_{}", crate::core::timestamp_millis());

                Ok(SubAccountResult {
                    id: Some(sub_account_id),
                    name: None,
                    accounts: vec![],
                    transaction_id: Some(tx_id),
                })
            }

            SubAccountOperation::GetBalance { sub_account_id } => {
                let path = GateioEndpoint::SubAccountBalance.path(None)
                    .replace("{user_id}", &sub_account_id);
                let url = format!("{}{}", base_url, path);
                let headers = auth.sign_request("GET", &path, "", "");
                let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;
                GateioParser::check_error(&response)?;

                Ok(SubAccountResult {
                    id: Some(sub_account_id),
                    name: None,
                    accounts: vec![],
                    transaction_id: None,
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (not part of core traits)
// ═══════════════════════════════════════════════════════════════════════════════

impl GateioConnector {
    /// Get recent public spot trades.
    ///
    /// `GET /api/v4/spot/trades`
    ///
    /// # Parameters
    /// - `currency_pair`: Spot symbol e.g. `BTC_USDT`
    /// - `limit`: Max number of trades (optional, max 1000)
    pub async fn get_spot_trades(
        &self,
        currency_pair: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency_pair".to_string(), currency_pair.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(GateioEndpoint::SpotTrades, params, AccountType::Spot).await
    }

    /// Get personal spot trade history (requires auth).
    ///
    /// `GET /api/v4/spot/my_trades`
    ///
    /// # Parameters
    /// - `currency_pair`: Spot symbol e.g. `BTC_USDT`
    /// - `limit`: Max number of trades (optional)
    /// - `page`: Page number (optional)
    pub async fn get_spot_my_trades(
        &self,
        currency_pair: &str,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency_pair".to_string(), currency_pair.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        self.get(GateioEndpoint::SpotMyTrades, params, AccountType::Spot).await
    }

    /// Get recent public futures trades.
    ///
    /// `GET /api/v4/futures/usdt/trades`
    ///
    /// # Parameters
    /// - `contract`: Futures contract e.g. `BTC_USDT`
    /// - `limit`: Max number of trades (optional, max 1000)
    pub async fn get_futures_trades(
        &self,
        contract: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("contract".to_string(), contract.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(GateioEndpoint::FuturesTrades, params, AccountType::FuturesCross).await
    }

    /// Get personal futures trade history (requires auth).
    ///
    /// `GET /api/v4/futures/usdt/my_trades`
    ///
    /// # Parameters
    /// - `contract`: Futures contract e.g. `BTC_USDT` (optional)
    /// - `limit`: Max number of trades (optional)
    /// - `offset`: Offset for pagination (optional)
    pub async fn get_futures_my_trades(
        &self,
        contract: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(c) = contract {
            params.insert("contract".to_string(), c.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(o) = offset {
            params.insert("offset".to_string(), o.to_string());
        }
        self.get(GateioEndpoint::FuturesMyTrades, params, AccountType::FuturesCross).await
    }

    /// Get futures open interest statistics.
    ///
    /// `GET /api/v4/futures/usdt/contract_stats`
    ///
    /// # Parameters
    /// - `contract`: Futures contract e.g. `BTC_USDT`
    /// - `from`: Start timestamp in seconds (optional)
    /// - `interval`: Stat interval: `5m`, `15m`, `30m`, `1h`, `4h`, `1d` (optional)
    /// - `limit`: Number of entries (optional, max 100)
    pub async fn get_futures_open_interest(
        &self,
        contract: &str,
        from: Option<i64>,
        interval: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("contract".to_string(), contract.to_string());
        if let Some(f) = from {
            params.insert("from".to_string(), f.to_string());
        }
        if let Some(i) = interval {
            params.insert("interval".to_string(), i.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(GateioEndpoint::FuturesOpenInterest, params, AccountType::FuturesCross).await
    }

    /// Get futures funding rate history.
    ///
    /// `GET /api/v4/futures/usdt/funding_rate`
    ///
    /// # Parameters
    /// - `contract`: Futures contract e.g. `BTC_USDT`
    /// - `limit`: Number of entries (optional, max 1000)
    pub async fn get_futures_funding_rate_history(
        &self,
        contract: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("contract".to_string(), contract.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(GateioEndpoint::FuturesFundingRateHistory, params, AccountType::FuturesCross).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for GateioConnector {
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        let mut params = HashMap::new();
        if let Some(symbol) = &filter.symbol {
            params.insert("contract".to_string(), symbol.clone());
        }
        if let Some(start) = filter.start_time {
            // Gate.io expects seconds
            params.insert("from".to_string(), (start / 1000).to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("to".to_string(), (end / 1000).to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self
            .get(GateioEndpoint::FuturesFundingPayments, params, AccountType::FuturesCross)
            .await?;
        GateioParser::parse_funding_payments(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountLedger for GateioConnector {
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let mut params = HashMap::new();
        if let Some(asset) = &filter.asset {
            params.insert("currency".to_string(), asset.clone());
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

        let (endpoint, effective_account_type) = match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                (GateioEndpoint::FuturesAccountBook, account_type)
            }
            _ => (GateioEndpoint::WalletLedger, AccountType::Spot),
        };

        let response = self
            .get(endpoint, params, effective_account_type)
            .await?;
        GateioParser::parse_ledger(&response)
    }
}
