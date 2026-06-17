use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// One Kagi segment (vertical) or horizontal connector.
///
/// `up` — direction of the segment (true = up segment).
/// `yang` — line thickness flag (true = thick "yang", false = thin "yin").
///   Flips when the segment crosses the prior shoulder (up segments) or
///   waist (down segments).
/// `is_connector` — true for horizontal connector segments emitted at
///   direction flips; false for the main vertical stems.
///
/// 48-byte fixed record (LE):
///   u64 open_time_ms
///   f64 start_price, end_price
///   u8  up, yang, is_connector
///   21 bytes reserved
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KagiSegmentPoint {
    pub open_time: i64,
    pub start_price: f64,
    pub end_price: f64,
    pub up: bool,
    pub yang: bool,
    pub is_connector: bool,
}

const SIZE: usize = 48;

impl DataPoint for KagiSegmentPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.open_time as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.start_price.to_le_bytes());
        out[16..24].copy_from_slice(&self.end_price.to_le_bytes());
        out[24] = self.up as u8;
        out[25] = self.yang as u8;
        out[26] = self.is_connector as u8;
        // bytes 27..48 reserved.
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            open_time: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            start_price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            end_price: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            up: bytes[24] != 0,
            yang: bytes[25] != 0,
            is_connector: bytes[26] != 0,
        })
    }

    fn timestamp_ms(&self) -> i64 { self.open_time }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> { None }
}
