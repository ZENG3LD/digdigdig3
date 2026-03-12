//! # Kraken Connector
//!
//! Implementation of all core traits for Kraken.
//!
//! ## Core traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//! - `Positions` - futures positions

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
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::DecayingRateLimiter;

use super::endpoints::{KrakenUrls, KrakenEndpoint, format_symbol, map_ohlc_interval};
use super::auth::KrakenAuth;
use super::parser::KrakenParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Kraken connector
pub struct KrakenConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<KrakenAuth>,
    /// URLs (mainnet/testnet)
    urls: KrakenUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (Kraken Spot Starter tier: max=15, decay=0.33/s)
    rate_limiter: Arc<Mutex<DecayingRateLimiter>>,
}

impl KrakenConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            KrakenUrls::TESTNET
        } else {
            KrakenUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(KrakenAuth::new)
            .transpose()?;

        // Initialize rate limiter: Kraken Spot Starter tier (max=15, decay=0.33/s)
        let rate_limiter = Arc::new(Mutex::new(
            DecayingRateLimiter::new(15.0, 0.33)
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
        })
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if needed
    async fn rate_limit_wait(&self) {
        loop {
            let wait_time = {
                let mut limiter = self.rate_limiter.lock().expect("Mutex poisoned");
                if limiter.try_acquire(1.0) {
                    return;
                }
                limiter.time_until_ready(1.0)
            };

            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: KrakenEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

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

        let response = self.http.get(&url, &HashMap::new()).await?;
        Ok(response)
    }

    /// POST request (Spot API uses POST for both public and private)
    ///
    /// Note: Kraken expects application/x-www-form-urlencoded, but our HttpClient
    /// always sends JSON. As a workaround, we send form params as query params
    /// since Kraken private endpoints accept parameters in either the body or URL.
    async fn post(
        &self,
        endpoint: KrakenEndpoint,
        params: HashMap<String, String>,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url(account_type);
        let path = endpoint.path();

        if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            // Sign request to get headers and form body
            let (headers, _body_str) = auth.sign_request(path, &params);

            // Build URL with path
            let url = format!("{}{}", base_url, path);

            // Use post_with_params - sends params as query string
            // The signature covers the POST body, but Kraken also accepts params in URL
            self.http.post_with_params(&url, &params, &json!({}), &headers).await
        } else {
            // Public POST endpoints (rare for Kraken)
            let url = format!("{}{}", base_url, path);
            self.http.post_with_params(&url, &params, &json!({}), &HashMap::new()).await
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Kraken-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all asset pairs information
    pub async fn get_asset_pairs(&self) -> ExchangeResult<Value> {
        self.get(KrakenEndpoint::SpotAssetPairs, HashMap::new(), AccountType::Spot).await
    }

    /// Get WebSocket authentication token
    pub async fn get_ws_token(&self) -> ExchangeResult<String> {
        let response = self.post(
            KrakenEndpoint::SpotWebSocketToken,
            HashMap::new(),
            AccountType::Spot,
        ).await?;

        let result = KrakenParser::extract_result(&response)?;
        result.get("token")
            .and_then(|t| t.as_str())
            .map(String::from)
            .ok_or_else(|| ExchangeError::Parse("Missing WebSocket token".to_string()))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for KrakenConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Kraken
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut lim) = self.rate_limiter.lock() {
            (lim.current_level() as u32, lim.max_level() as u32)
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
impl MarketData for KrakenConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());

        let response = self.get(KrakenEndpoint::SpotTicker, params, account_type).await?;

        // Response will use full format (XXBTZUSD), try both formats
        KrakenParser::parse_price(&response, &formatted)
            .or_else(|_| {
                // Try with XX prefix for BTC
                let full_format = if formatted.starts_with("XBT")
                    || formatted.starts_with("ETH")
                    || formatted.starts_with("LTC") {
                    format!("X{}", formatted)
                } else {
                    formatted.clone()
                };
                // Add Z prefix for USD
                let full_format = if full_format.ends_with("USD") {
                    format!("{}Z{}", &full_format[..full_format.len()-3], "USD")
                } else {
                    full_format
                };
                KrakenParser::parse_price(&response, &full_format)
            })
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());
        if let Some(d) = depth {
            params.insert("count".to_string(), d.to_string());
        }

        let response = self.get(KrakenEndpoint::SpotOrderbook, params, account_type).await?;

        // Try with different symbol formats
        KrakenParser::parse_orderbook(&response, &formatted)
            .or_else(|_| {
                let full_format = Self::to_full_format(&formatted);
                KrakenParser::parse_orderbook(&response, &full_format)
            })
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        _limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());
        params.insert("interval".to_string(), map_ohlc_interval(interval).to_string());

        let response = self.get(KrakenEndpoint::SpotOHLC, params, account_type).await?;

        KrakenParser::parse_klines(&response, &formatted)
            .or_else(|_| {
                let full_format = Self::to_full_format(&formatted);
                KrakenParser::parse_klines(&response, &full_format)
            })
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("pair".to_string(), formatted.clone());

        let response = self.get(KrakenEndpoint::SpotTicker, params, account_type).await?;

        KrakenParser::parse_ticker(&response, &formatted)
            .or_else(|_| {
                let full_format = Self::to_full_format(&formatted);
                KrakenParser::parse_ticker(&response, &full_format)
            })
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let response = self.get(KrakenEndpoint::ServerTime, HashMap::new(), AccountType::Spot).await?;
        KrakenParser::extract_result(&response)?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get_asset_pairs().await?;
        KrakenParser::parse_exchange_info(&response)
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



// Helper methods
impl KrakenConnector {
    /// Convert simplified symbol to full ISO format
    ///
    /// XBTUSD → XXBTZUSD
    /// ETHUSD → XETHZUSD
    fn to_full_format(symbol: &str) -> String {
        // Common conversions
        let mut result = symbol.to_string();

        // Add X prefix to crypto if not present
        if (result.starts_with("XBT") && !result.starts_with("XXBT"))
            || ((result.starts_with("ETH") || result.starts_with("LTC"))
                && !result.starts_with("XETH") && !result.starts_with("XLTC")) {
            result = format!("X{}", result);
        }

        // Add Z prefix to fiat if not present
        if result.ends_with("USD") && !result.ends_with("ZUSD") {
            let base = &result[..result.len() - 3];
            result = format!("{}ZUSD", base);
        } else if result.ends_with("EUR") && !result.ends_with("ZEUR") {
            let base = &result[..result.len() - 3];
            result = format!("{}ZEUR", base);
        }

        result
    }
}
