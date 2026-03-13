//! # OANDA HTTP Streaming
//!
//! HTTP streaming implementation for OANDA pricing and transaction streams.
//!
//! IMPORTANT: OANDA uses HTTP streaming (NOT WebSocket)
//! - Uses chunked transfer encoding
//! - Newline-delimited JSON messages
//! - Heartbeats every 5 seconds

use reqwest::Client;
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
    auth: OandaAuth,
    urls: OandaUrls,
    account_id: String,
    instruments: Vec<String>,
    /// Byte buffer for partial lines received from the chunk stream
    line_buf: Vec<u8>,
    /// Pending complete lines waiting to be returned as messages
    pending_lines: std::collections::VecDeque<String>,
    /// Active HTTP response stream
    response: Option<reqwest::Response>,
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
            auth,
            urls,
            account_id,
            instruments,
            line_buf: Vec::new(),
            pending_lines: std::collections::VecDeque::new(),
            response: None,
        }
    }

    /// Connect to pricing stream
    ///
    /// Issues GET `{stream_url}/v3/accounts/{account_id}/pricing/stream?instruments=...`
    /// with `Authorization: Bearer {token}` and keeps the response alive for streaming.
    pub async fn connect(&mut self) -> ExchangeResult<()> {
        let instruments_param = self.instruments.join(",");
        let url = format!(
            "{}/v3/accounts/{}/pricing/stream?instruments={}",
            self.urls.stream_url,
            self.account_id,
            instruments_param
        );

        let client = Client::new();
        let response = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.auth.token()),
            )
            .header("Accept-Datetime-Format", "RFC3339")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to connect to pricing stream: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Http(format!(
                "Pricing stream HTTP {}: {}",
                status, body
            )));
        }

        self.line_buf.clear();
        self.pending_lines.clear();
        self.response = Some(response);
        Ok(())
    }

    /// Get next message from stream
    ///
    /// Reads the next complete newline-delimited JSON line from the HTTP chunked response
    /// and parses it into a [`StreamMessage`].
    pub async fn next_message(&mut self) -> ExchangeResult<StreamMessage> {
        loop {
            // Return any already-buffered complete lines first
            if let Some(line) = self.pending_lines.pop_front() {
                if !line.trim().is_empty() {
                    return parse_message(&line);
                }
                continue;
            }

            // Read next chunk from the HTTP body
            let response = self.response.as_mut().ok_or_else(|| {
                ExchangeError::Network("Not connected — call connect() first".to_string())
            })?;

            let chunk = response.chunk().await
                .map_err(|e| ExchangeError::Network(format!("Stream read error: {}", e)))?;

            match chunk {
                None => {
                    return Err(ExchangeError::Network("Pricing stream closed by server".to_string()));
                }
                Some(bytes) => {
                    // Accumulate bytes into line buffer and split on newlines
                    for b in bytes.iter().copied() {
                        if b == b'\n' {
                            let line = String::from_utf8_lossy(&self.line_buf).into_owned();
                            self.line_buf.clear();
                            if !line.trim().is_empty() {
                                self.pending_lines.push_back(line);
                            }
                        } else {
                            self.line_buf.push(b);
                        }
                    }
                }
            }
        }
    }

    /// Close the stream
    pub async fn close(&mut self) -> ExchangeResult<()> {
        self.response = None;
        self.line_buf.clear();
        self.pending_lines.clear();
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSACTION STREAM
// ═══════════════════════════════════════════════════════════════════════════════

/// HTTP streaming client for OANDA transaction stream
pub struct TransactionStream {
    auth: OandaAuth,
    urls: OandaUrls,
    account_id: String,
    /// Byte buffer for partial lines
    line_buf: Vec<u8>,
    /// Pending complete lines
    pending_lines: std::collections::VecDeque<String>,
    /// Active HTTP response stream
    response: Option<reqwest::Response>,
}

impl TransactionStream {
    /// Create new transaction stream
    pub fn new(
        auth: OandaAuth,
        urls: OandaUrls,
        account_id: String,
    ) -> Self {
        Self {
            auth,
            urls,
            account_id,
            line_buf: Vec::new(),
            pending_lines: std::collections::VecDeque::new(),
            response: None,
        }
    }

    /// Connect to transaction stream
    ///
    /// Issues GET `{stream_url}/v3/accounts/{account_id}/transactions/stream`
    /// with `Authorization: Bearer {token}`.
    pub async fn connect(&mut self) -> ExchangeResult<()> {
        let url = format!(
            "{}/v3/accounts/{}/transactions/stream",
            self.urls.stream_url,
            self.account_id,
        );

        let client = Client::new();
        let response = client
            .get(&url)
            .header(
                "Authorization",
                format!("Bearer {}", self.auth.token()),
            )
            .header("Accept-Datetime-Format", "RFC3339")
            .send()
            .await
            .map_err(|e| ExchangeError::Network(format!("Failed to connect to transaction stream: {}", e)))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(ExchangeError::Api {
                code: status.as_u16() as i32,
                message: format!("Transaction stream HTTP {}: {}", status, body),
            });
        }

        self.line_buf.clear();
        self.pending_lines.clear();
        self.response = Some(response);
        Ok(())
    }

    /// Get next message from stream
    ///
    /// Reads the next complete newline-delimited JSON line and parses it into a [`StreamMessage`].
    pub async fn next_message(&mut self) -> ExchangeResult<StreamMessage> {
        loop {
            if let Some(line) = self.pending_lines.pop_front() {
                if !line.trim().is_empty() {
                    return parse_message(&line);
                }
                continue;
            }

            let response = self.response.as_mut().ok_or_else(|| {
                ExchangeError::Network("Not connected — call connect() first".to_string())
            })?;

            let chunk = response.chunk().await
                .map_err(|e| ExchangeError::Network(format!("Stream read error: {}", e)))?;

            match chunk {
                None => {
                    return Err(ExchangeError::Network("Transaction stream closed by server".to_string()));
                }
                Some(bytes) => {
                    for &b in bytes.iter() {
                        if b == b'\n' {
                            let line = String::from_utf8_lossy(&self.line_buf).into_owned();
                            self.line_buf.clear();
                            if !line.trim().is_empty() {
                                self.pending_lines.push_back(line);
                            }
                        } else {
                            self.line_buf.push(b);
                        }
                    }
                }
            }
        }
    }

    /// Close the stream
    pub async fn close(&mut self) -> ExchangeResult<()> {
        self.response = None;
        self.line_buf.clear();
        self.pending_lines.clear();
        Ok(())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse a newline-delimited streaming message line into a [`StreamMessage`]
pub fn parse_message(line: &str) -> ExchangeResult<StreamMessage> {
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
