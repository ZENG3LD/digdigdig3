//! BIS response parsers
//!
//! Parse SDMX-JSON responses to domain types based on BIS API response formats.
//!
//! BIS is an economic data provider, not a trading exchange, so many standard
//! market data types (ticker, orderbook) don't apply directly.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct BisParser;

impl BisParser {
    // ═══════════════════════════════════════════════════════════════════════
    // BIS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse SDMX data observations
    ///
    /// BIS uses SDMX-JSON format:
    /// ```json
    /// {
    ///   "data": {
    ///     "dataSets": [{
    ///       "observations": {
    ///         "0:0:0:0": [1.5, null],
    ///         "0:0:0:1": [1.6, null]
    ///       }
    ///     }],
    ///     "structure": {
    ///       "dimensions": { ... },
    ///       "attributes": { ... }
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_data_observations(response: &Value) -> ExchangeResult<Vec<SdmxObservation>> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let datasets = data
            .get("dataSets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataSets' array".to_string()))?;

        let dataset = datasets
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'dataSets' array".to_string()))?;

        let observations_obj = dataset
            .get("observations")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'observations' object".to_string()))?;

        // Parse dimension structure if available
        let structure = data.get("structure");
        let _time_dimension = Self::extract_time_dimension(structure);

        let mut observations = Vec::new();
        for (key, value_array) in observations_obj.iter() {
            let value = if let Some(arr) = value_array.as_array() {
                arr.first()
                    .and_then(|v| v.as_f64())
            } else {
                None
            };

            observations.push(SdmxObservation {
                key: key.clone(),
                value,
                time_period: None, // Will be populated from structure if available
                attributes: None,
            });
        }

        Ok(observations)
    }

    /// Extract time dimension values from structure
    fn extract_time_dimension(structure: Option<&Value>) -> Option<Vec<String>> {
        structure
            .and_then(|s| s.get("dimensions"))
            .and_then(|d| d.get("observation"))
            .and_then(|obs| obs.as_array())
            .and_then(|arr| {
                arr.iter()
                    .find(|dim| {
                        dim.get("id")
                            .and_then(|id| id.as_str())
                            .map(|id| id == "TIME_PERIOD" || id == "TIME")
                            .unwrap_or(false)
                    })
            })
            .and_then(|time_dim| time_dim.get("values"))
            .and_then(|values| values.as_array())
            .map(|values| {
                values
                    .iter()
                    .filter_map(|v| v.get("id").and_then(|id| id.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
    }

    /// Parse dataflows list
    pub fn parse_dataflows(response: &Value) -> ExchangeResult<Vec<SdmxDataflow>> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let dataflows = data
            .get("dataflows")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataflows' array".to_string()))?;

        dataflows
            .iter()
            .map(|df| {
                Ok(SdmxDataflow {
                    id: Self::require_str(df, "id")?.to_string(),
                    name: Self::get_str(df, "name").map(|s| s.to_string()),
                    description: Self::get_str(df, "description").map(|s| s.to_string()),
                    version: Self::get_str(df, "version").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single dataflow metadata
    pub fn parse_dataflow(response: &Value) -> ExchangeResult<SdmxDataflow> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let dataflows = data
            .get("dataflows")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataflows' array".to_string()))?;

        let df = dataflows
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'dataflows' array".to_string()))?;

        Ok(SdmxDataflow {
            id: Self::require_str(df, "id")?.to_string(),
            name: Self::get_str(df, "name").map(|s| s.to_string()),
            description: Self::get_str(df, "description").map(|s| s.to_string()),
            version: Self::get_str(df, "version").map(|s| s.to_string()),
        })
    }

    /// Parse data structure definition
    pub fn parse_data_structure(response: &Value) -> ExchangeResult<SdmxDataStructure> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let structures = data
            .get("dataStructures")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dataStructures' array".to_string()))?;

        let structure = structures
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'dataStructures' array".to_string()))?;

        Ok(SdmxDataStructure {
            id: Self::require_str(structure, "id")?.to_string(),
            name: Self::get_str(structure, "name").map(|s| s.to_string()),
            dimensions: Self::get_i64(structure, "dimensions"),
            attributes: Self::get_i64(structure, "attributes"),
        })
    }

    /// Parse codelist
    pub fn parse_codelist(response: &Value) -> ExchangeResult<SdmxCodelist> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let codelists = data
            .get("codelists")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'codelists' array".to_string()))?;

        let codelist = codelists
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'codelists' array".to_string()))?;

        let codes = codelist
            .get("codes")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|code| {
                        let id = Self::get_str(code, "id")?;
                        let name = Self::get_str(code, "name");
                        Some(SdmxCode {
                            id: id.to_string(),
                            name: name.map(|s| s.to_string()),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        Ok(SdmxCodelist {
            id: Self::require_str(codelist, "id")?.to_string(),
            name: Self::get_str(codelist, "name").map(|s| s.to_string()),
            codes,
        })
    }

    /// Parse concept scheme
    pub fn parse_concept_scheme(response: &Value) -> ExchangeResult<SdmxConceptScheme> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        let schemes = data
            .get("conceptSchemes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'conceptSchemes' array".to_string()))?;

        let scheme = schemes
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty 'conceptSchemes' array".to_string()))?;

        Ok(SdmxConceptScheme {
            id: Self::require_str(scheme, "id")?.to_string(),
            name: Self::get_str(scheme, "name").map(|s| s.to_string()),
        })
    }

    /// Parse availability information
    pub fn parse_availability(response: &Value) -> ExchangeResult<SdmxAvailability> {
        let data = response
            .get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' object".to_string()))?;

        // Availability response structure varies, extract what we can
        Ok(SdmxAvailability {
            available: true, // If we got a response, data is available
            start_period: Self::get_str(data, "startPeriod").map(|s| s.to_string()),
            end_period: Self::get_str(data, "endPeriod").map(|s| s.to_string()),
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // SDMX errors are typically HTTP-level, but check for error objects
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown SDMX error")
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

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BIS-SPECIFIC TYPES (SDMX format)
// ═══════════════════════════════════════════════════════════════════════════

/// SDMX observation (single data point)
#[derive(Debug, Clone)]
pub struct SdmxObservation {
    pub key: String,
    pub value: Option<f64>,
    pub time_period: Option<String>,
    pub attributes: Option<serde_json::Value>,
}

/// SDMX dataflow
#[derive(Debug, Clone)]
pub struct SdmxDataflow {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub version: Option<String>,
}

/// SDMX data structure definition
#[derive(Debug, Clone)]
pub struct SdmxDataStructure {
    pub id: String,
    pub name: Option<String>,
    pub dimensions: Option<i64>,
    pub attributes: Option<i64>,
}

/// SDMX codelist
#[derive(Debug, Clone)]
pub struct SdmxCodelist {
    pub id: String,
    pub name: Option<String>,
    pub codes: Vec<SdmxCode>,
}

/// SDMX code (single code in a codelist)
#[derive(Debug, Clone)]
pub struct SdmxCode {
    pub id: String,
    pub name: Option<String>,
}

/// SDMX concept scheme
#[derive(Debug, Clone)]
pub struct SdmxConceptScheme {
    pub id: String,
    pub name: Option<String>,
}

/// SDMX data availability
#[derive(Debug, Clone)]
pub struct SdmxAvailability {
    pub available: bool,
    pub start_period: Option<String>,
    pub end_period: Option<String>,
}
