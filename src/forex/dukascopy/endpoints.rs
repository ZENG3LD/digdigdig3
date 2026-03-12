//! Dukascopy API endpoints
//!
//! Dukascopy uses binary file downloads (.bi5 format) instead of REST API.
//! URL pattern: https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5
//!
//! Month is 0-indexed: 00=January, 01=February, ..., 11=December

/// Base URLs for Dukascopy datafeed
pub struct DukascopyUrls {
    /// Base URL for binary tick data downloads
    pub datafeed_base: &'static str,
}

impl Default for DukascopyUrls {
    fn default() -> Self {
        Self {
            datafeed_base: "https://datafeed.dukascopy.com/datafeed",
        }
    }
}

/// API endpoint enum (for binary downloads)
#[derive(Debug, Clone)]
pub enum DukascopyEndpoint {
    /// Historical tick data (binary .bi5 file)
    /// Path: /{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5
    HistoricalTicks,
}

impl DukascopyEndpoint {
    /// Get endpoint path pattern
    pub fn path_pattern(&self) -> &'static str {
        match self {
            Self::HistoricalTicks => "/{symbol}/{year}/{month}/{day}/{hour}h_ticks.bi5",
        }
    }
}

/// Format symbol for Dukascopy API
///
/// Dukascopy uses format: EURUSD, GBPUSD, XAUUSD (no separator)
/// - Forex pairs: EURUSD, GBPUSD, USDJPY
/// - Metals: XAUUSD (gold), XAGUSD (silver)
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    format!("{}{}", symbol.base, symbol.quote).to_uppercase()
}

/// Parse symbol from Dukascopy format back to domain Symbol
///
/// Examples:
/// - "EURUSD" -> Symbol { base: "EUR", quote: "USD" }
/// - "GBPJPY" -> Symbol { base: "GBP", quote: "JPY" }
pub fn parse_symbol(api_symbol: &str) -> crate::core::types::Symbol {
    // Most forex pairs are 6 characters (3+3)
    if api_symbol.len() == 6 {
        crate::core::types::Symbol {
            base: api_symbol[0..3].to_string(),
            quote: api_symbol[3..6].to_string(),
            raw: Some(api_symbol.to_string()),
        }
    } else if api_symbol.len() == 7 && api_symbol.starts_with("XAU") {
        // Metals: XAUUSD, XAGUSD
        crate::core::types::Symbol {
            base: api_symbol[0..3].to_string(),
            quote: api_symbol[3..].to_string(),
            raw: Some(api_symbol.to_string()),
        }
    } else {
        // Fallback: assume 3-char base, rest is quote
        let base_len = std::cmp::min(3, api_symbol.len() / 2);
        crate::core::types::Symbol {
            base: api_symbol[0..base_len].to_string(),
            quote: api_symbol[base_len..].to_string(),
            raw: Some(api_symbol.to_string()),
        }
    }
}

/// Build URL for hourly tick data download
///
/// # Arguments
/// * `symbol` - Instrument symbol (e.g., "EURUSD")
/// * `year` - 4-digit year (e.g., 2024)
/// * `month` - 0-indexed month (0=Jan, 11=Dec)
/// * `day` - Day of month (1-31, but 0-padded in URL)
/// * `hour` - Hour of day (0-23)
///
/// # Example
/// ```ignore
/// let url = build_tick_data_url("EURUSD", 2024, 0, 15, 14);
/// // Returns: "https://datafeed.dukascopy.com/datafeed/EURUSD/2024/00/15/14h_ticks.bi5"
/// ```
pub fn build_tick_data_url(
    symbol: &str,
    year: u32,
    month: u32,
    day: u32,
    hour: u32,
) -> String {
    let urls = DukascopyUrls::default();
    format!(
        "{}/{}/{:04}/{:02}/{:02}/{:02}h_ticks.bi5",
        urls.datafeed_base,
        symbol.to_uppercase(),
        year,
        month,
        day,
        hour
    )
}

/// Get point value (pip precision) for a symbol
///
/// - Most forex pairs: 0.00001 (5 decimals)
/// - JPY pairs: 0.001 (3 decimals)
/// - Metals: 0.00001 (5 decimals)
pub fn get_point_value(symbol: &str) -> f64 {
    if symbol.contains("JPY") {
        0.001 // 3 decimals for JPY pairs
    } else {
        0.00001 // 5 decimals for most pairs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        let symbol = crate::core::types::Symbol {
            base: "EUR".to_string(),
            quote: "USD".to_string(),
            raw: None,
        };
        assert_eq!(format_symbol(&symbol), "EURUSD");
    }

    #[test]
    fn test_parse_symbol() {
        let symbol = parse_symbol("EURUSD");
        assert_eq!(symbol.base, "EUR");
        assert_eq!(symbol.quote, "USD");

        let symbol = parse_symbol("GBPJPY");
        assert_eq!(symbol.base, "GBP");
        assert_eq!(symbol.quote, "JPY");

        let symbol = parse_symbol("XAUUSD");
        assert_eq!(symbol.base, "XAU");
        assert_eq!(symbol.quote, "USD");
    }

    #[test]
    fn test_build_tick_data_url() {
        let url = build_tick_data_url("EURUSD", 2024, 0, 15, 14);
        assert_eq!(
            url,
            "https://datafeed.dukascopy.com/datafeed/EURUSD/2024/00/15/14h_ticks.bi5"
        );

        // Test month padding (November = month 10)
        let url = build_tick_data_url("GBPUSD", 2023, 10, 5, 9);
        assert_eq!(
            url,
            "https://datafeed.dukascopy.com/datafeed/GBPUSD/2023/10/05/09h_ticks.bi5"
        );
    }

    #[test]
    fn test_get_point_value() {
        assert_eq!(get_point_value("EURUSD"), 0.00001);
        assert_eq!(get_point_value("GBPUSD"), 0.00001);
        assert_eq!(get_point_value("USDJPY"), 0.001);
        assert_eq!(get_point_value("EURJPY"), 0.001);
        assert_eq!(get_point_value("XAUUSD"), 0.00001);
    }
}
