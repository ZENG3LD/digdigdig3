//! Feodo Tracker response parsers
//!
//! Parse JSON responses to domain types.
//!
//! Feodo Tracker returns JSON arrays with C2 server metadata.

use crate::core::types::{ExchangeError, ExchangeResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct FeodoTrackerParser;

impl FeodoTrackerParser {
    /// Parse Feodo Tracker JSON blocklist response
    ///
    /// Example response structure:
    /// ```json
    /// [
    ///   {
    ///     "ip_address": "162.243.103.246",
    ///     "port": 8080,
    ///     "status": "offline",
    ///     "hostname": null,
    ///     "as_number": 14061,
    ///     "as_name": "DIGITALOCEAN-ASN",
    ///     "country": "US",
    ///     "first_seen": "2022-06-04 21:24:53",
    ///     "last_online": "2026-02-06",
    ///     "malware": "Emotet"
    ///   }
    /// ]
    /// ```
    pub fn parse_blocklist(data: &Value) -> ExchangeResult<Vec<C2Server>> {
        let array = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected JSON array".to_string()))?;

        let mut servers = Vec::new();

        for item in array {
            let server = serde_json::from_value::<C2Server>(item.clone())
                .map_err(|e| ExchangeError::Parse(format!("Failed to parse C2 entry: {}", e)))?;
            servers.push(server);
        }

        Ok(servers)
    }

    /// Parse recommended blocklist (simple array of IPs)
    ///
    /// Example:
    /// ```json
    /// ["1.2.3.4", "5.6.7.8"]
    /// ```
    pub fn parse_recommended_blocklist(data: &Value) -> ExchangeResult<Vec<String>> {
        let array = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected JSON array".to_string()))?;

        let ips: Vec<String> = array
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        Ok(ips)
    }
}

// =============================================================================
// FEODO TRACKER-SPECIFIC TYPES
// =============================================================================

/// C2 server entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct C2Server {
    /// IPv4 address of C2 server
    pub ip_address: String,
    /// TCP port number
    pub port: u16,
    /// C2 operational status
    pub status: C2Status,
    /// Reverse DNS hostname (nullable)
    pub hostname: Option<String>,
    /// Autonomous System Number
    pub as_number: u32,
    /// ISP/hosting provider name
    pub as_name: String,
    /// ISO 3166-1 alpha-2 country code
    pub country: String,
    /// First detection timestamp (UTC)
    pub first_seen: String,
    /// Most recent activity date (UTC)
    pub last_online: String,
    /// Botnet family name
    pub malware: String,
}

/// C2 server status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum C2Status {
    /// C2 is currently active
    Online,
    /// C2 is not responding
    #[default]
    Offline,
}

impl C2Status {
    /// Check if server is online
    pub fn is_online(&self) -> bool {
        matches!(self, Self::Online)
    }

    /// Check if server is offline
    pub fn is_offline(&self) -> bool {
        matches!(self, Self::Offline)
    }
}

impl C2Server {
    /// Check if this C2 server is currently online
    pub fn is_online(&self) -> bool {
        self.status.is_online()
    }

    /// Check if this C2 server is offline
    pub fn is_offline(&self) -> bool {
        self.status.is_offline()
    }

    /// Get full endpoint address (IP:port)
    pub fn endpoint(&self) -> String {
        format!("{}:{}", self.ip_address, self.port)
    }

    /// Check if malware family matches (case-insensitive)
    pub fn is_malware_family(&self, family: &str) -> bool {
        self.malware.eq_ignore_ascii_case(family)
    }

    /// Check if country matches (case-insensitive)
    pub fn is_country(&self, country_code: &str) -> bool {
        self.country.eq_ignore_ascii_case(country_code)
    }
}

/// Blocklist summary statistics
#[derive(Debug, Clone)]
pub struct BlocklistStats {
    /// Total number of C2 servers
    pub total_servers: usize,
    /// Number of online servers
    pub online_count: usize,
    /// Number of offline servers
    pub offline_count: usize,
    /// Servers by malware family
    pub by_malware: Vec<(String, usize)>,
    /// Servers by country
    pub by_country: Vec<(String, usize)>,
}

impl BlocklistStats {
    /// Calculate statistics from a list of C2 servers
    pub fn from_servers(servers: &[C2Server]) -> Self {
        use std::collections::HashMap;

        let total_servers = servers.len();
        let online_count = servers.iter().filter(|s| s.is_online()).count();
        let offline_count = total_servers - online_count;

        // Count by malware family
        let mut malware_counts: HashMap<String, usize> = HashMap::new();
        for server in servers {
            *malware_counts.entry(server.malware.clone()).or_insert(0) += 1;
        }
        let mut by_malware: Vec<(String, usize)> = malware_counts.into_iter().collect();
        by_malware.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

        // Count by country
        let mut country_counts: HashMap<String, usize> = HashMap::new();
        for server in servers {
            *country_counts.entry(server.country.clone()).or_insert(0) += 1;
        }
        let mut by_country: Vec<(String, usize)> = country_counts.into_iter().collect();
        by_country.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by count descending

        Self {
            total_servers,
            online_count,
            offline_count,
            by_malware,
            by_country,
        }
    }
}
