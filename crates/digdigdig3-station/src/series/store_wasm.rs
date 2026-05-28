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

use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use web_sys::{
    FileSystemCreateWritableOptions, FileSystemDirectoryHandle, FileSystemFileHandle,
    FileSystemGetDirectoryOptions, FileSystemGetFileOptions, FileSystemWritableFileStream,
    WriteCommandType, WriteParams,
};

use super::{DataPoint, SeriesKey};

// ─── Error type ──────────────────────────────────────────────────────────────

/// Errors from the wasm OPFS DiskStore.
#[derive(Debug)]
pub enum OpfsError {
    /// `window()` returned `None` — called outside browser context.
    NoWindow,
    /// An OPFS operation returned a JS exception (`JsValue`).
    Js(wasm_bindgen::JsValue),
    /// File not found in OPFS (not a hard error; read returns empty Vec).
    FileNotFound,
    /// Day rotation failed during flush.
    RotationFailed(Box<OpfsError>),
}

impl std::fmt::Display for OpfsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OpfsError::NoWindow => write!(f, "OPFS: no browser window"),
            OpfsError::Js(v) => write!(f, "OPFS JS error: {:?}", v),
            OpfsError::FileNotFound => write!(f, "OPFS: file not found"),
            OpfsError::RotationFailed(inner) => write!(f, "OPFS: day rotation failed: {inner}"),
        }
    }
}

impl std::error::Error for OpfsError {}

impl From<wasm_bindgen::JsValue> for OpfsError {
    fn from(v: wasm_bindgen::JsValue) -> Self {
        OpfsError::Js(v)
    }
}

// ─── Main struct ─────────────────────────────────────────────────────────────

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
            let _ = JsFuture::from(p).await; // ignore grant/deny result
        }

        let root: FileSystemDirectoryHandle =
            JsFuture::from(storage.get_directory()).await?.dyn_into()?;

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
        // Flush remaining old-day data before switching handles.
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

        // Acquire the OPFS root again for the new day (same directory path).
        let storage = web_sys::window()
            .ok_or(OpfsError::NoWindow)?
            .navigator()
            .storage();
        let root: FileSystemDirectoryHandle =
            JsFuture::from(storage.get_directory()).await?.dyn_into()?;

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

// ─── OPFS helper functions ────────────────────────────────────────────────────

/// Walk (or create) the four-segment directory path for `key` under `root`.
///
/// Path structure mirrors the native `paths()` helper:
/// `<root>/<kind-slug>/<exchange>/<account>/<symbol-lowercase>/`
async fn opfs_walk_or_create(
    root: &FileSystemDirectoryHandle,
    key: &SeriesKey,
) -> Result<FileSystemDirectoryHandle, OpfsError> {
    let opts = FileSystemGetDirectoryOptions::new();
    opts.set_create(true);

    let kind_slug = key.kind.slug();
    let kind_dir: FileSystemDirectoryHandle = JsFuture::from(
        root.get_directory_handle_with_options(&kind_slug, &opts),
    )
    .await?
    .dyn_into()?;

    let exch_dir: FileSystemDirectoryHandle = JsFuture::from(
        kind_dir.get_directory_handle_with_options(&key.exchange_label(), &opts),
    )
    .await?
    .dyn_into()?;

    let acct_dir: FileSystemDirectoryHandle = JsFuture::from(
        exch_dir.get_directory_handle_with_options(&key.account_label(), &opts),
    )
    .await?
    .dyn_into()?;

    let sym_dir: FileSystemDirectoryHandle = JsFuture::from(
        acct_dir.get_directory_handle_with_options(&key.symbol.to_lowercase(), &opts),
    )
    .await?
    .dyn_into()?;

    Ok(sym_dir)
}

/// Append `data` to `name` inside `dir` via `createWritable(keepExistingData:
/// true)` + seek-to-end + write + close.
///
/// `keepExistingData: true` preserves existing file bytes but positions the
/// write cursor at byte 0. We explicitly seek to `file.size()` before writing
/// to avoid overwriting existing data.
///
/// This is the correct OPFS append pattern on the main thread (no
/// `FileSystemSyncAccessHandle` available outside of DedicatedWorker).
async fn opfs_append(
    dir: &FileSystemDirectoryHandle,
    name: &str,
    data: &[u8],
) -> Result<(), OpfsError> {
    let fopts = FileSystemGetFileOptions::new();
    fopts.set_create(true);

    let fh: FileSystemFileHandle =
        JsFuture::from(dir.get_file_handle_with_options(name, &fopts))
            .await?
            .dyn_into()?;

    // Read current file size BEFORE opening writable (createWritable opens a
    // swap copy — file.size() on the swap copy reflects the pre-write size).
    let file_obj: web_sys::File = JsFuture::from(fh.get_file()).await?.dyn_into()?;
    let existing_size = file_obj.size() as u64;

    let write_opts = FileSystemCreateWritableOptions::new();
    write_opts.set_keep_existing_data(true);

    let writable: FileSystemWritableFileStream =
        JsFuture::from(fh.create_writable_with_options(&write_opts))
            .await?
            .dyn_into()?;

    // Seek to end — required because keepExistingData starts cursor at 0.
    // Use the typed WriteParams dictionary with WriteCommandType::Seek.
    let seek_params = WriteParams::new(WriteCommandType::Seek);
    seek_params.set_position(Some(existing_size as f64));
    JsFuture::from(
        writable
            .write_with_write_params(&seek_params)
            .map_err(|e| OpfsError::Js(e))?,
    )
    .await?;

    // Write the buffered bytes.
    let arr = js_sys::Uint8Array::from(data);
    JsFuture::from(
        writable
            .write_with_buffer_source(&arr)
            .map_err(|e| OpfsError::Js(e))?,
    )
    .await?;

    // Commit (close atomically swaps the temp file into place).
    JsFuture::from(writable.close()).await?;
    Ok(())
}

/// Wrapper around `opfs_append` that catches `QuotaExceededError` and logs a
/// warning instead of propagating it. Any other error is returned.
async fn opfs_append_quota_guard(
    dir: &FileSystemDirectoryHandle,
    name: &str,
    data: &[u8],
) -> Result<(), OpfsError> {
    match opfs_append(dir, name, data).await {
        Ok(()) => Ok(()),
        Err(OpfsError::Js(ref jsval)) => {
            // Check for QuotaExceededError by name.
            let is_quota = js_sys::Reflect::get(jsval, &"name".into())
                .ok()
                .and_then(|v| v.as_string())
                .map(|n| n == "QuotaExceededError")
                .unwrap_or(false);
            if is_quota {
                tracing::warn!(
                    target: "dig3_station::disk_store_wasm",
                    file = name,
                    bytes = data.len(),
                    "OPFS write skipped: QuotaExceededError — browser storage quota exceeded"
                );
                Ok(()) // drop the record, continue
            } else {
                Err(OpfsError::Js(jsval.clone()))
            }
        }
        Err(e) => Err(e),
    }
}

/// Read the entire content of `name` in `dir` as `Vec<u8>`.
///
/// Returns `Err(OpfsError::FileNotFound)` when the file does not exist (the
/// OPFS `getFileHandle` call rejects with `NotFoundError`).
async fn opfs_read_all(
    dir: &FileSystemDirectoryHandle,
    name: &str,
) -> Result<Vec<u8>, OpfsError> {
    let fh_result = JsFuture::from(dir.get_file_handle(name)).await;
    let fh: FileSystemFileHandle = match fh_result {
        Ok(v) => v.dyn_into().map_err(OpfsError::Js)?,
        Err(jsval) => {
            // NotFoundError → FileNotFound; anything else is a real error.
            let is_not_found = js_sys::Reflect::get(&jsval, &"name".into())
                .ok()
                .and_then(|v| v.as_string())
                .map(|n| n == "NotFoundError")
                .unwrap_or(false);
            if is_not_found {
                return Err(OpfsError::FileNotFound);
            }
            return Err(OpfsError::Js(jsval));
        }
    };

    let file: web_sys::File = JsFuture::from(fh.get_file()).await?.dyn_into()?;
    let ab: js_sys::ArrayBuffer = JsFuture::from(file.array_buffer()).await?.dyn_into()?;
    Ok(js_sys::Uint8Array::new(&ab).to_vec())
}

/// Return the current byte size of `name` in `dir`.
///
/// Returns `0` when the file does not exist (i.e. fresh session).
async fn opfs_file_size(
    dir: &FileSystemDirectoryHandle,
    name: &str,
) -> Result<u64, OpfsError> {
    let fh_result = JsFuture::from(dir.get_file_handle(name)).await;
    let fh: FileSystemFileHandle = match fh_result {
        Ok(v) => v.dyn_into().map_err(OpfsError::Js)?,
        Err(jsval) => {
            let is_not_found = js_sys::Reflect::get(&jsval, &"name".into())
                .ok()
                .and_then(|v| v.as_string())
                .map(|n| n == "NotFoundError")
                .unwrap_or(false);
            if is_not_found {
                return Ok(0);
            }
            return Err(OpfsError::Js(jsval));
        }
    };

    let file: web_sys::File = JsFuture::from(fh.get_file()).await?.dyn_into()?;
    Ok(file.size() as u64)
}

/// Return today's UTC date as `YYYY-MM-DD` string via `js_sys::Date`.
///
/// Uses `Date.now()` which reflects UTC time. Mirrors native `utc_today()` but
/// without `chrono` stdlib — `chrono` with `wasmbind` feature is available in
/// this crate's wasm32 dep block, so we use it for consistency.
fn utc_today_wasm() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d").to_string()
}
