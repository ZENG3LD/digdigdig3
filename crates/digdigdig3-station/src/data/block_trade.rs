use digdigdig3::core::types::{StreamEvent, TradeSide};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Block trade — large off-book print, identified by exchange-issued `block_id`.
///
/// Disk layout:
/// - Header (44 B): `ts_ms (8) | price (8) | quantity (8) | side (1) | is_iv (1) | _pad (6) | blob_off (8) | blob_len (4)`.
/// - Blob: `len(block_id):u16 | block_id_utf8`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTradePoint {
    pub ts_ms: i64,
    pub block_id: String,
    pub price: f64,
    pub quantity: f64,
    pub side: TradeSide,
    pub is_iv: bool,
}

const TAIL_OFFSET: usize = 32;

impl DataPoint for BlockTradePoint {
    const RECORD_SIZE: usize = 44;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.price.to_le_bytes());
        out[16..24].copy_from_slice(&self.quantity.to_le_bytes());
        out[24] = match self.side {
            TradeSide::Buy => 0,
            TradeSide::Sell => 1,
        };
        out[25] = u8::from(self.is_iv);
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::RECORD_SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            block_id: String::new(),
            price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            quantity: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            side: match bytes[24] { 0 => TradeSide::Buy, _ => TradeSide::Sell },
            is_iv: bytes[25] != 0,
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::BlockTrade {
            symbol: _,
            block_id,
            price,
            quantity,
            side,
            timestamp,
            is_iv,
        } = ev {
            Some(Self {
                ts_ms: *timestamp,
                block_id: block_id.clone(),
                price: *price,
                quantity: *quantity,
                side: *side,
                is_iv: *is_iv,
            })
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let bytes = self.block_id.as_bytes();
        let mut out = Vec::with_capacity(2 + bytes.len());
        out.extend_from_slice(&(bytes.len() as u16).to_le_bytes());
        out.extend_from_slice(bytes);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        if blob.len() >= 2 {
            let len = u16::from_le_bytes(blob[0..2].try_into().ok()?) as usize;
            if blob.len() >= 2 + len {
                p.block_id = String::from_utf8_lossy(&blob[2..2 + len]).into_owned();
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}
