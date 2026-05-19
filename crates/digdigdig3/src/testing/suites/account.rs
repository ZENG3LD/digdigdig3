//! # Account Suite
//!
//! Tests for the `Account` trait: get_balance, get_account_info, get_fees.

use crate::core::traits::Account;
use crate::core::types::{AccountType, BalanceQuery};
use super::{TestResult, assert_balance_sane, is_unsupported};

// ═══════════════════════════════════════════════════════════════════════════════
// ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════════════

/// Run all account tests for the given connector.
///
/// `symbol` is used for the `get_fees` call, e.g. `"BTC/USDT"`.
///
/// Returns one `TestResult` per test function. Tests that hit
/// `UnsupportedOperation` are returned as `Skipped`, not `Error`.
pub async fn run_all(
    connector: &dyn Account,
    symbol: &str,
    account_type: AccountType,
) -> Vec<TestResult> {
    let name = connector.exchange_name().to_string();
    let mut results = vec![];
    results.push(test_get_balance(connector, &name, account_type).await);
    results.push(test_get_account_info(connector, &name, account_type).await);
    results.push(test_get_fees(connector, &name, symbol).await);
    results
}

// ═══════════════════════════════════════════════════════════════════════════════
// INDIVIDUAL TESTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate `get_balance`: all returned entries must have non-negative values.
///
/// An empty balance list is acceptable (fresh/unfunded account).
async fn test_get_balance(
    connector: &dyn Account,
    exchange: &str,
    account_type: AccountType,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_balance";
    let duration = || start.elapsed().as_millis() as u64;

    let query = BalanceQuery {
        account_type,
        asset: None,
    };

    match connector.get_balance(query).await {
        Ok(balances) => {
            for (i, balance) in balances.iter().enumerate() {
                if let Err(reason) = assert_balance_sane(balance) {
                    return TestResult::fail(
                        test_name,
                        exchange,
                        duration(),
                        format!("balance[{i}]: {reason}"),
                    );
                }
            }
            TestResult::pass(test_name, exchange, duration())
        }
        Err(e) if is_unsupported(&e) => {
            TestResult::skip(test_name, exchange, duration(), e.to_string())
        }
        Err(e) => TestResult::error(test_name, exchange, duration(), e.to_string()),
    }
}

/// Validate `get_account_info`: returned account_type must match what was requested.
async fn test_get_account_info(
    connector: &dyn Account,
    exchange: &str,
    account_type: AccountType,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_account_info";
    let duration = || start.elapsed().as_millis() as u64;

    match connector.get_account_info(account_type).await {
        Ok(info) => {
            if info.account_type != account_type {
                return TestResult::fail(
                    test_name,
                    exchange,
                    duration(),
                    format!(
                        "account_info: returned account_type {:?} does not match requested {:?}",
                        info.account_type, account_type
                    ),
                );
            }
            TestResult::pass(test_name, exchange, duration())
        }
        Err(e) if is_unsupported(&e) => {
            TestResult::skip(test_name, exchange, duration(), e.to_string())
        }
        Err(e) => TestResult::error(test_name, exchange, duration(), e.to_string()),
    }
}

/// Validate `get_fees`: maker_rate and taker_rate must be in [0, 1]; taker >= maker.
async fn test_get_fees(
    connector: &dyn Account,
    exchange: &str,
    symbol: &str,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_fees";
    let duration = || start.elapsed().as_millis() as u64;

    match connector.get_fees(Some(symbol)).await {
        Ok(fees) => {
            if fees.maker_rate < 0.0 || fees.maker_rate > 1.0 {
                return TestResult::fail(
                    test_name,
                    exchange,
                    duration(),
                    format!(
                        "fees: maker_rate {} is out of range [0, 1]",
                        fees.maker_rate
                    ),
                );
            }
            if fees.taker_rate < 0.0 || fees.taker_rate > 1.0 {
                return TestResult::fail(
                    test_name,
                    exchange,
                    duration(),
                    format!(
                        "fees: taker_rate {} is out of range [0, 1]",
                        fees.taker_rate
                    ),
                );
            }
            if fees.taker_rate < fees.maker_rate {
                return TestResult::fail(
                    test_name,
                    exchange,
                    duration(),
                    format!(
                        "fees: taker_rate ({}) < maker_rate ({}) — unusual fee structure",
                        fees.taker_rate, fees.maker_rate
                    ),
                );
            }
            TestResult::pass(test_name, exchange, duration())
        }
        Err(e) if is_unsupported(&e) => {
            TestResult::skip(test_name, exchange, duration(), e.to_string())
        }
        Err(e) => TestResult::error(test_name, exchange, duration(), e.to_string()),
    }
}
