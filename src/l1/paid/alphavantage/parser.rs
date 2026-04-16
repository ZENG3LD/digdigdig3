//! # AlphaVantage Response Parsers
//!
//! Parse AlphaVantage JSON responses to domain types.
//!
//! ## Key Characteristics
//! - All numeric values returned as STRINGS (need parsing)
//! - Field names use numbered prefixes (e.g., "1. open", "2. high")
//! - Time series responses have nested structure
//! - FX data has NO volume field

use serde_json::Value;
use crate::core::types::*;
use crate::core::types::{ExchangeError, ExchangeResult};

pub struct AlphaVantageParser;

impl AlphaVantageParser {
    // ═══════════════════════════════════════════════════════════════════════
    // ERROR CHECKING
    // ═══════════════════════════════════════════════════════════════════════

    /// Check response for API errors
    ///
    /// AlphaVantage returns errors in the response body, not HTTP status codes.
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check for error message
        if let Some(err_msg) = response.get("Error Message").and_then(|v| v.as_str()) {
            return Err(ExchangeError::Api {
                code: 0,
                message: err_msg.to_string(),
            });
        }

        // Check for demo key limitation
        if let Some(info) = response.get("Information").and_then(|v| v.as_str()) {
            if info.contains("demo") || info.contains("API key") {
                return Err(ExchangeError::Auth(
                    "Demo API key does not support this endpoint. Please use a real API key.".to_string()
                ));
            }
        }

        // Check for notes (often rate limits or premium features)
        if let Some(note) = response.get("Note").and_then(|v| v.as_str()) {
            // Rate limit exceeded
            if note.contains("call frequency") {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: Some(60), // Wait 1 minute
                    message: note.to_string(),
                });
            }

            // Daily limit reached
            if note.contains("daily limit") {
                return Err(ExchangeError::RateLimitExceeded {
                    retry_after: Some(86400), // Wait 24 hours
                    message: note.to_string(),
                });
            }

            // Premium feature required
            if note.contains("not available on your current plan") {
                return Err(ExchangeError::UnsupportedOperation(note.to_string()));
            }

            // Generic note - might be informational
            // Return as warning in logs but don't fail
        }

        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // FOREX PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse CURRENCY_EXCHANGE_RATE response
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "Realtime Currency Exchange Rate": {
    ///     "5. Exchange Rate": "1.08450000",
    ///     "8. Bid Price": "1.08440000",
    ///     "9. Ask Price": "1.08460000"
    ///   }
    /// }
    /// ```
    pub fn parse_exchange_rate(response: &Value) -> ExchangeResult<f64> {
        let rate_obj = response
            .get("Realtime Currency Exchange Rate")
            .ok_or_else(|| ExchangeError::Parse("Missing 'Realtime Currency Exchange Rate'".to_string()))?;

        Self::require_f64(rate_obj, "5. Exchange Rate")
    }

    /// Parse FX_DAILY response to klines
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "Meta Data": {...},
    ///   "Time Series FX (Daily)": {
    ///     "2024-01-25": {
    ///       "1. open": "1.08300000",
    ///       "2. high": "1.08850000",
    ///       "3. low": "1.08250000",
    ///       "4. close": "1.08450000"
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_fx_daily(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let time_series = response
            .get("Time Series FX (Daily)")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'Time Series FX (Daily)'".to_string()))?;

        Self::parse_time_series_ohlc(time_series, false)
    }

    /// Parse FX_WEEKLY response
    pub fn parse_fx_weekly(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let time_series = response
            .get("Time Series FX (Weekly)")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'Time Series FX (Weekly)'".to_string()))?;

        Self::parse_time_series_ohlc(time_series, false)
    }

    /// Parse FX_MONTHLY response
    pub fn parse_fx_monthly(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let time_series = response
            .get("Time Series FX (Monthly)")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'Time Series FX (Monthly)'".to_string()))?;

        Self::parse_time_series_ohlc(time_series, false)
    }

    /// Parse FX_INTRADAY response
    ///
    /// Key name depends on interval: "Time Series FX (5min)", etc.
    pub fn parse_fx_intraday(response: &Value, interval: &str) -> ExchangeResult<Vec<Kline>> {
        let key = format!("Time Series FX ({})", interval);
        let time_series = response
            .get(&key)
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing '{}'", key)))?;

        Self::parse_time_series_ohlc(time_series, true)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // STOCK PARSERS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse GLOBAL_QUOTE response
    pub fn parse_global_quote(response: &Value) -> ExchangeResult<f64> {
        let quote = response
            .get("Global Quote")
            .ok_or_else(|| ExchangeError::Parse("Missing 'Global Quote'".to_string()))?;

        Self::require_f64(quote, "05. price")
    }

    /// Parse TIME_SERIES_DAILY response
    pub fn parse_time_series_daily(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let time_series = response
            .get("Time Series (Daily)")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing 'Time Series (Daily)'".to_string()))?;

        Self::parse_time_series_ohlc(time_series, false)
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse generic OHLC time series
    ///
    /// Works for both forex and stocks, handles both date-only and datetime timestamps.
    fn parse_time_series_ohlc(
        time_series: &serde_json::Map<String, Value>,
        has_datetime: bool,
    ) -> ExchangeResult<Vec<Kline>> {
        let mut klines = Vec::with_capacity(time_series.len());

        for (timestamp_str, candle) in time_series {
            let timestamp = if has_datetime {
                Self::parse_datetime_to_timestamp(timestamp_str)?
            } else {
                Self::parse_date_to_timestamp(timestamp_str)?
            };

            let kline = Kline {
                open_time: timestamp,
                open: Self::require_f64(candle, "1. open")?,
                high: Self::require_f64(candle, "2. high")?,
                low: Self::require_f64(candle, "3. low")?,
                close: Self::require_f64(candle, "4. close")?,
                volume: Self::get_f64(candle, "5. volume").unwrap_or(0.0), // FX has no volume
                quote_volume: None,
                close_time: None,
                trades: None,
            };
            klines.push(kline);
        }

        // Sort by timestamp (oldest first)
        klines.sort_by_key(|k| k.open_time);

        Ok(klines)
    }

    /// Parse required f64 field (value is string in response)
    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
            .ok_or_else(|| {
                ExchangeError::Parse(format!("Missing or invalid field '{}'", field))
            })
    }

    /// Parse optional f64 field
    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<f64>().ok())
    }

    /// Parse date string to Unix timestamp (milliseconds)
    ///
    /// Format: "YYYY-MM-DD"
    /// Assumes midnight UTC
    fn parse_date_to_timestamp(date_str: &str) -> ExchangeResult<i64> {
        use chrono::NaiveDate;

        NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
            .map(|date| date.and_hms_opt(0, 0, 0).expect("Valid time 00:00:00").and_utc().timestamp() * 1000)
            .map_err(|e| ExchangeError::Parse(format!("Invalid date '{}': {}", date_str, e)))
    }

    /// Parse datetime string to Unix timestamp (milliseconds)
    ///
    /// Format: "YYYY-MM-DD HH:MM:SS"
    fn parse_datetime_to_timestamp(datetime_str: &str) -> ExchangeResult<i64> {
        use chrono::NaiveDateTime;

        NaiveDateTime::parse_from_str(datetime_str, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc().timestamp() * 1000)
            .map_err(|e| {
                ExchangeError::Parse(format!("Invalid datetime '{}': {}", datetime_str, e))
            })
    }
}
