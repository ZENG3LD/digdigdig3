use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use super::{DataPoint, SeriesKey};

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
            .finish()
    }
}

impl<T: DataPoint> DiskStore<T> {
    pub fn new(storage_root: &Path, key: SeriesKey) -> std::io::Result<Self> {
        Self::with_idx_every(storage_root, key, 1024)
    }

    pub fn with_idx_every(
        storage_root: &Path,
        key: SeriesKey,
        idx_every: u32,
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
        Ok(Self {
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
            _phantom: PhantomData,
        })
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

    pub fn flush(&mut self) -> std::io::Result<()> {
        self.dat.flush()?;
        self.idx.flush()?;
        if let Some(b) = self.blob.as_mut() {
            b.flush()?;
        }
        Ok(())
    }

    /// Read up to `limit` most-recent records from the current day's `.dat`.
    /// Used for warm-start. Returns oldest → newest order.
    ///
    /// For types with `blob_pointer_offset()`, opens the `.blob` file
    /// read-only and reconstructs string fields via [`DataPoint::decode_blob`].
    pub fn read_tail(&self, limit: usize) -> std::io::Result<Vec<T>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let paths = paths(&self.root, &self.key, &self.current_day);
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

    pub fn key(&self) -> &SeriesKey {
        &self.key
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn rotate_if_new_day(&mut self) -> std::io::Result<()> {
        let today = utc_today();
        if today == self.current_day {
            return Ok(());
        }
        self.flush()?;
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
        Ok(())
    }
}

impl<T: DataPoint> Drop for DiskStore<T> {
    fn drop(&mut self) {
        let _ = self.flush();
    }
}

struct DayPaths {
    dat: PathBuf,
    idx: PathBuf,
    blob: PathBuf,
}

fn paths(root: &Path, key: &SeriesKey, day: &str) -> DayPaths {
    let dir = root
        .join(key.kind.slug())
        .join(key.exchange_label())
        .join(key.account_label())
        .join(key.symbol.to_lowercase());
    DayPaths {
        dat: dir.join(format!("{day}.dat")),
        idx: dir.join(format!("{day}.idx")),
        blob: dir.join(format!("{day}.blob")),
    }
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
