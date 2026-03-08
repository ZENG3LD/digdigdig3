//! IMF PortWatch response parsers
//!
//! Parse JSON/GeoJSON responses to domain types based on IMF PortWatch API response formats.

use serde_json::Value;
use std::collections::HashMap;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct ImfPortWatchParser;

impl ImfPortWatchParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CHOKEPOINT PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse chokepoints list
    ///
    /// May be in GeoJSON FeatureCollection format or plain JSON array
    pub fn parse_chokepoints(response: &Value) -> ExchangeResult<Vec<PortWatchChokepoint>> {
        // Try GeoJSON format first
        if let Some(features) = response.get("features").and_then(|v| v.as_array()) {
            return features
                .iter()
                .map(Self::parse_chokepoint_feature)
                .collect();
        }

        // Try plain array format
        if let Some(array) = response.as_array() {
            return array
                .iter()
                .map(Self::parse_chokepoint_object)
                .collect();
        }

        // Try data field
        if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            return data
                .iter()
                .map(Self::parse_chokepoint_object)
                .collect();
        }

        Err(ExchangeError::Parse(
            "Invalid chokepoints response format".to_string(),
        ))
    }

    fn parse_chokepoint_feature(feature: &Value) -> ExchangeResult<PortWatchChokepoint> {
        let properties = feature
            .get("properties")
            .ok_or_else(|| ExchangeError::Parse("Missing properties in feature".to_string()))?;

        let geometry = feature.get("geometry");
        let coordinates = geometry
            .and_then(|g| g.get("coordinates"))
            .and_then(|c| c.as_array());

        let (longitude, latitude) = if let Some(coords) = coordinates {
            (
                coords.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
                coords.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            )
        } else {
            (0.0, 0.0)
        };

        Ok(PortWatchChokepoint {
            id: Self::require_str(properties, "id")?.to_string(),
            name: Self::require_str(properties, "name")?.to_string(),
            latitude,
            longitude,
            region: Self::get_str(properties, "region")
                .unwrap_or("Unknown")
                .to_string(),
            description: Self::get_str(properties, "description")
                .map(|s| s.to_string()),
            avg_daily_vessels: Self::get_f64(properties, "avg_daily_vessels")
                .or_else(|| Self::get_f64(properties, "avgDailyVessels")),
            trade_value_pct: Self::get_f64(properties, "trade_value_pct")
                .or_else(|| Self::get_f64(properties, "tradeValuePct")),
        })
    }

    fn parse_chokepoint_object(obj: &Value) -> ExchangeResult<PortWatchChokepoint> {
        Ok(PortWatchChokepoint {
            id: Self::require_str(obj, "id")?.to_string(),
            name: Self::require_str(obj, "name")?.to_string(),
            latitude: Self::get_f64(obj, "latitude")
                .or_else(|| Self::get_f64(obj, "lat"))
                .unwrap_or(0.0),
            longitude: Self::get_f64(obj, "longitude")
                .or_else(|| Self::get_f64(obj, "lon"))
                .or_else(|| Self::get_f64(obj, "lng"))
                .unwrap_or(0.0),
            region: Self::get_str(obj, "region")
                .unwrap_or("Unknown")
                .to_string(),
            description: Self::get_str(obj, "description").map(|s| s.to_string()),
            avg_daily_vessels: Self::get_f64(obj, "avg_daily_vessels")
                .or_else(|| Self::get_f64(obj, "avgDailyVessels")),
            trade_value_pct: Self::get_f64(obj, "trade_value_pct")
                .or_else(|| Self::get_f64(obj, "tradeValuePct")),
        })
    }

    /// Parse chokepoint statistics
    pub fn parse_chokepoint_stats(response: &Value) -> ExchangeResult<PortWatchTrafficStats> {
        // Try to extract from data field or root
        let data = response.get("data").unwrap_or(response);

        Ok(PortWatchTrafficStats {
            chokepoint_id: Self::get_str(data, "chokepoint_id")
                .or_else(|| Self::get_str(data, "chokepointId"))
                .or_else(|| Self::get_str(data, "id"))
                .unwrap_or("unknown")
                .to_string(),
            period: Self::get_str(data, "period")
                .unwrap_or("unknown")
                .to_string(),
            vessel_count: Self::get_u64(data, "vessel_count")
                .or_else(|| Self::get_u64(data, "vesselCount"))
                .unwrap_or(0),
            trade_value_usd: Self::get_f64(data, "trade_value_usd")
                .or_else(|| Self::get_f64(data, "tradeValueUsd")),
            vessel_types: Self::parse_vessel_types(data),
        })
    }

    fn parse_vessel_types(obj: &Value) -> HashMap<String, u64> {
        let mut vessel_types = HashMap::new();

        if let Some(types_obj) = obj.get("vessel_types").or_else(|| obj.get("vesselTypes")) {
            if let Some(map) = types_obj.as_object() {
                for (key, value) in map.iter() {
                    if let Some(count) = value.as_u64() {
                        vessel_types.insert(key.clone(), count);
                    } else if let Some(count_str) = value.as_str() {
                        if let Ok(count) = count_str.parse::<u64>() {
                            vessel_types.insert(key.clone(), count);
                        }
                    }
                }
            }
        }

        vessel_types
    }

    // ═══════════════════════════════════════════════════════════════════════
    // PORT PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse ports list
    pub fn parse_ports(response: &Value) -> ExchangeResult<Vec<PortWatchPort>> {
        // Try GeoJSON format
        if let Some(features) = response.get("features").and_then(|v| v.as_array()) {
            return features
                .iter()
                .map(Self::parse_port_feature)
                .collect();
        }

        // Try plain array
        if let Some(array) = response.as_array() {
            return array
                .iter()
                .map(Self::parse_port_object)
                .collect();
        }

        // Try data field
        if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            return data
                .iter()
                .map(Self::parse_port_object)
                .collect();
        }

        Err(ExchangeError::Parse(
            "Invalid ports response format".to_string(),
        ))
    }

    fn parse_port_feature(feature: &Value) -> ExchangeResult<PortWatchPort> {
        let properties = feature
            .get("properties")
            .ok_or_else(|| ExchangeError::Parse("Missing properties in feature".to_string()))?;

        let geometry = feature.get("geometry");
        let coordinates = geometry
            .and_then(|g| g.get("coordinates"))
            .and_then(|c| c.as_array());

        let (longitude, latitude) = if let Some(coords) = coordinates {
            (
                coords.first().and_then(|v| v.as_f64()).unwrap_or(0.0),
                coords.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0),
            )
        } else {
            (0.0, 0.0)
        };

        Ok(PortWatchPort {
            id: Self::require_str(properties, "id")?.to_string(),
            name: Self::require_str(properties, "name")?.to_string(),
            country: Self::get_str(properties, "country")
                .unwrap_or("Unknown")
                .to_string(),
            latitude,
            longitude,
            port_type: Self::get_str(properties, "port_type")
                .or_else(|| Self::get_str(properties, "portType"))
                .unwrap_or("Unknown")
                .to_string(),
            throughput: Self::get_f64(properties, "throughput"),
        })
    }

    fn parse_port_object(obj: &Value) -> ExchangeResult<PortWatchPort> {
        Ok(PortWatchPort {
            id: Self::require_str(obj, "id")?.to_string(),
            name: Self::require_str(obj, "name")?.to_string(),
            country: Self::get_str(obj, "country")
                .unwrap_or("Unknown")
                .to_string(),
            latitude: Self::get_f64(obj, "latitude")
                .or_else(|| Self::get_f64(obj, "lat"))
                .unwrap_or(0.0),
            longitude: Self::get_f64(obj, "longitude")
                .or_else(|| Self::get_f64(obj, "lon"))
                .or_else(|| Self::get_f64(obj, "lng"))
                .unwrap_or(0.0),
            port_type: Self::get_str(obj, "port_type")
                .or_else(|| Self::get_str(obj, "portType"))
                .unwrap_or("Unknown")
                .to_string(),
            throughput: Self::get_f64(obj, "throughput"),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DISRUPTION PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse disruptions list
    pub fn parse_disruptions(response: &Value) -> ExchangeResult<Vec<PortWatchDisruption>> {
        let array = if let Some(arr) = response.as_array() {
            arr
        } else if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            data
        } else {
            return Err(ExchangeError::Parse(
                "Invalid disruptions response format".to_string(),
            ));
        };

        array
            .iter()
            .map(|item| {
                Ok(PortWatchDisruption {
                    id: Self::require_str(item, "id")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                    description: Self::get_str(item, "description")
                        .unwrap_or("")
                        .to_string(),
                    start_date: Self::require_str(item, "start_date")?.to_string(),
                    end_date: Self::get_str(item, "end_date").map(|s| s.to_string()),
                    severity: Self::get_str(item, "severity")
                        .unwrap_or("Unknown")
                        .to_string(),
                    affected_chokepoints: Self::parse_string_array(
                        item,
                        "affected_chokepoints",
                    ),
                })
            })
            .collect()
    }

    fn parse_string_array(obj: &Value, field: &str) -> Vec<String> {
        obj.get(field)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = if let Some(msg) = error.as_str() {
                msg.to_string()
            } else if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
                msg.to_string()
            } else {
                "Unknown error".to_string()
            };

            let code = error
                .get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            return Err(ExchangeError::Api { code, message });
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

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| {
            v.as_f64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| {
            v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// IMF PORTWATCH-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// IMF PortWatch chokepoint
#[derive(Debug, Clone)]
pub struct PortWatchChokepoint {
    pub id: String,
    pub name: String,
    pub latitude: f64,
    pub longitude: f64,
    pub region: String,
    pub description: Option<String>,
    pub avg_daily_vessels: Option<f64>,
    pub trade_value_pct: Option<f64>,
}

/// IMF PortWatch port
#[derive(Debug, Clone)]
pub struct PortWatchPort {
    pub id: String,
    pub name: String,
    pub country: String,
    pub latitude: f64,
    pub longitude: f64,
    pub port_type: String,
    pub throughput: Option<f64>,
}

/// IMF PortWatch traffic statistics
#[derive(Debug, Clone)]
pub struct PortWatchTrafficStats {
    pub chokepoint_id: String,
    pub period: String,
    pub vessel_count: u64,
    pub trade_value_usd: Option<f64>,
    pub vessel_types: HashMap<String, u64>,
}

/// IMF PortWatch disruption
#[derive(Debug, Clone)]
pub struct PortWatchDisruption {
    pub id: String,
    pub name: String,
    pub description: String,
    pub start_date: String,
    pub end_date: Option<String>,
    pub severity: String,
    pub affected_chokepoints: Vec<String>,
}
