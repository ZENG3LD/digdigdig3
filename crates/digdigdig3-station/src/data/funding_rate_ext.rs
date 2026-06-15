//! Extended FundingRate DataPoint types for Indicators and Full depth.
//!
//! `FundingRatePoint` (Compact, 24 B) is unchanged — see `funding_rate.rs`.

use digdigdig3::core::types::{FundingRate, StreamEvent};
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

// ─── FundingRateIndicatorsPoint ───────────────────────────────────────────────

/// 112 B FundingRate record for Indicators depth.
///
/// Layout (all LE):
///   u64 ts_ms                                          (8)
///   f64 rate                                           (8)
///   u64 next_funding_time_ms (sentinel i64::MIN)      (8)
///   f64 mark_price, index_price, prev_index_price     (3 × 8 = 24)
///   f64 premium, interest_rate                        (2 × 8 = 16)
///   f64 realized_rate, estimated_rate                 (2 × 8 = 16)
///   f64 funding_interval_hours                        (8)
///   f64 relative_funding_rate                         (8)
///   f64 accrued_funding                               (8)
///   u64 funding_step (sentinel)                       (8)
///
/// Total: 8+8+8+24+16+16+8+8+8+8 = 112 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRateIndicatorsPoint {
    pub ts_ms: i64,
    pub rate: f64,
    pub next_funding_time_ms: Option<i64>,
    pub mark_price: f64,
    pub index_price: f64,
    pub prev_index_price: f64,
    pub premium: f64,
    pub interest_rate: f64,
    pub realized_rate: f64,
    pub estimated_rate: f64,
    pub funding_interval_hours: f64,
    pub relative_funding_rate: f64,
    pub accrued_funding: f64,
    pub funding_step: Option<i64>,
}

const INDICATORS_SIZE: usize = 112;

impl FundingRateIndicatorsPoint {
    /// Construct from a REST `FundingRate` record (e.g. from `get_funding_rate_history`).
    pub fn from_funding_rate(fr: &FundingRate) -> Self {
        Self {
            ts_ms: fr.timestamp,
            rate: fr.rate,
            next_funding_time_ms: fr.next_funding_time,
            mark_price: opt_f64(fr.mark_price),
            index_price: opt_f64(fr.index_price),
            prev_index_price: opt_f64(fr.prev_index_price),
            premium: opt_f64(fr.premium),
            interest_rate: opt_f64(fr.interest_rate),
            realized_rate: opt_f64(fr.realized_rate),
            estimated_rate: opt_f64(fr.estimated_rate),
            funding_interval_hours: opt_f64(fr.funding_interval_hours),
            relative_funding_rate: opt_f64(fr.relative_funding_rate),
            accrued_funding: opt_f64(fr.accrued_funding),
            funding_step: fr.funding_step,
        }
    }
}

impl DataPoint for FundingRateIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.rate.to_le_bytes());
        out[16..24].copy_from_slice(&opt_i64_enc(self.next_funding_time_ms).to_le_bytes());
        out[24..32].copy_from_slice(&self.mark_price.to_le_bytes());
        out[32..40].copy_from_slice(&self.index_price.to_le_bytes());
        out[40..48].copy_from_slice(&self.prev_index_price.to_le_bytes());
        out[48..56].copy_from_slice(&self.premium.to_le_bytes());
        out[56..64].copy_from_slice(&self.interest_rate.to_le_bytes());
        out[64..72].copy_from_slice(&self.realized_rate.to_le_bytes());
        out[72..80].copy_from_slice(&self.estimated_rate.to_le_bytes());
        out[80..88].copy_from_slice(&self.funding_interval_hours.to_le_bytes());
        out[88..96].copy_from_slice(&self.relative_funding_rate.to_le_bytes());
        out[96..104].copy_from_slice(&self.accrued_funding.to_le_bytes());
        out[104..112].copy_from_slice(&opt_i64_enc(self.funding_step).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            rate: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            next_funding_time_ms: opt_i64_dec(u64::from_le_bytes(bytes[16..24].try_into().ok()?)),
            mark_price: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            index_price: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            prev_index_price: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            premium: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            interest_rate: f64::from_le_bytes(bytes[56..64].try_into().ok()?),
            realized_rate: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
            estimated_rate: f64::from_le_bytes(bytes[72..80].try_into().ok()?),
            funding_interval_hours: f64::from_le_bytes(bytes[80..88].try_into().ok()?),
            relative_funding_rate: f64::from_le_bytes(bytes[88..96].try_into().ok()?),
            accrued_funding: f64::from_le_bytes(bytes[96..104].try_into().ok()?),
            funding_step: opt_i64_dec(u64::from_le_bytes(bytes[104..112].try_into().ok()?)),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::FundingRate { funding, .. } = ev {
            Some(Self {
                ts_ms: funding.timestamp,
                rate: funding.rate,
                next_funding_time_ms: funding.next_funding_time,
                mark_price: opt_f64(funding.mark_price),
                index_price: opt_f64(funding.index_price),
                prev_index_price: opt_f64(funding.prev_index_price),
                premium: opt_f64(funding.premium),
                interest_rate: opt_f64(funding.interest_rate),
                realized_rate: opt_f64(funding.realized_rate),
                estimated_rate: opt_f64(funding.estimated_rate),
                funding_interval_hours: opt_f64(funding.funding_interval_hours),
                relative_funding_rate: opt_f64(funding.relative_funding_rate),
                accrued_funding: opt_f64(funding.accrued_funding),
                funding_step: funding.funding_step,
            })
        } else {
            None
        }
    }
}

// ─── FundingRateFullPoint ─────────────────────────────────────────────────────

/// Full FundingRate record — every numeric wire field.
///
/// String fields (symbol, sett_state, method, formula_type, fee_asset) are
/// stored in the blob as consecutive u16-len-prefixed UTF-8 strings.
///
/// Layout (numeric fixed part, all LE):
///   u64 ts_ms                                          (8)
///   f64 rate                                           (8)
///   u64 next_funding_time_ms (sentinel)               (8)
///   f64 mark_price, index_price, prev_index_price     (3 × 8 = 24)
///   f64 realized_rate, estimated_rate                 (2 × 8 = 16)
///   f64 premium, interest_rate                        (2 × 8 = 16)
///   f64 interest_1h, interest_8h                      (2 × 8 = 16)
///   f64 relative_funding_rate, avg_premium_index      (2 × 8 = 16)
///   f64 impact_value                                   (8)
///   f64 funding_interval_hours                        (8)
///   f64 max_funding_rate, min_funding_rate            (2 × 8 = 16)
///   f64 sett_funding_rate                             (8)
///   f64 next_funding_rate                             (8)
///   u64 prev_funding_time (sentinel)                  (8)
///   f64 accrued_funding                               (8)
///   u64 funding_step (sentinel)                       (8)
///   u64 blob_offset (8), u32 blob_len (4)             (12)
///
/// Numeric total: 8+8+8+24+16+16+16+16+8+8+16+8+8+8+8+8+12 = 200 B
const FULL_BLOB_OFFSET: usize = 188;
const FULL_SIZE: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRateFullPoint {
    pub ts_ms: i64,
    pub rate: f64,
    pub next_funding_time_ms: Option<i64>,
    pub mark_price: f64,
    pub index_price: f64,
    pub prev_index_price: f64,
    pub realized_rate: f64,
    pub estimated_rate: f64,
    pub premium: f64,
    pub interest_rate: f64,
    pub interest_1h: f64,
    pub interest_8h: f64,
    pub relative_funding_rate: f64,
    pub avg_premium_index: f64,
    pub impact_value: f64,
    pub funding_interval_hours: f64,
    pub max_funding_rate: f64,
    pub min_funding_rate: f64,
    pub sett_funding_rate: f64,
    pub next_funding_rate: f64,
    pub prev_funding_time: Option<i64>,
    pub accrued_funding: f64,
    pub funding_step: Option<i64>,
    // String fields in blob (5 strings, u16-len-prefixed each):
    // symbol, sett_state, method, formula_type, fee_asset
    pub symbol: Option<String>,
    pub sett_state: Option<String>,
    pub method: Option<String>,
    pub formula_type: Option<String>,
    pub fee_asset: Option<String>,
}

fn encode_fr_full_numeric(p: &FundingRateFullPoint, out: &mut [u8]) {
    let mut o = 0usize;
    macro_rules! w8f { ($v:expr) => { out[o..o+8].copy_from_slice(&($v as f64).to_le_bytes()); o += 8; }; }
    macro_rules! w8u { ($v:expr) => { out[o..o+8].copy_from_slice(&($v as u64).to_le_bytes()); o += 8; }; }
    w8u!(p.ts_ms as u64);
    w8f!(p.rate);
    w8u!(opt_i64_enc(p.next_funding_time_ms));
    w8f!(p.mark_price);
    w8f!(p.index_price);
    w8f!(p.prev_index_price);
    w8f!(p.realized_rate);
    w8f!(p.estimated_rate);
    w8f!(p.premium);
    w8f!(p.interest_rate);
    w8f!(p.interest_1h);
    w8f!(p.interest_8h);
    w8f!(p.relative_funding_rate);
    w8f!(p.avg_premium_index);
    w8f!(p.impact_value);
    w8f!(p.funding_interval_hours);
    w8f!(p.max_funding_rate);
    w8f!(p.min_funding_rate);
    w8f!(p.sett_funding_rate);
    w8f!(p.next_funding_rate);
    w8u!(opt_i64_enc(p.prev_funding_time));
    w8f!(p.accrued_funding);
    w8u!(opt_i64_enc(p.funding_step));
    debug_assert_eq!(o, FULL_BLOB_OFFSET);
}

fn decode_fr_full_numeric(bytes: &[u8]) -> Option<FundingRateFullPoint> {
    if bytes.len() < FULL_BLOB_OFFSET {
        return None;
    }
    let mut o = 0usize;
    macro_rules! rf64 {
        () => {{ let v = f64::from_le_bytes(bytes[o..o+8].try_into().ok()?); o += 8; v }};
    }
    macro_rules! ru64 {
        () => {{ let v = u64::from_le_bytes(bytes[o..o+8].try_into().ok()?); o += 8; v }};
    }
    let result = Some(FundingRateFullPoint {
        ts_ms: ru64!() as i64,
        rate: rf64!(),
        next_funding_time_ms: opt_i64_dec(ru64!()),
        mark_price: rf64!(),
        index_price: rf64!(),
        prev_index_price: rf64!(),
        realized_rate: rf64!(),
        estimated_rate: rf64!(),
        premium: rf64!(),
        interest_rate: rf64!(),
        interest_1h: rf64!(),
        interest_8h: rf64!(),
        relative_funding_rate: rf64!(),
        avg_premium_index: rf64!(),
        impact_value: rf64!(),
        funding_interval_hours: rf64!(),
        max_funding_rate: rf64!(),
        min_funding_rate: rf64!(),
        sett_funding_rate: rf64!(),
        next_funding_rate: rf64!(),
        prev_funding_time: opt_i64_dec(ru64!()),
        accrued_funding: rf64!(),
        funding_step: opt_i64_dec(ru64!()),
        symbol: None,
        sett_state: None,
        method: None,
        formula_type: None,
        fee_asset: None,
    });
    debug_assert_eq!(o, FULL_BLOB_OFFSET, "fr full decode offset mismatch");
    result
}

fn encode_str_blob(s: Option<&str>, out: &mut Vec<u8>) {
    let bytes = s.unwrap_or("").as_bytes();
    let len = bytes.len().min(u16::MAX as usize);
    out.extend_from_slice(&(len as u16).to_le_bytes());
    out.extend_from_slice(&bytes[..len]);
}

fn decode_str_blob(blob: &[u8], offset: &mut usize) -> Option<Option<String>> {
    if *offset + 2 > blob.len() {
        return None;
    }
    let slen = u16::from_le_bytes(blob[*offset..*offset + 2].try_into().ok()?) as usize;
    *offset += 2;
    if *offset + slen > blob.len() {
        return None;
    }
    let s = std::str::from_utf8(&blob[*offset..*offset + slen]).ok()?;
    *offset += slen;
    Some(if s.is_empty() { None } else { Some(s.to_owned()) })
}

impl FundingRateFullPoint {
    /// Construct from a REST `FundingRate` record (e.g. from `get_funding_rate_history`).
    pub fn from_funding_rate(fr: &FundingRate) -> Self {
        Self {
            ts_ms: fr.timestamp,
            rate: fr.rate,
            next_funding_time_ms: fr.next_funding_time,
            mark_price: opt_f64(fr.mark_price),
            index_price: opt_f64(fr.index_price),
            prev_index_price: opt_f64(fr.prev_index_price),
            realized_rate: opt_f64(fr.realized_rate),
            estimated_rate: opt_f64(fr.estimated_rate),
            premium: opt_f64(fr.premium),
            interest_rate: opt_f64(fr.interest_rate),
            interest_1h: opt_f64(fr.interest_1h),
            interest_8h: opt_f64(fr.interest_8h),
            relative_funding_rate: opt_f64(fr.relative_funding_rate),
            avg_premium_index: opt_f64(fr.avg_premium_index),
            impact_value: opt_f64(fr.impact_value),
            funding_interval_hours: opt_f64(fr.funding_interval_hours),
            max_funding_rate: opt_f64(fr.max_funding_rate),
            min_funding_rate: opt_f64(fr.min_funding_rate),
            sett_funding_rate: opt_f64(fr.sett_funding_rate),
            next_funding_rate: opt_f64(fr.next_funding_rate),
            prev_funding_time: fr.prev_funding_time,
            accrued_funding: opt_f64(fr.accrued_funding),
            funding_step: fr.funding_step,
            symbol: fr.symbol.clone(),
            sett_state: fr.sett_state.clone(),
            method: fr.method.clone(),
            formula_type: fr.formula_type.clone(),
            fee_asset: fr.fee_asset.clone(),
        }
    }
}

impl DataPoint for FundingRateFullPoint {
    const RECORD_SIZE: usize = FULL_SIZE;

    fn encode(&self, out: &mut [u8]) {
        encode_fr_full_numeric(self, out);
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FULL_SIZE {
            return None;
        }
        decode_fr_full_numeric(bytes)
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::FundingRate { funding, .. } = ev {
            Some(Self {
                ts_ms: funding.timestamp,
                rate: funding.rate,
                next_funding_time_ms: funding.next_funding_time,
                mark_price: opt_f64(funding.mark_price),
                index_price: opt_f64(funding.index_price),
                prev_index_price: opt_f64(funding.prev_index_price),
                realized_rate: opt_f64(funding.realized_rate),
                estimated_rate: opt_f64(funding.estimated_rate),
                premium: opt_f64(funding.premium),
                interest_rate: opt_f64(funding.interest_rate),
                interest_1h: opt_f64(funding.interest_1h),
                interest_8h: opt_f64(funding.interest_8h),
                relative_funding_rate: opt_f64(funding.relative_funding_rate),
                avg_premium_index: opt_f64(funding.avg_premium_index),
                impact_value: opt_f64(funding.impact_value),
                funding_interval_hours: opt_f64(funding.funding_interval_hours),
                max_funding_rate: opt_f64(funding.max_funding_rate),
                min_funding_rate: opt_f64(funding.min_funding_rate),
                sett_funding_rate: opt_f64(funding.sett_funding_rate),
                next_funding_rate: opt_f64(funding.next_funding_rate),
                prev_funding_time: funding.prev_funding_time,
                accrued_funding: opt_f64(funding.accrued_funding),
                funding_step: funding.funding_step,
                symbol: funding.symbol.clone(),
                sett_state: funding.sett_state.clone(),
                method: funding.method.clone(),
                formula_type: funding.formula_type.clone(),
                fee_asset: funding.fee_asset.clone(),
            })
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let mut blob = Vec::new();
        encode_str_blob(self.symbol.as_deref(), &mut blob);
        encode_str_blob(self.sett_state.as_deref(), &mut blob);
        encode_str_blob(self.method.as_deref(), &mut blob);
        encode_str_blob(self.formula_type.as_deref(), &mut blob);
        encode_str_blob(self.fee_asset.as_deref(), &mut blob);
        Some(blob)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = decode_fr_full_numeric(header)?;
        let mut off = 0usize;
        p.symbol = decode_str_blob(blob, &mut off).unwrap_or(None);
        p.sett_state = decode_str_blob(blob, &mut off).unwrap_or(None);
        p.method = decode_str_blob(blob, &mut off).unwrap_or(None);
        p.formula_type = decode_str_blob(blob, &mut off).unwrap_or(None);
        p.fee_asset = decode_str_blob(blob, &mut off).unwrap_or(None);
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(FULL_BLOB_OFFSET) }
}
