use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 56 B fixed record (LE):
///   i64 ts_ms (8), f64 long_liq_size (8), f64 short_liq_size (8),
///   f64 long_liq_amount (8), f64 short_liq_amount (8),
///   f64 long_liq_usd (8), f64 short_liq_usd (8)
///
/// All `f64` fields store `f64::NAN` when the wire response omits that field
/// (all are `Option<f64>` in the source `LiquidationBucket` struct).
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct LiquidationBucketPoint {
    /// Unix timestamp in milliseconds.
    pub ts_ms: i64,
    /// Long liquidation size in contracts (GateIO `long_liq_size`).
    /// `f64::NAN` when absent.
    pub long_liq_size: f64,
    /// Short liquidation size in contracts (GateIO `short_liq_size`).
    /// `f64::NAN` when absent.
    pub short_liq_size: f64,
    /// Long liquidation amount in base (GateIO `long_liq_amount`).
    /// `f64::NAN` when absent.
    pub long_liq_amount: f64,
    /// Short liquidation amount in base (GateIO `short_liq_amount`).
    /// `f64::NAN` when absent.
    pub short_liq_amount: f64,
    /// Long liquidation value in USD (GateIO `long_liq_usd`).
    /// `f64::NAN` when absent.
    pub long_liq_usd: f64,
    /// Short liquidation value in USD (GateIO `short_liq_usd`).
    /// `f64::NAN` when absent.
    pub short_liq_usd: f64,
}

const SIZE: usize = 56;

impl DataPoint for LiquidationBucketPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.long_liq_size.to_le_bytes());
        out[16..24].copy_from_slice(&self.short_liq_size.to_le_bytes());
        out[24..32].copy_from_slice(&self.long_liq_amount.to_le_bytes());
        out[32..40].copy_from_slice(&self.short_liq_amount.to_le_bytes());
        out[40..48].copy_from_slice(&self.long_liq_usd.to_le_bytes());
        out[48..56].copy_from_slice(&self.short_liq_usd.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            long_liq_size: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            short_liq_size: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            long_liq_amount: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            short_liq_amount: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            long_liq_usd: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            short_liq_usd: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> {
        // Poll-only — no WS event produces LiquidationBucketPoint.
        None
    }
}
