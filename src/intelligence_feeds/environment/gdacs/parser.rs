//! GDACS response parsers
//!
//! Parse GeoJSON responses to domain types.
//!
//! GDACS returns GeoJSON FeatureCollection format with disaster event data.

use crate::core::types::{ExchangeError, ExchangeResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub struct GdacsParser;

impl GdacsParser {
    /// Parse GDACS GeoJSON response to event list
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "type": "FeatureCollection",
    ///   "features": [
    ///     {
    ///       "type": "Feature",
    ///       "geometry": { "type": "Point", "coordinates": [lon, lat] },
    ///       "properties": {
    ///         "eventtype": "EQ",
    ///         "eventid": 1522345,
    ///         "episodeid": 1685234,
    ///         "eventname": "",
    ///         "alertlevel": "Orange",
    ///         ...
    ///       }
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_events(data: &Value) -> ExchangeResult<Vec<DisasterEvent>> {
        let features = data["features"]
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing features array".to_string()))?;

        let mut events = Vec::new();

        for feature in features {
            if let Ok(event) = Self::parse_event(feature) {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Parse a single GeoJSON feature to DisasterEvent
    fn parse_event(feature: &Value) -> ExchangeResult<DisasterEvent> {
        let props = &feature["properties"];
        let geometry = &feature["geometry"];

        // Extract coordinates
        let coordinates = geometry["coordinates"]
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Missing coordinates".to_string()))?;

        let lon = coordinates
            .first()
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid longitude".to_string()))?;

        let lat = coordinates
            .get(1)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| ExchangeError::Parse("Invalid latitude".to_string()))?;

        // Parse event type
        let event_type_str = props["eventtype"]
            .as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing eventtype".to_string()))?;

        let event_type = DisasterType::from_code(event_type_str)
            .ok_or_else(|| ExchangeError::Parse(format!("Unknown event type: {}", event_type_str)))?;

        // Parse alert level
        let alert_level_str = props["alertlevel"]
            .as_str()
            .ok_or_else(|| ExchangeError::Parse("Missing alertlevel".to_string()))?;

        let alert_level = AlertLevel::parse(alert_level_str)
            .ok_or_else(|| ExchangeError::Parse(format!("Unknown alert level: {}", alert_level_str)))?;

        // Extract severity data
        let severity_data = &props["severitydata"];
        let severity_value = severity_data["severity"].as_f64();
        let severity_text = severity_data["severitytext"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let severity_unit = severity_data["severityunit"]
            .as_str()
            .unwrap_or("")
            .to_string();

        // Extract URLs
        let url_obj = &props["url"];
        let report_url = url_obj["report"].as_str().unwrap_or("").to_string();
        let _geometry_url = url_obj["geometry"].as_str().unwrap_or("").to_string();
        let details_url = url_obj["details"].as_str().unwrap_or("").to_string();

        let url = if !report_url.is_empty() {
            Some(report_url)
        } else if !details_url.is_empty() {
            Some(details_url)
        } else {
            None
        };

        Ok(DisasterEvent {
            event_id: props["eventid"]
                .as_i64()
                .ok_or_else(|| ExchangeError::Parse("Missing eventid".to_string()))?,
            episode_id: props["episodeid"].as_i64(),
            event_type,
            name: props["eventname"].as_str().unwrap_or("").to_string(),
            description: props["htmldescription"].as_str().unwrap_or("").to_string(),
            alert_level,
            alert_score: props["alertscore"].as_f64(),
            country: props["country"]
                .as_str()
                .ok_or_else(|| ExchangeError::Parse("Missing country".to_string()))?
                .to_string(),
            iso3: props["iso3"].as_str().map(|s| s.to_string()),
            lat,
            lon,
            from_date: props["fromdate"].as_str().unwrap_or("").to_string(),
            to_date: props["todate"].as_str().unwrap_or("").to_string(),
            date_modified: props["datemodified"].as_str().map(|s| s.to_string()),
            severity_value,
            severity_text,
            severity_unit,
            url,
            is_current: props["iscurrent"].as_bool().unwrap_or(false),
        })
    }
}

// =============================================================================
// GDACS-SPECIFIC TYPES
// =============================================================================

/// Disaster event from GDACS
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterEvent {
    /// Unique event identifier
    pub event_id: i64,
    /// Episode ID (increments with updates)
    pub episode_id: Option<i64>,
    /// Type of disaster
    pub event_type: DisasterType,
    /// Event name (for named storms, volcanoes, etc.)
    pub name: String,
    /// HTML description
    pub description: String,
    /// Alert level
    pub alert_level: AlertLevel,
    /// Numeric alert score (0.0-3.0)
    pub alert_score: Option<f64>,
    /// Affected country/countries
    pub country: String,
    /// ISO 3166-1 alpha-3 country code
    pub iso3: Option<String>,
    /// Latitude
    pub lat: f64,
    /// Longitude
    pub lon: f64,
    /// Event start date (ISO 8601)
    pub from_date: String,
    /// Event end/update date (ISO 8601)
    pub to_date: String,
    /// Last modification date (ISO 8601)
    pub date_modified: Option<String>,
    /// Severity numeric value (magnitude, wind speed, area, etc.)
    pub severity_value: Option<f64>,
    /// Human-readable severity description
    pub severity_text: String,
    /// Unit of severity measurement
    pub severity_unit: String,
    /// URL to event details or report
    pub url: Option<String>,
    /// Whether event is current/active
    pub is_current: bool,
}

/// Type of disaster
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DisasterType {
    /// Earthquake
    #[serde(rename = "EQ")]
    Earthquake,
    /// Tropical Cyclone (Hurricane, Typhoon)
    #[serde(rename = "TC")]
    TropicalCyclone,
    /// Flood
    #[serde(rename = "FL")]
    Flood,
    /// Volcano
    #[serde(rename = "VO")]
    Volcano,
    /// Wildfire
    #[serde(rename = "WF")]
    Wildfire,
    /// Drought
    #[serde(rename = "DR")]
    Drought,
    /// Tsunami
    #[serde(rename = "TS")]
    Tsunami,
}

impl DisasterType {
    /// Get disaster type from two-letter code
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "EQ" => Some(Self::Earthquake),
            "TC" => Some(Self::TropicalCyclone),
            "FL" => Some(Self::Flood),
            "VO" => Some(Self::Volcano),
            "WF" => Some(Self::Wildfire),
            "DR" => Some(Self::Drought),
            "TS" => Some(Self::Tsunami),
            _ => None,
        }
    }

    /// Get two-letter code for disaster type
    pub fn code(&self) -> &'static str {
        match self {
            Self::Earthquake => "EQ",
            Self::TropicalCyclone => "TC",
            Self::Flood => "FL",
            Self::Volcano => "VO",
            Self::Wildfire => "WF",
            Self::Drought => "DR",
            Self::Tsunami => "TS",
        }
    }
}

/// GDACS alert level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlertLevel {
    /// Green - Minor impact, local response sufficient
    Green,
    /// Orange - Moderate impact, national response needed
    Orange,
    /// Red - Major impact, international response likely
    Red,
}

impl AlertLevel {
    /// Parse alert level from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "green" => Some(Self::Green),
            "orange" => Some(Self::Orange),
            "red" => Some(Self::Red),
            _ => None,
        }
    }

    /// Get string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Orange => "orange",
            Self::Red => "red",
        }
    }
}
