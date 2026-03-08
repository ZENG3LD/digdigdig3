//! AISStream.io response parsers
//!
//! Parse JSON messages to domain types based on AISStream.io message formats.
//!
//! AISStream provides real-time vessel tracking data via WebSocket.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct AisStreamParser;

impl AisStreamParser {
    // ═══════════════════════════════════════════════════════════════════════
    // AISSTREAM-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse AIS message from WebSocket
    ///
    /// Example message:
    /// ```json
    /// {
    ///   "MessageType": "PositionReport",
    ///   "MetaData": {
    ///     "MMSI": 235069602,
    ///     "MMSI_String": "235069602",
    ///     "ShipName": "EXAMPLE VESSEL",
    ///     "latitude": 51.234,
    ///     "longitude": -0.567,
    ///     "time_utc": "2024-01-15T12:34:56Z"
    ///   },
    ///   "Message": {
    ///     "PositionReport": {
    ///       "Latitude": 51.234,
    ///       "Longitude": -0.567,
    ///       "Sog": 12.3,
    ///       "Cog": 45.6,
    ///       "TrueHeading": 47,
    ///       "Timestamp": "2024-01-15T12:34:56Z",
    ///       "NavigationalStatus": 0
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_message(response: &Value) -> ExchangeResult<AisMessage> {
        let message_type = Self::require_str(response, "MessageType")?.to_string();

        let metadata_obj = response
            .get("MetaData")
            .ok_or_else(|| ExchangeError::Parse("Missing 'MetaData' object".to_string()))?;

        let metadata = AisMetadata {
            mmsi: Self::require_u64(metadata_obj, "MMSI")?,
            mmsi_string: Self::get_str(metadata_obj, "MMSI_String")
                .unwrap_or("")
                .to_string(),
            ship_name: Self::get_str(metadata_obj, "ShipName").map(|s| s.to_string()),
            latitude: Self::require_f64(metadata_obj, "latitude")?,
            longitude: Self::require_f64(metadata_obj, "longitude")?,
            time_utc: Self::require_str(metadata_obj, "time_utc")?.to_string(),
        };

        let message_obj = response
            .get("Message")
            .ok_or_else(|| ExchangeError::Parse("Missing 'Message' object".to_string()))?;

        // Parse position report if present
        let position = if let Some(pos_obj) = message_obj.get("PositionReport") {
            Some(AisPosition {
                mmsi: metadata.mmsi,
                latitude: Self::require_f64(pos_obj, "Latitude")?,
                longitude: Self::require_f64(pos_obj, "Longitude")?,
                speed: Self::get_f64(pos_obj, "Sog").unwrap_or(0.0),
                course: Self::get_f64(pos_obj, "Cog").unwrap_or(0.0),
                heading: Self::get_u32(pos_obj, "TrueHeading").map(|h| h as f64),
                timestamp: Self::require_str(pos_obj, "Timestamp")?.to_string(),
                navigation_status: Self::get_u32(pos_obj, "NavigationalStatus")
                    .map(|s| format!("{}", s)),
            })
        } else {
            None
        };

        // Parse static data if present
        let static_data = if let Some(static_obj) = message_obj.get("ShipStaticData") {
            Some(AisVesselStatic {
                mmsi: metadata.mmsi,
                imo: Self::get_u64(static_obj, "ImoNumber"),
                name: Self::get_str(static_obj, "Name")
                    .unwrap_or("")
                    .to_string(),
                ship_type: Self::get_u32(static_obj, "Type").unwrap_or(0),
                ship_type_text: Self::get_str(static_obj, "TypeAndCargo").map(|s| s.to_string()),
                length: Self::get_f64(static_obj, "Dimension").map(|_| {
                    let a = Self::get_f64(static_obj, "A").unwrap_or(0.0);
                    let b = Self::get_f64(static_obj, "B").unwrap_or(0.0);
                    a + b
                }),
                width: Self::get_f64(static_obj, "Dimension").map(|_| {
                    let c = Self::get_f64(static_obj, "C").unwrap_or(0.0);
                    let d = Self::get_f64(static_obj, "D").unwrap_or(0.0);
                    c + d
                }),
                draught: Self::get_f64(static_obj, "MaximumStaticDraught"),
                destination: Self::get_str(static_obj, "Destination").map(|s| s.to_string()),
                eta: Self::get_str(static_obj, "Eta").map(|s| s.to_string()),
                flag: Self::get_str(static_obj, "Flag").map(|s| s.to_string()),
            })
        } else {
            None
        };

        Ok(AisMessage {
            message_type,
            metadata,
            position,
            static_data,
        })
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error_msg) = response.get("error").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: 0,
                message: error_msg.to_string(),
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

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AISSTREAM-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// AIS vessel position data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AisPosition {
    pub mmsi: u64,
    pub latitude: f64,
    pub longitude: f64,
    pub speed: f64, // Speed over ground (knots)
    pub course: f64, // Course over ground (degrees)
    pub heading: Option<f64>, // True heading (degrees)
    pub timestamp: String,
    pub navigation_status: Option<String>,
}

/// AIS vessel static data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AisVesselStatic {
    pub mmsi: u64,
    pub imo: Option<u64>,
    pub name: String,
    pub ship_type: u32,
    pub ship_type_text: Option<String>,
    pub length: Option<f64>, // meters
    pub width: Option<f64>,  // meters
    pub draught: Option<f64>, // meters
    pub destination: Option<String>,
    pub eta: Option<String>,
    pub flag: Option<String>,
}

/// AIS message metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AisMetadata {
    pub mmsi: u64,
    pub mmsi_string: String,
    pub ship_name: Option<String>,
    pub latitude: f64,
    pub longitude: f64,
    pub time_utc: String,
}

/// Complete AIS message
#[derive(Debug, Clone)]
pub struct AisMessage {
    pub message_type: String,
    pub metadata: AisMetadata,
    pub position: Option<AisPosition>,
    pub static_data: Option<AisVesselStatic>,
}

/// Bounding box for geographic filtering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BoundingBox {
    #[serde(rename = "TopLeftLatitude")]
    pub lat_min: f64,
    #[serde(rename = "TopLeftLongitude")]
    pub lon_min: f64,
    #[serde(rename = "BottomRightLatitude")]
    pub lat_max: f64,
    #[serde(rename = "BottomRightLongitude")]
    pub lon_max: f64,
}

impl BoundingBox {
    /// Create new bounding box
    pub fn new(lat_min: f64, lon_min: f64, lat_max: f64, lon_max: f64) -> Self {
        Self {
            lat_min,
            lon_min,
            lat_max,
            lon_max,
        }
    }
}

/// WebSocket subscription message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionMessage {
    #[serde(rename = "APIKey")]
    pub api_key: String,

    #[serde(rename = "BoundingBoxes", skip_serializing_if = "Option::is_none")]
    pub bounding_boxes: Option<Vec<Vec<BoundingBox>>>,

    #[serde(rename = "FiltersShipMMSI", skip_serializing_if = "Option::is_none")]
    pub mmsi_filter: Option<Vec<String>>,

    #[serde(rename = "FilterMessageTypes", skip_serializing_if = "Option::is_none")]
    pub message_types: Option<Vec<String>>,
}
