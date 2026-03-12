//! # Crypto.com Endpoints
//!
//! URL'ы и endpoint enum для Crypto.com Exchange API v1.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Crypto.com API
#[derive(Debug, Clone)]
pub struct CryptoComUrls {
    pub rest: &'static str,
    pub ws_user: &'static str,
    pub ws_market: &'static str,
}

impl CryptoComUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest: "https://api.crypto.com/exchange/v1",
        ws_user: "wss://stream.crypto.com/exchange/v1/user",
        ws_market: "wss://stream.crypto.com/exchange/v1/market",
    };

    /// UAT Sandbox URLs
    pub const TESTNET: Self = Self {
        rest: "https://uat-api.3ona.co/exchange/v1",
        ws_user: "wss://uat-stream.3ona.co/exchange/v1/user",
        ws_market: "wss://uat-stream.3ona.co/exchange/v1/market",
    };

    /// Get REST base URL (same for all account types in Crypto.com)
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Get WebSocket URL for user data
    pub fn ws_user_url(&self) -> &str {
        self.ws_user
    }

    /// Get WebSocket URL for market data
    pub fn ws_market_url(&self) -> &str {
        self.ws_market
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Crypto.com API methods
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CryptoComEndpoint {
    // === PUBLIC ===
    GetInstruments,
    GetBook,
    GetCandlestick,
    GetTrades,
    GetTickers,
    GetValuations,

    // === TRADING ===
    CreateOrder,
    CreateOrderList,
    CancelOrderList,
    AmendOrder,
    CancelOrder,
    CancelAllOrders,
    GetOpenOrders,
    GetOrderDetail,
    GetOrderHistory,
    GetUserTrades,

    // === ADVANCED ORDER MANAGEMENT (migrated 2026-01-28) ===
    /// Stop-loss and take-profit conditional orders.
    /// Replaces the legacy STOP_LOSS / STOP_LIMIT / TAKE_PROFIT types in CreateOrder.
    AdvancedCreateOrder,
    /// OCO (One-Cancels-Other) — Spot only.
    AdvancedCreateOco,
    /// OTO (One-Triggers-Other).
    AdvancedCreateOto,
    /// OTOCO (One-Triggers-One-Cancels-Other).
    AdvancedCreateOtoco,

    // === ACCOUNT ===
    UserBalance,
    GetAccounts,
    GetFeeRate,
    GetInstrumentFeeRate,
    GetTransactions,

    // === POSITIONS ===
    GetPositions,
    ClosePosition,
    ChangeAccountLeverage,
    ChangeIsolatedMarginLeverage,

    // === WEBSOCKET ===
    WsAuth,
    WsHeartbeat,
}

impl CryptoComEndpoint {
    /// Get method name for the endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // Public
            Self::GetInstruments => "public/get-instruments",
            Self::GetBook => "public/get-book",
            Self::GetCandlestick => "public/get-candlestick",
            Self::GetTrades => "public/get-trades",
            Self::GetTickers => "public/get-tickers",
            Self::GetValuations => "public/get-valuations",

            // Trading
            Self::CreateOrder => "private/create-order",
            Self::CreateOrderList => "private/create-order-list",
            Self::CancelOrderList => "private/cancel-order-list",
            Self::AmendOrder => "private/amend-order",
            Self::CancelOrder => "private/cancel-order",
            Self::CancelAllOrders => "private/cancel-all-orders",
            Self::GetOpenOrders => "private/get-open-orders",
            Self::GetOrderDetail => "private/get-order-detail",
            Self::GetOrderHistory => "private/get-order-history",
            Self::GetUserTrades => "private/get-trades",

            // Advanced order management (migrated 2026-01-28)
            Self::AdvancedCreateOrder => "private/advanced/create-order",
            Self::AdvancedCreateOco => "private/advanced/create-oco",
            Self::AdvancedCreateOto => "private/advanced/create-oto",
            Self::AdvancedCreateOtoco => "private/advanced/create-otoco",

            // Account
            Self::UserBalance => "private/user-balance",
            Self::GetAccounts => "private/get-accounts",
            Self::GetFeeRate => "private/get-fee-rate",
            Self::GetInstrumentFeeRate => "private/get-instrument-fee-rate",
            Self::GetTransactions => "private/get-transactions",

            // Positions
            Self::GetPositions => "private/get-positions",
            Self::ClosePosition => "private/close-position",
            Self::ChangeAccountLeverage => "private/change-account-leverage",
            Self::ChangeIsolatedMarginLeverage => "private/change-isolated-margin-leverage",

            // WebSocket
            Self::WsAuth => "public/auth",
            Self::WsHeartbeat => "public/respond-heartbeat",
        }
    }

    /// Does the endpoint require authentication
    pub fn requires_auth(&self) -> bool {
        self.method().starts_with("private/")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Instrument type for Crypto.com
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstrumentType {
    Spot,
    Perpetual,
    Futures,
    Index,
}

/// Format symbol for Crypto.com API
///
/// # Format Rules
/// - Spot: `BASE_QUOTE` (e.g., `BTC_USDT`) - underscore separator
/// - Perpetual: `BASEQUOTE-PERP` (e.g., `BTCUSD-PERP`) - no separator, hyphen before PERP
/// - Futures: Use `format_futures_symbol()` instead with expiry date
/// - Index: `BASEQUOTE-INDEX` (e.g., `BTCUSD-INDEX`)
///
/// # Examples
/// ```
/// format_symbol("BTC", "USDT", InstrumentType::Spot) // "BTC_USDT"
/// format_symbol("ETH", "USD", InstrumentType::Perpetual) // "ETHUSD-PERP"
/// ```
///
/// # Panics
/// Panics if InstrumentType::Futures is used. Use `format_futures_symbol()` instead.
pub fn format_symbol(base: &str, quote: &str, instrument_type: InstrumentType) -> String {
    let base_upper = base.to_uppercase();
    let quote_upper = quote.to_uppercase();

    match instrument_type {
        InstrumentType::Spot => {
            // Spot uses underscore: BTC_USDT
            format!("{}_{}", base_upper, quote_upper)
        }
        InstrumentType::Perpetual => {
            // Perpetual: no separator, -PERP suffix
            format!("{}{}-PERP", base_upper, quote_upper)
        }
        InstrumentType::Futures => {
            // This is a programming error - caller should use format_futures_symbol()
            // Not a runtime error, so expect() is appropriate here
            unreachable!("Futures symbols require expiry date. Use format_futures_symbol() instead.")
        }
        InstrumentType::Index => {
            // Index: no separator, -INDEX suffix
            format!("{}{}-INDEX", base_upper, quote_upper)
        }
    }
}

/// Format futures symbol with expiry date
///
/// # Format
/// `BASEQUOTE-YYMMDD...` (e.g., `BTCUSD-210528m2`)
///
/// # Arguments
/// * `base` - Base asset (e.g., "BTC")
/// * `quote` - Quote asset (e.g., "USD")
/// * `expiry` - Expiry date string (e.g., "210528m2")
pub fn _format_futures_symbol(base: &str, quote: &str, expiry: &str) -> String {
    format!("{}{}-{}", base.to_uppercase(), quote.to_uppercase(), expiry)
}

/// Convert AccountType to InstrumentType
pub fn account_type_to_instrument(account_type: AccountType) -> InstrumentType {
    match account_type {
        AccountType::Spot | AccountType::Margin => InstrumentType::Spot,
        AccountType::FuturesCross | AccountType::FuturesIsolated => InstrumentType::Perpetual,
    }
}

/// Map standard interval notation to Crypto.com timeframe
///
/// # Crypto.com Timeframes
/// - Modern format: `1m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `12h`, `1D`, `7D`, `14D`, `1M`
/// - Legacy format: `M1`, `M5`, etc. (also supported)
///
/// # Examples
/// ```
/// map_kline_interval("1m") // "1m"
/// map_kline_interval("1h") // "1h"
/// map_kline_interval("1d") // "1D"
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1h",
        "2h" => "2h",
        "4h" => "4h",
        "12h" => "12h",
        "1d" => "1D",
        "1w" => "7D",
        "1M" => "1M",
        _ => "1h", // default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spot_symbol_format() {
        assert_eq!(
            format_symbol("btc", "usdt", InstrumentType::Spot),
            "BTC_USDT"
        );
        assert_eq!(
            format_symbol("ETH", "USD", InstrumentType::Spot),
            "ETH_USD"
        );
    }

    #[test]
    fn test_perpetual_symbol_format() {
        assert_eq!(
            format_symbol("btc", "usd", InstrumentType::Perpetual),
            "BTCUSD-PERP"
        );
        assert_eq!(
            format_symbol("ETH", "USDT", InstrumentType::Perpetual),
            "ETHUSDT-PERP"
        );
    }

    #[test]
    fn test_index_symbol_format() {
        assert_eq!(
            format_symbol("btc", "usd", InstrumentType::Index),
            "BTCUSD-INDEX"
        );
    }

    #[test]
    fn test_kline_interval_mapping() {
        assert_eq!(map_kline_interval("1m"), "1m");
        assert_eq!(map_kline_interval("1h"), "1h");
        assert_eq!(map_kline_interval("1d"), "1D");
        assert_eq!(map_kline_interval("1w"), "7D");
    }

    #[test]
    fn test_method_names() {
        assert_eq!(CryptoComEndpoint::GetInstruments.method(), "public/get-instruments");
        assert_eq!(CryptoComEndpoint::CreateOrder.method(), "private/create-order");
        assert_eq!(CryptoComEndpoint::UserBalance.method(), "private/user-balance");
    }

    #[test]
    fn test_requires_auth() {
        assert!(!CryptoComEndpoint::GetInstruments.requires_auth());
        assert!(!CryptoComEndpoint::GetTickers.requires_auth());
        assert!(CryptoComEndpoint::CreateOrder.requires_auth());
        assert!(CryptoComEndpoint::UserBalance.requires_auth());
    }
}
