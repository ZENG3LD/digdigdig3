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
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
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
    /// expects. The V2 endpoint (`/v2/settings/common/symbols`) returns a different
    /// envelope (`{"code": 200, "data": ...}`) with a nested data structure and would
    /// require `extract_result_v2`, but the V1 endpoint has identical symbol coverage
    /// and simpler hyphenated field names that the parser already handles correctly.
    pub async fn get_symbols(&self) -> ExchangeResult<Value> {
        self.get(HtxEndpoint::SymbolsV1, HashMap::new()).await
    }

    /// Get exchange info (parsed symbol list)
    pub async fn get_exchange_info_parsed(&self) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get_symbols().await?;
        HtxParser::parse_exchange_info(&response)
    }

    /// Cancel all orders
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



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS (Spot has no positions)
// ═══════════════════════════════════════════════════════════════════════════════


