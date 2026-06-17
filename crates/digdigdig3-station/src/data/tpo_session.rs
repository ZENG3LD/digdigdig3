use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// TPO (Time-Price Opportunity) Market Profile snapshot for one session.
///
/// One emit per session (live snapshot on every closed 1-minute kline
/// during the session; persisted under `open_time = session_date_ms`
/// via Series upsert, so the on-disk record represents the FINAL
/// session profile after rollover).
///
/// Disk layout:
/// - Header (72 B): `open_time (8) | tick_size (8) | session_high (8) |
///   session_low (8) | poc_price (8) | vah_price (8) | val_price (8) |
///   _pad (4) | blob_off (8) | blob_len (4)`.
/// - Blob: per-row `[price:f64 | letters_len:u16 | letters_utf8]`,
///   prefixed by `row_count:u32`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TpoSessionPoint {
    pub open_time: i64,
    pub tick_size: f64,
    pub session_high: f64,
    pub session_low: f64,
    pub poc_price: f64,
    pub vah_price: f64,
    pub val_price: f64,
    /// Sorted ascending by price. Each row carries the letters (period
    /// IDs) that touched that price level during the session.
    pub rows: Vec<(f64, Vec<char>)>,
}

const HEADER_SIZE: usize = 72;
const TAIL_OFFSET: usize = 60;

impl DataPoint for TpoSessionPoint {
    const RECORD_SIZE: usize = HEADER_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.open_time as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.tick_size.to_le_bytes());
        out[16..24].copy_from_slice(&self.session_high.to_le_bytes());
        out[24..32].copy_from_slice(&self.session_low.to_le_bytes());
        out[32..40].copy_from_slice(&self.poc_price.to_le_bytes());
        out[40..48].copy_from_slice(&self.vah_price.to_le_bytes());
        out[48..56].copy_from_slice(&self.val_price.to_le_bytes());
        // bytes 56..60 reserved.
        // 60..72 blob tail (offset+len) is patched by DiskStore.
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != HEADER_SIZE { return None; }
        Some(Self {
            open_time: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            tick_size: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            session_high: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            session_low: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            poc_price: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            vah_price: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            val_price: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            rows: Vec::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.open_time }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> { None }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let mut out: Vec<u8> = Vec::with_capacity(4 + self.rows.len() * 16);
        out.extend_from_slice(&(self.rows.len() as u32).to_le_bytes());
        for (price, letters) in &self.rows {
            out.extend_from_slice(&price.to_le_bytes());
            // Letters are ASCII A-Z/a-z so 1 byte each.
            let letters_bytes: Vec<u8> = letters.iter().map(|c| *c as u8).collect();
            out.extend_from_slice(&(letters_bytes.len() as u16).to_le_bytes());
            out.extend_from_slice(&letters_bytes);
        }
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        if blob.len() < 4 { return Some(p); }
        let row_count = u32::from_le_bytes(blob[0..4].try_into().ok()?) as usize;
        let mut cursor = 4usize;
        for _ in 0..row_count {
            if cursor + 10 > blob.len() { break; }
            let price = f64::from_le_bytes(blob[cursor..cursor + 8].try_into().ok()?);
            cursor += 8;
            let letters_len = u16::from_le_bytes(blob[cursor..cursor + 2].try_into().ok()?) as usize;
            cursor += 2;
            if cursor + letters_len > blob.len() { break; }
            let letters: Vec<char> = blob[cursor..cursor + letters_len].iter().map(|b| *b as char).collect();
            cursor += letters_len;
            p.rows.push((price, letters));
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}
