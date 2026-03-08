//! NASA EONET response parsers
//!
//! Parse JSON responses to domain types.

use crate::core::types::{ExchangeError, ExchangeResult};
use serde_json::Value;

pub struct NasaEonetParser;

impl NasaEonetParser {
    /// Parse EONET events response
    ///
    /// Example response structure from /events endpoint
    pub fn parse_events(data: &Value) -> ExchangeResult<Vec<NaturalEvent>> {
        let events_array = data
            .get("events")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'events' field".to_string()))?;

        let mut events = Vec::new();

        for event_val in events_array {
            if let Ok(event) = Self::parse_event(event_val) {
                events.push(event);
            }
        }

        Ok(events)
    }

    /// Parse single event object
    fn parse_event(event: &Value) -> ExchangeResult<NaturalEvent> {
        let id = event
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing event id".to_string()))?
            .to_string();

        let title = event
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing event title".to_string()))?
            .to_string();

        let description = event
            .get("description")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let link = event
            .get("link")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing event link".to_string()))?
            .to_string();

        let closed = event
            .get("closed")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Parse categories array
        let categories = event
            .get("categories")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|cat| Self::parse_category(cat).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Parse sources array
        let sources = event
            .get("sources")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|src| Self::parse_source(src).ok())
                    .collect()
            })
            .unwrap_or_default();

        // Parse geometry array
        let geometry = event
            .get("geometry")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|geom| Self::parse_geometry(geom).ok())
                    .collect()
            })
            .unwrap_or_default();

        Ok(NaturalEvent {
            id,
            title,
            description,
            link,
            closed: closed.is_some(),
            closed_date: closed,
            categories,
            sources,
            geometry,
        })
    }

    /// Parse category object
    fn parse_category(cat: &Value) -> ExchangeResult<EventCategory> {
        let id = cat
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing category id".to_string()))?
            .to_string();

        let title = cat
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing category title".to_string()))?
            .to_string();

        Ok(EventCategory { id, title })
    }

    /// Parse source object
    fn parse_source(src: &Value) -> ExchangeResult<EventSource> {
        let id = src
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing source id".to_string()))?
            .to_string();

        let url = src
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing source url".to_string()))?
            .to_string();

        Ok(EventSource { id, url })
    }

    /// Parse geometry object
    fn parse_geometry(geom: &Value) -> ExchangeResult<EventGeometry> {
        let magnitude_value = geom
            .get("magnitudeValue")
            .and_then(|v| v.as_f64());

        let magnitude_unit = geom
            .get("magnitudeUnit")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let date = geom
            .get("date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing geometry date".to_string()))?
            .to_string();

        let event_type = geom
            .get("type")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing geometry type".to_string()))?
            .to_string();

        // Parse coordinates - can be Point [lon, lat] or Polygon [[[lon, lat]...]]
        let coordinates = geom
            .get("coordinates")
            .ok_or_else(|| ExchangeError::Parse("Missing coordinates".to_string()))?;

        let (lon, lat) = Self::extract_coordinates(coordinates, &event_type)?;

        Ok(EventGeometry {
            magnitude_value,
            magnitude_unit,
            date,
            event_type,
            longitude: lon,
            latitude: lat,
        })
    }

    /// Extract longitude and latitude from coordinates array
    /// For Point: [lon, lat]
    /// For Polygon: [[[lon, lat], ...]] - extract first point
    fn extract_coordinates(coords: &Value, geom_type: &str) -> ExchangeResult<(f64, f64)> {
        if geom_type == "Point" {
            // Point format: [lon, lat]
            let arr = coords
                .as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid Point coordinates".to_string()))?;

            if arr.len() < 2 {
                return Err(ExchangeError::Parse("Point requires 2 coordinates".to_string()));
            }

            let lon = arr[0]
                .as_f64()
                .ok_or_else(|| ExchangeError::Parse("Invalid longitude".to_string()))?;
            let lat = arr[1]
                .as_f64()
                .ok_or_else(|| ExchangeError::Parse("Invalid latitude".to_string()))?;

            Ok((lon, lat))
        } else if geom_type == "Polygon" {
            // Polygon format: [[[lon, lat], ...]]
            // Extract first point from first ring
            let outer_arr = coords
                .as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid Polygon coordinates".to_string()))?;

            if outer_arr.is_empty() {
                return Err(ExchangeError::Parse("Empty Polygon coordinates".to_string()));
            }

            let ring = outer_arr[0]
                .as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid Polygon ring".to_string()))?;

            if ring.is_empty() {
                return Err(ExchangeError::Parse("Empty Polygon ring".to_string()));
            }

            let point = ring[0]
                .as_array()
                .ok_or_else(|| ExchangeError::Parse("Invalid Polygon point".to_string()))?;

            if point.len() < 2 {
                return Err(ExchangeError::Parse("Polygon point requires 2 coordinates".to_string()));
            }

            let lon = point[0]
                .as_f64()
                .ok_or_else(|| ExchangeError::Parse("Invalid longitude".to_string()))?;
            let lat = point[1]
                .as_f64()
                .ok_or_else(|| ExchangeError::Parse("Invalid latitude".to_string()))?;

            Ok((lon, lat))
        } else {
            Err(ExchangeError::Parse(format!("Unsupported geometry type: {}", geom_type)))
        }
    }

    /// Parse categories list response
    pub fn parse_categories(data: &Value) -> ExchangeResult<Vec<EventCategory>> {
        let categories_array = data
            .get("categories")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'categories' field".to_string()))?;

        let mut categories = Vec::new();

        for cat_val in categories_array {
            if let Ok(category) = Self::parse_category(cat_val) {
                categories.push(category);
            }
        }

        Ok(categories)
    }

    /// Parse sources list response
    pub fn parse_sources(data: &Value) -> ExchangeResult<Vec<EventSource>> {
        let sources_array = data
            .get("sources")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'sources' field".to_string()))?;

        let mut sources = Vec::new();

        for src_val in sources_array {
            if let Ok(source) = Self::parse_source(src_val) {
                sources.push(source);
            }
        }

        Ok(sources)
    }

    /// Parse single event response (for get_event_by_id)
    pub fn parse_single_event(data: &Value) -> ExchangeResult<NaturalEvent> {
        Self::parse_event(data)
    }
}

// =============================================================================
// NASA EONET-SPECIFIC TYPES
// =============================================================================

/// Natural disaster event
#[derive(Debug, Clone)]
pub struct NaturalEvent {
    /// Event ID (e.g., "EONET_17841")
    pub id: String,
    /// Event title
    pub title: String,
    /// Event description (optional)
    pub description: Option<String>,
    /// Event detail link
    pub link: String,
    /// Whether event is closed
    pub closed: bool,
    /// Closed date if event is closed
    pub closed_date: Option<String>,
    /// Event categories
    pub categories: Vec<EventCategory>,
    /// Event sources
    pub sources: Vec<EventSource>,
    /// Event geometries (location data with timestamps)
    pub geometry: Vec<EventGeometry>,
}

/// Event category
#[derive(Debug, Clone)]
pub struct EventCategory {
    /// Category ID (e.g., "wildfires", "severeStorms")
    pub id: String,
    /// Category display name
    pub title: String,
}

/// Event source
#[derive(Debug, Clone)]
pub struct EventSource {
    /// Source ID (e.g., "InciWeb", "USGS_EHP")
    pub id: String,
    /// Source URL
    pub url: String,
}

/// Event geometry (location + timestamp)
#[derive(Debug, Clone)]
pub struct EventGeometry {
    /// Magnitude value (optional, e.g., acres for wildfires)
    pub magnitude_value: Option<f64>,
    /// Magnitude unit (optional, e.g., "acres", "kts")
    pub magnitude_unit: Option<String>,
    /// Timestamp for this location (ISO 8601)
    pub date: String,
    /// Geometry type ("Point" or "Polygon")
    pub event_type: String,
    /// Longitude
    pub longitude: f64,
    /// Latitude
    pub latitude: f64,
}
