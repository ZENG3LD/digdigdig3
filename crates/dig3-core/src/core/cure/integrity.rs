//! Integrity check — reads a stored stream, reports stats + anomalies.
//!
//! Pure analysis, no mutation.

use std::collections::BTreeMap;

use sha2::{Digest, Sha256};

use crate::core::storage::{StorageManager, StreamKey};

// ── Public types ──────────────────────────────────────────────────────────────

/// A period longer than the configured threshold with no events.
#[derive(Debug, Clone)]
pub struct TimeGap {
    pub start_ms: i64,
    pub end_ms: i64,
    pub duration_ms: i64,
}

/// A jump in sequence numbers detected in orderbook delta payloads.
#[derive(Debug, Clone)]
pub struct SequenceGap {
    pub from_seq: u64,
    pub to_seq: u64,
    pub ts_ms: i64,
}

/// Full analysis result for one stream range.
#[derive(Debug, Clone)]
pub struct IntegrityReport {
    pub stream: StreamKey,
    pub from_ms: i64,
    pub to_ms: i64,
    pub record_count: u64,
    pub duplicate_count: u64,
    pub out_of_order_count: u64,
    pub parse_errors: u64,
    /// Periods longer than `time_gap_threshold_ms` with no events.
    pub time_gaps: Vec<TimeGap>,
    /// Sequence number jumps detected in orderbook payloads.
    pub sequence_gaps: Vec<SequenceGap>,
    pub first_ts: Option<i64>,
    pub last_ts: Option<i64>,
    /// Average interval between successive records (ms).
    pub avg_interval_ms: Option<f64>,
}

// ── IntegrityChecker ─────────────────────────────────────────────────────────

/// Reads a stream range and produces an [`IntegrityReport`].
pub struct IntegrityChecker<'a> {
    storage: &'a StorageManager,
    /// Minimum quiet period (ms) that is reported as a [`TimeGap`]. Default: 60 000 (1 min).
    time_gap_threshold_ms: i64,
}

impl<'a> IntegrityChecker<'a> {
    pub fn new(storage: &'a StorageManager) -> Self {
        Self {
            storage,
            time_gap_threshold_ms: 60_000,
        }
    }

    /// Override the time-gap threshold (milliseconds).
    pub fn with_time_gap_threshold(mut self, ms: i64) -> Self {
        self.time_gap_threshold_ms = ms;
        self
    }

    /// Run the full integrity analysis over `[from_ms, to_ms]`.
    pub async fn check(
        &self,
        key: &StreamKey,
        from_ms: i64,
        to_ms: i64,
    ) -> std::io::Result<IntegrityReport> {
        let records = self.storage.read_range(key, from_ms, to_ms).await?;

        let mut report = IntegrityReport {
            stream: key.clone(),
            from_ms,
            to_ms,
            record_count: 0,
            duplicate_count: 0,
            out_of_order_count: 0,
            parse_errors: 0,
            time_gaps: Vec::new(),
            sequence_gaps: Vec::new(),
            first_ts: None,
            last_ts: None,
            avg_interval_ms: None,
        };
        report.record_count = records.len() as u64;

        if records.is_empty() {
            return Ok(report);
        }

        report.first_ts = records.first().map(|(t, _)| *t);
        report.last_ts = records.last().map(|(t, _)| *t);

        let mut last_ts: Option<i64> = None;
        let mut last_seq: Option<u64> = None;
        // Key: (ts_ms, sha256_first16) → occurrence count
        let mut seen: BTreeMap<(i64, [u8; 16]), u32> = BTreeMap::new();

        for (ts, payload) in &records {
            // ── time gap / out-of-order ───────────────────────────────────────
            if let Some(prev) = last_ts {
                let delta = *ts - prev;
                if delta < 0 {
                    report.out_of_order_count += 1;
                } else if delta > self.time_gap_threshold_ms {
                    report.time_gaps.push(TimeGap {
                        start_ms: prev,
                        end_ms: *ts,
                        duration_ms: delta,
                    });
                }
            }
            last_ts = Some(*ts);

            // ── dedup fingerprint ─────────────────────────────────────────────
            let hash = sha256_first16(payload);
            *seen.entry((*ts, hash)).or_insert(0) += 1;

            // ── sequence gap from JSON payload ────────────────────────────────
            match serde_json::from_slice::<serde_json::Value>(payload) {
                Ok(v) => {
                    if let Some(seq) = extract_sequence(&v) {
                        if let Some(prev) = last_seq {
                            if seq > prev + 1 {
                                report.sequence_gaps.push(SequenceGap {
                                    from_seq: prev,
                                    to_seq: seq,
                                    ts_ms: *ts,
                                });
                            }
                        }
                        last_seq = Some(seq);
                    }
                }
                Err(_) => {
                    report.parse_errors += 1;
                }
            }
        }

        report.duplicate_count = seen
            .values()
            .filter(|&&c| c > 1)
            .map(|&c| (c - 1) as u64)
            .sum();

        if let (Some(first), Some(last)) = (report.first_ts, report.last_ts) {
            if report.record_count > 1 {
                report.avg_interval_ms =
                    Some((last - first) as f64 / (report.record_count - 1) as f64);
            }
        }

        Ok(report)
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

pub(crate) fn sha256_first16(bytes: &[u8]) -> [u8; 16] {
    let mut h = Sha256::new();
    h.update(bytes);
    let full = h.finalize();
    let mut out = [0u8; 16];
    out.copy_from_slice(&full[..16]);
    out
}

/// Try to extract a sequence number from common orderbook event JSON shapes.
///
/// Tries fields: `last_update_id`, `sequence`, `u`, `seq`.
fn extract_sequence(v: &serde_json::Value) -> Option<u64> {
    v.get("last_update_id")
        .and_then(|x| x.as_u64())
        .or_else(|| v.get("sequence").and_then(|x| x.as_u64()))
        .or_else(|| v.get("u").and_then(|x| x.as_u64()))
        .or_else(|| v.get("seq").and_then(|x| x.as_u64()))
}
