//! # MEXC Endpoints
//!
//! URL structures and endpoint enum for MEXC Spot API.

use crate::core::types::{AccountType, Symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL endpoints for MEXC API
pub struct MexcUrls;

impl MexcUrls {
    /// Get REST API base URL for spot trading
    pub fn base_url() -> &'static str {
        "https://api.mexc.com"
    }

    /// Get REST API base URL for futures trading
    pub fn futures_base_url() -> &'static str {
        "https://contract.mexc.com"
    }

    /// Get WebSocket URL for spot trading
    ///
    /// # Note
    /// The old endpoint `wss://wbs.mexc.com/ws` was deprecated in August 2025.
    /// New endpoint supports both JSON and Protobuf formats.
    pub fn ws_url() -> &'static str {
        "wss://wbs-api.mexc.com/ws"
    }

    /// Get WebSocket URL for futures trading
    pub fn futures_ws_url() -> &'static str {
        "wss://contract.mexc.com/edge"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// MEXC API endpoints (Spot + Futures)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MexcEndpoint {
    // === SPOT MARKET DATA ===
    Ping,             // GET /api/v3/ping
    ServerTime,       // GET /api/v3/time
    ExchangeInfo,     // GET /api/v3/exchangeInfo
    Orderbook,        // GET /api/v3/depth
    RecentTrades,     // GET /api/v3/trades
    Klines,           // GET /api/v3/klines
    Ticker24hr,       // GET /api/v3/ticker/24hr
    TickerPrice,      // GET /api/v3/ticker/price
    BookTicker,       // GET /api/v3/ticker/bookTicker
    AvgPrice,         // GET /api/v3/avgPrice

    // === SPOT ACCOUNT ===
    Account,          // GET /api/v3/account
    MyTrades,         // GET /api/v3/myTrades
    TradeFee,         // GET /api/v3/tradeFee

    // === SPOT TRADING ===
    PlaceOrder,       // POST /api/v3/order
    TestOrder,        // POST /api/v3/order/test
    CancelOrder,      // DELETE /api/v3/order
    CancelAllOrders,  // DELETE /api/v3/openOrders
    QueryOrder,       // GET /api/v3/order
    OpenOrders,       // GET /api/v3/openOrders
    AllOrders,        // GET /api/v3/allOrders

    // === FUTURES MARKET DATA ===
    FuturesPing,          // GET /api/v1/contract/ping
    FuturesTicker,        // GET /api/v1/contract/ticker
    FuturesOrderbook,     // GET /api/v1/contract/depth/{symbol}
    FuturesKlines,        // GET /api/v1/contract/kline/{symbol}
    FuturesRecentTrades,  // GET /api/v1/contract/deals/{symbol}
    FuturesContractInfo,  // GET /api/v1/contract/detail
}

impl MexcEndpoint {
    /// Get endpoint path (without symbol parameter for futures)
    pub fn path(&self) -> &'static str {
        match self {
            // Spot Market Data
            Self::Ping => "/api/v3/ping",
            Self::ServerTime => "/api/v3/time",
            Self::ExchangeInfo => "/api/v3/exchangeInfo",
            Self::Orderbook => "/api/v3/depth",
            Self::RecentTrades => "/api/v3/trades",
            Self::Klines => "/api/v3/klines",
            Self::Ticker24hr => "/api/v3/ticker/24hr",
            Self::TickerPrice => "/api/v3/ticker/price",
            Self::BookTicker => "/api/v3/ticker/bookTicker",
            Self::AvgPrice => "/api/v3/avgPrice",

            // Spot Account
            Self::Account => "/api/v3/account",
            Self::MyTrades => "/api/v3/myTrades",
            Self::TradeFee => "/api/v3/tradeFee",

            // Spot Trading
            Self::PlaceOrder => "/api/v3/order",
            Self::TestOrder => "/api/v3/order/test",
            Self::CancelOrder => "/api/v3/order",
            Self::CancelAllOrders => "/api/v3/openOrders",
            Self::QueryOrder => "/api/v3/order",
            Self::OpenOrders => "/api/v3/openOrders",
            Self::AllOrders => "/api/v3/allOrders",

            // Futures Market Data
            Self::FuturesPing => "/api/v1/contract/ping",
            Self::FuturesTicker => "/api/v1/contract/ticker",
            Self::FuturesOrderbook => "/api/v1/contract/depth", // Symbol added as path param
            Self::FuturesKlines => "/api/v1/contract/kline",     // Symbol added as path param
            Self::FuturesRecentTrades => "/api/v1/contract/deals", // Symbol added as path param
            Self::FuturesContractInfo => "/api/v1/contract/detail",
        }
    }

    /// Get HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST requests
            Self::PlaceOrder
            | Self::TestOrder => "POST",

            // DELETE requests
            Self::CancelOrder
            | Self::CancelAllOrders => "DELETE",

            // GET requests (default)
            _ => "GET",
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_private(&self) -> bool {
        match self {
            // Public spot endpoints
            Self::Ping
            | Self::ServerTime
            | Self::ExchangeInfo
            | Self::Orderbook
            | Self::RecentTrades
            | Self::Klines
            | Self::Ticker24hr
            | Self::TickerPrice
            | Self::BookTicker
            | Self::AvgPrice
            // Public futures endpoints
            | Self::FuturesPing
            | Self::FuturesTicker
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesRecentTrades
            | Self::FuturesContractInfo => false,

            // Private endpoints
            _ => true,
        }
    }

    /// Check if endpoint is for futures
    pub fn is_futures(&self) -> bool {
        matches!(
            self,
            Self::FuturesPing
            | Self::FuturesTicker
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesRecentTrades
            | Self::FuturesContractInfo
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for MEXC API
///
/// # Format
/// - Spot: `BTCUSDT` (no separator)
/// - Futures: `BTC_USDT` (underscore separator)
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::mexc::format_symbol;
/// use connectors_v5::core::types::{Symbol, AccountType};
///
/// let symbol = Symbol::new("BTC", "USDT");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "BTCUSDT");
/// ```
pub fn format_symbol(symbol: &Symbol, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot: concatenated without separator
            format!("{}{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
        },
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Futures: underscore separator
            format!("{}_{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// WEBSOCKET CHANNEL NAMES (Protobuf format)
// ═══════════════════════════════════════════════════════════════════════════════

/// WebSocket channel helpers for MEXC's protobuf-based WS API.
///
/// Since August 2025, MEXC WebSocket uses protobuf encoding exclusively.
/// All channel names must include the `.pb` suffix.
pub struct MexcWsChannels;

impl MexcWsChannels {
    /// Mini ticker for a single symbol (protobuf).
    /// Example: `spot@public.miniTicker.v3.api.pb@BTCUSDT@UTC+0`
    pub fn mini_ticker(symbol: &str) -> String {
        format!("spot@public.miniTicker.v3.api.pb@{}@UTC+0", symbol)
    }

    /// Aggregated deals (trades) for a symbol (protobuf, 100ms batching).
    /// Example: `spot@public.aggre.deals.v3.api.pb@100ms@BTCUSDT`
    pub fn aggre_deals(symbol: &str) -> String {
        format!("spot@public.aggre.deals.v3.api.pb@100ms@{}", symbol)
    }

    /// Aggregated depth for a symbol (protobuf, 100ms batching).
    /// Example: `spot@public.aggre.depth.v3.api.pb@100ms@BTCUSDT`
    pub fn aggre_depth(symbol: &str) -> String {
        format!("spot@public.aggre.depth.v3.api.pb@100ms@{}", symbol)
    }

    /// Kline/candlestick for a symbol (protobuf).
    /// Example: `spot@public.kline.v3.api.pb@BTCUSDT@1m`
    pub fn kline(symbol: &str, interval: &str) -> String {
        format!("spot@public.kline.v3.api.pb@{}@{}", symbol, interval)
    }

    /// Book ticker for a single symbol (protobuf).
    /// Example: `spot@public.bookTicker.v3.api.pb@BTCUSDT`
    pub fn book_ticker(symbol: &str) -> String {
        format!("spot@public.bookTicker.v3.api.pb@{}", symbol)
    }
}

/// Map kline interval to MEXC format
///
/// # MEXC Interval Format
/// - Minutes: `1m`, `5m`, `15m`, `30m`, `60m`
/// - Hours: `4h`, `8h`
/// - Day: `1d`
/// - Week: `1w`
/// - Month: `1M`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::mexc::map_kline_interval;
///
/// assert_eq!(map_kline_interval("1m"), "1m");
/// assert_eq!(map_kline_interval("1h"), "60m");
/// assert_eq!(map_kline_interval("1d"), "1d");
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "60m",
        "4h" => "4h",
        "8h" => "8h",
        "1d" => "1d",
        "1w" => "1w",
        "1M" => "1M",
        _ => "1h", // default to 1 hour
    }
}
