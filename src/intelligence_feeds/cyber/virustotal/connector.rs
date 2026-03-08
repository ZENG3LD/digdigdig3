//! VirusTotal connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    VirusTotalParser, VtFileReport, VtDomainReport, VtIpReport,
};

/// VirusTotal API v3 connector
///
/// Provides access to VirusTotal's threat intelligence data including file analysis,
/// URL scanning, domain and IP reputation, and search functionality.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::virustotal::VirusTotalConnector;
///
/// let connector = VirusTotalConnector::from_env();
///
/// // Get file report by hash
/// let report = connector.get_file_report("44d88612fea8a8f36de82e1278abb02f").await?;
///
/// // Get domain report
/// let domain = connector.get_domain_report("example.com").await?;
///
/// // Search for files
/// let results = connector.search("type:peexe", Some(10)).await?;
/// ```
pub struct VirusTotalConnector {
    client: Client,
    auth: VirusTotalAuth,
    endpoints: VirusTotalEndpoints,
}

impl VirusTotalConnector {
    /// Create new VirusTotal connector with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: VirusTotalAuth::new(api_key),
            endpoints: VirusTotalEndpoints::default(),
        }
    }

    /// Create connector from environment variable (VIRUSTOTAL_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: VirusTotalAuth::from_env(),
            endpoints: VirusTotalEndpoints::default(),
        }
    }

    /// Internal: Make GET request to VirusTotal API
    async fn get(
        &self,
        endpoint: VirusTotalEndpoint,
        params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut headers = HashMap::new();

        // Add authentication (API key as header)
        self.auth.sign_headers(&mut headers);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(&key, &value);
        }

        // Add query params
        if !params.is_empty() {
            request = request.query(&params);
        }

        let response = request
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        // Check HTTP status
        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            // Check for rate limit (429)
            if status.as_u16() == 429 {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: None,
                    message: "VirusTotal API rate limit exceeded".to_string(),
                });
            }

            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("HTTP {} - {}", status, error_text),
            });
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))?;

        // Check for VirusTotal API errors
        VirusTotalParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // VIRUSTOTAL-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get file report by hash
    ///
    /// # Arguments
    /// - `hash` - File hash (MD5, SHA1, or SHA256)
    ///
    /// # Returns
    /// File analysis report including detection stats, file metadata, and threat intelligence
    ///
    /// # Example
    /// ```ignore
    /// let report = connector.get_file_report("44d88612fea8a8f36de82e1278abb02f").await?;
    /// println!("Malicious detections: {}", report.last_analysis_stats.unwrap().malicious);
    /// ```
    pub async fn get_file_report(&self, hash: &str) -> ExchangeResult<VtFileReport> {
        let params = HashMap::new();
        let response = self.get(VirusTotalEndpoint::FileReport { hash: hash.to_string() }, params).await?;
        VirusTotalParser::parse_file_report(&response)
    }

    /// Get domain report
    ///
    /// # Arguments
    /// - `domain` - Domain name (e.g., "example.com")
    ///
    /// # Returns
    /// Domain reputation and analysis data including registrar, categories, and detection stats
    ///
    /// # Example
    /// ```ignore
    /// let report = connector.get_domain_report("example.com").await?;
    /// println!("Reputation: {}", report.reputation.unwrap_or(0));
    /// ```
    pub async fn get_domain_report(&self, domain: &str) -> ExchangeResult<VtDomainReport> {
        let params = HashMap::new();
        let response = self.get(VirusTotalEndpoint::DomainReport { domain: domain.to_string() }, params).await?;
        VirusTotalParser::parse_domain_report(&response)
    }

    /// Get IP address report
    ///
    /// # Arguments
    /// - `ip` - IP address (e.g., "8.8.8.8")
    ///
    /// # Returns
    /// IP reputation and analysis data including country, ASN, owner, and detection stats
    ///
    /// # Example
    /// ```ignore
    /// let report = connector.get_ip_report("8.8.8.8").await?;
    /// println!("Country: {}", report.country.unwrap_or_default());
    /// ```
    pub async fn get_ip_report(&self, ip: &str) -> ExchangeResult<VtIpReport> {
        let params = HashMap::new();
        let response = self.get(VirusTotalEndpoint::IpReport { ip: ip.to_string() }, params).await?;
        VirusTotalParser::parse_ip_report(&response)
    }

    /// Search for files, URLs, domains, or IPs
    ///
    /// # Arguments
    /// - `query` - Search query using VirusTotal Intelligence syntax
    /// - `limit` - Optional limit on number of results (default: 10, max depends on plan)
    ///
    /// # Returns
    /// List of search results as raw JSON values
    ///
    /// # Query Examples
    /// - "type:peexe" - Find PE executables
    /// - "positives:5+" - Find files with 5+ detections
    /// - "tag:malware" - Find files tagged as malware
    /// - "entity:domain example.com" - Search for specific domain
    /// - "p:443 country:US" - Find IPs in US with port 443 open
    ///
    /// # Example
    /// ```ignore
    /// let results = connector.search("type:peexe positives:10+", Some(20)).await?;
    /// println!("Found {} results", results.len());
    /// ```
    pub async fn search(&self, query: &str, limit: Option<u32>) -> ExchangeResult<Vec<serde_json::Value>> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        if let Some(lim) = limit {
            params.insert("limit".to_string(), lim.to_string());
        }

        let response = self.get(VirusTotalEndpoint::Search, params).await?;
        VirusTotalParser::parse_search_results(&response)
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for VirusTotalConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
