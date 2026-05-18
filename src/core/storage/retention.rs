//! Retention sweeper.
//!
//! Walks storage root, deletes `{YYYY-MM-DD}.bin` files older than
//! `retention_days`. Returns count of deleted files.

use std::path::{Path, PathBuf};

use chrono::{Duration, NaiveDate, Utc, DateTime};

/// Delete all daily `.bin` files under `root` whose date is older than
/// `now - retention_days`.
///
/// Recurses through all subdirectories. Only files whose stem parses as
/// `YYYY-MM-DD` and whose date falls before `cutoff` are deleted.
///
/// Returns the number of files deleted.
pub fn sweep(root: &Path, now: DateTime<Utc>, retention_days: u32) -> std::io::Result<usize> {
    let cutoff = now.date_naive() - Duration::days(retention_days as i64);
    let mut deleted = 0usize;
    walk_and_delete(root, cutoff, &mut deleted)?;
    Ok(deleted)
}

fn walk_and_delete(dir: &Path, cutoff: NaiveDate, deleted: &mut usize) -> std::io::Result<()> {
    if !dir.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path: PathBuf = entry.path();
        if path.is_dir() {
            walk_and_delete(&path, cutoff, deleted)?;
        } else if let Some(date) = parse_date_from_stem(&path) {
            if date < cutoff {
                std::fs::remove_file(&path)?;
                *deleted += 1;
            }
        }
    }
    Ok(())
}

/// Parse `YYYY-MM-DD` from a file stem. Returns `None` if the stem doesn't
/// match the pattern or the extension is not `.bin`.
fn parse_date_from_stem(path: &Path) -> Option<NaiveDate> {
    if path.extension()?.to_str()? != "bin" {
        return None;
    }
    let stem = path.file_stem()?.to_str()?;
    NaiveDate::parse_from_str(stem, "%Y-%m-%d").ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_date_parses_valid_stem() {
        let p = Path::new("/data/2024-01-15.bin");
        assert_eq!(
            parse_date_from_stem(p),
            Some(NaiveDate::from_ymd_opt(2024, 1, 15).unwrap())
        );
    }

    #[test]
    fn parse_date_rejects_non_bin() {
        let p = Path::new("/data/2024-01-15.idx");
        assert!(parse_date_from_stem(p).is_none());
    }

    #[test]
    fn parse_date_rejects_non_date_stem() {
        let p = Path::new("/data/ticker.bin");
        assert!(parse_date_from_stem(p).is_none());
    }

    #[test]
    fn sweep_deletes_old_files() {
        use std::fs;
        let tmp = std::env::temp_dir().join(format!("dig3_retention_test_{}", std::process::id()));
        fs::create_dir_all(&tmp).unwrap();

        // Create an old file (2020-01-01) and a recent one (today).
        let old_path = tmp.join("2020-01-01.bin");
        let today = Utc::now().date_naive();
        let recent_path = tmp.join(format!("{}.bin", today.format("%Y-%m-%d")));

        fs::write(&old_path, b"old").unwrap();
        fs::write(&recent_path, b"recent").unwrap();

        let deleted = sweep(&tmp, Utc::now(), 30).unwrap();
        assert_eq!(deleted, 1);
        assert!(!old_path.exists());
        assert!(recent_path.exists());

        fs::remove_dir_all(&tmp).ok();
    }
}
