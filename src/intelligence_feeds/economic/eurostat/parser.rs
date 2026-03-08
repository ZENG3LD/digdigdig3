//! Eurostat response parsers
//!
//! Parse JSON responses to domain types based on Eurostat API response formats.
//!
//! Eurostat uses JSON-stat format v2 for data responses and SDMX-JSON for metadata.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult, Kline};
use std::collections::HashMap;

pub struct EurostatParser;

impl EurostatParser {
    // ═══════════════════════════════════════════════════════════════════════
    // EUROSTAT-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse dataset in JSON-stat v2 format
    ///
    /// Example JSON-stat v2 response:
    /// ```json
    /// {
    ///   "version": "2.0",
    ///   "class": "dataset",
    ///   "label": "GDP and main components",
    ///   "id": ["geo", "time"],
    ///   "size": [2, 10],
    ///   "dimension": {
    ///     "geo": {
    ///       "label": "Geopolitical entity",
    ///       "category": {
    ///         "index": {"DE": 0, "FR": 1},
    ///         "label": {"DE": "Germany", "FR": "France"}
    ///       }
    ///     },
    ///     "time": {
    ///       "label": "Time",
    ///       "category": {
    ///         "index": {"2020": 0, "2021": 1, ...}
    ///       }
    ///     }
    ///   },
    ///   "value": [123.4, 234.5, ...],
    ///   "status": ["", "", ...]
    /// }
    /// ```
    pub fn parse_dataset(response: &Value) -> ExchangeResult<EurostatDataset> {
        let version = Self::get_str(response, "version")
            .unwrap_or("unknown")
            .to_string();

        let label = Self::get_str(response, "label")
            .unwrap_or("")
            .to_string();

        let id = response
            .get("id")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        let size = response
            .get("size")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_u64())
                    .map(|n| n as usize)
                    .collect()
            })
            .unwrap_or_default();

        let values = response
            .get("value")
            .and_then(|v| {
                if v.is_array() {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .map(|val| {
                                if val.is_null() {
                                    None
                                } else {
                                    val.as_f64()
                                }
                            })
                            .collect()
                    })
                } else if v.is_object() {
                    // JSON-stat can also use object format: {"0": 123.4, "1": 234.5}
                    v.as_object().map(|obj| {
                        let mut vec = vec![None; obj.len()];
                        for (key, val) in obj.iter() {
                            if let Ok(idx) = key.parse::<usize>() {
                                if idx < vec.len() {
                                    vec[idx] = if val.is_null() {
                                        None
                                    } else {
                                        val.as_f64()
                                    };
                                }
                            }
                        }
                        vec
                    })
                } else {
                    None
                }
            })
            .unwrap_or_default();

        let dimensions = Self::parse_dimensions(response)?;

        Ok(EurostatDataset {
            version,
            label,
            id,
            size,
            dimensions,
            values,
        })
    }

    fn parse_dimensions(response: &Value) -> ExchangeResult<HashMap<String, EurostatDimension>> {
        let dim_obj = response
            .get("dimension")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'dimension' object".to_string()))?;

        let mut dimensions = HashMap::new();

        for (dim_name, dim_value) in dim_obj.iter() {
            let label = Self::get_str(dim_value, "label")
                .unwrap_or("")
                .to_string();

            let category = dim_value.get("category");

            let mut index = HashMap::new();
            let mut labels = HashMap::new();

            if let Some(cat) = category {
                if let Some(idx_obj) = cat.get("index").and_then(|v| v.as_object()) {
                    for (code, pos) in idx_obj.iter() {
                        if let Some(position) = pos.as_u64() {
                            index.insert(code.clone(), position as usize);
                        }
                    }
                }

                if let Some(label_obj) = cat.get("label").and_then(|v| v.as_object()) {
                    for (code, lbl) in label_obj.iter() {
                        if let Some(label_str) = lbl.as_str() {
                            labels.insert(code.clone(), label_str.to_string());
                        }
                    }
                }
            }

            dimensions.insert(
                dim_name.clone(),
                EurostatDimension {
                    label,
                    index,
                    labels,
                },
            );
        }

        Ok(dimensions)
    }

    /// Parse dataset label metadata
    pub fn parse_label(response: &Value) -> ExchangeResult<EurostatLabel> {
        let label = response
            .get("label")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing 'label' field".to_string()))?
            .to_string();

        Ok(EurostatLabel { label })
    }

    /// Parse SDMX dataflows list
    pub fn parse_dataflows(response: &Value) -> ExchangeResult<Vec<EurostatDataflow>> {
        // SDMX-JSON format varies, try to extract dataflow info
        let mut dataflows = Vec::new();

        // Try structure: data.dataflows[...]
        if let Some(data) = response.get("data") {
            if let Some(flows) = data.get("dataflows").and_then(|v| v.as_array()) {
                for flow in flows.iter() {
                    if let Ok(df) = Self::parse_single_dataflow(flow) {
                        dataflows.push(df);
                    }
                }
            }
        }

        // Try alternative structure
        if dataflows.is_empty() {
            if let Some(flows) = response.get("dataflows").and_then(|v| v.as_array()) {
                for flow in flows.iter() {
                    if let Ok(df) = Self::parse_single_dataflow(flow) {
                        dataflows.push(df);
                    }
                }
            }
        }

        Ok(dataflows)
    }

    fn parse_single_dataflow(flow: &Value) -> ExchangeResult<EurostatDataflow> {
        let id = Self::get_str(flow, "id")
            .or_else(|| Self::get_str(flow, "agencyID"))
            .unwrap_or("unknown")
            .to_string();

        let name = Self::get_str(flow, "name")
            .or_else(|| Self::get_str(flow, "label"))
            .unwrap_or("")
            .to_string();

        Ok(EurostatDataflow { id, name })
    }

    /// Parse table of contents
    pub fn parse_toc(response: &Value) -> ExchangeResult<Vec<EurostatTocEntry>> {
        let mut entries = Vec::new();

        // TOC can be array or nested structure
        if let Some(arr) = response.as_array() {
            for item in arr.iter() {
                if let Ok(entry) = Self::parse_toc_entry(item) {
                    entries.push(entry);
                }
            }
        } else if let Some(toc) = response.get("toc").and_then(|v| v.as_array()) {
            for item in toc.iter() {
                if let Ok(entry) = Self::parse_toc_entry(item) {
                    entries.push(entry);
                }
            }
        }

        Ok(entries)
    }

    fn parse_toc_entry(entry: &Value) -> ExchangeResult<EurostatTocEntry> {
        let code = Self::get_str(entry, "code")
            .unwrap_or("")
            .to_string();

        let title = Self::get_str(entry, "title")
            .or_else(|| Self::get_str(entry, "label"))
            .unwrap_or("")
            .to_string();

        let entry_type = Self::get_str(entry, "type")
            .unwrap_or("dataset")
            .to_string();

        Ok(EurostatTocEntry {
            code,
            title,
            entry_type,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ADAPTER: Convert Eurostat data to Klines
    // ═══════════════════════════════════════════════════════════════════════

    /// Convert Eurostat time series data to Kline format
    ///
    /// Extracts time dimension and converts values to klines
    pub fn dataset_to_klines(dataset: EurostatDataset) -> ExchangeResult<Vec<Kline>> {
        // Find time dimension
        let time_dim_name = dataset
            .id
            .iter()
            .find(|dim| dim.to_lowercase().contains("time") || dim.to_lowercase() == "period")
            .ok_or_else(|| ExchangeError::Parse("No time dimension found".to_string()))?;

        let time_dim = dataset
            .dimensions
            .get(time_dim_name)
            .ok_or_else(|| ExchangeError::Parse("Time dimension not found in dimensions".to_string()))?;

        // Get time dimension size
        let time_idx = dataset
            .id
            .iter()
            .position(|d| d == time_dim_name)
            .ok_or_else(|| ExchangeError::Parse("Time dimension position not found".to_string()))?;

        let _time_size = dataset
            .size
            .get(time_idx)
            .copied()
            .ok_or_else(|| ExchangeError::Parse("Time dimension size not found".to_string()))?;

        // Extract time values in order
        let mut time_values: Vec<(usize, String)> = time_dim
            .index
            .iter()
            .map(|(time_str, idx)| (*idx, time_str.clone()))
            .collect();
        time_values.sort_by_key(|(idx, _)| *idx);

        // Calculate stride for multi-dimensional data
        let stride: usize = dataset.size.iter().skip(time_idx + 1).product();

        let mut klines = Vec::new();

        for (time_position, time_str) in time_values.iter() {
            // Calculate value index for this time position
            let value_idx = time_position * stride;

            if value_idx < dataset.values.len() {
                if let Some(value) = dataset.values[value_idx] {
                    let timestamp = Self::parse_eurostat_time(time_str)?;

                    klines.push(Kline {
                        open_time: timestamp,
                        open: value,
                        high: value,
                        low: value,
                        close: value,
                        volume: 0.0,
                        close_time: Some(timestamp),
                        quote_volume: None,
                        trades: None,
                    });
                }
            }
        }

        Ok(klines)
    }

    /// Parse Eurostat time format to Unix timestamp (milliseconds)
    ///
    /// Eurostat uses formats like:
    /// - "2024" (year)
    /// - "2024-Q1" (quarter)
    /// - "2024-M01" (month)
    /// - "2024-W01" (week)
    fn parse_eurostat_time(time_str: &str) -> ExchangeResult<i64> {
        // Year only: "2024"
        if time_str.len() == 4 {
            if let Ok(year) = time_str.parse::<i64>() {
                return Ok((year - 1970) * 365 * 24 * 60 * 60 * 1000);
            }
        }

        // Year-Quarter: "2024-Q1"
        if time_str.contains("-Q") {
            let parts: Vec<&str> = time_str.split("-Q").collect();
            if parts.len() == 2 {
                if let (Ok(year), Ok(quarter)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                    let month = (quarter - 1) * 3 + 1;
                    let days = (year - 1970) * 365 + (month - 1) * 30;
                    return Ok(days * 24 * 60 * 60 * 1000);
                }
            }
        }

        // Year-Month: "2024-M01"
        if time_str.contains("-M") {
            let parts: Vec<&str> = time_str.split("-M").collect();
            if parts.len() == 2 {
                if let (Ok(year), Ok(month)) = (parts[0].parse::<i64>(), parts[1].parse::<i64>()) {
                    let days = (year - 1970) * 365 + (month - 1) * 30;
                    return Ok(days * 24 * 60 * 60 * 1000);
                }
            }
        }

        // ISO date: "2024-01-15"
        if time_str.contains('-') && time_str.len() >= 10 {
            let parts: Vec<&str> = time_str.split('-').collect();
            if parts.len() >= 3 {
                if let (Ok(year), Ok(month), Ok(day)) = (
                    parts[0].parse::<i64>(),
                    parts[1].parse::<i64>(),
                    parts[2].parse::<i64>(),
                ) {
                    let days = (year - 1970) * 365 + (month - 1) * 30 + day;
                    return Ok(days * 24 * 60 * 60 * 1000);
                }
            }
        }

        Err(ExchangeError::Parse(format!("Unknown time format: {}", time_str)))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .or_else(|| error.get("message").and_then(|v| v.as_str()))
                .unwrap_or("Unknown error")
                .to_string();

            let code = error
                .get("code")
                .and_then(|v| v.as_i64())
                .unwrap_or(0) as i32;

            return Err(ExchangeError::Api { code, message });
        }

        // Check for SDMX error structure
        if let Some(error_msg) = response.get("errorMessage").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: 0,
                message: error_msg.to_string(),
            });
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// EUROSTAT-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Eurostat dataset in JSON-stat v2 format
#[derive(Debug, Clone)]
pub struct EurostatDataset {
    pub version: String,
    pub label: String,
    pub id: Vec<String>,        // Dimension names
    pub size: Vec<usize>,       // Dimension sizes
    pub dimensions: HashMap<String, EurostatDimension>,
    pub values: Vec<Option<f64>>, // Flat array of values
}

/// Eurostat dimension metadata
#[derive(Debug, Clone)]
pub struct EurostatDimension {
    pub label: String,
    pub index: HashMap<String, usize>,        // code -> position
    pub labels: HashMap<String, String>,      // code -> label
}

/// Dataset label metadata
#[derive(Debug, Clone)]
pub struct EurostatLabel {
    pub label: String,
}

/// SDMX dataflow
#[derive(Debug, Clone)]
pub struct EurostatDataflow {
    pub id: String,
    pub name: String,
}

/// Table of contents entry
#[derive(Debug, Clone)]
pub struct EurostatTocEntry {
    pub code: String,
    pub title: String,
    pub entry_type: String,
}
