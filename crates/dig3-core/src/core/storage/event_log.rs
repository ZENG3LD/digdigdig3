//! EventLog — append-only binary record store.
//!
//! Format compatible with mli StorageRoot. Each record:
//!   [i64 ts_ms LE][u32 payload_len LE][payload_bytes]
//!
//! File layout: `{root}/{symbol}/{stream_kind}.bin`
//!
//! Append-only. Truncated tail records are silently skipped on read.

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::PathBuf;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};

/// Append-only binary event log, one file per `(symbol, stream_kind)`.
pub struct EventLog {
    root: PathBuf,
}

/// A single record to append.
pub struct EventRecord<'a> {
    /// Exchange timestamp in milliseconds (UTC).
    pub ts_ms: i64,
    /// Raw payload bytes (caller serialises to JSON before passing in).
    pub payload: &'a [u8],
}

impl EventLog {
    /// Create an `EventLog` rooted at `root`. Creates the directory if absent.
    pub fn new(root: impl Into<PathBuf>) -> std::io::Result<Self> {
        let root = root.into();
        create_dir_all(&root)?;
        Ok(Self { root })
    }

    /// Append one record to `{symbol}/{stream_kind}.bin`.
    ///
    /// Creates subdirectory and file on first call.
    pub fn append(
        &self,
        symbol: &str,
        stream_kind: &str,
        record: &EventRecord<'_>,
    ) -> std::io::Result<()> {
        let dir = self.root.join(symbol);
        create_dir_all(&dir)?;
        let path = dir.join(format!("{stream_kind}.bin"));

        let mut f = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;

        let mut buf = Vec::with_capacity(8 + 4 + record.payload.len());
        buf.write_i64::<LittleEndian>(record.ts_ms)?;
        buf.write_u32::<LittleEndian>(record.payload.len() as u32)?;
        buf.write_all(record.payload)?;
        f.write_all(&buf)?;
        Ok(())
    }

    /// Read all records from `{symbol}/{stream_kind}.bin`.
    ///
    /// Returns `(ts_ms, payload_bytes)` pairs. Truncated tail records are
    /// silently skipped (truncation-safe).
    pub fn read_all(
        &self,
        symbol: &str,
        stream_kind: &str,
    ) -> std::io::Result<Vec<(i64, Vec<u8>)>> {
        let path = self.root.join(symbol).join(format!("{stream_kind}.bin"));
        if !path.exists() {
            return Ok(vec![]);
        }
        let mut reader = BufReader::new(File::open(&path)?);
        let mut out = Vec::new();
        loop {
            let ts = match reader.read_i64::<LittleEndian>() {
                Ok(v) => v,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };
            let len = match reader.read_u32::<LittleEndian>() {
                Ok(v) => v as usize,
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            };
            let mut payload = vec![0u8; len];
            match reader.read_exact(&mut payload) {
                Ok(()) => out.push((ts, payload)),
                Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
        }
        Ok(out)
    }

    /// Read records with `ts_ms` in `[from_ms, to_ms]` (inclusive).
    pub fn read_range(
        &self,
        symbol: &str,
        stream_kind: &str,
        from_ms: i64,
        to_ms: i64,
    ) -> std::io::Result<Vec<(i64, Vec<u8>)>> {
        let all = self.read_all(symbol, stream_kind)?;
        Ok(all
            .into_iter()
            .filter(|(ts, _)| *ts >= from_ms && *ts <= to_ms)
            .collect())
    }

    /// Lazy iterator over records — avoids loading the entire file into memory.
    ///
    /// Returns an iterator that yields `Ok((ts_ms, payload))` or `Err` on I/O
    /// failure. Truncated tail records produce `None` (iterator stops cleanly).
    pub fn iter(
        &self,
        symbol: &str,
        stream_kind: &str,
    ) -> std::io::Result<EventLogIter> {
        let path = self.root.join(symbol).join(format!("{stream_kind}.bin"));
        let reader = if path.exists() {
            Some(BufReader::new(File::open(&path)?))
        } else {
            None
        };
        Ok(EventLogIter { reader })
    }
}

/// Lazy iterator returned by [`EventLog::iter`].
pub struct EventLogIter {
    reader: Option<BufReader<File>>,
}

impl Iterator for EventLogIter {
    type Item = std::io::Result<(i64, Vec<u8>)>;

    fn next(&mut self) -> Option<Self::Item> {
        let r = self.reader.as_mut()?;

        let ts = match r.read_i64::<LittleEndian>() {
            Ok(v) => v,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(e)),
        };
        let len = match r.read_u32::<LittleEndian>() {
            Ok(v) => v as usize,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return None,
            Err(e) => return Some(Err(e)),
        };
        let mut payload = vec![0u8; len];
        match r.read_exact(&mut payload) {
            Ok(()) => Some(Ok((ts, payload))),
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => None,
            Err(e) => Some(Err(e)),
        }
    }
}
