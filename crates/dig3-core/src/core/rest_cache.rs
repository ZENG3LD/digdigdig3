//! Generic TTL cache for REST responses (instrument metadata, exchange info, etc).
//!
//! Wraps any async fn, caches by key. Per-key TTL. Concurrent-safe via [`DashMap`].
//!
//! # Recommended TTLs
//!
//! | Data | TTL |
//! |------|-----|
//! | Instrument list / exchange info | 1 hour |
//! | Symbol precision / lot size | 1 hour |
//! | Server time | 30 sec |
//! | Recent ticker (REST polling fallback) | 10 sec |
//! | Funding rate | 5 min |
//! | Open interest | 30 sec |
//!
//! These are suggestions — consumers choose their own TTL at construction time.
//!
//! # Example
//!
//! ```rust,no_run
//! # use std::time::Duration;
//! # use digdigdig3_core::core::rest_cache::RestCache;
//! # async fn example() -> Result<(), String> {
//! let cache: RestCache<String, String> = RestCache::new(Duration::from_secs(3600));
//! let info = cache.get_or_fetch("binance".to_string(), || async {
//!     Ok::<String, String>("exchange_info_payload".to_string())
//! }).await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Single-flight note
//!
//! `get_or_fetch` is **not single-flight**: if two tasks call it concurrently on the same
//! missing key, both will invoke the loader. The second result simply overwrites the first.
//! For most REST metadata use-cases (slow changing, cheap on race) this is acceptable.
//! If strict single-flight is required, layer a `tokio::sync::Mutex` per key on top.

use dashmap::DashMap;
use std::future::Future;
use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
struct Entry<V> {
    value: V,
    inserted_at: Instant,
    ttl: Duration,
}

impl<V> Entry<V> {
    fn is_fresh(&self) -> bool {
        self.inserted_at.elapsed() < self.ttl
    }
}

/// Concurrent TTL cache.
///
/// Use [`RestCache::get_or_fetch`] to look up a value, fetching from the loader if
/// missing or expired. The loader is async and returns `Result<V, E>`.
///
/// Clone is cheap — the internal map is `Arc`-wrapped.
pub struct RestCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone + Send + Sync,
{
    inner: Arc<DashMap<K, Entry<V>>>,
    default_ttl: Duration,
}

impl<K, V> RestCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone + Send + Sync,
{
    /// Create a new cache with the given default TTL applied by [`RestCache::insert`].
    pub fn new(default_ttl: Duration) -> Self {
        Self {
            inner: Arc::new(DashMap::new()),
            default_ttl,
        }
    }

    /// Get cached value if still fresh, else `None`.
    pub fn get(&self, key: &K) -> Option<V> {
        let entry = self.inner.get(key)?;
        if entry.is_fresh() {
            Some(entry.value.clone())
        } else {
            None
        }
    }

    /// Insert with the default TTL configured at construction.
    pub fn insert(&self, key: K, value: V) {
        self.insert_with_ttl(key, value, self.default_ttl);
    }

    /// Insert with an explicit TTL.
    pub fn insert_with_ttl(&self, key: K, value: V, ttl: Duration) {
        self.inner.insert(
            key,
            Entry {
                value,
                inserted_at: Instant::now(),
                ttl,
            },
        );
    }

    /// Get or fetch.
    ///
    /// Returns the cached value if fresh. Otherwise calls `loader`, caches the result
    /// with the default TTL, and returns it.
    ///
    /// See the module-level doc for single-flight semantics (short: not single-flight).
    pub async fn get_or_fetch<F, Fut, E>(&self, key: K, loader: F) -> Result<V, E>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<V, E>>,
    {
        if let Some(v) = self.get(&key) {
            return Ok(v);
        }
        let value = loader().await?;
        self.insert(key, value.clone());
        Ok(value)
    }

    /// Manually expire one key (next `get` or `get_or_fetch` will reload).
    pub fn invalidate(&self, key: &K) {
        self.inner.remove(key);
    }

    /// Remove all entries.
    pub fn clear(&self) {
        self.inner.clear();
    }

    /// Number of entries (including expired ones not yet swept).
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Returns `true` if no entries are present.
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Eviction sweep — removes all expired entries.
    ///
    /// Returns the number of entries removed. Call periodically if the cache can
    /// accumulate many distinct keys (e.g. per-symbol caches).
    pub fn sweep_expired(&self) -> usize {
        let before = self.inner.len();
        self.inner.retain(|_, entry| entry.is_fresh());
        before - self.inner.len()
    }
}

impl<K, V> Clone for RestCache<K, V>
where
    K: Eq + Hash + Clone,
    V: Clone + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            default_ttl: self.default_ttl,
        }
    }
}
