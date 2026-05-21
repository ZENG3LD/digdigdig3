use digdigdig3::core::types::{AccountType, ExchangeId};

use crate::data::{
    AggTradePoint, BarPoint, FundingRatePoint, LiquidationPoint, MarkPricePoint, ObSnapshotPoint,
    OpenInterestPoint, TickerPoint, TradePoint,
};
use crate::series::{Kind, SeriesKey};

/// User-facing stream class to request in a `SubscriptionSet`.
///
/// `Kline` takes a timeframe string ("1m", "5m", "1h", "1d"). All other
/// variants are parameterless.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Stream {
    Ticker,
    Trade,
    Orderbook,
    Kline(String),
    MarkPrice,
    FundingRate,
    OpenInterest,
    Liquidation,
    AggTrade,
}

impl Stream {
    pub(crate) fn to_kind(&self) -> Kind {
        match self {
            Stream::Trade => Kind::Trade,
            Stream::AggTrade => Kind::AggTrade,
            Stream::Kline(iv) => Kind::Kline(iv.clone()),
            Stream::Ticker => Kind::Ticker,
            Stream::Orderbook => Kind::Orderbook,
            Stream::MarkPrice => Kind::MarkPrice,
            Stream::FundingRate => Kind::FundingRate,
            Stream::OpenInterest => Kind::OpenInterest,
            Stream::Liquidation => Kind::Liquidation,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) exchange: ExchangeId,
    pub(crate) symbol: String,
    pub(crate) account_type: AccountType,
    pub(crate) streams: Vec<Stream>,
}

/// Declarative subscription request — built up fluently, consumed by
/// [`crate::Station::subscribe`].
#[derive(Debug, Default, Clone)]
pub struct SubscriptionSet {
    pub(crate) entries: Vec<Entry>,
}

impl SubscriptionSet {
    pub fn new() -> Self { Self::default() }

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
        timeframe: String,
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
}

impl Event {
    pub fn exchange(&self) -> ExchangeId {
        match self {
            Event::Trade { exchange, .. } | Event::AggTrade { exchange, .. } |
            Event::Bar { exchange, .. } | Event::Ticker { exchange, .. } |
            Event::OrderbookSnapshot { exchange, .. } | Event::MarkPrice { exchange, .. } |
            Event::FundingRate { exchange, .. } | Event::OpenInterest { exchange, .. } |
            Event::Liquidation { exchange, .. } => *exchange,
        }
    }
    pub fn symbol(&self) -> &str {
        match self {
            Event::Trade { symbol, .. } | Event::AggTrade { symbol, .. } |
            Event::Bar { symbol, .. } | Event::Ticker { symbol, .. } |
            Event::OrderbookSnapshot { symbol, .. } | Event::MarkPrice { symbol, .. } |
            Event::FundingRate { symbol, .. } | Event::OpenInterest { symbol, .. } |
            Event::Liquidation { symbol, .. } => symbol,
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
            | Event::MarkPrice { symbol, .. }
            | Event::FundingRate { symbol, .. }
            | Event::OpenInterest { symbol, .. }
            | Event::Liquidation { symbol, .. } => *symbol = new_symbol,
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
            Event::MarkPrice { point, .. } => point.timestamp_ms(),
            Event::FundingRate { point, .. } => point.timestamp_ms(),
            Event::OpenInterest { point, .. } => point.timestamp_ms(),
            Event::Liquidation { point, .. } => point.timestamp_ms(),
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
