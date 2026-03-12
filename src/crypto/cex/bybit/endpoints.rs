//! # Bybit Endpoints
//!
//! URL structures and endpoint enum for Bybit V5 API.

use crate::core::types::{AccountType, Symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL endpoints for Bybit API
pub struct BybitUrls;

impl BybitUrls {
    /// Get REST API base URL
    pub fn base_url(testnet: bool) -> &'static str {
        if testnet {
            "https://api-testnet.bybit.com"
        } else {
            "https://api.bybit.com"
        }
    }

    /// Get WebSocket URL for public channels (spot)
    pub fn ws_spot_url(testnet: bool) -> &'static str {
        if testnet {
            "wss://stream-testnet.bybit.com/v5/public/spot"
        } else {
            "wss://stream.bybit.com/v5/public/spot"
        }
    }

    /// Get WebSocket URL for public channels (linear/futures)
    pub fn ws_linear_url(testnet: bool) -> &'static str {
        if testnet {
            "wss://stream-testnet.bybit.com/v5/public/linear"
        } else {
            "wss://stream.bybit.com/v5/public/linear"
        }
    }

    /// Get WebSocket URL for private channels
    pub fn ws_private_url(testnet: bool) -> &'static str {
        if testnet {
            "wss://stream-testnet.bybit.com/v5/private"
        } else {
            "wss://stream.bybit.com/v5/private"
        }
    }

    /// Get appropriate WebSocket URL based on account type
    pub fn ws_url(account_type: AccountType, testnet: bool) -> &'static str {
        match account_type {
            AccountType::Spot | AccountType::Margin => Self::ws_spot_url(testnet),
            AccountType::FuturesCross | AccountType::FuturesIsolated => Self::ws_linear_url(testnet),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bybit V5 API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BybitEndpoint {
    // === MARKET DATA ===
    Ticker,           // GET /v5/market/tickers
    Orderbook,        // GET /v5/market/orderbook
    Klines,           // GET /v5/market/kline
    Symbols,          // GET /v5/market/instruments-info
    RecentTrades,     // GET /v5/market/recent-trade
    ServerTime,       // GET /v5/market/time

    // === ACCOUNT ===
    Balance,          // GET /v5/account/wallet-balance
    AccountInfo,      // GET /v5/account/info

    // === TRADING ===
    PlaceOrder,       // POST /v5/order/create
    CancelOrder,      // POST /v5/order/cancel
    CancelAllOrders,  // POST /v5/order/cancel-all
    OrderStatus,      // GET /v5/order/realtime
    OpenOrders,       // GET /v5/order/realtime
    OrderHistory,     // GET /v5/order/history

    // === POSITIONS (FUTURES) ===
    Positions,        // GET /v5/position/list
    SetLeverage,      // POST /v5/position/set-leverage
    SetMarginMode,    // POST /v5/position/switch-isolated
    AddMargin,        // POST /v5/position/add-margin
    TpSlMode,         // POST /v5/position/set-tpsl
    FundingRate,      // GET /v5/market/funding/history

    // === FEES ===
    FeeRate,          // GET /v5/account/fee-rate
}

impl BybitEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::Ticker => "/v5/market/tickers",
            Self::Orderbook => "/v5/market/orderbook",
            Self::Klines => "/v5/market/kline",
            Self::Symbols => "/v5/market/instruments-info",
            Self::RecentTrades => "/v5/market/recent-trade",
            Self::ServerTime => "/v5/market/time",

            // Account
            Self::Balance => "/v5/account/wallet-balance",
            Self::AccountInfo => "/v5/account/info",

            // Trading
            Self::PlaceOrder => "/v5/order/create",
            Self::CancelOrder => "/v5/order/cancel",
            Self::CancelAllOrders => "/v5/order/cancel-all",
            Self::OrderStatus => "/v5/order/realtime",
            Self::OpenOrders => "/v5/order/realtime",
            Self::OrderHistory => "/v5/order/history",

            // Positions
            Self::Positions => "/v5/position/list",
            Self::SetLeverage => "/v5/position/set-leverage",
            Self::SetMarginMode => "/v5/position/switch-isolated",
            Self::AddMargin => "/v5/position/add-margin",
            Self::TpSlMode => "/v5/position/trading-stop",
            Self::FundingRate => "/v5/market/funding/history",

            // Fees
            Self::FeeRate => "/v5/account/fee-rate",
        }
    }

    /// Get HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST requests
            Self::PlaceOrder
            | Self::CancelOrder
            | Self::CancelAllOrders
            | Self::SetLeverage
            | Self::SetMarginMode
            | Self::AddMargin
            | Self::TpSlMode => "POST",

            // GET requests
            _ => "GET",
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_private(&self) -> bool {
        match self {
            // Public endpoints
            Self::Ticker
            | Self::Orderbook
            | Self::Klines
            | Self::Symbols
            | Self::RecentTrades
            | Self::ServerTime
            | Self::FundingRate => false,

            // Private endpoints
            _ => true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Bybit API
///
/// # Format
/// - Spot: `BTCUSDT` (no separator)
/// - Linear: `BTCUSDT` (same format, distinguished by `category` parameter)
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::bybit::format_symbol;
/// use connectors_v5::core::types::{Symbol, AccountType};
///
/// let symbol = Symbol::new("BTC", "USDT");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "BTCUSDT");
/// assert_eq!(format_symbol(&symbol, AccountType::FuturesCross), "BTCUSDT");
/// ```
pub fn format_symbol(symbol: &Symbol, _account_type: AccountType) -> String {
    // Bybit uses concatenated format for both spot and futures
    // Category parameter differentiates between spot/linear/inverse
    format!("{}{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
}

/// Get category parameter for account type
///
/// # Returns
/// - `"spot"` for Spot and Margin
/// - `"linear"` for Futures (USDT-margined perpetuals)
pub fn account_type_to_category(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot | AccountType::Margin => "spot",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "linear",
    }
}

/// Map kline interval to Bybit format
///
/// # Bybit Interval Format
/// - Minutes: `1`, `3`, `5`, `15`, `30`, `60`, `120`, `240`, `360`, `720`
/// - Day: `D`
/// - Week: `W`
/// - Month: `M`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::bybit::map_kline_interval;
///
/// assert_eq!(map_kline_interval("1m"), "1");
/// assert_eq!(map_kline_interval("1h"), "60");
/// assert_eq!(map_kline_interval("1d"), "D");
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1",
        "3m" => "3",
        "5m" => "5",
        "15m" => "15",
        "30m" => "30",
        "1h" => "60",
        "2h" => "120",
        "4h" => "240",
        "6h" => "360",
        "12h" => "720",
        "1d" => "D",
        "1w" => "W",
        "1M" => "M",
        _ => "60", // default to 1 hour
    }
}
