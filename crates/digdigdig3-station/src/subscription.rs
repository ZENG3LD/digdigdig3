use digdigdig3::core::types::{AccountType, ExchangeId};

/// Streams a `SubscriptionSet` entry may request.
///
/// Phase 1: `Trade`. Phase 2 adds `Orderbook` end-to-end. Other variants are
/// reserved for Phase 3+.
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

#[derive(Debug, Clone)]
pub(crate) struct Entry {
    pub(crate) exchange: ExchangeId,
    pub(crate) symbol: String,
    pub(crate) account_type: AccountType,
    pub(crate) streams: Vec<Stream>,
}

/// Declarative subscription request — built up fluently, consumed by [`crate::Station::subscribe`].
#[derive(Debug, Default, Clone)]
pub struct SubscriptionSet {
    pub(crate) entries: Vec<Entry>,
}

impl SubscriptionSet {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Events forwarded to consumers.
#[derive(Debug, Clone)]
pub enum Event {
    Trade {
        exchange: ExchangeId,
        symbol: String,
        price: f64,
        quantity: f64,
        side: String,
        timestamp: i64,
    },
    OrderbookSnapshot {
        exchange: ExchangeId,
        symbol: String,
        /// Sorted descending by price (best bid first).
        bids: Vec<(f64, f64)>,
        /// Sorted ascending by price (best ask first).
        asks: Vec<(f64, f64)>,
        timestamp: i64,
    },
}

/// Key uniquely identifying a multiplexed wire subscription. N consumers can
/// share one WS subscription on the same `StreamKey`.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct StreamKey {
    pub exchange: ExchangeId,
    pub account_type: AccountType,
    /// Exchange-native raw symbol (already normalized).
    pub symbol_raw: String,
    pub kind: StreamKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StreamKind {
    Trade,
    Orderbook,
}

impl StreamKind {
    pub(crate) fn from_stream(s: &Stream) -> Option<Self> {
        match s {
            Stream::Trade => Some(Self::Trade),
            Stream::Orderbook => Some(Self::Orderbook),
            _ => None,
        }
    }
}

/// RAII handle returned from `Station::subscribe`.
///
/// Dropping releases this handle's reference on each shared multiplexer. When
/// the last consumer of a `StreamKey` drops, the mux closes its WS subscription
/// and exits.
pub struct SubscriptionHandle {
    pub(crate) rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    /// Held to keep multiplexer ref-counts alive. On Drop, each releases.
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

/// Drop-anchor: every consumer holds one `MultiplexRef` per StreamKey it
/// subscribed to. When the ref count drops to zero the multiplexer shuts down.
pub(crate) struct MultiplexRef {
    pub(crate) station: std::sync::Weak<crate::station::StationInner>,
    pub(crate) key: StreamKey,
}

impl Drop for MultiplexRef {
    fn drop(&mut self) {
        if let Some(inner) = self.station.upgrade() {
            inner.release_consumer(&self.key);
        }
    }
}
