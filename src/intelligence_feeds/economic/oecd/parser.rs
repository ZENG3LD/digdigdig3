//! OECD SDMX-JSON response parsers
//!
//! Parse SDMX-JSON format responses to domain types.
//!
//! OECD uses SDMX 2.0 JSON format with nested structure:
//! - dataSets[0].observations contains the actual data
//! - dimensions are encoded in keys like "0:0:0:0"
//! - structure contains metadata about dimensions and attributes

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct OecdParser;

impl OecdParser {
    // ═══════════════════════════════════════════════════════════════════════
    // OECD-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse SDMX-JSON data response
    ///
    /// SDMX structure:
    /// ```json
    /// {
    ///   "dataSets": [{
    ///     "observations": {
    ///       "0:0:0:0": [123.45, null],
    ///       "0:0:1:0": [456.78, null]
    ///     }
    ///   }],
    ///   "structure": {
    ///     "dimensions": {
    ///       "observation": [...]
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_data(response: &Value) -> ExchangeResult<Vec<OecdObservation>> {
        let data_sets = response
            .get("dataSets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataSets' array".to_string()))?;

        let first_dataset = data_sets
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'dataSets' array".to_string()))?;

        let observations = first_dataset
            .get("observations")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'observations' object".to_string()))?;

        let mut results = Vec::new();

        for (key, value_array) in observations.iter() {
            // Value is typically [value, attributes]
            let value = if let Some(arr) = value_array.as_array() {
                arr.first()
                    .and_then(|v| v.as_f64())
            } else {
                value_array.as_f64()
            };

            results.push(OecdObservation {
                key: key.clone(),
                value,
            });
        }

        Ok(results)
    }

    /// Parse dataflow list
    pub fn parse_dataflows(response: &Value) -> ExchangeResult<Vec<OecdDataflow>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let dataflows_obj = data
            .get("dataflows")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataflows' array".to_string()))?;

        let mut results = Vec::new();

        for df in dataflows_obj.iter() {
            let id = Self::get_str(df, "id")
                .ok_or_else(|| ExchangeError::Parse("Missing dataflow 'id'".to_string()))?
                .to_string();

            let name = Self::get_nested_str(df, &["name", "en"])
                .or_else(|| Self::get_nested_str(df, &["name", "0"]))
                .unwrap_or("Unknown")
                .to_string();

            let agency = Self::get_str(df, "agencyID")
                .unwrap_or("OECD")
                .to_string();

            results.push(OecdDataflow {
                id,
                name,
                agency,
                version: Self::get_str(df, "version").map(|s| s.to_string()),
            });
        }

        Ok(results)
    }

    /// Parse single dataflow
    pub fn parse_dataflow(response: &Value) -> ExchangeResult<OecdDataflow> {
        let dataflows = Self::parse_dataflows(response)?;
        dataflows
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::Parse("No dataflow found".to_string()))
    }

    /// Parse datastructure definition
    pub fn parse_datastructure(response: &Value) -> ExchangeResult<OecdDatastructure> {
        let data = response
            .get("data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let structures = data
            .get("dataStructures")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataStructures' array".to_string()))?;

        let structure = structures
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'dataStructures' array".to_string()))?;

        let id = Self::require_str(structure, "id")?.to_string();
        let name = Self::get_nested_str(structure, &["name", "en"])
            .unwrap_or("Unknown")
            .to_string();
        let agency = Self::get_str(structure, "agencyID")
            .unwrap_or("OECD")
            .to_string();

        Ok(OecdDatastructure {
            id,
            name,
            agency,
            version: Self::get_str(structure, "version").map(|s| s.to_string()),
        })
    }

    /// Parse codelist
    pub fn parse_codelist(response: &Value) -> ExchangeResult<OecdCodelist> {
        let data = response
            .get("data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let codelists = data
            .get("codelists")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'codelists' array".to_string()))?;

        let codelist = codelists
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'codelists' array".to_string()))?;

        let id = Self::require_str(codelist, "id")?.to_string();
        let name = Self::get_nested_str(codelist, &["name", "en"])
            .unwrap_or("Unknown")
            .to_string();
        let agency = Self::get_str(codelist, "agencyID")
            .unwrap_or("OECD")
            .to_string();

        let mut codes = Vec::new();
        if let Some(codes_array) = codelist.get("codes").and_then(|v| v.as_array()) {
            for code_obj in codes_array.iter() {
                let code_id = Self::get_str(code_obj, "id").unwrap_or("").to_string();
                let code_name = Self::get_nested_str(code_obj, &["name", "en"])
                    .unwrap_or("")
                    .to_string();

                codes.push(OecdCode {
                    id: code_id,
                    name: code_name,
                });
            }
        }

        Ok(OecdCodelist {
            id,
            name,
            agency,
            codes,
        })
    }

    /// Parse availability constraints
    pub fn parse_availability(response: &Value) -> ExchangeResult<OecdAvailability> {
        // SDMX availability response contains constraint information
        // This is a simplified parser - actual structure is complex
        let available = response
            .get("data")
            .is_some();

        Ok(OecdAvailability {
            available,
            constraints: response.clone(),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error")
                .to_string();

            let code = error
                .get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            return Err(ExchangeError::Api { code, message });
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_nested_str<'a>(obj: &'a Value, path: &[&str]) -> Option<&'a str> {
        let mut current = obj;
        for &key in path {
            current = current.get(key)?;
        }
        current.as_str()
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OECD-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// OECD observation (single data point)
#[derive(Debug, Clone)]
pub struct OecdObservation {
    /// Dimension key (e.g., "0:0:0:0")
    pub key: String,
    /// Observation value
    pub value: Option<f64>,
}

/// OECD dataflow metadata
#[derive(Debug, Clone)]
pub struct OecdDataflow {
    pub id: String,
    pub name: String,
    pub agency: String,
    pub version: Option<String>,
}

/// OECD datastructure definition
#[derive(Debug, Clone)]
pub struct OecdDatastructure {
    pub id: String,
    pub name: String,
    pub agency: String,
    pub version: Option<String>,
}

/// OECD codelist
#[derive(Debug, Clone)]
pub struct OecdCodelist {
    pub id: String,
    pub name: String,
    pub agency: String,
    pub codes: Vec<OecdCode>,
}

/// OECD code (item in a codelist)
#[derive(Debug, Clone)]
pub struct OecdCode {
    pub id: String,
    pub name: String,
}

/// OECD data availability information
#[derive(Debug, Clone)]
pub struct OecdAvailability {
    pub available: bool,
    pub constraints: Value,
}
