//! ReliefWeb response parsers
//!
//! Parse JSON responses to domain types based on ReliefWeb API response formats.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::ExchangeError;

pub type ExchangeResult<T> = Result<T, ExchangeError>;

pub struct ReliefWebParser;

impl ReliefWebParser {
    /// Parse ReliefWeb reports response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "totalCount": 1000,
    ///   "count": 10,
    ///   "data": [
    ///     {
    ///       "id": "123456",
    ///       "fields": {
    ///         "title": "Syria: Humanitarian Update",
    ///         "body": "Situation update...",
    ///         "date": { "created": "2024-01-15T10:00:00+00:00" },
    ///         "source": [{"name": "OCHA"}],
    ///         "country": [{"name": "Syria"}],
    ///         "theme": [{"name": "Coordination"}],
    ///         "format": [{"name": "Situation Report"}],
    ///         "url": "https://reliefweb.int/report/..."
    ///       }
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_reports(response: &Value) -> ExchangeResult<ReliefWebSearchResult<ReliefWebReport>> {
        let total_count = response
            .get("totalCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let count = response
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let data_array = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let data: Result<Vec<ReliefWebReport>, ExchangeError> = data_array
            .iter()
            .map(Self::parse_report)
            .collect();

        Ok(ReliefWebSearchResult {
            count,
            total: total_count,
            data: data?,
        })
    }

    /// Parse single ReliefWeb report
    fn parse_report(item: &Value) -> ExchangeResult<ReliefWebReport> {
        let id = Self::require_str(item, "id")?.to_string();
        let fields = item
            .get("fields")
            .ok_or_else(|| ExchangeError::Parse("Missing 'fields' object".to_string()))?;

        Ok(ReliefWebReport {
            id,
            title: Self::require_str(fields, "title")?.to_string(),
            body: Self::get_str(fields, "body").unwrap_or("").to_string(),
            date: Self::extract_date_created(fields).unwrap_or_default(),
            source: Self::extract_array_names(fields, "source"),
            country: Self::extract_array_names(fields, "country"),
            theme: Self::extract_array_names(fields, "theme"),
            format: Self::extract_array_names(fields, "format"),
            url: Self::get_str(fields, "url").unwrap_or("").to_string(),
        })
    }

    /// Parse ReliefWeb disasters response
    pub fn parse_disasters(response: &Value) -> ExchangeResult<ReliefWebSearchResult<ReliefWebDisaster>> {
        let total_count = response
            .get("totalCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let count = response
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let data_array = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let data: Result<Vec<ReliefWebDisaster>, ExchangeError> = data_array
            .iter()
            .map(Self::parse_disaster)
            .collect();

        Ok(ReliefWebSearchResult {
            count,
            total: total_count,
            data: data?,
        })
    }

    /// Parse single ReliefWeb disaster
    fn parse_disaster(item: &Value) -> ExchangeResult<ReliefWebDisaster> {
        let id = Self::require_str(item, "id")?.to_string();
        let fields = item
            .get("fields")
            .ok_or_else(|| ExchangeError::Parse("Missing 'fields' object".to_string()))?;

        Ok(ReliefWebDisaster {
            id,
            name: Self::require_str(fields, "name")?.to_string(),
            glide: Self::get_str(fields, "glide").unwrap_or("").to_string(),
            status: Self::get_str(fields, "status").unwrap_or("").to_string(),
            country: Self::extract_array_names(fields, "country"),
            date_event: Self::extract_date_field(fields, "date", "event").unwrap_or_default(),
            date_created: Self::extract_date_field(fields, "date", "created").unwrap_or_default(),
            type_name: Self::extract_first_name(fields, "type").unwrap_or_default(),
        })
    }

    /// Parse ReliefWeb countries response
    pub fn parse_countries(response: &Value) -> ExchangeResult<ReliefWebSearchResult<ReliefWebCountry>> {
        let total_count = response
            .get("totalCount")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let count = response
            .get("count")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let data_array = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        let data: Result<Vec<ReliefWebCountry>, ExchangeError> = data_array
            .iter()
            .map(Self::parse_country)
            .collect();

        Ok(ReliefWebSearchResult {
            count,
            total: total_count,
            data: data?,
        })
    }

    /// Parse single ReliefWeb country
    fn parse_country(item: &Value) -> ExchangeResult<ReliefWebCountry> {
        let id = Self::require_str(item, "id")?.to_string();
        let fields = item
            .get("fields")
            .ok_or_else(|| ExchangeError::Parse("Missing 'fields' object".to_string()))?;

        Ok(ReliefWebCountry {
            id,
            name: Self::require_str(fields, "name")?.to_string(),
            iso3: Self::get_str(fields, "iso3").unwrap_or("").to_string(),
            status: Self::get_str(fields, "status").unwrap_or("").to_string(),
            description: Self::get_str(fields, "description").unwrap_or("").to_string(),
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

        // Check for errors array
        if let Some(errors) = response.get("errors") {
            if let Some(error_array) = errors.as_array() {
                if !error_array.is_empty() {
                    let message = error_array[0]
                        .as_str()
                        .unwrap_or("Request failed")
                        .to_string();
                    return Err(ExchangeError::Api { code: 0, message });
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

    /// Extract array of names from field (e.g., source, country, theme)
    fn extract_array_names(obj: &Value, field: &str) -> Vec<String> {
        obj.get(field)
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.get("name").and_then(|n| n.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Extract first name from array field
    fn extract_first_name(obj: &Value, field: &str) -> Option<String> {
        obj.get(field)
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .and_then(|item| item.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
    }

    /// Extract date.created field
    fn extract_date_created(obj: &Value) -> Option<String> {
        obj.get("date")
            .and_then(|d| d.get("created"))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    }

    /// Extract date.{subfield} field
    fn extract_date_field(obj: &Value, field: &str, subfield: &str) -> Option<String> {
        obj.get(field)
            .and_then(|d| d.get(subfield))
            .and_then(|c| c.as_str())
            .map(|s| s.to_string())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RELIEFWEB-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// ReliefWeb report data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliefWebReport {
    pub id: String,
    pub title: String,
    pub body: String,
    pub date: String,
    pub source: Vec<String>,
    pub country: Vec<String>,
    pub theme: Vec<String>,
    pub format: Vec<String>,
    pub url: String,
}

/// ReliefWeb disaster data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliefWebDisaster {
    pub id: String,
    pub name: String,
    pub glide: String,
    pub status: String,
    pub country: Vec<String>,
    pub date_event: String,
    pub date_created: String,
    pub type_name: String,
}

/// ReliefWeb country data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReliefWebCountry {
    pub id: String,
    pub name: String,
    pub iso3: String,
    pub status: String,
    pub description: String,
}

/// ReliefWeb API search result wrapper
#[derive(Debug, Serialize, Deserialize)]
pub struct ReliefWebSearchResult<T> {
    pub count: u32,
    pub total: u32,
    pub data: Vec<T>,
}
