//! # JQuants API Endpoints
//!
//! Base URL: https://api.jquants.com/v1

use crate::core::types::Symbol;

/// Base URLs for JQuants API
pub struct JQuantsUrls {
    pub rest_base: &'static str,
}

impl Default for JQuantsUrls {
    fn default() -> Self {
        Self {
            rest_base: "https://api.jquants.com/v1",
        }
    }
}

/// API endpoint enum
#[derive(Debug, Clone)]
pub enum JQuantsEndpoint {
    // Authentication
    AuthUser,          // POST /token/auth_user (get refresh token)
    AuthRefresh,       // POST /token/auth_refresh (get ID token)

    // Stock Price Data
    DailyQuotes,       // GET /prices/daily_quotes (daily OHLC)

    // Listed Issues / Symbols
    ListedInfo,        // GET /listed/info (symbol master)

    // Indices
    Indices,           // GET /indices (TOPIX OHLC)
    IndicesTopix,      // GET /indices/topix (TOPIX specific)

    // Derivatives
    DerivativesFutures,  // GET /derivatives/futures (futures OHLC)
    DerivativesOptions,  // GET /derivatives/options (options OHLC)

    // Financial Data
    FinStatements,     // GET /fins/statements (financial statements)
    FinDividend,       // GET /fins/dividend (cash dividends)
    FinAnnouncement,   // GET /fins/announcement (earnings calendar)

    // Market Trading Data
    MarketsTradingByType,  // GET /markets/trading_by_type
    MarketsShortSelling,   // GET /markets/short_selling
    MarketsBreakdown,      // GET /markets/breakdown
    MarketsMargin,         // GET /markets/margin
    MarketsTradingCalendar, // GET /markets/trading_calendar

    // Options
    OptionIndexOption,  // GET /option/index_option
}

impl JQuantsEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Authentication
            Self::AuthUser => "/token/auth_user",
            Self::AuthRefresh => "/token/auth_refresh",

            // Stock Price Data
            Self::DailyQuotes => "/prices/daily_quotes",

            // Listed Issues
            Self::ListedInfo => "/listed/info",

            // Indices
            Self::Indices => "/indices",
            Self::IndicesTopix => "/indices/topix",

            // Derivatives
            Self::DerivativesFutures => "/derivatives/futures",
            Self::DerivativesOptions => "/derivatives/options",

            // Financial Data
            Self::FinStatements => "/fins/statements",
            Self::FinDividend => "/fins/dividend",
            Self::FinAnnouncement => "/fins/announcement",

            // Market Trading Data
            Self::MarketsTradingByType => "/markets/trading_by_type",
            Self::MarketsShortSelling => "/markets/short_selling",
            Self::MarketsBreakdown => "/markets/breakdown",
            Self::MarketsMargin => "/markets/margin",
            Self::MarketsTradingCalendar => "/markets/trading_calendar",

            // Options
            Self::OptionIndexOption => "/option/index_option",
        }
    }
}

/// Format symbol for JQuants API
///
/// JQuants expects stock codes (4 or 5 digits), not base-quote pairs.
/// For Japanese stocks, we use the base as the stock code.
///
/// Examples:
/// - Symbol { base: "7203", quote: "JPY" } → "7203" (Toyota)
/// - Symbol { base: "6758", quote: "JPY" } → "6758" (Sony)
pub fn format_symbol(symbol: &Symbol) -> String {
    // For Japanese stocks, the "base" is the stock code
    symbol.base.clone()
}

/// Parse symbol from API format back to domain Symbol
///
/// JQuants returns stock codes. We convert them to Symbol with JPY quote.
pub fn _parse_symbol(code: &str) -> Symbol {
    Symbol {
        base: code.to_string(),
        quote: "JPY".to_string(),
        raw: Some(code.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        let symbol = Symbol {
            base: "7203".to_string(),
            quote: "JPY".to_string(),
            raw: None,
        };
        assert_eq!(format_symbol(&symbol), "7203");
    }

    #[test]
    fn test_parse_symbol() {
        let symbol = _parse_symbol("6758");
        assert_eq!(symbol.base, "6758");
        assert_eq!(symbol.quote, "JPY");
    }
}
