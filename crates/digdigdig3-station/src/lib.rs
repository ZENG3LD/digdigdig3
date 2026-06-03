//! `digdigdig3-station` — Layer 2 of the digdigdig3 workspace.
//!
//! High-level consumer-facing builder over [`digdigdig3::connector_manager::ExchangeHub`].
//! See `docs/plans/station-architecture.md` for design, `docs/plans/station-phase-1-plan.md`
//! for the phase-by-phase implementation roadmap.
//!
//! Phase 1 scope (current): skeleton only. Modules below are stubs.

pub mod backfill;
pub use backfill::fetch_history;
pub mod bar_align;
pub use bar_align::{load_bar_aligned, load_for_key, BarAlignedSeries, FillPolicy, ScalarBar};
pub mod ws_health;
pub use ws_health::WsHealth;
pub mod builder;
pub mod settings;
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

// OPFS helpers — wasm32 only, shared by store_wasm + manager_wasm.
#[cfg(target_arch = "wasm32")]
pub(crate) mod opfs_helpers;

// storage: native uses std::fs; wasm32 uses OPFS-backed manager.
// On wasm32 only StorageManager / StorageConfig / StreamKey are available;
// the sub-modules that use std::fs (event_log, rotation, retention, snapshot,
// index) remain native-only inside storage/mod.rs.
pub mod storage;

// cure: pure logic over StorageManager — compiles on both targets.
pub mod cure;

// replay: backed by StorageManager; ws.rs has cfg-split for tokio::spawn /
// wasm_bindgen_futures::spawn_local and tokio::time::sleep / gloo_timers.
pub mod replay;

// polling + gap_heal: REST-based; work on wasm via rest_override (Workstream A).
// spawn_poller uses cfg-split tokio::spawn / wasm_bindgen_futures::spawn_local.
pub(crate) mod polling;

pub mod gap_heal;

pub use builder::StationBuilder;
pub use cache::{ticker_cache, CacheConfig, TickerKey};
pub use error::{Result, StationError};
pub use persistence::PersistenceConfig;
pub use series::{DataPoint, Kind, Series, SeriesKey, SharedSeries, SharedSeriesMap};
pub use quota::{ConsumerHandle, ConsumerQuota, ConsumerWhitelist, QuotaError};
pub use station::Station;
pub use subscription::{
    Event, FailedStream, Stream, SubscribeReport, SubscriptionHandle, SubscriptionSet,
    WarmupReport,
};

// DiskStore is available on both targets (native: std::fs; wasm32: OPFS).
pub use series::DiskStore;

// PollSpec, PollSource, GapHealConfig: available on both targets.
// polling is now un-gated; gap_heal is un-gated.
pub use series::PollSpec;
pub use polling::PollSource;
pub use gap_heal::GapHealConfig;

// StorageManager and friends — available on both targets now.
pub use storage::{StorageConfig, StorageManager, StreamKey};

// EventLog is native-only (std::fs backed).
#[cfg(not(target_arch = "wasm32"))]
pub use storage::{EventLog, EventLogIter, EventRecord};

pub use replay::{ReplayHub, ReplayConfig, ReplayRate};

pub use orderbook::{OrderBookTracker, OrderBookError};
pub use rest_cache::RestCache;

// Settings store — native (file) and wasm32 (OPFS) at API parity.
pub use settings::{SettingsError, SettingsStore};

pub use cure::{
    IntegrityChecker, IntegrityReport,
    Deduper,
    GapDetector, GapInfo,
    RepairPipeline, RepairReport,
};

// Re-export common core types so consumers can build a SubscriptionSet without
// pulling `digdigdig3` directly.
pub use digdigdig3::core::types::{AccountType, ExchangeId};
pub use digdigdig3::SymbolInfo;
// Re-export Credentials so callers can use add_authenticated without a direct
// digdigdig3 dependency.
pub use digdigdig3::core::traits::Credentials;
// Re-export private DataPoint types for consumers that inspect private events.
pub use data::{BalanceUpdatePoint, OrderUpdatePoint, PositionUpdatePoint};
