use digdigdig3::core::types::{OrderBook, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Top-N levels per side, fixed size.
pub const OB_LEVELS_PER_SIDE: usize = 25;

/// OB snapshot record (LE):
///   u64 ts_ms
///   25 * (f64 bid_price, f64 bid_size) = 400 B
///   25 * (f64 ask_price, f64 ask_size) = 400 B
///   Total = 8 + 400 + 400 = 808 B
///
/// Absent levels are zero-padded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObSnapshotPoint {
    pub ts_ms: i64,
    pub bids: Vec<(f64, f64)>, // sorted high→low, up to OB_LEVELS_PER_SIDE
    pub asks: Vec<(f64, f64)>, // sorted low→high, up to OB_LEVELS_PER_SIDE
}

const PAIR: usize = 16; // f64 + f64
const SIDE_BYTES: usize = OB_LEVELS_PER_SIDE * PAIR;
const SIZE: usize = 8 + SIDE_BYTES + SIDE_BYTES;

impl ObSnapshotPoint {
    pub fn from_orderbook(ob: &OrderBook) -> Self {
        let bids = ob
            .bids
            .iter()
            .take(OB_LEVELS_PER_SIDE)
            .map(|l| (l.price, l.size))
            .collect();
        let asks = ob
            .asks
            .iter()
            .take(OB_LEVELS_PER_SIDE)
            .map(|l| (l.price, l.size))
            .collect();
        Self { ts_ms: ob.timestamp, bids, asks }
    }
}

impl DataPoint for ObSnapshotPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        encode_side(&self.bids, &mut out[8..(8 + SIDE_BYTES)]);
        encode_side(&self.asks, &mut out[(8 + SIDE_BYTES)..(8 + 2 * SIDE_BYTES)]);
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        let ts_ms = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64;
        let bids = decode_side(&bytes[8..(8 + SIDE_BYTES)]);
        let asks = decode_side(&bytes[(8 + SIDE_BYTES)..(8 + 2 * SIDE_BYTES)]);
        Some(Self { ts_ms, bids, asks })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderbookSnapshot(ob) = ev {
            Some(Self::from_orderbook(ob))
        } else {
            None
        }
    }
}

fn encode_side(levels: &[(f64, f64)], out: &mut [u8]) {
    for (i, (p, q)) in levels.iter().take(OB_LEVELS_PER_SIDE).enumerate() {
        let off = i * PAIR;
        out[off..off + 8].copy_from_slice(&p.to_le_bytes());
        out[off + 8..off + 16].copy_from_slice(&q.to_le_bytes());
    }
    // remaining bytes stay zero (caller buffer is zero-initialized)
}

fn decode_side(bytes: &[u8]) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(OB_LEVELS_PER_SIDE);
    for i in 0..OB_LEVELS_PER_SIDE {
        let off = i * PAIR;
        let p = f64::from_le_bytes(bytes[off..off + 8].try_into().unwrap_or_default());
        let q = f64::from_le_bytes(bytes[off + 8..off + 16].try_into().unwrap_or_default());
        if p == 0.0 && q == 0.0 {
            break; // zero-padded tail
        }
        out.push((p, q));
    }
    out
}
