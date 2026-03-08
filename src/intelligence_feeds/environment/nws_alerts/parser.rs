//! NWS Weather Alerts response parsers
//!
//! Parse GeoJSON responses to domain types based on CAP v1.2 format.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct NwsAlertsParser;

impl NwsAlertsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // NWS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse alerts from GeoJSON FeatureCollection response
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "type": "FeatureCollection",
    ///   "features": [
    ///     {
    ///       "id": "https://api.weather.gov/alerts/urn:oid:...",
    ///       "type": "Feature",
    ///       "properties": {
    ///         "id": "urn:oid:...",
    ///         "event": "Winter Weather Advisory",
    ///         "severity": "Moderate",
    ///         ...
    ///       }
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_alerts(data: &Value) -> ExchangeResult<Vec<WeatherAlert>> {
        let features = data
            .get("features")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'features' array".to_string()))?;

        features
            .iter()
            .map(Self::parse_alert)
            .collect()
    }

    /// Parse a single alert from a GeoJSON Feature
    pub fn parse_alert(feature: &Value) -> ExchangeResult<WeatherAlert> {
        let props = feature
            .get("properties")
            .ok_or_else(|| ExchangeError::Parse("Missing 'properties' object".to_string()))?;

        let id = Self::require_str(props, "id")?.to_string();
        let event = Self::require_str(props, "event")?.to_string();
        let headline = Self::get_str(props, "headline").map(|s| s.to_string());
        let description = Self::get_str(props, "description").map(|s| s.to_string());
        let instruction = Self::get_str(props, "instruction").map(|s| s.to_string());
        let area_desc = Self::get_str(props, "areaDesc").map(|s| s.to_string());
        let sender_name = Self::get_str(props, "senderName").map(|s| s.to_string());
        let sent = Self::get_str(props, "sent").map(|s| s.to_string());
        let effective = Self::get_str(props, "effective").map(|s| s.to_string());
        let onset = Self::get_str(props, "onset").map(|s| s.to_string());
        let expires = Self::get_str(props, "expires").map(|s| s.to_string());
        let ends = Self::get_str(props, "ends").map(|s| s.to_string());
        let status = Self::get_str(props, "status").map(|s| s.to_string());
        let message_type = Self::get_str(props, "messageType").map(|s| s.to_string());
        let category = Self::get_str(props, "category").map(|s| s.to_string());
        let response = Self::get_str(props, "response").map(|s| s.to_string());

        // Parse severity enum
        let severity = Self::get_str(props, "severity")
            .map(|s| match s {
                "Extreme" => Severity::Extreme,
                "Severe" => Severity::Severe,
                "Moderate" => Severity::Moderate,
                "Minor" => Severity::Minor,
                _ => Severity::Unknown,
            })
            .unwrap_or(Severity::Unknown);

        // Parse certainty enum
        let certainty = Self::get_str(props, "certainty")
            .map(|s| match s {
                "Observed" => Certainty::Observed,
                "Likely" => Certainty::Likely,
                "Possible" => Certainty::Possible,
                "Unlikely" => Certainty::Unlikely,
                _ => Certainty::Unknown,
            })
            .unwrap_or(Certainty::Unknown);

        // Parse urgency enum
        let urgency = Self::get_str(props, "urgency")
            .map(|s| match s {
                "Immediate" => Urgency::Immediate,
                "Expected" => Urgency::Expected,
                "Future" => Urgency::Future,
                "Past" => Urgency::Past,
                _ => Urgency::Unknown,
            })
            .unwrap_or(Urgency::Unknown);

        Ok(WeatherAlert {
            id,
            event,
            severity,
            certainty,
            urgency,
            headline,
            description,
            instruction,
            area_desc,
            onset,
            expires,
            ends,
            sent,
            effective,
            sender_name,
            status,
            message_type,
            category,
            response,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Extract required string field
    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    /// Extract optional string field
    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

// =============================================================================
// NWS ALERTS-SPECIFIC TYPES
// =============================================================================

/// Weather alert severity level
#[derive(Debug, Clone, PartialEq)]
pub enum Severity {
    /// Extraordinary threat to life or property
    Extreme,
    /// Significant threat to life or property
    Severe,
    /// Possible threat to life or property
    Moderate,
    /// Minimal threat to life or property
    Minor,
    /// Unknown severity
    Unknown,
}

/// Weather alert certainty level
#[derive(Debug, Clone, PartialEq)]
pub enum Certainty {
    /// Determined to have occurred or to be ongoing
    Observed,
    /// Likely to occur (>50% probability)
    Likely,
    /// Possible but not likely (<50% probability)
    Possible,
    /// Not expected to occur
    Unlikely,
    /// Unknown certainty
    Unknown,
}

/// Weather alert urgency level
#[derive(Debug, Clone, PartialEq)]
pub enum Urgency {
    /// Responsive action should be taken immediately
    Immediate,
    /// Responsive action should be taken soon (within next hour)
    Expected,
    /// Responsive action should be taken in the near future
    Future,
    /// Responsive action is no longer required
    Past,
    /// Unknown urgency
    Unknown,
}

/// NWS Weather Alert
#[derive(Debug, Clone)]
pub struct WeatherAlert {
    /// Alert identifier (URN format)
    pub id: String,
    /// Event type (e.g., "Tornado Warning", "Winter Weather Advisory")
    pub event: String,
    /// Severity level
    pub severity: Severity,
    /// Certainty level
    pub certainty: Certainty,
    /// Urgency level
    pub urgency: Urgency,
    /// Alert headline
    pub headline: Option<String>,
    /// Detailed description
    pub description: Option<String>,
    /// Safety instructions
    pub instruction: Option<String>,
    /// Affected area description
    pub area_desc: Option<String>,
    /// Expected onset time (ISO 8601)
    pub onset: Option<String>,
    /// Expiration time (ISO 8601)
    pub expires: Option<String>,
    /// End time (ISO 8601)
    pub ends: Option<String>,
    /// Time alert was sent (ISO 8601)
    pub sent: Option<String>,
    /// Effective time (ISO 8601)
    pub effective: Option<String>,
    /// Issuing NWS office name
    pub sender_name: Option<String>,
    /// Alert status (e.g., "Actual", "Test")
    pub status: Option<String>,
    /// Message type (e.g., "Alert", "Update", "Cancel")
    pub message_type: Option<String>,
    /// Alert category (e.g., "Met" for meteorological)
    pub category: Option<String>,
    /// Recommended response type (e.g., "Prepare", "Shelter", "Execute")
    pub response: Option<String>,
}
