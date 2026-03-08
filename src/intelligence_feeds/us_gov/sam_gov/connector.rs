//! SAM.gov connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{SamGovParser, SamEntity, SamOpportunity};

/// SAM.gov (System for Award Management) connector
///
/// Provides access to federal contractor registration data and contract opportunities.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::sam_gov::SamGovConnector;
///
/// let connector = SamGovConnector::from_env();
///
/// // Search for entities
/// let entities = connector.search_entities("technology", None, None).await?;
///
/// // Get entity by UEI
/// let entity = connector.get_entity_by_uei("ABCD1234EFGH").await?;
///
/// // Search opportunities
/// let opportunities = connector.search_opportunities(Some("software"), None, None, None).await?;
/// ```
pub struct SamGovConnector {
    client: Client,
    auth: SamGovAuth,
    endpoints: SamGovEndpoints,
}

impl SamGovConnector {
    /// Create new SAM.gov connector with authentication
    pub fn new(auth: SamGovAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: SamGovEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `SAM_GOV_API_KEY` environment variable
    pub fn from_env() -> Self {
        Self::new(SamGovAuth::from_env())
    }

    /// Internal: Make GET request to SAM.gov API
    async fn get(
        &self,
        endpoint: SamGovEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add API key authentication
        self.auth.sign_query(&mut params);

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

        // Check for SAM.gov API errors
        SamGovParser::check_error(&json)?;

        Ok(json)
    }

    /// Search for entities
    ///
    /// # Arguments
    /// - `query` - Search query string
    /// - `page` - Optional page number (0-based)
    /// - `size` - Optional page size (default 10, max 1000)
    ///
    /// # Returns
    /// Vector of entities matching the search criteria
    pub async fn search_entities(
        &self,
        query: &str,
        page: Option<u32>,
        size: Option<u32>,
    ) -> ExchangeResult<Vec<SamEntity>> {
        let mut params = HashMap::new();
        params.insert("q".to_string(), query.to_string());
        params.insert("samRegistered".to_string(), "Yes".to_string());

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }
        if let Some(s) = size {
            params.insert("size".to_string(), s.to_string());
        }

        let response = self.get(SamGovEndpoint::Entities, params).await?;
        SamGovParser::parse_entities(&response)
    }

    /// Get entity by UEI (Unique Entity Identifier)
    ///
    /// # Arguments
    /// - `uei` - The Unique Entity Identifier
    ///
    /// # Returns
    /// Entity information for the specified UEI
    pub async fn get_entity_by_uei(&self, uei: &str) -> ExchangeResult<SamEntity> {
        let mut params = HashMap::new();
        params.insert("ueiSAM".to_string(), uei.to_string());
        params.insert("samRegistered".to_string(), "Yes".to_string());

        let response = self.get(SamGovEndpoint::Entities, params).await?;
        SamGovParser::parse_entity(&response)
    }

    /// Search contract opportunities
    ///
    /// # Arguments
    /// - `query` - Optional search query string
    /// - `posted_from` - Optional start date (YYYY-MM-DD)
    /// - `posted_to` - Optional end date (YYYY-MM-DD)
    /// - `limit` - Optional result limit (default 10, max 1000)
    ///
    /// # Returns
    /// Vector of contract opportunities matching the search criteria
    pub async fn search_opportunities(
        &self,
        query: Option<&str>,
        posted_from: Option<&str>,
        posted_to: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<SamOpportunity>> {
        let mut params = HashMap::new();

        if let Some(q) = query {
            params.insert("q".to_string(), q.to_string());
        }
        if let Some(from) = posted_from {
            params.insert("postedFrom".to_string(), from.to_string());
        }
        if let Some(to) = posted_to {
            params.insert("postedTo".to_string(), to.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(SamGovEndpoint::Opportunities, params).await?;
        SamGovParser::parse_opportunities(&response)
    }

    /// Get entities by NAICS code (industry classification)
    ///
    /// # Arguments
    /// - `naics` - NAICS code (e.g., "541511" for custom computer programming)
    /// - `page` - Optional page number (0-based)
    ///
    /// # Returns
    /// Vector of entities with the specified NAICS code
    pub async fn get_entities_by_naics(
        &self,
        naics: &str,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<SamEntity>> {
        let mut params = HashMap::new();
        params.insert("naicsCode".to_string(), naics.to_string());
        params.insert("samRegistered".to_string(), "Yes".to_string());

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(SamGovEndpoint::Entities, params).await?;
        SamGovParser::parse_entities(&response)
    }
}
