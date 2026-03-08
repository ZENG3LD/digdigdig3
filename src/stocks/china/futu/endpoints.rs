//! Futu OpenAPI connection parameters
//!
//! Futu does NOT use HTTP REST endpoints. Instead, it uses:
//! - TCP connection to OpenD gateway
//! - Protocol Buffer messages
//! - Push-based subscriptions

use crate::core::types::Symbol;

/// OpenD connection parameters
pub struct FutuEndpoints {
    /// OpenD host (usually 127.0.0.1 for local)
    pub host: String,
    /// OpenD port (default: 11111)
    pub port: u16,
    /// Enable encryption (required for remote OpenD)
    pub use_encryption: bool,
}

impl Default for FutuEndpoints {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 11111,
            use_encryption: false,
        }
    }
}

/// Futu OpenD operations (not REST endpoints)
///
/// These correspond to Protocol Buffer message types, not HTTP paths.
#[derive(Debug, Clone)]
pub enum FutuOperation {
    // Connection
    Connect,
    Disconnect,

    // Subscription
    Subscribe,
    Unsubscribe,
    QuerySubscription,

    // Market Data
    GetStockQuote,
    GetOrderBook,
    GetCurrentKline,
    RequestHistoryKline,
    GetRealtimeTicker,
    GetTimeFrame,
    GetBrokerQueue,

    // Market Metadata
    GetTradingDays,
    GetStockBasicInfo,
    GetMarketState,
    GetPlateList,
    GetPlateStock,

    // Trading - Account
    GetAccountList,
    UnlockTrade,
    GetFunds,
    GetAccountInfo,

    // Trading - Positions
    GetPositionList,

    // Trading - Orders
    PlaceOrder,
    ModifyOrder,
    CancelOrder,
    GetOrderList,
    GetHistoryOrderList,

    // Trading - Deals
    GetDealList,
    GetHistoryDealList,
}

impl FutuOperation {
    /// Get protocol ID (for Protocol Buffer messages)
    ///
    /// These are the actual protocol IDs used in Futu's Protocol Buffer format.
    pub fn protocol_id(&self) -> u32 {
        match self {
            Self::Subscribe => 3001,
            Self::GetStockQuote => 3004,
            Self::GetOrderBook => 3012,
            Self::RequestHistoryKline => 3103,
            Self::PlaceOrder => 2202,
            Self::GetAccountList => 2001,
            // Add more based on research
            _ => 0, // Unknown
        }
    }
}

/// Format symbol for Futu API
///
/// Futu uses "MARKET.CODE" format (e.g., "US.AAPL", "HK.00700")
pub fn format_symbol(symbol: &Symbol) -> String {
    // For stocks, Futu uses market prefix
    // This is a stub - actual implementation would need market detection
    format!("US.{}", symbol.base)
}

/// Parse symbol from Futu format
pub fn parse_symbol(futu_code: &str) -> Result<Symbol, String> {
    let parts: Vec<&str> = futu_code.split('.').collect();
    if parts.len() != 2 {
        return Err(format!("Invalid Futu code format: {}", futu_code));
    }

    Ok(Symbol {
        base: parts[1].to_string(),
        quote: "USD".to_string(), // Default, would need market-specific logic
    })
}
