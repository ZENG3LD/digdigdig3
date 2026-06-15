//! Wasm end-to-end Station tests — real browser, real WS, real events.
//!
//! Verifies that `Station::subscribe` + `SubscriptionHandle::recv` work in a
//! headless Chrome environment via the dig2-wasm-test runner.
//!
//! Run with:
//!   cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!       --test wasm_station_smoke
//!
//! Requires: dig2-wasm-test runner in PATH (configured in .cargo/config.toml).
//!
//! Architecture note: on wasm32, Station connects WebSocket via
//! UniversalWsTransport + browser-native WS (web-sys). The factory only
//! supports Binance/Bybit/OKX on wasm; other exchanges return
//! NotImplemented from connect_websocket and land in report.failed.

#![cfg(target_arch = "wasm32")]

use std::collections::HashSet;
use std::time::Duration;

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3::core::types::{AccountType, ExchangeId};
use digdigdig3_station::{Event, Station, Stream, SubscriptionSet};

// ─── helpers ─────────────────────────────────────────────────────────────────

/// Poll `handle.recv()` for up to `max_wait`, returning the first event.
/// Uses gloo_timers for the deadline — safe on wasm32 (no tokio::time::timeout).
async fn wait_for_event(
    handle: &mut digdigdig3_station::SubscriptionHandle,
    max_wait: Duration,
) -> Option<Event> {
    use futures_util::{future::Either, pin_mut};

    let recv_fut = handle.recv();
    let sleep_fut = gloo_timers::future::sleep(max_wait);
    pin_mut!(recv_fut, sleep_fut);

    match futures_util::future::select(recv_fut, sleep_fut).await {
        Either::Left((ev, _)) => ev,
        Either::Right(_) => None,
    }
}

// ─── Test 1: Binance BTCUSDT Trade ───────────────────────────────────────────

/// Subscribe to Binance BTCUSDT Trade stream in the browser and verify that at
/// least one Event arrives within 15 seconds.
///
/// BTCUSDT trade stream fires ~10-50 frames/s — a hit within 2s is expected.
/// The 15s budget (15 × 1s windows) guards against momentary exchange latency.
#[wasm_bindgen_test]
async fn station_binance_trade_recv_event() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build must succeed on wasm");

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );

    let mut report = station.subscribe(set).await.expect("subscribe must not return Err");

    // Binance Spot Trade WS is supported on wasm (UniversalWsTransport + browser WS).
    // If it lands in failed, something is structurally broken.
    assert!(
        report.failed.is_empty(),
        "expected no failures for Binance Trade; got: {:?}",
        report.failed.iter().map(|f| format!("{:?}: {}", f.stream, f.error)).collect::<Vec<_>>()
    );

    // Wait for at least 1 event. Poll with 1s windows, up to 15 total iterations.
    let mut got_event = false;
    for _ in 0..15 {
        match wait_for_event(&mut report.handle, Duration::from_secs(1)).await {
            Some(_ev) => {
                got_event = true;
                break;
            }
            None => continue,
        }
    }

    assert!(
        got_event,
        "expected at least 1 Trade event from Binance BTCUSDT in 15s"
    );
}

// ─── Test 2: Multi-venue subscribe ───────────────────────────────────────────

/// Subscribe to Binance + Bybit + OKX Trade streams in parallel and verify
/// that at least 1 venue delivers events within 20s.
///
/// CORS/Origin behaviour may differ per exchange — some may reject browser WS
/// connections even though our transport supports them. The test passes as long
/// as at least 1 venue delivers a live event.
///
/// # Why this test is ignored
///
/// Same root cause as `station_binance_trade_recv_event` — `std::time::Instant`
/// panics on wasm32. All three venues (Binance, Bybit, OKX) use
/// `UniversalWsTransport` which calls `Instant::now()` at construction.
/// See that test's doc-comment for the fix path.
#[wasm_bindgen_test]
async fn station_multi_venue_subscribe_in_browser() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build must succeed");

    let set = SubscriptionSet::new()
        .add(ExchangeId::Binance, "BTC-USDT", AccountType::Spot, [Stream::Trade])
        .add(ExchangeId::Bybit, "BTCUSDT", AccountType::FuturesCross, [Stream::Trade])
        .add(ExchangeId::OKX, "BTC-USDT-SWAP", AccountType::FuturesCross, [Stream::Trade]);

    let mut report = station.subscribe(set).await.expect("subscribe must not return Err");

    let ok_exchanges: Vec<ExchangeId> = report.ok.iter().map(|k| k.exchange).collect();
    let failed_streams: Vec<String> = report
        .failed
        .iter()
        .map(|f| format!("{:?}/{:?}: {}", f.exchange, f.stream, f.error))
        .collect();

    // At least 1 venue must have subscribed successfully.
    assert!(
        !report.ok.is_empty(),
        "expected at least 1 venue to succeed; all failed: {:?}",
        failed_streams
    );

    // Collect events for up to 20s; track which exchanges actually delivered.
    let mut exchanges_seen: HashSet<ExchangeId> = HashSet::new();
    for _ in 0..20 {
        match wait_for_event(&mut report.handle, Duration::from_secs(1)).await {
            Some(ev) => {
                exchanges_seen.insert(ev.exchange());
                if exchanges_seen.len() >= 2 {
                    break;
                }
            }
            None => continue,
        }
    }

    assert!(
        !exchanges_seen.is_empty(),
        "expected events from at least 1 venue within 20s; ok={:?}",
        ok_exchanges
    );
}

// ─── Test 3: REST base override plumbing ─────────────────────────────────────

/// Verify that `ExchangeHub::set_rest_base_override` / `get_rest_base_override`
/// are correctly plumbed on wasm32.
///
/// Full proxy-routed REST fetch is NOT tested here because:
///   a) No CORS proxy is reliably available at test time.
///   b) Connector-side consumption of the override (substituting the base URL
///      in each REST method) is a connector-level change deferred to v1.1.
///
/// This test proves the override storage mechanism works. Once the connectors
/// read the override from the hub, a proxy-fetch test can be added on top.
#[wasm_bindgen_test]
async fn rest_via_proxy_override_plumbing() {
    use digdigdig3::connector_manager::ExchangeHub;

    let hub = ExchangeHub::new();

    // Default: no override set.
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        None,
        "no override should be set initially"
    );

    // Set an override URL.
    hub.set_rest_base_override(ExchangeId::Binance, "https://example.com/api".to_string());
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        Some("https://example.com/api".to_string()),
        "override must be stored and retrievable"
    );

    // Different exchange must be unaffected.
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Bybit),
        None,
        "Bybit must be unaffected by Binance override"
    );

    // Clearing via empty string.
    hub.set_rest_base_override(ExchangeId::Binance, String::new());
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::Binance),
        None,
        "empty string must clear the override"
    );

    // Explicit clear.
    hub.set_rest_base_override(ExchangeId::OKX, "https://proxy.example.com".to_string());
    hub.clear_rest_base_override(ExchangeId::OKX);
    assert_eq!(
        hub.get_rest_base_override(ExchangeId::OKX),
        None,
        "clear_rest_base_override must remove the entry"
    );

}

// ─── Test 4: Reconnect after force-close — SKIPPED ───────────────────────────
//
// Forcing a WS close from the test side is not straightforward: the actor
// owns the connection and runs as a `spawn_local` task. Options would be:
//   a) Expose a "force-disconnect" hook on the WS connector (invasive API change).
//   b) Let Binance time us out via subscription expiry (slow, ~60s).
//   c) Use a mock WS server that closes on demand (requires test infra).
//
// The reconnect logic is exercised continuously in production whenever the
// exchange drops the connection (ping timeout, server restart, rate-limit).
// For the wasm smoke layer, the existing transport-level reconnect tests
// (UniversalWsTransport internal) plus the living proof of Test 1/2 passing
// after a clean subscribe are sufficient for v1 validation.
//
// A dedicated reconnect test is deferred to a future iteration when a
// controllable mock WS endpoint is available.
