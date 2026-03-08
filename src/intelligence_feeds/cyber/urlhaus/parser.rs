//! URLhaus response parsers
//!
//! Parse JSON responses to domain types based on URLhaus API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct UrlhausParser;

impl UrlhausParser {
    // ═══════════════════════════════════════════════════════════════════════
    // URLHAUS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse recent URLs response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "query_status": "ok",
    ///   "urls": [...]
    /// }
    /// ```
    pub fn parse_recent_urls(data: &Value) -> ExchangeResult<Vec<UrlhausEntry>> {
        // Check query status
        let query_status = data
            .get("query_status")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if query_status != "ok" {
            return Err(ExchangeError::Api {
                code: 0,
                message: format!("Query failed with status: {}", query_status),
            });
        }

        let urls = data
            .get("urls")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'urls' array".to_string()))?;

        let entries = urls
            .iter()
            .filter_map(|url_data| Self::parse_url_entry(url_data).ok())
            .collect();

        Ok(entries)
    }

    /// Parse single URL entry
    ///
    /// Example entry:
    /// ```json
    /// {
    ///   "id": "12345",
    ///   "url": "http://malicious-site.com/payload.exe",
    ///   "url_status": "online",
    ///   "threat": "malware_download",
    ///   "host": "malicious-site.com",
    ///   "date_added": "2024-01-15 10:30:00 UTC",
    ///   "tags": ["Dridex", "Emotet"],
    ///   "reporter": "abuse_ch",
    ///   "larted": "true"
    /// }
    /// ```
    pub fn parse_url_entry(data: &Value) -> ExchangeResult<UrlhausEntry> {
        let id = Self::require_str(data, "id")?.to_string();
        let url = Self::require_str(data, "url")?.to_string();
        let url_status = Self::get_str(data, "url_status").map(|s| s.to_string());

        let threat = Self::get_str(data, "threat")
            .and_then(UrlhausThreatType::from_str);

        let host = Self::get_str(data, "host").map(|s| s.to_string());
        let date_added = Self::get_str(data, "date_added").map(|s| s.to_string());
        let reporter = Self::get_str(data, "reporter").map(|s| s.to_string());
        let larted = Self::get_str(data, "larted").map(|s| s == "true").unwrap_or(false);

        let tags = data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(UrlhausEntry {
            id,
            url,
            url_status,
            threat,
            host,
            date_added,
            tags,
            reporter,
            larted,
        })
    }

    /// Parse URL lookup response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "query_status": "ok",
    ///   "id": "12345",
    ///   "url": "http://example.com/bad",
    ///   "url_status": "online",
    ///   "host": "example.com",
    ///   "date_added": "2024-01-15 10:30:00 UTC",
    ///   "threat": "malware_download",
    ///   "blacklists": {...},
    ///   "tags": ["Dridex"],
    ///   "payloads": [...]
    /// }
    /// ```
    pub fn parse_url_info(data: &Value) -> ExchangeResult<UrlhausUrlInfo> {
        let query_status = Self::require_str(data, "query_status")?;

        if query_status != "ok" {
            return Err(ExchangeError::Api {
                code: 0,
                message: format!("URL not found or query failed: {}", query_status),
            });
        }

        let id = Self::require_str(data, "id")?.to_string();
        let url = Self::require_str(data, "url")?.to_string();
        let url_status = Self::get_str(data, "url_status").map(|s| s.to_string());
        let host = Self::get_str(data, "host").map(|s| s.to_string());
        let date_added = Self::get_str(data, "date_added").map(|s| s.to_string());

        let threat = Self::get_str(data, "threat")
            .and_then(UrlhausThreatType::from_str);

        let tags = data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let payloads = data
            .get("payloads")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|p| Self::parse_payload(p).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(UrlhausUrlInfo {
            id,
            url,
            url_status,
            host,
            date_added,
            threat,
            tags,
            payloads,
        })
    }

    /// Parse host lookup response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "query_status": "ok",
    ///   "host": "example.com",
    ///   "firstseen": "2024-01-01",
    ///   "url_count": 42,
    ///   "blacklists": {...},
    ///   "urls": [...]
    /// }
    /// ```
    pub fn parse_host_info(data: &Value) -> ExchangeResult<UrlhausHostInfo> {
        let query_status = Self::require_str(data, "query_status")?;

        if query_status != "ok" {
            return Err(ExchangeError::Api {
                code: 0,
                message: format!("Host not found or query failed: {}", query_status),
            });
        }

        let host = Self::require_str(data, "host")?.to_string();
        let firstseen = Self::get_str(data, "firstseen").map(|s| s.to_string());
        let url_count = Self::get_u64(data, "url_count").unwrap_or(0);

        let urls = data
            .get("urls")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|u| Self::parse_url_entry(u).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(UrlhausHostInfo {
            host,
            firstseen,
            url_count,
            urls,
        })
    }

    /// Parse payload information
    ///
    /// Example payload:
    /// ```json
    /// {
    ///   "firstseen": "2024-01-15",
    ///   "filename": "malware.exe",
    ///   "file_type": "exe",
    ///   "response_size": "524288",
    ///   "response_md5": "abc123...",
    ///   "response_sha256": "def456...",
    ///   "virustotal": {...}
    /// }
    /// ```
    fn parse_payload(data: &Value) -> ExchangeResult<UrlhausPayload> {
        let firstseen = Self::get_str(data, "firstseen").map(|s| s.to_string());
        let filename = Self::get_str(data, "filename").map(|s| s.to_string());
        let file_type = Self::get_str(data, "file_type").map(|s| s.to_string());
        let response_size = Self::get_str(data, "response_size").map(|s| s.to_string());
        let response_md5 = Self::get_str(data, "response_md5").map(|s| s.to_string());
        let response_sha256 = Self::get_str(data, "response_sha256").map(|s| s.to_string());

        Ok(UrlhausPayload {
            firstseen,
            filename,
            file_type,
            response_size,
            response_md5,
            response_sha256,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(query_status) = response.get("query_status").and_then(|v| v.as_str()) {
            if query_status == "no_results" {
                return Err(ExchangeError::Api {
                    code: 0,
                    message: "No results found".to_string(),
                });
            }

            if query_status == "invalid_url" {
                return Err(ExchangeError::Api {
                    code: 0,
                    message: "Invalid URL provided".to_string(),
                });
            }

            if query_status == "invalid_host" {
                return Err(ExchangeError::Api {
                    code: 0,
                    message: "Invalid host provided".to_string(),
                });
            }
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
}

// ═══════════════════════════════════════════════════════════════════════════
// URLHAUS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// URLhaus malicious URL entry
#[derive(Debug, Clone)]
pub struct UrlhausEntry {
    pub id: String,
    pub url: String,
    pub url_status: Option<String>,
    pub threat: Option<UrlhausThreatType>,
    pub host: Option<String>,
    pub date_added: Option<String>,
    pub tags: Vec<String>,
    pub reporter: Option<String>,
    pub larted: bool,
}

/// URLhaus threat type classification
#[derive(Debug, Clone)]
pub enum UrlhausThreatType {
    MalwareDownload,
    Phishing,
    CryptoMining,
    Other(String),
}

impl UrlhausThreatType {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "malware_download" => Some(Self::MalwareDownload),
            "phishing" => Some(Self::Phishing),
            "crypto_mining" => Some(Self::CryptoMining),
            other => Some(Self::Other(other.to_string())),
        }
    }
}

/// Detailed URL information from URL lookup
#[derive(Debug, Clone)]
pub struct UrlhausUrlInfo {
    pub id: String,
    pub url: String,
    pub url_status: Option<String>,
    pub host: Option<String>,
    pub date_added: Option<String>,
    pub threat: Option<UrlhausThreatType>,
    pub tags: Vec<String>,
    pub payloads: Vec<UrlhausPayload>,
}

/// Detailed host information from host lookup
#[derive(Debug, Clone)]
pub struct UrlhausHostInfo {
    pub host: String,
    pub firstseen: Option<String>,
    pub url_count: u64,
    pub urls: Vec<UrlhausEntry>,
}

/// Malware payload information
#[derive(Debug, Clone)]
pub struct UrlhausPayload {
    pub firstseen: Option<String>,
    pub filename: Option<String>,
    pub file_type: Option<String>,
    pub response_size: Option<String>,
    pub response_md5: Option<String>,
    pub response_sha256: Option<String>,
}
