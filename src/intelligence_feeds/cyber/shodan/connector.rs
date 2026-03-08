//! Shodan connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    ShodanParser, ShodanHost, ShodanSearchResult,
    ShodanApiInfo, ShodanDnsResult,
};

/// Shodan Internet Scanner connector
///
/// Provides access to Shodan's internet scanning data including host information,
/// search functionality, DNS resolution, and vulnerability data.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::shodan::ShodanConnector;
///
/// let connector = ShodanConnector::from_env();
///
/// // Get host information
/// let host = connector.get_host("8.8.8.8").await?;
///
/// // Search for hosts
/// let results = connector.search("apache", Some(1)).await?;
///
/// // Get API info
/// let info = connector.get_api_info().await?;
/// ```
pub struct ShodanConnector {
    client: Client,
    auth: ShodanAuth,
    endpoints: ShodanEndpoints,
}

impl ShodanConnector {
    /// Create new Shodan connector with explicit API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            auth: ShodanAuth::new(api_key),
            endpoints: ShodanEndpoints::default(),
        }
    }

    /// Create connector from environment variable (SHODAN_API_KEY)
    pub fn from_env() -> Self {
        Self {
            client: Client::new(),
            auth: ShodanAuth::from_env(),
            endpoints: ShodanEndpoints::default(),
        }
    }

    /// Internal: Make GET request to Shodan API
    async fn get(
        &self,
        endpoint: ShodanEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (API key as query param)
        self.auth.sign_query(&mut params);

        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .query(&params)
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
                    message: "Shodan API rate limit exceeded".to_string(),
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

        // Check for Shodan API errors
        ShodanParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // SHODAN-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get host information for an IP address
    ///
    /// # Arguments
    /// - `ip` - IP address to lookup (e.g., "8.8.8.8")
    ///
    /// # Returns
    /// Host information including ports, services, vulnerabilities, and organization
    pub async fn get_host(&self, ip: &str) -> ExchangeResult<ShodanHost> {
        let params = HashMap::new();
        let response = self.get(ShodanEndpoint::HostInfo { ip: ip.to_string() }, params).await?;
        ShodanParser::parse_host(&response)
    }

    /// Search Shodan for hosts matching a query
    ///
    /// # Arguments
    /// - `query` - Search query (e.g., "apache", "port:22", "country:US")
    /// - `page` - Optional page number (1-based, default: 1)
    ///
    /// # Returns
    /// Search results with total count and matching hosts
    ///
    /// # Query Examples
    /// - "apache" - Find Apache servers
    /// - "port:22" - Find hosts with port 22 open
    /// - "country:US" - Find hosts in the US
    /// - "product:nginx" - Find nginx servers
    /// - "vuln:CVE-2021-44228" - Find hosts vulnerable to Log4j
    pub async fn search(&self, query: &str, page: Option<u32>) -> ExchangeResult<ShodanSearchResult> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        if let Some(p) = page {
            params.insert("page".to_string(), p.to_string());
        }

        let response = self.get(ShodanEndpoint::HostSearch, params).await?;
        ShodanParser::parse_search_result(&response)
    }

    /// Count results for a search query without returning the results
    ///
    /// # Arguments
    /// - `query` - Search query (same format as search())
    ///
    /// # Returns
    /// Total number of results for the query
    pub async fn count(&self, query: &str) -> ExchangeResult<u64> {
        let mut params = HashMap::new();
        params.insert("query".to_string(), query.to_string());

        let response = self.get(ShodanEndpoint::HostCount, params).await?;
        ShodanParser::parse_count(&response)
    }

    /// Resolve hostnames to IP addresses
    ///
    /// # Arguments
    /// - `hostnames` - List of hostnames to resolve (e.g., ["google.com", "github.com"])
    ///
    /// # Returns
    /// List of DNS resolution results mapping hostnames to IPs
    pub async fn resolve_dns(&self, hostnames: &[&str]) -> ExchangeResult<Vec<ShodanDnsResult>> {
        let mut params = HashMap::new();
        params.insert("hostnames".to_string(), hostnames.join(","));

        let response = self.get(ShodanEndpoint::DnsResolve, params).await?;
        ShodanParser::parse_dns_results(&response)
    }

    /// Reverse DNS lookup for IP addresses
    ///
    /// # Arguments
    /// - `ips` - List of IP addresses (e.g., ["8.8.8.8", "1.1.1.1"])
    ///
    /// # Returns
    /// List of reverse DNS results mapping IPs to hostnames
    pub async fn reverse_dns(&self, ips: &[&str]) -> ExchangeResult<Vec<ShodanDnsResult>> {
        let mut params = HashMap::new();
        params.insert("ips".to_string(), ips.join(","));

        let response = self.get(ShodanEndpoint::DnsReverse, params).await?;
        ShodanParser::parse_dns_results(&response)
    }

    /// Get your current public IP address
    ///
    /// # Returns
    /// Your public IP address as seen by Shodan
    pub async fn get_my_ip(&self) -> ExchangeResult<String> {
        let params = HashMap::new();
        let response = self.get(ShodanEndpoint::MyIp, params).await?;
        ShodanParser::parse_string(&response)
    }

    /// Get API plan information
    ///
    /// # Returns
    /// API plan details including scan credits, query credits, and plan type
    pub async fn get_api_info(&self) -> ExchangeResult<ShodanApiInfo> {
        let params = HashMap::new();
        let response = self.get(ShodanEndpoint::ApiInfo, params).await?;
        ShodanParser::parse_api_info(&response)
    }

    /// Get list of ports that Shodan crawls
    ///
    /// # Returns
    /// List of port numbers
    pub async fn get_ports(&self) -> ExchangeResult<Vec<u16>> {
        let params = HashMap::new();
        let response = self.get(ShodanEndpoint::Ports, params).await?;
        ShodanParser::parse_ports(&response)
    }

    /// Get list of protocols that Shodan supports
    ///
    /// # Returns
    /// List of protocol names
    pub async fn get_protocols(&self) -> ExchangeResult<Vec<String>> {
        let params = HashMap::new();
        let response = self.get(ShodanEndpoint::Protocols, params).await?;
        ShodanParser::parse_protocols(&response)
    }

    /// Check if API key is configured
    pub fn is_authenticated(&self) -> bool {
        self.auth.is_authenticated()
    }
}

impl Default for ShodanConnector {
    fn default() -> Self {
        Self::from_env()
    }
}
