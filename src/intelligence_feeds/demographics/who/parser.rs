//! WHO GHO response parsers
//!
//! Parse JSON responses to domain types based on WHO GHO API response formats.
//!
//! WHO GHO uses OData format with {"value":[...]} wrapper for collections.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct WhoParser;

impl WhoParser {
    // ═══════════════════════════════════════════════════════════════════════
    // WHO-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse indicators list
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "value": [
    ///     {
    ///       "IndicatorCode": "WHOSIS_000001",
    ///       "IndicatorName": "Life expectancy at birth (years)",
    ///       "Language": "EN"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_indicators(response: &Value) -> ExchangeResult<Vec<WhoIndicator>> {
        let value_array = response
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'value' array".to_string()))?;

        value_array
            .iter()
            .map(|ind| {
                Ok(WhoIndicator {
                    code: Self::require_str(ind, "IndicatorCode")?.to_string(),
                    label: Self::get_str(ind, "IndicatorName")
                        .or_else(|| Self::get_str(ind, "label"))
                        .map(|s| s.to_string()),
                    display: Self::get_str(ind, "Display")
                        .or_else(|| Self::get_str(ind, "display"))
                        .map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse indicator data points
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "value": [
    ///     {
    ///       "IndicatorCode": "WHOSIS_000001",
    ///       "SpatialDim": "USA",
    ///       "TimeDim": 2020,
    ///       "NumericValue": 78.9,
    ///       "Value": "78.9",
    ///       "Low": 78.5,
    ///       "High": 79.3
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_data_points(response: &Value) -> ExchangeResult<Vec<WhoDataPoint>> {
        let value_array = response
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'value' array".to_string()))?;

        value_array
            .iter()
            .map(|dp| {
                Ok(WhoDataPoint {
                    indicator_code: Self::require_str(dp, "IndicatorCode")?.to_string(),
                    spatial_dim: Self::get_str(dp, "SpatialDim")
                        .or_else(|| Self::get_str(dp, "SpatialDimCode"))
                        .map(|s| s.to_string()),
                    time_dim: Self::get_i64(dp, "TimeDim")
                        .or_else(|| {
                            Self::get_str(dp, "TimeDim")
                                .and_then(|s| s.parse::<i64>().ok())
                        }),
                    value: Self::get_f64(dp, "NumericValue")
                        .or_else(|| {
                            Self::get_str(dp, "Value")
                                .and_then(|s| s.parse::<f64>().ok())
                        }),
                    value_str: Self::get_str(dp, "Value").map(|s| s.to_string()),
                    low: Self::get_f64(dp, "Low"),
                    high: Self::get_f64(dp, "High"),
                })
            })
            .collect()
    }

    /// Parse countries
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "value": [
    ///     {
    ///       "Code": "USA",
    ///       "Title": "United States of America"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_countries(response: &Value) -> ExchangeResult<Vec<WhoCountry>> {
        let value_array = response
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'value' array".to_string()))?;

        value_array
            .iter()
            .map(|country| {
                Ok(WhoCountry {
                    code: Self::require_str(country, "Code")?.to_string(),
                    title: Self::require_str(country, "Title")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse regions
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "value": [
    ///     {
    ///       "Code": "AMR",
    ///       "Title": "Americas"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_regions(response: &Value) -> ExchangeResult<Vec<WhoRegion>> {
        let value_array = response
            .get("value")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'value' array".to_string()))?;

        value_array
            .iter()
            .map(|region| {
                Ok(WhoRegion {
                    code: Self::require_str(region, "Code")?.to_string(),
                    title: Self::require_str(region, "Title")?.to_string(),
                })
            })
            .collect()
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
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);

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

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WHO-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// WHO health indicator
#[derive(Debug, Clone)]
pub struct WhoIndicator {
    pub code: String,
    pub label: Option<String>,
    pub display: Option<String>,
}

/// WHO data point
#[derive(Debug, Clone)]
pub struct WhoDataPoint {
    pub indicator_code: String,
    pub spatial_dim: Option<String>,  // Country code (e.g., "USA")
    pub time_dim: Option<i64>,        // Year
    pub value: Option<f64>,           // Numeric value
    pub value_str: Option<String>,    // String value
    pub low: Option<f64>,             // Lower bound
    pub high: Option<f64>,            // Upper bound
}

/// WHO country
#[derive(Debug, Clone)]
pub struct WhoCountry {
    pub code: String,
    pub title: String,
}

/// WHO region
#[derive(Debug, Clone)]
pub struct WhoRegion {
    pub code: String,
    pub title: String,
}
