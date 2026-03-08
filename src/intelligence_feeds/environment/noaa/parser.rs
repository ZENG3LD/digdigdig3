//! NOAA CDO response parsers
//!
//! Parse JSON responses to domain types based on NOAA CDO API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct NoaaParser;

impl NoaaParser {
    // ═══════════════════════════════════════════════════════════════════════
    // UTILITY METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for API errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(status) = response.get("status").and_then(|v| v.as_u64()) {
            if status >= 400 {
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");

                return Err(ExchangeError::Api {
                    code: status as i32,
                    message: message.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Get string field or return error
    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", field)))
    }

    /// Get optional string field
    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    /// Get optional number field
    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // NOAA-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse climate data observations
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "metadata": {
    ///     "resultset": {
    ///       "offset": 1,
    ///       "count": 25,
    ///       "limit": 25
    ///     }
    ///   },
    ///   "results": [
    ///     {
    ///       "date": "2024-01-01T00:00:00",
    ///       "datatype": "TMAX",
    ///       "station": "GHCND:USW00094728",
    ///       "attributes": ",,N,",
    ///       "value": 56
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_data(response: &Value) -> ExchangeResult<Vec<ClimateData>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                let date = Self::require_str(item, "date")?;
                let datatype = Self::require_str(item, "datatype")?;
                let station = Self::require_str(item, "station")?;
                let value = Self::get_f64(item, "value")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'value' field".to_string()))?;

                Ok(ClimateData {
                    date: date.to_string(),
                    datatype: datatype.to_string(),
                    station: station.to_string(),
                    value,
                    attributes: Self::get_str(item, "attributes").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse dataset list
    pub fn parse_datasets(response: &Value) -> ExchangeResult<Vec<Dataset>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                let id = Self::require_str(item, "id")?;
                let name = Self::require_str(item, "name")?;

                Ok(Dataset {
                    id: id.to_string(),
                    name: name.to_string(),
                    datacoverage: Self::get_f64(item, "datacoverage"),
                    mindate: Self::get_str(item, "mindate").map(|s| s.to_string()),
                    maxdate: Self::get_str(item, "maxdate").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse single dataset
    pub fn parse_dataset(response: &Value) -> ExchangeResult<Dataset> {
        let id = Self::require_str(response, "id")?;
        let name = Self::require_str(response, "name")?;

        Ok(Dataset {
            id: id.to_string(),
            name: name.to_string(),
            datacoverage: Self::get_f64(response, "datacoverage"),
            mindate: Self::get_str(response, "mindate").map(|s| s.to_string()),
            maxdate: Self::get_str(response, "maxdate").map(|s| s.to_string()),
        })
    }

    /// Parse datatype list
    pub fn parse_datatypes(response: &Value) -> ExchangeResult<Vec<Datatype>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                let id = Self::require_str(item, "id")?;

                Ok(Datatype {
                    id: id.to_string(),
                    name: Self::get_str(item, "name").map(|s| s.to_string()),
                    mindate: Self::get_str(item, "mindate").map(|s| s.to_string()),
                    maxdate: Self::get_str(item, "maxdate").map(|s| s.to_string()),
                    datacoverage: Self::get_f64(item, "datacoverage"),
                })
            })
            .collect()
    }

    /// Parse single datatype
    pub fn parse_datatype(response: &Value) -> ExchangeResult<Datatype> {
        let id = Self::require_str(response, "id")?;

        Ok(Datatype {
            id: id.to_string(),
            name: Self::get_str(response, "name").map(|s| s.to_string()),
            mindate: Self::get_str(response, "mindate").map(|s| s.to_string()),
            maxdate: Self::get_str(response, "maxdate").map(|s| s.to_string()),
            datacoverage: Self::get_f64(response, "datacoverage"),
        })
    }

    /// Parse location category list
    pub fn parse_location_categories(response: &Value) -> ExchangeResult<Vec<LocationCategory>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                let id = Self::require_str(item, "id")?;
                let name = Self::require_str(item, "name")?;

                Ok(LocationCategory {
                    id: id.to_string(),
                    name: name.to_string(),
                })
            })
            .collect()
    }

    /// Parse location list
    pub fn parse_locations(response: &Value) -> ExchangeResult<Vec<Location>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                let id = Self::require_str(item, "id")?;
                let name = Self::require_str(item, "name")?;

                Ok(Location {
                    id: id.to_string(),
                    name: name.to_string(),
                    mindate: Self::get_str(item, "mindate").map(|s| s.to_string()),
                    maxdate: Self::get_str(item, "maxdate").map(|s| s.to_string()),
                    datacoverage: Self::get_f64(item, "datacoverage"),
                })
            })
            .collect()
    }

    /// Parse single location
    pub fn parse_location(response: &Value) -> ExchangeResult<Location> {
        let id = Self::require_str(response, "id")?;
        let name = Self::require_str(response, "name")?;

        Ok(Location {
            id: id.to_string(),
            name: name.to_string(),
            mindate: Self::get_str(response, "mindate").map(|s| s.to_string()),
            maxdate: Self::get_str(response, "maxdate").map(|s| s.to_string()),
            datacoverage: Self::get_f64(response, "datacoverage"),
        })
    }

    /// Parse station list
    pub fn parse_stations(response: &Value) -> ExchangeResult<Vec<Station>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|item| {
                let id = Self::require_str(item, "id")?;
                let name = Self::require_str(item, "name")?;

                Ok(Station {
                    id: id.to_string(),
                    name: name.to_string(),
                    latitude: Self::get_f64(item, "latitude"),
                    longitude: Self::get_f64(item, "longitude"),
                    elevation: Self::get_f64(item, "elevation"),
                    mindate: Self::get_str(item, "mindate").map(|s| s.to_string()),
                    maxdate: Self::get_str(item, "maxdate").map(|s| s.to_string()),
                    datacoverage: Self::get_f64(item, "datacoverage"),
                })
            })
            .collect()
    }

    /// Parse single station
    pub fn parse_station(response: &Value) -> ExchangeResult<Station> {
        let id = Self::require_str(response, "id")?;
        let name = Self::require_str(response, "name")?;

        Ok(Station {
            id: id.to_string(),
            name: name.to_string(),
            latitude: Self::get_f64(response, "latitude"),
            longitude: Self::get_f64(response, "longitude"),
            elevation: Self::get_f64(response, "elevation"),
            mindate: Self::get_str(response, "mindate").map(|s| s.to_string()),
            maxdate: Self::get_str(response, "maxdate").map(|s| s.to_string()),
            datacoverage: Self::get_f64(response, "datacoverage"),
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Climate data observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClimateData {
    pub date: String,
    pub datatype: String,
    pub station: String,
    pub value: f64,
    pub attributes: Option<String>,
}

/// Climate dataset metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub id: String,
    pub name: String,
    pub datacoverage: Option<f64>,
    pub mindate: Option<String>,
    pub maxdate: Option<String>,
}

/// Datatype metadata (temperature, precipitation, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Datatype {
    pub id: String,
    pub name: Option<String>,
    pub mindate: Option<String>,
    pub maxdate: Option<String>,
    pub datacoverage: Option<f64>,
}

/// Location category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCategory {
    pub id: String,
    pub name: String,
}

/// Geographic location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub id: String,
    pub name: String,
    pub mindate: Option<String>,
    pub maxdate: Option<String>,
    pub datacoverage: Option<f64>,
}

/// Weather station
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Station {
    pub id: String,
    pub name: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub elevation: Option<f64>,
    pub mindate: Option<String>,
    pub maxdate: Option<String>,
    pub datacoverage: Option<f64>,
}
