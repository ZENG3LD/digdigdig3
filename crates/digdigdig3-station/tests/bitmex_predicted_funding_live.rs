#![cfg(not(target_arch = "wasm32"))]
//! Live integration test: BitMEX `instrument` WS channel emits
//! `StreamEvent::PredictedFunding` via `indicativeFundingRate` field.
//!
//! Gated with `--ignored`. Run with:
//!   cargo test -p digdigdig3-station --test bitmex_predicted_funding_live -- --ignored --nocapture

use std::time::Duration;

use digdigdig3_station::{AccountType, ExchangeId, Station, Stream, SubscriptionSet};

#[tokio::test]
#[ignore]
async fn bitmex_predicted_funding_xbtusd_emits_events() {
    let station = Station::builder().build().await.expect("Station::build");

    // XBTUSD: BitMEX perpetual — indicativeFundingRate is populated every second.
    let set = SubscriptionSet::new().add_raw(
        ExchangeId::Bitmex,
        "XBTUSD",
        AccountType::FuturesCross,
        [Stream::PredictedFunding],
    );

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe should not fail for XBTUSD PredictedFunding: {:?}",
        report.failed
    );

    let mut got = 0usize;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(30);
    loop {
        if tokio::time::Instant::now() >= deadline {
            break;
        }
        let remaining = deadline - tokio::time::Instant::now();
        let r = tokio::time::timeout(
            remaining.min(Duration::from_secs(5)),
            report.handle.recv(),
        )
        .await;
        match r {
            Ok(Some(ev)) => {
                println!("got event: {:?}", ev);
                got += 1;
                if got >= 1 {
                    break;
                }
            }
            Ok(None) => break,
            Err(_) => continue,
        }
    }

    assert!(
        got >= 1,
        "expected at least 1 PredictedFunding event from XBTUSD within 30s — got {got}"
    );
}
