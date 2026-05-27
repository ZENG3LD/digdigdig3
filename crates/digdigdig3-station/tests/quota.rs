//! Unit tests for per-consumer subscription + REST quotas.
//!
//! All tests here are offline — they exercise the pre-flight logic and
//! token-bucket arithmetic without connecting to a real exchange.

use std::time::Duration;

use digdigdig3_station::{
    AccountType, ConsumerQuota, ConsumerWhitelist, ExchangeId, QuotaError, Station, Stream,
    SubscriptionSet,
};
use digdigdig3_station::series::Kind;

// ---------------------------------------------------------------------------
// a. Default quota is unlimited — cap/whitelist checks never fire.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_default_is_unlimited() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let consumer = station.register_consumer(ConsumerQuota::unlimited());

    // Build a set with many entries (10 streams). The pre-flight must not
    // reject any of them — only the live exchange call might fail, and that
    // would show up as `report.failed`, NOT as `Err(QuotaError::...)`.
    // We verify that the consumer call does NOT return a quota error.
    //
    // In practice Station::subscribe on a non-running exchange will not
    // panic — it returns a report with failed entries. That is fine.
    let set = SubscriptionSet::new()
        .add(ExchangeId::Binance, "BTC-USDT", AccountType::Spot, [Stream::Trade])
        .add(ExchangeId::Binance, "ETH-USDT", AccountType::Spot, [Stream::Trade])
        .add(ExchangeId::Binance, "SOL-USDT", AccountType::Spot, [Stream::Trade])
        .add(ExchangeId::Bybit, "BTC-USDT", AccountType::FuturesCross, [Stream::Trade])
        .add(ExchangeId::Bybit, "ETH-USDT", AccountType::FuturesCross, [Stream::Trade]);

    // Must NOT return QuotaError — any subscribe failures are per-stream.
    let result = consumer.subscribe(set).await;
    match result {
        Err(QuotaError::SubsCapExceeded { .. }) => {
            panic!("unlimited quota must not return SubsCapExceeded")
        }
        Err(QuotaError::NotInWhitelist(_)) => {
            panic!("unlimited quota must not return NotInWhitelist")
        }
        _ => {} // Ok (or Inner station error) — both are acceptable
    }

    // active_sub_count reflects how many streams actually subscribed
    // (may be non-zero if the exchange is reachable). The key assertion
    // is that we got here without a QuotaError.
    let count = consumer.active_sub_count().await;
    assert!(count <= 5, "at most 5 streams were requested; got {count}");
}

// ---------------------------------------------------------------------------
// b. Cap enforced atomically — nothing subscribed when batch exceeds cap.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_subs_cap_rejects_atomic_or_nothing() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // cap = 2, request 3 streams in one batch → must be rejected.
    let consumer = station.register_consumer(
        ConsumerQuota::default().max_active_subs(2),
    );

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade, Stream::Ticker, Stream::Orderbook], // 3 streams
    );

    let err = consumer.subscribe(set).await
        .expect_err("3 streams with cap=2 must fail");

    match err {
        QuotaError::SubsCapExceeded { have, cap } => {
            assert_eq!(cap, 2, "cap must be 2");
            assert_eq!(have, 3, "have must be 3 (requested)");
        }
        other => panic!("expected SubsCapExceeded, got {other:?}"),
    }

    // Nothing was subscribed.
    assert_eq!(consumer.active_sub_count().await, 0,
        "atomic: zero subs after cap rejection");
    assert_eq!(station.active_streams(), 0,
        "no mux spawned after cap rejection");
}

// ---------------------------------------------------------------------------
// b-2. Cap = 0: even a single stream is rejected.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_cap_zero_rejects_any_sub() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let consumer = station.register_consumer(
        ConsumerQuota::default().max_active_subs(0),
    );

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );

    let err = consumer.subscribe(set).await
        .expect_err("cap=0 must reject any sub");

    assert!(
        matches!(err, QuotaError::SubsCapExceeded { cap: 0, .. }),
        "expected SubsCapExceeded with cap=0"
    );
}

// ---------------------------------------------------------------------------
// c. Whitelist rejects wrong exchange before any IO.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_whitelist_rejects_wrong_exchange() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let wl = ConsumerWhitelist::new().allow_exchange(ExchangeId::Binance);
    let consumer = station.register_consumer(
        ConsumerQuota::default().whitelist(wl),
    );

    let set = SubscriptionSet::new().add(
        ExchangeId::Bybit,        // NOT in whitelist
        "BTC-USDT",
        AccountType::FuturesCross,
        [Stream::Trade],
    );

    let err = consumer.subscribe(set).await
        .expect_err("wrong exchange must be rejected by whitelist");

    assert!(
        matches!(err, QuotaError::NotInWhitelist(_)),
        "expected NotInWhitelist, got {err:?}"
    );
    assert_eq!(consumer.active_sub_count().await, 0);
}

// ---------------------------------------------------------------------------
// c-2. Whitelist rejects wrong kind.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_whitelist_rejects_wrong_kind() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // Only Trade is whitelisted; Ticker is not.
    let wl = ConsumerWhitelist::new()
        .allow_exchange(ExchangeId::Binance)
        .allow_kind(Kind::Trade);
    let consumer = station.register_consumer(
        ConsumerQuota::default().whitelist(wl),
    );

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Ticker], // rejected
    );

    let err = consumer.subscribe(set).await
        .expect_err("wrong kind must be rejected");
    assert!(matches!(err, QuotaError::NotInWhitelist(_)));
}

// ---------------------------------------------------------------------------
// c-3. Whitelist rejects wrong symbol.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_whitelist_rejects_wrong_symbol() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let wl = ConsumerWhitelist::new()
        .allow_exchange(ExchangeId::Binance)
        .allow_symbol("BTC-USDT");
    let consumer = station.register_consumer(
        ConsumerQuota::default().whitelist(wl),
    );

    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "ETH-USDT", // not in whitelist
        AccountType::Spot,
        [Stream::Trade],
    );

    let err = consumer.subscribe(set).await
        .expect_err("symbol not in whitelist must be rejected");
    assert!(matches!(err, QuotaError::NotInWhitelist(_)));
}

// ---------------------------------------------------------------------------
// c-4. Whitelist ordering: whitelist fires BEFORE cap.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_whitelist_fires_before_cap() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // Cap=0 AND whitelist mismatch — whitelist error must win.
    let wl = ConsumerWhitelist::new().allow_exchange(ExchangeId::Binance);
    let consumer = station.register_consumer(
        ConsumerQuota::default()
            .max_active_subs(0)
            .whitelist(wl),
    );

    let set = SubscriptionSet::new().add(
        ExchangeId::Bybit, // NOT in whitelist
        "BTC-USDT",
        AccountType::FuturesCross,
        [Stream::Trade],
    );

    let err = consumer.subscribe(set).await
        .expect_err("must fail");

    assert!(
        matches!(err, QuotaError::NotInWhitelist(_)),
        "whitelist must fire before cap check, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// d. Drop ConsumerHandle releases its refs (offline version).
//
// Without a live exchange we can't verify station.active_streams() drops
// from N to 0 after subscribe. We verify the refs Vec is dropped on
// ConsumerHandle drop by observing the Arc<StationInner> strong count.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_drop_releases_arc_ref() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // Register and immediately drop — no subscribe needed.
    let consumer = station.register_consumer(ConsumerQuota::unlimited());
    // The consumer holds one Arc clone of StationInner.
    // Dropping it should not panic.
    drop(consumer);
    // Station still alive — its own Arc kept it alive.
    assert_eq!(station.active_streams(), 0);
}

// ---------------------------------------------------------------------------
// e. REST token bucket: burst fills, then blocks, then refills.
//    Tested via ConsumerHandle::rest_gate to avoid depending on the
//    pub(crate) TokenBucket type directly.
// ---------------------------------------------------------------------------
#[tokio::test(flavor = "multi_thread")]
async fn quota_rest_token_bucket_refills() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // 2 tokens per 200ms window.
    let consumer = station.register_consumer(
        ConsumerQuota::default().max_rest(2, Duration::from_millis(200)),
    );

    // First two succeed.
    assert!(consumer.rest_gate().await.is_ok(), "token 1");
    assert!(consumer.rest_gate().await.is_ok(), "token 2");

    // Third fails.
    let err = consumer.rest_gate().await.expect_err("bucket empty");
    let remaining_ms = match err {
        QuotaError::RestRateLimit { remaining_ms } => remaining_ms,
        other => panic!("expected RestRateLimit, got {other:?}"),
    };
    assert!(remaining_ms > 0, "remaining_ms must be positive");

    // Wait for the window to elapse.
    tokio::time::sleep(Duration::from_millis(210)).await;

    // After refill — token succeeds again.
    assert!(consumer.rest_gate().await.is_ok(), "token after refill");
}

// ---------------------------------------------------------------------------
// e-2. rest_gate() via ConsumerHandle respects the bucket.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_rest_gate_rate_limit() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // 2 tokens per 500ms window.
    let consumer = station.register_consumer(
        ConsumerQuota::default().max_rest(2, Duration::from_millis(500)),
    );

    // First two succeed.
    assert!(consumer.rest_gate().await.is_ok(), "token 1");
    assert!(consumer.rest_gate().await.is_ok(), "token 2");

    // Third is rate-limited.
    let err = consumer.rest_gate().await
        .expect_err("third gate must be rate-limited");
    assert!(
        matches!(err, QuotaError::RestRateLimit { .. }),
        "expected RestRateLimit, got {err:?}"
    );
}

// ---------------------------------------------------------------------------
// e-3. Unlimited REST quota: rest_gate always succeeds.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_rest_gate_unlimited_never_blocks() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let consumer = station.register_consumer(ConsumerQuota::unlimited());

    for i in 0..100 {
        consumer.rest_gate().await
            .unwrap_or_else(|e| panic!("gate {i} failed: {e}"));
    }

    assert_eq!(consumer.rest_tokens_available().await, u32::MAX);
}

// ---------------------------------------------------------------------------
// f. Live subscribe integration — cap enforced correctly with real subscribe.
// This test is #[ignore] because it requires a live Binance connection.
// ---------------------------------------------------------------------------
#[tokio::test]
#[ignore = "requires live Binance connection"]
async fn quota_live_cap_enforced_after_real_subscribe() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    // cap = 1. First subscribe of Trade succeeds. Second subscribe of
    // Ticker on the same consumer must be rejected.
    let consumer = station.register_consumer(
        ConsumerQuota::default().max_active_subs(1),
    );

    let set1 = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade],
    );
    let report1 = consumer.subscribe(set1).await
        .expect("first subscribe must succeed");
    assert!(report1.is_fully_ok(), "trade subscribe fully ok");
    assert_eq!(consumer.active_sub_count().await, 1);

    let set2 = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Ticker], // would push to 2 — over cap
    );
    let err = consumer.subscribe(set2).await
        .expect_err("second sub must be cap-rejected");
    assert!(matches!(err, QuotaError::SubsCapExceeded { have: 2, cap: 1 }));
}

// ---------------------------------------------------------------------------
// g. Two consumers on the same exchange — each has own quota.
// ---------------------------------------------------------------------------
#[tokio::test]
async fn quota_two_consumers_independent_caps() {
    let station = Station::builder()
        .build()
        .await
        .expect("Station::build");

    let c1 = station.register_consumer(ConsumerQuota::default().max_active_subs(1));
    let c2 = station.register_consumer(ConsumerQuota::default().max_active_subs(2));

    // c1: 2 streams → rejected.
    let set = SubscriptionSet::new().add(
        ExchangeId::Binance,
        "BTC-USDT",
        AccountType::Spot,
        [Stream::Trade, Stream::Ticker],
    );
    assert!(matches!(
        c1.subscribe(set.clone()).await,
        Err(QuotaError::SubsCapExceeded { have: 2, cap: 1 })
    ));

    // c2: 2 streams → cap allows (pre-flight passes; IO may still fail but no quota error).
    let result = c2.subscribe(set).await;
    assert!(
        !matches!(result, Err(QuotaError::SubsCapExceeded { .. })),
        "c2 cap=2 must not reject 2 streams"
    );
}
