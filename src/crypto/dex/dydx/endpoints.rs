//! # dYdX v4 Endpoints
//!
//! URL'ы и endpoint enum для dYdX v4 Indexer API.

use crate::core::types::AccountType;

// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для dYdX v4 API
#[derive(Debug, Clone)]
pub struct DydxUrls {
    pub indexer_rest: &'static str,
    pub indexer_ws: &'static str,
}

impl DydxUrls {
    /// Production URLs (Mainnet)
    pub const MAINNET: Self = Self {
        indexer_rest: "https://indexer.dydx.trade/v4",
        indexer_ws: "wss://indexer.dydx.trade/v4/ws",
    };

    /// Testnet URLs
    pub const TESTNET: Self = Self {
        indexer_rest: "https://indexer.v4testnet.dydx.exchange/v4",
        indexer_ws: "wss://indexer.v4testnet.dydx.exchange/v4/ws",
    };

    /// Получить REST base URL (dYdX только futures, но поддерживаем интерфейс)
    pub fn rest_url(&self, _account_type: AccountType) -> &str {
        self.indexer_rest
    }

    /// Получить WebSocket URL (dYdX только futures)
    pub fn ws_url(&self, _account_type: AccountType) -> &str {
        self.indexer_ws
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// dYdX v4 Indexer API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DydxEndpoint {
    // === GENERAL ===
    ServerTime,
    BlockHeight,

    // === MARKET DATA ===
    PerpetualMarkets,
    Orderbook,
    Trades,
    Candles,
    HistoricalFunding,
    Sparklines,

    // === ACCOUNT ===
    Addresses,
    SpecificSubaccount,
    ParentSubaccount,
    AssetPositions,
    Transfers,
    TradingRewards,
    AggregatedRewards,

    // === POSITIONS ===
    PerpetualPositions,
    ParentPositions,
    HistoricalPnl,
    ParentHistoricalPnl,
    FundingPayments,
    ParentFundingPayments,

    // === TRADING (Read-only via Indexer) ===
    Orders,
    SpecificOrder,
    Fills,
    ParentOrders,
    ParentFills,

    // === COMPLIANCE ===
    ComplianceScreen,

    // === TRANSFERS (Extended) ===
    /// GET /v4/transfers/between — transfers between two subaccounts
    TransfersBetween,
    /// GET /v4/assetPositions/parentSubaccountNumber — asset positions for parent subaccount
    ParentAssetPositions,
    /// GET /v4/transfers/parentSubaccountNumber — transfers for parent subaccount
    ParentTransfers,

    // === VAULT ===
    /// GET /v4/vault/megavault/historicalPnl — MegaVault historical PnL
    MegaVaultPnl,
    /// GET /v4/vault/megavault/positions — MegaVault positions
    MegaVaultPositions,
    /// GET /v4/vault/vaults/historicalPnl — all vaults historical PnL
    AllVaultsPnl,

    // === AFFILIATES ===
    /// GET /v4/affiliates/metadata — affiliate program metadata
    AffiliateMetadata,
    /// GET /v4/affiliates/address — affiliate address info
    AffiliateAddress,
}

impl DydxEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // General
            Self::ServerTime => "/time",
            Self::BlockHeight => "/height",

            // Market Data
            Self::PerpetualMarkets => "/perpetualMarkets",
            Self::Orderbook => "/orderbooks/perpetualMarket/{market}",
            Self::Trades => "/trades/perpetualMarket/{market}",
            Self::Candles => "/candles/perpetualMarkets/{market}",
            Self::HistoricalFunding => "/historicalFunding/{market}",
            Self::Sparklines => "/sparklines",

            // Account
            Self::Addresses => "/addresses/{address}",
            Self::SpecificSubaccount => "/addresses/{address}/subaccountNumber/{subaccount_number}",
            Self::ParentSubaccount => "/addresses/{address}/parentSubaccountNumber/{number}",
            Self::AssetPositions => "/assetPositions",
            Self::Transfers => "/transfers",
            Self::TradingRewards => "/historicalBlockTradingRewards/{address}",
            Self::AggregatedRewards => "/historicalTradingRewardAggregations/{address}",

            // Positions
            Self::PerpetualPositions => "/perpetualPositions",
            Self::ParentPositions => "/perpetualPositions/parentSubaccountNumber",
            Self::HistoricalPnl => "/historical-pnl",
            Self::ParentHistoricalPnl => "/historical-pnl/parentSubaccountNumber",
            Self::FundingPayments => "/fundingPayments",
            Self::ParentFundingPayments => "/fundingPayments/parentSubaccount",

            // Trading
            Self::Orders => "/orders",
            Self::SpecificOrder => "/orders/{orderId}",
            Self::Fills => "/fills",
            Self::ParentOrders => "/orders/parentSubaccountNumber",
            Self::ParentFills => "/fills/parentSubaccountNumber",

            // Compliance
            Self::ComplianceScreen => "/compliance/screen/{address}",

            // Transfers (Extended)
            Self::TransfersBetween => "/transfers/between",
            Self::ParentAssetPositions => "/assetPositions/parentSubaccountNumber",
            Self::ParentTransfers => "/transfers/parentSubaccountNumber",

            // Vault
            Self::MegaVaultPnl => "/vault/megavault/historicalPnl",
            Self::MegaVaultPositions => "/vault/megavault/positions",
            Self::AllVaultsPnl => "/vault/vaults/historicalPnl",

            // Affiliates
            Self::AffiliateMetadata => "/affiliates/metadata",
            Self::AffiliateAddress => "/affiliates/address",
        }
    }

    /// Требует ли endpoint авторизации (dYdX Indexer API все публичные)
    pub fn requires_auth(&self) -> bool {
        // All Indexer endpoints are public
        false
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        // All Indexer endpoints are GET
        "GET"
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Форматирование символа для dYdX
///
/// # dYdX v4 Symbol Format
/// - All perpetual markets: `{BASE}-USD` (e.g., `BTC-USD`, `ETH-USD`)
/// - Case-sensitive: Must be uppercase
/// - Quote asset: Always USDC (shown as USD)
/// - No spot markets (perpetuals only)
///
/// # Examples
/// - `BTC-USD` (Bitcoin perpetual)
/// - `ETH-USD` (Ethereum perpetual)
/// - `SOL-USD` (Solana perpetual)
pub fn format_symbol(base: &str, _quote: &str, _account_type: AccountType) -> String {
    // dYdX v4 only has perpetual markets with USD (USDC) quote
    format!("{}-USD", base.to_uppercase())
}

/// Маппинг интервала kline для dYdX API
///
/// # dYdX API Format
/// Parameter: `resolution` (string)
/// Values: `"1MIN"`, `"5MINS"`, `"15MINS"`, `"30MINS"`, `"1HOUR"`, `"4HOURS"`, `"1DAY"`
pub fn map_kline_interval(interval: &str) -> &'static str {
    match interval {
        "1m" => "1MIN",
        "5m" => "5MINS",
        "15m" => "15MINS",
        "30m" => "30MINS",
        "1h" => "1HOUR",
        "4h" => "4HOURS",
        "1d" => "1DAY",
        _ => "1HOUR", // default
    }
}

/// Нормализовать символ в формат dYdX (uppercase с дефисом)
///
/// # Examples
/// - `"btc-usd"` → `"BTC-USD"`
/// - `"BTC-USD"` → `"BTC-USD"`
/// - `"BTC/USD"` → `"BTC-USD"`
/// - `"BTC"` → `"BTC-USD"`
pub fn normalize_symbol(symbol: &str) -> String {
    let upper = symbol.to_uppercase();

    // Replace / with - if present
    let normalized = if upper.contains('/') {
        upper.replace('/', "-")
    } else if upper.contains('-') {
        upper
    } else {
        format!("{}-USD", upper)
    };

    normalized
}

/// Валидация символа dYdX (должен быть формата BASE-USD)
pub fn _is_valid_symbol(symbol: &str) -> bool {
    symbol.contains('-') && symbol.ends_with("-USD") && symbol == symbol.to_uppercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol() {
        assert_eq!(format_symbol("BTC", "USD", AccountType::FuturesCross), "BTC-USD");
        assert_eq!(format_symbol("eth", "usd", AccountType::FuturesCross), "ETH-USD");
        assert_eq!(format_symbol("SOL", "USDC", AccountType::FuturesCross), "SOL-USD");
    }

    #[test]
    fn test_normalize_symbol() {
        assert_eq!(normalize_symbol("btc-usd"), "BTC-USD");
        assert_eq!(normalize_symbol("BTC-USD"), "BTC-USD");
        assert_eq!(normalize_symbol("eth"), "ETH-USD");
        assert_eq!(normalize_symbol("SOL-USD"), "SOL-USD");
    }

    #[test]
    fn test_is_valid_symbol() {
        assert!(is_valid_symbol("BTC-USD"));
        assert!(is_valid_symbol("ETH-USD"));
        assert!(!is_valid_symbol("btc-usd")); // lowercase
        assert!(!is_valid_symbol("BTC")); // no hyphen
        assert!(!is_valid_symbol("BTCUSD")); // no hyphen
        assert!(!is_valid_symbol("BTC/USD")); // wrong separator
    }

    #[test]
    fn test_map_kline_interval() {
        assert_eq!(map_kline_interval("1m"), "1MIN");
        assert_eq!(map_kline_interval("5m"), "5MINS");
        assert_eq!(map_kline_interval("1h"), "1HOUR");
        assert_eq!(map_kline_interval("4h"), "4HOURS");
        assert_eq!(map_kline_interval("1d"), "1DAY");
    }
}
