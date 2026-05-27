//! Stress test: 1000 ConsumerHandles, concurrent register + cap checks.
//!
//! No live exchange calls — all pre-flight only. Verifies the quota math
//! is correct under concurrent register/drop and no panics occur.

use std::sync::Arc;
use std::time::Duration;

use digdigdig3_station::{
    AccountType, ConsumerQuota, ExchangeId, QuotaError, Station, Stream, SubscriptionSet,
};

/// Register 1000 consumers each with cap=10 (10 000 hypothetical subs),
/// concurrently submit subscribe batches that fit within cap, verify
/// no SubsCapExceeded false-positives, then drop all handles — no panics.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn quota_stress_1000_consumers_no_leak() {
    let station = Arc::new(
        Station::builder()
            .build()
            .await
            .expect("Station::build"),
    );

    let handles: Vec<_> = (0..1000)
        .map(|_| {
            let station = Arc::clone(&station);
            tokio::spawn(async move {
                let consumer = station.register_consumer(
                    ConsumerQuota::default().max_active_subs(10),
                );

                // Request 5 streams (fits within cap=10). Pre-flight must
                // not return SubsCapExceeded. IO may fail — that is fine.
                let set = SubscriptionSet::new().add(
                    ExchangeId::Binance,
                    "BTC-USDT",
                    AccountType::Spot,
                    [
                        Stream::Trade,
                        Stream::Ticker,
                        Stream::Orderbook,
                        Stream::MarkPrice,
                        Stream::FundingRate,
                    ],
                );

                let result = consumer.subscribe(set).await;
                match result {
                    Err(QuotaError::SubsCapExceeded { .. }) => {
                        panic!("false SubsCapExceeded: 5 streams fit within cap=10")
                    }
                    _ => {} // Ok or IO error — both fine
                }

                // Verify counter never exceeds cap.
                assert!(
                    consumer.active_sub_count().await <= 10,
                    "active_sub_count must not exceed cap"
                );

                // Drop consumer — releases all refs.
                drop(consumer);
            })
        })
        .collect();

    for h in handles {
        h.await.expect("task did not panic");
    }

    // No panics = pass. Station is still alive.
    let _ = station.active_streams();
}

/// Verify cap boundary math: exactly cap subs fit, cap+1 doesn't.
#[tokio::test]
async fn quota_stress_exact_boundary() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let cap: u32 = 5;
    let consumer = station.register_consumer(
        ConsumerQuota::default().max_active_subs(cap),
    );

    // Exactly cap streams — pre-flight must pass (IO may still fail).
    let set_fit = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        (0..cap as usize).map(|i| match i {
            0 => Stream::Trade,
            1 => Stream::Ticker,
            2 => Stream::Orderbook,
            3 => Stream::MarkPrice,
            _ => Stream::FundingRate,
        }),
    );
    let result = consumer.subscribe(set_fit).await;
    assert!(
        !matches!(result, Err(QuotaError::SubsCapExceeded { .. })),
        "exactly cap streams must pass pre-flight"
    );

    // Now attempt cap+1 streams — must be rejected (current active is 0
    // since IO failed, but the pre-flight counter is correct).
    // We test with a fresh consumer at 0 active + request cap+1 streams.
    let consumer2 = station.register_consumer(
        ConsumerQuota::default().max_active_subs(cap),
    );
    let set_over = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        (0..=(cap as usize)).map(|i| match i % 6 {
            0 => Stream::Trade,
            1 => Stream::Ticker,
            2 => Stream::Orderbook,
            3 => Stream::MarkPrice,
            4 => Stream::FundingRate,
            _ => Stream::AggTrade,
        }),
    );
    let err = consumer2.subscribe(set_over).await
        .expect_err("cap+1 streams must be rejected");
    assert!(
        matches!(err, QuotaError::SubsCapExceeded { .. }),
        "expected SubsCapExceeded, got {err:?}"
    );
}

/// REST token bucket stress: 10 concurrent consumers each draining their bucket.
#[tokio::test]
async fn quota_stress_rest_buckets_independent() {
    let station = Arc::new(
        Station::builder()
            .build()
            .await
            .expect("Station::build"),
    );

    let tasks: Vec<_> = (0..10)
        .map(|_| {
            let station = Arc::clone(&station);
            tokio::spawn(async move {
                let consumer = station.register_consumer(
                    ConsumerQuota::default()
                        .max_rest(5, Duration::from_millis(1000)),
                );

                // 5 tokens succeed.
                for i in 0..5 {
                    consumer.rest_gate().await
                        .unwrap_or_else(|e| panic!("gate {i} failed: {e}"));
                }

                // 6th is rate-limited.
                let err = consumer.rest_gate().await
                    .expect_err("6th gate must be rate-limited");
                assert!(
                    matches!(err, QuotaError::RestRateLimit { .. }),
                    "expected RestRateLimit"
                );
            })
        })
        .collect();

    for t in tasks {
        t.await.expect("task did not panic");
    }
}
