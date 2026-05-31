use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// On-disk record (64 bytes LE) for a futures position change event:
///   u64  ts_ms             (8)
///   u8   side              (1)  — 0=Long 1=Short 2=Net/Both
///   u8   _pad              (7)
///   f64  qty               (8)  — position size (signed in net mode; absolute here)
///   f64  entry_price       (8)
///   f64  mark_price        (8)  — 0.0 if unknown
///   f64  unrealized_pnl    (8)
///   f64  realized_pnl      (8)  — 0.0 if not provided
///   f64  liquidation_price (8)  — 0.0 if unknown
/// Total: 64 bytes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionUpdatePoint {
    pub ts_ms: i64,
    /// 0=Long, 1=Short, 2=Net/Both
    pub side: u8,
    pub qty: f64,
    pub entry_price: f64,
    pub mark_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub liquidation_price: f64,
}

const SIZE: usize = 64;

impl PositionUpdatePoint {
    pub fn side_label(&self) -> &'static str {
        match self.side {
            0 => "Long",
            1 => "Short",
            _ => "Net",
        }
    }
}

impl DataPoint for PositionUpdatePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8] = self.side;
        // out[9..16] = padding
        out[16..24].copy_from_slice(&self.qty.to_le_bytes());
        out[24..32].copy_from_slice(&self.entry_price.to_le_bytes());
        out[32..40].copy_from_slice(&self.mark_price.to_le_bytes());
        out[40..48].copy_from_slice(&self.unrealized_pnl.to_le_bytes());
        out[48..56].copy_from_slice(&self.realized_pnl.to_le_bytes());
        out[56..64].copy_from_slice(&self.liquidation_price.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        let ts_ms = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64;
        let side = bytes[8];
        let qty = f64::from_le_bytes(bytes[16..24].try_into().ok()?);
        let entry_price = f64::from_le_bytes(bytes[24..32].try_into().ok()?);
        let mark_price = f64::from_le_bytes(bytes[32..40].try_into().ok()?);
        let unrealized_pnl = f64::from_le_bytes(bytes[40..48].try_into().ok()?);
        let realized_pnl = f64::from_le_bytes(bytes[48..56].try_into().ok()?);
        let liquidation_price = f64::from_le_bytes(bytes[56..64].try_into().ok()?);
        Some(Self {
            ts_ms,
            side,
            qty,
            entry_price,
            mark_price,
            unrealized_pnl,
            realized_pnl,
            liquidation_price,
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::PositionUpdate { event, .. } = ev {
            use digdigdig3::core::types::PositionSide;
            let side = match event.side {
                PositionSide::Long => 0,
                PositionSide::Short => 1,
                PositionSide::Both => 2,
            };
            Some(Self {
                ts_ms: event.timestamp,
                side,
                qty: event.quantity,
                entry_price: event.entry_price,
                mark_price: event.mark_price.unwrap_or(0.0),
                unrealized_pnl: event.unrealized_pnl,
                realized_pnl: event.realized_pnl.unwrap_or(0.0),
                liquidation_price: event.liquidation_price.unwrap_or(0.0),
            })
        } else {
            None
        }
    }
}
