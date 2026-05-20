use digdigdig3::core::types::{StreamEvent, TradeSide};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 48 B record — same shape as TradePoint but separate kind / disk slot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggTradePoint {
    pub ts_ms: i64,
    pub price: f64,
    pub quantity: f64,
    pub side: u8,
    pub agg_id: u64,
}

const SIZE: usize = 48;

impl DataPoint for AggTradePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24] = self.side;
        out[25..33].copy_from_slice(&self.agg_id.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            quantity: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            side: bytes[24],
            agg_id: u64::from_le_bytes(bytes[25..33].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::AggTrade { price, quantity, side, timestamp, aggregate_id, .. } = ev {
            let s = match side {
                TradeSide::Buy => 0,
                TradeSide::Sell => 1,
            };
            Some(Self {
                ts_ms: *timestamp,
                price: *price,
                quantity: *quantity,
                side: s,
                agg_id: *aggregate_id as u64,
            })
        } else {
            None
        }
    }
}
