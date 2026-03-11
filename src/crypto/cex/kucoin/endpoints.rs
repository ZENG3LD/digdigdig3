//! # KuCoin Endpoints
//!
//! URL'ы и endpoint enum для KuCoin API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для KuCoin API
#[derive(Debug, Clone)]
pub struct KuCoinUrls {
    pub spot_rest: &'static str,
    pub futures_rest: &'static str,
    pub spot_ws: &'static str,
    pub futures_ws: &'static str,
}

impl KuCoinUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        spot_rest: "https://api.kucoin.com",
        futures_rest: "https://api-futures.kucoin.com",
        spot_ws: "wss://ws-api-spot.kucoin.com",
        futures_ws: "wss://ws-api-futures.kucoin.com",
    };

    /// Sandbox URLs
    pub const TESTNET: Self = Self {
        spot_rest: "https://openapi-sandbox.kucoin.com",
        futures_rest: "https://api-sandbox-futures.kucoin.com",
        spot_ws: "wss://ws-api-sandbox.kucoin.com",
        futures_ws: "wss://ws-api-sandbox-futures.kucoin.com",
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

/// KuCoin API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KuCoinEndpoint {
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
    SpotCancelAllOrders,

    // === SPOT ACCOUNT ===
    SpotAccounts,
    SpotAccountDetail,

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
    FuturesCancelAllOrders,

    // === FUTURES ACCOUNT ===
    FuturesAccount,
    FuturesPositions,
    FuturesPosition,
    FuturesSetLeverage,

    // === WEBSOCKET ===
    WsPublicToken,
    WsPrivateToken,
}

impl KuCoinEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Общие
            Self::Timestamp => "/api/v1/timestamp",

            // Spot Market Data
            Self::SpotPrice => "/api/v1/market/orderbook/level1",
            Self::SpotOrderbook => "/api/v1/market/orderbook/level2_100",
            Self::SpotKlines => "/api/v1/market/candles",
            Self::SpotTicker => "/api/v1/market/stats",
            Self::SpotAllTickers => "/api/v1/market/allTickers",
            Self::SpotSymbols => "/api/v2/symbols",

            // Spot Trading
            Self::SpotCreateOrder => "/api/v1/orders",
            Self::SpotCancelOrder => "/api/v1/orders/{orderId}",
            Self::SpotGetOrder => "/api/v1/orders/{orderId}",
            Self::SpotOpenOrders => "/api/v1/orders",
            Self::SpotAllOrders => "/api/v1/orders",
            Self::SpotCancelAllOrders => "/api/v1/orders",

            // Spot Account
            Self::SpotAccounts => "/api/v1/accounts",
            Self::SpotAccountDetail => "/api/v1/accounts/{accountId}",

            // Futures Market Data
            Self::FuturesPrice => "/api/v1/ticker",
            Self::FuturesOrderbook => "/api/v1/level2/depth100",
            Self::FuturesKlines => "/api/v1/kline/query",
            Self::FuturesTicker => "/api/v1/ticker",
            Self::FuturesAllTickers => "/api/v1/allTickers",
            Self::FuturesContracts => "/api/v1/contracts/active",
            Self::FundingRate => "/api/v1/funding-rate/{symbol}/current",

            // Futures Trading
            Self::FuturesCreateOrder => "/api/v1/orders",
            Self::FuturesCancelOrder => "/api/v1/orders/{orderId}",
            Self::FuturesGetOrder => "/api/v1/orders/{orderId}",
            Self::FuturesOpenOrders => "/api/v1/orders",
            Self::FuturesAllOrders => "/api/v1/orders",
            Self::FuturesCancelAllOrders => "/api/v1/orders",

            // Futures Account
            Self::FuturesAccount => "/api/v1/account-overview",
            Self::FuturesPositions => "/api/v1/positions",
            Self::FuturesPosition => "/api/v1/position",
            Self::FuturesSetLeverage => "/api/v1/position/risk-limit-level/change",

            // WebSocket
            Self::WsPublicToken => "/api/v1/bullet-public",
            Self::WsPrivateToken => "/api/v1/bullet-private",
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
            | Self::FundingRate
            | Self::WsPublicToken => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::SpotCreateOrder
            | Self::FuturesCreateOrder
            | Self::WsPublicToken
            | Self::WsPrivateToken => "POST",

            Self::SpotCancelOrder
            | Self::SpotCancelAllOrders
            | Self::FuturesCancelOrder
            | Self::FuturesCancelAllOrders => "DELETE",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для KuCoin
///
/// # Futures Symbol Format
/// - USDT-margined perpetuals: `XBTUSDTM` (mini contracts, linear)
/// - USD-margined perpetuals: `XBTUSDM` (inverse, coin-margined)
/// - BTC → XBT mapping only applies to futures contracts
///
/// # Examples
/// - Spot: `BTC-USDT`, `ETH-BTC`
/// - USDT Futures: `XBTUSDTM`, `ETHUSDTM`
/// - USD Futures: `XBTUSDM`, `ETHUSDM`
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot: BASE-QUOTE with hyphen
            format!("{}-{}", base, quote)
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // BTC → XBT mapping for futures contracts (ISO 4217 standard)
            let base = if base.to_uppercase() == "BTC" { "XBT" } else { base };

            // Distinguish between USDT-margined and USD-margined futures
            // CRITICAL: USDT-margined uses "USDTM", USD-margined uses "USDM"
            match quote.to_uppercase().as_str() {
                "USDT" => format!("{}USDTM", base),  // USDT-margined perpetual (mini, linear)
                "USD" => format!("{}USDM", base),    // USD-margined perpetual (inverse, coin-margined)
                _ => format!("{}{}M", base, quote.to_uppercase()), // Generic fallback
            }
        }
    }
}

/// Маппинг интервала kline для Spot API (возвращает строки)
///
/// # Spot API Format
/// Parameter: `type` (string)
/// Values: `"1min"`, `"1hour"`, `"1day"`, etc.
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1min",
        "3m" => "3min",
        "5m" => "5min",
        "15m" => "15min",
        "30m" => "30min",
        "1h" => "1hour",
        "2h" => "2hour",
        "4h" => "4hour",
        "6h" => "6hour",
        "8h" => "8hour",
        "12h" => "12hour",
        "1d" => "1day",
        "1w" => "1week",
        "1M" => "1month",
        _ => "1hour",
    }
}

/// Map kline interval to Futures granularity (minutes as integer)
///
/// # Futures API Format
/// Parameter: `granularity` (integer representing minutes)
/// Values: `1`, `60`, `1440`, etc.
///
/// # Differences from Spot
/// - Futures uses integer minutes instead of strings
/// - No 3-minute or 6-hour intervals for Futures
/// - No monthly interval for Futures
pub fn map_futures_granularity(interval: &str) -> u32 {
    match interval {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        "2h" => 120,
        "4h" => 240,
        "8h" => 480,
        "12h" => 720,
        "1d" => 1440,
        "1w" => 10080,
        _ => 60, // default 1 hour
    }
}
