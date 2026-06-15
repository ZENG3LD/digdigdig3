use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// 32 B record (LE):
///   i64 ts_ms (8), f64 settled_rate (8), i64 settlement_time (8), f64 _pad (8, NaN)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingSettlementPoint {
    pub ts_ms: i64,
    pub settled_rate: f64,
    pub settlement_time: i64,
}

const SIZE: usize = 32;

impl DataPoint for FundingSettlementPoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.settled_rate.to_le_bytes());
        out[16..24].copy_from_slice(&(self.settlement_time as u64).to_le_bytes());
        out[24..32].copy_from_slice(&f64::NAN.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE { return None; }
        Some(Self {
            ts_ms: u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64,
            settled_rate: f64::from_le_bytes(bytes[8..16].try_into().ok()?),
            settlement_time: u64::from_le_bytes(bytes[16..24].try_into().ok()?) as i64,
        })
    }

    fn timestamp_ms(&self) -> i64 { self.ts_ms }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::FundingSettlement { settlement, .. } = ev {
            Some(Self {
                ts_ms: settlement.timestamp,
                settled_rate: settlement.settled_rate,
                settlement_time: settlement.settlement_time,
            })
        } else {
            None
        }
    }
}
