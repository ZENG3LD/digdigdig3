use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// One Renko brick.
///
/// A brick is NOT a time-bucketed OHLCV bar — it has no notion of
/// "open price" separate from one of its edges. The shape is
/// `(bottom, top, up)` plus the timestamp of the trade that closed it,
/// the volume that accumulated while the brick was forming, and the
/// intra-brick adverse extreme (`wick`) so renderers can draw
/// TradingView-style directional wicks.
///
/// `wick` is the distance beyond the brick's body reached by the
/// adverse price excursion while the brick was forming: for an up
/// brick, how far below `bottom` price dipped before recovering to
/// close the brick; for a down brick, how far above `top` price spiked
/// before recovering. `0.0` when no adverse excursion occurred (the
/// common case — most bricks close cleanly at their pinned edge).
///
/// 56-byte fixed record (LE, bumped from 48 B — layout change, old
/// day-files with the 48 B layout become unreadable; dev data, no
/// migration needed):
///   u64 open_time_ms
///   f64 bottom, top, volume, wick
///   u64 trades_count
///   u8  up
///   7 bytes reserved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenkoBrickPoint {
    pub open_time: i64,
    pub bottom: f64,
    pub top: f64,
    pub up: bool,
    pub volume: f64,
    pub trades_count: u64,
    /// Intra-brick adverse extreme beyond the body, `0.0` when none.
    pub wick: f64,
}

const SIZE: usize = 56;

impl DataPoint for RenkoBrickPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.open_time as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.bottom.to_le_bytes());
        out[16..24].copy_from_slice(&self.top.to_le_bytes());
        out[24..32].copy_from_slice(&self.volume.to_le_bytes());
        out[32..40].copy_from_slice(&self.trades_count.to_le_bytes());
        out[40] = self.up as u8;
        // bytes 41..48 reserved.
        out[48..56].copy_from_slice(&self.wick.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            open_time: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            bottom: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            top: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            volume: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            trades_count: u64::from_le_bytes(bytes[32..40].try_into().ok()?),
            up: bytes[40] != 0,
            wick: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.open_time }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> { None }
}
