//! AIS response parsers
//!
//! Parse JSON responses to domain types based on Datalastic AIS API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct AisParser;

impl AisParser {
    // ═══════════════════════════════════════════════════════════════════════
    // AIS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse vessel search results
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": [{
    ///     "uuid": "...",
    ///     "name": "VESSEL NAME",
    ///     "mmsi": 123456789,
    ///     "imo": 9876543,
    ///     "callsign": "CALL",
    ///     "ship_type": "Cargo",
    ///     "flag": "US",
    ///     "length": 200.0,
    ///     "width": 32.0,
    ///     "draught": 10.5
    ///   }]
    /// }
    /// ```
    pub fn parse_vessels(response: &Value) -> ExchangeResult<Vec<AisVessel>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|vessel| {
                Ok(AisVessel {
                    uuid: Self::require_str(vessel, "uuid")?.to_string(),
                    name: Self::require_str(vessel, "name")?.to_string(),
                    mmsi: Self::get_u64(vessel, "mmsi"),
                    imo: Self::get_u64(vessel, "imo"),
                    callsign: Self::get_str(vessel, "callsign").map(|s| s.to_string()),
                    ship_type: Self::get_str(vessel, "ship_type").map(|s| s.to_string()),
                    flag: Self::get_str(vessel, "flag").map(|s| s.to_string()),
                    length: Self::get_f64(vessel, "length"),
                    width: Self::get_f64(vessel, "width"),
                    draught: Self::get_f64(vessel, "draught"),
                    current_port: Self::get_str(vessel, "current_port").map(|s| s.to_string()),
                    last_position_lat: Self::get_f64(vessel, "lat")
                        .or_else(|| Self::get_f64(vessel, "latitude")),
                    last_position_lon: Self::get_f64(vessel, "lon")
                        .or_else(|| Self::get_f64(vessel, "longitude")),
                    last_position_timestamp: Self::get_str(vessel, "timestamp")
                        .or_else(|| Self::get_str(vessel, "last_position_time"))
                        .map(|s| s.to_string()),
                    speed: Self::get_f64(vessel, "speed"),
                    course: Self::get_f64(vessel, "course")
                        .or_else(|| Self::get_f64(vessel, "heading")),
                    status: Self::get_str(vessel, "status")
                        .or_else(|| Self::get_str(vessel, "nav_status"))
                        .map(|s| s.to_string()),
                    destination: Self::get_str(vessel, "destination").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single vessel info
    pub fn parse_vessel_info(response: &Value) -> ExchangeResult<AisVessel> {
        let vessel = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        Ok(AisVessel {
            uuid: Self::require_str(vessel, "uuid")?.to_string(),
            name: Self::require_str(vessel, "name")?.to_string(),
            mmsi: Self::get_u64(vessel, "mmsi"),
            imo: Self::get_u64(vessel, "imo"),
            callsign: Self::get_str(vessel, "callsign").map(|s| s.to_string()),
            ship_type: Self::get_str(vessel, "ship_type").map(|s| s.to_string()),
            flag: Self::get_str(vessel, "flag").map(|s| s.to_string()),
            length: Self::get_f64(vessel, "length"),
            width: Self::get_f64(vessel, "width"),
            draught: Self::get_f64(vessel, "draught"),
            current_port: Self::get_str(vessel, "current_port").map(|s| s.to_string()),
            last_position_lat: Self::get_f64(vessel, "lat")
                .or_else(|| Self::get_f64(vessel, "latitude")),
            last_position_lon: Self::get_f64(vessel, "lon")
                .or_else(|| Self::get_f64(vessel, "longitude")),
            last_position_timestamp: Self::get_str(vessel, "timestamp")
                .or_else(|| Self::get_str(vessel, "last_position_time"))
                .map(|s| s.to_string()),
            speed: Self::get_f64(vessel, "speed"),
            course: Self::get_f64(vessel, "course")
                .or_else(|| Self::get_f64(vessel, "heading")),
            status: Self::get_str(vessel, "status")
                .or_else(|| Self::get_str(vessel, "nav_status"))
                .map(|s| s.to_string()),
            destination: Self::get_str(vessel, "destination").map(|s| s.to_string()),
        })
    }

    /// Parse vessel position history
    pub fn parse_vessel_history(response: &Value) -> ExchangeResult<Vec<AisPosition>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|pos| {
                Ok(AisPosition {
                    latitude: Self::require_f64(pos, "lat")
                        .or_else(|_| Self::require_f64(pos, "latitude"))?,
                    longitude: Self::require_f64(pos, "lon")
                        .or_else(|_| Self::require_f64(pos, "longitude"))?,
                    timestamp: Self::require_str(pos, "timestamp")?.to_string(),
                    speed: Self::get_f64(pos, "speed"),
                    course: Self::get_f64(pos, "course")
                        .or_else(|| Self::get_f64(pos, "heading")),
                    status: Self::get_str(pos, "status")
                        .or_else(|| Self::get_str(pos, "nav_status"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse port search results
    pub fn parse_ports(response: &Value) -> ExchangeResult<Vec<AisPort>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|port| {
                Ok(AisPort {
                    uuid: Self::require_str(port, "uuid")?.to_string(),
                    name: Self::require_str(port, "name")?.to_string(),
                    country: Self::get_str(port, "country").map(|s| s.to_string()),
                    latitude: Self::require_f64(port, "lat")
                        .or_else(|_| Self::require_f64(port, "latitude"))?,
                    longitude: Self::require_f64(port, "lon")
                        .or_else(|_| Self::require_f64(port, "longitude"))?,
                    port_type: Self::get_str(port, "port_type")
                        .or_else(|| Self::get_str(port, "type"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single port info
    pub fn parse_port_info(response: &Value) -> ExchangeResult<AisPort> {
        let port = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        Ok(AisPort {
            uuid: Self::require_str(port, "uuid")?.to_string(),
            name: Self::require_str(port, "name")?.to_string(),
            country: Self::get_str(port, "country").map(|s| s.to_string()),
            latitude: Self::require_f64(port, "lat")
                .or_else(|_| Self::require_f64(port, "latitude"))?,
            longitude: Self::require_f64(port, "lon")
                .or_else(|_| Self::require_f64(port, "longitude"))?,
            port_type: Self::get_str(port, "port_type")
                .or_else(|| Self::get_str(port, "type"))
                .map(|s| s.to_string()),
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
                .or_else(|| error.get("message").and_then(|v| v.as_str()))
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 0,
                message
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

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AIS-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// AIS vessel information
#[derive(Debug, Clone)]
pub struct AisVessel {
    pub uuid: String,
    pub name: String,
    pub mmsi: Option<u64>,
    pub imo: Option<u64>,
    pub callsign: Option<String>,
    pub ship_type: Option<String>,
    pub flag: Option<String>,
    pub length: Option<f64>,
    pub width: Option<f64>,
    pub draught: Option<f64>,
    pub current_port: Option<String>,
    pub last_position_lat: Option<f64>,
    pub last_position_lon: Option<f64>,
    pub last_position_timestamp: Option<String>,
    pub speed: Option<f64>,
    pub course: Option<f64>,
    pub status: Option<String>,
    pub destination: Option<String>,
}

/// AIS port information
#[derive(Debug, Clone)]
pub struct AisPort {
    pub uuid: String,
    pub name: String,
    pub country: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub port_type: Option<String>,
}

/// AIS vessel position
#[derive(Debug, Clone)]
pub struct AisPosition {
    pub latitude: f64,
    pub longitude: f64,
    pub timestamp: String,
    pub speed: Option<f64>,
    pub course: Option<f64>,
    pub status: Option<String>,
}
