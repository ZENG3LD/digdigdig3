//! Extended OrderBook delta DataPoint types for Indicators and Full depth.
//!
//! `ObDeltaPoint` (Compact, 20 B header + blob) is unchanged.
//!
//! Indicators adds `last_update_id`, `first_update_id`, `prev_update_id` and
//! `checksum` to the fixed header (these are the modeling-relevant metadata
//! for sequencing and integrity checking).
//! Full = same as Indicators for OB delta (no additional stable numeric fields).
//! `ObDeltaFullPoint` is an alias to `ObDeltaIndicatorsPoint`.
//!
//! Layout:
//!   Compact header:    ts_ms(8) | blob_off(8) | blob_len(4) = 20 B
//!   Indicators header: ts_ms(8) | last_update_id(8) | first_update_id(8)
//!                    | prev_update_id(8) | checksum(8) | blob_off(8) | blob_len(4) = 52 B
//!
//! All sentinels: i64::MIN for signed, u64::MAX for unsigned absent values.

use digdigdig3::core::types::{OrderbookDelta, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

#[inline]
fn opt_u64_enc(v: Option<u64>) -> u64 {
    v.unwrap_or(u64::MAX)
}

#[inline]
fn opt_u64_dec(raw: u64) -> Option<u64> {
    if raw == u64::MAX { None } else { Some(raw) }
}

#[inline]
fn opt_i64_enc(v: Option<i64>) -> u64 {
    match v { Some(x) => x as u64, None => i64::MIN as u64 }
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
    if blob.len() < 4 { return None; }
    let count = u32::from_le_bytes(blob[0..4].try_into().ok()?) as usize;
    let need = 4 + count * 16;
    if blob.len() < need { return None; }
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

// ─── ObDeltaIndicatorsPoint ───────────────────────────────────────────────────

/// Header-52 B orderbook delta for Indicators depth.
///
/// Fixed header (LE):
///   u64 ts_ms              (8)
///   u64 last_update_id     (8, u64::MAX = absent)
///   u64 first_update_id    (8, u64::MAX = absent)
///   u64 prev_update_id     (8, u64::MAX = absent)
///   u64 checksum           (8, i64::MIN as u64 = absent)
///   u64 blob_off (8), u32 blob_len (4)   ← patched by DiskStore
///
/// Total header: 52 B. Blob = bid/ask level pairs (same as Compact).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObDeltaIndicatorsPoint {
    pub ts_ms: i64,
    pub last_update_id: Option<u64>,
    pub first_update_id: Option<u64>,
    pub prev_update_id: Option<u64>,
    pub checksum: Option<i64>,
    pub bid_changes: Vec<(f64, f64)>,
    pub ask_changes: Vec<(f64, f64)>,
}

const INDICATORS_BLOB_OFFSET: usize = 40;
const INDICATORS_SIZE: usize = 52;

impl ObDeltaIndicatorsPoint {
    pub fn from_delta(d: &OrderbookDelta) -> Self {
        Self {
            ts_ms: d.timestamp,
            last_update_id: d.last_update_id,
            first_update_id: d.first_update_id,
            prev_update_id: d.prev_update_id,
            checksum: d.checksum,
            bid_changes: d.bids.iter().map(|l| (l.price, l.size)).collect(),
            ask_changes: d.asks.iter().map(|l| (l.price, l.size)).collect(),
        }
    }
}

impl DataPoint for ObDeltaIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&opt_u64_enc(self.last_update_id).to_le_bytes());
        out[16..24].copy_from_slice(&opt_u64_enc(self.first_update_id).to_le_bytes());
        out[24..32].copy_from_slice(&opt_u64_enc(self.prev_update_id).to_le_bytes());
        out[32..40].copy_from_slice(&opt_i64_enc(self.checksum).to_le_bytes());
        // [40..52] = blob pointer, patched by DiskStore
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            last_update_id: opt_u64_dec(u64::from_le_bytes(bytes[8..16].try_into().ok()?)),
            first_update_id: opt_u64_dec(u64::from_le_bytes(bytes[16..24].try_into().ok()?)),
            prev_update_id: opt_u64_dec(u64::from_le_bytes(bytes[24..32].try_into().ok()?)),
            checksum: opt_i64_dec(u64::from_le_bytes(bytes[32..40].try_into().ok()?)),
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

    fn blob_pointer_offset() -> Option<usize> { Some(INDICATORS_BLOB_OFFSET) }
}

// Full = Indicators for OB delta (no extra stable numeric fields).
/// Full OB delta = Indicators (52 B header + blob). OB delta has no additional
/// stable numeric fields beyond what Indicators captures.
pub type ObDeltaFullPoint = ObDeltaIndicatorsPoint;
