//! # Vertex Protocol Endpoints
//!
//! URL constants, endpoint enum, and symbol formatting for Vertex Protocol API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL constants for Vertex Protocol API
#[derive(Debug, Clone)]
pub struct VertexUrls {
    pub rest: &'static str,
    pub websocket: &'static str,
    pub subscribe: &'static str,
    pub indexer: &'static str,
}

impl VertexUrls {
    /// Production URLs (Arbitrum One)
    pub const MAINNET: Self = Self {
        rest: "https://gateway.prod.vertexprotocol.com/v1",
        websocket: "wss://gateway.prod.vertexprotocol.com/v1/ws",
        subscribe: "wss://gateway.prod.vertexprotocol.com/v1/subscribe",
        indexer: "https://archive.prod.vertexprotocol.com/v1",
    };

    /// Testnet URLs (Arbitrum Sepolia)
    pub const TESTNET: Self = Self {
        rest: "https://gateway.sepolia-test.vertexprotocol.com/v1",
        websocket: "wss://gateway.sepolia-test.vertexprotocol.com/v1/ws",
        subscribe: "wss://gateway.sepolia-test.vertexprotocol.com/v1/subscribe",
        indexer: "https://archive.sepolia-test.vertexprotocol.com/v1",
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Vertex Protocol API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VertexEndpoint {
    // === GATEWAY QUERY ===
    AllProducts,
    Symbols,
    MarketLiquidity,
    MarketPrice,
    Contracts,
    Status,
    SubaccountInfo,
    FeeRates,
    MaxWithdrawable,
    SubaccountOrders,
    Order,
    MaxOrderSize,

    // === GATEWAY EXECUTE ===
    Execute,

    // === ARCHIVE INDEXER ===
    Candlesticks,
    ProductSnapshots,
    FundingRate,
}

impl VertexEndpoint {
    /// Get the path for this endpoint
    pub fn path(&self) -> &'static str {
        match self {
            // Query endpoints
            Self::AllProducts => "/query",
            Self::Symbols => "/symbols",
            Self::MarketLiquidity => "/query",
            Self::MarketPrice => "/query",
            Self::Contracts => "/query",
            Self::Status => "/query",
            Self::SubaccountInfo => "/query",
            Self::FeeRates => "/query",
            Self::MaxWithdrawable => "/query",
            Self::SubaccountOrders => "/query",
            Self::Order => "/query",
            Self::MaxOrderSize => "/query",

            // Execute endpoint
            Self::Execute => "/execute",

            // Archive indexer
            Self::Candlesticks => "",
            Self::ProductSnapshots => "",
            Self::FundingRate => "",
        }
    }

    /// Whether this endpoint requires authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::AllProducts
            | Self::Symbols
            | Self::MarketLiquidity
            | Self::MarketPrice
            | Self::Contracts
            | Self::Status => false,

            // Private endpoints
            Self::SubaccountInfo
            | Self::FeeRates
            | Self::MaxWithdrawable
            | Self::SubaccountOrders
            | Self::Order
            | Self::MaxOrderSize
            | Self::Execute => true,

            // Archive endpoints (public)
            Self::Candlesticks | Self::ProductSnapshots | Self::FundingRate => false,
        }
    }

    /// HTTP method for this endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Self::Execute | Self::Candlesticks | Self::ProductSnapshots | Self::FundingRate => {
                "POST"
            }
            Self::Symbols | Self::Status => "GET",
            _ => "GET",
        }
    }

    /// Query type parameter (for /query endpoints)
    pub fn query_type(&self) -> Option<&'static str> {
        match self {
            Self::AllProducts => Some("all_products"),
            Self::MarketLiquidity => Some("market_liquidity"),
            Self::MarketPrice => Some("market_price"),
            Self::Contracts => Some("contracts"),
            Self::Status => Some("status"),
            Self::SubaccountInfo => Some("subaccount_info"),
            Self::FeeRates => Some("fee_rates"),
            Self::MaxWithdrawable => Some("max_withdrawable"),
            Self::SubaccountOrders => Some("subaccount_orders"),
            Self::Order => Some("order"),
            Self::MaxOrderSize => Some("max_order_size"),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Vertex Protocol
///
/// # Format
/// - Spot: `{BASE}` (e.g., "BTC")
/// - Perpetuals: `{BASE}-PERP` (e.g., "BTC-PERP")
/// - All markets quote in USDC
///
/// # Examples
/// - Spot BTC: `format_symbol("BTC", "USDC", AccountType::Spot)` → "BTC"
/// - BTC Perpetual: `format_symbol("BTC", "USDC", AccountType::FuturesCross)` → "BTC-PERP"
pub fn format_symbol(base: &str, _quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => base.to_uppercase(),
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            format!("{}-PERP", base.to_uppercase())
        }
    }
}

/// Parse symbol from Vertex format to (base, quote)
///
/// # Examples
/// - "BTC" → ("BTC", "USDC")
/// - "BTC-PERP" → ("BTC", "USDC")
pub fn parse_symbol(symbol: &str) -> (String, String) {
    let base = symbol.replace("-PERP", "");
    (base, "USDC".to_string())
}

/// Check if symbol is a perpetual
pub fn is_perpetual(symbol: &str) -> bool {
    symbol.ends_with("-PERP")
}

/// Get base asset from symbol
pub fn get_base_asset(symbol: &str) -> String {
    symbol.replace("-PERP", "")
}

/// Map kline interval to granularity (seconds)
///
/// # Supported Intervals
/// - 1m → 60
/// - 5m → 300
/// - 15m → 900
/// - 1h → 3600
/// - 4h → 14400
/// - 1d → 86400
pub fn map_kline_interval(interval: &str) -> u32 {
    match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "1h" => 3600,
        "4h" => 14400,
        "1d" => 86400,
        _ => 3600, // default to 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_spot() {
        assert_eq!(
            format_symbol("BTC", "USDC", AccountType::Spot),
            "BTC"
        );
        assert_eq!(
            format_symbol("eth", "usdc", AccountType::Spot),
            "ETH"
        );
    }

    #[test]
    fn test_format_symbol_perp() {
        assert_eq!(
            format_symbol("BTC", "USDC", AccountType::FuturesCross),
            "BTC-PERP"
        );
        assert_eq!(
            format_symbol("eth", "usdc", AccountType::FuturesIsolated),
            "ETH-PERP"
        );
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_symbol("BTC"), ("BTC".to_string(), "USDC".to_string()));
        assert_eq!(
            parse_symbol("BTC-PERP"),
            ("BTC".to_string(), "USDC".to_string())
        );
    }

    #[test]
    fn test_is_perpetual() {
        assert!(!is_perpetual("BTC"));
        assert!(is_perpetual("BTC-PERP"));
        assert!(!is_perpetual("ETH"));
        assert!(is_perpetual("ETH-PERP"));
    }

    #[test]
    fn test_get_base_asset() {
        assert_eq!(get_base_asset("BTC"), "BTC");
        assert_eq!(get_base_asset("BTC-PERP"), "BTC");
        assert_eq!(get_base_asset("ETH-PERP"), "ETH");
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), 60);
        assert_eq!(map_kline_interval("5m"), 300);
        assert_eq!(map_kline_interval("15m"), 900);
        assert_eq!(map_kline_interval("1h"), 3600);
        assert_eq!(map_kline_interval("4h"), 14400);
        assert_eq!(map_kline_interval("1d"), 86400);
        assert_eq!(map_kline_interval("unknown"), 3600); // default
    }
}
