//! Event deduplication.
//!
//! Reads a stream range, identifies duplicate records by `(ts_ms, sha256[:16])`,
//! and writes unique records to a sister stream (`{stream_kind}_deduped`).
//!
//! The original stream is never modified — the deduped output is a separate file,
//! allowing comparison and rollback.

use std::collections::HashSet;

use crate::core::cure::integrity::sha256_first16;
use crate::core::storage::{StorageManager, StreamKey};

// ── Deduper ───────────────────────────────────────────────────────────────────

/// Deduplicates a stored stream, writing unique records to a `_deduped` sister stream.
pub struct Deduper<'a> {
    storage: &'a StorageManager,
}

impl<'a> Deduper<'a> {
    pub fn new(storage: &'a StorageManager) -> Self {
        Self { storage }
    }

    /// Read `[from_ms, to_ms]` from `key`, deduplicate, write unique records to
    /// `{key.stream_kind}_deduped`.
    ///
    /// Returns `(kept_count, removed_count)`.
    ///
    /// The output stream uses the same exchange/account/symbol as `key` but
    /// `stream_kind` is suffixed with `_deduped`.  This lets you diff original
    /// vs deduped without losing the raw data.
    pub async fn dedup(
        &self,
        key: &StreamKey,
        from_ms: i64,
        to_ms: i64,
    ) -> std::io::Result<(u64, u64)> {
        let records = self.storage.read_range(key, from_ms, to_ms).await?;

        let out_key = StreamKey {
            exchange: key.exchange.clone(),
            account: key.account.clone(),
            symbol: key.symbol.clone(),
            stream_kind: format!("{}_deduped", key.stream_kind),
        };

        let mut seen: HashSet<(i64, [u8; 16])> = HashSet::new();
        let mut kept = 0u64;
        let mut removed = 0u64;

        for (ts, payload) in records {
            let hash = sha256_first16(&payload);
            if seen.insert((ts, hash)) {
                self.storage.append(&out_key, ts, &payload).await?;
                kept += 1;
            } else {
                removed += 1;
            }
        }

        Ok((kept, removed))
    }
}
