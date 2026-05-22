use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// Venue / symbol notification (delisting, halt, max-leverage change, ...).
///
/// Disk layout:
/// - Header (20 B): `ts_ms (8) | blob_off (8) | blob_len (4)`.
/// - Blob: `len(warning_kind):u16 | warning_kind_utf8 | len(message):u16 | message_utf8`.
///
/// `symbol` on the wire is `Option<String>` — `None` for venue-wide notices.
/// `MarketWarningPoint` does not carry symbol; the forwarder's
/// `SeriesKey.symbol` provides routing context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketWarningPoint {
    pub ts_ms: i64,
    pub warning_kind: String,
    pub message: String,
}

const TAIL_OFFSET: usize = 8;

impl DataPoint for MarketWarningPoint {
    const RECORD_SIZE: usize = 20;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != Self::RECORD_SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            warning_kind: String::new(),
            message: String::new(),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::MarketWarning { symbol: _, warning_kind, message, timestamp } = ev {
            Some(Self {
                ts_ms: *timestamp,
                warning_kind: warning_kind.clone(),
                message: message.clone(),
            })
        } else {
            None
        }
    }

    fn encode_blob(&self) -> Option<Vec<u8>> {
        let k = self.warning_kind.as_bytes();
        let m = self.message.as_bytes();
        let mut out = Vec::with_capacity(4 + k.len() + m.len());
        out.extend_from_slice(&(k.len() as u16).to_le_bytes());
        out.extend_from_slice(k);
        out.extend_from_slice(&(m.len() as u16).to_le_bytes());
        out.extend_from_slice(m);
        Some(out)
    }

    fn decode_blob(header: &[u8], blob: &[u8]) -> Option<Self> {
        let mut p = Self::decode(header)?;
        let mut cur = 0usize;
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.warning_kind = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
                cur += l;
            }
        }
        if blob.len() >= cur + 2 {
            let l = u16::from_le_bytes(blob[cur..cur + 2].try_into().ok()?) as usize;
            cur += 2;
            if blob.len() >= cur + l {
                p.message = String::from_utf8_lossy(&blob[cur..cur + l]).into_owned();
            }
        }
        Some(p)
    }

    fn blob_pointer_offset() -> Option<usize> { Some(TAIL_OFFSET) }
}
