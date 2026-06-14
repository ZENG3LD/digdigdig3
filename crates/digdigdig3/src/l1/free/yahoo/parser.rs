//! Yahoo Finance response parsers
//!
//! Parse JSON responses to domain types based on Yahoo Finance API formats

use serde_json::Value;
use crate::core::types::*;

pub struct YahooFinanceParser;

impl YahooFinanceParser {
    // ═══════════════════════════════════════════════════════════════════════
    // STANDARD MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse price from /v8/finance/chart/{symbol} response
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "chart": {
    ///     "result": [{
    ///       "meta": {"regularMarketPrice": 150.25, ...}
    ///     }]
    ///   }
    /// }
    /// ```
    ///
    /// Note: Changed from quote endpoint to chart endpoint (quote returns 401 as of Jan 2026)
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        let result = Self::get_chart_result(response)?;
        let first = result
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty result array".to_string()))?;

        let meta = first
            .get("meta")
            .ok_or_else(|| ExchangeError::Parse("Missing meta field in chart response".to_string()))?;

        Self::require_f64(meta, "regularMarketPrice")
    }

    /// Parse ticker from /v8/finance/chart/{symbol} response
    ///
    /// Note: Changed from quote endpoint to chart endpoint (quote returns 401 as of Jan 2026)
    pub fn parse_ticker(response: &Value, _symbol: &str) -> ExchangeResult<Ticker> {
        let result = Self::get_chart_result(response)?;
        let first = result
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty result array".to_string()))?;

        let meta = first
            .get("meta")
            .ok_or_else(|| ExchangeError::Parse("Missing meta field in chart response".to_string()))?;

        Ok(Ticker {
            last_price: Self::require_f64(meta, "regularMarketPrice")?,
            // Chart endpoint carries no bid/ask. Even /v7/finance/quote omits
            // bid/ask for crypto symbols (Yahoo does not expose live bid/ask for
            // crypto — only for stocks during market hours). This is expected; None
            // is the correct signal, not a parse failure.
            bid_price: None,
            ask_price: None,
            high_24h: Self::get_f64(meta, "regularMarketDayHigh"),
            low_24h: Self::get_f64(meta, "regularMarketDayLow"),
            volume_24h: Self::get_f64(meta, "regularMarketVolume"),
            quote_volume_24h: None, // Yahoo doesn't provide quote volume
            price_change_24h: Self::get_f64(meta, "regularMarketChange"),
            price_change_percent_24h: Self::get_f64(meta, "regularMarketChangePercent")
                .map(|p| p * 100.0), // Convert to percentage
            // Yahoo `regularMarketTime` is Unix **seconds**; project-wide
            // `Ticker.timestamp` is **milliseconds**. Multiply on the way in.
            timestamp: Self::get_i64(meta, "regularMarketTime")
                .map(|s| s * 1_000)
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis()), ..Default::default() 
        })
    }

    /// Parse klines from /v8/finance/chart/{symbol} response
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "chart": {
    ///     "result": [{
    ///       "timestamp": [1640563200, 1640649600, ...],
    ///       "indicators": {
    ///         "quote": [{
    ///           "open": [148.50, ...],
    ///           "high": [149.50, ...],
    ///           "low": [147.00, ...],
    ///           "close": [148.00, ...],
    ///           "volume": [75000000, ...]
    ///         }]
    ///       }
    ///     }]
    ///   }
    /// }
    /// ```
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let result = Self::get_chart_result(response)?;
        let first = result
            .first()
            .ok_or_else(|| ExchangeError::Parse("Empty result array".to_string()))?;

        // Get timestamp array
        let timestamps = first
            .get("timestamp")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing timestamp array".to_string()))?;

        // Get indicators.quote[0]
        let quote = first
            .get("indicators")
            .and_then(|i| i.get("quote"))
            .and_then(|q| q.as_array())
            .and_then(|a| a.first())
            .ok_or_else(|| ExchangeError::Parse("Missing indicators.quote".to_string()))?;

        // Get OHLCV arrays
        let opens = Self::get_array(quote, "open")?;
        let highs = Self::get_array(quote, "high")?;
        let lows = Self::get_array(quote, "low")?;
        let closes = Self::get_array(quote, "close")?;
        let volumes = Self::get_array(quote, "volume")?;

        // Build klines
        let mut klines = Vec::new();
        let len = timestamps.len();

        for i in 0..len {
            // Yahoo /chart endpoint returns timestamps in **seconds**. Convert
            // to ms — `Kline.open_time` is ms project-wide.
            let timestamp_s = timestamps
                .get(i)
                .and_then(|v| v.as_i64())
                .ok_or_else(|| ExchangeError::Parse(format!("Invalid timestamp at {}", i)))?;
            let timestamp = timestamp_s.saturating_mul(1_000);

            klines.push(Kline {
                open_time: timestamp,
                open: Self::extract_f64(opens, i)?,
                high: Self::extract_f64(highs, i)?,
                low: Self::extract_f64(lows, i)?,
                close: Self::extract_f64(closes, i)?,
                volume: Self::extract_f64(volumes, i)?,
                quote_volume: None,
                close_time: None,
                trades: None,
                ..Default::default()
            });
        }

        Ok(klines)
    }

    /// Parse ticker from /v7/finance/quote?symbols={symbol} response
    ///
    /// Response format:
    /// ```json
    /// {
    ///   "quoteResponse": {
    ///     "result": [{
    ///       "symbol": "AAPL",
    ///       "regularMarketPrice": 189.51,
    ///       "regularMarketTime": 1718788303,
    ///       "bid": 189.50,
    ///       "ask": 189.52,
    ///       "regularMarketDayHigh": 190.00,
    ///       ...
    ///     }]
    ///   }
    /// }
    /// ```
    ///
    /// `regularMarketTime` is Unix **seconds**; converted to ms on the way out.
    pub fn parse_quote_ticker(response: &Value, _symbol: &str) -> ExchangeResult<Ticker> {
        let result = Self::get_quote_response_result(response)?;
        let first = result
            .first()
            .ok_or_else(|| ExchangeError::Parse("quoteResponse.result is empty".to_string()))?;

        Ok(Ticker {
            last_price: Self::require_f64(first, "regularMarketPrice")?,
            bid_price: Self::get_f64(first, "bid"),
            ask_price: Self::get_f64(first, "ask"),
            high_24h: Self::get_f64(first, "regularMarketDayHigh"),
            low_24h: Self::get_f64(first, "regularMarketDayLow"),
            volume_24h: Self::get_f64(first, "regularMarketVolume"),
            quote_volume_24h: None,
            price_change_24h: Self::get_f64(first, "regularMarketChange"),
            price_change_percent_24h: Self::get_f64(first, "regularMarketChangePercent"),
            // regularMarketTime is seconds → convert to ms
            timestamp: Self::get_i64(first, "regularMarketTime")
                .map(|s| s * 1_000)
                .unwrap_or_else(|| chrono::Utc::now().timestamp_millis()), ..Default::default() 
        })
    }

    /// Parse symbols from /v7/finance/quote response (multiple symbols)
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        let result = Self::get_quote_response_result(response)?;

        Ok(result
            .iter()
            .filter_map(|quote| {
                quote
                    .get("symbol")
                    .and_then(|s| s.as_str())
                    .map(str::to_string)
            })
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ORDERBOOK (NOT AVAILABLE - Yahoo doesn't provide orderbook)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse orderbook - NOT SUPPORTED by Yahoo Finance
    pub fn parse_orderbook(_response: &Value) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Yahoo Finance does not provide orderbook data".to_string(),
        ))
    }

    // ═══════════════════════════════════════════════════════════════════════
    // EXTENDED DATA TYPES (Yahoo-specific)
    // ═══════════════════════════════════════════════════════════════════════

    /// Parse crumb from /v1/test/getcrumb response
    ///
    /// Response is plain text: "AbCdEfGhIjK"
    pub fn parse_crumb(response_text: &str) -> ExchangeResult<String> {
        let crumb = response_text.trim();
        if crumb.is_empty() {
            return Err(ExchangeError::Parse("Empty crumb response".to_string()));
        }
        Ok(crumb.to_string())
    }

    // ═══════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════

    /// Get quoteResponse.result array
    fn get_quote_response_result(response: &Value) -> ExchangeResult<&Vec<Value>> {
        response
            .get("quoteResponse")
            .and_then(|qr| qr.get("result"))
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing quoteResponse.result".to_string()))
    }

    /// Get chart.result array
    fn get_chart_result(response: &Value) -> ExchangeResult<&Vec<Value>> {
        response
            .get("chart")
            .and_then(|c| c.get("result"))
            .and_then(|r| r.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing chart.result".to_string()))
    }

    /// Get array field from object
    fn get_array<'a>(obj: &'a Value, field: &str) -> ExchangeResult<&'a Vec<Value>> {
        obj.get(field)
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse(format!("Missing array field '{}'", field)))
    }

    /// Extract f64 from array at index (handles null values)
    fn extract_f64(array: &[Value], index: usize) -> ExchangeResult<f64> {
        array
            .get(index)
            .and_then(|v| {
                if v.is_null() {
                    // Yahoo sometimes has null values in data
                    // Use 0.0 as fallback or previous value
                    Some(0.0)
                } else {
                    v.as_f64()
                }
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Invalid f64 at index {}", index)))
    }

    /// Require f64 field
    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    /// Get optional f64 field
    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| {
            v.as_f64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }

    /// Get optional i64 field
    fn get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field).and_then(|v| {
            v.as_i64()
                .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
        })
    }

    /// Check for error in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Check quoteResponse.error
        if let Some(error) = response
            .get("quoteResponse")
            .and_then(|qr| qr.get("error"))
            .and_then(|e| e.as_object())
        {
            if !error.is_empty() {
                let code = error
                    .get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("Unknown");
                let desc = error
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("No description");
                return Err(ExchangeError::Api { code: 0, message: format!("{}: {}", code, desc) });
            }
        }

        // Check chart.error
        if let Some(error) = response
            .get("chart")
            .and_then(|c| c.get("error"))
            .and_then(|e| e.as_object())
        {
            if !error.is_empty() {
                let code = error
                    .get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("Unknown");
                let desc = error
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("No description");
                return Err(ExchangeError::Api { code: 0, message: format!("{}: {}", code, desc) });
            }
        }

        // Check finance.error (for other endpoints)
        if let Some(error) = response
            .get("finance")
            .and_then(|f| f.get("error"))
            .and_then(|e| e.as_object())
        {
            if !error.is_empty() {
                let code = error
                    .get("code")
                    .and_then(|c| c.as_str())
                    .unwrap_or("Unknown");
                let desc = error
                    .get("description")
                    .and_then(|d| d.as_str())
                    .unwrap_or("No description");
                return Err(ExchangeError::Api { code: 0, message: format!("{}: {}", code, desc) });
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "chart": {
                "result": [{
                    "meta": {
                        "symbol": "AAPL",
                        "regularMarketPrice": 150.25
                    }
                }],
                "error": null
            }
        });

        let price = YahooFinanceParser::parse_price(&response).unwrap();
        assert_eq!(price, 150.25);
    }

    #[test]
    fn test_parse_quote_ticker() {
        let response = json!({
            "quoteResponse": {
                "result": [{
                    "symbol": "AAPL",
                    "regularMarketPrice": 189.51,
                    "regularMarketTime": 1718788303_i64,
                    "bid": 189.50,
                    "ask": 189.52,
                    "bidSize": 800,
                    "askSize": 1100,
                    "regularMarketDayHigh": 190.00,
                    "regularMarketDayLow": 188.50,
                    "regularMarketVolume": 50000000,
                    "regularMarketChange": 1.0,
                    "regularMarketChangePercent": 0.5
                }],
                "error": null
            }
        });

        let ticker = YahooFinanceParser::parse_quote_ticker(&response, "AAPL").unwrap();
        assert!((ticker.last_price - 189.51).abs() < 1e-4);
        assert_eq!(ticker.bid_price, Some(189.50));
        assert_eq!(ticker.ask_price, Some(189.52));
        assert_eq!(ticker.timestamp, 1718788303_i64 * 1_000);
    }

    #[test]
    fn test_parse_quote_ticker_no_bid_ask() {
        // Crypto may return no bid/ask — must not error, just None
        let response = json!({
            "quoteResponse": {
                "result": [{
                    "symbol": "BTC-USD",
                    "regularMarketPrice": 65000.0,
                    "regularMarketTime": 1718788303_i64
                }],
                "error": null
            }
        });

        let ticker = YahooFinanceParser::parse_quote_ticker(&response, "BTC-USD").unwrap();
        assert!(ticker.bid_price.is_none());
        assert!(ticker.ask_price.is_none());
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "chart": {
                "result": [{
                    "meta": {
                        "symbol": "AAPL",
                        "regularMarketPrice": 150.25,
                        "regularMarketDayHigh": 151.50,
                        "regularMarketDayLow": 149.00,
                        "regularMarketVolume": 75234000,
                        "regularMarketChange": 1.25,
                        "regularMarketChangePercent": 0.835,
                        "regularMarketTime": 1640980800
                    }
                }],
                "error": null
            }
        });

        let ticker = YahooFinanceParser::parse_ticker(&response, "AAPL").unwrap();
        assert_eq!(ticker.last_price, 150.25);
        assert_eq!(ticker.bid_price, None); // Chart endpoint doesn't provide bid/ask
        assert_eq!(ticker.ask_price, None); // Chart endpoint doesn't provide bid/ask
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "chart": {
                "result": [{
                    "timestamp": [1640563200, 1640649600],
                    "indicators": {
                        "quote": [{
                            "open": [148.50, 149.00],
                            "high": [149.50, 150.00],
                            "low": [147.00, 148.00],
                            "close": [148.00, 149.50],
                            "volume": [75000000, 80000000]
                        }]
                    }
                }],
                "error": null
            }
        });

        let klines = YahooFinanceParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 2);
        assert_eq!(klines[0].open, 148.50);
        assert_eq!(klines[0].high, 149.50);
        assert_eq!(klines[0].low, 147.00);
        assert_eq!(klines[0].close, 148.00);
        assert_eq!(klines[0].volume, 75000000.0);
    }

    #[test]
    fn test_check_error() {
        let error_response = json!({
            "chart": {
                "result": null,
                "error": {
                    "code": "Not Found",
                    "description": "No data found"
                }
            }
        });

        let result = YahooFinanceParser::check_error(&error_response);
        assert!(result.is_err());
    }
}
