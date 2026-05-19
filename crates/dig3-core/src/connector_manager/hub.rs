//! # ExchangeHub — unified single-entry pool for REST + WS surfaces.
//!
//! Bundles ConnectorPool (REST) and WebSocketPool (WS) behind one entry.
//! Consumer connects ONCE per exchange (with chosen account_types) and reaches
//! both surfaces through the same hub:
//!
//! ```ignore
//! let hub = ExchangeHub::new();
//! hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;
//!
//! let rest = hub.rest(ExchangeId::Binance).unwrap();
//! let ticker = rest.get_ticker(symbol, AccountType::Spot).await?;
//!
//! let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();
//! ws.connect(AccountType::Spot).await?;
//!
//! let caps = hub.capabilities(ExchangeId::Binance).unwrap();
//! ```

use std::sync::Arc;

use crate::connector_manager::{ConnectorFactory, ConnectorPool, WebSocketPool};
use crate::core::traits::{CoreConnector, WebSocketConnector};
use crate::core::types::{AccountType, ConnectorCapabilities, ExchangeError, ExchangeId, ExchangeResult};

/// Unified holder of REST and WS connector pools.
///
/// Wraps `ConnectorPool` (REST) and `WebSocketPool` (WS) behind one entry point.
/// `clone()` is O(1) — both underlying pools use `Arc<DashMap<…>>` internally.
#[derive(Clone, Default)]
pub struct ExchangeHub {
    rest: ConnectorPool,
    ws: WebSocketPool,
}

impl ExchangeHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    // ── REST methods ──────────────────────────────────────────────────────

    /// Connect ONLY the public REST connector for an exchange.
    pub async fn connect_public(&self, id: ExchangeId, testnet: bool) -> ExchangeResult<()> {
        let conn = ConnectorFactory::create_public(id, testnet).await?;
        self.rest.insert(id, conn);
        Ok(())
    }

    /// Get REST surface for an exchange.
    pub fn rest(&self, id: ExchangeId) -> Option<Arc<dyn CoreConnector>> {
        self.rest.get(&id)
    }

    // ── WS methods ────────────────────────────────────────────────────────

    /// Connect ONLY the WebSocket for a specific (exchange, account_type).
    pub async fn connect_websocket(
        &self,
        id: ExchangeId,
        account_type: AccountType,
        testnet: bool,
    ) -> ExchangeResult<()> {
        let ws = ConnectorFactory::create_websocket(id, account_type, testnet).await?;
        self.ws.insert(id, account_type, ws);
        Ok(())
    }

    /// Get WS surface for an exchange + account_type.
    pub fn ws(&self, id: ExchangeId, account_type: AccountType) -> Option<Arc<dyn WebSocketConnector>> {
        self.ws.get(id, account_type)
    }

    // ── Combined ──────────────────────────────────────────────────────────

    /// Wire REST + WS for all listed account_types in one call.
    ///
    /// REST connection is required (fails if it errors). WS connections are
    /// best-effort — if a particular (id, account_type) doesn't have native
    /// WS support, that one is silently skipped and the REST half remains.
    pub async fn connect_full(
        &self,
        id: ExchangeId,
        account_types: &[AccountType],
        testnet: bool,
    ) -> ExchangeResult<()> {
        let conn = ConnectorFactory::create_public(id, testnet).await?;
        self.rest.insert(id, conn);

        for &at in account_types {
            if let Ok(ws) = ConnectorFactory::create_websocket(id, at, testnet).await {
                self.ws.insert(id, at, ws);
            }
        }
        Ok(())
    }

    /// Wire REST + WS for all listed account_types, then refuse if the connector
    /// has no `ValidationStamp` (i.e. it was never smoke-tested with live data).
    ///
    /// Use `connect_full` to bypass this guard (e.g. for untested/auth-gated connectors).
    pub async fn connect_full_validated(
        &self,
        id: ExchangeId,
        account_types: &[AccountType],
        testnet: bool,
    ) -> ExchangeResult<()> {
        self.connect_full(id, account_types, testnet).await?;
        let conn = self.rest.get(&id).ok_or_else(|| {
            ExchangeError::NotValidated(format!(
                "{:?} connected but rest() returned None — internal error",
                id
            ))
        })?;
        if conn.validation_status().is_none() {
            // Roll back so the hub isn't left in a half-connected state
            self.shutdown(id);
            return Err(ExchangeError::NotValidated(format!(
                "{:?} has no ValidationStamp — refusing connect_full_validated; \
                 use connect_full() to bypass",
                id
            )));
        }
        Ok(())
    }

    /// Convenience: capabilities of the REST entry. None if exchange not connected.
    pub fn capabilities(&self, id: ExchangeId) -> Option<ConnectorCapabilities> {
        self.rest.get(&id).map(|c| c.capabilities())
    }

    /// List exchange IDs currently connected (REST side).
    pub fn ids(&self) -> Vec<ExchangeId> {
        self.rest.ids()
    }

    /// REST entry count.
    pub fn len_rest(&self) -> usize {
        self.rest.len()
    }

    /// WS entry count.
    pub fn len_ws(&self) -> usize {
        self.ws.len()
    }

    /// Total entries (REST + WS).
    pub fn len(&self) -> usize {
        self.rest.len() + self.ws.len()
    }

    /// Returns `true` if neither REST nor WS pools contain any entries.
    pub fn is_empty(&self) -> bool {
        self.rest.is_empty() && self.ws.is_empty()
    }

    /// Disconnect everything for an exchange — REST + all WS account_types.
    ///
    /// Sweeps all known `AccountType` variants for the WS pool since
    /// `WebSocketPool` doesn't expose iteration.
    pub fn shutdown(&self, id: ExchangeId) {
        self.rest.remove(&id);
        for at in [
            AccountType::Spot,
            AccountType::Margin,
            AccountType::FuturesCross,
            AccountType::FuturesIsolated,
            AccountType::Earn,
            AccountType::Lending,
            AccountType::Options,
            AccountType::Convert,
        ] {
            self.ws.remove(id, at);
        }
    }

    /// Clear all REST and WS entries.
    pub fn clear(&self) {
        self.rest.clear();
        self.ws.clear();
    }

    /// Returns `true` if the exchange has a REST entry in the hub.
    pub fn is_connected(&self, id: ExchangeId) -> bool {
        self.rest.contains(&id)
    }

    /// List exchange IDs with REST entries (alias of `ids()`).
    pub fn list_connected(&self) -> Vec<ExchangeId> {
        self.rest.ids()
    }
}
