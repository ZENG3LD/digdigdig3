use digdigdig3::core::types::{L3Action, OrderBookSide, OrderSide, StreamEvent};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Per-order L3 book update.
///
/// Disk layout:
/// - Header (44 B): `ts_ms (8) | price (8) | quantity (8) | side (1) | _pad (7) | blob_off (8) | blob_len (4)`.
/// - Blob: `len(order_id):u16 | order_id_utf8 | len(action):u16 | action_utf8`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderbookL3Point {
    pub ts_ms: i64,
    pub side: OrderSide,
    pub order_id: String,
    pub price: f64,
    pub quantity: f64,
    pub action: String,
}

const TAIL_OFFSET: usize = 32;

impl DataPoint for OrderbookL3Point {
    const RECORD_SIZE: usize = 44;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24] = match self.side {
            OrderSide::Buy => 0,
            OrderSide::Sell => 1,
        };
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::RECORD_SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            side: match bytes[24] { 0 => OrderSide::Buy, _ => OrderSide::Sell },
            order_id: String::new(),
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            quantity: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            action: String::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderbookL3 { event, .. } = ev {
            // OrderbookL3Event.side is OrderBookSide (Bid/Ask); station stores OrderSide (Buy/Sell).
            let side = match event.side {
                OrderBookSide::Bid => OrderSide::Buy,
                OrderBookSide::Ask => OrderSide::Sell,
            };
            // OrderbookL3Event.action is L3Action enum; station stores it as a string tag.
            let action = match event.action {
                L3Action::Add => "add",
                L3Action::Modify => "modify",
                L3Action::Delete => "delete",
            }.to_string();
            Some(Self {
                ts_ms: event.timestamp,
                side,
                order_id: event.order_id.clone(),
                price: event.price,
                quantity: event.quantity,
                action,
            })
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let id = self.order_id.as_bytes();
        let a = self.action.as_bytes();
        let mut out = Vec::with_capacity(4 + id.len() + a.len());
        out.extend_from_slice(&(id.len() as u16).to_le_bytes());
        out.extend_from_slice(id);
        out.extend_from_slice(&(a.len() as u16).to_le_bytes());
        out.extend_from_slice(a);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        let mut cur = 0usize;
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.order_id = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
                cur += l;
            }
        }
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.action = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}
