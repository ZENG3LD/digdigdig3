//! KRX response parsers
//!
//! Parse JSON responses to domain types based on KRX response formats.
//!
//! CRITICAL: KRX returns numeric values as comma-formatted strings.
//! Example: "76,200" instead of 76200
//!
//! Dates are returned in YYYY/MM/DD format instead of input YYYYMMDD format.
//!
//! NOTE: The new Open API may use different field names than the old Data Marketplace.
//! This parser attempts to handle both formats where possible.

use serde_json::Value;
use crate::core::types::*;

pub struct KrxParser;

impl KrxParser {
    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price from ticker data
    ///
    /// KRX doesn't have a simple "price" endpoint.
    /// Price is extracted from ticker or OHLCV data.
    pub fn _parse_price(response: &Value) -> ExchangeResult<f64> {
        // Try to get closing price from various possible fields
        if let Some(price) = response.get("TDD_CLSPRC") {
            return Self::parse_krx_number(price);
        }
        if let Some(price) = response.get("close") {
            return Self::parse_krx_number(price);
        }
        if let Some(price) = response.get("CLSPRC_IDX") {
            return Self::parse_krx_number(price);
        }

        Err(ExchangeError::Parse("Price field not found".to_string()))
    }

    /// Parse ticker (24h stats)
    ///
    /// KRX provides ticker-like data through OHLCV endpoint
    pub fn _parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: Self::_require_krx_number(response, "TDD_CLSPRC")?,
            bid_price: None, // KRX doesn't provide bid/ask in ticker
            ask_price: None,
            high_24h: Self::get_krx_number(response, "TDD_HGPRC"),
            low_24h: Self::get_krx_number(response, "TDD_LWPRC"),
            volume_24h: Self::get_krx_number(response, "ACC_TRDVOL"),
            quote_volume_24h: Self::get_krx_number(response, "ACC_TRDVAL"),
            price_change_24h: Self::get_krx_number(response, "CMPPRVDD_PRC"),
            price_change_percent_24h: Self::get_krx_number(response, "FLUC_RT"),
            timestamp: chrono::Utc::now().timestamp_millis(),
        })
    }

    /// Parse klines/candles from OHLCV response
    ///
    /// Old format (Data Marketplace):
    /// ```json
    /// {
    ///   "OutBlock_1": [
    ///     {
    ///       "TRD_DD": "2026/01/20",
    ///       "TDD_OPNPRC": "75,000",
    ///       "TDD_HGPRC": "76,500",
    ///       "TDD_LWPRC": "74,800",
    ///       "TDD_CLSPRC": "76,200",
    ///       "ACC_TRDVOL": "12,345,678",
    ///       "ACC_TRDVAL": "935,432,100,000"
    ///     }
    ///   ]
    /// }
    /// ```
    ///
    /// New format (Open API) - TODO: Update when actual format is known
    /// Assuming similar structure but possibly different wrapper or field names
    pub fn parse_klines(response: &Value, symbol_filter: &str) -> ExchangeResult<Vec<Kline>> {
        // Try new API format first (direct array or different wrapper)
        // TODO: Update this when actual API response format is known
        let array = if let Some(data) = response.get("data").and_then(|v| v.as_array()) {
            // New API might use "data" wrapper
            data
        } else if let Some(items) = response.get("items").and_then(|v| v.as_array()) {
            // Or "items" wrapper
            items
        } else if let Some(block) = response.get("OutBlock_1").and_then(|v| v.as_array()) {
            // Fall back to old format
            block
        } else if let Some(arr) = response.as_array() {
            // Or direct array
            arr
        } else {
            return Err(ExchangeError::Parse(
                "Could not find klines array in response (tried: data, items, OutBlock_1, direct array)".to_string()
            ));
        };

        if array.is_empty() {
            return Ok(Vec::new());
        }

        // Filter by symbol if the response contains multiple symbols
        let filtered: Vec<&Value> = array
            .iter()
            .filter(|item| {
                // If ISU_SRT_CD field exists, filter by it
                if let Some(code) = item.get("ISU_SRT_CD").and_then(|v| v.as_str()) {
                    code == symbol_filter || symbol_filter.is_empty()
                } else {
                    // No symbol field, assume it's the right data
                    true
                }
            })
            .collect();

        Ok(filtered
            .iter()
            .filter_map(|candle| {
                // Try multiple date field names
                let date_str = Self::get_str(candle, "TRD_DD")
                    .or_else(|| Self::get_str(candle, "basDd"))
                    .or_else(|| Self::get_str(candle, "date"))?;

                let timestamp = Self::parse_krx_date(date_str).ok()?;

                // Try to parse OHLCV with multiple field name possibilities
                Some(Kline {
                    open_time: timestamp,
                    open: Self::get_krx_number(candle, "TDD_OPNPRC")
                        .or_else(|| Self::get_krx_number(candle, "open"))?,
                    high: Self::get_krx_number(candle, "TDD_HGPRC")
                        .or_else(|| Self::get_krx_number(candle, "high"))?,
                    low: Self::get_krx_number(candle, "TDD_LWPRC")
                        .or_else(|| Self::get_krx_number(candle, "low"))?,
                    close: Self::get_krx_number(candle, "TDD_CLSPRC")
                        .or_else(|| Self::get_krx_number(candle, "close"))?,
                    volume: Self::get_krx_number(candle, "ACC_TRDVOL")
                        .or_else(|| Self::get_krx_number(candle, "volume"))?,
                    quote_volume: Self::get_krx_number(candle, "ACC_TRDVAL")
                        .or_else(|| Self::get_krx_number(candle, "quote_volume")),
                    close_time: None,
                    trades: None,
                })
            })
            .collect())
    }

    /// Parse stock list from ticker list endpoint
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "OutBlock_1": [
    ///     {
    ///       "ISU_SRT_CD": "005930",
    ///       "ISU_CD": "KR7005930003",
    ///       "ISU_NM": "삼성전자",
    ///       "MKT_NM": "KOSPI"
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let array = response
            .get("OutBlock_1")
            .or_else(|| response.get("data"))
            .or_else(|| response.get("items"))
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing data array".to_string()))?;

        Ok(array
            .iter()
            .filter_map(|v| {
                v.get("ISU_SRT_CD")
                    .or_else(|| v.get("srtnCd"))
                    .or_else(|| v.get("ticker"))
                    .and_then(|s| s.as_str())
            })
            .map(|s| s.to_string())
            .collect())
    }

    /// Parse stock information from Public Data Portal
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "response": {
    ///     "body": {
    ///       "items": {
    ///         "item": [
    ///           {
    ///             "basDt": "20260120",
    ///             "srtnCd": "005930",
    ///             "isinCd": "KR7005930003",
    ///             "itmsNm": "삼성전자",
    ///             "mrktCtg": "KOSPI"
    ///           }
    ///         ]
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_stock_info(response: &Value) -> ExchangeResult<Vec<Value>> {
        let items = response
            .get("response")
            .and_then(|r| r.get("body"))
            .and_then(|b| b.get("items"))
            .and_then(|i| i.get("item"))
            .and_then(|i| i.as_array())
            .ok_or_else(|| ExchangeError::Parse("Invalid stock info response structure".to_string()))?;

        Ok(items.clone())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse KRX comma-formatted number string
    ///
    /// Examples:
    /// - "76,200" -> 76200.0
    /// - "12,345,678" -> 12345678.0
    /// - "935,432,100,000" -> 935432100000.0
    /// - "-1,200" -> -1200.0
    fn parse_krx_number(value: &Value) -> ExchangeResult<f64> {
        if let Some(num) = value.as_f64() {
            return Ok(num);
        }

        if let Some(s) = value.as_str() {
            let cleaned = s.replace(',', "").trim().to_string();
            cleaned
                .parse::<f64>()
                .map_err(|_| ExchangeError::Parse(format!("Invalid number format: '{}'", s)))
        } else {
            Err(ExchangeError::Parse(format!(
                "Expected number or string, got {:?}",
                value
            )))
        }
    }

    /// Require KRX number field (returns error if missing)
    fn _require_krx_number(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing field '{}'", field)))
            .and_then(Self::parse_krx_number)
    }

    /// Get optional KRX number field
    fn get_krx_number(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| Self::parse_krx_number(v).ok())
    }

    /// Get optional string field
    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    /// Parse KRX date format
    ///
    /// KRX old format: "YYYY/MM/DD" (e.g., "2026/01/20")
    /// KRX new format: "YYYYMMDD" (e.g., "20260120")
    /// Convert to Unix timestamp in milliseconds
    fn parse_krx_date(date_str: &str) -> ExchangeResult<i64> {
        use chrono::NaiveDate;

        // Try YYYY/MM/DD format first (old API)
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y/%m/%d") {
            return Ok(date.and_hms_opt(0, 0, 0)
                .expect("Valid time 00:00:00")
                .and_utc()
                .timestamp_millis());
        }

        // Try YYYYMMDD format (new API)
        if let Ok(date) = NaiveDate::parse_from_str(date_str, "%Y%m%d") {
            return Ok(date.and_hms_opt(0, 0, 0)
                .expect("Valid time 00:00:00")
                .and_utc()
                .timestamp_millis());
        }

        Err(ExchangeError::Parse(format!(
            "Invalid date format '{}' (expected YYYY/MM/DD or YYYYMMDD)",
            date_str
        )))
    }

    /// Check for API errors in response
    ///
    /// New Open API error format:
    /// ```json
    /// {
    ///   "respCode": "401",
    ///   "respMsg": "Unauthorized Key"
    /// }
    /// ```
    ///
    /// Public Data Portal error format:
    /// ```json
    /// {
    ///   "response": {
    ///     "header": {
    ///       "resultCode": "00",
    ///       "resultMsg": "NORMAL SERVICE."
    ///     }
    ///   }
    /// }
    /// ```
    pub fn check_api_error(response: &Value) -> ExchangeResult<()> {
        // Check for new Open API error format
        if let Some(code) = response.get("respCode").and_then(|c| c.as_str()) {
            if code != "200" && code != "0" {
                let msg = response
                    .get("respMsg")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown error");

                return Err(match code {
                    "401" => ExchangeError::Auth(format!("Authentication failed: {}", msg)),
                    "403" => ExchangeError::PermissionDenied(format!("Permission denied: {}", msg)),
                    "429" => ExchangeError::RateLimit,
                    _ => ExchangeError::Api {
                        code: code.parse().unwrap_or(-1),
                        message: msg.to_string(),
                    },
                });
            }
        }

        // Check for Public Data Portal error format
        if let Some(header) = response.get("response").and_then(|r| r.get("header")) {
            if let Some(code) = header.get("resultCode").and_then(|c| c.as_str()) {
                if code != "00" {
                    let msg = header
                        .get("resultMsg")
                        .and_then(|m| m.as_str())
                        .unwrap_or("Unknown error");
                    return Err(ExchangeError::Api {
                        code: code.parse().unwrap_or(-1),
                        message: msg.to_string(),
                    });
                }
            }
        }

        // Check for generic error fields
        if let Some(error) = response.get("error") {
            let message = error
                .get("message")
                .and_then(|m| m.as_str())
                .unwrap_or("Unknown error");
            let code = error
                .get("code")
                .and_then(|c| c.as_i64())
                .unwrap_or(-1) as i32;
            return Err(ExchangeError::Api {
                code,
                message: message.to_string(),
            });
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_krx_number() {
        let val = serde_json::json!("76,200");
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), 76200.0);

        let val = serde_json::json!("12,345,678");
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), 12345678.0);

        let val = serde_json::json!("-1,200");
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), -1200.0);

        let val = serde_json::json!(12345.67);
        assert_eq!(KrxParser::parse_krx_number(&val).unwrap(), 12345.67);
    }

    #[test]
    fn test_parse_krx_date_old_format() {
        let timestamp = KrxParser::parse_krx_date("2026/01/20").unwrap();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_parse_krx_date_new_format() {
        let timestamp = KrxParser::parse_krx_date("20260120").unwrap();
        assert!(timestamp > 0);
    }

    #[test]
    fn test_check_api_error_new_format() {
        let error_response = serde_json::json!({
            "respCode": "401",
            "respMsg": "Unauthorized Key"
        });

        let result = KrxParser::check_api_error(&error_response);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ExchangeError::Auth(_)));
    }

    #[test]
    fn test_check_api_error_success() {
        let success_response = serde_json::json!({
            "respCode": "200",
            "data": []
        });

        let result = KrxParser::check_api_error(&success_response);
        assert!(result.is_ok());
    }
}
