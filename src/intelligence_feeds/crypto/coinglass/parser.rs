//! # Coinglass Response Parser
//!
//! Парсинг JSON ответов от Coinglass API V4.
//!
//! Coinglass специализируется на derivatives analytics,
//! поэтому data structures отличаются от обычных MarketData типов.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
};

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTOM DATA STRUCTURES FOR DERIVATIVES ANALYTICS
// ═══════════════════════════════════════════════════════════════════════════════

/// Standard Coinglass API response wrapper
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CoinglassResponse<T> {
    pub code: String,
    pub msg: String,
    pub success: bool,
    #[serde(default)]
    pub data: Option<T>,
}

/// Liquidation event data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LiquidationData {
    pub t: i64,                      // timestamp (seconds)
    pub symbol: String,              // "BTC", "ETH", etc.
    pub side: String,                // "long" or "short"
    pub price: String,               // liquidation price
    pub quantity: String,            // liquidation quantity
    pub value_usd: String,           // liquidation value in USD
    #[serde(default)]
    pub exchange: Option<String>,    // exchange name
}

/// Open Interest OHLC data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OpenInterestOhlc {
    pub t: i64,       // timestamp (seconds)
    pub o: String,    // open
    pub h: String,    // high
    pub l: String,    // low
    pub c: String,    // close
}

/// Funding Rate data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FundingRateData {
    pub t: i64,                      // timestamp (seconds)
    pub symbol: String,              // "BTC", "ETH", etc.
    pub exchange: String,            // exchange name
    pub funding_rate: String,        // current funding rate
    #[serde(default)]
    pub next_funding_time: Option<i64>, // next funding timestamp
}

/// Long/Short Ratio data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LongShortRatio {
    pub t: i64,                      // timestamp (seconds)
    pub long_rate: String,           // long ratio (0-1)
    pub short_rate: String,          // short ratio (0-1)
    #[serde(default)]
    pub long_account: Option<String>, // long account count
    #[serde(default)]
    pub short_account: Option<String>, // short account count
}

/// Supported coins response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct SupportedCoins {
    pub coins: Vec<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PARSER
// ═══════════════════════════════════════════════════════════════════════════════

/// Парсер ответов Coinglass API
pub struct CoinglassParser;

impl CoinglassParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check if response is successful
    pub fn is_success(response: &Value) -> bool {
        response.get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    /// Extract error message from response
    pub fn extract_error(response: &Value) -> String {
        response.get("msg")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown error")
            .to_string()
    }

    /// Extract data field from response
    pub fn extract_data(response: &Value) -> ExchangeResult<&Value> {
        if !Self::is_success(response) {
            let error_msg = Self::extract_error(response);
            let error_code = response.get("code")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<i32>().ok())
                .unwrap_or(0);
            return Err(ExchangeError::Api {
                code: error_code,
                message: error_msg,
            });
        }

        response.get("data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'data' field".to_string()))
    }

    /// Parse f64 from string or number
    fn _parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Get f64 from field
    fn _get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::_parse_f64)
    }

    /// Get string from field
    fn _get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Get i64 from field
    fn _get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key)
            .and_then(|v| v.as_i64())
            .or_else(|| data.get(key).and_then(|v| v.as_str()).and_then(|s| s.parse().ok()))
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // SUPPORTED COINS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse supported coins list
    pub fn parse_supported_coins(response: &Value) -> ExchangeResult<Vec<String>> {
        let data = Self::extract_data(response)?;

        // Data can be an array directly or wrapped in an object
        let coins_array = if let Some(arr) = data.as_array() {
            arr
        } else if let Some(obj) = data.as_object() {
            // Try to find array in object fields
            obj.values()
                .find_map(|v| v.as_array())
                .ok_or_else(|| ExchangeError::Parse("No array found in data".to_string()))?
        } else {
            return Err(ExchangeError::Parse("Data is not an array or object".to_string()));
        };

        let coins = coins_array
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();

        Ok(coins)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LIQUIDATIONS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse liquidation history
    pub fn parse_liquidations(response: &Value) -> ExchangeResult<Vec<LiquidationData>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let liquidations: Vec<LiquidationData> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse liquidations: {}", e)))?;

        Ok(liquidations)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // OPEN INTEREST
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse Open Interest OHLC data
    pub fn parse_oi_ohlc(response: &Value) -> ExchangeResult<Vec<OpenInterestOhlc>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let oi_data: Vec<OpenInterestOhlc> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse OI OHLC: {}", e)))?;

        Ok(oi_data)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FUNDING RATES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse funding rate history
    pub fn parse_funding_rates(response: &Value) -> ExchangeResult<Vec<FundingRateData>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let funding_rates: Vec<FundingRateData> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse funding rates: {}", e)))?;

        Ok(funding_rates)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // LONG/SHORT RATIOS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse long/short ratio history
    pub fn parse_long_short_ratio(response: &Value) -> ExchangeResult<Vec<LongShortRatio>> {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        let ratios: Vec<LongShortRatio> = serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse long/short ratios: {}", e)))?;

        Ok(ratios)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // GENERIC PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse generic array response
    pub fn parse_array<T>(response: &Value) -> ExchangeResult<Vec<T>>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = Self::extract_data(response)?;
        let arr = data.as_array()
            .ok_or_else(|| ExchangeError::Parse("'data' is not an array".to_string()))?;

        serde_json::from_value(Value::Array(arr.clone()))
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse array: {}", e)))
    }

    /// Parse generic object response
    pub fn parse_object<T>(response: &Value) -> ExchangeResult<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let data = Self::extract_data(response)?;
        serde_json::from_value(data.clone())
            .map_err(|e| ExchangeError::Parse(format!("Failed to parse object: {}", e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "success": true,
            "data": ["BTC", "ETH", "SOL"]
        });

        assert!(CoinglassParser::is_success(&response));
        let data = CoinglassParser::extract_data(&response).unwrap();
        assert!(data.is_array());
    }

    #[test]
    fn test_error_response() {
        let response = json!({
            "code": "30001",
            "msg": "API key missing",
            "success": false
        });

        assert!(!CoinglassParser::is_success(&response));
        let error = CoinglassParser::extract_data(&response);
        assert!(error.is_err());
    }

    #[test]
    fn test_parse_supported_coins() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "success": true,
            "data": ["BTC", "ETH", "SOL", "XRP"]
        });

        let coins = CoinglassParser::parse_supported_coins(&response).unwrap();
        assert_eq!(coins.len(), 4);
        assert_eq!(coins[0], "BTC");
        assert_eq!(coins[1], "ETH");
    }

    #[test]
    fn test_parse_oi_ohlc() {
        let response = json!({
            "code": "0",
            "msg": "success",
            "success": true,
            "data": [
                {
                    "t": 1641522717,
                    "o": "1234567.89",
                    "h": "1245678.90",
                    "l": "1223456.78",
                    "c": "1239876.54"
                }
            ]
        });

        let oi_data = CoinglassParser::parse_oi_ohlc(&response).unwrap();
        assert_eq!(oi_data.len(), 1);
        assert_eq!(oi_data[0].t, 1641522717);
        assert_eq!(oi_data[0].o, "1234567.89");
    }
}
