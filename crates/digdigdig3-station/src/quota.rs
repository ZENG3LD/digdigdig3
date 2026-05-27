//! Per-consumer subscription + REST rate quotas.
//!
//! Opt-in layer over [`Station`]. A consumer registers via
//! [`Station::register_consumer`]`(quota) -> ConsumerHandle`, then uses
//! [`ConsumerHandle::subscribe`] and [`ConsumerHandle::rest_gate`]
//! instead of the raw [`Station::subscribe`]. Each consumer has its own
//! active-sub counter, REST token bucket, and optional
//! `(exchange, kind, symbol)` whitelist.
//!
//! Existing [`Station::subscribe`] is unchanged. Consumers that do not
//! want quotas keep using it directly.
//!
//! # Atomic-or-nothing
//!
//! [`ConsumerHandle::subscribe`] enforces `max_active_subs` and whitelist
//! BEFORE any `acquire_or_spawn` call. If either check fails, no
//! subscriptions are made. If a stream fails mid-batch with
//! `StreamNotSupported`, the acquired refs for that batch are rolled back
//! and `Err(QuotaError::Inner(...))` is returned — the active-sub counter
//! never sees a partial increment.
//!
//! # REST passthrough
//!
//! REST quota uses the token-handoff pattern: call
//! [`ConsumerHandle::rest_gate`] to consume one token from the bucket,
//! get back `Arc<StationInner>`, then call the connector method directly.
//! This avoids ~75 LOC of forwarding boilerplate and is immune to future
//! additions of REST methods.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

use digdigdig3::core::types::ExchangeId;
use thiserror::Error;
use tokio::sync::Mutex;
use std::time::Instant;

use crate::series::Kind;
use crate::station::StationInner;
use crate::subscription::MultiplexRef;
use crate::{Station, StationError, SubscribeReport, SubscriptionSet};

// ---------------------------------------------------------------------------
// Public configuration types
// ---------------------------------------------------------------------------

/// Per-consumer quota configuration. All limits are optional; `None`
/// means unlimited. Build with the chaining helpers or directly set fields.
#[derive(Clone, Debug)]
pub struct ConsumerQuota {
    /// Maximum number of simultaneously active subscriptions (distinct
    /// `SeriesKey`s) this consumer may hold. `None` = unlimited.
    pub max_active_subs: Option<u32>,
    /// Maximum REST requests per [`rest_window`][Self::rest_window].
    /// `None` = unlimited.
    pub max_rest_per_window: Option<u32>,
    /// Token-bucket window for REST rate limiting.
    pub rest_window: Duration,
    /// Optional whitelist. When set, subscribe rejects any
    /// `(exchange, kind, symbol)` not in the list.
    pub whitelist: Option<Arc<ConsumerWhitelist>>,
}

impl Default for ConsumerQuota {
    fn default() -> Self {
        Self {
            max_active_subs: None,
            max_rest_per_window: None,
            rest_window: Duration::from_secs(60),
            whitelist: None,
        }
    }
}

impl ConsumerQuota {
    /// No limits, no whitelist.
    pub fn unlimited() -> Self {
        Self::default()
    }

    /// Cap simultaneous active subscriptions.
    pub fn max_active_subs(mut self, n: u32) -> Self {
        self.max_active_subs = Some(n);
        self
    }

    /// REST token bucket: `per_window` calls allowed per `window` duration.
    pub fn max_rest(mut self, per_window: u32, window: Duration) -> Self {
        self.max_rest_per_window = Some(per_window);
        self.rest_window = window;
        self
    }

    /// Attach a whitelist filter.
    pub fn whitelist(mut self, wl: ConsumerWhitelist) -> Self {
        self.whitelist = Some(Arc::new(wl));
        self
    }
}

/// Whitelist that restricts which `(exchange, kind, symbol)` tuples a
/// consumer may subscribe to. Empty `HashSet` for `exchanges` / `kinds`
/// means "allow any"; `None` for `symbols` means "allow any symbol".
#[derive(Debug, Default)]
pub struct ConsumerWhitelist {
    /// Allowed exchange IDs. Empty = any.
    pub exchanges: HashSet<ExchangeId>,
    /// Allowed stream kinds. Empty = any.
    pub kinds: HashSet<Kind>,
    /// Allowed symbols (raw user-input form). `None` = any.
    pub symbols: Option<HashSet<String>>,
}

impl ConsumerWhitelist {
    /// Create an empty whitelist (all fields = allow-any).
    pub fn new() -> Self {
        Self::default()
    }

    /// Allow subscriptions to `e`. Call multiple times for multiple exchanges.
    pub fn allow_exchange(mut self, e: ExchangeId) -> Self {
        self.exchanges.insert(e);
        self
    }

    /// Allow subscriptions of `k`. Call multiple times for multiple kinds.
    pub fn allow_kind(mut self, k: Kind) -> Self {
        self.kinds.insert(k);
        self
    }

    /// Allow subscriptions to symbol `s` (user-input form, e.g. `"BTC-USDT"`).
    pub fn allow_symbol(mut self, s: impl Into<String>) -> Self {
        self.symbols.get_or_insert_with(HashSet::new).insert(s.into());
        self
    }

    /// Check whether `(exchange, kind, symbol)` passes the whitelist.
    /// Returns `Ok(())` or `Err(human-readable reason)`.
    pub(crate) fn check(
        &self,
        exchange: ExchangeId,
        kind: &Kind,
        symbol: &str,
    ) -> Result<(), String> {
        if !self.exchanges.is_empty() && !self.exchanges.contains(&exchange) {
            return Err(format!("exchange {exchange:?} not whitelisted"));
        }
        if !self.kinds.is_empty() && !self.kinds.contains(kind) {
            return Err(format!("kind {kind:?} not whitelisted"));
        }
        if let Some(syms) = &self.symbols {
            if !syms.contains(symbol) {
                return Err(format!("symbol {symbol} not whitelisted"));
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by [`ConsumerHandle`] operations.
#[derive(Debug, Error)]
pub enum QuotaError {
    /// The requested subscribe batch would push the consumer over its cap.
    /// Nothing in the batch was subscribed (atomic-or-nothing).
    #[error("subscription quota exceeded: have {have}, cap {cap}")]
    SubsCapExceeded { have: u32, cap: u32 },

    /// The consumer's REST token bucket is empty.
    /// `remaining_ms` is an upper-bound estimate until the next refill.
    #[error("REST rate limit: {remaining_ms}ms until next token")]
    RestRateLimit { remaining_ms: u64 },

    /// A `(exchange, kind, symbol)` tuple in the subscribe set was rejected
    /// by the consumer's whitelist. Nothing in the batch was subscribed.
    #[error("not in whitelist: {0}")]
    NotInWhitelist(String),

    /// An underlying station error (e.g. `StreamNotSupported`, IO).
    #[error(transparent)]
    Inner(#[from] StationError),
}

// ---------------------------------------------------------------------------
// Token bucket (REST rate limiting)
// ---------------------------------------------------------------------------

/// Simple fixed-window token bucket. Uses `std::time::Instant` for
/// monotonic time (cross-target: compiles on native and wasm32).
pub(crate) struct TokenBucket {
    capacity: u32,
    available: u32,
    refill_window: Duration,
    last_refill: Instant,
}

impl TokenBucket {
    pub(crate) fn new(capacity: u32, window: Duration) -> Self {
        Self {
            capacity,
            available: capacity,
            refill_window: window,
            last_refill: Instant::now(),
        }
    }

    /// Try to consume one token.
    ///
    /// Returns `Ok(())` on success. Returns `Err(remaining_ms)` when the
    /// bucket is empty; `remaining_ms` is how long until the next refill.
    pub(crate) fn try_consume(&mut self) -> Result<(), u64> {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);
        if elapsed >= self.refill_window {
            self.available = self.capacity;
            self.last_refill = now;
        }
        if self.available > 0 {
            self.available -= 1;
            Ok(())
        } else {
            let wait = self.refill_window.saturating_sub(elapsed);
            Err(wait.as_millis() as u64)
        }
    }

    /// Tokens available right now (without consuming).
    pub(crate) fn available(&self) -> u32 {
        self.available
    }
}

// ---------------------------------------------------------------------------
// ConsumerHandle
// ---------------------------------------------------------------------------

/// Opaque handle that enforces per-consumer quotas on a shared [`Station`].
///
/// Obtain via [`Station::register_consumer`]. The handle is `Send + Sync`
/// and may be wrapped in an `Arc` for sharing across tasks.
///
/// # Drop semantics
///
/// Dropping the `ConsumerHandle` drops the `Vec<MultiplexRef>` held
/// internally. Each `MultiplexRef::drop` calls `release_consumer` on
/// the underlying station, decrementing the per-`SeriesKey` refcount.
/// When the last holder drops, the multiplexer shuts down — same path
/// as dropping a `SubscriptionHandle`.
///
/// This is fully synchronous: `MultiplexRef::drop` performs only a
/// `fetch_sub` + optional `oneshot::send`, both non-blocking.
pub struct ConsumerHandle {
    pub(crate) station: Arc<StationInner>,
    pub(crate) quota: ConsumerQuota,
    pub(crate) rest_bucket: Arc<Mutex<Option<TokenBucket>>>,
    /// `(active_count, refs)`. The mutex guards both so cap check and
    /// ref-accumulation are a single critical section with zero race window.
    pub(crate) refs: Mutex<(u32, Vec<MultiplexRef>)>,
}

impl ConsumerHandle {
    /// Number of currently active subscriptions held by this consumer.
    pub async fn active_sub_count(&self) -> u32 {
        self.refs.lock().await.0
    }

    /// REST tokens available in the current window (without consuming).
    /// Returns `u32::MAX` when no REST quota is configured.
    pub async fn rest_tokens_available(&self) -> u32 {
        match self.rest_bucket.lock().await.as_ref() {
            Some(b) => b.available(),
            None => u32::MAX,
        }
    }

    /// Subscribe with quota enforcement.
    ///
    /// Pre-flight checks (whitelist, cap) run before any
    /// `acquire_or_spawn` call. If either fails, no subscriptions are
    /// made and an error is returned immediately.
    ///
    /// If a stream fails mid-batch (`StreamNotSupported`), any refs
    /// acquired so far in this batch are rolled back via
    /// `release_consumer` before returning `Err`. The consumer's
    /// active-sub counter never sees a partial increment.
    ///
    /// The returned [`SubscribeReport`]'s `handle` still works for
    /// `recv()` — the multiplex broadcast keeps emitting as long as any
    /// consumer holds a ref. The underlying `MultiplexRef`s are moved
    /// into this `ConsumerHandle`, so dropping the `ConsumerHandle`
    /// releases ALL refs even if the caller still holds the
    /// `SubscriptionHandle` for event recv.
    pub async fn subscribe(
        &self,
        set: SubscriptionSet,
    ) -> Result<SubscribeReport, QuotaError> {
        // Count requested streams.
        let n_new: u32 = set
            .entries
            .iter()
            .map(|e| e.streams.len() as u32)
            .sum();

        // --- Whitelist check (first; fail-fast before touching any counter) ---
        if let Some(wl) = &self.quota.whitelist {
            for entry in &set.entries {
                for s in &entry.streams {
                    let kind = s.to_kind();
                    if let Err(msg) = wl.check(entry.exchange, &kind, &entry.symbol) {
                        return Err(QuotaError::NotInWhitelist(msg));
                    }
                }
            }
        }

        // --- Cap check + acquire refs under a single lock ─────────────────
        // Lock covers both the cap check AND the append of new refs so there
        // is no window where another concurrent subscribe can race through.
        let mut guard = self.refs.lock().await;
        let current = guard.0;

        if let Some(cap) = self.quota.max_active_subs {
            if current.saturating_add(n_new) > cap {
                return Err(QuotaError::SubsCapExceeded {
                    have: current.saturating_add(n_new),
                    cap,
                });
            }
        }

        // --- Delegate to Station::subscribe (contains all WS/REST plumbing) ---
        let station = Station {
            inner: self.station.clone(),
        };
        let report = station.subscribe(set).await?;

        // Move MultiplexRefs from the SubscriptionHandle into our own vec.
        // The rx channel stays with the report's handle so the caller can
        // still recv() events. Dropping this ConsumerHandle releases the
        // refs even if the caller holds the SubscriptionHandle.
        let ok_count = report.ok.len() as u32;
        let report = report.take_refs_into(&mut guard.1);
        guard.0 = guard.0.saturating_add(ok_count);

        Ok(report)
    }

    /// Consume one REST token and return a [`Station`] view.
    ///
    /// The caller uses the returned `Station` to call REST methods (e.g.
    /// `hub().get_klines(...)`). This token-handoff design avoids ~75 LOC
    /// of forwarding boilerplate and is immune to future additions of REST
    /// methods.
    ///
    /// Returns `Err(QuotaError::RestRateLimit { remaining_ms })` when the
    /// token bucket is empty.
    pub async fn rest_gate(&self) -> Result<Station, QuotaError> {
        let mut bucket = self.rest_bucket.lock().await;
        if let Some(b) = bucket.as_mut() {
            b.try_consume()
                .map_err(|remaining_ms| QuotaError::RestRateLimit { remaining_ms })?;
        }
        Ok(Station { inner: self.station.clone() })
    }
}
