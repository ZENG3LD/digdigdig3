//! # Bybit Endpoints
//!
//! URL structures and endpoint enum for Bybit V5 API.

use crate::core::types::{AccountType, Symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL endpoints for Bybit API
pub struct BybitUrls;

impl BybitUrls {
    /// Get REST API base URL
    pub fn base_url(testnet: bool) -> &'static str {
        if testnet {
            "https://api-testnet.bybit.com"
        } else {
            "https://api.bybit.com"
        }
    }

    /// Get WebSocket URL for public channels (spot)
    pub fn ws_spot_url(testnet: bool) -> &'static str {
        if testnet {
            "wss://stream-testnet.bybit.com/v5/public/spot"
        } else {
            "wss://stream.bybit.com/v5/public/spot"
        }
    }

    /// Get WebSocket URL for public channels (linear/futures)
    pub fn ws_linear_url(testnet: bool) -> &'static str {
        if testnet {
            "wss://stream-testnet.bybit.com/v5/public/linear"
        } else {
            "wss://stream.bybit.com/v5/public/linear"
        }
    }

    /// Get WebSocket URL for private channels
    pub fn ws_private_url(testnet: bool) -> &'static str {
        if testnet {
            "wss://stream-testnet.bybit.com/v5/private"
        } else {
            "wss://stream.bybit.com/v5/private"
        }
    }

    /// Get appropriate WebSocket URL based on account type
    pub fn ws_url(account_type: AccountType, testnet: bool) -> &'static str {
        match account_type {
            AccountType::Spot | AccountType::Margin => Self::ws_spot_url(testnet),
            AccountType::FuturesCross | AccountType::FuturesIsolated => Self::ws_linear_url(testnet),
            _ => Self::ws_spot_url(testnet),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bybit V5 API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BybitEndpoint {
    // === MARKET DATA ===
    Ticker,           // GET /v5/market/tickers
    Orderbook,        // GET /v5/market/orderbook
    Klines,           // GET /v5/market/kline
    Symbols,          // GET /v5/market/instruments-info
    RecentTrades,     // GET /v5/market/recent-trade
    ServerTime,       // GET /v5/market/time

    // === ACCOUNT ===
    Balance,          // GET /v5/account/wallet-balance
    AccountInfo,      // GET /v5/account/info

    // === TRADING ===
    PlaceOrder,       // POST /v5/order/create
    CancelOrder,      // POST /v5/order/cancel
    CancelAllOrders,  // POST /v5/order/cancel-all
    OrderStatus,      // GET /v5/order/realtime
    OpenOrders,       // GET /v5/order/realtime
    OrderHistory,     // GET /v5/order/history

    // === POSITIONS (FUTURES) ===
    Positions,        // GET /v5/position/list
    SetLeverage,      // POST /v5/position/set-leverage
    SetMarginMode,    // POST /v5/position/switch-isolated
    AddMargin,        // POST /v5/position/add-margin
    TpSlMode,         // POST /v5/position/set-tpsl
    FundingRate,      // GET /v5/market/funding/history

    // === OPTIONAL TRAITS ===
    AmendOrder,        // POST /v5/order/amend
    BatchPlaceOrders,  // POST /v5/order/create-batch
    BatchCancelOrders, // POST /v5/order/cancel-batch
    /// Batch amend multiple orders: POST /v5/order/amend-batch
    BatchAmendOrders,  // POST /v5/order/amend-batch

    // === MARKET DATA EXTENSIONS ===
    OpenInterest,       // GET /v5/market/open-interest
    LongShortRatio,     // GET /v5/market/account-ratio
    MarkPriceKline,     // GET /v5/market/mark-price-kline
    IndexPriceKline,    // GET /v5/market/index-price-kline
    PremiumIndexKline,  // GET /v5/market/premium-index-price-kline

    // === FILL/TRADE HISTORY ===
    MyTrades,           // GET /v5/execution/list (signed)
    ClosedPnl,          // GET /v5/position/closed-pnl (signed)

    // === FEES ===
    FeeRate,          // GET /v5/account/fee-rate

    // === ACCOUNT TRANSFERS ===
    InterTransfer,         // POST /v5/asset/transfer/inter-transfer
    TransferHistory,       // GET  /v5/asset/transfer/query-inter-transfer-list

    // === CUSTODIAL FUNDS ===
    DepositAddress,        // GET  /v5/asset/deposit/query-address
    Withdraw,              // POST /v5/asset/withdraw/create
    DepositHistory,        // GET  /v5/asset/deposit/query-record
    WithdrawHistory,       // GET  /v5/asset/withdraw/query-record

    // === SUB-ACCOUNTS ===
    CreateSubMember,       // POST /v5/user/create-sub-member
    ListSubMembers,        // GET  /v5/user/query-sub-members
    UniversalTransfer,     // POST /v5/asset/transfer/universal-transfer
    SubAccountBalance,     // GET  /v5/asset/transfer/query-account-coins-balance

    // === ACCOUNT LEDGER / FUNDING HISTORY ===
    /// GET /v5/account/transaction-log — full ledger (all types incl. SETTLEMENT)
    TransactionLog,
}

impl BybitEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::Ticker => "/v5/market/tickers",
            Self::Orderbook => "/v5/market/orderbook",
            Self::Klines => "/v5/market/kline",
            Self::Symbols => "/v5/market/instruments-info",
            Self::RecentTrades => "/v5/market/recent-trade",
            Self::ServerTime => "/v5/market/time",

            // Account
            Self::Balance => "/v5/account/wallet-balance",
            Self::AccountInfo => "/v5/account/info",

            // Trading
            Self::PlaceOrder => "/v5/order/create",
            Self::CancelOrder => "/v5/order/cancel",
            Self::CancelAllOrders => "/v5/order/cancel-all",
            Self::OrderStatus => "/v5/order/realtime",
            Self::OpenOrders => "/v5/order/realtime",
            Self::OrderHistory => "/v5/order/history",

            // Positions
            Self::Positions => "/v5/position/list",
            Self::SetLeverage => "/v5/position/set-leverage",
            Self::SetMarginMode => "/v5/position/switch-isolated",
            Self::AddMargin => "/v5/position/add-margin",
            Self::TpSlMode => "/v5/position/trading-stop",
            Self::FundingRate => "/v5/market/funding/history",

            // Optional traits
            Self::AmendOrder => "/v5/order/amend",
            Self::BatchPlaceOrders => "/v5/order/create-batch",
            Self::BatchCancelOrders => "/v5/order/cancel-batch",
            Self::BatchAmendOrders => "/v5/order/amend-batch",

            // Market Data Extensions
            Self::OpenInterest => "/v5/market/open-interest",
            Self::LongShortRatio => "/v5/market/account-ratio",
            Self::MarkPriceKline => "/v5/market/mark-price-kline",
            Self::IndexPriceKline => "/v5/market/index-price-kline",
            Self::PremiumIndexKline => "/v5/market/premium-index-price-kline",

            // Fill/Trade History
            Self::MyTrades => "/v5/execution/list",
            Self::ClosedPnl => "/v5/position/closed-pnl",

            // Fees
            Self::FeeRate => "/v5/account/fee-rate",

            // Account Transfers
            Self::InterTransfer => "/v5/asset/transfer/inter-transfer",
            Self::TransferHistory => "/v5/asset/transfer/query-inter-transfer-list",

            // Custodial Funds
            Self::DepositAddress => "/v5/asset/deposit/query-address",
            Self::Withdraw => "/v5/asset/withdraw/create",
            Self::DepositHistory => "/v5/asset/deposit/query-record",
            Self::WithdrawHistory => "/v5/asset/withdraw/query-record",

            // Sub-Accounts
            Self::CreateSubMember => "/v5/user/create-sub-member",
            Self::ListSubMembers => "/v5/user/query-sub-members",
            Self::UniversalTransfer => "/v5/asset/transfer/universal-transfer",
            Self::SubAccountBalance => "/v5/asset/transfer/query-account-coins-balance",

            // Account Ledger / Funding History
            Self::TransactionLog => "/v5/account/transaction-log",
        }
    }

    /// Get HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST requests
            Self::PlaceOrder
            | Self::CancelOrder
            | Self::CancelAllOrders
            | Self::AmendOrder
            | Self::BatchPlaceOrders
            | Self::BatchCancelOrders
            | Self::BatchAmendOrders
            | Self::SetLeverage
            | Self::SetMarginMode
            | Self::AddMargin
            | Self::TpSlMode
            | Self::InterTransfer
            | Self::Withdraw
            | Self::CreateSubMember
            | Self::UniversalTransfer => "POST",

            // GET requests
            _ => "GET",
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_private(&self) -> bool {
        match self {
            // Public endpoints
            Self::Ticker
            | Self::Orderbook
            | Self::Klines
            | Self::Symbols
            | Self::RecentTrades
            | Self::ServerTime
            | Self::FundingRate
            | Self::OpenInterest
            | Self::LongShortRatio
            | Self::MarkPriceKline
            | Self::IndexPriceKline
            | Self::PremiumIndexKline => false,

            // Private endpoints
            _ => true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Bybit API
///
/// # Format
/// - Spot: `BTCUSDT` (no separator)
/// - Linear: `BTCUSDT` (same format, distinguished by `category` parameter)
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::bybit::format_symbol;
/// use connectors_v5::core::types::{Symbol, AccountType};
///
/// let symbol = Symbol::new("BTC", "USDT");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "BTCUSDT");
/// assert_eq!(format_symbol(&symbol, AccountType::FuturesCross), "BTCUSDT");
/// ```
pub fn format_symbol(symbol: &Symbol, _account_type: AccountType) -> String {
    // Bybit uses concatenated format for both spot and futures
    // Category parameter differentiates between spot/linear/inverse
    format!("{}{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
}

/// Get category parameter for account type
///
/// # Returns
/// - `"spot"` for Spot and Margin
/// - `"linear"` for Futures (USDT-margined perpetuals)
pub fn account_type_to_category(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot | AccountType::Margin => "spot",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "linear",
        _ => "spot",
    }
}

/// Map our AccountType to Bybit transfer account type string.
///
/// Bybit inter-transfer account types:
/// - `UNIFIED`  — Unified Trading Account (UTA)
/// - `SPOT`     — Spot account
/// - `CONTRACT` — Derivatives / Futures account
/// - `FUND`     — Funding (asset) account
pub fn account_type_to_transfer_type(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "SPOT",
        AccountType::Margin => "UNIFIED",
        AccountType::FuturesCross => "CONTRACT",
        AccountType::FuturesIsolated => "CONTRACT",
        _ => "SPOT",
    }
}

/// Map kline interval to Bybit format
///
/// # Bybit Interval Format
/// - Minutes: `1`, `3`, `5`, `15`, `30`, `60`, `120`, `240`, `360`, `720`
/// - Day: `D`
/// - Week: `W`
/// - Month: `M`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::bybit::map_kline_interval;
///
/// assert_eq!(map_kline_interval("1m"), "1");
/// assert_eq!(map_kline_interval("1h"), "60");
/// assert_eq!(map_kline_interval("1d"), "D");
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1",
        "3m" => "3",
        "5m" => "5",
        "15m" => "15",
        "30m" => "30",
        "1h" => "60",
        "2h" => "120",
        "4h" => "240",
        "6h" => "360",
        "12h" => "720",
        "1d" => "D",
        "1w" => "W",
        "1M" => "M",
        _ => "60", // default to 1 hour
    }
}
