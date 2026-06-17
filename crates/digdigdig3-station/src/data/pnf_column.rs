use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// One Point-and-Figure column (X or O).
///
/// `open_time` is the timestamp at which the column opened (Station
/// upsert key). `column_id` is a strictly monotonic per-Derived-instance
/// counter — consumers group emit-snapshots into the same column by
/// `column_id` rather than by `(top, bottom)` (which mutates as the
/// column extends).
///
/// `is_x = true` ⇒ rising column drawn with "X" glyphs.
/// `is_x = false` ⇒ falling column drawn with "O" glyphs.
///
/// 56-byte fixed record (LE):
///   u64 open_time_ms
///   u64 column_id
///   f64 bottom, top, volume
///   u64 trades_count
///   u8  is_x
///   7 bytes reserved (zero-padded)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnfColumnPoint {
    pub open_time: i64,
    pub column_id: u64,
    pub is_x: bool,
    pub bottom: f64,
    pub top: f64,
    pub volume: f64,
    pub trades_count: u64,
}

const SIZE: usize = 56;

impl DataPoint for PnfColumnPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.open_time as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.column_id.to_le_bytes());
        out[16..24].copy_from_slice(&self.bottom.to_le_bytes());
        out[24..32].copy_from_slice(&self.top.to_le_bytes());
        out[32..40].copy_from_slice(&self.volume.to_le_bytes());
        out[40..48].copy_from_slice(&self.trades_count.to_le_bytes());
        out[48] = self.is_x as u8;
        // bytes 49..56 reserved (zero already).
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            open_time: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            column_id: u64::from_le_bytes(bytes[8..16].try_into().ok()?),
            bottom: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            top: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            volume: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            trades_count: u64::from_le_bytes(bytes[40..48].try_into().ok()?),
            is_x: bytes[48] != 0,
        })
    }

    fn timestamp_ms(&self) -> i64 { self.open_time }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> {
        // PnfColumnPoint is a derived type (TradeToPnfBarDerived), never
        // sourced directly from a wire StreamEvent.
        None
    }
}
