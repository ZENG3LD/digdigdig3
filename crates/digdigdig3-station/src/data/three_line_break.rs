use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// One Three Line Break line (san-sen-ashi).
///
/// Unlike Renko, a TLB line is NOT fixed-height: its height equals the
/// price move that caused it to print. The key invariant is the
/// **direction** relative to the recent N lines (default N=3), not a
/// fixed box size.
///
/// `direction`: 0 = up, 1 = down.
/// `ts_open` / `ts_close`: millisecond timestamps of the first and
/// last trade that contributed to the line.
/// `volume`: sum of trade volumes that formed the line.
///
/// 48-byte fixed record (LE):
///   i64  ts_open   (8 B)
///   i64  ts_close  (8 B)
///   f64  open      (8 B)
///   f64  close     (8 B)
///   f64  volume    (8 B)
///   u8   direction (1 B)  — 0=up, 1=down
///   7 bytes reserved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreeLineBreakLinePoint {
    pub ts_open: i64,
    pub ts_close: i64,
    /// Price at which the line opened (= previous line's close).
    pub open: f64,
    /// Price at which the line closed (= the trade price that triggered
    /// the line).
    pub close: f64,
    /// Summed trade volume during the line.
    pub volume: f64,
    /// 0 = up line (close > open), 1 = down line (close < open).
    pub direction: u8,
}

const SIZE: usize = 48;

impl DataPoint for ThreeLineBreakLinePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_open as u64).to_le_bytes());
        out[8..16].copy_from_slice(&(self.ts_close as u64).to_le_bytes());
        out[16..24].copy_from_slice(&self.open.to_le_bytes());
        out[24..32].copy_from_slice(&self.close.to_le_bytes());
        out[32..40].copy_from_slice(&self.volume.to_le_bytes());
        out[40] = self.direction;
        // bytes 41..48 reserved.
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        Some(Self {
            ts_open: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            ts_close: u64::from_le_bytes(bytes[8..16].try_into().ok()?) as i64,
            open: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            close: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            volume: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            direction: bytes[40],
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_open
    }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> {
        None
    }
}
