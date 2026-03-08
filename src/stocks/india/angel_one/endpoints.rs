//! # Angel One SmartAPI Endpoints
//!
//! URL definitions and endpoint enum for Angel One API.

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Angel One SmartAPI
#[derive(Debug, Clone)]
pub struct AngelOneUrls {
    pub rest_base: &'static str,
    pub ws_base: &'static str,
}

impl AngelOneUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest_base: "https://apiconnect.angelone.in",
        ws_base: "wss://smartapisocket.angelone.in/smart-stream",
    };

    /// Angel One does not provide testnet/sandbox
    pub const TESTNET: Self = Self {
        rest_base: "https://apiconnect.angelone.in", // Same as production
        ws_base: "wss://smartapisocket.angelone.in/smart-stream",
    };

    /// Get appropriate URL set based on testnet flag
    pub fn get(testnet: bool) -> Self {
        if testnet {
            Self::TESTNET
        } else {
            Self::MAINNET
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Angel One SmartAPI endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AngelOneEndpoint {
    // === SESSION MANAGEMENT ===
    Login,
    TokenRefresh,
    GetProfile,
    Logout,
    GetFeedToken,

    // === MARKET DATA ===
    Quote,              // LTP, OHLC, or FULL mode
    HistoricalCandles,  // Historical OHLC data
    SearchScrip,        // Search symbol by name

    // === TRADING ===
    PlaceOrder,
    PlaceOrderFullResponse,
    ModifyOrder,
    CancelOrder,
    GetOrderBook,
    GetOrderDetails,
    GetTradeBook,

    // === GTT (GOOD TILL TRIGGERED) ===
    CreateGTT,
    ModifyGTT,
    CancelGTT,
    GetGTTDetails,
    ListGTT,

    // === PORTFOLIO ===
    GetHoldings,
    GetPositions,
    ConvertPosition,

    // === ACCOUNT & FUNDS ===
    GetRMS,

    // === MARGIN CALCULATOR ===
    MarginCalculator,
}

impl AngelOneEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Session Management
            Self::Login => "/rest/auth/angelbroking/user/v1/loginByPassword",
            Self::TokenRefresh => "/rest/auth/angelbroking/jwt/v1/generateTokens",
            Self::GetProfile => "/rest/secure/angelbroking/user/v1/getProfile",
            Self::Logout => "/rest/secure/angelbroking/user/v1/logout",
            Self::GetFeedToken => "/rest/secure/angelbroking/user/v1/getfeedToken",

            // Market Data
            Self::Quote => "/rest/secure/angelbroking/market/v1/quote/",
            Self::HistoricalCandles => "/rest/secure/angelbroking/historical/v1/getCandleData",
            Self::SearchScrip => "/rest/secure/angelbroking/order/v1/searchScrip",

            // Trading
            Self::PlaceOrder => "/rest/secure/angelbroking/order/v1/placeOrder",
            Self::PlaceOrderFullResponse => "/rest/secure/angelbroking/order/v1/placeOrderFullResponse",
            Self::ModifyOrder => "/rest/secure/angelbroking/order/v1/modifyOrder",
            Self::CancelOrder => "/rest/secure/angelbroking/order/v1/cancelOrder",
            Self::GetOrderBook => "/rest/secure/angelbroking/order/v1/getOrderBook",
            Self::GetOrderDetails => "/rest/secure/angelbroking/order/v1/details/",
            Self::GetTradeBook => "/rest/secure/angelbroking/order/v1/getTradeBook",

            // GTT
            Self::CreateGTT => "/rest/secure/angelbroking/gtt/v1/createRule",
            Self::ModifyGTT => "/rest/secure/angelbroking/gtt/v1/modifyRule",
            Self::CancelGTT => "/rest/secure/angelbroking/gtt/v1/cancelRule",
            Self::GetGTTDetails => "/rest/secure/angelbroking/gtt/v1/ruleDetails",
            Self::ListGTT => "/rest/secure/angelbroking/gtt/v1/ruleList",

            // Portfolio
            Self::GetHoldings => "/rest/secure/angelbroking/portfolio/v1/getHolding",
            Self::GetPositions => "/rest/secure/angelbroking/portfolio/v1/getPosition",
            Self::ConvertPosition => "/rest/secure/angelbroking/portfolio/v1/convertPosition",

            // Account & Funds
            Self::GetRMS => "/rest/secure/angelbroking/user/v1/getRMS",

            // Margin Calculator
            Self::MarginCalculator => "/rest/secure/angelbroking/margin/v1/batch",
        }
    }

    /// HTTP method for the endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::Login
            | Self::TokenRefresh
            | Self::Logout
            | Self::Quote
            | Self::HistoricalCandles
            | Self::SearchScrip
            | Self::PlaceOrder
            | Self::PlaceOrderFullResponse
            | Self::ModifyOrder
            | Self::CancelOrder
            | Self::CreateGTT
            | Self::ModifyGTT
            | Self::CancelGTT
            | Self::GetGTTDetails
            | Self::ListGTT
            | Self::MarginCalculator => "POST",

            // GET endpoints
            _ => "GET",
        }
    }

    /// Does the endpoint require authentication?
    pub fn requires_auth(&self) -> bool {
        match self {
            Self::Login => false, // Login endpoint doesn't need auth (it creates auth)
            _ => true,             // All other endpoints require JWT token
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Angel One API
///
/// Angel One uses trading symbols in format: `{SYMBOL}-{SEGMENT}`
///
/// # Examples
/// - Equity: `SBIN-EQ` (State Bank of India - Equity)
/// - Futures: `NIFTY26JAN24000CE` (Nifty Call Option)
/// - Currency: `USDINR-EQ`
///
/// # Notes
/// - For most use cases, the base symbol is sufficient (e.g., "SBIN" for SBI equity)
/// - Symbol tokens are required for API calls (obtained from instrument master)
/// - This function returns a simplified format for display/search purposes
pub fn format_symbol(symbol: &crate::core::types::Symbol) -> String {
    // For Indian equity stocks, typically just the base symbol is used
    // The exchange suffix (EQ, BE, etc.) is often added by the API
    symbol.base.to_uppercase()
}

/// Map interval to Angel One historical data interval format
///
/// Angel One supports these intervals:
/// - ONE_MINUTE, THREE_MINUTE, FIVE_MINUTE, TEN_MINUTE
/// - FIFTEEN_MINUTE, THIRTY_MINUTE
/// - ONE_HOUR
/// - ONE_DAY
pub fn map_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "ONE_MINUTE",
        "3m" => "THREE_MINUTE",
        "5m" => "FIVE_MINUTE",
        "10m" => "TEN_MINUTE",
        "15m" => "FIFTEEN_MINUTE",
        "30m" => "THIRTY_MINUTE",
        "1h" => "ONE_HOUR",
        "1d" | "1D" => "ONE_DAY",
        _ => "ONE_HOUR", // default
    }
}

/// Exchange type codes for WebSocket subscription
///
/// Used in WebSocket V2 subscription messages
#[allow(dead_code)]
pub mod exchange_type {
    pub const NSE: u8 = 1;
    pub const NFO: u8 = 2;
    pub const BSE: u8 = 3;
    pub const BFO: u8 = 4;
    pub const MCX: u8 = 5;
    pub const CDS: u8 = 7;
    pub const NCDEX: u8 = 13;
}

/// WebSocket subscription modes
#[allow(dead_code)]
pub mod ws_mode {
    pub const LTP: u8 = 1;         // Last Traded Price only
    pub const QUOTE: u8 = 2;       // OHLC + Volume
    pub const SNAP_QUOTE: u8 = 3;  // Full market depth snapshot
    pub const DEPTH_20: u8 = 4;    // 20-level order book (unique feature)
}
