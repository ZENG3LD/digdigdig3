use digdigdig3::core::types::StreamEvent;
use serde::{Deserialize, Serialize};

use crate::series::DataPoint;

/// On-disk record (96 bytes LE) for an order lifecycle event:
///   u64  ts_ms              (8)  — event timestamp
///   u64  order_id_hash      (8)  — fnv1a-64 of order_id string
///   u64  client_id_hash     (8)  — fnv1a-64 of client_order_id, or 0
///   u8   status             (1)  — OrderStatus enum value (see STATUS_* consts)
///   u8   side               (1)  — 0=Buy 1=Sell
///   u8   order_type         (1)  — 0=Market 1=Limit 2=Other
///   u8   _pad               (1)
///   f64  price              (8)  — limit price (0.0 for market)
///   f64  qty                (8)  — total order quantity
///   f64  filled_qty         (8)  — cumulative filled quantity
///   f64  avg_price          (8)  — average fill price (0.0 if none)
///   f64  last_fill_price    (8)  — last fill price (0.0 if none)
///   f64  last_fill_qty      (8)  — last fill quantity (0.0 if none)
///   f64  last_fill_comm     (8)  — last fill commission (0.0 if none)
///   32 bytes reserved (zero-padded)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderUpdatePoint {
    pub ts_ms: i64,
    pub order_id_hash: u64,
    pub client_id_hash: u64,
    /// OrderStatus encoded: 0=New/Open, 1=PartiallyFilled, 2=Filled,
    /// 3=Canceled, 4=Rejected/Expired
    pub status: u8,
    /// 0=Buy, 1=Sell
    pub side: u8,
    /// 0=Market, 1=Limit, 2=Other
    pub order_type: u8,
    pub price: f64,
    pub qty: f64,
    pub filled_qty: f64,
    pub avg_price: f64,
    pub last_fill_price: f64,
    pub last_fill_qty: f64,
    pub last_fill_commission: f64,
}

const SIZE: usize = 96;

impl OrderUpdatePoint {
    pub fn status_label(&self) -> &'static str {
        match self.status {
            0 => "New",
            1 => "PartiallyFilled",
            2 => "Filled",
            3 => "Canceled",
            4 => "Rejected/Expired",
            _ => "Unknown",
        }
    }

    pub fn side_label(&self) -> &'static str {
        match self.side {
            0 => "Buy",
            1 => "Sell",
            _ => "?",
        }
    }

    pub fn order_type_label(&self) -> &'static str {
        match self.order_type {
            0 => "Market",
            1 => "Limit",
            _ => "Other",
        }
    }
}

impl DataPoint for OrderUpdatePoint {
    const RECORD_SIZE: usize = SIZE;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&(self.ts_ms as u64).to_le_bytes());
        out[8..16].copy_from_slice(&self.order_id_hash.to_le_bytes());
        out[16..24].copy_from_slice(&self.client_id_hash.to_le_bytes());
        out[24] = self.status;
        out[25] = self.side;
        out[26] = self.order_type;
        // out[27] = pad
        out[28..36].copy_from_slice(&self.price.to_le_bytes());
        out[36..44].copy_from_slice(&self.qty.to_le_bytes());
        out[44..52].copy_from_slice(&self.filled_qty.to_le_bytes());
        out[52..60].copy_from_slice(&self.avg_price.to_le_bytes());
        out[60..68].copy_from_slice(&self.last_fill_price.to_le_bytes());
        out[68..76].copy_from_slice(&self.last_fill_qty.to_le_bytes());
        out[76..84].copy_from_slice(&self.last_fill_commission.to_le_bytes());
        // bytes 84..96 reserved
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != SIZE {
            return None;
        }
        let ts_ms = u64::from_le_bytes(bytes[0..8].try_into().ok()?) as i64;
        let order_id_hash = u64::from_le_bytes(bytes[8..16].try_into().ok()?);
        let client_id_hash = u64::from_le_bytes(bytes[16..24].try_into().ok()?);
        let status = bytes[24];
        let side = bytes[25];
        let order_type = bytes[26];
        let price = f64::from_le_bytes(bytes[28..36].try_into().ok()?);
        let qty = f64::from_le_bytes(bytes[36..44].try_into().ok()?);
        let filled_qty = f64::from_le_bytes(bytes[44..52].try_into().ok()?);
        let avg_price = f64::from_le_bytes(bytes[52..60].try_into().ok()?);
        let last_fill_price = f64::from_le_bytes(bytes[60..68].try_into().ok()?);
        let last_fill_qty = f64::from_le_bytes(bytes[68..76].try_into().ok()?);
        let last_fill_commission = f64::from_le_bytes(bytes[76..84].try_into().ok()?);
        Some(Self {
            ts_ms,
            order_id_hash,
            client_id_hash,
            status,
            side,
            order_type,
            price,
            qty,
            filled_qty,
            avg_price,
            last_fill_price,
            last_fill_qty,
            last_fill_commission,
        })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts_ms
    }

    fn from_stream_event(ev: &StreamEvent) -> Option<Self> {
        if let StreamEvent::OrderUpdate { event, .. } = ev {
            use digdigdig3::core::types::{OrderSide, OrderStatus, OrderType};
            let status = match event.status {
                OrderStatus::New => 0,
                OrderStatus::Open => 0,
                OrderStatus::PartiallyFilled => 1,
                OrderStatus::Filled => 2,
                OrderStatus::Canceled => 3,
                OrderStatus::Rejected => 4,
                OrderStatus::Expired => 4,
            };
            let side = match event.side {
                OrderSide::Buy => 0,
                OrderSide::Sell => 1,
            };
            let order_type = match event.order_type {
                OrderType::Market => 0,
                OrderType::Limit { .. } => 1,
                _ => 2,
            };
            Some(Self {
                ts_ms: event.timestamp,
                order_id_hash: fnv1a_64(event.order_id.as_bytes()),
                client_id_hash: event
                    .client_order_id
                    .as_deref()
                    .map(|s| fnv1a_64(s.as_bytes()))
                    .unwrap_or(0),
                status,
                side,
                order_type,
                price: event.price.unwrap_or(0.0),
                qty: event.quantity,
                filled_qty: event.filled_quantity,
                avg_price: event.average_price.unwrap_or(0.0),
                last_fill_price: event.last_fill_price.unwrap_or(0.0),
                last_fill_qty: event.last_fill_quantity.unwrap_or(0.0),
                last_fill_commission: event.last_fill_commission.unwrap_or(0.0),
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
