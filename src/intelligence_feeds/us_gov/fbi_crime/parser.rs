//! FBI Crime Data API response parsers
//!
//! Parse JSON responses to domain types based on FBI Crime Data API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct FbiCrimeParser;

impl FbiCrimeParser {
    /// Parse crime estimates response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [
    ///     {
    ///       "year": 2022,
    ///       "population": 331097593,
    ///       "violent_crime": 1232428,
    ///       "homicide": 21156,
    ///       "rape": 139815,
    ///       "robbery": 363109,
    ///       "aggravated_assault": 708348,
    ///       "property_crime": 5298918,
    ///       "burglary": 924949,
    ///       "larceny": 3854449,
    ///       "motor_vehicle_theft": 519520
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_estimates(response: &Value) -> ExchangeResult<Vec<CrimeEstimate>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let estimates: Result<Vec<CrimeEstimate>, ExchangeError> = results
            .iter()
            .map(Self::parse_estimate)
            .collect();

        estimates
    }

    /// Parse single crime estimate
    fn parse_estimate(item: &Value) -> ExchangeResult<CrimeEstimate> {
        Ok(CrimeEstimate {
            year: Self::require_u32(item, "year")?,
            population: Self::get_u64(item, "population"),
            violent_crime: Self::get_u32(item, "violent_crime"),
            homicide: Self::get_u32(item, "homicide"),
            rape: Self::get_u32(item, "rape"),
            robbery: Self::get_u32(item, "robbery"),
            aggravated_assault: Self::get_u32(item, "aggravated_assault"),
            property_crime: Self::get_u32(item, "property_crime"),
            burglary: Self::get_u32(item, "burglary"),
            larceny: Self::get_u32(item, "larceny"),
            motor_vehicle_theft: Self::get_u32(item, "motor_vehicle_theft"),
        })
    }

    /// Parse agencies response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [
    ///     {
    ///       "ori": "AK0010100",
    ///       "agency_name": "Anchorage Police Department",
    ///       "state_abbr": "AK",
    ///       "county_name": "Anchorage",
    ///       "agency_type_name": "City"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_agencies(response: &Value) -> ExchangeResult<Vec<CrimeAgency>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let agencies: Result<Vec<CrimeAgency>, ExchangeError> = results
            .iter()
            .map(Self::parse_agency)
            .collect();

        agencies
    }

    /// Parse single agency
    fn parse_agency(item: &Value) -> ExchangeResult<CrimeAgency> {
        Ok(CrimeAgency {
            ori: Self::require_str(item, "ori")?.to_string(),
            agency_name: Self::require_str(item, "agency_name")?.to_string(),
            state_abbr: Self::get_str(item, "state_abbr").map(|s| s.to_string()),
            county_name: Self::get_str(item, "county_name").map(|s| s.to_string()),
            agency_type_name: Self::get_str(item, "agency_type_name").map(|s| s.to_string()),
        })
    }

    /// Parse NIBRS data response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "results": [
    ///     {
    ///       "key": "20-29",
    ///       "value": 15234
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_nibrs(response: &Value) -> ExchangeResult<Vec<NibrsData>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        let nibrs_data: Result<Vec<NibrsData>, ExchangeError> = results
            .iter()
            .map(Self::parse_nibrs_item)
            .collect();

        nibrs_data
    }

    /// Parse single NIBRS data item
    fn parse_nibrs_item(item: &Value) -> ExchangeResult<NibrsData> {
        Ok(NibrsData {
            key: Self::require_str(item, "key")?.to_string(),
            value: Self::require_u32(item, "value")?,
        })
    }

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(error) = response.get("error") {
            let message = error
                .as_str()
                .unwrap_or("Unknown error")
                .to_string();
            return Err(ExchangeError::Api { code: 0, message });
        }

        // Check for error message in different format
        if let Some(message) = response.get("message") {
            if let Some(msg_str) = message.as_str() {
                if msg_str.to_lowercase().contains("error") {
                    return Err(ExchangeError::Api {
                        code: 0,
                        message: msg_str.to_string(),
                    });
                }
            }
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

    fn require_u32(obj: &Value, field: &str) -> ExchangeResult<u32> {
        obj.get(field)
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_u32(obj: &Value, field: &str) -> Option<u32> {
        obj.get(field).and_then(|v| v.as_u64()).map(|v| v as u32)
    }

    fn get_u64(obj: &Value, field: &str) -> Option<u64> {
        obj.get(field).and_then(|v| v.as_u64())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FBI CRIME-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// Crime estimate data (national or state-level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrimeEstimate {
    pub year: u32,
    pub population: Option<u64>,
    pub violent_crime: Option<u32>,
    pub homicide: Option<u32>,
    pub rape: Option<u32>,
    pub robbery: Option<u32>,
    pub aggravated_assault: Option<u32>,
    pub property_crime: Option<u32>,
    pub burglary: Option<u32>,
    pub larceny: Option<u32>,
    pub motor_vehicle_theft: Option<u32>,
}

/// Crime reporting agency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrimeAgency {
    pub ori: String,
    pub agency_name: String,
    pub state_abbr: Option<String>,
    pub county_name: Option<String>,
    pub agency_type_name: Option<String>,
}

/// NIBRS data (offense, victim, or offender counts)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NibrsData {
    pub key: String,
    pub value: u32,
}
