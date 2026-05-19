//! `digdigdig3-station` — Layer 2 of the digdigdig3 workspace.
//!
//! High-level consumer-facing builder over [`digdigdig3_core::connector_manager::ExchangeHub`].
//! See `docs/plans/station-architecture.md` for design, `docs/plans/station-phase-1-plan.md`
//! for the phase-by-phase implementation roadmap.
//!
//! Phase 1 scope (current): skeleton only. Modules below are stubs.

pub mod builder;
pub mod error;
pub mod station;
pub mod subscription;

#[cfg(feature = "cache")]
pub mod cache;
#[cfg(feature = "persistence")]
pub mod persistence;
#[cfg(feature = "reconnect")]
pub mod reconnect;

pub use builder::StationBuilder;
pub use error::{Result, StationError};
pub use station::Station;
pub use subscription::{Event, Stream, SubscriptionHandle, SubscriptionSet};

// Re-export common core types so consumers can build a SubscriptionSet without
// pulling `digdigdig3-core` directly.
pub use digdigdig3_core::core::types::{AccountType, ExchangeId};
