//! Time-range index sidecar files.
//!
//! For each `{day}.bin`, optional `{day}.idx` maps `(hour, file_offset)`.
//! Allows skipping leading hours during `read_range` without scanning.
//!
//! Stub implementation: index building is a no-op returning an empty vec.
//! Real disk-persisted index is future work.

use std::path::Path;

/// Build an in-memory hour → offset index for a daily `.bin` file.
///
/// Returns `Vec<(hour_of_day, file_offset)>` sorted ascending by hour.
/// Currently returns an empty vec (stub) — reads fall back to full scan.
pub fn build_index(_bin_path: &Path) -> std::io::Result<Vec<(u8, u64)>> {
    Ok(vec![])
}
