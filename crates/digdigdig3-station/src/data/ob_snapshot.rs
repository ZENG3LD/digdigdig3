use digdigdig3::core::types::{OrderBook, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Order-book snapshot — variable depth.
///
/// Depth is whatever the exchange/connector delivered (`OrderBook.bids/asks`
/// is a full `Vec`, no truncation). REST snapshots can be 1000+ levels deep
/// and WS snapshots vary per venue, so a fixed level cap would silently drop
/// real depth on both the channel `Event` and persistence. Stored as a fixed
/// 20-byte header (`ts_ms` + blob pointer) plus a companion `.blob` file
/// holding the variable-length level pairs.
///
/// Blob layout (LE):
///   u32 bid_count | bid_count * (f64 price, f64 size)
///   u32 ask_count | ask_count * (f64 price, f64 size)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObSnapshotPoint {
    pub ts_ms: i64,
    pub bids: Vec<(f64, f64)>, // sorted high→low
    pub asks: Vec<(f64, f64)>, // sorted low→high
}

/// Header = `ts_ms (8) | blob_off (8) | blob_len (4)` = 20 B.
const TAIL_OFFSET: usize = 8;
const HEADER_SIZE: usize = 20;

impl ObSnapshotPoint {
    pub fn from_orderbook(ob: &OrderBook) -> Self {
        // Full depth — no truncation. Keep every level the connector parsed.
        let bids = ob.bids.iter().map(|l| (l.price, l.size)).collect();
        let asks = ob.asks.iter().map(|l| (l.price, l.size)).collect();
        Self { ts_ms: ob.timestamp, bids, asks }
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

/// Decode a side written by [`encode_side_blob`]; returns levels + bytes used.
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

impl DataPoint for ObSnapshotPoint {
    const RECORD_SIZE: usize = HEADER_SIZE;

    fn encode(&self, out: &mut [u8]) {
        // Header only: ts_ms. The (blob_off, blob_len) tail at TAIL_OFFSET is
        // patched by DiskStore after encode runs.
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        // Header-only decode (blob unavailable): timestamp survives, levels
        // empty. decode_blob reconstructs the levels from the companion file.
        if bytes.len() != Self::RECORD_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            bids: Vec::new(),
            asks: Vec::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderbookSnapshot { book, .. } = ev {
            Some(Self::from_orderbook(book))
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(8 + (self.bids.len() + self.asks.len()) * 16);
        encode_side_blob(&self.bids, &mut out);
        encode_side_blob(&self.asks, &mut out);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        if let Some((bids, consumed)) = decode_side_blob(blob) {
            p.bids = bids;
            if let Some((asks, _)) = decode_side_blob(&blob[consumed..]) {
                p.asks = asks;
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
    fn from_orderbook_keeps_full_depth() {
        // 200 levels per side — a deep REST snapshot well past the old 25 cap.
        let bids: Vec<OrderBookLevel> =
            (0..200).map(|i| lvl(1000.0 - i as f64, i as f64 + 1.0)).collect();
        let asks: Vec<OrderBookLevel> =
            (0..200).map(|i| lvl(1001.0 + i as f64, i as f64 + 1.0)).collect();
        let ob = OrderBook { bids, asks, timestamp: 555, ..Default::default() };
        let p = ObSnapshotPoint::from_orderbook(&ob);
        assert_eq!(p.bids.len(), 200);
        assert_eq!(p.asks.len(), 200);
    }

    #[test]
    fn blob_round_trip_deep_snapshot() {
        let bids: Vec<(f64, f64)> = (0..150).map(|i| (1000.0 - i as f64, i as f64 + 1.0)).collect();
        let asks: Vec<(f64, f64)> = (0..120).map(|i| (1001.0 + i as f64, i as f64 + 1.0)).collect();
        let p = ObSnapshotPoint { ts_ms: 777, bids: bids.clone(), asks: asks.clone() };

        let mut header = vec![0u8; ObSnapshotPoint::RECORD_SIZE];
        p.encode(&mut header);
        let blob = p.encode_blob().expect("blob");

        let back = ObSnapshotPoint::decode_blob(&header, &blob).expect("decode_blob");
        assert_eq!(back.ts_ms, 777);
        assert_eq!(back.bids, bids);
        assert_eq!(back.asks, asks);
    }

    #[test]
    fn empty_snapshot_round_trip() {
        let p = ObSnapshotPoint { ts_ms: 3, bids: vec![], asks: vec![] };
        let mut header = vec![0u8; ObSnapshotPoint::RECORD_SIZE];
        p.encode(&mut header);
        let blob = p.encode_blob().expect("blob");
        let back = ObSnapshotPoint::decode_blob(&header, &blob).expect("decode_blob");
        assert!(back.bids.is_empty());
        assert!(back.asks.is_empty());
    }
}
