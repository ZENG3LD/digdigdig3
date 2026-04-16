//! # Twelvedata Endpoints
//!
//! URL's and endpoint enum for Twelvedata API.

use crate::core::types::Symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Twelvedata API
#[derive(Debug, Clone)]
pub struct TwelvedataUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl TwelvedataUrls {
    /// Production URLs
    pub const PRODUCTION: Self = Self {
        rest: "https://api.twelvedata.com",
        ws: "wss://ws.twelvedata.com",
    };
}

impl Default for TwelvedataUrls {
    fn default() -> Self {
        Self::PRODUCTION
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Twelvedata API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TwelvedataEndpoint {
    // === CORE MARKET DATA ===
    /// Latest price (1 credit per symbol)
    Price,
    /// Real-time quote (bid/ask, OHLCV, volume, 52w highs/lows) - 1 credit
    Quote,
    /// Historical OHLCV data (1 credit per symbol)
    TimeSeries,
    /// End of day OHLC (1 credit per symbol)
    Eod,
    /// Current exchange rate (1 credit)
    ExchangeRate,

    // === REFERENCE DATA (CATALOGS) ===
    /// List all stocks (1 credit, public)
    Stocks,
    /// List forex pairs (1 credit, public)
    ForexPairs,
    /// List cryptocurrencies (1 credit, public)
    Cryptocurrencies,
    /// List ETFs (1 credit, public)
    Etf,
    /// List commodities (1 credit, public)
    Commodities,
    /// List indices (1 credit, public)
    Indices,

    // === DISCOVERY & SEARCH ===
    /// Search symbols (1 credit)
    SymbolSearch,
    /// Get earliest available timestamp (1 credit)
    EarliestTimestamp,

    // === MARKETS INFO ===
    /// List exchanges (1 credit, public)
    Exchanges,
    /// Market open/closed status (1 credit)
    MarketState,

    // === TECHNICAL INDICATORS (selected common ones) ===
    /// RSI - Relative Strength Index
    Rsi,
    /// MACD - Moving Average Convergence/Divergence
    Macd,
    /// Bollinger Bands
    BBands,
    /// Simple Moving Average
    Sma,
    /// Exponential Moving Average
    Ema,

    // === FUNDAMENTALS (Grow+ tier) ===
    /// Company logo (1 credit)
    Logo,
    /// Company profile (10 credits, Grow+)
    Profile,
    /// Key statistics (varies)
    Statistics,

    // === REAL-TIME & COMPLEX DATA ===
    /// Real-time price (1 credit) — simpler than Quote, returns only price
    RealTimePrice,
    /// Complex data endpoint — batch multiple instruments/indicators in one call
    ComplexData,

    // === FUND REFERENCE DATA ===
    /// Mutual funds list (1 credit, public)
    MutualFundsList,
    /// Bonds list (1 credit, public)
    BondsList,
}

impl TwelvedataEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Core Market Data
            Self::Price => "/price",
            Self::Quote => "/quote",
            Self::TimeSeries => "/time_series",
            Self::Eod => "/eod",
            Self::ExchangeRate => "/exchange_rate",

            // Reference Data
            Self::Stocks => "/stocks",
            Self::ForexPairs => "/forex_pairs",
            Self::Cryptocurrencies => "/cryptocurrencies",
            Self::Etf => "/etf",
            Self::Commodities => "/commodities",
            Self::Indices => "/indices",

            // Discovery
            Self::SymbolSearch => "/symbol_search",
            Self::EarliestTimestamp => "/earliest_timestamp",

            // Markets Info
            Self::Exchanges => "/exchanges",
            Self::MarketState => "/market_state",

            // Technical Indicators
            Self::Rsi => "/rsi",
            Self::Macd => "/macd",
            Self::BBands => "/bbands",
            Self::Sma => "/sma",
            Self::Ema => "/ema",

            // Fundamentals
            Self::Logo => "/logo",
            Self::Profile => "/profile",
            Self::Statistics => "/statistics",

            // Real-time & Complex Data
            Self::RealTimePrice => "/price",
            Self::ComplexData => "/complex_data",

            // Fund Reference Data
            Self::MutualFundsList => "/mutual_funds/list",
            Self::BondsList => "/bonds/list",
        }
    }

    /// Requires authentication? (Some endpoints work with demo key)
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints (work without auth, but recommended)
            Self::Stocks
            | Self::ForexPairs
            | Self::Cryptocurrencies
            | Self::Etf
            | Self::Commodities
            | Self::Indices
            | Self::Exchanges
            | Self::MutualFundsList
            | Self::BondsList => false,

            // All other endpoints require API key
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        // All Twelvedata endpoints use GET
        "GET"
    }

    /// Credit cost (for rate limiting planning)
    pub fn credit_cost(&self) -> u32 {
        match self {
            // Basic market data - 1 credit
            Self::Price
            | Self::Quote
            | Self::TimeSeries
            | Self::Eod
            | Self::ExchangeRate
            | Self::SymbolSearch
            | Self::EarliestTimestamp
            | Self::MarketState
            | Self::Logo => 1,

            // Reference data - 1 credit
            Self::Stocks
            | Self::ForexPairs
            | Self::Cryptocurrencies
            | Self::Etf
            | Self::Commodities
            | Self::Indices
            | Self::Exchanges => 1,

            // Technical indicators - varies (typically 1)
            Self::Rsi | Self::Macd | Self::BBands | Self::Sma | Self::Ema => 1,

            // Fundamentals - higher cost
            Self::Profile => 10,
            Self::Statistics => 5,

            // Real-time & Complex Data
            Self::RealTimePrice => 1,
            Self::ComplexData => 1, // cost varies by included instruments/indicators

            // Fund Reference Data
            Self::MutualFundsList | Self::BondsList => 1,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Twelvedata API
///
/// # Twelvedata Symbol Formats
/// - **Stocks**: Ticker symbol (e.g., "AAPL", "TSLA")
/// - **Forex**: "EUR/USD", "GBP/JPY"
/// - **Crypto**: "BTC/USD", "ETH/USDT"
/// - **Exchange specification**: "AAPL:NASDAQ" (optional but recommended)
///
/// # Examples
/// ```ignore
/// format_symbol(&Symbol::new("AAPL", "USD")) => "AAPL"
/// format_symbol(&Symbol::new("BTC", "USD")) => "BTC/USD"
/// format_symbol(&Symbol::new("EUR", "USD")) => "EUR/USD"
/// ```
pub fn format_symbol(symbol: &Symbol) -> String {
    // For stocks, typically only base is used (ticker symbol)
    if symbol.quote.is_empty() || symbol.quote == "USD" {
        // Stock ticker - just base
        symbol.base.to_uppercase()
    } else {
        // Forex/Crypto - use BASE/QUOTE format
        format!("{}/{}", symbol.base, symbol.quote)
    }
}

/// Parse symbol from Twelvedata format back to domain Symbol
pub fn _parse_symbol(api_symbol: &str) -> Symbol {
    if let Some((base, quote)) = api_symbol.split_once('/') {
        // Forex/Crypto format: "BTC/USD"
        Symbol::new(base, quote)
    } else if let Some((ticker, _exchange)) = api_symbol.split_once(':') {
        // Exchange-qualified format: "AAPL:NASDAQ"
        Symbol::new(ticker, "USD")
    } else {
        // Stock format: "AAPL" (assume USD quote)
        Symbol::new(api_symbol, "USD")
    }
}

/// Map interval to Twelvedata format
///
/// # Twelvedata Intervals
/// - 1min, 5min, 15min, 30min, 45min
/// - 1h, 2h, 4h
/// - 1day, 1week, 1month
pub fn map_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1min",
        "3m" => "3min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "45m" => "45min",
        "1h" => "1h",
        "2h" => "2h",
        "4h" => "4h",
        "1d" => "1day",
        "1w" => "1week",
        "1M" => "1month",
        _ => "1h", // Default to 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_stock() {
        let symbol = Symbol::new("AAPL", "USD");
        assert_eq!(format_symbol(&symbol), "AAPL");
    }

    #[test]
    fn test_format_symbol_crypto() {
        let symbol = Symbol::new("BTC", "USDT");
        assert_eq!(format_symbol(&symbol), "BTC/USDT");
    }

    #[test]
    fn test_format_symbol_forex() {
        let symbol = Symbol::new("EUR", "USD");
        assert_eq!(format_symbol(&symbol), "EUR/USD");
    }

    #[test]
    fn test_parse_symbol_stock() {
        let symbol = _parse_symbol("AAPL");
        assert_eq!(symbol.base, "AAPL");
        assert_eq!(symbol.quote, "USD");
    }

    #[test]
    fn test_parse_symbol_crypto() {
        let symbol = _parse_symbol("BTC/USD");
        assert_eq!(symbol.base, "BTC");
        assert_eq!(symbol.quote, "USD");
    }

    #[test]
    fn test_parse_symbol_exchange_qualified() {
        let symbol = _parse_symbol("AAPL:NASDAQ");
        assert_eq!(symbol.base, "AAPL");
        assert_eq!(symbol.quote, "USD");
    }

    #[test]
    fn test_map_interval() {
        assert_eq!(map_interval("1m"), "1min");
        assert_eq!(map_interval("5m"), "5min");
        assert_eq!(map_interval("1h"), "1h");
        assert_eq!(map_interval("1d"), "1day");
        assert_eq!(map_interval("1w"), "1week");
        assert_eq!(map_interval("unknown"), "1h");
    }
}
