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
///
/// UTC day rotation. Idx interval is configurable (default 1024).
pub struct DiskStore<T: DataPoint> {
    root: PathBuf,
    key: SeriesKey,
    current_day: String,
    dat: BufWriter<File>,
    idx: BufWriter<File>,
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
        let (dat_path, idx_path) = paths(storage_root, &key, &day);
        let (dat, idx, offset) = open_pair(&dat_path, &idx_path)?;
        Ok(Self {
            root: storage_root.to_path_buf(),
            key,
            current_day: day,
            dat: BufWriter::new(dat),
            idx: BufWriter::new(idx),
            records: 0,
            file_offset: offset,
            idx_every: idx_every.max(1),
            _phantom: PhantomData,
        })
    }

    /// Append one record, possibly rotating to a new UTC day file.
    pub fn append(&mut self, point: &T) -> std::io::Result<()> {
        self.rotate_if_new_day()?;

        let mut buf = vec![0u8; T::RECORD_SIZE];
        point.encode(&mut buf);
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
        Ok(())
    }

    /// Read up to `limit` most-recent records from the current day's `.dat`.
    /// Used for warm-start. Returns oldest → newest order.
    pub fn read_tail(&self, limit: usize) -> std::io::Result<Vec<T>> {
        if limit == 0 {
            return Ok(Vec::new());
        }
        let (dat_path, _) = paths(&self.root, &self.key, &self.current_day);
        if !dat_path.exists() {
            return Ok(Vec::new());
        }
        let mut f = File::open(&dat_path)?;
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
        let mut out = Vec::with_capacity(take);
        for chunk in buf.chunks_exact(T::RECORD_SIZE) {
            if let Some(p) = T::decode(chunk) {
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
        let (dat_path, idx_path) = paths(&self.root, &self.key, &today);
        let (dat, idx, offset) = open_pair(&dat_path, &idx_path)?;
        self.dat = BufWriter::new(dat);
        self.idx = BufWriter::new(idx);
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

fn paths(root: &Path, key: &SeriesKey, day: &str) -> (PathBuf, PathBuf) {
    let dir = root
        .join(key.kind.slug())
        .join(key.exchange_label())
        .join(key.account_label())
        .join(key.symbol.to_lowercase());
    let dat = dir.join(format!("{day}.dat"));
    let idx = dir.join(format!("{day}.idx"));
    (dat, idx)
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

fn utc_today() -> String {
    use chrono::Utc;
    Utc::now().format("%Y-%m-%d").to_string()
}
