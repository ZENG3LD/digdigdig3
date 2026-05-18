//! Shared server state — hub + storage + bus + startup time.

use std::sync::Arc;
use std::time::Instant;

use crate::connector_manager::ExchangeHub;
use crate::core::storage::{StorageConfig, StorageManager};
use crate::server::bus::EventBus;

/// Shared state injected into every gRPC service impl.
#[derive(Clone)]
pub struct ServerState {
    pub hub: Arc<ExchangeHub>,
    pub storage: Arc<StorageManager>,
    pub bus: EventBus,
    pub started_at: Arc<Instant>,
}

impl ServerState {
    pub fn new(hub: ExchangeHub, storage: StorageManager) -> Self {
        Self {
            hub: Arc::new(hub),
            storage: Arc::new(storage),
            bus: EventBus::new(),
            started_at: Arc::new(Instant::now()),
        }
    }

    /// Uptime in whole seconds.
    pub fn uptime_secs(&self) -> i64 {
        self.started_at.elapsed().as_secs() as i64
    }

    /// Create a `StorageManager` from a `StorageConfig`.
    pub fn build_storage(cfg: StorageConfig) -> std::io::Result<StorageManager> {
        StorageManager::new(cfg)
    }
}
