//! Finnhub API endpoints

/// Base URLs for Finnhub API
pub struct FinnhubEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for FinnhubEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://finnhub.io/api/v1",
            ws_base: Some("wss://ws.finnhub.io"),
        }
    }
}

/// Finnhub API endpoint enum
#[derive(Debug, Clone)]
pub enum FinnhubEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // STOCK ENDPOINTS (7)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get real-time quote data
    Quote,
    /// Get stock candles/OHLC data
    StockCandles,
    /// Search for symbols
    SymbolSearch,
    /// Get company profile
    CompanyProfile,
    /// Get company peers
    CompanyPeers,
    /// Get financial statements
    Financials,
    /// Get basic financial metrics
    BasicFinancials,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET ENDPOINTS (5)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get market news by category
    MarketNews,
    /// Get company-specific news
    CompanyNews,
    /// Get market status for exchange
    MarketStatus,
    /// Get earnings calendar
    EarningsCalendar,
    /// Get IPO calendar
    IpoCalendar,

    // ═══════════════════════════════════════════════════════════════════════
    // FOREX ENDPOINTS (3)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get forex exchange rates
    ForexRates,
    /// Get forex candles
    ForexCandles,
    /// Get forex symbols
    ForexSymbols,

    // ═══════════════════════════════════════════════════════════════════════
    // CRYPTO ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get crypto candles
    CryptoCandles,
    /// Get crypto symbols
    CryptoSymbols,

    // ═══════════════════════════════════════════════════════════════════════
    // ECONOMIC ENDPOINTS (3)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get economic calendar
    EconomicCalendar,
    /// Get country list
    CountryList,
    /// Get economic data by code
    EconomicData,

    // ═══════════════════════════════════════════════════════════════════════
    // SENTIMENT ENDPOINTS (4)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get social sentiment for stock
    SocialSentiment,
    /// Get insider transactions
    InsiderTransactions,
    /// Get insider sentiment
    InsiderSentiment,
    /// Get analyst recommendation trends
    RecommendationTrends,
}

impl FinnhubEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Stock
            Self::Quote => "/quote",
            Self::StockCandles => "/stock/candle",
            Self::SymbolSearch => "/search",
            Self::CompanyProfile => "/stock/profile2",
            Self::CompanyPeers => "/stock/peers",
            Self::Financials => "/stock/financials",
            Self::BasicFinancials => "/stock/metric",

            // Market
            Self::MarketNews => "/news",
            Self::CompanyNews => "/company-news",
            Self::MarketStatus => "/stock/market-status",
            Self::EarningsCalendar => "/calendar/earnings",
            Self::IpoCalendar => "/calendar/ipo",

            // Forex
            Self::ForexRates => "/forex/rates",
            Self::ForexCandles => "/forex/candle",
            Self::ForexSymbols => "/forex/symbol",

            // Crypto
            Self::CryptoCandles => "/crypto/candle",
            Self::CryptoSymbols => "/crypto/symbol",

            // Economic
            Self::EconomicCalendar => "/calendar/economic",
            Self::CountryList => "/country",
            Self::EconomicData => "/economic",

            // Sentiment
            Self::SocialSentiment => "/stock/social-sentiment",
            Self::InsiderTransactions => "/stock/insider-transactions",
            Self::InsiderSentiment => "/stock/insider-sentiment",
            Self::RecommendationTrends => "/stock/recommendation",
        }
    }
}

/// Format symbol for Finnhub API
///
/// Finnhub uses standard stock symbols like "AAPL", "MSFT"
/// For forex: "OANDA:EUR_USD"
/// For crypto: "BINANCE:BTCUSDT"
pub fn format_symbol(symbol: &str) -> String {
    symbol.to_uppercase()
}
