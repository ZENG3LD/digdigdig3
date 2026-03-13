//! # Hyperliquid Endpoints
//!
//! URL definitions and endpoint enum for Hyperliquid API.
//!
//! ## API Structure
//!
//! Hyperliquid uses unified POST endpoints:
//! - `/info` - All market data and account queries (GET-like operations)
//! - `/exchange` - All trading operations (requires signatures)
//!
//! ## Symbol Formats
//!
//! - **Perpetuals**: Use coin name directly ("BTC", "ETH")
//! - **Spot**: Use "@{index}" format ("@0", "@107")
//! - **Info endpoints**: Accept symbol names
//! - **Exchange endpoints**: Require asset IDs (integers)

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Hyperliquid API
#[derive(Debug, Clone)]
pub struct HyperliquidUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl HyperliquidUrls {
    /// Mainnet URLs
    pub const MAINNET: Self = Self {
        rest: "https://api.hyperliquid.xyz",
        ws: "wss://api.hyperliquid.xyz/ws",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        rest: "https://api.hyperliquid-testnet.xyz",
        ws: "wss://api.hyperliquid-testnet.xyz/ws",
    };

    /// Get REST base URL
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Get WebSocket URL
    pub fn ws_url(&self) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Hyperliquid API endpoints
///
/// All requests go through two unified endpoints:
/// - `/info` for queries
/// - `/exchange` for trading operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HyperliquidEndpoint {
    // === UNIFIED ENDPOINTS ===
    Info,     // All queries (market data, account info)
    Exchange, // All trading operations (orders, leverage, transfers)
}

impl HyperliquidEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            Self::Info => "/info",
            Self::Exchange => "/exchange",
        }
    }

    /// Does endpoint require authentication
    ///
    /// Note: /info can be used with or without auth depending on query type
    pub fn requires_auth(&self) -> bool {
        match self {
            Self::Info => false,     // Can query public data without auth
            Self::Exchange => true,  // Always requires signature
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        "POST" // Both endpoints use POST with JSON body
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INFO REQUEST TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Info endpoint request types (unified query system)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum InfoType {
    // Market Data
    MetaAndAssetCtxs,       // Get all asset metadata and contexts (ticker data)
    Meta,                   // Get perpetuals metadata
    SpotMeta,               // Get spot metadata
    AllMids,                // Get all mid prices
    L2Book,                 // Get order book
    RecentTrades,           // Get recent trades
    CandleSnapshot,         // Get klines/candles
    FundingHistory,         // Get historical funding rates

    // Account Data (requires user address — NO signature needed, just the address)
    ClearinghouseState,     // Get perpetuals account state (balances + positions)
    SpotClearinghouseState, // Get spot account state
    OpenOrders,             // Get open orders
    OrderStatus,            // Get single order status by oid
    UserFills,              // Get trade history (fills)
    UserFillsByTime,        // Get trade history with time range
    UserFees,               // Get user fee tier
    UserRateLimit,          // Get rate limit status
    HistoricalOrders,       // Get historical orders
}

impl InfoType {
    /// Get the type string for the request body
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MetaAndAssetCtxs => "metaAndAssetCtxs",
            Self::Meta => "meta",
            Self::SpotMeta => "spotMeta",
            Self::AllMids => "allMids",
            Self::L2Book => "l2Book",
            Self::RecentTrades => "recentTrades",
            Self::CandleSnapshot => "candleSnapshot",
            Self::FundingHistory => "fundingHistory",
            Self::ClearinghouseState => "clearinghouseState",
            Self::SpotClearinghouseState => "spotClearinghouseState",
            Self::OpenOrders => "openOrders",
            Self::OrderStatus => "orderStatus",
            Self::UserFills => "userFills",
            Self::UserFillsByTime => "userFillsByTime",
            Self::UserFees => "userFees",
            Self::UserRateLimit => "userRateLimit",
            Self::HistoricalOrders => "historicalOrders",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE ACTION TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Exchange endpoint action types (trading operations)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[allow(dead_code)]
pub enum ActionType {
    // Trading
    Order,              // Place order(s)
    Cancel,             // Cancel order(s) by ID
    CancelByCloid,      // Cancel order(s) by client order ID
    Modify,             // Modify existing order
    /// Batch modify multiple orders: batchModify action on /exchange
    BatchModify,        // Batch modify multiple orders in one request

    // Position Management
    UpdateLeverage,     // Set leverage for asset
    UpdateIsolatedMargin, // Update isolated margin

    // Transfers
    UsdClassTransfer,   // Transfer between spot and perp
    UsdSend,            // Internal USDC transfer
    SpotSend,           // Internal spot token transfer
    Withdraw3,          // Withdraw to L1
}

impl ActionType {
    /// Get the type string for the action
    #[allow(dead_code)]
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Order => "order",
            Self::Cancel => "cancel",
            Self::CancelByCloid => "cancelByCloid",
            Self::Modify => "modify",
            Self::BatchModify => "batchModify",
            Self::UpdateLeverage => "updateLeverage",
            Self::UpdateIsolatedMargin => "updateIsolatedMargin",
            Self::UsdClassTransfer => "usdClassTransfer",
            Self::UsdSend => "usdSend",
            Self::SpotSend => "spotSend",
            Self::Withdraw3 => "withdraw3",
        }
    }

    /// Does this action require L1 signing (phantom agent)
    #[allow(dead_code)]
    pub fn is_l1_action(&self) -> bool {
        matches!(self,
            Self::Order
            | Self::Cancel
            | Self::CancelByCloid
            | Self::Modify
            | Self::BatchModify
            | Self::UpdateLeverage
            | Self::UpdateIsolatedMargin
            | Self::UsdClassTransfer
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Hyperliquid API
///
/// # Symbol Formats
/// - **Perpetuals**: Direct coin name ("BTC", "ETH", "SOL")
/// - **Spot**: "@{index}" format ("@0" for PURR/USDC, "@107" for HYPE/USDC)
///
/// # Examples
/// ```
/// // Perpetuals
/// assert_eq!(_format_symbol("BTC", AccountType::FuturesCross), "BTC");
///
/// // Spot (if input already has @ prefix)
/// assert_eq!(_format_symbol("@107", AccountType::Spot), "@107");
/// ```
#[allow(dead_code)]
pub fn format_symbol(symbol: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot => {
            // Spot symbols use @index format
            if symbol.starts_with('@') {
                symbol.to_string()
            } else {
                // If it's a pair name like "HYPE/USDC", pass through as-is
                // The connector will need to convert to @index using metadata
                symbol.to_string()
            }
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Perpetuals use direct coin names
            symbol.to_uppercase()
        }
        AccountType::Margin => {
            // Hyperliquid doesn't have margin trading in the traditional sense
            // Cross margin is the default for perpetuals
            symbol.to_uppercase()
        }
    }
}

/// Map kline interval to Hyperliquid format
///
/// # Supported Intervals
/// "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "8h", "12h", "1d", "3d", "1w", "1M"
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "3m" => "3m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1h",
        "2h" => "2h",
        "4h" => "4h",
        "8h" => "8h",
        "12h" => "12h",
        "1d" => "1d",
        "3d" => "3d",
        "1w" => "1w",
        "1M" => "1M",
        _ => "1h", // default to 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        assert_eq!(format_symbol("BTC", AccountType::FuturesCross), "BTC");
        assert_eq!(format_symbol("eth", AccountType::FuturesCross), "ETH");
        assert_eq!(format_symbol("@107", AccountType::Spot), "@107");
        assert_eq!(format_symbol("HYPE/USDC", AccountType::Spot), "HYPE/USDC");
    }

    #[test]
    fn test_endpoints() {
        assert_eq!(HyperliquidEndpoint::Info.path(), "/info");
        assert_eq!(HyperliquidEndpoint::Exchange.path(), "/exchange");
        assert_eq!(HyperliquidEndpoint::Info.method(), "POST");
        assert_eq!(HyperliquidEndpoint::Exchange.method(), "POST");
    }

    #[test]
    fn test_info_types() {
        assert_eq!(InfoType::Meta.as_str(), "meta");
        assert_eq!(InfoType::SpotMeta.as_str(), "spotMeta");
        assert_eq!(InfoType::L2Book.as_str(), "l2Book");
    }

    #[test]
    fn test_action_types() {
        assert_eq!(ActionType::Order.as_str(), "order");
        assert_eq!(ActionType::Cancel.as_str(), "cancel");
        assert!(ActionType::Order.is_l1_action());
        assert!(!ActionType::UsdSend.is_l1_action());
    }
}
