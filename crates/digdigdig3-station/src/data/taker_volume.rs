use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 48 B fixed record (LE):
///   i64 ts_ms (8), f64 buy_volume (8), f64 sell_volume (8),
///   f64 buy_sell_ratio (8), f64 long_taker_size (8), f64 short_taker_size (8)
///
/// Optional fields (`buy_sell_ratio`, `long_taker_size`, `short_taker_size`) are
/// stored as `f64::NAN` when absent from the wire response.
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TakerVolumePoint {
    /// Unix timestamp in milliseconds.
    pub ts_ms: i64,
    /// Taker buy (aggressor-buy) volume in the bucket.
    pub buy_volume: f64,
    /// Taker sell (aggressor-sell) volume in the bucket.
    pub sell_volume: f64,
    /// Buy/sell ratio precomputed by the venue (Binance `buySellRatio`).
    /// `f64::NAN` when absent.
    pub buy_sell_ratio: f64,
    /// Long-taker size from a bundled stats endpoint (GateIO `long_taker_size`).
    /// `f64::NAN` when absent.
    pub long_taker_size: f64,
    /// Short-taker size from a bundled stats endpoint (GateIO `short_taker_size`).
    /// `f64::NAN` when absent.
    pub short_taker_size: f64,
}

const SIZE: usize = 48;

impl DataPoint for TakerVolumePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.buy_volume.to_le_bytes());
        out[16..24].copy_from_slice(&self.sell_volume.to_le_bytes());
        out[24..32].copy_from_slice(&self.buy_sell_ratio.to_le_bytes());
        out[32..40].copy_from_slice(&self.long_taker_size.to_le_bytes());
        out[40..48].copy_from_slice(&self.short_taker_size.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            buy_volume: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            sell_volume: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            buy_sell_ratio: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            long_taker_size: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            short_taker_size: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> {
        // Poll-only — no WS event produces TakerVolumePoint.
        None
    }
}
