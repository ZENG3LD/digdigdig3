use std::time::Duration;

use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;

/// What kind of stream this series carries.
///
/// `Kline` carries a typed `KlineInterval` so different timeframes of the
/// same symbol get their own series. All other kinds have no extra parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Trade,
    AggTrade,
    Kline(KlineInterval),
    Ticker,
    Orderbook,
    OrderbookDelta,
    MarkPrice,
    FundingRate,
    OpenInterest,
    Liquidation,
    // --- extended stream types ---
    BlockTrade,
    AuctionEvent,
    IndexPrice,
    CompositeIndex,
    OptionGreeks,
    VolatilityIndex,
    HistoricalVolatility,
    LongShortRatio,
    TakerVolume,
    LiquidationBucket,
    Basis,
    InsuranceFund,
    OrderbookL3,
    SettlementEvent,
    MarketWarning,
    RiskLimit,
    PredictedFunding,
    FundingSettlement,
    MarkPriceKline(KlineInterval),
    IndexPriceKline(KlineInterval),
    PremiumIndexKline(KlineInterval),
    // --- derived bar aggregators (always computed from Stream::Trade) ---
    /// Range bar: a new bar opens when |trade.price − bar_open| ≥ range.
    ///
    /// `range` is expressed as a **fixed-point integer: price × 1e8**.
    /// Example: a $1.00 range on a dollar-denominated pair = `100_000_000u64`.
    /// This avoids carrying floats in `Hash`/`Eq` while keeping the unit
    /// explicit and independent of the minimum exchange tick.
    RangeBar(u64),
    /// Tick bar: a new bar closes every `n` trades.
    TickBar(u32),
    /// Volume bar: a new bar closes when cumulative volume ≥ threshold.
    ///
    /// `threshold` is expressed as a **fixed-point integer: volume × 1e8**.
    /// Example: 0.5 BTC threshold = `50_000_000u64`.
    VolumeBar(u64),
    /// Footprint bar: time-bucketed OHLCV with per-price buy/sell breakdown.
    ///
    /// Reuses `KlineInterval` for the time bucket (e.g. `"1m"`, `"5m"`).
    Footprint(KlineInterval),
    // --- private (auth-required) stream types ---
    /// Order lifecycle events (create/fill/cancel/expire).  Auth-required.
    OrderUpdate,
    /// Account balance changes.  Auth-required.
    BalanceUpdate,
    /// Futures position changes.  Auth-required.
    PositionUpdate,
}

/// Polling cadence + anti-alignment jitter for REST-only stream kinds.
///
/// Returned by [`Kind::is_poll_only`] for kinds that have no WS feed and
/// must be driven by periodic REST calls.
#[derive(Debug, Clone, Copy)]
pub struct PollSpec {
    /// How often to call the REST endpoint.
    pub cadence: Duration,
    /// Jitter applied to the FIRST tick only, expressed as percent of cadence.
    /// Prevents N symbols × M exchanges all calling REST at the same wall-clock
    /// second. Value 10 means first tick fires at `cadence ± (cadence * 10 / 100)`.
    pub jitter_pct: u8,
}

impl Kind {
    /// True for stream kinds that are computed inside Station from upstream
    /// WS-backed streams, rather than arriving directly from an exchange WS.
    ///
    /// Derived kinds bypass the `ws.subscribe(req)` path in `acquire_or_spawn`
    /// and instead use `acquire_or_spawn_derived<D>(...)`.
    pub(crate) fn is_derived(&self) -> bool {
        matches!(
            self,
            Kind::Basis
            | Kind::FundingSettlement
            | Kind::RangeBar(_)
            | Kind::TickBar(_)
            | Kind::VolumeBar(_)
            | Kind::Footprint(_)
        )
    }

    /// If this kind has no WS feed and must be driven by REST polling,
    /// returns the default cadence + jitter spec.
    ///
    /// Poll-only kinds bypass `ws.subscribe` entirely in `acquire_or_spawn`
    /// and instead use the `spawn_poller` actor path.
    pub fn is_poll_only(&self) -> Option<PollSpec> {
        match self {
            Kind::LongShortRatio => Some(PollSpec {
                cadence: Duration::from_secs(5 * 60), // 5 min bucket cadence
                jitter_pct: 10,
            }),
            Kind::TakerVolume => Some(PollSpec {
                cadence: Duration::from_secs(5 * 60),
                jitter_pct: 10,
            }),
            Kind::LiquidationBucket => Some(PollSpec {
                cadence: Duration::from_secs(5 * 60),
                jitter_pct: 10,
            }),
            Kind::HistoricalVolatility => Some(PollSpec {
                cadence: Duration::from_secs(60 * 60), // 1 h Deribit update cadence
                jitter_pct: 5,
            }),
            _ => None,
        }
    }

    /// Short kebab-case label for filesystem paths.
    pub fn slug(&self) -> String {
        match self {
            Kind::Trade => "trades".to_string(),
            Kind::AggTrade => "agg_trades".to_string(),
            Kind::Kline(iv) => format!("klines_{}", iv.as_str()),
            Kind::Ticker => "tickers".to_string(),
            Kind::Orderbook => "orderbook_snapshots".to_string(),
            Kind::OrderbookDelta => "orderbook_deltas".to_string(),
            Kind::MarkPrice => "mark_price".to_string(),
            Kind::FundingRate => "funding_rate".to_string(),
            Kind::OpenInterest => "open_interest".to_string(),
            Kind::Liquidation => "liquidations".to_string(),
            Kind::BlockTrade => "block_trades".to_string(),
            Kind::AuctionEvent => "auction_events".to_string(),
            Kind::IndexPrice => "index_price".to_string(),
            Kind::CompositeIndex => "composite_index".to_string(),
            Kind::OptionGreeks => "option_greeks".to_string(),
            Kind::VolatilityIndex => "volatility_index".to_string(),
            Kind::HistoricalVolatility => "historical_volatility".to_string(),
            Kind::LongShortRatio => "long_short_ratio".to_string(),
            Kind::TakerVolume => "taker_volume".to_string(),
            Kind::LiquidationBucket => "liquidation_bucket".to_string(),
            Kind::Basis => "basis".to_string(),
            Kind::InsuranceFund => "insurance_fund".to_string(),
            Kind::OrderbookL3 => "orderbook_l3".to_string(),
            Kind::SettlementEvent => "settlement_events".to_string(),
            Kind::MarketWarning => "market_warnings".to_string(),
            Kind::RiskLimit => "risk_limit".to_string(),
            Kind::PredictedFunding => "predicted_funding".to_string(),
            Kind::FundingSettlement => "funding_settlement".to_string(),
            Kind::MarkPriceKline(iv) => format!("mark_price_klines_{}", iv.as_str()),
            Kind::IndexPriceKline(iv) => format!("index_price_klines_{}", iv.as_str()),
            Kind::PremiumIndexKline(iv) => format!("premium_index_klines_{}", iv.as_str()),
            Kind::RangeBar(r) => format!("range_bars_{r}"),
            Kind::TickBar(n) => format!("tick_bars_{n}"),
            Kind::VolumeBar(v) => format!("volume_bars_{v}"),
            Kind::Footprint(iv) => format!("footprint_{}", iv.as_str()),
            Kind::OrderUpdate => "order_updates".to_string(),
            Kind::BalanceUpdate => "balance_updates".to_string(),
            Kind::PositionUpdate => "position_updates".to_string(),
        }
    }
}

/// Canonical series identity. Matches the MLC `BarSeriesKey` shape but is
/// generalized over data-class via `kind`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SeriesKey {
    pub exchange: ExchangeId,
    pub account_type: AccountType,
    /// Exchange-native raw symbol (already normalized via `SymbolNormalizer`).
    pub symbol: String,
    pub kind: Kind,
}

impl SeriesKey {
    pub fn new(
        exchange: ExchangeId,
        account_type: AccountType,
        symbol: impl Into<String>,
        kind: Kind,
    ) -> Self {
        Self {
            exchange,
            account_type,
            symbol: symbol.into(),
            kind,
        }
    }

    /// Filesystem-friendly account label (matches CLI `--account` spelling).
    pub fn account_label(&self) -> &'static str {
        match self.account_type {
            AccountType::Spot => "spot",
            AccountType::Margin => "margin",
            AccountType::FuturesCross => "futures_cross",
            AccountType::FuturesIsolated => "futures_isolated",
            AccountType::Earn => "earn",
            AccountType::Lending => "lending",
            AccountType::Options => "options",
            AccountType::Convert => "convert",
        }
    }

    /// Lower-case exchange label.
    pub fn exchange_label(&self) -> String {
        format!("{:?}", self.exchange).to_lowercase()
    }
}
