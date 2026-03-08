//! Bank of Korea ECOS response parsers
//!
//! Parse JSON responses to domain types based on ECOS API response formats.
//!
//! ECOS is an economic data provider from the Bank of Korea, similar to FRED but
//! for Korean economic statistics.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct EcosParser;

// ═══════════════════════════════════════════════════════════════════════
// RESPONSE TYPES
// ═══════════════════════════════════════════════════════════════════════

/// Statistical data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticData {
    #[serde(rename = "STAT_CODE")]
    pub stat_code: String,
    #[serde(rename = "STAT_NAME")]
    pub stat_name: String,
    #[serde(rename = "ITEM_CODE1")]
    pub item_code1: Option<String>,
    #[serde(rename = "ITEM_NAME1")]
    pub item_name1: Option<String>,
    #[serde(rename = "ITEM_CODE2")]
    pub item_code2: Option<String>,
    #[serde(rename = "ITEM_NAME2")]
    pub item_name2: Option<String>,
    #[serde(rename = "ITEM_CODE3")]
    pub item_code3: Option<String>,
    #[serde(rename = "ITEM_NAME3")]
    pub item_name3: Option<String>,
    #[serde(rename = "TIME")]
    pub time: String,
    #[serde(rename = "DATA_VALUE")]
    pub data_value: String,
}

/// Key statistic metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyStatistic {
    #[serde(rename = "CLASS_NAME")]
    pub class_name: String,
    #[serde(rename = "KEYSTAT_NAME")]
    pub keystat_name: String,
    #[serde(rename = "STAT_CODE")]
    pub stat_code: String,
    #[serde(rename = "CYCLE")]
    pub cycle: String,
    #[serde(rename = "STAT_NAME")]
    pub stat_name: String,
}

/// Statistical table metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticTable {
    #[serde(rename = "STAT_CODE")]
    pub stat_code: String,
    #[serde(rename = "STAT_NAME")]
    pub stat_name: String,
    #[serde(rename = "CYCLE")]
    pub cycle: String,
    #[serde(rename = "ORG_NAME")]
    pub org_name: String,
}

/// Statistical item metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticItem {
    #[serde(rename = "STAT_CODE")]
    pub stat_code: String,
    #[serde(rename = "STAT_NAME")]
    pub stat_name: String,
    #[serde(rename = "GRP_NAME")]
    pub grp_name: String,
    #[serde(rename = "ITEM_CODE")]
    pub item_code: String,
    #[serde(rename = "ITEM_NAME")]
    pub item_name: String,
}

/// Statistical word search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticWord {
    #[serde(rename = "WORD")]
    pub word: String,
    #[serde(rename = "STAT_CODE")]
    pub stat_code: String,
    #[serde(rename = "STAT_NAME")]
    pub stat_name: String,
}

/// Statistical metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatMeta {
    #[serde(rename = "LVL")]
    pub lvl: String,
    #[serde(rename = "P_CONT_CODE")]
    pub p_cont_code: String,
    #[serde(rename = "CONT_CODE")]
    pub cont_code: String,
    #[serde(rename = "CONT_NAME")]
    pub cont_name: String,
}

impl EcosParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ERROR CHECKING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check for ECOS API errors
    ///
    /// ECOS returns errors in the response JSON
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check for error fields in various possible locations
        if let Some(error) = response.get("RESULT") {
            if let Some(code) = error.get("CODE").and_then(|v| v.as_str()) {
                if code != "INFO-000" {
                    let message = error
                        .get("MESSAGE")
                        .and_then(|v| v.as_str())
                        .unwrap_or("Unknown ECOS error");
                    return Err(ExchangeError::Api {
                        code: -1,
                        message: format!("ECOS error {}: {}", code, message),
                    });
                }
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ECOS-SPECIFIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse statistical data response
    ///
    /// Example response:
    /// ```json
    /// {
    ///   "StatisticSearch": {
    ///     "list_total_count": 100,
    ///     "row": [
    ///       {
    ///         "STAT_CODE": "200Y001",
    ///         "STAT_NAME": "Gross Domestic Product",
    ///         "ITEM_CODE1": "10101",
    ///         "ITEM_NAME1": "GDP",
    ///         "TIME": "2023Q4",
    ///         "DATA_VALUE": "1234.5"
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    pub fn parse_statistical_data(response: &Value) -> ExchangeResult<Vec<StatisticData>> {
        let data = response
            .get("StatisticSearch")
            .ok_or_else(|| ExchangeError::Parse("Missing 'StatisticSearch' field".to_string()))?;

        let rows = data
            .get("row")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'row' array".to_string()))?;

        serde_json::from_value(Value::Array(rows.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse statistical data: {}", e)))
    }

    /// Parse key statistics list
    pub fn parse_key_statistics(response: &Value) -> ExchangeResult<Vec<KeyStatistic>> {
        let data = response
            .get("KeyStatisticList")
            .ok_or_else(|| ExchangeError::Parse("Missing 'KeyStatisticList' field".to_string()))?;

        let rows = data
            .get("row")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'row' array".to_string()))?;

        serde_json::from_value(Value::Array(rows.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse key statistics: {}", e)))
    }

    /// Parse statistical table list
    pub fn parse_statistic_tables(response: &Value) -> ExchangeResult<Vec<StatisticTable>> {
        let data = response
            .get("StatisticTableList")
            .ok_or_else(|| ExchangeError::Parse("Missing 'StatisticTableList' field".to_string()))?;

        let rows = data
            .get("row")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'row' array".to_string()))?;

        serde_json::from_value(Value::Array(rows.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse statistic tables: {}", e)))
    }

    /// Parse statistical item list
    pub fn parse_statistic_items(response: &Value) -> ExchangeResult<Vec<StatisticItem>> {
        let data = response
            .get("StatisticItemList")
            .ok_or_else(|| ExchangeError::Parse("Missing 'StatisticItemList' field".to_string()))?;

        let rows = data
            .get("row")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'row' array".to_string()))?;

        serde_json::from_value(Value::Array(rows.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse statistic items: {}", e)))
    }

    /// Parse statistical word search results
    pub fn parse_statistic_words(response: &Value) -> ExchangeResult<Vec<StatisticWord>> {
        let data = response
            .get("StatisticWord")
            .ok_or_else(|| ExchangeError::Parse("Missing 'StatisticWord' field".to_string()))?;

        let rows = data
            .get("row")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'row' array".to_string()))?;

        serde_json::from_value(Value::Array(rows.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse statistic words: {}", e)))
    }

    /// Parse statistical metadata
    pub fn parse_stat_meta(response: &Value) -> ExchangeResult<Vec<StatMeta>> {
        let data = response
            .get("StatMeta")
            .ok_or_else(|| ExchangeError::Parse("Missing 'StatMeta' field".to_string()))?;

        let rows = data
            .get("row")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'row' array".to_string()))?;

        serde_json::from_value(Value::Array(rows.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse stat meta: {}", e)))
    }
}
