//! OPFS-backed `SettingsStore` for wasm32.
//!
//! Persists a `HashMap<String, serde_json::Value>` as a pretty-printed JSON
//! file at `{namespace}.json` in the OPFS root. The full file is rewritten on
//! each `save()` (settings files are small — typically < 4 KB).
//!
//! # Design
//!
//! - `open` reads the existing file from OPFS (or starts empty if absent).
//! - `get`/`set`/`remove`/`contains`/`keys` — synchronous in-memory operations.
//! - `save` — rewrites the entire file via `createWritable` (no keepExistingData,
//!   so the file is truncated and replaced atomically at the browser level).

use std::collections::HashMap;

use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemCreateWritableOptions, FileSystemGetFileOptions, FileSystemWritableFileStream,
};

use crate::opfs_helpers::{opfs_read_all, opfs_root, OpfsError};

use super::SettingsError;

// ── OpfsError → SettingsError ─────────────────────────────────────────────────

impl From<OpfsError> for SettingsError {
    fn from(e: OpfsError) -> Self {
        SettingsError::Opfs(e.to_string())
    }
}

// ── SettingsStore ─────────────────────────────────────────────────────────────

/// OPFS-backed JSON settings store for wasm32.
///
/// Public API is call-site-compatible with the native `store.rs` sibling
/// except `open` takes only `namespace` (no root path — OPFS root is always
/// `navigator.storage.getDirectory()`).
#[derive(Debug)]
pub struct SettingsStore {
    namespace: String,
    map: HashMap<String, Value>,
}

impl SettingsStore {
    /// Open (or create) a settings store in OPFS.
    ///
    /// `namespace` — the file stored in OPFS root will be `{namespace}.json`.
    ///
    /// Missing file → empty map (not an error).
    /// Parse error → `Err(Serde)`.
    pub async fn open(namespace: &str) -> Result<Self, SettingsError> {
        let root = opfs_root().await?;
        let filename = format!("{namespace}.json");

        let map = match opfs_read_all(&root, &filename).await {
            Ok(bytes) => {
                let text =
                    String::from_utf8(bytes).map_err(|e| SettingsError::Serde(e.to_string()))?;
                serde_json::from_str::<HashMap<String, Value>>(&text)
                    .map_err(|e| SettingsError::Serde(e.to_string()))?
            }
            Err(OpfsError::FileNotFound) => HashMap::new(),
            Err(e) => return Err(SettingsError::from(e)),
        };

        Ok(Self {
            namespace: namespace.to_string(),
            map,
        })
    }

    /// Look up a key and deserialize it as `T`.
    ///
    /// Returns `None` when the key is absent or the value cannot be
    /// deserialized as `T` (wrong type — not a panic).
    pub fn get<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        let value = self.map.get(key)?;
        serde_json::from_value(value.clone()).ok()
    }

    /// Serialize `value` as JSON and insert it under `key`.
    ///
    /// The store is only modified in memory; call `save()` to persist.
    pub fn set<T: serde::Serialize>(
        &mut self,
        key: &str,
        value: &T,
    ) -> Result<(), SettingsError> {
        let json_value =
            serde_json::to_value(value).map_err(|e| SettingsError::Serde(e.to_string()))?;
        self.map.insert(key.to_string(), json_value);
        Ok(())
    }

    /// Remove a key. Returns `true` if the key was present.
    pub fn remove(&mut self, key: &str) -> bool {
        self.map.remove(key).is_some()
    }

    /// Return all keys in the store (order unspecified).
    pub fn keys(&self) -> Vec<String> {
        self.map.keys().cloned().collect()
    }

    /// Return `true` if `key` exists in the store.
    pub fn contains(&self, key: &str) -> bool {
        self.map.contains_key(key)
    }

    /// Persist the in-memory map to OPFS.
    ///
    /// Rewrites the entire file using `createWritable` (without
    /// `keepExistingData`) so the file is truncated and replaced. Maps JS
    /// errors to `Err(Opfs)`.
    pub async fn save(&self) -> Result<(), SettingsError> {
        let json = serde_json::to_string_pretty(&self.map)
            .map_err(|e| SettingsError::Serde(e.to_string()))?;

        let root = opfs_root().await?;
        let filename = format!("{}.json", self.namespace);

        // Get-or-create the file handle.
        let fopts = FileSystemGetFileOptions::new();
        fopts.set_create(true);

        let fh: web_sys::FileSystemFileHandle =
            JsFuture::from(root.get_file_handle_with_options(&filename, &fopts))
                .await
                .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?
                .dyn_into()
                .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?;

        // createWritable without keepExistingData → truncate + replace.
        let write_opts = FileSystemCreateWritableOptions::new();
        write_opts.set_keep_existing_data(false);

        let writable: FileSystemWritableFileStream =
            JsFuture::from(fh.create_writable_with_options(&write_opts))
                .await
                .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?
                .dyn_into()
                .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?;

        // Write the UTF-8 JSON bytes.
        let bytes = json.into_bytes();
        let arr = js_sys::Uint8Array::from(bytes.as_slice());
        JsFuture::from(
            writable
                .write_with_buffer_source(&arr)
                .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?,
        )
        .await
        .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?;

        JsFuture::from(writable.close())
            .await
            .map_err(|e| SettingsError::Opfs(format!("{e:?}")))?;

        Ok(())
    }
}
