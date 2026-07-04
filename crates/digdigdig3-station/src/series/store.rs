use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use chrono::NaiveDate;

use super::{DataPoint, SeriesKey};

/// Maximum number of day files a single `read_tail` call will walk backwards
/// across when the current day's file does not yield `limit` records. Bounds
/// startup latency against a huge archive — a warm-start should never stall
/// on years of history.
const MAX_TAIL_DAYS: usize = 14;

/// Generic fixed-record binary writer + reader for `T: DataPoint`.
///
/// Layout under `<storage_root>/<kind-slug>/<exchange>/<account>/<symbol>/<YYYY-MM-DD>.dat`:
/// - `.dat`: append-only, `T::RECORD_SIZE` bytes per record (little-endian).
/// - `.idx`: sparse `[u64 ts_ms, u64 file_offset]` every N records.
/// - `.blob`: append-only UTF-8 byte stream (only for types where
///   [`DataPoint::blob_pointer_offset`] is `Some(_)`). Each header's trailing
///   `(blob_offset: u64, blob_len: u32)` references a slice in this file.
///
/// UTC day rotation. Idx interval is configurable (default 1024).
pub struct DiskStore<T: DataPoint> {
    root: PathBuf,
    key: SeriesKey,
    current_day: String,
    dat: BufWriter<File>,
    idx: BufWriter<File>,
    blob: Option<BufWriter<File>>,
    blob_pos: u64,
    records: u32,
    file_offset: u64,
    idx_every: u32,
    /// When `Some(n)`, day files older than `today - n` are swept on open and
    /// on day rotation. `None` = retention disabled (default, no behavior
    /// change). Wired from [`crate::persistence::PersistenceConfig::retention_days`].
    retention_days: Option<u32>,
    _phantom: PhantomData<T>,
}

impl<T: DataPoint> std::fmt::Debug for DiskStore<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiskStore")
            .field("root", &self.root)
            .field("key", &self.key)
            .field("day", &self.current_day)
            .field("records", &self.records)
            .field("offset", &self.file_offset)
            .field("blob_pos", &self.blob_pos)
            .field("retention_days", &self.retention_days)
            .finish()
    }
}

impl<T: DataPoint> DiskStore<T> {
    /// Open (or create) a DiskStore. `async` to match the wasm32 sibling API
    /// — the actual work is synchronous; the `async` wrapper is zero-cost.
    pub async fn new(storage_root: &Path, key: SeriesKey) -> std::io::Result<Self> {
        Self::with_idx_every(storage_root, key, 1024).await
    }

    /// Like `new` but with a custom idx sparsity factor.
    /// `async` to match the wasm32 sibling API — synchronous internally.
    pub async fn with_idx_every(
        storage_root: &Path,
        key: SeriesKey,
        idx_every: u32,
    ) -> std::io::Result<Self> {
        Self::with_idx_every_and_retention(storage_root, key, idx_every, None).await
    }

    /// Like `with_idx_every` but also takes an initial retention setting,
    /// applied via [`Self::sweep_retention`] immediately on open (trigger a).
    /// `async` to match the wasm32 sibling API — synchronous internally.
    pub async fn with_idx_every_and_retention(
        storage_root: &Path,
        key: SeriesKey,
        idx_every: u32,
        retention_days: Option<u32>,
    ) -> std::io::Result<Self> {
        let day = utc_today();
        let paths = paths(storage_root, &key, &day);
        let (dat, idx, offset) = open_pair(&paths.dat, &paths.idx)?;
        let (blob, blob_pos) = if T::blob_pointer_offset().is_some() {
            let (f, len) = open_blob(&paths.blob)?;
            (Some(BufWriter::new(f)), len)
        } else {
            (None, 0)
        };
        let store = Self {
            root: storage_root.to_path_buf(),
            key,
            current_day: day,
            dat: BufWriter::new(dat),
            idx: BufWriter::new(idx),
            blob,
            blob_pos,
            records: 0,
            file_offset: offset,
            idx_every: idx_every.max(1),
            retention_days,
            _phantom: PhantomData,
        };
        if let Some(days) = store.retention_days {
            if let Err(e) = store.sweep_retention(days) {
                tracing::warn!(
                    target: "dig3_station::disk_store",
                    error = ?e,
                    "retention sweep on open failed"
                );
            }
        }
        Ok(store)
    }

    /// Set (or clear) the retention window on an already-open store.
    /// Does NOT immediately sweep — the change takes effect at the next
    /// day-rotation trigger, or call [`Self::sweep_retention`] directly for
    /// an immediate pass. Builder-style: returns `self`.
    pub fn retention_days(mut self, days: Option<u32>) -> Self {
        self.retention_days = days;
        self
    }

    /// Append one record, possibly rotating to a new UTC day file.
    ///
    /// Write order: blob bytes first, header second. Crash mid-blob leaves
    /// orphan bytes (no header references them); crash between blob and
    /// header still references valid blob range only if the header itself
    /// was flushed — partial header is detected on read.
    pub fn append(&mut self, point: &T) -> std::io::Result<()> {
        self.rotate_if_new_day()?;

        let mut buf = vec![0u8; T::RECORD_SIZE];
        point.encode(&mut buf);

        if let (Some(blob_w), Some(tail_off)) = (self.blob.as_mut(), T::blob_pointer_offset()) {
            if let Some(blob_bytes) = point.encode_blob() {
                let off = self.blob_pos;
                let len = blob_bytes.len() as u32;
                blob_w.write_all(&blob_bytes)?;
                self.blob_pos += blob_bytes.len() as u64;
                buf[tail_off..tail_off + 8].copy_from_slice(&off.to_le_bytes());
                buf[tail_off + 8..tail_off + 12].copy_from_slice(&len.to_le_bytes());
            }
        }

        self.dat.write_all(&buf)?;

        if self.records % self.idx_every == 0 {
            let mut idx_buf = [0u8; 16];
            idx_buf[0..8].copy_from_slice(&(point.timestamp_ms() as u64).to_le_bytes());
            idx_buf[8..16].copy_from_slice(&self.file_offset.to_le_bytes());
            self.idx.write_all(&idx_buf)?;
        }

        self.records = self.records.wrapping_add(1);
        self.file_offset += T::RECORD_SIZE as u64;
        Ok(())
    }

    /// Append many points (batch). Same per-record semantics.
    pub fn append_batch(&mut self, points: &[T]) -> std::io::Result<()> {
        for p in points {
            self.append(p)?;
        }
        Ok(())
    }

    /// Flush OS write buffers. `async` to match the wasm32 sibling API —
    /// synchronous internally. The work completes before the future resolves.
    pub async fn flush(&mut self) -> std::io::Result<()> {
        self.flush_sync()
    }

    /// Synchronous flush — called from `Drop` where `async` is unavailable.
    fn flush_sync(&mut self) -> std::io::Result<()> {
        self.dat.flush()?;
        self.idx.flush()?;
        if let Some(b) = self.blob.as_mut() {
            b.flush()?;
        }
        Ok(())
    }

    /// Read up to `limit` most-recent records, walking backwards across
    /// previous UTC day files when the current day alone does not have
    /// enough. Used for warm-start. Returns oldest → newest order across ALL
    /// collected days — identical contract to the single-day version.
    ///
    /// Fast path: if the current day's file alone satisfies `limit`, no
    /// directory listing happens — behavior and cost are unchanged from
    /// before cross-day support existed.
    ///
    /// Slow path: when the current day yields fewer than `limit` records
    /// (including zero, e.g. right after UTC midnight rotation), the symbol
    /// directory is listed once, `.dat` filenames are parsed as `YYYY-MM-DD`
    /// dates, sorted descending, and walked strictly-older-than-current-day
    /// until either `limit` is satisfied or [`MAX_TAIL_DAYS`] distinct days
    /// (current day included) have been visited. Each visited day's records
    /// are read oldest→newest within that day, then the day-blocks are
    /// concatenated oldest-day-first so the final vector stays oldest→newest.
    ///
    /// For types with `blob_pointer_offset()`, each day's own companion
    /// `.blob` file is opened and stitched independently — blob offsets are
    /// per-day, never shared across day boundaries.
    ///
    /// `async` to match the wasm32 sibling API — synchronous internally.
    pub async fn read_tail(&self, limit: usize) -> std::io::Result<Vec<T>> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let current = read_day_tail::<T>(&self.root, &self.key, &self.current_day, limit)?;
        if current.len() >= limit {
            // Fast path: current day alone satisfies the request. No
            // directory listing, no older-day I/O.
            return Ok(current);
        }

        let mut remaining = limit - current.len();
        let mut older_blocks: Vec<Vec<T>> = Vec::new();
        let mut days_visited = 1usize; // current day already counted.

        if remaining > 0 && days_visited < MAX_TAIL_DAYS {
            let sym_dir = symbol_dir(&self.root, &self.key);
            let mut candidate_days = list_older_days(&sym_dir, &self.current_day);
            // Descending (most recent first) so we walk backwards in time.
            candidate_days.sort_unstable_by(|a, b| b.cmp(a));

            for day in candidate_days {
                if remaining == 0 || days_visited >= MAX_TAIL_DAYS {
                    break;
                }
                let day_str = day.format("%Y-%m-%d").to_string();
                let block = read_day_tail::<T>(&self.root, &self.key, &day_str, remaining)?;
                days_visited += 1;
                if block.is_empty() {
                    continue;
                }
                remaining = remaining.saturating_sub(block.len());
                older_blocks.push(block);
            }
        }

        // older_blocks is newest-older-day-first (we walked backwards);
        // reverse so the oldest visited day comes first, then append the
        // current (newest) day's block last. Final order: oldest → newest.
        older_blocks.reverse();
        let mut out = Vec::with_capacity(
            older_blocks.iter().map(Vec::len).sum::<usize>() + current.len(),
        );
        for block in older_blocks {
            out.extend(block);
        }
        out.extend(current);
        Ok(out)
    }

    pub fn key(&self) -> &SeriesKey {
        &self.key
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Delete `.dat`/`.idx`/`.blob` triplets for days strictly older than
    /// `today - retention_days`. No-op when `retention_days` is `None`.
    /// Never touches the current day's files, even if `retention_days == 0`
    /// (0 is treated as "keep only today", i.e. every OTHER day is swept).
    ///
    /// Called from [`Self::new`] (on open) and [`Self::rotate_if_new_day`]
    /// (on day rotation) — both cheap, non-timer triggers. Missing files are
    /// tolerated (best-effort sweep; a partially-deleted triplet from a prior
    /// crash is still cleaned up incrementally).
    fn sweep_retention(&self, retention_days: u32) -> std::io::Result<()> {
        let today = match NaiveDate::parse_from_str(&self.current_day, "%Y-%m-%d") {
            Ok(d) => d,
            Err(_) => return Ok(()),
        };
        let cutoff = today - chrono::Duration::days(retention_days as i64);

        let sym_dir = symbol_dir(&self.root, &self.key);
        let entries = match std::fs::read_dir(&sym_dir) {
            Ok(e) => e,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            let Ok(day) = NaiveDate::parse_from_str(stem, "%Y-%m-%d") else {
                continue;
            };
            if day >= cutoff {
                continue;
            }
            match std::fs::remove_file(&path) {
                Ok(()) => {}
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
                Err(e) => {
                    tracing::warn!(
                        target: "dig3_station::disk_store",
                        ?path, error = ?e,
                        "retention sweep: failed to remove stale file"
                    );
                }
            }
        }
        Ok(())
    }

    fn rotate_if_new_day(&mut self) -> std::io::Result<()> {
        let today = utc_today();
        if today == self.current_day {
            return Ok(());
        }
        self.flush_sync()?;
        let paths = paths(&self.root, &self.key, &today);
        let (dat, idx, offset) = open_pair(&paths.dat, &paths.idx)?;
        self.dat = BufWriter::new(dat);
        self.idx = BufWriter::new(idx);
        if T::blob_pointer_offset().is_some() {
            let (f, len) = open_blob(&paths.blob)?;
            self.blob = Some(BufWriter::new(f));
            self.blob_pos = len;
        } else {
            self.blob = None;
            self.blob_pos = 0;
        }
        self.records = 0;
        self.file_offset = offset;
        self.current_day = today;

        if let Some(days) = self.retention_days {
            if let Err(e) = self.sweep_retention(days) {
                tracing::warn!(
                    target: "dig3_station::disk_store",
                    error = ?e,
                    "retention sweep on day-rotation failed"
                );
            }
        }
        Ok(())
    }
}

impl<T: DataPoint> Drop for DiskStore<T> {
    fn drop(&mut self) {
        // flush_sync flushes OS write buffers synchronously — safe in Drop.
        // The public flush() is async (matches wasm32 sibling API) so cannot
        // be called from Drop; flush_sync() is the sync-only entry point.
        let _ = self.flush_sync();
    }
}

struct DayPaths {
    dat: PathBuf,
    idx: PathBuf,
    blob: PathBuf,
}

fn paths(root: &Path, key: &SeriesKey, day: &str) -> DayPaths {
    let dir = symbol_dir(root, key);
    DayPaths {
        dat: dir.join(format!("{day}.dat")),
        idx: dir.join(format!("{day}.idx")),
        blob: dir.join(format!("{day}.blob")),
    }
}

/// `<storage_root>/<kind-slug>/<exchange>/<account>/<symbol>/` — the directory
/// holding every day file for one `SeriesKey`.
fn symbol_dir(root: &Path, key: &SeriesKey) -> PathBuf {
    root.join(key.kind.slug())
        .join(key.exchange_label())
        .join(key.account_label())
        .join(key.symbol.to_lowercase())
}

/// List `.dat` filenames in `sym_dir` that parse as `YYYY-MM-DD` and are
/// strictly older than `current_day`. Used by the `read_tail` cross-day walk
/// and unrelated to write-path performance (only invoked when the current
/// day's file does not already satisfy the requested `limit`).
fn list_older_days(sym_dir: &Path, current_day: &str) -> Vec<NaiveDate> {
    let Ok(current) = NaiveDate::parse_from_str(current_day, "%Y-%m-%d") else {
        return Vec::new();
    };
    let entries = match std::fs::read_dir(sym_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };
    entries
        .flatten()
        .filter_map(|entry| {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("dat") {
                return None;
            }
            let stem = path.file_stem()?.to_str()?;
            let day = NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()?;
            if day < current {
                Some(day)
            } else {
                None
            }
        })
        .collect()
}

/// Read up to `limit` most-recent records from a single day's `.dat` (+
/// companion `.blob` if applicable). Returns oldest → newest order WITHIN
/// this one day. Empty `Vec` if the day file does not exist or is smaller
/// than one record. This is the single-day primitive `read_tail` composes
/// across multiple days.
fn read_day_tail<T: DataPoint>(
    root: &Path,
    key: &SeriesKey,
    day: &str,
    limit: usize,
) -> std::io::Result<Vec<T>> {
    let paths = paths(root, key, day);
    if !paths.dat.exists() {
        return Ok(Vec::new());
    }
    let mut f = File::open(&paths.dat)?;
    let total = f.metadata()?.len();
    if total < T::RECORD_SIZE as u64 {
        return Ok(Vec::new());
    }
    let max_records = (total / T::RECORD_SIZE as u64) as usize;
    let take = limit.min(max_records);
    let offset = total - (take as u64 * T::RECORD_SIZE as u64);
    f.seek(SeekFrom::Start(offset))?;
    let mut buf = vec![0u8; take * T::RECORD_SIZE];
    f.read_exact(&mut buf)?;

    let mut blob_file = match T::blob_pointer_offset() {
        Some(_) if paths.blob.exists() => Some(File::open(&paths.blob)?),
        _ => None,
    };
    let blob_len_total = blob_file
        .as_ref()
        .and_then(|f| f.metadata().ok())
        .map(|m| m.len())
        .unwrap_or(0);

    let mut out = Vec::with_capacity(take);
    for chunk in buf.chunks_exact(T::RECORD_SIZE) {
        let decoded = if let (Some(tail_off), Some(bf)) =
            (T::blob_pointer_offset(), blob_file.as_mut())
        {
            let off = u64::from_le_bytes(
                chunk[tail_off..tail_off + 8].try_into().unwrap_or([0u8; 8]),
            );
            let len = u32::from_le_bytes(
                chunk[tail_off + 8..tail_off + 12]
                    .try_into()
                    .unwrap_or([0u8; 4]),
            ) as u64;
            if len == 0 {
                T::decode_blob(chunk, &[])
            } else if off
                .checked_add(len)
                .map(|end| end > blob_len_total)
                .unwrap_or(true)
            {
                tracing::warn!(
                    target: "dig3_station::disk_store",
                    off, len, blob_len_total,
                    "blob pointer out of bounds, skipping record"
                );
                None
            } else {
                bf.seek(SeekFrom::Start(off))?;
                let mut blob_buf = vec![0u8; len as usize];
                if bf.read_exact(&mut blob_buf).is_err() {
                    None
                } else {
                    T::decode_blob(chunk, &blob_buf)
                }
            }
        } else {
            T::decode(chunk)
        };
        if let Some(p) = decoded {
            out.push(p);
        }
    }
    Ok(out)
}

fn open_pair(dat_path: &Path, idx_path: &Path) -> std::io::Result<(File, File, u64)> {
    if let Some(parent) = dat_path.parent() {
        create_dir_all(parent)?;
    }
    let dat = OpenOptions::new().create(true).append(true).open(dat_path)?;
    let offset = dat.metadata()?.len();
    let idx = OpenOptions::new().create(true).append(true).open(idx_path)?;
    Ok((dat, idx, offset))
}

fn open_blob(blob_path: &Path) -> std::io::Result<(File, u64)> {
    if let Some(parent) = blob_path.parent() {
        create_dir_all(parent)?;
    }
    let f = OpenOptions::new().create(true).append(true).open(blob_path)?;
    let len = f.metadata()?.len();
    Ok((f, len))
}

fn utc_today() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::TradePoint;
    use crate::series::Kind;
    use digdigdig3::core::types::{AccountType, ExchangeId};

    /// Isolated temp dir per test, cleaned up on drop.
    struct TmpRoot(PathBuf);

    impl TmpRoot {
        fn new(tag: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "dig3-store-test-{tag}-{}-{}",
                std::process::id(),
                chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0),
            ));
            let _ = create_dir_all(&dir);
            Self(dir)
        }

        fn path(&self) -> &Path {
            &self.0
        }
    }

    impl Drop for TmpRoot {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    fn trade(ts_ms: i64) -> TradePoint {
        TradePoint {
            ts_ms,
            price: 100.0,
            quantity: 1.0,
            side: 0,
            trade_id_hash: ts_ms as u64,
        }
    }

    /// Write a raw `.dat` file for `day` directly (bypassing `DiskStore`, which
    /// only ever writes to `utc_today()`). Mirrors the exact record layout
    /// `DiskStore::append` produces: `T::RECORD_SIZE` bytes per record, LE,
    /// no idx / blob needed for `TradePoint` (no blob_pointer_offset).
    fn write_day_file(root: &Path, key: &SeriesKey, day: &str, points: &[TradePoint]) {
        let p = paths(root, key, day);
        create_dir_all(p.dat.parent().unwrap()).unwrap();
        let mut buf = Vec::with_capacity(points.len() * TradePoint::RECORD_SIZE);
        for pt in points {
            let mut rec = vec![0u8; TradePoint::RECORD_SIZE];
            pt.encode(&mut rec);
            buf.extend_from_slice(&rec);
        }
        std::fs::write(&p.dat, &buf).unwrap();
    }

    fn test_key(symbol: &str) -> SeriesKey {
        SeriesKey::new(ExchangeId::Binance, AccountType::Spot, symbol, Kind::Trade)
    }

    fn days_ago(n: i64) -> String {
        (chrono::Utc::now().date_naive() - chrono::Duration::days(n))
            .format("%Y-%m-%d")
            .to_string()
    }

    #[tokio::test]
    async fn read_tail_spans_multiple_days_oldest_to_newest() {
        let tmp = TmpRoot::new("span");
        let key = test_key("BTCUSDT");

        // 2 days ago: 3 records. 1 day ago: 3 records. today (via DiskStore): 2.
        write_day_file(tmp.path(), &key, &days_ago(2), &[trade(1), trade(2), trade(3)]);
        write_day_file(tmp.path(), &key, &days_ago(1), &[trade(4), trade(5), trade(6)]);

        let mut store = DiskStore::<TradePoint>::new(tmp.path(), key.clone()).await.unwrap();
        store.append(&trade(7)).unwrap();
        store.append(&trade(8)).unwrap();
        store.flush().await.unwrap();

        // Ask for more than today alone has (2) — must cross into older days.
        let tail = store.read_tail(5).await.unwrap();
        let ts: Vec<i64> = tail.iter().map(|p| p.ts_ms).collect();
        assert_eq!(ts, vec![4, 5, 6, 7, 8], "oldest→newest across day boundary, exactly `limit` records");
    }

    #[tokio::test]
    async fn read_tail_respects_limit_across_days() {
        let tmp = TmpRoot::new("limit");
        let key = test_key("ETHUSDT");

        write_day_file(tmp.path(), &key, &days_ago(2), &[trade(1), trade(2), trade(3)]);
        write_day_file(tmp.path(), &key, &days_ago(1), &[trade(4), trade(5)]);

        let mut store = DiskStore::<TradePoint>::new(tmp.path(), key.clone()).await.unwrap();
        store.append(&trade(6)).unwrap();
        store.flush().await.unwrap();

        // limit=3 should pull from today(1) + yesterday(2), not touch 2-days-ago.
        let tail = store.read_tail(3).await.unwrap();
        let ts: Vec<i64> = tail.iter().map(|p| p.ts_ms).collect();
        assert_eq!(ts, vec![4, 5, 6]);
    }

    #[tokio::test]
    async fn read_tail_current_day_fast_path_skips_older_days() {
        let tmp = TmpRoot::new("fastpath");
        let key = test_key("BTCUSDT");

        // Deliberately malformed older-day file: if the fast path is hit
        // correctly, this file is never opened / parsed.
        let p = paths(tmp.path(), &key, &days_ago(1));
        create_dir_all(p.dat.parent().unwrap()).unwrap();
        std::fs::write(&p.dat, b"not a valid record stream at all, wrong size").unwrap();

        let mut store = DiskStore::<TradePoint>::new(tmp.path(), key.clone()).await.unwrap();
        for i in 1..=5 {
            store.append(&trade(i)).unwrap();
        }
        store.flush().await.unwrap();

        let tail = store.read_tail(3).await.unwrap();
        let ts: Vec<i64> = tail.iter().map(|p| p.ts_ms).collect();
        assert_eq!(ts, vec![3, 4, 5]);
    }

    #[tokio::test]
    async fn read_tail_stops_at_max_tail_days() {
        let tmp = TmpRoot::new("maxdays");
        let key = test_key("BTCUSDT");

        // Populate MAX_TAIL_DAYS + 2 days of single-record history, oldest
        // far beyond the walk bound. Only the newest MAX_TAIL_DAYS-1 older
        // days (plus "today", opened via DiskStore below with 0 records)
        // should ever be visited.
        // ts_ms is chosen to DECREASE as `n` (days-ago) increases, so
        // numeric-ascending == chronological-ascending: the most recent
        // synthetic day (n=1, "yesterday") gets the LARGEST timestamp.
        let total_older_days = MAX_TAIL_DAYS + 2;
        for n in 1..=total_older_days {
            let ts_ms = 100_000 - n as i64;
            write_day_file(tmp.path(), &key, &days_ago(n as i64), &[trade(ts_ms)]);
        }

        // Today's DiskStore has zero records — forces the walk to go as far
        // back as MAX_TAIL_DAYS allows while trying to satisfy a huge limit.
        let store = DiskStore::<TradePoint>::new(tmp.path(), key.clone()).await.unwrap();

        let tail = store.read_tail(1000).await.unwrap();
        // Current day (empty) counts as 1 visited day; at most
        // MAX_TAIL_DAYS - 1 older days are walked → at most MAX_TAIL_DAYS - 1
        // records recovered (1 record per synthetic day).
        assert!(
            tail.len() <= MAX_TAIL_DAYS - 1,
            "expected at most {} records bounded by MAX_TAIL_DAYS, got {}",
            MAX_TAIL_DAYS - 1,
            tail.len()
        );
        assert!(!tail.is_empty(), "walk should have recovered at least the nearest older days");

        // The recovered slice must be the NEWEST eligible older days (walk
        // starts from yesterday backwards), oldest→newest ordering preserved.
        let ts: Vec<i64> = tail.iter().map(|p| p.ts_ms).collect();
        let mut sorted = ts.clone();
        sorted.sort_unstable();
        assert_eq!(ts, sorted, "must stay oldest→newest");
        // Newest recovered record must correspond to "1 day ago" (n=1 ->
        // ts_ms = 99_999), since the walk starts nearest-first and the walk
        // never reaches days older than MAX_TAIL_DAYS - 1.
        assert_eq!(*ts.last().unwrap(), 100_000 - 1);
    }

    #[test]
    fn sweep_retention_removes_only_stale_day_files() {
        let tmp = TmpRoot::new("retention");
        let key = test_key("BTCUSDT");

        let today = utc_today();
        let old_day = days_ago(40);
        let fresh_day = days_ago(5);

        let old_paths = paths(tmp.path(), &key, &old_day);
        let fresh_paths = paths(tmp.path(), &key, &fresh_day);
        let today_paths = paths(tmp.path(), &key, &today);
        create_dir_all(old_paths.dat.parent().unwrap()).unwrap();
        std::fs::write(&old_paths.dat, b"old").unwrap();
        std::fs::write(&fresh_paths.dat, b"fresh").unwrap();
        std::fs::write(&today_paths.dat, b"today").unwrap();

        // Build a DiskStore instance purely to call the private
        // sweep_retention method — bypass async new() to avoid opening
        // today's file twice; construct via with_idx_every_and_retention
        // synchronously through block_on since sweep itself is sync.
        let rt = tokio::runtime::Runtime::new().unwrap();
        let store = rt
            .block_on(DiskStore::<TradePoint>::with_idx_every_and_retention(
                tmp.path(),
                key.clone(),
                1024,
                None, // no auto-sweep on open; we call sweep_retention directly below
            ))
            .unwrap();

        store.sweep_retention(30).unwrap();

        assert!(!old_paths.dat.exists(), "40-day-old file must be swept at retention_days=30");
        assert!(fresh_paths.dat.exists(), "5-day-old file must survive retention_days=30");
        assert!(today_paths.dat.exists(), "current day file must never be swept");
    }

    #[tokio::test]
    async fn sweep_retention_runs_automatically_on_open_when_configured() {
        let tmp = TmpRoot::new("retention-auto");
        let key = test_key("BTCUSDT");

        let old_day = days_ago(100);
        let old_paths = paths(tmp.path(), &key, &old_day);
        create_dir_all(old_paths.dat.parent().unwrap()).unwrap();
        std::fs::write(&old_paths.dat, b"stale").unwrap();

        let _store = DiskStore::<TradePoint>::with_idx_every_and_retention(
            tmp.path(),
            key.clone(),
            1024,
            Some(30),
        )
        .await
        .unwrap();

        assert!(!old_paths.dat.exists(), "retention_days=Some(30) must sweep on open");
    }
}
