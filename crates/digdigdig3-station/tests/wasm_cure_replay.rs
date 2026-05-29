//! wasm32 cure + replay round-trip tests (Wave 4).
//!
//! Verifies that the OPFS-backed `StorageManager`, `cure` pipeline, and
//! `ReplayHub` all compile and run correctly in a browser context.
//!
//! # Running
//!
//! ```sh
//! cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!     --test wasm_cure_replay
//! ```
//!
//! Requires: wasm-bindgen-test runner configured in `.cargo/config.toml` +
//! Chrome/Firefox/Safari with OPFS support.
//!
//! Note: these tests do NOT run automatically in CI (compile-only gate on
//! `cargo check --target wasm32-unknown-unknown`). Browser execution is
//! verified by the coordinator in the Wave 4 acceptance pass.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::core::types::{
    AccountType, ExchangeId, StreamEvent, SubscriptionRequest, Symbol, Ticker,
};
use digdigdig3_station::{
    cure::{IntegrityChecker, RepairPipeline},
    ReplayConfig, ReplayHub, ReplayRate, StorageConfig, StorageManager, StreamKey,
};
use futures_util::StreamExt;
use std::path::PathBuf;
use std::time::Duration;

// ── helpers ───────────────────────────────────────────────────────────────────

fn test_key(suffix: &str) -> StreamKey {
    StreamKey {
        exchange: "binance".to_string(),
        account: "spot".to_string(),
        symbol: format!("BTCUSDT-wasm-{suffix}"),
        stream_kind: "ticker".to_string(),
    }
}

fn make_storage(suffix: &str) -> StorageManager {
    StorageManager::new(StorageConfig {
        root: PathBuf::from(format!("wasm-test-{suffix}")),
        default_retention_days: 30,
        orderbook_snapshot_interval_secs: 0,
    })
    .expect("StorageManager::new must succeed on wasm32")
}

fn make_ticker_event(price: f64, ts_ms: i64) -> StreamEvent {
    StreamEvent::Ticker {
        symbol: "BTCUSDT".to_string(),
        ticker: Ticker {
            last_price: price,
            bid_price: Some(price - 1.0),
            ask_price: Some(price + 1.0),
            volume_24h: Some(500.0),
            quote_volume_24h: None,
            price_change_24h: None,
            price_change_percent_24h: None,
            high_24h: None,
            low_24h: None,
            timestamp: ts_ms,
        },
    }
}

// ── Test 1: StorageManager round-trip ─────────────────────────────────────────

/// Write 10 records via OPFS StorageManager, flush, read_range → assert 10 back.
///
/// Uses a unique symbol to avoid collision with prior runs (OPFS persists).
#[wasm_bindgen_test]
async fn wasm_storage_manager_round_trip() {
    let mgr = make_storage("sm-round-trip");
    let key = test_key("sm-round-trip");
    let base_ms = 1_700_000_000_000i64;

    for i in 0..10i64 {
        let payload = format!("record-{i}").into_bytes();
        mgr.append(&key, base_ms + i, &payload)
            .await
            .expect("append must succeed");
    }

    mgr.flush_all().await.expect("flush_all must succeed");

    let records = mgr
        .read_range(&key, base_ms, base_ms + 9)
        .await
        .expect("read_range must succeed");

    assert_eq!(
        records.len(),
        10,
        "expected 10 records back, got {}",
        records.len()
    );

    for (i, (ts, payload)) in records.iter().enumerate() {
        assert_eq!(*ts, base_ms + i as i64, "ts mismatch at index {i}");
        let expected = format!("record-{i}").into_bytes();
        assert_eq!(payload, &expected, "payload mismatch at index {i}");
    }
}

// ── Test 2: cure integrity check ──────────────────────────────────────────────

/// Write 5 unique records + 1 duplicate, run IntegrityChecker, assert
/// duplicate_count >= 1 and record_count == 6.
#[wasm_bindgen_test]
async fn wasm_integrity_check() {
    let mgr = make_storage("cure-integrity");
    let key = test_key("cure-integrity");
    let base_ms = 1_710_000_000_000i64;

    // 5 unique payloads.
    for i in 0..5i64 {
        let payload = format!("unique-{i}").into_bytes();
        mgr.append(&key, base_ms + i, &payload)
            .await
            .expect("append must succeed");
    }
    // A TRUE duplicate of record 2 — same payload AND same ts. The dedup
    // fingerprint is (ts_ms, sha256[:16]) (see cure/integrity.rs), so a record
    // with a different ts is distinct, not a duplicate.
    mgr.append(&key, base_ms + 2, b"unique-2")
        .await
        .expect("dup append must succeed");

    mgr.flush_all().await.expect("flush_all must succeed");

    let checker = IntegrityChecker::new(&mgr);
    let report = checker
        .check(&key, base_ms, base_ms + 100)
        .await
        .expect("integrity check must succeed");

    assert_eq!(
        report.record_count, 6,
        "expected 6 total records, got {}",
        report.record_count
    );
    assert!(
        report.duplicate_count >= 1,
        "expected at least 1 duplicate, got {}",
        report.duplicate_count
    );
}

// ── Test 3: replay instant rate ────────────────────────────────────────────────

/// Write 5 serialised StreamEvents to OPFS, replay with ReplayRate::Instant,
/// collect and assert 5 events emitted.
#[wasm_bindgen_test]
async fn wasm_replay_instant() {
    // Write via storage manager first.
    let mgr = make_storage("replay-instant");
    let key = test_key("replay-instant");
    let base_ms = 1_720_000_000_000i64;

    for i in 0..5i64 {
        let ev = make_ticker_event(50000.0 + i as f64, base_ms + i * 100);
        let payload = serde_json::to_vec(&ev).expect("serialize must succeed");
        mgr.append(&key, base_ms + i * 100, &payload)
            .await
            .expect("append must succeed");
    }
    mgr.flush_all().await.expect("flush_all must succeed");

    // ReplayHub uses same OPFS root (the storage root path is cosmetic on wasm).
    let hub = ReplayHub::new(ReplayConfig {
        storage_root: PathBuf::from("wasm-test-replay-instant"),
        rate: ReplayRate::Instant,
        from_ms: Some(base_ms),
        to_ms: Some(base_ms + 500),
    })
    .await
    .expect("ReplayHub::new must succeed");

    hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false)
        .await
        .expect("connect_full must succeed");

    let ws = hub
        .ws(ExchangeId::Binance, AccountType::Spot)
        .expect("ws must be present after connect_full");

    // The replay key is derived from the SUBSCRIBED symbol (replay/ws.rs), so it
    // must match the symbol the records were WRITTEN under (test_key suffix) or
    // load_records finds nothing and the broadcast stream never fires.
    let sub = SubscriptionRequest::ticker(Symbol::with_raw(
        "BTC",
        "USDT",
        "BTCUSDT-wasm-replay-instant".into(),
    ));
    ws.subscribe(sub).await.expect("subscribe must succeed");

    let mut stream = ws.event_stream();
    let mut events: Vec<StreamEvent> = Vec::new();

    // Collect up to 5 events, bounded by a 15s deadline. Replay emits a finite
    // set and the hub keeps the broadcast sender open afterward, so an unbounded
    // `stream.next()` loop would hang forever if fewer than 5 events arrive —
    // the deadline turns that into a clean assertion failure instead.
    let deadline = gloo_timers::future::sleep(Duration::from_secs(15));
    futures_util::pin_mut!(deadline);
    loop {
        let next_fut = stream.next();
        futures_util::pin_mut!(next_fut);
        match futures_util::future::select(next_fut, &mut deadline).await {
            futures_util::future::Either::Left((Some(Ok(ev)), _)) => {
                events.push(ev);
                if events.len() >= 5 {
                    break;
                }
            }
            futures_util::future::Either::Left((Some(Err(_)), _))
            | futures_util::future::Either::Left((None, _)) => break,
            futures_util::future::Either::Right(_) => break, // deadline hit
        }
    }

    assert_eq!(
        events.len(),
        5,
        "expected 5 replay events, got {}",
        events.len()
    );

    // Verify first event has the right price.
    if let StreamEvent::Ticker { ticker: t, .. } = &events[0] {
        assert!(
            (t.last_price - 50000.0).abs() < 0.001,
            "first event price mismatch: {}",
            t.last_price
        );
    } else {
        panic!("expected Ticker event, got {:?}", events[0]);
    }
}

// ── Test 4: repair pipeline dry-run on wasm ────────────────────────────────────

/// Run RepairPipeline (dry_run = true) on a stream with a duplicate. Verify it
/// completes without error and reports the correct record count.
#[wasm_bindgen_test]
async fn wasm_repair_pipeline_dry_run() {
    let mgr = make_storage("repair-pipeline");
    let key = test_key("repair-pipeline");
    let base_ms = 1_730_000_000_000i64;

    for i in 0..5i64 {
        let payload = format!("data-{i}").into_bytes();
        mgr.append(&key, base_ms + i, &payload)
            .await
            .expect("append");
    }
    // A true duplicate of record 3 — same payload AND same ts (dedup key is
    // (ts_ms, sha256[:16]); a different ts would be a distinct record).
    mgr.append(&key, base_ms + 3, b"data-3")
        .await
        .expect("dup append");

    mgr.flush_all().await.expect("flush_all");

    let pipeline = RepairPipeline::new(&mgr);
    let report = pipeline
        .run(&key, base_ms, base_ms + 100, true)
        .await
        .expect("repair pipeline must complete");

    assert_eq!(
        report.integrity.record_count, 6,
        "expected 6 records in integrity report, got {}",
        report.integrity.record_count
    );
}
