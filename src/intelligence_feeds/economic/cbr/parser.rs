//! CBR response parsers
//!
//! Manual XML parsing for CBR legacy endpoints.
//! No external XML crate dependencies - uses simple string operations.

use crate::core::types::{ExchangeError, ExchangeResult};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════
// DATA STRUCTURES
// ═══════════════════════════════════════════════════════════════════════

/// Key rate record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyRate {
    pub date: String,
    pub rate: f64,
}

/// Currency exchange rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrencyRate {
    pub id: String,
    pub num_code: String,
    pub char_code: String,
    pub nominal: i32,
    pub name: String,
    pub value: f64,
    pub vunit_rate: f64,
}

/// Currency metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Currency {
    pub id: String,
    pub name: String,
    pub eng_name: String,
    pub nominal: i32,
    pub parent_code: String,
}

/// Daily exchange rates response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyRates {
    pub date: String,
    pub rates: Vec<CurrencyRate>,
}

/// Historical rate point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RatePoint {
    pub date: String,
    pub value: f64,
    pub nominal: i32,
}

/// Metal price
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetalPrice {
    pub date: String,
    pub code: String,
    pub value: f64,
}

/// Repo rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoRate {
    pub date: String,
    pub rate: f64,
}

/// Reserve data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReserveData {
    pub date: String,
    pub value: f64,
}

// ═══════════════════════════════════════════════════════════════════════
// PARSER UTILITIES
// ═══════════════════════════════════════════════════════════════════════

/// Extract text between XML tags
fn extract_tag_content<'a>(xml: &'a str, tag: &str) -> Option<&'a str> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    let start = xml.find(&open_tag)? + open_tag.len();
    let end = xml[start..].find(&close_tag)? + start;

    Some(&xml[start..end])
}

/// Extract attribute value from XML tag
fn extract_attribute<'a>(tag_line: &'a str, attr: &str) -> Option<&'a str> {
    let pattern = format!("{}=\"", attr);
    let start = tag_line.find(&pattern)? + pattern.len();
    let end = tag_line[start..].find('"')? + start;
    Some(&tag_line[start..end])
}

/// Parse Russian decimal format (comma separator) to f64
fn parse_russian_decimal(value: &str) -> ExchangeResult<f64> {
    let normalized = value.replace(',', ".").trim().to_string();
    normalized
        .parse::<f64>()
        .map_err(|e| ExchangeError::Parse(format!("Failed to parse decimal '{}': {}", value, e)))
}

// ═══════════════════════════════════════════════════════════════════════
// JSON PARSERS
// ═══════════════════════════════════════════════════════════════════════

/// CBR JSON parser
pub struct CbrParser;

impl CbrParser {
    /// Check for API errors in JSON response
    pub fn check_error(json: &serde_json::Value) -> ExchangeResult<()> {
        if let Some(error) = json.get("error") {
            return Err(ExchangeError::Api {
                code: -1,
                message: error.to_string(),
            });
        }
        Ok(())
    }

    /// Parse key rate response (JSON)
    pub fn parse_key_rate(json: &serde_json::Value) -> ExchangeResult<Vec<KeyRate>> {
        Self::check_error(json)?;

        // CBR key rate response is an array of records
        let records = json
            .as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        let mut rates = Vec::new();
        for record in records {
            let date = record.get("Date")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing Date".to_string()))?
                .to_string();

            let rate = record.get("Rate")
                .and_then(|v| v.as_f64())
                .ok_or_else(|| ExchangeError::Parse("Missing Rate".to_string()))?;

            rates.push(KeyRate { date, rate });
        }

        Ok(rates)
    }

    /// Parse daily rates response (JSON)
    pub fn parse_daily_json(json: &serde_json::Value) -> ExchangeResult<DailyRates> {
        Self::check_error(json)?;

        let date = json.get("Date")
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse("Missing Date".to_string()))?
            .to_string();

        let valute = json.get("Valute")
            .ok_or_else(|| ExchangeError::Parse("Missing Valute".to_string()))?;

        let mut rates = Vec::new();

        if let Some(obj) = valute.as_object() {
            for (code, data) in obj {
                let id = data.get("ID")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let num_code = data.get("NumCode")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let char_code = code.to_string();

                let nominal = data.get("Nominal")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1) as i32;

                let name = data.get("Name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();

                let value = data.get("Value")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);

                let vunit_rate = data.get("VunitRate")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(value);

                rates.push(CurrencyRate {
                    id,
                    num_code,
                    char_code,
                    nominal,
                    name,
                    value,
                    vunit_rate,
                });
            }
        }

        Ok(DailyRates { date, rates })
    }
}

// ═══════════════════════════════════════════════════════════════════════
// XML PARSERS
// ═══════════════════════════════════════════════════════════════════════

impl CbrParser {
    /// Parse daily rates (XML format)
    pub fn parse_daily_xml(xml: &str) -> ExchangeResult<DailyRates> {
        // Extract date from ValCurs tag
        let valcurs_start = xml.find("<ValCurs")
            .ok_or_else(|| ExchangeError::Parse("Missing ValCurs tag".to_string()))?;
        let valcurs_line_end = xml[valcurs_start..].find('>')
            .ok_or_else(|| ExchangeError::Parse("Malformed ValCurs tag".to_string()))?
            + valcurs_start;
        let valcurs_tag = &xml[valcurs_start..valcurs_line_end];

        let date = extract_attribute(valcurs_tag, "Date")
            .ok_or_else(|| ExchangeError::Parse("Missing Date attribute".to_string()))?
            .to_string();

        let mut rates = Vec::new();

        // Split by <Valute> tags
        for valute_section in xml.split("<Valute").skip(1) {
            if let Some(end) = valute_section.find("</Valute>") {
                let valute_xml = &valute_section[..end];

                // Extract ID attribute
                let id = if let Some(id_start) = valute_section.find("ID=\"") {
                    let id_content_start = id_start + 4;
                    if let Some(id_end) = valute_section[id_content_start..].find('"') {
                        valute_section[id_content_start..id_content_start + id_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let num_code = extract_tag_content(valute_xml, "NumCode").unwrap_or("").to_string();
                let char_code = extract_tag_content(valute_xml, "CharCode").unwrap_or("").to_string();
                let nominal = extract_tag_content(valute_xml, "Nominal")
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(1);
                let name = extract_tag_content(valute_xml, "Name").unwrap_or("").to_string();

                let value_str = extract_tag_content(valute_xml, "Value").unwrap_or("0");
                let value = parse_russian_decimal(value_str)?;

                let vunit_str = extract_tag_content(valute_xml, "VunitRate").unwrap_or(value_str);
                let vunit_rate = parse_russian_decimal(vunit_str)?;

                rates.push(CurrencyRate {
                    id,
                    num_code,
                    char_code,
                    nominal,
                    name,
                    value,
                    vunit_rate,
                });
            }
        }

        Ok(DailyRates { date, rates })
    }

    /// Parse currency list (XML)
    pub fn parse_currency_list(xml: &str) -> ExchangeResult<Vec<Currency>> {
        let mut currencies = Vec::new();

        for item_section in xml.split("<Item").skip(1) {
            if let Some(end) = item_section.find("</Item>") {
                let item_xml = &item_section[..end];

                // Extract ID attribute
                let id = if let Some(id_start) = item_section.find("ID=\"") {
                    let id_content_start = id_start + 4;
                    if let Some(id_end) = item_section[id_content_start..].find('"') {
                        item_section[id_content_start..id_content_start + id_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let name = extract_tag_content(item_xml, "Name").unwrap_or("").to_string();
                let eng_name = extract_tag_content(item_xml, "EngName").unwrap_or("").to_string();
                let nominal = extract_tag_content(item_xml, "Nominal")
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(1);
                let parent_code = extract_tag_content(item_xml, "ParentCode").unwrap_or("").to_string();

                currencies.push(Currency {
                    id,
                    name,
                    eng_name,
                    nominal,
                    parent_code,
                });
            }
        }

        Ok(currencies)
    }

    /// Parse exchange rate dynamic (XML)
    pub fn parse_rate_dynamic(xml: &str) -> ExchangeResult<Vec<RatePoint>> {
        let mut points = Vec::new();

        for record_section in xml.split("<Record").skip(1) {
            if let Some(end) = record_section.find("</Record>") {
                let record_xml = &record_section[..end];

                // Extract Date attribute
                let date = if let Some(date_start) = record_section.find("Date=\"") {
                    let date_content_start = date_start + 6;
                    if let Some(date_end) = record_section[date_content_start..].find('"') {
                        record_section[date_content_start..date_content_start + date_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let nominal = extract_tag_content(record_xml, "Nominal")
                    .and_then(|s| s.parse::<i32>().ok())
                    .unwrap_or(1);

                let value_str = extract_tag_content(record_xml, "Value").unwrap_or("0");
                let value = parse_russian_decimal(value_str)?;

                points.push(RatePoint {
                    date,
                    value,
                    nominal,
                });
            }
        }

        Ok(points)
    }

    /// Parse metal prices (XML)
    pub fn parse_metal_prices(xml: &str) -> ExchangeResult<Vec<MetalPrice>> {
        let mut prices = Vec::new();

        for record_section in xml.split("<Record").skip(1) {
            if let Some(end) = record_section.find("</Record>") {
                let record_xml = &record_section[..end];

                // Extract Date attribute
                let date = if let Some(date_start) = record_section.find("Date=\"") {
                    let date_content_start = date_start + 6;
                    if let Some(date_end) = record_section[date_content_start..].find('"') {
                        record_section[date_content_start..date_content_start + date_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let code = if let Some(code_start) = record_section.find("Code=\"") {
                    let code_content_start = code_start + 6;
                    if let Some(code_end) = record_section[code_content_start..].find('"') {
                        record_section[code_content_start..code_content_start + code_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let value_str = extract_tag_content(record_xml, "Buy").unwrap_or("0");
                let value = parse_russian_decimal(value_str)?;

                prices.push(MetalPrice {
                    date,
                    code,
                    value,
                });
            }
        }

        Ok(prices)
    }

    /// Parse simple value records (repo, reserves, monetary base, etc.)
    pub fn parse_value_records(xml: &str, value_tag: &str) -> ExchangeResult<Vec<ReserveData>> {
        let mut records = Vec::new();

        for record_section in xml.split("<Record").skip(1) {
            if let Some(end) = record_section.find("</Record>") {
                let record_xml = &record_section[..end];

                // Extract Date attribute
                let date = if let Some(date_start) = record_section.find("Date=\"") {
                    let date_content_start = date_start + 6;
                    if let Some(date_end) = record_section[date_content_start..].find('"') {
                        record_section[date_content_start..date_content_start + date_end].to_string()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                let value_str = extract_tag_content(record_xml, value_tag).unwrap_or("0");
                let value = parse_russian_decimal(value_str)?;

                records.push(ReserveData {
                    date,
                    value,
                });
            }
        }

        Ok(records)
    }
}
