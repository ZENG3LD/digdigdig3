//! # Deribit Endpoints
//!
//! URL'ы и JSON-RPC методы для Deribit API.
//!
//! ## JSON-RPC Format
//! All requests use JSON-RPC 2.0 format:
//! - Method: `{scope}/{method_name}` (e.g., `public/get_instruments`, `private/buy`)
//! - Parameters: Named objects (no positional parameters)
//! - All requests via POST (even queries)

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Deribit API
#[derive(Debug, Clone)]
pub struct DeribitUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl DeribitUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest: "https://www.deribit.com/api/v2",
        ws: "wss://www.deribit.com/ws/api/v2",
    };

    /// Test URLs
    pub const TESTNET: Self = Self {
        rest: "https://test.deribit.com/api/v2",
        ws: "wss://test.deribit.com/ws/api/v2",
    };

    /// Get REST base URL (same for all account types)
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Get WebSocket URL (same for all account types)
    pub fn ws_url(&self) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// JSON-RPC METHODS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deribit JSON-RPC API methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DeribitMethod {
    // === AUTHENTICATION ===
    Auth,

    // === PUBLIC MARKET DATA ===
    GetInstruments,
    GetOrderBook,
    Ticker,
    GetBookSummaryByCurrency,
    GetLastTradesByInstrument,
    GetLastTradesByInstrumentAndTime,
    GetTradingviewChartData,

    // === PRIVATE TRADING ===
    Buy,
    Sell,
    Edit,
    Cancel,
    CancelByLabel,
    CancelAll,
    CancelAllByCurrency,
    CancelAllByInstrument,
    GetOpenOrders,
    GetOpenOrdersByCurrency,
    GetOpenOrdersByInstrument,
    GetOrderState,
    ClosePosition,

    // === PRIVATE ACCOUNT ===
    GetAccountSummary,
    GetUserTradesByInstrument,
    GetUserTradesByCurrency,
    GetSettlementHistoryByInstrument,

    // === PRIVATE CUSTODIAL FUNDS ===
    GetCurrentDepositAddress,
    Withdraw,
    GetDeposits,
    GetWithdrawals,

    // === PRIVATE POSITIONS ===
    GetPosition,
    GetPositions,

    // === WEBSOCKET ===
    Subscribe,
    Unsubscribe,
    SubscribePrivate,
    UnsubscribePrivate,
    Test,
}

impl DeribitMethod {
    /// Get JSON-RPC method name
    pub fn method(&self) -> &'static str {
        match self {
            // Authentication
            Self::Auth => "public/auth",

            // Public Market Data
            Self::GetInstruments => "public/get_instruments",
            Self::GetOrderBook => "public/get_order_book",
            Self::Ticker => "public/ticker",
            Self::GetBookSummaryByCurrency => "public/get_book_summary_by_currency",
            Self::GetLastTradesByInstrument => "public/get_last_trades_by_instrument",
            Self::GetLastTradesByInstrumentAndTime => "public/get_last_trades_by_instrument_and_time",
            Self::GetTradingviewChartData => "public/get_tradingview_chart_data",

            // Private Trading
            Self::Buy => "private/buy",
            Self::Sell => "private/sell",
            Self::Edit => "private/edit",
            Self::Cancel => "private/cancel",
            Self::CancelByLabel => "private/cancel_by_label",
            Self::CancelAll => "private/cancel_all",
            Self::CancelAllByCurrency => "private/cancel_all_by_currency",
            Self::CancelAllByInstrument => "private/cancel_all_by_instrument",
            Self::GetOpenOrders => "private/get_open_orders",
            Self::GetOpenOrdersByCurrency => "private/get_open_orders_by_currency",
            Self::GetOpenOrdersByInstrument => "private/get_open_orders_by_instrument",
            Self::GetOrderState => "private/get_order_state",
            Self::ClosePosition => "private/close_position",

            // Private Account
            Self::GetAccountSummary => "private/get_account_summary",
            Self::GetUserTradesByInstrument => "private/get_user_trades_by_instrument",
            Self::GetUserTradesByCurrency => "private/get_user_trades_by_currency",
            Self::GetSettlementHistoryByInstrument => "private/get_settlement_history_by_instrument",

            // Private Custodial Funds
            Self::GetCurrentDepositAddress => "private/get_current_deposit_address",
            Self::Withdraw => "private/withdraw",
            Self::GetDeposits => "private/get_deposits",
            Self::GetWithdrawals => "private/get_withdrawals",

            // Private Positions
            Self::GetPosition => "private/get_position",
            Self::GetPositions => "private/get_positions",

            // WebSocket
            Self::Subscribe => "public/subscribe",
            Self::Unsubscribe => "public/unsubscribe",
            Self::SubscribePrivate => "private/subscribe",
            Self::UnsubscribePrivate => "private/unsubscribe",
            Self::Test => "public/test",
        }
    }

    /// Check if method requires authentication
    pub fn requires_auth(&self) -> bool {
        self.method().starts_with("private/")
    }

    /// HTTP method (all JSON-RPC use POST)
    pub fn http_method(&self) -> &'static str {
        "POST"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Deribit
///
/// # Deribit Instrument Naming
/// - **Perpetuals**: `BTC-PERPETUAL`, `ETH-PERPETUAL`
/// - **Linear Perpetuals**: `SOL_USDC-PERPETUAL`, `XRP_USDC-PERPETUAL`
/// - **Futures**: `BTC-29MAR24`, `ETH-27DEC24`
/// - **Options**: `BTC-27DEC24-50000-C`, `ETH-29MAR24-3000-P`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::deribit::endpoints::format_symbol;
/// use connectors_v5::core::types::AccountType;
///
/// // Perpetual futures
/// assert_eq!(format_symbol("BTC", "USD", AccountType::FuturesCross), "BTC-PERPETUAL");
/// assert_eq!(format_symbol("ETH", "USD", AccountType::FuturesCross), "ETH-PERPETUAL");
///
/// // Linear perpetuals (USDC-settled)
/// assert_eq!(format_symbol("SOL", "USDC", AccountType::FuturesCross), "SOL_USDC-PERPETUAL");
/// ```
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot => {
            // Deribit has very limited spot (mainly derivatives exchange)
            // For consistency, use hyphen separator
            format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            let base = base.to_uppercase();
            let quote = quote.to_uppercase();

            // Linear perpetuals (USDC-settled): BASE_USDC-PERPETUAL
            if quote == "USDC" {
                format!("{}_USDC-PERPETUAL", base)
            }
            // Inverse perpetuals (BTC/ETH-settled): BASE-PERPETUAL
            else {
                format!("{}-PERPETUAL", base)
            }
        }
        AccountType::Margin => {
            // Deribit doesn't have traditional margin trading
            // Default to perpetual format
            format!("{}-PERPETUAL", base.to_uppercase())
        }
    }
}

/// Parse currency from instrument name
///
/// # Examples
/// - `BTC-PERPETUAL` -> `BTC`
/// - `ETH-29MAR24` -> `ETH`
/// - `SOL_USDC-PERPETUAL` -> `SOL`
/// - `BTC-27DEC24-50000-C` -> `BTC`
pub fn parse_currency(instrument_name: &str) -> Option<&str> {
    // Split by hyphen or underscore
    let first_part = instrument_name.split(&['-', '_'][..]).next()?;
    Some(first_part)
}

/// Parse instrument kind from instrument name
///
/// # Returns
/// - `"future"` for dated futures (e.g., `BTC-29MAR24`)
/// - `"option"` for options (e.g., `BTC-27DEC24-50000-C`)
/// - `"perpetual"` for perpetuals (e.g., `BTC-PERPETUAL`)
/// - `"linear_perpetual"` for USDC perpetuals (e.g., `SOL_USDC-PERPETUAL`)
pub fn parse_instrument_kind(instrument_name: &str) -> &'static str {
    if instrument_name.ends_with("-PERPETUAL") {
        if instrument_name.contains("_USDC") {
            "linear_perpetual"
        } else {
            "perpetual"
        }
    } else if instrument_name.matches('-').count() == 3 {
        // Format: BASE-DDMMMYY-STRIKE-C/P
        "option"
    } else if instrument_name.matches('-').count() == 1 {
        // Format: BASE-DDMMMYY
        "future"
    } else {
        "unknown"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        // Inverse perpetuals
        assert_eq!(
            format_symbol("BTC", "USD", AccountType::FuturesCross),
            "BTC-PERPETUAL"
        );
        assert_eq!(
            format_symbol("ETH", "USD", AccountType::FuturesCross),
            "ETH-PERPETUAL"
        );

        // Linear perpetuals (USDC)
        assert_eq!(
            format_symbol("SOL", "USDC", AccountType::FuturesCross),
            "SOL_USDC-PERPETUAL"
        );
        assert_eq!(
            format_symbol("XRP", "USDC", AccountType::FuturesCross),
            "XRP_USDC-PERPETUAL"
        );

        // Spot (limited on Deribit)
        assert_eq!(
            format_symbol("BTC", "USDC", AccountType::Spot),
            "BTC-USDC"
        );
    }

    #[test]
    fn test_parse_currency() {
        assert_eq!(parse_currency("BTC-PERPETUAL"), Some("BTC"));
        assert_eq!(parse_currency("ETH-29MAR24"), Some("ETH"));
        assert_eq!(parse_currency("SOL_USDC-PERPETUAL"), Some("SOL"));
        assert_eq!(parse_currency("BTC-27DEC24-50000-C"), Some("BTC"));
    }

    #[test]
    fn test_parse_instrument_kind() {
        assert_eq!(parse_instrument_kind("BTC-PERPETUAL"), "perpetual");
        assert_eq!(parse_instrument_kind("ETH-PERPETUAL"), "perpetual");
        assert_eq!(parse_instrument_kind("SOL_USDC-PERPETUAL"), "linear_perpetual");
        assert_eq!(parse_instrument_kind("BTC-29MAR24"), "future");
        assert_eq!(parse_instrument_kind("ETH-27DEC24"), "future");
        assert_eq!(parse_instrument_kind("BTC-27DEC24-50000-C"), "option");
        assert_eq!(parse_instrument_kind("ETH-29MAR24-3000-P"), "option");
    }

    #[test]
    fn test_method_names() {
        assert_eq!(DeribitMethod::Auth.method(), "public/auth");
        assert_eq!(DeribitMethod::Buy.method(), "private/buy");
        assert_eq!(DeribitMethod::GetOrderBook.method(), "public/get_order_book");
        assert_eq!(DeribitMethod::GetPositions.method(), "private/get_positions");
    }

    #[test]
    fn test_requires_auth() {
        assert!(!DeribitMethod::Auth.requires_auth());
        assert!(!DeribitMethod::GetOrderBook.requires_auth());
        assert!(DeribitMethod::Buy.requires_auth());
        assert!(DeribitMethod::GetPositions.requires_auth());
    }
}
