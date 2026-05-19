//! StorageManager — top-level storage owner.
//!
//! Manages multiple per-`(exchange, account, symbol, stream_kind)` log streams.
//! Handles file lifecycle, rotation, and retention.
//!
//! All write operations are async-safe: each `RotatingWriter` is wrapped in
//! `tokio::sync::Mutex` and accessed through an `Arc`.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;

use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use tokio::sync::Mutex;

use super::retention;
use super::rotation::{read_file_range, RotatingWriter};

// ── Config ────────────────────────────────────────────────────────────────────

/// Top-level configuration for `StorageManager`.
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Root directory. Layout: `{root}/{exchange}/{account}/{symbol}/{stream_kind}/`
    pub root: PathBuf,
    /// How many days to keep daily files before auto-deletion. Default: 30.
    pub default_retention_days: u32,
    /// How often (seconds) to write orderbook snapshots. Default: 300 (5 min).
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

// ── StorageManager ────────────────────────────────────────────────────────────

/// Manages rotating daily log files for multiple concurrent streams.
///
/// # Example
/// ```no_run
/// use digdigdig3_core::core::storage::{StorageManager, StorageConfig, StreamKey};
///
/// #[tokio::main]
/// async fn main() -> std::io::Result<()> {
///     let mgr = StorageManager::new(StorageConfig::default())?;
///     let key = StreamKey {
///         exchange: "binance".into(),
///         account: "spot".into(),
///         symbol: "BTCUSDT".into(),
///         stream_kind: "ticker".into(),
///     };
///     mgr.append(&key, 1_700_000_000_000, b"payload").await?;
///     Ok(())
/// }
/// ```
pub struct StorageManager {
    config: StorageConfig,
    open_writers: Arc<Mutex<HashMap<StreamKey, Arc<Mutex<RotatingWriter>>>>>,
}

impl StorageManager {
    /// Create a `StorageManager`. Creates `config.root` if it doesn't exist.
    pub fn new(config: StorageConfig) -> std::io::Result<Self> {
        std::fs::create_dir_all(&config.root)?;
        Ok(Self {
            config,
            open_writers: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Append one record to the stream identified by `key`.
    ///
    /// Rotates to a new daily file automatically at UTC midnight.
    pub async fn append(&self, key: &StreamKey, ts_ms: i64, payload: &[u8]) -> std::io::Result<()> {
        let writer = self.get_or_open(key).await?;
        let mut w = writer.lock().await;
        w.append(ts_ms, payload)
    }

    /// Flush all open writers to the OS.
    pub async fn flush_all(&self) -> std::io::Result<()> {
        let writers = self.open_writers.lock().await;
        for w in writers.values() {
            w.lock().await.flush()?;
        }
        Ok(())
    }

    /// Read records in `[from_ms, to_ms]` (inclusive) for `key`.
    ///
    /// Spans multiple daily files automatically. Accepts any `i64` (including
    /// `i64::MAX` sentinel for "read to end") — values outside the
    /// representable date range are clamped to `[0, MAX_SAFE_MS]`.
    pub async fn read_range(
        &self,
        key: &StreamKey,
        from_ms: i64,
        to_ms: i64,
    ) -> std::io::Result<Vec<(i64, Vec<u8>)>> {
        let dir = self.stream_dir(key);
        if !dir.exists() {
            return Ok(vec![]);
        }

        let from_ms = from_ms.max(0);
        let to_ms = to_ms.clamp(0, MAX_SAFE_MS);
        if from_ms > to_ms {
            return Ok(vec![]);
        }

        let from_day = ms_to_date(from_ms)
            .ok_or_else(|| std::io::Error::other("bad from_ms timestamp"))?;
        let to_day = ms_to_date(to_ms)
            .ok_or_else(|| std::io::Error::other("bad to_ms timestamp"))?;

        let mut files: Vec<(NaiveDate, PathBuf)> = Vec::new();
        for entry in std::fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            if path.extension().and_then(|s| s.to_str()) != Some("bin") { continue }
            let Ok(date) = NaiveDate::parse_from_str(stem, "%Y-%m-%d") else { continue };
            if date >= from_day && date <= to_day {
                files.push((date, path));
            }
        }
        files.sort_by_key(|(d, _)| *d);

        let mut out = Vec::new();
        for (_, file) in files {
            let records = read_file_range(&file, from_ms, to_ms)?;
            out.extend(records);
        }
        Ok(out)
    }

    /// Run retention sweep — delete daily files older than `config.default_retention_days`.
    ///
    /// Returns the count of deleted files.
    pub fn cleanup(&self, now: DateTime<Utc>) -> std::io::Result<usize> {
        retention::sweep(&self.config.root, now, self.config.default_retention_days)
    }

    /// Return the directory for a stream: `{root}/{exchange}/{account}/{symbol}/{stream_kind}/`
    pub fn stream_dir(&self, key: &StreamKey) -> PathBuf {
        self.config
            .root
            .join(&key.exchange)
            .join(&key.account)
            .join(&key.symbol)
            .join(&key.stream_kind)
    }

    /// Return reference to config.
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }

    // ── internal ──────────────────────────────────────────────────────────────

    async fn get_or_open(
        &self,
        key: &StreamKey,
    ) -> std::io::Result<Arc<Mutex<RotatingWriter>>> {
        let mut writers = self.open_writers.lock().await;
        if let Some(w) = writers.get(key) {
            return Ok(w.clone());
        }
        let dir = self.stream_dir(key);
        let w = Arc::new(Mutex::new(RotatingWriter::new(dir)?));
        writers.insert(key.clone(), w.clone());
        Ok(w)
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Largest UTC millisecond timestamp that `chrono` can convert back to a
/// `NaiveDate` — corresponds to `9999-12-31T23:59:59.999Z`. Anything above
/// this overflows `ms_to_date`.
pub const MAX_SAFE_MS: i64 = 253_402_300_799_999;

fn ms_to_date(ts_ms: i64) -> Option<NaiveDate> {
    Utc.timestamp_millis_opt(ts_ms)
        .single()
        .map(|dt| dt.date_naive())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TmpDir(PathBuf);
    impl TmpDir {
        fn new(tag: &str) -> Self {
            let p = std::env::temp_dir().join(format!(
                "dig3-mgr-test-{}-{}",
                tag,
                Utc::now().timestamp_nanos_opt().unwrap_or(0)
            ));
            std::fs::create_dir_all(&p).unwrap();
            Self(p)
        }
        fn path(&self) -> &std::path::Path { &self.0 }
    }
    impl Drop for TmpDir {
        fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); }
    }

    fn make_key() -> StreamKey {
        StreamKey {
            exchange: "binance".into(),
            account: "spot".into(),
            symbol: "BTCUSDT".into(),
            stream_kind: "trade".into(),
        }
    }

    #[tokio::test]
    async fn read_range_clamps_i64_max_sentinel() {
        let tmp = TmpDir::new("max");
        let mgr = StorageManager::new(StorageConfig {
            root: tmp.path().to_path_buf(),
            ..Default::default()
        }).unwrap();
        let key = make_key();

        let now = Utc::now().timestamp_millis();
        mgr.append(&key, now, b"a").await.unwrap();
        mgr.append(&key, now + 1, b"b").await.unwrap();
        mgr.flush_all().await.unwrap();

        // i64::MAX must NOT hang or error — must clamp + return all records.
        let out = mgr.read_range(&key, 0, i64::MAX).await.unwrap();
        assert_eq!(out.len(), 2, "expected 2 records back, got {}", out.len());
        assert_eq!(out[0].1, b"a");
        assert_eq!(out[1].1, b"b");
    }

    #[tokio::test]
    async fn read_range_negative_from_is_clamped() {
        let tmp = TmpDir::new("neg");
        let mgr = StorageManager::new(StorageConfig {
            root: tmp.path().to_path_buf(),
            ..Default::default()
        }).unwrap();
        let key = make_key();
        let now = Utc::now().timestamp_millis();
        mgr.append(&key, now, b"r").await.unwrap();
        mgr.flush_all().await.unwrap();

        let out = mgr.read_range(&key, i64::MIN, i64::MAX).await.unwrap();
        assert_eq!(out.len(), 1);
    }

    #[tokio::test]
    async fn read_range_missing_dir_returns_empty() {
        let tmp = TmpDir::new("ghost");
        let mgr = StorageManager::new(StorageConfig {
            root: tmp.path().to_path_buf(),
            ..Default::default()
        }).unwrap();
        let key = StreamKey {
            exchange: "ghost".into(), account: "x".into(),
            symbol: "x".into(), stream_kind: "x".into(),
        };
        let out = mgr.read_range(&key, 0, i64::MAX).await.unwrap();
        assert!(out.is_empty());
    }
}
