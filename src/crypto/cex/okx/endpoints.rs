//! # OKX Endpoints
//!
//! URL'ы и endpoint enum для OKX API v5.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для OKX API
#[derive(Debug, Clone)]
pub struct OkxUrls {
    pub rest: &'static str,
    pub ws_public: &'static str,
    pub ws_private: &'static str,
    pub ws_business: &'static str,
}

impl OkxUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest: "https://www.okx.com",
        ws_public: "wss://ws.okx.com:8443/ws/v5/public",
        ws_private: "wss://ws.okx.com:8443/ws/v5/private",
        ws_business: "wss://ws.okx.com:8443/ws/v5/business",
    };

    /// Demo trading URLs (testnet)
    pub const TESTNET: Self = Self {
        rest: "https://www.okx.com", // Same as mainnet, use header x-simulated-trading: 1
        ws_public: "wss://wspap.okx.com:8443/ws/v5/public",
        ws_private: "wss://wspap.okx.com:8443/ws/v5/private",
        ws_business: "wss://wspap.okx.com:8443/ws/v5/business",
    };

    /// Получить REST base URL
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Получить WebSocket URL (public channel)
    pub fn ws_url(&self, private: bool) -> &str {
        if private {
            self.ws_private
        } else {
            self.ws_public
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// OKX API v5 endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OkxEndpoint {
    // === GENERAL ===
    ServerTime,

    // === MARKET DATA ===
    Ticker,
    AllTickers,
    Orderbook,
    OrderbookFull,
    Klines,
    HistoryKlines,
    Trades,
    HistoryTrades,
    Instruments,

    // === TRADING ===
    PlaceOrder,
    PlaceBatchOrders,
    CancelOrder,
    CancelBatchOrders,
    CancelAllAfter,
    AmendOrder,
    GetOrder,
    OpenOrders,
    OrderHistory,
    OrderHistoryArchive,

    // === ALGO TRADING ===
    /// POST /api/v5/trade/order-algo
    /// For conditional (stop), move_order_stop (trailing), oco, twap, iceberg orders.
    AlgoOrder,
    /// POST /api/v5/trade/cancel-algos
    /// Cancel algo orders by algoId — different endpoint from regular cancel.
    AlgoOrderCancel,
    /// GET /api/v5/trade/orders-algo-pending
    /// Fetch open algo orders.
    AlgoOpenOrders,

    // === ACCOUNT ===
    Balance,
    AssetBalances,
    AccountConfig,
    Positions,
    PositionHistory,
    MaxOrderSize,
    SetLeverage,
    GetLeverage,
    SetPositionMode,

    // === FUNDING ===
    FundingRate,
    FundingRateHistory,

    // === ASSET TRANSFERS ===
    /// POST /api/v5/asset/transfer
    AssetTransfer,
    /// GET /api/v5/asset/transfer-state
    TransferState,
    /// GET /api/v5/asset/bills
    AssetBills,

    // === CUSTODIAL FUNDS ===
    /// GET /api/v5/asset/deposit-address
    DepositAddress,
    /// POST /api/v5/asset/withdrawal
    Withdrawal,
    /// GET /api/v5/asset/deposit-history
    DepositHistory,
    /// GET /api/v5/asset/withdrawal-history
    WithdrawalHistory,

    // === SUB-ACCOUNTS ===
    /// POST /api/v5/users/subaccount/create
    SubAccountCreate,
    /// GET /api/v5/users/subaccount/list
    SubAccountList,
    /// POST /api/v5/asset/subaccount/transfer
    SubAccountTransfer,
    /// GET /api/v5/account/subaccount/balances
    SubAccountBalances,
}

impl OkxEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // General
            Self::ServerTime => "/api/v5/public/time",

            // Market Data
            Self::Ticker => "/api/v5/market/ticker",
            Self::AllTickers => "/api/v5/market/tickers",
            Self::Orderbook => "/api/v5/market/books",
            Self::OrderbookFull => "/api/v5/market/books-full",
            Self::Klines => "/api/v5/market/candles",
            Self::HistoryKlines => "/api/v5/market/history-candles",
            Self::Trades => "/api/v5/market/trades",
            Self::HistoryTrades => "/api/v5/market/history-trades",
            Self::Instruments => "/api/v5/public/instruments",

            // Trading
            Self::PlaceOrder => "/api/v5/trade/order",
            Self::PlaceBatchOrders => "/api/v5/trade/batch-orders",
            Self::CancelOrder => "/api/v5/trade/cancel-order",
            Self::CancelBatchOrders => "/api/v5/trade/cancel-batch-orders",
            Self::CancelAllAfter => "/api/v5/trade/cancel-all-after",
            Self::AmendOrder => "/api/v5/trade/amend-order",
            Self::GetOrder => "/api/v5/trade/order",
            Self::OpenOrders => "/api/v5/trade/orders-pending",
            Self::OrderHistory => "/api/v5/trade/orders-history",
            Self::OrderHistoryArchive => "/api/v5/trade/orders-history-archive",

            // Account
            Self::Balance => "/api/v5/account/balance",
            Self::AssetBalances => "/api/v5/asset/balances",
            Self::AccountConfig => "/api/v5/account/config",
            Self::Positions => "/api/v5/account/positions",
            Self::PositionHistory => "/api/v5/account/positions-history",
            Self::MaxOrderSize => "/api/v5/account/max-size",
            Self::SetLeverage => "/api/v5/account/set-leverage",
            Self::GetLeverage => "/api/v5/account/leverage-info",
            Self::SetPositionMode => "/api/v5/account/set-position-mode",

            // Funding
            Self::FundingRate => "/api/v5/public/funding-rate",
            Self::FundingRateHistory => "/api/v5/public/funding-rate-history",

            // Algo Trading
            Self::AlgoOrder => "/api/v5/trade/order-algo",
            Self::AlgoOrderCancel => "/api/v5/trade/cancel-algos",
            Self::AlgoOpenOrders => "/api/v5/trade/orders-algo-pending",

            // Asset Transfers
            Self::AssetTransfer => "/api/v5/asset/transfer",
            Self::TransferState => "/api/v5/asset/transfer-state",
            Self::AssetBills => "/api/v5/asset/bills",

            // Custodial Funds
            Self::DepositAddress => "/api/v5/asset/deposit-address",
            Self::Withdrawal => "/api/v5/asset/withdrawal",
            Self::DepositHistory => "/api/v5/asset/deposit-history",
            Self::WithdrawalHistory => "/api/v5/asset/withdrawal-history",

            // Sub-accounts
            Self::SubAccountCreate => "/api/v5/users/subaccount/create",
            Self::SubAccountList => "/api/v5/users/subaccount/list",
            Self::SubAccountTransfer => "/api/v5/asset/subaccount/transfer",
            Self::SubAccountBalances => "/api/v5/account/subaccount/balances",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::ServerTime
            | Self::Ticker
            | Self::AllTickers
            | Self::Orderbook
            | Self::OrderbookFull
            | Self::Klines
            | Self::HistoryKlines
            | Self::Trades
            | Self::HistoryTrades
            | Self::Instruments
            | Self::FundingRate
            | Self::FundingRateHistory => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::PlaceOrder
            | Self::PlaceBatchOrders
            | Self::CancelOrder
            | Self::CancelBatchOrders
            | Self::CancelAllAfter
            | Self::AmendOrder
            | Self::SetLeverage
            | Self::SetPositionMode
            | Self::AlgoOrder
            | Self::AlgoOrderCancel
            | Self::AssetTransfer
            | Self::Withdrawal
            | Self::SubAccountCreate
            | Self::SubAccountTransfer => "POST",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для OKX
///
/// # Symbol Format
/// - Spot: `BTC-USDT`, `ETH-BTC`
/// - Futures (SWAP): `BTC-USDT-SWAP`, `ETH-USDT-SWAP`
/// - Futures (dated): `BTC-USDT-240329` (quarterly)
///
/// # Examples
/// - Spot: `BTC-USDT`
/// - Perpetual Futures: `BTC-USDT-SWAP`
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot/Margin: BASE-QUOTE
            format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Futures: BASE-QUOTE-SWAP (perpetual)
            format!("{}-{}-SWAP", base.to_uppercase(), quote.to_uppercase())
        }
    }
}

/// Маппинг интервала kline для OKX API
///
/// # OKX Bar Format
/// - Minutes: `1m`, `3m`, `5m`, `15m`, `30m`
/// - Hours (Hong Kong): `1H`, `2H`, `4H`, `6H`, `12H`
/// - Hours (UTC): `6Hutc`, `12Hutc`
/// - Days/Weeks/Months (Hong Kong): `1D`, `1W`, `1M`, `3M`, `6M`, `1Y`
/// - Days/Weeks/Months (UTC): `1Dutc`, `1Wutc`, `1Mutc`, `3Mutc`, `6Mutc`, `1Yutc`
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "3m" => "3m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1H",
        "2h" => "2H",
        "4h" => "4H",
        "6h" => "6H",
        "12h" => "12H",
        "1d" => "1D",
        "1w" => "1W",
        "1M" => "1M",
        "3M" => "3M",
        "6M" => "6M",
        "1y" => "1Y",
        _ => "1H", // default 1 hour
    }
}

/// Получить instType для OKX API
pub fn get_inst_type(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "SPOT",
        AccountType::Margin => "MARGIN",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "SWAP",
    }
}

/// Получить trade mode для OKX API
pub fn get_trade_mode(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "cash",
        AccountType::Margin => "cross",
        AccountType::FuturesCross => "cross",
        AccountType::FuturesIsolated => "isolated",
    }
}

/// Map AccountType to OKX account ID for asset transfer endpoint.
///
/// OKX account IDs:
/// - 1  = Spot (Classic Account)
/// - 5  = Margin
/// - 6  = Funding (Asset)
/// - 18 = Unified Trading Account (SWAP / Futures)
///
/// Spot maps to 6 (Funding) as source when transferring from spot wallet,
/// or to 1 (Spot) when referring to spot trading account.
/// For simplicity: Spot → 6 (funding/asset), Margin → 5, Futures → 18.
pub fn get_account_id(account_type: AccountType) -> &'static str {
    match account_type {
        AccountType::Spot => "6",
        AccountType::Margin => "5",
        AccountType::FuturesCross | AccountType::FuturesIsolated => "18",
    }
}
