use digdigdig3::core::types::{OrderSide, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// In-memory only. `order_id` and `action` are variable-length strings;
/// disk persistence is disabled for `Kind::OrderbookL3`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookL3Point {
    pub ts_ms: i64,
    pub side: OrderSide,
    pub order_id: String,
    pub price: f64,
    pub quantity: f64,
    pub action: String,
}

impl DataPoint for OrderbookL3Point {
    const RECORD_SIZE: usize = 8;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            side: OrderSide::Buy,
            order_id: String::new(),
            price: 0.0,
            quantity: 0.0,
            action: String::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderbookL3 {
            symbol: _,
            side,
            order_id,
            price,
            quantity,
            action,
            timestamp,
        } = ev {
            Some(Self {
                ts_ms: *timestamp,
                side: *side,
                order_id: order_id.clone(),
                price: *price,
                quantity: *quantity,
                action: action.clone(),
            })
        } else {
            None
        }
    }
}
