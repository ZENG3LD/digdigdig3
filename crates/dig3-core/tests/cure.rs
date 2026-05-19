//! Tests for the cure module — Phase ξ.
//!
//! Pure in-memory storage + tokio runtime. No network.

use std::path::PathBuf;

use digdigdig3_core::core::cure::{
    dedup::Deduper,
    gap::GapDetector,
    integrity::IntegrityChecker,
    repair::RepairPipeline,
};
use digdigdig3_core::core::storage::{StorageConfig, StorageManager, StreamKey};

// ── helpers ───────────────────────────────────────────────────────────────────

fn tmpdir(name: &str) -> PathBuf {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::SystemTime;
    let mut h = DefaultHasher::new();
    SystemTime::now().hash(&mut h);
    let suffix = h.finish();
    let mut dir = std::env::temp_dir();
    dir.push(format!("dig3_cure_{}_{}", name, suffix));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn make_storage(name: &str) -> StorageManager {
    let dir = tmpdir(name);
    let cfg = StorageConfig {
        root: dir,
        default_retention_days: 30,
        orderbook_snapshot_interval_secs: 300,
    };
    StorageManager::new(cfg).unwrap()
}

fn key(kind: &str) -> StreamKey {
    StreamKey {
        exchange: "test".into(),
        account: "spot".into(),
        symbol: "BTCUSDT".into(),
        stream_kind: kind.into(),
    }
}

/// Current time in ms — matches the file RotatingWriter opened (today's file).
fn now_ms() -> i64 {
    chrono::Utc::now().timestamp_millis()
}

// ── integrity tests ───────────────────────────────────────────────────────────

#[tokio::test]
async fn integrity_detects_duplicates() {
    let mgr = make_storage("dup");
    let k = key("trade");
    let base = now_ms();

    // 10 unique records
    for i in 0u64..10 {
        let payload = format!(r#"{{"id":{}}}"#, i);
        mgr.append(&k, base + i as i64 * 100, payload.as_bytes())
            .await
            .unwrap();
    }
    // same 10 again — same ts + same payload = duplicate
    for i in 0u64..10 {
        let payload = format!(r#"{{"id":{}}}"#, i);
        mgr.append(&k, base + i as i64 * 100, payload.as_bytes())
            .await
            .unwrap();
    }
    mgr.flush_all().await.unwrap();

    let report = IntegrityChecker::new(&mgr)
        .check(&k, base - 1, base + 10_000)
        .await
        .unwrap();

    assert_eq!(report.record_count, 20, "total records");
    assert_eq!(report.duplicate_count, 10, "duplicates");
}

#[tokio::test]
async fn integrity_detects_time_gaps() {
    let mgr = make_storage("timegap");
    let k = key("trade");
    let base = now_ms();

    // Three records: base, base+1000, base+1000+5min+1s (> 60s threshold)
    let t0 = base;
    let t1 = base + 1_000;
    let t2 = base + 1_000 + 5 * 60_000 + 1_000;

    for (t, id) in [(t0, 1u32), (t1, 2), (t2, 3)] {
        let payload = format!(r#"{{"id":{}}}"#, id);
        mgr.append(&k, t, payload.as_bytes()).await.unwrap();
    }
    mgr.flush_all().await.unwrap();

    let report = IntegrityChecker::new(&mgr)
        .with_time_gap_threshold(60_000)
        .check(&k, t0 - 1, t2 + 1)
        .await
        .unwrap();

    assert_eq!(report.record_count, 3);
    assert_eq!(report.time_gaps.len(), 1, "exactly one time gap");
    assert!(report.time_gaps[0].duration_ms > 60_000);
}

#[tokio::test]
async fn integrity_out_of_order() {
    let mgr = make_storage("ooo");
    let k = key("trade");
    let base = now_ms();

    // Insert: t+100, t+200, t+50 — last one is out of order
    mgr.append(&k, base + 100, b"a").await.unwrap();
    mgr.append(&k, base + 200, b"b").await.unwrap();
    mgr.append(&k, base + 50, b"c").await.unwrap();
    mgr.flush_all().await.unwrap();

    // read_range returns records in file-insertion order → t+50 appears after t+200
    let report = IntegrityChecker::new(&mgr)
        .check(&k, base, base + 300)
        .await
        .unwrap();

    assert_eq!(report.record_count, 3);
    assert!(
        report.out_of_order_count >= 1,
        "expected at least 1 out-of-order, got {}",
        report.out_of_order_count
    );
}

// ── dedup tests ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn dedup_removes_duplicates() {
    let mgr = make_storage("dedup");
    let k = key("trade");
    let base = now_ms();

    // 100 unique records
    for i in 0u64..100 {
        let payload = format!(r#"{{"id":{}}}"#, i);
        mgr.append(&k, base + i as i64 * 10, payload.as_bytes())
            .await
            .unwrap();
    }
    // 10 duplicates of first 10
    for i in 0u64..10 {
        let payload = format!(r#"{{"id":{}}}"#, i);
        mgr.append(&k, base + i as i64 * 10, payload.as_bytes())
            .await
            .unwrap();
    }
    mgr.flush_all().await.unwrap();

    let (kept, removed) = Deduper::new(&mgr)
        .dedup(&k, base - 1, base + 100_000)
        .await
        .unwrap();

    assert_eq!(kept, 100, "kept 100 unique");
    assert_eq!(removed, 10, "removed 10 dups");

    // Flush to ensure the _deduped stream is readable
    mgr.flush_all().await.unwrap();

    // Verify the deduped stream has exactly 100 records
    let dedup_key = StreamKey {
        stream_kind: "trade_deduped".into(),
        ..k.clone()
    };
    let deduped_records = mgr
        .read_range(&dedup_key, base - 1, base + 100_000)
        .await
        .unwrap();
    assert_eq!(deduped_records.len(), 100);
}

// ── gap detector tests ────────────────────────────────────────────────────────

#[tokio::test]
async fn gap_detector_finds_sequence_jump() {
    let mgr = make_storage("gaps");
    let k = key("orderbook_delta");
    let base = now_ms();

    // Snapshot: last_update_id=1
    let snap = serde_json::json!({
        "bids": [{"price": 100.0, "size": 1.0}],
        "asks": [{"price": 101.0, "size": 1.0}],
        "timestamp": base,
        "last_update_id": 1u64
    });
    mgr.append(&k, base, snap.to_string().as_bytes())
        .await
        .unwrap();

    // Delta: prev_update_id=1 → sequential, no gap
    let d1 = serde_json::json!({
        "bids": [], "asks": [],
        "timestamp": base + 100,
        "prev_update_id": 1u64,
        "last_update_id": 2u64
    });
    mgr.append(&k, base + 100, d1.to_string().as_bytes())
        .await
        .unwrap();

    // Delta: prev_update_id=5 — gap (book has last_update_id=2, delta expects prev=5)
    let d2 = serde_json::json!({
        "bids": [], "asks": [],
        "timestamp": base + 200,
        "prev_update_id": 5u64,
        "last_update_id": 6u64
    });
    mgr.append(&k, base + 200, d2.to_string().as_bytes())
        .await
        .unwrap();
    mgr.flush_all().await.unwrap();

    let gaps = GapDetector::new(&mgr)
        .detect(&k, base - 1, base + 1000)
        .await
        .unwrap();

    assert_eq!(gaps.len(), 1, "expected exactly 1 sequence gap");
    assert_eq!(gaps[0].expected, 2, "tracker had last_update_id=2");
    assert_eq!(gaps[0].got, 5, "delta carried prev_update_id=5");
}

// ── repair pipeline tests ─────────────────────────────────────────────────────

#[tokio::test]
async fn repair_pipeline_end_to_end() {
    let mgr = make_storage("repair");
    let k = key("trade");
    let base = now_ms();

    // 20 unique records
    for i in 0u64..20 {
        let payload = format!(r#"{{"id":{}}}"#, i);
        mgr.append(&k, base + i as i64 * 50, payload.as_bytes())
            .await
            .unwrap();
    }
    // 5 duplicates
    for i in 0u64..5 {
        let payload = format!(r#"{{"id":{}}}"#, i);
        mgr.append(&k, base + i as i64 * 50, payload.as_bytes())
            .await
            .unwrap();
    }
    mgr.flush_all().await.unwrap();

    // dry_run=true: no deduped stream written, counts reflect simulation
    let report = RepairPipeline::new(&mgr)
        .run(&k, base - 1, base + 100_000, true)
        .await
        .unwrap();

    assert_eq!(report.integrity.record_count, 25);
    assert_eq!(report.integrity.duplicate_count, 5);
    assert_eq!(report.deduped_kept, 25); // dry-run: record_count
    assert_eq!(report.deduped_removed, 5);
    assert!(report.orderbook_gaps.is_empty(), "trade stream has no OB gaps");

    // non-dry_run: deduped stream actually written
    let report2 = RepairPipeline::new(&mgr)
        .run(&k, base - 1, base + 100_000, false)
        .await
        .unwrap();

    assert_eq!(report2.deduped_kept, 20);
    assert_eq!(report2.deduped_removed, 5);
}
