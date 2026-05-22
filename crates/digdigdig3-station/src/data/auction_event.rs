use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// In-memory only. `auction_id` and `state` are variable-length strings;
/// disk persistence is disabled for `Kind::AuctionEvent`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionEventPoint {
    pub ts_ms: i64,
    pub auction_id: String,
    pub indicative_price: f64,
    pub indicative_qty: f64,
    pub state: String,
}

impl DataPoint for AuctionEventPoint {
    const RECORD_SIZE: usize = 8;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            auction_id: String::new(),
            indicative_price: f64::NAN,
            indicative_qty: f64::NAN,
            state: String::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::AuctionEvent {
            symbol: _,
            auction_id,
            indicative_price,
            indicative_qty,
            state,
            timestamp,
        } = ev {
            Some(Self {
                ts_ms: *timestamp,
                auction_id: auction_id.clone(),
                indicative_price: indicative_price.unwrap_or(f64::NAN),
                indicative_qty: indicative_qty.unwrap_or(f64::NAN),
                state: state.clone(),
            })
        } else {
            None
        }
    }
}
