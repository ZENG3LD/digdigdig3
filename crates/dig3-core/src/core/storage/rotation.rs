//! RotatingWriter — opens a daily file, rotates on UTC midnight boundary.
//!
//! File naming: `{dir}/{YYYY-MM-DD}.bin`

use std::fs::{create_dir_all, File, OpenOptions};
use std::io::{BufWriter, Write};
use std::path::PathBuf;

use byteorder::{LittleEndian, WriteBytesExt};
use chrono::{NaiveDate, Utc};

/// Reads binary records `[i64 ts_ms LE][u32 len LE][payload]` from a single daily file,
/// filtering to `[from_ms, to_ms]` (inclusive).
pub fn read_file_range(
    path: &std::path::Path,
    from_ms: i64,
    to_ms: i64,
) -> std::io::Result<Vec<(i64, Vec<u8>)>> {
    use byteorder::ReadBytesExt;
    use std::io::{BufReader, Read};

    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
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
            Ok(()) => {
                if ts >= from_ms && ts <= to_ms {
                    out.push((ts, payload));
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        }
    }
    Ok(out)
}

/// Wraps a daily `{YYYY-MM-DD}.bin` file with automatic rotation at UTC midnight.
pub struct RotatingWriter {
    dir: PathBuf,
    current_day: NaiveDate,
    writer: BufWriter<File>,
}

impl RotatingWriter {
    /// Open (or create) today's daily file under `dir`.
    pub fn new(dir: impl Into<PathBuf>) -> std::io::Result<Self> {
        let dir = dir.into();
        create_dir_all(&dir)?;
        let today = Utc::now().date_naive();
        let path = dir.join(format!("{}.bin", today.format("%Y-%m-%d")));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        Ok(Self {
            dir,
            current_day: today,
            writer: BufWriter::new(file),
        })
    }

    /// Append one record. Rotates to a new file if UTC date has advanced.
    pub fn append(&mut self, ts_ms: i64, payload: &[u8]) -> std::io::Result<()> {
        let today = Utc::now().date_naive();
        if today != self.current_day {
            self.rotate(today)?;
        }
        self.writer.write_i64::<LittleEndian>(ts_ms)?;
        self.writer.write_u32::<LittleEndian>(payload.len() as u32)?;
        self.writer.write_all(payload)?;
        Ok(())
    }

    /// Flush buffered data to the OS.
    pub fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }

    fn rotate(&mut self, new_day: NaiveDate) -> std::io::Result<()> {
        self.writer.flush()?;
        let path = self.dir.join(format!("{}.bin", new_day.format("%Y-%m-%d")));
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&path)?;
        self.writer = BufWriter::new(file);
        self.current_day = new_day;
        Ok(())
    }

    /// Force rotate to a specific date. Used in tests to simulate midnight crossing.
    pub fn rotate_to(&mut self, day: NaiveDate) -> std::io::Result<()> {
        self.rotate(day)
    }

    /// Return the current day this writer is writing to.
    pub fn current_day(&self) -> NaiveDate {
        self.current_day
    }
}
