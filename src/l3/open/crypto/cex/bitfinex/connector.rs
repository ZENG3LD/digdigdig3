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
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    UserTrade, UserTradeFilter,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
};
use crate::core::types::{MarketDataCapabilities, TradingCapabilities, AccountCapabilities};
use crate::core::{CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts};
use crate::core::traits::{FundingHistory, AccountLedger};
use crate::core::types::{
    ConnectorStats, CancelAllResponse, OrderResult, AmendRequest,
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawResponse, FundsRecord,
};
use crate::core::types::{
    WithdrawRequest, FundsHistoryFilter, FundsRecordType,
    SubAccountOperation, SubAccountResult, SubAccount,
};
use crate::core::types::{
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerEntryType, LedgerFilter,
    RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits,
};
use crate::core::utils::{RuntimeLimiter, RateLimitMonitor, RateLimitPressure};
use crate::core::utils::PrecisionCache;

use super::endpoints::{BitfinexUrls, BitfinexEndpoint, format_symbol, build_candle_key};
use super::auth::BitfinexAuth;
use super::parser::BitfinexParser;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CAPABILITIES (static — embedded in binary, no allocation)
// ═══════════════════════════════════════════════════════════════════════════════

static BITFINEX_RATE_POOLS: &[RestLimitPool] = &[RestLimitPool {
    name: "default",
    max_budget: 90,
    window_seconds: 60,
    is_weight: false,
    has_server_headers: false,
    server_header: None,
    header_reports_used: false,
}];

static BITFINEX_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Simple,
    rest_pools: BITFINEX_RATE_POOLS,
    decaying: None,
    endpoint_weights: &[],
    ws: WsLimits {
        max_connections: None,
        max_subs_per_conn: Some(30),
        max_msg_per_sec: None,
        max_streams_per_conn: None,
    },
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitfinex connector
pub struct BitfinexConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods only)
    auth: Option<BitfinexAuth>,
    /// URLs (mainnet — Bitfinex has no separate testnet URLs)
    urls: BitfinexUrls,
    /// Paper-trading mode flag.
    /// Bitfinex has no dedicated testnet; paper trading uses prefixed symbols
    /// (e.g., tTESTBTC:TESTUSD) on the same mainnet endpoints.
    /// Stored here for future paper trading symbol routing support.
    testnet: bool,
    /// Runtime rate limiter (Simple model: 90 req/60s)
    limiter: Arc<Mutex<RuntimeLimiter>>,
    /// Pressure monitor — logs transitions, gates non-essential requests at >= 90%
    monitor: Arc<Mutex<RateLimitMonitor>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl BitfinexConnector {
    /// Create new connector
    ///
    /// Note: Bitfinex has no separate testnet URLs. When `testnet` is `true`
    /// the connector still connects to the same mainnet endpoints; paper trading
    /// requires using prefixed symbols like `tTESTBTC:TESTUSD`.
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = BitfinexUrls::MAINNET;
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(BitfinexAuth::new)
            .transpose()?;

        let limiter = Arc::new(Mutex::new(RuntimeLimiter::from_caps(&BITFINEX_RATE_CAPS)));
        let monitor = Arc::new(Mutex::new(RateLimitMonitor::new("Bitfinex")));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            limiter,
            monitor,
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
        endpoint: BitfinexEndpoint,
        path_params: &[(&str, &str)],
        query_params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }

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
        self.rate_limit_wait(1, true).await;

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

    /// Format symbol helper
    fn fmt_symbol(symbol: &Symbol, account_type: AccountType) -> String {
        if let Some(raw) = symbol.raw() {
            raw.to_string()
        } else {
            format_symbol(&symbol.base, &symbol.quote, account_type)
        }
    }

    /// Determine if account type is derivatives (margin/futures)
    fn is_derivatives(account_type: AccountType) -> bool {
        matches!(account_type, AccountType::Margin | AccountType::FuturesCross | AccountType::FuturesIsolated)
    }

    /// Get order type string prefix ("EXCHANGE " for spot, "" for margin/futures)
    fn order_type_prefix(account_type: AccountType) -> &'static str {
        if Self::is_derivatives(account_type) {
            ""
        } else {
            "EXCHANGE "
        }
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
        // Bitfinex has no separate testnet URLs; this flag enables paper trading symbol routing
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

    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        BITFINEX_RATE_CAPS
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
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

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
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

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
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);
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
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

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

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Use Bitfinex v1 symbols_details endpoint (returns array with pair info)
        // Note: v1 is still supported and returns more detail than v2 conf endpoints
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }
        let url = "https://api.bitfinex.com/v1/symbols_details";
        let response = self.http.get(url, &HashMap::new()).await?;
        let info = BitfinexParser::parse_exchange_info(&response, account_type)?;
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
            // No public recent-trades endpoint implemented in this connector.
            has_recent_trades: false,
            // Bitfinex candle timeframes: 1m 3m 5m 15m 30m 1h 2h 3h 4h 6h 8h 12h 1D 1W 14D 1M
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m",
                "1h", "2h", "3h", "4h", "6h", "8h", "12h",
                "1d", "1w", "2w", "1M",
            ],
            // Bitfinex accepts up to 10 000 candles per request (capped in get_klines with .min(10000)).
            max_kline_limit: Some(10000),
            // ticker, trades, book, candles channels all available in Bitfinex WS v2.
            has_ws_ticker: true,
            has_ws_trades: true,
            has_ws_orderbook: true,
            has_ws_klines: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for BitfinexConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);
        let prefix = Self::order_type_prefix(account_type);

        // Amount: positive=buy, negative=sell (apply qty precision to absolute value then re-sign)
        let qty_str = self.precision.qty(&formatted_symbol, quantity);
        let amount_str = match side {
            OrderSide::Buy => qty_str,
            OrderSide::Sell => format!("-{}", qty_str),
        };

        match req.order_type {
            OrderType::Market => {
                let body = json!({
                    "type": format!("{}MARKET", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let body = json!({
                    "type": format!("{}LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, price),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                // Bitfinex: EXCHANGE STOP (triggers market at stop_price)
                let body = json!({
                    "type": format!("{}STOP", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, stop_price),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Bitfinex: EXCHANGE STOP LIMIT
                let body = json!({
                    "type": format!("{}STOP LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, limit_price),
                    "price_aux_limit": self.precision.price(&formatted_symbol, stop_price),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::TrailingStop { callback_rate, activation_price: _ } => {
                // Bitfinex: EXCHANGE TRAILING STOP
                // trail_pct is callback_rate in percent
                let body = json!({
                    "type": format!("{}TRAILING STOP", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": callback_rate.to_string(), // trail distance as % string
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::PostOnly { price } => {
                // Bitfinex: EXCHANGE LIMIT with flags = 4096 (POST_ONLY)
                let body = json!({
                    "type": format!("{}LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, price),
                    "flags": 4096,
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                // Bitfinex: EXCHANGE IOC
                let price_val = price.unwrap_or(0.0);
                let body = json!({
                    "type": format!("{}IOC", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, price_val),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Fok { price } => {
                // Bitfinex: EXCHANGE FOK
                let body = json!({
                    "type": format!("{}FOK", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, price),
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Iceberg { price, display_quantity } => {
                // Bitfinex: EXCHANGE LIMIT with max_show parameter
                let body = json!({
                    "type": format!("{}LIMIT", prefix),
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "price": self.precision.price(&formatted_symbol, price),
                    "meta": {
                        "max_show": self.precision.qty(&formatted_symbol, display_quantity),
                    },
                    "flags": 64, // Hidden flag
                });
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::ReduceOnly { price } => {
                // Bitfinex: LIMIT with reduce-only flag (only valid for margin/futures)
                if !Self::is_derivatives(account_type) {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly not supported for Spot".to_string()
                    ));
                }
                let order_type_str = if price.is_some() { "LIMIT" } else { "MARKET" };
                let mut body = json!({
                    "type": order_type_str,
                    "symbol": formatted_symbol,
                    "amount": amount_str,
                    "flags": 1024, // REDUCE_ONLY flag
                });
                if let Some(p) = price {
                    body["price"] = json!(self.precision.price(&formatted_symbol, p));
                }
                let response = self.post(BitfinexEndpoint::SubmitOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response).map(PlaceOrderResponse::Simple)
            }

            OrderType::Oto { .. } => Err(ExchangeError::UnsupportedOperation(
                "Oto orders not supported on Bitfinex".into()
            )),
            OrderType::ConditionalPlan { .. } => Err(ExchangeError::UnsupportedOperation(
                "ConditionalPlan orders not supported on Bitfinex".into()
            )),
            OrderType::DcaRecurring { .. } => Err(ExchangeError::UnsupportedOperation(
                "DcaRecurring orders not supported on Bitfinex".into()
            )),
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let id = order_id.parse::<i64>()
                    .map_err(|_| ExchangeError::InvalidRequest("Invalid order ID".to_string()))?;

                let body = json!({ "id": id });
                let response = self.post(BitfinexEndpoint::CancelOrder, &[], body).await?;
                BitfinexParser::parse_submit_order(&response)
            }

            CancelScope::Batch { ref order_ids } => {
                // Bitfinex: POST /auth/w/order/cancel/multi with {"id": [...]}
                let ids: Vec<i64> = order_ids.iter()
                    .filter_map(|id| id.parse::<i64>().ok())
                    .collect();

                if ids.is_empty() {
                    return Err(ExchangeError::InvalidRequest("No valid order IDs".to_string()));
                }

                let body = json!({ "id": ids });
                let _response = self.post(BitfinexEndpoint::CancelMultipleOrders, &[], body).await?;

                // Return a placeholder — Bitfinex multi-cancel returns notifications, not a single order
                Ok(Order {
                    id: ids[0].to_string(),
                    client_order_id: None,
                    symbol: req.symbol.as_ref().map(|s| s.to_string()).unwrap_or_default(),
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
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported — use CancelAll trait for All/BySymbol", req.scope)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut body = json!({});

        if let Some(sym) = &filter.symbol {
            // sym is already a Symbol struct
            let formatted_symbol = Self::fmt_symbol(sym, _account_type);
            body["symbol"] = json!(formatted_symbol);
        }

        if let Some(start) = filter.start_time {
            body["start"] = json!(start);
        }
        if let Some(end) = filter.end_time {
            body["end"] = json!(end);
        }
        if let Some(limit) = filter.limit {
            body["limit"] = json!(limit.min(2500));
        }

        let response = self.post(BitfinexEndpoint::OrderHistory, &[], body).await?;
        BitfinexParser::parse_orders(&response)
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let formatted_symbol = Self::fmt_symbol(&symbol, account_type);

        let body = json!({ "symbol": formatted_symbol });

        let response = self.post(
            BitfinexEndpoint::ActiveOrdersBySymbol,
            &[("symbol", &formatted_symbol)],
            body,
        ).await?;

        let orders = BitfinexParser::parse_orders(&response)?;

        orders.into_iter()
            .find(|o| o.id == order_id)
            .ok_or_else(|| ExchangeError::Parse(format!("Order {} not found", order_id)))
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let symbol: Option<crate::core::Symbol> = symbol.map(|s| {
            let parts: Vec<&str> = s.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: s.to_string(), quote: String::new(), raw: Some(s.to_string()) }
            }
        });

        let response = if let Some(s) = symbol {
            let formatted_symbol = Self::fmt_symbol(&s, account_type);
            self.post(
                BitfinexEndpoint::ActiveOrdersBySymbol,
                &[("symbol", &formatted_symbol)],
                json!({}),
            ).await?
        } else {
            self.post(
                BitfinexEndpoint::ActiveOrders,
                &[],
                json!({}),
            ).await?
        };

        BitfinexParser::parse_orders(&response)
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        // Build request body — Bitfinex uses POST with JSON body for authenticated endpoints
        let mut body = json!({ "sort": -1 });

        if let Some(start) = filter.start_time {
            body["start"] = json!(start);
        }
        if let Some(end) = filter.end_time {
            body["end"] = json!(end);
        }
        if let Some(lim) = filter.limit {
            body["limit"] = json!(lim.min(250));
        }

        // Use symbol-scoped endpoint when symbol is provided
        let response = if let Some(sym_raw) = &filter.symbol {
            // Parse symbol string "BTC/USDT" or raw "tBTCUSD"
            let formatted = if sym_raw.starts_with('t') || sym_raw.starts_with('f') {
                // Already in Bitfinex format
                sym_raw.clone()
            } else {
                let parts: Vec<&str> = sym_raw.split('/').collect();
                if parts.len() == 2 {
                    let s = Symbol::new(parts[0], parts[1]);
                    Self::fmt_symbol(&s, account_type)
                } else {
                    format!("t{}", sym_raw.to_uppercase())
                }
            };
            self.post(
                BitfinexEndpoint::TradeHistoryBySymbol,
                &[("symbol", &formatted)],
                body,
            ).await?
        } else {
            self.post(BitfinexEndpoint::TradeHistory, &[], body).await?
        };

        BitfinexParser::parse_user_trades(&response)
    }

    fn trading_capabilities(&self, _account_type: AccountType) -> TradingCapabilities {
        // All order types (Market, Limit, Stop, StopLimit, TrailingStop, PostOnly, IOC, FOK,
        // Iceberg) work for both Spot and Derivatives. The only Derivatives-only order type is
        // ReduceOnly, but TradingCapabilities has no has_reduce_only field — the gate is
        // enforced at runtime in place_order(). Wire format differences (the "EXCHANGE " prefix
        // for Spot) are handled transparently by order_type_prefix(). No per-account branching.
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: true,  // "EXCHANGE STOP" (Spot) / "STOP" (Derivatives)
            has_stop_limit: true,   // "EXCHANGE STOP LIMIT" / "STOP LIMIT"
            has_trailing_stop: true, // "EXCHANGE TRAILING STOP" / "TRAILING STOP"
            // Bitfinex has no bracket (TP+SL combo) order type.
            has_bracket: false,
            // Bitfinex has no OCO order type.
            has_oco: false,
            has_amend: true,        // AmendOrder trait implemented via UpdateOrder endpoint
            has_batch: true,        // BatchOrders trait implemented via OrderMulti endpoint
            // Bitfinex OrderMulti accepts up to 75 operations per call.
            max_batch_size: Some(75),
            has_cancel_all: true,   // CancelAll trait implemented via CancelMultipleOrders
            has_user_trades: true,
            has_order_history: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for BitfinexConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let response = self.post(
            BitfinexEndpoint::Wallets,
            &[],
            json!({}),
        ).await?;

        BitfinexParser::parse_balances(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.1,  // Default Bitfinex fees
            taker_commission: 0.2,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Bitfinex: POST /auth/r/trades/hist returns trades with fee info
        // Use a recent trade to get actual fee rates, or return defaults from account summary
        let body = json!({ "limit": 1 });
        let response = self.post(BitfinexEndpoint::TradeHistory, &[], body).await?;

        // Try to extract fee from most recent trade
        if let Some(arr) = response.as_array() {
            if let Some(trade) = arr.first() {
                if let Some(trade_arr) = trade.as_array() {
                    // Trade array format: [ID, PAIR, MTS_CREATE, ORDER_ID, EXEC_AMOUNT, EXEC_PRICE, ORDER_TYPE, ORDER_PRICE, MAKER, FEE, FEE_CURRENCY]
                    if trade_arr.len() > 9 {
                        let fee = trade_arr[9].as_f64().unwrap_or(0.0).abs();
                        let amount = trade_arr[4].as_f64().unwrap_or(1.0).abs();
                        let price = trade_arr[5].as_f64().unwrap_or(1.0);
                        let rate = if amount * price > 0.0 { fee / (amount * price) } else { 0.002 };

                        return Ok(FeeInfo {
                            maker_rate: rate,
                            taker_rate: rate,
                            symbol: symbol.map(|s| s.to_string()),
                            tier: None,
                        });
                    }
                }
            }
        }

        // Fallback: default Bitfinex fees
        Ok(FeeInfo {
            maker_rate: 0.001,  // 0.1%
            taker_rate: 0.002,  // 0.2%
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }

    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);

        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            has_transfers: true,        // AccountTransfers trait implemented (wallet-to-wallet)
            has_sub_accounts: true,     // SubAccounts trait implemented (list + transfer)
            has_deposit_withdraw: true, // CustodialFunds trait implemented (deposit address + withdraw + movements)
            // Bitfinex margin borrowing is order-flag-based, no dedicated borrow/repay endpoint.
            has_margin: false,
            // No earn or staking product endpoints in this connector.
            has_earn_staking: false,
            // Funding payments (ledger category 28) are perpetual interest charges on open
            // derivative positions — not applicable to Spot accounts.
            has_funding_history: is_futures,
            has_ledger: true,           // AccountLedger trait works for all account types
            // No coin-to-coin conversion endpoint implemented.
            has_convert: false,
            // Positions trait implemented; applies to margin/futures account types.
            has_positions: is_futures,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for BitfinexConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let account_type = query.account_type;

        if account_type == AccountType::Spot {
            return Err(ExchangeError::UnsupportedOperation(
                "Positions not supported for Spot".to_string()
            ));
        }

        let response = self.post(
            BitfinexEndpoint::Positions,
            &[],
            json!({}),
        ).await?;

        BitfinexParser::parse_positions(&response)
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<crate::core::FundingRate> {
        match account_type {
            AccountType::Spot | AccountType::Margin
            | AccountType::Earn | AccountType::Lending
            | AccountType::Options | AccountType::Convert => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        // Bitfinex doesn't have a direct funding rate endpoint for perpetuals
        // Would need to implement via derivatives API or funding book
        Err(ExchangeError::UnsupportedOperation(
            "Funding rate endpoint not implemented for Bitfinex".to_string()
        ))
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { account_type, .. } => {
                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "Leverage not supported for Spot".to_string()
                    ));
                }
                // Bitfinex handles leverage via order flags, not a separate endpoint
                Err(ExchangeError::UnsupportedOperation(
                    "Set leverage not available - use order flags instead".to_string()
                ))
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
impl CancelAll for BitfinexConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match scope {
            CancelScope::All { symbol: None } => {
                // Cancel all orders across all symbols
                let body = json!({ "all": 1 });
                let _response = self.post(BitfinexEndpoint::CancelMultipleOrders, &[], body).await?;

                Ok(CancelAllResponse {
                    cancelled_count: 0, // Bitfinex doesn't return count
                    failed_count: 0,
                    details: vec![],
                })
            }

            CancelScope::All { symbol: Some(sym) } | CancelScope::BySymbol { symbol: sym } => {
                // Cancel all orders for a specific symbol
                let formatted_symbol = Self::fmt_symbol(&sym, _account_type);
                let body = json!({ "symbol": formatted_symbol });
                let _response = self.post(BitfinexEndpoint::CancelMultipleOrders, &[], body).await?;

                Ok(CancelAllResponse {
                    cancelled_count: 0,
                    failed_count: 0,
                    details: vec![],
                })
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported in cancel_all_orders", scope)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AmendOrder for BitfinexConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let id = req.order_id.parse::<i64>()
            .map_err(|_| ExchangeError::InvalidRequest("Invalid order ID".to_string()))?;

        let symbol = &req.symbol;
        let formatted_symbol = Self::fmt_symbol(symbol, req.account_type);

        let mut body = json!({ "id": id });

        if let Some(price) = req.fields.price {
            body["price"] = json!(self.precision.price(&formatted_symbol, price));
        }
        if let Some(qty) = req.fields.quantity {
            // For Bitfinex, amount sign determines buy/sell — preserve original sign
            body["amount"] = json!(self.precision.qty(&formatted_symbol, qty));
        }
        if let Some(stop_price) = req.fields.trigger_price {
            body["price_aux_limit"] = json!(self.precision.price(&formatted_symbol, stop_price));
        }

        let response = self.post(BitfinexEndpoint::UpdateOrder, &[], body).await?;
        BitfinexParser::parse_submit_order(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl BatchOrders for BitfinexConnector {
    /// Place multiple orders in a single batch request.
    ///
    /// Bitfinex endpoint: POST /v2/auth/w/order/multi
    /// Body: `{"ops": [["on", {...order_params}], ...]}`
    /// Max 75 operations per request.
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let ops: Vec<Value> = orders.iter().map(|req| {
            let account_type = req.account_type;
            let formatted_symbol = Self::fmt_symbol(&req.symbol, account_type);
            let prefix = Self::order_type_prefix(account_type);

            let qty_str = self.precision.qty(&formatted_symbol, req.quantity);
            let amount_str = match req.side {
                OrderSide::Buy => qty_str,
                OrderSide::Sell => format!("-{}", qty_str),
            };

            let (order_type_str, price, price_aux) = match &req.order_type {
                OrderType::Market => (format!("{}MARKET", prefix), None, None),
                OrderType::Limit { price } => (format!("{}LIMIT", prefix), Some(*price), None),
                OrderType::StopMarket { stop_price } => (format!("{}STOP", prefix), Some(*stop_price), None),
                OrderType::StopLimit { stop_price, limit_price } => (
                    format!("{}STOP LIMIT", prefix),
                    Some(*limit_price),
                    Some(*stop_price),
                ),
                OrderType::PostOnly { price } => (format!("{}LIMIT", prefix), Some(*price), None),
                OrderType::Ioc { price } => (format!("{}IOC", prefix), price.map(|p| p), None),
                OrderType::Fok { price } => (format!("{}FOK", prefix), Some(*price), None),
                _ => (format!("{}MARKET", prefix), None, None),
            };

            let mut order_obj = json!({
                "type": order_type_str,
                "symbol": formatted_symbol,
                "amount": amount_str,
            });

            if let Some(p) = price {
                order_obj["price"] = json!(self.precision.price(&formatted_symbol, p));
            }
            if let Some(aux) = price_aux {
                order_obj["price_aux_limit"] = json!(self.precision.price(&formatted_symbol, aux));
            }

            // PostOnly flag
            if matches!(req.order_type, OrderType::PostOnly { .. }) {
                order_obj["flags"] = json!(4096);
            }

            json!(["on", order_obj])
        }).collect();

        let body = json!({ "ops": ops });
        let response = self.post(BitfinexEndpoint::OrderMulti, &[], body).await?;

        // Bitfinex returns array of notification arrays
        // Each notification: [0, "on-req", null, null, [order_array], ...]
        // We parse what we can from the response
        let results = if let Some(arr) = response.as_array() {
            arr.iter().enumerate().map(|(i, item)| {
                // Try to extract order from notification
                let order_arr = item.as_array()
                    .and_then(|a| a.get(4))
                    .and_then(|v| v.as_array());

                if let Some(order_data) = order_arr {
                    if let Some(id_val) = order_data.first() {
                        if let Some(id) = id_val.as_i64() {
                            let order = orders.get(i);
                            return OrderResult {
                                order: Some(Order {
                                    id: id.to_string(),
                                    client_order_id: None,
                                    symbol: order.map(|o| o.symbol.to_string()).unwrap_or_default(),
                                    side: order.map(|o| o.side).unwrap_or(OrderSide::Buy),
                                    order_type: order.map(|o| o.order_type.clone()).unwrap_or(OrderType::Market),
                                    status: crate::core::OrderStatus::New,
                                    price: None,
                                    stop_price: None,
                                    quantity: order.map(|o| o.quantity).unwrap_or(0.0),
                                    filled_quantity: 0.0,
                                    average_price: None,
                                    commission: None,
                                    commission_asset: None,
                                    created_at: crate::core::timestamp_millis() as i64,
                                    updated_at: None,
                                    time_in_force: crate::core::TimeInForce::Gtc,
                                }),
                                client_order_id: None,
                                success: true,
                                error: None,
                                error_code: None,
                            };
                        }
                    }
                }

                OrderResult {
                    order: None,
                    client_order_id: None,
                    success: false,
                    error: Some("Failed to parse batch order response".to_string()),
                    error_code: None,
                }
            }).collect()
        } else {
            orders.iter().map(|_| OrderResult {
                order: None,
                client_order_id: None,
                success: false,
                error: Some("Unexpected response format".to_string()),
                error_code: None,
            }).collect()
        };

        Ok(results)
    }

    /// Cancel multiple orders in a single batch request.
    ///
    /// Bitfinex endpoint: POST /v2/auth/w/order/multi
    /// Body: `{"ops": [["oc", {"id": N}], ...]}`
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        let ops: Vec<Value> = order_ids.iter()
            .filter_map(|id| id.parse::<i64>().ok())
            .map(|id| json!(["oc", { "id": id }]))
            .collect();

        if ops.is_empty() {
            return Err(ExchangeError::InvalidRequest("No valid order IDs".to_string()));
        }

        let body = json!({ "ops": ops });
        let _response = self.post(BitfinexEndpoint::OrderMulti, &[], body).await?;

        // Return success results — Bitfinex returns notifications, not per-order status
        let results = order_ids.iter().map(|_id| OrderResult {
            order: None,
            client_order_id: None,
            success: true,
            error: None,
            error_code: None,
        }).collect();

        let _ = order_ids; // silence unused after move
        Ok(results)
    }

    /// Maximum batch place size (Bitfinex limit: 75 operations per request).
    fn max_batch_place_size(&self) -> usize {
        75
    }

    /// Maximum batch cancel size (Bitfinex limit: 75 operations per request).
    fn max_batch_cancel_size(&self) -> usize {
        75
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountTransfers for BitfinexConnector {
    /// Transfer between Bitfinex wallets (exchange/margin/funding).
    ///
    /// Endpoint: POST /v2/auth/w/transfer
    /// Body: from, to, currency, currency_to, amount
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        fn account_type_to_wallet(account_type: AccountType) -> &'static str {
            match account_type {
                AccountType::Spot => "exchange",
                AccountType::Margin => "margin",
                AccountType::FuturesCross | AccountType::FuturesIsolated => "funding",
                AccountType::Lending => "funding",
                AccountType::Earn | AccountType::Options | AccountType::Convert => "exchange",
            }
        }

        let from_wallet = account_type_to_wallet(req.from_account);
        let to_wallet = account_type_to_wallet(req.to_account);

        let body = json!({
            "from": from_wallet,
            "to": to_wallet,
            "currency": req.asset.to_uppercase(),
            "currency_to": req.asset.to_uppercase(),
            "amount": req.amount.to_string(),
        });

        let response = self.post(BitfinexEndpoint::Transfer, &[], body).await?;

        // Response: [1, "SUCCESS", null, "transfer", [...transfer_data]]
        // Or: [0, "ERROR", null, "transfer", "error message"]
        if let Some(arr) = response.as_array() {
            let mts_created = arr.get(4)
                .and_then(|v| v.as_array())
                .and_then(|inner| inner.first())
                .and_then(|v| v.as_i64());

            return Ok(TransferResponse {
                transfer_id: format!("{}", mts_created.unwrap_or(0)),
                status: "Successful".to_string(),
                asset: req.asset,
                amount: req.amount,
                timestamp: mts_created,
            });
        }

        Err(ExchangeError::Parse("Unexpected transfer response format".to_string()))
    }

    /// Transfer history is not available via a standard endpoint on Bitfinex.
    ///
    /// Returns an empty vec — use movements endpoint for deposit/withdrawal history instead.
    async fn get_transfer_history(
        &self,
        _filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        Ok(vec![])
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for BitfinexConnector {
    /// Get deposit address for an asset on a given network/method.
    ///
    /// Endpoint: POST /v2/auth/w/deposit/address
    /// Body: wallet (exchange), method (network/coin), op_renew (0/1)
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let method = network.unwrap_or(asset).to_lowercase();

        let body = json!({
            "wallet": "exchange",
            "method": method,
            "op_renew": 0,
        });

        let response = self.post(BitfinexEndpoint::DepositAddress, &[], body).await?;

        // Response: [MTS, TYPE, MESSAGE_ID, null, [nil, METHOD, CURRENCY_CODE, nil, nil, ADDRESS, ...]]
        if let Some(arr) = response.as_array() {
            if let Some(inner) = arr.get(4).and_then(|v| v.as_array()) {
                let address = inner.get(5)
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let tag = inner.get(6)
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                return Ok(DepositAddress {
                    address,
                    tag,
                    network: Some(method),
                    asset: asset.to_string(),
                    created_at: None,
                });
            }
        }

        Err(ExchangeError::Parse("Unexpected deposit address response format".to_string()))
    }

    /// Submit a withdrawal request.
    ///
    /// Endpoint: POST /v2/auth/w/withdraw
    /// Body: wallet, method, amount, address, payment_id (tag)
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let method = req.network
            .as_deref()
            .unwrap_or(&req.asset)
            .to_lowercase();

        let mut body = json!({
            "wallet": "exchange",
            "method": method,
            "amount": req.amount.to_string(),
            "address": req.address,
        });

        if let Some(tag) = &req.tag {
            body["payment_id"] = json!(tag);
        }

        let response = self.post(BitfinexEndpoint::Withdraw, &[], body).await?;

        // Response: [MTS, TYPE, MESSAGE_ID, null, [WITHDRAWAL_ID, ...]]
        if let Some(arr) = response.as_array() {
            let withdraw_id = arr.get(4)
                .and_then(|v| v.as_array())
                .and_then(|inner| inner.first())
                .and_then(|v| v.as_i64())
                .map(|id| id.to_string())
                .unwrap_or_else(|| "0".to_string());

            return Ok(WithdrawResponse {
                withdraw_id,
                status: "Pending".to_string(),
                tx_hash: None,
            });
        }

        Err(ExchangeError::Parse("Unexpected withdraw response format".to_string()))
    }

    /// Get deposit and/or withdrawal history via movements endpoint.
    ///
    /// Endpoint: POST /v2/auth/r/movements/{Symbol}/hist
    /// Movements cover both deposits (positive amount) and withdrawals (negative amount).
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let symbol = filter.asset.as_deref().unwrap_or("").to_uppercase();
        let symbol_path = if symbol.is_empty() { "".to_string() } else { symbol.clone() };

        let mut body = json!({});
        if let Some(start) = filter.start_time {
            body["start"] = json!(start);
        }
        if let Some(end) = filter.end_time {
            body["end"] = json!(end);
        }
        if let Some(limit) = filter.limit {
            body["limit"] = json!(limit.min(1000));
        }

        let response = self.post(
            BitfinexEndpoint::Movements,
            &[("symbol", &symbol_path)],
            body,
        ).await?;

        // Movements array: each element is:
        // [ID, CURRENCY, CURRENCY_NAME, nil, nil, MTS_STARTED, MTS_UPDATED, nil, nil,
        //  STATUS, nil, nil, AMOUNT, FEES, nil, DESTINATION_ADDRESS, nil, nil, nil, TRANSACTION_ID, ...]
        let records = if let Some(arr) = response.as_array() {
            arr.iter().filter_map(|item| {
                let m = item.as_array()?;
                let id = m.first()?.as_i64()?.to_string();
                let currency = m.get(1)?.as_str().unwrap_or("").to_string();
                let timestamp = m.get(5)?.as_i64().unwrap_or(0);
                let status = m.get(9)?.as_str().unwrap_or("Unknown").to_string();
                let amount = m.get(12)?.as_f64().unwrap_or(0.0);
                let _fee = m.get(13).and_then(|v| v.as_f64());
                let address = m.get(15).and_then(|v| v.as_str()).unwrap_or("").to_string();
                let tx_hash = m.get(20).and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());

                // Filter by asset if specified
                if let Some(ref asset_filter) = filter.asset {
                    if !currency.eq_ignore_ascii_case(asset_filter) {
                        return None;
                    }
                }

                if amount >= 0.0 {
                    // Deposit
                    if matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both) {
                        Some(FundsRecord::Deposit {
                            id,
                            asset: currency,
                            amount,
                            tx_hash,
                            network: None,
                            status,
                            timestamp,
                        })
                    } else {
                        None
                    }
                } else {
                    // Withdrawal (negative amount)
                    if matches!(filter.record_type, FundsRecordType::Withdrawal | FundsRecordType::Both) {
                        Some(FundsRecord::Withdrawal {
                            id,
                            asset: currency,
                            amount: amount.abs(),
                            fee: _fee.map(|f| f.abs()),
                            address,
                            tag: None,
                            tx_hash,
                            network: None,
                            status,
                            timestamp,
                        })
                    } else {
                        None
                    }
                }
            }).collect()
        } else {
            vec![]
        };

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB ACCOUNTS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl SubAccounts for BitfinexConnector {
    /// Perform sub-account operations (list, transfer; create/get_balance not supported).
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        match op {
            SubAccountOperation::List => {
                let response = self.post(BitfinexEndpoint::SubAccountList, &[], json!({})).await?;

                // Response is an array of sub-account objects
                let accounts = if let Some(arr) = response.as_array() {
                    arr.iter().filter_map(|item| {
                        let obj = item.as_object()?;
                        let id = obj.get("id")?.as_i64()?.to_string();
                        let name = obj.get("email")
                            .or_else(|| obj.get("name"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();
                        let status = obj.get("status")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Normal")
                            .to_string();
                        Some(SubAccount { id, name, status })
                    }).collect()
                } else {
                    vec![]
                };

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                // Bitfinex sub-account transfer: POST /v2/auth/w/sub_account/transfer
                // Body: sub_account_id, wallet_from, wallet_to, amount, currency
                // Bitfinex uses "exchange" wallet for both master→sub and sub→master transfers
                let (wallet_from, wallet_to) = ("exchange", "exchange");

                let body = json!({
                    "sub_account_id": sub_account_id.parse::<i64>().unwrap_or(0),
                    "wallet_from": wallet_from,
                    "wallet_to": wallet_to,
                    "amount": amount.to_string(),
                    "currency": asset.to_uppercase(),
                    "to_sub": to_sub,
                });

                let response = self.post(BitfinexEndpoint::SubAccountTransfer, &[], body).await?;

                let transaction_id = response.as_array()
                    .and_then(|arr| arr.get(4))
                    .and_then(|v| v.as_array())
                    .and_then(|inner| inner.first())
                    .and_then(|v| v.as_i64())
                    .map(|id| id.to_string());

                Ok(SubAccountResult {
                    id: Some(sub_account_id),
                    name: None,
                    accounts: vec![],
                    transaction_id,
                })
            }

            SubAccountOperation::Create { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Create sub-account not supported via Bitfinex REST API".to_string()
                ))
            }

            SubAccountOperation::GetBalance { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Get sub-account balance not supported via standard Bitfinex REST API".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for BitfinexConnector {
    /// Get historical funding payments for the account.
    ///
    /// Uses `POST /v2/auth/r/ledgers/{currency}/hist` with `{"category": 28}`.
    /// Category 28 = funding charges/payments on perpetual positions.
    ///
    /// `filter.symbol` is treated as the settlement currency (e.g. "UST", "BTC").
    /// When `None`, defaults to "UST" (Bitfinex perpetual settlement asset).
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        // Bitfinex uses currency-level ledger queries; derive currency from symbol or use UST
        let currency = filter.symbol
            .as_deref()
            .map(|s| {
                // Symbol may be like "tBTCF0:USTF0" — extract the settlement currency
                if let Some(idx) = s.rfind(':') {
                    // e.g. "USTF0" → strip "F0"
                    s[idx + 1..].trim_end_matches("F0").to_uppercase()
                } else {
                    s.to_uppercase()
                }
            })
            .unwrap_or_else(|| "UST".to_string());

        // Build POST body: category 28 = funding charges
        let mut body = json!({"category": 28});

        if let Some(start) = filter.start_time {
            body["start"] = json!(start);
        }
        if let Some(end) = filter.end_time {
            body["end"] = json!(end);
        }
        if let Some(limit) = filter.limit {
            body["limit"] = json!(limit.min(500));
        }

        let response = self.post(
            BitfinexEndpoint::LedgerHist,
            &[("currency", &currency)],
            body,
        ).await?;

        // Response: array of arrays [[ID, CURRENCY, null, MTS, null, AMOUNT, BALANCE, null, DESCRIPTION], ...]
        let entries = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for ledger response".to_string()))?;

        let payments = entries.iter().filter_map(|row| {
            let arr = row.as_array()?;

            let id = arr.first()?.as_i64()?.to_string();
            let asset = arr.get(1)?.as_str()?.to_string();
            let timestamp = arr.get(3)?.as_i64()?;
            let amount = arr.get(5)?.as_f64()?;
            let balance = arr.get(6).and_then(|v| v.as_f64());
            let description = arr.get(8)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Derive symbol from the description when possible (Bitfinex includes it)
            // Description format: "Margin funding payment on wallet margin for BTC/USD @ 0.001"
            // Use id as ref; payment amount is the actual funding payment
            let _ = (id, balance, description);

            Some(FundingPayment {
                // Bitfinex ledger doesn't carry per-entry symbol/instrument info directly
                symbol: currency.clone(),
                // Funding rate is not returned in ledger entries (only in separate funding data)
                funding_rate: 0.0,
                // Position size not available in ledger entries
                position_size: 0.0,
                payment: amount,
                asset,
                timestamp,
            })
        }).collect();

        Ok(payments)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountLedger for BitfinexConnector {
    /// Get account ledger entries.
    ///
    /// Uses `POST /v2/auth/r/ledgers/{currency}/hist`.
    /// When `filter.asset` is `None`, queries with currency "USD" as default.
    /// To fetch all currencies, callers should query per currency.
    ///
    /// Bitfinex ledger categories:
    /// - 1  = deposit
    /// - 2  = withdrawal
    /// - 4  = exchange
    /// - 5  = margin (trade)
    /// - 28 = funding (perpetual interest)
    /// - 68 = affiliates earning
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let currency = filter.asset
            .as_deref()
            .unwrap_or("USD")
            .to_uppercase();

        // Map entry_type filter to Bitfinex category
        let category: Option<i32> = filter.entry_type.as_ref().map(|t| match t {
            LedgerEntryType::Deposit   => 1,
            LedgerEntryType::Withdrawal => 2,
            LedgerEntryType::Trade     => 5,
            LedgerEntryType::Funding   => 28,
            LedgerEntryType::Fee       => 4,
            LedgerEntryType::Rebate    => 4,
            LedgerEntryType::Transfer  => 2,
            LedgerEntryType::Liquidation => 5,
            LedgerEntryType::Settlement  => 5,
            LedgerEntryType::Other(_)    => -1, // -1 = no category filter
        });

        let mut body = json!({});
        if let Some(cat) = category {
            if cat >= 0 {
                body["category"] = json!(cat);
            }
        }
        if let Some(start) = filter.start_time {
            body["start"] = json!(start);
        }
        if let Some(end) = filter.end_time {
            body["end"] = json!(end);
        }
        if let Some(limit) = filter.limit {
            body["limit"] = json!(limit.min(500));
        }

        let response = self.post(
            BitfinexEndpoint::LedgerHist,
            &[("currency", &currency)],
            body,
        ).await?;

        // Response: [[ID, CURRENCY, null, MTS, null, AMOUNT, BALANCE, null, DESCRIPTION], ...]
        let entries = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array for ledger response".to_string()))?;

        let ledger: Vec<LedgerEntry> = entries.iter().filter_map(|row| {
            let arr = row.as_array()?;

            let id = arr.first()?.as_i64()?.to_string();
            let asset = arr.get(1)?.as_str().unwrap_or(&currency).to_string();
            let timestamp = arr.get(3)?.as_i64()?;
            let amount = arr.get(5)?.as_f64()?;
            let balance = arr.get(6).and_then(|v| v.as_f64());
            let description = arr.get(8)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            // Classify the entry type from description keywords
            let entry_type = classify_bitfinex_ledger_entry(&description);

            Some(LedgerEntry {
                id,
                asset,
                amount,
                balance,
                entry_type,
                description,
                ref_id: None,
                timestamp,
            })
        }).collect();

        Ok(ledger)
    }
}

/// Classify a Bitfinex ledger entry type from its description string.
fn classify_bitfinex_ledger_entry(description: &str) -> LedgerEntryType {
    let lower = description.to_lowercase();
    if lower.contains("deposit") || lower.contains("crypto deposit") {
        LedgerEntryType::Deposit
    } else if lower.contains("withdraw") {
        LedgerEntryType::Withdrawal
    } else if lower.contains("transfer") {
        LedgerEntryType::Transfer
    } else if lower.contains("margin funding") || lower.contains("funding payment") {
        LedgerEntryType::Funding
    } else if lower.contains("trading fee") || lower.contains("taker fee") || lower.contains("maker fee") {
        LedgerEntryType::Fee
    } else if lower.contains("rebate") {
        LedgerEntryType::Rebate
    } else if lower.contains("liquidat") {
        LedgerEntryType::Liquidation
    } else if lower.contains("settlement") || lower.contains("settle") {
        LedgerEntryType::Settlement
    } else if lower.contains("trade") || lower.contains("exchange") || lower.contains("order") {
        LedgerEntryType::Trade
    } else {
        LedgerEntryType::Other(description.to_string())
    }
}
