//! # Kraken Endpoints
//!
//! URL's and endpoint enum for Kraken API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL's for Kraken API
#[derive(Debug, Clone)]
pub struct KrakenUrls {
    pub spot_rest: &'static str,
    pub futures_rest: &'static str,
    pub spot_ws: &'static str,
    pub futures_ws: &'static str,
}

impl KrakenUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        spot_rest: "https://api.kraken.com",
        futures_rest: "https://futures.kraken.com",
        spot_ws: "wss://ws.kraken.com/v2",
        futures_ws: "wss://futures.kraken.com/ws/v1",
    };

    /// Sandbox URLs (Kraken doesn't have official testnet, use demo futures)
    pub const TESTNET: Self = Self {
        spot_rest: "https://api.kraken.com", // No testnet for spot
        futures_rest: "https://demo-futures.kraken.com",
        spot_ws: "wss://ws.kraken.com/v2",
        futures_ws: "wss://demo-futures.kraken.com/ws/v1",
    };

    /// Get REST base URL for account type
    pub fn rest_url(&self, account_type: AccountType) -> &str {
        match account_type {
            AccountType::Spot | AccountType::Margin => self.spot_rest,
            AccountType::FuturesCross | AccountType::FuturesIsolated => self.futures_rest,
        }
    }

    /// Get WebSocket URL for account type
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

/// Kraken API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KrakenEndpoint {
    // === COMMON ===
    ServerTime,

    // === SPOT MARKET DATA ===
    SpotTicker,
    SpotOrderbook,
    SpotOHLC,
    SpotAssetPairs,

    // === SPOT TRADING ===
    SpotAddOrder,
    SpotCancelOrder,
    SpotCancelAll,
    SpotEditOrder,
    SpotGetOrder,
    SpotOpenOrders,
    SpotClosedOrders,

    // === SPOT ACCOUNT ===
    SpotBalance,
    SpotTradeBalance,

    // === SPOT WEBSOCKET ===
    SpotWebSocketToken,

    // === FUTURES MARKET DATA ===
    FuturesTickers,
    FuturesOrderbook,
    FuturesInstruments,
    FuturesHistory,

    // === FUTURES TRADING ===
    FuturesSendOrder,
    FuturesCancelOrder,
    FuturesBatchOrder,
    FuturesEditOrder,

    // === FUTURES ACCOUNT ===
    FuturesAccounts,
    FuturesOpenPositions,
    FuturesHistoricalFunding,

    // === FUTURES LEVERAGE ===
    FuturesSetLeverage,
}

impl KrakenEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Common
            Self::ServerTime => "/0/public/Time",

            // Spot Market Data
            Self::SpotTicker => "/0/public/Ticker",
            Self::SpotOrderbook => "/0/public/Depth",
            Self::SpotOHLC => "/0/public/OHLC",
            Self::SpotAssetPairs => "/0/public/AssetPairs",

            // Spot Trading
            Self::SpotAddOrder => "/0/private/AddOrder",
            Self::SpotCancelOrder => "/0/private/CancelOrder",
            Self::SpotCancelAll => "/0/private/CancelAll",
            Self::SpotEditOrder => "/0/private/EditOrder",
            Self::SpotGetOrder => "/0/private/QueryOrders",
            Self::SpotOpenOrders => "/0/private/OpenOrders",
            Self::SpotClosedOrders => "/0/private/ClosedOrders",

            // Spot Account
            Self::SpotBalance => "/0/private/Balance",
            Self::SpotTradeBalance => "/0/private/TradeBalance",

            // Spot WebSocket
            Self::SpotWebSocketToken => "/0/private/GetWebSocketsToken",

            // Futures Market Data
            Self::FuturesTickers => "/derivatives/api/v3/tickers",
            Self::FuturesOrderbook => "/derivatives/api/v3/orderbook",
            Self::FuturesInstruments => "/derivatives/api/v3/instruments",
            Self::FuturesHistory => "/derivatives/api/v3/history",

            // Futures Trading
            Self::FuturesSendOrder => "/derivatives/api/v3/sendorder",
            Self::FuturesCancelOrder => "/derivatives/api/v3/cancelorder",
            Self::FuturesBatchOrder => "/derivatives/api/v3/batchorder",
            Self::FuturesEditOrder => "/derivatives/api/v3/editorder",

            // Futures Account
            Self::FuturesAccounts => "/derivatives/api/v3/accounts",
            Self::FuturesOpenPositions => "/derivatives/api/v3/openpositions",
            Self::FuturesHistoricalFunding => "/derivatives/api/v4/historicalfundingrates",

            // Futures Leverage
            Self::FuturesSetLeverage => "/derivatives/api/v3/leveragepreferences",
        }
    }

    /// Does endpoint require authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::ServerTime
            | Self::SpotTicker
            | Self::SpotOrderbook
            | Self::SpotOHLC
            | Self::SpotAssetPairs
            | Self::FuturesTickers
            | Self::FuturesOrderbook
            | Self::FuturesInstruments
            | Self::FuturesHistory => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::SpotAddOrder
            | Self::SpotCancelOrder
            | Self::SpotCancelAll
            | Self::SpotEditOrder
            | Self::SpotGetOrder
            | Self::SpotOpenOrders
            | Self::SpotClosedOrders
            | Self::SpotBalance
            | Self::SpotTradeBalance
            | Self::SpotWebSocketToken
            | Self::FuturesSendOrder
            | Self::FuturesCancelOrder
            | Self::FuturesBatchOrder
            | Self::FuturesEditOrder
            | Self::FuturesSetLeverage => "POST",

            // GET endpoints
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Kraken API
///
/// # Symbol Format Differences
///
/// ## Spot REST API
/// - Request: Simplified format (`XBTUSD`)
/// - Response: Full ISO format (`XXBTZUSD`)
/// - Use BTC → XBT mapping
/// - Prefix convention: X for crypto, Z for fiat
///
/// ## Futures API
/// - Format: `PI_{base}{quote}` for perpetual inverse
/// - Example: `PI_XBTUSD`, `PI_ETHUSD`
/// - Product prefixes:
///   - `PI_`: Perpetual Inverse (crypto collateral)
///   - `PF_`: Perpetual Forward (linear, USD collateral)
///   - `FI_`: Fixed maturity Inverse
///   - `FF_`: Fixed maturity Forward
///
/// # Examples
/// - Spot: `XBTUSD` (request) → `XXBTZUSD` (response)
/// - Spot: `ETHUSD` → `XETHZUSD`
/// - Futures: `BTC` + `USD` → `PI_XBTUSD`
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot uses simplified format for requests
            // BTC → XBT for Bitcoin
            let base = if base.to_uppercase() == "BTC" { "XBT" } else { base };
            format!("{}{}", base, quote)
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Futures: PI_{BASE}{QUOTE} for perpetual inverse
            // BTC → XBT for Bitcoin
            let base = if base.to_uppercase() == "BTC" { "XBT" } else { base };
            format!("PI_{}{}", base, quote)
        }
    }
}

/// Parse response symbol to extract base and quote
///
/// Kraken responses use full ISO format with prefixes:
/// - `XXBTZUSD` → (base: "XBT", quote: "USD")
/// - `XETHZUSD` → (base: "ETH", quote: "USD")
/// - `PI_XBTUSD` → (base: "XBT", quote: "USD")
#[allow(dead_code)]
pub fn parse_response_symbol(symbol: &str) -> Option<(String, String)> {
    // Futures format: PI_XBTUSD
    if symbol.starts_with("PI_") || symbol.starts_with("PF_") {
        let parts = symbol.split('_').nth(1)?;
        // Simple split: assume 3-letter base
        if parts.len() >= 6 {
            let base = &parts[0..3];
            let quote = &parts[3..];
            return Some((base.to_string(), quote.to_string()));
        }
    }

    // Spot format: XXBTZUSD, XETHZUSD
    // Strip X prefix from crypto, Z prefix from fiat
    let clean = symbol
        .strip_prefix("XX")
        .or_else(|| symbol.strip_prefix("X"))
        .unwrap_or(symbol);

    // Common pairs
    for fiat in &["ZUSD", "ZEUR", "ZGBP", "ZJPY", "ZCAD"] {
        if let Some(base) = clean.strip_suffix(fiat) {
            return Some((base.to_string(), fiat.strip_prefix("Z").expect("Fiat codes start with Z").to_string()));
        }
    }

    // Crypto pairs (e.g., XETHXXBT)
    if clean.len() >= 6 {
        let base = &clean[0..3];
        let quote = &clean[3..];
        return Some((base.to_string(), quote.to_string()));
    }

    None
}

/// Map kline interval to Kraken OHLC interval
///
/// Kraken uses integer minutes for intervals
pub fn map_ohlc_interval(interval: &str) -> u32 {
    match interval {
        "1m" => 1,
        "5m" => 5,
        "15m" => 15,
        "30m" => 30,
        "1h" => 60,
        "4h" => 240,
        "1d" => 1440,
        "1w" => 10080,
        "15d" => 21600,
        _ => 60, // default 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_spot() {
        assert_eq!(format_symbol("BTC", "USD", AccountType::Spot), "XBTUSD");
        assert_eq!(format_symbol("ETH", "USD", AccountType::Spot), "ETHUSD");
        assert_eq!(format_symbol("XBT", "EUR", AccountType::Spot), "XBTEUR");
    }

    #[test]
    fn test_format_symbol_futures() {
        assert_eq!(
            format_symbol("BTC", "USD", AccountType::FuturesCross),
            "PI_XBTUSD"
        );
        assert_eq!(
            format_symbol("ETH", "USD", AccountType::FuturesCross),
            "PI_ETHUSD"
        );
    }

    #[test]
    fn test_parse_response_symbol() {
        assert_eq!(
            parse_response_symbol("XXBTZUSD"),
            Some(("XBT".to_string(), "USD".to_string()))
        );
        assert_eq!(
            parse_response_symbol("XETHZUSD"),
            Some(("ETH".to_string(), "USD".to_string()))
        );
        assert_eq!(
            parse_response_symbol("PI_XBTUSD"),
            Some(("XBT".to_string(), "USD".to_string()))
        );
    }

    #[test]
    fn test_map_ohlc_interval() {
        assert_eq!(map_ohlc_interval("1m"), 1);
        assert_eq!(map_ohlc_interval("1h"), 60);
        assert_eq!(map_ohlc_interval("1d"), 1440);
        assert_eq!(map_ohlc_interval("unknown"), 60);
    }
}
