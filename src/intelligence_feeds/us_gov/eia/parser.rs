//! EIA response parser

use crate::core::types::{ExchangeError, ExchangeResult};
use serde::{Deserialize, Serialize};

/// EIA API error response
#[derive(Debug, Deserialize)]
struct EiaError {
    #[serde(default)]
    error: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

/// EIA data observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EiaObservation {
    /// Time period (e.g., "2024-01", "2024-01-15", "2024")
    pub period: String,

    /// Data value
    pub value: Option<f64>,

    /// Series description (metadata)
    #[serde(rename = "series-description", default)]
    pub series_description: Option<String>,

    /// Unit of measurement
    #[serde(default)]
    pub unit: Option<String>,

    /// Additional fields as needed
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// EIA data response wrapper
#[derive(Debug, Deserialize)]
pub struct EiaDataResponse {
    pub response: EiaDataInner,
}

#[derive(Debug, Deserialize)]
pub struct EiaDataInner {
    /// Total number of records available
    #[serde(default)]
    pub total: Option<u64>,

    /// Data observations
    #[serde(default)]
    pub data: Vec<EiaObservation>,
}

/// EIA metadata response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EiaMetadata {
    pub response: EiaMetadataInner,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EiaMetadataInner {
    /// Route ID
    #[serde(default)]
    pub id: Option<String>,

    /// Route name
    #[serde(default)]
    pub name: Option<String>,

    /// Route description
    #[serde(default)]
    pub description: Option<String>,

    /// Frequency options
    #[serde(default)]
    pub frequency: Vec<serde_json::Value>,

    /// Available data columns
    #[serde(default)]
    pub data: Vec<serde_json::Value>,

    /// Additional metadata
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// EIA facet response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EiaFacet {
    /// Facet ID
    pub id: String,

    /// Facet description
    #[serde(default)]
    pub description: Option<String>,

    /// Additional facet data
    #[serde(flatten)]
    pub extra: serde_json::Map<String, serde_json::Value>,
}

/// EIA facets response wrapper
#[derive(Debug, Deserialize)]
pub struct EiaFacetsResponse {
    pub response: EiaFacetsInner,
}

#[derive(Debug, Deserialize)]
pub struct EiaFacetsInner {
    #[serde(default)]
    pub facets: Vec<EiaFacet>,
}

pub struct EiaParser;

impl EiaParser {
    /// Check for API errors in response
    pub fn check_error(json: &serde_json::Value) -> ExchangeResult<()> {
        // Check for error object
        if let Some(error_obj) = json.get("error") {
            if let Ok(err) = serde_json::from_value::<EiaError>(error_obj.clone()) {
                let msg = err.error.or(err.message).unwrap_or_else(|| "Unknown EIA API error".to_string());
                return Err(ExchangeError::Api {
                    code: -1,
                    message: msg,
                });
            }
        }

        // Check for direct error message
        if let Some(msg) = json.get("message").and_then(|v| v.as_str()) {
            if msg.to_lowercase().contains("error") {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: msg.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Parse data observations from response
    pub fn parse_data(json: &serde_json::Value) -> ExchangeResult<Vec<EiaObservation>> {
        let response: EiaDataResponse = serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse EIA data response: {}", e)))?;

        Ok(response.response.data)
    }

    /// Parse metadata from response
    pub fn parse_metadata(json: &serde_json::Value) -> ExchangeResult<EiaMetadata> {
        serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse EIA metadata: {}", e)))
    }

    /// Parse facets from response
    pub fn parse_facets(json: &serde_json::Value) -> ExchangeResult<Vec<EiaFacet>> {
        let response: EiaFacetsResponse = serde_json::from_value(json.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse EIA facets response: {}", e)))?;

        Ok(response.response.facets)
    }
}
