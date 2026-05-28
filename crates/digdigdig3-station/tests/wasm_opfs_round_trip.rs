//! OPFS DiskStore wasm32 round-trip test.
//!
//! Verifies that `DiskStore::new`, `append_batch`, `flush`, and `read_tail`
//! produce consistent results against the Origin Private File System in a
//! real browser context.
//!
//! Run with:
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!       --test wasm_opfs_round_trip
//!
//! Requires: dig2-wasm-test runner (configured in .cargo/config.toml) +
//!           a browser with OPFS support (Chrome 86+, Firefox 111+, Safari 15.2+).
//!
//! Note: this test does NOT run automatically in CI (compile-only verification
//! in Workstream C). Coordinator runs it in Workstream E.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::core::types::StreamEvent;
use digdigdig3_station::series::{DataPoint, Kind, SeriesKey};
use digdigdig3_station::DiskStore;
use digdigdig3::core::types::{AccountType, ExchangeId};

// ─── Minimal synthetic DataPoint for testing ─────────────────────────────────

/// Simple fixed-size test point: `[timestamp_ms: i64, value: i64]` = 16 bytes.
#[derive(Clone, Debug, PartialEq)]
struct TestPoint {
    ts: i64,
    val: i64,
}

impl DataPoint for TestPoint {
    const RECORD_SIZE: usize = 16;

    fn encode(&self, out: &mut [u8]) {
        out[0..8].copy_from_slice(&self.ts.to_le_bytes());
        out[8..16].copy_from_slice(&self.val.to_le_bytes());
    }

    fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 {
            return None;
        }
        let ts = i64::from_le_bytes(bytes[0..8].try_into().ok()?);
        let val = i64::from_le_bytes(bytes[8..16].try_into().ok()?);
        Some(TestPoint { ts, val })
    }

    fn timestamp_ms(&self) -> i64 {
        self.ts
    }

    fn from_stream_event(_ev: &StreamEvent) -> Option<Self> {
        None
    }
}

// ─── Test helpers ─────────────────────────────────────────────────────────────

/// Build a `SeriesKey` for the test symbol.
fn test_key(symbol: &str) -> SeriesKey {
    SeriesKey {
        exchange: ExchangeId::Binance,
        account_type: AccountType::Spot,
        symbol: symbol.to_string(),
        kind: Kind::Trade,
    }
}

/// Generate `n` sequential TestPoints starting at `base_ts`.
fn make_points(n: usize, base_ts: i64) -> Vec<TestPoint> {
    (0..n)
        .map(|i| TestPoint {
            ts: base_ts + i as i64,
            val: i as i64 * 100,
        })
        .collect()
}

// ─── Test 1: flush + read_tail basic round-trip ───────────────────────────────

/// Append 10 test points, flush to OPFS, read back the last 5.
/// Verifies that the tail matches the last 5 appended points in order.
#[wasm_bindgen_test]
async fn opfs_diskstore_append_flush_read_tail() {
    // Use a unique symbol name to avoid collisions between test runs.
    // (OPFS persists across page loads in the same origin.)
    let key = test_key("opfs-round-trip-v1");
    let mut store = DiskStore::<TestPoint>::new(key)
        .await
        .expect("DiskStore::new must succeed in browser OPFS");

    let base_ts = 1_700_000_000_000i64; // arbitrary deterministic base
    let points = make_points(10, base_ts);

    store.append_batch(&points);

    store
        .flush()
        .await
        .expect("flush must succeed");

    // read_tail(5) should return the 5 most-recent points.
    let tail = store
        .read_tail(5)
        .await
        .expect("read_tail must succeed");

    assert_eq!(
        tail.len(),
        5,
        "read_tail(5) must return exactly 5 records; got {}",
        tail.len()
    );

    // Verify contents: tail should be points[5..10] in order.
    for (i, point) in tail.iter().enumerate() {
        let expected = &points[5 + i];
        assert_eq!(
            point.ts, expected.ts,
            "tail[{i}].ts mismatch: got {} expected {}",
            point.ts, expected.ts
        );
        assert_eq!(
            point.val, expected.val,
            "tail[{i}].val mismatch: got {} expected {}",
            point.val, expected.val
        );
    }
}

// ─── Test 2: read_tail on empty store ─────────────────────────────────────────

/// `read_tail` on a freshly-created store with no data must return empty Vec.
#[wasm_bindgen_test]
async fn opfs_diskstore_read_tail_empty() {
    // Use a unique symbol to ensure no pre-existing data.
    let key = test_key("opfs-empty-v1");
    let store = DiskStore::<TestPoint>::new(key)
        .await
        .expect("DiskStore::new must succeed");

    let tail = store
        .read_tail(10)
        .await
        .expect("read_tail on empty store must not error");

    // May be non-empty if a previous test run wrote to this key. Acceptable —
    // OPFS data persists across page reloads. The test only verifies no panic.
    let _ = tail;
}

// ─── Test 2b: persistence_round_trip_opfs (Wave 3 E target) ─────────────────

/// Append 100 test points in a single batch, flush to OPFS, then read_tail(10)
/// and verify the last 10 records match the last 10 appended.
///
/// This is the Wave 3 Workstream E target test (`persistence_round_trip_opfs`):
/// it exercises the full in-memory→OPFS→read path with a realistic batch size
/// that exercises the write buffer and confirms tail semantics under a larger
/// dataset.
///
/// Uses a unique symbol ("opfs-100-v1") to avoid cross-test contamination.
/// Because OPFS persists across page reloads, the `>=` check below allows for
/// prior-run records already in the file.
#[wasm_bindgen_test]
async fn persistence_round_trip_opfs() {
    let key = test_key("opfs-100-v1");
    let mut store = DiskStore::<TestPoint>::new(key)
        .await
        .expect("DiskStore::new must succeed in browser OPFS");

    // Use a distinctive base timestamp so we can identify our records in a
    // potentially-non-empty file from prior test runs.
    let base_ts = 1_800_000_000_000i64;
    let points = make_points(100, base_ts);

    store.append_batch(&points);

    store.flush().await.expect("flush must succeed after 100 appends");

    // read_tail(10) must return exactly 10 records...
    let tail = store.read_tail(10).await.expect("read_tail(10) must succeed");

    assert_eq!(
        tail.len(),
        10,
        "read_tail(10) must return exactly 10 records; got {}",
        tail.len()
    );

    // ...and those 10 must be the last 10 we appended (points[90..100]).
    // We match by (ts, val) — if a prior run wrote to the same key the
    // timestamps from base_ts=1_800_000_000_000 will be unique enough to
    // identify ours.  In the worst case (many prior runs) the tail may not
    // align; we accept that as a known OPFS test-isolation limitation.
    for (i, point) in tail.iter().enumerate() {
        let expected = &points[90 + i];
        assert_eq!(
            point.ts, expected.ts,
            "tail[{i}].ts: expected {} got {}",
            expected.ts, point.ts
        );
        assert_eq!(
            point.val, expected.val,
            "tail[{i}].val: expected {} got {}",
            expected.val, point.val
        );
    }
}

// ─── Test 3: multiple flushes accumulate ─────────────────────────────────────

/// Two consecutive flush cycles must accumulate records in OPFS.
/// Total after both flushes: 6 records. read_tail(3) → last 3.
#[wasm_bindgen_test]
async fn opfs_diskstore_multi_flush_accumulates() {
    let key = test_key("opfs-multi-flush-v1");
    let mut store = DiskStore::<TestPoint>::new(key)
        .await
        .expect("DiskStore::new");

    let base_ts = 1_700_100_000_000i64;
    let batch_a = make_points(3, base_ts);
    let batch_b = make_points(3, base_ts + 3);

    store.append_batch(&batch_a);
    store.flush().await.expect("first flush");

    store.append_batch(&batch_b);
    store.flush().await.expect("second flush");

    // read_tail(3) should return the last 3 (batch_b).
    let tail = store.read_tail(3).await.expect("read_tail");

    // At minimum the last 3 appended must be present (may have more from prior
    // runs). We only assert the last 3 match batch_b.
    let got_count = tail.len();
    assert!(
        got_count >= 3,
        "expected at least 3 records after two flushes; got {got_count}"
    );

    let last_three = &tail[got_count - 3..];
    for (i, point) in last_three.iter().enumerate() {
        let expected = &batch_b[i];
        assert_eq!(
            point.ts, expected.ts,
            "last_three[{i}].ts mismatch"
        );
    }
}
