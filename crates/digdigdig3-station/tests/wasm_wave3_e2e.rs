//! Wave 3 wasm end-to-end test suite — Workstream E.
//!
//! Tests Wave 3 features through the public Station / ExchangeHub API in a
//! real browser context.  Covers:
//!
//! - `rest_via_encoded_proxy_get_klines` — REST override end-to-end: Binance
//!   klines fetched from the browser through the codetabs encoded-proxy.
//!   Uses the `{url}` placeholder mode added in this wave: the full real target
//!   URL is percent-encoded and substituted into the proxy template.
//! - `polling_lsr_returns_unsupported_on_wasm` — negative test: LSR subscribe
//!   lands in `report.failed` with `StreamNotSupported` on wasm (native-only
//!   timer dependency; Station gracefully degrades).
//! - `persistence_round_trip_via_station_builder` — Station builder with
//!   `PersistenceConfig::default()` constructs without panic on wasm; DiskStore
//!   round-trip is verified separately in `wasm_opfs_round_trip.rs`.
//!
//! Run with:
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!       --test wasm_wave3_e2e
//!
//! Requires: dig2-wasm-test runner (configured in .cargo/config.toml) + a
//! browser with OPFS support (Chrome 86+, Firefox 111+, Safari 15.2+).
//!
//! Network tests (Test 1) require outbound HTTPS from the browser process.
//! In headless CI without outbound access, Test 1 will fail at the REST call;
//! the coordinator is expected to run these against a network-enabled browser.

#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::connector_manager::ExchangeHub;
use digdigdig3::core::types::{AccountType, ExchangeId, SymbolInput};
use digdigdig3_station::{Station, Stream, SubscriptionSet};

// ─── Test 1: REST via encoded-proxy — end-to-end klines fetch ────────────────

/// Set `rest_base_override` to an encoded-proxy template and call `get_klines`
/// for Binance BTCUSDT.  Asserts ≥1 real kline is returned.
///
/// ## How the `{url}` template mode works
///
/// When the override string contains the literal `{url}`, `assemble_rest_url`
/// percent-encodes the full real target (`real_base + path + query`) and
/// substitutes it for `{url}` before making the request.  Example:
///
/// ```text
/// override:     "https://api.codetabs.com/v1/proxy/?quest={url}"
/// real target:  "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&..."
/// final URL:    "https://api.codetabs.com/v1/proxy/?quest=https%3A%2F%2Fapi.binance.com%2F..."
/// ```
///
/// `api.codetabs.com/v1/proxy/` returns the upstream body verbatim with
/// `Access-Control-Allow-Origin: *`, which satisfies the browser's preflight
/// check.
///
/// ## Failure modes
///
/// If the test fails at the REST call the error message distinguishes:
/// - Network error / wrong encoding → the encoded URL was malformed or
///   codetabs rejected it.
/// - Parse error → our JSON parser failed on the upstream body.
/// - HTTP 4xx/5xx → codetabs is down or Binance returned an error.
///
/// Prints kline count on success as proof that real data flowed through Chrome.
#[wasm_bindgen_test]
async fn rest_via_encoded_proxy_get_klines() {
    let hub = ExchangeHub::new();

    // Encoded-proxy template: api.codetabs.com/v1/proxy/ returns the upstream
    // body verbatim with Access-Control-Allow-Origin: *.  The {url} placeholder
    // is replaced at request time with the percent-encoded full Binance target.
    hub.set_rest_base_override(
        ExchangeId::Binance,
        "https://api.codetabs.com/v1/proxy/?quest={url}".to_string(),
    );

    hub.connect_public(ExchangeId::Binance, false)
        .await
        .expect("connect_public must succeed with codetabs encoded-proxy override set");

    let rest = hub
        .rest(ExchangeId::Binance)
        .expect("REST connector must be present after connect_public");

    // limit=1 keeps the response tiny.
    let klines = rest
        .get_klines(
            SymbolInput::Raw("BTCUSDT"),
            "1m",
            Some(1),
            AccountType::Spot,
            None,
        )
        .await
        .expect("get_klines via codetabs encoded-proxy must succeed");

    // Print kline count as proof that real data flowed through Chrome.
    web_sys::console::log_1(
        &format!("rest_via_encoded_proxy_get_klines: {} kline(s) received", klines.len()).into()
    );

    assert!(
        !klines.is_empty(),
        "expected ≥1 kline from Binance BTCUSDT via codetabs encoded proxy; got 0"
    );

    // Verify the kline has sensible data (open > 0).
    let k = &klines[0];
    assert!(
        k.open > 0.0,
        "kline.open must be positive; got {}",
        k.open
    );
}

// ─── Test 2: LSR polling returns StreamNotSupported on wasm ──────────────────

/// Subscribe to `Stream::LongShortRatio` for Binance BTCUSDT on wasm and assert
/// it lands in `report.failed` with a `StreamNotSupported`-shaped error.
///
/// Why this is the expected behavior:
///   `LongShortRatioPoll` compiles on wasm (types are un-gated) but
///   `spawn_poller` is `#[cfg(not(target_arch = "wasm32"))]`.  Station's
///   `acquire_or_spawn` path reaches the poll-spawn gate and returns
///   `StationError::StreamNotSupported`.  The consumer-facing contract is:
///   "LSR is unavailable in browser; subscribe returns a failure entry with a
///   clear reason so the consumer can fall back gracefully."
///
/// This is a NEGATIVE test — we assert the failure rather than success.
/// Passing proves graceful degradation, not a bug.
#[wasm_bindgen_test]
async fn polling_lsr_returns_unsupported_on_wasm() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build must succeed on wasm");

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::FuturesCross,
        [Stream::LongShortRatio],
    );

    let report = station
        .subscribe(set)
        .await
        .expect("subscribe must not return Err at the batch level");

    // The LSR stream must land in `failed`, not in `ok`.
    assert!(
        report.ok.is_empty(),
        "LSR must not succeed on wasm; ok = {:?}",
        report.ok
    );
    assert!(
        !report.failed.is_empty(),
        "LSR subscribe must produce a failure entry on wasm"
    );

    // The failure reason must reference "not supported" or "StreamNotSupported".
    let failure = &report.failed[0];
    let err_str = failure.error.to_string().to_lowercase();
    assert!(
        err_str.contains("not supported") || err_str.contains("notsupported"),
        "failure reason must indicate stream-not-supported; got: {:?}",
        failure.error
    );
}

// ─── Test 3: Station builder round-trip — PersistenceConfig wasm path ────────

/// Verify that `Station::builder()` with default `PersistenceConfig` (disabled)
/// constructs successfully on wasm and `active_streams()` starts at 0.
///
/// This is a smoke test for the builder's wasm path.  It does NOT exercise OPFS
/// I/O — that is covered by `wasm_opfs_round_trip.rs` (Workstream C).
///
/// A subscribe call is omitted here because the OPFS-backed persistence pipeline
/// (writing live trade events to DiskStore) requires waiting for actual WS
/// events, which is covered end-to-end by the Workstream C integration tests.
/// Separating builder-construct from subscribe-and-wait keeps this test fast
/// and deterministic (no network dependency).
#[wasm_bindgen_test]
async fn persistence_station_builder_wasm_path() {
    use digdigdig3_station::PersistenceConfig;

    // Persistence disabled (default).  On wasm, storage_root is unused.
    let station = Station::builder()
        .persistence(PersistenceConfig::default())
        .build()
        .await
        .expect("Station with default PersistenceConfig must build on wasm");

    assert_eq!(
        station.active_streams(),
        0,
        "newly built station must have 0 active streams"
    );

    // Confirm the storage_root is accessible (path may be empty on wasm).
    let _root = station.storage_root();
}

// ─── Test 3b: gap_heal module compiles and constructs on wasm ────────────────

/// Verify that `GapHealConfig` constructs correctly on wasm32 and that
/// `heal_limit` / `kline_interval_to_duration` compute deterministically.
///
/// Why we test at construction level rather than live disconnect:
///   Forcing a WS disconnect from a test is invasive — the actor owns the
///   connection and the only clean way to trigger a heal would be either:
///   (a) expose a "force-disconnect" hook (API pollution), or
///   (b) let the exchange time us out (~60s — too slow for a unit test).
///
///   The actual gap-fill REST call fires in the Station forwarder loop on
///   native integration tests (`gap_heal_e2e.rs`). Here we verify the
///   wasm-side plumbing: `GapHealConfig` is un-gated and all pure-compute
///   helpers compile + produce correct values on wasm32.
///
///   Live disconnect simulation remains a native-only integration test
///   (see `crates/digdigdig3-station/tests/gap_heal_e2e.rs`).
#[wasm_bindgen_test]
async fn gap_heal_module_compiles_and_config_constructs() {
    use digdigdig3_station::GapHealConfig;
    use digdigdig3_station::gap_heal::{heal_limit, kline_interval_to_duration};

    // Default config: disabled, default_limit=300, max_limit=1000.
    let cfg = GapHealConfig::default();
    assert!(!cfg.enabled, "default GapHealConfig must be disabled");
    assert_eq!(cfg.default_limit, 300);
    assert_eq!(cfg.max_limit, 1000);

    // Builder API.
    let cfg_on = GapHealConfig::on().default_limit(50).max_limit(500);
    assert!(cfg_on.enabled);
    assert_eq!(cfg_on.default_limit, 50);
    assert_eq!(cfg_on.max_limit, 500);

    // Interval parsing.
    use std::time::Duration;
    assert_eq!(kline_interval_to_duration("1m"), Some(Duration::from_secs(60)));
    assert_eq!(kline_interval_to_duration("1h"), Some(Duration::from_secs(3600)));
    assert_eq!(kline_interval_to_duration("1d"), Some(Duration::from_secs(86400)));
    assert_eq!(kline_interval_to_duration("bad"), None);

    // heal_limit: no gap, returns default_limit (capped to max_limit).
    let limit = heal_limit(&cfg_on, "1m", 0, 0);
    assert_eq!(limit, cfg_on.default_limit.min(cfg_on.max_limit));

    // heal_limit: 10-minute gap on 1m bars → need=10, but default_limit=50 wins.
    let now_ms = 1_700_000_600_000i64;
    let last_ms = now_ms - 10 * 60 * 1000; // 10 minutes ago
    let limit2 = heal_limit(&cfg_on, "1m", last_ms, now_ms);
    assert!(limit2 >= 10, "heal_limit must be ≥ gap/interval = 10; got {limit2}");
    assert!(limit2 <= cfg_on.max_limit, "heal_limit must not exceed max_limit");
}

// ─── Test 4: REST override survives connect_public → rest() round-trip ────────

/// End-to-end check that:
/// 1. Override set BEFORE `connect_public` is forwarded to the factory.
/// 2. `hub.rest()` returns a connector (factory did not reject it).
/// 3. `hub.get_rest_base_override()` still reflects the stored value.
///
/// This complements Test 1 (which proves the HTTP round-trip) and Test 3 in
/// `wasm_station_smoke.rs` (which proves plumbing in isolation).  Here we
/// verify that `connect_public` does NOT consume/clear the override.
#[wasm_bindgen_test]
async fn rest_override_persists_after_connect_public() {
    let hub = ExchangeHub::new();

    let proxy = "https://api.codetabs.com/v1/proxy/?quest={url}".to_string();
    hub.set_rest_base_override(ExchangeId::Binance, proxy.clone());

    hub.connect_public(ExchangeId::Binance, false)
        .await
        .expect("connect_public must not error with override set");

    // REST connector must be present.
    assert!(
        hub.rest(ExchangeId::Binance).is_some(),
        "rest() must return Some after connect_public"
    );

    // Override must still be readable (connect_public must not clear it).
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        Some(proxy),
        "override must survive connect_public"
    );
}

// ─── Deferred tests — cure + replay (Wave 4 followup) ────────────────────────
//
// `cure_in_browser` (Wave 3 Workstream E target #5):
//   `IntegrityChecker` and `RepairPipeline` depend on `StorageManager` which in
//   turn uses `sled` (BTree on-disk) and `tokio::fs` — both unavailable on
//   wasm32. All cure types are cfg-gated `#[cfg(not(target_arch = "wasm32"))]`
//   in `lib.rs`. Porting cure to wasm would require replacing StorageManager
//   with an OPFS-backed equivalent — a Wave 4 task.
//
//   Until that port lands, `cure` is a native-only module and no wasm test
//   for it is possible. The native integration tests live in
//   `crates/digdigdig3-station/tests/cure.rs`.
//
// `replay_from_opfs` (Wave 3 Workstream E target #6):
//   `ReplayHub` reads from `StorageManager` which has the same sled/tokio::fs
//   dependency. It is cfg-gated identically. The native replay integration tests
//   live in `crates/digdigdig3-station/tests/replay.rs`.
//
//   Wave 4 plan: port StorageManager to a cfg-split design (native: sled/tokio::fs;
//   wasm: OPFS DiskStore index), then un-gate cure + replay and add wasm tests
//   for both modules here.
//
// No placeholder test functions are emitted — a `#[wasm_bindgen_test]` with a
// `compile_error!` body would fail to compile; a no-op test would give a
// misleading "passed" count. The deferred state is correctly captured here in
// documentation only.
