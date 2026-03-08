//! UN Population response parsers
//!
//! Parse JSON responses from the UN Population Data Portal API.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct UnPopParser;

impl UnPopParser {
    /// Parse locations response
    pub fn parse_locations(response: &Value) -> ExchangeResult<Vec<UnPopLocation>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|loc| {
                serde_json::from_value(loc.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse location: {}", e)))
            })
            .collect()
    }

    /// Parse indicators response
    pub fn parse_indicators(response: &Value) -> ExchangeResult<Vec<UnPopIndicator>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|ind| {
                serde_json::from_value(ind.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse indicator: {}", e)))
            })
            .collect()
    }

    /// Parse data points response
    pub fn parse_data_points(response: &Value) -> ExchangeResult<UnPopResponse<UnPopDataPoint>> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse data points: {}", e)))
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error.as_str().unwrap_or("Unknown error").to_string();
            return Err(ExchangeError::Api { code: 0, message });
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UN POPULATION TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// UN Population location (country or region)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnPopLocation {
    pub id: u32,
    pub name: String,
    #[serde(rename = "iso3Code")]
    pub iso3: Option<String>,
    #[serde(rename = "iso2Code")]
    pub iso2: Option<String>,
    #[serde(rename = "locationType")]
    pub location_type: String,
}

/// UN Population indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnPopIndicator {
    pub id: u32,
    pub name: String,
    #[serde(rename = "shortName")]
    pub short_name: String,
    pub description: String,
}

/// UN Population data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnPopDataPoint {
    #[serde(rename = "locationId")]
    pub location_id: u32,
    pub location: String,
    #[serde(rename = "timeLabel")]
    pub year: String,
    pub value: f64,
    pub sex: Option<String>,
    pub variant: Option<String>,
}

/// UN Population paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnPopResponse<T> {
    pub total: u64,
    pub data: Vec<T>,
}
