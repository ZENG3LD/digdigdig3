//! VirusTotal response parsers
//!
//! Parse JSON responses to domain types based on VirusTotal API v3 response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct VirusTotalParser;

impl VirusTotalParser {
    // ═══════════════════════════════════════════════════════════════════════
    // VIRUSTOTAL-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse file report
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": {
    ///     "attributes": {
    ///       "sha256": "...",
    ///       "sha1": "...",
    ///       "md5": "...",
    ///       "meaningful_name": "example.exe",
    ///       "type_description": "Win32 EXE",
    ///       "size": 12345,
    ///       "times_submitted": 10,
    ///       "last_analysis_stats": {
    ///         "malicious": 5,
    ///         "suspicious": 2,
    ///         "undetected": 60,
    ///         "harmless": 0,
    ///         "timeout": 0,
    ///         "failure": 0
    ///       },
    ///       "last_analysis_date": 1640000000,
    ///       "reputation": -50,
    ///       "tags": ["malware", "trojan"]
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_file_report(data: &Value) -> ExchangeResult<VtFileReport> {
        let attributes = data
            .get("data")
            .and_then(|d| d.get("attributes"))
            .ok_or_else(|| ExchangeError::Parse("Missing 'data.attributes'".to_string()))?;

        let sha256 = Self::require_str(attributes, "sha256")?.to_string();
        let sha1 = Self::require_str(attributes, "sha1")?.to_string();
        let md5 = Self::require_str(attributes, "md5")?.to_string();

        let meaningful_name = Self::get_str(attributes, "meaningful_name").map(|s| s.to_string());
        let type_description = Self::get_str(attributes, "type_description").map(|s| s.to_string());
        let size = Self::get_u64(attributes, "size");
        let times_submitted = Self::get_u64(attributes, "times_submitted");

        let last_analysis_stats = attributes
            .get("last_analysis_stats")
            .map(Self::parse_analysis_stats)
            .transpose()?;

        let last_analysis_date = Self::get_u64(attributes, "last_analysis_date");
        let reputation = Self::get_i64(attributes, "reputation");

        let tags = attributes
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(VtFileReport {
            sha256,
            sha1,
            md5,
            meaningful_name,
            type_description,
            size,
            times_submitted,
            last_analysis_stats,
            last_analysis_date,
            reputation,
            tags,
        })
    }

    /// Parse analysis stats
    fn parse_analysis_stats(data: &Value) -> ExchangeResult<VtAnalysisStats> {
        let malicious = Self::get_u64(data, "malicious").unwrap_or(0);
        let suspicious = Self::get_u64(data, "suspicious").unwrap_or(0);
        let undetected = Self::get_u64(data, "undetected").unwrap_or(0);
        let harmless = Self::get_u64(data, "harmless").unwrap_or(0);
        let timeout = Self::get_u64(data, "timeout").unwrap_or(0);
        let failure = Self::get_u64(data, "failure").unwrap_or(0);

        Ok(VtAnalysisStats {
            malicious,
            suspicious,
            undetected,
            harmless,
            timeout,
            failure,
        })
    }

    /// Parse domain report
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": {
    ///     "id": "example.com",
    ///     "attributes": {
    ///       "registrar": "GoDaddy",
    ///       "creation_date": 1234567890,
    ///       "last_analysis_stats": {...},
    ///       "reputation": 100,
    ///       "categories": {"Alexa": "search engines"}
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_domain_report(data: &Value) -> ExchangeResult<VtDomainReport> {
        let data_obj = data
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data'".to_string()))?;

        let id = Self::require_str(data_obj, "id")?.to_string();

        let attributes = data_obj
            .get("attributes")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data.attributes'".to_string()))?;

        let registrar = Self::get_str(attributes, "registrar").map(|s| s.to_string());
        let creation_date = Self::get_u64(attributes, "creation_date");

        let last_analysis_stats = attributes
            .get("last_analysis_stats")
            .map(Self::parse_analysis_stats)
            .transpose()?;

        let reputation = Self::get_i64(attributes, "reputation");

        let categories = attributes
            .get("categories")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        Ok(VtDomainReport {
            id,
            registrar,
            creation_date,
            last_analysis_stats,
            reputation,
            categories,
        })
    }

    /// Parse IP address report
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": {
    ///     "id": "8.8.8.8",
    ///     "attributes": {
    ///       "country": "US",
    ///       "asn": 15169,
    ///       "as_owner": "Google LLC",
    ///       "last_analysis_stats": {...},
    ///       "reputation": 100
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_ip_report(data: &Value) -> ExchangeResult<VtIpReport> {
        let data_obj = data
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data'".to_string()))?;

        let id = Self::require_str(data_obj, "id")?.to_string();

        let attributes = data_obj
            .get("attributes")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data.attributes'".to_string()))?;

        let country = Self::get_str(attributes, "country").map(|s| s.to_string());
        let asn = Self::get_u64(attributes, "asn");
        let as_owner = Self::get_str(attributes, "as_owner").map(|s| s.to_string());

        let last_analysis_stats = attributes
            .get("last_analysis_stats")
            .map(Self::parse_analysis_stats)
            .transpose()?;

        let reputation = Self::get_i64(attributes, "reputation");

        Ok(VtIpReport {
            id,
            country,
            asn,
            as_owner,
            last_analysis_stats,
            reputation,
        })
    }

    /// Parse search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": [
    ///     {...},
    ///     {...}
    ///   ]
    /// }
    /// ```
    pub fn parse_search_results(data: &Value) -> ExchangeResult<Vec<Value>> {
        let data_arr = data
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data_arr.clone())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            let code = error
                .get("code")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            return Err(ExchangeError::Api {
                code: 0,
                message: format!("{}: {}", code, message),
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
}

// ═══════════════════════════════════════════════════════════════════════════
// VIRUSTOTAL-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// VirusTotal file report
#[derive(Debug, Clone)]
pub struct VtFileReport {
    pub sha256: String,
    pub sha1: String,
    pub md5: String,
    pub meaningful_name: Option<String>,
    pub type_description: Option<String>,
    pub size: Option<u64>,
    pub times_submitted: Option<u64>,
    pub last_analysis_stats: Option<VtAnalysisStats>,
    pub last_analysis_date: Option<u64>,
    pub reputation: Option<i64>,
    pub tags: Vec<String>,
}

/// VirusTotal analysis statistics
#[derive(Debug, Clone)]
pub struct VtAnalysisStats {
    pub malicious: u64,
    pub suspicious: u64,
    pub undetected: u64,
    pub harmless: u64,
    pub timeout: u64,
    pub failure: u64,
}

/// VirusTotal domain report
#[derive(Debug, Clone)]
pub struct VtDomainReport {
    pub id: String,
    pub registrar: Option<String>,
    pub creation_date: Option<u64>,
    pub last_analysis_stats: Option<VtAnalysisStats>,
    pub reputation: Option<i64>,
    pub categories: std::collections::HashMap<String, String>,
}

/// VirusTotal IP address report
#[derive(Debug, Clone)]
pub struct VtIpReport {
    pub id: String,
    pub country: Option<String>,
    pub asn: Option<u64>,
    pub as_owner: Option<String>,
    pub last_analysis_stats: Option<VtAnalysisStats>,
    pub reputation: Option<i64>,
}
