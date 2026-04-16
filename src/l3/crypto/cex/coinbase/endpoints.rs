//! # Coinbase Endpoints
//!
//! URL structures and endpoint enum for Coinbase Advanced Trade API.

use crate::core::types::{AccountType, Symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL endpoints for Coinbase API
pub struct CoinbaseUrls;

impl CoinbaseUrls {
    /// Get REST API base URL
    /// Note: Coinbase does not have a separate testnet for Advanced Trade API
    pub fn base_url() -> &'static str {
        "https://api.coinbase.com/api/v3/brokerage"
    }

    /// Get v2 API base URL (used for deposits/withdrawals)
    pub fn v2_url() -> &'static str {
        "https://api.coinbase.com/v2"
    }

    /// Get public market data base URL (no auth required)
    /// Public endpoints use the /market prefix under the brokerage path
    pub fn market_url() -> &'static str {
        "https://api.coinbase.com/api/v3/brokerage/market"
    }

    /// Get WebSocket URL for public channels
    pub fn ws_public_url() -> &'static str {
        "wss://advanced-trade-ws.coinbase.com"
    }

    /// Get WebSocket URL for private channels (user data)
    pub fn ws_user_url() -> &'static str {
        "wss://advanced-trade-ws-user.coinbase.com"
    }

    /// Get appropriate WebSocket URL based on whether authentication is needed
    pub fn ws_url(authenticated: bool) -> &'static str {
        if authenticated {
            Self::ws_user_url()
        } else {
            Self::ws_public_url()
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Coinbase Advanced Trade API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CoinbaseEndpoint {
    // === MARKET DATA ===
    ServerTime,       // GET /time
    Products,         // GET /products (private) or /market/products (public)
    ProductDetails,   // GET /products/{product_id}
    BestBidAsk,       // GET /best_bid_ask
    ProductBook,      // GET /product_book
    Candles,          // GET /products/{product_id}/candles
    MarketTrades,     // GET /products/{product_id}/ticker

    // === ACCOUNT ===
    Accounts,         // GET /accounts
    AccountDetails,   // GET /accounts/{account_uuid}
    TransactionSummary, // GET /transaction_summary

    // === TRADING ===
    CreateOrder,      // POST /orders
    CancelOrders,     // POST /orders/batch_cancel
    EditOrder,        // POST /orders/edit
    OrderDetails,     // GET /orders/historical/{order_id}
    ListOrders,       // GET /orders/historical/batch
    ListFills,        // GET /orders/historical/fills
    FillHistory,      // GET /orders/historical/fills (alias — paginated fill history)
    PreviewOrder,     // POST /orders/preview

    // === CUSTODIAL FUNDS (v2 API) ===
    // Note: these use the v2 base URL, not the brokerage URL.
    // The account_id must be embedded in the path by the connector.
    V2AccountDeposits,      // GET  /v2/accounts/{id}/deposits
    V2AccountTransactions,  // GET  /v2/accounts/{id}/transactions
    V2CreateAddress,        // POST /v2/accounts/{id}/addresses (generate deposit address)
    V2SendTransaction,      // POST /v2/accounts/{id}/transactions (type=send for withdrawal)
}

impl CoinbaseEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::ServerTime => "/time",
            Self::Products => "/products",
            Self::ProductDetails => "/products", // Need to append /{product_id}
            Self::BestBidAsk => "/best_bid_ask",
            Self::ProductBook => "/product_book",
            Self::Candles => "/products", // Need to append /{product_id}/candles
            Self::MarketTrades => "/products", // Need to append /{product_id}/ticker

            // Account
            Self::Accounts => "/accounts",
            Self::AccountDetails => "/accounts", // Need to append /{account_uuid}
            Self::TransactionSummary => "/transaction_summary",

            // Trading
            Self::CreateOrder => "/orders",
            Self::CancelOrders => "/orders/batch_cancel",
            Self::EditOrder => "/orders/edit",
            Self::OrderDetails => "/orders/historical", // Need to append /{order_id}
            Self::ListOrders => "/orders/historical/batch",
            Self::ListFills => "/orders/historical/fills",
            Self::FillHistory => "/orders/historical/fills",
            Self::PreviewOrder => "/orders/preview",

            // v2 paths — caller must substitute {account_id} in the path
            Self::V2AccountDeposits => "/accounts/{account_id}/deposits",
            Self::V2AccountTransactions => "/accounts/{account_id}/transactions",
            Self::V2CreateAddress => "/accounts/{account_id}/addresses",
            Self::V2SendTransaction => "/accounts/{account_id}/transactions",
        }
    }

    /// Get HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST requests
            Self::CreateOrder
            | Self::CancelOrders
            | Self::EditOrder
            | Self::PreviewOrder
            | Self::V2CreateAddress
            | Self::V2SendTransaction => "POST",

            // GET requests
            _ => "GET",
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_private(&self) -> bool {
        match self {
            // Public endpoints (can use /market prefix)
            Self::ServerTime => false,

            // All other endpoints require authentication
            _ => true,
        }
    }

    /// Check if this endpoint uses the v2 API base URL (not the brokerage URL)
    pub fn is_v2(&self) -> bool {
        matches!(
            self,
            Self::V2AccountDeposits
                | Self::V2AccountTransactions
                | Self::V2CreateAddress
                | Self::V2SendTransaction
        )
    }

    /// Check if endpoint has public alternative
    pub fn has_public_alternative(&self) -> bool {
        matches!(
            self,
            Self::Products | Self::ProductDetails | Self::ProductBook | Self::Candles | Self::MarketTrades
        )
    }

    /// Get public market path (for endpoints that support it)
    pub fn market_path(&self) -> Option<&'static str> {
        match self {
            Self::Products => Some("/products"),
            Self::ProductDetails => Some("/products"), // Append /{product_id}
            Self::ProductBook => Some("/product_book"),
            Self::Candles => Some("/products"), // Append /{product_id}/candles
            Self::MarketTrades => Some("/products"), // Append /{product_id}/ticker
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Coinbase API
///
/// # Format
/// - Spot: `BTC-USD` (hyphen separator, uppercase)
/// - Futures/Perpetuals: `BTC-PERP` (hyphen separator, uppercase, PERP suffix)
///
/// # Important Limitations
/// Perpetual futures are available via Advanced Trade API, but:
/// - Orderbook REST endpoint is SPOT ONLY
/// - Candles REST endpoint is SPOT ONLY
/// - Only product listing and ticker work for perpetuals via REST
/// - Full market data requires WebSocket or INTX API (with auth)
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::coinbase::format_symbol;
/// use connectors_v5::core::types::{Symbol, AccountType};
///
/// let symbol = Symbol::new("BTC", "USD");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "BTC-USD");
/// assert_eq!(format_symbol(&symbol, AccountType::FuturesCross), "BTC-PERP");
/// ```
pub fn format_symbol(symbol: &Symbol, account_type: AccountType) -> String {
    // Coinbase uses hyphen-separated uppercase format
    match account_type {
        AccountType::Spot => {
            // Spot: BTC-USD
            format!("{}-{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Perpetuals: BTC-PERP (ignore quote currency for perpetuals)
            format!("{}-PERP", symbol.base.to_uppercase())
        }
        _ => {
            // Default to spot format
            format!("{}-{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
        }
    }
}

/// Parse Coinbase symbol format back to base/quote
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::coinbase::parse_symbol;
///
/// let (base, quote) = parse_symbol("BTC-USD");
/// assert_eq!(base, "BTC");
/// assert_eq!(quote, "USD");
///
/// let (base, quote) = parse_symbol("BTC-PERP");
/// assert_eq!(base, "BTC");
/// assert_eq!(quote, "PERP");
/// ```
pub fn parse_symbol(product_id: &str) -> (String, String) {
    let parts: Vec<&str> = product_id.split('-').collect();
    if parts.len() == 2 {
        (parts[0].to_string(), parts[1].to_string())
    } else {
        ("".to_string(), "".to_string())
    }
}

/// Check if a product_id is a perpetual futures contract
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::coinbase::is_perpetual;
///
/// assert!(is_perpetual("BTC-PERP"));
/// assert!(is_perpetual("ETH-PERP"));
/// assert!(!is_perpetual("BTC-USD"));
/// ```
pub fn is_perpetual(product_id: &str) -> bool {
    product_id.ends_with("-PERP")
}

/// Map kline interval to Coinbase granularity enum
///
/// # Coinbase Granularity Format
/// - `ONE_MINUTE`, `FIVE_MINUTE`, `FIFTEEN_MINUTE`, `THIRTY_MINUTE`
/// - `ONE_HOUR`, `TWO_HOUR`, `SIX_HOUR`
/// - `ONE_DAY`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::coinbase::map_kline_interval;
///
/// assert_eq!(map_kline_interval("1m"), "ONE_MINUTE");
/// assert_eq!(map_kline_interval("1h"), "ONE_HOUR");
/// assert_eq!(map_kline_interval("1d"), "ONE_DAY");
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "ONE_MINUTE",
        "5m" => "FIVE_MINUTE",
        "15m" => "FIFTEEN_MINUTE",
        "30m" => "THIRTY_MINUTE",
        "1h" => "ONE_HOUR",
        "2h" => "TWO_HOUR",
        "6h" => "SIX_HOUR",
        "1d" => "ONE_DAY",
        _ => "ONE_HOUR", // default to 1 hour
    }
}

/// Convert Coinbase granularity to seconds
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::coinbase::granularity_to_seconds;
///
/// assert_eq!(granularity_to_seconds("ONE_MINUTE"), 60);
/// assert_eq!(granularity_to_seconds("ONE_HOUR"), 3600);
/// assert_eq!(granularity_to_seconds("ONE_DAY"), 86400);
/// ```
pub fn granularity_to_seconds(granularity: &str) -> u64 {
    match granularity {
        "ONE_MINUTE" => 60,
        "FIVE_MINUTE" => 300,
        "FIFTEEN_MINUTE" => 900,
        "THIRTY_MINUTE" => 1800,
        "ONE_HOUR" => 3600,
        "TWO_HOUR" => 7200,
        "SIX_HOUR" => 21600,
        "ONE_DAY" => 86400,
        _ => 3600,
    }
}
