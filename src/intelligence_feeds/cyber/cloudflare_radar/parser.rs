//! Cloudflare Radar response parsers
//!
//! Parse JSON responses to domain types based on Cloudflare Radar API response formats.
//!
//! Cloudflare Radar API returns responses in the format:
//! ```json
//! {
//!   "success": true,
//!   "result": { ... actual data ... }
//! }
//! ```

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct CloudflareRadarParser;

impl CloudflareRadarParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CLOUDFLARE RADAR-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse time series data
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "success": true,
    ///   "result": {
    ///     "timestamps": ["2024-01-01T00:00:00Z", "2024-01-02T00:00:00Z"],
    ///     "values": [1234.5, 2345.6]
    ///   }
    /// }
    /// ```
    pub fn parse_timeseries(response: &Value) -> ExchangeResult<RadarTimeSeries> {
        let result = Self::get_result(response)?;

        let timestamps = result
            .get("timestamps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'timestamps' array".to_string()))?
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let values = result
            .get("values")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'values' array".to_string()))?
            .iter()
            .filter_map(|v| v.as_f64())
            .collect();

        Ok(RadarTimeSeries { timestamps, values })
    }

    /// Parse top locations
    pub fn parse_top_locations(response: &Value) -> ExchangeResult<Vec<RadarTopLocation>> {
        let result = Self::get_result(response)?;

        let top = result
            .get("top")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'top' array".to_string()))?;

        top.iter()
            .map(|item| {
                Ok(RadarTopLocation {
                    name: Self::require_str(item, "name")?.to_string(),
                    value: Self::require_f64(item, "value")?,
                    rank: Self::require_u32(item, "rank")?,
                })
            })
            .collect()
    }

    /// Parse top ASes
    pub fn parse_top_ases(response: &Value) -> ExchangeResult<Vec<RadarTopAs>> {
        let result = Self::get_result(response)?;

        let top = result
            .get("top")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'top' array".to_string()))?;

        top.iter()
            .map(|item| {
                Ok(RadarTopAs {
                    asn: Self::require_u64(item, "asn")?,
                    name: Self::require_str(item, "name")?.to_string(),
                    value: Self::require_f64(item, "value")?,
                    rank: Self::require_u32(item, "rank")?,
                })
            })
            .collect()
    }

    /// Parse bot summary
    pub fn parse_bot_summary(response: &Value) -> ExchangeResult<RadarBotSummary> {
        let result = Self::get_result(response)?;

        Ok(RadarBotSummary {
            bot: Self::require_f64(result, "bot")?,
            human: Self::require_f64(result, "human")?,
        })
    }

    /// Parse device summary
    pub fn parse_device_summary(response: &Value) -> ExchangeResult<RadarDeviceSummary> {
        let result = Self::get_result(response)?;

        Ok(RadarDeviceSummary {
            desktop: Self::require_f64(result, "desktop")?,
            mobile: Self::require_f64(result, "mobile")?,
            other: Self::require_f64(result, "other")?,
        })
    }

    /// Parse protocol summary
    pub fn parse_protocol_summary(response: &Value) -> ExchangeResult<RadarProtocolSummary> {
        let result = Self::get_result(response)?;

        Ok(RadarProtocolSummary {
            http: Self::get_f64(result, "http").unwrap_or(0.0),
            https: Self::get_f64(result, "https").unwrap_or(0.0),
            http2: Self::get_f64(result, "http2").unwrap_or(0.0),
            http3: Self::get_f64(result, "http3").unwrap_or(0.0),
        })
    }

    /// Parse OS summary
    pub fn parse_os_summary(response: &Value) -> ExchangeResult<RadarOsSummary> {
        let result = Self::get_result(response)?;

        // OS data is typically a map of OS name to percentage
        let mut os_data = std::collections::HashMap::new();

        if let Some(obj) = result.as_object() {
            for (key, value) in obj.iter() {
                if let Some(percent) = value.as_f64() {
                    os_data.insert(key.clone(), percent);
                }
            }
        }

        Ok(RadarOsSummary { os_data })
    }

    /// Parse browser summary
    pub fn parse_browser_summary(response: &Value) -> ExchangeResult<RadarBrowserSummary> {
        let result = Self::get_result(response)?;

        // Browser data is typically a map of browser name to percentage
        let mut browser_data = std::collections::HashMap::new();

        if let Some(obj) = result.as_object() {
            for (key, value) in obj.iter() {
                if let Some(percent) = value.as_f64() {
                    browser_data.insert(key.clone(), percent);
                }
            }
        }

        Ok(RadarBrowserSummary { browser_data })
    }

    /// Parse attack summary
    pub fn parse_attack_summary(response: &Value) -> ExchangeResult<RadarAttackSummary> {
        let result = Self::get_result(response)?;

        let empty = vec![];
        let top_protocols = Self::get_array(result, "top_protocols")
            .unwrap_or(&empty)
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        let top_locations = Self::get_array(result, "top_locations")
            .unwrap_or(&empty)
            .iter()
            .filter_map(|v| v.as_str().map(|s| s.to_string()))
            .collect();

        Ok(RadarAttackSummary {
            total_attacks: Self::require_u64(result, "total_attacks")?,
            total_bytes: Self::require_u64(result, "total_bytes")?,
            top_protocols,
            top_locations,
        })
    }

    /// Parse top domains
    pub fn parse_top_domains(response: &Value) -> ExchangeResult<Vec<RadarTopDomain>> {
        let result = Self::get_result(response)?;

        let top = result
            .get("top")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'top' array".to_string()))?;

        top.iter()
            .map(|item| {
                let empty = vec![];
                let categories = Self::get_array(item, "categories")
                    .unwrap_or(&empty)
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect();

                Ok(RadarTopDomain {
                    domain: Self::require_str(item, "domain")?.to_string(),
                    rank: Self::require_u32(item, "rank")?,
                    categories,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(success) = response.get("success").and_then(|v| v.as_bool()) {
            if !success {
                let errors = response
                    .get("errors")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|e| e.get("message").and_then(|m| m.as_str()))
                            .collect::<Vec<_>>()
                            .join(", ")
                    })
                    .unwrap_or_else(|| "Unknown error".to_string());

                return Err(ExchangeError::Api {
                    code: 0,
                    message: errors,
                });
            }
        }
        Ok(())
    }

    /// Get result object from response
    fn get_result(response: &Value) -> ExchangeResult<&Value> {
        response
            .get("result")
            .ok_or_else(|| ExchangeError::Parse("Missing 'result' object".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .map(|n| n as u32)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn require_u64(obj: &Value, field: &str) -> ExchangeResult<u64> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_array<'a>(obj: &'a Value, field: &str) -> Option<&'a Vec<Value>> {
        obj.get(field).and_then(|v| v.as_array())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CLOUDFLARE RADAR-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Radar time series data
#[derive(Debug, Clone)]
pub struct RadarTimeSeries {
    pub timestamps: Vec<String>,
    pub values: Vec<f64>,
}

/// Radar top location data
#[derive(Debug, Clone)]
pub struct RadarTopLocation {
    pub name: String,
    pub value: f64,
    pub rank: u32,
}

/// Radar top AS data
#[derive(Debug, Clone)]
pub struct RadarTopAs {
    pub asn: u64,
    pub name: String,
    pub value: f64,
    pub rank: u32,
}

/// Radar bot vs human traffic summary
#[derive(Debug, Clone)]
pub struct RadarBotSummary {
    pub bot: f64,
    pub human: f64,
}

/// Radar device type summary
#[derive(Debug, Clone)]
pub struct RadarDeviceSummary {
    pub desktop: f64,
    pub mobile: f64,
    pub other: f64,
}

/// Radar HTTP protocol summary
#[derive(Debug, Clone)]
pub struct RadarProtocolSummary {
    pub http: f64,
    pub https: f64,
    pub http2: f64,
    pub http3: f64,
}

/// Radar OS summary
#[derive(Debug, Clone)]
pub struct RadarOsSummary {
    pub os_data: std::collections::HashMap<String, f64>,
}

/// Radar browser summary
#[derive(Debug, Clone)]
pub struct RadarBrowserSummary {
    pub browser_data: std::collections::HashMap<String, f64>,
}

/// Radar DDoS attack summary
#[derive(Debug, Clone)]
pub struct RadarAttackSummary {
    pub total_attacks: u64,
    pub total_bytes: u64,
    pub top_protocols: Vec<String>,
    pub top_locations: Vec<String>,
}

/// Radar top domain
#[derive(Debug, Clone)]
pub struct RadarTopDomain {
    pub domain: String,
    pub rank: u32,
    pub categories: Vec<String>,
}
