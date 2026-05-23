use digdigdig3::core::types::{OrderbookDelta, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Top-N delta changes per side, fixed size.
///
/// A delta carries the levels whose `size` changed since the previous
/// frame. `size == 0.0` means the level was removed; otherwise the level
/// was inserted (new price) or updated (existing price, new size). The
/// consumer composes a live book by applying deltas on top of a snapshot.
pub const OB_DELTA_LEVELS_PER_SIDE: usize = 25;

/// OB delta record (LE):
///   u64 ts_ms
///   25 * (f64 bid_price, f64 bid_size) = 400 B
///   25 * (f64 ask_price, f64 ask_size) = 400 B
///   Total = 8 + 400 + 400 = 808 B
///
/// Absent change-slots are zero-padded. A wire delta carrying > 25
/// changes per side is truncated to the first 25; this is the same
/// guarantee `ObSnapshotPoint` makes about depth.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObDeltaPoint {
    pub ts_ms: i64,
    /// Bid-side changes: (price, new_size). `new_size == 0.0` ⇒ remove.
    pub bid_changes: Vec<(f64, f64)>,
    /// Ask-side changes: (price, new_size). `new_size == 0.0` ⇒ remove.
    pub ask_changes: Vec<(f64, f64)>,
}

const PAIR: usize = 16; // f64 + f64
const SIDE_BYTES: usize = OB_DELTA_LEVELS_PER_SIDE * PAIR;
const SIZE: usize = 8 + SIDE_BYTES + SIDE_BYTES;

impl ObDeltaPoint {
    pub fn from_delta(d: &OrderbookDelta) -> Self {
        let bid_changes = d
            .bids
            .iter()
            .take(OB_DELTA_LEVELS_PER_SIDE)
            .map(|l| (l.price, l.size))
            .collect();
        let ask_changes = d
            .asks
            .iter()
            .take(OB_DELTA_LEVELS_PER_SIDE)
            .map(|l| (l.price, l.size))
            .collect();
        Self { ts_ms: d.timestamp, bid_changes, ask_changes }
    }
}

impl DataPoint for ObDeltaPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        encode_side(&self.bid_changes, &mut out[8..(8 + SIDE_BYTES)]);
        encode_side(&self.ask_changes, &mut out[(8 + SIDE_BYTES)..(8 + 2 * SIDE_BYTES)]);
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        let ts_ms = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64;
        let bid_changes = decode_side(&bytes[8..(8 + SIDE_BYTES)]);
        let ask_changes = decode_side(&bytes[(8 + SIDE_BYTES)..(8 + 2 * SIDE_BYTES)]);
        Some(Self { ts_ms, bid_changes, ask_changes })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderbookDelta { delta, .. } = ev {
            Some(Self::from_delta(delta))
        } else {
            None
        }
    }
}

fn encode_side(levels: &[(f64, f64)], out: &mut [u8]) {
    for (i, (p, q)) in levels.iter().take(OB_DELTA_LEVELS_PER_SIDE).enumerate() {
        let off = i * PAIR;
        out[off..off + 8].copy_from_slice(&p.to_le_bytes());
        out[off + 8..off + 16].copy_from_slice(&q.to_le_bytes());
    }
}

fn decode_side(bytes: &[u8]) -> Vec<(f64, f64)> {
    let mut out = Vec::with_capacity(OB_DELTA_LEVELS_PER_SIDE);
    for i in 0..OB_DELTA_LEVELS_PER_SIDE {
        let off = i * PAIR;
        let p = f64::from_le_bytes(bytes[off..off + 8].try_into().unwrap_or_default());
        let q = f64::from_le_bytes(bytes[off + 8..off + 16].try_into().unwrap_or_default());
        // A delta CAN legitimately carry `q == 0.0` for a removal at a
        // non-zero price. Only treat `(0.0, 0.0)` as zero-padding.
        if p == 0.0 && q == 0.0 {
            break;
        }
        out.push((p, q));
    }
    out
}
