//! Twelvedata response parsers
//!
//! Parse JSON responses to domain types based on Twelvedata API response formats.
//!
//! ## Important Notes
//!
//! 1. **String numerics**: Time series values returned as strings to preserve precision
//! 2. **Null handling**: Many fields may be null when data unavailable
//! 3. **Error format**: `{"code": 400, "message": "...", "status": "error"}`
//! 4. **Success format**: `{"data": [...], "status": "ok"}` or direct object

use serde_json::Value;
use crate::core::types::*;

pub struct TwelvedataParser;

impl TwelvedataParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check if response is an error
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(status) = response.get("status").and_then(|v| v.as_str()) {
            if status == "error" {
                let code = response
                    .get("code")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(0) as i32;
                let message = response
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error")
                    .to_string();

                // Handle rate limit specifically
                if code == 429 {
                    return Err(ExchangeError::RateLimitExceeded {
                        retry_after: None,
                        message,
                    });
                }

                return Err(ExchangeError::Api { code, message });
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // MARKET DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price from /price endpoint
    ///
    /// Response: `{"price": "150.25"}`
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_error(response)?;

        response
            .get("price")
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid price field".to_string()))
    }

    /// Parse ticker from /quote endpoint
    ///
    /// Response includes:
    /// - symbol, name, exchange, currency, datetime
    /// - open, high, low, close, previous_close
    /// - price (current), change, percent_change
    /// - volume, average_volume
    /// - fifty_two_week: {low, high, low_change, high_change, ...}
    /// - extended_change, extended_percent_change
    pub fn parse_ticker(response: &Value, symbol: &str) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        // Get last price (required field)
        let last_price = response
            .get("price")
            .or_else(|| response.get("close"))
            .and_then(Self::parse_float)
            .ok_or_else(|| ExchangeError::Parse("Missing last_price in quote".to_string()))?;

        // Optional fields with null handling
        let high_24h = response.get("high").and_then(Self::parse_float);
        let low_24h = response.get("low").and_then(Self::parse_float);
        let volume_24h = response.get("volume").and_then(Self::parse_float);

        let price_change_24h = response.get("change").and_then(Self::parse_float);
        let price_change_percent_24h = response
            .get("percent_change")
            .and_then(Self::parse_float);

        // Bid/Ask (may not be available for all asset types)
        let bid_price = None; // Twelvedata quote doesn't include bid/ask in basic plan
        let ask_price = None;

        // Timestamp (parse from datetime field or use current time)
        let timestamp = response
            .get("datetime")
            .and_then(|v| Self::parse_datetime(v.as_str()?))
            .unwrap_or_else(|| chrono::Utc::now().timestamp_millis());

        Ok(Ticker {
            symbol: symbol.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h,
            low_24h,
            volume_24h,
            quote_volume_24h: None, // Not provided by Twelvedata
            price_change_24h,
            price_change_percent_24h,
            timestamp,
        })
    }

    /// Parse klines from /time_series endpoint
    ///
    /// Response: `{"values": [{"datetime": "2024-01-26", "open": "149.50", ...}], ...}`
    ///
    /// CRITICAL: Values are STRINGS to preserve precision
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let values = response
            .get("values")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing values array in time_series".to_string()))?;

        let mut klines = Vec::with_capacity(values.len());

        for bar in values {
            // Parse timestamp from datetime field
            let datetime_str = bar
                .get("datetime")
                .and_then(|v| v.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing datetime in kline".to_string()))?;

            let open_time = Self::parse_datetime(datetime_str)
                .ok_or_else(|| ExchangeError::Parse("Invalid datetime format".to_string()))?;

            // Parse OHLCV - all are strings!
            let open = Self::parse_string_float(bar, "open")?;
            let high = Self::parse_string_float(bar, "high")?;
            let low = Self::parse_string_float(bar, "low")?;
            let close = Self::parse_string_float(bar, "close")?;
            let volume = Self::parse_string_float(bar, "volume")?;

            klines.push(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                quote_volume: None,
                close_time: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse orderbook (not available in Twelvedata - stocks don't have L2 depth)
    pub fn parse_orderbook(_response: &Value) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Orderbook not available from Twelvedata (stocks data provider)".to_string(),
        ))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse float from Value (handles both string and number)
    fn parse_float(value: &Value) -> Option<f64> {
        value
            .as_f64()
            .or_else(|| value.as_str()?.parse::<f64>().ok())
    }

    /// Parse string float from object field
    fn parse_string_float(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(Self::parse_float)
            .ok_or_else(|| {
                ExchangeError::Parse(format!("Missing or invalid {} field", field))
            })
    }

    /// Parse datetime string to Unix timestamp in milliseconds
    ///
    /// Twelvedata datetime formats:
    /// - "2024-01-26" (date only)
    /// - "2024-01-26 15:30:00" (datetime)
    /// - Unix timestamp (numeric)
    fn parse_datetime(datetime: &str) -> Option<i64> {
        // Try parsing as Unix timestamp first
        if let Ok(timestamp) = datetime.parse::<i64>() {
            return Some(timestamp * 1000); // Convert to milliseconds
        }

        // Try parsing as date/datetime
        use chrono::NaiveDateTime;

        // Try "YYYY-MM-DD HH:MM:SS" format
        if let Ok(dt) = NaiveDateTime::parse_from_str(datetime, "%Y-%m-%d %H:%M:%S") {
            return Some(dt.and_utc().timestamp_millis());
        }

        // Try "YYYY-MM-DD" format (assume midnight UTC)
        if let Ok(date) = chrono::NaiveDate::parse_from_str(datetime, "%Y-%m-%d") {
            return Some(
                date.and_hms_opt(0, 0, 0)?
                    .and_utc()
                    .timestamp_millis(),
            );
        }

        None
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_error() {
        let error_response = json!({
            "code": 401,
            "message": "Invalid API key",
            "status": "error"
        });

        let result = TwelvedataParser::check_error(&error_response);
        assert!(result.is_err());
    }

    #[test]
    fn test_check_error_rate_limit() {
        let error_response = json!({
            "code": 429,
            "message": "Rate limit exceeded",
            "status": "error"
        });

        let result = TwelvedataParser::check_error(&error_response);
        assert!(matches!(
            result,
            Err(ExchangeError::RateLimitExceeded { .. })
        ));
    }

    #[test]
    fn test_parse_price() {
        let response = json!({"price": "150.25"});
        let price = TwelvedataParser::parse_price(&response).unwrap();
        assert_eq!(price, 150.25);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "symbol": "AAPL",
            "price": "150.25",
            "high": "151.20",
            "low": "148.80",
            "volume": "65432100",
            "change": "2.50",
            "percent_change": "1.69",
            "datetime": "2024-01-26"
        });

        let ticker = TwelvedataParser::parse_ticker(&response, "AAPL").unwrap();
        assert_eq!(ticker.symbol, "AAPL");
        assert_eq!(ticker.last_price, 150.25);
        assert_eq!(ticker.high_24h, Some(151.20));
        assert_eq!(ticker.low_24h, Some(148.80));
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "values": [
                {
                    "datetime": "2024-01-26 15:00:00",
                    "open": "149.50",
                    "high": "150.25",
                    "low": "149.30",
                    "close": "150.00",
                    "volume": "1000000"
                },
                {
                    "datetime": "2024-01-26 14:00:00",
                    "open": "149.00",
                    "high": "149.75",
                    "low": "148.90",
                    "close": "149.50",
                    "volume": "900000"
                }
            ],
            "status": "ok"
        });

        let klines = TwelvedataParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 2);
        assert_eq!(klines[0].open, 149.50);
        assert_eq!(klines[0].close, 150.00);
        assert_eq!(klines[0].volume, 1000000.0);
    }

    #[test]
    fn test_parse_datetime() {
        // Date only
        let ts1 = TwelvedataParser::parse_datetime("2024-01-26");
        assert!(ts1.is_some());

        // Date and time
        let ts2 = TwelvedataParser::parse_datetime("2024-01-26 15:30:00");
        assert!(ts2.is_some());

        // Unix timestamp
        let ts3 = TwelvedataParser::parse_datetime("1706284800");
        assert!(ts3.is_some());
    }

    #[test]
    fn test_parse_float() {
        // Numeric value
        assert_eq!(TwelvedataParser::parse_float(&json!(150.25)), Some(150.25));

        // String value
        assert_eq!(
            TwelvedataParser::parse_float(&json!("150.25")),
            Some(150.25)
        );

        // Null value
        assert_eq!(TwelvedataParser::parse_float(&json!(null)), None);
    }
}
