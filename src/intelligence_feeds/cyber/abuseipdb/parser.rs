//! AbuseIPDB response parsers
//!
//! Parse JSON responses to domain types based on AbuseIPDB API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct AbuseIpdbParser;

impl AbuseIpdbParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ABUSEIPDB-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse IP check report
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": {
    ///     "ipAddress": "192.0.2.1",
    ///     "isPublic": true,
    ///     "ipVersion": 4,
    ///     "isWhitelisted": false,
    ///     "abuseConfidenceScore": 100,
    ///     "countryCode": "US",
    ///     "usageType": "Data Center/Web Hosting/Transit",
    ///     "isp": "Example ISP",
    ///     "domain": "example.com",
    ///     "totalReports": 42,
    ///     "numDistinctUsers": 12,
    ///     "lastReportedAt": "2024-01-15T10:30:00+00:00"
    ///   }
    /// }
    /// ```
    pub fn parse_check(data: &Value) -> ExchangeResult<AbuseIpReport> {
        let data_obj = data
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let ip_address = Self::require_str(data_obj, "ipAddress")?.to_string();
        let is_public = Self::get_bool(data_obj, "isPublic").unwrap_or(true);
        let abuse_confidence_score = Self::get_i64(data_obj, "abuseConfidenceScore").unwrap_or(0);
        let country_code = Self::get_str(data_obj, "countryCode").map(|s| s.to_string());
        let usage_type = Self::get_str(data_obj, "usageType").map(|s| s.to_string());
        let isp = Self::get_str(data_obj, "isp").map(|s| s.to_string());
        let domain = Self::get_str(data_obj, "domain").map(|s| s.to_string());
        let total_reports = Self::get_u64(data_obj, "totalReports").unwrap_or(0);
        let num_distinct_users = Self::get_u64(data_obj, "numDistinctUsers").unwrap_or(0);
        let last_reported_at = Self::get_str(data_obj, "lastReportedAt").map(|s| s.to_string());
        let is_whitelisted = Self::get_bool(data_obj, "isWhitelisted").unwrap_or(false);

        Ok(AbuseIpReport {
            ip_address,
            is_public,
            abuse_confidence_score,
            country_code,
            usage_type,
            isp,
            domain,
            total_reports,
            num_distinct_users,
            last_reported_at,
            is_whitelisted,
        })
    }

    /// Parse blacklist entries
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": [
    ///     {
    ///       "ipAddress": "192.0.2.1",
    ///       "abuseConfidenceScore": 100,
    ///       "lastReportedAt": "2024-01-15T10:30:00+00:00"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_blacklist(data: &Value) -> ExchangeResult<Vec<BlacklistEntry>> {
        let data_array = data
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let entries = data_array
            .iter()
            .filter_map(|entry_data| {
                let ip_address = Self::require_str(entry_data, "ipAddress").ok()?.to_string();
                let abuse_confidence_score = Self::get_i64(entry_data, "abuseConfidenceScore").unwrap_or(0);
                let last_reported_at = Self::get_str(entry_data, "lastReportedAt").map(|s| s.to_string());

                Some(BlacklistEntry {
                    ip_address,
                    abuse_confidence_score,
                    last_reported_at,
                })
            })
            .collect();

        Ok(entries)
    }

    /// Parse check-block response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": {
    ///     "networkAddress": "192.0.2.0/24",
    ///     "netmask": "255.255.255.0",
    ///     "minAddress": "192.0.2.0",
    ///     "maxAddress": "192.0.2.255",
    ///     "numPossibleHosts": 256,
    ///     "addressSpaceDesc": "Public",
    ///     "reportedAddress": [
    ///       {
    ///         "ipAddress": "192.0.2.1",
    ///         "numReports": 42,
    ///         "mostRecentReport": "2024-01-15T10:30:00+00:00",
    ///         "abuseConfidenceScore": 100,
    ///         "countryCode": "US"
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    pub fn parse_check_block(data: &Value) -> ExchangeResult<CheckBlockReport> {
        let data_obj = data
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let network_address = Self::require_str(data_obj, "networkAddress")?.to_string();
        let netmask = Self::get_str(data_obj, "netmask").map(|s| s.to_string());
        let min_address = Self::get_str(data_obj, "minAddress").map(|s| s.to_string());
        let max_address = Self::get_str(data_obj, "maxAddress").map(|s| s.to_string());
        let num_possible_hosts = Self::get_u64(data_obj, "numPossibleHosts").unwrap_or(0);
        let address_space_desc = Self::get_str(data_obj, "addressSpaceDesc").map(|s| s.to_string());

        let reported_addresses = data_obj
            .get("reportedAddress")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        let ip_address = Self::require_str(entry, "ipAddress").ok()?.to_string();
                        let num_reports = Self::get_u64(entry, "numReports").unwrap_or(0);
                        let most_recent_report = Self::get_str(entry, "mostRecentReport").map(|s| s.to_string());
                        let abuse_confidence_score = Self::get_i64(entry, "abuseConfidenceScore").unwrap_or(0);
                        let country_code = Self::get_str(entry, "countryCode").map(|s| s.to_string());

                        Some(BlockReportedAddress {
                            ip_address,
                            num_reports,
                            most_recent_report,
                            abuse_confidence_score,
                            country_code,
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(CheckBlockReport {
            network_address,
            netmask,
            min_address,
            max_address,
            num_possible_hosts,
            address_space_desc,
            reported_addresses,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(errors) = response.get("errors") {
            if let Some(error_array) = errors.as_array() {
                if let Some(first_error) = error_array.first() {
                    let message = first_error
                        .get("detail")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error")
                        .to_string();

                    let status = first_error
                        .get("status")
                        .and_then(|v| v.as_i64())
                        .unwrap_or(0);

                    return Err(ExchangeError::Api {
                        code: status as i32,
                        message,
                    });
                }
            }
        }

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

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ABUSEIPDB-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// AbuseIPDB IP check report
#[derive(Debug, Clone)]
pub struct AbuseIpReport {
    pub ip_address: String,
    pub is_public: bool,
    pub abuse_confidence_score: i64,
    pub country_code: Option<String>,
    pub usage_type: Option<String>,
    pub isp: Option<String>,
    pub domain: Option<String>,
    pub total_reports: u64,
    pub num_distinct_users: u64,
    pub last_reported_at: Option<String>,
    pub is_whitelisted: bool,
}

/// AbuseIPDB blacklist entry
#[derive(Debug, Clone)]
pub struct BlacklistEntry {
    pub ip_address: String,
    pub abuse_confidence_score: i64,
    pub last_reported_at: Option<String>,
}

/// AbuseIPDB check-block report
#[derive(Debug, Clone)]
pub struct CheckBlockReport {
    pub network_address: String,
    pub netmask: Option<String>,
    pub min_address: Option<String>,
    pub max_address: Option<String>,
    pub num_possible_hosts: u64,
    pub address_space_desc: Option<String>,
    pub reported_addresses: Vec<BlockReportedAddress>,
}

/// Reported address within a block
#[derive(Debug, Clone)]
pub struct BlockReportedAddress {
    pub ip_address: String,
    pub num_reports: u64,
    pub most_recent_report: Option<String>,
    pub abuse_confidence_score: i64,
    pub country_code: Option<String>,
}

/// AbuseIPDB abuse category
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbuseCategory {
    DnsCompromise = 1,
    DnsPoison = 2,
    FraudOrders = 3,
    DDoS = 4,
    FtpBrute = 5,
    PingOfDeath = 6,
    Phishing = 7,
    FraudVoIP = 8,
    OpenProxy = 9,
    WebSpam = 10,
    EmailSpam = 11,
    BlogSpam = 12,
    VpnIP = 13,
    PortScan = 14,
    Hacking = 15,
    SqlInjection = 16,
    Spoofing = 17,
    BruteForce = 18,
    BadWebBot = 19,
    ExploitedHost = 20,
    WebAppAttack = 21,
    SSH = 22,
    IoTTargeted = 23,
}

impl AbuseCategory {
    /// Get all categories as (id, name) pairs
    pub fn all() -> Vec<(u8, &'static str)> {
        vec![
            (1, "DNS Compromise"),
            (2, "DNS Poisoning"),
            (3, "Fraud Orders"),
            (4, "DDoS Attack"),
            (5, "FTP Brute-Force"),
            (6, "Ping of Death"),
            (7, "Phishing"),
            (8, "Fraud VoIP"),
            (9, "Open Proxy"),
            (10, "Web Spam"),
            (11, "Email Spam"),
            (12, "Blog Spam"),
            (13, "VPN IP"),
            (14, "Port Scan"),
            (15, "Hacking"),
            (16, "SQL Injection"),
            (17, "Spoofing"),
            (18, "Brute-Force"),
            (19, "Bad Web Bot"),
            (20, "Exploited Host"),
            (21, "Web App Attack"),
            (22, "SSH"),
            (23, "IoT Targeted"),
        ]
    }

    /// Get category name
    pub fn name(&self) -> &'static str {
        Self::all()[*self as usize - 1].1
    }

    /// Get category ID
    pub fn id(&self) -> u8 {
        *self as u8
    }
}
