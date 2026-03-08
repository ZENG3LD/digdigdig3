//! Feodo Tracker connector implementation

use reqwest::Client;
use serde_json::Value;

use crate::core::types::{ExchangeError, ExchangeResult};

use super::auth::*;
use super::endpoints::*;
use super::parser::{BlocklistStats, C2Server, FeodoTrackerParser};

/// Feodo Tracker (abuse.ch) connector
///
/// Provides access to botnet C2 server blocklists and threat intelligence.
///
/// # Usage
/// ```ignore
/// use connectors_v5::data_feeds::feodo_tracker::FeodoTrackerConnector;
///
/// let connector = FeodoTrackerConnector::new();
///
/// // Get full blocklist (past 30 days)
/// let servers = connector.get_blocklist().await?;
///
/// // Get only online servers
/// let online = connector.get_online_servers().await?;
///
/// // Filter by malware family
/// let emotet = connector.get_servers_by_malware("Emotet").await?;
///
/// // Filter by country
/// let us_servers = connector.get_servers_by_country("US").await?;
///
/// // Get statistics
/// let stats = connector.get_blocklist_stats().await?;
/// ```
pub struct FeodoTrackerConnector {
    client: Client,
    _auth: FeodoTrackerAuth,
    endpoints: FeodoTrackerEndpoints,
}

impl FeodoTrackerConnector {
    /// Create new Feodo Tracker connector
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            _auth: FeodoTrackerAuth::new(),
            endpoints: FeodoTrackerEndpoints::default(),
        }
    }

    /// Internal: Make GET request to Feodo Tracker API
    async fn get(&self, endpoint: FeodoTrackerEndpoint) -> ExchangeResult<String> {
        let url = format!("{}{}", self.endpoints.rest_base, endpoint.path());

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ExchangeError::Api {
                code: response.status().as_u16() as i32,
                message: format!("HTTP {}", response.status()),
            });
        }

        let text = response
            .text()
            .await
            .map_err(|e| ExchangeError::Parse(format!("Failed to read response: {}", e)))?;

        Ok(text)
    }

    /// Internal: Parse JSON response
    async fn get_json(&self, endpoint: FeodoTrackerEndpoint) -> ExchangeResult<Value> {
        let text = self.get(endpoint).await?;
        serde_json::from_str(&text)
            .map_err(|e| ExchangeError::Parse(format!("JSON parse error: {}", e)))
    }

    // ==========================================================================
    // PUBLIC API METHODS
    // ==========================================================================

    /// Get IP blocklist (past 30 days) with full metadata
    ///
    /// Returns all C2 servers seen in the past 30 days with complete metadata
    /// including port, status, hostname, ASN, geolocation, and malware family.
    ///
    /// # Returns
    /// Vector of C2 server entries (may be empty if no active threats)
    pub async fn get_blocklist(&self) -> ExchangeResult<Vec<C2Server>> {
        let data = self.get_json(FeodoTrackerEndpoint::IpBlocklist).await?;
        FeodoTrackerParser::parse_blocklist(&data)
    }

    /// Get recommended blocklist (simple IP list)
    ///
    /// Returns recommended blocklist with lowest false positive rate.
    /// Contains only IP addresses without metadata.
    ///
    /// # Returns
    /// Vector of IP addresses
    pub async fn get_recommended_ips(&self) -> ExchangeResult<Vec<String>> {
        let data = self.get_json(FeodoTrackerEndpoint::IpBlocklistRecommended).await?;
        FeodoTrackerParser::parse_recommended_blocklist(&data)
    }

    /// Get only online C2 servers
    ///
    /// Filters blocklist to return only currently active C2 servers.
    /// These are the highest priority for blocking.
    ///
    /// # Returns
    /// Vector of online C2 servers
    pub async fn get_online_servers(&self) -> ExchangeResult<Vec<C2Server>> {
        let servers = self.get_blocklist().await?;
        Ok(servers.into_iter().filter(|s| s.is_online()).collect())
    }

    /// Get only offline C2 servers
    ///
    /// Returns C2 servers that are currently not responding.
    ///
    /// # Returns
    /// Vector of offline C2 servers
    pub async fn get_offline_servers(&self) -> ExchangeResult<Vec<C2Server>> {
        let servers = self.get_blocklist().await?;
        Ok(servers.into_iter().filter(|s| s.is_offline()).collect())
    }

    /// Get C2 servers by malware family
    ///
    /// Filter blocklist by specific malware family.
    ///
    /// # Arguments
    /// - `family` - Malware family name (e.g., "Emotet", "TrickBot", "QakBot", "Dridex", "BazarLoader")
    ///
    /// # Returns
    /// Vector of C2 servers for the specified malware family
    pub async fn get_servers_by_malware(&self, family: &str) -> ExchangeResult<Vec<C2Server>> {
        let servers = self.get_blocklist().await?;
        Ok(servers
            .into_iter()
            .filter(|s| s.is_malware_family(family))
            .collect())
    }

    /// Get C2 servers by country
    ///
    /// Filter blocklist by country code.
    ///
    /// # Arguments
    /// - `country` - ISO 3166-1 alpha-2 country code (e.g., "US", "DE", "RU")
    ///
    /// # Returns
    /// Vector of C2 servers in the specified country
    pub async fn get_servers_by_country(&self, country: &str) -> ExchangeResult<Vec<C2Server>> {
        let servers = self.get_blocklist().await?;
        Ok(servers.into_iter().filter(|s| s.is_country(country)).collect())
    }

    /// Get C2 servers by ASN (Autonomous System Number)
    ///
    /// Filter blocklist by hosting provider / ISP.
    ///
    /// # Arguments
    /// - `asn` - Autonomous System Number (e.g., 14061 for DigitalOcean)
    ///
    /// # Returns
    /// Vector of C2 servers on the specified ASN
    pub async fn get_servers_by_asn(&self, asn: u32) -> ExchangeResult<Vec<C2Server>> {
        let servers = self.get_blocklist().await?;
        Ok(servers.into_iter().filter(|s| s.as_number == asn).collect())
    }

    /// Get C2 servers by port
    ///
    /// Filter blocklist by TCP port number.
    ///
    /// # Arguments
    /// - `port` - TCP port number (e.g., 443, 8080)
    ///
    /// # Returns
    /// Vector of C2 servers using the specified port
    pub async fn get_servers_by_port(&self, port: u16) -> ExchangeResult<Vec<C2Server>> {
        let servers = self.get_blocklist().await?;
        Ok(servers.into_iter().filter(|s| s.port == port).collect())
    }

    /// Get blocklist statistics
    ///
    /// Returns summary statistics including total counts, online/offline breakdown,
    /// and distribution by malware family and country.
    ///
    /// # Returns
    /// Blocklist statistics summary
    pub async fn get_blocklist_stats(&self) -> ExchangeResult<BlocklistStats> {
        let servers = self.get_blocklist().await?;
        Ok(BlocklistStats::from_servers(&servers))
    }

    /// Get unique malware families in current blocklist
    ///
    /// # Returns
    /// Vector of unique malware family names
    pub async fn get_malware_families(&self) -> ExchangeResult<Vec<String>> {
        let servers = self.get_blocklist().await?;
        let mut families: Vec<String> = servers
            .iter()
            .map(|s| s.malware.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        families.sort();
        Ok(families)
    }

    /// Get unique countries in current blocklist
    ///
    /// # Returns
    /// Vector of unique country codes
    pub async fn get_countries(&self) -> ExchangeResult<Vec<String>> {
        let servers = self.get_blocklist().await?;
        let mut countries: Vec<String> = servers
            .iter()
            .map(|s| s.country.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        countries.sort();
        Ok(countries)
    }

    /// Get unique ASNs in current blocklist
    ///
    /// # Returns
    /// Vector of unique ASNs with their names
    pub async fn get_asns(&self) -> ExchangeResult<Vec<(u32, String)>> {
        let servers = self.get_blocklist().await?;
        let mut asns: Vec<(u32, String)> = servers
            .iter()
            .map(|s| (s.as_number, s.as_name.clone()))
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();
        asns.sort_by_key(|(asn, _)| *asn);
        Ok(asns)
    }

    /// Check if blocklist is empty
    ///
    /// Returns true if there are no C2 servers in the current dataset.
    /// This is normal during periods after successful law enforcement operations.
    ///
    /// # Returns
    /// True if blocklist is empty, false otherwise
    pub async fn is_empty(&self) -> ExchangeResult<bool> {
        let servers = self.get_blocklist().await?;
        Ok(servers.is_empty())
    }
}

impl Default for FeodoTrackerConnector {
    fn default() -> Self {
        Self::new()
    }
}
