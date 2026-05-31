use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// On-disk record (48 bytes LE) for an account balance change event:
///   u64  ts_ms       (8)
///   u64  asset_hash  (8)  — fnv1a-64 of asset string (e.g. "USDT")
///   f64  free        (8)  — available balance after change
///   f64  locked      (8)  — locked/reserved balance after change
///   f64  total       (8)  — free + locked
///   f64  delta       (8)  — signed change (positive = credit, negative = debit, 0.0 if unknown)
/// Total: 48 bytes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BalanceUpdatePoint {
    pub ts_ms: i64,
    /// fnv1a-64 hash of the asset ticker string.  Use `asset` field for display;
    /// `asset_hash` is the compact storage form.
    pub asset_hash: u64,
    pub free: f64,
    pub locked: f64,
    pub total: f64,
    pub delta: f64,
}

const SIZE: usize = 48;

impl DataPoint for BalanceUpdatePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.asset_hash.to_le_bytes());
        out[16..24].copy_from_slice(&self.free.to_le_bytes());
        out[24..32].copy_from_slice(&self.locked.to_le_bytes());
        out[32..40].copy_from_slice(&self.total.to_le_bytes());
        out[40..48].copy_from_slice(&self.delta.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        let ts_ms = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64;
        let asset_hash = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let free = f64::from_le_bytes(bytes[16..24].try_into().ok()?);
        let locked = f64::from_le_bytes(bytes[24..32].try_into().ok()?);
        let total = f64::from_le_bytes(bytes[32..40].try_into().ok()?);
        let delta = f64::from_le_bytes(bytes[40..48].try_into().ok()?);
        Some(Self { ts_ms, asset_hash, free, locked, total, delta })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::BalanceUpdate(event) = ev {
            Some(Self {
                ts_ms: event.timestamp,
                asset_hash: fnv1a_64(event.asset.as_bytes()),
                free: event.free,
                locked: event.locked,
                total: event.total,
                delta: event.delta.unwrap_or(0.0),
            })
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
