//! Bundesbank response parsers
//!
//! Parse SDMX-JSON responses to domain types based on Bundesbank API response formats.
//!
//! The Bundesbank uses SDMX (Statistical Data and Metadata eXchange) standard,
//! which is common for statistical/economic data providers.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct BundesbankParser;

impl BundesbankParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for API errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // SDMX errors are typically in "error" field or HTTP status
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            return Err(ExchangeError::Api {
                code: -1,
                message: message.to_string(),
            });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse SDMX time series observations
    ///
    /// SDMX-JSON data structure (simplified):
    /// ```json
    /// {
    ///   "dataSets": [{
    ///     "series": {
    ///       "0:0:0:0:0:0": {
    ///         "observations": {
    ///           "0": [1.234],
    ///           "1": [1.235]
    ///         }
    ///       }
    ///     }
    ///   }],
    ///   "structure": {
    ///     "dimensions": {
    ///       "observation": [{
    ///         "id": "TIME_PERIOD",
    ///         "values": [
    ///           {"id": "2024-01", "name": "2024-01"},
    ///           {"id": "2024-02", "name": "2024-02"}
    ///         ]
    ///       }]
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_observations(response: &Value) -> ExchangeResult<Vec<SdmxObservation>> {
        let mut result = Vec::new();

        // Extract time period dimension values
        let time_periods = Self::extract_time_periods(response)?;

        // Extract data sets
        let datasets = response
            .get("dataSets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataSets' array".to_string()))?;

        // Process each dataset
        for dataset in datasets {
            if let Some(series_obj) = dataset.get("series").and_then(|v| v.as_object()) {
                // Process each series
                for (_series_key, series_data) in series_obj {
                    if let Some(obs_obj) = series_data.get("observations").and_then(|v| v.as_object()) {
                        // Process each observation
                        for (time_idx, obs_array) in obs_obj {
                            let idx: usize = time_idx
                                .parse()
                                .map_err(|_| ExchangeError::Parse(format!("Invalid time index: {}", time_idx)))?;

                            let period = time_periods
                                .get(idx)
                                .ok_or_else(|| ExchangeError::Parse(format!("Time period index {} out of range", idx)))?;

                            let value = obs_array
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|v| v.as_f64());

                            result.push(SdmxObservation {
                                period: period.clone(),
                                value,
                            });
                        }
                    }
                }
            }
        }

        Ok(result)
    }

    /// Extract time periods from SDMX structure
    fn extract_time_periods(response: &Value) -> ExchangeResult<Vec<String>> {
        let dimensions = response
            .pointer("/structure/dimensions/observation")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing observation dimensions".to_string()))?;

        // Find TIME_PERIOD dimension
        for dim in dimensions {
            if let Some(id) = dim.get("id").and_then(|v| v.as_str()) {
                if id == "TIME_PERIOD" {
                    let values = dim
                        .get("values")
                        .and_then(|v| v.as_array())
                        .ok_or_else(|| ExchangeError::Parse("Missing TIME_PERIOD values".to_string()))?;

                    return values
                        .iter()
                        .map(|v| {
                            v.get("id")
                                .and_then(|id| id.as_str())
                                .map(|s| s.to_string())
                                .ok_or_else(|| ExchangeError::Parse("Invalid time period value".to_string()))
                        })
                        .collect();
                }
            }
        }

        Err(ExchangeError::Parse("TIME_PERIOD dimension not found".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STRUCTURE PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse list of dataflows
    pub fn parse_dataflows(response: &Value) -> ExchangeResult<Vec<SdmxDataflow>> {
        let dataflows = response
            .pointer("/data/dataflows")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing dataflows array".to_string()))?;

        dataflows
            .iter()
            .map(|df| {
                let id = Self::require_str(df, "id")?;
                let name = Self::get_str(df, "name").unwrap_or("Unnamed");
                let description = Self::get_str(df, "description");

                Ok(SdmxDataflow {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: description.map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse dataflow metadata
    pub fn parse_dataflow(response: &Value) -> ExchangeResult<SdmxDataflow> {
        let dataflow = response
            .pointer("/data/dataflows/0")
            .ok_or_else(|| ExchangeError::Parse("Missing dataflow object".to_string()))?;

        let id = Self::require_str(dataflow, "id")?;
        let name = Self::get_str(dataflow, "name").unwrap_or("Unnamed");
        let description = Self::get_str(dataflow, "description");

        Ok(SdmxDataflow {
            id: id.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
        })
    }

    /// Parse data structure definition
    pub fn parse_datastructure(response: &Value) -> ExchangeResult<SdmxDataStructure> {
        let dsd = response
            .pointer("/data/dataStructures/0")
            .ok_or_else(|| ExchangeError::Parse("Missing dataStructure object".to_string()))?;

        let id = Self::require_str(dsd, "id")?;
        let name = Self::get_str(dsd, "name").unwrap_or("Unnamed");

        Ok(SdmxDataStructure {
            id: id.to_string(),
            name: name.to_string(),
            dimensions: Vec::new(), // Simplified - full parsing would extract dimension details
        })
    }

    /// Parse codelist
    pub fn parse_codelist(response: &Value) -> ExchangeResult<SdmxCodelist> {
        let codelist = response
            .pointer("/data/codelists/0")
            .ok_or_else(|| ExchangeError::Parse("Missing codelist object".to_string()))?;

        let id = Self::require_str(codelist, "id")?;
        let name = Self::get_str(codelist, "name").unwrap_or("Unnamed");

        let codes = if let Some(codes_array) = codelist.get("codes").and_then(|v| v.as_array()) {
            codes_array
                .iter()
                .filter_map(|code| {
                    let code_id = Self::get_str(code, "id")?;
                    let code_name = Self::get_str(code, "name")?;
                    Some(SdmxCode {
                        id: code_id.to_string(),
                        name: code_name.to_string(),
                    })
                })
                .collect()
        } else {
            Vec::new()
        };

        Ok(SdmxCodelist {
            id: id.to_string(),
            name: name.to_string(),
            codes,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", key)))
    }

    fn get_str<'a>(obj: &'a Value, key: &str) -> Option<&'a str> {
        obj.get(key).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// SDMX time series observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxObservation {
    /// Time period (e.g., "2024-01-15", "2024-Q1", "2024")
    pub period: String,
    /// Observation value (None if missing/unavailable)
    pub value: Option<f64>,
}

/// SDMX dataflow (dataset definition)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxDataflow {
    /// Dataflow ID (e.g., "BBEX3", "BBSIS")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Optional description
    pub description: Option<String>,
}

/// SDMX data structure definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxDataStructure {
    /// DSD ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Dimension definitions (simplified)
    pub dimensions: Vec<String>,
}

/// SDMX codelist (list of valid dimension values)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxCodelist {
    /// Codelist ID
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// List of codes
    pub codes: Vec<SdmxCode>,
}

/// SDMX code (single dimension value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxCode {
    /// Code ID (e.g., "EUR", "USD", "D" for daily)
    pub id: String,
    /// Human-readable name
    pub name: String,
}

/// SDMX concept scheme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxConceptScheme {
    /// Concept scheme ID
    pub id: String,
    /// Human-readable name
    pub name: String,
}
