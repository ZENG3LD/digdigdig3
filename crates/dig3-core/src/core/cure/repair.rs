//! Combined repair pipeline.
//!
//! Runs integrity check → dedup → gap detection in sequence, returning a single
//! [`RepairReport`].  Pass `dry_run = true` to skip the actual dedup write.

use crate::core::cure::{
    dedup::Deduper,
    gap::{GapDetector, GapInfo},
    integrity::{IntegrityChecker, IntegrityReport},
};
use crate::core::storage::{StorageManager, StreamKey};

// ── RepairReport ──────────────────────────────────────────────────────────────

/// Combined output of one pipeline run.
#[derive(Debug)]
pub struct RepairReport {
    pub integrity: IntegrityReport,
    /// Records kept after deduplication (or `record_count` when `dry_run = true`).
    pub deduped_kept: u64,
    /// Records removed as duplicates (or `duplicate_count` when `dry_run = true`).
    pub deduped_removed: u64,
    /// Sequence gaps found during orderbook replay (empty for non-orderbook streams).
    pub orderbook_gaps: Vec<GapInfo>,
}

// ── RepairPipeline ────────────────────────────────────────────────────────────

/// Runs integrity → dedup → gap detection on a stored stream.
pub struct RepairPipeline<'a> {
    storage: &'a StorageManager,
}

impl<'a> RepairPipeline<'a> {
    pub fn new(storage: &'a StorageManager) -> Self {
        Self { storage }
    }

    /// Run the full pipeline over `[from_ms, to_ms]`.
    ///
    /// When `dry_run = true`, deduplication is simulated: counts are taken from
    /// `IntegrityReport` and no files are written.
    ///
    /// Gap detection is only performed when `key.stream_kind` contains `"orderbook"`,
    /// `"Orderbook"`, or `"delta"` / `"Delta"` (case-insensitive check).
    pub async fn run(
        &self,
        key: &StreamKey,
        from_ms: i64,
        to_ms: i64,
        dry_run: bool,
    ) -> std::io::Result<RepairReport> {
        let integrity = IntegrityChecker::new(self.storage)
            .check(key, from_ms, to_ms)
            .await?;

        let (kept, removed) = if dry_run {
            (integrity.record_count, integrity.duplicate_count)
        } else {
            Deduper::new(self.storage)
                .dedup(key, from_ms, to_ms)
                .await?
        };

        let kind_lower = key.stream_kind.to_lowercase();
        let is_orderbook = kind_lower.contains("orderbook") || kind_lower.contains("delta");

        let orderbook_gaps = if is_orderbook {
            GapDetector::new(self.storage)
                .detect(key, from_ms, to_ms)
                .await?
        } else {
            vec![]
        };

        Ok(RepairReport {
            integrity,
            deduped_kept: kept,
            deduped_removed: removed,
            orderbook_gaps,
        })
    }
}
