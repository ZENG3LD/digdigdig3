//! ADS-B Exchange response parsers
//!
//! Parse JSON responses to domain types based on ADS-B Exchange API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct AdsbExchangeParser;

impl AdsbExchangeParser {
    // ═══════════════════════════════════════════════════════════════════════
    // MAIN PARSER
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse ADS-B Exchange response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "ac": [
    ///     {
    ///       "hex": "a1b2c3",
    ///       "flight": "UAL123  ",
    ///       "lat": 37.7749,
    ///       "lon": -122.4194,
    ///       "alt_baro": 35000,
    ///       "alt_geom": 35500,
    ///       "gs": 450.5,
    ///       "track": 275.3,
    ///       "baro_rate": 0,
    ///       "squawk": "1234",
    ///       "category": "A3",
    ///       "nav_altitude_mcp": 36000,
    ///       "nav_heading": 270,
    ///       "t": "B738",
    ///       "r": "N12345",
    ///       "dbFlags": 0,
    ///       "emergency": "none",
    ///       "mil": false,
    ///       "seen": 0.5,
    ///       "rssi": -25.3
    ///     }
    ///   ],
    ///   "total": 1,
    ///   "now": 1234567890,
    ///   "msg": "No error"
    /// }
    /// ```
    pub fn parse_response(response: &Value) -> ExchangeResult<AdsbResponse> {
        // Check for error message
        Self::check_error(response)?;

        let ac_array = response
            .get("ac")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'ac' array".to_string()))?;

        let aircraft: Vec<AdsbAircraft> = ac_array
            .iter()
            .map(|ac| {
                serde_json::from_value(ac.clone()).map_err(|e| {
                    ExchangeError::Parse(format!("Failed to parse aircraft: {}", e))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let total = response
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(aircraft.len() as u64);

        let now = response
            .get("now")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let msg = response
            .get("msg")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        Ok(AdsbResponse {
            ac: aircraft,
            total,
            now,
            msg,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check for error message
        if let Some(msg) = response.get("msg").and_then(|v| v.as_str()) {
            if msg.to_lowercase().contains("error") {
                return Err(ExchangeError::Api {
                    code: 400,
                    message: msg.to_string(),
                });
            }
        }

        // Check for error field
        if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: 400,
                message: error.to_string(),
            });
        }

        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// ADS-B EXCHANGE-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// ADS-B Exchange aircraft data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsbAircraft {
    /// ICAO hex code (required)
    pub hex: String,

    /// Flight callsign (may have trailing spaces)
    #[serde(default)]
    pub flight: Option<String>,

    /// Latitude (decimal degrees)
    #[serde(default)]
    pub lat: Option<f64>,

    /// Longitude (decimal degrees)
    #[serde(default)]
    pub lon: Option<f64>,

    /// Barometric altitude (feet)
    #[serde(default)]
    pub alt_baro: Option<f64>,

    /// Geometric altitude (feet)
    #[serde(default)]
    pub alt_geom: Option<f64>,

    /// Ground speed (knots)
    #[serde(default)]
    pub gs: Option<f64>,

    /// Track/heading (degrees)
    #[serde(default)]
    pub track: Option<f64>,

    /// Barometric rate of climb/descent (feet/min)
    #[serde(default)]
    pub baro_rate: Option<f64>,

    /// Squawk code (e.g., "7700" for emergency)
    #[serde(default)]
    pub squawk: Option<String>,

    /// Aircraft category (e.g., "A3" for large aircraft)
    #[serde(default)]
    pub category: Option<String>,

    /// MCP/FCU selected altitude (feet)
    #[serde(default)]
    pub nav_altitude_mcp: Option<f64>,

    /// MCP/FCU selected heading (degrees)
    #[serde(default)]
    pub nav_heading: Option<f64>,

    /// Aircraft type (e.g., "B738", "F16")
    #[serde(default)]
    pub t: Option<String>,

    /// Registration (e.g., "N12345")
    #[serde(default)]
    pub r: Option<String>,

    /// Database flags (bit 0 = military)
    #[serde(default)]
    #[serde(rename = "dbFlags")]
    pub db_flags: Option<u32>,

    /// Emergency status (e.g., "none", "general", "lifeguard", "minfuel", "nordo", "unlawful", "downed")
    #[serde(default)]
    pub emergency: Option<String>,

    /// Military flag (explicitly set if military aircraft)
    #[serde(default)]
    pub mil: Option<bool>,

    /// Seconds since last message
    #[serde(default)]
    pub seen: Option<f64>,

    /// Signal strength (RSSI)
    #[serde(default)]
    pub rssi: Option<f64>,
}

impl AdsbAircraft {
    /// Check if this is a military aircraft
    pub fn is_military(&self) -> bool {
        // Check explicit mil flag
        if let Some(mil) = self.mil {
            return mil;
        }

        // Check dbFlags bit 0
        if let Some(flags) = self.db_flags {
            return (flags & 1) == 1;
        }

        false
    }

    /// Get trimmed callsign (removes trailing spaces)
    pub fn callsign(&self) -> Option<&str> {
        self.flight.as_ref().map(|s| s.trim())
    }

    /// Get aircraft type
    pub fn aircraft_type(&self) -> Option<&str> {
        self.t.as_deref()
    }

    /// Get registration
    pub fn registration(&self) -> Option<&str> {
        self.r.as_deref()
    }

    /// Check if aircraft is in emergency
    pub fn is_emergency(&self) -> bool {
        // Check squawk code
        if let Some(squawk) = &self.squawk {
            if squawk == "7700" || squawk == "7500" || squawk == "7600" {
                return true;
            }
        }

        // Check emergency field
        if let Some(emergency) = &self.emergency {
            return emergency != "none" && !emergency.is_empty();
        }

        false
    }

    /// Check if aircraft is hijacked (squawk 7500)
    pub fn is_hijacked(&self) -> bool {
        self.squawk.as_deref() == Some("7500")
    }

    /// Check if aircraft has radio failure (squawk 7600)
    pub fn has_radio_failure(&self) -> bool {
        self.squawk.as_deref() == Some("7600")
    }
}

/// ADS-B Exchange response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdsbResponse {
    /// Array of aircraft
    pub ac: Vec<AdsbAircraft>,

    /// Total number of aircraft
    pub total: u64,

    /// Current Unix timestamp
    pub now: u64,

    /// Message (e.g., "No error")
    pub msg: Option<String>,
}
