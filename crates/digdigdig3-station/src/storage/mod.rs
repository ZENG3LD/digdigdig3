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
//!
//! On wasm32 only `manager` is available (OPFS-backed). The rest of the
//! sub-modules (`event_log`, `rotation`, `retention`, `snapshot`, `index`)
//! use `std::fs` and remain native-only.

// ── Native-only sub-modules ────────────────────────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub mod event_log;
#[cfg(not(target_arch = "wasm32"))]
pub mod index;
#[cfg(not(target_arch = "wasm32"))]
pub mod retention;
#[cfg(not(target_arch = "wasm32"))]
pub mod rotation;
#[cfg(not(target_arch = "wasm32"))]
pub mod snapshot;

// ── StorageManager — cfg-split between native and wasm32 ──────────────────────

#[cfg(not(target_arch = "wasm32"))]
pub mod manager;

#[cfg(target_arch = "wasm32")]
#[path = "manager_wasm.rs"]
pub mod manager;

// ── Re-exports ─────────────────────────────────────────────────────────────────

// StorageManager + config are available on both targets.
pub use manager::{StorageConfig, StorageManager, StreamKey};

// Native-only types that don't exist on wasm32.
#[cfg(not(target_arch = "wasm32"))]
pub use event_log::{EventLog, EventLogIter, EventRecord};
