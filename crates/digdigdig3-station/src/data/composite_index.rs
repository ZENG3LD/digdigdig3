use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 24 B record (LE):
///   i64 ts_ms (8), f64 price (8), f64 _pad (8, NaN)
///
/// Component breakdown is not persisted (variable-length); only composite price
/// and timestamp are stored on disk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositeIndexPoint {
    pub ts_ms: i64,
    pub price: f64,
}

const SIZE: usize = 24;

impl DataPoint for CompositeIndexPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&f64::NAN.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::CompositeIndex { symbol: _, price, timestamp, .. } = ev {
            Some(Self { ts_ms: *timestamp, price: *price })
        } else {
            None
        }
    }
}
