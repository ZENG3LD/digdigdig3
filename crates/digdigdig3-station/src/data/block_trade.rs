use digdigdig3::core::types::{StreamEvent, TradeSide};
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// In-memory only. Disk persistence is not supported for this type because
/// `block_id` is a variable-length string. `RECORD_SIZE` encodes only the
/// timestamp as an 8-byte stub so the type satisfies the `DataPoint` bound;
/// `is_enabled_for(Kind::BlockTrade)` returns `false` so `DiskStore` is
/// never opened for this kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockTradePoint {
    pub ts_ms: i64,
    pub block_id: String,
    pub price: f64,
    pub quantity: f64,
    pub side: TradeSide,
    pub is_iv: bool,
}

impl DataPoint for BlockTradePoint {
    /// Stub size — encode/decode only carry the timestamp.
    /// Persistence is disabled for this kind so this path is never taken.
    const RECORD_SIZE: usize = 8;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            block_id: String::new(),
            price: 0.0,
            quantity: 0.0,
            side: TradeSide::Buy,
            is_iv: false,
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
}
