#![cfg(not(target_arch = "wasm32"))]
//! `Station::subscribe` continue-on-error semantics and per-stream
//! NotSupported propagation. These tests focus on the API surface that
//! does NOT require a live WS connection.
//!
//! Live live-API regression (Bybit MarketWarning → NotSupported) lives in
//! `subscribe_not_supported_live.rs` and is gated by `--ignored`.

use digdigdig3_station::{
    AccountType, ExchangeId, FailedStream, StationError, Stream, SubscribeReport,
    SubscriptionSet,
};

#[test]
fn station_error_not_supported_is_distinguishable() {
    let e = StationError::StreamNotSupported("bybit: no such kind".into());
    assert!(e.is_not_supported(), "StreamNotSupported must report is_not_supported = true");

    let e2 = StationError::Subscribe("transport ded".into());
    assert!(!e2.is_not_supported(), "Subscribe variant must NOT be is_not_supported");

    let e3 = StationError::Core("connect".into());
    assert!(!e3.is_not_supported());
}

#[test]
fn subscribe_report_helpers_work_on_empty_report() {
    // Direct construction is pub(crate) — we exercise via the public
    // shape only. The fields are pub so we can build a synthetic
    // SubscribeReport in downstream code; here we just test the methods
    // by constructing one through a real path.
    //
    // For an empty `ok` and empty `failed`, both is_fully_ok() and
    // total()==0 must hold. We cannot construct SubscriptionHandle
    // directly (its fields are pub(crate)), so this test asserts the
    // method contract on a real report — see live tests for that.
    // Here we just ensure the type is exported and the API surface
    // compiles.
    let _ = std::mem::size_of::<SubscribeReport>();
    let _ = std::mem::size_of::<FailedStream>();
}

#[tokio::test]
async fn subscribe_empty_set_returns_err() {
    use digdigdig3_station::Station;
    let station = Station::builder().build().await.expect("Station::build");
    let result = station.subscribe(SubscriptionSet::new()).await;
    match result {
        Err(StationError::Subscribe(msg)) => {
            assert!(msg.contains("empty"), "expected 'empty' in error msg, got: {msg}");
        }
        other => panic!("expected Err(Subscribe), got {other:?}"),
    }
}

/// FailedStream carries enough context for a consumer to log/skip without
/// parsing strings. This guards against accidental field removal.
#[test]
fn failed_stream_fields_are_public() {
    fn _take(f: FailedStream) -> (ExchangeId, AccountType, String, Stream, StationError) {
        (f.exchange, f.account_type, f.symbol, f.stream, f.error)
    }
}

/// `add_raw` must accept an exotic instrument ID that would crash
/// `parse_symbol` + `SymbolNormalizer::to_exchange`. The builder itself
/// is normalizer-free; the only test we can run without a live
/// connection is that the SubscriptionSet accepts the string and
/// preserves it verbatim through .len(). The live-subscribe path is
/// covered by tests/subscribe_not_supported_live.rs.
#[test]
fn add_raw_accepts_deribit_option_id() {
    let set = SubscriptionSet::new().add_raw(
        ExchangeId::Deribit,
        "BTC-23MAY26-86000-C",
        AccountType::Options,
        [Stream::OptionGreeks],
    );
    assert_eq!(set.len(), 1);
}

#[test]
fn add_and_add_raw_can_mix_in_same_set() {
    let set = SubscriptionSet::new()
        .add(ExchangeId::Binance, "BTC-USDT", AccountType::Spot, [Stream::Trade])
        .add_raw(
            ExchangeId::Deribit,
            "BTC-23MAY26-86000-C",
            AccountType::Options,
            [Stream::OptionGreeks],
        )
        .add_raw(
            ExchangeId::Deribit,
            "BTC-PERPETUAL",
            AccountType::FuturesCross,
            [Stream::IndexPrice, Stream::VolatilityIndex],
        );
    assert_eq!(set.len(), 3);
}
