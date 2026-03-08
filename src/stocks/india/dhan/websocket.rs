//! # Dhan WebSocket
//!
//! WebSocket connection for Dhan real-time data.
//!
//! ## Binary Format
//!
//! Dhan uses Little Endian binary format for WebSocket messages.
//! See research/websocket_full.md for complete binary packet structures.
//!
//! ## Channels
//!
//! - Ticker (52 bytes) - LTP, volume, OI
//! - Quote (180 bytes) - Full market depth (5 levels)
//! - Full Packet (652 bytes) - Complete market data
//! - Order Updates - Real-time order status
//!
//! ## Connection
//!
//! ```ignore
//! wss://api-feed.dhan.co?token={JWT}&version=2
//! ```

use std::collections::HashMap;
use serde_json::json;

use crate::core::{
    ExchangeResult, ExchangeError,
};

/// Dhan WebSocket client
pub struct DhanWebSocket {
    access_token: String,
}

impl DhanWebSocket {
    /// Create new WebSocket client
    pub fn new(access_token: String) -> Self {
        Self { access_token }
    }

    /// Build WebSocket URL with authentication
    pub fn build_url(&self) -> String {
        format!("wss://api-feed.dhan.co?token={}&version=2", self.access_token)
    }

    /// Build subscription message for Ticker channel
    ///
    /// # Request Code
    /// - 15: Ticker
    /// - 16: Quote
    /// - 17: Full Packet
    /// - 21: Market Depth (20-level)
    /// - 22: Market Depth (200-level)
    pub fn build_subscription(&self, request_code: u8, instruments: Vec<(u8, &str)>) -> String {
        let instrument_list: Vec<_> = instruments
            .iter()
            .map(|(segment, security_id)| {
                json!({
                    "ExchangeSegment": segment,
                    "SecurityId": security_id
                })
            })
            .collect();

        json!({
            "RequestCode": request_code,
            "InstrumentCount": instruments.len(),
            "InstrumentList": instrument_list
        })
        .to_string()
    }

    /// Parse binary ticker packet (52 bytes, Little Endian)
    ///
    /// # Packet Structure
    /// - \[0-1\]: Exchange Segment (u16)
    /// - \[2-6\]: Security ID (u32)
    /// - \[6-10\]: LTP (f32)
    /// - \[10-14\]: Volume (i32)
    /// - \[14-18\]: Open Interest (i32)
    /// - ... (see research/websocket_full.md for full structure)
    pub fn parse_ticker_packet(&self, data: &[u8]) -> ExchangeResult<HashMap<String, f64>> {
        if data.len() < 52 {
            return Err(ExchangeError::Parse(format!(
                "Invalid ticker packet size: {} (expected 52)",
                data.len()
            )));
        }

        // Parse using Little Endian byte order
        use byteorder::{LittleEndian, ByteOrder};

        let mut result = HashMap::new();

        // Exchange Segment (u16 at offset 0)
        let exchange_segment = LittleEndian::read_u16(&data[0..2]);
        result.insert("exchange_segment".to_string(), exchange_segment as f64);

        // Security ID (u32 at offset 2)
        let security_id = LittleEndian::read_u32(&data[2..6]);
        result.insert("security_id".to_string(), security_id as f64);

        // LTP (f32 at offset 6)
        let ltp = LittleEndian::read_f32(&data[6..10]);
        result.insert("ltp".to_string(), ltp as f64);

        // Volume (i32 at offset 10)
        let volume = LittleEndian::read_i32(&data[10..14]);
        result.insert("volume".to_string(), volume as f64);

        // Open Interest (i32 at offset 14)
        let open_interest = LittleEndian::read_i32(&data[14..18]);
        result.insert("open_interest".to_string(), open_interest as f64);

        Ok(result)
    }

    /// Parse binary quote packet (180 bytes, Little Endian)
    ///
    /// Contains full 5-level market depth
    pub fn parse_quote_packet(&self, data: &[u8]) -> ExchangeResult<HashMap<String, f64>> {
        if data.len() < 180 {
            return Err(ExchangeError::Parse(format!(
                "Invalid quote packet size: {} (expected 180)",
                data.len()
            )));
        }

        // TODO: Implement full quote packet parsing
        // See research/websocket_full.md for complete structure

        let mut result = HashMap::new();
        result.insert("packet_size".to_string(), data.len() as f64);

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_url() {
        let ws = DhanWebSocket::new("test_token".to_string());
        let url = ws.build_url();
        assert!(url.contains("wss://api-feed.dhan.co"));
        assert!(url.contains("token=test_token"));
    }

    #[test]
    fn test_build_subscription() {
        let ws = DhanWebSocket::new("test_token".to_string());
        let sub = ws.build_subscription(15, vec![(0, "1333")]);

        assert!(sub.contains("RequestCode"));
        assert!(sub.contains("InstrumentCount"));
        assert!(sub.contains("1333"));
    }

    #[test]
    fn test_parse_ticker_packet_invalid_size() {
        let ws = DhanWebSocket::new("test_token".to_string());
        let data = vec![0u8; 10]; // Too small

        let result = ws.parse_ticker_packet(&data);
        assert!(result.is_err());
    }
}
