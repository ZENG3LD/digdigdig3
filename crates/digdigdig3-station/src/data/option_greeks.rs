use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 72 B record (LE):
///   i64 ts_ms (8), then 8 × f64 (NaN = absent):
///   delta, gamma, vega, theta, rho, mark_iv, bid_iv, ask_iv
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionGreeksPoint {
    pub ts_ms: i64,
    pub delta: f64,
    pub gamma: f64,
    pub vega: f64,
    pub theta: f64,
    pub rho: f64,
    pub mark_iv: f64,
    pub bid_iv: f64,
    pub ask_iv: f64,
}

const SIZE: usize = 72;

impl DataPoint for OptionGreeksPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.delta.to_le_bytes());
        out[16..24].copy_from_slice(&self.gamma.to_le_bytes());
        out[24..32].copy_from_slice(&self.vega.to_le_bytes());
        out[32..40].copy_from_slice(&self.theta.to_le_bytes());
        out[40..48].copy_from_slice(&self.rho.to_le_bytes());
        out[48..56].copy_from_slice(&self.mark_iv.to_le_bytes());
        out[56..64].copy_from_slice(&self.bid_iv.to_le_bytes());
        out[64..72].copy_from_slice(&self.ask_iv.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            delta: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            gamma: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            vega: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            theta: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            rho: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
            mark_iv: f64::from_le_bytes(bytes[48..56].try_into().ok()?),
            bid_iv: f64::from_le_bytes(bytes[56..64].try_into().ok()?),
            ask_iv: f64::from_le_bytes(bytes[64..72].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OptionGreeks {
            symbol: _,
            delta,
            gamma,
            vega,
            theta,
            rho,
            mark_iv,
            bid_iv,
            ask_iv,
            timestamp,
        } = ev {
            Some(Self {
                ts_ms: *timestamp,
                delta: delta.unwrap_or(f64::NAN),
                gamma: gamma.unwrap_or(f64::NAN),
                vega: vega.unwrap_or(f64::NAN),
                theta: theta.unwrap_or(f64::NAN),
                rho: rho.unwrap_or(f64::NAN),
                mark_iv: mark_iv.unwrap_or(f64::NAN),
                bid_iv: bid_iv.unwrap_or(f64::NAN),
                ask_iv: ask_iv.unwrap_or(f64::NAN),
            })
        } else {
            None
        }
    }
}
