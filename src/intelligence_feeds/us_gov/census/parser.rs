//! Census API response parser

use crate::core::types::{ExchangeError, ExchangeResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ═══════════════════════════════════════════════════════════════════════════
// CENSUS DATA TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Generic Census data row
///
/// Census returns data as array of arrays, where first row is header.
/// This structure represents a parsed row with column name -> value mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CensusDataRow {
    /// Column name -> value mapping
    pub data: HashMap<String, String>,
}

impl CensusDataRow {
    /// Get value for a column
    pub fn get(&self, column: &str) -> Option<&str> {
        self.data.get(column).map(|s| s.as_str())
    }

    /// Get numeric value for a column
    pub fn get_numeric(&self, column: &str) -> Option<f64> {
        self.get(column)?.parse::<f64>().ok()
    }
}

/// Economic Indicator time series observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicIndicatorObservation {
    /// Cell value (the actual data point)
    pub cell_value: String,

    /// Data type code
    pub data_type_code: String,

    /// Time slot ID (e.g., "2024-01", "2024-Q1")
    pub time_slot_id: String,

    /// Error data (if any)
    pub error_data: Option<String>,

    /// Category code (e.g., "TOTAL")
    pub category_code: Option<String>,
}

impl EconomicIndicatorObservation {
    /// Get value as f64
    pub fn value(&self) -> Option<f64> {
        self.cell_value.parse::<f64>().ok()
    }

    /// Check if observation has error
    pub fn has_error(&self) -> bool {
        self.error_data.is_some() && !self.error_data.as_ref().unwrap().is_empty()
    }
}

/// Dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatasetInfo {
    /// Dataset identifier (e.g., "acs/acs1")
    pub identifier: String,

    /// Title
    pub title: String,

    /// Description
    pub description: String,

    /// Vintage/year
    pub vintage: Option<String>,

    /// Contact information
    pub contact: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════
// PARSER IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════════

pub struct CensusParser;

impl CensusParser {
    /// Check for API errors in response
    pub fn check_error(json: &serde_json::Value) -> ExchangeResult<()> {
        // Census API returns array on success, object with "error" on failure
        if let Some(error_msg) = json.get("error").and_then(|e| e.as_str()) {
            return Err(ExchangeError::Api {
                code: -1,
                message: error_msg.to_string(),
            });
        }

        Ok(())
    }

    /// Parse generic Census dataset response
    ///
    /// Census returns: [["col1","col2"],["val1","val2"],["val3","val4"]]
    pub fn parse_dataset(json: &serde_json::Value) -> ExchangeResult<Vec<CensusDataRow>> {
        let array = json
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        if array.is_empty() {
            return Ok(Vec::new());
        }

        // First row is header
        let header_row = &array[0];
        let headers: Vec<String> = header_row
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Header row is not an array".to_string()))?
            .iter()
            .map(|v| v.as_str().unwrap_or("").to_string())
            .collect();

        // Parse data rows
        let mut rows = Vec::new();
        for row_value in array.iter().skip(1) {
            let row_array = row_value
                .as_array()
                .ok_or_else(|| ExchangeError::Parse("Data row is not an array".to_string()))?;

            let mut data = HashMap::new();
            for (i, value) in row_array.iter().enumerate() {
                if i < headers.len() {
                    let key = headers[i].clone();
                    let val = value.as_str().unwrap_or("").to_string();
                    data.insert(key, val);
                }
            }

            rows.push(CensusDataRow { data });
        }

        Ok(rows)
    }

    /// Parse economic indicator time series response
    ///
    /// Expected format:
    /// [["cell_value","data_type_code","time_slot_id","error_data"],
    ///  ["12345.6","TOTAL","2024-01",""]]
    pub fn parse_economic_indicator(
        json: &serde_json::Value,
    ) -> ExchangeResult<Vec<EconomicIndicatorObservation>> {
        let rows = Self::parse_dataset(json)?;

        let mut observations = Vec::new();
        for row in rows {
            let obs = EconomicIndicatorObservation {
                cell_value: row.get("cell_value").unwrap_or("").to_string(),
                data_type_code: row.get("data_type_code").unwrap_or("").to_string(),
                time_slot_id: row.get("time_slot_id").unwrap_or("").to_string(),
                error_data: row.get("error_data").map(|s| s.to_string()),
                category_code: row.get("category_code").map(|s| s.to_string()),
            };
            observations.push(obs);
        }

        Ok(observations)
    }

    /// Parse dataset list response
    ///
    /// Returns raw JSON value containing dataset metadata
    pub fn parse_dataset_list(json: &serde_json::Value) -> ExchangeResult<Vec<DatasetInfo>> {
        // The list endpoint returns a complex JSON structure
        // We'll return a simplified version for now
        let datasets_obj = json
            .get("dataset")
            .ok_or_else(|| ExchangeError::Parse("No 'dataset' field in response".to_string()))?;

        // If it's an array of datasets
        if let Some(array) = datasets_obj.as_array() {
            let mut datasets = Vec::new();
            for item in array {
                if let Some(dataset) = Self::parse_single_dataset(item) {
                    datasets.push(dataset);
                }
            }
            return Ok(datasets);
        }

        // If it's a single dataset
        if let Some(dataset) = Self::parse_single_dataset(datasets_obj) {
            return Ok(vec![dataset]);
        }

        Ok(Vec::new())
    }

    /// Parse single dataset metadata
    fn parse_single_dataset(json: &serde_json::Value) -> Option<DatasetInfo> {
        Some(DatasetInfo {
            identifier: json.get("identifier")?.as_str()?.to_string(),
            title: json.get("title")?.as_str()?.to_string(),
            description: json.get("description")?.as_str().unwrap_or("").to_string(),
            vintage: json.get("vintage").and_then(|v| v.as_str()).map(|s| s.to_string()),
            contact: json.get("contact").and_then(|v| v.as_str()).map(|s| s.to_string()),
        })
    }
}
