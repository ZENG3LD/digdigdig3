//! Twelvedata WebSocket implementation
//!
//! WebSocket streaming is available on Pro+ plans only.
//!
//! ## Features
//! - Real-time price updates (~170ms average latency)
//! - Multi-asset support (stocks, forex, crypto)
//! - Max 3 connections per API key
//! - Heartbeat required every 10 seconds
//!
//! ## Limitations
//! - **Pro+ tier only** (not available on free/Basic plan)
//! - Price events only (no orderbook/trades/klines)
//! - Max 3 concurrent connections per API key
//! - Heartbeat must be sent every 10 seconds or connection closes


/// Twelvedata WebSocket connector
///
/// NOTE: WebSocket is only available on Pro+ plans.
/// Free/Basic tier users will get connection errors.
pub struct TwelvedataWebSocket {
    api_key: String,
}

impl TwelvedataWebSocket {
    /// Create new WebSocket connector
    ///
    /// WARNING: Requires Pro+ plan subscription.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
        }
    }

    /// Get WebSocket URL with API key
    ///
    /// Format: `wss://ws.twelvedata.com/v1/quotes/price?apikey=YOUR_KEY`
    pub fn ws_url(&self) -> String {
        format!("wss://ws.twelvedata.com/v1/quotes/price?apikey={}", self.api_key)
    }

    /// Create subscribe message for symbols
    ///
    /// ```json
    /// {
    ///   "action": "subscribe",
    ///   "params": {
    ///     "symbols": "AAPL,TSLA,BTC/USD"
    ///   }
    /// }
    /// ```
    pub fn subscribe_message(&self, symbols: &[String]) -> String {
        let symbols_str = symbols.join(",");
        format!(
            r#"{{"action":"subscribe","params":{{"symbols":"{}"}}}}"#,
            symbols_str
        )
    }

    /// Create unsubscribe message
    pub fn unsubscribe_message(&self, symbols: &[String]) -> String {
        let symbols_str = symbols.join(",");
        format!(
            r#"{{"action":"unsubscribe","params":{{"symbols":"{}"}}}}"#,
            symbols_str
        )
    }

    /// Create heartbeat message
    ///
    /// Must be sent every 10 seconds to keep connection alive.
    pub fn heartbeat_message(&self) -> String {
        r#"{"action":"heartbeat"}"#.to_string()
    }

    /// Create reset message (reset all subscriptions)
    pub fn reset_message(&self) -> String {
        r#"{"action":"reset"}"#.to_string()
    }
}

/// WebSocket price event
///
/// ```json
/// {
///   "event": "price",
///   "symbol": "AAPL",
///   "currency": "USD",
///   "exchange": "NASDAQ",
///   "type": "Common Stock",
///   "timestamp": 1706284800,
///   "price": 150.25,
///   "bid": 150.20,
///   "ask": 150.30,
///   "day_volume": 65432100
/// }
/// ```
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TwelvedataPriceEvent {
    pub symbol: String,
    pub exchange: String,
    pub asset_type: String,
    pub currency: String,
    pub price: f64,
    pub bid: Option<f64>,
    pub ask: Option<f64>,
    pub day_volume: Option<f64>,
    pub timestamp: i64,
}

/// WebSocket connection status event
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum TwelvedataWsEvent {
    /// Price update
    Price(TwelvedataPriceEvent),
    /// Connection established
    Connected,
    /// Subscription confirmed
    Subscribed { symbols: Vec<String> },
    /// Unsubscription confirmed
    Unsubscribed { symbols: Vec<String> },
    /// Heartbeat acknowledged
    HeartbeatAck,
    /// Error occurred
    Error { code: i32, message: String },
    /// Connection closed
    Closed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ws_url() {
        let ws = TwelvedataWebSocket::new("test_key");
        let url = ws.ws_url();
        assert!(url.starts_with("wss://ws.twelvedata.com"));
        assert!(url.contains("apikey=test_key"));
    }

    #[test]
    fn test_subscribe_message() {
        let ws = TwelvedataWebSocket::new("test_key");
        let msg = ws.subscribe_message(&["AAPL".to_string(), "TSLA".to_string()]);
        assert!(msg.contains(r#""action":"subscribe""#));
        assert!(msg.contains("AAPL,TSLA"));
    }

    #[test]
    fn test_heartbeat_message() {
        let ws = TwelvedataWebSocket::new("test_key");
        let msg = ws.heartbeat_message();
        assert_eq!(msg, r#"{"action":"heartbeat"}"#);
    }
}
