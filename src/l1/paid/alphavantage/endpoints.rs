//! # AlphaVantage API Endpoints
//!
//! AlphaVantage uses a function-based API where all requests go to the same base URL
//! with different `function` parameter values.

use crate::core::types::Symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// BASE URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// AlphaVantage API base URLs
#[derive(Debug, Clone)]
pub struct AlphaVantageEndpoints {
    pub rest_base: &'static str,
}

impl Default for AlphaVantageEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://www.alphavantage.co/query",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNCTION ENUM
// ═══════════════════════════════════════════════════════════════════════════════

/// AlphaVantage API function parameter values
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaVantageFunction {
    // === FOREX ===
    /// Real-time exchange rate for a currency pair
    CurrencyExchangeRate,
    /// Intraday forex time series (1min-60min intervals) - PREMIUM ONLY
    FxIntraday,
    /// Daily forex time series (OHLC, no volume)
    FxDaily,
    /// Weekly forex time series
    FxWeekly,
    /// Monthly forex time series
    FxMonthly,

    // === STOCKS ===
    /// Latest price and stats for a symbol
    GlobalQuote,
    /// Intraday stock time series - PREMIUM ONLY
    TimeSeriesIntraday,
    /// Daily stock time series
    TimeSeriesDaily,
    /// Weekly stock time series
    TimeSeriesWeekly,
    /// Monthly stock time series
    TimeSeriesMonthly,

    // === CRYPTO ===
    /// Daily crypto time series
    DigitalCurrencyDaily,
    /// Crypto fundamental rating
    CryptoRating,

    // === UTILITIES ===
    /// Search for ticker symbols
    SymbolSearch,
    /// Global market open/closed status
    MarketStatus,
}

impl AlphaVantageFunction {
    /// Get function name as string for API requests
    pub fn as_str(&self) -> &'static str {
        match self {
            // Forex
            Self::CurrencyExchangeRate => "CURRENCY_EXCHANGE_RATE",
            Self::FxIntraday => "FX_INTRADAY",
            Self::FxDaily => "FX_DAILY",
            Self::FxWeekly => "FX_WEEKLY",
            Self::FxMonthly => "FX_MONTHLY",

            // Stocks
            Self::GlobalQuote => "GLOBAL_QUOTE",
            Self::TimeSeriesIntraday => "TIME_SERIES_INTRADAY",
            Self::TimeSeriesDaily => "TIME_SERIES_DAILY",
            Self::TimeSeriesWeekly => "TIME_SERIES_WEEKLY",
            Self::TimeSeriesMonthly => "TIME_SERIES_MONTHLY",

            // Crypto
            Self::DigitalCurrencyDaily => "DIGITAL_CURRENCY_DAILY",
            Self::CryptoRating => "CRYPTO_RATING",

            // Utilities
            Self::SymbolSearch => "SYMBOL_SEARCH",
            Self::MarketStatus => "MARKET_STATUS",
        }
    }

    /// Check if function requires premium tier
    pub fn is_premium(&self) -> bool {
        matches!(self, Self::FxIntraday | Self::TimeSeriesIntraday)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format forex symbol for AlphaVantage API
///
/// AlphaVantage uses separate `from_symbol` and `to_symbol` parameters for forex.
///
/// # Examples
/// ```ignore
/// let symbol = Symbol::new("EUR", "USD");
/// let (from, to) = format_fx_symbol(&symbol);
/// assert_eq!(from, "EUR");
/// assert_eq!(to, "USD");
/// ```
pub fn format_fx_symbol(symbol: &Symbol) -> (String, String) {
    (symbol.base.to_uppercase(), symbol.quote.to_uppercase())
}

/// Map interval string to AlphaVantage interval format
///
/// # Examples
/// - "1m" -> "1min"
/// - "5m" -> "5min"
/// - "1h" -> "60min"
/// - "1d" -> "1d" (unchanged)
pub fn map_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "60m" | "1h" => "60min",
        // For unknown intervals, return a default
        _ => "60min",
    }
}
