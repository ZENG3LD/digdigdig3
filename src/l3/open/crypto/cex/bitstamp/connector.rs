//! # Bitstamp Connector
//!
//! Implementation of all core traits for Bitstamp V2 API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//!
//! ## Extended Methods
//! Additional Bitstamp-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use async_trait::async_trait;
use serde_json::Value;

use crate::core::{
    HttpClient, Credentials,
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    CancelAllResponse,
    ExchangeIdentity, MarketData, Trading, Account,
    CancelAll, AmendOrder, CustodialFunds,
    AmendRequest,
    DepositAddress, WithdrawResponse, FundsRecord,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::traits::AccountLedger;
use crate::core::types::SymbolInfo;
use crate::core::types::ConnectorStats;
use crate::core::types::{WithdrawRequest, FundsHistoryFilter, FundsRecordType, LedgerEntry, LedgerFilter};
use crate::core::types::{UserTrade, UserTradeFilter};
use crate::core::types::{RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits, OrderbookCapabilities, WsBookChannel};
use crate::core::utils::{RuntimeLimiter, RateLimitMonitor, RateLimitPressure};
use crate::core::utils::PrecisionCache;

use super::endpoints::{BitstampUrls, BitstampEndpoint, format_symbol, map_kline_interval};
use super::auth::BitstampAuth;
use super::parser::BitstampParser;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CAPABILITIES (static — embedded in binary, no allocation)
// ═══════════════════════════════════════════════════════════════════════════════

static BITSTAMP_RATE_POOLS: &[RestLimitPool] = &[RestLimitPool {
    name: "default",
    max_budget: 400,
    window_seconds: 1,
    is_weight: false,
    has_server_headers: false,
    server_header: None,
    header_reports_used: false,
}];

static BITSTAMP_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Simple,
    rest_pools: BITSTAMP_RATE_POOLS,
    decaying: None,
    endpoint_weights: &[],
    ws: WsLimits {
        max_connections: None,
        max_subs_per_conn: None,
        max_msg_per_sec: None,
        max_streams_per_conn: None,
    },
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitstamp connector
pub struct BitstampConnector {
    /// HTTP client
    http: HttpClient,
    /// Reqwest client for form-encoded POSTs
    reqwest_client: reqwest::Client,
    /// Authentication (None for public methods)
    auth: Option<BitstampAuth>,
    /// Runtime rate limiter (Simple model: 400 req/1s)
    limiter: Arc<Mutex<RuntimeLimiter>>,
    /// Pressure monitor — logs transitions, gates non-essential requests at >= 90%
    monitor: Arc<Mutex<RateLimitMonitor>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl BitstampConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let reqwest_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .map_err(|e| ExchangeError::Network(format!("Failed to create HTTP client: {}", e)))?;

        let auth = credentials.as_ref().map(BitstampAuth::new);

        let limiter = Arc::new(Mutex::new(RuntimeLimiter::from_caps(&BITSTAMP_RATE_CAPS)));
        let monitor = Arc::new(Mutex::new(RateLimitMonitor::new("Bitstamp")));

        Ok(Self {
            http,
            reqwest_client,
            auth,
            limiter,
            monitor,
            precision: PrecisionCache::new(),
        })
    }

    /// Create connector only for public methods
    pub async fn public() -> ExchangeResult<Self> {
        Self::new(None).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HTTP HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit budget. Non-essential requests are dropped at >= 90% utilization.
    ///
    /// Returns `true` if acquired, `false` if dropped due to cutoff pressure.
    /// Trading endpoints should pass `essential: true` to always wait through.
    async fn rate_limit_wait(&self, weight: u32, essential: bool) -> bool {
        loop {
            let wait_time = {
                let mut limiter = self.limiter.lock()
                    .expect("rate limiter mutex poisoned");

                let pressure = self.monitor.lock()
                    .expect("rate monitor mutex poisoned")
                    .check(&mut limiter);
                if pressure >= RateLimitPressure::Cutoff && !essential {
                    return false;
                }

                if limiter.try_acquire("default", weight) {
                    return true;
                }
                limiter.time_until_ready("default", weight)
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: BitstampEndpoint,
        pair: Option<&str>,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }

        let base_url = BitstampUrls::base_url();
        let path = if let Some(p) = pair {
            endpoint.path_with_pair(p)
        } else {
            endpoint.path().to_string()
        };

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

        let response = self.http.get(&url, &HashMap::new()).await?;
        Ok(response)
    }

    /// POST request (authenticated)
    async fn post(
        &self,
        endpoint: BitstampEndpoint,
        pair: Option<&str>,
        body_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1, true).await;

        let base_url = BitstampUrls::base_url();
        let path = if let Some(p) = pair {
            endpoint.path_with_pair(p)
        } else {
            endpoint.path().to_string()
        };

        // Build form-encoded body
        let body = if body_params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = body_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            pairs.join("&")
        };

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("POST", &path, "", &body);

        let url = format!("{}{}", base_url, path);

        // Use reqwest directly for form-encoded POST
        let mut request = self.reqwest_client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded");

        // Add auth headers
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        // Set body
        if !body.is_empty() {
            request = request.body(body.clone());
        }

        let response = request.send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        let body_text = response.text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: body_text,
            });
        }

        let json: Value = serde_json::from_str(&body_text)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // POST WITH CUSTOM PATH (for currency-parameterized endpoints)
    // ═══════════════════════════════════════════════════════════════════════════

    /// POST request using a fully-formed custom path.
    ///
    /// Used for Bitstamp endpoints where the path depends on a currency code
    /// rather than a trading pair (e.g. `/api/v2/btc_address/`).
    async fn post_path(
        &self,
        path: &str,
        body_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait(1, true).await;

        let base_url = BitstampUrls::base_url();

        let body = if body_params.is_empty() {
            String::new()
        } else {
            let pairs: Vec<String> = body_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            pairs.join("&")
        };

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let headers = auth.sign_request("POST", path, "", &body);

        let url = format!("{}{}", base_url, path);

        let mut request = self.reqwest_client
            .post(&url)
            .header("Content-Type", "application/x-www-form-urlencoded");

        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        if !body.is_empty() {
            request = request.body(body.clone());
        }

        let response = request.send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();
        let body_text = response.text()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: body_text,
            });
        }

        let json: Value = serde_json::from_str(&body_text)
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (Bitstamp-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all tickers
    pub async fn get_all_tickers(&self) -> ExchangeResult<Vec<Ticker>> {
        // Bitstamp doesn't have a single endpoint for all tickers
        // Would need to fetch markets list and then ticker for each
        Ok(vec![])
    }

    /// Get markets information
    pub async fn get_markets(&self) -> ExchangeResult<Value> {
        self.get(BitstampEndpoint::Markets, None, HashMap::new()).await
    }

    /// Cancel all orders
    pub async fn cancel_all_orders(&self) -> ExchangeResult<Vec<String>> {
        let response = self.post(BitstampEndpoint::CancelAllOrders, None, HashMap::new()).await?;
        // Response is success/error, not a list of cancelled IDs
        BitstampParser::check_error(&response)?;
        Ok(vec![])
    }

    /// Get open positions (perpetual futures)
    pub async fn get_open_positions(&self) -> ExchangeResult<Value> {
        self.post(BitstampEndpoint::OpenPositions, None, HashMap::new()).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for BitstampConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitstamp
    }

    fn metrics(&self) -> ConnectorStats {
        let (http_requests, http_errors, last_latency_ms) = self.http.stats();
        let (rate_used, rate_max) = if let Ok(mut limiter) = self.limiter.lock() {
            limiter.primary_stats()
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
        false // Bitstamp doesn't have testnet via this API
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

    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        BITSTAMP_RATE_CAPS
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITSTAMP_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::snapshot("order_book",      100, 1000),
            WsBookChannel::delta("diff_order_book",    None, None),
        ];
        OrderbookCapabilities {
            ws_depths: &[],
            ws_default_depth: None,
            rest_max_depth: None,
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITSTAMP_CHANNELS,
            checksum: None,
            has_sequence: false,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &["0", "1", "2"],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for BitstampConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let pair = format_symbol(&symbol, account_type);
        let response = self.get(BitstampEndpoint::Ticker, Some(&pair), HashMap::new()).await?;
        let ticker = BitstampParser::parse_ticker(&response)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        _depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let pair = format_symbol(&symbol, account_type);
        let response = self.get(BitstampEndpoint::Orderbook, Some(&pair), HashMap::new()).await?;
        BitstampParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let pair = format_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("step".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(1000).to_string());
        }

        if let Some(et) = end_time {
            params.insert("end".to_string(), (et / 1000).to_string());
        }

        let response = self.get(BitstampEndpoint::Ohlc, Some(&pair), params).await?;
        BitstampParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let pair = format_symbol(&symbol, account_type);
        let response = self.get(BitstampEndpoint::Ticker, Some(&pair), HashMap::new()).await?;
        BitstampParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Use markets endpoint as ping (always available)
        let _ = self.get(BitstampEndpoint::Markets, None, HashMap::new()).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // GET /api/v2/trading-pairs-info/ returns detailed symbol info with name, url_symbol, etc.
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }
        let url = format!("{}/api/v2/trading-pairs-info/", BitstampUrls::base_url());
        let response = self.http.get(&url, &HashMap::new()).await?;
        let info = BitstampParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&info);
        Ok(info)
    }

    fn market_data_capabilities(&self, _account_type: AccountType) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            // MarketData::get_recent_trades is not overridden — default returns UnsupportedOperation.
            has_recent_trades: false,
            // Bitstamp OHLC `step` values: 60, 180, 300, 900, 1800, 3600, 7200, 14400, 21600,
            // 43200, 86400, 259200 — mapped in map_kline_interval().
            // No 8h, 1w, or 1M equivalent available.
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m",
                "1h", "2h", "4h", "6h", "12h", "1d", "3d",
            ],
            // Bitstamp OHLC endpoint accepts `limit` up to 1000.
            max_kline_limit: Some(1000),
            // live_trades channel emits Ticker events; diff_order_book emits OrderbookDelta.
            has_ws_ticker: true,
            has_ws_trades: true,
            has_ws_orderbook: true,
            // Bitstamp WebSocket has no candle/kline channel.
            has_ws_klines: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BitstampConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        match req.order_type {
            OrderType::Market => {
                let pair = format_symbol(&symbol, account_type);

                        let mut params = HashMap::new();
                        params.insert("amount".to_string(), self.precision.qty(&pair, quantity));

                        let endpoint = match side {
                            OrderSide::Buy => BitstampEndpoint::BuyMarket,
                            OrderSide::Sell => BitstampEndpoint::SellMarket,
                        };

                        let response = self.post(endpoint, Some(&pair), params).await?;
                        BitstampParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::Limit { price } => {
                let pair = format_symbol(&symbol, account_type);

                        let mut params = HashMap::new();
                        params.insert("amount".to_string(), self.precision.qty(&pair, quantity));
                        params.insert("price".to_string(), self.precision.price(&pair, price));

                        let endpoint = match side {
                            OrderSide::Buy => BitstampEndpoint::BuyLimit,
                            OrderSide::Sell => BitstampEndpoint::SellLimit,
                        };

                        let response = self.post(endpoint, Some(&pair), params).await?;
                        BitstampParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                // Bitstamp only supports sell-side stop-limit orders.
                // Buy-side stop-limit is not available via the REST API.
                if side != OrderSide::Sell {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Buy-side StopLimit not supported on Bitstamp — only sell-side".to_string()
                    ));
                }

                let pair = format_symbol(&symbol, account_type);
                let mut params = HashMap::new();
                params.insert("amount".to_string(), self.precision.qty(&pair, quantity));
                params.insert("stop_price".to_string(), self.precision.price(&pair, stop_price));
                params.insert("limit_price".to_string(), self.precision.price(&pair, limit_price));

                let response = self.post(BitstampEndpoint::SellStopLimit, Some(&pair), params).await?;
                BitstampParser::parse_order(&response).map(PlaceOrderResponse::Simple)
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // POST /api/v2/user_transactions/ returns trade executions
        let mut params = HashMap::new();
        params.insert("sort".to_string(), "desc".to_string());

        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        // Bitstamp user_transactions supports offset but not a direct date filter in v2
        let response = self.post(BitstampEndpoint::UserTransactions, None, params).await?;
        BitstampParser::parse_user_transactions(&response)
    }
async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let _symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let _account_type = req.account_type;

            let mut params = HashMap::new();
            params.insert("id".to_string(), order_id.to_string());

            let response = self.post(BitstampEndpoint::CancelOrder, None, params).await?;
            BitstampParser::parse_order(&response)
    
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let mut params = HashMap::new();
        params.insert("id".to_string(), order_id.to_string());

        let response = self.post(BitstampEndpoint::OrderStatus, None, params).await?;
        BitstampParser::parse_order(&response)
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Convert Option<&str> to Option<Symbol>
        let _symbol_str = _symbol;
        let _symbol: Option<crate::core::Symbol> = _symbol_str.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let response = self.post(BitstampEndpoint::OpenOrders, None, HashMap::new()).await?;
        BitstampParser::parse_orders(&response)

    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let mut params = HashMap::new();
        params.insert("sort".to_string(), "desc".to_string());

        if let Some(lim) = filter.limit {
            params.insert("limit".to_string(), lim.min(1000).to_string());
        }

        // Bitstamp user_transactions supports `offset` but not a direct time filter.
        // We fetch and filter by timestamp client-side when start_time is specified.

        let response = if let Some(ref symbol_str) = filter.symbol {
            // Symbol can be "BTC/USD" or "btcusd" — normalise to lowercase pair.
            let pair = if symbol_str.contains('/') {
                let parts: Vec<&str> = symbol_str.splitn(2, '/').collect();
                let base = parts[0].to_lowercase();
                let quote = parts.get(1).unwrap_or(&"usd").to_lowercase();
                format!("{}{}", base, quote)
            } else {
                symbol_str.to_lowercase()
            };
            let path = format!("/api/v2/user_transactions/{}/", pair);
            self.post_path(&path, params).await?
        } else {
            // No symbol — fetch all transactions
            self.post(BitstampEndpoint::UserTransactions, None, params).await?
        };

        BitstampParser::parse_user_trades(&response, filter.symbol.as_deref(), filter.start_time, filter.end_time)
    }

    fn trading_capabilities(&self, _account_type: AccountType) -> TradingCapabilities {
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            // Bitstamp has no stop-market endpoint — only stop-limit (sell side).
            has_stop_market: false,
            // StopLimit is implemented for sell-side; buy-side returns UnsupportedOperation.
            // We advertise true because the StopLimit arm is handled (not a blanket reject).
            has_stop_limit: true,
            has_trailing_stop: false,
            has_bracket: false,
            has_oco: false,
            // AmendOrder trait is implemented via POST /api/v2/replace_order/.
            has_amend: true,
            // No batch order placement/cancellation endpoint on Bitstamp.
            has_batch: false,
            max_batch_size: None,
            // CancelAll trait is implemented via POST /api/v2/cancel_all_orders/.
            has_cancel_all: true,
            has_user_trades: true,
            has_order_history: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BitstampConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.post(BitstampEndpoint::Balance, None, HashMap::new()).await?;
        BitstampParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.post(BitstampEndpoint::Balance, None, HashMap::new()).await?;
        let balances = BitstampParser::parse_balance(&response)?;

        // Bitstamp doesn't have a separate account info endpoint
        // We'll construct from balances
        Ok(AccountInfo {
            account_type: _account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.5, // Bitstamp default maker fee
            taker_commission: 0.5, // Bitstamp default taker fee
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // POST /api/v2/fees/trading/ — returns per-pair fee info
        let response = self.post(BitstampEndpoint::TradingFees, None, HashMap::new()).await?;
        BitstampParser::parse_fee_rate(&response, symbol)
    }

    fn account_capabilities(&self, _account_type: AccountType) -> AccountCapabilities {
        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            // sub_account_transfer() exists as an inherent method but the AccountTransfers
            // trait is not implemented on this connector.
            has_transfers: false,
            // No SubAccounts trait implemented.
            has_sub_accounts: false,
            // CustodialFunds trait is implemented: deposit addresses + withdrawals.
            has_deposit_withdraw: true,
            // Spot-only exchange — no margin borrowing/repayment trait.
            has_margin: false,
            has_earn_staking: false,
            // Spot-only — no perpetual funding payments.
            has_funding_history: false,
            // AccountLedger trait is implemented via POST /api/v2/user_transactions/.
            has_ledger: true,
            has_convert: false,
            // Bitstamp is spot-only — no futures positions.
            has_positions: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CancelAll for BitstampConnector {
    async fn cancel_all_orders(
        &self,
        _scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        // Bitstamp only supports cancel-all globally (no per-symbol cancel-all via API)
        let response = self.post(BitstampEndpoint::CancelAllOrders, None, HashMap::new()).await?;
        BitstampParser::check_error(&response)?;

        Ok(CancelAllResponse {
            cancelled_count: 0, // Bitstamp returns true/false, not count
            failed_count: 0,
            details: vec![],
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for BitstampConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        // Bitstamp implements amend as an atomic cancel-and-replace via
        // POST /api/v2/replace_order/
        // Accepts: id (or orig_client_order_id), amount, price
        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "AmendOrder requires at least one of: price, quantity".to_string(),
            ));
        }

        let mut params = HashMap::new();
        params.insert("id".to_string(), req.order_id.clone());

        let symbol_str = format_symbol(&req.symbol, req.account_type);
        if let Some(new_price) = req.fields.price {
            params.insert("price".to_string(), self.precision.price(&symbol_str, new_price));
        }
        if let Some(new_qty) = req.fields.quantity {
            params.insert("amount".to_string(), self.precision.qty(&symbol_str, new_qty));
        }

        let response = self.post(BitstampEndpoint::ReplaceOrder, None, params).await?;
        BitstampParser::parse_order(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for BitstampConnector {
    /// Get deposit address for an asset.
    ///
    /// Bitstamp uses currency-specific endpoints:
    /// `POST /api/v2/{currency}_address/` — e.g. `/api/v2/btc_address/`
    async fn get_deposit_address(
        &self,
        asset: &str,
        _network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let currency = asset.to_lowercase();
        let path = format!("/api/v2/{}_address/", currency);

        let response = self.post_path(&path, HashMap::new()).await?;

        // Response: {"address": "...", "destination_tag": "..."} or {"address": "..."}
        let address = response.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing address in deposit address response".to_string()))?
            .to_string();

        let tag = response.get("destination_tag")
            .or_else(|| response.get("memo"))
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(DepositAddress {
            address,
            tag,
            network: None, // Bitstamp doesn't return network info here
            asset: asset.to_string(),
            created_at: None,
        })
    }

    /// Submit a withdrawal request.
    ///
    /// Bitstamp uses currency-specific endpoints:
    /// `POST /api/v2/{currency}_withdrawal/` — e.g. `/api/v2/btc_withdrawal/`
    /// Params: amount, address, destination_tag
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let currency = req.asset.to_lowercase();
        let path = format!("/api/v2/{}_withdrawal/", currency);

        let mut params = HashMap::new();
        params.insert("amount".to_string(), req.amount.to_string());
        params.insert("address".to_string(), req.address.clone());

        if let Some(tag) = &req.tag {
            params.insert("destination_tag".to_string(), tag.clone());
        }

        let response = self.post_path(&path, params).await?;

        // Response: {"id": 12345, ...} on success
        let withdraw_id = response.get("id")
            .and_then(|v| v.as_i64())
            .map(|id| id.to_string())
            .unwrap_or_else(|| "0".to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Get withdrawal history.
    ///
    /// Bitstamp does not provide a standalone deposit history endpoint.
    /// `POST /api/v2/withdrawal-requests/` — params: timedelta (seconds from now)
    ///
    /// Deposit queries return an empty vec since there is no deposit history endpoint.
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        // Deposits: not available as standalone — return empty
        if filter.record_type == FundsRecordType::Deposit {
            return Ok(vec![]);
        }

        // Withdrawals via /api/v2/withdrawal-requests/
        let mut params = HashMap::new();

        // timedelta: how many seconds back to look (from now)
        let timedelta_secs = if let Some(start) = filter.start_time {
            let now_ms = crate::core::timestamp_millis() as i64;
            ((now_ms - start) / 1000).max(0) as u64
        } else {
            86400 * 30 // default: 30 days
        };

        params.insert("timedelta".to_string(), timedelta_secs.to_string());

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self.post(BitstampEndpoint::WithdrawalRequests, None, params).await?;

        let records = if let Some(arr) = response.as_array() {
            arr.iter().filter_map(|item| {
                let obj = item.as_object()?;

                // Filter by asset if specified
                let currency = obj.get("currency")
                    .or_else(|| obj.get("asset"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_uppercase();

                if let Some(ref asset_filter) = filter.asset {
                    if !currency.eq_ignore_ascii_case(asset_filter) {
                        return None;
                    }
                }

                let id = obj.get("id")?.as_i64()?.to_string();
                let amount_str = obj.get("amount").and_then(|v| v.as_str()).unwrap_or("0");
                let amount = amount_str.parse::<f64>().unwrap_or(0.0);
                let address = obj.get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let timestamp = obj.get("datetime")
                    .or_else(|| obj.get("created_at"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);
                let status_code = obj.get("status").and_then(|v| v.as_i64()).unwrap_or(0);
                let status = match status_code {
                    0 => "Open",
                    1 => "InProcess",
                    2 => "Finished",
                    3 => "Canceled",
                    4 => "Failed",
                    _ => "Unknown",
                }.to_string();
                let tx_hash = obj.get("transaction_id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                Some(FundsRecord::Withdrawal {
                    id,
                    asset: currency,
                    amount,
                    fee: None,
                    address,
                    tag: None,
                    tx_hash,
                    network: None,
                    status,
                    timestamp,
                })
            }).collect()
        } else {
            vec![]
        };

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// C3 ADDITIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl BitstampConnector {
    /// Get order history for a specific trading pair.
    ///
    /// `POST /api/v2/order_history/{pair}/`
    /// Optional parameter: `offset` (default 0).
    pub async fn get_order_history_pair(
        &self,
        pair: &str,
        offset: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(o) = offset {
            params.insert("offset".to_string(), o.to_string());
        }
        self.post(BitstampEndpoint::OrderHistoryPair, Some(pair), params).await
    }

    /// Place an instant buy order (market order) for a trading pair.
    ///
    /// `POST /api/v2/buy/instant/{pair}/`
    /// Required parameter: `amount` (amount to buy in quote currency).
    pub async fn instant_buy(&self, pair: &str, amount: f64) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("amount".to_string(), amount.to_string());
        self.post(BitstampEndpoint::InstantBuy, Some(pair), params).await
    }

    /// Place an instant sell order (market order) for a trading pair.
    ///
    /// `POST /api/v2/sell/instant/{pair}/`
    /// Required parameter: `amount` (amount to sell in base currency).
    pub async fn instant_sell(&self, pair: &str, amount: f64) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("amount".to_string(), amount.to_string());
        self.post(BitstampEndpoint::InstantSell, Some(pair), params).await
    }

    /// Transfer funds between sub-accounts.
    ///
    /// `POST /api/v2/sub-account/transfer/`
    /// Required parameters: `amount`, `currency`, `subAccount`.
    pub async fn sub_account_transfer(
        &self,
        amount: f64,
        currency: &str,
        sub_account: &str,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("amount".to_string(), amount.to_string());
        params.insert("currency".to_string(), currency.to_string());
        params.insert("subAccount".to_string(), sub_account.to_string());
        self.post(BitstampEndpoint::SubAccountTransfer, None, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountLedger for BitstampConnector {
    /// Get account ledger from `POST /api/v2/user_transactions/`.
    ///
    /// Bitstamp returns all transaction types (deposits, withdrawals, trades,
    /// sub-account transfers) via this endpoint.
    ///
    /// Params: `offset`, `limit` (max 1000), `sort` (asc/desc).
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let mut params = HashMap::new();

        params.insert("offset".to_string(), "0".to_string());

        if let Some(limit) = filter.limit {
            // Bitstamp max is 1000
            params.insert("limit".to_string(), limit.min(1000).to_string());
        } else {
            params.insert("limit".to_string(), "100".to_string());
        }

        // Default to descending (newest first)
        params.insert("sort".to_string(), "desc".to_string());

        let response = self.post(
            BitstampEndpoint::UserTransactions,
            None,
            params,
        ).await?;

        BitstampParser::parse_ledger(&response)
    }
}

