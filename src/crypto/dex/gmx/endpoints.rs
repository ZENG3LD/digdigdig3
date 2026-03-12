//! # GMX Endpoints
//!
//! URL's and endpoint enum for GMX V2 API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL's for GMX API
#[derive(Debug, Clone)]
pub struct GmxUrls {
    pub arbitrum_rest: &'static str,
    pub arbitrum_fallback1: &'static str,
    pub arbitrum_fallback2: &'static str,
    pub avalanche_rest: &'static str,
    pub avalanche_fallback1: &'static str,
    pub avalanche_fallback2: &'static str,
}

impl GmxUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        arbitrum_rest: "https://arbitrum-api.gmxinfra.io",
        arbitrum_fallback1: "https://arbitrum-api-fallback.gmxinfra.io",
        arbitrum_fallback2: "https://arbitrum-api-fallback2.gmxinfra.io",
        avalanche_rest: "https://avalanche-api.gmxinfra.io",
        avalanche_fallback1: "https://avalanche-api-fallback.gmxinfra.io",
        avalanche_fallback2: "https://avalanche-api-fallback2.gmxinfra.io",
    };

    /// Get REST base URL for chain
    pub fn rest_url(&self, chain: &str) -> &str {
        match chain.to_lowercase().as_str() {
            "arbitrum" | "arb" => self.arbitrum_rest,
            "avalanche" | "avax" => self.avalanche_rest,
            _ => self.arbitrum_rest, // Default to Arbitrum
        }
    }

    /// Get all fallback URLs for a chain
    pub fn fallback_urls(&self, chain: &str) -> Vec<&str> {
        match chain.to_lowercase().as_str() {
            "arbitrum" | "arb" => vec![
                self.arbitrum_fallback1,
                self.arbitrum_fallback2,
            ],
            "avalanche" | "avax" => vec![
                self.avalanche_fallback1,
                self.avalanche_fallback2,
            ],
            _ => vec![
                self.arbitrum_fallback1,
                self.arbitrum_fallback2,
            ],
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// GMX API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GmxEndpoint {
    // === GENERAL ===
    Ping,

    // === MARKET DATA ===
    Tickers,
    SignedPrices,
    Candles,
    Tokens,
    Markets,
    MarketInfo,
    FeeAPY,
    Performance,
    GlvTokens,
    GlvInfo,
}

impl GmxEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // General
            Self::Ping => "/ping",

            // Market Data
            Self::Tickers => "/prices/tickers",
            Self::SignedPrices => "/signed_prices/latest",
            Self::Candles => "/prices/candles",
            Self::Tokens => "/tokens",
            Self::Markets => "/markets",
            Self::MarketInfo => "/markets/info",
            Self::FeeAPY => "/apy",
            Self::Performance => "/performance/annualized",
            Self::GlvTokens => "/glvs",
            Self::GlvInfo => "/glvs/info",
        }
    }

    /// Does endpoint require authentication?
    /// GMX REST endpoints are all public (no auth required)
    pub fn requires_auth(&self) -> bool {
        false
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        "GET"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for GMX
///
/// # GMX Symbol Format
/// GMX uses a unique market structure:
/// - Format: `{INDEX}/{QUOTE} [{LONG}-{SHORT}]`
/// - Example: `ETH/USD [ETH-USDC]`
/// - Multiple pools per index token (different collateral pairs)
///
/// # Examples
/// - `ETH/USD [ETH-USDC]` - Long ETH with ETH collateral, short with USDC
/// - `BTC/USD [BTC-USDT]` - Long BTC with BTC collateral, short with USDT
///
/// # Note
/// This function returns the simplified format without collateral info.
/// Use get_markets() to get full market details including collateral tokens.
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    // GMX only supports USD-quoted perpetuals
    // AccountType is ignored as GMX has a single market type
    format!("{}/{}", base.to_uppercase(), quote.to_uppercase())
}

/// Parse GMX market symbol into components
///
/// # Formats
/// - Simple: "ETH/USD" -> (ETH, USD, None, None)
/// - Full: "ETH/USD [ETH-USDC]" -> (ETH, USD, Some(ETH), Some(USDC))
pub fn _parse_symbol(symbol: &str) -> (String, String, Option<String>, Option<String>) {
    if let Some(bracket_pos) = symbol.find('[') {
        // Full format: "ETH/USD [ETH-USDC]"
        let pair_part = &symbol[..bracket_pos].trim();
        let collateral_part = &symbol[bracket_pos + 1..]
            .trim()
            .trim_end_matches(']');

        let pair_parts: Vec<&str> = pair_part.split('/').collect();
        let collateral_parts: Vec<&str> = collateral_part.split('-').collect();

        let index = pair_parts.first().unwrap_or(&"").to_string();
        let quote = pair_parts.get(1).unwrap_or(&"USD").to_string();
        let long = collateral_parts.first().map(|s| s.to_string());
        let short = collateral_parts.get(1).map(|s| s.to_string());

        (index, quote, long, short)
    } else {
        // Simple format: "ETH/USD"
        let parts: Vec<&str> = symbol.split('/').collect();
        let index = parts.first().unwrap_or(&"").to_string();
        let quote = parts.get(1).unwrap_or(&"USD").to_string();

        (index, quote, None, None)
    }
}

/// Map kline interval to GMX period format
///
/// # GMX Supported Periods
/// - `1m`, `5m`, `15m`, `1h`, `4h`, `1d`
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "15m" => "15m",
        "1h" => "1h",
        "4h" => "4h",
        "1d" => "1d",
        _ => "1h", // Default to 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        assert_eq!(
            format_symbol("ETH", "USD", AccountType::Spot),
            "ETH/USD"
        );
        assert_eq!(
            format_symbol("btc", "usd", AccountType::FuturesCross),
            "BTC/USD"
        );
    }

    #[test]
    fn test_parse_symbol() {
        // Simple format
        let (index, quote, long, short) = parse_symbol("ETH/USD");
        assert_eq!(index, "ETH");
        assert_eq!(quote, "USD");
        assert_eq!(long, None);
        assert_eq!(short, None);

        // Full format
        let (index, quote, long, short) = parse_symbol("ETH/USD [ETH-USDC]");
        assert_eq!(index, "ETH");
        assert_eq!(quote, "USD");
        assert_eq!(long, Some("ETH".to_string()));
        assert_eq!(short, Some("USDC".to_string()));
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "1m");
        assert_eq!(map_kline_interval("1h"), "1h");
        assert_eq!(map_kline_interval("4h"), "4h");
        assert_eq!(map_kline_interval("1d"), "1d");
        assert_eq!(map_kline_interval("invalid"), "1h"); // default
    }

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(GmxEndpoint::Ping.path(), "/ping");
        assert_eq!(GmxEndpoint::Tickers.path(), "/prices/tickers");
        assert_eq!(GmxEndpoint::Candles.path(), "/prices/candles");
        assert_eq!(GmxEndpoint::Markets.path(), "/markets");
    }

    #[test]
    fn test_no_auth_required() {
        // All GMX REST endpoints are public
        assert!(!GmxEndpoint::Ping.requires_auth());
        assert!(!GmxEndpoint::Tickers.requires_auth());
        assert!(!GmxEndpoint::SignedPrices.requires_auth());
        assert!(!GmxEndpoint::Markets.requires_auth());
    }
}
