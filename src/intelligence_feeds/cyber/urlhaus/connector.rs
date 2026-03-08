//! URLhaus (abuse.ch) connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    UrlhausParser, UrlhausEntry, UrlhausUrlInfo, UrlhausHostInfo,
};

/// URLhaus (abuse.ch) connector
///
/// Provides access to malicious URL database including malware distribution sites,
/// phishing URLs, and associated threat intelligence.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::cyber::urlhaus::UrlhausConnector;
///
/// let connector = UrlhausConnector::from_env();
///
/// // Get recent malicious URLs
/// let urls = connector.get_recent_urls(Some(100)).await?;
///
/// // Lookup specific URL
/// let url_info = connector.lookup_url("http://malicious-site.com/payload.exe").await?;
///
/// // Lookup host
/// let host_info = connector.lookup_host("malicious-site.com").await?;
/// ```
pub struct UrlhausConnector {
    client: Client,
    auth: UrlhausAuth,
    endpoints: UrlhausEndpoints,
}

impl UrlhausConnector {
    /// Create new URLhaus connector with explicit API key
    pub fn new(auth_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: UrlhausAuth::new(auth_key),
            endpoints: UrlhausEndpoints::default(),
        }
    }

    /// Create connector from environment variable (URLHAUS_AUTH_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: UrlhausAuth::from_env(),
            endpoints: UrlhausEndpoints::default(),
        }
    }

    /// Internal: Make GET request to URLhaus API
    async fn get(
        &self,
        endpoint: UrlhausEndpoint,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Create headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.get(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
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
                    message: "URLhaus API rate limit exceeded".to_string(),
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

        // Check for URLhaus API errors
        UrlhausParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to URLhaus API
    async fn post(
        &self,
        endpoint: UrlhausEndpoint,
        body: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        // Create headers with authentication
        let mut headers = HashMap::new();
        self.auth.sign_headers(&mut headers);

        let mut request = self.client.post(&url);

        // Add headers
        for (key, value) in headers {
            request = request.header(key, value);
        }

        // Add form body
        request = request.form(&body);

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
                    message: "URLhaus API rate limit exceeded".to_string(),
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

        // Check for URLhaus API errors
        UrlhausParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // URLHAUS-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get recent malicious URLs
    ///
    /// # Arguments
    /// - `limit` - Optional maximum number of results (default: 100, max: 1000)
    ///
    /// # Returns
    /// List of recent malicious URL entries
    ///
    /// # Example
    /// ```ignore
    /// let urls = connector.get_recent_urls(Some(500)).await?;
    /// for url in urls {
    ///     println!("URL: {} - Threat: {:?}", url.url, url.threat);
    /// }
    /// ```
    pub async fn get_recent_urls(
        &self,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<UrlhausEntry>> {
        let limit = limit.unwrap_or(100).min(1000);
        let response = self.get(UrlhausEndpoint::RecentUrls { limit }).await?;
        UrlhausParser::parse_recent_urls(&response)
    }

    /// Lookup detailed information about a specific URL
    ///
    /// # Arguments
    /// - `url` - The URL to lookup (must be exact match)
    ///
    /// # Returns
    /// Detailed information about the URL including payloads, tags, and threat type
    ///
    /// # Example
    /// ```ignore
    /// let info = connector.lookup_url("http://malicious-site.com/payload.exe").await?;
    /// println!("Threat: {:?}, Status: {:?}", info.threat, info.url_status);
    /// ```
    pub async fn lookup_url(&self, url: &str) -> ExchangeResult<UrlhausUrlInfo> {
        let mut body = HashMap::new();
        body.insert("url".to_string(), url.to_string());

        let response = self.post(UrlhausEndpoint::UrlLookup, body).await?;
        UrlhausParser::parse_url_info(&response)
    }

    /// Lookup information about a specific host
    ///
    /// # Arguments
    /// - `host` - The hostname or IP address to lookup
    ///
    /// # Returns
    /// Host information including URL count and associated malicious URLs
    ///
    /// # Example
    /// ```ignore
    /// let info = connector.lookup_host("malicious-site.com").await?;
    /// println!("Host: {}, URL count: {}", info.host, info.url_count);
    /// ```
    pub async fn lookup_host(&self, host: &str) -> ExchangeResult<UrlhausHostInfo> {
        let mut body = HashMap::new();
        body.insert("host".to_string(), host.to_string());

        let response = self.post(UrlhausEndpoint::HostLookup, body).await?;
        UrlhausParser::parse_host_info(&response)
    }

    /// Lookup information about a malware payload by SHA256 hash
    ///
    /// # Arguments
    /// - `sha256` - The SHA256 hash of the payload
    ///
    /// # Returns
    /// Payload information as raw JSON
    ///
    /// # Example
    /// ```ignore
    /// let payload = connector.lookup_payload("abc123...").await?;
    /// ```
    pub async fn lookup_payload(&self, sha256: &str) -> ExchangeResult<serde_json::Value> {
        let mut body = HashMap::new();
        body.insert("sha256_hash".to_string(), sha256.to_string());

        self.post(UrlhausEndpoint::PayloadLookup, body).await
    }

    /// Lookup URLs associated with a specific tag
    ///
    /// # Arguments
    /// - `tag` - The tag to search for (e.g., "Dridex", "Emotet", "phishing")
    ///
    /// # Returns
    /// List of URLs associated with the tag
    ///
    /// # Example
    /// ```ignore
    /// let urls = connector.lookup_tag("Emotet").await?;
    /// ```
    pub async fn lookup_tag(&self, tag: &str) -> ExchangeResult<Vec<UrlhausEntry>> {
        let mut body = HashMap::new();
        body.insert("tag".to_string(), tag.to_string());

        let response = self.post(UrlhausEndpoint::TagLookup, body).await?;
        UrlhausParser::parse_recent_urls(&response) // Same format as recent URLs
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for UrlhausConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
