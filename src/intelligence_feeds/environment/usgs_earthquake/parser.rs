//! USGS Earthquake response parsers
//!
//! Parse GeoJSON responses to domain types based on USGS API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct UsgsEarthquakeParser;

impl UsgsEarthquakeParser {
    /// Parse earthquake query response (GeoJSON format)
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "type": "FeatureCollection",
    ///   "metadata": {
    ///     "generated": 1234567890000,
    ///     "url": "https://...",
    ///     "title": "USGS Earthquakes",
    ///     "status": 200,
    ///     "api": "1.10.3",
    ///     "count": 100
    ///   },
    ///   "features": [
    ///     {
    ///       "type": "Feature",
    ///       "properties": {
    ///         "mag": 5.4,
    ///         "place": "10km NE of Example",
    ///         "time": 1234567890000,
    ///         "updated": 1234567890000,
    ///         "url": "https://...",
    ///         "detail": "https://...",
    ///         "tsunami": 0,
    ///         "status": "reviewed",
    ///         "sig": 442
    ///       },
    ///       "geometry": {
    ///         "type": "Point",
    ///         "coordinates": [-122.5, 37.8, 10.2]
    ///       },
    ///       "id": "us1000abcd"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_earthquake_response(response: &Value) -> ExchangeResult<EarthquakeResponse> {
        let type_field = Self::require_str(response, "type")?;

        let metadata_obj = response
            .get("metadata")
            .ok_or_else(|| ExchangeError::Parse("Missing 'metadata' object".to_string()))?;

        let metadata = EqMetadata {
            generated: Self::require_u64(metadata_obj, "generated")?,
            url: Self::require_str(metadata_obj, "url")?.to_string(),
            title: Self::require_str(metadata_obj, "title")?.to_string(),
            status: Self::require_u32(metadata_obj, "status")?,
            api: Self::require_str(metadata_obj, "api")?.to_string(),
            count: Self::require_u64(metadata_obj, "count")?,
        };

        let features_array = response
            .get("features")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'features' array".to_string()))?;

        let features: Result<Vec<EarthquakeFeature>, ExchangeError> = features_array
            .iter()
            .map(|feat| {
                let id = Self::require_str(feat, "id")?.to_string();

                let props_obj = feat
                    .get("properties")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'properties' object".to_string()))?;

                let properties = EqProperties {
                    mag: Self::get_f64(props_obj, "mag"),
                    place: Self::get_str(props_obj, "place").map(|s| s.to_string()),
                    time: Self::require_u64(props_obj, "time")?,
                    updated: Self::require_u64(props_obj, "updated")?,
                    url: Self::require_str(props_obj, "url")?.to_string(),
                    detail: Self::require_str(props_obj, "detail")?.to_string(),
                    felt: Self::get_u32(props_obj, "felt"),
                    cdi: Self::get_f64(props_obj, "cdi"),
                    mmi: Self::get_f64(props_obj, "mmi"),
                    alert: Self::get_str(props_obj, "alert").map(|s| s.to_string()),
                    status: Self::require_str(props_obj, "status")?.to_string(),
                    tsunami: Self::require_u32(props_obj, "tsunami")?,
                    sig: Self::require_u32(props_obj, "sig")?,
                    net: Self::require_str(props_obj, "net")?.to_string(),
                    code: Self::require_str(props_obj, "code")?.to_string(),
                    ids: Self::require_str(props_obj, "ids")?.to_string(),
                    sources: Self::require_str(props_obj, "sources")?.to_string(),
                    types: Self::require_str(props_obj, "types")?.to_string(),
                    nst: Self::get_u32(props_obj, "nst"),
                    dmin: Self::get_f64(props_obj, "dmin"),
                    rms: Self::get_f64(props_obj, "rms"),
                    gap: Self::get_f64(props_obj, "gap"),
                    mag_type: Self::require_str(props_obj, "magType")?.to_string(),
                    event_type: Self::require_str(props_obj, "type")?.to_string(),
                    title: Self::require_str(props_obj, "title")?.to_string(),
                };

                let geom_obj = feat
                    .get("geometry")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'geometry' object".to_string()))?;

                let type_field_geom = Self::require_str(geom_obj, "type")?;

                let coordinates_array = geom_obj
                    .get("coordinates")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ExchangeError::Parse("Missing 'coordinates' array".to_string()))?;

                let coordinates: Vec<f64> = coordinates_array
                    .iter()
                    .filter_map(|v| v.as_f64())
                    .collect();

                if coordinates.len() != 3 {
                    return Err(ExchangeError::Parse(format!(
                        "Expected 3 coordinates, got {}",
                        coordinates.len()
                    )));
                }

                let geometry = EqGeometry {
                    type_field: type_field_geom.to_string(),
                    coordinates,
                };

                Ok(EarthquakeFeature {
                    id,
                    properties,
                    geometry,
                })
            })
            .collect();

        Ok(EarthquakeResponse {
            type_field: type_field.to_string(),
            metadata,
            features: features?,
        })
    }

    /// Parse count response
    pub fn parse_count_response(response: &Value) -> ExchangeResult<u64> {
        Self::require_u64(response, "count")
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // USGS API returns error in different formats
        if let Some(error_msg) = response.get("error") {
            let message = error_msg.as_str().unwrap_or("Unknown error").to_string();
            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }
        Ok(())
    }

    // Helper methods
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

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// USGS-specific types

/// USGS Earthquake query response (GeoJSON FeatureCollection)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarthquakeResponse {
    #[serde(rename = "type")]
    pub type_field: String,
    pub metadata: EqMetadata,
    pub features: Vec<EarthquakeFeature>,
}

/// Earthquake response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqMetadata {
    pub generated: u64,
    pub url: String,
    pub title: String,
    pub status: u32,
    pub api: String,
    pub count: u64,
}

/// Individual earthquake feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarthquakeFeature {
    pub id: String,
    pub properties: EqProperties,
    pub geometry: EqGeometry,
}

/// Earthquake properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqProperties {
    #[serde(default)]
    pub mag: Option<f64>,
    #[serde(default)]
    pub place: Option<String>,
    pub time: u64,
    pub updated: u64,
    pub url: String,
    pub detail: String,
    #[serde(default)]
    pub felt: Option<u32>,
    #[serde(default)]
    pub cdi: Option<f64>,
    #[serde(default)]
    pub mmi: Option<f64>,
    #[serde(default)]
    pub alert: Option<String>,
    pub status: String,
    pub tsunami: u32,
    pub sig: u32,
    pub net: String,
    pub code: String,
    pub ids: String,
    pub sources: String,
    pub types: String,
    #[serde(default)]
    pub nst: Option<u32>,
    #[serde(default)]
    pub dmin: Option<f64>,
    #[serde(default)]
    pub rms: Option<f64>,
    #[serde(default)]
    pub gap: Option<f64>,
    #[serde(rename = "magType")]
    pub mag_type: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub title: String,
}

/// Earthquake geometry (GeoJSON Point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EqGeometry {
    #[serde(rename = "type")]
    pub type_field: String,
    /// Coordinates: [longitude, latitude, depth]
    pub coordinates: Vec<f64>,
}
