//! OPFS-backed `DiskStore<T>` for wasm32 — Option C: in-memory write buffer
//! with periodic async `createWritable` flush to Origin Private File System.
//!
//! Design doc: `docs/research/wasm-wave3/opfs-disk-store.md` §7.
//!
//! # Architecture (Option C)
//!
//! - `append()` and `append_batch()` are **synchronous** — bytes accumulate in
//!   a `Vec<u8>` in-memory write buffer. No OPFS I/O happens on the hot path.
//! - `flush()` drains the buffer to OPFS via `createWritable(keepExistingData:
//!   true)` + seek-to-end + write + close. One async round-trip per flush
//!   interval, not per record.
//! - `read_tail()` reads the current day's `.dat` from OPFS. Full file read;
//!   acceptable for files up to a few MB (well within 1-day market data volume).
//! - Day rotation is detected in `flush()`, not `append()`. Records written
//!   after midnight go to the new-day file at the next flush (max one-interval
//!   drift — acceptable per design spec §9).
//!
//! # Drop behaviour
//!
//! **`Drop` cannot be async** — JS will not await it. A sync `Drop` impl
//! cannot call `createWritable` (Promise-based). Any data buffered at drop time
//! will be lost unless the caller:
//!   a) calls `flush().await` explicitly before dropping, or
//!   b) hooks `visibilitychange` / `beforeunload` and calls `flush().await`
//!      from there (Station's periodic-flush mechanism covers the hot path).
//! This is documented behaviour; not a bug. The trading terminal has a
//! station-level periodic flush that covers the hot path.

use web_sys::FileSystemDirectoryHandle;

use crate::opfs_helpers::{
    opfs_append_quota_guard, opfs_file_size, opfs_read_all, opfs_root,
};

pub use crate::opfs_helpers::OpfsError;

use super::{DataPoint, SeriesKey};

// ─── Walk helper (SeriesKey layout) ──────────────────────────────────────────

/// Walk (or create) the four-segment directory for a `SeriesKey`.
///
/// Path: `<root>/<kind-slug>/<exchange>/<account>/<symbol-lowercase>/`
async fn opfs_walk_or_create(
    root: &FileSystemDirectoryHandle,
    key: &SeriesKey,
) -> Result<FileSystemDirectoryHandle, OpfsError> {
    crate::opfs_helpers::opfs_walk_or_create_stream(
        root,
        &key.kind.slug(),
        &key.exchange_label(),
        &key.account_label(),
        &key.symbol,
    )
    .await
}

// ─── Main struct ──────────────────────────────────────────────────────────────

/// OPFS-backed binary time-series store for wasm32.
///
/// Public API is call-site-compatible with the native `DiskStore<T>`:
/// - `new` / `flush` / `read_tail` are `async`.
/// - `append` / `append_batch` are sync (buffer to memory).
///
/// The native counterpart (`store.rs`) mirrors these signatures after the
/// Workstream C widening — zero-cost on native (trivially-async wrappers).
pub struct DiskStore<T: DataPoint> {
    key: SeriesKey,
    current_day: String,
    /// Cached OPFS handle for the symbol leaf directory.
    /// Re-obtained in `flush()` when day rotation is detected.
    sym_dir: FileSystemDirectoryHandle,
    /// In-memory buffer for `.dat` records pending flush.
    dat_buf: Vec<u8>,
    /// In-memory buffer for `.idx` sparse index entries pending flush.
    idx_buf: Vec<u8>,
    /// In-memory buffer for `.blob` variable-length bytes pending flush.
    /// `None` when `T::blob_pointer_offset()` returns `None`.
    blob_buf: Option<Vec<u8>>,
    /// Logical position of the next record in the persisted `.dat` file.
    /// Tracks the on-disk file size + buffered bytes size so idx entries have
    /// the correct offset even before a flush.
    file_offset: u64,
    /// Current end position of the `.blob` file (on-disk + buffered).
    blob_pos: u64,
    /// Total records appended (wrapping). Used for idx sparsity.
    records: u32,
    /// Write one idx entry every N records.
    idx_every: u32,
    _phantom: std::marker::PhantomData<T>,
}

// ─── impl DiskStore ───────────────────────────────────────────────────────────

impl<T: DataPoint> DiskStore<T> {
    /// Open (or create) a DiskStore for `key` in OPFS.
    ///
    /// - Walks / creates the four-level directory path under the OPFS root.
    /// - Calls `navigator.storage.persist()` once (best-effort; ignores result).
    /// - Reads the current day's `.dat` file size to resume offset tracking.
    pub async fn new(key: SeriesKey) -> Result<Self, OpfsError> {
        Self::with_idx_every(key, 1024).await
    }

    /// Like `new` but with a custom idx sparsity factor.
    pub async fn with_idx_every(key: SeriesKey, idx_every: u32) -> Result<Self, OpfsError> {
        let storage = web_sys::window()
            .ok_or(OpfsError::NoWindow)?
            .navigator()
            .storage();

        // Request persistent storage (best-effort; browsers may silently deny).
        if let Ok(p) = storage.persist() {
            let _ = wasm_bindgen_futures::JsFuture::from(p).await;
        }

        let root = opfs_root().await?;
        let day = utc_today_wasm();
        let sym_dir = opfs_walk_or_create(&root, &key).await?;

        let file_offset = opfs_file_size(&sym_dir, &format!("{day}.dat")).await?;
        let blob_pos = if T::blob_pointer_offset().is_some() {
            opfs_file_size(&sym_dir, &format!("{day}.blob")).await?
        } else {
            0
        };

        Ok(Self {
            key,
            current_day: day,
            sym_dir,
            dat_buf: Vec::new(),
            idx_buf: Vec::new(),
            blob_buf: if T::blob_pointer_offset().is_some() {
                Some(Vec::new())
            } else {
                None
            },
            file_offset,
            blob_pos,
            records: 0,
            idx_every: idx_every.max(1),
            _phantom: std::marker::PhantomData,
        })
    }

    /// Encode and buffer one record. No OPFS I/O — sync, zero-latency hot path.
    ///
    /// Day rotation is deferred to `flush()`. Records written after midnight
    /// will land in the new-day file at the next flush (max one-flush-interval
    /// drift).
    pub fn append(&mut self, point: &T) {
        let mut buf = vec![0u8; T::RECORD_SIZE];
        point.encode(&mut buf);

        if let (Some(blob_w), Some(tail_off)) =
            (self.blob_buf.as_mut(), T::blob_pointer_offset())
        {
            if let Some(blob_bytes) = point.encode_blob() {
                let off = self.blob_pos;
                let len = blob_bytes.len() as u32;
                blob_w.extend_from_slice(&blob_bytes);
                self.blob_pos += blob_bytes.len() as u64;
                buf[tail_off..tail_off + 8].copy_from_slice(&off.to_le_bytes());
                buf[tail_off + 8..tail_off + 12].copy_from_slice(&len.to_le_bytes());
            }
        }

        if self.records % self.idx_every == 0 {
            let mut idx_entry = [0u8; 16];
            idx_entry[0..8].copy_from_slice(&(point.timestamp_ms() as u64).to_le_bytes());
            idx_entry[8..16].copy_from_slice(&self.file_offset.to_le_bytes());
            self.idx_buf.extend_from_slice(&idx_entry);
        }

        self.dat_buf.extend_from_slice(&buf);
        self.records = self.records.wrapping_add(1);
        self.file_offset += T::RECORD_SIZE as u64;
    }

    /// Buffer multiple records. Equivalent to calling `append` in a loop.
    pub fn append_batch(&mut self, points: &[T]) {
        for p in points {
            self.append(p);
        }
    }

    /// Flush in-memory buffers to OPFS.
    ///
    /// Detects day rotation: if the current UTC date has changed since `new()`
    /// or the last rotation, acquires new file handles for the new day and
    /// flushes to those files instead. Offset tracking resets.
    ///
    /// On `QuotaExceededError` from the browser, the write is skipped with a
    /// warning. Lost records are acceptable for market-data capture; quota will
    /// not free itself until the user clears site data.
    pub async fn flush(&mut self) -> Result<(), OpfsError> {
        let today = utc_today_wasm();
        if today != self.current_day {
            if let Err(e) = self.rotate_day(&today).await {
                tracing::warn!(
                    target: "dig3_station::disk_store_wasm",
                    error = %e,
                    old_day = %self.current_day,
                    new_day = %today,
                    "OPFS day rotation failed; keeping old day handle"
                );
            }
        }

        if !self.dat_buf.is_empty() {
            opfs_append_quota_guard(
                &self.sym_dir,
                &format!("{}.dat", self.current_day),
                &self.dat_buf,
            )
            .await?;
            self.dat_buf.clear();
        }
        if !self.idx_buf.is_empty() {
            opfs_append_quota_guard(
                &self.sym_dir,
                &format!("{}.idx", self.current_day),
                &self.idx_buf,
            )
            .await?;
            self.idx_buf.clear();
        }
        if let Some(b) = self.blob_buf.as_mut() {
            if !b.is_empty() {
                opfs_append_quota_guard(
                    &self.sym_dir,
                    &format!("{}.blob", self.current_day),
                    b,
                )
                .await?;
                b.clear();
            }
        }
        Ok(())
    }

    /// Read up to `limit` most-recent records from the current day's `.dat` in
    /// OPFS. The full file is read into memory (sliced at the tail). Acceptable
    /// for files up to a few MB; called once at warm-start.
    ///
    /// Returns an empty `Vec` when the file does not exist yet.
    pub async fn read_tail(&self, limit: usize) -> Result<Vec<T>, OpfsError> {
        if limit == 0 {
            return Ok(Vec::new());
        }

        let dat_bytes =
            match opfs_read_all(&self.sym_dir, &format!("{}.dat", self.current_day)).await {
                Ok(b) => b,
                Err(OpfsError::FileNotFound) => return Ok(Vec::new()),
                Err(e) => return Err(e),
            };

        let total = dat_bytes.len();
        if total < T::RECORD_SIZE {
            return Ok(Vec::new());
        }
        let max_records = total / T::RECORD_SIZE;
        let take = limit.min(max_records);
        let offset = total - take * T::RECORD_SIZE;
        let slice = &dat_bytes[offset..];

        let blob_data: Option<Vec<u8>> = if T::blob_pointer_offset().is_some() {
            opfs_read_all(&self.sym_dir, &format!("{}.blob", self.current_day))
                .await
                .ok()
        } else {
            None
        };

        let mut out = Vec::with_capacity(take);
        for chunk in slice.chunks_exact(T::RECORD_SIZE) {
            let point = match (T::blob_pointer_offset(), &blob_data) {
                (Some(tail_off), Some(bd)) => {
                    let off = u64::from_le_bytes(
                        chunk[tail_off..tail_off + 8]
                            .try_into()
                            .unwrap_or([0u8; 8]),
                    ) as usize;
                    let len = u32::from_le_bytes(
                        chunk[tail_off + 8..tail_off + 12]
                            .try_into()
                            .unwrap_or([0u8; 4]),
                    ) as usize;
                    let blob_slice = bd.get(off..off + len).unwrap_or(&[]);
                    T::decode_blob(chunk, blob_slice)
                }
                _ => T::decode(chunk),
            };
            if let Some(p) = point {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// Return the `SeriesKey` this store is writing for.
    pub fn key(&self) -> &SeriesKey {
        &self.key
    }

    // ─── private helpers ─────────────────────────────────────────────────────

    /// Rotate to a new UTC day: flush pending buffers to old day, then acquire
    /// new OPFS handles and reset offset tracking.
    async fn rotate_day(&mut self, new_day: &str) -> Result<(), OpfsError> {
        if !self.dat_buf.is_empty() {
            opfs_append_quota_guard(
                &self.sym_dir,
                &format!("{}.dat", self.current_day),
                &self.dat_buf,
            )
            .await
            .map_err(|e| OpfsError::RotationFailed(Box::new(e)))?;
            self.dat_buf.clear();
        }
        if !self.idx_buf.is_empty() {
            opfs_append_quota_guard(
                &self.sym_dir,
                &format!("{}.idx", self.current_day),
                &self.idx_buf,
            )
            .await
            .map_err(|e| OpfsError::RotationFailed(Box::new(e)))?;
            self.idx_buf.clear();
        }
        if let Some(b) = self.blob_buf.as_mut() {
            if !b.is_empty() {
                opfs_append_quota_guard(
                    &self.sym_dir,
                    &format!("{}.blob", self.current_day),
                    b,
                )
                .await
                .map_err(|e| OpfsError::RotationFailed(Box::new(e)))?;
                b.clear();
            }
        }

        let root = opfs_root()
            .await
            .map_err(|e| OpfsError::RotationFailed(Box::new(e)))?;

        let new_sym_dir = opfs_walk_or_create(&root, &self.key)
            .await
            .map_err(|e| OpfsError::RotationFailed(Box::new(e)))?;

        let new_offset = opfs_file_size(&new_sym_dir, &format!("{new_day}.dat"))
            .await
            .unwrap_or(0);
        let new_blob_pos = if T::blob_pointer_offset().is_some() {
            opfs_file_size(&new_sym_dir, &format!("{new_day}.blob"))
                .await
                .unwrap_or(0)
        } else {
            0
        };

        self.sym_dir = new_sym_dir;
        self.current_day = new_day.to_string();
        self.file_offset = new_offset;
        self.blob_pos = new_blob_pos;
        self.records = 0;
        Ok(())
    }
}

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Return today's UTC date as `YYYY-MM-DD` string via chrono + wasmbind.
fn utc_today_wasm() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d").to_string()
}
