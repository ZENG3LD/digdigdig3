//! # BingX Endpoints
//!
//! URL'ы и endpoint enum для BingX API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для BingX API
///
/// # Testnet / Paper Trading
///
/// BingX has no dedicated testnet URLs. "Testnet" mode uses VST (Virtual Simulated Trading)
/// pairs on the same mainnet endpoints. VST pairs have "-VST" suffix (e.g., BTC-USDT-VST).
/// The testnet bool is stored for future VST pair routing support.
#[derive(Debug, Clone)]
pub struct BingxUrls {
    pub base_rest: &'static str,
}

impl BingxUrls {
    /// Production URL
    pub const MAINNET: Self = Self {
        base_rest: "https://open-api.bingx.com",
    };

    /// Получить REST base URL для account type
    pub fn rest_url(&self, _account_type: AccountType) -> &str {
        // BingX uses same base URL for all account types
        self.base_rest
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// BingX API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BingxEndpoint {
    // === SPOT MARKET DATA ===
    SpotSymbols,
    SpotTrades,
    SpotDepth,
    SpotKlines,
    SpotTicker24hr,
    SpotTickerPrice,
    SpotTickerBookTicker,

    // === SPOT TRADING ===
    SpotOrder,
    SpotOpenOrders,
    SpotHistoryOrders,
    SpotCancelAllOrders,

    // === SPOT ACCOUNT ===
    SpotBalance,
    SpotCommissionRate,

    // === SWAP MARKET DATA ===
    SwapContracts,
    SwapDepth,
    SwapTrades,
    SwapKlines,
    SwapTicker,

    // === SWAP TRADING ===
    SwapOrder,
    SwapOpenOrders,
    SwapAllOrders,
    SwapCancelAllOrders,
    SwapBatchOrders,
    SwapBatchCancelOrders,
    SwapCloseAllPositions,
    SwapAmend,
    SwapFundingRate,

    // === SWAP ACCOUNT ===
    SwapBalance,
    SwapCommissionRate,
    SwapIncome,

    // === SWAP POSITIONS ===
    SwapPositions,
    SwapLeverage,
    SwapMarginType,

    // === ACCOUNT TRANSFERS ===
    InnerTransfer,
    TransferHistory,

    // === CUSTODIAL FUNDS ===
    DepositAddress,
    Withdraw,
    DepositHistory,
    WithdrawHistory,

    // === SUB ACCOUNTS ===
    SubAccountCreate,
    SubAccountList,
    SubAccountTransfer,
    SubAccountAssets,

    // === SPOT TRADE HISTORY ===
    /// GET /openApi/spot/v1/trade/myTrades (signed)
    SpotMyTrades,

    // === SWAP TRADE HISTORY & DERIVATIVES ===
    /// GET /openApi/swap/v2/trade/allFillOrders (signed)
    SwapAllFillOrders,
    /// GET /openApi/swap/v2/trade/fillHistory (signed) — paginated fill history
    SwapFillHistory,
    /// GET /openApi/swap/v2/quote/openInterest
    SwapOpenInterest,
    /// GET /openApi/swap/v2/quote/fundingRateHistory
    SwapFundingRateHistory,
    /// GET /openApi/swap/v2/quote/premiumIndex
    SwapPremiumIndex,
}

impl BingxEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Spot Market Data
            Self::SpotSymbols => "/openApi/spot/v1/common/symbols",
            Self::SpotTrades => "/openApi/spot/v1/market/trades",
            Self::SpotDepth => "/openApi/spot/v1/market/depth",
            Self::SpotKlines => "/openApi/spot/v1/market/kline",
            Self::SpotTicker24hr => "/openApi/spot/v1/ticker/24hr",
            Self::SpotTickerPrice => "/openApi/spot/v1/ticker/price",
            Self::SpotTickerBookTicker => "/openApi/spot/v1/ticker/bookTicker",

            // Spot Trading
            Self::SpotOrder => "/openApi/spot/v1/trade/order",
            Self::SpotOpenOrders => "/openApi/spot/v1/trade/openOrders",
            Self::SpotHistoryOrders => "/openApi/spot/v1/trade/historyOrders",
            Self::SpotCancelAllOrders => "/openApi/spot/v1/trade/cancelAllOrders",

            // Spot Account
            Self::SpotBalance => "/openApi/spot/v1/account/balance",
            Self::SpotCommissionRate => "/openApi/spot/v1/account/commissionRate",

            // Swap Market Data
            Self::SwapContracts => "/openApi/swap/v2/quote/contracts",
            Self::SwapDepth => "/openApi/swap/v2/quote/depth",
            Self::SwapTrades => "/openApi/swap/v2/quote/trades",
            Self::SwapKlines => "/openApi/swap/v2/quote/klines",
            Self::SwapTicker => "/openApi/swap/v2/quote/ticker",

            // Swap Trading
            Self::SwapOrder => "/openApi/swap/v2/trade/order",
            Self::SwapOpenOrders => "/openApi/swap/v2/trade/openOrders",
            Self::SwapAllOrders => "/openApi/swap/v2/trade/allOrders",
            Self::SwapCancelAllOrders => "/openApi/swap/v2/trade/allOpenOrders",
            Self::SwapBatchOrders => "/openApi/swap/v2/trade/batchOrders",
            Self::SwapBatchCancelOrders => "/openApi/swap/v2/trade/batchOrders",
            Self::SwapCloseAllPositions => "/openApi/swap/v2/trade/closeAllPositions",
            Self::SwapAmend => "/openApi/swap/v1/trade/amend",
            Self::SwapFundingRate => "/openApi/swap/v2/quote/fundingRate",

            // Swap Account
            Self::SwapBalance => "/openApi/swap/v2/user/balance",
            Self::SwapCommissionRate => "/openApi/swap/v2/user/commissionRate",
            Self::SwapIncome => "/openApi/swap/v2/user/income",

            // Swap Positions
            Self::SwapPositions => "/openApi/swap/v2/user/positions",
            Self::SwapLeverage => "/openApi/swap/v2/trade/leverage",
            Self::SwapMarginType => "/openApi/swap/v2/trade/marginType",

            // Account Transfers
            Self::InnerTransfer => "/openApi/api/v3/post/account/innerTransfer",
            Self::TransferHistory => "/openApi/api/v3/get/asset/transfer",

            // Custodial Funds
            Self::DepositAddress => "/openApi/wallets/v1/capital/deposit/address",
            Self::Withdraw => "/openApi/wallets/v1/capital/withdraw/apply",
            Self::DepositHistory => "/openApi/api/v3/capital/deposit/hisrec",
            Self::WithdrawHistory => "/openApi/api/v3/capital/withdraw/history",

            // Sub Accounts
            Self::SubAccountCreate => "/openApi/subAccount/v1/create",
            Self::SubAccountList => "/openApi/subAccount/v1/list",
            Self::SubAccountTransfer => "/openApi/subAccount/v1/transfer",
            Self::SubAccountAssets => "/openApi/subAccount/v1/assets",

            // Spot Trade History
            Self::SpotMyTrades => "/openApi/spot/v1/trade/myTrades",

            // Swap Trade History & Derivatives
            Self::SwapAllFillOrders => "/openApi/swap/v2/trade/allFillOrders",
            Self::SwapFillHistory => "/openApi/swap/v2/trade/fillHistory",
            Self::SwapOpenInterest => "/openApi/swap/v2/quote/openInterest",
            Self::SwapFundingRateHistory => "/openApi/swap/v2/quote/fundingRateHistory",
            Self::SwapPremiumIndex => "/openApi/swap/v2/quote/premiumIndex",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::SpotSymbols
            | Self::SpotTrades
            | Self::SpotDepth
            | Self::SpotKlines
            | Self::SpotTicker24hr
            | Self::SpotTickerPrice
            | Self::SpotTickerBookTicker
            | Self::SwapContracts
            | Self::SwapDepth
            | Self::SwapTrades
            | Self::SwapKlines
            | Self::SwapTicker
            | Self::SwapOpenInterest
            | Self::SwapFundingRateHistory
            | Self::SwapPremiumIndex => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::SpotOrder
            | Self::SwapOrder
            | Self::SwapBatchOrders
            | Self::SwapCloseAllPositions
            | Self::SwapLeverage
            | Self::SwapMarginType
            | Self::SwapAmend
            | Self::InnerTransfer
            | Self::Withdraw
            | Self::SubAccountCreate
            | Self::SubAccountTransfer => "POST",

            Self::SpotCancelAllOrders
            | Self::SwapCancelAllOrders
            | Self::SwapBatchCancelOrders => "DELETE",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для BingX
///
/// # Format
/// BingX uses hyphenated format for both Spot and Swap:
/// - Spot: `BTC-USDT`
/// - Swap: `BTC-USDT` (same format)
///
/// # Examples
/// ```ignore
/// format_symbol("BTC", "USDT", AccountType::Spot) => "BTC-USDT"
/// format_symbol("ETH", "USDT", AccountType::FuturesCross) => "ETH-USDT"
/// ```
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    // BingX uses same hyphenated format for both spot and futures
    format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
}

/// Маппинг интервала kline для BingX API
///
/// # Format
/// BingX uses: `1m`, `3m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `6h`, `8h`, `12h`, `1d`, `3d`, `1w`, `1M`
///
/// # Note
/// BingX uses simple format like `1m` for 1 minute, `1h` for 1 hour, `1d` for 1 day
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
        _ => "1h", // default to 1 hour
    }
}
