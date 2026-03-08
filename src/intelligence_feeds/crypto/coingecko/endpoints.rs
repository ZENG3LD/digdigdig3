//! CoinGecko API endpoints

/// Base URLs for CoinGecko API
pub struct CoinGeckoEndpoints {
    pub rest_base: &'static str,
    pub ws_base: Option<&'static str>,
}

impl Default for CoinGeckoEndpoints {
    fn default() -> Self {
        Self {
            rest_base: "https://api.coingecko.com/api/v3",
            ws_base: None, // CoinGecko does not support WebSocket
        }
    }
}

/// CoinGecko API endpoint enum
#[derive(Debug, Clone)]
pub enum CoinGeckoEndpoint {
    // ═══════════════════════════════════════════════════════════════════════
    // SIMPLE ENDPOINTS (1)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get simple price for one or more coins
    /// GET /simple/price?ids=bitcoin&vs_currencies=usd
    SimplePrice,

    // ═══════════════════════════════════════════════════════════════════════
    // COINS ENDPOINTS (5)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get list of all coins
    /// GET /coins/list
    CoinsList,

    /// Get coin details by ID
    /// GET /coins/{id}
    CoinDetail,

    /// Get market chart data (price/volume history)
    /// GET /coins/{id}/market_chart?vs_currency=usd&days=30
    CoinMarketChart,

    /// Get coins market data (paginated)
    /// GET /coins/markets?vs_currency=usd&order=market_cap_desc
    CoinsMarkets,

    /// Get coin tickers on exchanges
    /// GET /coins/{id}/tickers
    CoinTickers,

    // ═══════════════════════════════════════════════════════════════════════
    // SEARCH ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Search for coins/exchanges
    /// GET /search?query=bitcoin
    Search,

    /// Get trending coins
    /// GET /search/trending
    SearchTrending,

    // ═══════════════════════════════════════════════════════════════════════
    // GLOBAL ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get global market data
    /// GET /global
    Global,

    /// Get DeFi market data
    /// GET /global/decentralized_finance_defi
    GlobalDefi,

    // ═══════════════════════════════════════════════════════════════════════
    // EXCHANGE ENDPOINTS (2)
    // ═══════════════════════════════════════════════════════════════════════
    /// Get list of exchanges
    /// GET /exchanges?per_page=100
    Exchanges,

    /// Get exchange details by ID
    /// GET /exchanges/{id}
    ExchangeDetail,
}

impl CoinGeckoEndpoint {
    /// Get endpoint path
    pub fn path(&self) -> &'static str {
        match self {
            // Simple
            Self::SimplePrice => "/simple/price",

            // Coins
            Self::CoinsList => "/coins/list",
            Self::CoinDetail => "/coins", // /{id} appended in request
            Self::CoinMarketChart => "/coins", // /{id}/market_chart appended
            Self::CoinsMarkets => "/coins/markets",
            Self::CoinTickers => "/coins", // /{id}/tickers appended

            // Search
            Self::Search => "/search",
            Self::SearchTrending => "/search/trending",

            // Global
            Self::Global => "/global",
            Self::GlobalDefi => "/global/decentralized_finance_defi",

            // Exchanges
            Self::Exchanges => "/exchanges",
            Self::ExchangeDetail => "/exchanges", // /{id} appended
        }
    }

    /// Build full path with ID parameter if needed
    pub fn build_path(&self, id: Option<&str>, suffix: Option<&str>) -> String {
        let base = self.path();
        match (id, suffix) {
            (Some(id), Some(suf)) => format!("{}/{}/{}", base, id, suf),
            (Some(id), None) => format!("{}/{}", base, id),
            (None, _) => base.to_string(),
        }
    }
}
