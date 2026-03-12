//! # BingX Endpoints
//!
//! URL'ы и endpoint enum для BingX API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для BingX API
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
            | Self::SwapTicker => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::SpotOrder
            | Self::SwapOrder
            | Self::SwapCloseAllPositions
            | Self::SwapLeverage
            | Self::SwapMarginType
            | Self::SwapAmend => "POST",

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
