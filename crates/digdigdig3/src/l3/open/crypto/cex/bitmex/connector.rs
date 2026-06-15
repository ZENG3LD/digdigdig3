//! BitmexConnector — public-only CoreConnector implementation.
//!
//! Trading and account operations all return `NotSupported` (wire-not-present
//! without API credentials). The sole purpose of this connector is to satisfy
//! `CoreConnector` so the factory can wire it, while the real value is
//! delivered through `BitmexWebSocket` (PredictedFunding).

use reqwest::Client;

use crate::core::{
    ExchangeId, ExchangeType, AccountType,
    ExchangeError, ExchangeResult,
    Kline, Ticker, OrderBook, Price,
    Order, Balance, AccountInfo,
    Position, FundingRate, PublicTrade,
    OrderRequest, CancelRequest,
    BalanceQuery, PositionQuery, PositionModification,
    OrderHistoryFilter, PlaceOrderResponse, FeeInfo,
    MarketDataCapabilities, TradingCapabilities, AccountCapabilities,
    SymbolInput,
};
use crate::core::types::Liquidation;
use crate::core::traits::{
    ExchangeIdentity, MarketData, MarketDataPublic, Trading, Account, Positions,
    CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds,
    SubAccounts, FundingHistory, AccountLedger, HasCapabilities,
};
use crate::core::types::{
    ConnectorStats,
    RateLimitCapabilities, LimitModel, RestLimitPool, WsLimits,
    OrderbookCapabilities, WsBookChannel, ConnectorCapabilities, SymbolInfo,
};

// ─────────────────────────────────────────────────────────────────────────────
// Rate limit capabilities
// ─────────────────────────────────────────────────────────────────────────────

static BITMEX_POOL: &[RestLimitPool] = &[
    RestLimitPool {
        name: "public",
        // BitMEX public REST: 30 req/min on unauthenticated tier
        max_budget: 30,
        window_seconds: 60,
        is_weight: false,
        has_server_headers: true,
        server_header: Some("X-RateLimit-Remaining"),
        header_reports_used: false,
    },
];

static BITMEX_RATE_CAPS: RateLimitCapabilities = RateLimitCapabilities {
    model: LimitModel::Simple,
    rest_pools: BITMEX_POOL,
    decaying: None,
    endpoint_weights: &[],
    ws: WsLimits {
        max_connections: Some(10),
        max_subs_per_conn: Some(50),
        max_msg_per_sec: Some(10),
        max_streams_per_conn: None,
    },
};

// ─────────────────────────────────────────────────────────────────────────────
// BitmexConnector
// ─────────────────────────────────────────────────────────────────────────────

/// Minimal BitMEX connector — public market data via REST.
///
/// Trading / account methods all return `NotSupported` (require auth; wire-not-present
/// without API key). The WS side is the primary consumer surface.
pub struct BitmexConnector {
    client: Client,
    testnet: bool,
    base_url: String,
}

impl BitmexConnector {
    /// Create a public connector (no API credentials required).
    pub fn new(testnet: bool) -> Self {
        let base_url = if testnet {
            super::endpoints::REST_URL_TESTNET
        } else {
            super::endpoints::REST_URL
        };
        Self {
            client: Client::builder()
                .user_agent("digdigdig3/0.3.9")
                .build()
                .expect("reqwest client build"),
            testnet,
            base_url: base_url.to_string(),
        }
    }

    async fn get_json(&self, path: &str, query: &[(&str, &str)]) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .client
            .get(&url)
            .query(query)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status().as_u16();
            let body = resp.text().await.unwrap_or_default();
            return Err(ExchangeError::Http(format!("{status}: {body}")));
        }

        resp.json::<serde_json::Value>()
            .await
            .map_err(|e| ExchangeError::Parse(e.to_string()))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ExchangeIdentity
// ─────────────────────────────────────────────────────────────────────────────

impl ExchangeIdentity for BitmexConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::Bitmex
    }

    fn metrics(&self) -> ConnectorStats {
        ConnectorStats::default()
    }

    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        BITMEX_RATE_CAPS
    }

    fn is_testnet(&self) -> bool {
        self.testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        vec![AccountType::FuturesCross]
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::Cex
    }

    fn orderbook_capabilities(&self, _account_type: AccountType) -> OrderbookCapabilities {
        static BITMEX_WS_CHANNELS: &[WsBookChannel] = &[
            WsBookChannel::delta("orderBookL2_25", Some(25), Some(0)),
            WsBookChannel::delta("orderBookL2",    None,     Some(0)),
        ];
        OrderbookCapabilities {
            ws_depths: &[25],
            ws_default_depth: Some(25),
            rest_max_depth: Some(25),
            rest_depth_values: &[],
            supports_snapshot: true,
            supports_delta: true,
            update_speeds_ms: &[],
            default_speed_ms: None,
            ws_channels: BITMEX_WS_CHANNELS,
            checksum: None,
            has_sequence: true,
            has_prev_sequence: false,
            supports_aggregation: false,
            aggregation_levels: &[],
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MarketData
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl MarketData for BitmexConnector {
    async fn get_price(
        &self,
        symbol: SymbolInput<'_>,
        account_type: AccountType,
    ) -> ExchangeResult<Price> {
        let sym = symbol.resolve(ExchangeId::Bitmex, account_type)?;
        let v = self.get_json("/instrument", &[("symbol", sym.as_ref())]).await?;
        let arr = v.as_array().ok_or_else(|| ExchangeError::Parse("expected array".into()))?;
        let item = arr.first().ok_or_else(|| ExchangeError::NotFound("symbol not found".into()))?;
        let last = item.get("lastPrice").and_then(|x| x.as_f64())
            .ok_or_else(|| ExchangeError::Parse("missing lastPrice".into()))?;
        Ok(Price::from(last))
    }

    async fn get_orderbook(
        &self,
        _symbol: SymbolInput<'_>,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "bitmex: REST orderbook not implemented — use WS orderBookL2_25 channel".into(),
        ))
    }

    async fn get_klines(
        &self,
        symbol: SymbolInput<'_>,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        let bin_size = super::endpoints::interval_to_bin_size(interval)
            .ok_or_else(|| ExchangeError::UnsupportedOperation(
                format!("bitmex: unsupported kline interval '{interval}' (supported: 1m, 5m, 1h, 1d)"),
            ))?;
        let bin_size_ms = super::endpoints::bin_size_duration_ms(bin_size);
        let sym = symbol.resolve(ExchangeId::Bitmex, account_type)?;
        let count = limit.unwrap_or(100).to_string();
        let v = self
            .get_json(
                super::endpoints::PATH_TRADE_BUCKETED,
                &[
                    ("symbol",  sym.as_ref()),
                    ("binSize", bin_size),
                    ("count",   count.as_str()),
                    ("reverse", "true"),
                ],
            )
            .await?;
        super::parser::parse_rest_klines(&v, bin_size_ms)
    }

    async fn get_ticker(
        &self,
        symbol: SymbolInput<'_>,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let sym = symbol.resolve(ExchangeId::Bitmex, account_type)?;
        let v = self.get_json("/instrument", &[("symbol", sym.as_ref())]).await?;
        let arr = v.as_array().ok_or_else(|| ExchangeError::Parse("expected array".into()))?;
        let item = arr.first().ok_or_else(|| ExchangeError::NotFound("symbol not found".into()))?;

        let last_price = item.get("lastPrice").and_then(|x| x.as_f64()).unwrap_or(0.0);
        let bid_price = item.get("bidPrice").and_then(|x| x.as_f64());
        let ask_price = item.get("askPrice").and_then(|x| x.as_f64());
        let volume_24h = item.get("volume24h").and_then(|x| x.as_f64());

        Ok(Ticker {
            last_price,
            bid_price,
            ask_price,
            high_24h: item.get("highPrice").and_then(|x| x.as_f64()),
            low_24h: item.get("lowPrice").and_then(|x| x.as_f64()),
            volume_24h,
            quote_volume_24h: item.get("turnover24h").and_then(|x| x.as_f64()),
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: chrono::Utc::now().timestamp_millis(), ..Default::default() 
        })
    }

    async fn ping(&self) -> ExchangeResult<()> {
        self.get_json("/instrument/activeIntervals", &[]).await?;
        Ok(())
    }

    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        // GET /instrument?count=500 — returns all instruments (no status filter).
        // BitMEX paginates at 500; a second call with start=500 would be needed for
        // very large results, but the active set is well under 500 instruments.
        let v = self.get_json("/instrument", &[("count", "500")]).await?;
        let arr = v.as_array()
            .ok_or_else(|| ExchangeError::Parse("bitmex get_exchange_info: expected array".into()))?;

        let mut result = Vec::with_capacity(arr.len());
        for item in arr {
            let symbol = item.get("symbol")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();
            if symbol.is_empty() {
                continue;
            }

            // Native venue status verbatim (e.g. "Open", "Closed", "Settled", "Unlisted")
            let status = item.get("state")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let base_asset = item.get("underlying")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let quote_asset = item.get("quoteCurrency")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let tick_size = item.get("tickSize").and_then(|v| v.as_f64());
            let lot_size  = item.get("lotSize").and_then(|v| v.as_f64());

            // Native instrument type (e.g. "FFWCSX", "BIQUSD", "OPECCS", "FFICSX")
            let instrument_type = item.get("typ")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());

            // RAW passthrough — full instrument object
            let extra = item.clone();

            result.push(SymbolInfo {
                symbol,
                base_asset,
                quote_asset,
                status,
                price_precision: 8,
                quantity_precision: 0,
                min_quantity: lot_size,
                max_quantity: None,
                tick_size,
                step_size: lot_size,
                min_notional: None,
                account_type,
                instrument_type,
                extra,
            });
        }
        Ok(result)
    }

    fn market_data_capabilities(&self, _account_type: AccountType) -> MarketDataCapabilities {
        MarketDataCapabilities {
            has_ping: true,
            has_price: true,
            has_ticker: true,
            has_orderbook: false,
            has_klines: true,
            has_exchange_info: true,
            has_recent_trades: true,
            has_ws_klines: false,
            has_ws_trades: true,
            has_ws_orderbook: true,
            has_ws_ticker: true,
            supported_intervals: &["1m", "5m", "1h", "1d"],
            max_kline_limit: Some(1000),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// MarketDataPublic — recent trades, funding history, liquidation history
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl MarketDataPublic for BitmexConnector {
    async fn get_recent_trades(
        &self,
        symbol: SymbolInput<'_>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<PublicTrade>> {
        let sym = symbol.resolve(ExchangeId::Bitmex, account_type)?;
        let count = limit.unwrap_or(100).to_string();
        let v = self
            .get_json(
                super::endpoints::PATH_TRADE,
                &[
                    ("symbol",  sym.as_ref()),
                    ("count",   count.as_str()),
                    ("reverse", "true"),
                ],
            )
            .await?;
        super::parser::parse_rest_recent_trades(&v)
    }

    async fn get_funding_rate_history(
        &self,
        symbol: SymbolInput<'_>,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingRate>> {
        let sym = symbol.resolve(ExchangeId::Bitmex, account_type)?;
        let count = limit.unwrap_or(100).to_string();
        let v = self
            .get_json(
                super::endpoints::PATH_FUNDING,
                &[
                    ("symbol",  sym.as_ref()),
                    ("count",   count.as_str()),
                    ("reverse", "true"),
                ],
            )
            .await?;
        super::parser::parse_rest_funding_rate_history(&v)
    }

    async fn get_liquidation_history(
        &self,
        symbol: Option<SymbolInput<'_>>,
        _start_time: Option<i64>,
        _end_time: Option<i64>,
        limit: Option<u32>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Liquidation>> {
        let sym_str;
        let sym_ref: &str = if let Some(s) = symbol {
            sym_str = s.resolve(ExchangeId::Bitmex, account_type)?;
            sym_str.as_ref()
        } else {
            "XBTUSD"
        };
        let count = limit.unwrap_or(100).to_string();
        let v = self
            .get_json(
                super::endpoints::PATH_LIQUIDATION,
                &[
                    ("symbol",  sym_ref),
                    ("count",   count.as_str()),
                    ("reverse", "true"),
                ],
            )
            .await?;
        super::parser::parse_rest_liquidation_history(&v, sym_ref)
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Trading — all NotSupported (no auth)
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Trading for BitmexConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::NotSupported(
            "bitmex: trading requires API key authentication — public-only connector".into(),
        ))
    }

    async fn cancel_order(&self, _req: CancelRequest) -> ExchangeResult<Order> {
        Err(ExchangeError::NotSupported(
            "bitmex: trading requires API key authentication — public-only connector".into(),
        ))
    }

    async fn get_order(
        &self,
        _symbol: &str,
        _order_id: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_order requires authentication — public-only connector".into(),
        ))
    }

    async fn get_open_orders(
        &self,
        _symbol: Option<&str>,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_open_orders requires authentication — public-only connector".into(),
        ))
    }

    async fn get_order_history(
        &self,
        _filter: OrderHistoryFilter,
        _account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_order_history requires authentication — public-only connector".into(),
        ))
    }

    fn trading_capabilities(&self, _account_type: AccountType) -> TradingCapabilities {
        TradingCapabilities::none()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Account — all NotSupported
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Account for BitmexConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_balance requires authentication — public-only connector".into(),
        ))
    }

    async fn get_account_info(&self, _account_type: AccountType) -> ExchangeResult<AccountInfo> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_account_info requires authentication — public-only connector".into(),
        ))
    }

    async fn get_fees(&self, _symbol: Option<&str>) -> ExchangeResult<FeeInfo> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_fees requires authentication — public-only connector".into(),
        ))
    }

    fn account_capabilities(&self, _account_type: AccountType) -> AccountCapabilities {
        AccountCapabilities::none()
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Positions
// ─────────────────────────────────────────────────────────────────────────────

#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
impl Positions for BitmexConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::NotSupported(
            "bitmex: get_positions requires authentication — public-only connector".into(),
        ))
    }

    async fn get_funding_rate(
        &self,
        symbol: &str,
        _account_type: AccountType,
    ) -> ExchangeResult<FundingRate> {
        // BitMEX REST: /funding?symbol=<sym>&count=1&reverse=true
        let v = self.get_json("/funding", &[("symbol", symbol), ("count", "1"), ("reverse", "true")]).await?;
        let arr = v.as_array().ok_or_else(|| ExchangeError::Parse("expected array".into()))?;
        let item = arr.first().ok_or_else(|| ExchangeError::NotFound("no funding record".into()))?;

        let rate = item.get("fundingRate").and_then(|x| x.as_f64())
            .ok_or_else(|| ExchangeError::Parse("missing fundingRate".into()))?;

        let timestamp = item.get("timestamp").and_then(|x| x.as_str())
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.timestamp_millis())
            .unwrap_or(0);

        Ok(FundingRate {
            rate,
            next_funding_time: None,
            timestamp, ..Default::default() 
        })
    }

    async fn modify_position(&self, _req: PositionModification) -> ExchangeResult<()> {
        Err(ExchangeError::NotSupported(
            "bitmex: modify_position requires authentication — public-only connector".into(),
        ))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Optional operations — all default to UnsupportedOperation
// ─────────────────────────────────────────────────────────────────────────────

impl CancelAll for BitmexConnector {}
impl AmendOrder for BitmexConnector {}
impl BatchOrders for BitmexConnector {}
impl AccountTransfers for BitmexConnector {}
impl CustodialFunds for BitmexConnector {}
impl SubAccounts for BitmexConnector {}
impl FundingHistory for BitmexConnector {}
impl AccountLedger for BitmexConnector {}

// ─────────────────────────────────────────────────────────────────────────────
// HasCapabilities
// ─────────────────────────────────────────────────────────────────────────────

impl HasCapabilities for BitmexConnector {
    fn capabilities(&self) -> ConnectorCapabilities {
        ConnectorCapabilities {
            // ── MarketData (REST) ─────────────────────────────────────────────
            has_ticker: true,
            has_orderbook: false,            // REST not implemented; WS only
            has_klines: true,                // GET /trade/bucketed (1m/5m/1h/1d)
            has_recent_trades: true,         // GET /trade
            has_exchange_info: true,         // GET /instrument
            // ── MarketDataPublic (REST) ───────────────────────────────────────
            has_funding_rate_history: true,  // GET /funding
            has_liquidation_history: true,   // GET /liquidation (sparse; [] is normal)
            // mark/index/premium klines, LSR, taker, basis — wire-absent on BitMEX
            has_mark_price_klines: false,
            has_index_price_klines: false,
            has_premium_index_klines: false,
            has_long_short_ratio_history: false,
            has_taker_volume_history: false,
            has_liquidation_aggregate_history: false,
            has_basis_history: false,
            has_open_interest_history: false,
            has_premium_index: false,
            // ── WebSocket ─────────────────────────────────────────────────────
            has_websocket: true,
            has_ws_ticker: true,             // quote channel
            has_ws_trades: true,             // trade channel
            has_ws_orderbook: true,          // orderBookL2_25 channel
            has_ws_klines: false,            // tradeBin not yet wired in protocol.rs
            has_ws_mark_price: true,         // instrument channel fan-out
            has_ws_funding_rate: true,       // instrument channel fan-out
            // ── Trading — none (no auth) ──────────────────────────────────────
            has_market_order: false,
            has_limit_order: false,
            has_open_orders: false,
            has_order_history: false,
            has_user_trades: false,
            // ── Account — none ────────────────────────────────────────────────
            has_balance: false,
            has_account_info: false,
            has_fees: false,
            has_transfers: false,
            has_deposit_withdraw: false,
            has_sub_accounts: false,
            has_funding_payments: false,
            has_ledger: false,
            // ── Operations ────────────────────────────────────────────────────
            has_cancel_all: false,
            has_amend_order: false,
            has_batch_place: false,
            has_batch_cancel: false,
            // ── Positions ─────────────────────────────────────────────────────
            has_positions: false,
            has_mark_price: false,
            has_long_short_ratio: false,
            has_closed_pnl: false,
            ..Default::default()
        }
    }
}

