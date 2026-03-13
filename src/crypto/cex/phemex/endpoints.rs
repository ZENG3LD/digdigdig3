//! # Phemex Endpoints
//!
//! URL'ы и endpoint enum для Phemex API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Phemex API
#[derive(Debug, Clone)]
pub struct PhemexUrls {
    pub rest: &'static str,
    pub ws: &'static str,
}

impl PhemexUrls {
    /// Production URLs
    /// NOTE: wss://phemex.com/ws is deprecated (returns 410 Gone).
    /// Current endpoint: wss://ws.phemex.com/ws
    pub const MAINNET: Self = Self {
        rest: "https://api.phemex.com",
        ws: "wss://ws.phemex.com/ws",
    };

    /// VIP URLs (higher rate limits)
    pub const VIP: Self = Self {
        rest: "https://vapi.phemex.com",
        ws: "wss://vapi.phemex.com/ws",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        rest: "https://testnet-api.phemex.com",
        ws: "wss://testnet.phemex.com/ws",
    };

    /// Получить REST base URL (Phemex uses unified URL for all account types)
    pub fn rest_url(&self, _account_type: AccountType) -> &str {
        self.rest
    }

    /// Получить WebSocket URL (Phemex uses unified WS URL)
    pub fn ws_url(&self, _account_type: AccountType) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Phemex API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhemexEndpoint {
    // === GENERAL ===
    ServerTime,
    Products,

    // === SPOT MARKET DATA ===
    SpotOrderbook,
    SpotTrades,
    SpotTicker24h,
    SpotKlines,

    // === CONTRACT MARKET DATA ===
    ContractOrderbook,
    ContractTrades,
    ContractTicker24h,
    ContractKlines,
    FundingRateHistory,

    // === SPOT TRADING ===
    SpotCreateOrder,
    SpotAmendOrder,
    SpotCancelOrder,
    SpotCancelAllOrders,
    SpotOpenOrders,

    // === CONTRACT TRADING ===
    ContractCreateOrder,
    ContractAmendOrder,
    ContractCancelOrder,
    ContractCancelAllOrders,
    ContractOpenOrders,
    ContractClosedOrders,
    ContractGetOrder,
    ContractGetTrades,

    // === HEDGED CONTRACT TRADING ===
    HedgedCreateOrder,
    HedgedAmendOrder,
    HedgedCancelOrder,

    // === SPOT ACCOUNT ===
    SpotWallets,

    // === CONTRACT ACCOUNT ===
    ContractAccount,
    Transfer,
    TransferHistory,

    // === POSITIONS ===
    Positions,
    SetLeverage,
    SetRiskLimit,
    AssignBalance,

    // === CUSTODIAL FUNDS ===
    DepositAddress,
    Withdraw,
    DepositList,
    WithdrawList,

    // === SUB ACCOUNTS ===
    SubAccountCreate,
    SubAccountList,
    SubAccountTransfer,
}

impl PhemexEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // General
            Self::ServerTime => "/public/time",
            Self::Products => "/public/products",

            // Spot Market Data
            Self::SpotOrderbook => "/md/orderbook",
            Self::SpotTrades => "/md/trade",
            Self::SpotTicker24h => "/md/spot/ticker/24hr",
            Self::SpotKlines => "/exchange/public/md/v2/kline",

            // Contract Market Data
            Self::ContractOrderbook => "/md/orderbook",
            Self::ContractTrades => "/md/trade",
            Self::ContractTicker24h => "/md/ticker/24hr",
            Self::ContractKlines => "/exchange/public/md/v2/kline",
            Self::FundingRateHistory => "/api-data/public/data/funding-rate-history",

            // Spot Trading
            Self::SpotCreateOrder => "/spot/orders",
            Self::SpotAmendOrder => "/spot/orders",
            Self::SpotCancelOrder => "/spot/orders",
            Self::SpotCancelAllOrders => "/spot/orders/all",
            Self::SpotOpenOrders => "/spot/orders/active",

            // Contract Trading
            Self::ContractCreateOrder => "/orders",
            Self::ContractAmendOrder => "/orders/replace",
            Self::ContractCancelOrder => "/orders",
            Self::ContractCancelAllOrders => "/orders/all",
            Self::ContractOpenOrders => "/orders/activeList",
            Self::ContractClosedOrders => "/exchange/order/list",
            Self::ContractGetOrder => "/exchange/order",
            Self::ContractGetTrades => "/exchange/order/trade",

            // Hedged Contract Trading
            Self::HedgedCreateOrder => "/g-orders/create",
            Self::HedgedAmendOrder => "/g-orders/replace",
            Self::HedgedCancelOrder => "/g-orders/cancel",

            // Spot Account
            Self::SpotWallets => "/spot/wallets",

            // Contract Account
            Self::ContractAccount => "/accounts/accountPositions",
            Self::Transfer => "/assets/transfer",
            Self::TransferHistory => "/assets/transfer",

            // Positions
            Self::Positions => "/accounts/accountPositions",
            Self::SetLeverage => "/positions/leverage",
            Self::SetRiskLimit => "/positions/riskLimit",
            Self::AssignBalance => "/positions/assign",

            // Custodial Funds
            Self::DepositAddress => "/exchange/wallets/v2/depositAddress",
            Self::Withdraw => "/exchange/wallets/createWithdraw",
            Self::DepositList => "/exchange/wallets/depositList",
            Self::WithdrawList => "/exchange/wallets/withdrawList",

            // Sub Accounts
            Self::SubAccountCreate => "/phemex-user/users/children",
            Self::SubAccountList => "/phemex-user/users/children",
            Self::SubAccountTransfer => "/assets/universal-transfer",
        }
    }

    /// Требует ли endpoint авторизации
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::ServerTime
            | Self::Products
            | Self::SpotOrderbook
            | Self::SpotTrades
            | Self::SpotTicker24h
            | Self::SpotKlines
            | Self::ContractOrderbook
            | Self::ContractTrades
            | Self::ContractTicker24h
            | Self::ContractKlines
            | Self::FundingRateHistory => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        match self {
            // POST endpoints
            Self::SpotCreateOrder
            | Self::ContractCreateOrder
            | Self::HedgedCreateOrder
            | Self::Transfer
            | Self::AssignBalance
            | Self::Withdraw
            | Self::SubAccountCreate
            | Self::SubAccountTransfer => "POST",

            // PUT endpoints
            Self::SpotAmendOrder
            | Self::ContractAmendOrder
            | Self::HedgedAmendOrder
            | Self::SetLeverage
            | Self::SetRiskLimit => "PUT",

            // DELETE endpoints
            Self::SpotCancelOrder
            | Self::SpotCancelAllOrders
            | Self::ContractCancelOrder
            | Self::ContractCancelAllOrders
            | Self::HedgedCancelOrder => "DELETE",

            // GET endpoints (default)
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для Phemex
///
/// # Symbol Formats
/// - Spot: `s{BASE}{QUOTE}` (with lowercase 's' prefix)
/// - Contract (coin-margined): `{BASE}{QUOTE}` (no prefix)
/// - Contract (USDT-margined): `u{BASE}{QUOTE}` (with lowercase 'u' prefix)
///
/// # Examples
/// - Spot: `sBTCUSDT`, `sETHUSDT`
/// - Contract: `BTCUSD`, `ETHUSD`
/// - USDT Contract: `uBTCUSD`, `uETHUSD`
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot => {
            // Spot: s + BASE + QUOTE (uppercase)
            format!("s{}{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Contract: BASE + QUOTE (uppercase, no prefix for coin-margined)
            // USDT-margined contracts use 'u' prefix: uBTCUSD
            // For now, we assume coin-margined (no 'u' prefix)
            // Extended methods can handle USDT-margined specifically
            format!("{}{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::Margin => {
            // Phemex doesn't have dedicated margin trading
            // Use spot format as fallback
            format!("s{}{}", base.to_uppercase(), quote.to_uppercase())
        }
    }
}

/// Map kline interval to Phemex resolution (seconds)
///
/// # Phemex API Format
/// Parameter: `resolution` (integer representing seconds)
///
/// # Supported Intervals
/// - 60, 300, 900, 1800 (minutes)
/// - 3600, 14400 (hours)
/// - 86400 (day)
/// - 604800 (week)
/// - 2592000 (month)
/// - 7776000 (season)
/// - 31104000 (year)
pub fn map_kline_interval(interval: &str) -> u32 {
    match interval {
        "1m" => 60,
        "5m" => 300,
        "15m" => 900,
        "30m" => 1800,
        "1h" => 3600,
        "4h" => 14400,
        "1d" => 86400,
        "1w" => 604800,
        "1M" => 2592000,
        "3M" => 7776000,  // season
        "1y" => 31104000,
        _ => 3600, // default 1 hour
    }
}

// Value scaling helpers
//
// Phemex uses integer representation with scale factors:
// - Ep (Price): scaled by priceScale (typically 4 or 8)
// - Er (Ratio): scaled by ratioScale (typically 8)
// - Ev (Value): scaled by valueScale (typically 4 or 8)

/// Unscale price from Ep format
pub fn unscale_price(price_ep: i64, price_scale: u8) -> f64 {
    price_ep as f64 / 10_f64.powi(price_scale as i32)
}

/// Scale price to Ep format
pub fn scale_price(price: f64, price_scale: u8) -> i64 {
    (price * 10_f64.powi(price_scale as i32)).round() as i64
}

/// Unscale value from Ev format
pub fn unscale_value(value_ev: i64, value_scale: u8) -> f64 {
    value_ev as f64 / 10_f64.powi(value_scale as i32)
}

/// Scale value to Ev format
pub fn scale_value(value: f64, value_scale: u8) -> i64 {
    (value * 10_f64.powi(value_scale as i32)).round() as i64
}

/// Unscale ratio from Er format
pub fn _unscale_ratio(ratio_er: i64, ratio_scale: u8) -> f64 {
    ratio_er as f64 / 10_f64.powi(ratio_scale as i32)
}

/// Scale ratio to Er format
pub fn _scale_ratio(ratio: f64, ratio_scale: u8) -> i64 {
    (ratio * 10_f64.powi(ratio_scale as i32)).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_spot() {
        assert_eq!(format_symbol("BTC", "USDT", AccountType::Spot), "sBTCUSDT");
        assert_eq!(format_symbol("eth", "usdt", AccountType::Spot), "sETHUSDT");
    }

    #[test]
    fn test_format_symbol_contract() {
        assert_eq!(
            format_symbol("BTC", "USD", AccountType::FuturesCross),
            "BTCUSD"
        );
        assert_eq!(
            format_symbol("eth", "usd", AccountType::FuturesIsolated),
            "ETHUSD"
        );
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), 60);
        assert_eq!(map_kline_interval("1h"), 3600);
        assert_eq!(map_kline_interval("1d"), 86400);
        assert_eq!(map_kline_interval("unknown"), 3600);
    }

    #[test]
    fn test_price_scaling() {
        // BTCUSD: priceScale = 4
        let price_ep = 87700000;
        let price = unscale_price(price_ep, 4);
        assert!((price - 8770.0).abs() < f64::EPSILON);

        let scaled = scale_price(8770.0, 4);
        assert_eq!(scaled, 87700000);
    }

    #[test]
    fn test_value_scaling() {
        // BTC: valueScale = 8
        let balance_ev = 100000000;
        let balance = unscale_value(balance_ev, 8);
        assert!((balance - 1.0).abs() < f64::EPSILON);

        let scaled = scale_value(1.0, 8);
        assert_eq!(scaled, 100000000);
    }

    #[test]
    fn test_ratio_scaling() {
        // Leverage 20x: ratioScale = 8
        let leverage_er = 2000000; // 0.02 = 20x when converted
        let ratio = unscale_ratio(leverage_er, 8);
        assert!((ratio - 0.02).abs() < f64::EPSILON);

        let scaled = scale_ratio(0.02, 8);
        assert_eq!(scaled, 2000000);
    }
}
