//! Paradex REST Integration Tests
//!
//! Tests REST market data against the real Paradex API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib l3::open::crypto::dex::paradex::_tests_rest -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests hit real Paradex endpoints and require network access.
//! Paradex only supports perpetual futures — AccountType::FuturesCross is used throughout.

use crate::core::types::{AccountType, ExchangeId};
use crate::testing::harness::TestHarness;
use crate::testing::suites::{market_data, TestStatus};

#[tokio::test]
#[ignore]
async fn test_market_data_suite() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Paradex, false)
        .await
        .expect("Failed to create public Paradex connector");

    let symbol = TestHarness::test_symbol(ExchangeId::Paradex);
    let account_type = AccountType::FuturesCross;

    let results = market_data::run_all(connector.as_ref(), symbol, account_type).await;

    println!("\n=== Paradex Market Data Suite ===");
    for r in &results {
        println!("  {}", r);
    }

    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::Error)
        .collect();
    assert!(
        failures.is_empty(),
        "Paradex market data tests failed: {:?}",
        failures
    );
}
