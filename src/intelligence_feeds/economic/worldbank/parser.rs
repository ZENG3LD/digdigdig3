//! World Bank response parsers
//!
//! Parse JSON responses to domain types based on World Bank API response formats.
//!
//! World Bank API returns data in a specific format:
//! - All responses are arrays with 2 elements: [pagination_info, data_array]
//! - Pagination info contains: page, pages, per_page, total
//! - Data is in the second array element

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct WorldBankParser;

impl WorldBankParser {
    // ═══════════════════════════════════════════════════════════════════════
    // CORE PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse indicator data (time series)
    pub fn parse_indicator_data(response: &Value) -> ExchangeResult<Vec<IndicatorObservation>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                let date = Self::require_str(item, "date")?;
                let value_raw = item.get("value");

                // Parse value - can be number, string, or null
                let value = match value_raw {
                    Some(Value::Number(n)) => n.as_f64(),
                    Some(Value::String(s)) => s.parse::<f64>().ok(),
                    Some(Value::Null) | None => None,
                    _ => None,
                };

                let indicator = item
                    .get("indicator")
                    .and_then(|v| v.get("id"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");

                let country = item
                    .get("countryiso3code")
                    .and_then(|v| v.as_str())
                    .or_else(|| {
                        item.get("country")
                            .and_then(|v| v.get("id"))
                            .and_then(|v| v.as_str())
                    })
                    .unwrap_or("unknown");

                Ok(IndicatorObservation {
                    indicator: indicator.to_string(),
                    country: country.to_string(),
                    date: date.to_string(),
                    value,
                    unit: Self::get_str(item, "unit").map(|s| s.to_string()),
                    obs_status: Self::get_str(item, "obs_status").map(|s| s.to_string()),
                    decimal: Self::get_i64(item, "decimal"),
                })
            })
            .collect()
    }

    /// Parse indicator metadata
    pub fn parse_indicator_metadata(response: &Value) -> ExchangeResult<IndicatorMetadata> {
        let data_array = Self::extract_data_array(response)?;

        let item = data_array
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty data array".to_string()))?;

        let topics: Vec<String> = item
            .get("topics")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|t| t.get("value").and_then(|v| v.as_str()))
                    .map(|s| s.to_string())
                    .collect()
            })
            .unwrap_or_default();

        Ok(IndicatorMetadata {
            id: Self::require_str(item, "id")?.to_string(),
            name: Self::require_str(item, "name")?.to_string(),
            source_note: Self::get_str(item, "sourceNote").map(|s| s.to_string()),
            source_organization: Self::get_str(item, "sourceOrganization").map(|s| s.to_string()),
            topics,
        })
    }

    /// Parse indicator list (search results or full list)
    pub fn parse_indicator_list(response: &Value) -> ExchangeResult<Vec<IndicatorInfo>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                Ok(IndicatorInfo {
                    id: Self::require_str(item, "id")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                    source_note: Self::get_str(item, "sourceNote").map(|s| s.to_string()),
                })
            })
            .collect()
    }

    /// Parse country metadata
    pub fn parse_country(response: &Value) -> ExchangeResult<Country> {
        let data_array = Self::extract_data_array(response)?;

        let item = data_array
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty data array".to_string()))?;

        Ok(Country {
            id: Self::require_str(item, "id")?.to_string(),
            iso2_code: Self::require_str(item, "iso2Code")?.to_string(),
            name: Self::require_str(item, "name")?.to_string(),
            region: Self::get_str(item, "region")
                .and_then(|v| {
                    if let Value::Object(obj) = item.get("region")? {
                        obj.get("value").and_then(|v| v.as_str())
                    } else {
                        Some(v)
                    }
                })
                .map(|s| s.to_string()),
            income_level: Self::get_str(item, "incomeLevel")
                .and_then(|_| {
                    item.get("incomeLevel")
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_str())
                })
                .map(|s| s.to_string()),
            lending_type: Self::get_str(item, "lendingType")
                .and_then(|_| {
                    item.get("lendingType")
                        .and_then(|v| v.get("value"))
                        .and_then(|v| v.as_str())
                })
                .map(|s| s.to_string()),
            capital_city: Self::get_str(item, "capitalCity").map(|s| s.to_string()),
            longitude: Self::get_str(item, "longitude").map(|s| s.to_string()),
            latitude: Self::get_str(item, "latitude").map(|s| s.to_string()),
        })
    }

    /// Parse country list
    pub fn parse_country_list(response: &Value) -> ExchangeResult<Vec<CountryInfo>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                Ok(CountryInfo {
                    id: Self::require_str(item, "id")?.to_string(),
                    iso2_code: Self::require_str(item, "iso2Code")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse topic metadata
    pub fn parse_topic(response: &Value) -> ExchangeResult<Topic> {
        let data_array = Self::extract_data_array(response)?;

        let item = data_array
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty data array".to_string()))?;

        Ok(Topic {
            id: Self::require_str(item, "id")?.to_string(),
            value: Self::require_str(item, "value")?.to_string(),
            source_note: Self::get_str(item, "sourceNote").map(|s| s.to_string()),
        })
    }

    /// Parse topic list
    pub fn parse_topic_list(response: &Value) -> ExchangeResult<Vec<TopicInfo>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                Ok(TopicInfo {
                    id: Self::require_str(item, "id")?.to_string(),
                    value: Self::require_str(item, "value")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse source metadata
    pub fn parse_source(response: &Value) -> ExchangeResult<Source> {
        let data_array = Self::extract_data_array(response)?;

        let item = data_array
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty data array".to_string()))?;

        Ok(Source {
            id: Self::require_str(item, "id")?.to_string(),
            name: Self::require_str(item, "name")?.to_string(),
            description: Self::get_str(item, "description").map(|s| s.to_string()),
            url: Self::get_str(item, "url").map(|s| s.to_string()),
        })
    }

    /// Parse source list
    pub fn parse_source_list(response: &Value) -> ExchangeResult<Vec<SourceInfo>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                Ok(SourceInfo {
                    id: Self::require_str(item, "id")?.to_string(),
                    name: Self::require_str(item, "name")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse income level list
    pub fn parse_income_levels(response: &Value) -> ExchangeResult<Vec<IncomeLevel>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                Ok(IncomeLevel {
                    id: Self::require_str(item, "id")?.to_string(),
                    value: Self::require_str(item, "value")?.to_string(),
                })
            })
            .collect()
    }

    /// Parse lending type list
    pub fn parse_lending_types(response: &Value) -> ExchangeResult<Vec<LendingType>> {
        let data_array = Self::extract_data_array(response)?;

        data_array
            .iter()
            .map(|item| {
                Ok(LendingType {
                    id: Self::require_str(item, "id")?.to_string(),
                    value: Self::require_str(item, "value")?.to_string(),
                })
            })
            .collect()
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Extract data array from World Bank response format
    ///
    /// World Bank returns: [pagination_info, data_array]
    fn extract_data_array(response: &Value) -> ExchangeResult<&Vec<Value>> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        if array.len() < 2 {
            return Err(ExchangeError::Parse(
                "Response array has less than 2 elements".to_string(),
            ));
        }

        array[1]
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Data element is not an array".to_string()))
    }

    /// Extract pagination info from World Bank response
    fn _extract_pagination(response: &Value) -> ExchangeResult<Pagination> {
        let array = response
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Response is not an array".to_string()))?;

        let pagination_obj = array
            .first()
            .ok_or_else(|| ExchangeError::Parse("Missing pagination info".to_string()))?;

        Ok(Pagination {
            page: Self::get_i64(pagination_obj, "page").unwrap_or(1),
            pages: Self::get_i64(pagination_obj, "pages").unwrap_or(1),
            per_page: Self::get_i64(pagination_obj, "per_page").unwrap_or(50),
            total: Self::get_i64(pagination_obj, "total").unwrap_or(0),
        })
    }

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

    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response contains an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // World Bank API returns errors in the first array element
        if let Some(array) = response.as_array() {
            if let Some(first) = array.first() {
                if let Some(error_obj) = first.get("message") {
                    if let Some(error_array) = error_obj.as_array() {
                        if let Some(error_item) = error_array.first() {
                            let code = error_item
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or("unknown");
                            let message = error_item
                                .get("value")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unknown error");

                            return Err(ExchangeError::Api {
                                code: code.parse::<i32>().unwrap_or(0),
                                message: message.to_string(),
                            });
                        }
                    }
                }
            }
        }
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// WORLD BANK-SPECIFIC TYPES
// ═══════════════════════════════════════════════════════════════════════════

/// World Bank indicator observation (single data point)
#[derive(Debug, Clone)]
pub struct IndicatorObservation {
    pub indicator: String,
    pub country: String,
    pub date: String,
    pub value: Option<f64>,
    pub unit: Option<String>,
    pub obs_status: Option<String>,
    pub decimal: Option<i64>,
}

/// World Bank indicator metadata (full details)
#[derive(Debug, Clone)]
pub struct IndicatorMetadata {
    pub id: String,
    pub name: String,
    pub source_note: Option<String>,
    pub source_organization: Option<String>,
    pub topics: Vec<String>,
}

/// World Bank indicator info (minimal)
#[derive(Debug, Clone)]
pub struct IndicatorInfo {
    pub id: String,
    pub name: String,
    pub source_note: Option<String>,
}

/// World Bank country (full details)
#[derive(Debug, Clone)]
pub struct Country {
    pub id: String,
    pub iso2_code: String,
    pub name: String,
    pub region: Option<String>,
    pub income_level: Option<String>,
    pub lending_type: Option<String>,
    pub capital_city: Option<String>,
    pub longitude: Option<String>,
    pub latitude: Option<String>,
}

/// World Bank country info (minimal)
#[derive(Debug, Clone)]
pub struct CountryInfo {
    pub id: String,
    pub iso2_code: String,
    pub name: String,
}

/// World Bank topic (full details)
#[derive(Debug, Clone)]
pub struct Topic {
    pub id: String,
    pub value: String,
    pub source_note: Option<String>,
}

/// World Bank topic info (minimal)
#[derive(Debug, Clone)]
pub struct TopicInfo {
    pub id: String,
    pub value: String,
}

/// World Bank source (full details)
#[derive(Debug, Clone)]
pub struct Source {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub url: Option<String>,
}

/// World Bank source info (minimal)
#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub id: String,
    pub name: String,
}

/// World Bank income level
#[derive(Debug, Clone)]
pub struct IncomeLevel {
    pub id: String,
    pub value: String,
}

/// World Bank lending type
#[derive(Debug, Clone)]
pub struct LendingType {
    pub id: String,
    pub value: String,
}

/// Pagination info
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: i64,
    pub pages: i64,
    pub per_page: i64,
    pub total: i64,
}
