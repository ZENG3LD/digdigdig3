//! Tests for `EventLog` binary persistence.
//!
//! Pure file I/O — no network, no async runtime. No new crate dependencies.

use digdigdig3_core::core::storage::{EventLog, EventRecord};
use std::io::Write;

// ── helpers ───────────────────────────────────────────────────────────────────

/// Create a unique temp subdirectory for each test.
/// Cleaned up by the OS after reboot; tests are hermetic because we use random names.
fn tmpdir(name: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    // Include the process id so parallel test runs don't clash.
    dir.push(format!("dig3_event_log_{}_{}", std::process::id(), name));
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[test]
fn write_then_read_roundtrip() {
    let dir = tmpdir("roundtrip");
    let log = EventLog::new(&dir).unwrap();

    let count = 100usize;
    let mut expected: Vec<(i64, Vec<u8>)> = Vec::with_capacity(count);

    for i in 0..count {
        let ts_ms = 1_700_000_000_000i64 + i as i64;
        let payload = format!(r#"{{"i":{i}}}"#);
        log.append(
            "BTCUSDT",
            "ticker",
            &EventRecord {
                ts_ms,
                payload: payload.as_bytes(),
            },
        )
        .unwrap();
        expected.push((ts_ms, payload.into_bytes()));
    }

    let got = log.read_all("BTCUSDT", "ticker").unwrap();
    assert_eq!(got.len(), count);
    for (i, ((got_ts, got_pay), (exp_ts, exp_pay))) in got.iter().zip(expected.iter()).enumerate() {
        assert_eq!(got_ts, exp_ts, "ts mismatch at record {i}");
        assert_eq!(got_pay, exp_pay, "payload mismatch at record {i}");
    }

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn read_range_filters_correctly() {
    let dir = tmpdir("range");
    let log = EventLog::new(&dir).unwrap();

    for i in 0i64..20 {
        let ts_ms = 1_000i64 + i * 100;
        let payload = format!("{i}");
        log.append(
            "ETHUSDT",
            "trade",
            &EventRecord {
                ts_ms,
                payload: payload.as_bytes(),
            },
        )
        .unwrap();
    }

    // i=5..=10 → ts 1500..=2000
    let got = log.read_range("ETHUSDT", "trade", 1500, 2000).unwrap();
    assert_eq!(got.len(), 6, "expected 6 records in [1500,2000]");
    assert_eq!(got.first().unwrap().0, 1500);
    assert_eq!(got.last().unwrap().0, 2000);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn truncated_file_handled_gracefully() {
    let dir = tmpdir("truncated");
    let log = EventLog::new(&dir).unwrap();

    // Write one complete record
    log.append(
        "SOLUSDT",
        "orderbook",
        &EventRecord {
            ts_ms: 9999,
            payload: b"complete",
        },
    )
    .unwrap();

    // Manually append a partial record (only the ts_ms header, no length/payload)
    let file_path = dir.join("SOLUSDT").join("orderbook.bin");
    let mut f = std::fs::OpenOptions::new()
        .append(true)
        .open(&file_path)
        .unwrap();
    f.write_all(&9999i64.to_le_bytes()).unwrap(); // partial: ts only, missing u32 len + payload
    drop(f);

    // read_all must not panic — truncated tail is silently skipped
    let got = log.read_all("SOLUSDT", "orderbook").unwrap();
    assert_eq!(got.len(), 1, "truncated tail must be skipped");
    assert_eq!(got[0].0, 9999);
    assert_eq!(got[0].1, b"complete");

    // Iterator must also stop cleanly
    let iter_count = log.iter("SOLUSDT", "orderbook").unwrap().count();
    assert_eq!(iter_count, 1);

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn multi_symbol_multi_stream_isolated() {
    let dir = tmpdir("multi");
    let log = EventLog::new(&dir).unwrap();

    let pairs = [
        ("BTCUSDT", "ticker"),
        ("BTCUSDT", "trade"),
        ("ETHUSDT", "ticker"),
        ("XRPUSDT", "funding_rate"),
    ];

    for (sym, kind) in &pairs {
        for i in 0u8..5 {
            log.append(
                sym,
                kind,
                &EventRecord {
                    ts_ms: i as i64,
                    payload: &[i],
                },
            )
            .unwrap();
        }
    }

    for (sym, kind) in &pairs {
        let records = log.read_all(sym, kind).unwrap();
        assert_eq!(
            records.len(),
            5,
            "({sym},{kind}) should have 5 records, got {}",
            records.len()
        );
        for (idx, (ts, pay)) in records.iter().enumerate() {
            assert_eq!(*ts, idx as i64);
            assert_eq!(*pay, vec![idx as u8]);
        }
    }

    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn nonexistent_file_returns_empty() {
    let dir = tmpdir("empty");
    let log = EventLog::new(&dir).unwrap();
    let got = log.read_all("NOBODY", "ghost").unwrap();
    assert!(got.is_empty());
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn iter_matches_read_all() {
    let dir = tmpdir("iter");
    let log = EventLog::new(&dir).unwrap();

    for i in 0i64..50 {
        log.append(
            "BNBUSDT",
            "mark_price",
            &EventRecord {
                ts_ms: i,
                payload: format!("payload_{i}").as_bytes(),
            },
        )
        .unwrap();
    }

    let via_read = log.read_all("BNBUSDT", "mark_price").unwrap();
    let via_iter: Vec<_> = log
        .iter("BNBUSDT", "mark_price")
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();

    assert_eq!(via_read, via_iter);

    std::fs::remove_dir_all(&dir).ok();
}
