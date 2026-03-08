//! OpenFIGI response parsers
//!
//! Parse JSON responses to domain types based on OpenFIGI API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct OpenFigiParser;

impl OpenFigiParser {
    // ═══════════════════════════════════════════════════════════════════════
    // OPENFIGI-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse mapping response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": [
    ///     [
    ///       {
    ///         "figi": "BBG000BLNNH6",
    ///         "name": "APPLE INC",
    ///         "ticker": "AAPL",
    ///         "exchCode": "US",
    ///         "compositeFIGI": "BBG000B9XRY4",
    ///         "shareClassFIGI": "BBG001S5N8V8",
    ///         "securityType": "Common Stock",
    ///         "marketSector": "Equity",
    ///         "securityDescription": "AAPL",
    ///         "uniqueID": "EQ0010169500001000"
    ///       }
    ///     ]
    ///   ]
    /// }
    /// ```
    pub fn parse_mapping_response(response: &Value) -> ExchangeResult<FigiMappingResponse> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let mut results: Vec<Vec<FigiResult>> = Vec::new();

        for job_results in data.iter() {
            if let Some(arr) = job_results.as_array() {
                let mut job_vec = Vec::new();
                for item in arr.iter() {
                    let result = Self::parse_figi_result(item)?;
                    job_vec.push(result);
                }
                results.push(job_vec);
            } else {
                // Handle error case in response
                if job_results.get("error").is_some() {
                    let error_msg = job_results
                        .get("error")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown error");
                    return Err(ExchangeError::Api {
                        code: 400,
                        message: error_msg.to_string(),
                    });
                }
                results.push(Vec::new());
            }
        }

        Ok(FigiMappingResponse { data: results })
    }

    /// Parse a single FIGI result
    fn parse_figi_result(item: &Value) -> ExchangeResult<FigiResult> {
        Ok(FigiResult {
            figi: Self::get_str(item, "figi").map(|s| s.to_string()),
            name: Self::get_str(item, "name").map(|s| s.to_string()),
            ticker: Self::get_str(item, "ticker").map(|s| s.to_string()),
            exchange_code: Self::get_str(item, "exchCode").map(|s| s.to_string()),
            composite_figi: Self::get_str(item, "compositeFIGI").map(|s| s.to_string()),
            share_class_figi: Self::get_str(item, "shareClassFIGI").map(|s| s.to_string()),
            security_type: Self::get_str(item, "securityType").map(|s| s.to_string()),
            market_sector: Self::get_str(item, "marketSector").map(|s| s.to_string()),
            security_description: Self::get_str(item, "securityDescription").map(|s| s.to_string()),
            unique_id: Self::get_str(item, "uniqueID").map(|s| s.to_string()),
        })
    }

    /// Parse search response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "data": [
    ///     {
    ///       "figi": "BBG000BLNNH6",
    ///       "name": "APPLE INC",
    ///       "ticker": "AAPL",
    ///       "exchCode": "US",
    ///       "compositeFIGI": "BBG000B9XRY4",
    ///       "securityType": "Common Stock",
    ///       "marketSector": "Equity"
    ///     }
    ///   ],
    ///   "total": 1
    /// }
    /// ```
    pub fn parse_search_response(response: &Value) -> ExchangeResult<FigiSearchResponse> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let results: Vec<FigiResult> = data
            .iter()
            .filter_map(|item| Self::parse_figi_result(item).ok())
            .collect();

        let total = response
            .get("total")
            .and_then(|v| v.as_u64())
            .unwrap_or(results.len() as u64);

        Ok(FigiSearchResponse {
            data: results,
            total,
        })
    }

    /// Parse enum values response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "values": [
    ///     "US",
    ///     "LN",
    ///     "JP"
    ///   ]
    /// }
    /// ```
    pub fn parse_enum_values(response: &Value) -> ExchangeResult<FigiEnumValues> {
        let values = response
            .get("values")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'values' array".to_string()))?;

        let string_values: Vec<String> = values
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(FigiEnumValues {
            values: string_values,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();

            return Err(ExchangeError::Api {
                code: 400,
                message,
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
// OPENFIGI-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// FIGI result (single instrument)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FigiResult {
    pub figi: Option<String>,
    pub name: Option<String>,
    pub ticker: Option<String>,
    pub exchange_code: Option<String>,
    pub composite_figi: Option<String>,
    pub share_class_figi: Option<String>,
    pub security_type: Option<String>,
    pub market_sector: Option<String>,
    pub security_description: Option<String>,
    pub unique_id: Option<String>,
}

/// FIGI mapping response (array of arrays - each job returns array of results)
#[derive(Debug, Clone)]
pub struct FigiMappingResponse {
    pub data: Vec<Vec<FigiResult>>,
}

/// FIGI search response
#[derive(Debug, Clone)]
pub struct FigiSearchResponse {
    pub data: Vec<FigiResult>,
    pub total: u64,
}

/// FIGI enum values
#[derive(Debug, Clone)]
pub struct FigiEnumValues {
    pub values: Vec<String>,
}
