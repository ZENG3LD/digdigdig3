//! # Binance Endpoints
//!
//! URL'ы и endpoint enum для Binance API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Binance API
#[derive(Debug, Clone)]
pub struct BinanceUrls {
    pub spot_rest: &'static str,
    pub futures_rest: &'static str,
    pub spot_ws: &'static str,
    pub futures_ws: &'static str,
}

impl BinanceUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        spot_rest: "https://api.binance.com",
        futures_rest: "https://fapi.binance.com",
        spot_ws: "wss://stream.binance.com:9443",
        futures_ws: "wss://fstream.binance.com",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        spot_rest: "https://testapi.binance.vision",
        futures_rest: "https://testnet.binancefuture.com",
        spot_ws: "wss://testnet.binance.vision",
        futures_ws: "wss://stream.binancefuture.com",
    };

    /// Получить REST base URL для account type
    pub fn rest_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_rest,
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_rest,
        }
    }

    /// Получить WebSocket URL для account type
    pub fn ws_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_ws,
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_ws,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Binance API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BinanceEndpoint {
    // === ОБЩИЕ ===
    Ping,
    ServerTime,

    // === SPOT MARKET DATA ===
    SpotPrice,
    SpotOrderbook,
    SpotKlines,
    SpotTicker,
    SpotExchangeInfo,

    // === SPOT TRADING ===
    SpotCreateOrder,
    SpotCancelOrder,
    SpotCancelAllOrders,
    SpotGetOrder,
    SpotOpenOrders,
    SpotAllOrders,
    SpotOcoOrder,
    /// OTOCO order list (One-Triggers-a-One-Cancels-the-Other) — Spot Bracket
    SpotOtocoOrder,
    SpotTradeFee,

    // === SPOT ALGO ===
    /// TWAP algo order for Spot: POST /sapi/v1/algo/spot/newOrderTwap
    SpotAlgoTwap,

    // === SPOT ACCOUNT ===
    SpotAccount,

    // === FUTURES MARKET DATA ===
    FuturesPrice,
    FuturesOrderbook,
    FuturesKlines,
    FuturesTicker,
    FuturesExchangeInfo,
    FundingRate,

    // === FUTURES TRADING ===
    FuturesCreateOrder,
    FuturesCancelOrder,
    FuturesCancelAllOrders,
    FuturesGetOrder,
    FuturesOpenOrders,
    FuturesAllOrders,
    FuturesAmendOrder,
    FuturesBatchOrders,
    /// Batch amend multiple futures orders: PATCH /fapi/v1/batchOrders
    FuturesBatchAmend,
    /// Futures conditional/algo orders (post-2025-12-09 migration endpoint)
    /// STOP, STOP_MARKET, TAKE_PROFIT, TAKE_PROFIT_MARKET, TRAILING_STOP_MARKET
    FuturesAlgoOrder,

    // === FUTURES ALGO ===
    /// TWAP algo order for Futures: POST /sapi/v1/algo/futures/newOrderTwap
    FuturesAlgoTwap,

    // === FUTURES ACCOUNT ===
    FuturesAccount,
    FuturesPositions,
    FuturesSetLeverage,
    FuturesSetMarginType,
    FuturesPositionMargin,
    FuturesCommissionRate,

    // === WEBSOCKET ===
    SpotListenKey,
    FuturesListenKey,

    // === MARKET DATA EXTENSIONS ===
    /// GET /api/v3/trades — recent spot trades
    SpotRecentTrades,
    /// GET /api/v3/historicalTrades — historical spot trades (signed)
    SpotHistoricalTrades,
    /// GET /api/v3/avgPrice — current average price
    SpotAvgPrice,
    /// GET /api/v3/ticker/bookTicker — best bid/ask
    SpotBookTicker,
    /// GET /fapi/v1/trades — recent futures trades
    FuturesRecentTrades,
    /// GET /fapi/v1/openInterest — open interest
    FuturesOpenInterest,
    /// GET /futures/data/openInterestHist — open interest history
    FuturesOpenInterestHist,
    /// GET /fapi/v1/premiumIndex — mark price and funding rate
    FuturesPremiumIndex,
    /// GET /futures/data/topLongShortAccountRatio — long/short ratio
    FuturesLongShortRatio,

    // === FILL/TRADE HISTORY ===
    /// GET /api/v3/myTrades — spot trade fills (signed)
    SpotMyTrades,
    /// GET /fapi/v1/userTrades — futures trade fills (signed)
    FuturesMyTrades,
    /// GET /fapi/v1/income — futures income history (signed)
    FuturesIncomeHistory,

    // === LISTEN KEY MANAGEMENT ===
    /// PUT /api/v3/userDataStream — keepalive spot listen key
    ListenKeyKeepAlive,
    /// DELETE /api/v3/userDataStream — close spot listen key
    ListenKeyClose,

    // === ACCOUNT TRANSFERS ===
    /// Universal transfer: POST /sapi/v1/asset/transfer
    AssetTransfer,
    /// Universal transfer history: GET /sapi/v1/asset/transfer
    AssetTransferHistory,

    // === CUSTODIAL FUNDS ===
    /// Deposit address: GET /sapi/v1/capital/deposit/address
    DepositAddress,
    /// Withdraw: POST /sapi/v1/capital/withdraw/apply
    Withdraw,
    /// Deposit history: GET /sapi/v1/capital/deposit/hisrec
    DepositHistory,
    /// Withdrawal history: GET /sapi/v1/capital/withdraw/history
    WithdrawHistory,

    // === SUB-ACCOUNTS ===
    /// Create virtual sub-account: POST /sapi/v1/sub-account/virtualSubAccount
    SubAccountCreate,
    /// List sub-accounts: GET /sapi/v1/sub-account/list
    SubAccountList,
    /// Universal transfer between sub-accounts: POST /sapi/v1/sub-account/universalTransfer
    SubAccountTransfer,
    /// Get sub-account assets/balance: GET /sapi/v3/sub-account/assets
    SubAccountAssets,
}

impl BinanceEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Общие
            Self::Ping => "/api/v3/ping",
            Self::ServerTime => "/api/v3/time",

            // Spot Market Data
            Self::SpotPrice => "/api/v3/ticker/price",
            Self::SpotOrderbook => "/api/v3/depth",
            Self::SpotKlines => "/api/v3/klines",
            Self::SpotTicker => "/api/v3/ticker/24hr",
            Self::SpotExchangeInfo => "/api/v3/exchangeInfo",

            // Spot Trading
            Self::SpotCreateOrder => "/api/v3/order",
            Self::SpotCancelOrder => "/api/v3/order",
            Self::SpotCancelAllOrders => "/api/v3/openOrders",
            Self::SpotGetOrder => "/api/v3/order",
            Self::SpotOpenOrders => "/api/v3/openOrders",
            Self::SpotAllOrders => "/api/v3/allOrders",
            Self::SpotOcoOrder => "/api/v3/orderList/oco",
            Self::SpotOtocoOrder => "/api/v3/orderList/otoco",
            Self::SpotTradeFee => "/sapi/v1/asset/tradeFee",

            // Spot Algo
            Self::SpotAlgoTwap => "/sapi/v1/algo/spot/newOrderTwap",

            // Spot Account
            Self::SpotAccount => "/api/v3/account",

            // Futures Market Data
            Self::FuturesPrice => "/fapi/v1/ticker/price",
            Self::FuturesOrderbook => "/fapi/v1/depth",
            Self::FuturesKlines => "/fapi/v1/klines",
            Self::FuturesTicker => "/fapi/v1/ticker/24hr",
            Self::FuturesExchangeInfo => "/fapi/v1/exchangeInfo",
            Self::FundingRate => "/fapi/v1/fundingRate",

            // Futures Trading
            Self::FuturesCreateOrder => "/fapi/v1/order",
            Self::FuturesCancelOrder => "/fapi/v1/order",
            Self::FuturesCancelAllOrders => "/fapi/v1/allOpenOrders",
            Self::FuturesGetOrder => "/fapi/v1/order",
            Self::FuturesOpenOrders => "/fapi/v1/openOrders",
            Self::FuturesAllOrders => "/fapi/v1/allOrders",
            Self::FuturesAmendOrder => "/fapi/v1/order",
            Self::FuturesBatchOrders => "/fapi/v1/batchOrders",
            Self::FuturesBatchAmend => "/fapi/v1/batchOrders",
            Self::FuturesAlgoOrder => "/fapi/v1/order/algo",

            // Futures Algo
            Self::FuturesAlgoTwap => "/sapi/v1/algo/futures/newOrderTwap",

            // Futures Account
            Self::FuturesAccount => "/fapi/v3/account",
            Self::FuturesPositions => "/fapi/v2/positionRisk",
            Self::FuturesSetLeverage => "/fapi/v1/leverage",
            Self::FuturesSetMarginType => "/fapi/v1/marginType",
            Self::FuturesPositionMargin => "/fapi/v1/positionMargin",
            Self::FuturesCommissionRate => "/fapi/v1/commissionRate",

            // WebSocket
            Self::SpotListenKey => "/api/v3/userDataStream",
            Self::FuturesListenKey => "/fapi/v1/listenKey",

            // Market Data Extensions
            Self::SpotRecentTrades => "/api/v3/trades",
            Self::SpotHistoricalTrades => "/api/v3/historicalTrades",
            Self::SpotAvgPrice => "/api/v3/avgPrice",
            Self::SpotBookTicker => "/api/v3/ticker/bookTicker",
            Self::FuturesRecentTrades => "/fapi/v1/trades",
            Self::FuturesOpenInterest => "/fapi/v1/openInterest",
            Self::FuturesOpenInterestHist => "/futures/data/openInterestHist",
            Self::FuturesPremiumIndex => "/fapi/v1/premiumIndex",
            Self::FuturesLongShortRatio => "/futures/data/topLongShortAccountRatio",

            // Fill/Trade History
            Self::SpotMyTrades => "/api/v3/myTrades",
            Self::FuturesMyTrades => "/fapi/v1/userTrades",
            Self::FuturesIncomeHistory => "/fapi/v1/income",

            // Listen Key Management
            Self::ListenKeyKeepAlive => "/api/v3/userDataStream",
            Self::ListenKeyClose => "/api/v3/userDataStream",

            // Account Transfers
            Self::AssetTransfer => "/sapi/v1/asset/transfer",
            Self::AssetTransferHistory => "/sapi/v1/asset/transfer",

            // Custodial Funds
            Self::DepositAddress => "/sapi/v1/capital/deposit/address",
            Self::Withdraw => "/sapi/v1/capital/withdraw/apply",
            Self::DepositHistory => "/sapi/v1/capital/deposit/hisrec",
            Self::WithdrawHistory => "/sapi/v1/capital/withdraw/history",

            // Sub-Accounts
            Self::SubAccountCreate => "/sapi/v1/sub-account/virtualSubAccount",
            Self::SubAccountList => "/sapi/v1/sub-account/list",
            Self::SubAccountTransfer => "/sapi/v1/sub-account/universalTransfer",
            Self::SubAccountAssets => "/sapi/v3/sub-account/assets",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::Ping
            | Self::ServerTime
            | Self::SpotPrice
            | Self::SpotOrderbook
            | Self::SpotKlines
            | Self::SpotTicker
            | Self::SpotExchangeInfo
            | Self::FuturesPrice
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesTicker
            | Self::FuturesExchangeInfo
            | Self::FundingRate
            | Self::SpotRecentTrades
            | Self::SpotAvgPrice
            | Self::SpotBookTicker
            | Self::FuturesRecentTrades
            | Self::FuturesOpenInterest
            | Self::FuturesOpenInterestHist
            | Self::FuturesPremiumIndex
            | Self::FuturesLongShortRatio => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::SpotCreateOrder
            | Self::FuturesCreateOrder
            | Self::FuturesSetLeverage
            | Self::FuturesSetMarginType
            | Self::FuturesPositionMargin
            | Self::SpotOcoOrder
            | Self::SpotOtocoOrder
            | Self::SpotAlgoTwap
            | Self::FuturesAlgoOrder
            | Self::FuturesAlgoTwap
            | Self::FuturesBatchOrders
            | Self::SpotListenKey
            | Self::FuturesListenKey
            | Self::AssetTransfer
            | Self::Withdraw
            | Self::SubAccountCreate
            | Self::SubAccountTransfer => "POST",

            Self::SpotCancelOrder
            | Self::SpotCancelAllOrders
            | Self::FuturesCancelOrder
            | Self::FuturesCancelAllOrders
            | Self::ListenKeyClose => "DELETE",

            Self::FuturesAmendOrder | Self::ListenKeyKeepAlive => "PUT",

            Self::FuturesBatchAmend => "PATCH",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Binance
///
/// # Symbol Format
/// - Spot: `BTCUSDT`, `ETHBTC` (no separator)
/// - Futures USDT-M: `BTCUSDT` (same as spot, perpetual contracts)
///
/// # Examples
/// - Spot: `BTCUSDT`, `ETHUSDT`
/// - Futures: `BTCUSDT`, `ETHUSDT`
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    // Binance uses same format for both spot and futures USDT-M
    // No separator, just concatenation
    format!("{}{}", base.to_uppercase(), quote.to_uppercase())
}

/// Маппинг интервала kline для Binance API
///
/// # API Format
/// Parameter: `interval` (string)
/// Values: `"1m"`, `"1h"`, `"1d"`, etc.
///
/// # Supported Intervals
/// - Minutes: 1m, 3m, 5m, 15m, 30m
/// - Hours: 1h, 2h, 4h, 6h, 8h, 12h
/// - Days: 1d, 3d
/// - Week: 1w
/// - Month: 1M
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "3m" => "3m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1h",
        "2h" => "2h",
        "4h" => "4h",
        "6h" => "6h",
        "8h" => "8h",
        "12h" => "12h",
        "1d" => "1d",
        "3d" => "3d",
        "1w" => "1w",
        "1M" => "1M",
        _ => "1h", // default
    }
}
