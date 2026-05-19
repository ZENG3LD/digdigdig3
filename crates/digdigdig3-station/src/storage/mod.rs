//! Storage subsystem — Phase μ.
//!
//! ## Modules
//!
//! - `event_log` — legacy append-only flat-file log (κ.2 compat)
//! - `rotation` — daily rotating writer + raw file reader
//! - `index` — time-range index sidecar (stub)
//! - `snapshot` — periodic snapshot writer/reader for stateful streams
//! - `retention` — auto-delete files older than N days
//! - `manager` — `StorageManager` owning multiple rotating streams

pub mod event_log;
pub mod index;
pub mod manager;
pub mod retention;
pub mod rotation;
pub mod snapshot;

// Flat re-exports for call sites that use `core::storage::Foo` directly.
pub use event_log::{EventLog, EventLogIter, EventRecord};
pub use manager::{StorageConfig, StorageManager, StreamKey};
