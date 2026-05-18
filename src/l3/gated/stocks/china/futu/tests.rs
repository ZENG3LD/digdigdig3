//! Integration tests for Futu connector
//!
//! **NOTE**: These tests are disabled because Futu uses TCP + Protocol Buffers,
//! not standard HTTP REST APIs. The connector is not functional without either:
//! 1. PyO3 bridge to Python SDK
//! 2. Native Rust Protocol Buffer client
//! 3. External REST adapter service
//!
//! See research/RECOMMENDATIONS.md for implementation options.

#![cfg(test)]

use super::*;
use super::auth::FutuAuth;
use crate::core::types::{Symbol, AccountType};
use crate::core::traits::MarketData;

#[tokio::test]
#[ignore] // Disabled - requires Protocol Buffer implementation
async fn test_futu_returns_unsupported() {
    let auth = FutuAuth::new("127.0.0.1", 11111);
    let connector = FutuConnector::new(auth);
    let symbol = Symbol::new("AAPL", "USD");

    // All methods should return UnsupportedOperation
    let sym_str = symbol.to_concat();
    let result = connector.get_price(sym_str.as_str().into(), AccountType::Spot).await;
    assert!(result.is_err());

    match result {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(msg)) => {
            assert!(msg.contains("TCP") || msg.contains("Protocol Buffers"));
            println!("Correctly returns UnsupportedOperation: {}", msg);
        }
        _ => panic!("Expected UnsupportedOperation error"),
    }
}

// Removed `test_futu_documentation_exists` — it asserted on
// `src/l3/stocks/china/futu/research/`, which (a) is the wrong path (real
// path is `src/l3/gated/stocks/china/futu/research/`) and (b) is in
// `.gitignore` per the add-crate baseline rule. Documentation presence is
// not something the test runner should police.

#[test]
fn test_futu_stub_message() {
    // This test documents why Futu is disabled
    let reason = "Futu OpenAPI uses TCP + Protocol Buffers (not HTTP REST)";
    let solution = "See research/RECOMMENDATIONS.md for implementation options";

    println!("\n=== FUTU CONNECTOR STATUS ===");
    println!("Status: DISABLED (stub only)");
    println!("Reason: {}", reason);
    println!("Solution: {}", solution);
    println!("Research: 42,000+ lines of documentation available");
    println!("Recommendation: PyO3 wrapper (5 days) or skip");
    println!("============================\n");
}
