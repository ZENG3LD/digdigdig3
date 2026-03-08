//! AlienVault OTX connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    OtxParser, OtxPulse, OtxIpReputation,
};

/// AlienVault OTX (Open Threat Exchange) connector
///
/// Provides access to threat intelligence data including pulses, IP reputation,
/// domain reputation, and indicators of compromise (IOCs).
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::alienvault_otx::OtxConnector;
///
/// let connector = OtxConnector::from_env();
///
/// // Get subscribed pulses
/// let pulses = connector.get_subscribed_pulses(Some(10), None).await?;
///
/// // Get IP reputation
/// let reputation = connector.get_ip_reputation("8.8.8.8").await?;
///
/// // Get domain reputation
/// let domain_info = connector.get_domain_reputation("example.com").await?;
/// ```
pub struct OtxConnector {
    client: Client,
    auth: OtxAuth,
    endpoints: OtxEndpoints,
}

impl OtxConnector {
    /// Create new OTX connector with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: OtxAuth::new(api_key),
            endpoints: OtxEndpoints::default(),
        }
    }

    /// Create connector from environment variable (OTX_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: OtxAuth::from_env(),
            endpoints: OtxEndpoints::default(),
        }
    }

    /// Internal: Make GET request to OTX API
    async fn get(
        &self,
        endpoint: OtxEndpoint,
        params: HashMap<String, String>,
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

        // Add query parameters
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
                    message: "OTX API rate limit exceeded".to_string(),
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

        // Check for OTX API errors
        OtxParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // OTX-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get subscribed threat intelligence pulses
    ///
    /// # Arguments
    /// - `limit` - Optional maximum number of results to return
    /// - `page` - Optional page number for pagination
    ///
    /// # Returns
    /// List of threat intelligence pulses you are subscribed to
    pub async fn get_subscribed_pulses(
        &self,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<OtxPulse>> {
        let mut params = HashMap::new();

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(OtxEndpoint::SubscribedPulses, params).await?;
        OtxParser::parse_pulses(&response)
    }

    /// Get recent pulse activity
    ///
    /// # Arguments
    /// - `limit` - Optional maximum number of results to return
    /// - `page` - Optional page number for pagination
    ///
    /// # Returns
    /// List of recent threat intelligence pulse activity
    pub async fn get_pulse_activity(
        &self,
        limit: Option<u32>,
        page: Option<u32>,
    ) -> ExchangeResult<Vec<OtxPulse>> {
        let mut params = HashMap::new();

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(OtxEndpoint::PulseActivity, params).await?;
        OtxParser::parse_pulses(&response)
    }

    /// Get IP reputation and threat intelligence
    ///
    /// # Arguments
    /// - `ip` - IPv4 address to lookup (e.g., "8.8.8.8")
    ///
    /// # Returns
    /// IP reputation information including country, ASN, and pulse count
    pub async fn get_ip_reputation(&self, ip: &str) -> ExchangeResult<OtxIpReputation> {
        let params = HashMap::new();
        let response = self.get(OtxEndpoint::IpReputation { ip: ip.to_string() }, params).await?;
        OtxParser::parse_ip_reputation(&response)
    }

    /// Get domain reputation and threat intelligence
    ///
    /// # Arguments
    /// - `domain` - Domain to lookup (e.g., "example.com")
    ///
    /// # Returns
    /// Domain reputation information as raw JSON
    pub async fn get_domain_reputation(&self, domain: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(OtxEndpoint::DomainReputation { domain: domain.to_string() }, params).await
    }

    /// Get hostname reputation and threat intelligence
    ///
    /// # Arguments
    /// - `hostname` - Hostname to lookup
    ///
    /// # Returns
    /// Hostname reputation information as raw JSON
    pub async fn get_hostname_reputation(&self, hostname: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(OtxEndpoint::HostnameReputation { hostname: hostname.to_string() }, params).await
    }

    /// Get file hash reputation and threat intelligence
    ///
    /// # Arguments
    /// - `hash` - File hash (MD5, SHA1, or SHA256)
    ///
    /// # Returns
    /// File hash reputation information as raw JSON
    pub async fn get_file_reputation(&self, hash: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(OtxEndpoint::FileReputation { hash: hash.to_string() }, params).await
    }

    /// Get URL reputation and threat intelligence
    ///
    /// # Arguments
    /// - `url` - URL to lookup
    ///
    /// # Returns
    /// URL reputation information as raw JSON
    pub async fn get_url_reputation(&self, url: &str) -> ExchangeResult<serde_json::Value> {
        let params = HashMap::new();
        self.get(OtxEndpoint::UrlReputation { url: url.to_string() }, params).await
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for OtxConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
