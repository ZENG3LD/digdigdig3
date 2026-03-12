//! # Bitfinex Connector
//!
//! Implementation of all core traits for Bitfinex API v2.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data endpoints
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Margin/futures positions

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
    Order, OrderSide, Balance, AccountInfo,
    Position,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{BitfinexUrls, BitfinexEndpoint, format_symbol, build_candle_key};
use super::auth::BitfinexAuth;
use super::parser::BitfinexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitfinex connector
pub struct BitfinexConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods only)
    auth: Option<BitfinexAuth>,
    /// URLs (mainnet)
    urls: BitfinexUrls,
    /// Rate limiter (conservative: 10 requests per 60 seconds)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
}

impl BitfinexConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, _testnet: bool) -> ExchangeResult<Self> {
        let urls = BitfinexUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(BitfinexAuth::new)
            .transpose()?;

        // Bitfinex rate limit: 90 requests per 60 seconds (matches registry rpm)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(90, Duration::from_secs(60))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            rate_limiter,
        })
    }

    /// Create connector for public methods only
    pub async fn public(_testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, _testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("lock");
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

    /// GET request
    async fn get(
        &self,
        endpoint: BitfinexEndpoint,
        path_params: &[(&str, &str)],
        query_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(endpoint.requires_auth());
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        // Build query string
        let query = if query_params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let url = format!("{}{}{}", base_url, path, query);

        let response = self.http.get(&url, &HashMap::new()).await?;
        BitfinexParser::check_error(&response)?;
        Ok(response)
    }

    /// POST request (authenticated)
    async fn post(
        &self,
        endpoint: BitfinexEndpoint,
        path_params: &[(&str, &str)],
        body: Value,
    ) -> ExchangeResult<Value> {
        // Rate limit before making request
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(true); // Always use auth URL for POST
        let mut path = endpoint.path().to_string();

        // Replace path parameters
        for (key, value) in path_params {
            path = path.replace(&format!("{{{}}}", key), value);
        }

        let url = format!("{}{}", base_url, path);

        // Get auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        // API path without /v2 prefix (auth expects "v2/auth/r/wallets" not "/v2/auth/r/wallets")
        let api_path = path.trim_start_matches('/');
        let body_str = body.to_string();
        let headers = auth.sign_request(api_path, &body_str);

        let response = self.http.post(&url, &body, &headers).await?;
        BitfinexParser::check_error(&response)?;
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitfinexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitfinex
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
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
        false // Bitfinex doesn't have a public testnet
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
impl MarketData for BitfinexConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };

        let response = self.get(
            BitfinexEndpoint::Ticker,
            &[("symbol", &formatted_symbol)],
            HashMap::new(),
        ).await?;

        let ticker = BitfinexParser::parse_ticker(&response, &formatted_symbol)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };

        // Use P0 precision (highest aggregation) for best performance
        let response = self.get(
            BitfinexEndpoint::Orderbook,
            &[("symbol", &formatted_symbol), ("precision", "P0")],
            HashMap::new(),
        ).await?;

        BitfinexParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };
        let candle_key = build_candle_key(&formatted_symbol, interval);

        let mut params = HashMap::new();
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.min(10000).to_string());
        }
        // Don't use sort=1 — it returns data from 2013. Default (newest-first) + parser.reverse() is correct.

        if let Some(et) = end_time {
            params.insert("end".to_string(), et.to_string());
        }

        let response = self.get(
            BitfinexEndpoint::Candles,
            &[("candle", &candle_key)],
            params,
        ).await?;

        BitfinexParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let formatted_symbol = if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        };

        let response = self.get(
            BitfinexEndpoint::Ticker,
            &[("symbol", &formatted_symbol)],
            HashMap::new(),
        ).await?;

        BitfinexParser::parse_ticker(&response, &formatted_symbol)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(
            BitfinexEndpoint::PlatformStatus,
            &[],
            HashMap::new(),
        ).await?;

        // Platform status returns [1] for operative, [0] for maintenance
        if let Some(arr) = response.as_array() {
            if !arr.is_empty() {
                if let Some(status) = arr[0].as_i64() {
                    if status == 1 {
                        return Ok(());
                    }
                }
            }
        }

        Err(ExchangeError::Network("Platform in maintenance".to_string()))
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Use Bitfinex v1 symbols_details endpoint (returns array with pair info)
        // Note: v1 is still supported and returns more detail than v2 conf endpoints
        self.rate_limit_wait().await;
        let url = "https://api.bitfinex.com/v1/symbols_details";
        let response = self.http.get(url, &HashMap::new()).await?;
        BitfinexParser::parse_exchange_info(&response)
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


