//! # Coinbase Connector
//!
//! Implementation of all core traits for Coinbase Advanced Trade API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data (spot + LIMITED perpetuals)
//! - `Trading` - trading operations (spot + perpetuals)
//! - `Account` - account information
//!
//! ## Perpetual Futures Support
//!
//! Coinbase offers perpetual futures through the Advanced Trade API with significant limitations:
//!
//! ### What Works (Public REST API):
//! - ✅ `get_price()` - Get current perpetual price via best bid/ask
//! - ✅ `get_ticker()` - Get ticker data for perpetuals
//! - ✅ Product listing with `product_type=FUTURE&contract_expiry_type=PERPETUAL`
//!
//! ### What Does NOT Work (Public REST API):
//! - ❌ `get_orderbook()` - Orderbook endpoint is **SPOT ONLY**
//! - ❌ `get_klines()` - Candles endpoint is **SPOT ONLY**
//!
//! ### Alternatives for Full Perpetuals Data:
//! 1. **WebSocket Feeds** - Use Advanced Trade WebSocket with channels:
//!    - `level2` - Real-time orderbook updates
//!    - `candles` - Real-time candlestick updates
//!    - `ticker` - Price updates
//!    - `futures_balance_summary` - Perpetuals-specific data
//!
//! 2. **INTX API** - Coinbase International Exchange for institutional users:
//!    - REST: `/instruments/{instrument}/candles` - Historical candles
//!    - REST: `/instruments/{instrument}/quote` - Best bid/ask (L1)
//!    - WebSocket: `L2_DATA` channel - Full orderbook depth
//!    - WebSocket: `CANDLES` channel - Candlestick updates
//!    - **Note**: Requires authentication even for market data
//!
//! 3. **Authenticated Advanced Trade** - With API credentials:
//!    - May have access to additional perpetuals endpoints
//!    - Still limited compared to INTX
//!
//! ### Symbol Format:
//! - Spot: `BTC-USD` (base-quote)
//! - Perpetuals: `BTC-PERP` (base-PERP, quote ignored)
//!
//! ### Trading:
//! - Perpetual futures trading IS supported via Advanced Trade API
//! - Requires USDC margin and proper collateral
//! - Up to 10x leverage available
//! - Same order endpoints work for both spot and perpetuals
//!
//! ## References:
//! - Research: `coinbase_futures_data_api_report.md`
//! - Advanced Trade Docs: https://docs.cdp.coinbase.com/advanced-trade/docs/perpetuals
//! - INTX Docs: https://docs.cloud.coinbase.com/intx/docs/welcome

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
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::WeightRateLimiter;

use super::endpoints::{CoinbaseUrls, CoinbaseEndpoint, format_symbol, map_kline_interval};
use super::auth::CoinbaseAuth;
use super::parser::CoinbaseParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Coinbase connector
pub struct CoinbaseConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<CoinbaseAuth>,
    /// Rate limiter (30 requests per second for private, 10 for public)
    rate_limiter: Arc<Mutex<WeightRateLimiter>>,
}

impl CoinbaseConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = if let Some(creds) = credentials {
            Some(CoinbaseAuth::new(&creds)
                .map_err(ExchangeError::Auth)?)
        } else {
            None
        };

        // Initialize rate limiter: 30 requests per second (Coinbase private tier)
        let rate_limiter = Arc::new(Mutex::new(
            WeightRateLimiter::new(30, Duration::from_secs(1))
        ));

        Ok(Self {
            http,
            auth,
            rate_limiter,
        })
    }

    /// Create connector only for public methods
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Update rate limiter from Coinbase response headers
    ///
    /// Coinbase reports: CB-RATELIMIT-REMAINING = remaining, CB-RATELIMIT-LIMIT = total limit
    fn update_rate_from_headers(&self, headers: &HeaderMap) {
        let remaining = headers
            .get("CB-RATELIMIT-REMAINING")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());

        let limit = headers
            .get("CB-RATELIMIT-LIMIT")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok())
            .or_else(|| {
                // Fall back to the limiter's max_weight if no limit header
                self.rate_limiter.lock().ok().map(|l| l.max_weight())
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
        endpoint: CoinbaseEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

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

        // Decide whether to use public or private endpoint
        let (base_url, use_public) = if endpoint.is_private() && self.auth.is_some() {
            (CoinbaseUrls::base_url(), false)
        } else if endpoint.has_public_alternative() {
            (CoinbaseUrls::market_url(), true)
        } else if !endpoint.is_private() {
            (CoinbaseUrls::base_url(), false)
        } else {
            return Err(ExchangeError::Auth("Authentication required".to_string()));
        };

        // Use public market path if available
        let final_path = if use_public && endpoint.market_path().is_some() {
            endpoint.market_path().expect("market_path() is Some, checked above")
        } else {
            path
        };

        let full_path = format!("{}{}", final_path, query);
        let url = format!("{}{}", base_url, full_path);

        // Add auth headers if needed
        let headers = if !use_public && endpoint.is_private() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            auth.sign_request("GET", &full_path)
                .map_err(ExchangeError::Auth)?
        } else {
            HashMap::new()
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: CoinbaseEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let base_url = CoinbaseUrls::base_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers (POST always requires auth)
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("POST", path)
            .map_err(ExchangeError::Auth)?;

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for CoinbaseConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Coinbase
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
        false // Coinbase doesn't have testnet for Advanced Trade
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // Spot: Full support
        // FuturesCross: LIMITED - only ticker/price data available via public REST
        //   - Orderbook and candles are SPOT ONLY via REST API
        //   - Full futures data requires WebSocket or INTX API with auth
        vec![AccountType::Spot, AccountType::FuturesCross]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for CoinbaseConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let product_id = format_symbol(&symbol, account_type);

        if self.auth.is_some() {
            // Authenticated: use BestBidAsk endpoint (private)
            let mut params = HashMap::new();
            params.insert("product_ids".to_string(), product_id);
            let response = self.get(CoinbaseEndpoint::BestBidAsk, params).await?;
            let ticker = CoinbaseParser::parse_ticker(&response)?;
            Ok(ticker.last_price)
        } else {
            // Public: use ProductBook endpoint (has public /market alternative)
            let mut params = HashMap::new();
            params.insert("product_id".to_string(), product_id);
            let response = self.get(CoinbaseEndpoint::ProductBook, params).await?;
            let orderbook = CoinbaseParser::parse_orderbook(&response)?;
            // Derive price from best bid/ask
            let bid = orderbook.bids.first().map(|(p, _)| *p);
            let ask = orderbook.asks.first().map(|(p, _)| *p);
            match (bid, ask) {
                (Some(b), Some(a)) => Ok((b + a) / 2.0),
                (Some(b), None) => Ok(b),
                (None, Some(a)) => Ok(a),
                (None, None) => Err(ExchangeError::Parse("No bid or ask in orderbook".into())),
            }
        }
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let product_id = format_symbol(&symbol, account_type);

        if self.auth.is_some() {
            // Authenticated: use BestBidAsk endpoint (private)
            let mut params = HashMap::new();
            params.insert("product_ids".to_string(), product_id.clone());
            let response = self.get(CoinbaseEndpoint::BestBidAsk, params).await?;
            CoinbaseParser::parse_ticker(&response)
        } else {
            // Public: use ProductBook endpoint (has public /market alternative)
            let mut params = HashMap::new();
            params.insert("product_id".to_string(), product_id.clone());
            let response = self.get(CoinbaseEndpoint::ProductBook, params).await?;
            let orderbook = CoinbaseParser::parse_orderbook(&response)?;
            // Build ticker from orderbook data
            let bid_price = orderbook.bids.first().map(|(p, _)| *p);
            let ask_price = orderbook.asks.first().map(|(p, _)| *p);
            let last_price = match (bid_price, ask_price) {
                (Some(b), Some(a)) => (b + a) / 2.0,
                (Some(b), None) => b,
                (None, Some(a)) => a,
                (None, None) => return Err(ExchangeError::Parse("No bid or ask in orderbook".into())),
            };
            Ok(Ticker {
                symbol: product_id,
                last_price,
                bid_price,
                ask_price,
                high_24h: None,
                low_24h: None,
                volume_24h: None,
                quote_volume_24h: None,
                price_change_24h: None,
                price_change_percent_24h: None,
                timestamp: orderbook.timestamp,
            })
        }
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        // LIMITATION: Coinbase REST API orderbook endpoint is SPOT ONLY
        // For perpetuals, use WebSocket level2 channel or INTX API
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            return Err(ExchangeError::NotSupported(
                "Coinbase REST API orderbook is SPOT ONLY. For perpetual futures orderbook, use WebSocket or INTX API".to_string()
            ));
        }

        let mut params = HashMap::new();
        params.insert("product_id".to_string(), format_symbol(&symbol, account_type));

        if let Some(d) = depth {
            params.insert("limit".to_string(), d.to_string());
        }

        let response = self.get(CoinbaseEndpoint::ProductBook, params).await?;
        CoinbaseParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            return Err(ExchangeError::NotSupported(
                "Coinbase REST API candles are SPOT ONLY".to_string()
            ));
        }

        let product_id = format_symbol(&symbol, account_type);
        let granularity = map_kline_interval(interval);

        let endpoint = CoinbaseEndpoint::Candles;
        let base_path = format!("{}/{}/candles", endpoint.path(), product_id);

        let mut params = HashMap::new();
        params.insert("granularity".to_string(), granularity.to_string());

        // Coinbase requires BOTH start + end, max 300 candles per window.
        // "end" alone is ignored.
        if let Some(et) = end_time {
            let end_s = et / 1000;
            let interval_s = interval_to_secs(interval) as i64;
            let count = limit.unwrap_or(350).min(350) as i64;
            let start_s = end_s - count * interval_s;
            params.insert("start".to_string(), start_s.to_string());
            params.insert("end".to_string(), end_s.to_string());
        }

        let query: Vec<String> = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_str = if query.is_empty() {
            String::new()
        } else {
            format!("?{}", query.join("&"))
        };

        let base_url = if self.auth.is_some() {
            CoinbaseUrls::base_url()
        } else {
            CoinbaseUrls::market_url()
        };

        let url = format!("{}{}{}", base_url, base_path, query_str);

        let headers = if let Some(auth) = &self.auth {
            let full_path = format!("{}{}", base_path, query_str);
            auth.sign_request("GET", &full_path)
                .map_err(ExchangeError::Auth)?
        } else {
            HashMap::new()
        };

        self.rate_limit_wait(1).await;
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        let mut klines = CoinbaseParser::parse_klines(&response)?;

        if let Some(l) = limit {
            klines.truncate(l.min(350) as usize);
        }

        Ok(klines)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Coinbase doesn't have a dedicated ping endpoint
        // Use the server time endpoint as a health check
        // base_url() already includes /api/v3/brokerage, so just append /time
        let url = format!("{}/time", CoinbaseUrls::base_url());
        self.http.get(&url, &HashMap::new()).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // GET /market/products (public) returns products list
        let params = HashMap::new();
        let response = self.get(CoinbaseEndpoint::Products, params).await?;
        CoinbaseParser::parse_exchange_info(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════



// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS (Not supported by Coinbase)
// ═══════════════════════════════════════════════════════════════════════════════


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
