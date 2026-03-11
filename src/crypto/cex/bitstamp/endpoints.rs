//! # Bitstamp Endpoints
//!
//! URL structures and endpoint enum for Bitstamp V2 API.

use crate::core::types::{AccountType, Symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL endpoints for Bitstamp API
pub struct BitstampUrls;

impl BitstampUrls {
    /// Get REST API base URL
    pub fn base_url() -> &'static str {
        "https://www.bitstamp.net"
    }

    /// Get WebSocket URL
    pub fn ws_url() -> &'static str {
        "wss://ws.bitstamp.net"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitstamp V2 API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitstampEndpoint {
    // === MARKET DATA ===
    Ticker,           // GET /api/v2/ticker/{pair}/
    Orderbook,        // GET /api/v2/order_book/{pair}/
    Transactions,     // GET /api/v2/transactions/{pair}/
    Ohlc,             // GET /api/v2/ohlc/{pair}/
    Markets,          // GET /api/v2/markets/
    Currencies,       // GET /api/v2/currencies/

    // === ACCOUNT ===
    Balance,          // POST /api/v2/account_balances/
    AccountInfo,      // POST /api/v2/balance/ (legacy format)
    TradingFees,      // POST /api/v2/fees/trading/
    UserTransactions, // POST /api/v2/user_transactions/

    // === TRADING ===
    BuyLimit,         // POST /api/v2/buy/{pair}/
    SellLimit,        // POST /api/v2/sell/{pair}/
    BuyMarket,        // POST /api/v2/buy/market/{pair}/
    SellMarket,       // POST /api/v2/sell/market/{pair}/
    CancelOrder,      // POST /api/v2/cancel_order/
    CancelAllOrders,  // POST /api/v2/cancel_all_orders/
    OrderStatus,      // POST /api/v2/order_status/
    OpenOrders,       // POST /api/v2/open_orders/all/

    // === FUTURES/PERPETUALS ===
    OpenPositions,    // POST /api/v2/open_positions/
}

impl BitstampEndpoint {
    /// Get endpoint path (without pair parameter)
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::Ticker => "/api/v2/ticker",
            Self::Orderbook => "/api/v2/order_book",
            Self::Transactions => "/api/v2/transactions",
            Self::Ohlc => "/api/v2/ohlc",
            Self::Markets => "/api/v2/markets/",
            Self::Currencies => "/api/v2/currencies/",

            // Account
            Self::Balance => "/api/v2/account_balances/",
            Self::AccountInfo => "/api/v2/balance/",
            Self::TradingFees => "/api/v2/fees/trading/",
            Self::UserTransactions => "/api/v2/user_transactions/",

            // Trading
            Self::BuyLimit => "/api/v2/buy",
            Self::SellLimit => "/api/v2/sell",
            Self::BuyMarket => "/api/v2/buy/market",
            Self::SellMarket => "/api/v2/sell/market",
            Self::CancelOrder => "/api/v2/cancel_order/",
            Self::CancelAllOrders => "/api/v2/cancel_all_orders/",
            Self::OrderStatus => "/api/v2/order_status/",
            Self::OpenOrders => "/api/v2/open_orders/all/",

            // Futures/Perpetuals
            Self::OpenPositions => "/api/v2/open_positions/",
        }
    }

    /// Get full path with pair parameter
    pub fn path_with_pair(&self, pair: &str) -> String {
        match self {
            Self::Ticker | Self::Orderbook | Self::Transactions | Self::Ohlc => {
                format!("{}/{}/", self.path(), pair)
            }
            Self::BuyLimit | Self::SellLimit => {
                format!("{}/{}/", self.path(), pair)
            }
            Self::BuyMarket | Self::SellMarket => {
                format!("{}/{}/", self.path(), pair)
            }
            _ => self.path().to_string(),
        }
    }

    /// Get HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // GET requests
            Self::Ticker
            | Self::Orderbook
            | Self::Transactions
            | Self::Ohlc
            | Self::Markets
            | Self::Currencies => "GET",

            // POST requests
            _ => "POST",
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_private(&self) -> bool {
        match self {
            // Public endpoints
            Self::Ticker
            | Self::Orderbook
            | Self::Transactions
            | Self::Ohlc
            | Self::Markets
            | Self::Currencies => false,

            // Private endpoints
            _ => true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Bitstamp API
///
/// # Format
/// - Spot: All lowercase, no separators: `btcusd`, `etheur`, `xrpbtc`
/// - Perpetuals: Lowercase with `-perp` suffix: `btcusd-perp`, `ethusd-perp`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::bitstamp::format_symbol;
/// use connectors_v5::core::types::{Symbol, AccountType};
///
/// let symbol = Symbol::new("BTC", "USD");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "btcusd");
///
/// let symbol = Symbol::new("BTC", "USD");
/// assert_eq!(format_symbol(&symbol, AccountType::FuturesCross), "btcusd-perp");
///
/// let symbol = Symbol::new("ETH", "EUR");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "etheur");
/// ```
pub fn format_symbol(symbol: &Symbol, account_type: AccountType) -> String {
    // Bitstamp uses lowercase concatenated format
    let base_pair = format!("{}{}", symbol.base.to_lowercase(), symbol.quote.to_lowercase());

    // Add -perp suffix for perpetual futures
    match account_type {
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            format!("{}-perp", base_pair)
        }
        _ => base_pair,
    }
}

/// Map kline interval to Bitstamp format (in seconds)
///
/// # Bitstamp Interval Format (step parameter)
/// - 60, 180, 300, 900, 1800, 3600, 7200, 14400, 21600, 43200, 86400, 259200
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::bitstamp::map_kline_interval;
///
/// assert_eq!(map_kline_interval("1m"), "60");
/// assert_eq!(map_kline_interval("1h"), "3600");
/// assert_eq!(map_kline_interval("1d"), "86400");
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "60",       // 1 minute
        "3m" => "180",      // 3 minutes
        "5m" => "300",      // 5 minutes
        "15m" => "900",     // 15 minutes
        "30m" => "1800",    // 30 minutes
        "1h" => "3600",     // 1 hour
        "2h" => "7200",     // 2 hours
        "4h" => "14400",    // 4 hours
        "6h" => "21600",    // 6 hours
        "12h" => "43200",   // 12 hours
        "1d" => "86400",    // 1 day
        "3d" => "259200",   // 3 days
        _ => "3600",        // default to 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        let symbol = Symbol::new("BTC", "USD");
        assert_eq!(format_symbol(&symbol, AccountType::Spot), "btcusd");

        let symbol = Symbol::new("ETH", "EUR");
        assert_eq!(format_symbol(&symbol, AccountType::Spot), "etheur");

        let symbol = Symbol::new("xrp", "btc");
        assert_eq!(format_symbol(&symbol, AccountType::Spot), "xrpbtc");

        // Test perpetual formatting
        let symbol = Symbol::new("BTC", "USD");
        assert_eq!(format_symbol(&symbol, AccountType::FuturesCross), "btcusd-perp");

        let symbol = Symbol::new("ETH", "USD");
        assert_eq!(format_symbol(&symbol, AccountType::FuturesIsolated), "ethusd-perp");
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "60");
        assert_eq!(map_kline_interval("1h"), "3600");
        assert_eq!(map_kline_interval("1d"), "86400");
        assert_eq!(map_kline_interval("invalid"), "3600"); // default
    }

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(BitstampEndpoint::Ticker.path(), "/api/v2/ticker");
        assert_eq!(BitstampEndpoint::Balance.path(), "/api/v2/account_balances/");

        assert_eq!(
            BitstampEndpoint::Ticker.path_with_pair("btcusd"),
            "/api/v2/ticker/btcusd/"
        );
    }

    #[test]
    fn test_endpoint_methods() {
        assert_eq!(BitstampEndpoint::Ticker.method(), "GET");
        assert_eq!(BitstampEndpoint::Balance.method(), "POST");
        assert_eq!(BitstampEndpoint::BuyLimit.method(), "POST");
    }

    #[test]
    fn test_endpoint_privacy() {
        assert!(!BitstampEndpoint::Ticker.is_private());
        assert!(BitstampEndpoint::Balance.is_private());
        assert!(BitstampEndpoint::BuyLimit.is_private());
    }
}
