//! Periodic snapshot writer for stateful streams (e.g. orderbook).
//!
//! Stores snapshots at:
//!   `{root}/{exchange}/{account}/{symbol}/{stream_kind}/snapshots/{YYYY-MM-DD-HH-MM-SS}.bin`
//!
//! Replay strategy: locate the latest snapshot whose timestamp is ≤ the
//! desired replay start, then apply deltas forward from that point.

use std::fs::{create_dir_all, OpenOptions};
use std::io::{BufReader, Read, Write};
use std::path::{Path, PathBuf};

use byteorder::{LittleEndian, WriteBytesExt};
use chrono::{DateTime, TimeZone, Utc};

fn snapshots_dir(stream_dir: &Path) -> PathBuf {
    stream_dir.join("snapshots")
}

fn snapshot_filename(ts_ms: i64) -> String {
    let dt = Utc
        .timestamp_millis_opt(ts_ms)
        .single()
        .unwrap_or_else(Utc::now);
    dt.format("%Y-%m-%d-%H-%M-%S").to_string() + ".bin"
}

/// Write a snapshot for `stream_dir` at the given timestamp.
///
/// Format: `[i64 ts_ms LE][u32 len LE][payload_bytes]`
pub fn write_snapshot(stream_dir: &Path, ts_ms: i64, payload: &[u8]) -> std::io::Result<()> {
    let dir = snapshots_dir(stream_dir);
    create_dir_all(&dir)?;
    let path = dir.join(snapshot_filename(ts_ms));
    let mut f = OpenOptions::new().create(true).truncate(true).write(true).open(&path)?;
    f.write_i64::<LittleEndian>(ts_ms)?;
    f.write_u32::<LittleEndian>(payload.len() as u32)?;
    f.write_all(payload)?;
    Ok(())
}

/// Find the latest snapshot whose `ts_ms` ≤ `before_ms`.
///
/// Returns `Some((ts_ms, payload))` or `None` if no suitable snapshot exists.
pub fn find_latest_snapshot_before(
    stream_dir: &Path,
    before_ms: i64,
) -> std::io::Result<Option<(i64, Vec<u8>)>> {
    use byteorder::ReadBytesExt;

    let dir = snapshots_dir(stream_dir);
    if !dir.exists() {
        return Ok(None);
    }

    let mut candidates: Vec<(i64, PathBuf)> = Vec::new();

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("bin") {
            continue;
        }
        // Read ts_ms from file header to get ground-truth timestamp.
        if let Ok(mut f) = std::fs::File::open(&path) {
            if let Ok(ts) = f.read_i64::<LittleEndian>() {
                if ts <= before_ms {
                    candidates.push((ts, path));
                }
            }
        }
    }

    if candidates.is_empty() {
        return Ok(None);
    }

    candidates.sort_by_key(|(ts, _)| *ts);
    let (ts, path) = candidates.into_iter().next_back().unwrap();

    let file = std::fs::File::open(&path)?;
    let mut reader = BufReader::new(file);
    let _ts_header = reader.read_i64::<LittleEndian>()?;
    let len = reader.read_u32::<LittleEndian>()? as usize;
    let mut payload = vec![0u8; len];
    reader.read_exact(&mut payload)?;

    Ok(Some((ts, payload)))
}

/// Check what the current UTC time is for snapshot scheduling.
pub fn now_ms() -> i64 {
    Utc::now().timestamp_millis()
}

/// Returns true when `now_ms` crosses a snapshot boundary.
///
/// `interval_secs` — how often to snapshot (e.g. 300 = every 5 minutes).
/// `last_snapshot_ms` — timestamp of last snapshot taken (0 = never).
pub fn should_snapshot(now_ms: i64, last_snapshot_ms: i64, interval_secs: u64) -> bool {
    let interval_ms = interval_secs as i64 * 1_000;
    now_ms - last_snapshot_ms >= interval_ms
}

/// Purge snapshots older than `retention_days` from `stream_dir`.
pub fn purge_old_snapshots(
    stream_dir: &Path,
    now: DateTime<Utc>,
    retention_days: u32,
) -> std::io::Result<usize> {
    let dir = snapshots_dir(stream_dir);
    if !dir.exists() {
        return Ok(0);
    }
    let cutoff_ms = (now
        - chrono::Duration::days(retention_days as i64))
    .timestamp_millis();
    let mut deleted = 0usize;

    for entry in std::fs::read_dir(&dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("bin") {
            continue;
        }
        // Read ts from file header.
        use byteorder::ReadBytesExt;
        if let Ok(mut f) = std::fs::File::open(&path) {
            if let Ok(ts) = f.read_i64::<LittleEndian>() {
                if ts < cutoff_ms {
                    std::fs::remove_file(&path)?;
                    deleted += 1;
                }
            }
        }
    }
    Ok(deleted)
}
