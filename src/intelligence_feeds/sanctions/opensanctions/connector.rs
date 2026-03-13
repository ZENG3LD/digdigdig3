//! OpenSanctions connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    OpenSanctionsParser, SanctionEntity, SanctionSearchResult,
    SanctionDataset, SanctionCollection,
};

/// OpenSanctions connector
///
/// Provides access to sanctions, PEP, and watchlist data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::opensanctions::OpenSanctionsConnector;
///
/// let connector = OpenSanctionsConnector::from_env();
///
/// // Search for entities
/// let results = connector.search("vladimir putin", None, None, Some(10), None).await?;
///
/// // Get entity details
/// let entity = connector.get_entity("Q123").await?;
///
/// // Check if sanctioned
/// let is_sanctioned = connector.check_entity("John Doe", Some("Person")).await?;
/// ```
pub struct OpenSanctionsConnector {
    client: Client,
    auth: OpenSanctionsAuth,
    endpoints: OpenSanctionsEndpoints,
    _testnet: bool,
}

impl OpenSanctionsConnector {
    /// Create new OpenSanctions connector with authentication
    pub fn new(auth: OpenSanctionsAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: OpenSanctionsEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `OPENSANCTIONS_API_KEY` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(OpenSanctionsAuth::from_env())
    }

    /// Create connector without authentication (free tier)
    pub fn anonymous() -> Self {
        Self::new(OpenSanctionsAuth::anonymous())
    }

    /// Internal: Make GET request to OpenSanctions API
    async fn get(
        &self,
        endpoint: OpenSanctionsEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
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
        OpenSanctionsParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request with path suffix (for entity and dataset endpoints)
    async fn get_with_path(
        &self,
        endpoint: OpenSanctionsEndpoint,
        path_suffix: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}/{}", self.endpoints.rest_base, endpoint.path(), path_suffix);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers.iter() {
            request = request.header(key, value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
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
        OpenSanctionsParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OPENSANCTIONS API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for entities
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `schema` - Optional schema type filter (Person, Company, Organization)
    /// - `countries` - Optional comma-separated country codes
    /// - `limit` - Optional result limit (default 20)
    /// - `offset` - Optional pagination offset
    ///
    /// # Returns
    /// Search results with total count and entities
    pub async fn search(
        &self,
        query: &str,
        schema: Option<&str>,
        countries: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<SanctionSearchResult> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());

        if let Some(s) = schema {
            params.insert("schema".to_string(), s.to_string());
        }
        if let Some(c) = countries {
            params.insert("countries".to_string(), c.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(OpenSanctionsEndpoint::Search, params).await?;
        OpenSanctionsParser::parse_search_result(&response)
    }

    /// Get entity details by ID
    ///
    /// # Arguments
    /// - `id` - Entity ID
    ///
    /// # Returns
    /// Full entity details
    pub async fn get_entity(&self, id: &str) -> ExchangeResult<SanctionEntity> {
        let params = HashMap::new();
        let response = self.get_with_path(OpenSanctionsEndpoint::Entity, id, params).await?;
        OpenSanctionsParser::parse_entity(&response)
    }

    /// Get all datasets
    ///
    /// # Returns
    /// List of available datasets
    pub async fn get_datasets(&self) -> ExchangeResult<Vec<SanctionDataset>> {
        let params = HashMap::new();
        let response = self.get(OpenSanctionsEndpoint::Datasets, params).await?;
        OpenSanctionsParser::parse_datasets(&response)
    }

    /// Get dataset details by name
    ///
    /// # Arguments
    /// - `name` - Dataset name (e.g., "us_ofac_sdn")
    ///
    /// # Returns
    /// Dataset metadata
    pub async fn get_dataset(&self, name: &str) -> ExchangeResult<SanctionDataset> {
        let params = HashMap::new();
        let response = self.get_with_path(OpenSanctionsEndpoint::Dataset, name, params).await?;
        OpenSanctionsParser::parse_dataset(&response)
    }

    /// Get all collections
    ///
    /// # Returns
    /// List of available collections
    pub async fn get_collections(&self) -> ExchangeResult<Vec<SanctionCollection>> {
        let params = HashMap::new();
        let response = self.get(OpenSanctionsEndpoint::Collections, params).await?;
        OpenSanctionsParser::parse_collections(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Search for persons
    ///
    /// Convenience method that filters by Person schema type.
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `countries` - Optional comma-separated country codes
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results filtered to Person entities
    pub async fn search_persons(
        &self,
        query: &str,
        countries: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<SanctionSearchResult> {
        self.search(query, Some("Person"), countries, limit, None).await
    }

    /// Search for companies
    ///
    /// Convenience method that filters by Company schema type.
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `countries` - Optional comma-separated country codes
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results filtered to Company entities
    pub async fn search_companies(
        &self,
        query: &str,
        countries: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<SanctionSearchResult> {
        self.search(query, Some("Company"), countries, limit, None).await
    }

    /// Search for sanctioned entities
    ///
    /// Convenience method that searches all entities and filters to targets.
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results with target=true entities
    pub async fn search_sanctioned(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<SanctionSearchResult> {
        let mut result = self.search(query, None, None, limit, None).await?;

        // Filter to only target entities
        result.results.retain(|entity| entity.target);
        result.total = result.results.len() as u64;

        Ok(result)
    }

    /// Get sanctions by country
    ///
    /// Convenience method to search entities from a specific country.
    ///
    /// # Arguments
    /// - `country_code` - ISO 3166-1 alpha-2 country code (e.g., "RU", "IR")
    /// - `limit` - Optional result limit
    ///
    /// # Returns
    /// Search results for entities from the specified country
    pub async fn get_sanctions_by_country(
        &self,
        country_code: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<SanctionSearchResult> {
        self.search("*", None, Some(country_code), limit, None).await
    }

    /// Quick sanctions check
    ///
    /// Convenience method to check if an entity name matches any sanctions.
    ///
    /// # Arguments
    /// - `name` - Entity name to check
    /// - `schema` - Optional schema type (Person, Company, Organization)
    ///
    /// # Returns
    /// Search results - if total > 0, entity may be sanctioned
    pub async fn check_entity(
        &self,
        name: &str,
        schema: Option<&str>,
    ) -> ExchangeResult<SanctionSearchResult> {
        self.search(name, schema, None, Some(10), None).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get entities adjacent (related) to a given entity
    ///
    /// Returns entities that share properties with the given entity —
    /// e.g. directors of a sanctioned company, or companies controlled by
    /// a sanctioned individual.
    ///
    /// # Arguments
    /// - `entity_id` - OpenSanctions entity ID (e.g., "NK-12345")
    ///
    /// # Returns
    /// Related entities as raw JSON value
    pub async fn get_entity_adjacency(
        &self,
        entity_id: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let params: HashMap<String, String> = HashMap::new();
        let endpoint = OpenSanctionsEndpoint::EntityAdjacency { entity_id: entity_id.to_string() };
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

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Get individual statements (raw facts) for an entity
    ///
    /// Each entity in OpenSanctions is assembled from multiple source statements.
    /// This endpoint returns the raw statements that comprise an entity's data.
    ///
    /// # Arguments
    /// - `entity_id` - OpenSanctions entity ID
    /// - `limit` - Optional result limit (default: 100)
    ///
    /// # Returns
    /// Statements as raw JSON value
    pub async fn get_statements(
        &self,
        entity_id: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("entity_id".to_string(), entity_id.to_string());
        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let url = format!("{}{}", self.endpoints.rest_base, OpenSanctionsEndpoint::Statements.path());

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

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    /// Use the OpenRefine-compatible Reconciliation API
    ///
    /// The Reconcile API supports entity deduplication and linking against
    /// the OpenSanctions dataset using a standard OpenRefine reconciliation
    /// service protocol.
    ///
    /// # Arguments
    /// - `query` - Query JSON body compatible with OpenRefine reconcile format
    ///
    /// # Returns
    /// Reconciliation results as raw JSON value
    pub async fn reconcile(
        &self,
        queries: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, OpenSanctionsEndpoint::ReconcileApi.path());

        let response = self
            .client
            .post(&url)
            .json(&queries)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }
}
