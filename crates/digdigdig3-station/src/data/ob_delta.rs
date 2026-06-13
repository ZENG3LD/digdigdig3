use digdigdig3::core::types::{OrderbookDelta, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Incremental order-book delta — variable depth.
///
/// A delta carries the levels whose `size` changed since the previous
/// frame. `size == 0.0` means the level was removed; otherwise the level
/// was inserted (new price) or updated (existing price, new size). The
/// consumer composes a live book by applying deltas on top of a snapshot.
///
/// Depth is whatever the exchange sent — `OrderbookDelta.bids/asks` is a
/// full `Vec` from the connector, never truncated. The on-disk record is a
/// fixed 20-byte header (`ts_ms` + blob pointer) plus a companion `.blob`
/// file holding the variable-length level pairs, so no hard level cap leaks
/// into either the channel `Event` or persistence.
///
/// Blob layout (LE):
///   u32 bid_count | bid_count * (f64 price, f64 size)
///   u32 ask_count | ask_count * (f64 price, f64 size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObDeltaPoint {
    pub ts_ms: i64,
    /// Bid-side changes: (price, new_size). `new_size == 0.0` ⇒ remove.
    pub bid_changes: Vec<(f64, f64)>,
    /// Ask-side changes: (price, new_size). `new_size == 0.0` ⇒ remove.
    pub ask_changes: Vec<(f64, f64)>,
}

/// Header = `ts_ms (8) | blob_off (8) | blob_len (4)` = 20 B.
const TAIL_OFFSET: usize = 8;
const HEADER_SIZE: usize = 20;

impl ObDeltaPoint {
    pub fn from_delta(d: &OrderbookDelta) -> Self {
        // Full depth — no truncation. The exchange/connector decides how many
        // levels a delta carries; we keep them all.
        let bid_changes = d.bids.iter().map(|l| (l.price, l.size)).collect();
        let ask_changes = d.asks.iter().map(|l| (l.price, l.size)).collect();
        Self { ts_ms: d.timestamp, bid_changes, ask_changes }
    }
}

/// Encode a side as `u32 count | count * (f64 price, f64 size)`.
fn encode_side_blob(levels: &[(f64, f64)], out: &mut Vec<u8>) {
    out.extend_from_slice(&(levels.len() as u32).to_le_bytes());
    for (p, q) in levels {
        out.extend_from_slice(&p.to_le_bytes());
        out.extend_from_slice(&q.to_le_bytes());
    }
}

/// Decode a side written by [`encode_side_blob`]. Returns the parsed levels
/// and the number of bytes consumed, or `None` on a malformed slice.
fn decode_side_blob(blob: &[u8]) -> Option<(Vec<(f64, f64)>, usize)> {
    if blob.len() < 4 {
        return None;
    }
    let count = u32::from_le_bytes(blob[0..4].try_into().ok()?) as usize;
    let need = 4 + count * 16;
    if blob.len() < need {
        return None;
    }
    let mut levels = Vec::with_capacity(count);
    let mut off = 4;
    for _ in 0..count {
        let p = f64::from_le_bytes(blob[off..off + 8].try_into().ok()?);
        let q = f64::from_le_bytes(blob[off + 8..off + 16].try_into().ok()?);
        levels.push((p, q));
        off += 16;
    }
    Some((levels, need))
}

impl DataPoint for ObDeltaPoint {
    const RECORD_SIZE: usize = HEADER_SIZE;

    fn encode(&self, out: &mut [u8]) {
        // Header only: ts_ms. The (blob_off, blob_len) tail at TAIL_OFFSET is
        // patched by DiskStore after encode runs.
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        // Header-only decode (blob unavailable): timestamp survives, levels
        // are empty. The blob path (decode_blob) reconstructs the levels.
        if bytes.len() != Self::RECORD_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            bid_changes: Vec::new(),
            ask_changes: Vec::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderbookDelta { delta, .. } = ev {
            Some(Self::from_delta(delta))
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(
            8 + (self.bid_changes.len() + self.ask_changes.len()) * 16,
        );
        encode_side_blob(&self.bid_changes, &mut out);
        encode_side_blob(&self.ask_changes, &mut out);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        if let Some((bids, consumed)) = decode_side_blob(blob) {
            p.bid_changes = bids;
            if let Some((asks, _)) = decode_side_blob(&blob[consumed..]) {
                p.ask_changes = asks;
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use digdigdig3::core::types::OrderBookLevel;

    fn lvl(price: f64, size: f64) -> OrderBookLevel {
        OrderBookLevel { price, size, order_count: None }
    }

    #[test]
    fn from_delta_keeps_full_depth() {
        // 40 levels per side — far past the old 25 cap.
        let bids: Vec<OrderBookLevel> = (0..40).map(|i| lvl(100.0 - i as f64, i as f64)).collect();
        let asks: Vec<OrderBookLevel> = (0..40).map(|i| lvl(101.0 + i as f64, i as f64)).collect();
        let d = OrderbookDelta { bids, asks, timestamp: 1234, ..Default::default() };
        let p = ObDeltaPoint::from_delta(&d);
        assert_eq!(p.bid_changes.len(), 40);
        assert_eq!(p.ask_changes.len(), 40);
    }

    #[test]
    fn blob_round_trip_full_depth() {
        let bids: Vec<(f64, f64)> = (0..30).map(|i| (100.0 - i as f64, i as f64)).collect();
        let asks: Vec<(f64, f64)> = (0..50).map(|i| (101.0 + i as f64, i as f64)).collect();
        let p = ObDeltaPoint { ts_ms: 9999, bid_changes: bids.clone(), ask_changes: asks.clone() };

        let mut header = vec![0u8; ObDeltaPoint::RECORD_SIZE];
        p.encode(&mut header);
        let blob = p.encode_blob().expect("blob");

        let back = ObDeltaPoint::decode_blob(&header, &blob).expect("decode_blob");
        assert_eq!(back.ts_ms, 9999);
        assert_eq!(back.bid_changes, bids);
        assert_eq!(back.ask_changes, asks);
    }

    #[test]
    fn blob_round_trip_with_removal_zero_size() {
        // A removal carries (price, 0.0) at a non-zero price — must survive.
        let p = ObDeltaPoint {
            ts_ms: 1,
            bid_changes: vec![(99.0, 0.0), (98.5, 3.0)],
            ask_changes: vec![(101.0, 0.0)],
        };
        let mut header = vec![0u8; ObDeltaPoint::RECORD_SIZE];
        p.encode(&mut header);
        let blob = p.encode_blob().expect("blob");
        let back = ObDeltaPoint::decode_blob(&header, &blob).expect("decode_blob");
        assert_eq!(back.bid_changes, vec![(99.0, 0.0), (98.5, 3.0)]);
        assert_eq!(back.ask_changes, vec![(101.0, 0.0)]);
    }

    #[test]
    fn header_only_decode_empty_levels() {
        let p = ObDeltaPoint { ts_ms: 42, bid_changes: vec![(1.0, 2.0)], ask_changes: vec![] };
        let mut header = vec![0u8; ObDeltaPoint::RECORD_SIZE];
        p.encode(&mut header);
        let back = ObDeltaPoint::decode(&header).expect("decode");
        assert_eq!(back.ts_ms, 42);
        assert!(back.bid_changes.is_empty());
        assert!(back.ask_changes.is_empty());
    }
}
