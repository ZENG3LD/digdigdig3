//! Async record iterator over StorageManager for a given StreamKey + ts range.
//!
//! Used internally by `ReplayWebSocket` to load records before emitting.

use std::sync::Arc;

use crate::storage::{StorageManager, StreamKey};

/// Load all `(ts_ms, payload)` records for `key` in `[from_ms, to_ms]`.
///
/// Delegates directly to `StorageManager::read_range`.  Returns an empty `Vec`
/// when no data is stored for the key or the range is empty.
pub async fn load_records(
    storage: &Arc<StorageManager>,
    key: &StreamKey,
    from_ms: i64,
    to_ms: i64,
) -> std::io::Result<Vec<(i64, Vec<u8>)>> {
    storage.read_range(key, from_ms, to_ms).await
}
