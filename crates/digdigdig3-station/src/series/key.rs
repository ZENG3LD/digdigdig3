use digdigdig3::core::types::{AccountType, ExchangeId};

/// What kind of stream this series carries.
///
/// `Kline` carries the timeframe ("1m", "5m", "1h", "1d") so different
/// timeframes of the same symbol get their own series. All other kinds
/// have no extra parameter.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Kind {
    Trade,
    AggTrade,
    Kline(String),
    Ticker,
    Orderbook,
    MarkPrice,
    FundingRate,
    OpenInterest,
    Liquidation,
}

impl Kind {
    /// Short kebab-case label for filesystem paths.
    pub fn slug(&self) -> String {
        match self {
            Kind::Trade => "trades".to_string(),
            Kind::AggTrade => "agg_trades".to_string(),
            Kind::Kline(iv) => format!("klines_{}", iv),
            Kind::Ticker => "tickers".to_string(),
            Kind::Orderbook => "orderbook_snapshots".to_string(),
            Kind::MarkPrice => "mark_price".to_string(),
            Kind::FundingRate => "funding_rate".to_string(),
            Kind::OpenInterest => "open_interest".to_string(),
            Kind::Liquidation => "liquidations".to_string(),
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
