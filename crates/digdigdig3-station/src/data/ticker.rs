use digdigdig3::core::types::{StreamEvent, Ticker};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 64 B ticker record (LE):
///   u64 ts_ms
///   f64 last, bid, ask
///   f64 high_24h, low_24h, vol_24h, quote_vol_24h, price_change_pct_24h
///
/// Absent Optional fields are stored as f64::NAN.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerPoint {
    pub ts_ms: i64,
    pub last: f64,
    pub bid: f64,
    pub ask: f64,
    pub high_24h: f64,
    pub low_24h: f64,
    pub vol_24h: f64,
    pub quote_vol_24h: f64,
    pub change_pct_24h: f64,
}

const SIZE: usize = 72;

impl TickerPoint {
    pub fn from_ticker(t: &Ticker) -> Self {
        Self {
            ts_ms: t.timestamp,
            last: t.last_price,
            bid: t.bid_price.unwrap_or(f64::NAN),
            ask: t.ask_price.unwrap_or(f64::NAN),
            high_24h: t.high_24h.unwrap_or(f64::NAN),
            low_24h: t.low_24h.unwrap_or(f64::NAN),
            vol_24h: t.volume_24h.unwrap_or(f64::NAN),
            quote_vol_24h: t.quote_volume_24h.unwrap_or(f64::NAN),
            change_pct_24h: t.price_change_percent_24h.unwrap_or(f64::NAN),
        }
    }
}

impl DataPoint for TickerPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.last.to_le_bytes());
        out[16..24].copy_from_slice(&self.bid.to_le_bytes());
        out[24..32].copy_from_slice(&self.ask.to_le_bytes());
        out[32..40].copy_from_slice(&self.high_24h.to_le_bytes());
        out[40..48].copy_from_slice(&self.low_24h.to_le_bytes());
        out[48..56].copy_from_slice(&self.vol_24h.to_le_bytes());
        out[56..64].copy_from_slice(&self.quote_vol_24h.to_le_bytes());
        out[64..72].copy_from_slice(&self.change_pct_24h.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            last: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            bid: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            ask: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            high_24h: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            low_24h: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            vol_24h: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            quote_vol_24h: f64::from_le_bytes(bytes[56..64].try_into().ok()?),
            change_pct_24h: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
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
