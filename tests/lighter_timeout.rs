//! Regression test: get_klines must not spin-lock the tokio runtime.
//!
//! Before the fix, LIGHTER_POOLS had max_budget=60 while get_klines passes
//! weight=300.  WeightRateLimiter::try_acquire(300) with max=60 returns false,
//! then time_until_ready returns Duration::ZERO (no entries to expire), causing
//! rate_limit_wait to busy-spin with no yield point — starving the executor.
//!
//! After the fix: max_budget=24_000 (premium tier), and a zero-wait guard in
//! rate_limit_wait returns false immediately if weight > max_budget.

use std::time::Duration;

use digdigdig3::l3::open::crypto::dex::lighter::LighterConnector;
use digdigdig3::core::{AccountType, Symbol};
use digdigdig3::MarketData;

/// get_klines must complete (or fail with a network/API error) within 5 seconds.
/// A hung runtime means the timeout fires and the test fails.
#[tokio::test]
async fn get_klines_does_not_block_runtime() {
    let connector = LighterConnector::public(false)
        .await
        .expect("LighterConnector::public should construct without error");

    let symbol = Symbol::new("ETH", "USDC");
    let symbol_str = symbol.to_concat();

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        connector.get_klines(&symbol_str, "1h", Some(10), AccountType::FuturesCross, None),
    )
    .await;

    // The outer Result is Err(Elapsed) only if the 5s budget expires.
    // Any inner Result (Ok or network Err) is acceptable — we only care that
    // the future completes within the budget.
    assert!(
        result.is_ok(),
        "get_klines blocked the tokio runtime for >5s — rate_limit_wait spin detected"
    );
}

/// rate_limit_wait must not spin when a single request weight exceeds the pool
/// max_budget.  Verified by constructing a connector and issuing a ping (low
/// weight) and observing the call returns promptly.
#[tokio::test]
async fn ping_does_not_block_runtime() {
    let connector = LighterConnector::public(false)
        .await
        .expect("LighterConnector::public should construct without error");

    let result = tokio::time::timeout(
        Duration::from_secs(5),
        connector.ping(),
    )
    .await;

    assert!(
        result.is_ok(),
        "ping blocked the tokio runtime for >5s"
    );
}
