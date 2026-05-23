use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3::core::websocket::KlineInterval;

use crate::data::{
    AggTradePoint, AuctionEventPoint, BarPoint, BasisPoint, BlockTradePoint, CompositeIndexPoint,
    FundingRatePoint, FundingSettlementPoint, HistoricalVolatilityPoint, IndexPriceKlinePoint,
    IndexPricePoint, InsuranceFundPoint, LiquidationPoint, MarkPriceKlinePoint, MarkPricePoint,
    MarketWarningPoint, ObDeltaPoint, ObSnapshotPoint, OpenInterestPoint, OptionGreeksPoint,
    OrderbookL3Point, PredictedFundingPoint, PremiumIndexKlinePoint, RiskLimitPoint,
    SettlementEventPoint, TickerPoint, TradePoint, VolatilityIndexPoint,
};
use crate::series::{Kind, SeriesKey};

/// User-facing stream class to request in a `SubscriptionSet`.
///
/// `Kline` carries a typed `KlineInterval` (e.g. `KlineInterval::new("1m")`).
/// All other variants are parameterless.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stream {
    Ticker,
    Trade,
    Orderbook,
    OrderbookDelta,
    Kline(KlineInterval),
    MarkPrice,
    FundingRate,
    OpenInterest,
    Liquidation,
    AggTrade,
    // --- extended stream types ---
    BlockTrade,
    IndexPrice,
    CompositeIndex,
    OptionGreeks,
    VolatilityIndex,
    HistoricalVolatility,
    Basis,
    InsuranceFund,
    OrderbookL3,
    SettlementEvent,
    AuctionEvent,
    MarketWarning,
    RiskLimit,
    PredictedFunding,
    FundingSettlement,
    MarkPriceKline(KlineInterval),
    IndexPriceKline(KlineInterval),
    PremiumIndexKline(KlineInterval),
}

impl Stream {
    pub(crate) fn to_kind(&self) -> Kind {
        match self {
            Stream::Trade => Kind::Trade,
            Stream::AggTrade => Kind::AggTrade,
            Stream::Kline(iv) => Kind::Kline(iv.clone()),
            Stream::Ticker => Kind::Ticker,
            Stream::Orderbook => Kind::Orderbook,
            Stream::OrderbookDelta => Kind::OrderbookDelta,
            Stream::MarkPrice => Kind::MarkPrice,
            Stream::FundingRate => Kind::FundingRate,
            Stream::OpenInterest => Kind::OpenInterest,
            Stream::Liquidation => Kind::Liquidation,
            Stream::BlockTrade => Kind::BlockTrade,
            Stream::IndexPrice => Kind::IndexPrice,
            Stream::CompositeIndex => Kind::CompositeIndex,
            Stream::OptionGreeks => Kind::OptionGreeks,
            Stream::VolatilityIndex => Kind::VolatilityIndex,
            Stream::HistoricalVolatility => Kind::HistoricalVolatility,
            Stream::Basis => Kind::Basis,
            Stream::InsuranceFund => Kind::InsuranceFund,
            Stream::OrderbookL3 => Kind::OrderbookL3,
            Stream::SettlementEvent => Kind::SettlementEvent,
            Stream::AuctionEvent => Kind::AuctionEvent,
            Stream::MarketWarning => Kind::MarketWarning,
            Stream::RiskLimit => Kind::RiskLimit,
            Stream::PredictedFunding => Kind::PredictedFunding,
            Stream::FundingSettlement => Kind::FundingSettlement,
            Stream::MarkPriceKline(iv) => Kind::MarkPriceKline(iv.clone()),
            Stream::IndexPriceKline(iv) => Kind::IndexPriceKline(iv.clone()),
            Stream::PremiumIndexKline(iv) => Kind::PremiumIndexKline(iv.clone()),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) exchange: ExchangeId,
    pub(crate) symbol: String,
    pub(crate) account_type: AccountType,
    pub(crate) streams: Vec<Stream>,
    /// If true, `symbol` is the raw exchange-native string and must be
    /// passed through to the WS connector verbatim — no `SymbolNormalizer`
    /// translation. Set by `SubscriptionSet::add_raw`. Used for exotic
    /// instrument IDs that don't fit the canonical BASE-QUOTE shape
    /// (Deribit options like `BTC-23MAY26-86000-C`, exchange-specific
    /// suffixes, etc.).
    pub(crate) is_raw: bool,
}

/// Declarative subscription request — built up fluently, consumed by
/// [`crate::Station::subscribe`].
#[derive(Debug, Default, Clone)]
pub struct SubscriptionSet {
    pub(crate) entries: Vec<Entry>,
}

impl SubscriptionSet {
    pub fn new() -> Self { Self::default() }

    /// Add a subscription. `symbol` is canonical (e.g. `"BTC-USDT"`,
    /// `"BTCUSDT"`, `"BTC/USDT"`) — it is parsed into a canonical
    /// `Symbol` and translated to the exchange-native form via
    /// `SymbolNormalizer`. Use [`Self::add_raw`] for instrument IDs that
    /// don't fit the canonical BASE-QUOTE shape (Deribit options,
    /// exchange-specific futures suffixes, etc.).
    pub fn add(
        mut self,
        exchange: ExchangeId,
        symbol: impl Into<String>,
        account_type: AccountType,
        streams: impl IntoIterator<Item = Stream>,
    ) -> Self {
        self.entries.push(Entry {
            exchange,
            symbol: symbol.into(),
            account_type,
            streams: streams.into_iter().collect(),
            is_raw: false,
        });
        self
    }

    /// Add a subscription with a raw exchange-native symbol. `symbol` is
    /// passed through to the connector verbatim — no `SymbolNormalizer`
    /// translation. Use for instrument IDs that don't fit the canonical
    /// BASE-QUOTE shape:
    /// - Deribit options: `"BTC-23MAY26-86000-C"`
    /// - Futures with date suffix: `"BTCUSDT_240329"`
    /// - Index symbols: `".DEFI"`, `"BTCUSD-PERP"`
    ///
    /// The caller is responsible for using the exact wire format the
    /// exchange expects — `Event.symbol` on the handle will mirror the
    /// raw string back.
    pub fn add_raw(
        mut self,
        exchange: ExchangeId,
        symbol: impl Into<String>,
        account_type: AccountType,
        streams: impl IntoIterator<Item = Stream>,
    ) -> Self {
        self.entries.push(Entry {
            exchange,
            symbol: symbol.into(),
            account_type,
            streams: streams.into_iter().collect(),
            is_raw: true,
        });
        self
    }

    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
}

/// Events forwarded to consumers. One variant per market-data class.
#[derive(Debug, Clone)]
pub enum Event {
    Trade {
        exchange: ExchangeId,
        symbol: String,
        point: TradePoint,
    },
    AggTrade {
        exchange: ExchangeId,
        symbol: String,
        point: AggTradePoint,
    },
    Bar {
        exchange: ExchangeId,
        symbol: String,
        timeframe: KlineInterval,
        point: BarPoint,
    },
    Ticker {
        exchange: ExchangeId,
        symbol: String,
        point: TickerPoint,
    },
    OrderbookSnapshot {
        exchange: ExchangeId,
        symbol: String,
        point: ObSnapshotPoint,
    },
    OrderbookDelta {
        exchange: ExchangeId,
        symbol: String,
        point: ObDeltaPoint,
    },
    MarkPrice {
        exchange: ExchangeId,
        symbol: String,
        point: MarkPricePoint,
    },
    FundingRate {
        exchange: ExchangeId,
        symbol: String,
        point: FundingRatePoint,
    },
    OpenInterest {
        exchange: ExchangeId,
        symbol: String,
        point: OpenInterestPoint,
    },
    Liquidation {
        exchange: ExchangeId,
        symbol: String,
        point: LiquidationPoint,
    },
    // --- extended stream types ---
    BlockTrade {
        exchange: ExchangeId,
        symbol: String,
        point: BlockTradePoint,
    },
    IndexPrice {
        exchange: ExchangeId,
        symbol: String,
        point: IndexPricePoint,
    },
    CompositeIndex {
        exchange: ExchangeId,
        symbol: String,
        point: CompositeIndexPoint,
    },
    OptionGreeks {
        exchange: ExchangeId,
        symbol: String,
        point: OptionGreeksPoint,
    },
    VolatilityIndex {
        exchange: ExchangeId,
        symbol: String,
        point: VolatilityIndexPoint,
    },
    HistoricalVolatility {
        exchange: ExchangeId,
        symbol: String,
        point: HistoricalVolatilityPoint,
    },
    Basis {
        exchange: ExchangeId,
        symbol: String,
        point: BasisPoint,
    },
    InsuranceFund {
        exchange: ExchangeId,
        symbol: String,
        point: InsuranceFundPoint,
    },
    OrderbookL3 {
        exchange: ExchangeId,
        symbol: String,
        point: OrderbookL3Point,
    },
    SettlementEvent {
        exchange: ExchangeId,
        symbol: String,
        point: SettlementEventPoint,
    },
    AuctionEvent {
        exchange: ExchangeId,
        symbol: String,
        point: AuctionEventPoint,
    },
    MarketWarning {
        exchange: ExchangeId,
        symbol: String,
        point: MarketWarningPoint,
    },
    RiskLimit {
        exchange: ExchangeId,
        symbol: String,
        point: RiskLimitPoint,
    },
    PredictedFunding {
        exchange: ExchangeId,
        symbol: String,
        point: PredictedFundingPoint,
    },
    FundingSettlement {
        exchange: ExchangeId,
        symbol: String,
        point: FundingSettlementPoint,
    },
    MarkPriceKline {
        exchange: ExchangeId,
        symbol: String,
        timeframe: KlineInterval,
        point: MarkPriceKlinePoint,
    },
    IndexPriceKline {
        exchange: ExchangeId,
        symbol: String,
        timeframe: KlineInterval,
        point: IndexPriceKlinePoint,
    },
    PremiumIndexKline {
        exchange: ExchangeId,
        symbol: String,
        timeframe: KlineInterval,
        point: PremiumIndexKlinePoint,
    },
}

impl Event {
    pub fn exchange(&self) -> ExchangeId {
        match self {
            Event::Trade { exchange, .. } | Event::AggTrade { exchange, .. } |
            Event::Bar { exchange, .. } | Event::Ticker { exchange, .. } |
            Event::OrderbookSnapshot { exchange, .. } | Event::OrderbookDelta { exchange, .. } |
            Event::MarkPrice { exchange, .. } |
            Event::FundingRate { exchange, .. } | Event::OpenInterest { exchange, .. } |
            Event::Liquidation { exchange, .. } |
            Event::BlockTrade { exchange, .. } | Event::IndexPrice { exchange, .. } |
            Event::CompositeIndex { exchange, .. } | Event::OptionGreeks { exchange, .. } |
            Event::VolatilityIndex { exchange, .. } | Event::HistoricalVolatility { exchange, .. } |
            Event::Basis { exchange, .. } | Event::InsuranceFund { exchange, .. } |
            Event::OrderbookL3 { exchange, .. } | Event::SettlementEvent { exchange, .. } |
            Event::AuctionEvent { exchange, .. } | Event::MarketWarning { exchange, .. } |
            Event::RiskLimit { exchange, .. } | Event::PredictedFunding { exchange, .. } |
            Event::FundingSettlement { exchange, .. } |
            Event::MarkPriceKline { exchange, .. } | Event::IndexPriceKline { exchange, .. } |
            Event::PremiumIndexKline { exchange, .. } => *exchange,
        }
    }
    pub fn symbol(&self) -> &str {
        match self {
            Event::Trade { symbol, .. } | Event::AggTrade { symbol, .. } |
            Event::Bar { symbol, .. } | Event::Ticker { symbol, .. } |
            Event::OrderbookSnapshot { symbol, .. } | Event::OrderbookDelta { symbol, .. } |
            Event::MarkPrice { symbol, .. } |
            Event::FundingRate { symbol, .. } | Event::OpenInterest { symbol, .. } |
            Event::Liquidation { symbol, .. } |
            Event::BlockTrade { symbol, .. } | Event::IndexPrice { symbol, .. } |
            Event::CompositeIndex { symbol, .. } | Event::OptionGreeks { symbol, .. } |
            Event::VolatilityIndex { symbol, .. } | Event::HistoricalVolatility { symbol, .. } |
            Event::Basis { symbol, .. } | Event::InsuranceFund { symbol, .. } |
            Event::OrderbookL3 { symbol, .. } | Event::SettlementEvent { symbol, .. } |
            Event::AuctionEvent { symbol, .. } | Event::MarketWarning { symbol, .. } |
            Event::RiskLimit { symbol, .. } | Event::PredictedFunding { symbol, .. } |
            Event::FundingSettlement { symbol, .. } |
            Event::MarkPriceKline { symbol, .. } | Event::IndexPriceKline { symbol, .. } |
            Event::PremiumIndexKline { symbol, .. } => symbol,
        }
    }

    /// Replace the symbol label on this event in-place.
    ///
    /// Used by `Station::subscribe` so each `SubscriptionHandle` sees the
    /// user-input symbol it passed in `SubscriptionSet::add(...)`, regardless
    /// of which other consumer first established the underlying multiplex.
    /// The routing key (raw exchange-native) is unaffected; this only changes
    /// the cosmetic label that `Event.symbol()` returns to the consumer.
    pub(crate) fn set_symbol(&mut self, new_symbol: String) {
        match self {
            Event::Trade { symbol, .. }
            | Event::AggTrade { symbol, .. }
            | Event::Bar { symbol, .. }
            | Event::Ticker { symbol, .. }
            | Event::OrderbookSnapshot { symbol, .. }
            | Event::OrderbookDelta { symbol, .. }
            | Event::MarkPrice { symbol, .. }
            | Event::FundingRate { symbol, .. }
            | Event::OpenInterest { symbol, .. }
            | Event::Liquidation { symbol, .. }
            | Event::BlockTrade { symbol, .. }
            | Event::IndexPrice { symbol, .. }
            | Event::CompositeIndex { symbol, .. }
            | Event::OptionGreeks { symbol, .. }
            | Event::VolatilityIndex { symbol, .. }
            | Event::HistoricalVolatility { symbol, .. }
            | Event::Basis { symbol, .. }
            | Event::InsuranceFund { symbol, .. }
            | Event::OrderbookL3 { symbol, .. }
            | Event::SettlementEvent { symbol, .. }
            | Event::AuctionEvent { symbol, .. }
            | Event::MarketWarning { symbol, .. }
            | Event::RiskLimit { symbol, .. }
            | Event::PredictedFunding { symbol, .. }
            | Event::FundingSettlement { symbol, .. }
            | Event::MarkPriceKline { symbol, .. }
            | Event::IndexPriceKline { symbol, .. }
            | Event::PremiumIndexKline { symbol, .. } => *symbol = new_symbol,
        }
    }
    pub fn timestamp_ms(&self) -> i64 {
        use crate::series::DataPoint;
        match self {
            Event::Trade { point, .. } => point.timestamp_ms(),
            Event::AggTrade { point, .. } => point.timestamp_ms(),
            Event::Bar { point, .. } => point.timestamp_ms(),
            Event::Ticker { point, .. } => point.timestamp_ms(),
            Event::OrderbookSnapshot { point, .. } => point.timestamp_ms(),
            Event::OrderbookDelta { point, .. } => point.timestamp_ms(),
            Event::MarkPrice { point, .. } => point.timestamp_ms(),
            Event::FundingRate { point, .. } => point.timestamp_ms(),
            Event::OpenInterest { point, .. } => point.timestamp_ms(),
            Event::Liquidation { point, .. } => point.timestamp_ms(),
            Event::BlockTrade { point, .. } => point.timestamp_ms(),
            Event::IndexPrice { point, .. } => point.timestamp_ms(),
            Event::CompositeIndex { point, .. } => point.timestamp_ms(),
            Event::OptionGreeks { point, .. } => point.timestamp_ms(),
            Event::VolatilityIndex { point, .. } => point.timestamp_ms(),
            Event::HistoricalVolatility { point, .. } => point.timestamp_ms(),
            Event::Basis { point, .. } => point.timestamp_ms(),
            Event::InsuranceFund { point, .. } => point.timestamp_ms(),
            Event::OrderbookL3 { point, .. } => point.timestamp_ms(),
            Event::SettlementEvent { point, .. } => point.timestamp_ms(),
            Event::AuctionEvent { point, .. } => point.timestamp_ms(),
            Event::MarketWarning { point, .. } => point.timestamp_ms(),
            Event::RiskLimit { point, .. } => point.timestamp_ms(),
            Event::PredictedFunding { point, .. } => point.timestamp_ms(),
            Event::FundingSettlement { point, .. } => point.timestamp_ms(),
            Event::MarkPriceKline { point, .. } => point.timestamp_ms(),
            Event::IndexPriceKline { point, .. } => point.timestamp_ms(),
            Event::PremiumIndexKline { point, .. } => point.timestamp_ms(),
        }
    }
}

/// RAII handle returned from `Station::subscribe`. Dropping releases the
/// per-StreamKey consumer ref count; when count hits zero the multiplexer
/// shuts down.
pub struct SubscriptionHandle {
    pub(crate) rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    pub(crate) _refs: Vec<MultiplexRef>,
}

impl std::fmt::Debug for SubscriptionHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscriptionHandle").finish()
    }
}

impl SubscriptionHandle {
    pub async fn recv(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}

/// Per-stream subscribe failure reported by [`crate::Station::subscribe`].
///
/// Carries everything a consumer needs to log + skip without forcing them
/// to parse a `Display` string. `error.is_not_supported()` distinguishes
/// venue-doesn't-expose-this-stream (architectural, quiet) from transient
/// failures (worth surfacing).
#[derive(Debug)]
pub struct FailedStream {
    pub exchange: ExchangeId,
    pub account_type: AccountType,
    /// User-input symbol form (NOT the normalized exchange-native form).
    pub symbol: String,
    pub stream: Stream,
    pub error: crate::StationError,
}

/// Outcome of [`crate::Station::subscribe`] in continue-on-error mode.
///
/// `handle` always exists and carries events for every stream in `ok`.
/// `failed` is a per-stream list of subscribes that did not produce a
/// live forwarder. The most common entry there is
/// `StationError::StreamNotSupported` — the venue genuinely does not
/// expose the requested stream on the WS wire. Other errors (transport,
/// REST, symbol normalize) also land here so the consumer can log them
/// without aborting the whole subscribe batch.
///
/// `failed` is empty on success — callers that want fail-fast semantics
/// can simply `if !report.failed.is_empty() { return Err(...) }`.
pub struct SubscribeReport {
    pub handle: SubscriptionHandle,
    pub ok: Vec<SeriesKey>,
    pub failed: Vec<FailedStream>,
}

impl SubscribeReport {
    /// True if every requested stream produced a live forwarder.
    pub fn is_fully_ok(&self) -> bool { self.failed.is_empty() }
    /// Convenience: total streams requested (`ok.len() + failed.len()`).
    pub fn total(&self) -> usize { self.ok.len() + self.failed.len() }
}

impl std::fmt::Debug for SubscribeReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SubscribeReport")
            .field("ok", &self.ok.len())
            .field("failed", &self.failed.len())
            .finish()
    }
}

pub(crate) struct MultiplexRef {
    pub(crate) station: std::sync::Weak<crate::station::StationInner>,
    pub(crate) key: SeriesKey,
}

impl Drop for MultiplexRef {
    fn drop(&mut self) {
        if let Some(inner) = self.station.upgrade() {
            inner.release_consumer(&self.key);
        }
    }
}
