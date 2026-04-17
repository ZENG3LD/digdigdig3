//! # Deribit Connector
//!
//! Implementation of core traits for Deribit exchange.
//!
//! ## Core Traits
//! - `ExchangeIdentity` - Exchange identification
//! - `MarketData` - Market data operations
//! - `Trading` - Trading operations
//! - `Account` - Account information
//! - `Positions` - Futures/options positions
//!
//! ## JSON-RPC Request Pattern
//! All requests use POST with JSON-RPC 2.0 format

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
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    CustodialFunds,
    DepositAddress, WithdrawResponse, FundsRecord,
};
use crate::core::types::{MarketDataCapabilities, TradingCapabilities, AccountCapabilities};
use crate::core::types::{WithdrawRequest, FundsHistoryFilter};
use crate::core::types::{ConnectorStats, SymbolInfo, CancelAllResponse, AmendRequest};
use crate::core::types::{UserTrade, UserTradeFilter};
use crate::core::types::{
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerEntryType, LedgerFilter,
};
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    FundingHistory, AccountLedger,
};
use crate::core::{CancelAll, AmendOrder};
use crate::core::utils::{RuntimeLimiter, RateLimitMonitor, RateLimitPressure};
use crate::core::types::{RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits, EndpointWeight, DecayingLimitConfig};
use crate::core::utils::PrecisionCache;

use super::endpoints::{DeribitUrls, DeribitMethod, format_symbol};
use super::auth::DeribitAuth;
use super::parser::DeribitParser;

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMIT CAPABILITIES
// ═══════════════════════════════════════════════════════════════════════════════

static DERIBIT_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Decaying,
    rest_pools: &[] as &[RestLimitPool],
    decaying: Some(DecayingLimitConfig {
        max_counter: 50000.0,
        decay_rate_per_sec: 10000.0,
        default_cost: 500.0,
    }),
    endpoint_weights: &[] as &[EndpointWeight],
    ws: WsLimits {
        max_connections: Some(32),
        max_subs_per_conn: None,
        max_msg_per_sec: None,
        max_streams_per_conn: None,
    },
};

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// Deribit connector
pub struct DeribitConnector {
    /// HTTP client
    http: HttpClient,
    /// Authentication (None for public methods)
    auth: Arc<Mutex<Option<DeribitAuth>>>,
    /// URLs (mainnet/testnet)
    urls: DeribitUrls,
    /// Testnet mode
    testnet: bool,
    /// Request counter for JSON-RPC ID
    request_id: Arc<Mutex<u64>>,
    /// Runtime rate limiter (Decaying: max=50000, decay=10000/s, cost=500/req)
    limiter: Arc<Mutex<RuntimeLimiter>>,
    /// Pressure monitor
    monitor: Arc<Mutex<RateLimitMonitor>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: PrecisionCache,
}

impl DeribitConnector {
    /// Create new connector
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            DeribitUrls::TESTNET
        } else {
            DeribitUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(DeribitAuth::new)
            .transpose()?;

        let limiter = Arc::new(Mutex::new(RuntimeLimiter::from_caps(&DERIBIT_RATE_CAPS)));
        let monitor = Arc::new(Mutex::new(RateLimitMonitor::new("Deribit")));

        let connector = Self {
            http,
            auth: Arc::new(Mutex::new(auth)),
            urls,
            testnet,
            request_id: Arc::new(Mutex::new(1)),
            limiter,
            monitor,
            precision: PrecisionCache::new(),
        };

        // Authenticate if we have credentials
        if credentials.is_some() {
            connector.authenticate().await?;
        }

        Ok(connector)
    }

    /// Create connector for public methods only
    pub async fn public(testnet: bool) -> ExchangeResult<Self> {
        Self::new(None, testnet).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // AUTHENTICATION
    // ═══════════════════════════════════════════════════════════════════════════

    /// Authenticate using client credentials grant
    async fn authenticate(&self) -> ExchangeResult<()> {
        let params = {
            let auth_guard = self.auth.lock().expect("Mutex poisoned");
            let auth = auth_guard.as_ref()
                .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

            // Use client signature (more secure than client credentials)
            auth.client_signature_params()
        };

        let response = self.rpc_call(DeribitMethod::Auth, params).await?;
        let (access_token, refresh_token, expires_in) = DeribitParser::parse_auth(&response)?;

        // Store tokens
        let mut auth_guard = self.auth.lock().expect("Mutex poisoned");
        if let Some(auth) = auth_guard.as_mut() {
            auth.store_tokens(access_token, refresh_token, expires_in);
        }

        Ok(())
    }

    /// Refresh access token
    async fn _refresh_token(&self) -> ExchangeResult<()> {
        let params = {
            let auth_guard = self.auth.lock().expect("Mutex poisoned");
            let auth = auth_guard.as_ref()
                .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

            auth.refresh_token_params()?
        };

        let response = self.rpc_call(DeribitMethod::Auth, params).await?;
        let (access_token, refresh_token, expires_in) = DeribitParser::parse_auth(&response)?;

        // Store new tokens
        let mut auth_guard = self.auth.lock().expect("Mutex poisoned");
        if let Some(auth) = auth_guard.as_mut() {
            auth.store_tokens(access_token, refresh_token, expires_in);
        }

        Ok(())
    }

    /// Ensure we have a valid access token (non-recursive version)
    async fn ensure_authenticated(&self) -> ExchangeResult<()> {
        let needs_refresh = {
            let auth_guard = self.auth.lock().expect("Mutex poisoned");
            if let Some(auth) = auth_guard.as_ref() {
                !auth.has_valid_token()
            } else {
                return Err(ExchangeError::Auth("No credentials configured".to_string()));
            }
        };

        if needs_refresh {
            // Directly refresh without going through ensure_authenticated again
            let params = {
                let auth_guard = self.auth.lock().expect("Mutex poisoned");
                let auth = auth_guard.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?;

                auth.refresh_token_params()?
            };

            let response = self.rpc_call_internal(DeribitMethod::Auth, params).await?;
            let (access_token, refresh_token, expires_in) = DeribitParser::parse_auth(&response)?;

            // Store new tokens
            let mut auth_guard = self.auth.lock().expect("Mutex poisoned");
            if let Some(auth) = auth_guard.as_mut() {
                auth.store_tokens(access_token, refresh_token, expires_in);
            }
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // RATE LIMITING
    // ═══════════════════════════════════════════════════════════════════════════

    /// Wait for rate limit if necessary (cost = 500 credits per request).
    ///
    /// Non-essential requests are dropped at >= 90% utilization.
    /// Returns `true` if acquired, `false` if dropped.
    async fn rate_limit_wait(&self, essential: bool) -> bool {
        loop {
            let wait_time = {
                let mut limiter = self.limiter.lock().expect("limiter poisoned");
                let pressure = self.monitor.lock().expect("monitor poisoned").check(&mut limiter);
                if pressure >= RateLimitPressure::Cutoff && !essential {
                    return false;
                }
                // Cost = 500 credits per request
                if limiter.try_acquire("default", 500) {
                    return true;
                }
                limiter.time_until_ready("default", 500)
            };
            if wait_time > Duration::ZERO {
                tokio::time::sleep(wait_time).await;
            }
        }
    }

    /// Internal RPC call without auth check (to avoid recursion)
    async fn rpc_call_internal(
        &self,
        method: DeribitMethod,
        params: HashMap<String, Value>,
    ) -> ExchangeResult<Value> {
        // All RPC calls are essential (both public market data and private trading)
        self.rate_limit_wait(true).await;

        let id = self.next_id();
        let url = self.urls.rest_url();

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method.method(),
            "params": params,
        });

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Make request (all Deribit requests use POST)
        let response = self.http.post(url, &request, &headers).await?;

        Ok(response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // JSON-RPC HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get next request ID
    fn next_id(&self) -> u64 {
        let mut id = self.request_id.lock().expect("Mutex poisoned");
        let current = *id;
        *id += 1;
        current
    }

    /// Make JSON-RPC call
    async fn rpc_call(
        &self,
        method: DeribitMethod,
        params: HashMap<String, Value>,
    ) -> ExchangeResult<Value> {
        // All RPC calls are essential (both public market data and private trading)
        self.rate_limit_wait(true).await;

        let id = self.next_id();
        let url = self.urls.rest_url();

        // Build JSON-RPC request
        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method.method(),
            "params": params,
        });

        // Build headers
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        // Add Authorization header for private methods
        if method.requires_auth() {
            self.ensure_authenticated().await?;

            let auth_header = {
                let auth_guard = self.auth.lock().expect("Mutex poisoned");
                auth_guard.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("No credentials configured".to_string()))?
                    .auth_header()?
            };

            headers.insert("Authorization".to_string(), auth_header);
        }

        // Make request (all Deribit requests use POST)
        let response = self.http.post(url, &request, &headers).await?;

        // Check for JSON-RPC errors (handled by parser)
        Ok(response)
    }

    /// Currency from symbol for Deribit
    fn _currency_from_symbol(symbol: &Symbol) -> String {
        // For Deribit, use base currency (BTC, ETH, SOL, etc.)
        symbol.base.to_uppercase()
    }

    /// Instrument name from symbol
    fn instrument_from_symbol(symbol: &Symbol, account_type: AccountType) -> String {
        format_symbol(&symbol.base, &symbol.quote, account_type)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for DeribitConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Deribit
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

    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        DERIBIT_RATE_CAPS
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![
            AccountType::Spot,          // Limited spot trading
            AccountType::FuturesCross,  // Inverse and linear perpetuals/futures
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
impl MarketData for DeribitConnector {
    fn market_data_capabilities(&self, _account_type: AccountType) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,   // public/ticker
            has_ticker: true,  // public/ticker
            has_orderbook: true, // public/get_order_book
            has_klines: true,  // public/get_tradingview_chart_data
            has_exchange_info: true, // public/get_instruments
            has_recent_trades: false, // GetLastTradesByInstrument exists but is not wired to the trait method
            has_ws_klines: true,     // chart.trades.{instrument}.{resolution}
            has_ws_trades: true,     // trades.{instrument}.100ms
            has_ws_orderbook: true,  // book.{instrument}.100ms
            has_ws_ticker: true,     // ticker.{instrument}.100ms
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m",
                "1h", "2h", "4h", "6h", "12h", "1d",
            ],
            max_kline_limit: Some(10000),
        }
    }

    async fn get_price(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Price> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));

        let response = self.rpc_call(DeribitMethod::Ticker, params).await?;
        DeribitParser::parse_price(&response)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));
        if let Some(d) = depth {
            params.insert("depth".to_string(), json!(d));
        }

        let response = self.rpc_call(DeribitMethod::GetOrderBook, params).await?;
        DeribitParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let (resolution, interval_ms): (&str, u64) = match interval {
            "1m"  => ("1",   60_000),
            "3m"  => ("3",   180_000),
            "5m"  => ("5",   300_000),
            "15m" => ("15",  900_000),
            "30m" => ("30",  1_800_000),
            "1h"  => ("60",  3_600_000),
            "2h"  => ("120", 7_200_000),
            "4h"  => ("240", 14_400_000),
            "6h"  => ("360", 21_600_000),
            "12h" => ("720", 43_200_000),
            "1d" | "1D" => ("1D", 86_400_000),
            other => return Err(ExchangeError::Parse(format!("Unsupported interval: {}", other))),
        };

        let count = limit.unwrap_or(2000).min(10000) as u64;
        let now_ms = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let end_ms = end_time.map(|t| t as u64).unwrap_or(now_ms);
        let start_ms = end_ms.saturating_sub(count * interval_ms);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));
        params.insert("start_timestamp".to_string(), json!(start_ms));
        params.insert("end_timestamp".to_string(), json!(end_ms));
        params.insert("resolution".to_string(), json!(resolution));

        let response = self.rpc_call(DeribitMethod::GetTradingviewChartData, params).await?;
        DeribitParser::parse_klines(&response, interval_ms)
    }

    async fn get_ticker(&self, symbol: Symbol, account_type: AccountType) -> ExchangeResult<Ticker> {
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));

        let response = self.rpc_call(DeribitMethod::Ticker, params).await?;
        DeribitParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        // Use test method for ping
        let params = HashMap::new();
        let _response = self.rpc_call(DeribitMethod::Test, params).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // Fetch instruments for major currencies: BTC, ETH, SOL, USDC
        let currencies = ["BTC", "ETH", "SOL", "USDC"];
        let mut all_symbols = Vec::new();

        for currency in &currencies {
            let mut params = HashMap::new();
            params.insert("currency".to_string(), json!(currency));
            params.insert("expired".to_string(), json!(false));

            match self.rpc_call(DeribitMethod::GetInstruments, params).await {
                Ok(response) => {
                    match DeribitParser::parse_exchange_info(&response, account_type) {
                        Ok(mut symbols) => all_symbols.append(&mut symbols),
                        Err(_) => continue,
                    }
                }
                Err(_) => continue,
            }
        }

        self.precision.load_from_symbols(&all_symbols);
        Ok(all_symbols)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for DeribitConnector {
    fn trading_capabilities(&self, _account_type: AccountType) -> TradingCapabilities {
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: true,    // type=stop_market with trigger_price
            has_stop_limit: true,     // type=stop_limit with trigger_price + price
            has_trailing_stop: true,  // type=trailing_stop with trailing_amount
            has_bracket: true,        // OTOCO via linked_order_type=one_triggers_one_cancels_other
            has_oco: false,           // no standalone OCO; bracket covers the use-case via OTOCO
            has_amend: true,          // AmendOrder trait: private/edit
            has_batch: false,         // no batch order endpoint
            max_batch_size: None,
            has_cancel_all: true,     // CancelAll trait: private/cancel_all / cancel_all_by_instrument
            has_user_trades: true,    // get_user_trades_by_instrument / by_currency
            has_order_history: true,  // get_order_history_by_currency / by_instrument
        }
    }

    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;
        let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

        let method = match side {
            OrderSide::Buy => DeribitMethod::Buy,
            OrderSide::Sell => DeribitMethod::Sell,
        };

        match req.order_type {
            OrderType::Market => {
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("market"));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::Limit { price } => {
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("limit"));
                params.insert("price".to_string(), json!(price));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopMarket { stop_price } => {
                // Deribit: type=stop_market with trigger_price
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("stop_market"));
                params.insert("trigger".to_string(), json!("last_price"));
                params.insert("trigger_price".to_string(), json!(stop_price));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::StopLimit { stop_price, limit_price } => {
                // Deribit: type=stop_limit with trigger_price + price
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("stop_limit"));
                params.insert("trigger".to_string(), json!("last_price"));
                params.insert("trigger_price".to_string(), json!(stop_price));
                params.insert("price".to_string(), json!(limit_price));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::TrailingStop { callback_rate, activation_price } => {
                // Deribit: type=trailing_stop with trailing_amount as percentage
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("trailing_stop"));
                params.insert("trailing_amount".to_string(), json!(callback_rate));
                if let Some(act_price) = activation_price {
                    params.insert("trigger_price".to_string(), json!(act_price));
                }

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::PostOnly { price } => {
                // Deribit: limit with post_only=true
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("limit"));
                params.insert("price".to_string(), json!(price));
                params.insert("post_only".to_string(), json!(true));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::Ioc { price } => {
                // Deribit: limit with time_in_force=immediate_or_cancel
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("limit"));
                params.insert("price".to_string(), json!(price));
                params.insert("time_in_force".to_string(), json!("immediate_or_cancel"));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::Fok { price } => {
                // Deribit: limit with time_in_force=fill_or_kill
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("limit"));
                params.insert("price".to_string(), json!(price));
                params.insert("time_in_force".to_string(), json!("fill_or_kill"));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::Gtd { price, expire_time } => {
                // Deribit: limit with time_in_force=good_til_day
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("limit"));
                params.insert("price".to_string(), json!(price));
                params.insert("time_in_force".to_string(), json!("good_til_day"));
                // Deribit doesn't have per-order expiry time in the same way,
                // good_til_day expires at end of trading day
                let _ = expire_time;

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::ReduceOnly { price } => {
                // Deribit: limit or market with reduce_only=true
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("reduce_only".to_string(), json!(true));

                if let Some(p) = price {
                    params.insert("type".to_string(), json!("limit"));
                    params.insert("price".to_string(), json!(p));
                } else {
                    params.insert("type".to_string(), json!("market"));
                }

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::Iceberg { price, display_quantity } => {
                // Deribit: limit with display_amount (visible slice) and refresh_amount
                // (how much to show after each fill — defaults to display_amount).
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!("limit"));
                params.insert("price".to_string(), json!(price));
                params.insert("display_amount".to_string(), json!(display_quantity));
                params.insert("refresh_amount".to_string(), json!(display_quantity));

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_order(&response, &instrument_name).map(PlaceOrderResponse::Simple)
            }

            OrderType::Bracket { price, take_profit, stop_loss } => {
                // Deribit: OTOCO (One-Triggers-One-Cancels-Other) via linked_order_type param.
                // The entry leg is the main order; otoco_config carries the TP and SL legs.
                //
                // API reference: private/buy + private/sell with:
                //   linked_order_type: "one_triggers_one_cancels_other"
                //   otoco_config: [
                //     { order_type: "limit" | "market", limit_price: ..., amount: ... },  // TP leg
                //     { order_type: "stop_market", trigger_price: ..., amount: ... },      // SL leg
                //   ]
                let entry_type = if price.is_some() { "limit" } else { "market" };

                let otoco_config = json!([
                    {
                        "order_type": "limit",
                        "limit_price": take_profit,
                        "amount": quantity,
                        "reduce_only": true,
                    },
                    {
                        "order_type": "stop_market",
                        "trigger_price": stop_loss,
                        "amount": quantity,
                        "reduce_only": true,
                    }
                ]);

                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                params.insert("amount".to_string(), json!(quantity));
                params.insert("type".to_string(), json!(entry_type));
                params.insert("linked_order_type".to_string(), json!("one_triggers_one_cancels_other"));
                params.insert("otoco_config".to_string(), otoco_config);

                if let Some(p) = price {
                    params.insert("price".to_string(), json!(p));
                }

                let response = self.rpc_call(method, params).await?;
                DeribitParser::parse_bracket_order(&response, &instrument_name)
                    .map(|b| PlaceOrderResponse::Bracket(Box::new(b)))
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
            )),
        }
    }

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let mut params = HashMap::new();
                params.insert("order_id".to_string(), json!(order_id));

                let response = self.rpc_call(DeribitMethod::Cancel, params).await?;
                DeribitParser::parse_order(&response, "")
            }

            _ => Err(ExchangeError::UnsupportedOperation(
                format!("{:?} cancel scope not supported — use CancelAll trait", req.scope)
            )),
        }
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();

        if let Some(sym) = &filter.symbol {
            // sym is already a Symbol struct
            let instrument_name = Self::instrument_from_symbol(sym, account_type);
            params.insert("instrument_name".to_string(), json!(instrument_name));
        } else {
            // Default to BTC currency if no symbol specified
            params.insert("currency".to_string(), json!("BTC"));
        }

        if let Some(limit) = filter.limit {
            params.insert("count".to_string(), json!(limit.min(1000)));
        }

        if let Some(start) = filter.start_time {
            params.insert("start_timestamp".to_string(), json!(start));
        }

        if let Some(end) = filter.end_time {
            params.insert("end_timestamp".to_string(), json!(end));
        }

        let method = if filter.symbol.is_some() {
            DeribitMethod::GetUserTradesByInstrument
        } else {
            DeribitMethod::GetUserTradesByCurrency
        };

        let response = self.rpc_call(method, params).await?;
        DeribitParser::parse_orders(&response)
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        let _ = (symbol, account_type); // Not needed for Deribit
        let mut params = HashMap::new();
        params.insert("order_id".to_string(), json!(order_id));

        let response = self.rpc_call(DeribitMethod::GetOrderState, params).await?;
        DeribitParser::parse_order(&response, "")
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

        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            let instrument_name = Self::instrument_from_symbol(&sym, account_type);
            params.insert("instrument_name".to_string(), json!(instrument_name));
            let response = self.rpc_call(DeribitMethod::GetOpenOrdersByInstrument, params).await?;
            DeribitParser::parse_orders(&response)
        } else {
            // Get all open orders (no specific instrument)
            let response = self.rpc_call(DeribitMethod::GetOpenOrders, params).await?;
            DeribitParser::parse_orders(&response)
        }
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let mut params = HashMap::new();

        if let Some(ref symbol_str) = filter.symbol {
            // Use instrument-specific endpoint when symbol is provided.
            // symbol_str may be in "BTC/USD" or "BTC-PERPETUAL" format.
            let instrument_name = if symbol_str.contains('/') {
                let parts: Vec<&str> = symbol_str.splitn(2, '/').collect();
                let sym = crate::core::Symbol::new(parts[0], *parts.get(1).unwrap_or(&"USD"));
                Self::instrument_from_symbol(&sym, account_type)
            } else {
                symbol_str.clone()
            };
            params.insert("instrument_name".to_string(), json!(instrument_name));

            if let Some(lim) = filter.limit {
                params.insert("count".to_string(), json!(lim.min(100)));
            }
            if let Some(st) = filter.start_time {
                params.insert("start_timestamp".to_string(), json!(st));
            }
            if let Some(et) = filter.end_time {
                params.insert("end_timestamp".to_string(), json!(et));
            }
            params.insert("sorting".to_string(), json!("desc"));

            let response = self.rpc_call(DeribitMethod::GetUserTradesByInstrument, params).await?;
            DeribitParser::parse_user_trades(&response)
        } else {
            // No symbol: use currency endpoint, defaulting to BTC.
            // Extract currency from filter symbol or fall back to "BTC".
            let currency = "BTC";
            params.insert("currency".to_string(), json!(currency));

            if let Some(lim) = filter.limit {
                params.insert("count".to_string(), json!(lim.min(100)));
            }
            if let Some(st) = filter.start_time {
                params.insert("start_timestamp".to_string(), json!(st));
            }
            if let Some(et) = filter.end_time {
                params.insert("end_timestamp".to_string(), json!(et));
            }
            params.insert("sorting".to_string(), json!("desc"));

            let response = self.rpc_call(DeribitMethod::GetUserTradesByCurrency, params).await?;
            DeribitParser::parse_user_trades(&response)
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for DeribitConnector {
    fn account_capabilities(&self, _account_type: AccountType) -> AccountCapabilities {
        AccountCapabilities {
            has_balances: true,       // private/get_account_summary
            has_account_info: true,   // derived from get_account_summary
            has_fees: true,           // maker/taker_commission from account summary extended
            has_transfers: false,     // no internal transfer endpoint implemented
            has_sub_accounts: false,  // Deribit has sub-accounts but not wired to the trait
            has_deposit_withdraw: true, // CustodialFunds: get_deposit_address + withdraw + funds history
            has_margin: false,        // Deribit uses dynamic margin; no explicit margin query implemented
            has_earn_staking: false,
            has_funding_history: true, // FundingHistory trait: private/get_transaction_log?query=funding
            has_ledger: true,         // AccountLedger trait: private/get_transaction_log
            has_convert: false,
            has_positions: true,      // Positions trait: futures/options/perpetuals
        }
    }

    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let _account_type = query.account_type;
        // Determine currency
        let currency = asset.map(|a| a.to_uppercase()).unwrap_or_else(|| "BTC".to_string());

        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(currency));
        params.insert("extended".to_string(), json!(false));

        let response = self.rpc_call(DeribitMethod::GetAccountSummary, params).await?;
        DeribitParser::parse_balances(&response)
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Get account summary for BTC (main currency on Deribit)
        let balances = self.get_balance(BalanceQuery { asset: Some("BTC".to_string()), account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.0, // Deribit has dynamic fees
            taker_commission: 0.0,
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Deribit: use GetAccountSummary which includes fee rates
        let currency = symbol
            .and_then(|s| s.split('/').next())
            .map(|b| b.to_uppercase())
            .unwrap_or_else(|| "BTC".to_string());

        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(currency));
        params.insert("extended".to_string(), json!(true));

        let response = self.rpc_call(DeribitMethod::GetAccountSummary, params).await?;

        // Extract fee rates from account summary
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let maker_rate = result.get("maker_commission")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0001); // Deribit default maker: 0.01%

        let taker_rate = result.get("taker_commission")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0005); // Deribit default taker: 0.05%

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(|s| s.to_string()),
            tier: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for DeribitConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            // Get single position
            let instrument_name = Self::instrument_from_symbol(&sym, account_type);
            params.insert("instrument_name".to_string(), json!(instrument_name));

            let response = self.rpc_call(DeribitMethod::GetPosition, params).await?;
            DeribitParser::parse_position(&response).map(|p| vec![p])
        } else {
            // Get all positions for BTC (default currency)
            params.insert("currency".to_string(), json!("BTC"));
            params.insert("kind".to_string(), json!("future"));

            let response = self.rpc_call(DeribitMethod::GetPositions, params).await?;
            DeribitParser::parse_positions(&response)
        }
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol_obj = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let instrument_name = Self::instrument_from_symbol(&symbol_obj, account_type);

        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), json!(instrument_name));

        let response = self.rpc_call(DeribitMethod::Ticker, params).await?;

        // Extract funding rate from ticker
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result".to_string()))?;

        let rate = result.get("current_funding")
            .or_else(|| result.get("funding_8h"))
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let next_funding_time = result.get("next_funding_rate_timestamp")
            .and_then(|v| v.as_i64());

        Ok(FundingRate {
            symbol: instrument_name,
            rate,
            next_funding_time,
            timestamp: crate::core::timestamp_millis() as i64,
        })
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();
                let instrument_name = Self::instrument_from_symbol(&symbol, account_type);

                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));
                // Use market order to close position immediately
                params.insert("type".to_string(), json!("market"));

                let _response = self.rpc_call(DeribitMethod::ClosePosition, params).await?;
                Ok(())
            }

            PositionModification::SetLeverage { .. } => {
                // Deribit uses dynamic leverage — no explicit set-leverage endpoint
                Err(ExchangeError::UnsupportedOperation(
                    "Deribit uses dynamic leverage — cannot set leverage directly".to_string()
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
impl CancelAll for DeribitConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match scope {
            CancelScope::All { symbol: None } => {
                // Cancel all orders across all instruments
                let params = HashMap::new();
                let response = self.rpc_call(DeribitMethod::CancelAll, params).await?;

                let cancelled_count = response.get("result")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                Ok(CancelAllResponse {
                    cancelled_count,
                    failed_count: 0,
                    details: vec![],
                })
            }

            CancelScope::All { symbol: Some(sym) } | CancelScope::BySymbol { symbol: sym } => {
                // Cancel all orders for a specific instrument
                let instrument_name = Self::instrument_from_symbol(&sym, account_type);
                let mut params = HashMap::new();
                params.insert("instrument_name".to_string(), json!(instrument_name));

                let response = self.rpc_call(DeribitMethod::CancelAllByInstrument, params).await?;

                let cancelled_count = response.get("result")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as u32;

                Ok(CancelAllResponse {
                    cancelled_count,
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
impl AmendOrder for DeribitConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let mut params = HashMap::new();
        params.insert("order_id".to_string(), json!(req.order_id));

        if let Some(qty) = req.fields.quantity {
            params.insert("amount".to_string(), json!(qty));
        }
        if let Some(price) = req.fields.price {
            params.insert("price".to_string(), json!(price));
        }
        if let Some(stop_price) = req.fields.trigger_price {
            params.insert("trigger_price".to_string(), json!(stop_price));
        }

        let response = self.rpc_call(DeribitMethod::Edit, params).await?;
        DeribitParser::parse_order(&response, "")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl CustodialFunds for DeribitConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        _network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        // GET /private/get_current_deposit_address — params: currency
        // Deribit does not use a separate "network" param; currency identifies the chain.
        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(asset.to_uppercase()));

        let response = self.rpc_call(DeribitMethod::GetCurrentDepositAddress, params).await?;

        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result in deposit address response".to_string()))?;

        let address = result.get("address")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing address field".to_string()))?
            .to_string();

        let tag = result.get("tag")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(DepositAddress {
            address,
            tag,
            network: None, // Deribit uses currency to identify the network
            asset: asset.to_uppercase(),
            created_at: None,
        })
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        // GET /private/withdraw — Deribit uses GET for some private endpoints
        // params: currency, address, amount, priority (optional)
        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(req.asset.to_uppercase()));
        params.insert("address".to_string(), json!(req.address));
        params.insert("amount".to_string(), json!(req.amount));

        // priority: "insane" | "extreme_high" | "very_high" | "high" | "mid" | "low" | "very_low"
        // Default to "mid" if not specified (standard fee, reasonable speed)
        params.insert("priority".to_string(), json!("mid"));

        let response = self.rpc_call(DeribitMethod::Withdraw, params).await?;

        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result in withdraw response".to_string()))?;

        let withdraw_id = result.get("id")
            .and_then(|v| v.as_u64())
            .map(|id| id.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing id in withdraw response".to_string()))?;

        let status = result.get("state")
            .and_then(|v| v.as_str())
            .unwrap_or("submitted")
            .to_string();

        let tx_hash = result.get("transaction_id")
            .and_then(|v| v.as_str())
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());

        Ok(WithdrawResponse {
            withdraw_id,
            status,
            tx_hash,
        })
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        use crate::core::types::FundsRecordType;

        // Deribit requires a currency param — default to BTC if none specified
        let currency = filter.asset
            .as_deref()
            .map(|a: &str| a.to_uppercase())
            .unwrap_or_else(|| "BTC".to_string());

        let fetch_deposits = matches!(
            filter.record_type,
            FundsRecordType::Deposit | FundsRecordType::Both
        );
        let fetch_withdrawals = matches!(
            filter.record_type,
            FundsRecordType::Withdrawal | FundsRecordType::Both
        );

        let count = filter.limit.unwrap_or(50).min(1000) as u64;
        let mut records = Vec::new();

        // Fetch deposit records
        if fetch_deposits {
            let mut params = HashMap::new();
            params.insert("currency".to_string(), json!(currency));
            params.insert("count".to_string(), json!(count));
            params.insert("offset".to_string(), json!(0u64));

            let response = self.rpc_call(DeribitMethod::GetDeposits, params).await?;

            let result = response.get("result").unwrap_or(&response);
            let data = result.get("data")
                .and_then(|v| v.as_array())
                .or_else(|| result.as_array())
                .cloned()
                .unwrap_or_default();

            for item in data {
                let id = item.get("id")
                    .and_then(|v| v.as_u64())
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                let amount = item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let tx_hash = item.get("transaction_id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let status = item.get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let timestamp = item.get("updated_timestamp")
                    .or_else(|| item.get("received_timestamp"))
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                records.push(FundsRecord::Deposit {
                    id,
                    asset: currency.clone(),
                    amount,
                    tx_hash,
                    network: None,
                    status,
                    timestamp,
                });
            }
        }

        // Fetch withdrawal records
        if fetch_withdrawals {
            let mut params = HashMap::new();
            params.insert("currency".to_string(), json!(currency));
            params.insert("count".to_string(), json!(count));
            params.insert("offset".to_string(), json!(0u64));

            let response = self.rpc_call(DeribitMethod::GetWithdrawals, params).await?;

            let result = response.get("result").unwrap_or(&response);
            let data = result.get("data")
                .and_then(|v| v.as_array())
                .or_else(|| result.as_array())
                .cloned()
                .unwrap_or_default();

            for item in data {
                let id = item.get("id")
                    .and_then(|v| v.as_u64())
                    .map(|n| n.to_string())
                    .unwrap_or_default();
                let amount = item.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
                let fee = item.get("fee").and_then(|v| v.as_f64());
                let address = item.get("address")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                let tx_hash = item.get("transaction_id")
                    .and_then(|v| v.as_str())
                    .filter(|s| !s.is_empty())
                    .map(|s| s.to_string());
                let status = item.get("state")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let timestamp = item.get("updated_timestamp")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0);

                records.push(FundsRecord::Withdrawal {
                    id,
                    asset: currency.clone(),
                    amount,
                    fee,
                    address,
                    tag: None, // Deribit doesn't use destination tags
                    tx_hash,
                    network: None,
                    status,
                    timestamp,
                });
            }
        }

        Ok(records)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXTENDED METHODS — Derivatives data & order/trade history additions
// ═══════════════════════════════════════════════════════════════════════════════

impl DeribitConnector {
    /// Funding rate history — `public/get_funding_rate_history` (public)
    ///
    /// Required: `instrument_name`, `start_timestamp` (ms), `end_timestamp` (ms).
    pub async fn get_funding_rate_history(
        &self,
        instrument_name: &str,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), serde_json::json!(instrument_name));
        params.insert("start_timestamp".to_string(), serde_json::json!(start_timestamp));
        params.insert("end_timestamp".to_string(), serde_json::json!(end_timestamp));
        self.rpc_call(DeribitMethod::GetFundingRateHistory, params).await
    }

    /// Funding rate value — `public/get_funding_rate_value` (public)
    ///
    /// Required: `instrument_name`, `start_timestamp` (ms), `end_timestamp` (ms).
    pub async fn get_funding_rate_value(
        &self,
        instrument_name: &str,
        start_timestamp: i64,
        end_timestamp: i64,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), serde_json::json!(instrument_name));
        params.insert("start_timestamp".to_string(), serde_json::json!(start_timestamp));
        params.insert("end_timestamp".to_string(), serde_json::json!(end_timestamp));
        self.rpc_call(DeribitMethod::GetFundingRateValue, params).await
    }

    /// Index price — `public/get_index_price` (public)
    ///
    /// Required: `index_name` (e.g. `btc_usd`, `eth_usd`).
    pub async fn get_index_price(&self, index_name: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("index_name".to_string(), serde_json::json!(index_name));
        self.rpc_call(DeribitMethod::GetIndexPrice, params).await
    }

    /// Historical volatility — `public/get_historical_volatility` (public)
    ///
    /// Required: `currency` (e.g. `BTC`, `ETH`).
    pub async fn get_historical_volatility(&self, currency: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), serde_json::json!(currency.to_uppercase()));
        self.rpc_call(DeribitMethod::GetHistoricalVolatility, params).await
    }

    /// Mark price history — `public/get_mark_price_history` (public)
    ///
    /// Required: `instrument_name`, `start_timestamp` (ms), `end_timestamp` (ms),
    /// `resolution` (seconds).
    pub async fn get_mark_price_history(
        &self,
        instrument_name: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        resolution: u32,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), serde_json::json!(instrument_name));
        params.insert("start_timestamp".to_string(), serde_json::json!(start_timestamp));
        params.insert("end_timestamp".to_string(), serde_json::json!(end_timestamp));
        params.insert("resolution".to_string(), serde_json::json!(resolution));
        self.rpc_call(DeribitMethod::GetMarkPriceHistory, params).await
    }

    /// Order history by currency — `private/get_order_history_by_currency` (signed)
    ///
    /// Required: `currency`. Optional: `kind`, `count`, `offset`, `include_old`,
    /// `include_unfilled`.
    pub async fn get_order_history_by_currency(
        &self,
        currency: &str,
        kind: Option<&str>,
        count: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), serde_json::json!(currency.to_uppercase()));
        if let Some(k) = kind {
            params.insert("kind".to_string(), serde_json::json!(k));
        }
        if let Some(c) = count {
            params.insert("count".to_string(), serde_json::json!(c.min(10000)));
        }
        self.rpc_call(DeribitMethod::GetOrderHistoryByCurrency, params).await
    }

    /// Order history by instrument — `private/get_order_history_by_instrument` (signed)
    ///
    /// Required: `instrument_name`. Optional: `count`, `offset`, `include_old`,
    /// `include_unfilled`.
    pub async fn get_order_history_by_instrument(
        &self,
        instrument_name: &str,
        count: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("instrument_name".to_string(), serde_json::json!(instrument_name));
        if let Some(c) = count {
            params.insert("count".to_string(), serde_json::json!(c.min(10000)));
        }
        self.rpc_call(DeribitMethod::GetOrderHistoryByInstrument, params).await
    }

    /// User trades by currency and time — `private/get_user_trades_by_currency_and_time`
    /// (signed)
    ///
    /// Required: `currency`, `start_timestamp` (ms), `end_timestamp` (ms).
    /// Optional: `kind`, `count`, `sorting`.
    pub async fn get_user_trades_by_currency_time(
        &self,
        currency: &str,
        start_timestamp: i64,
        end_timestamp: i64,
        kind: Option<&str>,
        count: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), serde_json::json!(currency.to_uppercase()));
        params.insert("start_timestamp".to_string(), serde_json::json!(start_timestamp));
        params.insert("end_timestamp".to_string(), serde_json::json!(end_timestamp));
        if let Some(k) = kind {
            params.insert("kind".to_string(), serde_json::json!(k));
        }
        if let Some(c) = count {
            params.insert("count".to_string(), serde_json::json!(c.min(10000)));
        }
        self.rpc_call(DeribitMethod::GetUserTradesByCurrencyTime, params).await
    }

    /// Trigger order history — `private/get_trigger_order_history` (signed)
    ///
    /// Required: `currency`. Optional: `instrument_name`, `count`, `continuation`.
    pub async fn get_trigger_order_history(
        &self,
        currency: &str,
        instrument_name: Option<&str>,
        count: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("currency".to_string(), serde_json::json!(currency.to_uppercase()));
        if let Some(inst) = instrument_name {
            params.insert("instrument_name".to_string(), serde_json::json!(inst));
        }
        if let Some(c) = count {
            params.insert("count".to_string(), serde_json::json!(c.min(10000)));
        }
        self.rpc_call(DeribitMethod::GetTriggerOrderHistory, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY (optional trait)
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for DeribitConnector {
    /// Get historical funding payments for the account.
    ///
    /// Uses `GET /api/v2/private/get_transaction_log` with `query=funding`.
    /// `filter.symbol` is treated as the Deribit currency (e.g. "BTC", "ETH").
    /// When `None`, defaults to "BTC".
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        // Deribit requires a currency (e.g. "BTC", "ETH", "USDC")
        let currency = filter.symbol
            .as_deref()
            .map(|s| {
                // Symbol may be "BTC-PERPETUAL" — extract the currency prefix
                s.split(&['-', '_'][..]).next().unwrap_or(s).to_uppercase()
            })
            .unwrap_or_else(|| "BTC".to_string());

        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(currency));
        params.insert("query".to_string(), json!("funding"));

        if let Some(start) = filter.start_time {
            params.insert("start_timestamp".to_string(), json!(start));
        }
        if let Some(end) = filter.end_time {
            params.insert("end_timestamp".to_string(), json!(end));
        }
        if let Some(limit) = filter.limit {
            params.insert("count".to_string(), json!(limit.min(100)));
        }

        let response = self.rpc_call(DeribitMethod::GetTransactionLog, params).await?;

        // Response: {"result": {"logs": [{...}, ...], "continuation": ...}}
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result in transaction log response".to_string()))?;

        let logs = result.get("logs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing logs array in transaction log response".to_string()))?;

        let payments = logs.iter().filter_map(|entry| {
            let timestamp = entry.get("timestamp")?.as_i64()?;
            let amount = entry.get("amount")
                .or_else(|| entry.get("change"))
                .and_then(|v| v.as_f64())?;
            let asset = entry.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or(&currency)
                .to_string();
            let instrument = entry.get("instrument_name")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            Some(FundingPayment {
                symbol: instrument,
                // Deribit transaction log doesn't include the rate directly
                funding_rate: 0.0,
                // Position size not carried in the log entry
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
impl AccountLedger for DeribitConnector {
    /// Get account ledger entries.
    ///
    /// Uses `GET /api/v2/private/get_transaction_log`.
    /// `filter.asset` is used as the Deribit currency (e.g. "BTC", "ETH").
    /// When `None`, defaults to "BTC".
    ///
    /// Deribit transaction log `type` values:
    /// - `"trade"` — fill from a trade
    /// - `"deposit"` — on-chain deposit
    /// - `"withdrawal"` — withdrawal
    /// - `"funding"` — perpetual funding payment
    /// - `"settlement"` — options/futures settlement
    /// - `"transfer"` — internal transfer
    /// - `"fee"` — fee charge
    /// - `"delivery"` — futures delivery
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let currency = filter.asset
            .as_deref()
            .unwrap_or("BTC")
            .to_uppercase();

        let mut params = HashMap::new();
        params.insert("currency".to_string(), json!(currency));

        // Map entry_type filter to Deribit query string
        if let Some(entry_type) = &filter.entry_type {
            let query = match entry_type {
                LedgerEntryType::Trade      => "trade",
                LedgerEntryType::Deposit    => "deposit",
                LedgerEntryType::Withdrawal => "withdrawal",
                LedgerEntryType::Funding    => "funding",
                LedgerEntryType::Fee        => "fee",
                LedgerEntryType::Rebate     => "rebate",
                LedgerEntryType::Transfer   => "transfer",
                LedgerEntryType::Liquidation => "liquidation",
                LedgerEntryType::Settlement  => "settlement",
                LedgerEntryType::Other(s)   => s.as_str(),
            };
            params.insert("query".to_string(), json!(query));
        }

        if let Some(start) = filter.start_time {
            params.insert("start_timestamp".to_string(), json!(start));
        }
        if let Some(end) = filter.end_time {
            params.insert("end_timestamp".to_string(), json!(end));
        }
        if let Some(limit) = filter.limit {
            params.insert("count".to_string(), json!(limit.min(100)));
        }

        let response = self.rpc_call(DeribitMethod::GetTransactionLog, params).await?;

        // Response: {"result": {"logs": [{...}], "continuation": ...}}
        let result = response.get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing result in transaction log response".to_string()))?;

        let logs = result.get("logs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing logs array in transaction log response".to_string()))?;

        let entries: Vec<LedgerEntry> = logs.iter().filter_map(|item| {
            let timestamp = item.get("timestamp")?.as_i64()?;

            let id = item.get("id")
                .and_then(|v| v.as_i64())
                .map(|n| n.to_string())
                .or_else(|| item.get("trade_id").and_then(|v| v.as_str()).map(|s| s.to_string()))
                .unwrap_or_else(|| timestamp.to_string());

            let asset = item.get("currency")
                .and_then(|v| v.as_str())
                .unwrap_or(&currency)
                .to_string();

            let amount = item.get("change")
                .or_else(|| item.get("amount"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.0);

            let balance = item.get("balance").and_then(|v| v.as_f64());

            let type_str = item.get("type")
                .and_then(|v| v.as_str())
                .unwrap_or("");

            let entry_type = classify_deribit_entry_type(type_str);

            let description = item.get("info")
                .or_else(|| item.get("note"))
                .and_then(|v| v.as_str())
                .unwrap_or(type_str)
                .to_string();

            let ref_id = item.get("trade_id")
                .or_else(|| item.get("order_id"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            Some(LedgerEntry {
                id,
                asset,
                amount,
                balance,
                entry_type,
                description,
                ref_id,
                timestamp,
            })
        }).collect();

        Ok(entries)
    }
}

/// Classify a Deribit transaction log entry type from its `type` field.
fn classify_deribit_entry_type(type_str: &str) -> LedgerEntryType {
    match type_str {
        "trade"      => LedgerEntryType::Trade,
        "deposit"    => LedgerEntryType::Deposit,
        "withdrawal" => LedgerEntryType::Withdrawal,
        "funding"    => LedgerEntryType::Funding,
        "fee"        => LedgerEntryType::Fee,
        "rebate"     => LedgerEntryType::Rebate,
        "transfer"   => LedgerEntryType::Transfer,
        "liquidation" => LedgerEntryType::Liquidation,
        "settlement" | "delivery" => LedgerEntryType::Settlement,
        other        => LedgerEntryType::Other(other.to_string()),
    }
}

