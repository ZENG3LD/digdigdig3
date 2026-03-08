//! Censys connector implementation

use reqwest::Client;
use serde_json::json;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    CensysParser, CensysHost, CensysSearchResult,
};

/// Censys Search API v2 connector
///
/// Provides access to Censys internet scanning data including host search,
/// host details, aggregation, and certificate search.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::censys::CensysConnector;
///
/// let connector = CensysConnector::from_env();
///
/// // Get host information
/// let host = connector.get_host("8.8.8.8").await?;
///
/// // Search for hosts
/// let results = connector.search_hosts("service.service_name: HTTP", Some(25)).await?;
///
/// // Aggregate hosts
/// let agg = connector.aggregate_hosts("service.service_name: HTTP", "location.country").await?;
/// ```
pub struct CensysConnector {
    client: Client,
    auth: CensysAuth,
    endpoints: CensysEndpoints,
}

impl CensysConnector {
    /// Create new Censys connector with explicit API credentials
    pub fn new(api_id: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: CensysAuth::new(api_id, api_secret),
            endpoints: CensysEndpoints::default(),
        }
    }

    /// Create connector from environment variables (CENSYS_API_ID, CENSYS_API_SECRET)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: CensysAuth::from_env(),
            endpoints: CensysEndpoints::default(),
        }
    }

    /// Internal: Make GET request to Censys API
    async fn get(
        &self,
        endpoint: CensysEndpoint,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.get(&url);

        // Add Basic Auth using reqwest's built-in method
        if let Some((username, password)) = self.auth.get_basic_auth() {
            request = request.basic_auth(username, Some(password));
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
                    message: "Censys API rate limit exceeded".to_string(),
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

        // Check for Censys API errors
        CensysParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to Censys API
    async fn post(
        &self,
        endpoint: CensysEndpoint,
        body: serde_json::Value,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let mut request = self.client.post(&url);

        // Add Basic Auth using reqwest's built-in method
        if let Some((username, password)) = self.auth.get_basic_auth() {
            request = request.basic_auth(username, Some(password));
        }

        let response = request
            .json(&body)
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
                    message: "Censys API rate limit exceeded".to_string(),
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

        // Check for Censys API errors
        CensysParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CENSYS-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get host information for an IP address
    ///
    /// # Arguments
    /// - `ip` - IP address to lookup (e.g., "8.8.8.8")
    ///
    /// # Returns
    /// Host information including services, location, and autonomous system
    pub async fn get_host(&self, ip: &str) -> ExchangeResult<CensysHost> {
        let response = self.get(CensysEndpoint::HostView { ip: ip.to_string() }).await?;
        CensysParser::parse_host(&response)
    }

    /// Search Censys hosts
    ///
    /// # Arguments
    /// - `query` - Search query using Censys query syntax
    /// - `per_page` - Optional results per page (default: 25, max: 100)
    ///
    /// # Returns
    /// Search results with total count and matching hosts
    ///
    /// # Query Examples
    /// - "service.service_name: HTTP" - Find HTTP servers
    /// - "service.port: 22" - Find hosts with SSH on port 22
    /// - "location.country: US" - Find hosts in the US
    /// - "autonomous_system.asn: 15169" - Find hosts in Google's AS
    pub async fn search_hosts(&self, query: &str, per_page: Option<u32>) -> ExchangeResult<CensysSearchResult> {
        let body = json!({
            "q": query,
            "per_page": per_page.unwrap_or(25),
        });

        let response = self.post(CensysEndpoint::HostsSearch, body).await?;
        CensysParser::parse_search_result(&response)
    }

    /// Aggregate host data by field
    ///
    /// # Arguments
    /// - `query` - Search query using Censys query syntax
    /// - `field` - Field to aggregate by (e.g., "location.country", "service.port")
    ///
    /// # Returns
    /// Aggregation results as raw JSON
    ///
    /// # Examples
    /// ```ignore
    /// // Count hosts by country
    /// let agg = connector.aggregate_hosts("service.service_name: HTTP", "location.country").await?;
    ///
    /// // Count hosts by port
    /// let agg = connector.aggregate_hosts("*", "service.port").await?;
    /// ```
    pub async fn aggregate_hosts(&self, query: &str, field: &str) -> ExchangeResult<serde_json::Value> {
        let body = json!({
            "q": query,
            "field": field,
        });

        self.post(CensysEndpoint::HostsAggregate, body).await
    }

    /// Compare host snapshots (diff)
    ///
    /// # Arguments
    /// - `ip_a` - First IP address
    /// - `ip_b` - Second IP address (optional, defaults to latest snapshot of ip_a)
    /// - `at_time_a` - Timestamp for first snapshot (optional)
    /// - `at_time_b` - Timestamp for second snapshot (optional)
    ///
    /// # Returns
    /// Diff results as raw JSON
    pub async fn diff_hosts(
        &self,
        ip_a: &str,
        ip_b: Option<&str>,
        at_time_a: Option<&str>,
        at_time_b: Option<&str>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut body = json!({
            "ip_a": ip_a,
        });

        if let Some(ip) = ip_b {
            body["ip_b"] = json!(ip);
        }
        if let Some(time) = at_time_a {
            body["at_time_a"] = json!(time);
        }
        if let Some(time) = at_time_b {
            body["at_time_b"] = json!(time);
        }

        self.post(CensysEndpoint::HostsDiff, body).await
    }

    /// Search certificates
    ///
    /// # Arguments
    /// - `query` - Search query using Censys query syntax
    /// - `per_page` - Optional results per page (default: 25, max: 100)
    ///
    /// # Returns
    /// Search results as raw JSON
    ///
    /// # Query Examples
    /// - "parsed.names: google.com" - Find certificates for google.com
    /// - "parsed.subject.common_name: *.example.com" - Find wildcard certificates
    pub async fn search_certificates(&self, query: &str, per_page: Option<u32>) -> ExchangeResult<serde_json::Value> {
        let body = json!({
            "q": query,
            "per_page": per_page.unwrap_or(25),
        });

        self.post(CensysEndpoint::CertificatesSearch, body).await
    }

    /// Check if API credentials are configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for CensysConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
