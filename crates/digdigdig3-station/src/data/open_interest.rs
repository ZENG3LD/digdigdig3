use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 24 B record (LE): u64 ts, f64 oi, f64 oi_value (NaN if absent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenInterestPoint {
    pub ts_ms: i64,
    pub open_interest: f64,
    pub open_interest_value: f64,
}

const SIZE: usize = 24;

impl DataPoint for OpenInterestPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.open_interest.to_le_bytes());
        out[16..24].copy_from_slice(&self.open_interest_value.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            open_interest: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            open_interest_value: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OpenInterestUpdate { open_interest, open_interest_value, timestamp, .. } = ev {
            Some(Self {
                ts_ms: *timestamp,
                open_interest: *open_interest,
                open_interest_value: open_interest_value.unwrap_or(f64::NAN),
            })
        } else {
            None
        }
    }
}
