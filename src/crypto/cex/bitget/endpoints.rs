//! # Bitget Endpoints
//!
//! URL'ы и endpoint enum для Bitget API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Bitget API
#[derive(Debug, Clone)]
pub struct BitgetUrls {
    pub spot_rest: &'static str,
    pub futures_rest: &'static str,
    pub ws_public: &'static str,
    pub ws_private: &'static str,
}

impl BitgetUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        spot_rest: "https://api.bitget.com",
        futures_rest: "https://api.bitget.com",
        ws_public: "wss://ws.bitget.com/v2/ws/public",
        ws_private: "wss://ws.bitget.com/v2/ws/private",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        spot_rest: "https://api.bitget.com",
        futures_rest: "https://api.bitget.com",
        ws_public: "wss://wspap.bitget.com/v2/ws/public",
        ws_private: "wss://wspap.bitget.com/v2/ws/private",
    };

    /// Получить REST base URL для account type
    pub fn rest_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_rest,
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_rest,
        }
    }

    /// Получить WebSocket public URL
    pub fn ws_public_url(&self) -> String {
        self.ws_public.to_string()
    }

    /// Получить WebSocket private URL
    pub fn ws_private_url(&self) -> String {
        self.ws_private.to_string()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitget API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitgetEndpoint {
    // === ОБЩИЕ ===
    Timestamp,

    // === SPOT MARKET DATA ===
    SpotPrice,
    SpotOrderbook,
    SpotKlines,
    SpotTicker,
    SpotAllTickers,
    SpotSymbols,

    // === SPOT TRADING ===
    SpotCreateOrder,
    SpotCancelOrder,
    SpotGetOrder,
    SpotOpenOrders,
    SpotAllOrders,

    // === SPOT ACCOUNT ===
    SpotAccounts,
    SpotAccountInfo,

    // === FUTURES MARKET DATA ===
    FuturesPrice,
    FuturesOrderbook,
    FuturesKlines,
    FuturesTicker,
    FuturesAllTickers,
    FuturesContracts,
    FundingRate,

    // === FUTURES TRADING ===
    FuturesCreateOrder,
    FuturesCancelOrder,
    FuturesGetOrder,
    FuturesOpenOrders,
    FuturesAllOrders,

    // === FUTURES ACCOUNT ===
    FuturesAccount,
    FuturesAllAccounts,
    FuturesPositions,
    FuturesPosition,
    FuturesSetLeverage,
}

impl BitgetEndpoint {
    /// Получить путь endpoint'а (V2 API)
    pub fn path(&self) -> &'static str {
        match self {
            // Общие
            Self::Timestamp => "/api/v2/public/time",

            // Spot Market Data
            Self::SpotPrice => "/api/v2/spot/market/tickers",
            Self::SpotOrderbook => "/api/v2/spot/market/orderbook",
            Self::SpotKlines => "/api/v2/spot/market/candles",
            Self::SpotTicker => "/api/v2/spot/market/tickers",
            Self::SpotAllTickers => "/api/v2/spot/market/tickers",
            Self::SpotSymbols => "/api/v2/spot/public/symbols",

            // Spot Trading
            Self::SpotCreateOrder => "/api/v2/spot/trade/place-order",
            Self::SpotCancelOrder => "/api/v2/spot/trade/cancel-order",
            Self::SpotGetOrder => "/api/v2/spot/trade/orderInfo",
            Self::SpotOpenOrders => "/api/v2/spot/trade/unfilled-orders",
            Self::SpotAllOrders => "/api/v2/spot/trade/history-orders",

            // Spot Account
            Self::SpotAccounts => "/api/v2/spot/account/assets",
            Self::SpotAccountInfo => "/api/v2/spot/account/info",

            // Futures Market Data
            Self::FuturesPrice => "/api/v2/mix/market/ticker",
            Self::FuturesOrderbook => "/api/v2/mix/market/merge-depth",
            Self::FuturesKlines => "/api/v2/mix/market/candles",
            Self::FuturesTicker => "/api/v2/mix/market/ticker",
            Self::FuturesAllTickers => "/api/v2/mix/market/tickers",
            Self::FuturesContracts => "/api/v2/mix/market/contracts",
            Self::FundingRate => "/api/v2/mix/market/current-fund-rate",

            // Futures Trading
            Self::FuturesCreateOrder => "/api/v2/mix/order/place-order",
            Self::FuturesCancelOrder => "/api/v2/mix/order/cancel-order",
            Self::FuturesGetOrder => "/api/v2/mix/order/detail",
            Self::FuturesOpenOrders => "/api/v2/mix/order/orders-pending",
            Self::FuturesAllOrders => "/api/v2/mix/order/orders-history",

            // Futures Account
            Self::FuturesAccount => "/api/v2/mix/account/account",
            Self::FuturesAllAccounts => "/api/v2/mix/account/accounts",
            Self::FuturesPositions => "/api/v2/mix/position/all-position",
            Self::FuturesPosition => "/api/v2/mix/position/single-position",
            Self::FuturesSetLeverage => "/api/v2/mix/account/set-leverage",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::Timestamp
            | Self::SpotPrice
            | Self::SpotOrderbook
            | Self::SpotKlines
            | Self::SpotTicker
            | Self::SpotAllTickers
            | Self::SpotSymbols
            | Self::FuturesPrice
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesTicker
            | Self::FuturesAllTickers
            | Self::FuturesContracts
            | Self::FundingRate => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::SpotCreateOrder
            | Self::SpotCancelOrder
            | Self::FuturesCreateOrder
            | Self::FuturesCancelOrder
            | Self::FuturesSetLeverage => "POST",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Bitget V2 API
///
/// # V2 Symbol Format (SIMPLIFIED)
/// V2 API uses plain symbol format without suffixes.
///
/// # Format
/// - All account types: `{BASE}{QUOTE}`
/// - Examples: `BTCUSDT`, `ETHUSDT`, `BTCUSD`
///
/// # Changes from V1
/// - V1: `BTCUSDT_SPBL` (spot), `BTCUSDT_UMCBL` (futures)
/// - V2: `BTCUSDT` (all types)
///
/// # Examples
/// - Spot: `BTCUSDT`, `ETHBTC`
/// - USDT Futures: `BTCUSDT`, `ETHUSDT`
/// - Coin Futures: `BTCUSD`, `ETHUSD`
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    let base = base.to_uppercase();
    let quote = quote.to_uppercase();

    // V2 uses plain format for all account types
    format!("{}{}", base, quote)
}

/// Получить productType для futures API (V2 format)
///
/// # V2 Product Types
/// V2 API uses uppercase format with dashes:
/// - `USDT-FUTURES` - USDT-margined perpetual
/// - `COIN-FUTURES` - Coin-margined perpetual
/// - `USDC-FUTURES` - USDC-margined perpetual
///
/// # Changes from V1
/// - V1: `umcbl`, `dmcbl`, `cmcbl` (lowercase, no dashes)
/// - V2: `USDT-FUTURES`, `COIN-FUTURES`, `USDC-FUTURES` (uppercase with dashes)
pub fn get_product_type(quote: &str) -> &'static str {
    match quote.to_uppercase().as_str() {
        "USDT" => "USDT-FUTURES",
        "USD" => "COIN-FUTURES",
        "USDC" => "USDC-FUTURES",
        _ => "USDT-FUTURES", // Default to USDT-margined
    }
}

/// Маппинг интервала kline для Spot API
///
/// # Spot API Format
/// Parameter: `period` (string)
/// Values: `"1min"`, `"1h"`, `"1day"`, etc.
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1min",
        "3m" => "3min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "1h" => "1h",
        "2h" => "2h",
        "4h" => "4h",
        "6h" => "6h",
        "12h" => "12h",
        "1d" => "1day",
        "1w" => "1week",
        "1M" => "1M",
        _ => "1h",
    }
}

/// Map kline interval to Futures granularity
///
/// # Futures API Format
/// Parameter: `granularity` (string)
/// Values: `"1m"`, `"1H"`, `"1D"`, etc.
///
/// # Differences from Spot
/// - Futures uses uppercase H, D, W, M
/// - Spot uses lowercase h, day, week
pub fn map_futures_granularity(interval: &str) -> &'static str {
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
        "3d" => "3D",
        "1w" => "1W",
        "1M" => "1M",
        _ => "1H", // default 1 hour
    }
}
