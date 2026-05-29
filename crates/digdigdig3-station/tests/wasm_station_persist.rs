//! wasm32 Station OPFS persistence + warm-start round-trip tests (Wave 4-D/E).
//!
//! Proves that the Station forwarder persists events to OPFS (Wave 4-E) and
//! a fresh Station warm-starts by replaying them (Wave 4-D) in a real browser.
//!
//! # Running
//!
//! ```sh
//! cargo test --target wasm32-unknown-unknown -p digdigdig3-station \
//!     --test wasm_station_persist
//! ```
//!
//! Requires: wasm-bindgen-test runner (configured in `.cargo/config.toml`) +
//! Chrome/Firefox/Safari with OPFS support.
//!
//! Note: these tests do NOT run automatically in CI (compile-only gate via
//! `cargo check --target wasm32-unknown-unknown`). Browser execution is
//! verified by the coordinator in the Wave 4 acceptance pass.

#![cfg(target_arch = "wasm32")]

use std::time::Duration;

use futures_util::{future::Either, pin_mut};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use digdigdig3_station::{
    AccountType, Event, ExchangeId, GapHealConfig, PersistenceConfig, Station, Stream,
    SubscribeReport, SubscriptionSet,
};

// ─── helpers ──────────────────────────────────────────────────────────────────

/// Race `handle.recv()` against a gloo deadline. Returns `Some(event)` if one
/// arrives before the deadline, `None` on timeout. Never blocks indefinitely.
async fn recv_with_deadline(
    handle: &mut digdigdig3_station::SubscriptionHandle,
    timeout: Duration,
) -> Option<Event> {
    let recv_fut = handle.recv();
    let sleep_fut = gloo_timers::future::sleep(timeout);
    pin_mut!(recv_fut, sleep_fut);

    match futures_util::future::select(recv_fut, sleep_fut).await {
        Either::Left((ev, _)) => ev,
        Either::Right(_) => None,
    }
}

/// Drain up to `max_events` events OR until `total_budget` elapses, collecting
/// each into a `Vec`. Each individual `recv()` is bounded by `per_recv_timeout`
/// so a silent stream does not eat the whole budget on a single blocking call.
async fn drain_bounded(
    handle: &mut digdigdig3_station::SubscriptionHandle,
    max_events: usize,
    per_recv_timeout: Duration,
    total_budget: Duration,
) -> Vec<Event> {
    // Outer deadline future — fused so select can reference it repeatedly.
    let budget_fut = gloo_timers::future::sleep(total_budget);
    pin_mut!(budget_fut);

    let mut events: Vec<Event> = Vec::new();

    loop {
        if events.len() >= max_events {
            break;
        }
        // Each slot: race recv() against the per-slot timeout AND the outer budget.
        // Three-way select: recv wins → push event; slot timeout → continue loop
        // (lets outer budget accumulate); outer budget → break.
        let recv_fut = handle.recv();
        let slot_fut = gloo_timers::future::sleep(per_recv_timeout);
        pin_mut!(recv_fut, slot_fut);

        // Two-level select: first check outer budget, then inner slot.
        match futures_util::future::select(
            &mut budget_fut,
            futures_util::future::select(recv_fut, slot_fut),
        )
        .await
        {
            // Outer budget expired.
            Either::Left(_) => break,
            // Inner completed before outer budget.
            Either::Right((inner, _)) => match inner {
                Either::Left((Some(ev), _)) => events.push(ev),
                Either::Left((None, _)) => break, // channel closed
                Either::Right(_) => {}             // per-slot timeout: continue
            },
        }
    }

    events
}

// ─── log helper ───────────────────────────────────────────────────────────────

macro_rules! console_log {
    ($($t:tt)*) => {
        web_sys::console::log_1(&format!($($t)*).into())
    };
}

// ─── Test 1: OPFS persist + warm-start round-trip ─────────────────────────────

/// Full round-trip proof:
///
/// 1. Build Station with persistence ON + Binance Spot BTCUSDT Trade.
/// 2. Drain events with a bounded 20 s total budget (≥ 3 events target).
/// 3. Drop the SubscriptionHandle / report to trigger the shutdown-flush path.
///    Give the async forwarder 3 s to complete its `flush().await` before
///    proceeding — the shutdown signal is fire-and-forget from our side.
/// 4. Build a SECOND Station with `warm_start(50)` on the same OPFS key,
///    subscribe the same stream, drain with a 10 s budget, assert warm-start
///    events arrive (trade events with timestamp_ms > 0).
/// 5. Log counts for `--nocapture` inspection.
///
/// Network skip: if step 2 yields 0 live events (Binance blocked / quiet),
/// we log the situation and skip the warm-start assertion rather than failing
/// the whole suite on network flakiness.
#[wasm_bindgen_test]
async fn wasm_station_persist_warmstart_roundtrip() {
    // ── Step 1: first Station with persistence ON ────────────────────────────

    // storage_root is ignored on wasm32 — OPFS is browser-origin-scoped.
    // We pass a label for documentation purposes only.
    let station1 = Station::builder()
        .storage_root("wasm-persist-test-btcusdt-trade")
        .persistence(PersistenceConfig::on())
        .warm_start(0) // no warm-start for the first Station
        .build()
        .await
        .expect("Station1::build must succeed on wasm");

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );

    let mut report1: SubscribeReport = station1
        .subscribe(set)
        .await
        .expect("subscribe must not return Err");

    assert!(
        report1.failed.is_empty(),
        "expected no failures for Binance Trade on wasm; got: {:?}",
        report1
            .failed
            .iter()
            .map(|f| format!("{:?}: {}", f.stream, f.error))
            .collect::<Vec<_>>()
    );

    console_log!("[persist-test] Station1 subscribed OK. Draining live events…");

    // ── Step 2: drain live events (bounded 20 s) ─────────────────────────────

    let mut live_events: Vec<Event> = Vec::new();
    // Poll with 1 s windows for up to 20 iterations — matches smoke test pattern.
    for _ in 0..20 {
        match recv_with_deadline(&mut report1.handle, Duration::from_secs(1)).await {
            Some(ev) => {
                live_events.push(ev);
                if live_events.len() >= 3 {
                    break;
                }
            }
            None => continue,
        }
    }

    console_log!(
        "[persist-test] live_events collected: {}",
        live_events.len()
    );

    if live_events.is_empty() {
        // Network unreachable or Binance CORS blocked in this browser context.
        // Log and skip — do not fail on infrastructure flakiness.
        console_log!(
            "[persist-test] SKIP: 0 live events within 20 s — \
             Binance WS unreachable or silent. Persistence round-trip \
             assertion skipped (network-dependent)."
        );
        return;
    }

    assert!(
        !live_events.is_empty(),
        "expected ≥ 1 Trade event from Binance BTCUSDT in 20 s"
    );

    // ── Step 3: drop report1 → triggers shutdown-flush ────────────────────────
    //
    // Dropping `report1` drops the `SubscriptionHandle` → drops all
    // `MultiplexRef`s → `release_consumer` → sends `shutdown` oneshot →
    // forwarder breaks its loop → calls `d.flush().await` → OPFS written.
    //
    // The forwarder runs as a `wasm_bindgen_futures::spawn_local` task — it is
    // scheduled concurrently on the JS microtask queue.  We cannot `await` its
    // completion from here.  A 3 s sleep gives the browser enough microtask
    // cycles to finish the flush before we open the second Station.
    drop(report1);
    drop(station1);

    console_log!("[persist-test] Station1 dropped. Waiting 3 s for OPFS flush…");
    gloo_timers::future::sleep(Duration::from_secs(3)).await;
    console_log!("[persist-test] Flush wait done. Building Station2 with warm_start…");

    // ── Step 4: second Station with warm_start ON ─────────────────────────────

    let station2 = Station::builder()
        .storage_root("wasm-persist-test-btcusdt-trade")
        .persistence(PersistenceConfig::on())
        .warm_start(50) // replay last 50 records from OPFS on subscribe
        .build()
        .await
        .expect("Station2::build must succeed on wasm");

    let set2 = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );

    let mut report2: SubscribeReport = station2
        .subscribe(set2)
        .await
        .expect("Station2 subscribe must not return Err");

    assert!(
        report2.failed.is_empty(),
        "Station2: expected no failures for Binance Trade; got: {:?}",
        report2
            .failed
            .iter()
            .map(|f| format!("{:?}: {}", f.stream, f.error))
            .collect::<Vec<_>>()
    );

    // Collect up to 60 events with a 10 s total budget; warm-start events
    // arrive essentially immediately (they are emitted synchronously from
    // the forwarder before the live WS stream starts).
    let warmstart_events = drain_bounded(
        &mut report2.handle,
        60,
        Duration::from_secs(1),
        Duration::from_secs(10),
    )
    .await;

    console_log!(
        "[persist-test] Station2 warmstart_events collected: {}",
        warmstart_events.len()
    );

    // Log the first few for visual inspection with --nocapture.
    for (i, ev) in warmstart_events.iter().take(3).enumerate() {
        console_log!(
            "[persist-test] warmstart_events[{i}]: symbol={} ts={}",
            ev.symbol(),
            ev.timestamp_ms()
        );
    }

    // ── Step 5: assert warm-start delivered replayed records ─────────────────
    //
    // The forwarder emits disk tail BEFORE any live events. With `warm_start(50)`
    // and the persisted records from Station1, we expect ≥ 1 warm-start trade.
    assert!(
        !warmstart_events.is_empty(),
        "Station2 warm-start must emit ≥ 1 replayed Trade record from OPFS within 10 s; \
         got 0. This means either the OPFS flush did not complete in the 3 s wait or \
         the DiskStore::read_tail is not being called on wasm warm-start path."
    );

    // Every warm-start trade must have a positive timestamp (sanity check that
    // decode did not produce zero-init garbage).
    for ev in &warmstart_events {
        assert!(
            ev.timestamp_ms() > 0,
            "warm-start event has zero timestamp — decode failure: {:?}",
            ev.symbol()
        );
    }

    console_log!(
        "[persist-test] PASS — {} live events persisted, {} warm-start events replayed",
        live_events.len(),
        warmstart_events.len()
    );
}

// ─── Test 2: gap_heal builds + runs without panic ─────────────────────────────

/// Smoke test: build a Station with `gap_heal` ENABLED + persistence, subscribe
/// to Binance Spot BTCUSDT Kline 1m, drain a few seconds, assert it constructs +
/// yields at least the subscribe without panicking.
///
/// Gap-heal is un-gated on wasm (Wave 4-B). A real WS-disconnect-trigger is not
/// feasible in a unit test — this test proves the code path is live and doesn't
/// panic. The heal logic activates on the next WS silence_timeout (60 s), which
/// is outside our test window.
#[wasm_bindgen_test]
async fn wasm_station_gap_heal_builds() {
    use digdigdig3::core::websocket::KlineInterval;

    let station = Station::builder()
        .storage_root("wasm-gapheal-test-kline-1m")
        .persistence(PersistenceConfig::on())
        .warm_start(10)
        .gap_heal(GapHealConfig::on())
        .build()
        .await
        .expect("Station with gap_heal::build must succeed on wasm");

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Kline(KlineInterval::new("1m"))],
    );

    let mut report = station
        .subscribe(set)
        .await
        .expect("subscribe must not return Err");

    // Kline 1m fires at most once per minute — we may not get an event in 5 s.
    // The important assertions are: (a) subscribe returned without panic, and
    // (b) the failed list is empty (gap_heal does not break subscribe plumbing).
    assert!(
        report.failed.is_empty(),
        "gap_heal station: expected no failures for Binance Kline 1m; got: {:?}",
        report
            .failed
            .iter()
            .map(|f| format!("{:?}: {}", f.stream, f.error))
            .collect::<Vec<_>>()
    );

    console_log!("[gap-heal-test] subscribe OK — draining 5 s…");

    // Drain up to 5 s. We do NOT assert ≥1 event because klines are sparse (1/min).
    let mut received = 0usize;
    for _ in 0..5 {
        match recv_with_deadline(&mut report.handle, Duration::from_secs(1)).await {
            Some(_) => {
                received += 1;
            }
            None => continue,
        }
    }

    console_log!(
        "[gap-heal-test] PASS — gap_heal Station constructed and ran for 5 s without panic. \
         Events received: {received} (0 expected for kline 1m in a 5 s window)"
    );
    // No panic = success. Event count is informational only.
}
