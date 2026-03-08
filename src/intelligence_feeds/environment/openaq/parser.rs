//! OpenAQ response parsers
//!
//! Parse JSON responses to domain types based on OpenAQ API response formats.
//!
//! OpenAQ is an environmental data provider, providing air quality measurements
//! from monitoring stations worldwide.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct OpenAqParser;

impl OpenAqParser {
    // ═══════════════════════════════════════════════════════════════════════
    // OPENAQ-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse locations
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "meta": { "found": 100 },
    ///   "results": [{
    ///     "id": 123,
    ///     "name": "Station Name",
    ///     "city": "City",
    ///     "country": "US",
    ///     "coordinates": { "latitude": 40.7, "longitude": -74.0 },
    ///     "parameters": [{"parameter": "pm25"}, {"parameter": "o3"}],
    ///     "measurements": 1000
    ///   }]
    /// }
    /// ```
    pub fn parse_locations(response: &Value) -> ExchangeResult<Vec<OpenAqLocation>> {
        let results = Self::get_results_array(response)?;

        results
            .iter()
            .map(|loc| {
                let id = Self::get_i64(loc, "id");
                let name = Self::get_str(loc, "name")
                    .unwrap_or("Unknown")
                    .to_string();
                let city = Self::get_str(loc, "city").map(|s| s.to_string());
                let country = Self::get_str(loc, "country")
                    .unwrap_or("Unknown")
                    .to_string();

                // Parse coordinates
                let (latitude, longitude) = if let Some(coords) = loc.get("coordinates") {
                    (
                        Self::get_f64(coords, "latitude"),
                        Self::get_f64(coords, "longitude"),
                    )
                } else {
                    (None, None)
                };

                // Parse parameters array
                let parameters = if let Some(params_array) = loc.get("parameters").and_then(|v| v.as_array()) {
                    params_array
                        .iter()
                        .filter_map(|p| {
                            p.get("parameter")
                                .or_else(|| p.get("id"))
                                .and_then(|v| v.as_str())
                        })
                        .map(|s| s.to_string())
                        .collect()
                } else {
                    Vec::new()
                };

                let measurements_count = Self::get_i64(loc, "measurements")
                    .or_else(|| Self::get_i64(loc, "count"));

                Ok(OpenAqLocation {
                    id,
                    name,
                    city,
                    country,
                    latitude,
                    longitude,
                    parameters,
                    measurements_count,
                })
            })
            .collect()
    }

    /// Parse measurements
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "meta": { "found": 1000 },
    ///   "results": [{
    ///     "locationId": 123,
    ///     "location": "Station Name",
    ///     "parameter": "pm25",
    ///     "value": 12.5,
    ///     "unit": "µg/m³",
    ///     "date": {
    ///       "utc": "2024-01-15T12:00:00Z",
    ///       "local": "2024-01-15T07:00:00-05:00"
    ///     },
    ///     "coordinates": { "latitude": 40.7, "longitude": -74.0 }
    ///   }]
    /// }
    /// ```
    pub fn parse_measurements(response: &Value) -> ExchangeResult<Vec<OpenAqMeasurement>> {
        let results = Self::get_results_array(response)?;

        results
            .iter()
            .map(|meas| {
                let location_id = Self::get_i64(meas, "locationId")
                    .or_else(|| Self::get_i64(meas, "location_id"));
                let location = Self::get_str(meas, "location")
                    .unwrap_or("Unknown")
                    .to_string();
                let parameter = Self::get_str(meas, "parameter")
                    .unwrap_or("unknown")
                    .to_string();

                let value = Self::get_f64(meas, "value");
                let unit = Self::get_str(meas, "unit")
                    .unwrap_or("unknown")
                    .to_string();

                // Parse date info
                let date = if let Some(date_obj) = meas.get("date") {
                    DateInfo {
                        utc: Self::get_str(date_obj, "utc").map(|s| s.to_string()),
                        local: Self::get_str(date_obj, "local").map(|s| s.to_string()),
                    }
                } else {
                    DateInfo {
                        utc: None,
                        local: None,
                    }
                };

                // Parse coordinates
                let (latitude, longitude) = if let Some(coords) = meas.get("coordinates") {
                    (
                        Self::get_f64(coords, "latitude"),
                        Self::get_f64(coords, "longitude"),
                    )
                } else {
                    (None, None)
                };

                Ok(OpenAqMeasurement {
                    location_id,
                    location,
                    parameter,
                    value,
                    unit,
                    date,
                    latitude,
                    longitude,
                })
            })
            .collect()
    }

    /// Parse countries
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "meta": { "found": 200 },
    ///   "results": [{
    ///     "code": "US",
    ///     "name": "United States",
    ///     "locations": 500,
    ///     "count": 1000000
    ///   }]
    /// }
    /// ```
    pub fn parse_countries(response: &Value) -> ExchangeResult<Vec<OpenAqCountry>> {
        let results = Self::get_results_array(response)?;

        results
            .iter()
            .map(|country| {
                Ok(OpenAqCountry {
                    code: Self::get_str(country, "code")
                        .unwrap_or("Unknown")
                        .to_string(),
                    name: Self::get_str(country, "name")
                        .unwrap_or("Unknown")
                        .to_string(),
                    locations_count: Self::get_i64(country, "locations")
                        .or_else(|| Self::get_i64(country, "locations_count")),
                    measurements_count: Self::get_i64(country, "count")
                        .or_else(|| Self::get_i64(country, "measurements")),
                })
            })
            .collect()
    }

    /// Parse cities
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "meta": { "found": 1000 },
    ///   "results": [{
    ///     "city": "New York",
    ///     "country": "US",
    ///     "count": 10000,
    ///     "locations": 50
    ///   }]
    /// }
    /// ```
    pub fn parse_cities(response: &Value) -> ExchangeResult<Vec<OpenAqCity>> {
        let results = Self::get_results_array(response)?;

        results
            .iter()
            .map(|city| {
                Ok(OpenAqCity {
                    city: Self::get_str(city, "city")
                        .unwrap_or("Unknown")
                        .to_string(),
                    country: Self::get_str(city, "country")
                        .unwrap_or("Unknown")
                        .to_string(),
                    count: Self::get_i64(city, "count")
                        .or_else(|| Self::get_i64(city, "measurements")),
                    locations: Self::get_i64(city, "locations")
                        .or_else(|| Self::get_i64(city, "locations_count")),
                })
            })
            .collect()
    }

    /// Parse parameters
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "meta": { "found": 6 },
    ///   "results": [{
    ///     "id": "pm25",
    ///     "name": "PM2.5",
    ///     "displayName": "PM2.5",
    ///     "description": "Particulate matter less than 2.5 micrometers in diameter",
    ///     "preferredUnit": "µg/m³"
    ///   }]
    /// }
    /// ```
    pub fn parse_parameters(response: &Value) -> ExchangeResult<Vec<OpenAqParameter>> {
        let results = Self::get_results_array(response)?;

        results
            .iter()
            .map(|param| {
                Ok(OpenAqParameter {
                    id: Self::get_str(param, "id")
                        .or_else(|| Self::get_str(param, "parameter"))
                        .unwrap_or("unknown")
                        .to_string(),
                    name: Self::get_str(param, "name")
                        .unwrap_or("Unknown")
                        .to_string(),
                    display_name: Self::get_str(param, "displayName")
                        .or_else(|| Self::get_str(param, "display_name"))
                        .map(|s| s.to_string()),
                    description: Self::get_str(param, "description").map(|s| s.to_string()),
                    preferred_unit: Self::get_str(param, "preferredUnit")
                        .or_else(|| Self::get_str(param, "preferred_unit"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse latest measurements
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "meta": { "found": 100 },
    ///   "results": [{
    ///     "location": "Station Name",
    ///     "city": "City",
    ///     "country": "US",
    ///     "measurements": [{
    ///       "parameter": "pm25",
    ///       "value": 12.5,
    ///       "unit": "µg/m³",
    ///       "lastUpdated": "2024-01-15T12:00:00Z"
    ///     }]
    ///   }]
    /// }
    /// ```
    pub fn parse_latest(response: &Value) -> ExchangeResult<Vec<OpenAqLatest>> {
        let results = Self::get_results_array(response)?;

        results
            .iter()
            .map(|latest| {
                let measurements = if let Some(meas_array) = latest.get("measurements").and_then(|v| v.as_array()) {
                    meas_array
                        .iter()
                        .filter_map(|m| {
                            let parameter = Self::get_str(m, "parameter")?.to_string();
                            let value = Self::get_f64(m, "value")?;
                            let unit = Self::get_str(m, "unit")
                                .unwrap_or("unknown")
                                .to_string();
                            let last_updated = Self::get_str(m, "lastUpdated")
                                .or_else(|| Self::get_str(m, "last_updated"))
                                .map(|s| s.to_string());

                            Some(LatestMeasurement {
                                parameter,
                                value,
                                unit,
                                last_updated,
                            })
                        })
                        .collect()
                } else {
                    Vec::new()
                };

                Ok(OpenAqLatest {
                    location: Self::get_str(latest, "location")
                        .unwrap_or("Unknown")
                        .to_string(),
                    city: Self::get_str(latest, "city").map(|s| s.to_string()),
                    country: Self::get_str(latest, "country")
                        .unwrap_or("Unknown")
                        .to_string(),
                    measurements,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check for error field
        if let Some(error) = response.get("error") {
            let message = if let Some(msg) = error.as_str() {
                msg.to_string()
            } else if let Some(msg) = error.get("message").and_then(|v| v.as_str()) {
                msg.to_string()
            } else {
                "Unknown error".to_string()
            };

            return Err(ExchangeError::Api {
                code: 0,
                message,
            });
        }

        // Check for status error
        if let Some(status) = response.get("status") {
            if let Some(status_str) = status.as_str() {
                if status_str == "error" || status_str == "fail" {
                    let message = response
                        .get("message")
                        .and_then(|v| v.as_str())
                        .unwrap_or("API error")
                        .to_string();

                    return Err(ExchangeError::Api {
                        code: 0,
                        message,
                    });
                }
            }
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get results array from response
    fn get_results_array(response: &Value) -> ExchangeResult<&Vec<Value>> {
        response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// OPENAQ-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// OpenAQ monitoring location
#[derive(Debug, Clone)]
pub struct OpenAqLocation {
    pub id: Option<i64>,
    pub name: String,
    pub city: Option<String>,
    pub country: String,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub parameters: Vec<String>,
    pub measurements_count: Option<i64>,
}

/// OpenAQ air quality measurement
#[derive(Debug, Clone)]
pub struct OpenAqMeasurement {
    pub location_id: Option<i64>,
    pub location: String,
    pub parameter: String,
    pub value: Option<f64>,
    pub unit: String,
    pub date: DateInfo,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

/// Date information (UTC and local time)
#[derive(Debug, Clone)]
pub struct DateInfo {
    pub utc: Option<String>,
    pub local: Option<String>,
}

/// OpenAQ country
#[derive(Debug, Clone)]
pub struct OpenAqCountry {
    pub code: String,
    pub name: String,
    pub locations_count: Option<i64>,
    pub measurements_count: Option<i64>,
}

/// OpenAQ city
#[derive(Debug, Clone)]
pub struct OpenAqCity {
    pub city: String,
    pub country: String,
    pub count: Option<i64>,
    pub locations: Option<i64>,
}

/// OpenAQ parameter (measurable pollutant)
#[derive(Debug, Clone)]
pub struct OpenAqParameter {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub description: Option<String>,
    pub preferred_unit: Option<String>,
}

/// Latest measurements from a location
#[derive(Debug, Clone)]
pub struct OpenAqLatest {
    pub location: String,
    pub city: Option<String>,
    pub country: String,
    pub measurements: Vec<LatestMeasurement>,
}

/// Single latest measurement
#[derive(Debug, Clone)]
pub struct LatestMeasurement {
    pub parameter: String,
    pub value: f64,
    pub unit: String,
    pub last_updated: Option<String>,
}
