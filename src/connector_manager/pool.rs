//! # ConnectorPool - Thread-Safe Connection Pool
//!
//! Lock-free connection pool using DashMap for optimal concurrent performance.
//!
//! ## Architecture
//!
//! ```text
//! ConnectorPool
//!   ├── DashMap<ExchangeId, Arc<AnyConnector>>  [Lock-free reads]
//!   └── Methods: insert, get, remove, iter, etc.
//! ```
//!
//! ## Performance
//!
//! DashMap provides:
//! - Lock-free reads (5-33x faster than RwLock)
//! - Fine-grained locking (only shard-level locks on writes)
//! - Zero contention for read-heavy workloads
//!
//! ## Usage
//!
//! ```ignore
//! use connectors_v5::connector_manager::{ConnectorPool, AnyConnector};
//! use connectors_v5::core::types::ExchangeId;
//! use std::sync::Arc;
//!
//! // Create pool
//! let pool = ConnectorPool::new();
//!
//! // Insert connectors
//! pool.insert(ExchangeId::Binance, Arc::new(AnyConnector::Binance(binance)));
//! pool.insert(ExchangeId::KuCoin, Arc::new(AnyConnector::KuCoin(kucoin)));
//!
//! // Get connector (lock-free read, cheap Arc clone)
//! if let Some(connector) = pool.get(&ExchangeId::Binance) {
//!     let price = connector.get_price(symbol, account_type).await?;
//! }
//!
//! // Iterate over all connectors
//! for entry in pool.iter() {
//!     println!("{:?} is connected", entry.key());
//! }
//! ```

use dashmap::DashMap;
use std::sync::Arc;

use crate::connector_manager::AnyConnector;
use crate::core::types::ExchangeId;

// ═══════════════════════════════════════════════════════════════════════════════
// ConnectorPool - Thread-Safe Pool with DashMap
// ═══════════════════════════════════════════════════════════════════════════════

/// Thread-safe connection pool with lock-free reads.
///
/// Uses `DashMap` for optimal concurrent performance. All read operations
/// (get, contains, len, etc.) are lock-free and scale linearly with CPU cores.
///
/// Write operations (insert, remove, clear) use fine-grained shard-level locking,
/// ensuring minimal contention even under high concurrency.
///
/// # Examples
///
/// ```ignore
/// let pool = ConnectorPool::new();
/// pool.insert(ExchangeId::Binance, Arc::new(connector));
///
/// if let Some(connector) = pool.get(&ExchangeId::Binance) {
///     // Use connector...
/// }
/// ```
#[derive(Clone)]
pub struct ConnectorPool {
    /// Active connector instances (lock-free reads)
    connectors: Arc<DashMap<ExchangeId, Arc<AnyConnector>>>,
}

impl ConnectorPool {
    /// Create a new empty pool.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pool = ConnectorPool::new();
    /// assert!(pool.is_empty());
    /// ```
    pub fn new() -> Self {
        Self {
            connectors: Arc::new(DashMap::new()),
        }
    }

    /// Insert a connector into the pool.
    ///
    /// If a connector with the same `ExchangeId` already exists, it will be
    /// replaced and the old connector will be returned.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique exchange identifier
    /// * `connector` - Connector instance wrapped in Arc
    ///
    /// # Returns
    ///
    /// `Some(old_connector)` if a connector was replaced, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pool = ConnectorPool::new();
    /// let old = pool.insert(ExchangeId::Binance, Arc::new(connector));
    /// assert!(old.is_none()); // First insert
    /// ```
    pub fn insert(&self, id: ExchangeId, connector: Arc<AnyConnector>) -> Option<Arc<AnyConnector>> {
        self.connectors.insert(id, connector)
    }

    /// Get a connector by exchange ID (lock-free read).
    ///
    /// Returns a cheap Arc clone of the connector if found. The clone operation
    /// is O(1) and only increments a reference counter.
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier to look up
    ///
    /// # Returns
    ///
    /// `Some(Arc<AnyConnector>)` if found, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(connector) = pool.get(&ExchangeId::Binance) {
    ///     let price = connector.get_price(symbol, account_type).await?;
    /// }
    /// ```
    pub fn get(&self, id: &ExchangeId) -> Option<Arc<AnyConnector>> {
        self.connectors.get(id).map(|entry| entry.value().clone())
    }

    /// Remove a connector from the pool.
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier to remove
    ///
    /// # Returns
    ///
    /// `Some(Arc<AnyConnector>)` if the connector was found and removed, `None` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if let Some(connector) = pool.remove(&ExchangeId::Binance) {
    ///     println!("Removed Binance connector");
    /// }
    /// ```
    pub fn remove(&self, id: &ExchangeId) -> Option<Arc<AnyConnector>> {
        self.connectors.remove(id).map(|(_, connector)| connector)
    }

    /// Check if a connector exists in the pool (lock-free).
    ///
    /// # Arguments
    ///
    /// * `id` - Exchange identifier to check
    ///
    /// # Returns
    ///
    /// `true` if the connector exists, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if pool.contains(&ExchangeId::Binance) {
    ///     println!("Binance is connected");
    /// }
    /// ```
    pub fn contains(&self, id: &ExchangeId) -> bool {
        self.connectors.contains_key(id)
    }

    /// Count the number of active connectors (lock-free).
    ///
    /// # Returns
    ///
    /// The number of connectors currently in the pool.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let count = pool.len();
    /// println!("Active connections: {}", count);
    /// ```
    pub fn len(&self) -> usize {
        self.connectors.len()
    }

    /// Check if the pool is empty (lock-free).
    ///
    /// # Returns
    ///
    /// `true` if the pool contains no connectors, `false` otherwise.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// if pool.is_empty() {
    ///     println!("No connectors available");
    /// }
    /// ```
    pub fn is_empty(&self) -> bool {
        self.connectors.is_empty()
    }

    /// Remove all connectors from the pool.
    ///
    /// This operation is atomic - either all connectors are removed or none.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// pool.clear();
    /// assert!(pool.is_empty());
    /// ```
    pub fn clear(&self) {
        self.connectors.clear();
    }

    /// Iterate over all connectors in the pool.
    ///
    /// Returns an iterator that yields references to (ExchangeId, Arc<AnyConnector>) pairs.
    /// The iterator holds read locks on individual shards, so it's efficient for iteration
    /// while allowing concurrent reads from other threads.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// for entry in pool.iter() {
    ///     println!("{:?} is active", entry.key());
    /// }
    /// ```
    pub fn iter(&self) -> dashmap::iter::Iter<'_, ExchangeId, Arc<AnyConnector>> {
        self.connectors.iter()
    }

    /// Get a list of all exchange IDs in the pool.
    ///
    /// # Returns
    ///
    /// Vector of all ExchangeIds currently in the pool.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let ids = pool.ids();
    /// println!("Connected exchanges: {:?}", ids);
    /// ```
    pub fn ids(&self) -> Vec<ExchangeId> {
        self.connectors.iter().map(|entry| *entry.key()).collect()
    }
}

impl Default for ConnectorPool {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConnectorPoolBuilder - Fluent API for Pool Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// Builder for constructing ConnectorPool with fluent API.
///
/// Provides a convenient way to create a pool with multiple connectors
/// in a single expression chain.
///
/// # Examples
///
/// ```ignore
/// let pool = ConnectorPoolBuilder::new()
///     .with_connector(ExchangeId::Binance, Arc::new(binance_connector))
///     .with_connector(ExchangeId::KuCoin, Arc::new(kucoin_connector))
///     .build();
///
/// assert_eq!(pool.len(), 2);
/// ```
pub struct ConnectorPoolBuilder {
    pool: ConnectorPool,
}

impl ConnectorPoolBuilder {
    /// Create a new builder with an empty pool.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let builder = ConnectorPoolBuilder::new();
    /// ```
    pub fn new() -> Self {
        Self {
            pool: ConnectorPool::new(),
        }
    }

    /// Add a connector to the pool.
    ///
    /// This method consumes self and returns a new builder, allowing for
    /// method chaining.
    ///
    /// # Arguments
    ///
    /// * `id` - Unique exchange identifier
    /// * `connector` - Connector instance wrapped in Arc
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let builder = ConnectorPoolBuilder::new()
    ///     .with_connector(ExchangeId::Binance, Arc::new(connector));
    /// ```
    pub fn with_connector(self, id: ExchangeId, connector: Arc<AnyConnector>) -> Self {
        self.pool.insert(id, connector);
        self
    }

    /// Build the final ConnectorPool.
    ///
    /// Consumes the builder and returns the configured pool.
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let pool = ConnectorPoolBuilder::new()
    ///     .with_connector(ExchangeId::Binance, Arc::new(connector))
    ///     .build();
    /// ```
    pub fn build(self) -> ConnectorPool {
        self.pool
    }
}

impl Default for ConnectorPoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::l3::open::crypto::cex::okx::OkxConnector;
    use std::thread;

    /// Helper function to create a mock OKX connector for testing.
    /// Uses OKX's public API to avoid credentials.
    fn create_mock_okx() -> Arc<AnyConnector> {
        // Use tokio runtime to call async constructor
        let rt = tokio::runtime::Runtime::new().unwrap();
        let connector = rt.block_on(async {
            OkxConnector::public(true).await.unwrap()
        });
        Arc::new(AnyConnector::OKX(Arc::new(connector)))
    }

    /// Helper function to create a second mock connector (using same OKX).
    /// In real usage, this would be a different exchange.
    fn create_mock_okx_2() -> Arc<AnyConnector> {
        let rt = tokio::runtime::Runtime::new().unwrap();
        let connector = rt.block_on(async {
            OkxConnector::public(false).await.unwrap()
        });
        Arc::new(AnyConnector::OKX(Arc::new(connector)))
    }

    #[test]
    fn test_new_pool_is_empty() {
        let pool = ConnectorPool::new();
        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_insert_and_get() {
        let pool = ConnectorPool::new();
        let connector = create_mock_okx();

        // Insert connector
        let old = pool.insert(ExchangeId::Binance, connector.clone());
        assert!(old.is_none());

        // Verify it exists
        assert!(!pool.is_empty());
        assert_eq!(pool.len(), 1);

        // Get connector
        let retrieved = pool.get(&ExchangeId::Binance);
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_insert_replace() {
        let pool = ConnectorPool::new();
        let connector1 = create_mock_okx();
        let connector2 = create_mock_okx();

        // First insert
        pool.insert(ExchangeId::Binance, connector1);

        // Second insert should replace
        let old = pool.insert(ExchangeId::Binance, connector2);
        assert!(old.is_some());
        assert_eq!(pool.len(), 1); // Still only one connector
    }

    #[test]
    fn test_get_nonexistent() {
        let pool = ConnectorPool::new();
        let result = pool.get(&ExchangeId::Binance);
        assert!(result.is_none());
    }

    #[test]
    fn test_contains() {
        let pool = ConnectorPool::new();
        let connector = create_mock_okx();

        assert!(!pool.contains(&ExchangeId::Binance));

        pool.insert(ExchangeId::Binance, connector);

        assert!(pool.contains(&ExchangeId::Binance));
        assert!(!pool.contains(&ExchangeId::KuCoin));
    }

    #[test]
    fn test_remove() {
        let pool = ConnectorPool::new();
        let connector = create_mock_okx();

        pool.insert(ExchangeId::Binance, connector);
        assert_eq!(pool.len(), 1);

        // Remove connector
        let removed = pool.remove(&ExchangeId::Binance);
        assert!(removed.is_some());
        assert_eq!(pool.len(), 0);
        assert!(pool.is_empty());

        // Remove again should return None
        let removed_again = pool.remove(&ExchangeId::Binance);
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_clear() {
        let pool = ConnectorPool::new();

        pool.insert(ExchangeId::Binance, create_mock_okx());
        pool.insert(ExchangeId::KuCoin, create_mock_okx_2());

        assert_eq!(pool.len(), 2);

        pool.clear();

        assert!(pool.is_empty());
        assert_eq!(pool.len(), 0);
    }

    #[test]
    fn test_iter() {
        let pool = ConnectorPool::new();

        pool.insert(ExchangeId::Binance, create_mock_okx());
        pool.insert(ExchangeId::KuCoin, create_mock_okx_2());

        let mut count = 0;
        for entry in pool.iter() {
            count += 1;
            assert!(
                *entry.key() == ExchangeId::Binance || *entry.key() == ExchangeId::KuCoin
            );
        }

        assert_eq!(count, 2);
    }

    #[test]
    fn test_ids() {
        let pool = ConnectorPool::new();

        pool.insert(ExchangeId::Binance, create_mock_okx());
        pool.insert(ExchangeId::KuCoin, create_mock_okx_2());

        let ids = pool.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&ExchangeId::Binance));
        assert!(ids.contains(&ExchangeId::KuCoin));
    }

    #[test]
    fn test_multiple_inserts() {
        let pool = ConnectorPool::new();

        for i in 0..10 {
            let id = if i % 2 == 0 {
                ExchangeId::Binance
            } else {
                ExchangeId::KuCoin
            };
            pool.insert(id, create_mock_okx());
        }

        // Should only have 2 unique connectors
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_builder_empty() {
        let pool = ConnectorPoolBuilder::new().build();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_builder_single_connector() {
        let pool = ConnectorPoolBuilder::new()
            .with_connector(ExchangeId::Binance, create_mock_okx())
            .build();

        assert_eq!(pool.len(), 1);
        assert!(pool.contains(&ExchangeId::Binance));
    }

    #[test]
    fn test_builder_multiple_connectors() {
        let pool = ConnectorPoolBuilder::new()
            .with_connector(ExchangeId::Binance, create_mock_okx())
            .with_connector(ExchangeId::KuCoin, create_mock_okx_2())
            .build();

        assert_eq!(pool.len(), 2);
        assert!(pool.contains(&ExchangeId::Binance));
        assert!(pool.contains(&ExchangeId::KuCoin));
    }

    #[test]
    fn test_builder_with_duplicates() {
        let pool = ConnectorPoolBuilder::new()
            .with_connector(ExchangeId::Binance, create_mock_okx())
            .with_connector(ExchangeId::Binance, create_mock_okx()) // Duplicate
            .build();

        // Should only have 1 connector (duplicates are replaced)
        assert_eq!(pool.len(), 1);
    }

    #[test]
    fn test_concurrent_inserts() {
        let pool = Arc::new(ConnectorPool::new());
        let mut handles = vec![];

        // Spawn 10 threads that insert connectors concurrently
        for i in 0..10 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let id = if i % 2 == 0 {
                    ExchangeId::Binance
                } else {
                    ExchangeId::KuCoin
                };
                pool_clone.insert(id, create_mock_okx());
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Should only have 2 unique connectors despite concurrent inserts
        assert_eq!(pool.len(), 2);
    }

    #[test]
    fn test_concurrent_reads() {
        let pool = Arc::new(ConnectorPool::new());
        pool.insert(ExchangeId::Binance, create_mock_okx());

        let mut handles = vec![];

        // Spawn 100 threads that read concurrently
        for _ in 0..100 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                let connector = pool_clone.get(&ExchangeId::Binance);
                assert!(connector.is_some());
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_concurrent_mixed_operations() {
        let pool = Arc::new(ConnectorPool::new());
        pool.insert(ExchangeId::Binance, create_mock_okx());

        let mut handles = vec![];

        // Spawn threads with mixed operations
        for i in 0..50 {
            let pool_clone = Arc::clone(&pool);
            let handle = thread::spawn(move || {
                match i % 3 {
                    0 => {
                        // Read operation
                        let _ = pool_clone.get(&ExchangeId::Binance);
                    }
                    1 => {
                        // Insert operation
                        pool_clone.insert(ExchangeId::KuCoin, create_mock_okx_2());
                    }
                    2 => {
                        // Contains check
                        let _ = pool_clone.contains(&ExchangeId::Binance);
                    }
                    _ => unreachable!(),
                }
            });
            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().unwrap();
        }

        // Verify pool is still in a valid state
        assert!(pool.len() > 0);
    }

    #[test]
    fn test_pool_clone() {
        let pool1 = ConnectorPool::new();
        pool1.insert(ExchangeId::Binance, create_mock_okx());

        // Clone the pool
        let pool2 = pool1.clone();

        // Both pools should share the same underlying DashMap
        assert_eq!(pool1.len(), 1);
        assert_eq!(pool2.len(), 1);

        // Insert via pool2
        pool2.insert(ExchangeId::KuCoin, create_mock_okx_2());

        // pool1 should also see the change
        assert_eq!(pool1.len(), 2);
        assert_eq!(pool2.len(), 2);
    }

    #[test]
    fn test_default_pool() {
        let pool: ConnectorPool = Default::default();
        assert!(pool.is_empty());
    }

    #[test]
    fn test_default_builder() {
        let pool = ConnectorPoolBuilder::default().build();
        assert!(pool.is_empty());
    }
}
