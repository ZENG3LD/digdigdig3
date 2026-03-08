//! UN COMTRADE response parsers
//!
//! Parse JSON responses to domain types based on COMTRADE API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct ComtradeParser;

impl ComtradeParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for COMTRADE API errors in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check for error field
        if let Some(error) = response.get("error") {
            if let Some(error_str) = error.as_str() {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: error_str.to_string(),
                });
            }
        }

        // Check for message field (sometimes used for errors)
        if let Some(message) = response.get("message") {
            if let Some(msg_str) = message.as_str() {
                if msg_str.to_lowercase().contains("error")
                    || msg_str.to_lowercase().contains("invalid")
                {
                    return Err(ExchangeError::Api {
                        code: -1,
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

    fn require_str<'a>(obj: &'a Value, key: &str) -> ExchangeResult<&'a str> {
        obj.get(key)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", key)))
    }

    fn get_str<'a>(obj: &'a Value, key: &str) -> Option<&'a str> {
        obj.get(key).and_then(|v| v.as_str())
    }

    fn _require_u64(obj: &Value, key: &str) -> ExchangeResult<u64> {
        obj.get(key)
            .and_then(|v| v.as_u64())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid field: {}", key)))
    }

    fn get_u64(obj: &Value, key: &str) -> Option<u64> {
        obj.get(key).and_then(|v| v.as_u64())
    }

    fn get_f64(obj: &Value, key: &str) -> Option<f64> {
        obj.get(key).and_then(|v| v.as_f64())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // COMTRADE-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse trade data response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "elapsedTime": "0.5s",
    ///   "count": 100,
    ///   "data": [{
    ///     "reporterCode": 842,
    ///     "reporterDesc": "United States",
    ///     "partnerCode": 156,
    ///     "partnerDesc": "China",
    ///     "cmdCode": "TOTAL",
    ///     "cmdDesc": "Total",
    ///     "flowCode": "M",
    ///     "flowDesc": "Import",
    ///     "period": "2024",
    ///     "primaryValue": 500000000000
    ///   }]
    /// }
    /// ```
    pub fn parse_trade_data(response: &Value) -> ExchangeResult<Vec<TradeRecord>> {
        let data = response
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' array".to_string()))?;

        data.iter()
            .map(|record| {
                Ok(TradeRecord {
                    reporter_code: Self::get_u64(record, "reporterCode"),
                    reporter_desc: Self::get_str(record, "reporterDesc").map(|s| s.to_string()),
                    partner_code: Self::get_u64(record, "partnerCode"),
                    partner_desc: Self::get_str(record, "partnerDesc").map(|s| s.to_string()),
                    cmd_code: Self::get_str(record, "cmdCode").map(|s| s.to_string()),
                    cmd_desc: Self::get_str(record, "cmdDesc").map(|s| s.to_string()),
                    flow_code: Self::get_str(record, "flowCode").map(|s| s.to_string()),
                    flow_desc: Self::get_str(record, "flowDesc").map(|s| s.to_string()),
                    period: Self::get_str(record, "period").map(|s| s.to_string()),
                    primary_value: Self::get_f64(record, "primaryValue"),
                    qty: Self::get_f64(record, "qty"),
                    net_wgt: Self::get_f64(record, "netWgt"),
                    gross_wgt: Self::get_f64(record, "grossWgt"),
                })
            })
            .collect()
    }

    /// Parse metadata response (reporters, partners, commodity codes, etc.)
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "count": 295,
    ///   "results": [{
    ///     "id": "842",
    ///     "text": "United States of America"
    ///   }]
    /// }
    /// ```
    pub fn parse_metadata(response: &Value) -> ExchangeResult<Vec<MetadataEntry>> {
        let results = response
            .get("results")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'results' array".to_string()))?;

        results
            .iter()
            .map(|entry| {
                Ok(MetadataEntry {
                    id: Self::require_str(entry, "id")?.to_string(),
                    text: Self::require_str(entry, "text")?.to_string(),
                })
            })
            .collect()
    }
}

// ═══════════════════════════════════════════════════════════════════════
// DOMAIN TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Trade record from COMTRADE API
#[derive(Debug, Clone)]
pub struct TradeRecord {
    pub reporter_code: Option<u64>,
    pub reporter_desc: Option<String>,
    pub partner_code: Option<u64>,
    pub partner_desc: Option<String>,
    pub cmd_code: Option<String>,
    pub cmd_desc: Option<String>,
    pub flow_code: Option<String>,
    pub flow_desc: Option<String>,
    pub period: Option<String>,
    pub primary_value: Option<f64>,
    pub qty: Option<f64>,
    pub net_wgt: Option<f64>,
    pub gross_wgt: Option<f64>,
}

/// Metadata entry (country, commodity code, etc.)
#[derive(Debug, Clone)]
pub struct MetadataEntry {
    pub id: String,
    pub text: String,
}
