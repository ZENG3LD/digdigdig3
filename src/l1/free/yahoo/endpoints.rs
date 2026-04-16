//! Yahoo Finance API endpoints

/// Base URLs for Yahoo Finance API
#[derive(Debug, Clone)]
pub struct YahooFinanceUrls {
    pub rest_base: &'static str,
    pub rest_base_alt: &'static str,  // Load-balanced alternative
    pub ws_base: Option<&'static str>,
}

impl Default for YahooFinanceUrls {
    fn default() -> Self {
        Self {
            rest_base: "https://query1.finance.yahoo.com",
            rest_base_alt: "https://query2.finance.yahoo.com",
            ws_base: Some("wss://streamer.finance.yahoo.com/"),
        }
    }
}

/// Yahoo Finance API endpoint enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum YahooFinanceEndpoint {
    // === MARKET DATA - CURRENT PRICES & QUOTES ===
    Quote,                  // /v7/finance/quote - Get current quote data (multiple symbols)
    Chart,                  // /v8/finance/chart/{symbol} - Real-time chart data with OHLCV
    QuoteSummary,           // /v11/finance/quoteSummary/{symbol} - Comprehensive data (modular)
    MarketSummary,          // /v6/finance/quote/marketSummary - Major indices overview
    Spark,                  // /v1/finance/spark - Mini sparkline data

    // === HISTORICAL DATA ===
    DownloadHistory,        // /v7/finance/download/{symbol} - CSV download (requires crumb)

    // === OPTIONS DATA ===
    Options,                // /v7/finance/options/{symbol} - Full options chain

    // === FUNDAMENTAL DATA (via quoteSummary modules) ===
    // Note: These all use QuoteSummary endpoint with different module parameters
    // Implemented as separate enum variants for clarity

    // === SEARCH & DISCOVERY ===
    Search,                 // /v1/finance/search - Symbol search
    Lookup,                 // /v1/finance/lookup - Exact symbol lookup
    ScreenerPredefined,     // /v1/finance/screener/predefined - Predefined screeners
    ScreenerCustom,         // /v1/finance/screener - Custom screener (POST)
    Trending,               // /v1/finance/trending/{region} - Trending symbols
    RecommendationsBySymbol, // /v1/finance/recommendationsBySymbol/{symbol} - Similar symbols

    // === FUNDAMENTALS TIME SERIES ===
    FundamentalsTimeSeries, // /ws/fundamentals-timeseries/v1/finance/timeseries/{symbol}

    // === AUTHENTICATION ===
    GetCrumb,               // /v1/test/getcrumb - Get authentication crumb
}

impl YahooFinanceEndpoint {
    /// Get endpoint path (without symbol substitution)
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::Quote => "/v7/finance/quote",
            Self::Chart => "/v8/finance/chart",
            Self::QuoteSummary => "/v10/finance/quoteSummary",
            Self::MarketSummary => "/v6/finance/quote/marketSummary",
            Self::Spark => "/v1/finance/spark",

            // Historical
            Self::DownloadHistory => "/v7/finance/download",

            // Options
            Self::Options => "/v7/finance/options",

            // Search & Discovery
            Self::Search => "/v1/finance/search",
            Self::Lookup => "/v1/finance/lookup",
            Self::ScreenerPredefined => "/v1/finance/screener/predefined",
            Self::ScreenerCustom => "/v1/finance/screener",
            Self::Trending => "/v1/finance/trending",
            Self::RecommendationsBySymbol => "/v1/finance/recommendationsBySymbol",

            // Fundamentals Time Series
            Self::FundamentalsTimeSeries => "/ws/fundamentals-timeseries/v1/finance/timeseries",

            // Auth
            Self::GetCrumb => "/v1/test/getcrumb",
        }
    }

    /// Build full URL with symbol substitution
    pub fn url(&self, base_url: &str, symbol: Option<&str>) -> String {
        let path = self.path();

        match (self, symbol) {
            // Endpoints that need symbol in path
            (Self::Chart, Some(sym)) => format!("{}{}/{}", base_url, path, sym),
            (Self::QuoteSummary, Some(sym)) => format!("{}{}/{}", base_url, path, sym),
            (Self::DownloadHistory, Some(sym)) => format!("{}{}/{}", base_url, path, sym),
            (Self::Options, Some(sym)) => format!("{}{}/{}", base_url, path, sym),
            (Self::Trending, Some(region)) => format!("{}{}/{}", base_url, path, region),
            (Self::RecommendationsBySymbol, Some(sym)) => format!("{}{}/{}", base_url, path, sym),
            (Self::FundamentalsTimeSeries, Some(sym)) => format!("{}{}/{}", base_url, path, sym),

            // Endpoints without symbol in path
            _ => format!("{}{}", base_url, path),
        }
    }

    /// HTTP method for the endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Self::ScreenerCustom => "POST",
            _ => "GET",
        }
    }

    /// Does this endpoint require cookie/crumb authentication?
    pub fn requires_crumb(&self) -> bool {
        matches!(self, Self::DownloadHistory)
    }
}

/// Format symbol for Yahoo Finance API
///
/// Yahoo uses specific formats for different asset types:
/// - US Stocks: "AAPL", "MSFT" (ticker only)
/// - Crypto: "BTC-USD", "ETH-USD" (base-quote with hyphen)
/// - Forex: "EURUSD=X" (pair with =X suffix)
/// - Commodities: "GC=F", "CL=F" (ticker with =F suffix for futures)
/// - Indices: "^GSPC", "^DJI" (^ prefix)
///
/// For now, we implement basic stock and crypto formats.
/// User can pass complete Yahoo symbols directly if needed.
pub fn format_symbol(base: &str, quote: &str) -> String {
    // If quote is empty or "USD", it's likely a stock ticker
    if quote.is_empty() || quote.eq_ignore_ascii_case("USD") {
        // For stocks, just return base (e.g., "AAPL")
        // For crypto with USD, return "BTC-USD" format
        if is_crypto_symbol(base) {
            format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
        } else {
            base.to_uppercase()
        }
    } else {
        // For other pairs (crypto, forex), use hyphen format
        format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
    }
}

/// Check if symbol is a cryptocurrency
fn is_crypto_symbol(symbol: &str) -> bool {
    matches!(
        symbol.to_uppercase().as_str(),
        "BTC" | "ETH" | "BNB" | "XRP" | "ADA" | "SOL" | "DOGE" |
        "DOT" | "MATIC" | "AVAX" | "LINK" | "UNI" | "ATOM" | "LTC"
    )
}

/// Parse symbol from Yahoo format back to (base, quote)
///
/// Examples:
/// - "AAPL" -> ("AAPL", "USD")
/// - "BTC-USD" -> ("BTC", "USD")
/// - "EURUSD=X" -> ("EUR", "USD")
/// - "^GSPC" -> ("^GSPC", "")
#[allow(dead_code)]
pub fn parse_symbol(yahoo_symbol: &str) -> (String, String) {
    // Handle indices (^GSPC, ^DJI)
    if yahoo_symbol.starts_with('^') {
        return (yahoo_symbol.to_string(), String::new());
    }

    // Handle forex (EURUSD=X)
    if yahoo_symbol.ends_with("=X") {
        let pair = yahoo_symbol.trim_end_matches("=X");
        if pair.len() >= 6 {
            return (pair[0..3].to_string(), pair[3..6].to_string());
        }
    }

    // Handle commodities (GC=F)
    if yahoo_symbol.ends_with("=F") {
        return (yahoo_symbol.to_string(), String::new());
    }

    // Handle crypto and normal pairs with hyphen (BTC-USD)
    if let Some(pos) = yahoo_symbol.find('-') {
        let base = yahoo_symbol[0..pos].to_string();
        let quote = yahoo_symbol[pos + 1..].to_string();
        return (base, quote);
    }

    // Default: treat as stock ticker with USD quote
    (yahoo_symbol.to_string(), "USD".to_string())
}

/// QuoteSummary module names
///
/// These are passed as comma-separated values to the `modules` parameter
/// of the /v10/finance/quoteSummary/{symbol} endpoint
#[allow(dead_code)]
pub mod quote_summary_modules {
    pub const ASSET_PROFILE: &str = "assetProfile";
    pub const BALANCE_SHEET_HISTORY: &str = "balanceSheetHistory";
    pub const BALANCE_SHEET_HISTORY_QUARTERLY: &str = "balanceSheetHistoryQuarterly";
    pub const CALENDAR_EVENTS: &str = "calendarEvents";
    pub const CASHFLOW_STATEMENT_HISTORY: &str = "cashflowStatementHistory";
    pub const CASHFLOW_STATEMENT_HISTORY_QUARTERLY: &str = "cashflowStatementHistoryQuarterly";
    pub const DEFAULT_KEY_STATISTICS: &str = "defaultKeyStatistics";
    pub const EARNINGS: &str = "earnings";
    pub const EARNINGS_HISTORY: &str = "earningsHistory";
    pub const EARNINGS_TREND: &str = "earningsTrend";
    pub const ESG_SCORES: &str = "esgScores";
    pub const FINANCIAL_DATA: &str = "financialData";
    pub const FUND_OWNERSHIP: &str = "fundOwnership";
    pub const FUND_PERFORMANCE: &str = "fundPerformance";
    pub const FUND_PROFILE: &str = "fundProfile";
    pub const INCOME_STATEMENT_HISTORY: &str = "incomeStatementHistory";
    pub const INCOME_STATEMENT_HISTORY_QUARTERLY: &str = "incomeStatementHistoryQuarterly";
    pub const INDEX_TREND: &str = "indexTrend";
    pub const INDUSTRY_TREND: &str = "industryTrend";
    pub const INSIDER_HOLDERS: &str = "insiderHolders";
    pub const INSIDER_TRANSACTIONS: &str = "insiderTransactions";
    pub const INSTITUTION_OWNERSHIP: &str = "institutionOwnership";
    pub const MAJOR_DIRECT_HOLDERS: &str = "majorDirectHolders";
    pub const MAJOR_HOLDERS_BREAKDOWN: &str = "majorHoldersBreakdown";
    pub const NET_SHARE_PURCHASE_ACTIVITY: &str = "netSharePurchaseActivity";
    pub const PRICE: &str = "price";
    pub const QUOTE_TYPE: &str = "quoteType";
    pub const RECOMMENDATION_TREND: &str = "recommendationTrend";
    pub const SEC_FILINGS: &str = "secFilings";
    pub const SECTOR_TREND: &str = "sectorTrend";
    pub const SUMMARY_DETAIL: &str = "summaryDetail";
    pub const SUMMARY_PROFILE: &str = "summaryProfile";
    pub const SYMBOL: &str = "symbol";
    pub const TOP_HOLDINGS: &str = "topHoldings";
    pub const UPGRADE_DOWNGRADE_HISTORY: &str = "upgradeDowngradeHistory";
}

/// Chart interval mapping
///
/// Yahoo Finance chart endpoint accepts:
/// - Intraday: 1m, 2m, 5m, 15m, 30m, 60m, 90m, 1h
/// - Daily+: 1d, 5d, 1wk, 1mo, 3mo
///
/// Note: Intraday data has limitations:
/// - 1m: last 7 days only
/// - <1d: last 60 days only
/// - 1h: last 730 days only
pub fn map_chart_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "2m" => "2m",
        "3m" => "5m",  // Yahoo doesn't have 3m, use 5m
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" | "60m" => "1h",
        "90m" => "90m",
        "2h" => "1h",   // Yahoo doesn't have 2h, use 1h
        "4h" => "1h",   // Yahoo doesn't have 4h, use 1h
        "1d" => "1d",
        "5d" => "5d",
        "1w" | "1wk" => "1wk",
        "1M" | "1mo" => "1mo",
        "3M" | "3mo" => "3mo",
        _ => "1d",  // default
    }
}
