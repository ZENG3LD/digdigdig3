//! # OANDA HTTP Streaming
//!
//! HTTP streaming implementation for OANDA pricing and transaction streams.
//!
//! IMPORTANT: OANDA uses HTTP streaming (NOT WebSocket)
//! - Uses chunked transfer encoding
//! - Newline-delimited JSON messages
//! - Heartbeats every 5 seconds

use serde_json::Value;

use crate::core::{ExchangeError, ExchangeResult};

use super::auth::OandaAuth;
use super::endpoints::OandaUrls;

// ═══════════════════════════════════════════════════════════════════════════════
// STREAMING MESSAGE TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Streaming message types
#[derive(Debug, Clone)]
pub enum StreamMessage {
    /// Price update
    Price(PriceUpdate),
    /// Heartbeat
    Heartbeat { time: String },
    /// Transaction update
    Transaction(Value),
}

/// Price update from streaming endpoint
#[derive(Debug, Clone)]
pub struct PriceUpdate {
    pub instrument: String,
    pub time: String,
    pub tradeable: bool,
    pub bid: f64,
    pub ask: f64,
    pub closeout_bid: f64,
    pub closeout_ask: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// PRICING STREAM
// ═══════════════════════════════════════════════════════════════════════════════

/// HTTP streaming client for OANDA pricing stream
pub struct PricingStream {
    // Will be implemented when needed - placeholder for now
    _auth: OandaAuth,
    _urls: OandaUrls,
    _account_id: String,
    _instruments: Vec<String>,
}

impl PricingStream {
    /// Create new pricing stream
    pub fn new(
        auth: OandaAuth,
        urls: OandaUrls,
        account_id: String,
        instruments: Vec<String>,
    ) -> Self {
        Self {
            _auth: auth,
            _urls: urls,
            _account_id: account_id,
            _instruments: instruments,
        }
    }

    /// Connect to pricing stream
    pub async fn connect(&mut self) -> ExchangeResult<()> {
        // TODO: Implement HTTP streaming connection
        // This requires using reqwest with stream feature
        Err(ExchangeError::UnsupportedOperation(
            "HTTP streaming not yet implemented - use REST polling for now".to_string()
        ))
    }

    /// Get next message from stream
    pub async fn next_message(&mut self) -> ExchangeResult<StreamMessage> {
        // TODO: Read next line from HTTP stream and parse JSON
        Err(ExchangeError::UnsupportedOperation(
            "HTTP streaming not yet implemented - use REST polling for now".to_string()
        ))
    }

    /// Close the stream
    pub async fn close(&mut self) -> ExchangeResult<()> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSACTION STREAM
// ═══════════════════════════════════════════════════════════════════════════════

/// HTTP streaming client for OANDA transaction stream
pub struct TransactionStream {
    _auth: OandaAuth,
    _urls: OandaUrls,
    _account_id: String,
}

impl TransactionStream {
    /// Create new transaction stream
    pub fn new(
        auth: OandaAuth,
        urls: OandaUrls,
        account_id: String,
    ) -> Self {
        Self {
            _auth: auth,
            _urls: urls,
            _account_id: account_id,
        }
    }

    /// Connect to transaction stream
    pub async fn connect(&mut self) -> ExchangeResult<()> {
        // TODO: Implement HTTP streaming connection
        Err(ExchangeError::UnsupportedOperation(
            "HTTP streaming not yet implemented - use REST polling for now".to_string()
        ))
    }

    /// Get next message from stream
    pub async fn next_message(&mut self) -> ExchangeResult<StreamMessage> {
        // TODO: Read next line from HTTP stream and parse JSON
        Err(ExchangeError::UnsupportedOperation(
            "HTTP streaming not yet implemented - use REST polling for now".to_string()
        ))
    }

    /// Close the stream
    pub async fn close(&mut self) -> ExchangeResult<()> {
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse streaming message
fn parse_message(line: &str) -> ExchangeResult<StreamMessage> {
    let value: Value = serde_json::from_str(line)
        .map_err(|e| ExchangeError::Parse(format!("Failed to parse JSON: {}", e)))?;

    let msg_type = value.get("type")
        .and_then(|t| t.as_str())
        .ok_or_else(|| ExchangeError::Parse("Missing 'type' field".to_string()))?;

    match msg_type {
        "PRICE" => {
            let instrument = value.get("instrument")
                .and_then(|i| i.as_str())
                .ok_or_else(|| ExchangeError::Parse("Missing 'instrument'".to_string()))?
                .to_string();

            let time = value.get("time")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            let tradeable = value.get("tradeable")
                .and_then(|t| t.as_bool())
                .unwrap_or(false);

            // Get best bid/ask
            let bid = value.get("bids")
                .and_then(|b| b.as_array())
                .and_then(|arr| arr.first())
                .and_then(|b| b.get("price"))
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            let ask = value.get("asks")
                .and_then(|a| a.as_array())
                .and_then(|arr| arr.first())
                .and_then(|a| a.get("price"))
                .and_then(|p| p.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(0.0);

            let closeout_bid = value.get("closeoutBid")
                .and_then(|c| c.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(bid);

            let closeout_ask = value.get("closeoutAsk")
                .and_then(|c| c.as_str())
                .and_then(|s| s.parse().ok())
                .unwrap_or(ask);

            Ok(StreamMessage::Price(PriceUpdate {
                instrument,
                time,
                tradeable,
                bid,
                ask,
                closeout_bid,
                closeout_ask,
            }))
        }
        "HEARTBEAT" => {
            let time = value.get("time")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            Ok(StreamMessage::Heartbeat { time })
        }
        _ => {
            // Transaction or other message type
            Ok(StreamMessage::Transaction(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_parse_price_message() {
        let json_str = r#"{
            "type": "PRICE",
            "time": "2026-01-26T12:34:56.789123456Z",
            "instrument": "EUR_USD",
            "tradeable": true,
            "bids": [{"price": "1.12157", "liquidity": 10000000}],
            "asks": [{"price": "1.12170", "liquidity": 10000000}],
            "closeoutBid": "1.12153",
            "closeoutAsk": "1.12174"
        }"#;

        let msg = parse_message(json_str).unwrap();
        match msg {
            StreamMessage::Price(price) => {
                assert_eq!(price.instrument, "EUR_USD");
                assert!(price.tradeable);
                assert!((price.bid - 1.12157).abs() < 0.00001);
                assert!((price.ask - 1.12170).abs() < 0.00001);
            }
            _ => panic!("Expected Price message"),
        }
    }

    #[test]
    fn test_parse_heartbeat_message() {
        let json_str = r#"{
            "type": "HEARTBEAT",
            "time": "2026-01-26T12:35:01.123456789Z"
        }"#;

        let msg = parse_message(json_str).unwrap();
        match msg {
            StreamMessage::Heartbeat { time } => {
                assert_eq!(time, "2026-01-26T12:35:01.123456789Z");
            }
            _ => panic!("Expected Heartbeat message"),
        }
    }
}
