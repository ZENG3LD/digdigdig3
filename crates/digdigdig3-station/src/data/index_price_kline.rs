use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 64 B bar record (same layout as BarPoint):
///   u64 open_time_ms, f64 open, high, low, close, volume, quote_volume, u64 trades_count
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexPriceKlinePoint {
    pub open_time: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub quote_volume: f64,
    pub trades_count: u64,
}

const SIZE: usize = 64;

impl DataPoint for IndexPriceKlinePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.open_time as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.open.to_le_bytes());
        out[16..24].copy_from_slice(&self.high.to_le_bytes());
        out[24..32].copy_from_slice(&self.low.to_le_bytes());
        out[32..40].copy_from_slice(&self.close.to_le_bytes());
        out[40..48].copy_from_slice(&self.volume.to_le_bytes());
        out[48..56].copy_from_slice(&self.quote_volume.to_le_bytes());
        out[56..64].copy_from_slice(&self.trades_count.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            open_time: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            open: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            high: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            low: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            close: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            volume: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            quote_volume: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            trades_count: u64::from_le_bytes(bytes[56..64].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.open_time }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::IndexPriceKline { symbol: _, interval: _, kline } = ev {
            Some(Self {
                open_time: kline.open_time,
                open: kline.open,
                high: kline.high,
                low: kline.low,
                close: kline.close,
                volume: kline.volume,
                quote_volume: kline.quote_volume.unwrap_or(f64::NAN),
                trades_count: kline.trades.unwrap_or(0),
            })
        } else {
            None
        }
    }
}
