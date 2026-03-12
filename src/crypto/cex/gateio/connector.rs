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
    Order, OrderSide, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
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



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════


