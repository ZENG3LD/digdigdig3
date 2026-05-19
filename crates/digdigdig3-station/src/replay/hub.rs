//! ReplayHub — drop-in replacement for `ExchangeHub` reading from `StorageManager`.
//!
//! Mirrors the `ExchangeHub` API surface:
//! - `connect_full(id, accounts, testnet)` — "connects" replay streams for each AccountType
//! - `ws(id, account_type)` — returns a `WebSocketConnector` backed by stored events
//! - `shutdown(id)` — drops all WS handles for an exchange
//!
//! Consumers use identical code to live operation; only the constructor differs.

use std::path::PathBuf;
use std::sync::Arc;

use dashmap::DashMap;

use crate::storage::{StorageConfig, StorageManager};
use digdigdig3::core::traits::WebSocketConnector;
use digdigdig3::core::types::{AccountType, ExchangeError, ExchangeId};

use super::rate::ReplayRate;
use super::ws::ReplayWebSocket;

// ── ReplayConfig ──────────────────────────────────────────────────────────────

/// Configuration for a `ReplayHub` instance.
#[derive(Debug, Clone)]
pub struct ReplayConfig {
    /// Root directory of the stored event log (matches `StorageConfig::root`).
    pub storage_root: PathBuf,
    /// Emission rate: `Realtime`, `Accelerated(x)`, or `Instant`.
    pub rate: ReplayRate,
    /// Inclusive start of replay window (UTC ms). `None` = from beginning of storage.
    pub from_ms: Option<i64>,
    /// Inclusive end of replay window (UTC ms). `None` = to end of storage.
    pub to_ms: Option<i64>,
}

// ── ReplayHub ─────────────────────────────────────────────────────────────────

/// Drop-in `ExchangeHub` replacement that replays stored events.
///
/// # Example
///
/// ```no_run
/// # use std::path::PathBuf;
/// # use digdigdig3_station::replay::{ReplayHub, ReplayConfig, ReplayRate};
/// # use digdigdig3::core::types::{AccountType, ExchangeId, SubscriptionRequest, Symbol};
/// # use futures_util::StreamExt;
/// # #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = ReplayConfig {
///     storage_root: PathBuf::from("./data/events"),
///     rate: ReplayRate::Accelerated(10.0),
///     from_ms: None,
///     to_ms: None,
/// };
/// let hub = ReplayHub::new(config).await?;
/// hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;
/// let ws = hub.ws(ExchangeId::Binance, AccountType::Spot).unwrap();
/// ws.subscribe(SubscriptionRequest::ticker(Symbol::new("BTC", "USDT"))).await?;
/// let mut stream = ws.event_stream();
/// while let Some(ev) = stream.next().await {
///     println!("{:?}", ev?);
/// }
/// # Ok(())
/// # }
/// ```
pub struct ReplayHub {
    storage: Arc<StorageManager>,
    config: ReplayConfig,
    /// Key: `(ExchangeId, AccountType)` → `ReplayWebSocket`.
    ws_pool: DashMap<(ExchangeId, AccountType), Arc<dyn WebSocketConnector>>,
}

impl ReplayHub {
    /// Create a new `ReplayHub` from `config`.
    ///
    /// Initialises the underlying `StorageManager` (creates `storage_root` if absent).
    pub async fn new(config: ReplayConfig) -> Result<Self, ExchangeError> {
        let storage_config = StorageConfig {
            root: config.storage_root.clone(),
            // Replay should not auto-delete historical data.
            default_retention_days: u32::MAX,
            orderbook_snapshot_interval_secs: 0,
        };
        let storage = StorageManager::new(storage_config)
            .map_err(|e| ExchangeError::Network(format!("replay storage init: {e}")))?;
        Ok(Self {
            storage: Arc::new(storage),
            config,
            ws_pool: DashMap::new(),
        })
    }

    /// Construct a `ReplayWebSocket` for each `(id, account)` pair.
    ///
    /// The `testnet` flag is accepted for API compatibility but ignored — replay
    /// reads whatever is in storage regardless of network origin.
    pub async fn connect_full(
        &self,
        id: ExchangeId,
        accounts: &[AccountType],
        _testnet: bool,
    ) -> Result<(), ExchangeError> {
        for &acct in accounts {
            let ws = ReplayWebSocket::new(id, acct, self.storage.clone(), self.config.clone());
            // Auto-connect so consumers don't need to call `.connect()` separately.
            ws.connect(acct)
                .await
                .map_err(|e| ExchangeError::Network(format!("replay connect: {e}")))?;
            self.ws_pool.insert((id, acct), Arc::new(ws));
        }
        Ok(())
    }

    /// Return the `WebSocketConnector` for `(id, acct)`, if connected.
    pub fn ws(&self, id: ExchangeId, acct: AccountType) -> Option<Arc<dyn WebSocketConnector>> {
        self.ws_pool.get(&(id, acct)).map(|v| Arc::clone(&v))
    }

    /// Drop all WS handles for `id`.
    pub fn shutdown(&self, id: ExchangeId) {
        self.ws_pool.retain(|(eid, _), _| *eid != id);
    }

    /// List all `(ExchangeId, AccountType)` pairs currently registered.
    pub fn list_connected(&self) -> Vec<(ExchangeId, AccountType)> {
        self.ws_pool.iter().map(|r| *r.key()).collect()
    }

    /// Check whether `(id, acct)` is registered.
    pub fn is_connected(&self, id: ExchangeId, acct: AccountType) -> bool {
        self.ws_pool.contains_key(&(id, acct))
    }
}
