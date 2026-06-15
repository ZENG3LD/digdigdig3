//! Extended Liquidation DataPoint types for Indicators and Full depth.
//!
//! `LiquidationPoint` (Compact, 33 B) is unchanged — see `liquidation.rs`.

use digdigdig3::core::types::{Liquidation, StreamEvent, TradeSide};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

#[inline]
fn opt_f64(v: Option<f64>) -> f64 {
    v.unwrap_or(f64::NAN)
}

fn side_byte(s: TradeSide) -> u8 {
    match s {
        TradeSide::Buy => 0,
        TradeSide::Sell => 1,
    }
}

// ─── LiquidationIndicatorsPoint ──────────────────────────────────────────────

/// 67 B Liquidation record for Indicators depth.
///
/// Layout (all LE):
///   u64 ts_ms                                          (8)
///   f64 price, quantity, value                        (3 × 8 = 24)
///   u8 side (0=buy/long-liq, 1=sell/short-liq)       (1)
///   f64 avg_price, executed_qty                       (2 × 8 = 16)
///   u8 position_side (0=long, 1=short, 255=absent)   (1)
///   f64 order_price, left                             (2 × 8 = 16)
///   u8 time_in_force (255=absent)                     (1)
///
/// Total: 8+24+1+16+1+16+1 = 67 B
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationIndicatorsPoint {
    pub ts_ms: i64,
    pub price: f64,
    pub quantity: f64,
    pub value: f64,
    /// 0 = buy side liquidated (long), 1 = sell side (short).
    pub side: u8,
    pub avg_price: f64,
    pub executed_qty: f64,
    /// 0 = long, 1 = short, 255 = absent.
    pub position_side: u8,
    pub order_price: f64,
    pub left: f64,
    /// Time-in-force as raw byte. 255 = absent.
    pub time_in_force: u8,
}

const INDICATORS_SIZE: usize = 67;

impl LiquidationIndicatorsPoint {
    /// Construct from a REST `Liquidation` record (e.g. from `get_liquidation_history`).
    pub fn from_liquidation(liq: &Liquidation) -> Self {
        let pos_side = match liq.position_side.as_deref() {
            Some("long") => 0u8,
            Some("short") => 1u8,
            _ => u8::MAX,
        };
        Self {
            ts_ms: liq.timestamp,
            price: liq.price,
            quantity: liq.quantity,
            value: opt_f64(liq.value),
            side: side_byte(liq.side),
            avg_price: opt_f64(liq.avg_price),
            executed_qty: opt_f64(liq.executed_qty),
            position_side: pos_side,
            order_price: opt_f64(liq.order_price),
            left: opt_f64(liq.left),
            time_in_force: u8::MAX,
        }
    }
}

impl DataPoint for LiquidationIndicatorsPoint {
    const RECORD_SIZE: usize = INDICATORS_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24..32].copy_from_slice(&self.value.to_le_bytes());
        out[32] = self.side;
        out[33..41].copy_from_slice(&self.avg_price.to_le_bytes());
        out[41..49].copy_from_slice(&self.executed_qty.to_le_bytes());
        out[49] = self.position_side;
        out[50..58].copy_from_slice(&self.order_price.to_le_bytes());
        out[58..66].copy_from_slice(&self.left.to_le_bytes());
        out[66] = self.time_in_force;
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != INDICATORS_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            quantity: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            value: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            side: bytes[32],
            avg_price: f64::from_le_bytes(bytes[33..41].try_into().ok()?),
            executed_qty: f64::from_le_bytes(bytes[41..49].try_into().ok()?),
            position_side: bytes[49],
            order_price: f64::from_le_bytes(bytes[50..58].try_into().ok()?),
            left: f64::from_le_bytes(bytes[58..66].try_into().ok()?),
            time_in_force: bytes[66],
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Liquidation { liquidation, .. } = ev {
            let pos_side = match liquidation.position_side.as_deref() {
                Some("long") => 0u8,
                Some("short") => 1u8,
                _ => u8::MAX,
            };
            Some(Self {
                ts_ms: liquidation.timestamp,
                price: liquidation.price,
                quantity: liquidation.quantity,
                value: opt_f64(liquidation.value),
                side: side_byte(liquidation.side),
                avg_price: opt_f64(liquidation.avg_price),
                executed_qty: opt_f64(liquidation.executed_qty),
                position_side: pos_side,
                order_price: opt_f64(liquidation.order_price),
                left: opt_f64(liquidation.left),
                time_in_force: u8::MAX, // not a numeric field in Liquidation
            })
        } else {
            None
        }
    }
}

// ─── LiquidationFullPoint ─────────────────────────────────────────────────────

/// Full Liquidation record — every numeric wire field + string fields in blob.
///
/// Numeric layout (all LE):
///   u64 ts_ms                                          (8)
///   f64 price, quantity, value                        (3 × 8 = 24)
///   u8 side                                            (1)
///   f64 avg_price, executed_qty, order_qty, order_price (4 × 8 = 32)
///   f64 fill_price, left, signed_size, base_price     (4 × 8 = 32)
///   u8 position_side_byte (0=long,1=short,255=absent) (1)
///   u64 blob_offset (8), u32 blob_len (4)             (12)
///
/// Numeric fixed: 8+24+1+32+32+1+12 = 110 B
///
/// Blob: 3 strings (order_id, order_type, status), u16-len-prefix each.
const FULL_BLOB_OFFSET: usize = 98;
const FULL_SIZE: usize = 110;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationFullPoint {
    pub ts_ms: i64,
    pub price: f64,
    pub quantity: f64,
    pub value: f64,
    pub side: u8,
    pub avg_price: f64,
    pub executed_qty: f64,
    pub order_qty: f64,
    pub order_price: f64,
    pub fill_price: f64,
    pub left: f64,
    pub signed_size: f64,
    pub base_price: f64,
    /// 0=long, 1=short, 255=absent.
    pub position_side_byte: u8,
    // String fields in blob:
    pub order_id: Option<String>,
    pub order_type: Option<String>,
    pub status: Option<String>,
}

fn encode_str(s: Option<&str>, out: &mut Vec<u8>) {
    let b = s.unwrap_or("").as_bytes();
    let len = b.len().min(u16::MAX as usize);
    out.extend_from_slice(&(len as u16).to_le_bytes());
    out.extend_from_slice(&b[..len]);
}

fn decode_str(blob: &[u8], off: &mut usize) -> Option<Option<String>> {
    if *off + 2 > blob.len() { return None; }
    let slen = u16::from_le_bytes(blob[*off..*off+2].try_into().ok()?) as usize;
    *off += 2;
    if *off + slen > blob.len() { return None; }
    let s = std::str::from_utf8(&blob[*off..*off + slen]).ok()?;
    *off += slen;
    Some(if s.is_empty() { None } else { Some(s.to_owned()) })
}

impl LiquidationFullPoint {
    /// Construct from a REST `Liquidation` record (e.g. from `get_liquidation_history`).
    pub fn from_liquidation(liq: &Liquidation) -> Self {
        let pos_side = match liq.position_side.as_deref() {
            Some("long") => 0u8,
            Some("short") => 1u8,
            _ => u8::MAX,
        };
        Self {
            ts_ms: liq.timestamp,
            price: liq.price,
            quantity: liq.quantity,
            value: opt_f64(liq.value),
            side: side_byte(liq.side),
            avg_price: opt_f64(liq.avg_price),
            executed_qty: opt_f64(liq.executed_qty),
            order_qty: opt_f64(liq.order_qty),
            order_price: opt_f64(liq.order_price),
            fill_price: opt_f64(liq.fill_price),
            left: opt_f64(liq.left),
            signed_size: opt_f64(liq.signed_size),
            base_price: opt_f64(liq.base_price),
            position_side_byte: pos_side,
            order_id: liq.order_id.clone(),
            order_type: liq.order_type.clone(),
            status: liq.status.clone(),
        }
    }
}

impl DataPoint for LiquidationFullPoint {
    const RECORD_SIZE: usize = FULL_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24..32].copy_from_slice(&self.value.to_le_bytes());
        out[32] = self.side;
        out[33..41].copy_from_slice(&self.avg_price.to_le_bytes());
        out[41..49].copy_from_slice(&self.executed_qty.to_le_bytes());
        out[49..57].copy_from_slice(&self.order_qty.to_le_bytes());
        out[57..65].copy_from_slice(&self.order_price.to_le_bytes());
        out[65..73].copy_from_slice(&self.fill_price.to_le_bytes());
        out[73..81].copy_from_slice(&self.left.to_le_bytes());
        out[81..89].copy_from_slice(&self.signed_size.to_le_bytes());
        out[89..97].copy_from_slice(&self.base_price.to_le_bytes());
        out[97] = self.position_side_byte;
        // blob pointer at [98..110] patched by DiskStore
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != FULL_SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            quantity: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            value: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            side: bytes[32],
            avg_price: f64::from_le_bytes(bytes[33..41].try_into().ok()?),
            executed_qty: f64::from_le_bytes(bytes[41..49].try_into().ok()?),
            order_qty: f64::from_le_bytes(bytes[49..57].try_into().ok()?),
            order_price: f64::from_le_bytes(bytes[57..65].try_into().ok()?),
            fill_price: f64::from_le_bytes(bytes[65..73].try_into().ok()?),
            left: f64::from_le_bytes(bytes[73..81].try_into().ok()?),
            signed_size: f64::from_le_bytes(bytes[81..89].try_into().ok()?),
            base_price: f64::from_le_bytes(bytes[89..97].try_into().ok()?),
            position_side_byte: bytes[97],
            order_id: None,
            order_type: None,
            status: None,
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Liquidation { liquidation, .. } = ev {
            let pos_side = match liquidation.position_side.as_deref() {
                Some("long") => 0u8,
                Some("short") => 1u8,
                _ => u8::MAX,
            };
            Some(Self {
                ts_ms: liquidation.timestamp,
                price: liquidation.price,
                quantity: liquidation.quantity,
                value: opt_f64(liquidation.value),
                side: side_byte(liquidation.side),
                avg_price: opt_f64(liquidation.avg_price),
                executed_qty: opt_f64(liquidation.executed_qty),
                order_qty: opt_f64(liquidation.order_qty),
                order_price: opt_f64(liquidation.order_price),
                fill_price: opt_f64(liquidation.fill_price),
                left: opt_f64(liquidation.left),
                signed_size: opt_f64(liquidation.signed_size),
                base_price: opt_f64(liquidation.base_price),
                position_side_byte: pos_side,
                order_id: liquidation.order_id.clone(),
                order_type: liquidation.order_type.clone(),
                status: liquidation.status.clone(),
            })
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let mut blob = Vec::new();
        encode_str(self.order_id.as_deref(), &mut blob);
        encode_str(self.order_type.as_deref(), &mut blob);
        encode_str(self.status.as_deref(), &mut blob);
        Some(blob)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        let mut off = 0usize;
        p.order_id = decode_str(blob, &mut off).unwrap_or(None);
        p.order_type = decode_str(blob, &mut off).unwrap_or(None);
        p.status = decode_str(blob, &mut off).unwrap_or(None);
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(FULL_BLOB_OFFSET) }
}
