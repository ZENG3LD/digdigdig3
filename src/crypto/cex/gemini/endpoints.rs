//! # Gemini Endpoints
//!
//! URL'ы и endpoint enum для Gemini API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Gemini API
#[derive(Debug, Clone)]
pub struct GeminiUrls {
    pub rest: &'static str,
    pub ws_market: &'static str,
    pub ws_orders: &'static str,
}

impl GeminiUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        rest: "https://api.gemini.com",
        ws_market: "wss://api.gemini.com/v2/marketdata",
        ws_orders: "wss://api.gemini.com/v1/order/events",
    };

    /// Sandbox URLs
    pub const TESTNET: Self = Self {
        rest: "https://api.sandbox.gemini.com",
        ws_market: "wss://api.sandbox.gemini.com/v2/marketdata",
        ws_orders: "wss://api.sandbox.gemini.com/v1/order/events",
    };

    /// Получить REST base URL (Gemini doesn't differentiate by account type)
    pub fn rest_url(&self, _account_type: AccountType) -> &str {
        self.rest
    }

    /// Получить WebSocket URL для market data
    pub fn ws_market_url(&self) -> &str {
        self.ws_market
    }

    /// Получить WebSocket URL для order events
    pub fn ws_orders_url(&self) -> &str {
        self.ws_orders
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Gemini API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GeminiEndpoint {
    // === MARKET DATA (PUBLIC) ===
    Symbols,
    SymbolDetails,
    Ticker,
    TickerV2,
    OrderBook,
    Trades,
    Candles,
    DerivativeCandles,
    PriceFeed,
    NetworkInfo,
    FundingAmount,
    FeePromos,
    RiskStats,

    // === TRADING (PRIVATE) ===
    NewOrder,
    CancelOrder,
    CancelAllOrders,
    CancelSessionOrders,
    OrderStatus,
    ActiveOrders,
    PastTrades,
    TradingVolume,
    NotionalVolume,
    WrapOrder,

    // === ACCOUNT (PRIVATE) ===
    Balances,
    NotionalBalances,
    StakingBalances,
    DepositAddresses,
    NewDepositAddress,
    Withdraw,
    WithdrawFeeEstimate,
    Transfers,
    AccountTransfer,
    Transactions,
    PaymentMethods,
    AccountDetail,

    // === POSITIONS (PRIVATE) ===
    Positions,
    Margin,
    MarginAccount,
    MarginRates,
    MarginOrderPreview,
    FundingPayments,
    FundingPaymentReport,
}

impl GeminiEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Market Data
            Self::Symbols => "/v1/symbols",
            Self::SymbolDetails => "/v1/symbols/details/{symbol}",
            Self::Ticker => "/v1/pubticker/{symbol}",
            Self::TickerV2 => "/v2/ticker/{symbol}",
            Self::OrderBook => "/v1/book/{symbol}",
            Self::Trades => "/v1/trades/{symbol}",
            Self::Candles => "/v2/candles/{symbol}/{time_frame}",
            Self::DerivativeCandles => "/v2/derivatives/candles/{symbol}/{time_frame}",
            Self::PriceFeed => "/v1/pricefeed",
            Self::NetworkInfo => "/v1/network/{token}",
            Self::FundingAmount => "/v1/fundingamount/{symbol}",
            Self::FeePromos => "/v1/feepromos",
            Self::RiskStats => "/v1/riskstats/{symbol}",

            // Trading
            Self::NewOrder => "/v1/order/new",
            Self::CancelOrder => "/v1/order/cancel",
            Self::CancelAllOrders => "/v1/order/cancel/all",
            Self::CancelSessionOrders => "/v1/order/cancel/session",
            Self::OrderStatus => "/v1/order/status",
            Self::ActiveOrders => "/v1/orders",
            Self::PastTrades => "/v1/mytrades",
            Self::TradingVolume => "/v1/tradevolume",
            Self::NotionalVolume => "/v1/notionalvolume",
            Self::WrapOrder => "/v1/wrap/{symbol}",

            // Account
            Self::Balances => "/v1/balances",
            Self::NotionalBalances => "/v1/notionalbalances/{currency}",
            Self::StakingBalances => "/v1/balances/staking",
            Self::DepositAddresses => "/v1/addresses/{network}",
            Self::NewDepositAddress => "/v1/deposit/{network}/newAddress",
            Self::Withdraw => "/v1/withdraw/{currency}",
            Self::WithdrawFeeEstimate => "/v1/withdraw/{currency}/feeEstimate",
            Self::Transfers => "/v1/transfers",
            Self::AccountTransfer => "/v1/account/transfer/{currency}",
            Self::Transactions => "/v1/transactions",
            Self::PaymentMethods => "/v1/payments/methods",
            Self::AccountDetail => "/v1/account",

            // Positions
            Self::Positions => "/v1/positions",
            Self::Margin => "/v1/margin",
            Self::MarginAccount => "/v1/margin/account",
            Self::MarginRates => "/v1/margin/rates",
            Self::MarginOrderPreview => "/v1/margin/order/preview",
            Self::FundingPayments => "/v1/perpetuals/fundingPayment",
            Self::FundingPaymentReport => "/v1/perpetuals/fundingpaymentreport/records.json",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::Symbols
            | Self::SymbolDetails
            | Self::Ticker
            | Self::TickerV2
            | Self::OrderBook
            | Self::Trades
            | Self::Candles
            | Self::DerivativeCandles
            | Self::PriceFeed
            | Self::NetworkInfo
            | Self::FundingAmount
            | Self::FeePromos
            | Self::RiskStats => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            Self::NewOrder
            | Self::CancelOrder
            | Self::CancelAllOrders
            | Self::CancelSessionOrders
            | Self::OrderStatus
            | Self::ActiveOrders
            | Self::PastTrades
            | Self::TradingVolume
            | Self::NotionalVolume
            | Self::WrapOrder
            | Self::Balances
            | Self::NotionalBalances
            | Self::StakingBalances
            | Self::DepositAddresses
            | Self::NewDepositAddress
            | Self::Withdraw
            | Self::WithdrawFeeEstimate
            | Self::Transfers
            | Self::AccountTransfer
            | Self::Transactions
            | Self::PaymentMethods
            | Self::AccountDetail
            | Self::Positions
            | Self::Margin
            | Self::MarginAccount
            | Self::MarginRates
            | Self::MarginOrderPreview
            | Self::FundingPayments
            | Self::FundingPaymentReport => "POST",

            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Gemini
///
/// # Symbol Format
/// - Spot: `btcusd`, `ethusd`, `ethbtc` (lowercase, no separator)
/// - Perpetuals: `btcgusdperp`, `ethgusdperp` (ends with "perp")
///
/// # Examples
/// - Spot: `btcusd`, `ethusd`
/// - Perpetuals: `btcgusdperp`, `ethgusdperp`
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot: lowercase, no separator
            format!("{}{}", base.to_lowercase(), quote.to_lowercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Perpetuals: base + "gusd" + "perp"
            // Note: Gemini perpetuals are denominated in GUSD (Gemini Dollar)
            format!("{}gusdperp", base.to_lowercase())
        }
    }
}

/// Normalize symbol to lowercase
pub fn normalize_symbol(symbol: &str) -> String {
    symbol.to_lowercase()
}

/// Check if symbol is a perpetual
pub fn _is_perpetual(symbol: &str) -> bool {
    symbol.to_lowercase().ends_with("perp")
}

/// Маппинг интервала kline для Gemini API
///
/// # Time Frames
/// - `1m` -> "1m"
/// - `5m` -> "5m"
/// - `15m` -> "15m"
/// - `30m` -> "30m"
/// - `1h` -> "1hr"
/// - `6h` -> "6hr"
/// - `1d` -> "1day"
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "15m" => "15m",
        "30m" => "30m",
        "1h" => "1hr",
        "6h" => "6hr",
        "1d" => "1day",
        _ => "1hr", // default 1 hour
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_spot() {
        let symbol = format_symbol("BTC", "USD", AccountType::Spot);
        assert_eq!(symbol, "btcusd");

        let symbol = format_symbol("ETH", "USD", AccountType::Spot);
        assert_eq!(symbol, "ethusd");

        let symbol = format_symbol("ETH", "BTC", AccountType::Spot);
        assert_eq!(symbol, "ethbtc");
    }

    #[test]
    fn test_format_symbol_perpetual() {
        let symbol = format_symbol("BTC", "USD", AccountType::FuturesCross);
        assert_eq!(symbol, "btcgusdperp");

        let symbol = format_symbol("ETH", "USD", AccountType::FuturesIsolated);
        assert_eq!(symbol, "ethgusdperp");
    }

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(normalize_symbol("BTCUSD"), "btcusd");
        assert_eq!(normalize_symbol("btcusd"), "btcusd");
        assert_eq!(normalize_symbol("BtCuSd"), "btcusd");
    }

    #[test]
    fn test_is_perpetual() {
        assert!(is_perpetual("btcgusdperp"));
        assert!(is_perpetual("BTCGUSDPERP"));
        assert!(!is_perpetual("btcusd"));
        assert!(!is_perpetual("ethusd"));
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "1m");
        assert_eq!(map_kline_interval("5m"), "5m");
        assert_eq!(map_kline_interval("1h"), "1hr");
        assert_eq!(map_kline_interval("6h"), "6hr");
        assert_eq!(map_kline_interval("1d"), "1day");
    }
}
