//! # Lighter Exchange Endpoints
//!
//! URL'ы и endpoint enum для Lighter API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для Lighter API
#[derive(Debug, Clone)]
pub struct LighterUrls {
    pub rest: &'static str,
    pub ws: &'static str,
    pub explorer: &'static str,
}

impl LighterUrls {
    /// Mainnet URLs
    pub const MAINNET: Self = Self {
        rest: "https://mainnet.zklighter.elliot.ai",
        ws: "wss://mainnet.zklighter.elliot.ai/stream",
        explorer: "https://explorer.elliot.ai",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        rest: "https://testnet.zklighter.elliot.ai",
        ws: "wss://testnet.zklighter.elliot.ai/stream",
        explorer: "https://explorer.elliot.ai",
    };

    /// Get REST base URL (same for all account types)
    pub fn rest_url(&self) -> &str {
        self.rest
    }

    /// Get WebSocket URL (same for all account types)
    pub fn ws_url(&self) -> &str {
        self.ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Lighter API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LighterEndpoint {
    // === GENERAL ===
    Status,
    Info,
    CurrentHeight,

    // === MARKET DATA ===
    OrderBooks,
    OrderBookDetails,
    OrderBookOrders,
    RecentTrades,
    Trades,
    Candlesticks,
    Fundings,
    ExchangeStats,

    // === TRADING ===
    SendTx,
    SendTxBatch,
    NextNonce,

    // === ACCOUNT ===
    Account,
    AccountsByL1Address,
    ApiKeys,
    AccountActiveOrders,
    AccountInactiveOrders,
    AccountTxs,
    Pnl,

    // === DEPOSIT/WITHDRAWAL ===
    DepositHistory,
    DepositLatest,
    WithdrawHistory,

    // === BLOCKCHAIN ===
    Block,
    Blocks,
    Transaction,
    Transactions,
    BlockTxs,
    TxFromL1TxHash,

    // === MISC ===
    PublicPools,
    TransferFeeInfo,

    // === MARKET DATA (Extended) ===
    /// GET /api/v1/funding-rates — latest funding rates per market
    FundingRates,
    /// GET /api/v1/exchangeMetrics — aggregate exchange metrics
    ExchangeMetrics,

    // === ACCOUNT (Extended) ===
    /// GET /api/v1/accountLimits — account-level trading limits
    AccountLimits,
    /// GET /api/v1/accountMetadata — account metadata (tier, settings)
    AccountMetadata,
    /// GET /api/v1/positionFunding — per-position funding payments
    PositionFunding,
    /// GET /api/v1/liquidations — account liquidation history
    Liquidations,

    // === CUSTODIAL ===
    /// GET /api/v1/withdrawalDelays — pending withdrawal delay info
    WithdrawalDelays,
}

impl LighterEndpoint {
    /// Get path for endpoint
    pub fn path(&self) -> &'static str {
        match self {
            // General
            Self::Status => "/",
            Self::Info => "/info",
            Self::CurrentHeight => "/api/v1/currentHeight",

            // Market Data
            Self::OrderBooks => "/api/v1/orderBooks",
            Self::OrderBookDetails => "/api/v1/orderBookDetails",
            Self::OrderBookOrders => "/api/v1/orderBookOrders",
            Self::RecentTrades => "/api/v1/recentTrades",
            Self::Trades => "/api/v1/trades",
            Self::Candlesticks => "/api/v1/candles",
            Self::Fundings => "/api/v1/fundings",
            Self::ExchangeStats => "/api/v1/exchangeStats",

            // Trading
            Self::SendTx => "/api/v1/sendTx",
            Self::SendTxBatch => "/api/v1/sendTxBatch",
            Self::NextNonce => "/api/v1/nextNonce",

            // Account
            Self::Account => "/api/v1/account",
            Self::AccountsByL1Address => "/api/v1/accountsByL1Address",
            Self::ApiKeys => "/api/v1/apikeys",
            Self::AccountActiveOrders => "/api/v1/accountActiveOrders",
            Self::AccountInactiveOrders => "/api/v1/accountInactiveOrders",
            Self::AccountTxs => "/api/v1/accountTxs",
            Self::Pnl => "/api/v1/pnl",

            // Deposit/Withdrawal
            Self::DepositHistory => "/api/v1/deposit/history",
            Self::DepositLatest => "/api/v1/deposit/latest",
            Self::WithdrawHistory => "/api/v1/withdraw/history",

            // Blockchain
            Self::Block => "/api/v1/block",
            Self::Blocks => "/api/v1/blocks",
            Self::Transaction => "/api/v1/tx",
            Self::Transactions => "/api/v1/txs",
            Self::BlockTxs => "/api/v1/blockTxs",
            Self::TxFromL1TxHash => "/api/v1/txFromL1TxHash",

            // Misc
            Self::PublicPools => "/api/v1/publicPools",
            Self::TransferFeeInfo => "/api/v1/transferFeeInfo",

            // Market Data (Extended)
            Self::FundingRates => "/api/v1/funding-rates",
            Self::ExchangeMetrics => "/api/v1/exchangeMetrics",

            // Account (Extended)
            Self::AccountLimits => "/api/v1/accountLimits",
            Self::AccountMetadata => "/api/v1/accountMetadata",
            Self::PositionFunding => "/api/v1/positionFunding",
            Self::Liquidations => "/api/v1/liquidations",

            // Custodial
            Self::WithdrawalDelays => "/api/v1/withdrawalDelays",
        }
    }

    /// Does endpoint require authentication
    pub fn requires_auth(&self) -> bool {
        match self {
            // Public endpoints
            Self::Status
            | Self::Info
            | Self::CurrentHeight
            | Self::OrderBooks
            | Self::OrderBookDetails
            | Self::OrderBookOrders
            | Self::RecentTrades
            | Self::Trades
            | Self::Candlesticks
            | Self::Fundings
            | Self::ExchangeStats
            | Self::Block
            | Self::Blocks
            | Self::Transaction
            | Self::Transactions
            | Self::BlockTxs
            | Self::TxFromL1TxHash
            | Self::PublicPools
            | Self::TransferFeeInfo
            | Self::FundingRates
            | Self::ExchangeMetrics => false,

            // Private endpoints
            _ => true,
        }
    }

    /// HTTP method for endpoint
    pub fn method(&self) -> &'static str {
        match self {
            Self::SendTx | Self::SendTxBatch => "POST",
            _ => "GET",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for Lighter
///
/// # Symbol Format
/// - Perpetuals: `ETH` (single asset symbol, quote is USDC)
/// - Spot: `ETH/USDC` (BASE/QUOTE with slash)
///
/// # Examples
/// - Perpetual: `ETH`, `BTC`, `SOL`
/// - Spot: `ETH/USDC`, `BTC/USDC`
pub fn format_symbol(base: &str, quote: &str, account_type: AccountType) -> String {
    match account_type {
        AccountType::Spot | AccountType::Margin => {
            // Spot: BASE/QUOTE with slash
            format!("{}/{}", base.to_uppercase(), quote.to_uppercase())
        }
        AccountType::FuturesCross | AccountType::FuturesIsolated => {
            // Perpetuals: Just base symbol (quote is implied USDC)
            base.to_uppercase()
        }
        _ => {
            // Unsupported account types default to spot format
            format!("{}/{}", base.to_uppercase(), quote.to_uppercase())
        }
    }
}

/// Normalize user input symbol to Lighter format
///
/// # Examples
/// - `BTCUSDC` → `BTC` (for perp) or `BTC/USDC` (for spot)
/// - `ETH-USDC` → `ETH` (for perp) or `ETH/USDC` (for spot)
/// - `eth` → `ETH`
/// - `ETHPERP` → `ETH`
pub fn normalize_symbol(input: &str) -> String {
    let upper = input.to_uppercase();

    // Remove common separators
    let clean = upper.replace(['-', '_'], "");

    // Remove PERP suffix if present
    let clean = if clean.ends_with("PERP") {
        &clean[..clean.len() - 4]
    } else {
        &clean
    };

    // Check if ends with USDC (spot market)
    if clean.ends_with("USDC") && clean.len() > 4 {
        let base = &clean[..clean.len() - 4];
        format!("{}/USDC", base)
    } else {
        clean.to_string()
    }
}

/// Map a symbol base asset to Lighter's numeric market ID.
///
/// Lighter uses numeric market indices for WebSocket channels and REST params:
/// - Perpetuals: 0=ETH, 1=BTC, 2=SOL, etc.
/// - Spot markets start at 2048.
///
/// This is a static mapping derived from actual API data.
pub fn symbol_to_market_id(base: &str) -> Option<u16> {
    match base.to_uppercase().as_str() {
        "ETH" => Some(0),
        "BTC" => Some(1),
        "SOL" => Some(2),
        "ARB" => Some(3),
        "OP" => Some(4),
        "DOGE" => Some(5),
        "MATIC" | "POL" => Some(6),
        "AVAX" => Some(7),
        "LINK" => Some(8),
        "SUI" => Some(9),
        "1000PEPE" | "PEPE" => Some(10),
        "WIF" => Some(11),
        "SEI" => Some(12),
        "AAVE" => Some(13),
        "NEAR" => Some(14),
        "WLD" => Some(15),
        "FTM" | "S" => Some(16),
        "BONK" => Some(17),
        "APT" => Some(19),
        "BNB" => Some(25),
        _ => None,
    }
}

/// Map kline interval to Lighter resolution
///
/// # Supported Resolutions
/// - `1m`, `5m`, `15m`, `1h`, `4h`, `1d`
///
/// # Default
/// Returns `1h` for unsupported intervals
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1m",
        "5m" => "5m",
        "15m" => "15m",
        "1h" | "60m" => "1h",
        "4h" | "240m" => "4h",
        "1d" | "1D" => "1d",
        _ => "1h", // default
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(normalize_symbol("BTCUSDC"), "BTC/USDC");
        assert_eq!(normalize_symbol("ETH-PERP"), "ETH");
        assert_eq!(normalize_symbol("eth"), "ETH");
        assert_eq!(normalize_symbol("SOL-USDC"), "SOL/USDC");
        assert_eq!(normalize_symbol("ETHUSDC"), "ETH/USDC");
    }

    #[test]
    fn test_format_symbol() {
        assert_eq!(
            format_symbol("ETH", "USDC", AccountType::Spot),
            "ETH/USDC"
        );
        assert_eq!(
            format_symbol("ETH", "USDC", AccountType::FuturesCross),
            "ETH"
        );
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "1m");
        assert_eq!(map_kline_interval("1h"), "1h");
        assert_eq!(map_kline_interval("1d"), "1d");
        assert_eq!(map_kline_interval("invalid"), "1h");
    }
}
