//! # WebSocketPool — Thread-safe per-(exchange, account_type) WS pool
//!
//! Many exchanges (Binance, Bybit) have separate WS endpoints for spot vs
//! futures. The pool keys on `(ExchangeId, AccountType)` for that reason.

use dashmap::DashMap;
use std::sync::Arc;

use crate::core::traits::WebSocketConnector;
use crate::core::types::{AccountType, ExchangeId};

/// Thread-safe WebSocket connection pool keyed on `(ExchangeId, AccountType)`.
///
/// Mirrors `ConnectorPool` but for `Arc<dyn WebSocketConnector>` — REST and WS
/// pools are kept separate because they have different lifecycles and
/// per-account-type WS endpoints (e.g. Binance spot vs futures WS URLs differ).
///
/// # Examples
///
/// ```ignore
/// let pool = WebSocketPool::new();
/// let ws = ConnectorFactory::create_websocket(ExchangeId::Binance, AccountType::Spot, false).await?;
/// pool.insert(ExchangeId::Binance, AccountType::Spot, ws);
/// if let Some(ws) = pool.get(ExchangeId::Binance, AccountType::Spot) {
///     ws.connect(AccountType::Spot).await?;
/// }
/// ```
#[derive(Clone)]
pub struct WebSocketPool {
    sockets: Arc<DashMap<(ExchangeId, AccountType), Arc<dyn WebSocketConnector>>>,
}

impl WebSocketPool {
    /// Create a new empty pool.
    pub fn new() -> Self {
        Self {
            sockets: Arc::new(DashMap::new()),
        }
    }

    /// Insert a WebSocket connector into the pool.
    ///
    /// If an entry already exists for `(id, account_type)`, it is replaced and
    /// the previous value is returned.
    pub fn insert(
        &self,
        id: ExchangeId,
        account_type: AccountType,
        socket: Arc<dyn WebSocketConnector>,
    ) -> Option<Arc<dyn WebSocketConnector>> {
        self.sockets.insert((id, account_type), socket)
    }

    /// Get a WebSocket connector by `(exchange, account_type)` (lock-free read).
    pub fn get(
        &self,
        id: ExchangeId,
        account_type: AccountType,
    ) -> Option<Arc<dyn WebSocketConnector>> {
        self.sockets.get(&(id, account_type)).map(|e| e.value().clone())
    }

    /// Remove a WebSocket connector from the pool.
    pub fn remove(
        &self,
        id: ExchangeId,
        account_type: AccountType,
    ) -> Option<Arc<dyn WebSocketConnector>> {
        self.sockets.remove(&(id, account_type)).map(|(_, v)| v)
    }

    /// Check whether the pool contains an entry for `(exchange, account_type)`.
    pub fn contains(&self, id: ExchangeId, account_type: AccountType) -> bool {
        self.sockets.contains_key(&(id, account_type))
    }

    /// Number of entries in the pool.
    pub fn len(&self) -> usize {
        self.sockets.len()
    }

    /// Returns `true` if the pool contains no entries.
    pub fn is_empty(&self) -> bool {
        self.sockets.is_empty()
    }

    /// Remove all entries from the pool.
    pub fn clear(&self) {
        self.sockets.clear();
    }
}

impl Default for WebSocketPool {
    fn default() -> Self {
        Self::new()
    }
}
