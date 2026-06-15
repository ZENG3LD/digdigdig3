use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 32 B fixed record (LE):
///   i64 ts_ms (8), f64 ratio (8), f64 long_pct (8), f64 short_pct (8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongShortRatioPoint {
    /// Unix timestamp in milliseconds.
    pub ts_ms: i64,
    /// long_pct / short_pct (≥ 0).
    pub ratio: f64,
    /// Fraction of long accounts/positions (0.0–1.0).
    pub long_pct: f64,
    /// Fraction of short accounts/positions (0.0–1.0).
    pub short_pct: f64,
}

const SIZE: usize = 32;

impl DataPoint for LongShortRatioPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.ratio.to_le_bytes());
        out[16..24].copy_from_slice(&self.long_pct.to_le_bytes());
        out[24..32].copy_from_slice(&self.short_pct.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            ratio: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            long_pct: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            short_pct: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::LongShortRatio { ratio, .. } = ev {
            // ratio.ratio is Option<f64> (pre-computed by exchange); derive if absent.
            let computed = ratio.ratio.unwrap_or_else(|| {
                if ratio.short_ratio > 0.0 {
                    ratio.long_ratio / ratio.short_ratio
                } else {
                    1.0
                }
            });
            Some(Self {
                ts_ms: ratio.timestamp,
                ratio: computed,
                long_pct: ratio.long_ratio,
                short_pct: ratio.short_ratio,
            })
        } else {
            None
        }
    }
}
