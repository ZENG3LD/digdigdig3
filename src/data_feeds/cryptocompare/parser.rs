//! CryptoCompare response parsers
//!
//! Parse JSON responses to domain types based on CryptoCompare API response formats.

use serde_json::Value;
use crate::core::types::{ExchangeError, ExchangeResult, Ticker, Kline};

pub struct CryptoCompareParser;

impl CryptoCompareParser {
    // ═══════════════════════════════════════════════════════════════════════════════
    // ERROR HANDLING
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Check if response is an error
    ///
    /// CryptoCompare error format:
    /// ```json
    /// {
    ///   "Response": "Error",
    ///   "Message": "Error description",
    ///   "Type": 99,
    ///   "Data": {}
    /// }
    /// ```
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        if let Some(resp_status) = response.get("Response").and_then(|v| v.as_str()) {
            if resp_status == "Error" {
                let message = response
                    .get("Message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");

                let error_type = response
                    .get("Type")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1);

                return match error_type {
                    99 => Err(ExchangeError::RateLimitExceeded {
                        retry_after: None,
                        message: message.to_string()
                    }),
                    2 => Err(ExchangeError::InvalidRequest(message.to_string())),
                    _ => Err(ExchangeError::Api {
                        code: error_type as i32,
                        message: message.to_string()
                    }),
                };
            }
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // STANDARD MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Parse simple price response
    ///
    /// Example from /data/price:
    /// ```json
    /// { "USD": 45000.50, "EUR": 41000.25 }
    /// ```
    pub fn parse_price(response: &Value, quote: &str) -> ExchangeResult<f64> {
        Self::check_error(response)?;

        response
            .get(quote)
            .and_then(|v| v.as_f64())
            .ok_or_else(|| {
                ExchangeError::Parse(format!("Missing or invalid price for quote '{}'", quote))
            })
    }

    /// Parse ticker response from pricemultifull
    ///
    /// Example response structure:
    /// ```json
    /// {
    ///   "RAW": {
    ///     "BTC": {
    ///       "USD": {
    ///         "PRICE": 45000.50,
    ///         "BID": 44999.00,
    ///         "ASK": 45001.00,
    ///         "HIGH24HOUR": 45500.00,
    ///         "LOW24HOUR": 44000.00,
    ///         "VOLUME24HOUR": 1500.50,
    ///         "VOLUME24HOURTO": 67500000.00,
    ///         "CHANGE24HOUR": 800.50,
    ///         "CHANGEPCT24HOUR": 1.81,
    ///         "LASTUPDATE": 1706280000
    ///       }
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_ticker(response: &Value, fsym: &str, tsym: &str) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        let raw = response
            .get("RAW")
            .ok_or_else(|| ExchangeError::Parse("Missing 'RAW' field".to_string()))?;

        let sym_data = raw
            .get(fsym)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing symbol '{}'", fsym)))?;

        let ticker_data = sym_data
            .get(tsym)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing quote '{}'", tsym)))?;

        Ok(Ticker {
            symbol: format!("{}{}", fsym, tsym),
            last_price: Self::require_f64(ticker_data, "PRICE")?,
            bid_price: Self::get_f64(ticker_data, "BID"),
            ask_price: Self::get_f64(ticker_data, "ASK"),
            high_24h: Self::get_f64(ticker_data, "HIGH24HOUR"),
            low_24h: Self::get_f64(ticker_data, "LOW24HOUR"),
            volume_24h: Self::get_f64(ticker_data, "VOLUME24HOUR"),
            quote_volume_24h: Self::get_f64(ticker_data, "VOLUME24HOURTO"),
            price_change_24h: Self::get_f64(ticker_data, "CHANGE24HOUR"),
            price_change_percent_24h: Self::get_f64(ticker_data, "CHANGEPCT24HOUR"),
            timestamp: Self::require_i64(ticker_data, "LASTUPDATE")? * 1000, // Convert to milliseconds
        })
    }

    /// Parse klines/candles response
    ///
    /// Example from /data/histoday:
    /// ```json
    /// {
    ///   "Response": "Success",
    ///   "Data": {
    ///     "Data": [
    ///       {
    ///         "time": 1706140800,
    ///         "high": 45200.00,
    ///         "low": 44800.00,
    ///         "open": 44900.00,
    ///         "volumefrom": 1250.50,
    ///         "volumeto": 56281250.00,
    ///         "close": 45000.00
    ///       }
    ///     ]
    ///   }
    /// }
    /// ```
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        let data_wrapper = response
            .get("Data")
            .ok_or_else(|| ExchangeError::Parse("Missing 'Data' wrapper".to_string()))?;

        let array = data_wrapper
            .get("Data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'Data.Data' array".to_string()))?;

        array
            .iter()
            .map(|candle| {
                Ok(Kline {
                    open_time: Self::require_i64(candle, "time")? * 1000, // Convert to milliseconds
                    open: Self::require_f64(candle, "open")?,
                    high: Self::require_f64(candle, "high")?,
                    low: Self::require_f64(candle, "low")?,
                    close: Self::require_f64(candle, "close")?,
                    volume: Self::require_f64(candle, "volumefrom")?,
                    quote_volume: Self::get_f64(candle, "volumeto"),
                    close_time: None, // CryptoCompare doesn't provide close_time
                    trades: None,     // CryptoCompare doesn't provide trade count
                })
            })
            .collect()
    }

    /// Parse symbols list from coinlist endpoint
    ///
    /// Example from /data/all/coinlist:
    /// ```json
    /// {
    ///   "Response": "Success",
    ///   "Data": {
    ///     "BTC": {
    ///       "Symbol": "BTC",
    ///       "CoinName": "Bitcoin",
    ///       ...
    ///     },
    ///     "ETH": {
    ///       "Symbol": "ETH",
    ///       "CoinName": "Ethereum",
    ///       ...
    ///     }
    ///   }
    /// }
    /// ```
    pub fn parse_symbols(response: &Value) -> ExchangeResult<Vec<String>> {
        Self::check_error(response)?;

        let data = response
            .get("Data")
            .and_then(|v| v.as_object())
            .ok_or_else(|| ExchangeError::Parse("Missing or invalid 'Data' object".to_string()))?;

        Ok(data
            .keys()
            .map(String::clone)
            .collect())
    }

    // ═══════════════════════════════════════════════════════════════════════════════
    // HELPER METHODS
    // ═══════════════════════════════════════════════════════════════════════════════

    fn require_f64(obj: &Value, field: &str) -> ExchangeResult<f64> {
        obj.get(field)
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn get_f64(obj: &Value, field: &str) -> Option<f64> {
        obj.get(field)
            .and_then(|v| {
                v.as_f64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
    }

    fn require_i64(obj: &Value, field: &str) -> ExchangeResult<i64> {
        obj.get(field)
            .and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
            .ok_or_else(|| ExchangeError::Parse(format!("Missing/invalid '{}'", field)))
    }

    fn _get_i64(obj: &Value, field: &str) -> Option<i64> {
        obj.get(field)
            .and_then(|v| {
                v.as_i64()
                    .or_else(|| v.as_str().and_then(|s| s.parse().ok()))
            })
    }

    fn _get_str<'a>(obj: &'a Value, field: &str) -> Option<&'a str> {
        obj.get(field).and_then(|v| v.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_check_error() {
        let error_response = json!({
            "Response": "Error",
            "Message": "Test error",
            "Type": 2,
            "Data": {}
        });

        let result = CryptoCompareParser::check_error(&error_response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_price() {
        let response = json!({
            "USD": 45000.50,
            "EUR": 41000.25
        });

        let price = CryptoCompareParser::parse_price(&response, "USD").unwrap();
        assert_eq!(price, 45000.50);
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "Response": "Success",
            "Data": {
                "Data": [
                    {
                        "time": 1706140800,
                        "open": 44900.00,
                        "high": 45200.00,
                        "low": 44800.00,
                        "close": 45000.00,
                        "volumefrom": 1250.50,
                        "volumeto": 56281250.00
                    }
                ]
            }
        });

        let klines = CryptoCompareParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 1);
        assert_eq!(klines[0].open, 44900.00);
        assert_eq!(klines[0].close, 45000.00);
    }
}
