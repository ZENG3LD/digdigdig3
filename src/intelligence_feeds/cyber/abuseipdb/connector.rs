//! AbuseIPDB connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    AbuseIpdbParser, AbuseIpReport, BlacklistEntry, CheckBlockReport, AbuseCategory,
};

/// AbuseIPDB connector
///
/// Provides access to IP reputation data, blacklists, and abuse reporting.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::abuseipdb::AbuseIpdbConnector;
///
/// let connector = AbuseIpdbConnector::from_env();
///
/// // Check IP reputation
/// let report = connector.check_ip("8.8.8.8", Some(90), None).await?;
///
/// // Get blacklist
/// let blacklist = connector.get_blacklist(Some(90), None).await?;
///
/// // Check network block
/// let block_report = connector.check_block("192.0.2.0/24", Some(32)).await?;
/// ```
pub struct AbuseIpdbConnector {
    client: Client,
    auth: AbuseIpdbAuth,
    endpoints: AbuseIpdbEndpoints,
}

impl AbuseIpdbConnector {
    /// Create new AbuseIPDB connector with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: AbuseIpdbAuth::new(api_key),
            endpoints: AbuseIpdbEndpoints::default(),
        }
    }

    /// Create connector from environment variable (ABUSEIPDB_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: AbuseIpdbAuth::from_env(),
            endpoints: AbuseIpdbEndpoints::default(),
        }
    }

    /// Internal: Make GET request to AbuseIPDB API
    async fn get(
        &self,
        endpoint: AbuseIpdbEndpoint,
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
                    message: "AbuseIPDB API rate limit exceeded".to_string(),
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

        // Check for AbuseIPDB API errors
        AbuseIpdbParser::check_error(&json)?;

        Ok(json)
    }

    /// Internal: Make POST request to AbuseIPDB API
    async fn post(
        &self,
        endpoint: AbuseIpdbEndpoint,
        params: HashMap<String, String>,
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

        // Add form parameters
        if !params.is_empty() {
            request = request.form(&params);
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
                    message: "AbuseIPDB API rate limit exceeded".to_string(),
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

        // Check for AbuseIPDB API errors
        AbuseIpdbParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ABUSEIPDB-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check IP address for abuse reports
    ///
    /// # Arguments
    /// - `ip` - IP address to check (e.g., "8.8.8.8")
    /// - `max_age_in_days` - Optional maximum age of reports to include (default: 30)
    /// - `verbose` - Optional flag to include detailed report data
    ///
    /// # Returns
    /// IP abuse report with confidence score, reports count, and metadata
    pub async fn check_ip(
        &self,
        ip: &str,
        max_age_in_days: Option<u32>,
        verbose: Option<bool>,
    ) -> ExchangeResult<AbuseIpReport> {
        let mut params = HashMap::new();
        params.insert("ipAddress".to_string(), ip.to_string());

        if let Some(max_age) = max_age_in_days {
            params.insert("maxAgeInDays".to_string(), max_age.to_string());
        }

        if let Some(v) = verbose {
            params.insert("verbose".to_string(), v.to_string());
        }

        let response = self.get(AbuseIpdbEndpoint::Check, params).await?;
        AbuseIpdbParser::parse_check(&response)
    }

    /// Get blacklist of malicious IP addresses
    ///
    /// # Arguments
    /// - `confidence_minimum` - Optional minimum abuse confidence score (0-100, default: 100)
    /// - `limit` - Optional maximum number of results (default: 10000, max: 10000)
    ///
    /// # Returns
    /// List of blacklisted IP addresses with abuse confidence scores
    pub async fn get_blacklist(
        &self,
        confidence_minimum: Option<u8>,
        limit: Option<u32>,
    ) -> ExchangeResult<Vec<BlacklistEntry>> {
        let mut params = HashMap::new();

        if let Some(confidence) = confidence_minimum {
            params.insert("confidenceMinimum".to_string(), confidence.to_string());
        }

        if let Some(l) = limit {
            params.insert("limit".to_string(), l.to_string());
        }

        let response = self.get(AbuseIpdbEndpoint::Blacklist, params).await?;
        AbuseIpdbParser::parse_blacklist(&response)
    }

    /// Check a network block (CIDR notation)
    ///
    /// # Arguments
    /// - `network` - Network in CIDR notation (e.g., "192.0.2.0/24")
    /// - `max_age_in_days` - Optional maximum age of reports to include (default: 30)
    ///
    /// # Returns
    /// Block report with network info and reported addresses
    pub async fn check_block(
        &self,
        network: &str,
        max_age_in_days: Option<u32>,
    ) -> ExchangeResult<CheckBlockReport> {
        let mut params = HashMap::new();
        params.insert("network".to_string(), network.to_string());

        if let Some(max_age) = max_age_in_days {
            params.insert("maxAgeInDays".to_string(), max_age.to_string());
        }

        let response = self.get(AbuseIpdbEndpoint::CheckBlock, params).await?;
        AbuseIpdbParser::parse_check_block(&response)
    }

    /// Report an IP address for abuse
    ///
    /// # Arguments
    /// - `ip` - IP address to report
    /// - `categories` - List of abuse category IDs (see AbuseCategory enum)
    /// - `comment` - Optional comment describing the abuse
    ///
    /// # Returns
    /// Raw API response (success confirmation)
    pub async fn report_ip(
        &self,
        ip: &str,
        categories: &[u8],
        comment: Option<&str>,
    ) -> ExchangeResult<serde_json::Value> {
        let mut params = HashMap::new();
        params.insert("ip".to_string(), ip.to_string());

        // Join category IDs with commas
        let categories_str = categories
            .iter()
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join(",");
        params.insert("categories".to_string(), categories_str);

        if let Some(c) = comment {
            params.insert("comment".to_string(), c.to_string());
        }

        self.post(AbuseIpdbEndpoint::Report, params).await
    }

    /// Get list of abuse categories
    ///
    /// # Returns
    /// Vector of (category_id, category_name) tuples
    pub fn get_categories(&self) -> Vec<(u8, &'static str)> {
        AbuseCategory::all()
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for AbuseIpdbConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
