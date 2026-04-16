//! # Polygon.io Endpoints
//!
//! URL'ы и endpoint enum для Polygon.io (Massive.com) API.


// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Polygon.io API
#[derive(Debug, Clone)]
pub struct PolygonUrls {
    pub rest_base: &'static str,
    pub ws_realtime: &'static str,
    pub ws_delayed: &'static str,
}

impl PolygonUrls {
    /// Production URLs (Massive.com rebranded)
    pub const MAINNET: Self = Self {
        rest_base: "https://api.massive.com",
        ws_realtime: "wss://socket.massive.com/stocks",
        ws_delayed: "wss://delayed.massive.com/stocks",
    };

    /// Get REST base URL
    pub fn rest_url(&self) -> &str {
        self.rest_base
    }

    /// Get WebSocket URL (use delayed for free/starter tiers)
    pub fn ws_url(&self, is_realtime: bool) -> &str {
        if is_realtime {
            self.ws_realtime
        } else {
            self.ws_delayed
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Polygon.io API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
#[allow(clippy::upper_case_acronyms)]
pub enum PolygonEndpoint {
    // === REFERENCE DATA ===
    Tickers,
    TickerDetails,
    TickerTypes,

    // === MARKET DATA ===
    Aggregates,           // OHLC bars
    PreviousClose,        // Previous day bar
    GroupedDaily,         // All tickers' daily bars

    // === SNAPSHOTS ===
    SingleSnapshot,       // Single ticker snapshot
    AllSnapshot,          // Full market snapshot
    UnifiedSnapshot,      // Unified snapshot (v3)

    // === TRADES & QUOTES ===
    Trades,               // Tick-level trades
    LastTrade,            // Most recent trade
    Quotes,               // Tick-level quotes
    LastQuote,            // Most recent NBBO quote

    // === TECHNICAL INDICATORS ===
    SMA,                  // Simple Moving Average
    EMA,                  // Exponential Moving Average
    MACD,                 // MACD
    RSI,                  // Relative Strength Index

    // === FUNDAMENTALS ===
    Dividends,            // Dividend data
    Splits,               // Stock splits
    FinancialRatios,      // Financial ratios

    // === MARKET STATUS ===
    MarketStatus,         // Current market status
    MarketHolidays,       // Upcoming holidays

    // === NEWS ===
    News,                 // News articles

    // === OPTIONS ===
    /// GET /v3/reference/options/contracts — options contracts reference data
    OptionsContracts,
    /// GET /v3/snapshot/options/{underlyingAsset} — options chain snapshot
    OptionsChain,

    // === INDICES ===
    /// GET /v3/snapshot/indices — indices snapshot
    IndicesSnapshot,

    // === FOREX ===
    /// GET /v1/last_quote/currencies/{from}/{to} — forex last quote
    ForexQuote,
    /// GET /v2/aggs/ticker/{ticker}/range/{mul}/{res}/{from}/{to} — forex OHLCV bars
    ForexAggregates,

    // === CRYPTO SNAPSHOT ===
    /// GET /v2/snapshot/locale/global/markets/crypto/tickers — crypto snapshot
    CryptoSnapshot,

    // === REFERENCE DATA ===
    /// GET /v3/reference/conditions — trade conditions
    ReferenceConditions,
    /// GET /v3/reference/exchanges — exchanges list
    ReferenceExchanges,
}

impl PolygonEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Reference Data
            Self::Tickers => "/v3/reference/tickers",
            Self::TickerDetails => "/v3/reference/tickers/{ticker}",
            Self::TickerTypes => "/v3/reference/tickers/types",

            // Market Data
            Self::Aggregates => "/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from}/{to}",
            Self::PreviousClose => "/v2/aggs/ticker/{ticker}/prev",
            Self::GroupedDaily => "/v2/aggs/grouped/locale/us/market/stocks/{date}",

            // Snapshots
            Self::SingleSnapshot => "/v2/snapshot/locale/us/markets/stocks/tickers/{ticker}",
            Self::AllSnapshot => "/v2/snapshot/locale/us/markets/stocks/tickers",
            Self::UnifiedSnapshot => "/v3/snapshot",

            // Trades & Quotes
            Self::Trades => "/v3/trades/{ticker}",
            Self::LastTrade => "/v2/last/trade/{ticker}",
            Self::Quotes => "/v3/quotes/{ticker}",
            Self::LastQuote => "/v2/last/nbbo/{ticker}",

            // Technical Indicators
            Self::SMA => "/v1/indicators/sma/{ticker}",
            Self::EMA => "/v1/indicators/ema/{ticker}",
            Self::MACD => "/v1/indicators/macd/{ticker}",
            Self::RSI => "/v1/indicators/rsi/{ticker}",

            // Fundamentals
            Self::Dividends => "/vX/reference/dividends",
            Self::Splits => "/vX/reference/splits",
            Self::FinancialRatios => "/vX/reference/financials",

            // Market Status
            Self::MarketStatus => "/v1/marketstatus/now",
            Self::MarketHolidays => "/v1/marketstatus/upcoming",

            // News
            Self::News => "/v2/reference/news",

            // Options
            Self::OptionsContracts => "/v3/reference/options/contracts",
            Self::OptionsChain => "/v3/snapshot/options/{underlyingAsset}",

            // Indices
            Self::IndicesSnapshot => "/v3/snapshot/indices",

            // Forex
            Self::ForexQuote => "/v1/last_quote/currencies/{from}/{to}",
            Self::ForexAggregates => "/v2/aggs/ticker/{ticker}/range/{multiplier}/{timespan}/{from}/{to}",

            // Crypto Snapshot
            Self::CryptoSnapshot => "/v2/snapshot/locale/global/markets/crypto/tickers",

            // Reference Data
            Self::ReferenceConditions => "/v3/reference/conditions",
            Self::ReferenceExchanges => "/v3/reference/exchanges",
        }
    }

    /// All endpoints require authentication (API key)
    pub fn _requires_auth(&self) -> bool {
        true
    }

    /// HTTP method for endpoint
    pub fn _method(&self) -> &'static str {
        "GET" // Polygon only has GET endpoints
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Polygon API
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
    // Polygon uses simple ticker symbols
    symbol.to_uppercase()
}

/// Map interval to Polygon timespan
///
/// # Polygon Timespan Format
/// Parameter: `timespan` (string)
/// Values: `"minute"`, `"hour"`, `"day"`, `"week"`, `"month"`, `"quarter"`, `"year"`
pub fn map_timespan(interval: &str) -> &'static str {
    match interval {
        "1m" | "3m" | "5m" | "15m" | "30m" => "minute",
        "1h" | "2h" | "4h" | "6h" | "8h" | "12h" => "hour",
        "1d" => "day",
        "1w" => "week",
        "1M" => "month",
        _ => "day", // default
    }
}

/// Extract multiplier from interval
///
/// # Examples
/// - "1m" -> 1
/// - "5m" -> 5
/// - "1h" -> 1
/// - "1d" -> 1
pub fn extract_multiplier(interval: &str) -> u32 {
    interval.chars()
        .take_while(|c| c.is_numeric())
        .collect::<String>()
        .parse()
        .unwrap_or(1)
}
