//! Live integration test: OKX funding-rate channel emits StreamEvent::PredictedFunding
//! for coin-margined inverse SWAPs (BTC-USD-SWAP, ETH-USD-SWAP).
//!
//! BTC-USD-SWAP uses `method: "next_period"` — nextFundingRate is populated.
//! BTC-USDT-SWAP uses `method: "current_period"` — nextFundingRate is `""`,
//! so no PredictedFunding events are emitted (correct filtering behaviour).
//!
//! Gated with `--ignored`. Run with:
//!   cargo test -p digdigdig3-station --test okx_predicted_funding_live -- --ignored --nocapture

use std::time::Duration;

use digdigdig3_station::{AccountType, ExchangeId, Station, Stream, SubscriptionSet};

#[tokio::test]
#[ignore]
async fn okx_predicted_funding_btc_usd_swap_emits_events() {
    let station = Station::builder().build().await.expect("Station::build");

    // BTC-USD-SWAP: coin-margined inverse SWAP, next_period → nextFundingRate populated.
    let set = SubscriptionSet::new().add_raw(
        ExchangeId::OKX,
        "BTC-USD-SWAP",
        AccountType::FuturesCross,
        [Stream::PredictedFunding],
    );

    let mut report = station.subscribe(set).await.expect("subscribe");
    assert!(
        report.failed.is_empty(),
        "subscribe should not fail for BTC-USD-SWAP PredictedFunding: {:?}",
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
            Ok(None) => break, // channel closed
            Err(_) => continue, // timeout, keep looping until deadline
        }
    }

    assert!(
        got >= 1,
        "expected at least 1 PredictedFunding event from BTC-USD-SWAP within 30s — got {got}"
    );
}
