//! # Dhan Endpoints
//!
//! URLs and endpoint enum for Dhan API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Dhan API
#[derive(Debug, Clone)]
pub struct DhanUrls {
    pub rest: &'static str,
    pub ws_live_feed: &'static str,
    pub ws_depth_20: &'static str,
    pub ws_depth_200: &'static str,
}

impl DhanUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest: "https://api.dhan.co",
        ws_live_feed: "wss://api-feed.dhan.co",
        ws_depth_20: "wss://depth-api-feed.dhan.co/twentydepth",
        ws_depth_200: "wss://full-depth-api.dhan.co/twohundreddepth",
    };

    /// Sandbox URLs (same as production, differentiated by account type)
    pub const TESTNET: Self = Self {
        rest: "https://api.dhan.co",
        ws_live_feed: "wss://api-feed.dhan.co",
        ws_depth_20: "wss://depth-api-feed.dhan.co/twentydepth",
        ws_depth_200: "wss://full-depth-api.dhan.co/twohundreddepth",
    };

    /// Get REST base URL (always same for Dhan)
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Get WebSocket URL for live feed
    pub fn ws_url(&self) -> &str {
        self.ws_live_feed
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Dhan API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DhanEndpoint {
    // === AUTHENTICATION ===
    GenerateToken,
    RenewToken,

    // === MARKET DATA ===
    LTP,
    OHLC,
    Quote,
    HistoricalDaily,
    HistoricalIntraday,
    OptionChain,
    InstrumentList,

    // === TRADING - ORDERS ===
    PlaceOrder,
    ModifyOrder,
    CancelOrder,
    GetOrderBook,
    GetOrder,
    PlaceSlicedOrder,

    // === TRADING - SUPER ORDERS ===
    PlaceSuperOrder,
    ModifySuperOrder,
    CancelSuperOrder,
    GetSuperOrders,
    GetSuperOrder,

    // === TRADING - FOREVER ORDERS ===
    PlaceForeverOrder,
    ModifyForeverOrder,
    CancelForeverOrder,
    GetForeverOrders,

    // === TRADING - TRADE HISTORY ===
    GetTradesByOrder,
    GetTradeHistory,

    // === PORTFOLIO ===
    GetHoldings,
    GetPositions,
    ConvertPosition,

    // === FUNDS ===
    GetFunds,
    GetLedger,

    // === EDIS ===
    GenerateTPIN,
    GetEDISForm,
    CheckEDISStatus,
}

impl DhanEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Authentication
            Self::GenerateToken => "/v2/access_token",
            Self::RenewToken => "/v2/access_token/renew",

            // Market Data
            Self::LTP => "/v2/marketfeed/ltp",
            Self::OHLC => "/v2/marketfeed/ohlc",
            Self::Quote => "/v2/marketfeed/quote",
            Self::HistoricalDaily => "/v2/charts/historical",
            Self::HistoricalIntraday => "/v2/charts/intraday",
            Self::OptionChain => "/v2/optionchain",
            Self::InstrumentList => "/v2/instrument/{exchangeSegment}",

            // Trading - Orders
            Self::PlaceOrder => "/v2/orders",
            Self::ModifyOrder => "/v2/orders/{orderId}",
            Self::CancelOrder => "/v2/orders/{orderId}",
            Self::GetOrderBook => "/v2/orders",
            Self::GetOrder => "/v2/orders/{orderId}",
            Self::PlaceSlicedOrder => "/v2/orders/slicing",

            // Trading - Super Orders
            Self::PlaceSuperOrder => "/v2/super/orders",
            Self::ModifySuperOrder => "/v2/super/orders/{orderId}",
            Self::CancelSuperOrder => "/v2/super/orders/{orderId}/{orderLeg}",
            Self::GetSuperOrders => "/v2/super/orders",
            Self::GetSuperOrder => "/v2/super/orders/{orderId}",

            // Trading - Forever Orders
            Self::PlaceForeverOrder => "/v2/forever/orders",
            Self::ModifyForeverOrder => "/v2/forever/orders/{orderId}",
            Self::CancelForeverOrder => "/v2/forever/orders/{orderId}",
            Self::GetForeverOrders => "/v2/forever/orders",

            // Trading - Trade History
            Self::GetTradesByOrder => "/v2/trades/{orderId}",
            Self::GetTradeHistory => "/v2/trades/{fromDate}/{toDate}/{page}",

            // Portfolio
            Self::GetHoldings => "/v2/holdings",
            Self::GetPositions => "/v2/positions",
            Self::ConvertPosition => "/v2/positions/convert",

            // Funds
            Self::GetFunds => "/v2/funds",
            Self::GetLedger => "/v2/ledger",

            // EDIS
            Self::GenerateTPIN => "/v2/edis/tpin",
            Self::GetEDISForm => "/v2/edis/form",
            Self::CheckEDISStatus => "/v2/edis/inquiry",
        }
    }

    /// Does endpoint require authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints (no auth required)
            Self::InstrumentList => false,

            // All other endpoints require auth
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::GenerateToken
            | Self::RenewToken
            | Self::LTP
            | Self::OHLC
            | Self::Quote
            | Self::HistoricalDaily
            | Self::HistoricalIntraday
            | Self::OptionChain
            | Self::PlaceOrder
            | Self::PlaceSlicedOrder
            | Self::PlaceSuperOrder
            | Self::PlaceForeverOrder
            | Self::ConvertPosition
            | Self::GenerateTPIN
            | Self::GetEDISForm
            | Self::CheckEDISStatus => "POST",

            // PUT endpoints
            Self::ModifyOrder | Self::ModifySuperOrder | Self::ModifyForeverOrder => "PUT",

            // DELETE endpoints
            Self::CancelOrder | Self::CancelSuperOrder | Self::CancelForeverOrder => "DELETE",

            // GET endpoints (everything else)
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE SEGMENT MAPPING
// ═══════════════════════════════════════════════════════════════════════════════

/// Dhan exchange segments
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum DhanExchangeSegment {
    /// NSE Equity (Cash Market)
    NseEq = 0,
    /// NSE Futures & Options
    NseFno = 1,
    /// BSE Equity
    BseEq = 2,
    /// MCX Commodities
    McxComm = 3,
}

impl DhanExchangeSegment {
    /// Convert to string for API requests
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::NseEq => "NSE_EQ",
            Self::NseFno => "NSE_FNO",
            Self::BseEq => "BSE_EQ",
            Self::McxComm => "MCX_COMM",
        }
    }

    /// Convert to integer for WebSocket
    pub fn as_int(&self) -> u8 {
        *self as u8
    }

    /// Parse from string
    pub fn _from_str(s: &str) -> Option<Self> {
        match s {
            "NSE_EQ" => Some(Self::NseEq),
            "NSE_FNO" => Some(Self::NseFno),
            "BSE_EQ" => Some(Self::BseEq),
            "MCX_COMM" => Some(Self::McxComm),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Dhan API
///
/// Dhan uses Security ID (numeric string) rather than symbols.
/// Symbols are used only for display purposes.
///
/// # Note
/// You must lookup Security ID from instrument list CSV.
pub fn _format_symbol(trading_symbol: &str, segment: DhanExchangeSegment) -> String {
    // Dhan uses trading symbols as-is, but requires security ID for orders
    // This is just for display/logging
    format!("{} ({})", trading_symbol, segment.as_str())
}

/// Map kline interval to Dhan format
///
/// # Dhan Intervals
/// - Intraday: "1", "5", "15", "25", "60" (minutes)
/// - Daily: Not applicable (use historical endpoint)
pub fn map_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1",
        "5m" => "5",
        "15m" => "15",
        "25m" => "25",
        "1h" | "60m" => "60",
        _ => "60", // default to 1 hour
    }
}

/// Map product type to Dhan format
pub fn map_product_type(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "CNC",           // Cash and Carry (Delivery)
        AccountType::Margin => "INTRADAY",    // Intraday (MIS)
        AccountType::FuturesCross => "MARGIN", // Margin (NRML)
        AccountType::FuturesIsolated => "MARGIN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(DhanEndpoint::PlaceOrder.path(), "/v2/orders");
        assert_eq!(DhanEndpoint::GetHoldings.path(), "/v2/holdings");
        assert_eq!(DhanEndpoint::LTP.path(), "/v2/marketfeed/ltp");
    }

    #[test]
    fn test_endpoint_methods() {
        assert_eq!(DhanEndpoint::PlaceOrder.method(), "POST");
        assert_eq!(DhanEndpoint::ModifyOrder.method(), "PUT");
        assert_eq!(DhanEndpoint::CancelOrder.method(), "DELETE");
        assert_eq!(DhanEndpoint::GetOrderBook.method(), "GET");
    }

    #[test]
    fn test_exchange_segment() {
        assert_eq!(DhanExchangeSegment::NseEq.as_str(), "NSE_EQ");
        assert_eq!(DhanExchangeSegment::NseEq.as_int(), 0);
        assert_eq!(
            DhanExchangeSegment::_from_str("NSE_EQ"),
            Some(DhanExchangeSegment::NseEq)
        );
    }

    #[test]
    fn test_interval_mapping() {
        assert_eq!(map_interval("1m"), "1");
        assert_eq!(map_interval("5m"), "5");
        assert_eq!(map_interval("1h"), "60");
    }
}
