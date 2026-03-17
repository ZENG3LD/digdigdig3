//! # Bithumb Endpoints
//!
//! URL'ы и endpoint enum для Bithumb Pro API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Bithumb Pro API
#[derive(Debug, Clone)]
pub struct BithumbUrls {
    pub spot_rest: &'static str,
    pub futures_rest: &'static str,
    pub ws: &'static str,
}

impl BithumbUrls {
    /// Production URLs (Bithumb Pro - Global platform)
    pub const MAINNET: Self = Self {
        spot_rest: "https://global-openapi.bithumb.pro/openapi/v1",
        futures_rest: "https://bithumbfutures.com/api/pro/v1",
        ws: "wss://global-api.bithumb.pro/message/realtime",
    };

    /// Bithumb does not offer a testnet/sandbox environment.
    ///
    /// This constant is kept as an alias of `MAINNET` for compatibility only.
    /// Connectors should return `ExchangeError::UnsupportedOperation` when
    /// `testnet = true` is requested rather than silently connecting to mainnet.
    #[deprecated(note = "Bithumb has no testnet. Use MAINNET and reject testnet requests explicitly.")]
    pub const TESTNET: Self = Self {
        spot_rest: "https://global-openapi.bithumb.pro/openapi/v1",
        futures_rest: "https://bithumbfutures.com/api/pro/v1",
        ws: "wss://global-api.bithumb.pro/message/realtime",
    };

    /// Получить REST base URL для account type
    pub fn rest_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_rest,
            // Bithumb Futures is a separate platform with separate API
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_rest,
        }
    }

    /// Получить WebSocket URL
    pub fn ws_url(&self) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bithumb API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BithumbEndpoint {
    // === ОБЩИЕ ===
    ServerTime,

    // === SPOT MARKET DATA ===
    SpotTicker,
    SpotOrderbook,
    SpotKlines,
    SpotTrades,
    SpotConfig,

    // === SPOT TRADING ===
    SpotCreateOrder,
    SpotCancelOrder,
    SpotOrderDetail,
    SpotOpenOrders,
    SpotHistoryOrders,

    // === SPOT ACCOUNT ===
    SpotAccount,
    SpotDepositAddress,
    SpotWithdraw,
    SpotDepositHistory,
    SpotWithdrawHistory,

    // === FUTURES MARKET DATA ===
    FuturesTicker,
    FuturesOrderbook,
    FuturesKlines,
    FuturesTrades,
    FuturesContracts,
    FuturesMarketData,
    FuturesFundingRates,

    // === C3 ADDITIONS ===
    /// POST /spot/singleOrder — query a single order by order ID
    SingleOrder,
    /// POST /spot/assetList — list all available assets with status
    AssetList,
}

impl BithumbEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Общие
            Self::ServerTime => "/serverTime",

            // Spot Market Data
            Self::SpotTicker => "/spot/ticker",
            Self::SpotOrderbook => "/spot/orderBook",
            Self::SpotKlines => "/spot/kline",
            Self::SpotTrades => "/spot/trades",
            Self::SpotConfig => "/spot/config",

            // Spot Trading
            Self::SpotCreateOrder => "/spot/placeOrder",
            Self::SpotCancelOrder => "/spot/cancelOrder",
            Self::SpotOrderDetail => "/spot/orderDetail",
            Self::SpotOpenOrders => "/spot/openOrders",
            Self::SpotHistoryOrders => "/spot/historyOrders",

            // Spot Account
            Self::SpotAccount => "/spot/account",
            Self::SpotDepositAddress => "/wallet/depositAddress",
            Self::SpotWithdraw => "/withdraw",
            Self::SpotDepositHistory => "/wallet/depositHistory",
            Self::SpotWithdrawHistory => "/wallet/withdrawHistory",

            // Futures Market Data
            Self::FuturesTicker => "/ticker",
            Self::FuturesOrderbook => "/depth",
            Self::FuturesKlines => "/barhist",
            Self::FuturesTrades => "/trades",
            Self::FuturesContracts => "/futures/contracts",
            Self::FuturesMarketData => "/futures/market-data",
            Self::FuturesFundingRates => "/futures/funding-rates",

            // C3 Additions
            Self::SingleOrder => "/spot/singleOrder",
            Self::AssetList => "/spot/assetList",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::ServerTime
            | Self::SpotTicker
            | Self::SpotOrderbook
            | Self::SpotKlines
            | Self::SpotTrades
            | Self::SpotConfig
            | Self::FuturesTicker
            | Self::FuturesOrderbook
            | Self::FuturesKlines
            | Self::FuturesTrades
            | Self::FuturesContracts
            | Self::FuturesMarketData
            | Self::FuturesFundingRates => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::SpotCreateOrder
            | Self::SpotCancelOrder
            | Self::SpotOrderDetail
            | Self::SpotOpenOrders
            | Self::SpotHistoryOrders
            | Self::SpotAccount
            | Self::SpotDepositAddress
            | Self::SpotWithdraw
            | Self::SpotDepositHistory
            | Self::SpotWithdrawHistory
            | Self::SingleOrder
            | Self::AssetList => "POST",

            // GET endpoints
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Bithumb Pro
///
/// # Symbol Format
/// - Spot: `BTC-USDT`, `ETH-USDT` (hyphen-separated)
/// - Futures: `BTC-PERP`, `ETH-PERP` (perpetual contracts)
///
/// # Examples
/// - Spot: `BTC-USDT`, `ETH-USDT`, `XRP-USDT`
/// - Futures: `BTC-PERP`, `ETH-PERP`
/// - Primary quote: USDT (spot), PERP (futures)
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Bithumb Futures uses PERP suffix for perpetual contracts
            // Ignore quote parameter for futures (always -PERP)
            format!("{}-PERP", base.to_uppercase())
        }
        _ => {
            // Spot uses hyphen separator: BTC-USDT
            format!("{}-{}", base.to_uppercase(), quote.to_uppercase())
        }
    }
}

/// Маппинг интервала kline для Bithumb API
///
/// # Bithumb Pro Spot Format (parameter: `type`)
/// Values: `"m1"`, `"m5"`, `"m15"`, `"m30"`, `"h1"`, `"h4"`, `"d1"`, `"w1"`, `"M1"`
///
/// # Bithumb Futures Format (parameter: `interval`)
/// Values: `"1"`, `"5"`, `"15"`, `"30"`, `"60"` (minutes), `"1d"`, `"1w"`, `"1M"`
///
/// Supported intervals:
/// - Minutes: 1, 5, 15, 30, 60
/// - Hours: 2h, 4h, 6h, 12h
/// - Days: 1d
/// - Weeks: 1w
/// - Months: 1M
pub fn map_kline_interval(interval: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Bithumb Futures uses numeric values for minutes
            match interval {
                "1m" => "1",
                "5m" => "5",
                "15m" => "15",
                "30m" => "30",
                "1h" => "60",
                "2h" => "2h",
                "4h" => "4h",
                "6h" => "6h",
                "12h" => "12h",
                "1d" => "1d",
                "1w" => "1w",
                "1M" => "1M",
                _ => "60", // default to 1 hour
            }.to_string()
        }
        _ => {
            // Bithumb Pro Spot uses letter codes
            match interval {
                "1m" => "m1",
                "5m" => "m5",
                "15m" => "m15",
                "30m" => "m30",
                "1h" => "h1",
                "4h" => "h4",
                "1d" => "d1",
                "1w" => "w1",
                "1M" => "M1",
                _ => "h1", // default to 1 hour
            }.to_string()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        assert_eq!(
            format_symbol("BTC", "USDT", AccountType::Spot),
            "BTC-USDT"
        );
        assert_eq!(
            format_symbol("eth", "usdt", AccountType::Spot),
            "ETH-USDT"
        );
        assert_eq!(
            format_symbol("BTC", "USDT", AccountType::FuturesCross),
            "BTC-PERP"
        );
        assert_eq!(
            format_symbol("eth", "usdt", AccountType::FuturesCross),
            "ETH-PERP"
        );
    }

    #[test]
    fn test_map_kline_interval() {
        // Spot intervals
        assert_eq!(map_kline_interval("1m", AccountType::Spot), "m1");
        assert_eq!(map_kline_interval("5m", AccountType::Spot), "m5");
        assert_eq!(map_kline_interval("1h", AccountType::Spot), "h1");
        assert_eq!(map_kline_interval("1d", AccountType::Spot), "d1");
        assert_eq!(map_kline_interval("1w", AccountType::Spot), "w1");
        assert_eq!(map_kline_interval("unknown", AccountType::Spot), "h1");

        // Futures intervals
        assert_eq!(map_kline_interval("1m", AccountType::FuturesCross), "1");
        assert_eq!(map_kline_interval("5m", AccountType::FuturesCross), "5");
        assert_eq!(map_kline_interval("1h", AccountType::FuturesCross), "60");
        assert_eq!(map_kline_interval("1d", AccountType::FuturesCross), "1d");
        assert_eq!(map_kline_interval("unknown", AccountType::FuturesCross), "60");
    }

    #[test]
    fn test_endpoint_path() {
        assert_eq!(BithumbEndpoint::SpotTicker.path(), "/spot/ticker");
        assert_eq!(BithumbEndpoint::SpotOrderbook.path(), "/spot/orderBook");
        assert_eq!(BithumbEndpoint::SpotCreateOrder.path(), "/spot/placeOrder");
    }

    #[test]
    fn test_endpoint_auth() {
        assert!(!BithumbEndpoint::SpotTicker.requires_auth());
        assert!(!BithumbEndpoint::SpotOrderbook.requires_auth());
        assert!(BithumbEndpoint::SpotCreateOrder.requires_auth());
        assert!(BithumbEndpoint::SpotAccount.requires_auth());
    }

    #[test]
    fn test_endpoint_method() {
        assert_eq!(BithumbEndpoint::SpotTicker.method(), "GET");
        assert_eq!(BithumbEndpoint::SpotCreateOrder.method(), "POST");
        assert_eq!(BithumbEndpoint::SpotAccount.method(), "POST");
    }
}
