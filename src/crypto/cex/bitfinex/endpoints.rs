//! # Bitfinex Endpoints
//!
//! URL's and endpoint enum for Bitfinex API v2.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL's for Bitfinex API
#[derive(Debug, Clone)]
pub struct BitfinexUrls {
    pub public_rest: &'static str,
    pub auth_rest: &'static str,
}

impl BitfinexUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        public_rest: "https://api-pub.bitfinex.com/v2",
        auth_rest: "https://api.bitfinex.com/v2",
    };

    /// Get REST base URL (public vs authenticated)
    pub fn rest_url(&self, authenticated: bool) -> &str {
        if authenticated {
            self.auth_rest
        } else {
            self.public_rest
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Bitfinex API v2 endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BitfinexEndpoint {
    // === PUBLIC ===
    PlatformStatus,

    // === MARKET DATA ===
    Ticker,
    Tickers,
    Orderbook,
    Trades,
    Candles,
    Symbols,

    // === TRADING ===
    SubmitOrder,
    CancelOrder,
    CancelMultipleOrders,
    UpdateOrder,
    OrderMulti,

    // === ACCOUNT ===
    Wallets,
    ActiveOrders,
    ActiveOrdersBySymbol,
    OrderHistory,
    OrderTrades,
    TradeHistory,
    TradeHistoryBySymbol,

    // === POSITIONS ===
    Positions,
    PositionHistory,
    PositionSnapshot,

    // === ACCOUNT TRANSFERS ===
    Transfer,

    // === CUSTODIAL FUNDS ===
    DepositAddress,
    Withdraw,
    Movements,

    // === SUB ACCOUNTS ===
    SubAccountList,
    SubAccountTransfer,
}

impl BitfinexEndpoint {
    /// Get path endpoint
    pub fn path(&self) -> &'static str {
        match self {
            // Public
            Self::PlatformStatus => "/platform/status",

            // Market Data
            Self::Ticker => "/ticker/{symbol}",
            Self::Tickers => "/tickers",
            Self::Orderbook => "/book/{symbol}/{precision}",
            Self::Trades => "/trades/{symbol}/hist",
            Self::Candles => "/candles/{candle}/hist",
            Self::Symbols => "/conf/pub:list:pair:exchange",

            // Trading
            Self::SubmitOrder => "/auth/w/order/submit",
            Self::CancelOrder => "/auth/w/order/cancel",
            Self::CancelMultipleOrders => "/auth/w/order/cancel/multi",
            Self::UpdateOrder => "/auth/w/order/update",
            Self::OrderMulti => "/auth/w/order/multi",

            // Account
            Self::Wallets => "/auth/r/wallets",
            Self::ActiveOrders => "/auth/r/orders",
            Self::ActiveOrdersBySymbol => "/auth/r/orders/{symbol}",
            Self::OrderHistory => "/auth/r/orders/hist",
            Self::OrderTrades => "/auth/r/order/{symbol}:{id}/trades",
            Self::TradeHistory => "/auth/r/trades/hist",
            Self::TradeHistoryBySymbol => "/auth/r/trades/{symbol}/hist",

            // Positions
            Self::Positions => "/auth/r/positions",
            Self::PositionHistory => "/auth/r/positions/hist",
            Self::PositionSnapshot => "/auth/r/positions/snap",

            // Account Transfers
            Self::Transfer => "/auth/w/transfer",

            // Custodial Funds
            Self::DepositAddress => "/auth/w/deposit/address",
            Self::Withdraw => "/auth/w/withdraw",
            Self::Movements => "/auth/r/movements/{symbol}/hist",

            // Sub Accounts
            Self::SubAccountList => "/auth/r/sub_accounts/list",
            Self::SubAccountTransfer => "/auth/w/sub_account/transfer",
        }
    }

    /// Requires authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::PlatformStatus
            | Self::Ticker
            | Self::Tickers
            | Self::Orderbook
            | Self::Trades
            | Self::Candles
            | Self::Symbols => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            // POST for all authenticated endpoints (Bitfinex v2 convention)
            Self::SubmitOrder
            | Self::CancelOrder
            | Self::CancelMultipleOrders
            | Self::UpdateOrder
            | Self::OrderMulti
            | Self::Wallets
            | Self::ActiveOrders
            | Self::ActiveOrdersBySymbol
            | Self::OrderHistory
            | Self::OrderTrades
            | Self::TradeHistory
            | Self::TradeHistoryBySymbol
            | Self::Positions
            | Self::PositionHistory
            | Self::PositionSnapshot
            | Self::Transfer
            | Self::DepositAddress
            | Self::Withdraw
            | Self::Movements
            | Self::SubAccountList
            | Self::SubAccountTransfer => "POST",

            // GET for all public endpoints
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Bitfinex
///
/// # Bitfinex Symbol Format
/// - Trading pairs use `t` prefix: `tBTCUSD`, `tETHBTC`
/// - Funding currencies use `f` prefix: `fUSD`, `fBTC`
/// - All symbols must be UPPERCASE
/// - No separators (hyphens, slashes, etc.)
///
/// # Examples
/// - Spot: `tBTCUSD`, `tETHUSD`
/// - Margin: `tBTCUSD` (same as spot)
/// - Futures: `tBTCF0:USTF0` (perpetual)
///
/// # Note
/// Bitfinex doesn't distinguish between spot and margin at the symbol level.
/// Margin trading is handled via account type and order flags.
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    let base_up = base.to_uppercase();
    let quote_up = quote.to_uppercase();

    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Bitfinex spot uses USD, not USDT — map USDT → USD
            let quote_mapped = if quote_up == "USDT" { "USD" } else { &quote_up };
            format!("t{}{}", base_up, quote_mapped)
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Perpetual futures format: tBASEF0:QUOTEF0
            // Bitfinex futures use UST, not USDT — map USDT → UST
            let quote_mapped = if quote_up == "USDT" { "UST" } else { &quote_up };
            format!("t{}F0:{}F0", base_up, quote_mapped)
        }
    }
}

/// Map kline interval to Bitfinex timeframe
///
/// # Bitfinex Candle Format
/// Candles use the format: `trade:{timeframe}:{symbol}`
/// Example: `trade:1m:tBTCUSD`
///
/// # Supported Timeframes
/// - 1m, 5m, 15m, 30m (minutes)
/// - 1h, 3h, 6h, 12h (hours)
/// - 1D (day)
/// - 1W (week)
/// - 14D (2 weeks)
/// - 1M (month)
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "3m" => "3m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1h",
        "2h" => "2h",
        "3h" => "3h",
        "4h" => "4h",
        "6h" => "6h",
        "8h" => "8h",
        "12h" => "12h",
        "1d" => "1D",
        "1w" => "1W",
        "2w" => "14D",
        "1M" => "1M",
        _ => "1h", // default
    }
}

/// Build candle key for Bitfinex API
///
/// Format: `trade:{timeframe}:{symbol}`
pub fn build_candle_key(symbol: &str, interval: &str) -> String {
    format!("trade:{}:{}", map_kline_interval(interval), symbol)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_spot() {
        let symbol = format_symbol("BTC", "USD", AccountType::Spot);
        assert_eq!(symbol, "tBTCUSD");

        let symbol = format_symbol("eth", "btc", AccountType::Spot);
        assert_eq!(symbol, "tETHBTC");
    }

    #[test]
    fn test_format_symbol_spot_usdt_mapped_to_usd() {
        // Bitfinex doesn't have BTC/USDT — USDT must map to USD for spot
        let symbol = format_symbol("BTC", "USDT", AccountType::Spot);
        assert_eq!(symbol, "tBTCUSD");

        let symbol = format_symbol("ETH", "USDT", AccountType::Margin);
        assert_eq!(symbol, "tETHUSD");
    }

    #[test]
    fn test_format_symbol_futures() {
        let symbol = format_symbol("BTC", "UST", AccountType::FuturesCross);
        assert_eq!(symbol, "tBTCF0:USTF0");

        let symbol = format_symbol("ETH", "UST", AccountType::FuturesIsolated);
        assert_eq!(symbol, "tETHF0:USTF0");
    }

    #[test]
    fn test_format_symbol_futures_usdt_mapped_to_ust() {
        // Bitfinex perpetual futures use UST, not USDT
        let symbol = format_symbol("BTC", "USDT", AccountType::FuturesCross);
        assert_eq!(symbol, "tBTCF0:USTF0");
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "1m");
        assert_eq!(map_kline_interval("1h"), "1h");
        assert_eq!(map_kline_interval("1d"), "1D");
        assert_eq!(map_kline_interval("1w"), "1W");
        assert_eq!(map_kline_interval("1M"), "1M");
    }

    #[test]
    fn test_build_candle_key() {
        let key = build_candle_key("tBTCUSD", "1m");
        assert_eq!(key, "trade:1m:tBTCUSD");

        let key = build_candle_key("tETHUSD", "1d");
        assert_eq!(key, "trade:1D:tETHUSD");
    }
}
