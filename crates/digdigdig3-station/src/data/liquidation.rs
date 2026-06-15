use digdigdig3::core::types::{StreamEvent, TradeSide};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 32 B record (LE):
///   u64 ts, f64 price, f64 quantity, f64 value (NaN if absent), u8 side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiquidationPoint {
    pub ts_ms: i64,
    pub price: f64,
    pub quantity: f64,
    pub value: f64,
    pub side: u8,
}

const SIZE: usize = 33;

impl DataPoint for LiquidationPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24..32].copy_from_slice(&self.value.to_le_bytes());
        out[32] = self.side;
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            quantity: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            value: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            side: bytes[32],
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Liquidation { liquidation, .. } = ev {
            let side = match liquidation.side {
                TradeSide::Buy => 0,
                TradeSide::Sell => 1,
            };
            Some(Self {
                ts_ms: liquidation.timestamp,
                price: liquidation.price,
                quantity: liquidation.quantity,
                value: liquidation.value.unwrap_or(f64::NAN),
                side,
            })
        } else {
            None
        }
    }
}
