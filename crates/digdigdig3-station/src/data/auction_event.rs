use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Auction event snapshot (Gemini opening / indicative / closing auction).
///
/// Disk layout:
/// - Header (36 B): `ts_ms (8) | indicative_price (8) | indicative_qty (8) | blob_off (8) | blob_len (4)`.
/// - Blob: `len(auction_id):u16 | auction_id_utf8 | len(state):u16 | state_utf8`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionEventPoint {
    /// Event timestamp in milliseconds.
    pub ts_ms: i64,
    /// Exchange-assigned auction identifier.
    pub auction_id: String,
    /// Indicative clearing price at current auction state.
    pub indicative_price: f64,
    /// Indicative clearing quantity at current auction state.
    pub indicative_qty: f64,
    /// Auction phase: `"opening"` | `"indicative"` | `"closing"`.
    pub state: String,
}

/// Byte offset in the fixed header where the blob (offset, len) pointer lives.
const TAIL_OFFSET: usize = 24;

impl DataPoint for AuctionEventPoint {
    /// 24 bytes of numeric data + 8 (blob_off) + 4 (blob_len) = 36 bytes.
    const RECORD_SIZE: usize = 36;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.indicative_price.to_le_bytes());
        out[16..24].copy_from_slice(&self.indicative_qty.to_le_bytes());
        // bytes[24..36] = blob pointer (off:u64, len:u32) written by DiskStore
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::RECORD_SIZE {
            return None;
        }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            // auction_id and state are populated by decode_blob; start empty.
            auction_id: String::new(),
            indicative_price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            indicative_qty: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            state: String::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::AuctionEvent { auction, .. } = ev {
            Some(Self {
                ts_ms: auction.timestamp,
                auction_id: auction.auction_id.clone(),
                indicative_price: auction.indicative_price,
                indicative_qty: auction.indicative_qty,
                state: auction.state.clone(),
            })
        } else {
            None
        }
    }

    // ── Blob persistence for string fields ────────────────────────────────────

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let id = self.auction_id.as_bytes();
        let st = self.state.as_bytes();
        let mut out = Vec::with_capacity(4 + id.len() + st.len());
        out.extend_from_slice(&(id.len() as u16).to_le_bytes());
        out.extend_from_slice(id);
        out.extend_from_slice(&(st.len() as u16).to_le_bytes());
        out.extend_from_slice(st);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        let mut cur = 0usize;
        // auction_id
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.auction_id = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
                cur += l;
            }
        }
        // state
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.state = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> {
        Some(TAIL_OFFSET)
    }
}
