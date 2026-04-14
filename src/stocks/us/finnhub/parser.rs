//! # Finnhub Response Parser
//!
//! Parse JSON responses from Finnhub API.

use serde_json::Value;

use crate::core::types::{
    ExchangeError, ExchangeResult,
    Kline, OrderBook, OrderBookLevel, Ticker, StreamEvent,
};

/// Parser for Finnhub API responses
pub struct FinnhubParser;

impl FinnhubParser {
    // ═══════════════════════════════════════════════════════════════════════════
    // HELPERS
    // ═══════════════════════════════════════════════════════════════════════════

    /// Check for API error in response
    pub fn check_error(response: &Value) -> ExchangeResult<()> {
        // Finnhub returns {"error": "message"} on errors
        if let Some(error) = response.get("error") {
            if let Some(error_msg) = error.as_str() {
                return Err(ExchangeError::Api {
                    code: -1,
                    message: error_msg.to_string(),
                });
            }
        }
        Ok(())
    }

    /// Parse f64 from string or number
    fn parse_f64(value: &Value) -> Option<f64> {
        value.as_str()
            .and_then(|s| s.parse().ok())
            .or_else(|| value.as_f64())
    }

    /// Parse f64 from field
    fn get_f64(data: &Value, key: &str) -> Option<f64> {
        data.get(key).and_then(Self::parse_f64)
    }

    /// Parse required f64
    fn require_f64(data: &Value, key: &str) -> ExchangeResult<f64> {
        Self::get_f64(data, key)
            .ok_or_else(|| ExchangeError::Parse(format!("Missing or invalid '{}'", key)))
    }

    /// Parse string from field
    fn get_str<'a>(data: &'a Value, key: &str) -> Option<&'a str> {
        data.get(key).and_then(|v| v.as_str())
    }

    /// Parse i64 from field
    fn get_i64(data: &Value, key: &str) -> Option<i64> {
        data.get(key).and_then(|v| v.as_i64())
    }

    /// Parse array from field
    fn get_array<'a>(data: &'a Value, key: &str) -> Option<&'a Vec<Value>> {
        data.get(key).and_then(|v| v.as_array())
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // MARKET DATA
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse quote (real-time price)
    /// Response format: {"c": current, "d": change, "dp": percent_change, "h": high, "l": low, "o": open, "pc": prev_close, "t": timestamp}
    pub fn parse_price(response: &Value) -> ExchangeResult<f64> {
        Self::check_error(response)?;

        // Get current price from "c" field
        Self::require_f64(response, "c")
    }

    /// Parse stock candles (OHLCV)
    /// Response format: {"c": [closes], "h": [highs], "l": [lows], "o": [opens], "t": [timestamps], "v": [volumes], "s": "ok"}
    pub fn parse_klines(response: &Value) -> ExchangeResult<Vec<Kline>> {
        Self::check_error(response)?;

        // Check status
        if let Some(status) = Self::get_str(response, "s") {
            if status != "ok" {
                return Err(ExchangeError::Parse(format!("Candle status: {}", status)));
            }
        }

        // Get arrays
        let closes = Self::get_array(response, "c")
            .ok_or_else(|| ExchangeError::Parse("Missing 'c' (closes) array".to_string()))?;
        let opens = Self::get_array(response, "o")
            .ok_or_else(|| ExchangeError::Parse("Missing 'o' (opens) array".to_string()))?;
        let highs = Self::get_array(response, "h")
            .ok_or_else(|| ExchangeError::Parse("Missing 'h' (highs) array".to_string()))?;
        let lows = Self::get_array(response, "l")
            .ok_or_else(|| ExchangeError::Parse("Missing 'l' (lows) array".to_string()))?;
        let timestamps = Self::get_array(response, "t")
            .ok_or_else(|| ExchangeError::Parse("Missing 't' (timestamps) array".to_string()))?;
        let volumes = Self::get_array(response, "v")
            .ok_or_else(|| ExchangeError::Parse("Missing 'v' (volumes) array".to_string()))?;

        // Check all arrays have same length
        let len = closes.len();
        if opens.len() != len || highs.len() != len || lows.len() != len ||
           timestamps.len() != len || volumes.len() != len {
            return Err(ExchangeError::Parse("Array lengths mismatch in candles".to_string()));
        }

        let mut klines = Vec::with_capacity(len);

        for i in 0..len {
            let open_time = timestamps[i].as_i64().unwrap_or(0);
            let open = Self::parse_f64(&opens[i]).unwrap_or(0.0);
            let high = Self::parse_f64(&highs[i]).unwrap_or(0.0);
            let low = Self::parse_f64(&lows[i]).unwrap_or(0.0);
            let close = Self::parse_f64(&closes[i]).unwrap_or(0.0);
            let volume = Self::parse_f64(&volumes[i]).unwrap_or(0.0);

            klines.push(Kline {
                open_time,
                open,
                high,
                low,
                close,
                volume,
                close_time: None,
                quote_volume: None,
                trades: None,
            });
        }

        Ok(klines)
    }

    /// Parse ticker/quote data into Ticker structure
    /// Response format: {"c": current, "d": change, "dp": percent_change, "h": high, "l": low, "o": open, "pc": prev_close, "t": timestamp}
    pub fn parse_ticker(response: &Value) -> ExchangeResult<Ticker> {
        Self::check_error(response)?;

        let last_price = Self::require_f64(response, "c")?;
        let high_24h = Self::get_f64(response, "h");
        let low_24h = Self::get_f64(response, "l");
        let _open = Self::get_f64(response, "o");
        let _prev_close = Self::get_f64(response, "pc");
        let change = Self::get_f64(response, "d");
        let change_percent = Self::get_f64(response, "dp");
        let timestamp = Self::get_i64(response, "t").unwrap_or(0);

        // Finnhub doesn't provide symbol in quote response, we'll add it from caller
        Ok(Ticker {
            symbol: String::new(), // Will be filled by caller
            last_price,
            bid_price: None,
            ask_price: None,
            high_24h,
            low_24h,
            volume_24h: None,
            quote_volume_24h: None,
            price_change_24h: change,
            price_change_percent_24h: change_percent,
            timestamp,
        })
    }

    /// Parse orderbook (Finnhub doesn't provide full orderbook, only bid/ask)
    /// This is only available on premium tiers
    pub fn parse_orderbook(response: &Value) -> ExchangeResult<OrderBook> {
        Self::check_error(response)?;

        // Finnhub's /stock/bidask endpoint returns:
        // {"a": ask_price, "as": ask_size, "b": bid_price, "bs": bid_size, "t": timestamp}
        let bid_price = Self::require_f64(response, "b")?;
        let ask_price = Self::require_f64(response, "a")?;
        let bid_size = Self::get_f64(response, "bs").unwrap_or(0.0);
        let ask_size = Self::get_f64(response, "as").unwrap_or(0.0);
        let timestamp = Self::get_i64(response, "t").unwrap_or(0);

        Ok(OrderBook {
            bids: vec![OrderBookLevel::new(bid_price, bid_size)],
            asks: vec![OrderBookLevel::new(ask_price, ask_size)],
            timestamp,
            sequence: None,
            last_update_id: None,
            first_update_id: None,
            prev_update_id: None,
            event_time: None,
            transaction_time: None,
            checksum: None,
        })
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // WEBSOCKET MESSAGES
    // ═══════════════════════════════════════════════════════════════════════════

    /// Parse WebSocket message to StreamEvent
    /// Finnhub WebSocket format: {"type": "trade", "data": [{...}]} or {"type": "ping"}
    pub fn parse_ws_message(msg: &Value) -> ExchangeResult<Vec<StreamEvent>> {
        let msg_type = Self::get_str(msg, "type")
            .ok_or_else(|| ExchangeError::Parse("Missing 'type' field in WebSocket message".to_string()))?;

        match msg_type {
            "ping" => {
                // Ping message, no data to process
                Ok(Vec::new())
            }
            "trade" => {
                // Trade message
                let data = Self::get_array(msg, "data")
                    .ok_or_else(|| ExchangeError::Parse("Missing 'data' field in trade message".to_string()))?;

                let mut events = Vec::new();
                for trade in data {
                    if let Ok(event) = Self::parse_ws_trade(trade) {
                        events.push(event);
                    }
                }
                Ok(events)
            }
            "error" => {
                // Error message
                let error_msg = Self::get_str(msg, "msg").unwrap_or("Unknown WebSocket error");
                Err(ExchangeError::Api {
                    code: -1,
                    message: error_msg.to_string(),
                })
            }
            _ => {
                // Unknown message type, skip
                Ok(Vec::new())
            }
        }
    }

    /// Parse WebSocket trade message
    /// Format: {"s": symbol, "p": price, "t": timestamp_ms, "v": volume, "c": [conditions]}
    fn parse_ws_trade(trade: &Value) -> ExchangeResult<StreamEvent> {
        let symbol = Self::get_str(trade, "s")
            .ok_or_else(|| ExchangeError::Parse("Missing 's' field in trade".to_string()))?
            .to_string();

        let price = Self::require_f64(trade, "p")?;
        let volume = Self::get_f64(trade, "v").unwrap_or(0.0);
        let timestamp = Self::get_i64(trade, "t").unwrap_or(0);

        // Create a ticker event from trade data
        Ok(StreamEvent::Ticker(Ticker {
            symbol,
            last_price: price,
            bid_price: None,
            ask_price: None,
            high_24h: None,
            low_24h: None,
            volume_24h: Some(volume),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            timestamp,
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price() {
        let response = json!({
            "c": 150.85,
            "d": 1.25,
            "dp": 0.83,
            "h": 151.50,
            "l": 150.20,
            "o": 150.50,
            "pc": 149.60,
            "t": 1609459200
        });

        let price = FinnhubParser::parse_price(&response).unwrap();
        assert_eq!(price, 150.85);
    }

    #[test]
    fn test_parse_klines() {
        let response = json!({
            "c": [150.85, 151.00],
            "h": [151.50, 151.75],
            "l": [150.20, 150.50],
            "o": [150.50, 150.85],
            "t": [1609459200, 1609545600],
            "v": [75234567.0, 82345678.0],
            "s": "ok"
        });

        let klines = FinnhubParser::parse_klines(&response).unwrap();
        assert_eq!(klines.len(), 2);
        assert_eq!(klines[0].close, 150.85);
        assert_eq!(klines[0].volume, 75234567.0);
        assert_eq!(klines[1].close, 151.00);
    }

    #[test]
    fn test_parse_ticker() {
        let response = json!({
            "c": 150.85,
            "d": 1.25,
            "dp": 0.83,
            "h": 151.50,
            "l": 150.20,
            "o": 150.50,
            "pc": 149.60,
            "t": 1609459200
        });

        let ticker = FinnhubParser::parse_ticker(&response).unwrap();
        assert_eq!(ticker.last_price, 150.85);
        assert_eq!(ticker.high_24h, Some(151.50));
        assert_eq!(ticker.low_24h, Some(150.20));
        assert_eq!(ticker.price_change_24h, Some(1.25));
    }

    #[test]
    fn test_parse_error() {
        let response = json!({
            "error": "Invalid API key"
        });

        let result = FinnhubParser::parse_price(&response);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_orderbook() {
        let response = json!({
            "a": 150.86,
            "as": 100.0,
            "b": 150.85,
            "bs": 150.0,
            "t": 1609459200
        });

        let orderbook = FinnhubParser::parse_orderbook(&response).unwrap();
        assert_eq!(orderbook.asks[0].price, 150.86);
        assert_eq!(orderbook.bids[0].price, 150.85);
    }
}
