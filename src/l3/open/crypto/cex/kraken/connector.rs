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
    Price, Kline, Ticker, OrderBook,
    Order, OrderSide, OrderType, Balance, AccountInfo,
    Position, FundingRate,
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendRequest, CancelAllResponse, OrderResult,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::types::{
    WithdrawRequest, WithdrawResponse, DepositAddress,
    FundsHistoryFilter, FundsRecord, FundsRecordType,
    SubAccountOperation, SubAccountResult,
    UserTrade, UserTradeFilter,
    FundingPayment, FundingFilter, LedgerEntry, LedgerFilter,
};
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};
use crate::core::types::ConnectorStats;
use crate::core::utils::DecayingRateLimiter;
use crate::core::utils::precision::PrecisionCache;

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
    /// Per-symbol precision cache (populated after get_exchange_info)
    precision: PrecisionCache,
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

    // ═══════════════════════════════════════════════════════════════════════════
    // FILL / TRADE HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get personal trade history (fills) for spot account.
    ///
    /// `trade_type`: optional filter — `"all"`, `"any position"`, `"closed position"`,
    /// `"closing position"`, `"no position"`.
    /// `start` and `end` are Unix timestamps (seconds).
    pub async fn get_trades_history(
        &self,
        trade_type: Option<&str>,
        start: Option<i64>,
        end: Option<i64>,
        offset: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(t) = trade_type {
            params.insert("type".to_string(), t.to_string());
        }
        if let Some(s) = start {
            params.insert("start".to_string(), s.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(o) = offset {
            params.insert("ofs".to_string(), o.to_string());
        }
        self.post(KrakenEndpoint::TradesHistory, params, AccountType::Spot).await
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

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let response = self.get_asset_pairs().await?;
        let symbols = KrakenParser::parse_exchange_info(&response, account_type)?;
        self.precision.load_from_symbols(&symbols);
        Ok(symbols)
    }

    fn market_data_capabilities(&self, _account_type: AccountType) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: true,
            has_klines: true,
            has_exchange_info: true,
            // Kraken has no public recent-trades REST endpoint in this connector.
            has_recent_trades: false,
            // Kraken OHLC intervals (integer minutes): 1, 5, 15, 30, 60, 240, 1440, 10080, 21600
            supported_intervals: &["1m", "5m", "15m", "30m", "1h", "4h", "1d", "1w", "15d"],
            // Kraken returns up to 720 candles per OHLC request.
            max_kline_limit: Some(720),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for KrakenConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);
        let side_str = match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };
        let sym = &formatted;

        // Futures endpoint selection
        let endpoint = match account_type {
            AccountType::Spot | AccountType::Margin => KrakenEndpoint::SpotAddOrder,
            _ => KrakenEndpoint::FuturesSendOrder,
        };

        let (mut params, order_type_out, price_out, stop_price_out, tif_out) = match req.order_type {
            OrderType::Market => {
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "market".to_string());
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                (p, OrderType::Market, None, None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Limit { price } => {
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "limit".to_string());
                p.insert("price".to_string(), self.precision.price(sym, price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                (p, OrderType::Limit { price }, Some(price), None, crate::core::TimeInForce::Gtc)
            }
            OrderType::PostOnly { price } => {
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "limit".to_string());
                p.insert("price".to_string(), self.precision.price(sym, price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                p.insert("oflags".to_string(), "post".to_string());
                (p, OrderType::PostOnly { price }, Some(price), None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Ioc { price } => {
                let px_val = price.unwrap_or(0.0);
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "limit".to_string());
                p.insert("price".to_string(), self.precision.price(sym, px_val));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                p.insert("timeinforce".to_string(), "IOC".to_string());
                (p, OrderType::Ioc { price }, price, None, crate::core::TimeInForce::Ioc)
            }
            OrderType::Fok { price } => {
                // Kraken does not natively support FOK; treat as IOC
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "limit".to_string());
                p.insert("price".to_string(), self.precision.price(sym, price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                p.insert("timeinforce".to_string(), "IOC".to_string());
                (p, OrderType::Fok { price }, Some(price), None, crate::core::TimeInForce::Fok)
            }
            OrderType::StopMarket { stop_price } => {
                // Kraken: ordertype=stop-loss, price=stop trigger
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "stop-loss".to_string());
                p.insert("price".to_string(), self.precision.price(sym, stop_price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                (p, OrderType::StopMarket { stop_price }, None, Some(stop_price), crate::core::TimeInForce::Gtc)
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                // Kraken: ordertype=stop-loss-limit, price=stop trigger, price2=limit price
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "stop-loss-limit".to_string());
                p.insert("price".to_string(), self.precision.price(sym, stop_price));
                p.insert("price2".to_string(), self.precision.price(sym, limit_price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                (p, OrderType::StopLimit { stop_price, limit_price }, Some(limit_price), Some(stop_price), crate::core::TimeInForce::Gtc)
            }
            OrderType::Gtd { price, expire_time } => {
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "limit".to_string());
                p.insert("price".to_string(), self.precision.price(sym, price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                // Kraken GTD: timeinforce=GTD + expiretm = Unix timestamp or +<seconds>
                p.insert("timeinforce".to_string(), "GTD".to_string());
                p.insert("expiretm".to_string(), (expire_time / 1000).to_string());
                (p, OrderType::Gtd { price, expire_time }, Some(price), None, crate::core::TimeInForce::Gtd)
            }
            OrderType::ReduceOnly { price } => {
                // Kraken Futures: reduceOnly flag
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ReduceOnly not supported for spot on Kraken".to_string()
                        ));
                    }
                    _ => {}
                }
                let ord_type = if price.is_some() { "lmt" } else { "mkt" };
                let mut p = HashMap::new();
                p.insert("symbol".to_string(), formatted.clone());
                p.insert("side".to_string(), side_str.to_string());
                p.insert("orderType".to_string(), ord_type.to_string());
                p.insert("size".to_string(), self.precision.qty(sym, quantity));
                p.insert("reduceOnly".to_string(), "true".to_string());
                if let Some(px) = price {
                    p.insert("limitPrice".to_string(), self.precision.price(sym, px));
                }
                (p, OrderType::ReduceOnly { price }, price, None, crate::core::TimeInForce::Gtc)
            }
            OrderType::Iceberg { price, display_quantity } => {
                // Kraken native iceberg: ordertype=iceberg, displayvol=visible slice size
                let mut p = HashMap::new();
                p.insert("pair".to_string(), formatted.clone());
                p.insert("type".to_string(), side_str.to_string());
                p.insert("ordertype".to_string(), "iceberg".to_string());
                p.insert("price".to_string(), self.precision.price(sym, price));
                p.insert("volume".to_string(), self.precision.qty(sym, quantity));
                p.insert("displayvol".to_string(), self.precision.qty(sym, display_quantity));
                (p, OrderType::Iceberg { price, display_quantity }, Some(price), None, crate::core::TimeInForce::Gtc)
            }
            OrderType::TrailingStop { .. } | OrderType::Oco { .. } | OrderType::Bracket { .. }
            | OrderType::Twap { .. }
            | OrderType::Oto { .. } | OrderType::ConditionalPlan { .. } | OrderType::DcaRecurring { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
                ));
            }
        };

        // For futures, rename params to Kraken Futures API format
        if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
            // Futures API uses different param names
            if let Some(pair) = params.remove("pair") {
                params.insert("symbol".to_string(), pair);
            }
            if let Some(t) = params.remove("type") {
                params.insert("side".to_string(), t);
            }
            if let Some(ot) = params.remove("ordertype") {
                let futures_type = match ot.as_str() {
                    "market" => "mkt",
                    "limit" => "lmt",
                    "stop-loss" => "stp",
                    _ => "lmt",
                };
                params.insert("orderType".to_string(), futures_type.to_string());
            }
            if let Some(vol) = params.remove("volume") {
                params.insert("size".to_string(), vol);
            }
            if let Some(px) = params.remove("price") {
                params.insert("limitPrice".to_string(), px);
            }
        }

        if let Some(ref cl_id) = req.client_order_id {
            params.insert("cl_ord_id".to_string(), cl_id.clone());
        }

        let response = self.post(endpoint, params, account_type).await?;
        let order_id = KrakenParser::parse_order_id(&response)?;

        Ok(PlaceOrderResponse::Simple(Order {
            id: order_id,
            client_order_id: req.client_order_id,
            symbol: symbol.to_string(),
            side,
            order_type: order_type_out,
            status: crate::core::OrderStatus::New,
            price: price_out,
            stop_price: stop_price_out,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: tif_out,
        }))
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        // Kraken Spot: POST /0/private/ClosedOrders
        // Kraken Futures: GET /derivatives/api/v3/fills
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                let mut params = HashMap::new();

                if let Some(start) = filter.start_time {
                    params.insert("start".to_string(), (start / 1000).to_string());
                }
                if let Some(end) = filter.end_time {
                    params.insert("end".to_string(), (end / 1000).to_string());
                }

                let response = self.post(KrakenEndpoint::SpotClosedOrders, params, account_type).await?;
                KrakenParser::parse_closed_orders(&response)
            }
            _ => {
                // Futures: GET /derivatives/api/v3/fills
                let mut params = HashMap::new();
                if let Some(start) = filter.start_time {
                    params.insert("lastFillTime".to_string(), start.to_string());
                }

                let response = self.get(KrakenEndpoint::FuturesHistory, params, account_type).await?;
                KrakenParser::parse_futures_fills(&response)
            }
        }
    }

async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                let mut params = HashMap::new();
                params.insert("txid".to_string(), order_id.to_string());

                let response = self.post(KrakenEndpoint::SpotCancelOrder, params, account_type).await?;
                KrakenParser::extract_result(&response)?;

                Ok(Order {
                    id: order_id.to_string(),
                    client_order_id: None,
                    symbol: symbol.to_string(),
                    side: OrderSide::Buy,
                    order_type: OrderType::Limit { price: 0.0 },
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
            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let mut params = HashMap::new();
                // For futures, optional symbol filter
                if let Some(sym) = symbol {
                    if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
                        params.insert("symbol".to_string(),
                            format_symbol(&sym.base, &sym.quote, account_type));
                    }
                }
                let cancel_all_endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => KrakenEndpoint::SpotCancelOrder,
                    _ => KrakenEndpoint::FuturesCancelOrder,
                };
                let response = self.post(cancel_all_endpoint, params, account_type).await?;
                let _ = response;
                let sym_str = symbol.as_ref().map(|s| s.to_string()).unwrap_or_default();
                Ok(Order {
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
                })
            }
            CancelScope::BySymbol { ref symbol } => {
                let account_type = req.account_type;
                let mut params = HashMap::new();
                if matches!(account_type, AccountType::FuturesCross | AccountType::FuturesIsolated) {
                    params.insert("symbol".to_string(),
                        format_symbol(&symbol.base, &symbol.quote, account_type));
                }
                let cancel_all_endpoint = match account_type {
                    AccountType::Spot | AccountType::Margin => KrakenEndpoint::SpotCancelOrder,
                    _ => KrakenEndpoint::FuturesCancelOrder,
                };
                let response = self.post(cancel_all_endpoint, params, account_type).await?;
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
            CancelScope::Batch { ref order_ids } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for batch cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                // Kraken Futures supports batch cancel: POST /derivatives/api/v3/cancelallorders
                // For spot, there's no native batch; return UnsupportedOperation
                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "Kraken Spot does not support batch cancel. Cancel orders individually.".to_string()
                        ));
                    }
                    _ => {}
                }

                // Futures batch: cancel each by sending multiple cancel requests (no single endpoint)
                // Per non-composition rule, return UnsupportedOperation for batch
                let _ = (order_ids, symbol);
                Err(ExchangeError::UnsupportedOperation(
                    "Kraken Futures batch cancel requires individual cancels. Use CancelScope::Single.".to_string()
                ))
            }
            CancelScope::ByLabel(_)
            | CancelScope::ByCurrencyKind { .. }
            | CancelScope::ScheduledAt(_) => Err(ExchangeError::UnsupportedOperation(
                "Kraken does not support this cancel scope".to_string()
            )),
        }
    }

    async fn get_order(
        &self,
        _symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let _symbol_parts: Vec<&str> = _symbol.split('/').collect();
        let _symbol = if _symbol_parts.len() == 2 {
            crate::core::Symbol::new(_symbol_parts[0], _symbol_parts[1])
        } else {
            crate::core::Symbol { base: _symbol.to_string(), quote: String::new(), raw: Some(_symbol.to_string()) }
        };

        let mut params = HashMap::new();
        params.insert("txid".to_string(), order_id.to_string());

        let response = self.post(KrakenEndpoint::SpotGetOrder, params, account_type).await?;
        KrakenParser::parse_order(&response, order_id)
    
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        account_type: AccountType,
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

        let params = HashMap::new();
        let response = self.post(KrakenEndpoint::SpotOpenOrders, params, account_type).await?;
        KrakenParser::parse_open_orders(&response)

    }

    /// Get personal trade fills from Kraken.
    ///
    /// Uses `POST /0/private/TradesHistory` for Spot/Margin.
    /// Futures fills use `GET /derivatives/api/v3/fills` — returns
    /// `UnsupportedOperation` since the Futures fills endpoint returns
    /// orders (not `UserTrade` format) and is already covered by
    /// `get_order_history`.
    ///
    /// Offset-based pagination: Kraken returns up to 50 records per request.
    /// When `filter.limit` exceeds 50, multiple pages are fetched automatically
    /// until the requested limit is reached or no more records exist.
    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => {
                return Err(ExchangeError::UnsupportedOperation(
                    "get_user_trades is not supported for Kraken Futures (use get_order_history)".to_string(),
                ));
            }
            _ => {}
        }

        // Kraken returns up to 50 trades per page; paginate to satisfy `limit`.
        let page_size: u32 = 50;
        let max_trades = filter.limit.unwrap_or(page_size);

        // Convert ms timestamps to Unix seconds for Kraken API.
        let start_secs = filter.start_time.map(|ms| ms / 1000);
        let end_secs = filter.end_time.map(|ms| ms / 1000);

        let mut all_trades: Vec<UserTrade> = Vec::new();
        let mut offset: u32 = 0;

        loop {
            let response = self.get_trades_history(
                None,
                start_secs.map(|s| s as i64),
                end_secs.map(|s| s as i64),
                if offset > 0 { Some(offset) } else { None },
            ).await?;

            let mut page = KrakenParser::parse_trades_history(&response)?;

            // Apply order_id filter (Kraken has no server-side filter for this).
            if let Some(ref oid) = filter.order_id {
                page.retain(|t| &t.order_id == oid);
            }

            // Apply symbol filter (Kraken has no server-side symbol filter for TradesHistory).
            if let Some(ref sym) = filter.symbol {
                let sym_upper = sym.to_uppercase();
                page.retain(|t| t.symbol.to_uppercase().contains(&sym_upper));
            }

            let page_len = page.len() as u32;
            all_trades.extend(page);

            // Stop if we have enough records or this page was smaller than a full page.
            if all_trades.len() as u32 >= max_trades || page_len < page_size {
                break;
            }

            offset += page_size;
        }

        // Truncate to requested limit.
        all_trades.truncate(max_trades as usize);

        Ok(all_trades)
    }

    fn trading_capabilities(&self, account_type: AccountType) -> TradingCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            has_stop_market: true,  // stop-loss (market trigger) implemented
            has_stop_limit: true,   // stop-loss-limit implemented
            // TrailingStop / OCO / Bracket all return UnsupportedOperation in place_order.
            has_trailing_stop: false,
            has_bracket: false,
            has_oco: false,
            // AmendOrder impl exists for both Spot (EditOrder) and Futures (editorder).
            has_amend: true,
            // Futures: native batch via /batchorder (max 10). Spot: no batch endpoint.
            has_batch: is_futures,
            max_batch_size: if is_futures { Some(10) } else { None },
            // CancelAll impl exists for both Spot (/CancelAll) and Futures (/cancelallorders).
            has_cancel_all: true,
            // get_user_trades: Spot only via TradesHistory. Futures returns UnsupportedOperation.
            has_user_trades: !is_futures,
            has_order_history: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for KrakenConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let _asset = query.asset.clone();
        let account_type = query.account_type;

        let params = HashMap::new();
        let response = self.post(KrakenEndpoint::SpotBalance, params, account_type).await?;
        KrakenParser::parse_balances(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            maker_commission: 0.16, // Kraken default maker fee (varies by tier)
            taker_commission: 0.26, // Kraken default taker fee
            balances,
        })
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // Kraken: POST /0/private/TradeVolume returns fee schedule
        let account_type = AccountType::Spot;
        let mut params = HashMap::new();

        if let Some(sym) = symbol {
            let parts: Vec<&str> = sym.split('/').collect();
            let formatted = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], account_type)
            } else {
                sym.to_string()
            };
            params.insert("pair".to_string(), formatted);
        }

        let response = self.post(KrakenEndpoint::SpotTradeBalance, params, account_type).await?;
        let result = KrakenParser::extract_result(&response)?;

        // Default Kraken fees (Starter tier: maker 0.16%, taker 0.26%)
        let maker_rate = result.get("fee")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .map(|v| v / 100.0)
            .unwrap_or(0.0016);
        let taker_rate = maker_rate; // TradeBalance returns taker fee

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(String::from),
            tier: None,
        })
    }

    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);
        AccountCapabilities {
            has_balances: true,
            has_account_info: true,
            has_fees: true,
            // No AccountTransfers trait implemented for Kraken.
            has_transfers: false,
            // SubAccounts: List and Transfer work for both. Create/GetBalance always UnsupportedOperation.
            has_sub_accounts: true,
            // CustodialFunds endpoints are Spot-only (/DepositAddresses, /Withdraw, etc.).
            has_deposit_withdraw: !is_futures,
            // No dedicated MarginTrading trait implemented.
            has_margin: false,
            // No EarnStaking trait implemented.
            has_earn_staking: false,
            // FundingHistory uses /Ledgers (type=rollover) — Spot endpoint only.
            has_funding_history: !is_futures,
            // AccountLedger uses /Ledgers — Spot endpoint only.
            has_ledger: !is_futures,
            // No ConvertSwap trait implemented.
            has_convert: false,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for KrakenConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let _symbol = query.symbol.clone();
        let account_type = query.account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Positions not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let response = self.get(
            KrakenEndpoint::FuturesOpenPositions,
            HashMap::new(),
            account_type,
        ).await?;

        KrakenParser::parse_futures_positions(&response)
    
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // Parse symbol string into Symbol struct
        let symbol_str = symbol;
        let symbol = {
            let parts: Vec<&str> = symbol_str.split('/').collect();
            if parts.len() == 2 {
                crate::core::Symbol::new(parts[0], parts[1])
            } else {
                crate::core::Symbol { base: symbol_str.to_string(), quote: String::new(), raw: Some(symbol_str.to_string()) }
            }
        };

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Funding rate not supported for Spot/Margin".to_string()
                ));
            }
            _ => {}
        }

        let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

        let mut params = HashMap::new();
        params.insert("symbol".to_string(), formatted.clone());

        let response = self.get(
            KrakenEndpoint::FuturesHistoricalFunding,
            params,
            account_type,
        ).await?;

        KrakenParser::parse_funding_rate(&response, &formatted)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "Leverage not supported for Spot/Margin".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("maxLeverage".to_string(), leverage.to_string());

                let response = self.post(KrakenEndpoint::FuturesSetLeverage, params, account_type).await?;
                KrakenParser::extract_futures_data(&response)?;
                Ok(())
            }
            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition only supported for futures on Kraken".to_string()
                        ));
                    }
                    _ => {}
                }

                let formatted = format_symbol(&symbol.base, &symbol.quote, account_type);

                let mut params = HashMap::new();
                params.insert("symbol".to_string(), formatted);
                params.insert("orderType".to_string(), "mkt".to_string());
                // Kraken Futures: send order with reduceOnly to close
                params.insert("reduceOnly".to_string(), "true".to_string());
                // Side will be auto-determined; we send a nominal "buy" which gets overridden by reduceOnly
                params.insert("side".to_string(), "buy".to_string());
                params.insert("size".to_string(), "0".to_string()); // 0 = entire position for some exchanges

                let response = self.post(KrakenEndpoint::FuturesSendOrder, params, account_type).await?;
                KrakenParser::extract_futures_data(&response)?;
                Ok(())
            }
            PositionModification::SetMarginMode { .. }
            | PositionModification::AddMargin { .. }
            | PositionModification::RemoveMargin { .. }
            | PositionModification::SetTpSl { .. }
            | PositionModification::SwitchPositionMode { .. }
            | PositionModification::MovePositions { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "This position modification is not supported on Kraken".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders across all symbols.
///
/// - Spot:    `POST /0/private/CancelAll`
/// - Futures: `POST /derivatives/api/v3/cancelallorders` (optionally filtered by symbol)
#[async_trait]
impl CancelAll for KrakenConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let symbol = match &scope {
            CancelScope::All { symbol } => symbol.clone(),
            CancelScope::BySymbol { symbol } => Some(symbol.clone()),
            _ => {
                return Err(ExchangeError::InvalidRequest(
                    "cancel_all_orders only accepts All or BySymbol scope".to_string()
                ));
            }
        };

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                // Spot CancelAll does not support per-symbol filtering
                let response = self.post(KrakenEndpoint::SpotCancelAll, HashMap::new(), account_type).await?;
                KrakenParser::parse_cancel_all_response(&response)
            }
            _ => {
                // Futures: optional symbol filter
                let mut params = HashMap::new();
                if let Some(sym) = symbol {
                    params.insert(
                        "symbol".to_string(),
                        format_symbol(&sym.base, &sym.quote, account_type),
                    );
                }
                let response = self.post(KrakenEndpoint::FuturesCancelOrder, params, account_type).await?;
                KrakenParser::parse_futures_cancel_all_response(&response)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Amend a live order in-place.
///
/// - Spot:    `POST /0/private/EditOrder`
/// - Futures: `POST /derivatives/api/v3/editorder`
#[async_trait]
impl AmendOrder for KrakenConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        if req.fields.price.is_none() && req.fields.quantity.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price or quantity must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let formatted = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
        let symbol_str = req.symbol.to_string();

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                // Kraken Spot EditOrder: POST /0/private/EditOrder
                let mut params = HashMap::new();
                params.insert("txid".to_string(), req.order_id.clone());
                params.insert("pair".to_string(), formatted.clone());

                if let Some(price) = req.fields.price {
                    params.insert("price".to_string(), self.precision.price(&formatted, price));
                }
                if let Some(qty) = req.fields.quantity {
                    params.insert("volume".to_string(), self.precision.qty(&formatted, qty));
                }

                let response = self.post(KrakenEndpoint::SpotEditOrder, params, account_type).await?;
                KrakenParser::parse_amend_spot_order(&response, &symbol_str)
            }
            _ => {
                // Kraken Futures editorder
                let mut params = HashMap::new();
                params.insert("orderId".to_string(), req.order_id.clone());
                params.insert("symbol".to_string(), formatted.clone());

                if let Some(price) = req.fields.price {
                    params.insert("limitPrice".to_string(), self.precision.price(&formatted, price));
                }
                if let Some(qty) = req.fields.quantity {
                    params.insert("size".to_string(), self.precision.qty(&formatted, qty));
                }

                let response = self.post(KrakenEndpoint::FuturesEditOrder, params, account_type).await?;
                KrakenParser::parse_amend_futures_order(&response, &symbol_str)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement (Futures only).
///
/// Kraken Futures: `POST /derivatives/api/v3/batchorder` — max 10 orders per batch.
/// Spot does NOT have a native batch placement endpoint.
#[async_trait]
impl BatchOrders for KrakenConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        let account_type = orders[0].account_type;

        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Batch orders not supported on Kraken Spot (futures only)".to_string()
                ));
            }
            _ => {}
        }

        if orders.len() > self.max_batch_place_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds Kraken Futures limit of {}", orders.len(), self.max_batch_place_size())
            ));
        }

        // Kraken Futures batchorder: POST with JSON body containing orders array
        let batch_json: Vec<serde_json::Value> = orders.iter().map(|req| {
            let formatted = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
            let side_str = match req.side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };

            let mut obj = json!({
                "order": "send",
                "symbol": formatted,
                "side": side_str,
                "size": req.quantity as i64,
            });
            match req.order_type {
                OrderType::Market => {
                    obj["orderType"] = json!("mkt");
                }
                OrderType::Limit { price } => {
                    obj["orderType"] = json!("lmt");
                    obj["limitPrice"] = json!(self.precision.price(&formatted, price));
                }
                _ => {
                    obj["orderType"] = json!("mkt");
                }
            }
            if req.reduce_only {
                obj["reduceOnly"] = json!(true);
            }
            if let Some(ref cid) = req.client_order_id {
                obj["cl_ord_id"] = json!(cid);
            }
            obj
        }).collect();

        let mut params = HashMap::new();
        let batch_str = serde_json::to_string(&batch_json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize batch orders: {}", e)))?;
        params.insert("json".to_string(), batch_str);

        let response = self.post(KrakenEndpoint::FuturesBatchOrder, params, account_type).await?;
        KrakenParser::parse_batch_orders_response(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        _symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        match account_type {
            AccountType::Spot | AccountType::Margin => {
                return Err(ExchangeError::UnsupportedOperation(
                    "Batch cancel not supported on Kraken Spot".to_string()
                ));
            }
            _ => {}
        }

        // Futures batchorder with cancel operations
        let cancel_json: Vec<serde_json::Value> = order_ids.iter().map(|id| {
            json!({
                "order": "cancel",
                "order_id": id,
            })
        }).collect();

        let mut params = HashMap::new();
        let batch_str = serde_json::to_string(&cancel_json)
            .map_err(|e| ExchangeError::Parse(format!("Failed to serialize cancel batch: {}", e)))?;
        params.insert("json".to_string(), batch_str);

        let response = self.post(KrakenEndpoint::FuturesBatchOrder, params, account_type).await?;
        KrakenParser::parse_batch_orders_response(&response)
    }

    fn max_batch_place_size(&self) -> usize {
        10 // Kraken Futures batchorder limit
    }

    fn max_batch_cancel_size(&self) -> usize {
        10 // Kraken Futures batchorder limit
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management for Kraken.
///
/// - Deposit address: `POST /0/private/DepositAddresses`
/// - Withdraw:        `POST /0/private/Withdraw`
/// - Deposit history: `POST /0/private/DepositStatus`
/// - Withdrawal hist: `POST /0/private/WithdrawStatus`
///
/// Note: Kraken asset names use internal format — XXBT for BTC, ZUSD for USD.
/// This implementation maps common tickers to Kraken's format.
#[async_trait]
impl CustodialFunds for KrakenConnector {
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        // Map common asset names to Kraken internal format
        let kraken_asset = map_asset_to_kraken(asset);

        let mut params = HashMap::new();
        params.insert("asset".to_string(), kraken_asset.to_string());
        if let Some(method) = network {
            params.insert("method".to_string(), method.to_string());
        }
        // new=true requests a fresh address instead of reusing an existing one
        params.insert("new".to_string(), "false".to_string());

        let response = self.post(
            KrakenEndpoint::SpotDepositAddresses,
            params,
            AccountType::Spot,
        ).await?;

        KrakenParser::parse_deposit_address(&response, asset)
    }

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let kraken_asset = map_asset_to_kraken(&req.asset);

        let mut params = HashMap::new();
        params.insert("asset".to_string(), kraken_asset.to_string());
        // Kraken uses a pre-registered withdrawal address "key" (name), not raw address
        // We use the address field as the key name
        params.insert("key".to_string(), req.address.clone());
        params.insert("amount".to_string(), req.amount.to_string());

        let response = self.post(
            KrakenEndpoint::SpotWithdraw,
            params,
            AccountType::Spot,
        ).await?;

        KrakenParser::parse_withdraw_response(&response)
    }

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let asset = filter.asset.as_deref().unwrap_or("");
        let kraken_asset = if asset.is_empty() {
            String::new()
        } else {
            map_asset_to_kraken(asset)
        };

        match filter.record_type {
            FundsRecordType::Deposit => {
                let mut params = HashMap::new();
                if !kraken_asset.is_empty() {
                    params.insert("asset".to_string(), kraken_asset.to_string());
                }
                let response = self.post(
                    KrakenEndpoint::SpotDepositStatus,
                    params,
                    AccountType::Spot,
                ).await?;
                KrakenParser::parse_deposit_history(&response)
            }
            FundsRecordType::Withdrawal => {
                let mut params = HashMap::new();
                if !kraken_asset.is_empty() {
                    params.insert("asset".to_string(), kraken_asset.to_string());
                }
                let response = self.post(
                    KrakenEndpoint::SpotWithdrawStatus,
                    params,
                    AccountType::Spot,
                ).await?;
                KrakenParser::parse_withdrawal_history(&response)
            }
            FundsRecordType::Both => {
                // Fetch both and combine
                let mut deposits_params = HashMap::new();
                let mut withdrawals_params = HashMap::new();
                if !kraken_asset.is_empty() {
                    deposits_params.insert("asset".to_string(), kraken_asset.to_string());
                    withdrawals_params.insert("asset".to_string(), kraken_asset.to_string());
                }
                let dep_response = self.post(
                    KrakenEndpoint::SpotDepositStatus,
                    deposits_params,
                    AccountType::Spot,
                ).await?;
                let wit_response = self.post(
                    KrakenEndpoint::SpotWithdrawStatus,
                    withdrawals_params,
                    AccountType::Spot,
                ).await?;

                let mut records = KrakenParser::parse_deposit_history(&dep_response)?;
                records.extend(KrakenParser::parse_withdrawal_history(&wit_response)?);
                Ok(records)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB-ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Sub-account management for Kraken.
///
/// Kraken supports listing sub-accounts and transferring funds between them
/// via the standard private REST API. Creating sub-accounts and querying
/// individual balances are not available through the standard API.
#[async_trait]
impl SubAccounts for KrakenConnector {
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        match op {
            SubAccountOperation::List => {
                let response = self.post(
                    KrakenEndpoint::SpotListSubaccounts,
                    HashMap::new(),
                    AccountType::Spot,
                ).await?;
                KrakenParser::parse_list_subaccounts(&response)
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                let kraken_asset = map_asset_to_kraken(&asset);
                let mut params = HashMap::new();
                params.insert("asset".to_string(), kraken_asset.to_string());
                params.insert("amount".to_string(), amount.to_string());
                params.insert("subaccount".to_string(), sub_account_id.clone());

                let endpoint = if to_sub {
                    KrakenEndpoint::SpotTransferToSubaccount
                } else {
                    KrakenEndpoint::SpotTransferFromSubaccount
                };

                let response = self.post(endpoint, params, AccountType::Spot).await?;
                KrakenParser::parse_subaccount_transfer(&response)
            }

            SubAccountOperation::Create { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Kraken does not support sub-account creation via standard API".to_string()
                ))
            }

            SubAccountOperation::GetBalance { .. } => {
                Err(ExchangeError::UnsupportedOperation(
                    "Kraken does not support per-sub-account balance queries via standard API".to_string()
                ))
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ASSET NAME MAPPING
// ═══════════════════════════════════════════════════════════════════════════════

/// Map common asset ticker to Kraken's internal asset name.
///
/// Kraken uses non-standard names for some assets:
/// - BTC → XXBT
/// - USD → ZUSD
/// - EUR → ZEUR
///
/// For all other assets the ticker is returned as-is (uppercased).
fn map_asset_to_kraken(asset: &str) -> String {
    match asset.to_uppercase().as_str() {
        "BTC" | "XBT" => "XXBT".to_string(),
        "ETH" => "XETH".to_string(),
        "LTC" => "XLTC".to_string(),
        "XRP" => "XXRP".to_string(),
        "USD" => "ZUSD".to_string(),
        "EUR" => "ZEUR".to_string(),
        "GBP" => "ZGBP".to_string(),
        "CAD" => "ZCAD".to_string(),
        "JPY" => "ZJPY".to_string(),
        // For assets not in the map, pass through as-is
        other => other.to_string(),
    }
}

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

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl FundingHistory for KrakenConnector {
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        let mut params = HashMap::new();
        params.insert("type".to_string(), "rollover".to_string());
        if let Some(start) = filter.start_time {
            // Kraken expects seconds
            params.insert("start".to_string(), (start / 1000).to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), (end / 1000).to_string());
        }

        let response = self
            .post(KrakenEndpoint::SpotLedgers, params, AccountType::Spot)
            .await?;
        KrakenParser::parse_funding_payments(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl AccountLedger for KrakenConnector {
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let mut params = HashMap::new();
        if let Some(asset) = &filter.asset {
            params.insert("asset".to_string(), asset.clone());
        }
        if let Some(start) = filter.start_time {
            params.insert("start".to_string(), (start / 1000).to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), (end / 1000).to_string());
        }
        // Pagination offset
        params.insert("ofs".to_string(), "0".to_string());

        let response = self
            .post(KrakenEndpoint::SpotLedgers, params, AccountType::Spot)
            .await?;
        KrakenParser::parse_ledger(&response)
    }
}
