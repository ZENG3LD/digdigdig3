//! Extended OpenInterest DataPoint types for Indicators and Full depth.
//!
//! `OpenInterestPoint` (Compact, 24 B) is unchanged — see `open_interest.rs`.
//!
//! OpenInterest Full = same numeric fields as Indicators (no extra stable
//! numeric fields exist beyond what Indicators covers; `symbol` is in the
//! file path, `business_type` is a string and rare). Full is a distinct type
//! with the same layout as Indicators.

use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

#[inline]
fn opt_f64(v: Option<f64>) -> f64 {
    v.unwrap_or(f64::NAN)
}

// ─── OpenInterestIndicatorsPoint ─────────────────────────────────────────────

/// 104 B OpenInterest record for Indicators depth.
///
/// Layout (all LE):
///   u64 ts_ms                                          (8)
///   f64 open_interest, open_interest_value            (2 × 8 = 16)
///   f64 open_interest_ccy, open_interest_usd          (2 × 8 = 16)
///   f64 single_open_interest, sum_open_interest       (2 × 8 = 16)
///   f64 single_open_interest_value, sum_open_interest_value (2 × 8 = 16)
///   f64 cmc_circulating_supply                        (8)
///   f64 trade_amount, trade_volume, trade_turnover    (3 × 8 = 24)
///
/// Total: 8+16+16+16+16+8+24 = 104 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenInterestIndicatorsPoint {
    pub ts_ms: i64,
    pub open_interest: f64,
    pub open_interest_value: f64,
    pub open_interest_ccy: f64,
    pub open_interest_usd: f64,
    pub single_open_interest: f64,
    pub sum_open_interest: f64,
    pub single_open_interest_value: f64,
    pub sum_open_interest_value: f64,
    pub cmc_circulating_supply: f64,
    pub trade_amount: f64,
    pub trade_volume: f64,
    pub trade_turnover: f64,
}

const INDICATORS_SIZE: usize = 104;

impl DataPoint for OpenInterestIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.open_interest.to_le_bytes());
        out[16..24].copy_from_slice(&self.open_interest_value.to_le_bytes());
        out[24..32].copy_from_slice(&self.open_interest_ccy.to_le_bytes());
        out[32..40].copy_from_slice(&self.open_interest_usd.to_le_bytes());
        out[40..48].copy_from_slice(&self.single_open_interest.to_le_bytes());
        out[48..56].copy_from_slice(&self.sum_open_interest.to_le_bytes());
        out[56..64].copy_from_slice(&self.single_open_interest_value.to_le_bytes());
        out[64..72].copy_from_slice(&self.sum_open_interest_value.to_le_bytes());
        out[72..80].copy_from_slice(&self.cmc_circulating_supply.to_le_bytes());
        out[80..88].copy_from_slice(&self.trade_amount.to_le_bytes());
        out[88..96].copy_from_slice(&self.trade_volume.to_le_bytes());
        out[96..104].copy_from_slice(&self.trade_turnover.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            open_interest: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            open_interest_value: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            open_interest_ccy: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            open_interest_usd: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            single_open_interest: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            sum_open_interest: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            single_open_interest_value: f64::from_le_bytes(bytes[56..64].try_into().ok()?),
            sum_open_interest_value: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
            cmc_circulating_supply: f64::from_le_bytes(bytes[72..80].try_into().ok()?),
            trade_amount: f64::from_le_bytes(bytes[80..88].try_into().ok()?),
            trade_volume: f64::from_le_bytes(bytes[88..96].try_into().ok()?),
            trade_turnover: f64::from_le_bytes(bytes[96..104].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OpenInterestUpdate { open_interest, .. } = ev {
            Some(Self {
                ts_ms: open_interest.timestamp,
                open_interest: open_interest.open_interest,
                open_interest_value: opt_f64(open_interest.open_interest_value),
                open_interest_ccy: opt_f64(open_interest.open_interest_ccy),
                open_interest_usd: opt_f64(open_interest.open_interest_usd),
                single_open_interest: opt_f64(open_interest.single_open_interest),
                sum_open_interest: opt_f64(open_interest.sum_open_interest),
                single_open_interest_value: opt_f64(open_interest.single_open_interest_value),
                sum_open_interest_value: opt_f64(open_interest.sum_open_interest_value),
                cmc_circulating_supply: opt_f64(open_interest.cmc_circulating_supply),
                trade_amount: opt_f64(open_interest.trade_amount),
                trade_volume: opt_f64(open_interest.trade_volume),
                trade_turnover: opt_f64(open_interest.trade_turnover),
            })
        } else {
            None
        }
    }
}

// ─── OpenInterestFullPoint ────────────────────────────────────────────────────

/// Full OpenInterest record.
///
/// OpenInterest has no extra numeric fields beyond Indicators (the only
/// remaining fields are `symbol: Option<String>` which lives in the file path,
/// and `business_type: Option<String>` which is a rare HTX-specific tag not
/// suitable for fixed-size encoding). Full = Indicators layout, same 104 B.
pub type OpenInterestFullPoint = OpenInterestIndicatorsPoint;
