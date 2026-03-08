//! ReliefWeb connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{ReliefWebParser, ReliefWebReport, ReliefWebDisaster, ReliefWebCountry, ReliefWebSearchResult};

/// ReliefWeb (UN OCHA) connector
///
/// Provides access to humanitarian reports, disasters, and country data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::reliefweb::ReliefWebConnector;
///
/// let connector = ReliefWebConnector::new();
///
/// // Search reports by query
/// let reports = connector.search_reports(Some("Syria"), None, Some(10)).await?;
///
/// // Get active disasters
/// let disasters = connector.get_active_disasters().await?;
///
/// // Get countries
/// let countries = connector.get_countries().await?;
/// ```
pub struct ReliefWebConnector {
    client: Client,
    auth: ReliefWebAuth,
    endpoints: ReliefWebEndpoints,
    _testnet: bool,
}

impl ReliefWebConnector {
    /// Create new ReliefWeb connector with authentication
    pub fn new(auth: ReliefWebAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: ReliefWebEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector with default anonymous access
    pub fn anonymous() -> Self {
        Self::new(ReliefWebAuth::anonymous())
    }

    /// Create connector from environment variables
    ///
    /// Expects: `RELIEFWEB_APPNAME` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(ReliefWebAuth::from_env())
    }

    /// Internal: Make GET request to ReliefWeb API
    async fn get(
        &self,
        endpoint: ReliefWebEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (appname if configured)
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

        // Check for ReliefWeb API errors
        ReliefWebParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RELIEFWEB-SPECIFIC METHODS (Extended API)
    // ═══════════════════════════════════════════════════════════════════════

    /// Search humanitarian reports
    ///
    /// # Arguments
    /// - `query` - Search query text (optional)
    /// - `country` - Filter by country name (optional)
    /// - `limit` - Number of results to return (optional, default: 10)
    ///
    /// # Returns
    /// Search result containing reports
    pub async fn search_reports(
        &self,
        query: Option<&str>,
        country: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<ReliefWebSearchResult<ReliefWebReport>> {
        let mut params = HashMap::new();

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        // Build filter conditions
        let mut filter_conditions = Vec::new();

        if let Some(q) = query {
            filter_conditions.push(format!("{{\"field\":\"query\",\"value\":\"{}\"}}", q));
        }

        if let Some(c) = country {
            filter_conditions.push(format!("{{\"field\":\"country\",\"value\":\"{}\"}}", c));
        }

        if !filter_conditions.is_empty() {
            let filter_json = format!("{{\"conditions\":[{}]}}", filter_conditions.join(","));
            params.insert("filter".to_string(), filter_json);
        }

        let response = self.get(ReliefWebEndpoint::Reports, params).await?;
        ReliefWebParser::parse_reports(&response)
    }

    /// Get disasters with optional limit
    ///
    /// # Arguments
    /// - `limit` - Number of results to return (optional, default: 10)
    ///
    /// # Returns
    /// Search result containing disasters
    pub async fn get_disasters(&self, limit: Option<u32>) -> ExchangeResult<ReliefWebSearchResult<ReliefWebDisaster>> {
        let mut params = HashMap::new();

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        let response = self.get(ReliefWebEndpoint::Disasters, params).await?;
        ReliefWebParser::parse_disasters(&response)
    }

    /// Get active (ongoing) disasters
    ///
    /// # Returns
    /// Search result containing active disasters only
    pub async fn get_active_disasters(&self) -> ExchangeResult<ReliefWebSearchResult<ReliefWebDisaster>> {
        let mut params = HashMap::new();

        // Filter for active/ongoing disasters
        let filter_json = r#"{"field":"status","value":"ongoing"}"#;
        params.insert("filter".to_string(), filter_json.to_string());

        let response = self.get(ReliefWebEndpoint::Disasters, params).await?;
        ReliefWebParser::parse_disasters(&response)
    }

    /// Get countries
    ///
    /// # Returns
    /// Search result containing country profiles
    pub async fn get_countries(&self) -> ExchangeResult<ReliefWebSearchResult<ReliefWebCountry>> {
        let params = HashMap::new();

        let response = self.get(ReliefWebEndpoint::Countries, params).await?;
        ReliefWebParser::parse_countries(&response)
    }

    /// Get disasters by country
    ///
    /// # Arguments
    /// - `country` - Country name (e.g., "Syria", "Ukraine")
    /// - `limit` - Number of results to return (optional)
    ///
    /// # Returns
    /// Search result containing disasters for the specified country
    pub async fn get_disasters_by_country(
        &self,
        country: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<ReliefWebSearchResult<ReliefWebDisaster>> {
        let mut params = HashMap::new();

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        let filter_json = format!("{{\"field\":\"country\",\"value\":\"{}\"}}", country);
        params.insert("filter".to_string(), filter_json);

        let response = self.get(ReliefWebEndpoint::Disasters, params).await?;
        ReliefWebParser::parse_disasters(&response)
    }

    /// Get reports by source organization
    ///
    /// # Arguments
    /// - `source` - Source organization name (e.g., "OCHA", "UNHCR")
    /// - `limit` - Number of results to return (optional)
    ///
    /// # Returns
    /// Search result containing reports from the specified source
    pub async fn get_reports_by_source(
        &self,
        source: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<ReliefWebSearchResult<ReliefWebReport>> {
        let mut params = HashMap::new();

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        let filter_json = format!("{{\"field\":\"source\",\"value\":\"{}\"}}", source);
        params.insert("filter".to_string(), filter_json);

        let response = self.get(ReliefWebEndpoint::Reports, params).await?;
        ReliefWebParser::parse_reports(&response)
    }

    /// Get reports by theme
    ///
    /// # Arguments
    /// - `theme` - Theme name (e.g., "Coordination", "Food and Nutrition")
    /// - `limit` - Number of results to return (optional)
    ///
    /// # Returns
    /// Search result containing reports matching the theme
    pub async fn get_reports_by_theme(
        &self,
        theme: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<ReliefWebSearchResult<ReliefWebReport>> {
        let mut params = HashMap::new();

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        let filter_json = format!("{{\"field\":\"theme\",\"value\":\"{}\"}}", theme);
        params.insert("filter".to_string(), filter_json);

        let response = self.get(ReliefWebEndpoint::Reports, params).await?;
        ReliefWebParser::parse_reports(&response)
    }

    /// Get reports by format
    ///
    /// # Arguments
    /// - `format` - Report format (e.g., "Situation Report", "Appeal")
    /// - `limit` - Number of results to return (optional)
    ///
    /// # Returns
    /// Search result containing reports of the specified format
    pub async fn get_reports_by_format(
        &self,
        format: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<ReliefWebSearchResult<ReliefWebReport>> {
        let mut params = HashMap::new();

        if let Some(limit_val) = limit {
            params.insert("limit".to_string(), limit_val.to_string());
        }

        let filter_json = format!("{{\"field\":\"format\",\"value\":\"{}\"}}", format);
        params.insert("filter".to_string(), filter_json);

        let response = self.get(ReliefWebEndpoint::Reports, params).await?;
        ReliefWebParser::parse_reports(&response)
    }
}
