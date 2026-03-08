//! IMF response parsers
//!
//! Parse JSON responses to domain types based on IMF JSON API response formats.
//!
//! IMF uses SDMX JSON format with @ prefixes for attributes.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct ImfParser;

impl ImfParser {
    // ═══════════════════════════════════════════════════════════════════════
    // IMF-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse dataflow list response
    ///
    /// Example response structure (simplified):
    /// ```json
    /// {
    ///   "Structure": {
    ///     "Dataflows": {
    ///       "Dataflow": [
    ///         {
    ///           "@id": "IFS",
    ///           "Name": [{"#text": "International Financial Statistics"}]
    ///         }
    ///       ]
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_dataflows(response: &Value) -> ExchangeResult<Vec<Dataflow>> {
        let dataflows = response
            .pointer("/Structure/Dataflows/Dataflow")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing dataflows array".to_string()))?;

        dataflows
            .iter()
            .map(|df| {
                let id = Self::require_attr(df, "@id")?;
                let name = Self::get_text_field(df, "Name").unwrap_or_else(|| id.to_string());
                let description = Self::get_text_field(df, "Description");

                Ok(Dataflow {
                    id: id.to_string(),
                    name,
                    description,
                })
            })
            .collect()
    }

    /// Parse compact data response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "CompactData": {
    ///     "DataSet": {
    ///       "Series": [
    ///         {
    ///           "@FREQ": "A",
    ///           "@REF_AREA": "US",
    ///           "@INDICATOR": "NGDP_RPCH",
    ///           "Obs": [
    ///             { "@TIME_PERIOD": "2020", "@OBS_VALUE": "2.3" }
    ///           ]
    ///         }
    ///       ]
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_compact_data(response: &Value) -> ExchangeResult<Vec<ImfSeries>> {
        let series_array = response
            .pointer("/CompactData/DataSet/Series")
            .or_else(|| response.pointer("/CompactData/DataSet/0/Series"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing series array".to_string()))?;

        series_array
            .iter()
            .map(|series| {
                let freq = Self::get_attr(series, "@FREQ").unwrap_or_default();
                let ref_area = Self::get_attr(series, "@REF_AREA").unwrap_or_default();
                let indicator = Self::get_attr(series, "@INDICATOR").unwrap_or_default();

                let observations = Self::parse_observations(series)?;

                Ok(ImfSeries {
                    freq,
                    ref_area,
                    indicator,
                    observations,
                })
            })
            .collect()
    }

    /// Parse observations from a series
    fn parse_observations(series: &Value) -> ExchangeResult<Vec<ImfObservation>> {
        let obs_array = series
            .get("Obs")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing Obs array".to_string()))?;

        obs_array
            .iter()
            .map(|obs| {
                let time_period = Self::require_attr(obs, "@TIME_PERIOD")?;
                let obs_value_str = Self::require_attr(obs, "@OBS_VALUE")?;

                let value = obs_value_str
                    .parse::<f64>()
                    .ok();

                Ok(ImfObservation {
                    time_period: time_period.to_string(),
                    value,
                    obs_status: Self::get_attr(obs, "@OBS_STATUS"),
                })
            })
            .collect()
    }

    /// Parse data structure response
    ///
    /// Contains dimension definitions and code lists
    pub fn parse_data_structure(response: &Value) -> ExchangeResult<DataStructure> {
        let structure = response
            .pointer("/Structure/Structures/DataStructures/DataStructure/0")
            .or_else(|| response.pointer("/Structure/Structures/DataStructures/DataStructure"))
            .ok_or_else(|| ExchangeError::Parse("Missing data structure".to_string()))?;

        let id = Self::require_attr(structure, "@id")?;
        let name = Self::get_text_field(structure, "Name").unwrap_or_else(|| id.to_string());

        // Parse dimensions
        let dimensions = Self::parse_dimensions_from_structure(structure)?;

        Ok(DataStructure {
            id: id.to_string(),
            name,
            dimensions,
        })
    }

    /// Parse dimensions from data structure
    fn parse_dimensions_from_structure(structure: &Value) -> ExchangeResult<Vec<Dimension>> {
        let empty_vec = vec![];
        let dims_array = structure
            .pointer("/DataStructureComponents/DimensionList/Dimension")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        dims_array
            .iter()
            .map(|dim| {
                let id = Self::require_attr(dim, "@id")?;
                let position = Self::get_attr(dim, "@position")
                    .and_then(|s| s.parse::<u32>().ok())
                    .unwrap_or(0);

                Ok(Dimension {
                    id: id.to_string(),
                    position,
                    code_list_id: Self::get_nested_attr(dim, "LocalRepresentation/Enumeration/@id"),
                })
            })
            .collect()
    }

    /// Parse code list response
    ///
    /// Contains available codes for a dimension
    pub fn parse_code_list(response: &Value) -> ExchangeResult<CodeList> {
        let code_list = response
            .pointer("/Structure/Structures/Codelists/Codelist/0")
            .or_else(|| response.pointer("/Structure/Structures/Codelists/Codelist"))
            .ok_or_else(|| ExchangeError::Parse("Missing code list".to_string()))?;

        let id = Self::require_attr(code_list, "@id")?;
        let name = Self::get_text_field(code_list, "Name").unwrap_or_else(|| id.to_string());

        let codes = Self::parse_codes(code_list)?;

        Ok(CodeList {
            id: id.to_string(),
            name,
            codes,
        })
    }

    /// Parse codes from code list
    fn parse_codes(code_list: &Value) -> ExchangeResult<Vec<Code>> {
        let empty_vec = vec![];
        let codes_array = code_list
            .get("Code")
            .and_then(|v| v.as_array())
            .unwrap_or(&empty_vec);

        Ok(codes_array
            .iter()
            .filter_map(|code| {
                let id = Self::get_attr(code, "@id")?;
                let name = Self::get_text_field(code, "Name").unwrap_or_else(|| id.to_string());

                Some(Code {
                    id: id.to_string(),
                    name,
                })
            })
            .collect())
    }

    /// Check for API errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // IMF returns errors in various formats
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

        // Check for empty or error structures
        if response.pointer("/CompactData/DataSet").is_none()
            && response.pointer("/Structure").is_none()
            && response.get("error").is_some()
        {
            return Err(ExchangeError::Api {
                code: -1,
                message: "Invalid response structure".to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get attribute with @ prefix
    fn get_attr(value: &Value, key: &str) -> Option<String> {
        value.get(key).and_then(|v| v.as_str()).map(|s| s.to_string())
    }

    /// Require attribute with @ prefix
    fn require_attr(value: &Value, key: &str) -> ExchangeResult<String> {
        Self::get_attr(value, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required attribute: {}", key)))
    }

    /// Get nested attribute using path
    fn get_nested_attr(value: &Value, path: &str) -> Option<String> {
        value.pointer(&format!("/{}", path.replace('.', "/")))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }

    /// Get text field (IMF uses {"#text": "value"} or array of such objects)
    fn get_text_field(value: &Value, key: &str) -> Option<String> {
        value.get(key).and_then(|v| {
            // Handle single object
            if let Some(text) = v.get("#text") {
                return text.as_str().map(|s| s.to_string());
            }
            // Handle array of objects
            if let Some(array) = v.as_array() {
                if let Some(first) = array.first() {
                    if let Some(text) = first.get("#text") {
                        return text.as_str().map(|s| s.to_string());
                    }
                }
            }
            None
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════

/// IMF dataflow (dataset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataflow {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// IMF time series
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImfSeries {
    pub freq: String,         // Frequency (A=Annual, Q=Quarterly, M=Monthly)
    pub ref_area: String,     // Country/region code
    pub indicator: String,    // Indicator code
    pub observations: Vec<ImfObservation>,
}

/// IMF observation (data point)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImfObservation {
    pub time_period: String,  // Time period (e.g., "2020", "2020-Q1")
    pub value: Option<f64>,   // Data value (None if missing)
    pub obs_status: Option<String>, // Observation status
}

/// IMF data structure definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStructure {
    pub id: String,
    pub name: String,
    pub dimensions: Vec<Dimension>,
}

/// Dimension in data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub id: String,
    pub position: u32,
    pub code_list_id: Option<String>,
}

/// Code list (valid values for a dimension)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeList {
    pub id: String,
    pub name: String,
    pub codes: Vec<Code>,
}

/// Code (valid value)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Code {
    pub id: String,
    pub name: String,
}
