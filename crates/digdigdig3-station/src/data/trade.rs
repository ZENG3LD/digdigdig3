use digdigdig3::core::types::{StreamEvent, TradeSide};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// On-disk record (48 bytes LE):
///   u64 ts_ms (8)
///   f64 price (8)
///   f64 quantity (8)
///   u8  side (1)  0=Buy 1=Sell
///   u64 trade_id_hash (8) — fnv1a-64 of original id string
///   23 bytes reserved (zero-padded) — flags / sequence (future use)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradePoint {
    pub ts_ms: i64,
    pub price: f64,
    pub quantity: f64,
    pub side: u8,
    pub trade_id_hash: u64,
}

const SIZE: usize = 48;

impl TradePoint {
    pub fn from_public(t: &digdigdig3::core::types::PublicTrade) -> Self {
        Self {
            ts_ms: t.timestamp,
            price: t.price,
            quantity: t.quantity,
            side: match t.side {
                TradeSide::Buy => 0,
                TradeSide::Sell => 1,
            },
            trade_id_hash: fnv1a_64(t.id.as_bytes()),
        }
    }

    pub fn side_label(&self) -> &'static str {
        match self.side {
            0 => "Buy",
            1 => "Sell",
            _ => "?",
        }
    }
}

impl DataPoint for TradePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24] = self.side;
        out[25..33].copy_from_slice(&self.trade_id_hash.to_le_bytes());
        // bytes 33..48 reserved (already zero in caller-allocated buffer)
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        let ts_ms = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64;
        let price = f64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let quantity = f64::from_le_bytes(bytes[16..24].try_into().ok()?);
        let side = bytes[24];
        let trade_id_hash = u64::from_le_bytes(bytes[25..33].try_into().ok()?);
        Some(Self { ts_ms, price, quantity, side, trade_id_hash })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::Trade(t) = ev {
            Some(Self::from_public(t))
        } else {
            None
        }
    }
}

fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}
