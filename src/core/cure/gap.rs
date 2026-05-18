//! Orderbook sequence-gap detection.
//!
//! Replays stored orderbook payloads through [`OrderBookTracker`] and collects
//! every point where the tracker reports a [`OrderBookError::SequenceGap`].
//!
//! Repair (REST snapshot fetch) is out of scope here — [`GapInfo`] gives the
//! caller the information needed to request a fresh snapshot from the exchange.

use crate::core::orderbook::{OrderBookError, OrderBookTracker};
use crate::core::storage::{StorageManager, StreamKey};
use crate::core::types::{OrderBook, OrderbookDelta};

// ── Public types ──────────────────────────────────────────────────────────────

/// One detected sequence gap in a replay.
#[derive(Debug, Clone)]
pub struct GapInfo {
    /// Timestamp (Unix ms) of the delta that triggered the gap.
    pub ts_ms: i64,
    /// The `last_update_id` the tracker expected.
    pub expected: u64,
    /// The `prev_update_id` the delta carried.
    pub got: u64,
}

// ── GapDetector ───────────────────────────────────────────────────────────────

/// Replays stored orderbook events, collecting sequence gaps.
pub struct GapDetector<'a> {
    storage: &'a StorageManager,
}

impl<'a> GapDetector<'a> {
    pub fn new(storage: &'a StorageManager) -> Self {
        Self { storage }
    }

    /// Replay `[from_ms, to_ms]` from `key` and return all detected sequence gaps.
    ///
    /// Each stored record is tried as [`OrderBook`] (snapshot) first, then as
    /// [`OrderbookDelta`].  Records that parse as neither are silently skipped —
    /// `integrity.parse_errors` captures those.
    pub async fn detect(
        &self,
        key: &StreamKey,
        from_ms: i64,
        to_ms: i64,
    ) -> std::io::Result<Vec<GapInfo>> {
        let records = self.storage.read_range(key, from_ms, to_ms).await?;
        let mut tracker = OrderBookTracker::new("replay");
        let mut gaps = Vec::new();

        for (ts, payload) in records {
            // Try to detect the record type from the JSON before dispatching.
            // OrderbookDelta has `prev_update_id` or no `sequence` field used
            // for snapshots.  We prefer delta when `prev_update_id` is present,
            // which is the field that enables gap detection.
            let is_delta = serde_json::from_slice::<serde_json::Value>(&payload)
                .ok()
                .and_then(|v| v.get("prev_update_id").cloned())
                .is_some();

            if is_delta {
                if let Ok(delta) = serde_json::from_slice::<OrderbookDelta>(&payload) {
                    if let Err(OrderBookError::SequenceGap { last, got }) =
                        tracker.apply_delta(&delta)
                    {
                        gaps.push(GapInfo {
                            ts_ms: ts,
                            expected: last,
                            got,
                        });
                    }
                }
            } else if let Ok(snapshot) = serde_json::from_slice::<OrderBook>(&payload) {
                if let Err(OrderBookError::SequenceGap { last, got }) =
                    tracker.apply_snapshot(&snapshot)
                {
                    gaps.push(GapInfo {
                        ts_ms: ts,
                        expected: last,
                        got,
                    });
                }
            }
        }

        Ok(gaps)
    }
}
