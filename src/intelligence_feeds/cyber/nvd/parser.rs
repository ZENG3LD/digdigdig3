//! NVD response parsers
//!
//! Parse JSON responses to domain types based on NVD API response formats.

use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct NvdParser;

impl NvdParser {
    /// Parse CVE search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "resultsPerPage": 20,
    ///   "startIndex": 0,
    ///   "totalResults": 1234,
    ///   "vulnerabilities": [
    ///     {
    ///       "cve": {
    ///         "id": "CVE-2021-44228",
    ///         "descriptions": [{"lang": "en", "value": "Apache Log4j2..."}],
    ///         "published": "2021-12-10T10:15:09.203",
    ///         "lastModified": "2021-12-14T12:02:23.117",
    ///         "metrics": {
    ///           "cvssMetricV31": [{
    ///             "cvssData": {
    ///               "baseScore": 10.0,
    ///               "baseSeverity": "CRITICAL"
    ///             }
    ///           }]
    ///         },
    ///         "references": [{"url": "https://..."}],
    ///         "weaknesses": [{"description": [{"lang": "en", "value": "CWE-502"}]}]
    ///       }
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_cve_search(response: &Value) -> ExchangeResult<NvdSearchResult> {
        let results_per_page = Self::get_u32(response, "resultsPerPage")
            .ok_or_else(|| ExchangeError::Parse("Missing 'resultsPerPage'".to_string()))?;
        let start_index = Self::get_u32(response, "startIndex")
            .ok_or_else(|| ExchangeError::Parse("Missing 'startIndex'".to_string()))?;
        let total_results = Self::get_u32(response, "totalResults")
            .ok_or_else(|| ExchangeError::Parse("Missing 'totalResults'".to_string()))?;

        let vulnerabilities = response
            .get("vulnerabilities")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'vulnerabilities' array".to_string()))?;

        let mut cves = Vec::new();
        for vuln in vulnerabilities {
            if let Some(cve_data) = vuln.get("cve") {
                if let Ok(cve) = Self::parse_cve(cve_data) {
                    cves.push(cve);
                }
            }
        }

        Ok(NvdSearchResult {
            results_per_page,
            start_index,
            total_results,
            vulnerabilities: cves,
        })
    }

    /// Parse individual CVE entry
    fn parse_cve(cve: &Value) -> ExchangeResult<NvdCve> {
        let id = Self::require_str(cve, "id")?.to_string();

        // Parse description (first English description)
        let description = cve
            .get("descriptions")
            .and_then(|v| v.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|d| d.get("lang").and_then(|l| l.as_str()) == Some("en"))
                    .and_then(|d| d.get("value"))
                    .and_then(|v| v.as_str())
            })
            .unwrap_or("No description available")
            .to_string();

        let published = Self::require_str(cve, "published")?.to_string();
        let last_modified = Self::require_str(cve, "lastModified")?.to_string();

        // Parse CVSS v3 metrics
        let (cvss_v3_score, cvss_v3_severity) = Self::parse_cvss_v3(cve);

        // Parse references
        let references = Self::parse_references(cve);

        // Parse weaknesses (CWE)
        let weaknesses = Self::parse_weaknesses(cve);

        Ok(NvdCve {
            id,
            description,
            published,
            last_modified,
            cvss_v3_score,
            cvss_v3_severity,
            references,
            weaknesses,
        })
    }

    /// Parse CVSS v3 metrics
    fn parse_cvss_v3(cve: &Value) -> (Option<f64>, Option<String>) {
        cve.get("metrics")
            .and_then(|m| m.get("cvssMetricV31"))
            .and_then(|arr| arr.as_array())
            .and_then(|arr| arr.first())
            .and_then(|metric| metric.get("cvssData"))
            .map(|cvss_data| {
                let score = cvss_data
                    .get("baseScore")
                    .and_then(|v| v.as_f64());
                let severity = cvss_data
                    .get("baseSeverity")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string());
                (score, severity)
            })
            .unwrap_or((None, None))
    }

    /// Parse reference URLs
    fn parse_references(cve: &Value) -> Vec<String> {
        cve.get("references")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|r| r.get("url").and_then(|u| u.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Parse weakness data (CWE references)
    fn parse_weaknesses(cve: &Value) -> Vec<String> {
        cve.get("weaknesses")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|w| {
                        w.get("description")
                            .and_then(|d| d.as_array())
                            .and_then(|desc_arr| {
                                desc_arr.iter()
                                    .find(|d| d.get("lang").and_then(|l| l.as_str()) == Some("en"))
                                    .and_then(|d| d.get("value"))
                                    .and_then(|v| v.as_str())
                            })
                    })
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(message) = response.get("message").and_then(|v| v.as_str()) {
            // NVD returns HTTP status codes with error messages
            if message.contains("error") || message.contains("Error") {
                return Err(ExchangeError::Api {
                    code: 400,
                    message: message.to_string(),
                });
            }
        }
        Ok(())
    }

    // Helper methods
    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|n| n as u32)
    }

    fn _get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// NVD-specific types

/// NVD CVE (Common Vulnerability and Exposure) entry
#[derive(Debug, Clone)]
pub struct NvdCve {
    pub id: String,
    pub description: String,
    pub published: String,
    pub last_modified: String,
    pub cvss_v3_score: Option<f64>,
    pub cvss_v3_severity: Option<String>,
    pub references: Vec<String>,
    pub weaknesses: Vec<String>,
}

/// NVD search result
#[derive(Debug, Clone)]
pub struct NvdSearchResult {
    pub results_per_page: u32,
    pub start_index: u32,
    pub total_results: u32,
    pub vulnerabilities: Vec<NvdCve>,
}
