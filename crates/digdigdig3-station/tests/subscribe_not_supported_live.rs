#![cfg(not(target_arch = "wasm32"))]
//! Live regression: WireAbsent subscribe MUST NOT spawn a forwarder.
//!
//! Reproduces MLI's 0.3.6 OOM trigger (see docs/plans/mli-0.3.6-findings.md):
//! Subscribing to a Bybit stream the venue does not expose used to silently
//! "succeed" at Station level; the forwarder would then heal/resub every
//! 60 s forever, leaking broadcast receivers. After 0.3.7, the same
//! subscribe must land in `SubscribeReport::failed` with
//! `StationError::StreamNotSupported`, and `Station::active_streams()` must
//! NOT count the failed stream.
//!
//! Live — requires network to Bybit. Gated `--ignored`.

use std::time::Duration;

use digdigdig3_station::{
    AccountType, ExchangeId, Station, StationError, Stream, SubscriptionSet,
};

#[tokio::test]
#[ignore] // live API
async fn bybit_market_warning_subscribe_lands_in_failed() {
    let station = Station::builder().build().await.expect("Station::build");

    // MarketWarning is not exposed by Bybit V5 public WS. Bybit's
    // `build_topic` returns NotImplemented for kinds outside the
    // supported set — transport propagates it, station maps to
    // StreamNotSupported.
    let set = SubscriptionSet::new().add(
        ExchangeId::Bybit,
        "BTCUSDT",
        AccountType::FuturesCross,
        [Stream::MarketWarning],
    );

    let report = station.subscribe(set).await.expect("subscribe call must Ok");

    assert!(
        report.failed.len() == 1,
        "expected exactly 1 failure, got failed={:?} ok={:?}",
        report.failed.len(), report.ok.len(),
    );
    let fail = &report.failed[0];
    assert_eq!(fail.exchange, ExchangeId::Bybit);
    assert_eq!(fail.account_type, AccountType::FuturesCross);
    assert!(
        fail.error.is_not_supported(),
        "expected StreamNotSupported, got: {:?}",
        fail.error,
    );
    assert!(matches!(fail.error, StationError::StreamNotSupported(_)));
    assert!(report.ok.is_empty(), "no stream should have succeeded");
    assert!(!report.is_fully_ok());
    assert_eq!(report.total(), 1);

    // Critical regression check: NO forwarder spawned for the failed
    // stream. active_streams counts entries in `inner.muxes` — if this
    // is non-zero, the bug is back.
    assert_eq!(
        station.active_streams(),
        0,
        "Station::subscribe must NOT register a mux for a WireAbsent stream",
    );
}

#[tokio::test]
#[ignore] // live API
async fn bybit_mixed_subscribe_continues_past_unsupported() {
    let station = Station::builder().build().await.expect("Station::build");

    // Mix: one supported (Trade), one NOT supported (MarketWarning).
    // Continue-on-error must subscribe the supported one and report the
    // other in `.failed`.
    let set = SubscriptionSet::new().add(
        ExchangeId::Bybit,
        "BTCUSDT",
        AccountType::FuturesCross,
        [Stream::Trade, Stream::MarketWarning],
    );

    let report = station.subscribe(set).await.expect("subscribe call must Ok");

    assert_eq!(report.ok.len(), 1, "Trade must succeed");
    assert_eq!(report.failed.len(), 1, "MarketWarning must fail");
    assert!(report.failed[0].error.is_not_supported());

    assert_eq!(
        station.active_streams(),
        1,
        "only the Trade mux should be registered",
    );

    // Drop the handle and ensure mux gets released cleanly (no hung
    // forwarders).
    drop(report);
    tokio::time::sleep(Duration::from_millis(200)).await;
}
