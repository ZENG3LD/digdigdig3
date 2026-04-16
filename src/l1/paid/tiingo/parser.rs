//! # Tiingo Response Parsers
//!
//! Parse JSON responses from Tiingo API to domain types.

use serde_json::Value;
use crate::core::types::*;
use crate::core::{ExchangeError, ExchangeResult};

pub struct TiingoParser;

impl TiingoParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // EOD STOCK DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse EOD daily prices to Klines
    ///
    /// Response format (array):
    /// ```json
    /// [
    ///   {
    ///     "date": "2020-01-02T00:00:00.000Z",
    ///     "close": 300.35,
    ///     "high": 300.58,
    ///     "low": 298.02,
    ///     "open": 296.24,
    ///     "volume": 135480400,
    ///     "adjClose": 298.12,
    ///     "adjHigh": 298.35,
    ///     "adjLow": 295.81,
    ///     "adjOpen": 294.08,
    ///     "adjVolume": 135480400,
    ///     "divCash": 0.0,
    ///     "splitFactor": 1.0
    ///   }
    /// ]
    /// ```
    pub fn parse_daily_prices(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of daily prices".to_string()))?;

        array.iter().map(|item| {
            let timestamp_str = Self::get_str(item, "date")
                .ok_or_else(|| ExchangeError::Parse("Missing 'date' field".to_string()))?;

            // Parse ISO8601 timestamp to Unix milliseconds
            let timestamp = Self::parse_iso8601_to_ms(timestamp_str)?;

            Ok(Kline {
                open_time: timestamp,
                open: Self::require_f64(item, "open")?,
                high: Self::require_f64(item, "high")?,
                low: Self::require_f64(item, "low")?,
                close: Self::require_f64(item, "close")?,
                volume: Self::require_f64(item, "volume")?,
                quote_volume: None,
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // IEX INTRADAY DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse IEX intraday prices to Klines
    ///
    /// Response format (array):
    /// ```json
    /// [
    ///   {
    ///     "date": "2020-01-02T09:30:00.000Z",
    ///     "open": 296.24,
    ///     "high": 297.15,
    ///     "low": 296.00,
    ///     "close": 296.80,
    ///     "volume": 1234567
    ///   }
    /// ]
    /// ```
    pub fn parse_iex_prices(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array of IEX prices".to_string()))?;

        array.iter().map(|item| {
            let timestamp_str = Self::get_str(item, "date")
                .ok_or_else(|| ExchangeError::Parse("Missing 'date' field".to_string()))?;

            let timestamp = Self::parse_iso8601_to_ms(timestamp_str)?;

            Ok(Kline {
                open_time: timestamp,
                open: Self::require_f64(item, "open")?,
                high: Self::require_f64(item, "high")?,
                low: Self::require_f64(item, "low")?,
                close: Self::require_f64(item, "close")?,
                volume: Self::require_f64(item, "volume")?,
                quote_volume: None,
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // CRYPTO DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse crypto top-of-book response
    ///
    /// Response format (array):
    /// ```json
    /// [
    ///   {
    ///     "ticker": "btcusd",
    ///     "baseCurrency": "btc",
    ///     "quoteCurrency": "usd",
    ///     "topOfBookData": [
    ///       {
    ///         "askPrice": 45001.00,
    ///         "bidPrice": 45000.00,
    ///         "lastPrice": 45000.50,
    ///         "lastSaleTimestamp": "2020-01-02T12:34:56.789012Z"
    ///       }
    ///     ]
    ///   }
    /// ]
    /// ```
    pub fn parse_crypto_top(response: &Value) -> ExchangeResult<Ticker> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        let item = array.first()
            .ok_or_else(|| ExchangeError::Parse("Empty response".to_string()))?;

        let ticker = Self::get_str(item, "ticker")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ticker' field".to_string()))?;

        let top_of_book = item.get("topOfBookData")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first())
            .ok_or_else(|| ExchangeError::Parse("Missing 'topOfBookData'".to_string()))?;

        let last_price = Self::require_f64(top_of_book, "lastPrice")?;
        let bid_price = Self::get_f64(top_of_book, "bidPrice");
        let ask_price = Self::get_f64(top_of_book, "askPrice");

        let timestamp_str = Self::get_str(top_of_book, "lastSaleTimestamp")
            .unwrap_or("2020-01-01T00:00:00.000000Z");
        let timestamp = Self::parse_iso8601_to_ms(timestamp_str)?;

        Ok(Ticker {
            symbol: ticker.to_string(),
            last_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    /// Parse crypto historical prices
    ///
    /// Response format (array):
    /// ```json
    /// [
    ///   {
    ///     "ticker": "btcusd",
    ///     "baseCurrency": "btc",
    ///     "quoteCurrency": "usd",
    ///     "priceData": [
    ///       {
    ///         "date": "2020-01-01T00:00:00.000Z",
    ///         "open": 44500.00,
    ///         "high": 44750.50,
    ///         "low": 44300.25,
    ///         "close": 44600.75,
    ///         "volume": 123.45,
    ///         "volumeNotional": 5500000.00
    ///       }
    ///     ]
    ///   }
    /// ]
    /// ```
    pub fn parse_crypto_prices(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        let item = array.first()
            .ok_or_else(|| ExchangeError::Parse("Empty response".to_string()))?;

        let price_data = item.get("priceData")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing 'priceData'".to_string()))?;

        price_data.iter().map(|candle| {
            let timestamp_str = Self::get_str(candle, "date")
                .ok_or_else(|| ExchangeError::Parse("Missing 'date' field".to_string()))?;

            let timestamp = Self::parse_iso8601_to_ms(timestamp_str)?;

            Ok(Kline {
                open_time: timestamp,
                open: Self::require_f64(candle, "open")?,
                high: Self::require_f64(candle, "high")?,
                low: Self::require_f64(candle, "low")?,
                close: Self::require_f64(candle, "close")?,
                volume: Self::require_f64(candle, "volume")?,
                quote_volume: Self::get_f64(candle, "volumeNotional"),
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // FOREX DATA PARSERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse forex top-of-book quote
    ///
    /// Response format (array):
    /// ```json
    /// [
    ///   {
    ///     "ticker": "eurusd",
    ///     "quoteTimestamp": "2020-01-02T12:34:56.789012Z",
    ///     "bidPrice": 1.1234,
    ///     "askPrice": 1.1236,
    ///     "midPrice": 1.1235
    ///   }
    /// ]
    /// ```
    pub fn parse_forex_top(response: &Value) -> ExchangeResult<Ticker> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        let item = array.first()
            .ok_or_else(|| ExchangeError::Parse("Empty response".to_string()))?;

        let ticker = Self::get_str(item, "ticker")
            .ok_or_else(|| ExchangeError::Parse("Missing 'ticker' field".to_string()))?;

        let mid_price = Self::require_f64(item, "midPrice")?;
        let bid_price = Self::get_f64(item, "bidPrice");
        let ask_price = Self::get_f64(item, "askPrice");

        let timestamp_str = Self::get_str(item, "quoteTimestamp")
            .unwrap_or("2020-01-01T00:00:00.000000Z");
        let timestamp = Self::parse_iso8601_to_ms(timestamp_str)?;

        Ok(Ticker {
            symbol: ticker.to_string(),
            last_price: mid_price,
            bid_price,
            ask_price,
            high_24h: None,
            low_24h: None,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        })
    }

    /// Parse forex historical prices
    ///
    /// Response format (array):
    /// ```json
    /// [
    ///   {
    ///     "date": "2020-01-01T00:00:00.000Z",
    ///     "ticker": "eurusd",
    ///     "open": 1.1230,
    ///     "high": 1.1240,
    ///     "low": 1.1225,
    ///     "close": 1.1235
    ///   }
    /// ]
    /// ```
    pub fn parse_forex_prices(response: &Value) -> ExchangeResult<Vec<Kline>> {
        let array = response.as_array()
            .ok_or_else(|| ExchangeError::Parse("Expected array".to_string()))?;

        array.iter().map(|candle| {
            let timestamp_str = Self::get_str(candle, "date")
                .ok_or_else(|| ExchangeError::Parse("Missing 'date' field".to_string()))?;

            let timestamp = Self::parse_iso8601_to_ms(timestamp_str)?;

            Ok(Kline {
                open_time: timestamp,
                open: Self::require_f64(candle, "open")?,
                high: Self::require_f64(candle, "high")?,
                low: Self::require_f64(candle, "low")?,
                close: Self::require_f64(candle, "close")?,
                volume: 0.0, // Forex doesn't have volume
                quote_volume: None,
                close_time: None,
                trades: None,
            })
        }).collect()
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| {
                v.as_f64().or_else(|| {
                    v.as_str().and_then(|s| s.parse().ok())
                }).or_else(|| {
                    v.as_i64().map(|i| i as f64)
                })
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field).and_then(|v| {
            v.as_f64().or_else(|| {
                v.as_str().and_then(|s| s.parse().ok())
            }).or_else(|| {
                v.as_i64().map(|i| i as f64)
            })
        })
    }

    fn get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }

    /// Parse ISO8601 timestamp to Unix milliseconds
    /// Handles formats:
    /// - "2020-01-02T00:00:00.000Z"
    /// - "2020-01-02T12:34:56.789012Z"
    fn parse_iso8601_to_ms(timestamp_str: &str) -> ExchangeResult<i64> {
        // Use chrono to parse ISO8601 timestamps
        use chrono::{DateTime, Utc};

        DateTime::parse_from_rfc3339(timestamp_str)
            .or_else(|_| {
                // Try without timezone
                let with_z = if !timestamp_str.ends_with('Z') {
                    format!("{}Z", timestamp_str)
                } else {
                    timestamp_str.to_string()
                };
                DateTime::parse_from_rfc3339(&with_z)
            })
            .map(|dt| dt.with_timezone(&Utc).timestamp_millis())
            .map_err(|e| ExchangeError::Parse(format!("Invalid timestamp '{}': {}", timestamp_str, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_daily_prices() {
        let response = json!([
            {
                "date": "2020-01-02T00:00:00.000Z",
                "open": 296.24,
                "high": 300.58,
                "low": 298.02,
                "close": 300.35,
                "volume": 135480400
            }
        ]);

        let klines = TiingoParser::parse_daily_prices(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open, 296.24);
        assert_eq!(klines[0].close, 300.35);
    }

    #[test]
    fn test_parse_crypto_top() {
        let response = json!([
            {
                "ticker": "btcusd",
                "baseCurrency": "btc",
                "quoteCurrency": "usd",
                "topOfBookData": [
                    {
                        "bidPrice": 45000.00,
                        "askPrice": 45001.00,
                        "lastPrice": 45000.50,
                        "lastSaleTimestamp": "2020-01-02T12:34:56.789012Z"
                    }
                ]
            }
        ]);

        let ticker = TiingoParser::parse_crypto_top(&response).unwrap();
        assert_eq!(ticker.symbol, "btcusd");
        assert_eq!(ticker.last_price, 45000.50);
        assert_eq!(ticker.bid_price, Some(45000.00));
        assert_eq!(ticker.ask_price, Some(45001.00));
    }

    #[test]
    fn test_parse_iso8601() {
        let timestamp = TiingoParser::parse_iso8601_to_ms("2020-01-02T00:00:00.000Z").unwrap();
        assert!(timestamp > 0);
    }
}
