//! `digdigdig3-station` — Layer 2 of the digdigdig3 workspace.
//!
//! High-level consumer-facing builder over [`digdigdig3::connector_manager::ExchangeHub`].
//! See `docs/plans/station-architecture.md` for design, `docs/plans/station-phase-1-plan.md`
//! for the phase-by-phase implementation roadmap.
//!
//! Phase 1 scope (current): skeleton only. Modules below are stubs.

pub mod backfill;
pub mod builder;
pub mod cache;
pub mod data;
pub(crate) mod derived;
pub mod error;
pub mod persistence;
pub mod quota;
pub mod series;
pub mod station;
pub mod subscription;

// Modules moved from dig3-core (persistence/cache/cure/OB concerns belong in station)
pub mod orderbook;
pub mod rest_cache;

#[cfg(feature = "reconnect")]
pub mod reconnect;

// native-only — file I/O, sled, zstd
#[cfg(not(target_arch = "wasm32"))]
pub mod storage;

// cure/replay depend on StorageManager (sled + tokio::fs) — native-only.
#[cfg(not(target_arch = "wasm32"))]
pub mod cure;

#[cfg(not(target_arch = "wasm32"))]
pub mod replay;

// polling + gap_heal: REST-based; work on wasm via rest_override (Workstream A).
// spawn_poller uses cfg-split tokio::spawn / wasm_bindgen_futures::spawn_local.
pub(crate) mod polling;

pub mod gap_heal;

pub use builder::StationBuilder;
pub use cache::{ticker_cache, CacheConfig, TickerKey};
pub use error::{Result, StationError};
pub use persistence::PersistenceConfig;
pub use series::{DataPoint, Kind, Series, SeriesKey, SharedSeriesMap};
pub use quota::{ConsumerHandle, ConsumerQuota, ConsumerWhitelist, QuotaError};
pub use station::Station;
pub use subscription::{
    Event, FailedStream, Stream, SubscribeReport, SubscriptionHandle, SubscriptionSet,
};

// DiskStore is available on both targets (native: std::fs; wasm32: OPFS).
pub use series::DiskStore;

// PollSpec, PollSource, GapHealConfig: available on both targets.
// polling is now un-gated; gap_heal is un-gated.
pub use series::PollSpec;
pub use polling::PollSource;
pub use gap_heal::GapHealConfig;

// Re-exports for moved modules (mirror what core used to expose)
#[cfg(not(target_arch = "wasm32"))]
pub use storage::{EventLog, EventLogIter, EventRecord, StorageManager, StorageConfig, StreamKey};

#[cfg(not(target_arch = "wasm32"))]
pub use replay::{ReplayHub, ReplayConfig, ReplayRate};

pub use orderbook::{OrderBookTracker, OrderBookError};
pub use rest_cache::RestCache;

#[cfg(not(target_arch = "wasm32"))]
pub use cure::{
    IntegrityChecker, IntegrityReport,
    Deduper,
    GapDetector, GapInfo,
    RepairPipeline, RepairReport,
};

// Re-export common core types so consumers can build a SubscriptionSet without
// pulling `digdigdig3` directly.
pub use digdigdig3::core::types::{AccountType, ExchangeId};
