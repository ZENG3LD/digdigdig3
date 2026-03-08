//! # MOEX ISS API Endpoints
//!
//! Moscow Exchange Informational & Statistical Server endpoint definitions.

use crate::core::types::Symbol;

// ═══════════════════════════════════════════════════════════════════════════════
// BASE URLS
// ═══════════════════════════════════════════════════════════════════════════════

/// MOEX ISS API base URLs
#[derive(Debug, Clone)]
pub struct MoexEndpoints {
    /// REST API base URL
    pub rest_base: &'static str,
    /// WebSocket base URL (STOMP protocol)
    pub ws_base: Option<&'static str>,
}

impl Default for MoexEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://iss.moex.com/iss",
            ws_base: Some("wss://iss.moex.com/infocx/v3/websocket"),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINT ENUM
// ═══════════════════════════════════════════════════════════════════════════════

/// MOEX ISS API endpoints
///
/// Note: MOEX has 400+ endpoints. This enum covers the most commonly used ones
/// for market data retrieval. Endpoints are organized by category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MoexEndpoint {
    // === METADATA ===
    /// GET /engines - List all trading engines
    Engines,
    /// GET /engines/{engine}/markets - Markets within engine
    EngineMarkets,
    /// GET /engines/{engine}/markets/{market}/boards - Trading boards
    MarketBoards,

    // === SECURITIES ===
    /// GET /securities - List all securities
    Securities,
    /// GET /securities/{security} - Security specification
    SecurityInfo,

    // === CURRENT MARKET DATA ===
    /// GET /engines/{engine}/markets/{market}/securities - Current session data
    MarketSecurities,
    /// GET /engines/{engine}/markets/{market}/securities/{security} - Specific security current data
    SecurityMarketData,
    /// GET /engines/{engine}/markets/{market}/boards/{board}/securities/{security} - Board-specific data
    BoardSecurityData,
    /// GET /engines/{engine}/markets/{market}/securities/{security}/trades - Recent trades
    SecurityTrades,
    /// GET /engines/{engine}/markets/{market}/securities/{security}/orderbook - Orderbook (requires subscription)
    SecurityOrderbook,

    // === HISTORICAL DATA (OHLC/CANDLES) ===
    /// GET /engines/{engine}/markets/{market}/securities/{security}/candles - Candles
    Candles,
    /// GET /engines/{engine}/markets/{market}/boards/{board}/securities/{security}/candles - Board candles
    BoardCandles,
    /// GET /engines/{engine}/markets/{market}/securities/{security}/candleborders - Candle date range
    CandleBorders,

    // === HISTORICAL TRADING DATA ===
    /// GET /history/engines/{engine}/markets/{market}/securities/{security} - Historical data
    HistoricalData,
    /// GET /history/engines/{engine}/markets/{market}/boards/{board}/securities/{security} - Board history
    BoardHistory,

    // === INDICES ===
    /// GET /statistics/engines/stock/markets/index/analytics - Stock indices
    StockIndices,
    /// GET /statistics/engines/stock/markets/index/analytics/{indexid} - Index data by date
    IndexAnalytics,

    // === DERIVATIVES (FUTURES & OPTIONS) ===
    /// GET /statistics/engines/futures/markets/forts/series - Futures list
    FuturesSeries,
    /// GET /statistics/engines/futures/markets/options/assets - Option series
    OptionsSeries,
    /// GET /statistics/engines/futures/markets/{market}/openpositions/{asset} - Open interest
    OpenInterest,

    // === STATISTICS ===
    /// GET /turnovers - Market turnovers
    Turnovers,
    /// GET /engines/{engine}/turnovers - Engine turnovers
    EngineTurnovers,

    // === CORPORATE INFORMATION (CCI) ===
    /// GET /cci/info/companies - Company information
    CompanyInfo,
    /// GET /cci/corp-actions - Corporate actions
    CorporateActions,
    /// GET /cci/consensus/shares-price - Analyst consensus
    ConsensusForecasts,
}

impl MoexEndpoint {
    /// Get endpoint path template
    ///
    /// Some endpoints contain placeholders like {engine}, {market}, {security}
    /// which need to be replaced with actual values.
    pub fn path(&self) -> &'static str {
        match self {
            // Metadata
            Self::Engines => "/engines.json",
            Self::EngineMarkets => "/engines/{engine}/markets.json",
            Self::MarketBoards => "/engines/{engine}/markets/{market}/boards.json",

            // Securities
            Self::Securities => "/securities.json",
            Self::SecurityInfo => "/securities/{security}.json",

            // Current market data
            Self::MarketSecurities => "/engines/{engine}/markets/{market}/securities.json",
            Self::SecurityMarketData => "/engines/{engine}/markets/{market}/securities/{security}.json",
            Self::BoardSecurityData => "/engines/{engine}/markets/{market}/boards/{board}/securities/{security}.json",
            Self::SecurityTrades => "/engines/{engine}/markets/{market}/securities/{security}/trades.json",
            Self::SecurityOrderbook => "/engines/{engine}/markets/{market}/securities/{security}/orderbook.json",

            // Historical data (candles)
            Self::Candles => "/engines/{engine}/markets/{market}/securities/{security}/candles.json",
            Self::BoardCandles => "/engines/{engine}/markets/{market}/boards/{board}/securities/{security}/candles.json",
            Self::CandleBorders => "/engines/{engine}/markets/{market}/securities/{security}/candleborders.json",

            // Historical trading data
            Self::HistoricalData => "/history/engines/{engine}/markets/{market}/securities/{security}.json",
            Self::BoardHistory => "/history/engines/{engine}/markets/{market}/boards/{board}/securities/{security}.json",

            // Indices
            Self::StockIndices => "/statistics/engines/stock/markets/index/analytics.json",
            Self::IndexAnalytics => "/statistics/engines/stock/markets/index/analytics/{indexid}.json",

            // Derivatives
            Self::FuturesSeries => "/statistics/engines/futures/markets/forts/series.json",
            Self::OptionsSeries => "/statistics/engines/futures/markets/options/assets.json",
            Self::OpenInterest => "/statistics/engines/futures/markets/{market}/openpositions/{asset}.json",

            // Statistics
            Self::Turnovers => "/turnovers.json",
            Self::EngineTurnovers => "/engines/{engine}/turnovers.json",

            // Corporate information
            Self::CompanyInfo => "/cci/info/companies.json",
            Self::CorporateActions => "/cci/corp-actions.json",
            Self::ConsensusForecasts => "/cci/consensus/shares-price.json",
        }
    }

    /// Build full URL with path parameters replaced
    ///
    /// # Example
    /// ```ignore
    /// let url = MoexEndpoint::SecurityMarketData.build_url(&[
    ///     ("engine", "stock"),
    ///     ("market", "shares"),
    ///     ("security", "SBER"),
    /// ]);
    /// // Returns: "/engines/stock/markets/shares/securities/SBER.json"
    /// ```
    pub fn build_path(&self, params: &[(&str, &str)]) -> String {
        let mut path = self.path().to_string();
        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            path = path.replace(&placeholder, value);
        }
        path
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SYMBOL FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format symbol for MOEX ISS API
///
/// MOEX uses ticker symbols without quote currency for stocks.
/// Examples:
/// - Stocks: "SBER", "GAZP", "LKOH"
/// - Indices: "IMOEX", "RTSI"
/// - Futures: Contract codes like "SiZ5" (Silver, December 2025)
///
/// The Symbol type from core contains base and quote, but MOEX typically
/// only needs the base (ticker) for stocks.
pub fn format_symbol(symbol: &Symbol) -> String {
    // For Russian stocks, just use the base (ticker)
    symbol.base.to_uppercase()
}

/// Parse symbol from MOEX format back to domain Symbol
///
/// MOEX returns security IDs (SECID) which are typically just tickers.
/// We create a Symbol with the ticker as base and "RUB" as quote (default).
pub fn _parse_symbol(api_symbol: &str) -> Symbol {
    Symbol {
        base: api_symbol.to_uppercase(),
        quote: "RUB".to_string(), // Default quote currency for Russian stocks
        raw: Some(api_symbol.to_string()),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTERVAL MAPPING
// ═══════════════════════════════════════════════════════════════════════════════

/// Map standard interval format to MOEX interval codes
///
/// MOEX supports:
/// - Integer minutes: 1, 10, 60, 24*60 (1440), 7*24*60 (10080), 31*24*60 (44640), 4*31*24*60 (178560)
/// - Text codes: m1, m10, H1, D1, W1, M1, Q1
///
/// We use integer minutes for simplicity.
pub fn map_interval(interval: &str) -> i32 {
    match interval {
        "1m" => 1,
        "10m" => 10,
        "1h" | "60m" => 60,
        "1d" => 24 * 60,        // 1440 minutes
        "1w" => 7 * 24 * 60,    // 10080 minutes
        "1M" => 31 * 24 * 60,   // 44640 minutes (approximation)
        "1Q" => 4 * 31 * 24 * 60, // 178560 minutes (approximation)
        _ => 60, // default to 1 hour
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENGINE AND MARKET HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Default engine and market for stock trading
///
/// Most stock queries use:
/// - Engine: "stock" (Фондовый рынок)
/// - Market: "shares" (Акции)
/// - Board: "TQBR" (T+ main board)
pub const DEFAULT_ENGINE: &str = "stock";
pub const DEFAULT_MARKET: &str = "shares";
pub const DEFAULT_BOARD: &str = "TQBR";

/// Get default engine, market, and board for stock queries
pub fn default_stock_params() -> (&'static str, &'static str, &'static str) {
    (DEFAULT_ENGINE, DEFAULT_MARKET, DEFAULT_BOARD)
}
