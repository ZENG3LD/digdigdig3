//! # HTX Endpoints
//!
//! URL structures and endpoint enum for HTX API.

use crate::core::types::{AccountType, Symbol};

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL endpoints for HTX API
pub struct HtxUrls;

impl HtxUrls {
    /// Get REST API base URL for spot trading
    pub fn base_url(_testnet: bool) -> &'static str {
        // HTX doesn't have a dedicated testnet, use main
        "https://api.huobi.pro"
    }

    /// Get REST API base URL for futures/perpetuals
    pub fn futures_base_url(_testnet: bool) -> &'static str {
        "https://api.hbdm.com"
    }

    /// Get WebSocket URL for market data (public, spot)
    pub fn ws_market_url(_testnet: bool) -> &'static str {
        "wss://api.huobi.pro/ws"
    }

    /// Get WebSocket URL for USDT-margined futures/perpetuals
    pub fn ws_linear_swap_url(_testnet: bool) -> &'static str {
        "wss://api.hbdm.com/linear-swap-ws"
    }

    /// Get WebSocket URL for MBP feed (public, incremental updates)
    pub fn ws_mbp_url(_testnet: bool) -> &'static str {
        "wss://api.huobi.pro/feed"
    }

    /// Get WebSocket URL for account/orders (private, requires auth)
    pub fn ws_account_url(_testnet: bool) -> &'static str {
        "wss://api.huobi.pro/ws/v2"
    }

    /// Get AWS-optimized REST API base URL
    pub fn base_url_aws(_testnet: bool) -> &'static str {
        "https://api-aws.huobi.pro"
    }

    /// Get AWS-optimized WebSocket URL for market data
    pub fn ws_market_url_aws(_testnet: bool) -> &'static str {
        "wss://api-aws.huobi.pro/ws"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// HTX API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HtxEndpoint {
    // === MARKET DATA ===
    ServerTime,       // GET /v1/common/timestamp
    Symbols,          // GET /v2/settings/common/symbols
    SymbolsV1,        // GET /v1/common/symbols (legacy)
    Ticker,           // GET /market/detail/merged
    Tickers,          // GET /market/tickers
    Orderbook,        // GET /market/depth
    Klines,           // GET /market/history/kline
    RecentTrades,     // GET /market/trade
    HistoryTrades,    // GET /market/history/trade

    // === ACCOUNT ===
    AccountList,      // GET /v1/account/accounts
    Balance,          // GET /v1/account/accounts/{account-id}/balance
    AccountInfo,      // GET /v2/account/asset-valuation
    Ledger,           // GET /v2/account/ledger

    // === TRADING ===
    PlaceOrder,             // POST /v1/order/orders/place
    CancelOrder,            // POST /v1/order/orders/{order-id}/submitcancel
    CancelAllOrders,        // POST /v1/order/orders/batchcancel (by IDs, up to 50)
    CancelOpenOrders,       // POST /v1/order/orders/batchCancelOpenOrders (true cancel-all)
    OrderStatus,            // GET /v1/order/orders/{order-id}
    OpenOrders,             // GET /v1/order/openOrders
    OrderHistory,           // GET /v1/order/orders
    MatchResults,           // GET /v1/order/matchresults
    TransactFee,            // GET /v2/reference/transact-fee-rate

    // === WALLET ===
    DepositAddress,   // GET /v2/account/deposit/address
    WithdrawQuota,    // GET /v2/account/withdraw/quota
    WithdrawAddress,  // GET /v2/account/withdraw/address
    Withdraw,         // POST /v1/dw/withdraw/api/create
    WithdrawCancel,   // POST /v1/dw/withdraw-virtual/{withdraw-id}/cancel
    DepositHistory,   // GET /v1/query/deposit-withdraw
    WithdrawHistory,  // GET /v1/query/deposit-withdraw

    // === FUTURES MARKET DATA (USDT-margined) ===
    FuturesTicker,    // GET /linear-swap-ex/market/detail/merged
    FuturesOrderbook, // GET /linear-swap-ex/market/depth
    FuturesKlines,    // GET /linear-swap-ex/market/history/kline
    FuturesTrades,    // GET /linear-swap-ex/market/trade

    // === EXTENDED ENDPOINTS ===
    /// GET /v1/order/orders/{order-id}/matchresults — fills for a specific order (signed)
    OrderMatchResults,
    /// GET /linear-swap-api/v1/swap-open-interest — USDT-margined open interest
    OpenInterest,
    /// GET /linear-swap-api/v3/swap-funding-rate-history — historical funding rates
    FundingRateHistory,
    /// GET /linear-swap-ex/market/index — mark price + index price
    MarkPrice,
    /// GET /linear-swap-ex/market/history/mark_price_kline — mark price kline
    MarkPriceKline,

    // === ALGO ORDERS ===
    /// POST /v2/algo-orders — place trailing stop or other algo orders
    AlgoOrders,

    // === TRANSFERS ===
    Transfer,                // POST /v1/futures/transfer
    TransferHistory,         // GET  /v2/account/transfer

    // === SUB ACCOUNTS ===
    SubAccountCreate,        // POST /v2/sub-user/creation
    SubAccountList,          // GET  /v2/sub-user/user-list
    SubAccountTransfer,      // POST /v1/subuser/transfer
    SubAccountBalance,       // GET  /v1/account/accounts/{sub-uid}
}

impl HtxEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::ServerTime => "/v1/common/timestamp",
            Self::Symbols => "/v2/settings/common/symbols",
            Self::SymbolsV1 => "/v1/common/symbols",
            Self::Ticker => "/market/detail/merged",
            Self::Tickers => "/market/tickers",
            Self::Orderbook => "/market/depth",
            Self::Klines => "/market/history/kline",
            Self::RecentTrades => "/market/trade",
            Self::HistoryTrades => "/market/history/trade",

            // Account
            Self::AccountList => "/v1/account/accounts",
            Self::Balance => "/v1/account/accounts/{account-id}/balance",
            Self::AccountInfo => "/v2/account/asset-valuation",
            Self::Ledger => "/v2/account/ledger",

            // Trading
            Self::PlaceOrder => "/v1/order/orders/place",
            Self::CancelOrder => "/v1/order/orders/{order-id}/submitcancel",
            Self::CancelAllOrders => "/v1/order/orders/batchcancel",
            Self::CancelOpenOrders => "/v1/order/orders/batchCancelOpenOrders",
            Self::OrderStatus => "/v1/order/orders/{order-id}",
            Self::OpenOrders => "/v1/order/openOrders",
            Self::OrderHistory => "/v1/order/orders",
            Self::MatchResults => "/v1/order/matchresults",
            Self::TransactFee => "/v2/reference/transact-fee-rate",

            // Wallet
            Self::DepositAddress => "/v2/account/deposit/address",
            Self::WithdrawQuota => "/v2/account/withdraw/quota",
            Self::WithdrawAddress => "/v2/account/withdraw/address",
            Self::Withdraw => "/v1/dw/withdraw/api/create",
            Self::WithdrawCancel => "/v1/dw/withdraw-virtual/{withdraw-id}/cancel",
            Self::DepositHistory => "/v1/query/deposit-withdraw",
            Self::WithdrawHistory => "/v1/query/deposit-withdraw",

            // Futures Market Data
            Self::FuturesTicker => "/linear-swap-ex/market/detail/merged",
            Self::FuturesOrderbook => "/linear-swap-ex/market/depth",
            Self::FuturesKlines => "/linear-swap-ex/market/history/kline",
            Self::FuturesTrades => "/linear-swap-ex/market/trade",

            // Extended endpoints
            Self::OrderMatchResults => "/v1/order/orders/{order-id}/matchresults",
            Self::OpenInterest => "/linear-swap-api/v1/swap-open-interest",
            Self::FundingRateHistory => "/linear-swap-api/v3/swap-funding-rate-history",
            Self::MarkPrice => "/linear-swap-ex/market/index",
            Self::MarkPriceKline => "/linear-swap-ex/market/history/mark_price_kline",

            // Algo Orders
            Self::AlgoOrders => "/v2/algo-orders",

            // Transfers
            Self::Transfer => "/v1/futures/transfer",
            Self::TransferHistory => "/v2/account/transfer",

            // Sub Accounts
            Self::SubAccountCreate => "/v2/sub-user/creation",
            Self::SubAccountList => "/v2/sub-user/user-list",
            Self::SubAccountTransfer => "/v1/subuser/transfer",
            Self::SubAccountBalance => "/v1/account/accounts/{sub-uid}",
        }
    }

    /// Get HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST requests
            Self::PlaceOrder
            | Self::CancelOrder
            | Self::CancelAllOrders
            | Self::CancelOpenOrders
            | Self::Withdraw
            | Self::WithdrawCancel
            | Self::AlgoOrders
            | Self::Transfer
            | Self::SubAccountCreate
            | Self::SubAccountTransfer => "POST",

            // GET requests
            _ => "GET",
        }
    }

    /// Check if endpoint requires authentication
    pub fn is_private(&self) -> bool {
        match self {
            // Public endpoints
            Self::ServerTime
            | Self::Symbols
            | Self::SymbolsV1
            | Self::Ticker
            | Self::Tickers
            | Self::Orderbook
            | Self::Klines
            | Self::RecentTrades
            | Self::HistoryTrades
            | Self::FuturesTicker
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesTrades
            | Self::OpenInterest
            | Self::FundingRateHistory
            | Self::MarkPrice
            | Self::MarkPriceKline => false,

            // Private endpoints
            _ => true,
        }
    }

    /// Replace path variables with actual values
    ///
    /// Example: "/v1/order/orders/{order-id}" -> "/v1/order/orders/12345"
    pub fn path_with_vars(&self, vars: &[(&str, &str)]) -> String {
        let mut path = self.path().to_string();
        for (key, value) in vars {
            let placeholder = format!("{{{}}}", key);
            path = path.replace(&placeholder, value);
        }
        path
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for HTX API
///
/// # Format
/// - Spot: All lowercase, no separator: `btcusdt`
/// - Futures: Uppercase with dash separator: `BTC-USDT`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::htx::format_symbol;
/// use connectors_v5::core::types::{Symbol, AccountType};
///
/// let symbol = Symbol::new("BTC", "USDT");
/// assert_eq!(format_symbol(&symbol, AccountType::Spot), "btcusdt");
/// assert_eq!(format_symbol(&symbol, AccountType::FuturesCross), "BTC-USDT");
/// ```
pub fn format_symbol(symbol: &Symbol, account_type: AccountType) -> String {
    match account_type {
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Futures use uppercase with dash: BTC-USDT
            format!("{}-{}", symbol.base.to_uppercase(), symbol.quote.to_uppercase())
        }
        _ => {
            // Spot uses lowercase concatenated format: btcusdt
            format!("{}{}", symbol.base.to_lowercase(), symbol.quote.to_lowercase())
        }
    }
}

/// Map kline interval to HTX format
///
/// # HTX Interval Format
/// - Minutes: `1min`, `5min`, `15min`, `30min`, `60min`
/// - Hours: `4hour`
/// - Day: `1day`
/// - Week: `1week`
/// - Month: `1mon`
/// - Year: `1year`
///
/// # Examples
/// ```
/// use connectors_v5::exchanges::htx::map_kline_interval;
///
/// assert_eq!(map_kline_interval("1m"), "1min");
/// assert_eq!(map_kline_interval("1h"), "60min");
/// assert_eq!(map_kline_interval("1d"), "1day");
/// ```
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "1h" => "60min",
        "4h" => "4hour",
        "1d" => "1day",
        "1w" => "1week",
        "1M" => "1mon",
        "1y" => "1year",
        _ => "60min", // default to 1 hour
    }
}

/// Get account type string for HTX API
///
/// # Returns
/// - `"spot"` for Spot and Margin
/// - `"margin"` for Margin (when needed to specify)
pub fn account_type_to_string(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "spot",
        AccountType::Margin => "margin",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "futures",
    }
}
