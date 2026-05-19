/// Streams a `SubscriptionSet` entry may request.
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

/// Declarative subscription request. Phase 1 stub.
#[derive(Debug, Default, Clone)]
pub struct SubscriptionSet {
    entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)] // phase 1 stub — fields wired in step 6
struct Entry {
    exchange: String,
    symbol: String,
    account_type: String,
    streams: Vec<Stream>,
}

impl SubscriptionSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(
        mut self,
        exchange: impl Into<String>,
        symbol: impl Into<String>,
        account_type: impl Into<String>,
        streams: impl IntoIterator<Item = Stream>,
    ) -> Self {
        self.entries.push(Entry {
            exchange: exchange.into(),
            symbol: symbol.into(),
            account_type: account_type.into(),
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

/// RAII handle returned from `Station::subscribe`. Phase 1 stub.
#[derive(Debug)]
pub struct SubscriptionHandle {
    _priv: (),
}

impl SubscriptionHandle {
    pub(crate) fn stub() -> Self {
        Self { _priv: () }
    }
}
