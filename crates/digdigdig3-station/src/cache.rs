//! Cache layer — Phase 1: a thin Station-level wrapper around
//! `digdigdig3::core::rest_cache::RestCache` keyed by
//! `(ExchangeId, AccountType, raw_symbol)` for `get_ticker`.
//!
//! Step 6+: extend with orderbook L1 TTL, symbol metadata, and warm-start from
//! disk. See `docs/plans/station-architecture.md` §6.

use std::time::Duration;

use crate::rest_cache::RestCache;
use digdigdig3::core::types::{AccountType, ExchangeId, Ticker};

pub type TickerKey = (ExchangeId, AccountType, String);

/// Build a default RestCache<TickerKey, Ticker> with the supplied TTL.
pub fn ticker_cache(ttl: Duration) -> RestCache<TickerKey, Ticker> {
    RestCache::new(ttl)
}

/// Configuration knobs the builder will accept in step 6+. For Phase 1 only
/// `ticker_ttl` is consumed.
#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub enabled: bool,
    pub ticker_ttl: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self { enabled: false, ticker_ttl: Duration::from_secs(1) }
    }
}

impl CacheConfig {
    pub fn on() -> Self {
        Self { enabled: true, ticker_ttl: Duration::from_secs(1) }
    }
    pub fn ticker_ttl(mut self, ttl: Duration) -> Self {
        self.ticker_ttl = ttl;
        self
    }
}
