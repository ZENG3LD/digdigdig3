//! Alpha Vantage response parser
//!
//! Handles parsing of various Alpha Vantage API responses

use serde::{Deserialize, Serialize};
use crate::core::types::{ExchangeError, ExchangeResult};

/// Parser for Alpha Vantage API responses
pub struct AlphaVantageParser;

impl AlphaVantageParser {
    /// Check for API errors in response
    pub fn check_error(json: &serde_json::Value) -> ExchangeResult<()> {
        // Alpha Vantage returns error messages in different formats:
        // 1. {"Error Message": "..."}
        // 2. {"Note": "..."}  (rate limit)
        // 3. {"Information": "..."} (API limit)

        if let Some(error_msg) = json.get("Error Message").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: 400,
                message: error_msg.to_string(),
            });
        }

        if let Some(note) = json.get("Note").and_then(|v| v.as_str()) {
            return Err(ExchangeError::RateLimitExceeded { retry_after: None, message: note.to_string() });
        }

        if let Some(info) = json.get("Information").and_then(|v| v.as_str()) {
            return Err(ExchangeError::RateLimitExceeded { retry_after: None, message: info.to_string() });
        }

        Ok(())
    }

    /// Parse global quote response
    pub fn parse_global_quote(json: &serde_json::Value) -> ExchangeResult<GlobalQuote> {
        json.get("Global Quote")
            .ok_or_else(|| ExchangeError::Parse("Missing 'Global Quote' field".to_string()))?
            .as_object()
            .ok_or_else(|| ExchangeError::Parse("Invalid quote format".to_string()))?;

        serde_json::from_value(json["Global Quote"].clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse quote: {}", e)))
    }

    /// Parse time series data (intraday/daily/weekly/monthly)
    pub fn parse_time_series(json: &serde_json::Value, key_prefix: &str) -> ExchangeResult<Vec<TimeSeriesEntry>> {
        let time_series_key = json.as_object()
            .and_then(|obj| obj.keys().find(|k| k.starts_with(key_prefix)))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing time series key starting with '{}'", key_prefix)))?;

        let time_series = json.get(time_series_key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Invalid time series format".to_string()))?;

        let mut entries = Vec::new();
        for (timestamp, values) in time_series {
            let entry = TimeSeriesEntry {
                timestamp: timestamp.clone(),
                open: parse_field(values, "1. open")?,
                high: parse_field(values, "2. high")?,
                low: parse_field(values, "3. low")?,
                close: parse_field(values, "4. close")?,
                volume: parse_field(values, "5. volume").ok(),
            };
            entries.push(entry);
        }

        Ok(entries)
    }

    /// Parse symbol search results
    pub fn parse_symbol_search(json: &serde_json::Value) -> ExchangeResult<Vec<SymbolMatch>> {
        let matches = json.get("bestMatches")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'bestMatches' field".to_string()))?;

        serde_json::from_value(serde_json::Value::Array(matches.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse symbol matches: {}", e)))
    }

    /// Parse forex exchange rate
    pub fn parse_fx_rate(json: &serde_json::Value) -> ExchangeResult<ForexRate> {
        serde_json::from_value(json["Realtime Currency Exchange Rate"].clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse forex rate: {}", e)))
    }

    /// Parse crypto rating
    pub fn parse_crypto_rating(json: &serde_json::Value) -> ExchangeResult<CryptoRating> {
        serde_json::from_value(json["Crypto Rating (FCAS)"].clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse crypto rating: {}", e)))
    }

    /// Parse economic indicator data
    pub fn parse_economic_data(json: &serde_json::Value) -> ExchangeResult<Vec<EconomicDataPoint>> {
        let data = json.get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))?;

        serde_json::from_value(serde_json::Value::Array(data.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse economic data: {}", e)))
    }

    /// Parse technical indicator data
    pub fn parse_technical_indicator(json: &serde_json::Value, key_prefix: &str) -> ExchangeResult<Vec<TechnicalIndicatorEntry>> {
        let indicator_key = json.as_object()
            .and_then(|obj| obj.keys().find(|k| k.starts_with(key_prefix)))
            .ok_or_else(|| ExchangeError::Parse(format!("Missing indicator key starting with '{}'", key_prefix)))?;

        let indicator_data = json.get(indicator_key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Invalid indicator format".to_string()))?;

        let mut entries = Vec::new();
        for (timestamp, values) in indicator_data {
            if let Some(obj) = values.as_object() {
                let mut indicator_values = std::collections::HashMap::new();
                for (k, v) in obj {
                    if let Some(val_str) = v.as_str() {
                        if let Ok(val) = val_str.parse::<f64>() {
                            indicator_values.insert(k.clone(), val);
                        }
                    }
                }
                entries.push(TechnicalIndicatorEntry {
                    timestamp: timestamp.clone(),
                    values: indicator_values,
                });
            }
        }

        Ok(entries)
    }
}

// Helper function to parse numeric fields
fn parse_field(obj: &serde_json::Value, field: &str) -> ExchangeResult<f64> {
    obj.get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| ExchangeError::Parse(format!("Missing field '{}'", field)))?
        .parse::<f64>()
        .map_err(|e| ExchangeError::Parse(format!("Failed to parse '{}': {}", field, e)))
}

// ═══════════════════════════════════════════════════════════════════════
// Response Types
// ═══════════════════════════════════════════════════════════════════════

/// Global quote (real-time stock quote)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalQuote {
    #[serde(rename = "01. symbol")]
    pub symbol: String,
    #[serde(rename = "05. price")]
    pub price: String,
    #[serde(rename = "06. volume")]
    pub volume: String,
    #[serde(rename = "09. change")]
    pub change: String,
    #[serde(rename = "10. change percent")]
    pub change_percent: String,
}

/// Time series entry (OHLCV data)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesEntry {
    pub timestamp: String,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: Option<f64>,
}

/// Symbol search match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMatch {
    #[serde(rename = "1. symbol")]
    pub symbol: String,
    #[serde(rename = "2. name")]
    pub name: String,
    #[serde(rename = "3. type")]
    pub symbol_type: String,
    #[serde(rename = "4. region")]
    pub region: String,
    #[serde(rename = "8. currency")]
    pub currency: String,
}

/// Forex exchange rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForexRate {
    #[serde(rename = "1. From_Currency Code")]
    pub from_currency: String,
    #[serde(rename = "3. To_Currency Code")]
    pub to_currency: String,
    #[serde(rename = "5. Exchange Rate")]
    pub exchange_rate: String,
    #[serde(rename = "6. Last Refreshed")]
    pub last_refreshed: String,
}

/// Crypto rating/health score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoRating {
    #[serde(rename = "1. symbol")]
    pub symbol: String,
    #[serde(rename = "2. name")]
    pub name: String,
    #[serde(rename = "3. fcas rating")]
    pub fcas_rating: String,
    #[serde(rename = "4. fcas score")]
    pub fcas_score: String,
}

/// Economic data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicDataPoint {
    pub date: String,
    pub value: String,
}

/// Technical indicator entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnicalIndicatorEntry {
    pub timestamp: String,
    pub values: std::collections::HashMap<String, f64>,
}
