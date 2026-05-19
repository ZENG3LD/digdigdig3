//! `FeedBuilder` — fluent builder for `MarketFeed`. Mirrors the stk-style
//! "Option<X> + runtime validation" pattern from tessera/servertoolkit.

use std::sync::Arc;
use std::time::Duration;

use super::feed::MarketFeed;
use super::options::{FeedOptions, OrderbookTrackerOpt, PersistenceOption, ReconnectPolicy};
use crate::connector_manager::ExchangeHub;

pub struct FeedBuilder {
    hub: Arc<ExchangeHub>,
    opts: FeedOptions,
}

impl FeedBuilder {
    pub(crate) fn new(hub: Arc<ExchangeHub>) -> Self {
        Self { hub, opts: FeedOptions::default() }
    }

    // ── persistence (bool/enum, per user spec) ───────────────────────────
    pub fn persistence(mut self, opt: PersistenceOption) -> Self { self.opts.persistence = opt; self }
    pub fn with_storage(mut self, on: bool) -> Self {
        self.opts.persistence = if on { PersistenceOption::Default } else { PersistenceOption::Off };
        self
    }

    // ── reconnect / multiplex / cache toggles ────────────────────────────
    pub fn reconnect(mut self, policy: ReconnectPolicy) -> Self { self.opts.reconnect = policy; self }
    pub fn with_orderbook_tracker(mut self, opt: OrderbookTrackerOpt) -> Self { self.opts.orderbook = opt; self }
    pub fn unsub_grace(mut self, d: Duration) -> Self { self.opts.unsub_grace = d; self }
    pub fn cache_symbols(mut self, on: bool) -> Self { self.opts.symbol_cache = on; self }
    pub fn broadcast_capacity(mut self, cap: usize) -> Self { self.opts.broadcast_capacity = cap.max(8); self }

    // ── terminal ─────────────────────────────────────────────────────────
    pub fn build(self) -> MarketFeed {
        MarketFeed::new(self.hub, self.opts)
    }
}
