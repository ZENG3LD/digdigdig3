//! General-purpose JSON settings/state persistence store.
//!
//! Works on both native (file-backed) and wasm32 (OPFS-backed).
//! Keys are arbitrary strings; values are `serde_json::Value` so any
//! `serde::Serialize` / `serde::de::DeserializeOwned` type round-trips
//! through the store without a custom codec.
//!
//! # Usage (native)
//!
//! ```rust,no_run
//! # async fn example() -> Result<(), digdigdig3_station::SettingsError> {
//! use digdigdig3_station::SettingsStore;
//!
//! let mut store = SettingsStore::open("./my_app", "ui-state").await?;
//! store.set("theme", &"dark")?;
//! store.save().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Usage (wasm32)
//!
//! ```rust,ignore
//! let mut store = SettingsStore::open("ui-state").await?;
//! store.set("theme", &"dark")?;
//! store.save().await?;
//! ```

use std::fmt;

// ── cfg-split (mirrors series/mod.rs exactly) ─────────────────────────────────

#[cfg(not(target_arch = "wasm32"))]
mod store;

#[cfg(target_arch = "wasm32")]
#[path = "store_wasm.rs"]
mod store;

pub use store::SettingsStore;

// ── Shared error type ─────────────────────────────────────────────────────────

/// Errors from `SettingsStore` operations.
///
/// Both native and wasm32 impls use this single enum so call sites compile
/// unchanged on either target.
#[derive(Debug)]
pub enum SettingsError {
    /// I/O error (native only — file read/write/rename failures).
    Io(String),
    /// JSON serialization or deserialization error.
    Serde(String),
    /// OPFS error (wasm32 only — browser storage API failures).
    Opfs(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Io(msg) => write!(f, "settings I/O error: {msg}"),
            SettingsError::Serde(msg) => write!(f, "settings JSON error: {msg}"),
            SettingsError::Opfs(msg) => write!(f, "settings OPFS error: {msg}"),
        }
    }
}

impl std::error::Error for SettingsError {}
