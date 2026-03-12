//! # Phemex Connector
//!
//! Реализация всех core трейтов для Phemex.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции
//!
//! ## CRITICAL: Value Scaling
//! Phemex uses integer representation with scale factors.
//! Always fetch /public/products first to get scale factors!

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol, Asset,
    ExchangeError, ExchangeResult,
    Price, Quantity, Kline, Ticker, OrderBook,
    Order, OrderSide, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::GroupRateLimiter;

use super::endpoints::{PhemexUrls, PhemexEndpoint, format_symbol, map_kline_interval, scale_price, scale_value};
use super::auth::PhemexAuth;
use super::parser::PhemexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Phemex коннектор
pub struct PhemexConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<PhemexAuth>,
    /// URL'ы (mainnet/testnet)
    urls: PhemexUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter with per-group pools (OTHERS/CONTRACT/SPOTORDER)
    rate_limiter: Arc<Mutex<GroupRateLimiter>>,
    /// Default price scale (used when scale is not provided)
    default_price_scale: u8,
    /// Default value scale
    default_value_scale: u8,
}

impl PhemexConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            PhemexUrls::TESTNET
        } else {
            PhemexUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(PhemexAuth::new)
            .transpose()?;

        // Initialize grouped rate limiter per Phemex API docs
        let mut group_limiter = GroupRateLimiter::new();
        group_limiter.add_group("OTHERS", 100, Duration::from_secs(60));
        group_limiter.add_group("CONTRACT", 500, Duration::from_secs(60));
        group_limiter.add_group("SPOTORDER", 500, Duration::from_secs(60));
        let rate_limiter = Arc::new(Mutex::new(group_limiter));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            default_price_scale: 4, // BTCUSD default
            default_value_scale: 8, // BTC default
        })
    }

    /// Создать коннектор только для публичных методов
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    /// Get price scale for symbol based on account type
    /// Spot symbols use priceScale=8, Contract symbols use priceScale=4
    fn get_price_scale(&self, account_type: AccountType) -> u8 {
        match account_type {
            AccountType::Spot => 8,
            _ => 4,
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Phemex response headers
    ///
    /// Phemex reports: x-ratelimit-remaining-<GROUP> = remaining (e.g. x-ratelimit-remaining-others)
    /// Parse the group name from the header and update the matching group.
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let prefix = "x-ratelimit-remaining-";
        for (name, value) in headers.iter() {
            let header_name = name.as_str().to_lowercase();
            if let Some(group_lower) = header_name.strip_prefix(prefix) {
                // Map header suffix to group name
                let group = match group_lower {
                    "contract" => "CONTRACT",
                    "spotorder" => "SPOTORDER",
                    _ => "OTHERS",
                };
                if let Some(remaining) = value.to_str().ok().and_then(|s| s.parse::<u32>().ok()) {
                    // Determine max for this group to compute used = max - remaining
                    let max = if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.group_stats(group).map(|(_, m)| m).unwrap_or(100)
                    } else {
                        continue;
                    };
                    let used = max.saturating_sub(remaining);
                    if let Ok(mut limiter) = self.rate_limiter.lock() {
                        limiter.update_from_server(group, used);
                    }
                }
            }
        }
    }

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self, group: &str, weight: u32) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(group, weight) {
                    return;
                }
                limiter.time_until_ready(group, weight)
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET запрос
    async fn get(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.get_weighted(endpoint, params, _account_type, 1).await
    }

    /// GET запрос with explicit weight (for endpoints that cost more, e.g. klines = 10)
    async fn get_weighted(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
        weight: u32,
    ) -> ExchangeResult<Value> {
        // Route to group based on whether endpoint is authenticated (order mgmt) or public (market data)
        let group = if endpoint.requires_auth() { "CONTRACT" } else { "OTHERS" };
        self.rate_limit_wait(group, weight).await;

        let base_url = self.urls.rest_url(_account_type);
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
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request(path, &query, "")
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: PhemexEndpoint,
        body: Value,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // POST is always order/account — use SPOTORDER for Spot, CONTRACT otherwise
        let group = match _account_type {
            AccountType::Spot => "SPOTORDER",
            _ => "CONTRACT",
        };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request(path, "", &body_str);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// PUT запрос
    async fn put(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        body: Value,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // PUT is always for order/position management
        let group = match _account_type {
            AccountType::Spot => "SPOTORDER",
            _ => "CONTRACT",
        };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();

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

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = auth.sign_request(path, &query, &body_str);

        self.http.put(&url, &body, &headers).await
    }

    /// DELETE запрос
    async fn delete(
        &self,
        endpoint: PhemexEndpoint,
        params: HashMap<String, String>,
        _account_type: AccountType,
    ) -> ExchangeResult<Value> {
        // DELETE is always for order cancellation
        let group = match _account_type {
            AccountType::Spot => "SPOTORDER",
            _ => "CONTRACT",
        };
        self.rate_limit_wait(group, 1).await;

        let base_url = self.urls.rest_url(_account_type);
        let path = endpoint.path();

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

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request(path, &query, "");

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for PhemexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Phemex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max, rate_groups) = if let Ok(mut limiter) = self.rate_limiter.lock() {
            let (used, max) = limiter.primary_stats();
            let groups = limiter.all_stats()
                .into_iter()
                .map(|(name, cur, mx)| (name.to_string(), cur, mx))
                .collect();
            (used, max, groups)
        } else {
            (0, 0, Vec::new())
        };
        ConnectorStats {
            http_requests,
            http_errors,
            last_latency_ms,
            rate_used,
            rate_max,
            rate_groups,
            ws_ping_rtt_ms: 0,
        }
    }

    fn is_testnet(&self) -> bool {
        self.testnet
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
impl MarketData for PhemexConnector {
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
            AccountType::Spot => PhemexEndpoint::SpotOrderbook,
            _ => PhemexEndpoint::ContractOrderbook,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        let price_scale = self.get_price_scale(account_type);
        PhemexParser::parse_orderbook(&response, price_scale)
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
            AccountType::Spot => PhemexEndpoint::SpotKlines,
            _ => PhemexEndpoint::ContractKlines,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("resolution".to_string(), map_kline_interval(interval).to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        // Phemex v2 requires BOTH from + to together.
        // "to" alone is silently ignored.
        if let Some(et) = end_time {
            let interval_s = interval_to_secs(interval) as i64;
            let count = limit.unwrap_or(500) as i64;
            let to_s = et / 1000;
            let from_s = to_s - count * interval_s;
            params.insert("from".to_string(), from_s.to_string());
            params.insert("to".to_string(), to_s.to_string());
        }

        // Kline endpoint has weight=10 per Phemex API docs
        let response = self.get_weighted(endpoint, params, account_type, 10).await?;
        let price_scale = self.get_price_scale(account_type);
        PhemexParser::parse_klines(&response, price_scale)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let endpoint = match account_type {
            AccountType::Spot => PhemexEndpoint::SpotTicker24h,
            _ => PhemexEndpoint::ContractTicker24h,
        };

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(endpoint, params, account_type).await?;
        let price_scale = self.get_price_scale(account_type);
        PhemexParser::parse_ticker(&response, price_scale, account_type)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _response = self.get(PhemexEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get(PhemexEndpoint::Products, HashMap::new(), AccountType::Spot).await?;
        PhemexParser::parse_exchange_info(&response)
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



#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_public_connector() {
        let connector = PhemexConnector::public(false).await;
        assert!(connector.is_ok());
    }

    #[test]
    fn test_exchange_identity() {
        let mut group_limiter = GroupRateLimiter::new();
        group_limiter.add_group("OTHERS", 100, Duration::from_secs(60));
        group_limiter.add_group("CONTRACT", 500, Duration::from_secs(60));
        group_limiter.add_group("SPOTORDER", 500, Duration::from_secs(60));

        let connector = PhemexConnector {
            http: HttpClient::new(30_000).unwrap(),
            auth: None,
            urls: PhemexUrls::MAINNET,
            testnet: false,
            rate_limiter: Arc::new(Mutex::new(group_limiter)),
            default_price_scale: 4,
            default_value_scale: 8,
        };

        assert_eq!(connector.exchange_id(), ExchangeId::Phemex);
        assert!(!connector.is_testnet());
        assert_eq!(connector.exchange_type(), ExchangeType::Cex);
    }
}

fn interval_to_secs(interval: &str) -> u64 {
    match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "4h" => 14400,
        "12h" => 43200,
        "1d" => 86400,
        "1w" => 604800,
        _ => 3600,
    }
}
