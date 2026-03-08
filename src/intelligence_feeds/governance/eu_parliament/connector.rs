//! EU Parliament connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{EuParliamentParser, EuMep, EuDocument, EuMeeting, EuCommittee};

/// EU Parliament (European Parliament) Open Data connector
///
/// Provides access to European Parliament open data including MEPs, documents,
/// meetings, and committees.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::eu_parliament::EuParliamentConnector;
///
/// let connector = EuParliamentConnector::new();
///
/// // Get MEPs from a specific country
/// let meps = connector.get_meps_by_country("BE", Some(10)).await?;
///
/// // Get recent plenary documents
/// let documents = connector.get_recent_documents(Some(20)).await?;
///
/// // Get committees
/// let committees = connector.get_committees().await?;
/// ```
pub struct EuParliamentConnector {
    client: Client,
    _auth: EuParliamentAuth,
    endpoints: EuParliamentEndpoints,
    _testnet: bool,
}

impl EuParliamentConnector {
    /// Create new EU Parliament connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: EuParliamentAuth::new(),
            endpoints: EuParliamentEndpoints::default(),
            _testnet: false,
        }
    }

    /// Internal: Make GET request to EU Parliament API
    async fn get(
        &self,
        endpoint: EuParliamentEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add format parameter
        params.insert("format".to_string(), "application/json".to_string());

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
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
        EuParliamentParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make GET request with ID appended to path
    async fn get_by_id(
        &self,
        endpoint: EuParliamentEndpoint,
        id: &str,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = params;
        params.insert("format".to_string(), "application/json".to_string());

        let url = format!("{}{}/{}", self.endpoints.rest_base, endpoint.path(), id);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
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
        EuParliamentParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CORE API METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get list of Members of European Parliament
    ///
    /// # Arguments
    /// - `country` - Optional country code filter
    /// - `limit` - Optional limit
    /// - `offset` - Optional offset for pagination
    pub async fn get_meps(
        &self,
        country: Option<&str>,
        limit: Option<u32>,
        offset: Option<u32>,
    ) -> ExchangeResult<Vec<EuMep>> {
        let mut params = HashMap::new();

        if let Some(c) = country {
            params.insert("country-code".to_string(), c.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }
        if let Some(off) = offset {
            params.insert("offset".to_string(), off.to_string());
        }

        let response = self.get(EuParliamentEndpoint::Meps, params).await?;
        EuParliamentParser::parse_meps(&response)
    }

    /// Get MEP details by ID
    ///
    /// # Arguments
    /// - `id` - MEP ID
    pub async fn get_mep(&self, id: &str) -> ExchangeResult<EuMep> {
        let params = HashMap::new();
        let response = self
            .get_by_id(EuParliamentEndpoint::MepById, id, params)
            .await?;
        EuParliamentParser::parse_mep(&response)
    }

    /// Get list of plenary documents
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `limit` - Optional limit
    pub async fn get_plenary_documents(
        &self,
        year: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<EuDocument>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(EuParliamentEndpoint::PlenaryDocuments, params).await?;
        EuParliamentParser::parse_documents(&response)
    }

    /// Get document details by ID
    ///
    /// # Arguments
    /// - `id` - Document ID
    pub async fn get_document(&self, id: &str) -> ExchangeResult<EuDocument> {
        let params = HashMap::new();
        let response = self
            .get_by_id(EuParliamentEndpoint::DocumentById, id, params)
            .await?;
        EuParliamentParser::parse_document(&response)
    }

    /// Get list of meetings
    ///
    /// # Arguments
    /// - `year` - Optional year filter
    /// - `limit` - Optional limit
    pub async fn get_meetings(
        &self,
        year: Option<u32>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<EuMeeting>> {
        let mut params = HashMap::new();

        if let Some(y) = year {
            params.insert("year".to_string(), y.to_string());
        }
        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(EuParliamentEndpoint::Meetings, params).await?;
        EuParliamentParser::parse_meetings(&response)
    }

    /// Get list of committees
    pub async fn get_committees(&self) -> ExchangeResult<Vec<EuCommittee>> {
        let params = HashMap::new();
        let response = self.get(EuParliamentEndpoint::Committees, params).await?;
        EuParliamentParser::parse_committees(&response)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CONVENIENCE METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get MEPs from a specific country (convenience method)
    ///
    /// # Arguments
    /// - `country_code` - Country code (e.g., "BE", "FR", "DE")
    /// - `limit` - Optional limit
    pub async fn get_meps_by_country(
        &self,
        country_code: &str,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<EuMep>> {
        self.get_meps(Some(country_code), limit, None).await
    }

    /// Get most recent plenary documents (convenience method)
    ///
    /// # Arguments
    /// - `limit` - Optional limit (defaults to 10)
    pub async fn get_recent_documents(&self, limit: Option<u32>) -> ExchangeResult<Vec<EuDocument>> {
        self.get_plenary_documents(None, limit.or(Some(10))).await
    }

    /// Get upcoming/recent meetings (convenience method)
    ///
    /// # Arguments
    /// - `limit` - Optional limit (defaults to 10)
    pub async fn get_recent_meetings(&self, limit: Option<u32>) -> ExchangeResult<Vec<EuMeeting>> {
        self.get_meetings(None, limit.or(Some(10))).await
    }

    /// Get legislation activity for a specific year (convenience method)
    ///
    /// Returns both documents and meetings for the given year
    ///
    /// # Arguments
    /// - `year` - Year to query
    pub async fn get_legislation_activity(
        &self,
        year: u32,
    ) -> ExchangeResult<(Vec<EuDocument>, Vec<EuMeeting>)> {
        let documents = self.get_plenary_documents(Some(year), None).await?;
        let meetings = self.get_meetings(Some(year), None).await?;
        Ok((documents, meetings))
    }

    /// Ping (check connection)
    pub async fn ping(&self) -> ExchangeResult<()> {
        // Simple ping - try to get committees (lightweight endpoint)
        let params = HashMap::new();
        let _ = self.get(EuParliamentEndpoint::Committees, params).await?;
        Ok(())
    }
}

impl Default for EuParliamentConnector {
    fn default() -> Self {
        Self::new()
    }
}
