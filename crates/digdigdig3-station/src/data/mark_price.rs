use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 24 B record (LE):
///   u64 ts_ms, f64 mark, f64 index (NaN if absent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkPricePoint {
    pub ts_ms: i64,
    pub mark: f64,
    pub index: f64,
}

const SIZE: usize = 24;

impl DataPoint for MarkPricePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.mark.to_le_bytes());
        out[16..24].copy_from_slice(&self.index.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            mark: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            index: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::MarkPrice { mark_price, index_price, timestamp, .. } = ev {
            Some(Self {
                ts_ms: *timestamp,
                mark: *mark_price,
                index: index_price.unwrap_or(f64::NAN),
            })
        } else {
            None
        }
    }
}
