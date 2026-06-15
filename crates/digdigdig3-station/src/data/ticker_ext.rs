//! Extended Ticker DataPoint types for Indicators and Full depth.
//!
//! `TickerPoint` (Compact, 72 B) is unchanged — see `ticker.rs`.
//!
//! Layout convention: all fields little-endian. `f64::NAN` for absent
//! `Option<f64>`, `i64::MIN` as u64 sentinel for absent `Option<i64>`.

use digdigdig3::core::types::{StreamEvent, Ticker};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

// ─── helpers ─────────────────────────────────────────────────────────────────

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

// ─── TickerIndicatorsPoint ───────────────────────────────────────────────────

/// 160 B ticker record for Indicators depth.
///
/// Fields (all LE):
///   u64 ts_ms (8)
///   f64 last, bid, ask, bid_qty, ask_qty              (5 × 8 = 40)
///   f64 high_24h, low_24h                             (2 × 8 = 16)
///   f64 open_price, prev_close_price                  (2 × 8 = 16)
///   f64 vol_24h, quote_vol_24h                        (2 × 8 = 16)
///   f64 change_pct_24h                                (8)
///   f64 weighted_avg_price                            (8)
///   f64 last_qty                                      (8)
///   f64 mark_price, index_price, open_interest        (3 × 8 = 24)
///   f64 funding_rate                                  (8)
///   u64 count (sentinel i64::MIN)                     (8)
///
/// Total: 8 + 40 + 16 + 16 + 16 + 8 + 8 + 8 + 24 + 8 + 8 = 160 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerIndicatorsPoint {
    pub ts_ms: i64,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub bid_qty: f64,
    pub ask_qty: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub open_price: f64,
    pub prev_close_price: f64,
    pub vol_24h: f64,
    pub quote_vol_24h: f64,
    pub change_pct_24h: f64,
    pub weighted_avg_price: f64,
    pub last_qty: f64,
    pub mark_price: f64,
    pub index_price: f64,
    pub open_interest: f64,
    pub funding_rate: f64,
    /// `None` stored as `i64::MIN`.
    pub count: Option<i64>,
}

const INDICATORS_SIZE: usize = 160;

impl TickerIndicatorsPoint {
    pub fn from_ticker(t: &Ticker) -> Self {
        Self {
            ts_ms: t.timestamp,
            last: t.last_price,
            bid: opt_f64(t.bid_price),
            ask: opt_f64(t.ask_price),
            bid_qty: opt_f64(t.bid_qty),
            ask_qty: opt_f64(t.ask_qty),
            high_24h: opt_f64(t.high_24h),
            low_24h: opt_f64(t.low_24h),
            open_price: opt_f64(t.open_price),
            prev_close_price: opt_f64(t.prev_close_price),
            vol_24h: opt_f64(t.volume_24h),
            quote_vol_24h: opt_f64(t.quote_volume_24h),
            change_pct_24h: opt_f64(t.price_change_percent_24h),
            weighted_avg_price: opt_f64(t.weighted_avg_price),
            last_qty: opt_f64(t.last_qty),
            mark_price: opt_f64(t.mark_price),
            index_price: opt_f64(t.index_price),
            open_interest: opt_f64(t.open_interest),
            funding_rate: opt_f64(t.funding_rate),
            count: t.count,
        }
    }
}

impl DataPoint for TickerIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.last.to_le_bytes());
        out[16..24].copy_from_slice(&self.bid.to_le_bytes());
        out[24..32].copy_from_slice(&self.ask.to_le_bytes());
        out[32..40].copy_from_slice(&self.bid_qty.to_le_bytes());
        out[40..48].copy_from_slice(&self.ask_qty.to_le_bytes());
        out[48..56].copy_from_slice(&self.high_24h.to_le_bytes());
        out[56..64].copy_from_slice(&self.low_24h.to_le_bytes());
        out[64..72].copy_from_slice(&self.open_price.to_le_bytes());
        out[72..80].copy_from_slice(&self.prev_close_price.to_le_bytes());
        out[80..88].copy_from_slice(&self.vol_24h.to_le_bytes());
        out[88..96].copy_from_slice(&self.quote_vol_24h.to_le_bytes());
        out[96..104].copy_from_slice(&self.change_pct_24h.to_le_bytes());
        out[104..112].copy_from_slice(&self.weighted_avg_price.to_le_bytes());
        out[112..120].copy_from_slice(&self.last_qty.to_le_bytes());
        out[120..128].copy_from_slice(&self.mark_price.to_le_bytes());
        out[128..136].copy_from_slice(&self.index_price.to_le_bytes());
        out[136..144].copy_from_slice(&self.open_interest.to_le_bytes());
        out[144..152].copy_from_slice(&self.funding_rate.to_le_bytes());
        out[152..160].copy_from_slice(&opt_i64_enc(self.count).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            last: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            bid: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            ask: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            bid_qty: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            ask_qty: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            high_24h: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            low_24h: f64::from_le_bytes(bytes[56..64].try_into().ok()?),
            open_price: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
            prev_close_price: f64::from_le_bytes(bytes[72..80].try_into().ok()?),
            vol_24h: f64::from_le_bytes(bytes[80..88].try_into().ok()?),
            quote_vol_24h: f64::from_le_bytes(bytes[88..96].try_into().ok()?),
            change_pct_24h: f64::from_le_bytes(bytes[96..104].try_into().ok()?),
            weighted_avg_price: f64::from_le_bytes(bytes[104..112].try_into().ok()?),
            last_qty: f64::from_le_bytes(bytes[112..120].try_into().ok()?),
            mark_price: f64::from_le_bytes(bytes[120..128].try_into().ok()?),
            index_price: f64::from_le_bytes(bytes[128..136].try_into().ok()?),
            open_interest: f64::from_le_bytes(bytes[136..144].try_into().ok()?),
            funding_rate: f64::from_le_bytes(bytes[144..152].try_into().ok()?),
            count: opt_i64_dec(u64::from_le_bytes(bytes[152..160].try_into().ok()?)),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Ticker { ticker, .. } = ev {
            Some(Self::from_ticker(ticker))
        } else {
            None
        }
    }
}

// ─── TickerFullPoint ─────────────────────────────────────────────────────────

/// Full Ticker record — every numeric wire field.
///
/// Layout (all LE):
///   u64 ts_ms                                          (8)
///   f64 last                                           (8)
///   f64 bid, ask, bid_qty, ask_qty                    (4 × 8 = 32)
///   f64 high_24h, low_24h, vol_24h, quote_vol_24h     (4 × 8 = 32)
///   f64 price_change_24h, price_change_pct_24h        (2 × 8 = 16)
///   f64 open_price, prev_close_price                   (2 × 8 = 16)
///   f64 prev_price_24h, prev_price_1h                 (2 × 8 = 16)
///   f64 weighted_avg_price, open_utc, turnover_24h    (3 × 8 = 24)
///   u64 first_id, last_id, count (sentinels)          (3 × 8 = 24)
///   f64 mark_price, index_price                        (2 × 8 = 16)
///   f64 open_interest, open_interest_value            (2 × 8 = 16)
///   f64 single_open_interest                          (8)
///   f64 funding_rate                                   (8)
///   u64 next_funding_time (sentinel)                  (8)
///   f64 funding_interval_hour, funding_cap            (2 × 8 = 16)
///   f64 basis, basis_rate                             (2 × 8 = 16)
///   f64 predicted_delivery_price                       (8)
///   u64 delivery_time (sentinel)                      (8)
///   f64 settlement_price, funding_8h                  (2 × 8 = 16)
///   f64 min_price, max_price, volume_notional         (3 × 8 = 24)
///   f64 last_qty, interest_value                      (2 × 8 = 16)
///   u64 last_trade_time, open_time, update_id (senti) (3 × 8 = 24)
///   u64 blob_offset (8), u32 blob_len (4)             (12)
///
/// Fixed total: 8+8+32+32+16+16+16+24+24+16+16+8+8+8+16+16+8+8+16+24+16+24+12
///            = 376 B
///
/// Blob (variable): u16-len-prefixed UTF-8 `state` string (empty if None).
/// `update_id` is stored as u64 with sentinel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerFullPoint {
    pub ts_ms: i64,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub bid_qty: f64,
    pub ask_qty: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub vol_24h: f64,
    pub quote_vol_24h: f64,
    pub price_change_24h: f64,
    pub price_change_pct_24h: f64,
    pub open_price: f64,
    pub prev_close_price: f64,
    pub prev_price_24h: f64,
    pub prev_price_1h: f64,
    pub weighted_avg_price: f64,
    pub open_utc: f64,
    pub turnover_24h: f64,
    pub first_id: Option<i64>,
    pub last_id: Option<i64>,
    pub count: Option<i64>,
    pub mark_price: f64,
    pub index_price: f64,
    pub open_interest: f64,
    pub open_interest_value: f64,
    pub single_open_interest: f64,
    pub funding_rate: f64,
    pub next_funding_time: Option<i64>,
    pub funding_interval_hour: f64,
    pub funding_cap: f64,
    pub basis: f64,
    pub basis_rate: f64,
    pub predicted_delivery_price: f64,
    pub delivery_time: Option<i64>,
    pub settlement_price: f64,
    pub funding_8h: f64,
    pub min_price: f64,
    pub max_price: f64,
    pub volume_notional: f64,
    pub last_qty: f64,
    pub interest_value: f64,
    pub last_trade_time: Option<i64>,
    pub open_time: Option<i64>,
    pub update_id: Option<i64>,
    /// Instrument state string ("open"/"closed" etc.) — stored in blob.
    pub state: Option<String>,
}

// Fixed part: 376 - 12 (blob pointer tail) = 364 numeric bytes.
// blob pointer at offset 364: u64 blob_off + u32 blob_len = 12 B.
const FULL_BLOB_OFFSET: usize = 364;
const FULL_SIZE: usize = 376;

fn encode_ticker_full_numeric(p: &TickerFullPoint, out: &mut [u8]) {
    let mut o = 0usize;
    macro_rules! w8 {
        ($v:expr) => { out[o..o+8].copy_from_slice(&$v.to_le_bytes()); o += 8; };
    }
    w8!((p.ts_ms as u64));
    w8!(p.last);
    w8!(p.bid);
    w8!(p.ask);
    w8!(p.bid_qty);
    w8!(p.ask_qty);
    w8!(p.high_24h);
    w8!(p.low_24h);
    w8!(p.vol_24h);
    w8!(p.quote_vol_24h);
    w8!(p.price_change_24h);
    w8!(p.price_change_pct_24h);
    w8!(p.open_price);
    w8!(p.prev_close_price);
    w8!(p.prev_price_24h);
    w8!(p.prev_price_1h);
    w8!(p.weighted_avg_price);
    w8!(p.open_utc);
    w8!(p.turnover_24h);
    w8!(opt_i64_enc(p.first_id));
    w8!(opt_i64_enc(p.last_id));
    w8!(opt_i64_enc(p.count));
    w8!(p.mark_price);
    w8!(p.index_price);
    w8!(p.open_interest);
    w8!(p.open_interest_value);
    w8!(p.single_open_interest);
    w8!(p.funding_rate);
    w8!(opt_i64_enc(p.next_funding_time));
    w8!(p.funding_interval_hour);
    w8!(p.funding_cap);
    w8!(p.basis);
    w8!(p.basis_rate);
    w8!(p.predicted_delivery_price);
    w8!(opt_i64_enc(p.delivery_time));
    w8!(p.settlement_price);
    w8!(p.funding_8h);
    w8!(p.min_price);
    w8!(p.max_price);
    w8!(p.volume_notional);
    w8!(p.last_qty);
    w8!(p.interest_value);
    w8!(opt_i64_enc(p.last_trade_time));
    w8!(opt_i64_enc(p.open_time));
    w8!(opt_i64_enc(p.update_id));
    debug_assert_eq!(o, FULL_BLOB_OFFSET);
}

fn decode_ticker_full_numeric(bytes: &[u8]) -> Option<TickerFullPoint> {
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
    let result = Some(TickerFullPoint {
        ts_ms: ru64!() as i64,
        last: rf64!(),
        bid: rf64!(),
        ask: rf64!(),
        bid_qty: rf64!(),
        ask_qty: rf64!(),
        high_24h: rf64!(),
        low_24h: rf64!(),
        vol_24h: rf64!(),
        quote_vol_24h: rf64!(),
        price_change_24h: rf64!(),
        price_change_pct_24h: rf64!(),
        open_price: rf64!(),
        prev_close_price: rf64!(),
        prev_price_24h: rf64!(),
        prev_price_1h: rf64!(),
        weighted_avg_price: rf64!(),
        open_utc: rf64!(),
        turnover_24h: rf64!(),
        first_id: opt_i64_dec(ru64!()),
        last_id: opt_i64_dec(ru64!()),
        count: opt_i64_dec(ru64!()),
        mark_price: rf64!(),
        index_price: rf64!(),
        open_interest: rf64!(),
        open_interest_value: rf64!(),
        single_open_interest: rf64!(),
        funding_rate: rf64!(),
        next_funding_time: opt_i64_dec(ru64!()),
        funding_interval_hour: rf64!(),
        funding_cap: rf64!(),
        basis: rf64!(),
        basis_rate: rf64!(),
        predicted_delivery_price: rf64!(),
        delivery_time: opt_i64_dec(ru64!()),
        settlement_price: rf64!(),
        funding_8h: rf64!(),
        min_price: rf64!(),
        max_price: rf64!(),
        volume_notional: rf64!(),
        last_qty: rf64!(),
        interest_value: rf64!(),
        last_trade_time: opt_i64_dec(ru64!()),
        open_time: opt_i64_dec(ru64!()),
        update_id: opt_i64_dec(ru64!()),
        state: None, // filled by decode_blob
    });
    debug_assert_eq!(o, FULL_BLOB_OFFSET, "ticker full decode offset mismatch");
    result
}

impl TickerFullPoint {
    pub fn from_ticker(t: &Ticker) -> Self {
        Self {
            ts_ms: t.timestamp,
            last: t.last_price,
            bid: opt_f64(t.bid_price),
            ask: opt_f64(t.ask_price),
            bid_qty: opt_f64(t.bid_qty),
            ask_qty: opt_f64(t.ask_qty),
            high_24h: opt_f64(t.high_24h),
            low_24h: opt_f64(t.low_24h),
            vol_24h: opt_f64(t.volume_24h),
            quote_vol_24h: opt_f64(t.quote_volume_24h),
            price_change_24h: opt_f64(t.price_change_24h),
            price_change_pct_24h: opt_f64(t.price_change_percent_24h),
            open_price: opt_f64(t.open_price),
            prev_close_price: opt_f64(t.prev_close_price),
            prev_price_24h: opt_f64(t.prev_price_24h),
            prev_price_1h: opt_f64(t.prev_price_1h),
            weighted_avg_price: opt_f64(t.weighted_avg_price),
            open_utc: opt_f64(t.open_utc),
            turnover_24h: opt_f64(t.turnover_24h),
            first_id: t.first_id,
            last_id: t.last_id,
            count: t.count,
            mark_price: opt_f64(t.mark_price),
            index_price: opt_f64(t.index_price),
            open_interest: opt_f64(t.open_interest),
            open_interest_value: opt_f64(t.open_interest_value),
            single_open_interest: opt_f64(t.single_open_interest),
            funding_rate: opt_f64(t.funding_rate),
            next_funding_time: t.next_funding_time,
            funding_interval_hour: opt_f64(t.funding_interval_hour),
            funding_cap: opt_f64(t.funding_cap),
            basis: opt_f64(t.basis),
            basis_rate: opt_f64(t.basis_rate),
            predicted_delivery_price: opt_f64(t.predicted_delivery_price),
            delivery_time: t.delivery_time,
            settlement_price: opt_f64(t.settlement_price),
            funding_8h: opt_f64(t.funding_8h),
            min_price: opt_f64(t.min_price),
            max_price: opt_f64(t.max_price),
            volume_notional: opt_f64(t.volume_notional),
            last_qty: opt_f64(t.last_qty),
            interest_value: opt_f64(t.interest_value),
            last_trade_time: t.last_trade_time,
            open_time: t.open_time,
            update_id: t.update_id,
            state: t.state.clone(),
        }
    }
}

impl DataPoint for TickerFullPoint {
    const RECORD_SIZE: usize = FULL_SIZE;

    fn encode(&self, out: &mut [u8]) {
        encode_ticker_full_numeric(self, out);
        // blob pointer tail (offset + len) patched by DiskStore after encode.
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FULL_SIZE {
            return None;
        }
        decode_ticker_full_numeric(bytes)
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Ticker { ticker, .. } = ev {
            Some(Self::from_ticker(ticker))
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let s = self.state.as_deref().unwrap_or("");
        let bytes = s.as_bytes();
        let len = bytes.len().min(u16::MAX as usize);
        let mut blob = Vec::with_capacity(2 + len);
        blob.extend_from_slice(&(len as u16).to_le_bytes());
        blob.extend_from_slice(&bytes[..len]);
        Some(blob)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = decode_ticker_full_numeric(header)?;
        if blob.len() >= 2 {
            let slen = u16::from_le_bytes(blob[0..2].try_into().ok()?) as usize;
            if blob.len() >= 2 + slen {
                let s = std::str::from_utf8(&blob[2..2 + slen]).ok()?;
                if !s.is_empty() {
                    p.state = Some(s.to_owned());
                }
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(FULL_BLOB_OFFSET) }
}
