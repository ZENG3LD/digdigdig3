//! GLEIF connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;
type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{GleifParser, GleifEntity, GleifOwnershipChain};

/// GLEIF (Global Legal Entity Identifier Foundation) connector
///
/// Provides access to LEI records and corporate ownership data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::gleif::GleifConnector;
///
/// let connector = GleifConnector::new();
///
/// // Get entity by LEI
/// let entity = connector.get_entity("549300XOCZUOQA850F50").await?;
///
/// // Search by name
/// let results = connector.search_by_name("Apple", None, None).await?;
///
/// // Get ownership chain
/// let chain = connector.get_ownership_chain("549300XOCZUOQA850F50").await?;
/// ```
pub struct GleifConnector {
    client: Client,
    auth: GleifAuth,
    endpoints: GleifEndpoints,
}

impl GleifConnector {
    /// Create new GLEIF connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: GleifAuth::new(),
            endpoints: GleifEndpoints::default(),
        }
    }

    /// Create connector from environment (no-op for GLEIF)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to GLEIF API
    async fn get(
        &self,
        endpoint: GleifEndpoint,
        query_params: Option<HashMap<String, String>>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();
        headers.insert("Accept".to_string(), "application/json".to_string());

        // Add authentication (no-op for GLEIF)
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add query parameters
        if let Some(params) = query_params {
            for (key, value) in params {
                request = request.query(&[(key, value)]);
            }
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check for rate limiting (429)
        if response.status().as_u16() == 429 {
            let retry_after = response
                .headers()
                .get("retry-after")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse::<u64>().ok());

            return Err(ExchangeError::RateLimitExceeded {
                retry_after,
                message: "GLEIF rate limit exceeded (60 requests/minute)".to_string(),
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

        // Check for GLEIF API errors
        GleifParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GLEIF-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get entity by LEI code
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier (e.g., "549300XOCZUOQA850F50")
    ///
    /// # Returns
    /// Entity information or error if not found
    pub async fn get_entity(&self, lei: &str) -> ExchangeResult<GleifEntity> {
        let endpoint = GleifEndpoint::LeiRecord {
            lei: lei.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        let entities = GleifParser::parse_lei_records(&response)?;

        entities
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse("No entity found".to_string()))
    }

    /// Search entities by legal name
    ///
    /// # Arguments
    /// - `name` - Legal name to search for (partial match)
    /// - `page` - Optional page number (1-based)
    /// - `per_page` - Optional results per page (default 10, max 100)
    ///
    /// # Returns
    /// Vector of matching entities
    pub async fn search_by_name(
        &self,
        name: &str,
        page: Option<u32>,
        per_page: Option<u32>,
    ) -> ExchangeResult<Vec<GleifEntity>> {
        let mut params = HashMap::new();
        params.insert("filter[entity.legalName]".to_string(), name.to_string());

        if let Some(p) = page {
            params.insert("page[number]".to_string(), p.to_string());
        }

        if let Some(pp) = per_page {
            params.insert("page[size]".to_string(), pp.to_string());
        }

        let endpoint = GleifEndpoint::SearchByName;
        let response = self.get(endpoint, Some(params)).await?;
        GleifParser::parse_lei_records(&response)
    }

    /// Get direct parent entity
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier
    ///
    /// # Returns
    /// Direct parent entity or None if no parent exists
    pub async fn get_direct_parent(&self, lei: &str) -> ExchangeResult<Option<GleifEntity>> {
        let endpoint = GleifEndpoint::DirectParent {
            lei: lei.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        GleifParser::parse_relationship(&response)
    }

    /// Get ultimate parent entity
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier
    ///
    /// # Returns
    /// Ultimate parent entity or None if no parent exists
    pub async fn get_ultimate_parent(&self, lei: &str) -> ExchangeResult<Option<GleifEntity>> {
        let endpoint = GleifEndpoint::UltimateParent {
            lei: lei.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        GleifParser::parse_relationship(&response)
    }

    /// Get direct children entities (subsidiaries)
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier
    ///
    /// # Returns
    /// Vector of direct children entities
    pub async fn get_children(&self, lei: &str) -> ExchangeResult<Vec<GleifEntity>> {
        let endpoint = GleifEndpoint::DirectChildren {
            lei: lei.to_string(),
        };
        let response = self.get(endpoint, None).await?;
        GleifParser::parse_children(&response)
    }

    /// Get complete ownership chain
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier
    ///
    /// # Returns
    /// Ownership chain with entity, parents, and children
    pub async fn get_ownership_chain(&self, lei: &str) -> ExchangeResult<GleifOwnershipChain> {
        // Fetch entity, parents, and children concurrently
        let entity_result = self.get_entity(lei);
        let direct_parent_result = self.get_direct_parent(lei);
        let ultimate_parent_result = self.get_ultimate_parent(lei);
        let children_result = self.get_children(lei);

        let (entity, direct_parent, ultimate_parent, children) = tokio::join!(
            entity_result,
            direct_parent_result,
            ultimate_parent_result,
            children_result
        );

        Ok(GleifOwnershipChain {
            entity: entity?,
            direct_parent: direct_parent?,
            ultimate_parent: ultimate_parent?,
            children: children?,
        })
    }

    /// Search entities by country
    ///
    /// # Arguments
    /// - `country` - ISO 3166-1 alpha-2 country code (e.g., "US", "GB")
    /// - `page` - Optional page number (1-based)
    ///
    /// # Returns
    /// Vector of entities in the specified country
    pub async fn search_by_country(
        &self,
        country: &str,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<GleifEntity>> {
        let mut params = HashMap::new();
        params.insert(
            "filter[entity.legalAddress.country]".to_string(),
            country.to_string(),
        );

        if let Some(p) = page {
            params.insert("page[number]".to_string(), p.to_string());
        }

        let endpoint = GleifEndpoint::SearchByCountry;
        let response = self.get(endpoint, Some(params)).await?;
        GleifParser::parse_lei_records(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if entity exists
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier
    ///
    /// # Returns
    /// True if entity exists, false otherwise
    pub async fn entity_exists(&self, lei: &str) -> bool {
        self.get_entity(lei).await.is_ok()
    }

    /// Search by name (first page, 10 results)
    ///
    /// # Arguments
    /// - `name` - Legal name to search for
    ///
    /// # Returns
    /// Vector of first 10 matching entities
    pub async fn quick_search(&self, name: &str) -> ExchangeResult<Vec<GleifEntity>> {
        self.search_by_name(name, Some(1), Some(10)).await
    }

    // ═══════════════════════════════════════════════════════════════════════
    // C7 ADDITIONS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get relationship records for a given LEI
    ///
    /// Relationship records describe ownership and control links between legal
    /// entities (parent, ultimate parent, direct child relationships).
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier of the start node
    /// - `relationship_type` - Optional: "IS_DIRECTLY_CONSOLIDATED_BY",
    ///   "IS_ULTIMATELY_CONSOLIDATED_BY", "IS_INTERNATIONAL_BRANCH_OF"
    ///
    /// # Returns
    /// Raw JSON with relationship records
    pub async fn get_relationship_records(
        &self,
        lei: &str,
        relationship_type: Option<&str>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("filter[startNode.id]".to_string(), lei.to_string());
        if let Some(rt) = relationship_type {
            params.insert("filter[relationshipType]".to_string(), rt.to_string());
        }

        let response = self.get(GleifEndpoint::RelationshipRecords, Some(params)).await?;
        Ok(response)
    }

    /// Map a BIC (Bank Identifier Code) to an LEI
    ///
    /// # Arguments
    /// - `bic` - SWIFT BIC code (8 or 11 characters)
    ///
    /// # Returns
    /// Vector of entities matching this BIC code
    pub async fn get_bic_map(&self, bic: &str) -> ExchangeResult<Vec<GleifEntity>> {
        let mut params = HashMap::new();
        params.insert("filter[bic]".to_string(), bic.to_string());

        let response = self.get(GleifEndpoint::BicMaps, Some(params)).await?;
        GleifParser::parse_lei_records(&response)
    }

    /// Get reporting exceptions for a legal entity
    ///
    /// When an entity cannot provide parent information (privacy, no parent,
    /// regulatory exemption), they file a reporting exception.
    ///
    /// # Arguments
    /// - `lei` - Legal Entity Identifier
    ///
    /// # Returns
    /// Raw JSON with reporting exception data
    pub async fn get_reporting_exceptions(
        &self,
        lei: &str,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("filter[LEI]".to_string(), lei.to_string());

        let response = self.get(GleifEndpoint::ReportingExceptions { lei: lei.to_string() }, Some(params)).await?;
        Ok(response)
    }
}

impl Default for GleifConnector {
    fn default() -> Self {
        Self::new()
    }
}
