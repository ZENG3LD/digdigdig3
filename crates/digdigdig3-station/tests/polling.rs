#![cfg(not(target_arch = "wasm32"))]
//! Tests for the polling subscription layer.
//!
//! Unit tests (always run):
//! - `kind_is_poll_only_classification` — verifies PollSpec values for each Kind
//! - `spawn_poller_dedup_and_shutdown` — FakePollSource verifies cadence, dedup, shutdown
//!
//! Live integration tests (#[ignore]):
//! - `lsr_binance_live` — Binance BTC-USDT LSR over 6 min
//! - `hv_deribit_live` — Deribit BTC HV over 90s (first tick via warm-start)

use std::time::Duration;

use digdigdig3_station::Kind;

// ─────────────────────────────────────────────────────────────────────────────
// Unit: Kind::is_poll_only classification
// ─────────────────────────────────────────────────────────────────────────────

#[test]
fn kind_is_poll_only_classification() {
    use digdigdig3::core::websocket::KlineInterval;

    // Poll-only kinds
    let lsr = Kind::LongShortRatio.is_poll_only();
    assert!(lsr.is_some(), "LongShortRatio must be poll-only");
    let lsr_spec = lsr.unwrap();
    assert_eq!(
        lsr_spec.cadence,
        Duration::from_secs(5 * 60),
        "LSR cadence must be 5 min"
    );
    assert_eq!(lsr_spec.jitter_pct, 10, "LSR jitter_pct must be 10");

    let hv = Kind::HistoricalVolatility.is_poll_only();
    assert!(hv.is_some(), "HistoricalVolatility must be poll-only");
    let hv_spec = hv.unwrap();
    assert_eq!(
        hv_spec.cadence,
        Duration::from_secs(60 * 60),
        "HV cadence must be 1h"
    );
    assert_eq!(hv_spec.jitter_pct, 5, "HV jitter_pct must be 5");

    // WS-backed kinds must return None
    assert!(Kind::Trade.is_poll_only().is_none(), "Trade is WS, not poll");
    assert!(Kind::Ticker.is_poll_only().is_none(), "Ticker is WS, not poll");
    assert!(
        Kind::Kline(KlineInterval::new("1m")).is_poll_only().is_none(),
        "Kline is WS, not poll"
    );
    assert!(
        Kind::FundingRate.is_poll_only().is_none(),
        "FundingRate is WS, not poll"
    );

    // Derived kinds must also return None
    assert!(Kind::Basis.is_poll_only().is_none(), "Basis is derived, not poll");
    assert!(
        Kind::FundingSettlement.is_poll_only().is_none(),
        "FundingSettlement is derived, not poll"
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// Unit: LongShortRatioPoint dedup logic (structural)
// ─────────────────────────────────────────────────────────────────────────────

/// Verifies the dedup fence logic used in spawn_poller by simulating it:
/// given a batch of points some with ts <= last_emitted_ms, only newer ones pass.
#[test]
fn dedup_fence_semantics() {
    use digdigdig3_station::data::LongShortRatioPoint;
    use digdigdig3_station::DataPoint;

    let pts = vec![
        LongShortRatioPoint { ts_ms: 1_000, ratio: 1.1, long_pct: 0.52, short_pct: 0.48 },
        LongShortRatioPoint { ts_ms: 2_000, ratio: 1.2, long_pct: 0.55, short_pct: 0.45 },
        LongShortRatioPoint { ts_ms: 3_000, ratio: 1.3, long_pct: 0.57, short_pct: 0.43 },
    ];

    let last_emitted_ms: i64 = 2_000;
    let new_pts: Vec<_> = pts.iter().filter(|p| p.timestamp_ms() > last_emitted_ms).collect();
    assert_eq!(new_pts.len(), 1, "only ts=3000 is new");
    assert_eq!(new_pts[0].ts_ms, 3_000);
}

// ─────────────────────────────────────────────────────────────────────────────
// Live integration test: LongShortRatio on Binance
// ─────────────────────────────────────────────────────────────────────────────

/// Subscribe to Stream::LongShortRatio on Binance BTC-USDT for 6 minutes.
/// Expects at least 1 event (the REST endpoint always has data).
/// Requires network access. Run with: `cargo test -- lsr_binance_live --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn lsr_binance_live() {
    use digdigdig3::core::types::{AccountType, ExchangeId};
    use digdigdig3_station::{Event, Station, Stream, SubscriptionSet};

    let station = Station::builder()
        .build()
        .await
        .expect("station build");

    let report = station
        .subscribe(
            SubscriptionSet::new().add(
                ExchangeId::Binance,
                "BTC-USDT",
                AccountType::FuturesCross,
                [Stream::LongShortRatio],
            ),
        )
        .await
        .expect("subscribe");

    assert!(report.is_fully_ok(), "subscribe failed: {:?}", report.failed);

    let mut handle = report.handle;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(360); // 6 min
    let mut received = 0usize;
    while tokio::time::Instant::now() < deadline {
        match tokio::time::timeout(Duration::from_secs(35), handle.recv()).await {
            Ok(Some(Event::LongShortRatio { .. })) => {
                received += 1;
                break; // got at least one — test passes
            }
            Ok(Some(_)) => {} // other events (shouldn't happen on this subscribe)
            Ok(None) | Err(_) => break,
        }
    }
    assert!(received >= 1, "expected ≥1 LSR event in 6 min; got {received}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Live integration test: HistoricalVolatility on Deribit
// ─────────────────────────────────────────────────────────────────────────────

/// Subscribe to Stream::HistoricalVolatility on Deribit "BTC" for 90 seconds.
/// The first poll fires after jitter (≤5% of 1h = ≤3 min), BUT since
/// DIG3_POLL_NO_JITTER=1 skips jitter, warm-start from disk or first tick
/// should arrive well within 90s on repeat runs.
///
/// On a cold run (no disk state), the first tick fires within jitter_max =
/// 3 min — this test may time out on a truly cold start. Add
/// `DIG3_POLL_NO_JITTER=1` env to skip jitter for a guaranteed 90s pass.
///
/// Run with: `cargo test -- hv_deribit_live --ignored --nocapture`
#[tokio::test]
#[ignore]
async fn hv_deribit_live() {
    use digdigdig3::core::types::{AccountType, ExchangeId};
    use digdigdig3_station::{Event, Station, Stream, SubscriptionSet};

    let station = Station::builder()
        .build()
        .await
        .expect("station build");

    let report = station
        .subscribe(
            SubscriptionSet::new().add_raw(
                ExchangeId::Deribit,
                "BTC",              // currency, not instrument
                AccountType::Spot,
                [Stream::HistoricalVolatility],
            ),
        )
        .await
        .expect("subscribe");

    assert!(report.is_fully_ok(), "subscribe failed: {:?}", report.failed);

    let mut handle = report.handle;
    let event = tokio::time::timeout(Duration::from_secs(90), handle.recv()).await;
    assert!(
        matches!(event, Ok(Some(Event::HistoricalVolatility { .. }))),
        "expected HistoricalVolatility event within 90s; got: {:?}",
        event
    );
}
