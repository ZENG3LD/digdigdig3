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
    AmendOrder,
    GetOrder,
    OpenOrders,
    OrderHistory,
    OrderHistoryArchive,

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
            | Self::AmendOrder
            | Self::SetLeverage
            | Self::SetPositionMode => "POST",

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
