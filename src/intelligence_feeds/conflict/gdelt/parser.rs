//! GDELT response parser

use serde::{Deserialize, Serialize};
use crate::core::types::{ExchangeError, ExchangeResult};

/// GDELT Article from DOC API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdeltArticle {
    /// Article URL
    pub url: String,
    /// Article title
    pub title: String,
    /// When GDELT first saw this article (ISO 8601: YYYY-MM-DDTHH:MM:SSZ)
    #[serde(rename = "seendate")]
    pub seen_date: String,
    /// Article language
    pub language: String,
    /// Source country code (e.g., "US", "GB", "RU")
    #[serde(rename = "sourcecountry")]
    pub source_country: String,
    /// Tone score (-100 to +100, average ~0)
    pub tone: f64,
}

/// GDELT DOC API response (artlist mode)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdeltDocResponse {
    /// List of articles
    pub articles: Vec<GdeltArticle>,
}

/// Timeline data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelinePoint {
    /// Timestamp
    pub date: String,
    /// Value (volume count or tone score)
    pub value: f64,
}

/// Geographic feature from GEO API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoFeature {
    /// Feature type (always "Feature")
    #[serde(rename = "type")]
    pub feature_type: String,
    /// Geometry
    pub geometry: GeoGeometry,
    /// Properties
    pub properties: serde_json::Value,
}

/// Geographic geometry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoGeometry {
    /// Geometry type (Point, Polygon, etc.)
    #[serde(rename = "type")]
    pub geometry_type: String,
    /// Coordinates
    pub coordinates: Vec<f64>,
}

/// GEO API response (GeoJSON)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GdeltGeoResponse {
    /// GeoJSON type (always "FeatureCollection")
    #[serde(rename = "type")]
    pub collection_type: String,
    /// Features
    pub features: Vec<GeoFeature>,
}

/// TV clip from TV API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TvClip {
    /// Show name
    pub show: String,
    /// Station
    pub station: String,
    /// Preview URL
    pub preview_url: String,
    /// Timestamp
    pub date: String,
}

/// Context API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextResponse {
    /// Context data (structure varies)
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// GDELT response parser
pub struct GdeltParser;

impl GdeltParser {
    /// Check for API errors in response
    pub fn check_error(json: &serde_json::Value) -> ExchangeResult<()> {
        // GDELT doesn't have a standard error format
        // HTTP errors are handled at the request level
        // If we get JSON, it's likely successful

        // Check for common error patterns
        if let Some(error) = json.get("error").and_then(|e| e.as_str()) {
            return Err(ExchangeError::Api {
                code: -1,
                message: error.to_string(),
            });
        }

        Ok(())
    }

    /// Parse DOC API response (artlist mode)
    pub fn parse_articles(json: &serde_json::Value) -> ExchangeResult<Vec<GdeltArticle>> {
        let response: GdeltDocResponse = serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse articles: {}", e)))?;

        Ok(response.articles)
    }

    /// Parse timeline data (timelinevol, timelinetone modes)
    pub fn parse_timeline(json: &serde_json::Value) -> ExchangeResult<Vec<TimelinePoint>> {
        // GDELT timeline format varies by mode
        // This is a generic parser - may need adjustment based on actual API responses
        let timeline: Vec<TimelinePoint> = serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse timeline: {}", e)))?;

        Ok(timeline)
    }

    /// Parse GEO API response (GeoJSON)
    pub fn parse_geo(json: &serde_json::Value) -> ExchangeResult<GdeltGeoResponse> {
        let geo: GdeltGeoResponse = serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse geo data: {}", e)))?;

        Ok(geo)
    }

    /// Parse TV API response
    pub fn parse_tv_clips(json: &serde_json::Value) -> ExchangeResult<Vec<TvClip>> {
        // TV API response structure varies by mode
        let clips: Vec<TvClip> = serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse TV clips: {}", e)))?;

        Ok(clips)
    }

    /// Parse Context API response
    pub fn parse_context(json: &serde_json::Value) -> ExchangeResult<ContextResponse> {
        Ok(ContextResponse {
            data: json.clone(),
        })
    }
}
