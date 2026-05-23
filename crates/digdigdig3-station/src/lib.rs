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
pub mod gap_heal;
pub mod persistence;
pub(crate) mod polling;
pub mod series;
pub mod station;
pub mod subscription;

// Modules moved from dig3-core (persistence/cache/cure/OB concerns belong in station)
pub mod storage;
pub mod orderbook;
pub mod rest_cache;
pub mod replay;
pub mod cure;

#[cfg(feature = "reconnect")]
pub mod reconnect;

pub use builder::StationBuilder;
pub use cache::{ticker_cache, CacheConfig, TickerKey};
pub use error::{Result, StationError};
pub use gap_heal::GapHealConfig;
pub use persistence::PersistenceConfig;
pub use polling::PollSource;
pub use series::{DataPoint, DiskStore, Kind, PollSpec, Series, SeriesKey, SharedSeriesMap};
pub use station::Station;
pub use subscription::{
    Event, FailedStream, Stream, SubscribeReport, SubscriptionHandle, SubscriptionSet,
};

// Re-exports for moved modules (mirror what core used to expose)
pub use storage::{EventLog, EventLogIter, EventRecord, StorageManager, StorageConfig, StreamKey};
pub use replay::{ReplayHub, ReplayConfig, ReplayRate};
pub use orderbook::{OrderBookTracker, OrderBookError};
pub use rest_cache::RestCache;
pub use cure::{
    IntegrityChecker, IntegrityReport,
    Deduper,
    GapDetector, GapInfo,
    RepairPipeline, RepairReport,
};

// Re-export common core types so consumers can build a SubscriptionSet without
// pulling `digdigdig3` directly.
pub use digdigdig3::core::types::{AccountType, ExchangeId};
