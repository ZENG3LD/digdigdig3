//! Extended OrderBook snapshot DataPoint types for Indicators and Full depth.
//!
//! `ObSnapshotPoint` (Compact, 20 B header + blob) is unchanged.
//!
//! Indicators adds `cts` and `prev_change_id` to the fixed header.
//! Full = same as Indicators for OB (no additional stable numeric fields).
//! `ObSnapshotFullPoint` is an alias to `ObSnapshotIndicatorsPoint`.
//!
//! Layout:
//!   Compact  header: ts_ms(8) | blob_off(8) | blob_len(4)     = 20 B
//!   Indicators header: ts_ms(8) | cts(8) | prev_change_id(8) | blob_off(8) | blob_len(4) = 36 B
//!
//! Blob layout (same for all depths):
//!   u32 bid_count | bid_count × (f64 price, f64 size)
//!   u32 ask_count | ask_count × (f64 price, f64 size)

use digdigdig3::core::types::{OrderBook, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

#[inline]
fn opt_i64_enc(v: Option<i64>) -> u64 {
    match v {
        Some(x) => x as u64,
        None => i64::MIN as u64,
    }
}

#[inline]
fn opt_i64_dec(raw: u64) -> Option<i64> {
    let v = raw as i64;
    if v == i64::MIN { None } else { Some(v) }
}

// ─── shared blob helpers ──────────────────────────────────────────────────────

fn encode_side_blob(levels: &[(f64, f64)], out: &mut Vec<u8>) {
    out.extend_from_slice(&(levels.len() as u32).to_le_bytes());
    for (p, q) in levels {
        out.extend_from_slice(&p.to_le_bytes());
        out.extend_from_slice(&q.to_le_bytes());
    }
}

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

// ─── ObSnapshotIndicatorsPoint ────────────────────────────────────────────────

/// Header-36 B orderbook snapshot for Indicators depth.
///
/// Fixed header (LE):
///   u64 ts_ms (8)
///   u64 cts — cross-transaction timestamp, sentinel i64::MIN (8)
///   u64 prev_change_id — gap-detect field, sentinel i64::MIN (8)
///   u64 blob_off (8), u32 blob_len (4)   ← patched by DiskStore
///
/// Total header: 36 B. Blob = same as Compact (bid/ask level pairs).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObSnapshotIndicatorsPoint {
    pub ts_ms: i64,
    pub cts: Option<i64>,
    pub prev_change_id: Option<i64>,
    pub bids: Vec<(f64, f64)>,
    pub asks: Vec<(f64, f64)>,
}

const INDICATORS_BLOB_OFFSET: usize = 24;
const INDICATORS_SIZE: usize = 36;

impl ObSnapshotIndicatorsPoint {
    pub fn from_orderbook(ob: &OrderBook) -> Self {
        Self {
            ts_ms: ob.timestamp,
            cts: ob.cts,
            prev_change_id: ob.prev_change_id,
            bids: ob.bids.iter().map(|l| (l.price, l.size)).collect(),
            asks: ob.asks.iter().map(|l| (l.price, l.size)).collect(),
        }
    }
}

impl DataPoint for ObSnapshotIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&opt_i64_enc(self.cts).to_le_bytes());
        out[16..24].copy_from_slice(&opt_i64_enc(self.prev_change_id).to_le_bytes());
        // [24..36] = blob pointer, patched by DiskStore
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            cts: opt_i64_dec(u64::from_le_bytes(bytes[8..16].try_into().ok()?)),
            prev_change_id: opt_i64_dec(u64::from_le_bytes(bytes[16..24].try_into().ok()?)),
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

    fn blob_pointer_offset() -> Option<usize> { Some(INDICATORS_BLOB_OFFSET) }
}

// Full = Indicators for OB snapshots (no extra numeric fields).
/// Full OB snapshot = Indicators (36 B header + blob). OB has no additional
/// stable numeric fields beyond what Indicators captures.
pub type ObSnapshotFullPoint = ObSnapshotIndicatorsPoint;
