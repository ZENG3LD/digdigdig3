//! UNHCR response parsers
//!
//! Parse JSON responses from the UNHCR Population Statistics API.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ExchangeError;

type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct UnhcrParser;

impl UnhcrParser {
    /// Parse population data response
    pub fn parse_population(response: &Value) -> ExchangeResult<Vec<UnhcrPopulationData>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|item| {
                serde_json::from_value(item.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse population data: {}", e)))
            })
            .collect()
    }

    /// Parse countries response
    pub fn parse_countries(response: &Value) -> ExchangeResult<Vec<UnhcrCountry>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|country| {
                serde_json::from_value(country.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse country: {}", e)))
            })
            .collect()
    }

    /// Parse generic JSON array response
    pub fn parse_json_array(response: &Value) -> ExchangeResult<Vec<Value>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        Ok(data.clone())
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error")
                .to_string();
            let code = error
                .get("code")
                .and_then(|c| c.as_i64())
                .unwrap_or(0) as i32;
            return Err(ExchangeError::Api { code, message });
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// UNHCR TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// UNHCR population data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnhcrPopulationData {
    #[serde(default)]
    pub year: Option<u32>,
    #[serde(default)]
    pub country_of_origin: Option<String>,
    #[serde(default)]
    pub country_of_asylum: Option<String>,
    #[serde(default)]
    pub refugees: Option<u64>,
    #[serde(default)]
    pub asylum_seekers: Option<u64>,
    #[serde(default)]
    pub idps: Option<u64>,
    #[serde(default)]
    pub stateless: Option<u64>,
}

/// UNHCR country
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnhcrCountry {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub iso3: Option<String>,
}
