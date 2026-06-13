//! `FootprintPoint` — time-bucketed OHLCV bar with per-price buy/sell volume.
//!
//! Each record is a fixed-size header (`open_time + OHLCV + blob pointer`,
//! 60 bytes) plus a variable-length companion `.blob` file holding the
//! per-price level data.
//!
//! ## Blob layout (LE)
//!
//! ```text
//! u32  level_count
//! level_count × (f64 price, f64 buy_vol, f64 sell_vol)   -- 24 bytes each
//! ```
//!
//! ## Price level granularity
//!
//! Levels are keyed by the **raw trade price** (no tick-size binning).
//! Binning is a render/display concern; the raw granularity gives consumers
//! complete freedom to re-bucket at any resolution.

use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Footprint bar: OHLCV + per-price buy/sell breakdown for a time bucket.
///
/// Derived from `Stream::Trade`; never arrives from the exchange wire directly.
/// `open_time` is the UTC-epoch bucket start in milliseconds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootprintPoint {
    /// UTC bucket start (ms).
    pub open_time: i64,
    pub open:  f64,
    pub high:  f64,
    pub low:   f64,
    pub close: f64,
    /// Total volume (buy + sell) across all price levels.
    pub volume: f64,
    /// Per-price levels: `(price, buy_vol, sell_vol)`.
    /// Ordered by insertion (first seen = first in vec).
    pub levels: Vec<(f64, f64, f64)>,
}

/// Fixed header: `open_time(8) open(8) high(8) low(8) close(8) volume(8) | blob_off(8) blob_len(4)`
/// = 60 B total.
///
/// The `(blob_off, blob_len)` tail at [`TAIL_OFFSET`] is patched by
/// `DiskStore` after `encode` runs — `encode` only fills the leading 48 bytes.
const TAIL_OFFSET: usize = 48;
const HEADER_SIZE: usize = 60;

impl DataPoint for FootprintPoint {
    const RECORD_SIZE: usize = HEADER_SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.open_time as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.open.to_le_bytes());
        out[16..24].copy_from_slice(&self.high.to_le_bytes());
        out[24..32].copy_from_slice(&self.low.to_le_bytes());
        out[32..40].copy_from_slice(&self.close.to_le_bytes());
        out[40..48].copy_from_slice(&self.volume.to_le_bytes());
        // Bytes 48..60 (blob_off + blob_len) are patched by DiskStore.
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != HEADER_SIZE {
            return None;
        }
        Some(Self {
            open_time: u64::from_le_bytes(bytes[0..8].try_into().ok()?)  as i64,
            open:  f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            high:  f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            low:   f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            close: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            volume: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            // Levels reconstructed from blob by decode_blob.
            levels: Vec::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.open_time }

    /// FootprintPoint is derived-only — never arrives from the exchange wire.
    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> { None }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let mut out = Vec::with_capacity(4 + self.levels.len() * 24);
        out.extend_from_slice(&(self.levels.len() as u32).to_le_bytes());
        for (price, buy, sell) in &self.levels {
            out.extend_from_slice(&price.to_le_bytes());
            out.extend_from_slice(&buy.to_le_bytes());
            out.extend_from_slice(&sell.to_le_bytes());
        }
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        if blob.len() < 4 {
            return Some(p);
        }
        let count = u32::from_le_bytes(blob[0..4].try_into().ok()?) as usize;
        let need = 4 + count * 24;
        if blob.len() < need {
            return Some(p);
        }
        let mut levels = Vec::with_capacity(count);
        let mut off = 4;
        for _ in 0..count {
            let price = f64::from_le_bytes(blob[off..off + 8].try_into().ok()?);
            let buy   = f64::from_le_bytes(blob[off + 8..off + 16].try_into().ok()?);
            let sell  = f64::from_le_bytes(blob[off + 16..off + 24].try_into().ok()?);
            levels.push((price, buy, sell));
            off += 24;
        }
        p.levels = levels;
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blob_round_trip_empty_levels() {
        let p = FootprintPoint {
            open_time: 1_000,
            open: 100.0, high: 110.0, low: 90.0, close: 105.0,
            volume: 0.0, levels: vec![],
        };
        let mut header = vec![0u8; FootprintPoint::RECORD_SIZE];
        p.encode(&mut header);
        let blob = p.encode_blob().unwrap();
        let back = FootprintPoint::decode_blob(&header, &blob).unwrap();
        assert_eq!(back.open_time, 1_000);
        assert!(back.levels.is_empty());
    }

    #[test]
    fn blob_round_trip_with_levels() {
        let levels = vec![
            (100.0_f64, 5.0_f64, 2.0_f64),
            (101.0,     3.0,     7.0),
            (99.5,      1.5,     0.5),
        ];
        let p = FootprintPoint {
            open_time: 2_000,
            open: 99.5, high: 101.0, low: 99.5, close: 100.0,
            volume: 19.0,
            levels: levels.clone(),
        };
        let mut header = vec![0u8; FootprintPoint::RECORD_SIZE];
        p.encode(&mut header);
        let blob = p.encode_blob().unwrap();
        let back = FootprintPoint::decode_blob(&header, &blob).unwrap();
        assert_eq!(back.open_time, 2_000);
        assert_eq!(back.levels.len(), 3);
        for (i, (price, buy, sell)) in back.levels.iter().enumerate() {
            let (ep, eb, es) = levels[i];
            assert!((price - ep).abs() < 1e-12, "price mismatch at {i}");
            assert!((buy   - eb).abs() < 1e-12, "buy mismatch at {i}");
            assert!((sell  - es).abs() < 1e-12, "sell mismatch at {i}");
        }
    }
}
