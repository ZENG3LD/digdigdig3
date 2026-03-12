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



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════


