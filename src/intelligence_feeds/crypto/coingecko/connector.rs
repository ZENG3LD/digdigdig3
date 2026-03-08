//! CoinGecko connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::*;

/// CoinGecko API connector
///
/// Provides access to cryptocurrency market data from CoinGecko.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::coingecko::CoinGeckoConnector;
///
/// // With API key (from env)
/// let connector = CoinGeckoConnector::from_env();
///
/// // Without API key (free tier)
/// let connector = CoinGeckoConnector::free();
///
/// // Get Bitcoin price in USD
/// let price = connector.get_bitcoin_price().await?;
///
/// // Get top coins by market cap
/// let top_coins = connector.get_top_coins(10).await?;
///
/// // Get trending coins
/// let trending = connector.get_trending().await?;
/// ```
pub struct CoinGeckoConnector {
    client: Client,
    auth: CoinGeckoAuth,
    endpoints: CoinGeckoEndpoints,
    _testnet: bool,
}

impl CoinGeckoConnector {
    /// Create new CoinGecko connector with authentication
    pub fn new(auth: CoinGeckoAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: CoinGeckoEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `COINGECKO_API_KEY` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(CoinGeckoAuth::from_env())
    }

    /// Create connector without API key (free tier)
    pub fn free() -> Self {
        Self::new(CoinGeckoAuth::free())
    }

    /// Internal: Make GET request to CoinGecko API
    async fn get(
        &self,
        endpoint: CoinGeckoEndpoint,
        path_id: Option<&str>,
        path_suffix: Option<&str>,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let path = endpoint.build_path(path_id, path_suffix);
        let url = format!("{}{}", self.endpoints.rest_base, path);

        // Build headers
        let mut headers = HashMap::new();
        self.auth.add_auth_headers(&mut headers);

        let mut req = self.client.get(&url);

        // Add headers
        for (key, value) in headers.iter() {
            req = req.header(key, value);
        }

        // Add query parameters
        if !params.is_empty() {
            req = req.query(&params);
        }

        let response = req
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for CoinGecko API errors
        CoinGeckoParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SIMPLE PRICE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get simple price for one or more coins
    ///
    /// # Arguments
    /// - `ids` - Comma-separated coin IDs (e.g., "bitcoin,ethereum")
    /// - `vs_currencies` - Comma-separated currencies (e.g., "usd,eur")
    ///
    /// # Returns
    /// HashMap<coin_id, HashMap<currency, price>>
    ///
    /// # Example
    /// ```ignore
    /// let prices = connector.get_price("bitcoin,ethereum", "usd,eur").await?;
    /// let btc_usd = prices.get("bitcoin").and_then(|p| p.get("usd"));
    /// ```
    pub async fn get_price(
        &self,
        ids: &str,
        vs_currencies: &str,
    ) -> ExchangeResult<HashMap<String, HashMap<String, f64>>> {
        let mut params = HashMap::new();
        params.insert("ids".to_string(), ids.to_string());
        params.insert("vs_currencies".to_string(), vs_currencies.to_string());

        let response = self.get(CoinGeckoEndpoint::SimplePrice, None, None, params).await?;
        CoinGeckoParser::parse_simple_price(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COINS ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of all coins with id, name, and symbol
    pub async fn get_coins_list(&self) -> ExchangeResult<Vec<SimpleCoin>> {
        let params = HashMap::new();
        let response = self.get(CoinGeckoEndpoint::CoinsList, None, None, params).await?;
        CoinGeckoParser::parse_coins_list(&response)
    }

    /// Get detailed coin data by ID
    ///
    /// # Arguments
    /// - `id` - Coin ID (e.g., "bitcoin", "ethereum")
    ///
    /// # Returns
    /// Detailed coin information including description, market data, categories
    pub async fn get_coin(&self, id: &str) -> ExchangeResult<CoinDetail> {
        let params = HashMap::new();
        let response = self.get(CoinGeckoEndpoint::CoinDetail, Some(id), None, params).await?;
        CoinGeckoParser::parse_coin_detail(&response)
    }

    /// Get historical market chart data (price, market cap, volume)
    ///
    /// # Arguments
    /// - `id` - Coin ID (e.g., "bitcoin")
    /// - `vs_currency` - Target currency (e.g., "usd")
    /// - `days` - Number of days (1, 7, 14, 30, 90, 180, 365, max)
    ///
    /// # Returns
    /// Historical data with prices, market caps, and volumes as [timestamp, value] pairs
    pub async fn get_market_chart(
        &self,
        id: &str,
        vs_currency: &str,
        days: &str,
    ) -> ExchangeResult<CoinMarketChart> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency.to_string());
        params.insert("days".to_string(), days.to_string());

        let response = self.get(
            CoinGeckoEndpoint::CoinMarketChart,
            Some(id),
            Some("market_chart"),
            params,
        ).await?;

        CoinGeckoParser::parse_market_chart(&response)
    }

    /// Get paginated list of coins with market data
    ///
    /// # Arguments
    /// - `vs_currency` - Target currency (e.g., "usd")
    /// - `order` - Sort order (e.g., "market_cap_desc", "volume_desc")
    /// - `per_page` - Results per page (1-250, default 100)
    /// - `page` - Page number (default 1)
    ///
    /// # Returns
    /// List of coins with current price, market cap, volume, and 24h changes
    pub async fn get_markets(
        &self,
        vs_currency: &str,
        order: Option<&str>,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<CoinPrice>> {
        let mut params = HashMap::new();
        params.insert("vs_currency".to_string(), vs_currency.to_string());

        if let Some(ord) = order {
            params.insert("order".to_string(), ord.to_string());
        }
        if let Some(pp) = per_page {
            params.insert("per_page".to_string(), pp.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(CoinGeckoEndpoint::CoinsMarkets, None, None, params).await?;
        CoinGeckoParser::parse_coins_markets(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SEARCH ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get trending search coins
    ///
    /// # Returns
    /// List of currently trending coins on CoinGecko
    pub async fn get_trending(&self) -> ExchangeResult<Vec<TrendingCoin>> {
        let params = HashMap::new();
        let response = self.get(CoinGeckoEndpoint::SearchTrending, None, None, params).await?;
        CoinGeckoParser::parse_trending(&response)
    }

    /// Search for coins and exchanges by query
    ///
    /// # Arguments
    /// - `query` - Search query string
    ///
    /// # Returns
    /// List of coins matching the search query
    pub async fn search(&self, query: &str) -> ExchangeResult<Vec<SimpleCoin>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        let response = self.get(CoinGeckoEndpoint::Search, None, None, params).await?;
        CoinGeckoParser::parse_search(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GLOBAL ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get global cryptocurrency market data
    ///
    /// # Returns
    /// Total market cap, volume, market cap percentages, and active cryptocurrencies
    pub async fn get_global(&self) -> ExchangeResult<GlobalData> {
        let params = HashMap::new();
        let response = self.get(CoinGeckoEndpoint::Global, None, None, params).await?;
        CoinGeckoParser::parse_global(&response)
    }

    /// Get global DeFi market data
    ///
    /// # Returns
    /// DeFi-specific global market metrics
    pub async fn get_defi_global(&self) -> ExchangeResult<GlobalData> {
        let params = HashMap::new();
        let response = self.get(CoinGeckoEndpoint::GlobalDefi, None, None, params).await?;
        CoinGeckoParser::parse_global(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXCHANGE ENDPOINTS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of exchanges
    ///
    /// # Arguments
    /// - `per_page` - Results per page (default 100)
    /// - `page` - Page number (default 1)
    ///
    /// # Returns
    /// List of exchanges with volume and trust score
    pub async fn get_exchanges(
        &self,
        per_page: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<CoinGeckoExchange>> {
        let mut params = HashMap::new();

        if let Some(pp) = per_page {
            params.insert("per_page".to_string(), pp.to_string());
        }
        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(CoinGeckoEndpoint::Exchanges, None, None, params).await?;
        CoinGeckoParser::parse_exchanges(&response)
    }

    /// Get exchange details by ID
    ///
    /// # Arguments
    /// - `id` - Exchange ID (e.g., "binance")
    ///
    /// # Returns
    /// Detailed exchange information
    pub async fn get_exchange(&self, id: &str) -> ExchangeResult<CoinGeckoExchange> {
        let params = HashMap::new();
        let response = self.get(CoinGeckoEndpoint::ExchangeDetail, Some(id), None, params).await?;

        // Parse as single exchange (wrapped in array format)
        if let Ok(exchanges) = CoinGeckoParser::parse_exchanges(&serde_json::Value::Array(vec![response])) {
            exchanges.into_iter().next()
                .ok_or_else(|| ExchangeError::Parse("No exchange data returned".to_string()))
        } else {
            Err(ExchangeError::Parse("Failed to parse exchange".to_string()))
        }
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get Bitcoin price in USD (convenience method)
    ///
    /// # Returns
    /// Bitcoin price in USD
    pub async fn get_bitcoin_price(&self) -> ExchangeResult<f64> {
        let prices = self.get_price("bitcoin", "usd").await?;

        prices.get("bitcoin")
            .and_then(|p| p.get("usd"))
            .copied()
            .ok_or_else(|| ExchangeError::NotFound("Bitcoin USD price not found".to_string()))
    }

    /// Get top coins by market cap (convenience method)
    ///
    /// # Arguments
    /// - `limit` - Number of coins to return (max 250)
    ///
    /// # Returns
    /// List of top coins sorted by market cap
    pub async fn get_top_coins(&self, limit: u32) -> ExchangeResult<Vec<CoinPrice>> {
        self.get_markets("usd", Some("market_cap_desc"), Some(limit.min(250)), Some(1)).await
    }
}
