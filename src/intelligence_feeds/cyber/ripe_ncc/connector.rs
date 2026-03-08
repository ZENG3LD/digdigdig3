//! RIPE NCC connector implementation

use reqwest::Client;
use std::collections::HashMap;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::endpoints::*;
use super::auth::*;
use super::parser::{
    RipeNccParser, RipeCountryStats, RipeAsOverview, RipeRoutingStatus,
    RipeBgpState, RipeAnnouncedPrefix, RipeAsnNeighbour, RipeNetworkInfo,
    RipeRirStats, RipeCountryResource, RipeAbuseContact,
};

/// RIPE NCC (RIPEstat) connector
///
/// Provides access to internet infrastructure data including BGP routing,
/// AS information, IP allocations, and network statistics.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::ripe_ncc::RipeNccConnector;
///
/// let connector = RipeNccConnector::new();
///
/// // Get country internet resources
/// let stats = connector.get_country_stats("NL").await?;
///
/// // Get ASN overview
/// let as_info = connector.get_as_overview(3333).await?;
///
/// // Get BGP routing status for a prefix
/// let status = connector.get_routing_status("193.0.0.0/21").await?;
/// ```
pub struct RipeNccConnector {
    client: Client,
    auth: RipeNccAuth,
    endpoints: RipeNccEndpoints,
    _testnet: bool,
}

impl RipeNccConnector {
    /// Create new RIPE NCC connector
    ///
    /// No authentication required - RIPE NCC API is completely public
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            auth: RipeNccAuth::new(),
            endpoints: RipeNccEndpoints::default(),
            _testnet: false,
        }
    }

    /// Create connector from environment (no-op, included for consistency)
    pub fn from_env() -> Self {
        Self::new()
    }

    /// Internal: Make GET request to RIPE NCC API
    async fn get(
        &self,
        endpoint: RipeNccEndpoint,
        mut params: HashMap<String, String>,
    ) -> ExchangeResult<serde_json::Value> {
        // Add authentication (no-op for RIPE NCC)
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

        // Check for RIPE NCC API errors
        RipeNccParser::check_error(&json)?;

        Ok(json)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // RIPE NCC-SPECIFIC METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get country internet resources statistics
    ///
    /// # Arguments
    /// - `country_code` - ISO 3166-1 alpha-2 country code (e.g., "NL", "US")
    ///
    /// # Returns
    /// Statistics including IPv4, IPv6, and ASN counts for the country
    pub async fn get_country_stats(&self, country_code: &str) -> ExchangeResult<RipeCountryStats> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), country_code.to_string());

        let response = self.get(RipeNccEndpoint::CountryResourceStats, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_country_stats(data)
    }

    /// Get AS (Autonomous System) overview
    ///
    /// # Arguments
    /// - `asn` - Autonomous System Number (e.g., 3333)
    ///
    /// # Returns
    /// ASN overview including holder, announcement status, and allocation block
    pub async fn get_as_overview(&self, asn: u64) -> ExchangeResult<RipeAsOverview> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), format!("AS{}", asn));

        let response = self.get(RipeNccEndpoint::AsOverview, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_as_overview(data)
    }

    /// Get BGP routing status for a resource (IP or prefix)
    ///
    /// # Arguments
    /// - `resource` - IP address or prefix (e.g., "193.0.0.0/21", "8.8.8.8")
    ///
    /// # Returns
    /// Routing status including visibility and announcement info
    pub async fn get_routing_status(&self, resource: &str) -> ExchangeResult<RipeRoutingStatus> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), resource.to_string());

        let response = self.get(RipeNccEndpoint::RoutingStatus, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_routing_status(data)
    }

    /// Get BGP state for a resource (IP or prefix)
    ///
    /// # Arguments
    /// - `resource` - IP address or prefix (e.g., "193.0.0.0/21")
    ///
    /// # Returns
    /// BGP state including number of prefixes and AS path
    pub async fn get_bgp_state(&self, resource: &str) -> ExchangeResult<RipeBgpState> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), resource.to_string());

        let response = self.get(RipeNccEndpoint::BgpState, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_bgp_state(data)
    }

    /// Get announced prefixes by an ASN
    ///
    /// # Arguments
    /// - `asn` - Autonomous System Number
    ///
    /// # Returns
    /// List of prefixes announced by the ASN with timeline information
    pub async fn get_announced_prefixes(&self, asn: u64) -> ExchangeResult<Vec<RipeAnnouncedPrefix>> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), format!("AS{}", asn));

        let response = self.get(RipeNccEndpoint::AnnouncedPrefixes, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_announced_prefixes(data)
    }

    /// Get ASN neighbors/peers
    ///
    /// # Arguments
    /// - `asn` - Autonomous System Number
    ///
    /// # Returns
    /// List of neighboring ASNs with type (left/right/uncertain) and power metrics
    pub async fn get_asn_neighbours(&self, asn: u64) -> ExchangeResult<Vec<RipeAsnNeighbour>> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), format!("AS{}", asn));

        let response = self.get(RipeNccEndpoint::AsnNeighbours, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_asn_neighbours(data)
    }

    /// Get network information for an IP address
    ///
    /// # Arguments
    /// - `ip` - IP address (e.g., "8.8.8.8")
    ///
    /// # Returns
    /// Network info including ASNs and prefix
    pub async fn get_network_info(&self, ip: &str) -> ExchangeResult<RipeNetworkInfo> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), ip.to_string());

        let response = self.get(RipeNccEndpoint::NetworkInfo, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_network_info(data)
    }

    /// Get RIR (Regional Internet Registry) allocation statistics by country
    ///
    /// # Arguments
    /// - `country_code` - ISO 3166-1 alpha-2 country code (e.g., "NL", "US")
    ///
    /// # Returns
    /// RIR allocation statistics broken down by registry and resource type
    pub async fn get_rir_stats(&self, country_code: &str) -> ExchangeResult<RipeRirStats> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), country_code.to_string());

        let response = self.get(RipeNccEndpoint::RirStatsCountry, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_rir_stats(data)
    }

    /// Get full list of internet resources for a country
    ///
    /// # Arguments
    /// - `country_code` - ISO 3166-1 alpha-2 country code (e.g., "NL", "US")
    ///
    /// # Returns
    /// List of all resources (ASNs, IPv4, IPv6) allocated to the country
    pub async fn get_country_resources(&self, country_code: &str) -> ExchangeResult<Vec<RipeCountryResource>> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), country_code.to_string());

        let response = self.get(RipeNccEndpoint::CountryResourceList, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_country_resources(data)
    }

    /// Get abuse contact information for a resource (IP or prefix)
    ///
    /// # Arguments
    /// - `resource` - IP address or prefix (e.g., "8.8.8.8", "193.0.0.0/21")
    ///
    /// # Returns
    /// Abuse contact email and update timestamp
    pub async fn get_abuse_contact(&self, resource: &str) -> ExchangeResult<RipeAbuseContact> {
        let mut params = HashMap::new();
        params.insert("resource".to_string(), resource.to_string());

        let response = self.get(RipeNccEndpoint::AbuseContactFinder, params).await?;
        let data = RipeNccParser::extract_data(&response)?;
        RipeNccParser::parse_abuse_contact(data)
    }
}

impl Default for RipeNccConnector {
    fn default() -> Self {
        Self::new()
    }
}
