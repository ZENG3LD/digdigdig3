//! Polymarket API endpoints
//!
//! Three base APIs:
//! - CLOB: `https://clob.polymarket.com` — order books, pricing, markets
//! - Gamma: `https://gamma-api.polymarket.com` — events and enhanced market metadata
//! - Data: `https://data-api.polymarket.com` — user positions and trades

// ═══════════════════════════════════════════════════════════════════════════
// BASE URLS
// ═══════════════════════════════════════════════════════════════════════════

/// Base URLs for all Polymarket APIs
pub struct PolymarketEndpoints {
    pub clob_base: &'static str,
    pub gamma_base: &'static str,
    pub data_base: &'static str,
    pub ws_clob: &'static str,
}

impl Default for PolymarketEndpoints {
    fn default() -> Self {
        Self {
            clob_base: "https://clob.polymarket.com",
            gamma_base: "https://gamma-api.polymarket.com",
            data_base: "https://data-api.polymarket.com",
            ws_clob: "wss://ws-subscriptions-clob.polymarket.com/ws/market",
        }
    }
}

/// Polymarket API endpoint enum
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolymarketEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // CLOB API - Order Book & Pricing
    // ═══════════════════════════════════════════════════════════════════════
    /// GET /markets — paginated market list
    ClobMarkets,
    /// GET /markets/{condition_id} — single market
    ClobMarket,
    /// GET /book?token_id=... — full order book
    OrderBook,
    /// GET /midpoint?token_id=... — mid price
    Midpoint,
    /// GET /price?token_id=...&side=... — best bid or ask
    Price,
    /// GET /spread?token_id=... — bid-ask spread
    Spread,
    /// GET /last-trade-price?token_id=... — last trade price
    LastTradePrice,
    /// GET /prices-history?token_id=...&interval=...&fidelity=... — price history (klines)
    PricesHistory,
    /// GET /time — server time
    Time,

    // ═══════════════════════════════════════════════════════════════════════
    // GAMMA API - Events & Enhanced Market Data
    // ═══════════════════════════════════════════════════════════════════════
    /// GET /events — list events
    GammaEvents,
    /// GET /events/{id} — single event
    GammaEvent,
    /// GET /markets — markets with enhanced metadata
    GammaMarkets,
    /// GET /markets/{id} — single market with enhanced metadata
    GammaMarket,

    // ═══════════════════════════════════════════════════════════════════════
    // AUTHENTICATED CLOB API - Orders (requires L2 auth)
    // ═══════════════════════════════════════════════════════════════════════
    /// GET /orders — list open orders
    ClobOrders,
    /// GET /orders/{id} — single order
    ClobOrder,

    // ═══════════════════════════════════════════════════════════════════════
    // DATA API - User positions
    // ═══════════════════════════════════════════════════════════════════════
    /// GET /positions?user=... — user positions
    DataPositions,
}

impl PolymarketEndpoint {
    /// Base path for this endpoint (without path parameters)
    pub fn path(&self) -> &'static str {
        match self {
            // CLOB
            Self::ClobMarkets => "/markets",
            Self::ClobMarket => "/markets",
            Self::OrderBook => "/book",
            Self::Midpoint => "/midpoint",
            Self::Price => "/price",
            Self::Spread => "/spread",
            Self::LastTradePrice => "/last-trade-price",
            Self::PricesHistory => "/prices-history",
            Self::Time => "/time",
            // Gamma
            Self::GammaEvents => "/events",
            Self::GammaEvent => "/events",
            Self::GammaMarkets => "/markets",
            Self::GammaMarket => "/markets",
            // Authenticated CLOB
            Self::ClobOrders => "/orders",
            Self::ClobOrder => "/orders",
            // Data
            Self::DataPositions => "/positions",
        }
    }

    /// Whether this endpoint uses the Gamma API base URL
    pub fn is_gamma(&self) -> bool {
        matches!(
            self,
            Self::GammaEvents | Self::GammaEvent | Self::GammaMarkets | Self::GammaMarket
        )
    }

    /// Whether this endpoint uses the Data API base URL
    pub fn is_data(&self) -> bool {
        matches!(self, Self::DataPositions)
    }

    /// Whether this endpoint requires L2 authentication
    pub fn requires_auth(&self) -> bool {
        matches!(self, Self::ClobOrders | Self::ClobOrder)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// URL CONSTRUCTION HELPERS
// ═══════════════════════════════════════════════════════════════════════════

/// Build CLOB API URL for markets list
pub fn _clob_markets() -> String {
    "https://clob.polymarket.com/markets".to_string()
}

/// Build CLOB API URL for a specific market
pub fn _clob_market(condition_id: &str) -> String {
    format!("https://clob.polymarket.com/markets/{}", condition_id)
}

/// Build CLOB API URL for order book
pub fn _clob_book(token_id: &str) -> String {
    format!("https://clob.polymarket.com/book?token_id={}", token_id)
}

/// Build CLOB API URL for midpoint price
pub fn _clob_midpoint(token_id: &str) -> String {
    format!("https://clob.polymarket.com/midpoint?token_id={}", token_id)
}

/// Build CLOB API URL for price (best bid or ask)
pub fn _clob_price(token_id: &str, side: &str) -> String {
    format!(
        "https://clob.polymarket.com/price?token_id={}&side={}",
        token_id, side
    )
}

/// Build CLOB API URL for spread
pub fn _clob_spread(token_id: &str) -> String {
    format!("https://clob.polymarket.com/spread?token_id={}", token_id)
}

/// Build CLOB API URL for last trade price
pub fn _clob_last_trade_price(token_id: &str) -> String {
    format!(
        "https://clob.polymarket.com/last-trade-price?token_id={}",
        token_id
    )
}

/// Build CLOB API URL for price history
///
/// `interval` — time grouping: "1m", "1h", "6h", "1d", "1w", "all"
/// `fidelity` — number of data points
pub fn _prices_history(token_id: &str, interval: &str, fidelity: u32) -> String {
    format!(
        "https://clob.polymarket.com/prices-history?market={}&interval={}&fidelity={}",
        token_id, interval, fidelity
    )
}

/// Build CLOB API URL for server time
pub fn _clob_time() -> String {
    "https://clob.polymarket.com/time".to_string()
}

/// Build Gamma API URL for events list
pub fn _gamma_events() -> String {
    "https://gamma-api.polymarket.com/events".to_string()
}

/// Build Gamma API URL for a specific event
pub fn _gamma_event(id: &str) -> String {
    format!("https://gamma-api.polymarket.com/events/{}", id)
}

/// Build Gamma API URL for markets list
pub fn _gamma_markets() -> String {
    "https://gamma-api.polymarket.com/markets".to_string()
}

/// Build Gamma API URL for a specific market
pub fn _gamma_market(id: &str) -> String {
    format!("https://gamma-api.polymarket.com/markets/{}", id)
}

/// Build authenticated CLOB API URL for orders list
pub fn _clob_orders() -> String {
    "https://clob.polymarket.com/orders".to_string()
}

/// Build authenticated CLOB API URL for a specific order
pub fn _clob_order(id: &str) -> String {
    format!("https://clob.polymarket.com/orders/{}", id)
}

/// Build Data API URL for positions
pub fn _data_positions(address: &str) -> String {
    format!("https://data-api.polymarket.com/positions?user={}", address)
}

/// Map V5 interval string to Polymarket interval string
///
/// Polymarket supports: "1m", "1h", "6h", "1d", "1w", "all"
/// For intervals shorter than 1h, maps to "1m".
/// For intervals between 1h-6h, maps to "1h".
pub fn map_interval(interval: &str) -> &'static str {
    match interval {
        "1m" | "3m" | "5m" | "15m" | "30m" => "1m",
        "1h" | "2h" | "4h" => "1h",
        "6h" | "8h" | "12h" => "6h",
        "1d" | "1D" => "1d",
        "1w" | "1W" => "1w",
        _ => "1d",
    }
}

/// Get fidelity (number of data points) for a given limit
pub fn get_fidelity(limit: Option<u16>) -> u32 {
    limit.unwrap_or(500).min(1000) as u32
}
