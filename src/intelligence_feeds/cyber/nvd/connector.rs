//! NVD connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{NvdParser, NvdCve, NvdSearchResult};

/// NVD (National Vulnerability Database) connector
///
/// Provides access to CVE, CPE, and vulnerability data from NIST.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::nvd::NvdConnector;
///
/// let connector = NvdConnector::from_env();
///
/// // Search for CVEs by keyword
/// let results = connector.search_cves(Some("log4j"), None, None, None, Some(10)).await?;
///
/// // Get specific CVE
/// let cve = connector.get_cve("CVE-2021-44228").await?;
///
/// // Get recent critical vulnerabilities
/// let critical = connector.get_recent_critical(Some(20)).await?;
/// ```
pub struct NvdConnector {
    client: Client,
    auth: NvdAuth,
    endpoints: NvdEndpoints,
}

impl NvdConnector {
    /// Create new NVD connector with authentication
    pub fn new(auth: NvdAuth) -> Self {
        Self {
            client: Client::new(),
            auth,
            endpoints: NvdEndpoints::default(),
        }
    }

    /// Create connector from environment variables
    ///
    /// Expects: `NVD_API_KEY` environment variable (optional)
    pub fn from_env() -> Self {
        Self::new(NvdAuth::from_env())
    }

    /// Create connector without API key (public access with lower rate limits)
    pub fn public() -> Self {
        Self::new(NvdAuth::public())
    }

    /// Internal: Make GET request to NVD API
    async fn get(
        &self,
        endpoint: NvdEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Add authentication headers
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url).query(&params);

        // Add headers to request
        for (key, value) in headers {
            request = request.header(key, value);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        let status = response.status();

        // Handle rate limiting
        if status.as_u16() == 403 || status.as_u16() == 429 {
            return Err(ExchangeError::RateLimitExceeded {
                retry_after: None,
                message: "NVD rate limit exceeded. Wait 30 seconds or use API key for higher limits.".to_string(),
            });
        }

        if !status.is_success() {
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {}", status),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for NVD API errors
        NvdParser::check_error(&json)?;

        Ok(json)
    }

    /// Search for CVEs
    ///
    /// # Arguments
    /// - `keyword` - Search keyword (searches in CVE description)
    /// - `severity` - CVSS v3 severity filter (LOW, MEDIUM, HIGH, CRITICAL)
    /// - `start_date` - Published start date (YYYY-MM-DDTHH:MM:SS.000)
    /// - `end_date` - Published end date (YYYY-MM-DDTHH:MM:SS.000)
    /// - `limit` - Results per page (max 2000, default 20)
    ///
    /// # Returns
    /// Search results with CVE entries
    pub async fn search_cves(
        &self,
        keyword: Option<&str>,
        severity: Option<&str>,
        start_date: Option<&str>,
        end_date: Option<&str>,
        limit: Option<u32>,
    ) -> ExchangeResult<NvdSearchResult> {
        let mut params = HashMap::new();

        if let Some(kw) = keyword {
            params.insert("keywordSearch".to_string(), kw.to_string());
        }

        if let Some(sev) = severity {
            params.insert("cvssV3Severity".to_string(), sev.to_string());
        }

        if let Some(start) = start_date {
            params.insert("pubStartDate".to_string(), start.to_string());
        }

        if let Some(end) = end_date {
            params.insert("pubEndDate".to_string(), end.to_string());
        }

        if let Some(lim) = limit {
            params.insert("resultsPerPage".to_string(), lim.to_string());
        }

        let response = self.get(NvdEndpoint::CvesSearch, params).await?;
        NvdParser::parse_cve_search(&response)
    }

    /// Get specific CVE by ID
    ///
    /// # Arguments
    /// - `cve_id` - CVE identifier (e.g., "CVE-2021-44228")
    ///
    /// # Returns
    /// Single CVE entry
    pub async fn get_cve(&self, cve_id: &str) -> ExchangeResult<NvdCve> {
        let mut params = HashMap::new();
        params.insert("cveId".to_string(), cve_id.to_string());

        let response = self.get(NvdEndpoint::CvesSearch, params).await?;
        let search_result = NvdParser::parse_cve_search(&response)?;

        search_result
            .vulnerabilities
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse(format!("CVE not found: {}", cve_id)))
    }

    /// Get recent critical vulnerabilities
    ///
    /// # Arguments
    /// - `limit` - Number of results (max 2000, default 20)
    ///
    /// # Returns
    /// Recent critical CVEs sorted by publication date
    pub async fn get_recent_critical(&self, limit: Option<u32>) -> ExchangeResult<NvdSearchResult> {
        self.search_cves(None, Some("CRITICAL"), None, None, limit)
            .await
    }
}
