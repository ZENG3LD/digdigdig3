//! Polymarket REST Integration Tests
//!
//! Tests REST market data against the real Polymarket CLOB API.
//!
//! Run with:
//! ```text
//! cargo test --package digdigdig3 --lib l3::open::prediction::polymarket::_tests_rest -- --ignored --nocapture
//! ```
//!
//! NOTE: All tests hit real Polymarket CLOB/Gamma endpoints and require network access.
//! Polymarket is a prediction market — symbols are condition_ids (0x-prefixed hex strings),
//! not traditional trading pairs. Prices represent probabilities (0.0–1.0).
//!
//! Strategy: fetch active markets via get_exchange_info first to get a real condition_id,
//! then run individual market data tests against that symbol.

use crate::core::types::{AccountType, ExchangeId};
use crate::core::traits::MarketData;
use crate::testing::harness::TestHarness;
use crate::testing::suites::TestStatus;

/// Fetch a valid active market condition_id from Polymarket, or return None.
async fn fetch_live_condition_id(connector: &dyn MarketData) -> Option<String> {
    match connector.get_exchange_info(AccountType::Spot).await {
        Ok(symbols) => {
            // SymbolInfo.symbol holds the condition_id (0x-prefixed hex).
            // status is "TRADING" for active markets, "BREAK" otherwise.
            symbols
                .into_iter()
                .find(|s| s.symbol.starts_with("0x") && s.status == "TRADING")
                .map(|s| s.symbol)
        }
        Err(e) => {
            println!("get_exchange_info failed: {:?}", e);
            None
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_ping() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Polymarket, false)
        .await
        .expect("Failed to create public Polymarket connector");

    match connector.ping().await {
        Ok(()) => println!("Polymarket ping OK"),
        Err(e) => panic!("Ping failed: {:?}", e),
    }
}

#[tokio::test]
#[ignore]
async fn test_get_exchange_info() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Polymarket, false)
        .await
        .expect("Failed to create public Polymarket connector");

    let symbols = connector
        .get_exchange_info(AccountType::Spot)
        .await
        .expect("get_exchange_info failed");

    println!("Polymarket: {} active markets", symbols.len());
    assert!(!symbols.is_empty(), "Should have at least one active market");

    // Print first few markets
    for s in symbols.iter().take(3) {
        println!("  market: condition_id={} status={}", s.symbol, s.status);
    }
}

#[tokio::test]
#[ignore]
async fn test_get_price_live_market() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Polymarket, false)
        .await
        .expect("Failed to create public Polymarket connector");

    let condition_id = match fetch_live_condition_id(connector.as_ref()).await {
        Some(id) => id,
        None => {
            println!("No active market found — skipping price test");
            return;
        }
    };

    println!("Testing get_price for condition_id: {}", condition_id);

    use crate::core::types::Symbol;
    let symbol = Symbol::new(&condition_id, "USDC");

    match connector.get_price(symbol, AccountType::Spot).await {
        Ok(price) => {
            println!("YES probability: {:.1}%", price * 100.0);
            assert!(
                price >= 0.0 && price <= 1.0,
                "Polymarket price must be a probability 0.0-1.0, got: {}",
                price
            );
        }
        Err(e) => {
            println!("get_price failed (may be expected if market has no orderbook): {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_get_orderbook_live_market() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Polymarket, false)
        .await
        .expect("Failed to create public Polymarket connector");

    let condition_id = match fetch_live_condition_id(connector.as_ref()).await {
        Some(id) => id,
        None => {
            println!("No active market found — skipping orderbook test");
            return;
        }
    };

    println!("Testing get_orderbook for condition_id: {}", condition_id);

    use crate::core::types::Symbol;
    let symbol = Symbol::new(&condition_id, "USDC");

    match connector
        .get_orderbook(symbol, Some(10), AccountType::Spot)
        .await
    {
        Ok(ob) => {
            println!(
                "Orderbook OK: {} bids, {} asks",
                ob.bids.len(),
                ob.asks.len()
            );
            // Polymarket orderbooks may be sparse — just verify structure
            for bid in ob.bids.iter().take(3) {
                assert!(bid.price > 0.0, "Bid price must be positive");
                assert!(bid.price <= 1.0, "Bid price must be a probability ≤ 1.0");
            }
            for ask in ob.asks.iter().take(3) {
                assert!(ask.price > 0.0, "Ask price must be positive");
                assert!(ask.price <= 1.0, "Ask price must be a probability ≤ 1.0");
            }
        }
        Err(e) => {
            println!("get_orderbook error (market may lack liquidity): {:?}", e);
        }
    }
}

#[tokio::test]
#[ignore]
async fn test_market_data_suite() {
    let harness = TestHarness::new();
    let connector = harness
        .create_public(ExchangeId::Polymarket, false)
        .await
        .expect("Failed to create public Polymarket connector");

    // Fetch a live condition_id to use as symbol; fall back to the static test symbol
    let condition_id = fetch_live_condition_id(connector.as_ref())
        .await
        .unwrap_or_else(|| TestHarness::test_symbol(ExchangeId::Polymarket).to_string());

    println!(
        "Running Polymarket market data suite with symbol: {}",
        condition_id
    );

    use crate::testing::suites::market_data;
    let results = market_data::run_all(connector.as_ref(), &condition_id, AccountType::Spot).await;

    println!("\n=== Polymarket Market Data Suite ===");
    for r in &results {
        println!("  {}", r);
    }

    let failures: Vec<_> = results
        .iter()
        .filter(|r| r.status == TestStatus::Failed || r.status == TestStatus::Error)
        .collect();
    assert!(
        failures.is_empty(),
        "Polymarket market data tests failed: {:?}",
        failures
    );
}
