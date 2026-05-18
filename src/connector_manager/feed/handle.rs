//! `FeedHandle` — per-subscription token returned by `MarketFeed::subscribe_*`.
//!
//! Holds a `tokio::sync::broadcast::Receiver` so each subscriber sees its own
//! copy of events. Dropping the handle decrements a refcount; when it hits
//! zero (and after `unsub_grace`) the upstream subscription is released —
//! mirrors MLC `WsActorMap` behaviour.

use std::sync::Arc;

use tokio::sync::broadcast;

use crate::core::types::{AccountType, ExchangeId, StreamEvent};

/// Event emitted by `MarketFeed`. Wraps the raw `StreamEvent` with the source
/// tuple so a single merged stream can carry events from many exchanges.
#[derive(Debug, Clone)]
pub struct FeedEvent {
    pub exchange: ExchangeId,
    pub account_type: AccountType,
    pub symbol: String,
    pub event: StreamEvent,
}

/// Handle returned to the consumer. Receive events via `.recv().await` or
/// (when `futures_util::StreamExt` is in scope) treat it as a `Stream` via
/// `into_stream()`.
pub struct FeedHandle {
    pub(crate) rx: broadcast::Receiver<FeedEvent>,
    pub(crate) _keep_alive: Arc<()>,
}

impl FeedHandle {
    /// Receive the next event. `None` on lagging-close / channel-closed.
    pub async fn recv(&mut self) -> Option<FeedEvent> {
        loop {
            match self.rx.recv().await {
                Ok(ev) => return Some(ev),
                Err(broadcast::error::RecvError::Closed) => return None,
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
            }
        }
    }

    /// Convert into a `Stream<Item = FeedEvent>` (requires the caller to bring
    /// `tokio_stream` or call `.recv()` in a loop).
    pub fn into_receiver(self) -> broadcast::Receiver<FeedEvent> { self.rx }
}
