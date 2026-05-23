use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 32 B record (LE):
///   i64 ts_ms (8), f64 value (8), f64 mark (8), f64 index (8)
///
/// `value = mark - index`. The derived path (`BasisDerived`) always
/// populates all four fields. The legacy WS path (no exchange emits
/// `StreamEvent::Basis` today) populates `mark = NaN, index = NaN`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasisPoint {
    pub ts_ms: i64,
    /// basis = mark − index
    pub value: f64,
    pub mark:  f64,
    pub index: f64,
}

const SIZE: usize = 32;

impl DataPoint for BasisPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.value.to_le_bytes());
        out[16..24].copy_from_slice(&self.mark.to_le_bytes());
        out[24..32].copy_from_slice(&self.index.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            value: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            mark:  f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            index: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    /// WS path: no exchange emits `StreamEvent::Basis` with real data today.
    /// Populates `mark = NaN, index = NaN` for forward-compat if one ever does.
    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Basis { symbol: _, basis, timestamp } = ev {
            Some(Self {
                ts_ms: *timestamp,
                value: *basis,
                mark:  f64::NAN,
                index: f64::NAN,
            })
        } else {
            None
        }
    }
}
