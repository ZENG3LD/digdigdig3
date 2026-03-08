//! RIPE NCC response parsers
//!
//! Parse JSON responses to domain types based on RIPE NCC API response formats.
//!
//! All responses follow the pattern: {"status":"ok","data":{...}} or {"status":"error","messages":[...]}

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct RipeNccParser;

impl RipeNccParser {
    // ═══════════════════════════════════════════════════════════════════════
    // RIPE NCC-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse country resource stats
    ///
    /// Example response data:
    /// ```json
    /// {
    ///   "resource": "NL",
    ///   "stats": {
    ///     "ipv4": {"count": 123456},
    ///     "ipv6": {"count": 789},
    ///     "asn": {"count": 42}
    ///   }
    /// }
    /// ```
    pub fn parse_country_stats(data: &Value) -> ExchangeResult<RipeCountryStats> {
        let resource = Self::require_str(data, "resource")?.to_string();

        let stats = data
            .get("stats")
            .ok_or_else(|| ExchangeError::Parse("Missing 'stats' object".to_string()))?;

        let ip4_count = stats
            .get("ipv4")
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let ip6_count = stats
            .get("ipv6")
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let asn_count = stats
            .get("asn")
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(RipeCountryStats {
            resource,
            ip4_count,
            ip6_count,
            asn_count,
        })
    }

    /// Parse AS overview
    pub fn parse_as_overview(data: &Value) -> ExchangeResult<RipeAsOverview> {
        let resource = Self::require_str(data, "resource")?.to_string();
        let holder = Self::require_str(data, "holder")?.to_string();
        let announced = Self::get_bool(data, "announced").unwrap_or(false);
        let block = data
            .get("block")
            .and_then(|v| v.get("resource"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse ASN from resource string (format: "ASXXXX")
        let asn = resource
            .trim_start_matches("AS")
            .parse::<u64>()
            .map_err(|_| ExchangeError::Parse(format!("Invalid ASN format: {}", resource)))?;

        Ok(RipeAsOverview {
            resource,
            asn,
            holder,
            announced,
            block,
        })
    }

    /// Parse routing status
    pub fn parse_routing_status(data: &Value) -> ExchangeResult<RipeRoutingStatus> {
        let resource = Self::require_str(data, "resource")?.to_string();
        let visibility = Self::get_f64(data, "visibility").unwrap_or(0.0);
        let announced = Self::get_bool(data, "announced").unwrap_or(false);
        let first_seen = data
            .get("first_seen")
            .and_then(|v| v.get("time"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let last_seen = data
            .get("last_seen")
            .and_then(|v| v.get("time"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(RipeRoutingStatus {
            resource,
            visibility,
            announced,
            first_seen,
            last_seen,
        })
    }

    /// Parse BGP state
    pub fn parse_bgp_state(data: &Value) -> ExchangeResult<RipeBgpState> {
        let resource = Self::require_str(data, "resource")?.to_string();

        let ris_peers = data
            .get("ris_peers")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'ris_peers' array".to_string()))?;

        let nr_prefixes = ris_peers.len() as u32;

        // Extract AS path from first peer (if available)
        let as_path = ris_peers
            .first()
            .and_then(|peer| peer.get("as_path"))
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64())
                    .collect()
            })
            .unwrap_or_default();

        Ok(RipeBgpState {
            resource,
            nr_prefixes,
            as_path,
        })
    }

    /// Parse announced prefixes
    pub fn parse_announced_prefixes(data: &Value) -> ExchangeResult<Vec<RipeAnnouncedPrefix>> {
        let prefixes = data
            .get("prefixes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'prefixes' array".to_string()))?;

        prefixes
            .iter()
            .map(|prefix_data| {
                let prefix = Self::require_str(prefix_data, "prefix")?.to_string();

                let timelines = prefix_data
                    .get("timelines")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|timeline| {
                                let starttime = timeline.get("starttime")?.as_str()?.to_string();
                                let endtime = timeline.get("endtime").and_then(|v| v.as_str()).map(|s| s.to_string());
                                Some(RipeTimeline { starttime, endtime })
                            })
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(RipeAnnouncedPrefix { prefix, timelines })
            })
            .collect()
    }

    /// Parse ASN neighbours
    pub fn parse_asn_neighbours(data: &Value) -> ExchangeResult<Vec<RipeAsnNeighbour>> {
        let neighbours = data
            .get("neighbours")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'neighbours' array".to_string()))?;

        neighbours
            .iter()
            .map(|neighbour| {
                let asn = Self::require_u64(neighbour, "asn")?;
                let type_str = Self::require_str(neighbour, "type")?.to_string();
                let power = Self::get_f64(neighbour, "power").unwrap_or(0.0);

                Ok(RipeAsnNeighbour {
                    asn,
                    type_str,
                    power,
                })
            })
            .collect()
    }

    /// Parse network info
    pub fn parse_network_info(data: &Value) -> ExchangeResult<RipeNetworkInfo> {
        let asns = data
            .get("asns")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_u64()).collect())
            .unwrap_or_default();

        let prefix = Self::require_str(data, "prefix")?.to_string();

        Ok(RipeNetworkInfo { asns, prefix })
    }

    /// Parse RIR stats
    pub fn parse_rir_stats(data: &Value) -> ExchangeResult<RipeRirStats> {
        let resource = Self::require_str(data, "resource")?.to_string();

        let located_resources = data
            .get("located_resources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'located_resources' array".to_string()))?;

        let stats = located_resources
            .iter()
            .map(|entry| {
                let registry = Self::get_str(entry, "registry").unwrap_or("unknown").to_string();
                let resource_type = Self::get_str(entry, "type").unwrap_or("unknown").to_string();
                let count = Self::get_u64(entry, "count").unwrap_or(0);

                RipeRirStatEntry {
                    registry,
                    resource_type,
                    count,
                }
            })
            .collect();

        Ok(RipeRirStats { resource, stats })
    }

    /// Parse country resource list
    pub fn parse_country_resources(data: &Value) -> ExchangeResult<Vec<RipeCountryResource>> {
        let resources = data
            .get("resources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'resources' array".to_string()))?;

        resources
            .iter()
            .map(|res| {
                let resource = Self::require_str(res, "resource")?.to_string();
                let resource_type = Self::require_str(res, "type")?.to_string();

                Ok(RipeCountryResource {
                    resource,
                    resource_type,
                })
            })
            .collect()
    }

    /// Parse abuse contact
    pub fn parse_abuse_contact(data: &Value) -> ExchangeResult<RipeAbuseContact> {
        let abuse_contacts = data
            .get("abuse_contacts")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'abuse_contacts' array".to_string()))?;

        let first_contact = abuse_contacts
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'abuse_contacts' array".to_string()))?;

        let email = Self::require_str(first_contact, "email")?.to_string();
        let updated = Self::get_str(first_contact, "updated").map(|s| s.to_string());

        Ok(RipeAbuseContact { email, updated })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        let status = response
            .get("status")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        if status == "error" || status != "ok" {
            let messages = response
                .get("messages")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_else(|| "Unknown error".to_string());

            return Err(ExchangeError::Api {
                code: 0,
                message: messages,
            });
        }
        Ok(())
    }

    /// Extract data object from response
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object in response".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn require_u64(obj: &Value, field: &str) -> ExchangeResult<u64> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RIPE NCC-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// RIPE country resource statistics
#[derive(Debug, Clone)]
pub struct RipeCountryStats {
    pub resource: String,
    pub ip4_count: u64,
    pub ip6_count: u64,
    pub asn_count: u64,
}

/// RIPE AS overview
#[derive(Debug, Clone)]
pub struct RipeAsOverview {
    pub resource: String,
    pub asn: u64,
    pub holder: String,
    pub announced: bool,
    pub block: Option<String>,
}

/// RIPE routing status
#[derive(Debug, Clone)]
pub struct RipeRoutingStatus {
    pub resource: String,
    pub visibility: f64,
    pub announced: bool,
    pub first_seen: Option<String>,
    pub last_seen: Option<String>,
}

/// RIPE BGP state
#[derive(Debug, Clone)]
pub struct RipeBgpState {
    pub resource: String,
    pub nr_prefixes: u32,
    pub as_path: Vec<u64>,
}

/// RIPE announced prefix
#[derive(Debug, Clone)]
pub struct RipeAnnouncedPrefix {
    pub prefix: String,
    pub timelines: Vec<RipeTimeline>,
}

/// RIPE timeline entry
#[derive(Debug, Clone)]
pub struct RipeTimeline {
    pub starttime: String,
    pub endtime: Option<String>,
}

/// RIPE ASN neighbour
#[derive(Debug, Clone)]
pub struct RipeAsnNeighbour {
    pub asn: u64,
    pub type_str: String,
    pub power: f64,
}

/// RIPE network info
#[derive(Debug, Clone)]
pub struct RipeNetworkInfo {
    pub asns: Vec<u64>,
    pub prefix: String,
}

/// RIPE RIR stats
#[derive(Debug, Clone)]
pub struct RipeRirStats {
    pub resource: String,
    pub stats: Vec<RipeRirStatEntry>,
}

/// RIPE RIR stat entry
#[derive(Debug, Clone)]
pub struct RipeRirStatEntry {
    pub registry: String,
    pub resource_type: String,
    pub count: u64,
}

/// RIPE country resource
#[derive(Debug, Clone)]
pub struct RipeCountryResource {
    pub resource: String,
    pub resource_type: String,
}

/// RIPE abuse contact
#[derive(Debug, Clone)]
pub struct RipeAbuseContact {
    pub email: String,
    pub updated: Option<String>,
}
