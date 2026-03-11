//! # Paradex Endpoints
//!
//! URL'ы и endpoint enum для Paradex API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Paradex API
#[derive(Debug, Clone)]
pub struct ParadexUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl ParadexUrls {
    /// Production URLs (Mainnet)
    pub const MAINNET: Self = Self {
        rest: "https://api.prod.paradex.trade/v1",
        ws: "wss://ws.api.prod.paradex.trade/v1",
    };

    /// Testnet URLs (Sepolia)
    pub const TESTNET: Self = Self {
        rest: "https://api.testnet.paradex.trade/v1",
        ws: "wss://ws.api.testnet.paradex.trade/v1",
    };

    /// Получить REST base URL (Paradex только perpetuals, всегда одна база)
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Получить WebSocket URL
    pub fn ws_url(&self) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Paradex API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ParadexEndpoint {
    // === AUTHENTICATION ===
    Auth,

    // === SYSTEM ===
    SystemConfig,
    SystemState,
    SystemTime,

    // === MARKET DATA (Public) ===
    Markets,
    MarketsSummary,
    Orderbook,
    OrderbookInteractive,
    BboInteractive,
    Trades,
    Klines,

    // === ACCOUNT (Private) ===
    Account,
    AccountInfo,
    AccountHistory,
    Balances,
    Positions,
    Subaccounts,

    // === TRADING (Private) ===
    CreateOrder,
    CreateOrderBatch,
    GetOrder,
    GetOrderByClientId,
    OpenOrders,
    OrdersHistory,
    CancelOrder,
    CancelOrderBatch,
    CancelAllOrders,
    ModifyOrder,

    // === TRADE HISTORY (Private) ===
    Fills,
    FundingPayments,
    Transactions,
    Transfers,
    Liquidations,
    Tradebusts,
}

impl ParadexEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Authentication
            Self::Auth => "/auth",

            // System
            Self::SystemConfig => "/system/config",
            Self::SystemState => "/system/state",
            Self::SystemTime => "/system/time",

            // Market Data
            Self::Markets => "/markets",
            Self::MarketsSummary => "/markets/summary",
            Self::Orderbook => "/orderbook/{market}",
            Self::OrderbookInteractive => "/orderbook/{market}/interactive",
            Self::BboInteractive => "/bbo/{market}/interactive",
            Self::Trades => "/trades",
            Self::Klines => "/klines",

            // Account
            Self::Account => "/account",
            Self::AccountInfo => "/account/info",
            Self::AccountHistory => "/account/history",
            Self::Balances => "/balances",
            Self::Positions => "/positions",
            Self::Subaccounts => "/subaccounts",

            // Trading
            Self::CreateOrder => "/orders",
            Self::CreateOrderBatch => "/orders/batch",
            Self::GetOrder => "/orders/{order_id}",
            Self::GetOrderByClientId => "/orders/by-client-id/{client_id}",
            Self::OpenOrders => "/orders",
            Self::OrdersHistory => "/orders/history",
            Self::CancelOrder => "/orders/{order_id}",
            Self::CancelOrderBatch => "/orders/batch",
            Self::CancelAllOrders => "/orders",
            Self::ModifyOrder => "/orders/{order_id}",

            // Trade History
            Self::Fills => "/fills",
            Self::FundingPayments => "/funding/payments",
            Self::Transactions => "/transactions",
            Self::Transfers => "/transfers",
            Self::Liquidations => "/liquidations",
            Self::Tradebusts => "/tradebusts",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::SystemConfig
            | Self::SystemState
            | Self::SystemTime
            | Self::Markets
            | Self::MarketsSummary
            | Self::Orderbook
            | Self::OrderbookInteractive
            | Self::BboInteractive
            | Self::Trades
            | Self::Klines => false,

            // Private endpoints (все остальные требуют JWT token)
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            // POST methods
            Self::Auth
            | Self::CreateOrder
            | Self::CreateOrderBatch => "POST",

            // DELETE methods
            Self::CancelOrder
            | Self::CancelOrderBatch
            | Self::CancelAllOrders => "DELETE",

            // PUT methods
            Self::ModifyOrder => "PUT",

            // GET methods (default)
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Paradex
///
/// # Paradex Symbol Format
/// - Perpetuals: `{BASE}-{QUOTE}-PERP` (e.g., "BTC-USD-PERP")
/// - Perpetual Options: `{BASE}-{QUOTE}-PERP_OPTION` (rare)
/// - All markets use cross-margin or isolated margin
///
/// # Examples
/// - BTC perpetual: `BTC-USD-PERP`
/// - ETH perpetual: `ETH-USD-PERP`
/// - SOL perpetual: `SOL-USD-PERP`
///
/// Note: Paradex only supports perpetual futures, no spot trading
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    // Paradex только perpetuals, всегда формат BASE-QUOTE-PERP
    format!("{}-{}-PERP", base.to_uppercase(), quote.to_uppercase())
}

/// Маппинг интервала kline для Paradex API
///
/// # Paradex API Format
/// Parameter: `resolution` (string)
/// Values: Based on Python SDK, likely similar to TradingView format
///
/// Supported intervals:
/// - Minutes: "1", "5", "15", "30"
/// - Hours: "60", "240" (1h, 4h)
/// - Days: "D" or "1D"
/// - Weeks: "W" or "1W"
pub fn map_kline_resolution(interval: &str) -> &'static str {
    match interval {
        "1m" => "1",
        "5m" => "5",
        "15m" => "15",
        "30m" => "30",
        "1h" => "60",
        "4h" => "240",
        "1d" => "D",
        "1w" => "W",
        _ => "60", // default 1 hour
    }
}
