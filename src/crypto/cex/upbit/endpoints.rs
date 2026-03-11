//! # Upbit Endpoints
//!
//! URL'ы и endpoint enum для Upbit API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Upbit API (региональные)
#[derive(Debug, Clone)]
pub struct UpbitUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl UpbitUrls {
    /// Singapore region (производство)
    pub const SINGAPORE: Self = Self {
        rest: "https://sg-api.upbit.com",
        ws: "wss://sg-api.upbit.com/websocket/v1",
    };

    /// Indonesia region (производство)
    pub const INDONESIA: Self = Self {
        rest: "https://id-api.upbit.com",
        ws: "wss://id-api.upbit.com/websocket/v1",
    };

    /// Thailand region (производство)
    pub const THAILAND: Self = Self {
        rest: "https://th-api.upbit.com",
        ws: "wss://th-api.upbit.com/websocket/v1",
    };

    /// Korea region (main Upbit platform with KRW markets)
    pub const KOREA: Self = Self {
        rest: "https://api.upbit.com",
        ws: "wss://api.upbit.com/websocket/v1",
    };

    /// Default to Korea region (KRW markets)
    pub const DEFAULT: Self = Self::KOREA;

    /// Получить REST base URL (Upbit только Spot)
    pub fn rest_url(&self, _account_type: AccountType) -> &str {
        self.rest
    }

    /// Получить WebSocket URL (публичный)
    pub fn ws_url(&self) -> &str {
        self.ws
    }

    /// Получить WebSocket URL (приватный)
    pub fn ws_private_url(&self) -> String {
        format!("{}/private", self.ws)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Upbit API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpbitEndpoint {
    // === MARKET DATA (PUBLIC) ===
    TradingPairs,
    CandlesMinutes,  // Path: /v1/candles/minutes/{unit}
    CandlesDays,
    CandlesWeeks,
    CandlesMonths,
    CandlesYears,
    CandlesSeconds,
    Tickers,
    TickersQuote,
    Orderbook,
    OrderbookInstruments,
    RecentTrades,

    // === TRADING (PRIVATE) ===
    OrderInfo,
    CreateOrder,
    TestOrder,
    GetOrder,
    ListOrders,
    CancelOrder,
    BatchCancelOrders,

    // === ACCOUNT (PRIVATE) ===
    Balances,
    DepositInfo,
    ListDepositAddresses,
    CreateDepositAddress,
    ListDeposits,
    WithdrawalInfo,
    ListWithdrawalAddresses,
    InitiateWithdrawal,
    ListWithdrawals,
}

impl UpbitEndpoint {
    /// Получить путь endpoint'а
    /// Для CandlesMinutes нужно вставить unit в путь отдельно
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data (Public)
            Self::TradingPairs => "/v1/market/all",
            Self::CandlesMinutes => "/v1/candles/minutes",  // + /{unit} в рантайме
            Self::CandlesDays => "/v1/candles/days",
            Self::CandlesWeeks => "/v1/candles/weeks",
            Self::CandlesMonths => "/v1/candles/months",
            Self::CandlesYears => "/v1/candles/years",
            Self::CandlesSeconds => "/v1/candles/seconds",
            Self::Tickers => "/v1/ticker",
            Self::TickersQuote => "/v1/ticker",
            Self::Orderbook => "/v1/orderbook",
            Self::OrderbookInstruments => "/v1/orderbook",
            Self::RecentTrades => "/v1/trades/ticks",

            // Trading (Private)
            Self::OrderInfo => "/v1/order",
            Self::CreateOrder => "/v1/orders",
            Self::TestOrder => "/v1/orders",
            Self::GetOrder => "/v1/order",
            Self::ListOrders => "/v1/orders",
            Self::CancelOrder => "/v1/order",
            Self::BatchCancelOrders => "/v1/orders",

            // Account (Private)
            Self::Balances => "/v1/accounts",
            Self::DepositInfo => "/v1/deposit",
            Self::ListDepositAddresses => "/v1/deposits/coin_addresses",
            Self::CreateDepositAddress => "/v1/deposits/generate_coin_address",
            Self::ListDeposits => "/v1/deposits",
            Self::WithdrawalInfo => "/v1/withdrawal",
            Self::ListWithdrawalAddresses => "/v1/withdraws/coin_addresses",
            Self::InitiateWithdrawal => "/v1/withdraws/coin",
            Self::ListWithdrawals => "/v1/withdraws",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::TradingPairs
            | Self::CandlesMinutes
            | Self::CandlesDays
            | Self::CandlesWeeks
            | Self::CandlesMonths
            | Self::CandlesYears
            | Self::CandlesSeconds
            | Self::Tickers
            | Self::TickersQuote
            | Self::Orderbook
            | Self::OrderbookInstruments
            | Self::RecentTrades => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::CreateOrder
            | Self::TestOrder
            | Self::CreateDepositAddress
            | Self::InitiateWithdrawal => "POST",

            Self::CancelOrder
            | Self::BatchCancelOrders => "DELETE",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Upbit
///
/// # Upbit Symbol Format
/// **CRITICAL**: Upbit uses **REVERSED** format: `{QUOTE}-{BASE}`
/// - Most exchanges: `BTC-USDT` or `BTCUSDT` (BASE-QUOTE)
/// - **Upbit**: `USDT-BTC` (QUOTE-BASE)
///
/// # Examples
/// - Bitcoin in Singapore Dollar: `SGD-BTC` (not `BTC-SGD`)
/// - Ethereum in Korean Won: `KRW-ETH` (not `ETH-KRW`)
/// - Ripple in Thai Baht: `THB-XRP` (not `XRP-THB`)
///
/// # Reading the format
/// `SGD-BTC` means: "Price of 1 BTC in SGD"
/// - Quote currency (SGD): what you pay
/// - Base currency (BTC): what you get
pub fn format_symbol(base: &str, quote: &str, _account_type: AccountType) -> String {
    // Upbit only supports Spot, no Futures
    // Format: QUOTE-BASE (reversed from standard)
    format!("{}-{}", quote.to_uppercase(), base.to_uppercase())
}

/// Парсинг Upbit символа в base и quote
///
/// # Example
/// ```ignore
/// let (base, quote) = parse_symbol("SGD-BTC").unwrap();
/// assert_eq!(base, "BTC");
/// assert_eq!(quote, "SGD");
/// ```
#[allow(dead_code)]
pub fn parse_symbol(symbol: &str) -> Option<(String, String)> {
    let parts: Vec<&str> = symbol.split('-').collect();
    if parts.len() != 2 {
        return None;
    }
    // Upbit format: QUOTE-BASE
    let quote = parts[0].to_string();
    let base = parts[1].to_string();
    Some((base, quote))
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERVAL MAPPING
// ═══════════════════════════════════════════════════════════════════════════════

/// Маппинг интервала kline для Upbit API
///
/// # Upbit Interval System
/// Upbit uses DIFFERENT ENDPOINTS for different timeframes:
/// - Minutes: `/v1/candles/minutes/{unit}` where unit = 1,3,5,10,15,30,60,240
/// - Days: `/v1/candles/days`
/// - Weeks: `/v1/candles/weeks`
/// - Months: `/v1/candles/months`
/// - Years: `/v1/candles/years`
///
/// # Returns
/// (endpoint, optional_unit)
/// - For minutes: (CandlesMinutes, Some(unit))
/// - For others: (endpoint, None)
pub fn map_kline_interval(interval: &str) -> (UpbitEndpoint, Option<u32>) {
    match interval {
        "1m" => (UpbitEndpoint::CandlesMinutes, Some(1)),
        "3m" => (UpbitEndpoint::CandlesMinutes, Some(3)),
        "5m" => (UpbitEndpoint::CandlesMinutes, Some(5)),
        "10m" => (UpbitEndpoint::CandlesMinutes, Some(10)),
        "15m" => (UpbitEndpoint::CandlesMinutes, Some(15)),
        "30m" => (UpbitEndpoint::CandlesMinutes, Some(30)),
        "1h" => (UpbitEndpoint::CandlesMinutes, Some(60)),
        "4h" => (UpbitEndpoint::CandlesMinutes, Some(240)),
        "1d" => (UpbitEndpoint::CandlesDays, None),
        "1w" => (UpbitEndpoint::CandlesWeeks, None),
        "1M" => (UpbitEndpoint::CandlesMonths, None),
        // Default to 1 hour
        _ => (UpbitEndpoint::CandlesMinutes, Some(60)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        // Upbit uses QUOTE-BASE format (reversed)
        assert_eq!(format_symbol("BTC", "SGD", AccountType::Spot), "SGD-BTC");
        assert_eq!(format_symbol("ETH", "USDT", AccountType::Spot), "USDT-ETH");
        assert_eq!(format_symbol("xrp", "krw", AccountType::Spot), "KRW-XRP");
    }

    #[test]
    fn test_parse_symbol() {
        let (base, quote) = parse_symbol("SGD-BTC").unwrap();
        assert_eq!(base, "BTC");
        assert_eq!(quote, "SGD");

        let (base, quote) = parse_symbol("KRW-ETH").unwrap();
        assert_eq!(base, "ETH");
        assert_eq!(quote, "KRW");

        assert!(parse_symbol("INVALID").is_none());
        assert!(parse_symbol("TOO-MANY-PARTS").is_none());
    }

    #[test]
    fn test_map_kline_interval() {
        let (endpoint, unit) = map_kline_interval("1m");
        assert_eq!(endpoint, UpbitEndpoint::CandlesMinutes);
        assert_eq!(unit, Some(1));

        let (endpoint, unit) = map_kline_interval("1h");
        assert_eq!(endpoint, UpbitEndpoint::CandlesMinutes);
        assert_eq!(unit, Some(60));

        let (endpoint, unit) = map_kline_interval("1d");
        assert_eq!(endpoint, UpbitEndpoint::CandlesDays);
        assert_eq!(unit, None);
    }
}
