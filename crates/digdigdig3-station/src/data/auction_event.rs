use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Pre-open / pre-close auction snapshot.
///
/// Disk layout:
/// - Header (36 B): `ts_ms (8) | indicative_price (8) | indicative_qty (8) | _pad (0) | blob_off (8) | blob_len (4)`.
/// - Blob: `len(auction_id):u16 | auction_id_utf8 | len(state):u16 | state_utf8`.
///
/// `indicative_price` / `indicative_qty` are stored as `f64`; on-wire `None`
/// is encoded as NaN to keep the header fixed-size.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuctionEventPoint {
    pub ts_ms: i64,
    pub auction_id: String,
    pub indicative_price: f64,
    pub indicative_qty: f64,
    pub state: String,
}

const TAIL_OFFSET: usize = 24;

impl DataPoint for AuctionEventPoint {
    const RECORD_SIZE: usize = 36;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.indicative_price.to_le_bytes());
        out[16..24].copy_from_slice(&self.indicative_qty.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::RECORD_SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            auction_id: String::new(),
            indicative_price: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            indicative_qty: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            state: String::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::AuctionEvent {
            symbol: _,
            auction_id,
            indicative_price,
            indicative_qty,
            state,
            timestamp,
        } = ev {
            Some(Self {
                ts_ms: *timestamp,
                auction_id: auction_id.clone(),
                indicative_price: indicative_price.unwrap_or(f64::NAN),
                indicative_qty: indicative_qty.unwrap_or(f64::NAN),
                state: state.clone(),
            })
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let a = self.auction_id.as_bytes();
        let s = self.state.as_bytes();
        let mut out = Vec::with_capacity(4 + a.len() + s.len());
        out.extend_from_slice(&(a.len() as u16).to_le_bytes());
        out.extend_from_slice(a);
        out.extend_from_slice(&(s.len() as u16).to_le_bytes());
        out.extend_from_slice(s);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        let mut cur = 0usize;
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.auction_id = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
                cur += l;
            }
        }
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.state = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}
