use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 24 B record (LE):
///   u64 ts_ms, f64 rate, i64 next_funding_time_ms (0 if absent)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRatePoint {
    pub ts_ms: i64,
    pub rate: f64,
    pub next_funding_time_ms: i64,
}

const SIZE: usize = 24;

impl DataPoint for FundingRatePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.rate.to_le_bytes());
        out[16..24].copy_from_slice(&(self.next_funding_time_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            rate: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            next_funding_time_ms: u64::from_le_bytes(bytes[16..24].try_into().ok()?) as i64,
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::FundingRate { funding, .. } = ev {
            Some(Self {
                ts_ms: funding.timestamp,
                rate: funding.rate,
                next_funding_time_ms: funding.next_funding_time.unwrap_or(0),
            })
        } else {
            None
        }
    }
}
