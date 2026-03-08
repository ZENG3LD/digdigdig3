//! AlienVault OTX response parsers
//!
//! Parse JSON responses to domain types based on OTX API response formats.

use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct OtxParser;

impl OtxParser {
    // ═══════════════════════════════════════════════════════════════════════
    // OTX-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse pulse information
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "id": "abc123",
    ///   "name": "Malware Campaign",
    ///   "description": "Active malware campaign targeting...",
    ///   "author_name": "researcher",
    ///   "created": "2024-01-15T10:30:00",
    ///   "modified": "2024-01-16T14:20:00",
    ///   "tags": ["malware", "apt"],
    ///   "indicator_count": 42,
    ///   "targeted_countries": ["US", "UK"]
    /// }
    /// ```
    pub fn parse_pulse(data: &Value) -> ExchangeResult<OtxPulse> {
        let id = Self::require_str(data, "id")?.to_string();
        let name = Self::require_str(data, "name")?.to_string();
        let description = Self::get_str(data, "description").map(|s| s.to_string());
        let author = Self::get_str(data, "author_name").map(|s| s.to_string());
        let created = Self::get_str(data, "created").map(|s| s.to_string());
        let modified = Self::get_str(data, "modified").map(|s| s.to_string());

        let tags = data
            .get("tags")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        let indicators_count = Self::get_u64(data, "indicator_count").unwrap_or(0);

        let targeted_countries = data
            .get("targeted_countries")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();

        Ok(OtxPulse {
            id,
            name,
            description,
            author,
            created,
            modified,
            tags,
            indicators_count,
            targeted_countries,
        })
    }

    /// Parse pulses list from subscribed or activity endpoints
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [...]
    /// }
    /// ```
    pub fn parse_pulses(data: &Value) -> ExchangeResult<Vec<OtxPulse>> {
        let results = data
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let pulses = results
            .iter()
            .filter_map(|pulse_data| Self::parse_pulse(pulse_data).ok())
            .collect();

        Ok(pulses)
    }

    /// Parse indicator information
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "indicator": "192.0.2.1",
    ///   "type": "IPv4",
    ///   "description": "Malicious IP",
    ///   "created": "2024-01-15T10:30:00"
    /// }
    /// ```
    pub fn parse_indicator(data: &Value) -> ExchangeResult<OtxIndicator> {
        let indicator = Self::require_str(data, "indicator")?.to_string();
        let type_name = Self::require_str(data, "type")?.to_string();
        let description = Self::get_str(data, "description").map(|s| s.to_string());
        let created = Self::get_str(data, "created").map(|s| s.to_string());

        Ok(OtxIndicator {
            indicator,
            type_name,
            description,
            created,
        })
    }

    /// Parse IP reputation information
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "indicator": "8.8.8.8",
    ///   "reputation": 0,
    ///   "country_name": "United States",
    ///   "city": "Mountain View",
    ///   "asn": "AS15169",
    ///   "pulse_info": {
    ///     "count": 5
    ///   }
    /// }
    /// ```
    pub fn parse_ip_reputation(data: &Value) -> ExchangeResult<OtxIpReputation> {
        let ip = Self::require_str(data, "indicator")?.to_string();
        let reputation = Self::get_i64(data, "reputation").unwrap_or(0);
        let country = Self::get_str(data, "country_name").map(|s| s.to_string());
        let city = Self::get_str(data, "city").map(|s| s.to_string());
        let asn = Self::get_str(data, "asn").map(|s| s.to_string());

        let pulse_count = data
            .get("pulse_info")
            .and_then(|v| v.get("count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        Ok(OtxIpReputation {
            ip,
            reputation,
            country,
            city,
            asn,
            pulse_count,
        })
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

        if let Some(detail) = response.get("detail") {
            let message = detail
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
}

// ═══════════════════════════════════════════════════════════════════════════
// OTX-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// OTX threat intelligence pulse
#[derive(Debug, Clone)]
pub struct OtxPulse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub author: Option<String>,
    pub created: Option<String>,
    pub modified: Option<String>,
    pub tags: Vec<String>,
    pub indicators_count: u64,
    pub targeted_countries: Vec<String>,
}

/// OTX indicator of compromise (IOC)
#[derive(Debug, Clone)]
pub struct OtxIndicator {
    pub indicator: String,
    pub type_name: String,
    pub description: Option<String>,
    pub created: Option<String>,
}

/// OTX IP reputation information
#[derive(Debug, Clone)]
pub struct OtxIpReputation {
    pub ip: String,
    pub reputation: i64,
    pub country: Option<String>,
    pub city: Option<String>,
    pub asn: Option<String>,
    pub pulse_count: u64,
}
