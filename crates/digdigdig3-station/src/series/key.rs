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
    MarkPrice,
    FundingRate,
    OpenInterest,
    Liquidation,
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

impl Kind {
    /// Short kebab-case label for filesystem paths.
    pub fn slug(&self) -> String {
        match self {
            Kind::Trade => "trades".to_string(),
            Kind::AggTrade => "agg_trades".to_string(),
            Kind::Kline(iv) => format!("klines_{}", iv.as_str()),
            Kind::Ticker => "tickers".to_string(),
            Kind::Orderbook => "orderbook_snapshots".to_string(),
            Kind::MarkPrice => "mark_price".to_string(),
            Kind::FundingRate => "funding_rate".to_string(),
            Kind::OpenInterest => "open_interest".to_string(),
            Kind::Liquidation => "liquidations".to_string(),
            Kind::BlockTrade => "block_trades".to_string(),
            Kind::IndexPrice => "index_price".to_string(),
            Kind::CompositeIndex => "composite_index".to_string(),
            Kind::OptionGreeks => "option_greeks".to_string(),
            Kind::VolatilityIndex => "volatility_index".to_string(),
            Kind::HistoricalVolatility => "historical_volatility".to_string(),
            Kind::Basis => "basis".to_string(),
            Kind::InsuranceFund => "insurance_fund".to_string(),
            Kind::OrderbookL3 => "orderbook_l3".to_string(),
            Kind::SettlementEvent => "settlement_events".to_string(),
            Kind::AuctionEvent => "auction_events".to_string(),
            Kind::MarketWarning => "market_warnings".to_string(),
            Kind::RiskLimit => "risk_limit".to_string(),
            Kind::PredictedFunding => "predicted_funding".to_string(),
            Kind::FundingSettlement => "funding_settlement".to_string(),
            Kind::MarkPriceKline(iv) => format!("mark_price_klines_{}", iv.as_str()),
            Kind::IndexPriceKline(iv) => format!("index_price_klines_{}", iv.as_str()),
            Kind::PremiumIndexKline(iv) => format!("premium_index_klines_{}", iv.as_str()),
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
