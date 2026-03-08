//! EU TED connector implementation

use reqwest::Client;
use std::collections::HashMap;
use serde_json::json;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{EuTedParser, TedNotice, TedEntity, TedSearchResult};

/// EU TED (Tenders Electronic Daily) connector
///
/// Provides access to European public procurement data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::eu_ted::EuTedConnector;
///
/// let connector = EuTedConnector::public();
///
/// // Search for notices
/// let results = connector.search_notices("software", None, None, None, None).await?;
///
/// // Get specific notice
/// let notice = connector.get_notice("123456-2024").await?;
///
/// // Search entities
/// let entities = connector.search_entities("University", None, None).await?;
///
/// // Get recent notices
/// let recent = connector.get_recent_notices(Some("DE"), Some(7)).await?;
/// ```
pub struct EuTedConnector {
    client: Client,
    auth: EuTedAuth,
    endpoints: EuTedEndpoints,
}

impl EuTedConnector {
    /// Create new EU TED connector with authentication
    pub fn new(auth: EuTedAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: EuTedEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects optional: `EU_TED_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(EuTedAuth::from_env())
    }

    /// Create connector for public access (no API key)
    pub fn public() -> Self {
        Self::new(EuTedAuth::public())
    }

    /// Internal: Make GET request to EU TED API
    async fn get(
        &self,
        endpoint: EuTedEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request_builder = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }

        // Add query parameters
        if !params.is_empty() {
            request_builder = request_builder.query(&params);
        }

        let response = request_builder
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

        // Check for EU TED API errors
        EuTedParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to EU TED API
    async fn post(
        &self,
        endpoint: EuTedEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request_builder = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request_builder = request_builder.header(key, value);
        }

        let response = request_builder
            .json(&body)
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

        // Check for EU TED API errors
        EuTedParser::check_error(&json)?;

        Ok(json)
    }

    /// Search procurement notices
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `country` - Optional country code (e.g., "DE", "FR", "IT")
    /// - `cpv` - Optional CPV code filter (e.g., "45000000" for construction)
    /// - `page` - Optional page number (1-based, default: 1)
    /// - `limit` - Optional page size (default: 20, max: 100)
    ///
    /// # Returns
    /// Search results with notices matching the criteria
    pub async fn search_notices(
        &self,
        query: &str,
        country: Option<&str>,
        cpv: Option<&str>,
        page: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<TedSearchResult> {
        let mut body_obj = json!({
            "query": query,
            "fields": ["title", "buyer-name", "total-value"],
            "page": page.unwrap_or(1),
            "limit": limit.unwrap_or(20),
            "scope": "ALL"
        });

        // Add optional filters
        if let Some(c) = country {
            body_obj["filters"] = json!({
                "country": c
            });
        }

        if let Some(cpv_code) = cpv {
            if body_obj.get("filters").is_some() {
                body_obj["filters"]["cpv"] = json!(cpv_code);
            } else {
                body_obj["filters"] = json!({
                    "cpv": cpv_code
                });
            }
        }

        let response = self.post(EuTedEndpoint::SearchNotices, body_obj).await?;
        EuTedParser::parse_search_results(&response)
    }

    /// Get specific notice by ID
    ///
    /// # Arguments
    /// - `notice_id` - Notice identifier (e.g., "123456-2024")
    ///
    /// # Returns
    /// Notice details for the specified ID
    pub async fn get_notice(&self, notice_id: &str) -> ExchangeResult<TedNotice> {
        let endpoint = EuTedEndpoint::NoticeDetail {
            notice_id: notice_id.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        EuTedParser::parse_notice(&response)
    }

    /// Search business entities (contracting authorities, economic operators)
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `country` - Optional country code filter
    /// - `page` - Optional page number (1-based)
    ///
    /// # Returns
    /// Vector of entities matching the search criteria
    pub async fn search_entities(
        &self,
        query: &str,
        country: Option<&str>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<TedEntity>> {
        let mut body_obj = json!({
            "query": query,
            "page": page.unwrap_or(1),
            "limit": 20
        });

        if let Some(c) = country {
            body_obj["filters"] = json!({
                "country": c
            });
        }

        let response = self.post(EuTedEndpoint::SearchEntities, body_obj).await?;
        EuTedParser::parse_entities(&response)
    }

    /// Get specific entity by ID
    ///
    /// # Arguments
    /// - `entity_id` - Entity identifier
    ///
    /// # Returns
    /// Entity details for the specified ID
    pub async fn get_entity(&self, entity_id: &str) -> ExchangeResult<TedEntity> {
        let endpoint = EuTedEndpoint::EntityDetail {
            entity_id: entity_id.to_string(),
        };
        let response = self.get(endpoint, HashMap::new()).await?;
        EuTedParser::parse_entity(&response)
    }

    /// Get recent notices
    ///
    /// # Arguments
    /// - `country` - Optional country code filter
    /// - `days` - Number of days to look back (default: 7)
    ///
    /// # Returns
    /// Recent notices from the specified time period
    pub async fn get_recent_notices(
        &self,
        country: Option<&str>,
        days: Option<u32>,
    ) -> ExchangeResult<TedSearchResult> {
        // Calculate date range
        let days_back = days.unwrap_or(7);

        let mut body_obj = json!({
            "query": "*",
            "fields": ["title", "buyer-name", "publication-date"],
            "page": 1,
            "limit": 50,
            "scope": "ALL",
            "sort": {
                "field": "publication-date",
                "order": "desc"
            }
        });

        // Add date filter for recent notices
        let mut filters = json!({});

        if let Some(c) = country {
            filters["country"] = json!(c);
        }

        // Add date filter (last N days)
        filters["publicationDateFrom"] = json!(format!("-{}d", days_back));

        if !filters.is_null() {
            body_obj["filters"] = filters;
        }

        let response = self.post(EuTedEndpoint::SearchNotices, body_obj).await?;
        EuTedParser::parse_search_results(&response)
    }

    /// Get codelist values
    ///
    /// # Arguments
    /// - `codelist_id` - Codelist identifier (e.g., "cpv", "country")
    ///
    /// # Returns
    /// Raw JSON response with codelist values
    pub async fn get_codelist(&self, codelist_id: &str) -> ExchangeResult<serde_json::Value> {
        let endpoint = EuTedEndpoint::Codelist {
            codelist_id: codelist_id.to_string(),
        };
        self.get(endpoint, HashMap::new()).await
    }
}
