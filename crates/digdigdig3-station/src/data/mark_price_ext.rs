//! Extended MarkPrice DataPoint types for Indicators and Full depth.
//!
//! `MarkPricePoint` (Compact, 24 B) is unchanged — see `mark_price.rs`.
//!
//! MarkPrice Full = same fields as Indicators (no extra numeric fields exist
//! beyond what Indicators covers). We define `MarkPriceFullPoint` as a
//! distinct type alias to `MarkPriceIndicatorsPoint` for API consistency, but
//! serialize with the same record size.

use digdigdig3::core::types::{MarkPrice, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

#[inline]
fn opt_f64(v: Option<f64>) -> f64 {
    v.unwrap_or(f64::NAN)
}

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

// ─── MarkPriceIndicatorsPoint ────────────────────────────────────────────────

/// 80 B MarkPrice record for Indicators depth.
///
/// Layout (all LE):
///   u64 ts_ms                                          (8)
///   f64 mark, index                                    (2 × 8 = 16)
///   f64 estimated_settle_price, indicative_settle_price (2 × 8 = 16)
///   f64 funding_rate, indicative_funding_rate          (2 × 8 = 16)
///   u64 next_funding_time (sentinel)                  (8)
///   f64 interest_rate                                  (8)
///   f64 deriv_price                                    (8)
///
/// Total: 8+16+16+16+8+8+8 = 80 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkPriceIndicatorsPoint {
    pub ts_ms: i64,
    pub mark: f64,
    pub index: f64,
    pub estimated_settle_price: f64,
    pub indicative_settle_price: f64,
    pub funding_rate: f64,
    pub indicative_funding_rate: f64,
    pub next_funding_time: Option<i64>,
    pub interest_rate: f64,
    pub deriv_price: f64,
}

const INDICATORS_SIZE: usize = 80;

impl MarkPriceIndicatorsPoint {
    /// Construct from a REST `MarkPrice` snapshot (e.g. from `get_premium_index`).
    pub fn from_mark_price(mp: &MarkPrice) -> Self {
        Self {
            ts_ms: mp.timestamp,
            mark: mp.mark_price,
            index: opt_f64(mp.index_price),
            estimated_settle_price: opt_f64(mp.estimated_settle_price),
            indicative_settle_price: opt_f64(mp.indicative_settle_price),
            funding_rate: opt_f64(mp.funding_rate),
            indicative_funding_rate: opt_f64(mp.indicative_funding_rate),
            next_funding_time: mp.next_funding_time,
            interest_rate: opt_f64(mp.interest_rate),
            deriv_price: opt_f64(mp.deriv_price),
        }
    }
}

impl DataPoint for MarkPriceIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.mark.to_le_bytes());
        out[16..24].copy_from_slice(&self.index.to_le_bytes());
        out[24..32].copy_from_slice(&self.estimated_settle_price.to_le_bytes());
        out[32..40].copy_from_slice(&self.indicative_settle_price.to_le_bytes());
        out[40..48].copy_from_slice(&self.funding_rate.to_le_bytes());
        out[48..56].copy_from_slice(&self.indicative_funding_rate.to_le_bytes());
        out[56..64].copy_from_slice(&opt_i64_enc(self.next_funding_time).to_le_bytes());
        out[64..72].copy_from_slice(&self.interest_rate.to_le_bytes());
        out[72..80].copy_from_slice(&self.deriv_price.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            mark: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            index: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            estimated_settle_price: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            indicative_settle_price: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            funding_rate: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            indicative_funding_rate: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            next_funding_time: opt_i64_dec(u64::from_le_bytes(bytes[56..64].try_into().ok()?)),
            interest_rate: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
            deriv_price: f64::from_le_bytes(bytes[72..80].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::MarkPrice { mark, .. } = ev {
            Some(Self {
                ts_ms: mark.timestamp,
                mark: mark.mark_price,
                index: opt_f64(mark.index_price),
                estimated_settle_price: opt_f64(mark.estimated_settle_price),
                indicative_settle_price: opt_f64(mark.indicative_settle_price),
                funding_rate: opt_f64(mark.funding_rate),
                indicative_funding_rate: opt_f64(mark.indicative_funding_rate),
                next_funding_time: mark.next_funding_time,
                interest_rate: opt_f64(mark.interest_rate),
                deriv_price: opt_f64(mark.deriv_price),
            })
        } else {
            None
        }
    }
}

// ─── MarkPriceFullPoint ───────────────────────────────────────────────────────

/// Full MarkPrice record.
///
/// MarkPrice has no string fields beyond `symbol` (not persisted here — it is
/// part of the file path). All wire numeric fields are included. Full covers
/// every field that Indicators covers plus `fair_price` and `spot_price`.
///
/// Layout (all LE):
///   same as Indicators (80 B) + f64 fair_price + f64 spot_price = 96 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkPriceFullPoint {
    pub ts_ms: i64,
    pub mark: f64,
    pub index: f64,
    pub estimated_settle_price: f64,
    pub indicative_settle_price: f64,
    pub funding_rate: f64,
    pub indicative_funding_rate: f64,
    pub next_funding_time: Option<i64>,
    pub interest_rate: f64,
    pub deriv_price: f64,
    pub fair_price: f64,
    pub spot_price: f64,
}

const FULL_SIZE: usize = 96;

impl MarkPriceFullPoint {
    /// Construct from a REST `MarkPrice` snapshot (e.g. from `get_premium_index`).
    pub fn from_mark_price(mp: &MarkPrice) -> Self {
        Self {
            ts_ms: mp.timestamp,
            mark: mp.mark_price,
            index: opt_f64(mp.index_price),
            estimated_settle_price: opt_f64(mp.estimated_settle_price),
            indicative_settle_price: opt_f64(mp.indicative_settle_price),
            funding_rate: opt_f64(mp.funding_rate),
            indicative_funding_rate: opt_f64(mp.indicative_funding_rate),
            next_funding_time: mp.next_funding_time,
            interest_rate: opt_f64(mp.interest_rate),
            deriv_price: opt_f64(mp.deriv_price),
            fair_price: opt_f64(mp.fair_price),
            spot_price: opt_f64(mp.spot_price),
        }
    }
}

impl DataPoint for MarkPriceFullPoint {
    const RECORD_SIZE: usize = FULL_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.mark.to_le_bytes());
        out[16..24].copy_from_slice(&self.index.to_le_bytes());
        out[24..32].copy_from_slice(&self.estimated_settle_price.to_le_bytes());
        out[32..40].copy_from_slice(&self.indicative_settle_price.to_le_bytes());
        out[40..48].copy_from_slice(&self.funding_rate.to_le_bytes());
        out[48..56].copy_from_slice(&self.indicative_funding_rate.to_le_bytes());
        out[56..64].copy_from_slice(&opt_i64_enc(self.next_funding_time).to_le_bytes());
        out[64..72].copy_from_slice(&self.interest_rate.to_le_bytes());
        out[72..80].copy_from_slice(&self.deriv_price.to_le_bytes());
        out[80..88].copy_from_slice(&self.fair_price.to_le_bytes());
        out[88..96].copy_from_slice(&self.spot_price.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FULL_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            mark: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            index: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            estimated_settle_price: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            indicative_settle_price: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            funding_rate: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            indicative_funding_rate: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            next_funding_time: opt_i64_dec(u64::from_le_bytes(bytes[56..64].try_into().ok()?)),
            interest_rate: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
            deriv_price: f64::from_le_bytes(bytes[72..80].try_into().ok()?),
            fair_price: f64::from_le_bytes(bytes[80..88].try_into().ok()?),
            spot_price: f64::from_le_bytes(bytes[88..96].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::MarkPrice { mark, .. } = ev {
            Some(Self {
                ts_ms: mark.timestamp,
                mark: mark.mark_price,
                index: opt_f64(mark.index_price),
                estimated_settle_price: opt_f64(mark.estimated_settle_price),
                indicative_settle_price: opt_f64(mark.indicative_settle_price),
                funding_rate: opt_f64(mark.funding_rate),
                indicative_funding_rate: opt_f64(mark.indicative_funding_rate),
                next_funding_time: mark.next_funding_time,
                interest_rate: opt_f64(mark.interest_rate),
                deriv_price: opt_f64(mark.deriv_price),
                fair_price: opt_f64(mark.fair_price),
                spot_price: opt_f64(mark.spot_price),
            })
        } else {
            None
        }
    }
}
