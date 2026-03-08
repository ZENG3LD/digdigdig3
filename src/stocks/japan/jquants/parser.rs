//! # JQuants Response Parsers
//!
//! Parse JSON responses to domain types

use serde_json::Value;
use crate::core::types::{Kline, Ticker, ExchangeError, ExchangeResult};

pub struct JQuantsParser;

impl JQuantsParser {
    // ═══════════════════════════════════════════════════════════════════════
    // AUTHENTICATION
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse refresh token response from /token/auth_user
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "refreshToken": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
    /// }
    /// ```
    pub fn parse_refresh_token(response: &Value) -> ExchangeResult<String> {
        response
            .get("refreshToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing 'refreshToken' field".to_string()))
    }

    /// Parse ID token response from /token/auth_refresh
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "idToken": "eyJraWQiOiJhYmNkZWYxMjM0NTY3ODkwIi..."
    /// }
    /// ```
    pub fn parse_id_token(response: &Value) -> ExchangeResult<String> {
        response
            .get("idToken")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| ExchangeError::Parse("Missing 'idToken' field".to_string()))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STOCK PRICE DATA
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse daily quotes response
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "daily_quotes": [
    ///     {
    ///       "Date": "2024-01-15",
    ///       "Code": "7203",
    ///       "Open": 2500.0,
    ///       "High": 2550.0,
    ///       "Low": 2480.0,
    ///       "Close": 2530.0,
    ///       "Volume": 12345678,
    ///       ...
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_daily_quotes(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let quotes = response
            .get("daily_quotes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'daily_quotes' array".to_string()))?;

        quotes
            .iter()
            .map(|quote| {
                let date_str = Self::require_str(quote, "Date")?;
                let open_time = Self::parse_date_to_timestamp(date_str)?;

                Ok(Kline {
                    open_time,
                    open: Self::require_f64(quote, "Open")?,
                    high: Self::require_f64(quote, "High")?,
                    low: Self::require_f64(quote, "Low")?,
                    close: Self::require_f64(quote, "Close")?,
                    volume: Self::require_f64(quote, "Volume")?,
                    quote_volume: Self::get_f64(quote, "TurnoverValue"),
                    close_time: None,
                    trades: None,
                })
            })
            .collect()
    }

    /// Parse single daily quote to get current price
    pub fn parse_current_price(response: &Value) -> ExchangeResult<f64> {
        let quotes = response
            .get("daily_quotes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'daily_quotes' array".to_string()))?;

        // Get the most recent quote (last in array)
        let latest = quotes
            .last()
            .ok_or_else(|| ExchangeError::Parse("Empty daily_quotes array".to_string()))?;

        Self::require_f64(latest, "Close")
    }

    /// Parse daily quote to ticker
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        let quotes = response
            .get("daily_quotes")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'daily_quotes' array".to_string()))?;

        let latest = quotes
            .last()
            .ok_or_else(|| ExchangeError::Parse("Empty daily_quotes array".to_string()))?;

        let date_str = Self::require_str(latest, "Date")?;
        let timestamp = Self::parse_date_to_timestamp(date_str)?;

        let close = Self::require_f64(latest, "Close")?;
        let high = Self::get_f64(latest, "High");
        let low = Self::get_f64(latest, "Low");
        let volume = Self::get_f64(latest, "Volume");

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price: close,
            bid_price: None, // JQuants doesn't provide bid/ask
            ask_price: None,
            high_24h: high,
            low_24h: low,
            volume_24h: volume,
            quote_volume_24h: Self::get_f64(latest, "TurnoverValue"),
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════
    // LISTED ISSUES / SYMBOLS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse listed info response
    ///
    /// Expected format:
    /// ```json
    /// {
    ///   "info": [
    ///     {
    ///       "Code": "7203",
    ///       "CompanyName": "トヨタ自動車株式会社",
    ///       "CompanyNameEnglish": "Toyota Motor Corporation",
    ///       ...
    ///     }
    ///   ]
    /// }
    /// ```
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let info = response
            .get("info")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'info' array".to_string()))?;

        Ok(info
            .iter()
            .filter_map(|item| item.get("Code").and_then(|c| c.as_str()))
            .map(|code| code.to_string())
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_i64().map(|i| i as f64))
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_i64().map(|i| i as f64))
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }

    fn require_str<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a str> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    /// Parse YYYY-MM-DD date string to Unix timestamp (milliseconds)
    fn parse_date_to_timestamp(date_str: &str) -> ExchangeResult<i64> {
        use chrono::NaiveDate;

        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map_err(|e| ExchangeError::Parse(format!("Invalid date format '{}': {}", date_str, e)))
            .map(|date| {
                // Convert to midnight UTC timestamp in milliseconds
                date.and_hms_opt(0, 0, 0)
                    .expect("Valid time 00:00:00")
                    .and_utc()
                    .timestamp_millis()
            })
    }

    /// Parse pagination key from response
    pub fn get_pagination_key(response: &Value) -> Option<String> {
        response
            .get("pagination_key")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_id_token() {
        let response = json!({
            "idToken": "test_token_123"
        });

        let token = JQuantsParser::parse_id_token(&response).unwrap();
        assert_eq!(token, "test_token_123");
    }

    #[test]
    fn test_parse_current_price() {
        let response = json!({
            "daily_quotes": [
                {
                    "Date": "2024-01-15",
                    "Code": "7203",
                    "Open": 2500.0,
                    "High": 2550.0,
                    "Low": 2480.0,
                    "Close": 2530.0,
                    "Volume": 12345678
                }
            ]
        });

        let price = JQuantsParser::parse_current_price(&response).unwrap();
        assert_eq!(price, 2530.0);
    }

    #[test]
    fn test_parse_symbols() {
        let response = json!({
            "info": [
                {
                    "Code": "7203",
                    "CompanyName": "トヨタ自動車株式会社"
                },
                {
                    "Code": "6758",
                    "CompanyName": "ソニーグループ株式会社"
                }
            ]
        });

        let symbols = JQuantsParser::parse_symbols(&response).unwrap();
        assert_eq!(symbols, vec!["7203", "6758"]);
    }
}
