//! Extended IndexPrice DataPoint types for Indicators and Full depth.
//!
//! `IndexPricePoint` (Compact, 24 B) is unchanged — see `index_price.rs`.
//!
//! IndexPrice Indicators = price + 24h stats (high/low/open). Full = same
//! as Indicators (IndexPrice currently has only ts + price + 3 24h fields;
//! no extra numeric wire fields exist). `IndexPriceFullPoint` is an alias to
//! `IndexPriceIndicatorsPoint`.

use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

#[inline]
fn opt_f64(v: Option<f64>) -> f64 {
    v.unwrap_or(f64::NAN)
}

// ─── IndexPriceIndicatorsPoint ────────────────────────────────────────────────

/// 40 B IndexPrice record for Indicators depth.
///
/// Layout (all LE):
///   u64 ts_ms        (8)
///   f64 price        (8)
///   f64 high_24h     (8)
///   f64 low_24h      (8)
///   f64 open_24h     (8)
///
/// Total: 40 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPriceIndicatorsPoint {
    pub ts_ms: i64,
    pub price: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub open_24h: f64,
}

const INDICATORS_SIZE: usize = 40;

impl DataPoint for IndexPriceIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.high_24h.to_le_bytes());
        out[24..32].copy_from_slice(&self.low_24h.to_le_bytes());
        out[32..40].copy_from_slice(&self.open_24h.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            high_24h: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            low_24h: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            open_24h: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::IndexPrice { index_price, .. } = ev {
            Some(Self {
                ts_ms: index_price.timestamp,
                price: index_price.price,
                high_24h: opt_f64(index_price.high_24h),
                low_24h: opt_f64(index_price.low_24h),
                open_24h: opt_f64(index_price.open_24h),
            })
        } else {
            None
        }
    }
}

// Full = Indicators for IndexPrice (no extra wire fields exist).
/// IndexPrice Full record = Indicators (40 B). IndexPrice wire carries only
/// ts + price + 3×24h stats; no additional numeric fields exist.
pub type IndexPriceFullPoint = IndexPriceIndicatorsPoint;
