//! # Finnhub Endpoints
//!
//! URL'ы и endpoint enum для Finnhub API.


// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Finnhub API
#[derive(Debug, Clone)]
pub struct FinnhubUrls {
    pub rest_base: &'static str,
    pub ws_url: &'static str,
}

impl FinnhubUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest_base: "https://finnhub.io/api/v1",
        ws_url: "wss://ws.finnhub.io",
    };

    /// Get REST base URL
    pub fn rest_url(&self) -> &str {
        self.rest_base
    }

    /// Get WebSocket URL
    pub fn websocket_url(&self) -> &str {
        self.ws_url
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Finnhub API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum FinnhubEndpoint {
    // === STOCK MARKET DATA ===
    Quote,                    // Real-time quote
    StockCandles,             // OHLC candles
    TickData,                 // Trade-level data (premium)
    BidAsk,                   // Last bid/ask (premium)
    StockSymbols,             // Stock symbol list for an exchange

    // === COMPANY FUNDAMENTALS ===
    CompanyProfile,           // Company information
    BasicFinancials,          // Financial metrics
    FinancialStatements,      // Balance sheet, income, cash flow
    CompanyPeers,             // Comparable companies
    CompanyExecutives,        // Executive information

    // === ESTIMATES & ANALYSIS ===
    EarningsCalendar,         // Earnings releases
    EpsEstimates,             // EPS forecasts
    RevenueEstimates,         // Revenue forecasts
    PriceTarget,              // Analyst price targets
    Recommendations,          // Buy/hold/sell ratings
    UpgradeDowngrade,         // Rating changes

    // === NEWS & SENTIMENT ===
    MarketNews,               // General market news
    CompanyNews,              // Company-specific news
    NewsSentiment,            // News sentiment analysis

    // === FOREX ===
    ForexExchanges,           // Forex exchange list
    ForexSymbols,             // Forex symbols
    ForexCandles,             // Forex OHLCV
    ExchangeRates,            // Current exchange rates

    // === CRYPTOCURRENCY ===
    CryptoExchanges,          // Crypto exchange list
    CryptoSymbols,            // Crypto symbols
    CryptoCandles,            // Crypto OHLCV

    // === ALTERNATIVE DATA ===
    InsiderTransactions,      // Form 3/4/5 filings
    InsiderSentiment,         // MSPR metric (premium)
    CongressionalTrading,     // Stock trades by legislators (premium)
    InstitutionalOwnership,   // Fund ownership
    PatentData,               // USPTO filings (premium)
    VisaApplications,         // H1-B data (premium)
    UsaSpending,              // Government contracts (premium)
    SupplyChain,              // Customers and suppliers (premium)
    SenateLobby,              // Lobbying activity (premium)

    // === ESG & CORPORATE ===
    EsgScores,                // ESG ratings (premium)
    ExecutiveCompensation,    // Officer salaries
    RevenueBreakdown,         // Revenue by segment

    // === TECHNICAL ANALYSIS ===
    TechnicalIndicators,      // MACD, RSI, MA, etc.
    PatternRecognition,       // Chart patterns (premium)
    SupportResistance,        // Support/resistance levels (premium)
    AggregateIndicators,      // Combined signals (premium)

    // === MARKET INFO ===
    MarketStatus,             // Exchange open/close status
    MarketHoliday,            // Holiday calendar
    EconomicCalendar,         // Economic indicators

    // === SEC FILINGS ===
    SecFilings,               // SEC documents

    // === ETF DATA ===
    EtfHoldings,              // GET /api/v1/etf/holdings
    EtfProfile,               // GET /api/v1/etf/profile
    EtfCountryExposure,       // GET /api/v1/etf/country
    EtfSectorExposure,        // GET /api/v1/etf/sector

    // === IPO & EARNINGS ===
    IpoCalendar,              // GET /api/v1/calendar/ipo
    EarningsSurprise,         // GET /api/v1/stock/earnings

    // === SOCIAL SENTIMENT ===
    SocialSentiment,          // GET /api/v1/stock/social-sentiment

    // === CRYPTO PROFILE ===
    CryptoProfile,            // GET /api/v1/crypto/profile
}

impl FinnhubEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Stock Market Data
            Self::Quote => "/quote",
            Self::StockCandles => "/stock/candle",
            Self::TickData => "/stock/tick",
            Self::BidAsk => "/stock/bidask",
            Self::StockSymbols => "/stock/symbol",

            // Company Fundamentals
            Self::CompanyProfile => "/stock/profile2",
            Self::BasicFinancials => "/stock/metric",
            Self::FinancialStatements => "/stock/financials",
            Self::CompanyPeers => "/stock/peers",
            Self::CompanyExecutives => "/stock/executive",

            // Estimates & Analysis
            Self::EarningsCalendar => "/calendar/earnings",
            Self::EpsEstimates => "/stock/eps-estimate",
            Self::RevenueEstimates => "/stock/revenue-estimate",
            Self::PriceTarget => "/stock/price-target",
            Self::Recommendations => "/stock/recommendation",
            Self::UpgradeDowngrade => "/stock/upgrade-downgrade",

            // News & Sentiment
            Self::MarketNews => "/news",
            Self::CompanyNews => "/company-news",
            Self::NewsSentiment => "/news-sentiment",

            // Forex
            Self::ForexExchanges => "/forex/exchange",
            Self::ForexSymbols => "/forex/symbol",
            Self::ForexCandles => "/forex/candle",
            Self::ExchangeRates => "/forex/rates",

            // Cryptocurrency
            Self::CryptoExchanges => "/crypto/exchange",
            Self::CryptoSymbols => "/crypto/symbol",
            Self::CryptoCandles => "/crypto/candle",

            // Alternative Data
            Self::InsiderTransactions => "/stock/insider-transactions",
            Self::InsiderSentiment => "/stock/insider-sentiment",
            Self::CongressionalTrading => "/stock/congressional-trading",
            Self::InstitutionalOwnership => "/stock/ownership",
            Self::PatentData => "/stock/usa-patent",
            Self::VisaApplications => "/stock/visa-application",
            Self::UsaSpending => "/stock/usa-spending",
            Self::SupplyChain => "/stock/supply-chain",
            Self::SenateLobby => "/stock/lobbying",

            // ESG & Corporate
            Self::EsgScores => "/stock/esg",
            Self::ExecutiveCompensation => "/stock/executive-compensation",
            Self::RevenueBreakdown => "/stock/revenue-breakdown",

            // Technical Analysis
            Self::TechnicalIndicators => "/indicator",
            Self::PatternRecognition => "/scan/pattern",
            Self::SupportResistance => "/scan/support-resistance",
            Self::AggregateIndicators => "/scan/technical-indicator",

            // Market Info
            Self::MarketStatus => "/stock/market-status",
            Self::MarketHoliday => "/stock/market-holiday",
            Self::EconomicCalendar => "/calendar/economic",

            // SEC Filings
            Self::SecFilings => "/stock/filings",

            // ETF Data
            Self::EtfHoldings => "/etf/holdings",
            Self::EtfProfile => "/etf/profile",
            Self::EtfCountryExposure => "/etf/country",
            Self::EtfSectorExposure => "/etf/sector",

            // IPO & Earnings
            Self::IpoCalendar => "/calendar/ipo",
            Self::EarningsSurprise => "/stock/earnings",

            // Social Sentiment
            Self::SocialSentiment => "/stock/social-sentiment",

            // Crypto Profile
            Self::CryptoProfile => "/crypto/profile",
        }
    }

    /// All endpoints require authentication (API key)
    pub fn _requires_auth(&self) -> bool {
        true
    }

    /// HTTP method for endpoint
    pub fn _method(&self) -> &'static str {
        "GET" // Finnhub only has GET endpoints
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Finnhub API
///
/// # Stock Symbol Format
/// - US stocks: Just the ticker symbol (e.g., "AAPL", "MSFT")
/// - No base/quote separation like crypto exchanges
///
/// # Examples
/// - Apple: "AAPL"
/// - Microsoft: "MSFT"
/// - Tesla: "TSLA"
pub fn format_symbol(symbol: &str) -> String {
    // Finnhub uses simple ticker symbols
    symbol.to_uppercase()
}

/// Map interval to Finnhub resolution
///
/// # Finnhub Resolution Format
/// Parameter: `resolution`
/// Values: `1`, `5`, `15`, `30`, `60`, `D`, `W`, `M`
///
/// # Supported Intervals
/// - Minute intervals: 1, 5, 15, 30, 60
/// - Daily: D
/// - Weekly: W
/// - Monthly: M
pub fn map_resolution(interval: &str) -> &'static str {
    match interval {
        "1m" => "1",
        "5m" => "5",
        "15m" => "15",
        "30m" => "30",
        "1h" | "60m" => "60",
        "1d" => "D",
        "1w" => "W",
        "1M" => "M",
        _ => "D", // default to daily
    }
}
