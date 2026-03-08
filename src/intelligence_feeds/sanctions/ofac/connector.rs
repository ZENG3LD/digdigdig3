//! OFAC API connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::auth::*;
use super::endpoints::*;
use super::parser::{OfacEntity, OfacParser, OfacScreenResult, OfacSearchResult, OfacSource};

/// OFAC API connector
///
/// Provides access to US Treasury OFAC sanctions data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::ofac::OfacConnector;
///
/// let connector = OfacConnector::from_env();
///
/// // Search for sanctioned entities
/// let results = connector.search("Putin", None, None, None, None).await?;
///
/// // Screen a name
/// let screen = connector.screen("John Smith", None).await?;
///
/// // Get available sources
/// let sources = connector.get_sources().await?;
/// ```
pub struct OfacConnector {
    client: Client,
    auth: OfacAuth,
    endpoints: OfacEndpoints,
}

impl OfacConnector {
    /// Create new OFAC connector with authentication
    pub fn new(auth: OfacAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OfacEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OFAC_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(OfacAuth::from_env())
    }

    /// Internal: Make GET request to OFAC API
    async fn get(
        &self,
        endpoint: OfacEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Build headers with authentication
        let mut headers = reqwest::header::HeaderMap::new();
        self.auth.sign_headers(&mut headers);

        let response = self
            .client
            .get(&url)
            .headers(headers)
            .query(&params)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check for rate limiting
        if response.status().as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|v| v.to_str().ok())
                .and_then(|v| v.parse::<u64>().ok());

            return Err(ExchangeError::RateLimitExceeded {
                retry_after,
                message: "Rate limit exceeded".to_string(),
            });
        }

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

        // Check for OFAC API errors
        OfacParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OFAC-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search sanctioned entities
    ///
    /// # Arguments
    /// - `name` - Name to search for
    /// - `entity_type` - Type filter: "individual" or "entity"
    /// - `source` - Source filter (e.g., "SDN", "EU", "UK")
    /// - `country` - Country filter (ISO 2-letter code)
    /// - `min_score` - Minimum match score (0.0 to 1.0)
    ///
    /// # Returns
    /// Search result with matching entities
    pub async fn search(
        &self,
        name: &str,
        entity_type: Option<&str>,
        source: Option<&str>,
        country: Option<&str>,
        min_score: Option<f64>,
    ) -> ExchangeResult<OfacSearchResult> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), name.to_string());

        if let Some(et) = entity_type {
            params.insert("type".to_string(), et.to_string());
        }
        if let Some(s) = source {
            params.insert("source".to_string(), s.to_string());
        }
        if let Some(c) = country {
            params.insert("country".to_string(), c.to_string());
        }
        if let Some(ms) = min_score {
            params.insert("minScore".to_string(), ms.to_string());
        }

        let response = self.get(OfacEndpoint::Search, params).await?;
        OfacParser::parse_search_result(&response)
    }

    /// Screen a name/entity against SDN list
    ///
    /// # Arguments
    /// - `name` - Name to screen
    /// - `entity_type` - Type: "individual" or "entity"
    ///
    /// # Returns
    /// Screen result indicating if there's a match
    pub async fn screen(
        &self,
        name: &str,
        entity_type: Option<&str>,
    ) -> ExchangeResult<OfacScreenResult> {
        let mut params = HashMap::new();
        params.insert("name".to_string(), name.to_string());

        if let Some(et) = entity_type {
            params.insert("type".to_string(), et.to_string());
        }

        let response = self.get(OfacEndpoint::Screen, params).await?;
        OfacParser::parse_screen_result(&response)
    }

    /// Get available sanction sources
    ///
    /// # Returns
    /// List of sanction sources with metadata
    pub async fn get_sources(&self) -> ExchangeResult<Vec<OfacSource>> {
        let params = HashMap::new();
        let response = self.get(OfacEndpoint::Sources, params).await?;
        OfacParser::parse_sources(&response)
    }

    /// Get SDN list entities (wrapper around search)
    ///
    /// # Arguments
    /// - `limit` - Maximum number of results
    ///
    /// # Returns
    /// List of SDN entities
    pub async fn get_sdn_list(&self, limit: Option<u32>) -> ExchangeResult<Vec<OfacEntity>> {
        let mut params = HashMap::new();
        params.insert("source".to_string(), "SDN".to_string());

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let response = self.get(OfacEndpoint::Sdn, params).await?;
        OfacParser::parse_search_result(&response).map(|r| r.matches)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Screen individual person
    ///
    /// Shortcut for `screen()` with entity_type="individual"
    pub async fn screen_individual(&self, name: &str) -> ExchangeResult<OfacScreenResult> {
        self.screen(name, Some("individual")).await
    }

    /// Screen entity/organization
    ///
    /// Shortcut for `screen()` with entity_type="entity"
    pub async fn screen_entity(&self, name: &str) -> ExchangeResult<OfacScreenResult> {
        self.screen(name, Some("entity")).await
    }

    /// Search by country
    ///
    /// Search all sanctioned entities from a specific country
    pub async fn search_by_country(&self, country: &str) -> ExchangeResult<OfacSearchResult> {
        self.search("", None, None, Some(country), None).await
    }

    /// High confidence matches only
    ///
    /// Search with minimum score threshold of 0.8
    pub async fn search_high_confidence(
        &self,
        name: &str,
    ) -> ExchangeResult<OfacSearchResult> {
        self.search(name, None, None, None, Some(0.8)).await
    }
}
