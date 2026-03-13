//! # Gate.io Endpoints
//!
//! URLs and endpoint enum for Gate.io API V4.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URLs for Gate.io API
#[derive(Debug, Clone)]
pub struct GateioUrls {
    pub spot_rest: &'static str,
    pub futures_rest: &'static str,
    pub spot_ws: &'static str,
    pub futures_ws: &'static str,
}

impl GateioUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        spot_rest: "https://api.gateio.ws/api/v4",
        futures_rest: "https://fx-api.gateio.ws/api/v4",
        spot_ws: "wss://api.gateio.ws/ws/v4/",
        futures_ws: "wss://fx-ws.gateio.ws/v4/ws/usdt",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        spot_rest: "https://api-testnet.gateapi.io/api/v4",
        futures_rest: "https://fx-api-testnet.gateio.ws/api/v4",
        spot_ws: "wss://api-testnet.gateapi.io/ws/v4",
        futures_ws: "wss://fx-api-testnet.gateio.ws/ws/v4",
    };

    /// Get REST base URL for account type
    pub fn rest_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_rest,
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_rest,
        }
    }

    /// Get WebSocket URL for account type
    pub fn ws_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_ws,
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_ws,
        }
    }

    /// Get settle parameter for futures endpoints (usdt or btc)
    pub fn settle(&self, account_type: AccountType) -> &'static str {
        match account_type {
            AccountType::FuturesCross | AccountType::FuturesIsolated => "usdt",
            _ => "usdt",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Gate.io API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GateioEndpoint {
    // === GENERAL ===
    ServerTime,

    // === SPOT MARKET DATA ===
    SpotTickers,
    SpotOrderbook,
    SpotKlines,
    SpotSymbols,

    // === SPOT TRADING ===
    SpotCreateOrder,
    SpotCancelOrder,
    SpotGetOrder,
    SpotOpenOrders,
    SpotCancelAllOrders,

    // === SPOT ACCOUNT ===
    SpotAccounts,

    // === FUTURES MARKET DATA ===
    FuturesTickers,
    FuturesOrderbook,
    FuturesKlines,
    FuturesContracts,
    FundingRate,

    // === FUTURES TRADING ===
    FuturesCreateOrder,
    FuturesCancelOrder,
    FuturesGetOrder,
    FuturesOpenOrders,
    FuturesCancelAllOrders,

    // === SPOT PRICE-TRIGGERED (STOP) ORDERS ===
    SpotPriceOrders,

    // === SPOT AMEND ORDER ===
    SpotAmendOrder,

    // === SPOT BATCH ORDERS ===
    SpotBatchOrders,

    // === FUTURES BATCH ORDERS ===
    FuturesBatchOrders,

    // === FUTURES AMEND ORDER ===
    FuturesAmendOrder,

    // === FUTURES ACCOUNT ===
    FuturesAccounts,
    FuturesPositions,
    FuturesPosition,
    FuturesSetLeverage,

    // === ACCOUNT TRANSFERS ===
    /// POST /wallet/transfers — transfer between account types
    WalletTransfer,
    /// GET /wallet/transfers — transfer history (paginated)
    WalletTransferHistory,

    // === CUSTODIAL FUNDS ===
    /// GET /wallet/deposit_address — deposit address for an asset
    DepositAddress,
    /// POST /withdrawals — submit withdrawal request
    Withdraw,
    /// GET /wallet/deposits — deposit history
    DepositHistory,
    /// GET /wallet/withdrawals — withdrawal history
    WithdrawalHistory,

    // === SUB-ACCOUNTS ===
    /// POST /sub_accounts — create sub-account
    SubAccountCreate,
    /// GET /sub_accounts — list sub-accounts
    SubAccountList,
    /// POST /sub_accounts/transfers — transfer to/from sub-account
    SubAccountTransfer,
    /// GET /sub_accounts/{user_id}/balances — get sub-account balances
    SubAccountBalance,
}

impl GateioEndpoint {
    /// Get path for endpoint
    pub fn path(&self, settle: Option<&str>) -> String {
        match self {
            // General
            Self::ServerTime => "/spot/time".to_string(),

            // Spot Market Data
            Self::SpotTickers => "/spot/tickers".to_string(),
            Self::SpotOrderbook => "/spot/order_book".to_string(),
            Self::SpotKlines => "/spot/candlesticks".to_string(),
            Self::SpotSymbols => "/spot/currency_pairs".to_string(),

            // Spot Trading
            Self::SpotCreateOrder => "/spot/orders".to_string(),
            Self::SpotCancelOrder => "/spot/orders/{order_id}".to_string(),
            Self::SpotGetOrder => "/spot/orders/{order_id}".to_string(),
            Self::SpotOpenOrders => "/spot/orders".to_string(),
            Self::SpotCancelAllOrders => "/spot/orders".to_string(),

            // Spot Account
            Self::SpotAccounts => "/spot/accounts".to_string(),

            // Futures Market Data
            Self::FuturesTickers => format!("/futures/{}/tickers", settle.unwrap_or("usdt")),
            Self::FuturesOrderbook => format!("/futures/{}/order_book", settle.unwrap_or("usdt")),
            Self::FuturesKlines => format!("/futures/{}/candlesticks", settle.unwrap_or("usdt")),
            Self::FuturesContracts => format!("/futures/{}/contracts", settle.unwrap_or("usdt")),
            Self::FundingRate => format!("/futures/{}/funding_rate", settle.unwrap_or("usdt")),

            // Futures Trading
            Self::FuturesCreateOrder => format!("/futures/{}/orders", settle.unwrap_or("usdt")),
            Self::FuturesCancelOrder => format!("/futures/{}/orders/{{order_id}}", settle.unwrap_or("usdt")),
            Self::FuturesGetOrder => format!("/futures/{}/orders/{{order_id}}", settle.unwrap_or("usdt")),
            Self::FuturesOpenOrders => format!("/futures/{}/orders", settle.unwrap_or("usdt")),
            Self::FuturesCancelAllOrders => format!("/futures/{}/orders", settle.unwrap_or("usdt")),

            // Spot Price-Triggered (Stop) Orders
            Self::SpotPriceOrders => "/spot/price_orders".to_string(),

            // Spot Amend Order
            Self::SpotAmendOrder => "/spot/orders/{order_id}".to_string(),

            // Spot Batch Orders
            Self::SpotBatchOrders => "/spot/batch_orders".to_string(),

            // Futures Batch Orders
            Self::FuturesBatchOrders => format!("/futures/{}/batch_orders", settle.unwrap_or("usdt")),

            // Futures Amend Order
            Self::FuturesAmendOrder => format!("/futures/{}/orders/{{order_id}}", settle.unwrap_or("usdt")),

            // Futures Account
            Self::FuturesAccounts => format!("/futures/{}/accounts", settle.unwrap_or("usdt")),
            Self::FuturesPositions => format!("/futures/{}/positions", settle.unwrap_or("usdt")),
            Self::FuturesPosition => format!("/futures/{}/positions/{{contract}}", settle.unwrap_or("usdt")),
            Self::FuturesSetLeverage => format!("/futures/{}/positions/{{contract}}/leverage", settle.unwrap_or("usdt")),

            // Account Transfers
            Self::WalletTransfer => "/wallet/transfers".to_string(),
            Self::WalletTransferHistory => "/wallet/transfers".to_string(),

            // Custodial Funds
            Self::DepositAddress => "/wallet/deposit_address".to_string(),
            Self::Withdraw => "/withdrawals".to_string(),
            Self::DepositHistory => "/wallet/deposits".to_string(),
            Self::WithdrawalHistory => "/wallet/withdrawals".to_string(),

            // Sub-Accounts
            Self::SubAccountCreate => "/sub_accounts".to_string(),
            Self::SubAccountList => "/sub_accounts".to_string(),
            Self::SubAccountTransfer => "/sub_accounts/transfers".to_string(),
            Self::SubAccountBalance => "/sub_accounts/{user_id}/balances".to_string(),
        }
    }

    /// Does endpoint require authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::ServerTime
            | Self::SpotTickers
            | Self::SpotOrderbook
            | Self::SpotKlines
            | Self::SpotSymbols
            | Self::FuturesTickers
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesContracts
            | Self::FundingRate => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Self::SpotCreateOrder
            | Self::FuturesCreateOrder
            | Self::SpotPriceOrders
            | Self::SpotBatchOrders
            | Self::FuturesBatchOrders
            | Self::FuturesSetLeverage
            | Self::WalletTransfer
            | Self::Withdraw
            | Self::SubAccountCreate
            | Self::SubAccountTransfer => "POST",

            Self::SpotCancelOrder
            | Self::SpotCancelAllOrders
            | Self::FuturesCancelOrder
            | Self::FuturesCancelAllOrders => "DELETE",

            Self::FuturesAmendOrder
            | Self::SpotAmendOrder => "PATCH",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Gate.io
///
/// # Symbol Format
/// - Spot: `BTC_USDT` (underscore separator)
/// - Futures: `BTC_USDT` (same format!)
///
/// # Examples
/// - Spot: `BTC_USDT`, `ETH_BTC`
/// - Futures USDT: `BTC_USDT`, `ETH_USDT`
/// - Futures BTC: `BTC_USD`, `ETH_USD`
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    // Gate.io uses same format for spot and futures: BASE_QUOTE with underscore
    format!("{}_{}", base.to_uppercase(), quote.to_uppercase())
}

/// Map kline interval to Gate.io format (same for spot and futures)
///
/// # Supported Intervals
/// - `10s`, `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `8h`, `1d`, `7d`, `30d`
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "10s" => "10s",
        "1m" => "1m",
        "3m" => "1m", // Not supported, use 1m
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1h",
        "2h" => "1h", // Not supported, use 1h
        "4h" => "4h",
        "6h" => "4h", // Not supported, use 4h
        "8h" => "8h",
        "12h" => "8h", // Not supported, use 8h
        "1d" => "1d",
        "1w" => "7d",
        "1M" => "30d",
        _ => "1h",
    }
}
