//! BEA API response parser

use serde::{Deserialize, Serialize};
use crate::core::types::{ExchangeError, ExchangeResult};

/// BEA API response parser
pub struct BeaParser;

impl BeaParser {
    /// Check for API errors in response
    pub fn check_error(json: &serde_json::Value) -> ExchangeResult<()> {
        // BEA wraps everything in BEAAPI object
        if let Some(beaapi) = json.get("BEAAPI") {
            // Check for error in Results
            if let Some(results) = beaapi.get("Results") {
                if let Some(error) = results.get("Error") {
                    let code = error.get("APIErrorCode")
                        .and_then(|v| v.as_str())
                        .unwrap_or("UNKNOWN");
                    let message = error.get("APIErrorDescription")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");

                    return Err(ExchangeError::Api {
                        code: code.parse().unwrap_or(0),
                        message: message.to_string(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Parse dataset list response
    pub fn parse_dataset_list(json: &serde_json::Value) -> ExchangeResult<Vec<BeaDataset>> {
        let datasets = json
            .pointer("/BEAAPI/Results/Dataset")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing Dataset array".to_string()))?;

        datasets
            .iter()
            .map(|d| {
                Ok(BeaDataset {
                    name: d.get("DatasetName")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExchangeError::Parse("Missing DatasetName".to_string()))?
                        .to_string(),
                    description: d.get("DatasetDescription")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExchangeError::Parse("Missing DatasetDescription".to_string()))?
                        .to_string(),
                })
            })
            .collect()
    }

    /// Parse parameter list response
    pub fn parse_parameter_list(json: &serde_json::Value) -> ExchangeResult<Vec<BeaParameter>> {
        let params = json
            .pointer("/BEAAPI/Results/Parameter")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing Parameter array".to_string()))?;

        params
            .iter()
            .map(|p| {
                Ok(BeaParameter {
                    name: p.get("ParameterName")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExchangeError::Parse("Missing ParameterName".to_string()))?
                        .to_string(),
                    description: p.get("ParameterDescription")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                    required: p.get("ParameterIsRequiredFlag")
                        .and_then(|v| v.as_str())
                        .map(|s| s == "1")
                        .unwrap_or(false),
                    default_value: p.get("ParameterDefaultValue")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse parameter values response
    pub fn parse_parameter_values(json: &serde_json::Value) -> ExchangeResult<Vec<BeaParameterValue>> {
        let values = json
            .pointer("/BEAAPI/Results/ParamValue")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing ParamValue array".to_string()))?;

        values
            .iter()
            .map(|v| {
                Ok(BeaParameterValue {
                    key: v.get("Key")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExchangeError::Parse("Missing Key".to_string()))?
                        .to_string(),
                    description: v.get("Desc")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string(),
                })
            })
            .collect()
    }

    /// Parse data response
    pub fn parse_data(json: &serde_json::Value) -> ExchangeResult<Vec<BeaDataPoint>> {
        let data = json
            .pointer("/BEAAPI/Results/Data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing Data array".to_string()))?;

        data
            .iter()
            .map(|d| {
                // Extract all fields as a generic map
                let mut fields = std::collections::HashMap::new();

                if let Some(obj) = d.as_object() {
                    for (key, value) in obj {
                        if let Some(str_val) = value.as_str() {
                            fields.insert(key.clone(), str_val.to_string());
                        }
                    }
                }

                Ok(BeaDataPoint {
                    table_name: d.get("TableName")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    line_number: d.get("LineNumber")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    line_description: d.get("LineDescription")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string()),
                    time_period: d.get("TimePeriod")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExchangeError::Parse("Missing TimePeriod".to_string()))?
                        .to_string(),
                    data_value: d.get("DataValue")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| ExchangeError::Parse("Missing DataValue".to_string()))?
                        .to_string(),
                    fields,
                })
            })
            .collect()
    }
}

/// BEA dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaDataset {
    /// Dataset name (e.g., "NIPA", "GDPbyIndustry")
    pub name: String,
    /// Human-readable description
    pub description: String,
}

/// BEA parameter metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaParameter {
    /// Parameter name
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this parameter is required
    pub required: bool,
    /// Default value if any
    pub default_value: Option<String>,
}

/// BEA parameter value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaParameterValue {
    /// Value key
    pub key: String,
    /// Human-readable description
    pub description: String,
}

/// BEA data point
///
/// Generic structure that can hold any BEA data response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeaDataPoint {
    /// Table name (NIPA tables, etc.)
    pub table_name: Option<String>,
    /// Line number within table
    pub line_number: Option<String>,
    /// Description of the line
    pub line_description: Option<String>,
    /// Time period (e.g., "2024Q3", "2024")
    pub time_period: String,
    /// Data value (often formatted with commas)
    pub data_value: String,
    /// All other fields from the response
    pub fields: std::collections::HashMap<String, String>,
}
