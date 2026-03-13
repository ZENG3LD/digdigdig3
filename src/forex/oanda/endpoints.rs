//! # OANDA v20 API Endpoints
//!
//! URL constants and endpoint definitions for OANDA v20 REST API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL configuration for OANDA API
#[derive(Debug, Clone)]
pub struct OandaUrls {
    pub rest_url: &'static str,
    pub stream_url: &'static str,
}

impl OandaUrls {
    /// Production (Live) URLs
    pub const LIVE: Self = Self {
        rest_url: "https://api-fxtrade.oanda.com",
        stream_url: "https://stream-fxtrade.oanda.com",
    };

    /// Practice (Demo) URLs
    pub const PRACTICE: Self = Self {
        rest_url: "https://api-fxpractice.oanda.com",
        stream_url: "https://stream-fxpractice.oanda.com",
    };

    /// Get REST base URL (AccountType not used for OANDA - single REST endpoint)
    pub fn rest_url(&self, _account_type: AccountType) -> &str {
        self.rest_url
    }

    /// Get streaming base URL (AccountType not used for OANDA)
    pub fn stream_url(&self, _account_type: AccountType) -> &str {
        self.stream_url
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// OANDA v20 API endpoints
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OandaEndpoint {
    // === ACCOUNT ===
    /// List all accounts: GET /v3/accounts
    ListAccounts,
    /// Get account details: GET /v3/accounts/{accountID}
    GetAccount(String),
    /// Get account summary: GET /v3/accounts/{accountID}/summary
    GetAccountSummary(String),
    /// Get tradeable instruments: GET /v3/accounts/{accountID}/instruments
    GetInstruments(String),
    /// Poll account changes: GET /v3/accounts/{accountID}/changes
    PollAccountChanges(String),

    // === PRICING ===
    /// Get current pricing: GET /v3/accounts/{accountID}/pricing
    GetPricing(String),
    /// Stream pricing: GET /v3/accounts/{accountID}/pricing/stream
    StreamPricing(String),
    /// Get latest candles: GET /v3/accounts/{accountID}/candles/latest
    GetLatestCandles(String),

    // === INSTRUMENTS ===
    /// Get historical candles: GET /v3/instruments/{instrument}/candles
    GetCandles(String),

    // === ORDERS ===
    /// Create order: POST /v3/accounts/{accountID}/orders
    CreateOrder(String),
    /// List orders: GET /v3/accounts/{accountID}/orders
    ListOrders(String),
    /// List pending orders: GET /v3/accounts/{accountID}/pendingOrders
    ListPendingOrders(String),
    /// Get order: GET /v3/accounts/{accountID}/orders/{orderSpecifier}
    GetOrder { account_id: String, order_id: String },
    /// Cancel order: PUT /v3/accounts/{accountID}/orders/{orderSpecifier}/cancel
    CancelOrder { account_id: String, order_id: String },
    /// Amend (replace) order: PUT /v3/accounts/{accountID}/orders/{orderSpecifier}
    AmendOrder { account_id: String, order_id: String },

    // === TRADES ===
    /// List trades: GET /v3/accounts/{accountID}/trades
    ListTrades(String),
    /// List open trades: GET /v3/accounts/{accountID}/openTrades
    ListOpenTrades(String),
    /// Get trade: GET /v3/accounts/{accountID}/trades/{tradeSpecifier}
    GetTrade { account_id: String, trade_id: String },
    /// Close trade: PUT /v3/accounts/{accountID}/trades/{tradeSpecifier}/close
    CloseTrade { account_id: String, trade_id: String },

    // === POSITIONS ===
    /// List all positions: GET /v3/accounts/{accountID}/positions
    ListPositions(String),
    /// List open positions: GET /v3/accounts/{accountID}/openPositions
    ListOpenPositions(String),
    /// Get position: GET /v3/accounts/{accountID}/positions/{instrument}
    GetPosition { account_id: String, instrument: String },
    /// Close position: PUT /v3/accounts/{accountID}/positions/{instrument}/close
    ClosePosition { account_id: String, instrument: String },

    // === TRANSACTIONS ===
    /// Stream transactions: GET /v3/accounts/{accountID}/transactions/stream
    StreamTransactions(String),
}

impl OandaEndpoint {
    /// Get the endpoint path
    pub fn path(&self) -> String {
        match self {
            // Account
            Self::ListAccounts => "/v3/accounts".to_string(),
            Self::GetAccount(account_id) => format!("/v3/accounts/{}", account_id),
            Self::GetAccountSummary(account_id) => format!("/v3/accounts/{}/summary", account_id),
            Self::GetInstruments(account_id) => format!("/v3/accounts/{}/instruments", account_id),
            Self::PollAccountChanges(account_id) => format!("/v3/accounts/{}/changes", account_id),

            // Pricing
            Self::GetPricing(account_id) => format!("/v3/accounts/{}/pricing", account_id),
            Self::StreamPricing(account_id) => format!("/v3/accounts/{}/pricing/stream", account_id),
            Self::GetLatestCandles(account_id) => format!("/v3/accounts/{}/candles/latest", account_id),

            // Instruments
            Self::GetCandles(instrument) => format!("/v3/instruments/{}/candles", instrument),

            // Orders
            Self::CreateOrder(account_id) => format!("/v3/accounts/{}/orders", account_id),
            Self::ListOrders(account_id) => format!("/v3/accounts/{}/orders", account_id),
            Self::ListPendingOrders(account_id) => format!("/v3/accounts/{}/pendingOrders", account_id),
            Self::GetOrder { account_id, order_id } => format!("/v3/accounts/{}/orders/{}", account_id, order_id),
            Self::CancelOrder { account_id, order_id } => format!("/v3/accounts/{}/orders/{}/cancel", account_id, order_id),
            Self::AmendOrder { account_id, order_id } => format!("/v3/accounts/{}/orders/{}", account_id, order_id),

            // Trades
            Self::ListTrades(account_id) => format!("/v3/accounts/{}/trades", account_id),
            Self::ListOpenTrades(account_id) => format!("/v3/accounts/{}/openTrades", account_id),
            Self::GetTrade { account_id, trade_id } => format!("/v3/accounts/{}/trades/{}", account_id, trade_id),
            Self::CloseTrade { account_id, trade_id } => format!("/v3/accounts/{}/trades/{}/close", account_id, trade_id),

            // Positions
            Self::ListPositions(account_id) => format!("/v3/accounts/{}/positions", account_id),
            Self::ListOpenPositions(account_id) => format!("/v3/accounts/{}/openPositions", account_id),
            Self::GetPosition { account_id, instrument } => format!("/v3/accounts/{}/positions/{}", account_id, instrument),
            Self::ClosePosition { account_id, instrument } => format!("/v3/accounts/{}/positions/{}/close", account_id, instrument),

            // Transactions
            Self::StreamTransactions(account_id) => format!("/v3/accounts/{}/transactions/stream", account_id),
        }
    }

    /// Check if endpoint requires authentication
    pub fn requires_auth(&self) -> bool {
        // All OANDA endpoints require Bearer token authentication
        true
    }

    /// Get HTTP method for this endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::CreateOrder(_) => "POST",

            // PUT endpoints
            Self::CancelOrder { .. }
            | Self::AmendOrder { .. }
            | Self::CloseTrade { .. }
            | Self::ClosePosition { .. } => "PUT",

            // GET endpoints (default)
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for OANDA API
///
/// OANDA uses underscore format: BASE_QUOTE
///
/// # Examples
/// - EUR/USD → EUR_USD
/// - GBP/JPY → GBP_JPY
/// - XAU/USD → XAU_USD (Gold)
/// - BTC/USD → BTC_USD (Bitcoin CFD, if available)
pub fn format_symbol(base: &str, quote: &str) -> String {
    format!("{}_{}", base.to_uppercase(), quote.to_uppercase())
}

/// Parse symbol from OANDA format (EUR_USD → Symbol)
pub fn parse_symbol(s: &str) -> Option<(String, String)> {
    if let Some((base, quote)) = s.split_once('_') {
        Some((base.to_string(), quote.to_string()))
    } else {
        None
    }
}

/// Map kline interval to OANDA granularity
///
/// OANDA granularities:
/// - S5, S10, S15, S30 (seconds)
/// - M1, M2, M4, M5, M10, M15, M30 (minutes)
/// - H1, H2, H3, H4, H6, H8, H12 (hours)
/// - D (daily)
/// - W (weekly)
/// - M (monthly)
pub fn map_granularity(interval: &str) -> &'static str {
    match interval {
        "5s" => "S5",
        "10s" => "S10",
        "15s" => "S15",
        "30s" => "S30",
        "1m" => "M1",
        "2m" => "M2",
        "4m" => "M4",
        "5m" => "M5",
        "10m" => "M10",
        "15m" => "M15",
        "30m" => "M30",
        "1h" => "H1",
        "2h" => "H2",
        "3h" => "H3",
        "4h" => "H4",
        "6h" => "H6",
        "8h" => "H8",
        "12h" => "H12",
        "1d" => "D",
        "1w" => "W",
        "1M" => "M",
        _ => "H1", // default to 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        assert_eq!(format_symbol("EUR", "USD"), "EUR_USD");
        assert_eq!(format_symbol("eur", "usd"), "EUR_USD");
        assert_eq!(format_symbol("GBP", "JPY"), "GBP_JPY");
        assert_eq!(format_symbol("XAU", "USD"), "XAU_USD");
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(parse_symbol("EUR_USD"), Some(("EUR".to_string(), "USD".to_string())));
        assert_eq!(parse_symbol("GBP_JPY"), Some(("GBP".to_string(), "JPY".to_string())));
        assert_eq!(parse_symbol("INVALID"), None);
    }

    #[test]
    fn test_map_granularity() {
        assert_eq!(map_granularity("1m"), "M1");
        assert_eq!(map_granularity("1h"), "H1");
        assert_eq!(map_granularity("1d"), "D");
        assert_eq!(map_granularity("1w"), "W");
        assert_eq!(map_granularity("invalid"), "H1");
    }

    #[test]
    fn test_endpoint_path() {
        let endpoint = OandaEndpoint::GetAccount("001-011-5838423-001".to_string());
        assert_eq!(endpoint.path(), "/v3/accounts/001-011-5838423-001");

        let endpoint = OandaEndpoint::GetCandles("EUR_USD".to_string());
        assert_eq!(endpoint.path(), "/v3/instruments/EUR_USD/candles");
    }

    #[test]
    fn test_endpoint_method() {
        assert_eq!(OandaEndpoint::ListAccounts.method(), "GET");
        assert_eq!(OandaEndpoint::CreateOrder("123".to_string()).method(), "POST");
        assert_eq!(OandaEndpoint::CancelOrder { account_id: "123".to_string(), order_id: "456".to_string() }.method(), "PUT");
    }
}
