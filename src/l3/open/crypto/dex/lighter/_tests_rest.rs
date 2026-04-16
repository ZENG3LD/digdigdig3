//! Lighter REST Integration Tests
//!
//! Tests REST market data against the real Lighter DEX API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib l3::open::crypto::dex::lighter::_tests_rest -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests hit real Lighter endpoints and require network access.
//! Lighter is a perpetual DEX on zkSync — all markets are USDC-margined.

use crate::core::types::{AccountType, ExchangeId};
use crate::testing::harness::TestHarness;
use crate::testing::suites::{market_data, TestStatus};

#[tokio::test]
#[ignore]
async fn test_market_data_suite() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Lighter, false)
        .await
        .expect("Failed to create public Lighter connector");

    let symbol = TestHarness::test_symbol(ExchangeId::Lighter);
    let account_type = AccountType::FuturesCross;

    let results = market_data::run_all(connector.as_ref(), symbol, account_type).await;

    println!("\n=== Lighter Market Data Suite ===");
    for r in &results {
        println!("  {}", r);
    }

    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::Error)
        .collect();
    assert!(
        failures.is_empty(),
        "Lighter market data tests failed: {:?}",
        failures
    );
}
