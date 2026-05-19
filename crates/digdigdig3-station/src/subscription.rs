use digdigdig3::core::types::{AccountType, ExchangeId};

/// Streams a `SubscriptionSet` entry may request.
///
/// Phase 1: only `Trade` is wired end-to-end. Other variants are reserved for
/// step 7+ when the multiplexer learns more StreamKinds.
#[derive(Debug, Clone, PartialEq, Eq)]
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

/// Events forwarded to consumers. Phase 1: only `Trade` is wired; richer variants
/// are added in step 7 as the multiplexer expands.
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
}

/// RAII handle returned from `Station::subscribe`.
///
/// Dropping cancels the forwarder task (its receivers go away, the `select!`
/// breaks, and the spawned task exits). The underlying `ExchangeHub` WS
/// connection is left intact — Station owns it for lifetime sharing in step 7.
pub struct SubscriptionHandle {
    pub(crate) rx: tokio::sync::mpsc::UnboundedReceiver<Event>,
    pub(crate) _shutdown: tokio::sync::oneshot::Sender<()>,
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
