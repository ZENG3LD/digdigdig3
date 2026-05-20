use std::path::PathBuf;

use crate::{PersistenceConfig, Result, Station};

/// Fluent builder for [`Station`].
#[derive(Debug)]
pub struct StationBuilder {
    pub(crate) storage_root: PathBuf,
    pub(crate) persistence: PersistenceConfig,
    pub(crate) warm_start: usize,
}

impl Default for StationBuilder {
    fn default() -> Self {
        // Storage root resolution: explicit `.storage_root(...)` > DIG3_STORAGE_ROOT
        // env > `./dig3_storage`. The CLI also honors the env explicitly.
        let env_root = std::env::var_os("DIG3_STORAGE_ROOT").map(PathBuf::from);
        Self {
            storage_root: env_root.unwrap_or_else(|| PathBuf::from("./dig3_storage")),
            persistence: PersistenceConfig::default(),
            warm_start: 0,
        }
    }
}

impl StationBuilder {
    pub fn new() -> Self { Self::default() }

    /// Override the root directory under which all Station-managed artefacts
    /// (trades, bars, snapshots, indexes) are written.
    pub fn storage_root(mut self, root: impl Into<PathBuf>) -> Self {
        self.storage_root = root.into();
        self
    }

    /// Configure trade/bar/snapshot persistence.
    pub fn persistence(mut self, cfg: PersistenceConfig) -> Self {
        self.persistence = cfg;
        self
    }

    /// Number of most-recent points to emit from disk on subscribe BEFORE live
    /// stream takes over. 0 disables warm-start. Acts as the in-memory
    /// series capacity too.
    pub fn warm_start(mut self, n: usize) -> Self {
        self.warm_start = n;
        self
    }

    pub async fn build(self) -> Result<Station> {
        Station::from_builder(self).await
    }
}
