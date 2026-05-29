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

use dashmap::DashMap;

use crate::connector_manager::{ConnectorFactory, ConnectorPool, WebSocketPool};
use crate::core::traits::{CoreConnector, WebSocketConnector};
use crate::core::types::{AccountType, ConnectorCapabilities, ExchangeError, ExchangeId, ExchangeResult};

/// Unified holder of REST and WS connector pools.
///
/// Wraps `ConnectorPool` (REST) and `WebSocketPool` (WS) behind one entry point.
/// `clone()` is O(1) — both underlying pools use `Arc<DashMap<…>>` internally.
#[derive(Clone)]
pub struct ExchangeHub {
    rest: ConnectorPool,
    ws: WebSocketPool,
    /// Per-exchange REST base URL overrides for proxy / Path-B routing.
    ///
    /// When set, connectors that respect this map should substitute the
    /// override for their native base URL.  Use `set_rest_base_override` /
    /// `get_rest_base_override` to read and write.
    rest_overrides: Arc<DashMap<ExchangeId, String>>,
}

impl Default for ExchangeHub {
    fn default() -> Self {
        Self {
            rest: ConnectorPool::default(),
            ws: WebSocketPool::default(),
            rest_overrides: Arc::new(DashMap::new()),
        }
    }
}

impl ExchangeHub {
    /// Create a new empty hub.
    pub fn new() -> Self {
        Self::default()
    }

    // ── REST methods ──────────────────────────────────────────────────────

    /// Connect ONLY the public REST connector for an exchange.
    pub async fn connect_public(&self, id: ExchangeId, testnet: bool) -> ExchangeResult<()> {
        let override_url = self.rest_overrides.get(&id).map(|v| v.clone());
        let conn = ConnectorFactory::create_public(id, testnet, override_url).await?;
        self.rest.insert(id, conn);
        Ok(())
    }

    /// Get REST surface for an exchange.
    pub fn rest(&self, id: ExchangeId) -> Option<Arc<dyn CoreConnector>> {
        self.rest.get(&id)
    }

    /// Set a REST base URL override for a specific exchange.
    ///
    /// Connectors that respect this call `hub.get_rest_base_override(id)` and
    /// substitute the result for their native endpoint base URL, enabling
    /// Path-B proxy routing (e.g. via a local relay or gateway).
    ///
    /// Passing an empty string removes the override (equivalent to
    /// `clear_rest_base_override`).
    pub fn set_rest_base_override(&self, id: ExchangeId, url: String) {
        if url.is_empty() {
            self.rest_overrides.remove(&id);
        } else {
            self.rest_overrides.insert(id, url);
        }
    }

    /// Remove the REST base URL override for a specific exchange.
    pub fn clear_rest_base_override(&self, id: ExchangeId) {
        self.rest_overrides.remove(&id);
    }

    /// Return the REST base URL override for an exchange, if one has been set.
    pub fn get_rest_base_override(&self, id: ExchangeId) -> Option<String> {
        self.rest_overrides.get(&id).map(|v| v.clone())
    }

    // ── WS methods ────────────────────────────────────────────────────────

    /// Connect ONLY the WebSocket for a specific (exchange, account_type).
    ///
    /// On native: full factory — supports all 47 exchanges.
    /// On wasm32: browser subset — Binance, Bybit, OKX, HyperLiquid, Gemini,
    /// CryptoCom, Bitfinex, BingX, Upbit, Dydx, Lighter (11 venues,
    /// all via UniversalWsTransport+web-sys).
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
    /// best-effort — if a particular (id, account_type) doesn't support WS
    /// on the current target, that one is silently skipped and the REST half
    /// remains. On wasm32 11 venues support WS (Binance/Bybit/OKX/HyperLiquid/
    /// Gemini/CryptoCom/Bitfinex/BingX/Upbit/Dydx/Lighter).
    pub async fn connect_full(
        &self,
        id: ExchangeId,
        account_types: &[AccountType],
        testnet: bool,
    ) -> ExchangeResult<()> {
        let override_url = self.rest_overrides.get(&id).map(|v| v.clone());
        let conn = ConnectorFactory::create_public(id, testnet, override_url).await?;
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
