//! # Tiingo API Endpoints
//!
//! URL structures and endpoint definitions for Tiingo API.

use crate::core::types::Symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL structures for Tiingo API
#[derive(Debug, Clone)]
pub struct TiingoUrls {
    pub rest_base: &'static str,
    pub _ws_iex: &'static str,
    pub _ws_forex: &'static str,
    pub _ws_crypto: &'static str,
}

impl TiingoUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest_base: "https://api.tiingo.com",
        _ws_iex: "wss://api.tiingo.com/iex",
        _ws_forex: "wss://api.tiingo.com/fx",
        _ws_crypto: "wss://api.tiingo.com/crypto",
    };

    /// Get REST base URL
    pub fn rest_url(&self) -> &str {
        self.rest_base
    }

    /// Get WebSocket URL for stocks (IEX)
    pub fn _ws_iex_url(&self) -> &str {
        self._ws_iex
    }

    /// Get WebSocket URL for forex
    pub fn _ws_forex_url(&self) -> &str {
        self._ws_forex
    }

    /// Get WebSocket URL for crypto
    pub fn _ws_crypto_url(&self) -> &str {
        self._ws_crypto
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Tiingo API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum TiingoEndpoint {
    // === EOD STOCK DATA ===
    DailyMeta,           // /tiingo/daily/{ticker} - ticker metadata
    DailyPrices,         // /tiingo/daily/{ticker}/prices - historical daily prices

    // === IEX INTRADAY DATA ===
    IexMeta,             // /iex/{ticker} - IEX metadata
    IexPrices,           // /iex/{ticker}/prices - intraday prices

    // === CRYPTO DATA ===
    CryptoMeta,          // /tiingo/crypto - crypto metadata
    CryptoTop,           // /tiingo/crypto/top - top-of-book quotes
    CryptoPrices,        // /tiingo/crypto/prices - historical crypto prices

    // === FOREX DATA ===
    ForexTop,            // /tiingo/fx/{ticker}/top - top-of-book FX quote
    ForexPrices,         // /tiingo/fx/{ticker}/prices - historical FX prices

    // === FUNDAMENTALS ===
    FundamentalsDefinitions, // /tiingo/fundamentals/definitions
    FundamentalsDaily,       // /tiingo/fundamentals/{ticker}/daily
    FundamentalsStatements,  // /tiingo/fundamentals/{ticker}/statements

    // === NEWS ===
    News,                // /tiingo/news - financial news
}

impl TiingoEndpoint {
    /// Get endpoint path template
    pub fn path(&self) -> &'static str {
        match self {
            // EOD Stock Data
            Self::DailyMeta => "/tiingo/daily/{ticker}",
            Self::DailyPrices => "/tiingo/daily/{ticker}/prices",

            // IEX Intraday Data
            Self::IexMeta => "/iex/{ticker}",
            Self::IexPrices => "/iex/{ticker}/prices",

            // Crypto Data
            Self::CryptoMeta => "/tiingo/crypto",
            Self::CryptoTop => "/tiingo/crypto/top",
            Self::CryptoPrices => "/tiingo/crypto/prices",

            // Forex Data
            Self::ForexTop => "/tiingo/fx/{ticker}/top",
            Self::ForexPrices => "/tiingo/fx/{ticker}/prices",

            // Fundamentals
            Self::FundamentalsDefinitions => "/tiingo/fundamentals/definitions",
            Self::FundamentalsDaily => "/tiingo/fundamentals/{ticker}/daily",
            Self::FundamentalsStatements => "/tiingo/fundamentals/{ticker}/statements",

            // News
            Self::News => "/tiingo/news",
        }
    }

    /// All endpoints require authentication
    pub fn _requires_auth(&self) -> bool {
        true
    }

    /// HTTP method for endpoint
    pub fn _method(&self) -> &'static str {
        "GET" // Tiingo only has GET endpoints
    }

    /// Build endpoint URL with ticker replacement
    pub fn build_url(&self, base_url: &str, ticker: Option<&str>) -> String {
        let path = self.path();
        let url = format!("{}{}", base_url, path);

        if let Some(ticker) = ticker {
            url.replace("{ticker}", ticker)
        } else {
            url
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Tiingo API (stock symbols)
///
/// # Stock Symbol Format
/// - US stocks: Just the ticker symbol (e.g., "AAPL", "MSFT")
/// - No base/quote separation for stocks
///
/// # Examples
/// - Apple: "AAPL"
/// - Microsoft: "MSFT"
/// - Tesla: "TSLA"
pub fn format_stock_symbol(symbol: &str) -> String {
    symbol.to_uppercase()
}

/// Format symbol for crypto pairs
///
/// # Crypto Symbol Format
/// - Lowercase, concatenated (e.g., "btcusd", "ethusd")
/// - basecurrency + quotecurrency
///
/// # Examples
/// - Bitcoin: "btcusd"
/// - Ethereum: "ethusd"
pub fn format_crypto_symbol(symbol: &Symbol) -> String {
    format!("{}{}", symbol.base, symbol.quote).to_lowercase()
}

/// Format symbol for forex pairs
///
/// # Forex Symbol Format
/// - Lowercase, concatenated (e.g., "eurusd", "gbpjpy")
///
/// # Examples
/// - EUR/USD: "eurusd"
/// - GBP/JPY: "gbpjpy"
pub fn format_forex_symbol(symbol: &Symbol) -> String {
    format!("{}{}", symbol.base, symbol.quote).to_lowercase()
}

/// Map interval string to Tiingo resample frequency
///
/// # Tiingo Resample Formats
/// - EOD: "daily", "weekly", "monthly", "annually"
/// - IEX: "1min", "5min", "15min", "30min", "1hour", "4hour"
/// - Crypto: "1min", "5min", "15min", "1hour", "1day"
/// - Forex: "1min", "5min", "15min", "30min", "1hour", "4hour", "1day"
pub fn map_interval(interval: &str) -> &'static str {
    match interval {
        // Minutes
        "1m" => "1min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",

        // Hours
        "1h" => "1hour",
        "4h" => "4hour",

        // Days/Weeks/Months
        "1d" => "1day",
        "1w" => "weekly",
        "1M" => "monthly",

        // Default
        _ => "1day",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_stock_symbol() {
        assert_eq!(format_stock_symbol("aapl"), "AAPL");
        assert_eq!(format_stock_symbol("MSFT"), "MSFT");
    }

    #[test]
    fn test_format_crypto_symbol() {
        let btc = Symbol::new("BTC", "USD");
        assert_eq!(format_crypto_symbol(&btc), "btcusd");

        let eth = Symbol::new("eth", "usdt");
        assert_eq!(format_crypto_symbol(&eth), "ethusdt");
    }

    #[test]
    fn test_format_forex_symbol() {
        let eur = Symbol::new("EUR", "USD");
        assert_eq!(format_forex_symbol(&eur), "eurusd");
    }

    #[test]
    fn test_map_interval() {
        assert_eq!(map_interval("1m"), "1min");
        assert_eq!(map_interval("5m"), "5min");
        assert_eq!(map_interval("1h"), "1hour");
        assert_eq!(map_interval("1d"), "1day");
        assert_eq!(map_interval("1w"), "weekly");
    }

    #[test]
    fn test_endpoint_build_url() {
        let endpoint = TiingoEndpoint::DailyPrices;
        let url = endpoint.build_url("https://api.tiingo.com", Some("AAPL"));
        assert_eq!(url, "https://api.tiingo.com/tiingo/daily/AAPL/prices");
    }
}
