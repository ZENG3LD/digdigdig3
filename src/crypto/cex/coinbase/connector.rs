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
    Order, OrderSide, OrderType,Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
};
use crate::core::types::{
    WithdrawRequest, WithdrawResponse, DepositAddress,
    FundsHistoryFilter, FundsRecord, FundsRecordType,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions, CancelAll, CustodialFunds,
};
use crate::core::types::{CancelAllResponse, OrderResult};
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

    /// GET request against the v2 API with a dynamic path (account-specific endpoints).
    ///
    /// `path` must be a fully constructed path like `/accounts/{uuid}/deposits`.
    async fn get_v2(&self, path: &str, params: HashMap<String, String>) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let query = if params.is_empty() {
            String::new()
        } else {
            let qs: Vec<String> = params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            format!("?{}", qs.join("&"))
        };

        let full_path = format!("{}{}", path, query);
        let url = format!("{}{}", CoinbaseUrls::v2_url(), full_path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("GET", &full_path)
            .map_err(ExchangeError::Auth)?;

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// POST request against the v2 API with a dynamic path.
    async fn post_v2(&self, path: &str, body: Value) -> ExchangeResult<Value> {
        self.rate_limit_wait(1).await;

        let url = format!("{}{}", CoinbaseUrls::v2_url(), path);

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("POST", path)
            .map_err(ExchangeError::Auth)?;

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        Ok(response)
    }

    /// Find the Coinbase account UUID for a given asset (e.g. "BTC", "ETH").
    ///
    /// Coinbase uses per-asset account UUIDs in the v2 API. This helper fetches
    /// the account list and returns the UUID for the requested asset.
    async fn find_account_id(&self, asset: &str) -> ExchangeResult<String> {
        let response = self.get(CoinbaseEndpoint::Accounts, HashMap::new()).await?;
        CoinbaseParser::find_account_id_for_asset(&response, asset)
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

#[async_trait]
impl Trading for CoinbaseConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let product_id = format_symbol(&symbol, account_type);
        let side_str = match side { OrderSide::Buy => "BUY", OrderSide::Sell => "SELL" };
        let client_order_id = req.client_order_id.clone()
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let order_config = match req.order_type {
            OrderType::Market => {
                // Coinbase market buy uses quote_size; market sell uses base_size
                let size_field = match side {
                    OrderSide::Buy => "quote_size",
                    OrderSide::Sell => "base_size",
                };
                json!({ "market_market_ioc": { size_field: quantity.to_string() } })
            }
            OrderType::Limit { price } => {
                let post_only = matches!(req.time_in_force, crate::core::TimeInForce::PostOnly);
                let tif_key = match req.time_in_force {
                    crate::core::TimeInForce::Ioc => "limit_limit_ioc",
                    crate::core::TimeInForce::Fok => "limit_limit_fok",
                    crate::core::TimeInForce::PostOnly => "limit_limit_gtc",
                    _ => "limit_limit_gtc",
                };
                json!({
                    tif_key: {
                        "base_size": quantity.to_string(),
                        "limit_price": price.to_string(),
                        "post_only": post_only,
                    }
                })
            }
            OrderType::PostOnly { price } => {
                json!({
                    "limit_limit_gtc": {
                        "base_size": quantity.to_string(),
                        "limit_price": price.to_string(),
                        "post_only": true,
                    }
                })
            }
            OrderType::Ioc { price } => {
                let px_str = price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string());
                json!({
                    "limit_limit_ioc": {
                        "base_size": quantity.to_string(),
                        "limit_price": px_str,
                        "post_only": false,
                    }
                })
            }
            OrderType::Fok { price } => {
                json!({
                    "limit_limit_fok": {
                        "base_size": quantity.to_string(),
                        "limit_price": price.to_string(),
                        "post_only": false,
                    }
                })
            }
            OrderType::StopMarket { stop_price } => {
                json!({
                    "stop_limit_stop_limit_gtc": {
                        "base_size": quantity.to_string(),
                        "limit_price": stop_price.to_string(),
                        "stop_price": stop_price.to_string(),
                        "stop_direction": match side {
                            OrderSide::Buy => "STOP_DIRECTION_STOP_UP",
                            OrderSide::Sell => "STOP_DIRECTION_STOP_DOWN",
                        },
                    }
                })
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                json!({
                    "stop_limit_stop_limit_gtc": {
                        "base_size": quantity.to_string(),
                        "limit_price": limit_price.to_string(),
                        "stop_price": stop_price.to_string(),
                        "stop_direction": match side {
                            OrderSide::Buy => "STOP_DIRECTION_STOP_UP",
                            OrderSide::Sell => "STOP_DIRECTION_STOP_DOWN",
                        },
                    }
                })
            }
            OrderType::Gtd { price, expire_time } => {
                // Coinbase supports GTD via end_time parameter in limit_limit_gtd
                let end_time = chrono::DateTime::from_timestamp(expire_time / 1000, 0)
                    .map(|dt| dt.to_rfc3339())
                    .unwrap_or_default();
                json!({
                    "limit_limit_gtd": {
                        "base_size": quantity.to_string(),
                        "limit_price": price.to_string(),
                        "end_time": end_time,
                        "post_only": false,
                    }
                })
            }
            OrderType::Oco { price, stop_price, stop_limit_price } => {
                // Coinbase supports bracket orders: trigger_bracket_gtc
                json!({
                    "trigger_bracket_gtc": {
                        "base_size": quantity.to_string(),
                        "limit_price": price.to_string(),
                        "stop_trigger_price": stop_price.to_string(),
                    }
                })
            }
            OrderType::Bracket { price, take_profit, stop_loss } => {
                let px_str = price.map(|p| p.to_string()).unwrap_or_else(|| "0".to_string());
                let _ = take_profit;
                json!({
                    "trigger_bracket_gtc": {
                        "base_size": quantity.to_string(),
                        "limit_price": px_str,
                        "stop_trigger_price": stop_loss.to_string(),
                    }
                })
            }
            OrderType::ReduceOnly { .. } | OrderType::TrailingStop { .. }
            | OrderType::Iceberg { .. } | OrderType::Twap { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
                ));
            }
        };

        let body = json!({
            "client_order_id": client_order_id,
            "product_id": product_id,
            "side": side_str,
            "order_configuration": order_config
        });

        let response = self.post(CoinbaseEndpoint::CreateOrder, body).await?;
        CoinbaseParser::parse_order(&response).map(PlaceOrderResponse::Simple)
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // GET /orders/historical/batch with order_status=FILLED,CANCELLED
        let mut params = HashMap::new();
        params.insert("order_status".to_string(), "FILLED,CANCELLED,EXPIRED".to_string());

        if let Some(ref symbol) = filter.symbol {
            params.insert("product_id".to_string(), format_symbol(symbol, account_type));
        }

        if let Some(start) = filter.start_time {
            // Coinbase uses RFC3339 timestamps
            if let Some(dt) = chrono::DateTime::from_timestamp(start / 1000, 0) {
                params.insert("start_date".to_string(), dt.to_rfc3339());
            }
        }

        if let Some(end) = filter.end_time {
            if let Some(dt) = chrono::DateTime::from_timestamp(end / 1000, 0) {
                params.insert("end_date".to_string(), dt.to_rfc3339());
            }
        }

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let query: Vec<String> = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_str = format!("?{}", query.join("&"));

        let path = format!("{}{}", CoinbaseEndpoint::ListOrders.path(), query_str);
        let url = format!("{}{}", CoinbaseUrls::base_url(), path);

        let headers = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?
            .sign_request("GET", &path)
            .map_err(ExchangeError::Auth)?;

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;

        let orders = response.get("orders")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing orders array".into()))?
            .iter()
            .filter_map(|order_json| {
                let order_obj = serde_json::json!({"order": order_json});
                CoinbaseParser::parse_order(&order_obj).ok()
            })
            .collect();

        Ok(orders)
    }

async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                // Get order details before cancelling
                let order = self.get_order(&symbol.to_string(), order_id, account_type).await?;

                let body = json!({ "order_ids": [order_id] });
                let response = self.post(CoinbaseEndpoint::CancelOrders, body).await?;

                let results = response.get("results")
                    .and_then(|r| r.as_array())
                    .ok_or_else(|| ExchangeError::Parse("Missing results array".into()))?;

                let success = results.iter()
                    .any(|r| r.get("success").and_then(|s| s.as_bool()).unwrap_or(false));

                if success {
                    Ok(order)
                } else {
                    Err(ExchangeError::Api { code: 0, message: "Order cancellation failed".to_string() })
                }
            }
            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let sym_str = symbol.as_ref().map(|s| s.to_string()).unwrap_or_default();
                let open_orders = self.get_open_orders(
                    symbol.as_ref().map(|s| s.to_string()).as_deref(),
                    account_type,
                ).await?;

                if open_orders.is_empty() {
                    return Ok(Order {
                        id: format!("cancel_all_{}", crate::core::timestamp_millis()),
                        client_order_id: None,
                        symbol: sym_str,
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
                    });
                }

                let order_ids_vec: Vec<String> = open_orders.iter().map(|o| o.id.clone()).collect();
                let body = json!({ "order_ids": order_ids_vec });
                let response = self.post(CoinbaseEndpoint::CancelOrders, body).await?;
                let _ = response;

                Ok(Order {
                    id: format!("cancel_all_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: symbol.as_ref().map(|s| s.to_string()).unwrap_or_default(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: open_orders.len() as f64,
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
                let sym_str = symbol.to_string();
                let open_orders = self.get_open_orders(
                    Some(&sym_str),
                    account_type,
                ).await?;

                if open_orders.is_empty() {
                    return Ok(Order {
                        id: format!("cancel_all_{}", crate::core::timestamp_millis()),
                        client_order_id: None,
                        symbol: sym_str,
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
                    });
                }

                let order_ids_vec: Vec<String> = open_orders.iter().map(|o| o.id.clone()).collect();
                let body = json!({ "order_ids": order_ids_vec });
                let response = self.post(CoinbaseEndpoint::CancelOrders, body).await?;
                let _ = response;

                Ok(Order {
                    id: format!("cancel_all_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: open_orders.len() as f64,
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
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for batch cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                // Coinbase supports batch cancel natively: POST /orders/batch_cancel
                let body = json!({ "order_ids": order_ids });
                let response = self.post(CoinbaseEndpoint::CancelOrders, body).await?;
                let _ = response;

                Ok(Order {
                    id: format!("batch_cancel_{}", crate::core::timestamp_millis()),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::Canceled,
                    price: None,
                    stop_price: None,
                    quantity: order_ids.len() as f64,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: 0,
                    updated_at: Some(crate::core::timestamp_millis() as i64),
                    time_in_force: crate::core::TimeInForce::Gtc,
                })
            }
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType, // Not used, order_id is globally unique
    ) -> ExchangeResult<Order> {
        // Build path with order_id
        let endpoint = CoinbaseEndpoint::OrderDetails;
        let path = format!("{}/{}", endpoint.path(), order_id);

        let url = format!("{}{}", CoinbaseUrls::base_url(), path);

        let headers = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?
            .sign_request("GET", &path)
            .map_err(ExchangeError::Auth)?;

        self.rate_limit_wait(1).await;
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_rate_from_headers(&resp_headers);
        CoinbaseParser::parse_order(&response)
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

        let mut params = HashMap::new();
        params.insert("order_status".to_string(), "OPEN".to_string());

        if let Some(s) = symbol {
            params.insert("product_id".to_string(), format_symbol(&s, account_type));
        }

        let query: Vec<String> = params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_str = format!("?{}", query.join("&"));

        let path = format!("{}{}", CoinbaseEndpoint::ListOrders.path(), query_str);
        let url = format!("{}{}", CoinbaseUrls::base_url(), path);

        let headers = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?
            .sign_request("GET", &path)
            .map_err(ExchangeError::Auth)?;

        let response = self.http.get_with_headers(&url, &HashMap::new(), &headers).await?;

        let orders = response.get("orders")
            .and_then(|o| o.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing orders array".into()))?
            .iter()
            .filter_map(|order_json| {
                let order_obj = json!({"order": order_json});
                CoinbaseParser::parse_order(&order_obj).ok()
            })
            .collect();

        Ok(orders)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for CoinbaseConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset;
        let _account_type = query.account_type;
        let response = self.get(CoinbaseEndpoint::Accounts, HashMap::new()).await?;
        CoinbaseParser::parse_balance(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Get transaction summary for fee tier info
        let response = self.get(CoinbaseEndpoint::TransactionSummary, HashMap::new()).await?;

        let maker_commission = response.get("fee_tier")
            .and_then(|ft| ft.get("maker_fee_rate"))
            .and_then(|mfr| mfr.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        let taker_commission = response.get("fee_tier")
            .and_then(|ft| ft.get("taker_fee_rate"))
            .and_then(|tfr| tfr.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);

        // Get balances
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission,
            taker_commission,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // GET /transaction_summary returns fee tier info
        let response = self.get(CoinbaseEndpoint::TransactionSummary, HashMap::new()).await?;

        let maker_rate = response.get("fee_tier")
            .and_then(|ft| ft.get("maker_fee_rate"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.006);

        let taker_rate = response.get("fee_tier")
            .and_then(|ft| ft.get("taker_fee_rate"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.008);

        let tier = response.get("fee_tier")
            .and_then(|ft| ft.get("pricing_tier"))
            .and_then(|v| v.as_str())
            .map(String::from);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(String::from),
            tier,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS (Not supported by Coinbase)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for CoinbaseConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let _account_type = query.account_type;

        Err(ExchangeError::NotSupported("Coinbase does not support futures/positions".to_string()))
    
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let _symbol_str = _symbol;
        let _symbol = {
            let parts: Vec<&str> = _symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: _symbol_str.to_string(), quote: String::new(), raw: Some(_symbol_str.to_string()) }
            }
        };

        Err(ExchangeError::NotSupported("Coinbase does not support funding rates".to_string()))
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { symbol: ref _symbol, leverage: _leverage, account_type: _account_type } => {
                let _symbol = _symbol.clone();

                Err(ExchangeError::NotSupported("Coinbase does not support leverage".to_string()))
    
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
impl CancelAll for CoinbaseConnector {
    /// Cancel all open orders, optionally filtered to a single symbol.
    ///
    /// Coinbase has no single "cancel all" endpoint — implementation is 2-step:
    /// 1. Fetch all open orders (optionally filtered by symbol).
    /// 2. Call `POST /orders/batch_cancel` in chunks of 100.
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let symbol_filter = match &scope {
            CancelScope::All { symbol } => symbol.as_ref().map(|s| s.to_string()),
            CancelScope::BySymbol { symbol } => Some(symbol.to_string()),
            _ => return Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported in cancel_all_orders", scope)
            )),
        };

        // Step 1: fetch open orders
        let open_orders = self.get_open_orders(
            symbol_filter.as_deref(),
            account_type,
        ).await?;

        if open_orders.is_empty() {
            return Ok(CancelAllResponse {
                cancelled_count: 0,
                failed_count: 0,
                details: vec![],
            });
        }

        let order_ids: Vec<String> = open_orders.iter().map(|o| o.id.clone()).collect();

        // Step 2: batch cancel in chunks of 100 (Coinbase limit)
        let mut cancelled_count = 0u32;
        let mut failed_count = 0u32;
        let mut details: Vec<OrderResult> = Vec::new();

        for chunk in order_ids.chunks(100) {
            let body = serde_json::json!({ "order_ids": chunk });
            let response = self.post(CoinbaseEndpoint::CancelOrders, body).await?;

            if let Some(results) = response.get("results").and_then(|r| r.as_array()) {
                for item in results {
                    let success = item.get("success")
                        .and_then(|s| s.as_bool())
                        .unwrap_or(false);
                    let order_id = item.get("order_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let failure_reason = item.get("failure_reason")
                        .and_then(|v| v.as_str())
                        .filter(|s| !s.is_empty())
                        .map(|s| s.to_string());

                    if success {
                        cancelled_count += 1;
                    } else {
                        failed_count += 1;
                    }

                    details.push(OrderResult {
                        order: None,
                        client_order_id: None,
                        success,
                        error: failure_reason,
                        error_code: None,
                    });
                    let _ = order_id;
                }
            }
        }

        Ok(CancelAllResponse {
            cancelled_count,
            failed_count,
            details,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management for Coinbase.
///
/// Uses the Coinbase v2 API endpoints which operate on per-asset account UUIDs.
/// The account UUID is resolved automatically via the `/accounts` endpoint.
///
/// - Deposit address: `POST /v2/accounts/{id}/addresses`
/// - Withdraw:        `POST /v2/accounts/{id}/transactions` (type=send)
/// - Deposit history: `GET  /v2/accounts/{id}/deposits`
/// - Withdrawal hist: `GET  /v2/accounts/{id}/transactions` (type=send)
#[async_trait]
impl CustodialFunds for CoinbaseConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        _network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        // Resolve the per-asset account UUID first
        let account_id = self.find_account_id(asset).await?;
        let path = format!("/accounts/{}/addresses", account_id);

        let response = self.post_v2(&path, serde_json::json!({})).await?;
        CoinbaseParser::parse_deposit_address(&response, asset)
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let account_id = self.find_account_id(&req.asset).await?;
        let path = format!("/accounts/{}/transactions", account_id);

        let mut body = serde_json::json!({
            "type": "send",
            "to": req.address,
            "amount": req.amount.to_string(),
            "currency": req.asset.to_uppercase(),
        });

        // Add destination tag / memo if present (required for XRP, XLM, etc.)
        if let Some(ref tag) = req.tag {
            body["destination_tag"] = serde_json::json!(tag);
        }

        // Network hint — Coinbase uses the network field for certain assets
        if let Some(ref network) = req.network {
            body["network"] = serde_json::json!(network);
        }

        let response = self.post_v2(&path, body).await?;
        CoinbaseParser::parse_withdraw_response(&response)
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let asset = filter.asset.as_deref().unwrap_or("USD");
        let account_id = self.find_account_id(asset).await?;

        match filter.record_type {
            FundsRecordType::Deposit => {
                let mut params = HashMap::new();
                if let Some(limit) = filter.limit {
                    params.insert("limit".to_string(), limit.to_string());
                }
                let path = format!("/accounts/{}/deposits", account_id);
                let response = self.get_v2(&path, params).await?;
                CoinbaseParser::parse_deposit_history(&response, asset)
            }

            FundsRecordType::Withdrawal => {
                let mut params = HashMap::new();
                if let Some(limit) = filter.limit {
                    params.insert("limit".to_string(), limit.to_string());
                }
                let path = format!("/accounts/{}/transactions", account_id);
                let response = self.get_v2(&path, params).await?;
                CoinbaseParser::parse_withdrawal_history(&response, asset)
            }

            FundsRecordType::Both => {
                // Fetch deposits and outgoing transactions, combine them
                let dep_path = format!("/accounts/{}/deposits", account_id);
                let txn_path = format!("/accounts/{}/transactions", account_id);

                let mut params = HashMap::new();
                if let Some(limit) = filter.limit {
                    params.insert("limit".to_string(), limit.to_string());
                }

                let dep_response = self.get_v2(&dep_path, params.clone()).await?;
                let txn_response = self.get_v2(&txn_path, params).await?;

                let mut records = CoinbaseParser::parse_deposit_history(&dep_response, asset)?;
                records.extend(CoinbaseParser::parse_withdrawal_history(&txn_response, asset)?);
                Ok(records)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (not part of core traits)
// ═══════════════════════════════════════════════════════════════════════════════

impl CoinbaseConnector {
    /// Get fill history — paginated list of all fills for completed orders.
    ///
    /// `GET /api/v3/brokerage/orders/historical/fills`
    ///
    /// # Parameters
    /// - `order_id`: Filter by order ID (optional)
    /// - `product_id`: Filter by product/symbol (optional)
    /// - `limit`: Max number of fills to return (optional, max 100)
    /// - `cursor`: Pagination cursor from a previous response (optional)
    pub async fn get_fill_history(
        &self,
        order_id: Option<&str>,
        product_id: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(oid) = order_id {
            params.insert("order_id".to_string(), oid.to_string());
        }
        if let Some(pid) = product_id {
            params.insert("product_id".to_string(), pid.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(c) = cursor {
            params.insert("cursor".to_string(), c.to_string());
        }
        self.get(CoinbaseEndpoint::FillHistory, params).await
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
