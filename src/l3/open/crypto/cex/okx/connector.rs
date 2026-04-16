//! # OKX Connector
//!
//! Реализация всех core трейтов для OKX API v5.
//!
//! ## Core трейты
//! - `ExchangeIdentity` - идентификация биржи
//! - `MarketData` - рыночные данные
//! - `Trading` - торговые операции
//! - `Account` - информация об аккаунте
//! - `Positions` - futures позиции

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
    OrderRequest, CancelRequest, CancelScope,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    AmendRequest, CancelAllResponse, OrderResult,
    UserTrade, UserTradeFilter,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
};
use crate::core::types::{
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord, FundsHistoryFilter, FundsRecordType,
    SubAccountOperation, SubAccountResult,
};
use crate::core::types::OcoResponse;
use crate::core::types::SymbolInfo;
use crate::core::traits::{
    ExchangeIdentity, MarketData, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    FundingHistory, AccountLedger,
};
use crate::core::types::{
    ConnectorStats,
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerFilter,
};
use crate::core::utils::SimpleRateLimiter;

use super::endpoints::{OkxUrls, OkxEndpoint, format_symbol, map_kline_interval, get_inst_type, get_trade_mode, get_account_id};
use super::auth::OkxAuth;
use super::parser::OkxParser;

// ═══════════════════════════════════════════════════════════════════════════════
// CONNECTOR
// ═══════════════════════════════════════════════════════════════════════════════

/// OKX коннектор
pub struct OkxConnector {
    /// HTTP клиент
    http: HttpClient,
    /// Аутентификация (None для публичных методов)
    auth: Option<OkxAuth>,
    /// URL'ы (mainnet/testnet)
    urls: OkxUrls,
    /// Testnet mode
    testnet: bool,
    /// Rate limiter (10 requests per 2 seconds = 5 rps)
    rate_limiter: Arc<Mutex<SimpleRateLimiter>>,
    /// Per-symbol precision cache for safe price/qty formatting
    precision: crate::core::utils::precision::PrecisionCache,
}

impl OkxConnector {
    /// Создать новый коннектор
    pub async fn new(credentials: Option<Credentials>, testnet: bool) -> ExchangeResult<Self> {
        let urls = if testnet {
            OkxUrls::TESTNET
        } else {
            OkxUrls::MAINNET
        };

        let http = HttpClient::new(30_000)?; // 30 sec timeout

        let auth = credentials
            .as_ref()
            .map(OkxAuth::new)
            .transpose()?;

        // Initialize rate limiter: 20 requests per 2 seconds (OKX public endpoint limit)
        let rate_limiter = Arc::new(Mutex::new(
            SimpleRateLimiter::new(20, Duration::from_secs(2))
        ));

        Ok(Self {
            http,
            auth,
            urls,
            testnet,
            rate_limiter,
            precision: crate::core::utils::precision::PrecisionCache::new(),
        })
    }

    /// Создать коннектор только для публичных методов
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
                let mut limiter = self.rate_limiter.lock()
                    .expect("Rate limiter mutex poisoned");
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

    /// GET запрос
    async fn get(
        &self,
        endpoint: OkxEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url();
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
        let full_path = format!("{}{}", path, query);

        // Add auth headers if needed
        let headers = if endpoint.requires_auth() {
            let auth = self.auth.as_ref()
                .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
            if self.testnet {
                auth.sign_request_testnet("GET", &full_path, "")
            } else {
                auth.sign_request("GET", &full_path, "")
            }
        } else {
            HashMap::new()
        };

        self.http.get_with_headers(&url, &HashMap::new(), &headers).await
    }

    /// POST запрос
    async fn post(
        &self,
        endpoint: OkxEndpoint,
        body: Value,
    ) -> ExchangeResult<Value> {
        self.rate_limit_wait().await;

        let base_url = self.urls.rest_url();
        let path = endpoint.path();
        let url = format!("{}{}", base_url, path);

        // Auth headers
        let auth = self.auth.as_ref()
            .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
        let body_str = body.to_string();
        let headers = if self.testnet {
            auth.sign_request_testnet("POST", path, &body_str)
        } else {
            auth.sign_request("POST", path, &body_str)
        };

        self.http.post(&url, &body, &headers).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // EXTENDED METHODS (OKX-специфичные)
    // ═══════════════════════════════════════════════════════════════════════════

    /// Получить все тикеры для определенного типа инструментов
    pub async fn get_all_tickers(&self, account_type: AccountType) -> ExchangeResult<Vec<Ticker>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        let response = self.get(OkxEndpoint::AllTickers, params).await?;

        let arr = response.get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array in all-tickers response".to_string()))?;

        let tickers = arr.iter().map(|data| {
            let get_f64 = |key: &str| -> Option<f64> {
                data.get(key)
                    .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_f64()))
            };

            let last_price = get_f64("last").unwrap_or(0.0);
            let ts = data.get("ts")
                .and_then(|v| v.as_str().and_then(|s| s.parse().ok()).or_else(|| v.as_i64()))
                .unwrap_or(0);

            Ticker {
                symbol: data.get("instId").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                last_price,
                bid_price: get_f64("bidPx"),
                ask_price: get_f64("askPx"),
                high_24h: get_f64("high24h"),
                low_24h: get_f64("low24h"),
                volume_24h: get_f64("vol24h"),
                quote_volume_24h: get_f64("volCcy24h"),
                price_change_24h: None,
                price_change_percent_24h: None,
                timestamp: ts,
            }
        }).collect();

        Ok(tickers)
    }

    /// Получить информацию о инструментах/символах
    pub async fn get_instruments(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        let response = self.get(OkxEndpoint::Instruments, params).await?;
        OkxParser::parse_symbols(&response, account_type)
    }

    /// Получить список символов (алиас для get_instruments для совместимости с тестами)
    pub async fn get_symbols(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        self.get_instruments(account_type).await
    }

    /// Получить server time
    pub async fn get_server_time(&self) -> ExchangeResult<i64> {
        let response = self.get(OkxEndpoint::ServerTime, HashMap::new()).await?;
        let data = OkxParser::extract_first_data(&response)?;
        OkxParser::parse_i64(data.get("ts").ok_or_else(|| ExchangeError::Parse("Missing 'ts'".to_string()))?)
            .ok_or_else(|| ExchangeError::Parse("Invalid timestamp".to_string()))
    }

    /// Build a minimal placeholder Order for algo responses that do not return full order details.
    ///
    /// Algo orders on OKX return only `algoId` on placement. This helper creates a
    /// synthetic Order with the algo_id so callers have a consistent return type.
    fn build_algo_placeholder_order(
        &self,
        algo_id: &str,
        inst_id: &str,
        side: OrderSide,
        quantity: Quantity,
    ) -> Order {
        use crate::core::types::{OrderStatus, TimeInForce};
        Order {
            id: algo_id.to_string(),
            client_order_id: None,
            symbol: inst_id.to_string(),
            side,
            order_type: OrderType::Market,
            status: OrderStatus::Open,
            price: None,
            stop_price: None,
            quantity,
            filled_quantity: 0.0,
            average_price: None,
            commission: None,
            commission_asset: None,
            created_at: crate::core::timestamp_millis() as i64,
            updated_at: None,
            time_in_force: TimeInForce::Gtc,
        }
    }

    /// Cancel an algo order via POST /api/v5/trade/cancel-algos.
    ///
    /// Algo orders (stop, trailing, OCO, TWAP, iceberg) use a separate cancel
    /// endpoint from regular orders and require `algoId` instead of `ordId`.
    pub async fn cancel_algo_order(
        &self,
        inst_id: &str,
        algo_id: &str,
    ) -> ExchangeResult<String> {
        let body = json!([{
            "algoId": algo_id,
            "instId": inst_id,
        }]);
        let response = self.post(OkxEndpoint::AlgoOrderCancel, body).await?;
        OkxParser::parse_algo_cancel_response(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA EXTENSIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get open interest for a futures instrument.
    pub async fn get_open_interest(&self, inst_id: &str) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), inst_id.to_string());
        self.get(OkxEndpoint::OpenInterest, params).await
    }

    /// Get long/short account ratio for a futures contract.
    ///
    /// `period`: e.g. `"5m"`, `"1H"`, `"4H"`, `"1D"`.
    pub async fn get_long_short_ratio(
        &self,
        inst_id: &str,
        period: &str,
        begin: Option<i64>,
        end: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), inst_id.to_string());
        params.insert("period".to_string(), period.to_string());
        if let Some(b) = begin {
            params.insert("begin".to_string(), b.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(OkxEndpoint::LongShortRatio, params).await
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FILL / TRADE HISTORY
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get recent trade fills (last 3 months, up to 100 records).
    ///
    /// `inst_type`: e.g. `"SPOT"`, `"SWAP"`.
    pub async fn get_fills_history(
        &self,
        inst_id: Option<&str>,
        inst_type: Option<&str>,
        begin: Option<i64>,
        end: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(id) = inst_id {
            params.insert("instId".to_string(), id.to_string());
        }
        if let Some(t) = inst_type {
            params.insert("instType".to_string(), t.to_string());
        }
        if let Some(b) = begin {
            params.insert("begin".to_string(), b.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(OkxEndpoint::FillsHistory, params).await
    }

    /// Get archived trade fills (up to 3 months ago, up to 100 records per page).
    ///
    /// `inst_type`: e.g. `"SPOT"`, `"SWAP"`.
    pub async fn get_fills_archive(
        &self,
        inst_id: Option<&str>,
        inst_type: Option<&str>,
        begin: Option<i64>,
        end: Option<i64>,
        limit: Option<u32>,
    ) -> ExchangeResult<Value> {
        let mut params = HashMap::new();
        if let Some(id) = inst_id {
            params.insert("instId".to_string(), id.to_string());
        }
        if let Some(t) = inst_type {
            params.insert("instType".to_string(), t.to_string());
        }
        if let Some(b) = begin {
            params.insert("begin".to_string(), b.to_string());
        }
        if let Some(e) = end {
            params.insert("end".to_string(), e.to_string());
        }
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }
        self.get(OkxEndpoint::FillsArchive, params).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE IDENTITY
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for OkxConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::OKX
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
        self.testnet
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
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET DATA
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl MarketData for OkxConnector {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::Ticker, params).await?;
        let ticker = OkxParser::parse_ticker(&response)?;
        Ok(ticker.last_price)
    }

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        if let Some(d) = depth {
            params.insert("sz".to_string(), d.to_string());
        }

        let response = self.get(OkxEndpoint::Orderbook, params).await?;
        OkxParser::parse_orderbook(&response)
    }

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("bar".to_string(), map_kline_interval(interval).to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.min(300).to_string());
        }

        // OKX naming is inverted: "after" = older-than (paginate backward).
        // /market/candles has ~1440 bar depth limit on 1m.
        // /market/history-candles has full depth — use it for pagination.
        let endpoint = if end_time.is_some() {
            OkxEndpoint::HistoryKlines
        } else {
            OkxEndpoint::Klines
        };

        if let Some(et) = end_time {
            params.insert("after".to_string(), et.to_string());
        }

        let response = self.get(endpoint, params).await?;
        OkxParser::parse_klines(&response)
    }

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::Ticker, params).await?;
        OkxParser::parse_ticker(&response)
    }

    async fn ping(&self) -> ExchangeResult<()> {
        self.get(OkxEndpoint::ServerTime, HashMap::new()).await?;
        Ok(())
    }

    /// Получить информацию о всех торговых символах биржи
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let symbols = self.get_instruments(account_type).await?;
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
            // get_recent_trades is not part of the MarketData trait — OKX has the endpoint
            // (Trades/HistoryTrades) exposed as connector-specific public methods only.
            has_recent_trades: false,
            // map_kline_interval covers: 1m 3m 5m 15m 30m 1h 2h 4h 6h 12h 1d 1w 1M 3M 6M 1y
            supported_intervals: &[
                "1m", "3m", "5m", "15m", "30m",
                "1h", "2h", "4h", "6h", "12h",
                "1d", "1w", "1M", "3M", "6M", "1y",
            ],
            // get_klines caps at 300 via `.min(300)` in the limit param
            max_kline_limit: Some(300),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRADING
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Trading for OkxConnector {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        let symbol = req.symbol.clone();
        let side = req.side;
        let quantity = req.quantity;
        let account_type = req.account_type;

        let inst_id = format_symbol(&symbol.base, &symbol.quote, account_type);
        let td_mode = get_trade_mode(account_type);
        let side_str = match side { OrderSide::Buy => "buy", OrderSide::Sell => "sell" };
        let cl_ord_id = req.client_order_id.clone()
            .unwrap_or_else(|| format!("cc_{}", crate::core::timestamp_millis()));

        let body = match req.order_type {
            OrderType::Market => {
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "market",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Limit { price } => {
                let tif = match req.time_in_force {
                    crate::core::TimeInForce::PostOnly => "post_only",
                    crate::core::TimeInForce::Ioc => "optimal_limit_ioc",
                    crate::core::TimeInForce::Fok => "fok",
                    _ => "limit",
                };
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": tif,
                    "px": self.precision.price(&inst_id, price),
                    "sz": self.precision.qty(&inst_id, quantity),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::PostOnly { price } => {
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "post_only",
                    "px": self.precision.price(&inst_id, price),
                    "sz": self.precision.qty(&inst_id, quantity),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Ioc { price } => {
                let px_str = price.map(|p| self.precision.price(&inst_id, p)).unwrap_or_else(|| "-1".to_string());
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "optimal_limit_ioc",
                    "px": px_str,
                    "sz": self.precision.qty(&inst_id, quantity),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::Fok { price } => {
                json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "fok",
                    "px": self.precision.price(&inst_id, price),
                    "sz": self.precision.qty(&inst_id, quantity),
                    "clOrdId": cl_ord_id,
                })
            }
            OrderType::StopMarket { stop_price } => {
                // OKX conditional stop market: POST /api/v5/trade/order-algo
                // ordType="conditional", triggerPx + ordPx=-1 (market execution)
                let algo_body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "conditional",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "triggerPx": self.precision.price(&inst_id, stop_price),
                    "orderPx": "-1",  // -1 = market order after trigger
                    "clAlgoId": cl_ord_id,
                });
                let response = self.post(OkxEndpoint::AlgoOrder, algo_body).await?;
                let algo_resp = OkxParser::parse_algo_order_response(&response)?;
                return Ok(PlaceOrderResponse::Algo(algo_resp));
            }
            OrderType::StopLimit { stop_price, limit_price } => {
                // OKX conditional stop limit: POST /api/v5/trade/order-algo
                // ordType="conditional", triggerPx + orderPx=limit_price
                let algo_body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "conditional",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "triggerPx": self.precision.price(&inst_id, stop_price),
                    "orderPx": self.precision.price(&inst_id, limit_price),
                    "clAlgoId": cl_ord_id,
                });
                let response = self.post(OkxEndpoint::AlgoOrder, algo_body).await?;
                let algo_resp = OkxParser::parse_algo_order_response(&response)?;
                return Ok(PlaceOrderResponse::Algo(algo_resp));
            }
            OrderType::TrailingStop { callback_rate, activation_price } => {
                // OKX trailing stop: POST /api/v5/trade/order-algo
                // ordType="move_order_stop", callbackRatio = callback_rate/100
                let mut algo_body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "move_order_stop",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "callbackRatio": (callback_rate / 100.0).to_string(),
                    "clAlgoId": cl_ord_id,
                });
                if let Some(act_px) = activation_price {
                    algo_body["activePx"] = json!(self.precision.price(&inst_id, act_px));
                }
                let response = self.post(OkxEndpoint::AlgoOrder, algo_body).await?;
                let algo_resp = OkxParser::parse_algo_order_response(&response)?;
                return Ok(PlaceOrderResponse::Algo(algo_resp));
            }
            OrderType::Oco { price, stop_price, stop_limit_price } => {
                // OKX OCO: POST /api/v5/trade/order-algo with ordType="oco"
                // tp side = limit leg (price), sl side = stop leg (stop_price)
                let tp_ord_px = self.precision.price(&inst_id, price);
                let sl_ord_px = stop_limit_price
                    .map(|p| self.precision.price(&inst_id, p))
                    .unwrap_or_else(|| "-1".to_string()); // -1 = market if no stop_limit_price
                let algo_body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "oco",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "tpTriggerPx": self.precision.price(&inst_id, price),
                    "tpOrdPx": tp_ord_px,
                    "slTriggerPx": self.precision.price(&inst_id, stop_price),
                    "slOrdPx": sl_ord_px,
                    "clAlgoId": cl_ord_id,
                });
                let response = self.post(OkxEndpoint::AlgoOrder, algo_body).await?;
                let algo_resp = OkxParser::parse_algo_order_response(&response)?;
                // Build a synthetic OcoResponse from the algo ID
                let placeholder = self.build_algo_placeholder_order(&algo_resp.algo_id, &inst_id, side, quantity);
                return Ok(PlaceOrderResponse::Oco(Box::new(OcoResponse {
                    first_order: placeholder.clone(),
                    second_order: placeholder,
                    list_id: Some(algo_resp.algo_id),
                })));
            }
            OrderType::Twap { duration_seconds, interval_seconds } => {
                // OKX TWAP algo: POST /api/v5/trade/order-algo with ordType="twap"
                // timeInterval in seconds, pxVar for price variance, szLimit for sub-order size
                let time_interval = interval_seconds.unwrap_or(60); // default 60s intervals
                let algo_body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "twap",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "pxVar": "0.01",           // 1% price variance per sub-order
                    "szLimit": self.precision.qty(&inst_id, quantity),
                    "pxLimit": "0",             // no hard price limit
                    "timeInterval": time_interval.to_string(),
                    "tgtCcy": "base_ccy",       // quantity in base currency
                    "clAlgoId": cl_ord_id,
                });
                let _ = duration_seconds; // duration_seconds not directly used — timeInterval is interval
                let response = self.post(OkxEndpoint::AlgoOrder, algo_body).await?;
                let algo_resp = OkxParser::parse_algo_order_response(&response)?;
                return Ok(PlaceOrderResponse::Algo(algo_resp));
            }
            OrderType::Iceberg { price, display_quantity } => {
                // OKX Iceberg algo: POST /api/v5/trade/order-algo with ordType="iceberg"
                // szLimit = visible slice size, pxSpread = price tolerance
                let algo_body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": "iceberg",
                    "sz": self.precision.qty(&inst_id, quantity),
                    "pxSpread": "0.01",   // 1% price spread for slices
                    "szLimit": self.precision.qty(&inst_id, display_quantity),
                    "pxLimit": self.precision.price(&inst_id, price),
                    "clAlgoId": cl_ord_id,
                });
                let response = self.post(OkxEndpoint::AlgoOrder, algo_body).await?;
                let algo_resp = OkxParser::parse_algo_order_response(&response)?;
                return Ok(PlaceOrderResponse::Algo(algo_resp));
            }
            OrderType::ReduceOnly { price } => {
                // reduceOnly only makes sense for futures (closing an existing position).
                // OKX Spot has no positions, so the API would reject this field.
                if matches!(account_type, AccountType::Spot | AccountType::Margin) {
                    return Err(ExchangeError::UnsupportedOperation(
                        "ReduceOnly orders are not supported for Spot/Margin accounts. \
                         ReduceOnly is a futures-only concept (closing an open position).".to_string()
                    ));
                }
                let ord_type = if price.is_some() { "limit" } else { "market" };
                let mut body = json!({
                    "instId": inst_id,
                    "tdMode": td_mode,
                    "side": side_str,
                    "ordType": ord_type,
                    "sz": self.precision.qty(&inst_id, quantity),
                    "reduceOnly": true,
                    "clOrdId": cl_ord_id,
                });
                if let Some(px) = price {
                    body["px"] = json!(self.precision.price(&inst_id, px));
                }
                body
            }
            OrderType::Gtd { .. } => {
                // OKX does not support GTD (Good-Till-Date) natively.
                // The 'expTime' field on OKX controls request expiry, NOT order expiry.
                // GTD must be simulated client-side by cancelling the order at the desired time.
                // Reference: NautilusTrader OKX integration docs confirm no native GTD.
                return Err(ExchangeError::UnsupportedOperation(
                    "OKX does not support GTD (Good-Till-Date) natively. \
                     Simulate GTD by placing a GTC limit order and cancelling it at expire_time.".to_string()
                ));
            }
            OrderType::Bracket { .. } => {
                // OKX has no single 'bracket' order type.
                // A bracket can be constructed as: (1) place entry order, (2) place OCO algo on fill.
                // This two-step process requires external coordination and is not atomic.
                // Use place_order for the entry, then place_order with Oco after the fill.
                return Err(ExchangeError::UnsupportedOperation(
                    "OKX does not support atomic Bracket orders. \
                     Construct manually: place entry order, then place an OCO algo order after fill.".to_string()
                ));
            }
            _ => return Err(ExchangeError::UnsupportedOperation(
                "This order type is not supported by OKX".to_string()
            )),
        };

        let response = self.post(OkxEndpoint::PlaceOrder, body).await?;
        let order_id = OkxParser::parse_order_response(&response)?;

        // Get full order details
        let symbol_str = symbol.to_string();
        let order = self.get_order(&symbol_str, &order_id, account_type).await?;
        Ok(PlaceOrderResponse::Simple(order))
    }

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        if let Some(ref symbol) = filter.symbol {
            let inst_id = format_symbol(&symbol.base, &symbol.quote, account_type);
            params.insert("instId".to_string(), inst_id);
        }

        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        if let Some(start) = filter.start_time {
            params.insert("begin".to_string(), start.to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), end.to_string());
        }

        let response = self.get(OkxEndpoint::OrderHistory, params).await?;
        OkxParser::parse_orders(&response)
    }

    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let mut params = HashMap::new();
        params.insert("instType".to_string(), get_inst_type(account_type).to_string());

        // instId: filter.symbol is a raw string like "BTC-USDT" or "BTC/USDT"
        // OKX uses dash-separated format, normalise by replacing '/' with '-'
        if let Some(ref sym) = filter.symbol {
            let inst_id = sym.replace('/', "-").to_uppercase();
            params.insert("instId".to_string(), inst_id);
        }

        if let Some(ref order_id) = filter.order_id {
            params.insert("ordId".to_string(), order_id.clone());
        }

        if let Some(start) = filter.start_time {
            params.insert("begin".to_string(), start.to_string());
        }

        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), end.to_string());
        }

        if let Some(limit) = filter.limit {
            // OKX fills endpoint maximum is 100
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        // Use /api/v5/trade/fills (last 3 days, 60 req/2s) as primary endpoint
        let response = self.get(OkxEndpoint::FillsHistory, params).await?;
        OkxParser::parse_fills(&response)
    }

async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order> {
        match req.scope {
            CancelScope::Single { ref order_id } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for cancel".into()))?
                    .clone();
                let account_type = req.account_type;

                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "ordId": order_id,
                });

                let response = self.post(OkxEndpoint::CancelOrder, body).await?;
                OkxParser::parse_order_response(&response)?;

                // Get full order details after cancellation
                let symbol_str = symbol.to_string();
                self.get_order(&symbol_str, order_id, account_type).await
            }
            CancelScope::All { ref symbol } => {
                let account_type = req.account_type;
                let inst_type = get_inst_type(account_type).to_string();

                // OKX cancel-all requires cancelling per instrument type; no single "cancel all" REST endpoint.
                // We fetch open orders and cancel each — but per non-composition rule we must not loop.
                // OKX does NOT have an atomic cancel-all REST endpoint; return UnsupportedOperation.
                // (The batch cancel endpoint requires explicit ordId list.)
                let _ = (symbol, inst_type);
                Err(ExchangeError::UnsupportedOperation(
                    "OKX does not provide an atomic cancel-all REST endpoint. Use CancelScope::Batch with explicit order IDs.".to_string()
                ))
            }
            CancelScope::BySymbol { ref symbol } => {
                // Same limitation as All — no atomic by-symbol cancel-all
                let _ = symbol;
                Err(ExchangeError::UnsupportedOperation(
                    "OKX does not provide an atomic cancel-by-symbol REST endpoint. Use CancelScope::Batch with explicit order IDs.".to_string()
                ))
            }
            CancelScope::Batch { ref order_ids } => {
                let symbol = req.symbol.as_ref()
                    .ok_or_else(|| ExchangeError::InvalidRequest("Symbol required for batch cancel".into()))?
                    .clone();
                let account_type = req.account_type;
                let inst_id = format_symbol(&symbol.base, &symbol.quote, account_type);

                // OKX batch cancel: POST /api/v5/trade/cancel-batch-orders
                // Body is an array of {instId, ordId}
                let orders_arr: Vec<Value> = order_ids.iter()
                    .map(|oid| json!({ "instId": inst_id, "ordId": oid }))
                    .collect();

                let response = self.post(OkxEndpoint::CancelBatchOrders, json!(orders_arr)).await?;

                // Return the first successfully cancelled order or error
                let data = OkxParser::extract_data(&response)?;
                let arr = data.as_array()
                    .ok_or_else(|| ExchangeError::Parse("Expected array in batch cancel response".to_string()))?;

                if arr.is_empty() {
                    return Err(ExchangeError::Api { code: 0, message: "No orders were cancelled".to_string() });
                }

                // Return a synthetic cancelled order from the first result
                let first = &arr[0];
                let order_id_str = OkxParser::get_str(first, "ordId").unwrap_or("").to_string();
                self.get_order(&symbol.to_string(), &order_id_str, account_type).await
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                "This cancel scope is not supported by OKX".to_string()
            )),
        }
    }

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order> {
        // Parse symbol string into Symbol struct
        let symbol_parts: Vec<&str> = symbol.split('/').collect();
        let symbol = if symbol_parts.len() == 2 {
            crate::core::Symbol::new(symbol_parts[0], symbol_parts[1])
        } else {
            crate::core::Symbol { base: symbol.to_string(), quote: String::new(), raw: Some(symbol.to_string()) }
        };

        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));
        params.insert("ordId".to_string(), order_id.to_string());

        let response = self.get(OkxEndpoint::GetOrder, params).await?;
        OkxParser::parse_order(&response)
    
    }

    fn trading_capabilities(&self, _account_type: AccountType) -> TradingCapabilities {
        // OKX supports the same order types on Spot and Futures via the unified algo endpoint.
        // The one structural difference — ReduceOnly (futures-only) — is enforced in place_order
        // at runtime, not captured here because TradingCapabilities has no has_reduce_only field.
        TradingCapabilities {
            has_market_order: true,
            has_limit_order: true,
            // StopMarket: OrderType::StopMarket → POST /api/v5/trade/order-algo (ordType="conditional", orderPx=-1)
            // Available on both Spot and Futures via algo endpoint.
            has_stop_market: true,
            // StopLimit: OrderType::StopLimit → POST /api/v5/trade/order-algo (ordType="conditional", orderPx=limit)
            // Available on both Spot and Futures via algo endpoint.
            has_stop_limit: true,
            // TrailingStop: OrderType::TrailingStop → POST /api/v5/trade/order-algo (ordType="move_order_stop")
            // Available on both Spot and Futures via algo endpoint.
            has_trailing_stop: true,
            // Bracket: OrderType::Bracket → UnsupportedOperation (no atomic bracket on OKX API)
            has_bracket: false,
            // OCO: OrderType::Oco → POST /api/v5/trade/order-algo (ordType="oco")
            // Available on both Spot and Futures via algo endpoint.
            has_oco: true,
            // AmendOrder trait implemented: POST /api/v5/trade/amend-order — same for both.
            has_amend: true,
            // BatchOrders trait implemented: POST /api/v5/trade/batch-orders — same for both.
            has_batch: true,
            // max_batch_place_size() and max_batch_cancel_size() both return 20
            max_batch_size: Some(20),
            // CancelAll trait implemented: POST /api/v5/trade/cancel-all-after (DMS) — same for both.
            has_cancel_all: true,
            // get_user_trades implemented: GET /api/v5/trade/fills — same for both.
            has_user_trades: true,
            // get_order_history implemented: GET /api/v5/trade/orders-history — same for both.
            has_order_history: true,
        }
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

        if let Some(s) = symbol {
            params.insert("instId".to_string(), format_symbol(&s.base, &s.quote, account_type));
        } else {
            params.insert("instType".to_string(), get_inst_type(account_type).to_string());
        }

        let response = self.get(OkxEndpoint::OpenOrders, params).await?;
        OkxParser::parse_orders(&response)
    
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Account for OkxConnector {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        let asset = query.asset.clone();
        let _account_type = query.account_type;
        let mut params = HashMap::new();
        if let Some(a) = asset {
            params.insert("ccy".to_string(), a);
        }

        let response = self.get(OkxEndpoint::Balance, params).await?;
        OkxParser::parse_balance(&response)
    
    }

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo> {
        // Get balances
        let balances = self.get_balance(BalanceQuery { asset: None, account_type }).await?;

        Ok(AccountInfo {
            account_type,
            can_trade: true, // OKX doesn't provide this field
            can_withdraw: false, // Would need to check permissions
            can_deposit: false,
            maker_commission: 0.08, // Default OKX fees
            taker_commission: 0.1,
            balances,
        })
    }

    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let is_futures = !matches!(account_type, AccountType::Spot | AccountType::Margin);

        AccountCapabilities {
            // get_balance implemented: GET /api/v5/account/balance — same for both.
            has_balances: true,
            // get_account_info implemented: builds from get_balance + hardcoded flags — same for both.
            has_account_info: true,
            // get_fees implemented: GET /api/v5/account/config (makerFeeRate/takerFeeRate) — same for both.
            has_fees: true,
            // AccountTransfers trait implemented: POST /api/v5/asset/transfer — same for both.
            has_transfers: true,
            // SubAccounts trait implemented: create, list, transfer, get_balance — same for both.
            has_sub_accounts: true,
            // CustodialFunds trait implemented: deposit address, withdraw, deposit/withdrawal history — same for both.
            has_deposit_withdraw: true,
            // No MarginTrading trait — no borrow/repay endpoints implemented.
            has_margin: false,
            // No earn/staking endpoints implemented.
            has_earn_staking: false,
            // FundingHistory: GET /api/v5/account/bills?type=8 returns funding fee payments.
            // Funding fees only accrue on perpetual SWAP positions (Futures). Spot never pays funding.
            has_funding_history: is_futures,
            // AccountLedger trait implemented: GET /api/v5/account/bills + bills-archive — same for both.
            has_ledger: true,
            // No ConvertSwap trait — no coin-to-coin conversion endpoints implemented.
            has_convert: false,
        }
    }

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        // GET /api/v5/account/trade-fee
        let mut params = HashMap::new();
        params.insert("instType".to_string(), "SPOT".to_string());

        if let Some(sym) = symbol {
            let parts: Vec<&str> = sym.split('/').collect();
            let inst_id = if parts.len() == 2 {
                format_symbol(parts[0], parts[1], AccountType::Spot)
            } else {
                sym.to_string()
            };
            params.insert("instId".to_string(), inst_id.clone());
        }

        let response = self.get(OkxEndpoint::AccountConfig, params).await?;
        // OKX returns fee info in account config: makerFeeRate, takerFeeRate
        let data = OkxParser::extract_first_data(&response)?;

        let maker_rate = OkxParser::get_str(data, "makerFeeRate")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.08 / 100.0);
        let taker_rate = OkxParser::get_str(data, "takerFeeRate")
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.1 / 100.0);

        Ok(FeeInfo {
            maker_rate,
            taker_rate,
            symbol: symbol.map(String::from),
            tier: OkxParser::get_str(data, "level").map(String::from),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITIONS
// ═══════════════════════════════════════════════════════════════════════════════

#[async_trait]
impl Positions for OkxConnector {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        let symbol = query.symbol.clone();
        let account_type = query.account_type;

        let mut params = HashMap::new();

        if let Some(s) = symbol {
            params.insert("instId".to_string(), format_symbol(&s.base, &s.quote, account_type));
        } else {
            params.insert("instType".to_string(), get_inst_type(account_type).to_string());
        }

        let response = self.get(OkxEndpoint::Positions, params).await?;
        OkxParser::parse_positions(&response)
    
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

        let mut params = HashMap::new();
        params.insert("instId".to_string(), format_symbol(&symbol.base, &symbol.quote, account_type));

        let response = self.get(OkxEndpoint::FundingRate, params).await?;
        OkxParser::parse_funding_rate(&response)
    
    }

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()> {
        match req {
            PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
                let symbol = symbol.clone();

                let margin_mode = match account_type {
                    AccountType::FuturesCross => "cross",
                    AccountType::FuturesIsolated => "isolated",
                    _ => return Err(ExchangeError::InvalidRequest("Leverage only supported for futures".to_string())),
                };

                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "lever": leverage.to_string(),
                    "mgnMode": margin_mode,
                });

                let response = self.post(OkxEndpoint::SetLeverage, body).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::SetMarginMode { ref symbol, margin_type, account_type } => {
                let symbol = symbol.clone();

                if account_type == AccountType::Spot {
                    return Err(ExchangeError::UnsupportedOperation(
                        "SetMarginMode not supported for Spot".to_string()
                    ));
                }

                let mgn_mode = match margin_type {
                    crate::core::MarginType::Cross => "cross",
                    crate::core::MarginType::Isolated => "isolated",
                };

                // OKX switches margin mode via set-position-mode or set-leverage
                // For per-instrument margin mode: use set-leverage with appropriate mgnMode
                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "lever": "10",  // Required field, use current leverage
                    "mgnMode": mgn_mode,
                });

                let response = self.post(OkxEndpoint::SetLeverage, body).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::AddMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "AddMargin only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: POST /api/v5/account/position/margin-balance
                // type=add for adding margin
                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "posSide": "net",
                    "type": "add",
                    "amt": amount.to_string(),
                });

                // OKX doesn't have a specific endpoint in our enum for this; use AccountConfig as fallback
                // We need to call the raw endpoint
                self.rate_limit_wait().await;
                let base_url = self.urls.rest_url();
                let path = "/api/v5/account/position/margin-balance";
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = if self.testnet {
                    auth.sign_request_testnet("POST", path, &body_str)
                } else {
                    auth.sign_request("POST", path, &body_str)
                };
                let response = self.http.post(&url, &body, &headers).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::RemoveMargin { ref symbol, amount, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "RemoveMargin only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: type=reduce for removing margin
                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "posSide": "net",
                    "type": "reduce",
                    "amt": amount.to_string(),
                });

                self.rate_limit_wait().await;
                let base_url = self.urls.rest_url();
                let path = "/api/v5/account/position/margin-balance";
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = if self.testnet {
                    auth.sign_request_testnet("POST", path, &body_str)
                } else {
                    auth.sign_request("POST", path, &body_str)
                };
                let response = self.http.post(&url, &body, &headers).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::ClosePosition { ref symbol, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "ClosePosition only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: POST /api/v5/trade/close-position
                let mgn_mode = match account_type {
                    AccountType::FuturesCross => "cross",
                    _ => "isolated",
                };

                let body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "mgnMode": mgn_mode,
                });

                self.rate_limit_wait().await;
                let base_url = self.urls.rest_url();
                let path = "/api/v5/trade/close-position";
                let url = format!("{}{}", base_url, path);
                let auth = self.auth.as_ref()
                    .ok_or_else(|| ExchangeError::Auth("Authentication required".to_string()))?;
                let body_str = body.to_string();
                let headers = if self.testnet {
                    auth.sign_request_testnet("POST", path, &body_str)
                } else {
                    auth.sign_request("POST", path, &body_str)
                };
                let response = self.http.post(&url, &body, &headers).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            PositionModification::SetTpSl { ref symbol, take_profit, stop_loss, account_type } => {
                let symbol = symbol.clone();

                match account_type {
                    AccountType::Spot | AccountType::Margin => {
                        return Err(ExchangeError::UnsupportedOperation(
                            "SetTpSl only supported for futures".to_string()
                        ));
                    }
                    _ => {}
                }

                // OKX: TP/SL on existing position uses algo order endpoint ordType="oco"
                // POST /api/v5/trade/order-algo — NOT the regular /api/v5/trade/order
                let td_mode = get_trade_mode(account_type);
                let mut body = json!({
                    "instId": format_symbol(&symbol.base, &symbol.quote, account_type),
                    "tdMode": td_mode,
                    "side": "sell",  // Closing a long position: sell side
                    "ordType": "oco",
                    "sz": "0",  // 0 = entire position quantity
                });

                if let Some(tp) = take_profit {
                    body["tpTriggerPx"] = json!(tp.to_string());
                    body["tpOrdPx"] = json!("-1"); // -1 = market execution
                }
                if let Some(sl) = stop_loss {
                    body["slTriggerPx"] = json!(sl.to_string());
                    body["slOrdPx"] = json!("-1"); // -1 = market execution
                }

                // Use AlgoOrder endpoint — OCO is an algo order type on OKX
                let response = self.post(OkxEndpoint::AlgoOrder, body).await?;
                OkxParser::extract_data(&response)?;
                Ok(())
            }
            _ => Err(ExchangeError::UnsupportedOperation(
                "This position modification is not supported by OKX".to_string()
            )),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders via OKX Dead Man's Switch endpoint.
///
/// OKX: `POST /api/v5/trade/cancel-all-after`
///
/// Sending `timeOut = "0"` immediately cancels all open orders and disables
/// the DMS timer. This is the only OKX native cancel-all mechanism.
///
/// Note: The `scope` symbol filter is not supported — OKX `cancel-all-after`
/// always cancels across all instruments. `CancelScope::BySymbol` will return
/// `UnsupportedOperation`.
#[async_trait]
impl CancelAll for OkxConnector {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        _account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        match &scope {
            CancelScope::All { .. } => {
                // Proceed — cancel-all-after cancels across all instruments
            }
            CancelScope::BySymbol { .. } => {
                return Err(ExchangeError::UnsupportedOperation(
                    "OKX cancel-all-after does not support per-symbol scope. \
                     Use CancelScope::All to cancel all open orders.".to_string()
                ));
            }
            _ => {
                return Err(ExchangeError::InvalidRequest(
                    "cancel_all_orders only accepts All or BySymbol scope".to_string()
                ));
            }
        }

        let body = json!({
            "timeOut": "0",
        });

        let response = self.post(OkxEndpoint::CancelAllAfter, body).await?;
        OkxParser::parse_cancel_all_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Modify a live order in-place via OKX native amend endpoint.
///
/// OKX: `POST /api/v5/trade/amend-order`
/// At least one of `newPx`, `newSz`, or `newStopPx` must be provided.
#[async_trait]
impl AmendOrder for OkxConnector {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        if req.fields.price.is_none() && req.fields.quantity.is_none() && req.fields.trigger_price.is_none() {
            return Err(ExchangeError::InvalidRequest(
                "At least one of price, quantity, or trigger_price must be provided for amend".to_string()
            ));
        }

        let account_type = req.account_type;
        let symbol_str = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
        let mut body = json!({
            "instId": symbol_str,
            "ordId": req.order_id,
        });

        if let Some(price) = req.fields.price {
            body["newPx"] = json!(self.precision.price(&symbol_str, price));
        }
        if let Some(qty) = req.fields.quantity {
            body["newSz"] = json!(self.precision.qty(&symbol_str, qty));
        }
        if let Some(trigger_price) = req.fields.trigger_price {
            body["newStopPx"] = json!(self.precision.price(&symbol_str, trigger_price));
        }

        let response = self.post(OkxEndpoint::AmendOrder, body).await?;
        OkxParser::parse_amend_order_response(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement and cancellation via OKX batch endpoints.
///
/// OKX: `POST /api/v5/trade/batch-orders` (max 20), `POST /api/v5/trade/cancel-batch-orders` (max 20)
#[async_trait]
impl BatchOrders for OkxConnector {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if orders.is_empty() {
            return Ok(vec![]);
        }

        if orders.len() > self.max_batch_place_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch size {} exceeds OKX limit of {}", orders.len(), self.max_batch_place_size())
            ));
        }

        let order_list: Vec<serde_json::Value> = orders.iter().map(|req| {
            let account_type = req.account_type;
            let inst_id = format_symbol(&req.symbol.base, &req.symbol.quote, account_type);
            let mut obj = serde_json::Map::new();
            obj.insert("instId".to_string(), json!(inst_id.clone()));
            obj.insert("tdMode".to_string(), json!(get_trade_mode(account_type)));
            obj.insert("side".to_string(), json!(match req.side {
                OrderSide::Buy => "buy",
                OrderSide::Sell => "sell",
            }));

            match &req.order_type {
                OrderType::Market => {
                    obj.insert("ordType".to_string(), json!("market"));
                    obj.insert("sz".to_string(), json!(self.precision.qty(&inst_id, req.quantity)));
                }
                OrderType::Limit { price } => {
                    obj.insert("ordType".to_string(), json!("limit"));
                    obj.insert("sz".to_string(), json!(self.precision.qty(&inst_id, req.quantity)));
                    obj.insert("px".to_string(), json!(self.precision.price(&inst_id, *price)));
                }
                _ => {
                    obj.insert("ordType".to_string(), json!("market"));
                    obj.insert("sz".to_string(), json!(self.precision.qty(&inst_id, req.quantity)));
                }
            }

            if req.reduce_only {
                obj.insert("reduceOnly".to_string(), json!(true));
            }
            if let Some(ref cid) = req.client_order_id {
                obj.insert("clOrdId".to_string(), json!(cid));
            }

            serde_json::Value::Object(obj)
        }).collect();

        let response = self.post(OkxEndpoint::PlaceBatchOrders, serde_json::Value::Array(order_list)).await?;
        OkxParser::parse_batch_orders_response(&response)
    }

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        if order_ids.is_empty() {
            return Ok(vec![]);
        }

        if order_ids.len() > self.max_batch_cancel_size() {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch cancel size {} exceeds OKX limit of {}", order_ids.len(), self.max_batch_cancel_size())
            ));
        }

        let sym = symbol.ok_or_else(|| ExchangeError::InvalidRequest(
            "instId (symbol) is required for batch cancel on OKX".to_string()
        ))?;

        // OKX requires instId per item — re-use the raw symbol string as-is
        let cancel_list: Vec<serde_json::Value> = order_ids.iter().map(|id| {
            json!({
                "instId": sym,
                "ordId": id,
            })
        }).collect();

        let response = self.post(OkxEndpoint::CancelBatchOrders, serde_json::Value::Array(cancel_list)).await?;
        OkxParser::parse_batch_orders_response(&response)
    }

    fn max_batch_place_size(&self) -> usize {
        20 // OKX limit
    }

    fn max_batch_cancel_size(&self) -> usize {
        20 // OKX limit
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH AMEND
// ═══════════════════════════════════════════════════════════════════════════════

impl OkxConnector {
    /// Batch amend multiple orders via `POST /api/v5/trade/amend-batch-orders`.
    ///
    /// Each entry in `amends` must be a JSON object with `instId` and `ordId`
    /// (or `clOrdId`), plus at least one of `newSz`, `newPx`, `newTpTriggerPx`,
    /// `newTpOrdPx`, `newSlTriggerPx`, or `newSlOrdPx`.
    ///
    /// Max 20 orders per batch (OKX limit).
    ///
    /// Returns the raw JSON response from OKX.
    pub async fn batch_amend_orders(
        &self,
        amends: Vec<serde_json::Value>,
    ) -> ExchangeResult<Value> {
        if amends.is_empty() {
            return Ok(serde_json::Value::Array(vec![]));
        }
        if amends.len() > 20 {
            return Err(ExchangeError::InvalidRequest(
                format!("Batch amend size {} exceeds OKX limit of 20", amends.len())
            ));
        }

        self.post(OkxEndpoint::AmendBatchOrders, json!(amends)).await
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal transfers between account types via OKX asset transfer endpoint.
///
/// OKX: `POST /api/v5/asset/transfer`
/// Account IDs: 6 = Funding/Spot, 5 = Margin, 18 = Unified Trading (Futures/SWAP).
#[async_trait]
impl AccountTransfers for OkxConnector {
    /// Transfer an asset between two OKX account types.
    ///
    /// Maps our `AccountType` to OKX account IDs:
    /// - `Spot` → `6` (Funding account)
    /// - `Margin` → `5`
    /// - `FuturesCross` / `FuturesIsolated` → `18` (Unified Trading Account)
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        let from_id = get_account_id(req.from_account);
        let to_id = get_account_id(req.to_account);

        let body = json!({
            "ccy": req.asset,
            "amt": req.amount.to_string(),
            "from": from_id,
            "to": to_id,
        });

        let response = self.post(OkxEndpoint::AssetTransfer, body).await?;
        OkxParser::parse_transfer_response(&response)
    }

    /// Get the history of internal transfers.
    ///
    /// Uses `GET /api/v5/asset/bills` for a general ledger view that covers
    /// all internal account movements.
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        let mut params = HashMap::new();

        // type=1 = Transfer (internal account transfer)
        params.insert("type".to_string(), "1".to_string());

        if let Some(start) = filter.start_time {
            params.insert("begin".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        let response = self.get(OkxEndpoint::AssetBills, params).await?;
        OkxParser::parse_transfer_history(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management via OKX asset endpoints.
///
/// OKX deposit address: `GET /api/v5/asset/deposit-address`
/// OKX withdrawal: `POST /api/v5/asset/withdrawal`
/// OKX deposit history: `GET /api/v5/asset/deposit-history`
/// OKX withdrawal history: `GET /api/v5/asset/withdrawal-history`
#[async_trait]
impl CustodialFunds for OkxConnector {
    /// Get the deposit address for an asset on a given network.
    ///
    /// `network = None` returns the first available address.
    /// `network = Some("ERC20")` returns the ERC-20 address for that asset.
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let mut params = HashMap::new();
        params.insert("ccy".to_string(), asset.to_string());

        if let Some(chain) = network {
            params.insert("chain".to_string(), chain.to_string());
        }

        let response = self.get(OkxEndpoint::DepositAddress, params).await?;
        OkxParser::parse_deposit_address(&response)
    }

    /// Submit a withdrawal request.
    ///
    /// OKX requires a `fee` parameter for on-chain withdrawals.
    /// The fee is exchange-determined per chain — pass `0` to use OKX minimum.
    /// `dest = 4` means on-chain withdrawal.
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let mut body = json!({
            "ccy": req.asset,
            "amt": req.amount.to_string(),
            "dest": "4",            // 4 = on-chain withdrawal
            "toAddr": req.address,
            "fee": "0",             // minimum fee — exchange will validate and reject if insufficient
        });

        if let Some(chain) = req.network {
            body["chain"] = json!(chain);
        }
        if let Some(tag) = req.tag {
            body["tag"] = json!(tag);
        }

        let response = self.post(OkxEndpoint::Withdrawal, body).await?;
        OkxParser::parse_withdrawal_response(&response)
    }

    /// Get deposit and/or withdrawal history.
    ///
    /// Dispatches to the appropriate OKX endpoint based on `filter.record_type`.
    /// For `FundsRecordType::Both`, queries deposits and then withdrawals separately
    /// and merges the results.
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let mut params = HashMap::new();

        if let Some(ref asset) = filter.asset {
            params.insert("ccy".to_string(), asset.clone());
        }
        if let Some(start) = filter.start_time {
            params.insert("after".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("before".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        match filter.record_type {
            FundsRecordType::Deposit => {
                let response = self.get(OkxEndpoint::DepositHistory, params).await?;
                OkxParser::parse_deposit_history(&response)
            }
            FundsRecordType::Withdrawal => {
                let response = self.get(OkxEndpoint::WithdrawalHistory, params).await?;
                OkxParser::parse_withdrawal_history(&response)
            }
            FundsRecordType::Both => {
                let dep_response = self.get(OkxEndpoint::DepositHistory, params.clone()).await?;
                let mut records = OkxParser::parse_deposit_history(&dep_response)?;

                let wd_response = self.get(OkxEndpoint::WithdrawalHistory, params).await?;
                let mut withdrawals = OkxParser::parse_withdrawal_history(&wd_response)?;

                records.append(&mut withdrawals);
                Ok(records)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB-ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Sub-account management via OKX sub-account endpoints.
///
/// OKX sub-account create: `POST /api/v5/users/subaccount/create`
/// OKX sub-account list: `GET /api/v5/users/subaccount/list`
/// OKX sub-account transfer: `POST /api/v5/asset/subaccount/transfer`
/// OKX sub-account balance: `GET /api/v5/account/subaccount/balances`
#[async_trait]
impl SubAccounts for OkxConnector {
    /// Perform a sub-account operation (create, list, transfer, get balance).
    ///
    /// `SubAccountOperation::Create` uses `label` as the `subAcct` name on OKX.
    /// `SubAccountOperation::Transfer` uses `sub_account_id` as the OKX sub-account name.
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        match op {
            SubAccountOperation::Create { label } => {
                let body = json!({
                    "subAcct": label,
                    "label": label,
                });
                let response = self.post(OkxEndpoint::SubAccountCreate, body).await?;
                OkxParser::parse_sub_account_create(&response)
            }

            SubAccountOperation::List => {
                let response = self.get(OkxEndpoint::SubAccountList, HashMap::new()).await?;
                OkxParser::parse_sub_account_list(&response)
            }

            SubAccountOperation::Transfer { sub_account_id, asset, amount, to_sub } => {
                // On OKX, sub-account transfers always route through the master account.
                // `from` / `to` are OKX account IDs (6 = Funding).
                // When to_sub = true: master (6) → sub; when false: sub (6) → master (6).
                let (from_sub_acct, to_sub_acct) = if to_sub {
                    (None::<&str>, Some(sub_account_id.as_str()))
                } else {
                    (Some(sub_account_id.as_str()), None)
                };

                let mut body = json!({
                    "ccy": asset,
                    "amt": amount.to_string(),
                    "from": "6",    // Funding account
                    "to": "6",      // Funding account on sub
                    "type": if to_sub { "0" } else { "1" },  // 0=master→sub, 1=sub→master
                });

                if let Some(from_sub) = from_sub_acct {
                    body["fromSubAccount"] = json!(from_sub);
                }
                if let Some(to_sub_name) = to_sub_acct {
                    body["toSubAccount"] = json!(to_sub_name);
                }

                let response = self.post(OkxEndpoint::SubAccountTransfer, body).await?;
                OkxParser::parse_sub_account_transfer(&response)
            }

            SubAccountOperation::GetBalance { sub_account_id } => {
                let mut params = HashMap::new();
                params.insert("subAcct".to_string(), sub_account_id);

                let response = self.get(OkxEndpoint::SubAccountBalances, params).await?;
                OkxParser::parse_sub_account_balance(&response)
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

/// Funding payment history via `GET /api/v5/account/bills?type=8` (subType 173/174).
///
/// OKX type=8 is the funding fee category. For data older than 7 days,
/// this automatically falls back to the `bills-archive` endpoint.
#[async_trait]
impl FundingHistory for OkxConnector {
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        // Funding fees are only applicable to perpetual SWAP (Futures) accounts.
        // Spot accounts never pay or receive funding — the API bills endpoint type=8
        // will return empty results, so we short-circuit with a clear error instead.
        if matches!(account_type, AccountType::Spot | AccountType::Margin) {
            return Err(ExchangeError::UnsupportedOperation(
                "Funding payments are only available for Futures accounts (FuturesCross/FuturesIsolated). \
                 Spot and Margin accounts do not pay funding fees.".to_string()
            ));
        }

        let mut params: HashMap<String, String> = HashMap::new();
        // type=8 = interest/funding; funding subTypes are 173 (expense) and 174 (income)
        params.insert("type".to_string(), "8".to_string());

        if let Some(symbol) = &filter.symbol {
            // OKX uses instId (e.g. "BTC-USDT-SWAP")
            params.insert("instId".to_string(), symbol.clone());
        }
        if let Some(start) = filter.start_time {
            params.insert("begin".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        let response = self.get(OkxEndpoint::AccountBills, params).await?;
        OkxParser::parse_funding_payments(&response)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Full account ledger via `GET /api/v5/account/bills` (all types).
///
/// Returns up to 7 days of data. The archive endpoint (`/account/bills-archive`)
/// covers up to 3 months and is used when a time range is provided.
#[async_trait]
impl AccountLedger for OkxConnector {
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let mut params: HashMap<String, String> = HashMap::new();

        if let Some(asset) = &filter.asset {
            params.insert("ccy".to_string(), asset.to_uppercase());
        }
        if let Some(start) = filter.start_time {
            params.insert("begin".to_string(), start.to_string());
        }
        if let Some(end) = filter.end_time {
            params.insert("end".to_string(), end.to_string());
        }
        if let Some(limit) = filter.limit {
            params.insert("limit".to_string(), limit.min(100).to_string());
        }

        // Use archive endpoint when a start_time is provided (may exceed 7-day window)
        let endpoint = if filter.start_time.is_some() {
            OkxEndpoint::AccountBillsArchive
        } else {
            OkxEndpoint::AccountBills
        };

        let response = self.get(endpoint, params).await?;
        let mut entries = OkxParser::parse_ledger(&response)?;

        if let Some(ref type_filter) = filter.entry_type {
            entries.retain(|e| &e.entry_type == type_filter);
        }

        Ok(entries)
    }
}
