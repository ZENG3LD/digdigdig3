use std::path::PathBuf;

use crate::{PersistenceConfig, Result, Station};

/// Fluent builder for [`Station`].
///
/// Phase 1: only storage root + (optional) persistence toggle are wired.
/// Cache / multiplex / reconnect overrides land in step 5+.
#[derive(Debug)]
pub struct StationBuilder {
    pub(crate) storage_root: PathBuf,
    pub(crate) persistence: PersistenceConfig,
}

impl Default for StationBuilder {
    fn default() -> Self {
        // Resolution order: explicit `.storage_root(...)` call > DIG3_STORAGE_ROOT env >
        // `./dig3_storage`. The CLI also honors the env var explicitly so the
        // user can override without code changes.
        let env_root = std::env::var_os("DIG3_STORAGE_ROOT").map(PathBuf::from);
        Self {
            storage_root: env_root.unwrap_or_else(|| PathBuf::from("./dig3_storage")),
            persistence: PersistenceConfig::default(),
        }
    }
}

impl StationBuilder {
    pub fn new() -> Self {
        Self::default()
    }

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

    pub async fn build(self) -> Result<Station> {
        Station::from_builder(self).await
    }
}
