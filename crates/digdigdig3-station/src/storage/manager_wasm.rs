//! OPFS-backed `StorageManager` for wasm32.
//!
//! Mirrors the native `manager.rs` public API exactly:
//! - `StorageConfig`, `StreamKey`, `StorageManager` structs.
//! - `new`, `append`, `flush_all`, `read_range`, `stream_dir`, `config`,
//!   `cleanup` methods — same signatures, OPFS backing instead of `std::fs`.
//!
//! Wire format is identical to the native RotatingWriter:
//!   `[i64 ts_ms LE][u32 len LE][payload bytes]`
//!
//! Files are named `{YYYY-MM-DD}.bin` inside a four-level OPFS directory:
//!   `<root>/<exchange>/<account>/<symbol>/<stream_kind>/`
//! ("root" is synthetic on wasm — the OPFS root is always the navigator storage
//! root; `StorageConfig::root` is kept for API compat but is unused.)
//!
//! # append vs flush
//!
//! `append` buffers records in memory (sync, cheap).
//! `flush_all` drains all writers to OPFS (async, one write per active stream).
//!
//! Callers that need an immediate durable write should call `flush_all` after
//! `append`.  Station's periodic-flush mechanism covers the hot path.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use tokio::sync::Mutex;
use web_sys::FileSystemDirectoryHandle;

use crate::opfs_helpers::{
    opfs_append_quota_guard, opfs_read_all, opfs_root, opfs_walk_or_create_stream, OpfsError,
};

// ── Config ────────────────────────────────────────────────────────────────────

/// Top-level configuration for `StorageManager`.
///
/// `root` is kept for API compatibility with native callers. On wasm32 the
/// OPFS root is always `navigator.storage.getDirectory()` — this field is
/// ignored at runtime but preserved so shared code that constructs
/// `StorageConfig { root: ..., .. }` compiles unchanged on wasm.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub root: PathBuf,
    pub default_retention_days: u32,
    pub orderbook_snapshot_interval_secs: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            root: PathBuf::from("dig3_storage"),
            default_retention_days: 30,
            orderbook_snapshot_interval_secs: 300,
        }
    }
}

// ── StreamKey ─────────────────────────────────────────────────────────────────

/// Unique identifier for one data stream.
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct StreamKey {
    pub exchange: String,
    pub account: String,
    pub symbol: String,
    pub stream_kind: String,
}

// ── per-stream writer ─────────────────────────────────────────────────────────

struct WasmStreamWriter {
    /// OPFS directory handle for this stream's leaf dir.
    sym_dir: FileSystemDirectoryHandle,
    /// Pending bytes bucketed by the record's OWN day (`YYYY-MM-DD`), derived
    /// from each record's `ts_ms` — NOT wall-clock. This keeps the write path
    /// symmetric with `read_range`, which derives filenames from query
    /// timestamps; a record dated 2024-03-09 must land in `2024-03-09.bin` so a
    /// later range read for that day finds it. (The native reader lists the dir
    /// and so is filename-agnostic; the wasm reader cannot, hence this.)
    day_bufs: HashMap<String, Vec<u8>>,
    /// StreamKey — retained for diagnostics / future re-open needs.
    #[allow(dead_code)]
    key: StreamKey,
}

impl WasmStreamWriter {
    /// Encode and buffer one record: `[i64 ts_ms LE][u32 len LE][payload]`,
    /// routed into the day-bucket for the record's own timestamp.
    fn push(&mut self, ts_ms: i64, payload: &[u8]) {
        let day = record_day(ts_ms);
        let buf = self.day_bufs.entry(day).or_default();
        buf.extend_from_slice(&ts_ms.to_le_bytes());
        buf.extend_from_slice(&(payload.len() as u32).to_le_bytes());
        buf.extend_from_slice(payload);
    }

    /// Flush all pending day-buckets to their `{day}.bin` files (append-only).
    async fn flush(&mut self) -> Result<(), OpfsError> {
        for (day, bytes) in self.day_bufs.drain() {
            if bytes.is_empty() {
                continue;
            }
            let fname = format!("{day}.bin");
            opfs_append_quota_guard(&self.sym_dir, &fname, &bytes).await?;
        }
        Ok(())
    }
}

// ── StorageManager ────────────────────────────────────────────────────────────

/// OPFS-backed manager of multiple per-stream append logs.
///
/// Mirrors the native `StorageManager` public API so `cure` and `replay`
/// compile unchanged on wasm32.
pub struct StorageManager {
    config: StorageConfig,
    open_writers: Arc<Mutex<HashMap<StreamKey, WasmStreamWriter>>>,
}

impl StorageManager {
    /// Create a `StorageManager`.
    ///
    /// On wasm32 this is a lightweight constructor — no OPFS I/O is performed
    /// until the first `append` or `flush_all`. The `config.root` field is
    /// accepted but unused (OPFS root is always `navigator.storage`).
    pub fn new(config: StorageConfig) -> std::io::Result<Self> {
        Ok(Self {
            config,
            open_writers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Buffer one record. The OPFS directory is opened lazily on first use.
    pub async fn append(
        &self,
        key: &StreamKey,
        ts_ms: i64,
        payload: &[u8],
    ) -> std::io::Result<()> {
        let mut writers = self.open_writers.lock().await;
        if !writers.contains_key(key) {
            let writer = open_writer(key).await.map_err(std::io::Error::from)?;
            writers.insert(key.clone(), writer);
        }
        writers
            .get_mut(key)
            .expect("just inserted")
            .push(ts_ms, payload);
        Ok(())
    }

    /// Flush all open writers to OPFS.
    pub async fn flush_all(&self) -> std::io::Result<()> {
        let mut writers = self.open_writers.lock().await;
        for w in writers.values_mut() {
            w.flush().await.map_err(std::io::Error::from)?;
        }
        Ok(())
    }

    /// Read records in `[from_ms, to_ms]` (inclusive) for `key`.
    ///
    /// Rather than using OPFS directory listing (AsyncIterator binding is
    /// fragile in web-sys), we enumerate the date range from the timestamps
    /// and try to open each day's file by name.  Files that don't exist are
    /// silently skipped.
    pub async fn read_range(
        &self,
        key: &StreamKey,
        from_ms: i64,
        to_ms: i64,
    ) -> std::io::Result<Vec<(i64, Vec<u8>)>> {
        let from_ms = from_ms.max(0);
        let to_ms = to_ms.clamp(0, MAX_SAFE_MS);
        if from_ms > to_ms {
            return Ok(vec![]);
        }

        let from_day =
            ms_to_date(from_ms).ok_or_else(|| std::io::Error::other("bad from_ms timestamp"))?;
        let to_day =
            ms_to_date(to_ms).ok_or_else(|| std::io::Error::other("bad to_ms timestamp"))?;

        // Get (or lazily open) OPFS directory for this key.
        let sym_dir = {
            let mut writers = self.open_writers.lock().await;
            if !writers.contains_key(key) {
                let writer = open_writer(key).await.map_err(std::io::Error::from)?;
                writers.insert(key.clone(), writer);
            }
            // SAFETY: just inserted above.
            writers
                .get(key)
                .expect("just inserted")
                .sym_dir
                .clone()
        };

        // Enumerate dates from from_day to to_day and try each file.
        let mut out: Vec<(i64, Vec<u8>)> = Vec::new();

        let mut day = from_day;
        while day <= to_day {
            let fname = format!("{}.bin", day.format("%Y-%m-%d"));
            match opfs_read_all(&sym_dir, &fname).await {
                Ok(bytes) => {
                    parse_records(&bytes, from_ms, to_ms, &mut out);
                }
                Err(OpfsError::FileNotFound) => {
                    // Normal — not every day has data.
                }
                Err(e) => {
                    return Err(std::io::Error::from(e));
                }
            }
            day = day.succ_opt().ok_or_else(|| {
                std::io::Error::other("date overflow iterating day range")
            })?;
        }

        Ok(out)
    }

    /// Return the synthetic directory path for `key`.
    ///
    /// On wasm32 the returned `PathBuf` is not a real filesystem path; it is a
    /// human-readable label for logging / debugging only.  The actual storage
    /// lives in OPFS under `navigator.storage.getDirectory()`.
    pub fn stream_dir(&self, key: &StreamKey) -> PathBuf {
        self.config
            .root
            .join(&key.exchange)
            .join(&key.account)
            .join(&key.symbol)
            .join(&key.stream_kind)
    }

    /// Retention sweep — no-op on wasm32. Returns `Ok(0)`.
    ///
    /// OPFS quota is managed by the browser, not by date-based retention.
    /// The signature is kept for API parity with the native implementation.
    pub fn cleanup(&self, _now: DateTime<Utc>) -> std::io::Result<usize> {
        Ok(0)
    }

    /// Return reference to config.
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Open (or create) the OPFS directory for `key` and return a `WasmStreamWriter`.
///
/// No file is opened here — files are append-only and named per record-day at
/// flush time, so the writer only needs the leaf directory handle. Existing
/// day files are picked up transparently because OPFS `append` opens-or-creates.
async fn open_writer(key: &StreamKey) -> Result<WasmStreamWriter, OpfsError> {
    let root = opfs_root().await?;
    let sym_dir = open_stream_dir(&root, key).await?;

    Ok(WasmStreamWriter {
        sym_dir,
        day_bufs: HashMap::new(),
        key: key.clone(),
    })
}

/// Walk / create the leaf directory for a `StreamKey`.
///
/// Layout: `<root>/<exchange>/<account>/<symbol>/<stream_kind>/`
async fn open_stream_dir(
    root: &FileSystemDirectoryHandle,
    key: &StreamKey,
) -> Result<FileSystemDirectoryHandle, OpfsError> {
    opfs_walk_or_create_stream(
        root,
        &key.exchange,
        &key.account,
        &key.symbol,
        &key.stream_kind,
    )
    .await
}

/// Parse `[i64 ts_ms LE][u32 len LE][payload]` records from `bytes`,
/// appending those in `[from_ms, to_ms]` to `out`.
fn parse_records(bytes: &[u8], from_ms: i64, to_ms: i64, out: &mut Vec<(i64, Vec<u8>)>) {
    let mut pos = 0usize;
    while pos + 12 <= bytes.len() {
        let ts = i64::from_le_bytes(
            bytes[pos..pos + 8]
                .try_into()
                .unwrap_or([0u8; 8]),
        );
        let len = u32::from_le_bytes(
            bytes[pos + 8..pos + 12]
                .try_into()
                .unwrap_or([0u8; 4]),
        ) as usize;
        pos += 12;
        if pos + len > bytes.len() {
            break; // truncated record — stop
        }
        let payload = bytes[pos..pos + len].to_vec();
        pos += len;
        if ts >= from_ms && ts <= to_ms {
            out.push((ts, payload));
        }
    }
}

/// Largest UTC millisecond timestamp `chrono` can convert back to a `NaiveDate`.
pub const MAX_SAFE_MS: i64 = 253_402_300_799_999;

fn ms_to_date(ts_ms: i64) -> Option<NaiveDate> {
    Utc.timestamp_millis_opt(ts_ms)
        .single()
        .map(|dt| dt.date_naive())
}

/// Day bucket (`YYYY-MM-DD`) for a record timestamp, derived from the record's
/// own `ts_ms`. Clamps out-of-range timestamps so no record is silently dropped.
fn record_day(ts_ms: i64) -> String {
    ms_to_date(ts_ms.clamp(0, MAX_SAFE_MS))
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).expect("epoch is valid"))
        .format("%Y-%m-%d")
        .to_string()
}
