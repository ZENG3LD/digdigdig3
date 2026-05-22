use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// In-memory only. `warning_kind` and `message` are variable-length strings;
/// disk persistence is disabled for `Kind::MarketWarning`.
///
/// `symbol` on the event is `Option<String>`; `MarketWarningPoint` always
/// carries the symbol that the forwarder is subscribed to (may be empty for
/// venue-wide warnings).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketWarningPoint {
    pub ts_ms: i64,
    pub warning_kind: String,
    pub message: String,
}

impl DataPoint for MarketWarningPoint {
    const RECORD_SIZE: usize = 8;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 8 { return None; }
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
}
