//! # MEXC Connector
//!
//! Implementation of all core traits for MEXC Spot API.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - exchange identification
//! - `MarketData` - market data
//! - `Trading` - trading operations
//! - `Account` - account information
//!
//! ## Extended Methods
//! Additional MEXC-specific methods as struct methods.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use reqwest::header::HeaderMap;
use serde_json::{json, Value};

use crate::core::{
    HttpClient, Credentials, assemble_rest_url,
    ExchangeId, ExchangeType, AccountType,
    ExchangeError, ExchangeResult,
    Price, Kline, Ticker, OrderBook, PublicTrade,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    UserTrade, UserTradeFilter,
    SymbolInput,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, MarketDataPublic,
};
use crate::core::{CancelAll, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts};
use crate::core::types::{
    ConnectorStats, CancelAllResponse, OrderResult,
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord, FundsHistoryFilter, FundsRecordType,
    SubAccountOperation, SubAccountResult, SubAccount,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
    FundingRate,
    AggTrade,
};
use crate::core::utils::{RuntimeLimiter, RateLimitMonitor, RateLimitPressure};
use crate::core::types::{RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits, OrderbookCapabilities, WsBookChannel};

use super::endpoints::{MexcUrls, MexcEndpoint, map_kline_interval};
use super::auth::MexcAuth;
use super::parser::MexcParser;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CAPABILITIES (static — embedded in binary, no allocation)
// ═══════════════════════════════════════════════════════════════════════════════

static MEXC_POOLS: &[RestLimitPool] = &[RestLimitPool {
    name: "default",
    max_budget: 500,
    window_seconds: 10,
    is_weight: true,
    has_server_headers: true,
    server_header: Some("X-MBX-USED-WEIGHT"),
    header_reports_used: true,
}];

static MEXC_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Weight,
    rest_pools: MEXC_POOLS,
    decaying: None,
    endpoint_weights: &[],
    ws: WsLimits {
        max_connections: None,
        max_subs_per_conn: Some(30),
        max_msg_per_sec: Some(100),
        max_streams_per_conn: None,
    },
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// MEXC connector
pub struct MexcConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Option<MexcAuth>,
    /// REST base URL override for proxy / CORS routing on wasm32.
    /// When set, replaces the exchange's native base URL at every REST call site.
    rest_override: Option<String>,
    /// Runtime rate limiter (Weight model: 500 weight per 10 seconds)
    limiter: Arc<Mutex<RuntimeLimiter>>,
    /// Pressure monitor — gates non-essential requests at >= 90%
    monitor: Arc<Mutex<RateLimitMonitor>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: crate::core::utils::precision::PrecisionCache,
}

impl MexcConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>) -> ExchangeResult<Self> {
        Self::new_with_override(credentials, None).await
    }

    /// Create connector with optional REST base URL override.
    ///
    /// When `rest_override` is `Some(url)`, all REST requests use that URL as
    /// the base instead of the exchange's native endpoint. Intended for proxy
    /// and CORS routing on wasm32 (e.g. `ExchangeHub::set_rest_base_override`).
    pub async fn new_with_override(credentials: Option<Credentials>, rest_override: Option<String>) -> ExchangeResult<Self> {
        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let mut auth = credentials.as_ref().map(MexcAuth::new);

        // Sync time with server if we have auth
        if auth.is_some() {
            let base_url = MexcUrls::base_url();
            let url = format!("{}/api/v3/time", base_url);
            if let Ok(response) = http.get(&url, &HashMap::new()).await {
                if let Some(server_time_ms) = response.get("serverTime")
                    .and_then(|t| t.as_i64())
                {
                    if let Some(ref mut a) = auth {
                        a.sync_time(server_time_ms);
                    }
                }
            }
        }

        let limiter = Arc::new(Mutex::new(RuntimeLimiter::from_caps(&MEXC_RATE_CAPS)));
        let monitor = Arc::new(Mutex::new(RateLimitMonitor::new("MEXC")));

        Ok(Self {
            http,
            auth,
            rest_override,
            limiter,
            monitor,
            precision: crate::core::utils::precision::PrecisionCache::new(),
        })
    }

    /// Create connector only for public methods
    pub async fn public(rest_override: Option<String>) -> ExchangeResult<Self> {
        Self::new_with_override(None, rest_override).await
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

    /// Sync limiter from MEXC response headers.
    ///
    /// MEXC reports: `X-MEXC-USED-WEIGHT-1M` = weight used in the last minute.
    fn update_weight_from_headers(&self, headers: &HeaderMap) {
        let used = headers
            .get("x-mexc-used-weight-1m")
            .or_else(|| headers.get("X-MEXC-USED-WEIGHT-1M"))
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u32>().ok());
        if let Some(used) = used {
            if let Ok(mut limiter) = self.limiter.lock() {
                limiter.update_from_server("default", used);
            }
        }
    }

    /// GET request
    async fn get(
        &self,
        endpoint: MexcEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Market data = non-essential: drop at >= 90% utilization to preserve budget for trading
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }

        let real_base = if endpoint.is_futures() {
            MexcUrls::futures_base_url()
        } else {
            MexcUrls::base_url()
        };
        let path = endpoint.path();

        let (url, headers) = if endpoint.is_private() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

            let (headers, signed_params) = auth.sign_request(params);

            let query_parts: Vec<String> = signed_params.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect();
            let query_string = format!("?{}", query_parts.join("&"));

            let url = assemble_rest_url(self.rest_override.as_deref(), real_base, path, &query_string);
            (url, headers)
        } else {
            let query = if params.is_empty() {
                String::new()
            } else {
                let qs: Vec<String> = params.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect();
                format!("?{}", qs.join("&"))
            };

            let url = assemble_rest_url(self.rest_override.as_deref(), real_base, path, &query);
            (url, HashMap::new())
        };

        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::check_error(&response)?;
        Ok(response)
    }

    /// POST request
    async fn post(
        &self,
        endpoint: MexcEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Order placement = essential: always wait, never drop
        self.rate_limit_wait(1, true).await;

        let real_base = MexcUrls::base_url();
        let path = endpoint.path();

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let (headers, signed_params) = auth.sign_request(params);

        let query_parts: Vec<String> = signed_params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_string = format!("?{}", query_parts.join("&"));

        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, path, &query_string);

        let (response, resp_headers) = self.http.post_with_response_headers(&url, &json!({}), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::check_error(&response)?;
        Ok(response)
    }

    /// DELETE request
    async fn delete(
        &self,
        endpoint: MexcEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        // Order cancellation = essential: always wait, never drop
        self.rate_limit_wait(1, true).await;

        let real_base = MexcUrls::base_url();
        let path = endpoint.path();

        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let (headers, signed_params) = auth.sign_request(params);

        let query_parts: Vec<String> = signed_params.iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();
        let query_string = format!("?{}", query_parts.join("&"));

        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, path, &query_string);

        let (response, resp_headers) = self.http.delete_with_response_headers(&url, &HashMap::new(), &headers).await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::check_error(&response)?;
        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (MEXC-specific)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get raw exchange information as Value
    pub async fn get_exchange_info_raw(&self) -> ExchangeResult<Value> {
        self.get(MexcEndpoint::ExchangeInfo, HashMap::new()).await
    }

    /// Cancel all orders for a symbol
    pub async fn cancel_all_orders(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());

        let response = self.delete(MexcEndpoint::CancelAllOrders, params).await?;

        // Response is array of cancelled orders
        MexcParser::parse_orders(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for MexcConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::MEXC
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
        false // MEXC doesn't have testnet for spot
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
        MEXC_RATE_CAPS
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static MEXC_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("aggre.depth@10ms",  None,     Some(10)  ),
            WsBookChannel::delta("aggre.depth@100ms", None,     Some(100) ),
        ];
        OrderbookCapabilities {
            ws_depths: &[5, 10, 20],
            ws_default_depth: None,
            rest_max_depth: Some(5000),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[10, 100],
            default_speed_ms: None,
            ws_channels: MEXC_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: true,
            aggregation_levels: &[],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl MarketData for MexcConnector {
    async fn get_price(
        &self,
        symbol: SymbolInput<'_>,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());

                let response = self.get(MexcEndpoint::TickerPrice, params).await?;

                let price = response["price"].as_str()
                    .and_then(|s| s.parse::<f64>().ok())
                    .ok_or_else(|| ExchangeError::Parse("Invalid price".into()))?;

                Ok(price)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let ticker = self.get_ticker(SymbolInput::Raw(&symbol), account_type).await?;
                Ok(ticker.last_price)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn get_orderbook(
        &self,
        symbol: SymbolInput<'_>,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());

                if let Some(d) = depth {
                    params.insert("limit".to_string(), d.to_string());
                }

                let response = self.get(MexcEndpoint::Orderbook, params).await?;
                MexcParser::parse_orderbook(&response)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let real_base = MexcUrls::futures_base_url();
                let path = format!("/api/v1/contract/depth/{}", symbol);
                let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, "");

                if !self.rate_limit_wait(1, false).await {
                    return Err(ExchangeError::RateLimitExceeded {
                        retry_after: None,
                        message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
                    });
                }
                let response = self.http.get(&url, &HashMap::new()).await?;
                MexcParser::check_error(&response)?;

                let data = response.get("data")
                    .ok_or_else(|| ExchangeError::Parse("Missing data field in futures orderbook".into()))?;
                MexcParser::parse_orderbook_futures(data)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn get_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());
                params.insert("interval".to_string(), map_kline_interval(interval).to_string());

                if let Some(l) = limit {
                    params.insert("limit".to_string(), l.min(1000).to_string());
                }

                if let Some(et) = end_time {
                    let interval_ms = interval_to_ms(interval);
                    let count = limit.unwrap_or(1000) as i64;
                    let st = et - count * interval_ms;
                    params.insert("startTime".to_string(), st.to_string());
                    params.insert("endTime".to_string(), et.to_string());
                }

                let response = self.get(MexcEndpoint::Klines, params).await?;
                MexcParser::parse_klines(&response)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let real_base = MexcUrls::futures_base_url();
                let path = format!("/api/v1/contract/kline/{}", symbol);

                let futures_interval = match interval {
                    "1m" => "Min1",
                    "5m" => "Min5",
                    "15m" => "Min15",
                    "30m" => "Min30",
                    "1h" => "Min60",
                    "4h" => "Hour4",
                    "8h" => "Hour8",
                    "1d" => "Day1",
                    "1w" => "Week1",
                    "1M" => "Month1",
                    _ => "Min60",
                };

                let mut params = HashMap::new();
                params.insert("interval".to_string(), futures_interval.to_string());

                if let Some(et) = end_time {
                    params.insert("endTime".to_string(), et.to_string());
                }

                let query_str = params.iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join("&");
                let query = format!("?{}", query_str);

                let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, &query);

                if !self.rate_limit_wait(1, false).await {
                    return Err(ExchangeError::RateLimitExceeded {
                        retry_after: None,
                        message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
                    });
                }
                let response = self.http.get(&url, &HashMap::new()).await?;
                MexcParser::check_error(&response)?;

                let klines_data = response.get("data")
                    .ok_or_else(|| ExchangeError::Parse("Missing data field in futures klines".into()))?;
                MexcParser::parse_klines_futures(klines_data)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn get_ticker(
        &self,
        symbol: SymbolInput<'_>,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());

                let response = self.get(MexcEndpoint::Ticker24hr, params).await?;
                MexcParser::parse_ticker(&response)
            },
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let response = self.get(MexcEndpoint::FuturesTicker, HashMap::new()).await?;

                let data_array = response.get("data")
                    .or_else(|| response.as_array().map(|_| &response))
                    .ok_or_else(|| ExchangeError::Parse("Invalid futures ticker response".into()))?;

                let ticker_data = if let Some(arr) = data_array.as_array() {
                    arr.iter()
                        .find(|t| t["symbol"].as_str() == Some(&*symbol))
                        .ok_or_else(|| ExchangeError::Parse(format!("Symbol {} not found", symbol)))?
                } else {
                    data_array
                };

                MexcParser::parse_ticker_futures(ticker_data)
            }
            AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => {
                Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} account type not supported on MEXC", account_type)
                ))
            }
        }
    }

    async fn ping(&self) -> ExchangeResult<()> {
        let _ = self.get(MexcEndpoint::Ping, HashMap::new()).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<crate::core::types::SymbolInfo>> {
        let response = self.get(MexcEndpoint::ExchangeInfo, HashMap::new()).await?;
        let symbols = MexcParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&symbols);
        Ok(symbols)
    }

    fn market_data_capabilities(&self, account_type: AccountType) -> MarketDataCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        if is_futures {
            MarketDataCapabilities {
                has_ping: true,
                has_price: true,
                has_ticker: true,
                has_orderbook: true,
                has_klines: true,
                // get_exchange_info parses spot symbols only; futures uses /api/v1/contract/detail
                has_exchange_info: false,
                // GET /api/v1/contract/deals/{symbol} is implemented via get_recent_trades
                has_recent_trades: true,
                // Futures kline intervals map to Min1/Min5/.../Month1
                supported_intervals: &["1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "1w", "1M"],
                // Futures /api/v1/contract/kline does not accept a limit param
                max_kline_limit: None,
                // WebSocket: kline channel supported
                has_ws_klines: true,
                // WebSocket: aggre.deals channel supported
                has_ws_trades: true,
                // WebSocket: aggre.depth channel supported
                has_ws_orderbook: true,
                // WebSocket: miniTicker channel supported
                has_ws_ticker: true,
            }
        } else {
            MarketDataCapabilities {
                has_ping: true,
                has_price: true,
                has_ticker: true,
                has_orderbook: true,
                has_klines: true,
                has_exchange_info: true,
                // get_recent_trades is implemented via the MarketData trait
                has_recent_trades: true,
                // MEXC spot intervals: 1m/5m/15m/30m are supported; 1h is mapped to "60m" internally
                supported_intervals: &["1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d", "1w", "1M"],
                max_kline_limit: Some(1000),
                // WebSocket: kline channel supported
                has_ws_klines: true,
                // WebSocket: aggre.deals channel supported
                has_ws_trades: true,
                // WebSocket: aggre.depth channel supported
                has_ws_orderbook: true,
                // WebSocket: miniTicker channel supported
                has_ws_ticker: true,
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Trading for MexcConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = &req.symbol;
        let side = req.side;
        let quantity = req.quantity;
        let _account_type = req.account_type;
        let client_order_id = format!("cc_{}", crate::core::timestamp_millis());
        let symbol_str = symbol.raw()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase()));
        let qty_str = self.precision.qty(&symbol_str, quantity);

        let side_str = match side {
            OrderSide::Buy => "BUY",
            OrderSide::Sell => "SELL",
        };

        match req.order_type {
            OrderType::Market => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "MARKET".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: Some(symbol.to_string()),
                    side,
                    order_type: OrderType::Market,
                    status: crate::core::OrderStatus::New,
                    price: None,
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::Limit { price } => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: Some(symbol.to_string()),
                    side,
                    order_type: OrderType::Limit { price: 0.0 },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::PostOnly { price } => {
                // MEXC: LIMIT_MAKER (post-only limit order)
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT_MAKER".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: Some(symbol.to_string()),
                    side,
                    order_type: OrderType::PostOnly { price },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Gtc,
                }))
            }

            OrderType::Ioc { price } => {
                // MEXC: LIMIT with timeInForce=IOC
                let price_val = price.unwrap_or(0.0);
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("timeInForce".to_string(), "IOC".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price_val));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: Some(symbol.to_string()),
                    side,
                    order_type: OrderType::Ioc { price },
                    status: crate::core::OrderStatus::New,
                    price,
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Ioc,
                }))
            }

            OrderType::Fok { price } => {
                // MEXC: LIMIT with timeInForce=FOK
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str.clone());
                params.insert("side".to_string(), side_str.to_string());
                params.insert("type".to_string(), "LIMIT".to_string());
                params.insert("timeInForce".to_string(), "FOK".to_string());
                params.insert("quantity".to_string(), qty_str.clone());
                params.insert("price".to_string(), self.precision.price(&symbol_str, price));
                params.insert("newClientOrderId".to_string(), client_order_id.clone());

                let response = self.post(MexcEndpoint::PlaceOrder, params).await?;

                let order_id = response["orderId"].as_str()
                    .ok_or_else(|| ExchangeError::Parse("Missing orderId".into()))?
                    .to_string();

                Ok(PlaceOrderResponse::Simple(Order {
                    id: order_id,
                    client_order_id: Some(client_order_id),
                    symbol: Some(symbol.to_string()),
                    side,
                    order_type: OrderType::Fok { price },
                    status: crate::core::OrderStatus::New,
                    price: Some(price),
                    stop_price: None,
                    quantity,
                    filled_quantity: 0.0,
                    average_price: None,
                    commission: None,
                    commission_asset: None,
                    created_at: crate::core::timestamp_millis() as i64,
                    updated_at: None,
                    time_in_force: crate::core::TimeInForce::Fok,
                }))
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?;
                let symbol_str = symbol.raw()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase()));

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol_str);
                params.insert("orderId".to_string(), order_id.to_string());

                let response = self.delete(MexcEndpoint::CancelOrder, params).await?;
                MexcParser::parse_order(&response)
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported — use CancelAll trait", req.scope)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // MEXC: GET /api/v3/allOrders — requires symbol
        let symbol = filter.symbol
            .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for order history on MEXC".to_string()))?;
        let symbol_str = symbol.raw()
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase()));

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str);

        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self.get(MexcEndpoint::AllOrders, params).await?;
        MexcParser::parse_orders(&response)
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("orderId".to_string(), order_id.to_string());

        let response = self.get(MexcEndpoint::QueryOrder, params).await?;
        MexcParser::parse_order(&response)
    }

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("symbol".to_string(), s.to_string());
        }

        let response = self.get(MexcEndpoint::OpenOrders, params).await?;
        MexcParser::parse_orders(&response)
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        // MEXC GET /api/v3/myTrades — symbol is required
        let symbol_str = filter.symbol
            .ok_or_else(|| ExchangeError::InvalidRequest(
                "Symbol required for get_user_trades on MEXC".to_string()
            ))?;

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol_str);

        if let Some(oid) = filter.order_id {
            params.insert("orderId".to_string(), oid);
        }
        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(1000).to_string());
        }

        let response = self.get(MexcEndpoint::MyTrades, params).await?;
        MexcParser::parse_user_trades(&response)
    }

    fn trading_capabilities(&self, account_type: AccountType) -> TradingCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        if is_futures {
            TradingCapabilities {
                has_market_order: true,
                has_limit_order: true,
                // Futures contract API supports STOP order type
                has_stop_market: true,
                has_stop_limit: true,
                has_trailing_stop: false,
                has_bracket: false,
                has_oco: false,
                // No amend/modify endpoint on MEXC futures
                has_amend: false,
                // POST /api/v3/batchOrders is a spot-only endpoint
                has_batch: false,
                max_batch_size: None,
                // DELETE /api/v3/openOrders is a spot-only endpoint; futures cancel-all not implemented
                has_cancel_all: false,
                // GET /api/v3/myTrades is spot-only; no futures trade history endpoint implemented
                has_user_trades: false,
                // GET /api/v3/allOrders is spot-only; no futures order history endpoint implemented
                has_order_history: false,
            }
        } else {
            TradingCapabilities {
                has_market_order: true,
                has_limit_order: true,
                // MEXC spot does not support stop-market or stop-limit order types
                has_stop_market: false,
                has_stop_limit: false,
                has_trailing_stop: false,
                has_bracket: false,
                // MEXC spot does not have an OCO endpoint
                has_oco: false,
                // No amend/modify order endpoint on MEXC spot
                has_amend: false,
                // BatchOrders trait is implemented: POST /api/v3/batchOrders (max 20)
                has_batch: true,
                max_batch_size: Some(20),
                // CancelAll trait is implemented: DELETE /api/v3/openOrders
                has_cancel_all: true,
                // get_user_trades is implemented via GET /api/v3/myTrades
                has_user_trades: true,
                // get_order_history is implemented via GET /api/v3/allOrders
                has_order_history: true,
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Account for MexcConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let _account_type = query.account_type;

        let response = self.get(MexcEndpoint::Account, HashMap::new()).await?;
        MexcParser::parse_balance(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let response = self.get(MexcEndpoint::Account, HashMap::new()).await?;

        let balances = MexcParser::parse_balance(&response)?;

        let can_trade = response.get("canTrade")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let can_withdraw = response.get("canWithdraw")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let can_deposit = response.get("canDeposit")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);

        let maker_commission = response.get("makerCommission")
            .and_then(|v| v.as_i64())
            .map(|c| c as f64 / 10000.0)
            .unwrap_or(0.002);

        let taker_commission = response.get("takerCommission")
            .and_then(|v| v.as_i64())
            .map(|c| c as f64 / 10000.0)
            .unwrap_or(0.002);

        Ok(AccountInfo {
            account_type,
            can_trade,
            can_withdraw,
            can_deposit,
            maker_commission,
            taker_commission,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // MEXC: GET /api/v3/tradeFee?symbol=BTCUSDT
        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            params.insert("symbol".to_string(), sym.to_uppercase().replace('/', ""));
        }

        let response = self.get(MexcEndpoint::TradeFee, params).await?;

        // Response: [{"symbol": "BTCUSDT", "makerCommission": "0.002", "takerCommission": "0.002"}]
        let fee_data = if let Some(arr) = response.as_array() {
            arr.first().cloned()
        } else {
            Some(response.clone())
        };

        let fee_data = fee_data
            .ok_or_else(|| ExchangeError::Parse("No fee data".to_string()))?;

        let maker_rate = fee_data.get("makerCommission")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.002);

        let taker_rate = fee_data.get("takerCommission")
            .and_then(|v| v.as_str().and_then(|s| s.parse::<f64>().ok())
                .or_else(|| v.as_f64()))
            .unwrap_or(0.002);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }

    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        if is_futures {
            AccountCapabilities {
                // Futures wallet balance requires a separate futures account endpoint (not implemented)
                has_balances: false,
                // GET /api/v3/account is spot-only; no futures account info endpoint implemented
                has_account_info: false,
                // GET /api/v3/tradeFee is spot-only; futures fees differ
                has_fees: false,
                // Capital transfers use spot base URL and work for all account types
                has_transfers: true,
                // Sub-accounts are a spot/master-level concept, not per-futures
                has_sub_accounts: false,
                // Deposits/withdrawals always go through the spot wallet, not futures directly
                has_deposit_withdraw: false,
                has_margin: false,
                has_earn_staking: false,
                // Futures perpetual contracts have funding payments; endpoint not yet implemented
                has_funding_history: false,
                has_ledger: false,
                has_convert: false,
                // Positions trait is not implemented for MEXC
                has_positions: false,
            }
        } else {
            AccountCapabilities {
                // GET /api/v3/account returns balances
                has_balances: true,
                // GET /api/v3/account returns canTrade/canWithdraw/canDeposit
                has_account_info: true,
                // GET /api/v3/tradeFee is implemented
                has_fees: true,
                // AccountTransfers trait is implemented: POST/GET /api/v3/capital/transfer
                has_transfers: true,
                // SubAccounts trait is implemented: create/list/transfer/getBalance
                has_sub_accounts: true,
                // CustodialFunds trait is implemented: deposit address, withdraw, deposit/withdraw history
                has_deposit_withdraw: true,
                // No margin borrow/repay endpoints implemented
                has_margin: false,
                // No earn or staking endpoints implemented
                has_earn_staking: false,
                // No funding payment history for spot
                has_funding_history: false,
                // No full account ledger/transaction log endpoint implemented
                has_ledger: false,
                // No coin-to-coin convert endpoint implemented
                has_convert: false,
                // Spot-only account type — no positions
                has_positions: false,
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl CancelAll for MexcConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match scope {
            CancelScope::All { symbol: Some(sym) } | CancelScope::BySymbol { symbol: sym } => {
                // MEXC requires symbol for cancel all
                let sym_str = sym.raw()
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("{}{}", sym.base.to_uppercase(), sym.quote.to_uppercase()));
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), sym_str);

                let response = self.delete(MexcEndpoint::CancelAllOrders, params).await?;

                // Response is array of cancelled orders
                let cancelled = if let Some(arr) = response.as_array() {
                    arr.len() as u32
                } else {
                    0
                };

                Ok(CancelAllResponse {
                    cancelled_count: cancelled,
                    failed_count: 0,
                    details: vec![],
                })
            }

            CancelScope::All { symbol: None } => {
                Err(ExchangeError::InvalidRequest(
                    "MEXC requires a symbol to cancel all orders — use BySymbol scope".to_string()
                ))
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} not supported in cancel_all_orders", scope)
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl BatchOrders for MexcConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // MEXC: POST /api/v3/batchOrders — max 20 orders
        // Build batch order array
        let batch_orders: Vec<Value> = orders.iter().map(|req| {
            let o_sym = req.symbol.raw()
                .map(|s| s.to_string())
                .unwrap_or_else(|| format!("{}{}", req.symbol.base.to_uppercase(), req.symbol.quote.to_uppercase()));
            let side_str = match req.side {
                OrderSide::Buy => "BUY",
                OrderSide::Sell => "SELL",
            };
            let (order_type, price) = match &req.order_type {
                OrderType::Market => ("MARKET".to_string(), None),
                OrderType::Limit { price } => ("LIMIT".to_string(), Some(*price)),
                OrderType::PostOnly { price } => ("LIMIT_MAKER".to_string(), Some(*price)),
                _ => ("LIMIT".to_string(), None),
            };

            let mut order_obj = json!({
                "symbol": o_sym,
                "side": side_str,
                "type": order_type,
                "quantity": self.precision.qty(&o_sym, req.quantity),
            });

            if let Some(p) = price {
                order_obj["price"] = json!(self.precision.price(&o_sym, p));
            }

            order_obj
        }).collect();

        // MEXC batch orders use JSON body
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;

        let params = HashMap::new();
        let (headers, _) = auth.sign_request(params);

        let real_base = MexcUrls::base_url();
        let path = MexcEndpoint::BatchOrders.path();
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, path, "");

        self.rate_limit_wait(1, true).await;
        let body = json!({ "batchOrders": batch_orders });
        let (response, _) = self.http.post_with_response_headers(&url, &body, &headers).await?;
        MexcParser::check_error(&response)?;

        // Parse response — array of order results
        let results = if let Some(arr) = response.as_array() {
            arr.iter().map(|item| {
                let success = item.get("orderId").is_some();
                let order_id = item.get("orderId")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());

                OrderResult {
                    order: order_id.map(|id| Order {
                        id,
                        client_order_id: None,
                        symbol: None,
                        side: OrderSide::Buy,
                        order_type: OrderType::Market,
                        status: crate::core::OrderStatus::New,
                        price: None,
                        stop_price: None,
                        quantity: 0.0,
                        filled_quantity: 0.0,
                        average_price: None,
                        commission: None,
                        commission_asset: None,
                        created_at: 0,
                        updated_at: None,
                        time_in_force: crate::core::TimeInForce::Gtc,
                    }),
                    client_order_id: None,
                    success,
                    error: if success { None } else {
                        item.get("msg").and_then(|v| v.as_str()).map(|s| s.to_string())
                    },
                    error_code: None,
                }
            }).collect()
        } else {
            vec![]
        };

        Ok(results)
    }

    async fn cancel_orders_batch(
        &self,
        _order_ids: Vec<String>,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        // MEXC doesn't have a true batch cancel — cancel one by one
        Err(ExchangeError::UnsupportedOperation(
            "MEXC does not support native batch cancel — use CancelAll for symbol-level cancel".to_string()
        ))
    }

    fn max_batch_place_size(&self) -> usize {
        20
    }

    fn max_batch_cancel_size(&self) -> usize {
        0 // Not supported
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl AccountTransfers for MexcConnector {
    /// Transfer between Spot, Margin, and Futures accounts.
    ///
    /// POST /api/v3/capital/transfer
    /// Params: asset, amount, fromAccountType (SPOT/FUTURES/MARGIN), toAccountType
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        let from_type = account_type_to_mexc_str(req.from_account);
        let to_type = account_type_to_mexc_str(req.to_account);

        let mut params = HashMap::new();
        params.insert("asset".to_string(), req.asset.clone());
        params.insert("amount".to_string(), req.amount.to_string());
        params.insert("fromAccountType".to_string(), from_type.to_string());
        params.insert("toAccountType".to_string(), to_type.to_string());

        let response = self.post(MexcEndpoint::Transfer, params).await?;

        let tran_id = response["tranId"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| response["tranId"].as_i64().map(|n| n.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(TransferResponse {
            transfer_id: tran_id,
            status: "Successful".to_string(),
            asset: req.asset,
            amount: req.amount,
            timestamp: Some(crate::core::timestamp_millis() as i64),
        })
    }

    /// Get internal transfer history.
    ///
    /// GET /api/v3/capital/transfer
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        let mut params = HashMap::new();

        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        let response = self.get(MexcEndpoint::TransferHistory, params).await?;

        let rows = response.get("rows")
            .and_then(|v| v.as_array())
            .or_else(|| response.as_array())
            .cloned()
            .unwrap_or_default();

        let records = rows.iter().map(|item| {
            let tran_id = item["tranId"]
                .as_str()
                .map(|s| s.to_string())
                .or_else(|| item["tranId"].as_i64().map(|n| n.to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            let asset = item["asset"].as_str().unwrap_or("").to_string();
            let amount = item["amount"]
                .as_str()
                .and_then(|s| s.parse::<f64>().ok())
                .or_else(|| item["amount"].as_f64())
                .unwrap_or(0.0);
            let status = item["status"].as_str().unwrap_or("Unknown").to_string();
            let timestamp = item["timestamp"].as_i64()
                .or_else(|| item["createTime"].as_i64());

            TransferResponse {
                transfer_id: tran_id,
                status,
                asset,
                amount,
                timestamp,
            }
        }).collect();

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl CustodialFunds for MexcConnector {
    /// Get deposit address for an asset.
    ///
    /// GET /api/v3/capital/deposit/address
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let mut params = HashMap::new();
        params.insert("coin".to_string(), asset.to_uppercase());

        if let Some(net) = network {
            params.insert("network".to_string(), net.to_string());
        }

        let response = self.get(MexcEndpoint::DepositAddress, params).await?;

        let address = response["address"]
            .as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing deposit address".into()))?
            .to_string();

        let tag = response["tag"]
            .as_str()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        let net = response["network"]
            .as_str()
            .or(network)
            .map(|s| s.to_string());

        Ok(DepositAddress {
            address,
            tag,
            network: net,
            asset: asset.to_uppercase(),
            created_at: None,
        })
    }

    /// Submit a withdrawal request.
    ///
    /// POST /api/v3/capital/withdraw
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let mut params = HashMap::new();
        params.insert("coin".to_string(), req.asset.clone());
        params.insert("address".to_string(), req.address.clone());
        params.insert("amount".to_string(), req.amount.to_string());

        if let Some(net) = &req.network {
            params.insert("network".to_string(), net.clone());
        }
        if let Some(memo) = &req.tag {
            params.insert("memo".to_string(), memo.clone());
        }

        let response = self.post(MexcEndpoint::Withdraw, params).await?;

        let withdraw_id = response["id"]
            .as_str()
            .map(|s| s.to_string())
            .or_else(|| response["id"].as_i64().map(|n| n.to_string()))
            .unwrap_or_else(|| "unknown".to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status: "Pending".to_string(),
            tx_hash: None,
        })
    }

    /// Get deposit and/or withdrawal history.
    ///
    /// GET /api/v3/capital/deposit/hisrec  (deposits)
    /// GET /api/v3/capital/withdraw/history (withdrawals)
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let mut records = Vec::new();

        let mut params = HashMap::new();
        if let Some(asset) = &filter.asset {
            params.insert("coin".to_string(), asset.to_uppercase());
        }
        if let Some(start) = filter.start_time {
            params.insert("startTime".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("endTime".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.to_string());
        }

        if matches!(filter.record_type, FundsRecordType::Deposit | FundsRecordType::Both) {
            let response = self.get(MexcEndpoint::DepositHistory, params.clone()).await?;

            let items = response.as_array().cloned().unwrap_or_default();
            for item in &items {
                let id = item["id"].as_str().unwrap_or("").to_string();
                let asset = item["coin"].as_str().unwrap_or("").to_string();
                let amount = item["amount"]
                    .as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| item["amount"].as_f64())
                    .unwrap_or(0.0);
                let tx_hash = item["txId"].as_str().map(|s| s.to_string());
                let network = item["network"].as_str().map(|s| s.to_string());
                let status = item["status"].as_str().unwrap_or("Unknown").to_string();
                let timestamp = item["insertTime"].as_i64().unwrap_or(0);

                records.push(FundsRecord::Deposit {
                    id,
                    asset,
                    amount,
                    tx_hash,
                    network,
                    status,
                    timestamp,
                });
            }
        }

        if matches!(filter.record_type, FundsRecordType::Withdrawal | FundsRecordType::Both) {
            let response = self.get(MexcEndpoint::WithdrawHistory, params).await?;

            let items = response.as_array().cloned().unwrap_or_default();
            for item in &items {
                let id = item["id"].as_str().unwrap_or("").to_string();
                let asset = item["coin"].as_str().unwrap_or("").to_string();
                let amount = item["amount"]
                    .as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| item["amount"].as_f64())
                    .unwrap_or(0.0);
                let fee = item["transactionFee"]
                    .as_str().and_then(|s| s.parse::<f64>().ok())
                    .or_else(|| item["transactionFee"].as_f64());
                let address = item["address"].as_str().unwrap_or("").to_string();
                let tag = item["addressTag"].as_str()
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let tx_hash = item["txId"].as_str().map(|s| s.to_string());
                let network = item["network"].as_str().map(|s| s.to_string());
                let status = item["status"].as_str().unwrap_or("Unknown").to_string();
                let timestamp = item["applyTime"].as_i64()
                    .or_else(|| item["insertTime"].as_i64())
                    .unwrap_or(0);

                records.push(FundsRecord::Withdrawal {
                    id,
                    asset,
                    amount,
                    fee,
                    address,
                    tag,
                    tx_hash,
                    network,
                    status,
                    timestamp,
                });
            }
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB ACCOUNTS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl SubAccounts for MexcConnector {
    /// Perform sub-account operations: Create, List, Transfer, GetBalance.
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        match op {
            SubAccountOperation::Create { label } => {
                // POST /api/v3/sub-account/virtualSubAccount
                let mut params = HashMap::new();
                params.insert("subUserName".to_string(), label.clone());

                let response = self.post(MexcEndpoint::SubAccountCreate, params).await?;

                let id = response["subUserId"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| response["subUserId"].as_i64().map(|n| n.to_string()));

                Ok(SubAccountResult {
                    id,
                    name: Some(label),
                    accounts: vec![],
                    transaction_id: None,
                })
            }

            SubAccountOperation::List => {
                // GET /api/v3/sub-account/list
                let response = self.get(MexcEndpoint::SubAccountList, HashMap::new()).await?;

                let items = response.get("subAccounts")
                    .and_then(|v| v.as_array())
                    .or_else(|| response.as_array())
                    .cloned()
                    .unwrap_or_default();

                let accounts = items.iter().map(|item| {
                    let id = item["subUserId"]
                        .as_str()
                        .map(|s| s.to_string())
                        .or_else(|| item["subUserId"].as_i64().map(|n| n.to_string()))
                        .unwrap_or_default();
                    let name = item["subUserName"].as_str().unwrap_or("").to_string();
                    let status = if item["isFreeze"].as_bool().unwrap_or(false) {
                        "Frozen".to_string()
                    } else {
                        "Normal".to_string()
                    };

                    SubAccount { id, name, status }
                }).collect();

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts,
                    transaction_id: None,
                })
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                // POST /api/v3/capital/sub-account/universalTransfer
                // fromEmail / toEmail identifies the direction
                // MEXC uses email as sub-account identifier
                let mut params = HashMap::new();
                if to_sub {
                    params.insert("toEmail".to_string(), sub_account_id.clone());
                    params.insert("fromAccountType".to_string(), "SPOT".to_string());
                    params.insert("toAccountType".to_string(), "SPOT".to_string());
                } else {
                    params.insert("fromEmail".to_string(), sub_account_id.clone());
                    params.insert("fromAccountType".to_string(), "SPOT".to_string());
                    params.insert("toAccountType".to_string(), "SPOT".to_string());
                }
                params.insert("asset".to_string(), asset);
                params.insert("amount".to_string(), amount.to_string());

                let response = self.post(MexcEndpoint::SubAccountTransfer, params).await?;

                let tran_id = response["tranId"]
                    .as_str()
                    .map(|s| s.to_string())
                    .or_else(|| response["tranId"].as_i64().map(|n| n.to_string()));

                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id: tran_id,
                })
            }

            SubAccountOperation::GetBalance { sub_account_id } => {
                // GET /api/v3/sub-account/assets?email={sub_account_id}
                let mut params = HashMap::new();
                params.insert("email".to_string(), sub_account_id);

                let _response = self.get(MexcEndpoint::SubAccountAssets, params).await?;

                // Balance is available in response but SubAccountResult doesn't carry it;
                // return the sub-account id as acknowledgement that data was fetched.
                Ok(SubAccountResult {
                    id: None,
                    name: None,
                    accounts: vec![],
                    transaction_id: None,
                })
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS (not part of core traits)
// ═══════════════════════════════════════════════════════════════════════════════

impl MexcConnector {
    /// Get recent public spot trades.
    ///
    /// `GET /api/v3/trades`
    ///
    /// # Parameters
    /// - `symbol`: Spot symbol e.g. `BTCUSDT`
    /// - `limit`: Number of trades to return (optional, default 500, max 1000)
    pub async fn get_recent_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(MexcEndpoint::RecentTrades, params).await
    }

    /// Get personal spot trade history (requires auth).
    ///
    /// `GET /api/v3/myTrades`
    ///
    /// # Parameters
    /// - `symbol`: Spot symbol e.g. `BTCUSDT`
    /// - `limit`: Max number of trades (optional, default 500, max 1000)
    /// - `start_time`: Start timestamp in ms (optional)
    /// - `end_time`: End timestamp in ms (optional)
    pub async fn get_my_trades(
        &self,
        symbol: &str,
        limit: Option<u32>,
        start_time: Option<i64>,
        end_time: Option<i64>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        if let Some(st) = start_time {
            params.insert("startTime".to_string(), st.to_string());
        }
        if let Some(et) = end_time {
            params.insert("endTime".to_string(), et.to_string());
        }
        self.get(MexcEndpoint::MyTrades, params).await
    }

    /// Get futures mark price and index price for a contract.
    ///
    /// `GET /api/v1/contract/index_price/{symbol}`
    ///
    /// Returns the current mark price and index price for the given futures contract.
    pub async fn get_futures_mark_price(&self, symbol: &str) -> ExchangeResult<Value> {
        // MEXC futures endpoints use path-based symbol: /api/v1/contract/index_price/{symbol}
        let real_base = MexcUrls::futures_base_url();
        let path = format!("{}/{}", MexcEndpoint::FuturesMarkPrice.path(), symbol);
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, "");
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.update_weight_from_headers(&resp_headers);
        Ok(response)
    }

    /// Get current funding rate for a futures contract.
    ///
    /// `GET /api/v1/contract/funding_rate/{symbol}` (MEXC futures domain)
    ///
    /// # TODO
    /// Verify exact endpoint path against live MEXC contract API documentation.
    pub async fn get_funding_rate(&self, symbol: &str) -> ExchangeResult<Value> {
        let real_base = MexcUrls::futures_base_url();
        let path = format!("{}/{}", MexcEndpoint::FuturesFundingRate.path(), symbol);
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, "");
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &HashMap::new(), &HashMap::new()).await?;
        self.update_weight_from_headers(&resp_headers);
        Ok(response)
    }

    /// Get open interest for a futures contract via the ticker endpoint.
    ///
    /// MEXC does not expose `/api/v1/contract/open_interest/{symbol}` — that path
    /// returns 404. OI is embedded in the ticker response as `data.holdVol`.
    /// See: `GET /api/v1/contract/ticker?symbol={symbol}` (MEXC futures domain)
    pub async fn get_futures_ticker_raw(&self, symbol: &str) -> ExchangeResult<Value> {
        let real_base = MexcUrls::futures_base_url();
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, MexcEndpoint::FuturesTicker.path(), "");
        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped".to_string(),
            });
        }
        let mut params = HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        let (response, resp_headers) = self.http.get_with_response_headers(&url, &params, &HashMap::new()).await?;
        self.update_weight_from_headers(&resp_headers);
        Ok(response)
    }
}

/// Map internal AccountType to MEXC's transfer account type string.
fn account_type_to_mexc_str(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "SPOT",
        AccountType::Margin => "MARGIN",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "FUTURES",
        AccountType::Earn | AccountType::Lending | AccountType::Options | AccountType::Convert => "SPOT",
    }
}

fn interval_to_ms(interval: &str) -> i64 {
    match interval {
        "1m" => 60_000,
        "5m" => 300_000,
        "15m" => 900_000,
        "30m" => 1_800_000,
        "1h" => 3_600_000,
        "4h" => 14_400_000,
        "12h" => 43_200_000,
        "1d" => 86_400_000,
        "1w" => 604_800_000,
        _ => 3_600_000,
    }
}

// MEXC Spot connector — no futures/positions support in v5 yet.
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl crate::core::traits::Positions for MexcConnector {
    async fn get_positions(
        &self,
        _query: crate::core::types::PositionQuery,
    ) -> ExchangeResult<Vec<crate::core::types::Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "MEXC positions not implemented in v5".into(),
        ))
    }

    async fn get_funding_rate(
        &self,
        _symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<crate::core::types::FundingRate> {
        Err(ExchangeError::UnsupportedOperation(
            "MEXC funding rate not implemented in v5".into(),
        ))
    }

    async fn modify_position(
        &self,
        _req: crate::core::types::PositionModification,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "MEXC position modification not implemented in v5".into(),
        ))
    }

    async fn get_open_interest(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<crate::core::types::OpenInterest> {
        // MEXC has no dedicated OI endpoint. OI lives in the futures ticker as `data.holdVol`.
        // Reference: GET /api/v1/contract/ticker?symbol=BTC_USDT → data.holdVol
        let raw_symbol = if symbol.contains('/') {
            let parts: Vec<&str> = symbol.split('/').collect();
            format!(
                "{}_{}",
                parts[0].to_uppercase(),
                parts.get(1).copied().unwrap_or("USDT").to_uppercase()
            )
        } else if symbol.contains('-') {
            symbol.to_uppercase().replace('-', "_")
        } else if !symbol.contains('_') {
            // Spot-format symbol (BTCUSDT) — convert to futures format (BTC_USDT).
            use crate::core::utils::symbol_normalizer::SymbolNormalizer;
            SymbolNormalizer::from_exchange(crate::core::types::ExchangeId::MEXC, symbol, AccountType::Spot)
                .and_then(|canonical| SymbolNormalizer::to_exchange(crate::core::types::ExchangeId::MEXC, &canonical, AccountType::FuturesCross))
                .unwrap_or_else(|_| symbol.to_uppercase())
        } else {
            symbol.to_uppercase()
        };
        let response = self.get_futures_ticker_raw(&raw_symbol).await?;
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("MEXC OI: missing 'data' in ticker response".to_string()))?;
        // data is an array (ticker list) when no symbol filter is applied,
        // but with ?symbol= it returns a single object.
        let ticker_obj = if let Some(arr) = data.as_array() {
            arr.iter()
                .find(|t| t.get("symbol").and_then(|s| s.as_str()) == Some(raw_symbol.as_str()))
                .ok_or_else(|| ExchangeError::Parse(format!("MEXC OI: symbol {} not found in ticker list", raw_symbol)))?
        } else {
            data
        };
        let oi = ticker_obj
            .get("holdVol")
            .and_then(|v| v.as_f64())
            .or_else(|| {
                ticker_obj.get("holdVol")
                    .and_then(|v| v.as_str())
                    .and_then(|s| s.parse::<f64>().ok())
            })
            .unwrap_or(0.0);
        Ok(crate::core::types::OpenInterest {
            open_interest: oi,
            open_interest_value: None,
            timestamp: crate::core::timestamp_millis() as i64, ..Default::default() 
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MarketDataPublic trait impl
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl MarketDataPublic for MexcConnector {
    /// Recent public trades for a symbol.
    ///
    /// - Spot: `GET /api/v3/trades` — id field is null on MEXC; array index used as fallback id.
    /// - Futures: `GET /api/v1/contract/deals/{symbol}` — direction T:1=Buy/2=Sell.
    async fn get_recent_trades(
        &self,
        symbol: SymbolInput<'_>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<PublicTrade>> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());
                if let Some(l) = limit {
                    params.insert("limit".to_string(), l.to_string());
                }
                let raw = self.get(MexcEndpoint::RecentTrades, params).await?;
                MexcParser::parse_recent_trades_spot(&raw)
            }
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                let real_base = MexcUrls::futures_base_url();
                let mut path = format!("{}/{}", MexcEndpoint::FuturesRecentTrades.path(), symbol);
                if let Some(l) = limit {
                    path = format!("{}?limit={}", path, l);
                }
                let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, "");
                if !self.rate_limit_wait(1, false).await {
                    return Err(ExchangeError::RateLimitExceeded {
                        retry_after: None,
                        message: "Rate limit budget >= 90% used; request dropped".to_string(),
                    });
                }
                let (response, resp_headers) = self.http
                    .get_with_response_headers(&url, &HashMap::new(), &HashMap::new())
                    .await?;
                self.update_weight_from_headers(&resp_headers);
                MexcParser::parse_recent_trades_futures(&response)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} account type not supported for get_recent_trades on MEXC", account_type),
            )),
        }
    }

    /// Aggregated trades for spot via `GET /api/v3/aggTrades`.
    ///
    /// Futures aggTrades are not materially different from recent trades on MEXC;
    /// returns `UnsupportedOperation` for non-spot account types — callers fall back
    /// to `get_recent_trades`.
    ///
    /// Note: MEXC aggTrade fields `a`/`f`/`l` (agg-id, first-fill-id, last-fill-id)
    /// are always null; array index is used as the id.
    async fn get_agg_trades(
        &self,
        symbol: SymbolInput<'_>,
        limit: Option<u32>,
        from_id: Option<u64>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<AggTrade>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
                let mut params = HashMap::new();
                params.insert("symbol".to_string(), symbol.to_string());
                if let Some(l) = limit {
                    params.insert("limit".to_string(), l.to_string());
                }
                if let Some(id) = from_id {
                    params.insert("fromId".to_string(), id.to_string());
                }
                let raw = self.get(MexcEndpoint::SpotAggTrades, params).await?;
                MexcParser::parse_agg_trades_spot(&raw)
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                "get_agg_trades: MEXC futures aggTrades are identical to recent trades — \
                 use get_recent_trades for futures account types".into(),
            )),
        }
    }

    /// Mark-price (fair price) klines from the MEXC contract API.
    ///
    /// Endpoint: `GET /api/v1/contract/kline/fair_price/{symbol}` on `contract.mexc.com`.
    ///
    /// # Quirks
    /// - Symbol format: `BTC_USDT` (underscore, not `BTCUSDT`).
    /// - `start`/`end` params are Unix **seconds** — `start_time`/`end_time` ms are ÷1000.
    /// - Returned timestamps are also Unix seconds — parser converts to ms.
    /// - `volume` is always 0.0 per API docs.
    /// - Max 2000 records per request.
    async fn get_mark_price_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u32>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> crate::core::ExchangeResult<Vec<Kline>> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        let futures_interval = map_mexc_futures_interval(interval);

        let real_base = MexcUrls::futures_base_url();
        let path = format!("{}/{}", MexcEndpoint::FuturesFairPriceKlines.path(), symbol);

        let mut params = std::collections::HashMap::new();
        params.insert("interval".to_string(), futures_interval.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(2000).to_string());
        }
        if let Some(et) = end_time {
            // MEXC contract API expects Unix seconds
            params.insert("end".to_string(), (et / 1000).to_string());
        }

        let query = build_query(&params);
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, &query);

        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped"
                    .to_string(),
            });
        }
        let (response, resp_headers) = self.http
            .get_with_response_headers(&url, &std::collections::HashMap::new(), &std::collections::HashMap::new())
            .await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::parse_derived_klines_futures(&response)
    }

    /// Index-price klines from the MEXC contract API.
    ///
    /// Endpoint: `GET /api/v1/contract/kline/index_price/{symbol}` on `contract.mexc.com`.
    /// Same quirks as `get_mark_price_klines` (seconds, 2000/req, vol=0).
    async fn get_index_price_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u32>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> crate::core::ExchangeResult<Vec<Kline>> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;
        let futures_interval = map_mexc_futures_interval(interval);

        let real_base = MexcUrls::futures_base_url();
        let path = format!("{}/{}", MexcEndpoint::FuturesIndexPriceKlines.path(), symbol);

        let mut params = std::collections::HashMap::new();
        params.insert("interval".to_string(), futures_interval.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(2000).to_string());
        }
        if let Some(et) = end_time {
            params.insert("end".to_string(), (et / 1000).to_string());
        }

        let query = build_query(&params);
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, &path, &query);

        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped"
                    .to_string(),
            });
        }
        let (response, resp_headers) = self.http
            .get_with_response_headers(&url, &std::collections::HashMap::new(), &std::collections::HashMap::new())
            .await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::parse_derived_klines_futures(&response)
    }

    /// Premium index klines — NOT SUPPORTED by MEXC contract API.
    ///
    /// MEXC has no premium-index kline endpoint. Premium can be derived manually
    /// as `(fair_price_kline - index_price_kline) / index_price_kline`.
    /// Source: https://mexcdevelop.github.io/apidocs/contract_v1_en/
    async fn get_premium_index_klines(
        &self,
        _symbol: SymbolInput<'_>,
        _interval: &str,
        _limit: Option<u32>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> crate::core::ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::NotSupported(
            "NotSupported: MEXC contract API has no premium-index kline endpoint. \
             Premium index can be derived: (fair_price_kline − index_price_kline) / index_price_kline. \
             Source: https://mexcdevelop.github.io/apidocs/contract_v1_en/"
                .into(),
        ))
    }

    /// Historical funding rates from the MEXC contract API.
    ///
    /// Endpoint: `GET /api/v1/contract/funding_rate/history` on `contract.mexc.com`.
    ///
    /// # Quirks
    /// - **No date filter**: `start_time`/`end_time` are silently ignored.
    ///   Pagination is page-based only (`page_num`/`page_size`, max 1000/page).
    /// - `limit` maps to `page_size` (capped at 1000).
    /// - To walk history, increment `page_num`; combine with external time filtering.
    async fn get_funding_rate_history(
        &self,
        symbol: SymbolInput<'_>,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> crate::core::ExchangeResult<Vec<FundingRate>> {
        let symbol = symbol.resolve(ExchangeId::MEXC, account_type)?;

        let real_base = MexcUrls::futures_base_url();
        let path = MexcEndpoint::FuturesFundingRateHistory.path();

        let mut params = std::collections::HashMap::new();
        params.insert("symbol".to_string(), symbol.to_string());
        params.insert("page_num".to_string(), "1".to_string());
        // page_size max 1000; default 20
        let page_size = limit.unwrap_or(20).min(1000);
        params.insert("page_size".to_string(), page_size.to_string());

        let query = build_query(&params);
        let url = assemble_rest_url(self.rest_override.as_deref(), real_base, path, &query);

        if !self.rate_limit_wait(1, false).await {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "Rate limit budget >= 90% used; non-essential market data request dropped"
                    .to_string(),
            });
        }
        let (response, resp_headers) = self.http
            .get_with_response_headers(&url, &std::collections::HashMap::new(), &std::collections::HashMap::new())
            .await?;
        self.update_weight_from_headers(&resp_headers);
        MexcParser::parse_funding_rate_history_futures(&response)
    }

    /// Open interest history — NOT SUPPORTED by MEXC contract API.
    ///
    /// MEXC exposes open interest only as a snapshot via `holdVol` in the ticker
    /// (`GET /api/v1/contract/ticker`). No time-series OI history endpoint exists.
    /// Source: https://mexcdevelop.github.io/apidocs/contract_v1_en/
    async fn get_open_interest_history(
        &self,
        _symbol: SymbolInput<'_>,
        _period: &str,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
        _limit: Option<u32>,
        _account_type: AccountType,
    ) -> crate::core::ExchangeResult<Vec<crate::core::types::OpenInterest>> {
        Err(ExchangeError::NotSupported(
            "NotSupported: MEXC contract API has no OI history endpoint. \
             OI is snapshot-only via 'holdVol' in GET /api/v1/contract/ticker. \
             Source: https://mexcdevelop.github.io/apidocs/contract_v1_en/"
                .into(),
        ))
    }

    /// Long/short ratio history — NOT SUPPORTED by MEXC contract API.
    ///
    /// Not documented anywhere in the MEXC contract API.
    /// Source: https://mexcdevelop.github.io/apidocs/contract_v1_en/
    async fn get_long_short_ratio_history(
        &self,
        _symbol: SymbolInput<'_>,
        _period: &str,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
        _limit: Option<u32>,
        _account_type: AccountType,
    ) -> crate::core::ExchangeResult<Vec<crate::core::types::LongShortRatio>> {
        Err(ExchangeError::NotSupported(
            "NotSupported: MEXC contract API does not expose a long/short ratio history endpoint. \
             Source: https://mexcdevelop.github.io/apidocs/contract_v1_en/"
                .into(),
        ))
    }
}

/// Map canonical kline interval to MEXC contract API interval string.
fn map_mexc_futures_interval(interval: &str) -> &'static str {
    match interval {
        "1m"  => "Min1",
        "5m"  => "Min5",
        "15m" => "Min15",
        "30m" => "Min30",
        "1h"  => "Min60",
        "4h"  => "Hour4",
        "8h"  => "Hour8",
        "1d"  => "Day1",
        "1w"  => "Week1",
        "1M"  => "Month1",
        _     => "Min60",
    }
}

/// Build URL query string from a HashMap (e.g. `?k1=v1&k2=v2`).
fn build_query(params: &std::collections::HashMap<String, String>) -> String {
    if params.is_empty() {
        return String::new();
    }
    let qs: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
    format!("?{}", qs.join("&"))
}

impl crate::core::traits::HasCapabilities for MexcConnector {
    fn capabilities(&self) -> crate::core::types::ConnectorCapabilities {
        crate::core::types::ConnectorCapabilities {
            has_ticker: true, has_orderbook: true, has_klines: true,
            has_recent_trades: true, has_exchange_info: true,
            has_liquidation_history: false, has_open_interest_history: false,
            has_premium_index: false, has_long_short_ratio_history: false,
            has_funding_rate_history: true, has_mark_price_klines: true,
            has_basis_history: false,
            has_taker_volume_history: false,
            has_index_price_klines: true,
            has_premium_index_klines: false,
            has_agg_trades: true,            has_market_order: true, has_limit_order: true,
            has_open_orders: true, has_order_history: true, has_user_trades: true,
            has_positions: true, has_mark_price: false, has_modify_position: false,
            has_closed_pnl: false, has_long_short_ratio: false,
            has_cancel_all: true, has_amend_order: false,
            has_batch_place: true, has_batch_cancel: false,
            max_batch_place_size: 20, max_batch_cancel_size: 0,
            has_balance: true, has_account_info: true, has_fees: true,
            has_transfers: true, has_deposit_withdraw: true, has_sub_accounts: true,
            has_funding_payments: false, has_ledger: false,
            has_websocket: true, has_ws_klines: true, has_ws_trades: true,
            has_ws_orderbook: true, has_ws_ticker: true,
            has_ws_mark_price: false, has_ws_funding_rate: false,
            validation: self.validation_status(),
        }
    }

    fn validation_status(&self) -> Option<&'static crate::core::types::ValidationStamp> {
        crate::core::utils::validation_snapshot::validation_for(crate::core::types::ExchangeId::MEXC)
    }
}
