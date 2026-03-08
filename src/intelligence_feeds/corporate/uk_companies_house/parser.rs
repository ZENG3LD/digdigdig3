//! UK Companies House response parsers
//!
//! Parse JSON responses to domain types based on Companies House API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct UkCompaniesHouseParser;

// ═══════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════

/// Company information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChCompany {
    pub company_number: String,
    pub company_name: String,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub company_status: Option<String>,
    pub date_of_creation: Option<String>,
    pub registered_office_address: Option<ChAddress>,
    pub sic_codes: Option<Vec<String>>,
    pub has_charges: Option<bool>,
    pub has_insolvency_history: Option<bool>,
}

/// Officer information (director, secretary, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChOfficer {
    pub name: String,
    pub officer_role: Option<String>,
    pub appointed_on: Option<String>,
    pub resigned_on: Option<String>,
    pub nationality: Option<String>,
    pub occupation: Option<String>,
    pub country_of_residence: Option<String>,
    pub address: Option<ChAddress>,
}

/// Person with Significant Control (beneficial owner)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChPsc {
    pub name: String,
    #[serde(rename = "kind")]
    pub kind: Option<String>,
    pub natures_of_control: Option<Vec<String>>,
    pub notified_on: Option<String>,
    pub nationality: Option<String>,
    pub country_of_residence: Option<String>,
    pub address: Option<ChAddress>,
}

/// Filing history item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChFiling {
    pub category: Option<String>,
    pub date: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub barcode: Option<String>,
}

/// Address structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChAddress {
    pub address_line_1: Option<String>,
    pub address_line_2: Option<String>,
    pub locality: Option<String>,
    pub region: Option<String>,
    pub postal_code: Option<String>,
    pub country: Option<String>,
}

/// Search result wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChSearchResult {
    pub total_results: Option<u32>,
    pub items_per_page: Option<u32>,
    pub start_index: Option<u32>,
    pub items: Vec<ChCompany>,
}

// ═══════════════════════════════════════════════════════════════════════
// PARSER IMPLEMENTATION
// ═══════════════════════════════════════════════════════════════════════

impl UkCompaniesHouseParser {
    /// Parse company profile response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "company_number": "00000006",
    ///   "company_name": "MARINE CURRENT TURBINES LIMITED",
    ///   "type": "ltd",
    ///   "company_status": "active",
    ///   "date_of_creation": "1989-12-12",
    ///   "registered_office_address": {
    ///     "address_line_1": "Ground Floor",
    ///     "locality": "Bristol",
    ///     "postal_code": "BS1 6NB"
    ///   },
    ///   "sic_codes": ["62012"]
    /// }
    /// ```
    pub fn parse_company(response: &Value) -> ExchangeResult<ChCompany> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse company: {}", e)))
    }

    /// Parse search results response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "total_results": 5,
    ///   "items_per_page": 20,
    ///   "start_index": 0,
    ///   "items": [
    ///     {
    ///       "company_number": "00000006",
    ///       "company_name": "MARINE CURRENT TURBINES LIMITED",
    ///       "type": "ltd",
    ///       "company_status": "active"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_search_results(response: &Value) -> ExchangeResult<ChSearchResult> {
        serde_json::from_value(response.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse search results: {}", e)))
    }

    /// Parse officers list response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [
    ///     {
    ///       "name": "SMITH, John",
    ///       "officer_role": "director",
    ///       "appointed_on": "2015-01-01",
    ///       "nationality": "British",
    ///       "occupation": "Director"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_officers(response: &Value) -> ExchangeResult<Vec<ChOfficer>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|item| {
                serde_json::from_value(item.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse officer: {}", e)))
            })
            .collect()
    }

    /// Parse PSC (Persons with Significant Control) list response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [
    ///     {
    ///       "name": "Mr John Smith",
    ///       "kind": "individual-person-with-significant-control",
    ///       "natures_of_control": [
    ///         "ownership-of-shares-75-to-100-percent",
    ///         "voting-rights-75-to-100-percent"
    ///       ],
    ///       "notified_on": "2016-04-06",
    ///       "nationality": "British"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_psc(response: &Value) -> ExchangeResult<Vec<ChPsc>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|item| {
                serde_json::from_value(item.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse PSC: {}", e)))
            })
            .collect()
    }

    /// Parse filing history response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "items": [
    ///     {
    ///       "category": "accounts",
    ///       "date": "2023-12-31",
    ///       "description": "accounts-with-accounts-type-full",
    ///       "type": "AA",
    ///       "barcode": "X9LBD9IH"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_filing_history(response: &Value) -> ExchangeResult<Vec<ChFiling>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        items
            .iter()
            .map(|item| {
                serde_json::from_value(item.clone())
                    .map_err(|e| ExchangeError::Parse(format!("Failed to parse filing: {}", e)))
            })
            .collect()
    }

    /// Parse officer appointments response (cross-company)
    ///
    /// Returns raw JSON for flexibility since appointments have varied structures
    pub fn parse_officer_appointments(response: &Value) -> ExchangeResult<Vec<Value>> {
        let items = response
            .get("items")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'items' array".to_string()))?;

        Ok(items.clone())
    }

    /// Helper: Extract required string field
    fn _require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing required field: {}", field)))
    }

    /// Helper: Extract optional string field
    fn _get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}
