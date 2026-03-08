//! ECB response parsers
//!
//! Parse SDMX-JSON responses to domain types based on ECB API response formats.
//!
//! ECB uses SDMX 2.1 JSON format for economic data.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct EcbParser;

impl EcbParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ECB-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for SDMX errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // SDMX doesn't have a standard error format, but typically returns HTTP errors
        // If we got JSON, it's likely a valid response
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown ECB API error");
            return Err(ExchangeError::Api {
                code: error.get("code").and_then(|c| c.as_i64()).unwrap_or(0) as i32,
                message: message.to_string(),
            });
        }
        Ok(())
    }

    /// Parse SDMX data observations
    ///
    /// SDMX-JSON format example:
    /// ```json
    /// {
    ///   "data": {
    ///     "dataSets": [{
    ///       "series": {
    ///         "0:0:0:0:0": {
    ///           "observations": {
    ///             "0": [1.2345],
    ///             "1": [1.2346]
    ///           }
    ///         }
    ///       }
    ///     }],
    ///     "structure": {
    ///       "dimensions": {
    ///         "observation": [{
    ///           "values": [
    ///             {"id": "2023-01-01"},
    ///             {"id": "2023-01-02"}
    ///           ]
    ///         }]
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_data_observations(response: &Value) -> ExchangeResult<Vec<SdmxObservation>> {
        let mut all_observations = Vec::new();

        // Navigate to data.dataSets
        let datasets = response
            .get("data")
            .and_then(|d| d.get("dataSets"))
            .and_then(|ds| ds.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data.dataSets' array".to_string()))?;

        // Get time dimension values
        let time_values = response
            .get("data")
            .and_then(|d| d.get("structure"))
            .and_then(|s| s.get("dimensions"))
            .and_then(|d| d.get("observation"))
            .and_then(|o| o.as_array())
            .and_then(|arr| arr.first())
            .and_then(|dim| dim.get("values"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing time dimension values".to_string()))?;

        // Process each dataset
        for dataset in datasets {
            if let Some(series_obj) = dataset.get("series").and_then(|s| s.as_object()) {
                // Process each series in the dataset
                for (series_key, series_data) in series_obj {
                    if let Some(obs_obj) = series_data.get("observations").and_then(|o| o.as_object()) {
                        // Process each observation
                        for (time_idx, values) in obs_obj {
                            let idx: usize = time_idx.parse()
                                .map_err(|_| ExchangeError::Parse(format!("Invalid time index: {}", time_idx)))?;

                            // Get time period
                            let time_period = time_values
                                .get(idx)
                                .and_then(|v| v.get("id"))
                                .and_then(|id| id.as_str())
                                .ok_or_else(|| ExchangeError::Parse(format!("Missing time value at index {}", idx)))?;

                            // Extract value (first element in array)
                            let value = values
                                .as_array()
                                .and_then(|arr| arr.first())
                                .and_then(|v| v.as_f64());

                            all_observations.push(SdmxObservation {
                                series_key: series_key.clone(),
                                time_period: time_period.to_string(),
                                value,
                            });
                        }
                    }
                }
            }
        }

        Ok(all_observations)
    }

    /// Parse dataflows list
    pub fn parse_dataflows(response: &Value) -> ExchangeResult<Vec<SdmxDataflow>> {
        let dataflows = response
            .get("data")
            .and_then(|d| d.get("dataflows"))
            .and_then(|df| df.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data.dataflows' array".to_string()))?;

        dataflows
            .iter()
            .map(|df| {
                let id = Self::require_str(df, "id")?;
                let name = Self::get_str(df, "name").unwrap_or("");
                let description = Self::get_str(df, "description").unwrap_or("");

                Ok(SdmxDataflow {
                    id: id.to_string(),
                    name: name.to_string(),
                    description: description.to_string(),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get required string field
    fn require_str<'a>(json: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        json.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    /// Get optional string field
    fn get_str<'a>(json: &'a Value, field: &str) -> Option<&'a str> {
        json.get(field).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// ECB DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// SDMX observation (time series data point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxObservation {
    /// Series key (e.g., "0:0:0:0:0" maps to dimension values)
    pub series_key: String,
    /// Time period (e.g., "2023-01-01", "2023-Q1", "2023-M01")
    pub time_period: String,
    /// Observation value (None if missing)
    pub value: Option<f64>,
}

/// SDMX dataflow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxDataflow {
    /// Dataflow ID (e.g., "EXR", "ICP")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
}

/// SDMX data structure definition metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxDataStructure {
    /// DSD ID
    pub id: String,
    /// Name
    pub name: String,
    /// Dimensions
    pub dimensions: Vec<String>,
}

/// SDMX codelist
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxCodelist {
    /// Codelist ID
    pub id: String,
    /// Name
    pub name: String,
    /// Codes
    pub codes: Vec<SdmxCode>,
}

/// SDMX code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SdmxCode {
    /// Code ID
    pub id: String,
    /// Name
    pub name: String,
}
