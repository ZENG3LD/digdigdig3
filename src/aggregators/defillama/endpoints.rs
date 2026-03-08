//! # DefiLlama Endpoints
//!
//! URL'ы и endpoint enum для DefiLlama API.
//!
//! ## URL Authentication
//!
//! DefiLlama uses API key in URL path (unique pattern):
//! - Free tier: https://api.llama.fi/<endpoint>
//! - Pro tier: https://pro-api.llama.fi/<API_KEY>/<endpoint>
//!
//! ## Data Update Frequency
//!
//! - TVL: Hourly updates
//! - Yields: Hourly updates
//! - Prices: Hourly updates
//! - Stablecoins: Hourly updates
//!
//! ## Endpoints
//!
//! Free tier (29 endpoints):
//! - Protocol TVL, historical data
//! - Token prices
//! - Stablecoin data
//! - Basic chain stats
//!
//! Pro tier only (35 additional endpoints):
//! - Advanced analytics
//! - Historical aggregates
//! - Premium data feeds


// ═══════════════════════════════════════════════════════════════════════════════
// URLs
// ═══════════════════════════════════════════════════════════════════════════════

/// URL'ы для DefiLlama API
///
/// DefiLlama uses different subdomains for different data categories:
/// - `api.llama.fi` - TVL, protocols
/// - `coins.llama.fi` - Token prices
/// - `stablecoins.llama.fi` - Stablecoin data
/// - `yields.llama.fi` - Yield pool data
#[derive(Debug, Clone)]
pub struct DefiLlamaUrls {
    pub api_base: &'static str,
    pub coins_base: &'static str,
    pub stablecoins_base: &'static str,
    pub yields_base: &'static str,
    pub pro_base: &'static str,
}

impl DefiLlamaUrls {
    /// Production URLs
    pub const MAINNET: Self = Self {
        api_base: "https://api.llama.fi",
        coins_base: "https://coins.llama.fi",
        stablecoins_base: "https://stablecoins.llama.fi",
        yields_base: "https://yields.llama.fi",
        pro_base: "https://pro-api.llama.fi",
    };

    /// Build URL for the given endpoint category and path
    pub fn build_url(&self, api_key: Option<&str>, category: EndpointCategory, endpoint_path: &str) -> String {
        match api_key {
            Some(key) if !key.is_empty() => {
                format!("{}/{}{}", self.pro_base, key, endpoint_path)
            }
            _ => {
                let base = match category {
                    EndpointCategory::Api => self.api_base,
                    EndpointCategory::Coins => self.coins_base,
                    EndpointCategory::Stablecoins => self.stablecoins_base,
                    EndpointCategory::Yields => self.yields_base,
                };
                format!("{}{}", base, endpoint_path)
            }
        }
    }
}

/// Which subdomain an endpoint belongs to
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndpointCategory {
    /// api.llama.fi - TVL, protocols, fees, volumes
    Api,
    /// coins.llama.fi - Token prices
    Coins,
    /// stablecoins.llama.fi - Stablecoin data
    Stablecoins,
    /// yields.llama.fi - Yield pool data
    Yields,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENDPOINTS
// ═══════════════════════════════════════════════════════════════════════════════

/// DefiLlama API endpoints
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DefiLlamaEndpoint {
    // === PROTOCOLS (Free Tier) ===
    /// Get all protocols
    Protocols,
    /// Get single protocol by slug
    Protocol,
    /// Get historical TVL for protocol
    ProtocolTvl,

    // === TVL (Free Tier) ===
    /// Get current TVL for all chains
    TvlAll,
    /// Get historical TVL for chain
    ChainTvl,

    // === PRICES (Free Tier) ===
    /// Get current prices for tokens
    PricesCurrent,
    /// Get historical prices
    PricesHistorical,
    /// Get first price record
    PricesFirst,

    // === STABLECOINS (Free Tier) ===
    /// Get all stablecoins
    Stablecoins,
    /// Get single stablecoin
    Stablecoin,
    /// Get stablecoin charts
    StablecoinCharts,
    /// Get stablecoin chain data
    StablecoinChain,

    // === YIELDS (Free Tier) ===
    /// Get all pools
    YieldPools,
    /// Get pool chart
    YieldPoolChart,

    // === FEES & REVENUE (Free Tier) ===
    /// Get protocol fees
    ProtocolFees,

    // === VOLUMES (Free Tier) ===
    /// Get DEX volumes
    DexVolumes,

    // === PRO TIER ONLY ===
    /// Advanced analytics (requires $300/mo tier)
    ProAnalytics,
}

impl DefiLlamaEndpoint {
    /// Получить путь endpoint'а
    pub fn path(&self) -> &'static str {
        match self {
            // Protocols
            Self::Protocols => "/protocols",
            Self::Protocol => "/protocol/{protocol}",
            Self::ProtocolTvl => "/tvl/{protocol}",

            // TVL
            Self::TvlAll => "/v2/chains",
            Self::ChainTvl => "/v2/historicalChainTvl/{chain}",

            // Prices
            Self::PricesCurrent => "/prices/current/{coins}",
            Self::PricesHistorical => "/prices/historical/{timestamp}/{coins}",
            Self::PricesFirst => "/prices/first/{coins}",

            // Stablecoins
            Self::Stablecoins => "/stablecoins",
            Self::Stablecoin => "/stablecoin/{id}",
            Self::StablecoinCharts => "/stablecoincharts/all",
            Self::StablecoinChain => "/stablecoinchains",

            // Yields
            Self::YieldPools => "/pools",
            Self::YieldPoolChart => "/chart/{pool}",

            // Fees & Revenue
            Self::ProtocolFees => "/summary/fees/{protocol}",

            // Volumes
            Self::DexVolumes => "/overview/dexs",

            // Pro tier
            Self::ProAnalytics => "/analytics/{type}",
        }
    }

    /// Which subdomain category this endpoint belongs to
    pub fn category(&self) -> EndpointCategory {
        match self {
            // Prices -> coins.llama.fi
            Self::PricesCurrent | Self::PricesHistorical | Self::PricesFirst => EndpointCategory::Coins,

            // Stablecoins -> stablecoins.llama.fi
            Self::Stablecoins | Self::Stablecoin | Self::StablecoinCharts | Self::StablecoinChain => EndpointCategory::Stablecoins,

            // Yields -> yields.llama.fi
            Self::YieldPools | Self::YieldPoolChart => EndpointCategory::Yields,

            // Everything else -> api.llama.fi
            _ => EndpointCategory::Api,
        }
    }

    /// Требует ли endpoint Pro tier ($300/mo)
    pub fn requires_pro_tier(&self) -> bool {
        matches!(self, Self::ProAnalytics)
    }

    /// HTTP метод для endpoint'а
    pub fn method(&self) -> &'static str {
        // DefiLlama API - все GET запросы
        "GET"
    }

    /// Рекомендуемый интервал обновления (секунды)
    pub fn update_interval_seconds(&self) -> u64 {
        match self {
            // TVL/Yields обновляются раз в час
            Self::ProtocolTvl
            | Self::TvlAll
            | Self::ChainTvl
            | Self::YieldPools
            | Self::YieldPoolChart => 3600,

            // Prices обновляются чаще
            Self::PricesCurrent
            | Self::PricesHistorical => 300, // 5 минут

            // Metadata обновляется редко
            Self::Protocols
            | Self::Protocol
            | Self::Stablecoins
            | Self::Stablecoin => 86400, // 1 день

            // Analytics
            _ => 3600,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PROTOCOL ID FORMATTING
// ═══════════════════════════════════════════════════════════════════════════════

/// Format protocol slug for DefiLlama API
///
/// # Examples
/// - "aave" (lowercase)
/// - "uniswap"
/// - "curve-finance"
/// - "lido"
pub fn format_protocol_slug(protocol: &str) -> String {
    protocol.to_lowercase().replace(' ', "-")
}

/// Format chain name for DefiLlama API
///
/// # Examples
/// - "ethereum" (lowercase)
/// - "bsc" (Binance Smart Chain)
/// - "polygon"
/// - "arbitrum"
pub fn format_chain_name(chain: &str) -> String {
    match chain.to_lowercase().as_str() {
        "ethereum" | "eth" => "ethereum".to_string(),
        "binance" | "bsc" | "bnb" => "bsc".to_string(),
        "polygon" | "matic" => "polygon".to_string(),
        other => other.to_string(),
    }
}

/// Format token address for prices endpoint
///
/// # Format: {chain}:{address}
/// # Examples
/// - "ethereum:0x6b175474e89094c44da98b954eedeac495271d0f" (DAI)
/// - "bsc:0x55d398326f99059ff775485246999027b3197955" (USDT on BSC)
pub fn format_coin_id(chain: &str, address: &str) -> String {
    format!("{}:{}", format_chain_name(chain), address.to_lowercase())
}

/// Format multiple coin IDs for batch price queries
///
/// # Example: "ethereum:0xabc,bsc:0xdef"
pub fn format_coin_ids(coins: &[(String, String)]) -> String {
    coins
        .iter()
        .map(|(chain, address)| format_coin_id(chain, address))
        .collect::<Vec<_>>()
        .join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_url_building() {
        let urls = DefiLlamaUrls::MAINNET;

        // Free tier - API
        let free_url = urls.build_url(None, EndpointCategory::Api, "/protocols");
        assert_eq!(free_url, "https://api.llama.fi/protocols");

        // Free tier - Coins
        let coins_url = urls.build_url(None, EndpointCategory::Coins, "/prices/current/coingecko:bitcoin");
        assert_eq!(coins_url, "https://coins.llama.fi/prices/current/coingecko:bitcoin");

        // Free tier - Stablecoins
        let stable_url = urls.build_url(None, EndpointCategory::Stablecoins, "/stablecoins");
        assert_eq!(stable_url, "https://stablecoins.llama.fi/stablecoins");

        // Free tier - Yields
        let yields_url = urls.build_url(None, EndpointCategory::Yields, "/pools");
        assert_eq!(yields_url, "https://yields.llama.fi/pools");

        // Pro tier
        let pro_url = urls.build_url(Some("test_key"), EndpointCategory::Api, "/protocols");
        assert_eq!(pro_url, "https://pro-api.llama.fi/test_key/protocols");
    }

    #[test]
    fn test_protocol_slug_formatting() {
        assert_eq!(format_protocol_slug("Aave"), "aave");
        assert_eq!(format_protocol_slug("Curve Finance"), "curve-finance");
    }

    #[test]
    fn test_chain_name_formatting() {
        assert_eq!(format_chain_name("ETH"), "ethereum");
        assert_eq!(format_chain_name("BSC"), "bsc");
        assert_eq!(format_chain_name("Polygon"), "polygon");
    }

    #[test]
    fn test_coin_id_formatting() {
        let coin_id = format_coin_id("ethereum", "0x6B175474E89094C44Da98b954EedeAC495271d0F");
        assert_eq!(coin_id, "ethereum:0x6b175474e89094c44da98b954eedeac495271d0f");
    }
}
