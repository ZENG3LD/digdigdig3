//! # DefiLlama Connector
//!
//! DefiLlama aggregator connector для TVL, protocol data, и token prices.
//!
//! ## Ключевые отличия от CEX коннекторов
//!
//! 1. **NO WebSocket** - только REST API с polling
//! 2. **NO Trading** - DeFi aggregator, не биржа
//! 3. **Protocol-centric** - работает с protocol IDs вместо trading pairs
//! 4. **Hourly updates** - данные обновляются раз в час
//!
//! ## Supported Operations
//!
//! - ✅ Protocol TVL data
//! - ✅ Token prices
//! - ✅ Stablecoin data
//! - ✅ Yield pool data
//! - ❌ Trading (UnsupportedOperation)
//! - ❌ Account balance (UnsupportedOperation)
//! - ❌ WebSocket (no real-time data)

use async_trait::async_trait;

use crate::core::{
    ExchangeId, ExchangeType, AccountType, Symbol,
    ExchangeResult, ExchangeError, ExchangeIdentity, MarketData,
    Price, OrderBook, Kline, Ticker, Credentials, HttpClient, SymbolInfo,
};

use super::{
    auth::DefiLlamaAuth,
    endpoints::{DefiLlamaEndpoint, format_protocol_slug, format_coin_id, format_coin_ids},
    parser::{
        DefiLlamaParser, ProtocolData, TvlDataPoint, PriceResponse,
        ChainData, StablecoinData, YieldPoolData,
    },
};

/// DefiLlama connector
pub struct DefiLlamaConnector {
    auth: DefiLlamaAuth,
    http_client: HttpClient,
    _testnet: bool,
}

impl DefiLlamaConnector {
    /// Create new DefiLlama connector
    ///
    /// # Arguments
    /// - `credentials`: Optional credentials with API key for Pro tier
    /// - `testnet`: Ignored (DefiLlama has no testnet)
    pub async fn new(
        credentials: Option<Credentials>,
        testnet: bool,
    ) -> ExchangeResult<Self> {
        let auth = DefiLlamaAuth::new(credentials.as_ref())?;
        let http_client = HttpClient::new(10000)?; // 10 second timeout

        Ok(Self {
            auth,
            http_client,
            _testnet: testnet,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CUSTOM METHODS - DeFi Data
    // ═══════════════════════════════════════════════════════════════════════════

    /// Get all protocols
    pub async fn get_protocols(&self) -> ExchangeResult<Vec<ProtocolData>> {
        let url = self.auth.build_url(DefiLlamaEndpoint::Protocols.path());
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch protocols: {}", e)))?;

        DefiLlamaParser::parse_protocols(&response)
    }

    /// Get single protocol data by slug
    ///
    /// # Examples
    /// - `get_protocol("aave")` - Aave protocol
    /// - `get_protocol("uniswap")` - Uniswap protocol
    pub async fn get_protocol(&self, protocol_slug: &str) -> ExchangeResult<ProtocolData> {
        let slug = format_protocol_slug(protocol_slug);
        let path = DefiLlamaEndpoint::Protocol.path().replace("{protocol}", &slug);
        let url = self.auth.build_url(&path);
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch protocol {}: {}", slug, e)))?;

        DefiLlamaParser::parse_protocol(&response)
    }

    /// Get historical TVL for protocol
    pub async fn get_protocol_tvl_history(&self, protocol_slug: &str) -> ExchangeResult<Vec<TvlDataPoint>> {
        let slug = format_protocol_slug(protocol_slug);
        let path = DefiLlamaEndpoint::ProtocolTvl.path().replace("{protocol}", &slug);
        let url = self.auth.build_url(&path);
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch TVL history: {}", e)))?;

        DefiLlamaParser::parse_tvl_history(&response)
    }

    /// Get current token prices
    ///
    /// # Arguments
    /// - `coins`: Vec of (chain, address) tuples
    ///
    /// # Examples
    /// ```ignore
    /// let coins = vec![
    ///     ("ethereum".to_string(), "0x6b175474e89094c44da98b954eedeac495271d0f".to_string()), // DAI
    ///     ("bsc".to_string(), "0x55d398326f99059ff775485246999027b3197955".to_string()), // USDT on BSC
    /// ];
    /// let prices = connector.get_token_prices(coins).await?;
    /// ```
    pub async fn get_token_prices(&self, coins: Vec<(String, String)>) -> ExchangeResult<PriceResponse> {
        let coins_param = format_coin_ids(&coins);
        let endpoint = DefiLlamaEndpoint::PricesCurrent;
        let path = endpoint.path().replace("{coins}", &coins_param);
        let url = self.auth.build_url_for(endpoint.category(), &path);
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch prices: {}", e)))?;

        DefiLlamaParser::parse_prices(&response)
    }

    /// Get single token price
    pub async fn get_token_price(&self, chain: &str, address: &str) -> ExchangeResult<f64> {
        let coins = vec![(chain.to_string(), address.to_string())];
        let prices = self.get_token_prices(coins).await?;

        let coin_id = format_coin_id(chain, address);
        DefiLlamaParser::extract_price(&coin_id, &prices)
            .ok_or_else(|| ExchangeError::Parse(format!("Price not found for {}", coin_id)))
    }

    /// Get all chains with TVL data
    pub async fn get_chains(&self) -> ExchangeResult<Vec<ChainData>> {
        let url = self.auth.build_url(DefiLlamaEndpoint::TvlAll.path());
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch chains: {}", e)))?;

        DefiLlamaParser::parse_chains(&response)
    }

    /// Get all stablecoins
    pub async fn get_stablecoins(&self) -> ExchangeResult<Vec<StablecoinData>> {
        let endpoint = DefiLlamaEndpoint::Stablecoins;
        let url = self.auth.build_url_for(endpoint.category(), endpoint.path());
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch stablecoins: {}", e)))?;

        DefiLlamaParser::parse_stablecoins(&response)
    }

    /// Get all yield pools
    pub async fn get_yield_pools(&self) -> ExchangeResult<Vec<YieldPoolData>> {
        let endpoint = DefiLlamaEndpoint::YieldPools;
        let url = self.auth.build_url_for(endpoint.category(), endpoint.path());
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch yield pools: {}", e)))?;

        DefiLlamaParser::parse_yield_pools(&response)
    }

    /// Check if using Pro tier
    pub fn is_pro_tier(&self) -> bool {
        self.auth.is_pro_tier()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRAIT IMPLEMENTATIONS
// ═══════════════════════════════════════════════════════════════════════════════

impl ExchangeIdentity for DefiLlamaConnector {
    fn exchange_id(&self) -> ExchangeId {
        ExchangeId::DefiLlama
    }

    fn is_testnet(&self) -> bool {
        // DefiLlama has no testnet; return the stored flag as passed by the caller
        self._testnet
    }

    fn supported_account_types(&self) -> Vec<AccountType> {
        // DefiLlama is an aggregator, not an exchange
        // Return empty vec to indicate no trading support
        vec![]
    }

    fn exchange_type(&self) -> ExchangeType {
        // DefiLlama is a data aggregator
        ExchangeType::Cex // Use Cex as placeholder (could add Aggregator type later)
    }
}

#[async_trait]
impl MarketData for DefiLlamaConnector {
    /// Get token price using the coins.llama.fi prices API
    ///
    /// # Symbol Format
    /// Uses `coingecko:{base}` format for well-known tokens.
    /// Example: Symbol::new("bitcoin", "usd") -> `coingecko:bitcoin`
    /// Example: Symbol::new("ethereum", "usd") -> `coingecko:ethereum`
    ///
    /// For ERC-20 tokens, use chain:address format in base:
    /// Example: Symbol::new("ethereum:0x6b17...1d0f", "usd") for DAI
    async fn get_price(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Price> {
        // Determine the coin ID format:
        // If base contains ":", it's already in chain:address format
        // Otherwise, treat as coingecko ID (e.g., "bitcoin", "ethereum")
        let coin_id = if symbol.base.contains(':') {
            symbol.base.to_lowercase()
        } else {
            format!("coingecko:{}", symbol.base.to_lowercase())
        };

        let endpoint = DefiLlamaEndpoint::PricesCurrent;
        let path = endpoint.path().replace("{coins}", &coin_id);
        let url = self.auth.build_url_for(endpoint.category(), &path);
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch price for {}: {}", coin_id, e)))?;

        let prices = DefiLlamaParser::parse_prices(&response)?;
        DefiLlamaParser::extract_price(&coin_id, &prices)
            .ok_or_else(|| ExchangeError::Parse(format!("Price not found for {}", coin_id)))
    }

    /// Get orderbook - NOT SUPPORTED for DeFi aggregator
    async fn get_orderbook(
        &self,
        _symbol: Symbol,
        _depth: Option<u16>,
        _account_type: AccountType,
    ) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "DefiLlama is a DeFi aggregator - orderbook not available".to_string()
        ))
    }

    /// Get klines - NOT SUPPORTED for DeFi aggregator
    async fn get_klines(
        &self,
        _symbol: Symbol,
        _interval: &str,
        _limit: Option<u16>,
        _account_type: AccountType,
        _end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "DefiLlama is a DeFi aggregator - klines not available (use get_protocol_tvl_history instead)".to_string()
        ))
    }

    /// Get ticker using the coins.llama.fi prices API
    ///
    /// Returns a minimal ticker with current price from DefiLlama.
    /// Note: DefiLlama does not provide 24h high/low/volume, so those fields are None.
    ///
    /// # Symbol Format
    /// Same as get_price(): uses `coingecko:{base}` for well-known tokens.
    async fn get_ticker(
        &self,
        symbol: Symbol,
        _account_type: AccountType,
    ) -> ExchangeResult<Ticker> {
        let coin_id = if symbol.base.contains(':') {
            symbol.base.to_lowercase()
        } else {
            format!("coingecko:{}", symbol.base.to_lowercase())
        };

        let endpoint = DefiLlamaEndpoint::PricesCurrent;
        let path = endpoint.path().replace("{coins}", &coin_id);
        let url = self.auth.build_url_for(endpoint.category(), &path);
        let headers = self.auth.get_headers();

        let response = self.http_client.get_with_headers(&url, &std::collections::HashMap::new(), &headers).await
            .map_err(|e| ExchangeError::Network(format!("Failed to fetch ticker for {}: {}", coin_id, e)))?;

        let prices = DefiLlamaParser::parse_prices(&response)?;
        let coin_price = prices.get(&coin_id)
            .ok_or_else(|| ExchangeError::Parse(format!("Price not found for {}", coin_id)))?;

        Ok(Ticker {
            symbol: coin_price.symbol.clone(),
            last_price: coin_price.price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp: coin_price.timestamp as i64,
        })
    }

    /// Ping DefiLlama API
    async fn ping(&self) -> ExchangeResult<()> {
        let url = self.auth.build_url(DefiLlamaEndpoint::Protocols.path());
        let headers = self.auth.get_headers();

        self.http_client.get(&url, &headers).await
            .map_err(|e| ExchangeError::Network(format!("Ping failed: {}", e)))?;

        Ok(())
    }

    /// Get all DeFi protocols tracked by DefiLlama
    ///
    /// Returns each protocol as a SymbolInfo where:
    /// - `symbol` = protocol slug (e.g. "aave", "uniswap")
    /// - `base_asset` = protocol slug
    /// - `quote_asset` = "USD" (TVL is denominated in USD)
    async fn get_exchange_info(&self, _account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let protocols = self.get_protocols().await?;

        let infos = protocols
            .into_iter()
            .filter_map(|protocol| {
                // Use slug (id) as primary identifier, fall back to name
                let slug = protocol.id
                    .or(protocol.name)
                    .filter(|s| !s.is_empty())?;

                let slug_lower = slug.to_lowercase().replace(' ', "-");

                Some(SymbolInfo {
                    symbol: slug_lower.clone(),
                    base_asset: slug_lower,
                    quote_asset: "USD".to_string(),
                    status: "TRADING".to_string(),
                    price_precision: 2,
                    quantity_precision: 0,
                    min_quantity: None,
                    max_quantity: None,
                    tick_size: None,
                    step_size: None,
                    min_notional: None,
                })
            })
            .collect();

        Ok(infos)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connector_creation() {
        let connector = DefiLlamaConnector::new(None, false).await.unwrap();
        assert_eq!(connector.exchange_id(), ExchangeId::DefiLlama);
        assert!(!connector.is_pro_tier());
    }

    #[tokio::test]
    async fn test_pro_tier_connector() {
        let credentials = Credentials::new("test_key", "");
        let connector = DefiLlamaConnector::new(Some(credentials), false).await.unwrap();
        assert!(connector.is_pro_tier());
    }
}
