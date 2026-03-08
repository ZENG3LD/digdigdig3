//! Shodan response parsers
//!
//! Parse JSON responses to domain types based on Shodan API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct ShodanParser;

impl ShodanParser {
    // ═══════════════════════════════════════════════════════════════════════
    // SHODAN-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse host information
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "ip_str": "8.8.8.8",
    ///   "ports": [53, 443],
    ///   "hostnames": ["dns.google"],
    ///   "org": "Google LLC",
    ///   "os": null,
    ///   "data": [
    ///     {
    ///       "port": 53,
    ///       "transport": "udp",
    ///       "product": "Google DNS",
    ///       "version": null,
    ///       "data": "..."
    ///     }
    ///   ],
    ///   "vulns": ["CVE-2021-1234"]
    /// }
    /// ```
    pub fn parse_host(data: &Value) -> ExchangeResult<ShodanHost> {
        let ip = Self::require_str(data, "ip_str")?.to_string();

        let ports = data
            .get("ports")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_u64().map(|n| n as u16)).collect())
            .unwrap_or_default();

        let hostnames = data
            .get("hostnames")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let org = Self::get_str(data, "org").map(|s| s.to_string());
        let os = Self::get_str(data, "os").map(|s| s.to_string());

        let services = data
            .get("data")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|service_data| Self::parse_service(service_data).ok())
                    .collect()
            })
            .unwrap_or_default();

        let vulns = data
            .get("vulns")
            .and_then(|v| {
                // Vulns can be either an array or an object
                if let Some(arr) = v.as_array() {
                    Some(arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                } else {
                    v.as_object().map(|obj| obj.keys().map(|k| k.to_string()).collect())
                }
            })
            .unwrap_or_default();

        Ok(ShodanHost {
            ip,
            ports,
            hostnames,
            org,
            os,
            services,
            vulns,
        })
    }

    /// Parse service information
    fn parse_service(data: &Value) -> ExchangeResult<ShodanService> {
        let port = Self::require_u64(data, "port")? as u16;
        let transport = Self::require_str(data, "transport")?.to_string();
        let product = Self::get_str(data, "product").map(|s| s.to_string());
        let version = Self::get_str(data, "version").map(|s| s.to_string());
        let banner = Self::get_str(data, "data").map(|s| s.to_string());

        Ok(ShodanService {
            port,
            transport,
            product,
            version,
            banner,
        })
    }

    /// Parse search result
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total": 1234,
    ///   "matches": [...]
    /// }
    /// ```
    pub fn parse_search_result(data: &Value) -> ExchangeResult<ShodanSearchResult> {
        let total = Self::require_u64(data, "total")?;

        let matches = data
            .get("matches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'matches' array".to_string()))?;

        let hosts = matches
            .iter()
            .filter_map(|host_data| Self::parse_host(host_data).ok())
            .collect();

        Ok(ShodanSearchResult { total, matches: hosts })
    }

    /// Parse API info
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "scan_credits": 100,
    ///   "query_credits": 1000,
    ///   "plan": "edu"
    /// }
    /// ```
    pub fn parse_api_info(data: &Value) -> ExchangeResult<ShodanApiInfo> {
        let scan_credits = Self::get_u64(data, "scan_credits").unwrap_or(0);
        let query_credits = Self::get_u64(data, "query_credits").unwrap_or(0);
        let plan = Self::require_str(data, "plan")?.to_string();

        Ok(ShodanApiInfo {
            scan_credits,
            query_credits,
            plan,
        })
    }

    /// Parse DNS result (resolve/reverse)
    ///
    /// Example resolve response:
    /// ```json
    /// {
    ///   "google.com": "142.250.185.46"
    /// }
    /// ```
    ///
    /// Example reverse response:
    /// ```json
    /// {
    ///   "8.8.8.8": "dns.google"
    /// }
    /// ```
    pub fn parse_dns_results(data: &Value) -> ExchangeResult<Vec<ShodanDnsResult>> {
        let obj = data
            .as_object()
            .ok_or_else(|| ExchangeError::Parse("DNS result is not an object".to_string()))?;

        let results = obj
            .iter()
            .map(|(key, value)| {
                let value_str = value
                    .as_str()
                    .ok_or_else(|| ExchangeError::Parse(format!("Invalid DNS value for {}", key)))?
                    .to_string();

                Ok(ShodanDnsResult {
                    hostname: key.clone(),
                    ip: value_str,
                })
            })
            .collect::<ExchangeResult<Vec<_>>>()?;

        Ok(results)
    }

    /// Parse simple string value (for myip)
    pub fn parse_string(data: &Value) -> ExchangeResult<String> {
        data.as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Expected string value".to_string()))
    }

    /// Parse count result
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total": 1234
    /// }
    /// ```
    pub fn parse_count(data: &Value) -> ExchangeResult<u64> {
        Self::require_u64(data, "total")
    }

    /// Parse ports list
    ///
    /// Example response:
    /// ```json
    /// [21, 22, 23, 25, 80, 443, ...]
    /// ```
    pub fn parse_ports(data: &Value) -> ExchangeResult<Vec<u16>> {
        let arr = data
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Ports is not an array".to_string()))?;

        Ok(arr.iter().filter_map(|v| v.as_u64().map(|n| n as u16)).collect())
    }

    /// Parse protocols list
    ///
    /// Example response:
    /// ```json
    /// ["http", "https", "ssh", "telnet", ...]
    /// ```
    pub fn parse_protocols(data: &Value) -> ExchangeResult<Vec<String>> {
        let arr = data
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Protocols is not an array".to_string()))?;

        Ok(arr
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }
        Ok(())
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
}

// ═══════════════════════════════════════════════════════════════════════════
// SHODAN-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Shodan host information
#[derive(Debug, Clone)]
pub struct ShodanHost {
    pub ip: String,
    pub ports: Vec<u16>,
    pub hostnames: Vec<String>,
    pub org: Option<String>,
    pub os: Option<String>,
    pub services: Vec<ShodanService>,
    pub vulns: Vec<String>,
}

/// Shodan service information
#[derive(Debug, Clone)]
pub struct ShodanService {
    pub port: u16,
    pub transport: String,
    pub product: Option<String>,
    pub version: Option<String>,
    pub banner: Option<String>,
}

/// Shodan search result
#[derive(Debug, Clone)]
pub struct ShodanSearchResult {
    pub total: u64,
    pub matches: Vec<ShodanHost>,
}

/// Shodan API plan information
#[derive(Debug, Clone)]
pub struct ShodanApiInfo {
    pub scan_credits: u64,
    pub query_credits: u64,
    pub plan: String,
}

/// Shodan DNS result
#[derive(Debug, Clone)]
pub struct ShodanDnsResult {
    pub hostname: String,
    pub ip: String,
}
