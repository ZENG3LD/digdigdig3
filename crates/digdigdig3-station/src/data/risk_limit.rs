use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 48 B record (LE):
///   i64 ts_ms (8), u32 tier (4), u32 _pad (4), f64 max_leverage (8),
///   f64 max_position_value (8), f64 maintenance_margin_rate (8), f64 initial_margin_rate (8)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLimitPoint {
    pub ts_ms: i64,
    pub tier: u32,
    pub max_leverage: f64,
    pub max_position_value: f64,
    pub maintenance_margin_rate: f64,
    pub initial_margin_rate: f64,
}

const SIZE: usize = 48;

impl DataPoint for RiskLimitPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..12].copy_from_slice(&self.tier.to_le_bytes());
        out[12..16].copy_from_slice(&0u32.to_le_bytes()); // padding
        out[16..24].copy_from_slice(&self.max_leverage.to_le_bytes());
        out[24..32].copy_from_slice(&self.max_position_value.to_le_bytes());
        out[32..40].copy_from_slice(&self.maintenance_margin_rate.to_le_bytes());
        out[40..48].copy_from_slice(&self.initial_margin_rate.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            tier: u32::from_le_bytes(bytes[8..12].try_into().ok()?),
            max_leverage: f64::from_le_bytes(bytes[16..24].try_into().ok()?),
            max_position_value: f64::from_le_bytes(bytes[24..32].try_into().ok()?),
            maintenance_margin_rate: f64::from_le_bytes(bytes[32..40].try_into().ok()?),
            initial_margin_rate: f64::from_le_bytes(bytes[40..48].try_into().ok()?),
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::RiskLimit {
            symbol: _,
            tier,
            max_leverage,
            max_position_value,
            maintenance_margin_rate,
            initial_margin_rate,
            timestamp,
        } = ev {
            Some(Self {
                ts_ms: *timestamp,
                tier: *tier,
                max_leverage: *max_leverage,
                max_position_value: *max_position_value,
                maintenance_margin_rate: *maintenance_margin_rate,
                initial_margin_rate: *initial_margin_rate,
            })
        } else {
            None
        }
    }
}
