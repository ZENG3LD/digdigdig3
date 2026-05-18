//! Internal broadcast bus for live WebSocket events.
//!
//! Each WS subscriber task publishes `BusEvent` into `tokio::sync::broadcast`.
//! gRPC `LiveEvents::Subscribe` receivers filter by their (exchange, account,
//! symbol, stream_kind) selector and forward matching events as `EventMessage`.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use tokio::sync::broadcast;

/// One event on the internal bus.
#[derive(Debug, Clone)]
pub struct BusEvent {
    pub exchange: String,
    pub account: String,
    pub symbol: String,
    pub stream_kind: String,
    pub timestamp_ms: i64,
    pub event_type: String,
    pub payload_json: Vec<u8>,
}

/// Capacity of the broadcast channel (events in flight before lagging).
const BUS_CAPACITY: usize = 4096;

/// Shared broadcast bus + active subscription counter.
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<BusEvent>,
    active_subs: Arc<AtomicU32>,
}

impl EventBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(BUS_CAPACITY);
        Self {
            sender,
            active_subs: Arc::new(AtomicU32::new(0)),
        }
    }

    /// Publish an event. Returns the number of active receivers that received it.
    pub fn publish(&self, ev: BusEvent) -> usize {
        self.sender.send(ev).unwrap_or(0)
    }

    /// Subscribe to the bus. Caller will receive all events published after this call.
    pub fn subscribe(&self) -> broadcast::Receiver<BusEvent> {
        self.active_subs.fetch_add(1, Ordering::Relaxed);
        self.sender.subscribe()
    }

    /// Called when a gRPC subscription stream ends.
    pub fn unsubscribe(&self) {
        self.active_subs.fetch_sub(1, Ordering::Relaxed);
    }

    /// Number of active gRPC streaming subscribers.
    pub fn active_subscriptions(&self) -> u32 {
        self.active_subs.load(Ordering::Relaxed)
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
