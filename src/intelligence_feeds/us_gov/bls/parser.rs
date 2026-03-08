//! BLS response parsers
//!
//! Parse JSON responses to domain types based on BLS API v2 response formats.
//!
//! BLS is an economic data provider (Bureau of Labor Statistics), similar to FRED.
//! It provides employment, inflation, and economic indicators data.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct BlsParser;

impl BlsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // BLS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for BLS API errors
    ///
    /// Example error response:
    /// ```json
    /// {
    ///   "status": "REQUEST_NOT_PROCESSED",
    ///   "message": ["series id is invalid"]
    /// }
    /// ```
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(status) = response.get("status").and_then(|v| v.as_str()) {
            if status != "REQUEST_SUCCEEDED" {
                let message = response
                    .get("message")
                    .and_then(|v| v.as_array())
                    .and_then(|arr| arr.first())
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown BLS API error");

                return Err(ExchangeError::Api {
                    code: -1,
                    message: format!("{}: {}", status, message),
                });
            }
        }
        Ok(())
    }

    /// Parse series data from BLS response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "status": "REQUEST_SUCCEEDED",
    ///   "responseTime": 50,
    ///   "message": [],
    ///   "Results": {
    ///     "series": [{
    ///       "seriesID": "LNS14000000",
    ///       "data": [{
    ///         "year": "2024",
    ///         "period": "M12",
    ///         "periodName": "December",
    ///         "latest": "true",
    ///         "value": "3.7",
    ///         "footnotes": [{}]
    ///       }]
    ///     }]
    ///   }
    /// }
    /// ```
    pub fn parse_series_data(response: &Value) -> ExchangeResult<Vec<BlsSeries>> {
        let series_array = response
            .get("Results")
            .and_then(|r| r.get("series"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'Results.series' array".to_string()))?;

        series_array
            .iter()
            .map(|series| {
                let series_id = Self::require_str(series, "seriesID")?;

                let data_array = series
                    .get("data")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

                let data: Result<Vec<BlsDataPoint>, ExchangeError> = data_array
                    .iter()
                    .map(|point| {
                        let year = Self::require_str(point, "year")?;
                        let period = Self::require_str(point, "period")?;
                        let value_str = Self::require_str(point, "value")?;

                        // Parse value - can be numeric or "-" for missing data
                        let value = if value_str == "-" {
                            None
                        } else {
                            Some(
                                value_str
                                    .parse::<f64>()
                                    .map_err(|_| ExchangeError::Parse(format!("Invalid value: {}", value_str)))?,
                            )
                        };

                        Ok(BlsDataPoint {
                            year: year.to_string(),
                            period: period.to_string(),
                            period_name: Self::get_str(point, "periodName").map(|s| s.to_string()),
                            value,
                            latest: Self::get_str(point, "latest")
                                .map(|s| s == "true")
                                .unwrap_or(false),
                            footnotes: Self::get_array(point, "footnotes")
                                .map(|arr| arr.len())
                                .unwrap_or(0),
                        })
                    })
                    .collect();

                Ok(BlsSeries {
                    series_id: series_id.to_string(),
                    data: data?,
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    fn get_array<'a>(obj: &'a Value, field: &str) -> Option<&'a Vec<Value>> {
        obj.get(field).and_then(|v| v.as_array())
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| v.as_i64())
    }

    fn _get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| v.as_f64())
    }

    fn _get_bool(obj: &Value, field: &str) -> Option<bool> {
        obj.get(field).and_then(|v| v.as_bool())
    }
}

// ═══════════════════════════════════════════════════════════════════════
// BLS DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════

/// BLS series data
#[derive(Debug, Clone)]
pub struct BlsSeries {
    pub series_id: String,
    pub data: Vec<BlsDataPoint>,
}

/// BLS data point
#[derive(Debug, Clone)]
pub struct BlsDataPoint {
    pub year: String,
    pub period: String, // M01-M12 for monthly, Q01-Q04 for quarterly, A01 for annual
    pub period_name: Option<String>, // "January", "Q1", etc.
    pub value: Option<f64>, // None if value is "-" (missing data)
    pub latest: bool,
    pub footnotes: usize, // Number of footnotes
}

impl BlsDataPoint {
    /// Convert period code to human-readable format
    ///
    /// Examples: M01 -> "January", M12 -> "December", Q01 -> "Q1", A01 -> "Annual"
    pub fn period_display(&self) -> String {
        self.period_name.clone().unwrap_or_else(|| self.period.clone())
    }

    /// Get full date string (YYYY-MM format for monthly data)
    pub fn date_string(&self) -> String {
        if self.period.starts_with('M') {
            let month = self.period.trim_start_matches('M');
            format!("{}-{}", self.year, month)
        } else if self.period.starts_with('Q') {
            format!("{}-{}", self.year, self.period)
        } else {
            self.year.clone()
        }
    }
}
