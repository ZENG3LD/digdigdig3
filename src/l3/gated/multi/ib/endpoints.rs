//! # Interactive Brokers API Endpoints
//!
//! URL constants, endpoint enum, and helper functions for IB Client Portal Web API.

use crate::core::types::Symbol;

/// Base URLs for IB Client Portal Web API
pub struct IBEndpoints {
    /// REST API base URL
    pub rest_base: String,
    /// WebSocket base URL (optional)
    pub _ws_base: Option<String>,
}

impl IBEndpoints {
    /// Create endpoints for Gateway (local)
    pub fn gateway() -> Self {
        Self {
            rest_base: "https://localhost:5000/v1/api".to_string(),
            _ws_base: Some("wss://localhost:5000/v1/api/ws".to_string()),
        }
    }

    /// Create endpoints for production OAuth
    #[allow(dead_code)]
    pub fn production() -> Self {
        Self {
            rest_base: "https://api.ibkr.com/v1/api".to_string(),
            _ws_base: Some("wss://api.ibkr.com/v1/api/ws".to_string()),
        }
    }

    /// Create custom endpoints
    pub fn custom(rest_base: impl Into<String>, ws_base: Option<impl Into<String>>) -> Self {
        Self {
            rest_base: rest_base.into(),
            _ws_base: ws_base.map(|s| s.into()),
        }
    }
}

impl Default for IBEndpoints {
    fn default() -> Self {
        Self::gateway()
    }
}

/// API endpoint paths
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum IBEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // Authentication & Session Management
    // ═══════════════════════════════════════════════════════════════════════
    /// Check authentication status
    AuthStatus,
    /// Initialize brokerage session
    AuthInit,
    /// Validate SSO session
    SsoValidate,
    /// Keep session alive (tickle)
    Tickle,
    /// Logout
    Logout,

    // ═══════════════════════════════════════════════════════════════════════
    // Portfolio & Account
    // ═══════════════════════════════════════════════════════════════════════
    /// Get portfolio accounts
    PortfolioAccounts,
    /// Get sub-accounts
    PortfolioSubAccounts,
    /// Get portfolio positions for account
    PortfolioPositions { account_id: String, page: u32 },
    /// Get position details
    PortfolioPosition { account_id: String, conid: i64 },
    /// Get account summary
    PortfolioSummary { account_id: String },
    /// Get account ledger
    PortfolioLedger { account_id: String },
    /// Get allocation data
    PortfolioAllocation { account_id: String },
    /// Get partitioned P&L
    PnlPartitioned,

    // ═══════════════════════════════════════════════════════════════════════
    // Contract & Symbol Search
    // ═══════════════════════════════════════════════════════════════════════
    /// Search contracts by symbol
    ContractSearch,
    /// Get contract details
    ContractInfo { conid: i64 },
    /// Get contract info with trading rules
    ContractInfoAndRules { conid: i64 },
    /// Get trading rules for contract
    ContractRules { conid: i64 },
    /// Get algorithm parameters
    ContractAlgos { conid: i64 },
    /// Security definition info
    SecdefInfo,

    // ═══════════════════════════════════════════════════════════════════════
    // Market Data
    // ═══════════════════════════════════════════════════════════════════════
    /// Get market data snapshot
    MarketDataSnapshot,
    /// Get historical market data
    MarketDataHistory,
    /// Unsubscribe market data for single contract
    MarketDataUnsubscribe { conid: i64 },
    /// Unsubscribe all market data
    MarketDataUnsubscribeAll,

    // ═══════════════════════════════════════════════════════════════════════
    // Trading & Orders
    // ═══════════════════════════════════════════════════════════════════════
    /// Place order
    PlaceOrder { account_id: String },
    /// Confirm order
    ConfirmOrder { reply_id: String },
    /// Get live orders
    LiveOrders,
    /// Get trades (executions)
    Trades,
    /// Modify order
    ModifyOrder { account_id: String, order_id: String },
    /// Cancel order
    CancelOrder { account_id: String, order_id: String },
    /// What-if order (preview)
    WhatIfOrder { account_id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // Market Scanner
    // ═══════════════════════════════════════════════════════════════════════
    /// Get scanner parameters
    ScannerParams,
    /// Run market scanner
    ScannerRun,

    // ═══════════════════════════════════════════════════════════════════════
    // Alerts & Notifications
    // ═══════════════════════════════════════════════════════════════════════
    /// Create alert
    CreateAlert { account_id: String },
    /// Get alerts
    GetAlerts { account_id: String },
    /// Delete alert
    DeleteAlert { order_id: String },
    /// Get unread notifications count
    NotificationsUnreadCount,
    /// Get notifications
    GetNotifications,
    /// Mark notification as read
    MarkNotificationRead { notification_id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // Watchlists
    // ═══════════════════════════════════════════════════════════════════════
    /// Create watchlist
    CreateWatchlist,
    /// Get all watchlists
    GetWatchlists,
    /// Get watchlist details
    GetWatchlist { watchlist_id: String },
    /// Delete watchlist
    DeleteWatchlist { watchlist_id: String },

    // ═══════════════════════════════════════════════════════════════════════
    // Portfolio Analytics
    // ═══════════════════════════════════════════════════════════════════════
    /// Get performance metrics
    PerformanceMetrics,
    /// Get performance summary
    PerformanceSummary,
    /// Get transaction history
    TransactionHistory,

    // ═══════════════════════════════════════════════════════════════════════
    // Flex Web Service
    // ═══════════════════════════════════════════════════════════════════════
    /// Generate flex report
    FlexGenerate,
    /// Check flex report status
    FlexStatus { request_id: String },
}

impl IBEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> String {
        match self {
            // Authentication & Session
            Self::AuthStatus => "/iserver/auth/status".to_string(),
            Self::AuthInit => "/iserver/auth/ssodh/init".to_string(),
            Self::SsoValidate => "/sso/validate".to_string(),
            Self::Tickle => "/tickle".to_string(),
            Self::Logout => "/logout".to_string(),

            // Portfolio & Account
            Self::PortfolioAccounts => "/portfolio/accounts".to_string(),
            Self::PortfolioSubAccounts => "/portfolio/subaccounts".to_string(),
            Self::PortfolioPositions { account_id, page } => {
                format!("/portfolio/{}/positions/{}", account_id, page)
            }
            Self::PortfolioPosition { account_id, conid } => {
                format!("/portfolio/{}/position/{}", account_id, conid)
            }
            Self::PortfolioSummary { account_id } => {
                format!("/portfolio/{}/summary", account_id)
            }
            Self::PortfolioLedger { account_id } => {
                format!("/portfolio/{}/ledger", account_id)
            }
            Self::PortfolioAllocation { account_id } => {
                format!("/portfolio/{}/allocation", account_id)
            }
            Self::PnlPartitioned => "/iserver/account/pnl/partitioned".to_string(),

            // Contract & Symbol Search
            Self::ContractSearch => "/iserver/secdef/search".to_string(),
            Self::ContractInfo { conid } => format!("/iserver/contract/{}/info", conid),
            Self::ContractInfoAndRules { conid } => {
                format!("/iserver/contract/{}/info-and-rules", conid)
            }
            Self::ContractRules { conid } => format!("/iserver/contract/{}/rules", conid),
            Self::ContractAlgos { conid } => format!("/iserver/contract/{}/algos", conid),
            Self::SecdefInfo => "/iserver/secdef/info".to_string(),

            // Market Data
            Self::MarketDataSnapshot => "/iserver/marketdata/snapshot".to_string(),
            Self::MarketDataHistory => "/iserver/marketdata/history".to_string(),
            Self::MarketDataUnsubscribe { conid } => {
                format!("/iserver/marketdata/{}/unsubscribe", conid)
            }
            Self::MarketDataUnsubscribeAll => "/iserver/marketdata/unsubscribe".to_string(),

            // Trading & Orders
            Self::PlaceOrder { account_id } => {
                format!("/iserver/account/{}/orders", account_id)
            }
            Self::ConfirmOrder { reply_id } => format!("/iserver/reply/{}", reply_id),
            Self::LiveOrders => "/iserver/account/orders".to_string(),
            Self::Trades => "/iserver/account/trades".to_string(),
            Self::ModifyOrder { account_id, order_id } => {
                format!("/iserver/account/{}/order/{}", account_id, order_id)
            }
            Self::CancelOrder { account_id, order_id } => {
                format!("/iserver/account/{}/order/{}", account_id, order_id)
            }
            Self::WhatIfOrder { account_id } => {
                format!("/iserver/account/{}/whatiforder", account_id)
            }

            // Market Scanner
            Self::ScannerParams => "/iserver/scanner/params".to_string(),
            Self::ScannerRun => "/iserver/scanner/run".to_string(),

            // Alerts & Notifications
            Self::CreateAlert { account_id } => {
                format!("/iserver/account/{}/alert", account_id)
            }
            Self::GetAlerts { account_id } => {
                format!("/iserver/account/{}/alerts", account_id)
            }
            Self::DeleteAlert { order_id } => format!("/iserver/account/alert/{}", order_id),
            Self::NotificationsUnreadCount => "/fyi/unreadnumber".to_string(),
            Self::GetNotifications => "/fyi/notifications".to_string(),
            Self::MarkNotificationRead { notification_id } => {
                format!("/fyi/notification/{}", notification_id)
            }

            // Watchlists
            Self::CreateWatchlist => "/iserver/watchlists".to_string(),
            Self::GetWatchlists => "/iserver/watchlists".to_string(),
            Self::GetWatchlist { watchlist_id } => {
                format!("/iserver/watchlists/{}", watchlist_id)
            }
            Self::DeleteWatchlist { watchlist_id } => {
                format!("/iserver/watchlists/{}", watchlist_id)
            }

            // Portfolio Analytics
            Self::PerformanceMetrics => "/pa/performance".to_string(),
            Self::PerformanceSummary => "/pa/summary".to_string(),
            Self::TransactionHistory => "/pa/transactions".to_string(),

            // Flex Web Service
            Self::FlexGenerate => "/pa/flex/generate".to_string(),
            Self::FlexStatus { request_id } => format!("/pa/flex/status/{}", request_id),
        }
    }
}

/// Format symbol for IB API
///
/// IB doesn't use symbols directly - it uses Contract IDs (conid).
/// This function is for display/logging purposes only.
/// Actual trading requires resolving symbol to conid via contract search.
pub fn _format_symbol(symbol: &Symbol) -> String {
    // For stocks: just the base (ticker)
    // For forex: base/quote format
    // This is mainly for display - actual API uses conid
    if symbol.quote == "USD" || symbol.quote == "EUR" || symbol.quote == "GBP" {
        // Likely a stock or forex pair
        if symbol.base.len() <= 5 {
            // Stock ticker
            symbol.base.to_uppercase()
        } else {
            // Forex pair
            format!("{}.{}", symbol.base, symbol.quote)
        }
    } else {
        // Generic format
        format!("{}/{}", symbol.base, symbol.quote)
    }
}

/// Parse symbol from IB API format
///
/// Note: IB primarily uses conid, not symbols.
/// This is for parsing symbol strings when available.
pub fn _parse_symbol(api_symbol: &str) -> Option<Symbol> {
    // Try different separators
    if let Some((base, quote)) = api_symbol.split_once('/') {
        return Some(Symbol::new(base, quote));
    }
    if let Some((base, quote)) = api_symbol.split_once('.') {
        return Some(Symbol::new(base, quote));
    }

    // If no separator, assume stock with USD quote
    if !api_symbol.is_empty() {
        return Some(Symbol::new(api_symbol, "USD"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gateway_endpoints() {
        let endpoints = IBEndpoints::gateway();
        assert_eq!(endpoints.rest_base, "https://localhost:5000/v1/api");
        assert_eq!(
            endpoints._ws_base.unwrap(),
            "wss://localhost:5000/v1/api/ws"
        );
    }

    #[test]
    fn test_production_endpoints() {
        let endpoints = IBEndpoints::production();
        assert_eq!(endpoints.rest_base, "https://api.ibkr.com/v1/api");
        assert_eq!(endpoints._ws_base.unwrap(), "wss://api.ibkr.com/v1/api/ws");
    }

    #[test]
    fn test_endpoint_paths() {
        assert_eq!(IBEndpoint::AuthStatus.path(), "/iserver/auth/status");
        assert_eq!(IBEndpoint::Tickle.path(), "/tickle");
        assert_eq!(
            IBEndpoint::PortfolioPositions {
                account_id: "DU12345".to_string(),
                page: 0
            }
            .path(),
            "/portfolio/DU12345/positions/0"
        );
        assert_eq!(
            IBEndpoint::MarketDataUnsubscribe { conid: 265598 }.path(),
            "/iserver/marketdata/265598/unsubscribe"
        );
    }

    #[test]
    fn test_format_symbol() {
        let symbol = Symbol::new("AAPL", "USD");
        assert_eq!(_format_symbol(&symbol), "AAPL");

        let symbol = Symbol::new("EUR", "USD");
        assert_eq!(_format_symbol(&symbol), "EUR.USD");

        let symbol = Symbol::new("BTC", "USDT");
        assert_eq!(_format_symbol(&symbol), "BTC/USDT");
    }

    #[test]
    fn test_parse_symbol() {
        assert_eq!(
            _parse_symbol("AAPL"),
            Some(Symbol::new("AAPL", "USD"))
        );
        assert_eq!(
            _parse_symbol("EUR.USD"),
            Some(Symbol::new("EUR", "USD"))
        );
        assert_eq!(
            _parse_symbol("BTC/USDT"),
            Some(Symbol::new("BTC", "USDT"))
        );
    }
}
