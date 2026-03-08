//! Futu OpenAPI response parsers
//!
//! Futu uses Protocol Buffers, not JSON. This is a stub documenting the format.

use serde_json::Value;
use crate::core::types::*;

pub struct FutuParser;

impl FutuParser {
    /// Stub: Parse price response
    ///
    /// Actual implementation would use Protocol Buffer definitions:
    /// ```protobuf
    /// message Response {
    ///     required int32 retType = 1;  // 0 = success, -1 = error
    ///     optional string retMsg = 2;
    ///     optional S2C s2c = 4;
    /// }
    /// ```
    pub fn parse_price(_response: &Value) -> ExchangeResult<f64> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu uses Protocol Buffers, not JSON. Use Futu SDK or implement protobuf client.".to_string()
        ))
    }

    /// Stub: Parse ticker response
    ///
    /// Futu SDK returns DataFrames (Python) with columns:
    /// - code, last_price, bid_price, ask_price, volume, turnover, etc.
    pub fn parse_ticker(_response: &Value, _symbol: &str) -> ExchangeResult<Ticker> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu uses Protocol Buffers, not JSON. Use Futu SDK or implement protobuf client.".to_string()
        ))
    }

    /// Stub: Parse klines response
    pub fn parse_klines(_response: &Value) -> ExchangeResult<Vec<Kline>> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu uses Protocol Buffers, not JSON. Use Futu SDK or implement protobuf client.".to_string()
        ))
    }

    /// Stub: Parse orderbook response
    pub fn parse_orderbook(_response: &Value) -> ExchangeResult<OrderBook> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu uses Protocol Buffers, not JSON. Use Futu SDK or implement protobuf client.".to_string()
        ))
    }

    /// Helper: Not applicable (JSON parsing)
    #[allow(dead_code)]
    fn require_f64(_obj: &Value, _field: &str) -> ExchangeResult<f64> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu does not use JSON".to_string()
        ))
    }

    #[allow(dead_code)]
    fn get_f64(_obj: &Value, _field: &str) -> Option<f64> {
        None
    }

    #[allow(dead_code)]
    fn require_i64(_obj: &Value, _field: &str) -> ExchangeResult<i64> {
        Err(ExchangeError::UnsupportedOperation(
            "Futu does not use JSON".to_string()
        ))
    }
}
