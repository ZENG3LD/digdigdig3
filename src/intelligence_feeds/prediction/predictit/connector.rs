//! PredictIt connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{PredictItParser, PredictItMarket};

/// PredictIt prediction markets connector
///
/// Provides access to political and economic prediction markets.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::predictit::PredictItConnector;
///
/// let connector = PredictItConnector::new();
///
/// // Get all markets
/// let markets = connector.get_all_markets().await?;
///
/// // Get specific market
/// let market = connector.get_market(7940).await?;
///
/// // Get election markets
/// let election_markets = connector.get_election_markets().await?;
/// ```
pub struct PredictItConnector {
    client: Client,
    _auth: PredictItAuth,
    endpoints: PredictItEndpoints,
    _testnet: bool,
}

impl PredictItConnector {
    /// Create new PredictIt connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: PredictItAuth::new(),
            endpoints: PredictItEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment (same as new() - no auth needed)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to PredictIt API
    async fn get(
        &self,
        endpoint: PredictItEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
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

        // Check for API errors
        PredictItParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request with market ID path parameter
    async fn get_with_path(
        &self,
        endpoint: PredictItEndpoint,
        path_param: u64,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}/{}", self.endpoints.rest_base, endpoint.path(), path_param);

        let response = self
            .client
            .get(&url)
            .query(&params)
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

        // Check for API errors
        PredictItParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PREDICTIT-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get all markets with contracts
    ///
    /// This is the CORE endpoint for retrieving all prediction markets.
    ///
    /// # Returns
    /// Vector of all active and closed markets
    pub async fn get_all_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let params = HashMap::new();
        let response = self.get(PredictItEndpoint::AllMarkets, params).await?;
        let parsed = PredictItParser::parse_all_markets(&response)?;
        Ok(parsed.markets)
    }

    /// Get specific market by ID
    ///
    /// # Arguments
    /// - `market_id` - PredictIt market ID
    ///
    /// # Returns
    /// Single market with all contracts
    pub async fn get_market(&self, market_id: u64) -> ExchangeResult<PredictItMarket> {
        let params = HashMap::new();
        let response = self.get_with_path(PredictItEndpoint::Market, market_id, params).await?;
        PredictItParser::parse_market(&response)
    }

    /// Get only active (Open) markets
    ///
    /// Convenience method to filter markets by status.
    ///
    /// # Returns
    /// Vector of markets with status "Open"
    pub async fn get_active_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| m.status == "Open")
            .collect())
    }

    /// Get election-related markets
    ///
    /// Convenience method to filter markets about elections or presidents.
    ///
    /// # Returns
    /// Vector of markets with "election" or "president" in name (case-insensitive)
    pub async fn get_election_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| {
                let name_lower = m.name.to_lowercase();
                name_lower.contains("election") || name_lower.contains("president")
            })
            .collect())
    }

    /// Get Congress-related markets
    ///
    /// Convenience method to filter markets about Congress, Senate, or House.
    ///
    /// # Returns
    /// Vector of markets with "congress", "senate", or "house" in name (case-insensitive)
    pub async fn get_congress_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| {
                let name_lower = m.name.to_lowercase();
                name_lower.contains("congress")
                    || name_lower.contains("senate")
                    || name_lower.contains("house")
            })
            .collect())
    }

    /// Get policy/legislation-related markets
    ///
    /// Convenience method to filter markets about policy or legislation.
    ///
    /// # Returns
    /// Vector of markets with "policy" or "legislation" in name (case-insensitive)
    pub async fn get_policy_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| {
                let name_lower = m.name.to_lowercase();
                name_lower.contains("policy") || name_lower.contains("legislation")
            })
            .collect())
    }

    /// Get economic-related markets
    ///
    /// Convenience method to filter markets about economy, GDP, or inflation.
    ///
    /// # Returns
    /// Vector of markets with "economy", "gdp", or "inflation" in name (case-insensitive)
    pub async fn get_economic_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| {
                let name_lower = m.name.to_lowercase();
                name_lower.contains("economy")
                    || name_lower.contains("gdp")
                    || name_lower.contains("inflation")
            })
            .collect())
    }

    /// Get high-volume markets
    ///
    /// Convenience method to filter markets with many contracts (>5).
    ///
    /// # Returns
    /// Vector of markets with more than 5 contracts
    pub async fn get_high_volume_markets(&self) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        Ok(markets
            .into_iter()
            .filter(|m| m.contracts.len() > 5)
            .collect())
    }

    /// Search markets by name substring
    ///
    /// Convenience method to search markets by keyword.
    ///
    /// # Arguments
    /// - `query` - Search query (case-insensitive)
    ///
    /// # Returns
    /// Vector of markets with query in name
    pub async fn search_markets(&self, query: &str) -> ExchangeResult<Vec<PredictItMarket>> {
        let markets = self.get_all_markets().await?;
        let query_lower = query.to_lowercase();
        Ok(markets
            .into_iter()
            .filter(|m| m.name.to_lowercase().contains(&query_lower))
            .collect())
    }
}

impl Default for PredictItConnector {
    fn default() -> Self {
        Self::new()
    }
}
