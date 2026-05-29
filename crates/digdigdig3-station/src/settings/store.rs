//! Native file-backed `SettingsStore`.
//!
//! Persists a `HashMap<String, serde_json::Value>` as a pretty-printed JSON
//! file at `{root}/{namespace}.json`. Writes are atomic: a `.tmp` file is
//! written first, then renamed over the target path.

use std::collections::HashMap;
use std::path::PathBuf;

use serde_json::Value;

use super::SettingsError;

/// Native JSON settings store.
///
/// All get/set/remove/contains/keys operations are synchronous (in-memory).
/// Only `open` and `save` are async (matching the wasm32 sibling API).
#[derive(Debug)]
pub struct SettingsStore {
    path: PathBuf,
    map: HashMap<String, Value>,
}

impl SettingsStore {
    /// Open (or create) a settings store.
    ///
    /// `root` — directory that will contain the settings file.
    /// `namespace` — base name of the file (`{namespace}.json`).
    ///
    /// Creates parent directories if missing. If the file exists it is parsed
    /// into the in-memory map; a corrupt or unreadable file returns `Err`
    /// rather than silently wiping it.
    pub async fn open(
        root: impl AsRef<std::path::Path>,
        namespace: &str,
    ) -> Result<Self, SettingsError> {
        let path = root.as_ref().join(format!("{namespace}.json"));

        // Create parent directory if needed.
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| SettingsError::Io(e.to_string()))?;
        }

        let map = if path.exists() {
            let content = std::fs::read_to_string(&path)
                .map_err(|e| SettingsError::Io(e.to_string()))?;
            serde_json::from_str::<HashMap<String, Value>>(&content)
                .map_err(|e| SettingsError::Serde(e.to_string()))?
        } else {
            HashMap::new()
        };

        Ok(Self { path, map })
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

    /// Persist the in-memory map to disk atomically.
    ///
    /// Writes to `{path}.tmp` first, then renames it over `path`. This ensures
    /// the previous file is never partially overwritten on a crash.
    pub async fn save(&self) -> Result<(), SettingsError> {
        let json = serde_json::to_string_pretty(&self.map)
            .map_err(|e| SettingsError::Serde(e.to_string()))?;

        let tmp_path = {
            let mut p = self.path.clone();
            let fname = p
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("settings")
                .to_string();
            p.set_file_name(format!("{fname}.tmp"));
            p
        };

        std::fs::write(&tmp_path, &json)
            .map_err(|e| SettingsError::Io(e.to_string()))?;

        std::fs::rename(&tmp_path, &self.path)
            .map_err(|e| SettingsError::Io(e.to_string()))?;

        Ok(())
    }
}
