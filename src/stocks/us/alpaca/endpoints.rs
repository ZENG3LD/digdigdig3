//! Alpaca API endpoints

use crate::core::types::Symbol;

/// Base URLs for Alpaca
pub struct AlpacaEndpoints {
    pub trading_base: &'static str,
    pub market_data_base: &'static str,
    pub _ws_market_data: &'static str,
    pub _ws_trading: &'static str,
}

impl AlpacaEndpoints {
    /// Production (live trading)
    pub fn live() -> Self {
        Self {
            trading_base: "https://api.alpaca.markets",
            market_data_base: "https://data.alpaca.markets",
            _ws_market_data: "wss://stream.data.alpaca.markets",
            _ws_trading: "wss://api.alpaca.markets/stream",
        }
    }

    /// Paper trading (free, global)
    pub fn paper() -> Self {
        Self {
            trading_base: "https://paper-api.alpaca.markets",
            market_data_base: "https://data.alpaca.markets",
            _ws_market_data: "wss://stream.data.alpaca.markets",
            _ws_trading: "wss://paper-api.alpaca.markets/stream",
        }
    }

    /// Sandbox (development)
    pub fn _sandbox() -> Self {
        Self {
            trading_base: "https://paper-api.alpaca.markets",
            market_data_base: "https://data.sandbox.alpaca.markets",
            _ws_market_data: "wss://stream.data.sandbox.alpaca.markets",
            _ws_trading: "wss://paper-api.alpaca.markets/stream",
        }
    }
}

impl Default for AlpacaEndpoints {
    fn default() -> Self {
        Self::paper() // Safe default for testing
    }
}

/// API endpoint enum
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum AlpacaEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // TRADING API - Account Management
    // ═══════════════════════════════════════════════════════════════════════
    Account,
    AccountPortfolioHistory,
    AccountActivities,

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING API - Orders
    // ═══════════════════════════════════════════════════════════════════════
    Orders,
    OrderById(String),
    OrderByClientId(String),

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING API - Positions
    // ═══════════════════════════════════════════════════════════════════════
    Positions,
    PositionBySymbol(String),

    // ═══════════════════════════════════════════════════════════════════════
    // TRADING API - Assets & Market Info
    // ═══════════════════════════════════════════════════════════════════════
    Assets,
    AssetBySymbol(String),
    OptionContracts,
    Calendar,
    Clock,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Stock Bars
    // ═══════════════════════════════════════════════════════════════════════
    StockBars,
    StockBarsLatest,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Stock Trades
    // ═══════════════════════════════════════════════════════════════════════
    StockTrades,
    StockTradesLatest,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Stock Quotes
    // ═══════════════════════════════════════════════════════════════════════
    StockQuotes,
    StockQuotesLatest,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Snapshots
    // ═══════════════════════════════════════════════════════════════════════
    StockSnapshots,
    StockSnapshotBySymbol(String),

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Options
    // ═══════════════════════════════════════════════════════════════════════
    OptionsSnapshots(String), // underlying symbol
    OptionsBars,
    OptionsTrades,
    OptionsQuotes,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Crypto
    // ═══════════════════════════════════════════════════════════════════════
    CryptoBars,
    CryptoTrades,
    CryptoQuotes,
    CryptoOrderbooks,
    CryptoSnapshots,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - News
    // ═══════════════════════════════════════════════════════════════════════
    News,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Corporate Actions
    // ═══════════════════════════════════════════════════════════════════════
    CorporateActions,

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA API - Screener
    // ═══════════════════════════════════════════════════════════════════════
    Movers,
}

impl AlpacaEndpoint {
    /// Get endpoint path and base URL type
    ///
    /// Returns a String instead of &'static str because some paths are dynamic
    pub fn path(&self) -> (String, EndpointBase) {
        match self {
            // Trading API
            Self::Account => ("/v2/account".to_string(), EndpointBase::Trading),
            Self::AccountPortfolioHistory => ("/v2/account/portfolio/history".to_string(), EndpointBase::Trading),
            Self::AccountActivities => ("/v2/account/activities".to_string(), EndpointBase::Trading),

            Self::Orders => ("/v2/orders".to_string(), EndpointBase::Trading),
            Self::OrderById(id) => (format!("/v2/orders/{}", id), EndpointBase::Trading),
            Self::OrderByClientId(id) => (format!("/v2/orders:by_client_order_id?client_order_id={}", id), EndpointBase::Trading),

            Self::Positions => ("/v2/positions".to_string(), EndpointBase::Trading),
            Self::PositionBySymbol(symbol) => (format!("/v2/positions/{}", symbol), EndpointBase::Trading),

            Self::Assets => ("/v2/assets".to_string(), EndpointBase::Trading),
            Self::AssetBySymbol(symbol) => (format!("/v2/assets/{}", symbol), EndpointBase::Trading),
            Self::OptionContracts => ("/v2/option_contracts".to_string(), EndpointBase::Trading),
            Self::Calendar => ("/v2/calendar".to_string(), EndpointBase::Trading),
            Self::Clock => ("/v2/clock".to_string(), EndpointBase::Trading),

            // Market Data API - Stocks
            Self::StockBars => ("/v2/stocks/bars".to_string(), EndpointBase::MarketData),
            Self::StockBarsLatest => ("/v2/stocks/bars/latest".to_string(), EndpointBase::MarketData),

            Self::StockTrades => ("/v2/stocks/trades".to_string(), EndpointBase::MarketData),
            Self::StockTradesLatest => ("/v2/stocks/trades/latest".to_string(), EndpointBase::MarketData),

            Self::StockQuotes => ("/v2/stocks/quotes".to_string(), EndpointBase::MarketData),
            Self::StockQuotesLatest => ("/v2/stocks/quotes/latest".to_string(), EndpointBase::MarketData),

            Self::StockSnapshots => ("/v2/stocks/snapshots".to_string(), EndpointBase::MarketData),
            Self::StockSnapshotBySymbol(symbol) => (format!("/v2/stocks/{}/snapshot", symbol), EndpointBase::MarketData),

            // Market Data API - Options
            Self::OptionsSnapshots(underlying) => (format!("/v1beta1/options/snapshots/{}", underlying), EndpointBase::MarketData),
            Self::OptionsBars => ("/v1beta1/options/bars".to_string(), EndpointBase::MarketData),
            Self::OptionsTrades => ("/v1beta1/options/trades".to_string(), EndpointBase::MarketData),
            Self::OptionsQuotes => ("/v1beta1/options/quotes".to_string(), EndpointBase::MarketData),

            // Market Data API - Crypto
            Self::CryptoBars => ("/v1beta3/crypto/us/bars".to_string(), EndpointBase::MarketData),
            Self::CryptoTrades => ("/v1beta3/crypto/us/trades".to_string(), EndpointBase::MarketData),
            Self::CryptoQuotes => ("/v1beta3/crypto/us/quotes".to_string(), EndpointBase::MarketData),
            Self::CryptoOrderbooks => ("/v1beta3/crypto/us/latest/orderbooks".to_string(), EndpointBase::MarketData),
            Self::CryptoSnapshots => ("/v1beta3/crypto/us/snapshots".to_string(), EndpointBase::MarketData),

            // Market Data API - News & Corporate Actions
            Self::News => ("/v1beta1/news".to_string(), EndpointBase::MarketData),
            Self::CorporateActions => ("/v1beta1/corporate-actions/announcements".to_string(), EndpointBase::MarketData),

            // Market Data API - Screener
            Self::Movers => ("/v1beta1/screener/stocks/movers".to_string(), EndpointBase::MarketData),
        }
    }
}

/// Endpoint base URL type
#[derive(Debug, Clone, Copy)]
pub enum EndpointBase {
    Trading,
    MarketData,
}

/// Format symbol for Alpaca API
///
/// Alpaca uses different formats for different asset classes:
/// - Stocks: Just ticker symbol (e.g., "AAPL")
/// - Crypto: Base/Quote format (e.g., "BTC/USD")
pub fn format_symbol(symbol: &Symbol) -> String {
    // Crypto symbols are detected by short base names (BTC, ETH, etc.)
    // Stock tickers are typically longer (AAPL, MSFT, etc.)
    // Common crypto bases: BTC, ETH, SOL, AVAX, etc.

    let common_crypto = [
        "BTC", "ETH", "SOL", "AVAX", "DOGE", "LTC", "BCH", "XLM",
        "LINK", "UNI", "AAVE", "SUSHI", "SHIB", "DOT", "MATIC", "ADA",
        "XRP", "BNB", "USDT", "USDC", "DAI"
    ];

    let base_upper = symbol.base.to_uppercase();

    // If it's a known crypto, always use BASE/QUOTE format
    if common_crypto.contains(&base_upper.as_str()) {
        format!("{}/{}", base_upper, symbol.quote.to_uppercase())
    } else if symbol.quote.is_empty() || symbol.quote == "USD" {
        // Stock ticker - just base
        base_upper
    } else {
        // Other crypto or forex - use BASE/QUOTE format
        format!("{}/{}", base_upper, symbol.quote.to_uppercase())
    }
}

/// Parse symbol from Alpaca API format back to domain Symbol
pub fn _parse_symbol(api_symbol: &str) -> Symbol {
    if let Some((base, quote)) = api_symbol.split_once('/') {
        // Crypto format: "BTC/USD"
        Symbol::new(base, quote)
    } else {
        // Stock format: "AAPL" (assume USD quote)
        Symbol::new(api_symbol, "USD")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_symbol_stock() {
        let symbol = Symbol::new("AAPL", "USD");
        assert_eq!(format_symbol(&symbol), "AAPL");
    }

    #[test]
    fn test_format_symbol_crypto() {
        let symbol = Symbol::new("BTC", "USD");
        assert_eq!(format_symbol(&symbol), "BTC/USD");
    }

    #[test]
    fn test_parse_symbol_stock() {
        let symbol = _parse_symbol("AAPL");
        assert_eq!(symbol.base, "AAPL");
        assert_eq!(symbol.quote, "USD");
    }

    #[test]
    fn test_parse_symbol_crypto() {
        let symbol = _parse_symbol("BTC/USD");
        assert_eq!(symbol.base, "BTC");
        assert_eq!(symbol.quote, "USD");
    }
}
