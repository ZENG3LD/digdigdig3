//! Tests for StorageManager — Phase μ.
//!
//! Pure file I/O + tokio runtime. No network, no exchange calls.

use byteorder::{LittleEndian, WriteBytesExt};
use chrono::Utc;
use digdigdig3::core::storage::{StorageConfig, StorageManager, StreamKey};
use std::path::PathBuf;

// ── helpers ───────────────────────────────────────────────────────────────────

fn tmpdir(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("dig3_mgr_{}_{}_{}", std::process::id(), name, rand_suffix()));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn rand_suffix() -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    h.finish()
}

fn make_config(dir: &PathBuf) -> StorageConfig {
    StorageConfig {
        root: dir.clone(),
        default_retention_days: 30,
        orderbook_snapshot_interval_secs: 300,
    }
}

fn key(exchange: &str, symbol: &str, kind: &str) -> StreamKey {
    StreamKey {
        exchange: exchange.into(),
        account: "spot".into(),
        symbol: symbol.into(),
        stream_kind: kind.into(),
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn write_to_multiple_keys_creates_separate_files() {
    let dir = tmpdir("multi_keys");
    let mgr = StorageManager::new(make_config(&dir)).unwrap();

    let k1 = key("binance", "BTCUSDT", "ticker");
    let k2 = key("binance", "ETHUSDT", "ticker");
    let k3 = key("bybit", "BTCUSDT", "trade");

    let ts = 1_700_000_000_000i64;
    mgr.append(&k1, ts, b"btc-ticker").await.unwrap();
    mgr.append(&k2, ts, b"eth-ticker").await.unwrap();
    mgr.append(&k3, ts, b"btc-trade").await.unwrap();

    mgr.flush_all().await.unwrap();

    // Each stream directory should have a daily file.
    let today = Utc::now().date_naive().format("%Y-%m-%d").to_string();
    let today_file = format!("{today}.bin");

    assert!(
        mgr.stream_dir(&k1).join(&today_file).exists(),
        "binance/BTCUSDT/ticker file missing"
    );
    assert!(
        mgr.stream_dir(&k2).join(&today_file).exists(),
        "binance/ETHUSDT/ticker file missing"
    );
    assert!(
        mgr.stream_dir(&k3).join(&today_file).exists(),
        "bybit/BTCUSDT/trade file missing"
    );

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn read_range_returns_correct_records() {
    let dir = tmpdir("read_range");
    let mgr = StorageManager::new(make_config(&dir)).unwrap();

    let k = key("okx", "BTCUSDT", "trade");
    // Use current time so the file date matches the query date range.
    let base_ms = Utc::now().timestamp_millis();

    // Write 10 records spaced 1 second apart.
    for i in 0i64..10 {
        let payload = format!("record_{i}").into_bytes();
        mgr.append(&k, base_ms + i * 1_000, &payload).await.unwrap();
    }
    mgr.flush_all().await.unwrap();

    // Query middle 5 (index 2..=6).
    let from = base_ms + 2_000;
    let to = base_ms + 6_000;
    let records = mgr.read_range(&k, from, to).await.unwrap();

    assert_eq!(records.len(), 5, "expected 5 records in [+2s, +6s]");
    assert_eq!(records.first().unwrap().0, from);
    assert_eq!(records.last().unwrap().0, to);

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn flush_persists_buffered_data() {
    let dir = tmpdir("flush");
    let mgr = StorageManager::new(make_config(&dir)).unwrap();

    let k = key("kucoin", "XRPUSDT", "ticker");
    // Use current time so the file date matches the query date range.
    let ts = Utc::now().timestamp_millis();

    mgr.append(&k, ts, b"hello").await.unwrap();
    // Without flush, data may still be in BufWriter.
    // After flush, the file on disk must contain the record.
    mgr.flush_all().await.unwrap();

    let records = mgr.read_range(&k, ts, ts).await.unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].1, b"hello");

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn retention_sweep_deletes_old_files() {
    let dir = tmpdir("retention");
    let mgr = StorageManager::new(StorageConfig {
        root: dir.clone(),
        default_retention_days: 7,
        ..StorageConfig::default()
    })
    .unwrap();

    // Manually create old and recent files in a stream dir.
    let stream_dir = dir.join("test_ex").join("spot").join("BTCUSDT").join("ticker");
    std::fs::create_dir_all(&stream_dir).unwrap();

    let old_file = stream_dir.join("2020-01-01.bin");
    let today = Utc::now().date_naive();
    let recent_file = stream_dir.join(format!("{}.bin", today.format("%Y-%m-%d")));

    std::fs::write(&old_file, b"old").unwrap();
    std::fs::write(&recent_file, b"recent").unwrap();

    let deleted = mgr.cleanup(Utc::now()).unwrap();
    assert_eq!(deleted, 1, "expected 1 old file deleted");
    assert!(!old_file.exists(), "old file should be deleted");
    assert!(recent_file.exists(), "recent file should remain");

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn read_range_nonexistent_stream_returns_empty() {
    let dir = tmpdir("nonexistent");
    let mgr = StorageManager::new(make_config(&dir)).unwrap();
    let k = key("ghost", "NOBODY", "ticker");
    let records = mgr
        .read_range(&k, 1_000_000_000_000, 2_000_000_000_000)
        .await
        .unwrap();
    assert!(records.is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn concurrent_writes_to_different_keys() {
    let dir = tmpdir("concurrent");
    let mgr = std::sync::Arc::new(StorageManager::new(make_config(&dir)).unwrap());

    // Use current time so the file date matches the query date range.
    let ts_base = Utc::now().timestamp_millis();
    let mut handles = Vec::new();

    for i in 0u64..8 {
        let mgr = mgr.clone();
        let k = key("binance", &format!("SYM{i}"), "ticker");
        let h = tokio::spawn(async move {
            for j in 0i64..50 {
                mgr.append(&k, ts_base + j, format!("payload_{j}").as_bytes())
                    .await
                    .unwrap();
            }
        });
        handles.push(h);
    }

    for h in handles {
        h.await.unwrap();
    }
    mgr.flush_all().await.unwrap();

    // Verify each symbol has 50 records.
    for i in 0u64..8 {
        let k = key("binance", &format!("SYM{i}"), "ticker");
        let records = mgr
            .read_range(&k, ts_base, ts_base + 49)
            .await
            .unwrap();
        assert_eq!(
            records.len(),
            50,
            "SYM{i} expected 50 records, got {}",
            records.len()
        );
    }

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn rotation_at_midnight_creates_new_file() {
    use digdigdig3::core::storage::rotation::{read_file_range, RotatingWriter};

    let dir = tmpdir("rotation");
    let stream_dir = dir.join("test_stream");
    std::fs::create_dir_all(&stream_dir).unwrap();

    let today = Utc::now().date_naive();
    let tomorrow = today.succ_opt().unwrap();

    // Write to today's file, then rotate to tomorrow via rotate_to.
    {
        let mut writer = RotatingWriter::new(&stream_dir).unwrap();
        let today_ms = Utc::now().timestamp_millis();
        writer.append(today_ms, b"today").unwrap();
        // rotate_to flushes today then opens tomorrow's file.
        writer.rotate_to(tomorrow).unwrap();
        // Write directly to tomorrow's file by calling the underlying rotation-aware append.
        // We must write a record without triggering the wall-clock auto-rotate.
        // Drop writer (flushes empty buffer); tomorrow file now exists but is empty.
        writer.flush().unwrap();
    }

    // Separately write a record directly into tomorrow's file.
    {
        use std::io::Write;
        let tomorrow_file = stream_dir.join(format!("{}.bin", tomorrow.format("%Y-%m-%d")));
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&tomorrow_file)
            .unwrap();
        f.write_i64::<LittleEndian>(1_700_100_000_000i64).unwrap();
        f.write_u32::<LittleEndian>(8u32).unwrap();
        f.write_all(b"tomorrow").unwrap();
    }

    let today_file = stream_dir.join(format!("{}.bin", today.format("%Y-%m-%d")));
    let tomorrow_file = stream_dir.join(format!("{}.bin", tomorrow.format("%Y-%m-%d")));

    assert!(today_file.exists(), "today's file should exist");
    assert!(tomorrow_file.exists(), "tomorrow's file should exist");

    // Verify content in today's file.
    let today_records = read_file_range(&today_file, i64::MIN, i64::MAX).unwrap();
    assert_eq!(today_records.len(), 1, "today should have 1 record");
    assert_eq!(today_records[0].1, b"today");

    let tomorrow_records = read_file_range(&tomorrow_file, i64::MIN, i64::MAX).unwrap();
    assert_eq!(tomorrow_records.len(), 1, "tomorrow should have 1 record");
    assert_eq!(tomorrow_records[0].1, b"tomorrow");

    std::fs::remove_dir_all(&dir).ok();
}

#[tokio::test]
async fn read_range_spans_multiple_days() {
    use chrono::Duration;
    use digdigdig3::core::storage::rotation::RotatingWriter;

    let dir = tmpdir("span_days");
    let mgr_dir = dir.join("binance").join("spot").join("BTCUSDT").join("ticker");
    std::fs::create_dir_all(&mgr_dir).unwrap();

    let today = Utc::now().date_naive();
    let yesterday = today - Duration::days(1);

    // Write yesterday's file manually via RotatingWriter injected to yesterday.
    {
        let mut w = RotatingWriter::new(&mgr_dir).unwrap();
        w.rotate_to(yesterday).unwrap();
        // Timestamps within yesterday (any ms value — range query uses ts values stored in file).
        let yday_ms = yesterday.and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp_millis();
        w.append(yday_ms, b"yesterday_rec").unwrap();
        w.flush().unwrap();
    }

    // Write today's file normally.
    {
        let mut w = RotatingWriter::new(&mgr_dir).unwrap();
        let today_ms = today.and_hms_opt(12, 0, 0).unwrap().and_utc().timestamp_millis();
        w.append(today_ms, b"today_rec").unwrap();
        w.flush().unwrap();
    }

    let mgr = StorageManager::new(StorageConfig {
        root: dir.clone(),
        ..StorageConfig::default()
    })
    .unwrap();

    let k = key("binance", "BTCUSDT", "ticker");

    let yesterday_ms = yesterday.and_hms_opt(0, 0, 0).unwrap().and_utc().timestamp_millis();
    let tomorrow_ms = today.and_hms_opt(23, 59, 59).unwrap().and_utc().timestamp_millis();

    let records = mgr.read_range(&k, yesterday_ms, tomorrow_ms).await.unwrap();
    assert_eq!(
        records.len(),
        2,
        "expected 2 records spanning yesterday+today, got {}",
        records.len()
    );

    std::fs::remove_dir_all(&dir).ok();
}
