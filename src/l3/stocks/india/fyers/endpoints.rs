//! # Fyers API Endpoints
//!
//! URL definitions and endpoint enums for Fyers Securities API v3.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Fyers API v3
#[derive(Debug, Clone)]
pub struct FyersUrls {
    pub rest_api: &'static str,
    pub rest_data: &'static str,
    pub ws_data: &'static str,
    pub ws_order: &'static str,
    pub ws_tbt: &'static str,
    pub auth_base: &'static str,
}

impl FyersUrls {
    /// Production URLs (v3)
    pub const PRODUCTION: Self = Self {
        rest_api: "https://api.fyers.in",
        rest_data: "https://api-t1.fyers.in",
        ws_data: "wss://api-t1.fyers.in/socket/v3/dataSock",
        ws_order: "wss://api-t1.fyers.in/socket/v3/orderSock",
        ws_tbt: "wss://rtsocket-api.fyers.in/versova",
        auth_base: "https://api.fyers.in",
    };

    /// Get REST base URL (API or Data)
    pub fn rest_url(&self, is_data_endpoint: bool) -> &str {
        if is_data_endpoint {
            self.rest_data
        } else {
            self.rest_api
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Fyers API v3 endpoints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FyersEndpoint {
    // === AUTHENTICATION ===
    GenerateAuthCode,
    ValidateAuthCode,
    GenerateToken,

    // === USER PROFILE & ACCOUNT ===
    Profile,
    Funds,
    Holdings,

    // === MARKET DATA (Data API - https://api-t1.fyers.in) ===
    Quotes,
    Depth,
    History,
    MarketStatus,
    SymbolMaster,

    // === TRADING ===
    PlaceOrder,
    PlaceOrderMulti,
    ModifyOrder,
    CancelOrder,
    GetOrders,
    GetOrderById,

    // === POSITIONS & TRADES ===
    Positions,
    ConvertPosition,
    Tradebook,

    // === E-DIS ===
    GenerateTpin,
    EdisTransactions,
    SubmitHoldings,
    InquireTransaction,

    // === ADDITIONAL ENDPOINTS ===
    /// Net positions (alias for Positions — GET /api/v3/positions)
    NetPosition,
    /// Basket orders (alias for PlaceOrderMulti — POST /api/v3/orders/multi)
    BasketOrders,
}

impl FyersEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Authentication
            Self::GenerateAuthCode => "/api/v3/generate-authcode",
            Self::ValidateAuthCode => "/api/v3/validate-authcode",
            Self::GenerateToken => "/api/v3/token",

            // User Profile & Account
            Self::Profile => "/api/v3/profile",
            Self::Funds => "/api/v3/funds",
            Self::Holdings => "/api/v3/holdings",

            // Market Data
            Self::Quotes => "/data/quotes",
            Self::Depth => "/data/depth/",
            Self::History => "/data/history",
            Self::MarketStatus => "/data/market-status",
            Self::SymbolMaster => "/data/symbol-master",

            // Trading
            Self::PlaceOrder => "/api/v3/orders",
            Self::PlaceOrderMulti => "/api/v3/orders/multi",
            Self::ModifyOrder => "/api/v3/orders",
            Self::CancelOrder => "/api/v3/orders",
            Self::GetOrders => "/api/v3/orders",
            Self::GetOrderById => "/api/v3/orders",

            // Positions & Trades
            Self::Positions => "/api/v3/positions",
            Self::ConvertPosition => "/api/v3/positions",
            Self::Tradebook => "/api/v3/tradebook",

            // E-DIS
            Self::GenerateTpin => "/api/v3/edis/generate-tpin",
            Self::EdisTransactions => "/api/v3/edis/transactions",
            Self::SubmitHoldings => "/api/v3/edis/submit-holdings",
            Self::InquireTransaction => "/api/v3/edis/inquire-transaction",

            // Additional Endpoints
            Self::NetPosition => "/api/v3/positions",
            Self::BasketOrders => "/api/v3/orders/multi",
        }
    }

    /// Requires authentication?
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints (no auth)
            Self::GenerateAuthCode
            | Self::ValidateAuthCode
            | Self::GenerateToken
            | Self::MarketStatus => false,

            // All other endpoints (including new ones) require auth
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::GenerateAuthCode
            | Self::ValidateAuthCode
            | Self::GenerateToken
            | Self::PlaceOrder
            | Self::PlaceOrderMulti
            | Self::BasketOrders
            | Self::GenerateTpin
            | Self::SubmitHoldings
            | Self::InquireTransaction => "POST",

            // PUT endpoints
            Self::ModifyOrder | Self::ConvertPosition => "PUT",

            // DELETE endpoints
            Self::CancelOrder => "DELETE",

            // GET endpoints (default)
            _ => "GET",
        }
    }

    /// Is this a data endpoint? (uses https://api-t1.fyers.in)
    pub fn is_data_endpoint(&self) -> bool {
        matches!(
            self,
            Self::Quotes
                | Self::Depth
                | Self::History
                | Self::MarketStatus
                | Self::SymbolMaster
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Fyers API
///
/// # Fyers Symbol Format
/// `EXCHANGE:SYMBOL-SERIES`
///
/// ## Examples
/// - Equity: `NSE:SBIN-EQ`
/// - Futures: `NSE:NIFTY24JANFUT`
/// - Options: `NSE:NIFTY2411921500CE`
/// - BSE: `BSE:SENSEX-EQ`
/// - MCX: `MCX:GOLDM24JANFUT`
/// - Currency: `NSE:USDINR24JANFUT`
///
/// ## Convention
/// - `base` = SYMBOL (e.g., "SBIN", "NIFTY24JANFUT")
/// - `quote` = EXCHANGE (e.g., "NSE", "BSE", "MCX")
/// - Default exchange: NSE
/// - Default series: EQ (equity)
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    let exchange = if quote.is_empty() {
        "NSE"
    } else {
        quote
    };

    // If symbol already contains series (e.g., "SBIN-EQ"), use as-is
    if base.contains('-') {
        format!("{}:{}", exchange, base)
    } else {
        // Default to equity series
        format!("{}:{}-EQ", exchange, base.to_uppercase())
    }
}

/// Map kline interval to Fyers resolution
///
/// # Fyers Resolution Values
/// - `1`, `2`, `3`, `5`, `10`, `15`, `30`, `45` - minutes
/// - `60`, `120`, `180`, `240` - hours (in minutes)
/// - `1D` - 1 day
/// - `1W` - 1 week
/// - `1M` - 1 month
pub fn map_kline_interval(interval: &str) -> String {
    match interval {
        "1m" => "1",
        "2m" => "2",
        "3m" => "3",
        "5m" => "5",
        "10m" => "10",
        "15m" => "15",
        "30m" => "30",
        "45m" => "45",
        "1h" => "60",
        "2h" => "120",
        "3h" => "180",
        "4h" => "240",
        "1d" => "1D",
        "1w" => "1W",
        "1M" => "1M",
        _ => "60", // default 1 hour
    }
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        // Basic equity symbol
        assert_eq!(
            format_symbol("SBIN", "NSE", AccountType::Spot),
            "NSE:SBIN-EQ"
        );

        // BSE equity
        assert_eq!(
            format_symbol("SENSEX", "BSE", AccountType::Spot),
            "BSE:SENSEX-EQ"
        );

        // Symbol with series already included
        assert_eq!(
            format_symbol("NIFTY24JANFUT", "NSE", AccountType::Spot),
            "NSE:NIFTY24JANFUT"
        );

        // Default exchange (NSE)
        assert_eq!(
            format_symbol("RELIANCE", "", AccountType::Spot),
            "NSE:RELIANCE-EQ"
        );
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "1");
        assert_eq!(map_kline_interval("5m"), "5");
        assert_eq!(map_kline_interval("1h"), "60");
        assert_eq!(map_kline_interval("1d"), "1D");
        assert_eq!(map_kline_interval("1w"), "1W");
        assert_eq!(map_kline_interval("1M"), "1M");
    }

    #[test]
    fn test_endpoint_methods() {
        assert_eq!(FyersEndpoint::PlaceOrder.method(), "POST");
        assert_eq!(FyersEndpoint::ModifyOrder.method(), "PUT");
        assert_eq!(FyersEndpoint::CancelOrder.method(), "DELETE");
        assert_eq!(FyersEndpoint::Profile.method(), "GET");
    }

    #[test]
    fn test_endpoint_auth() {
        assert!(!FyersEndpoint::MarketStatus.requires_auth());
        assert!(FyersEndpoint::Profile.requires_auth());
        assert!(FyersEndpoint::PlaceOrder.requires_auth());
    }

    #[test]
    fn test_data_endpoint() {
        assert!(FyersEndpoint::Quotes.is_data_endpoint());
        assert!(FyersEndpoint::History.is_data_endpoint());
        assert!(!FyersEndpoint::PlaceOrder.is_data_endpoint());
        assert!(!FyersEndpoint::Profile.is_data_endpoint());
    }
}
