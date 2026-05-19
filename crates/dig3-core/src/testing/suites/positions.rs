//! # Positions Suite
//!
//! Tests for the `Positions` trait: get_positions, get_funding_rate,
//! get_mark_price, get_open_interest, get_long_short_ratio.
//!
//! All methods in this suite are **read-only** — no positions are opened or
//! modified. Authentication is required (the `Positions` trait is private).

use std::time::Instant;

use crate::core::traits::{ExchangeIdentity, Positions};
use crate::core::types::{AccountType, PositionQuery, Symbol};

use super::{assert_position_sane, is_auth_error, is_unsupported, TestResult};

// ═══════════════════════════════════════════════════════════════════════════════
// RUN ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Run all positions suite tests against `connector`.
///
/// `symbol` — the perpetual/futures trading pair to use.
/// `account_type` — typically `AccountType::FuturesCross` or `FuturesIsolated`.
///
/// Returns one `TestResult` per test function.
pub async fn run_all(
    connector: &(dyn PositionsConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    results.push(test_get_positions(connector, symbol.clone(), account_type).await);
    results.push(test_get_funding_rate(connector, symbol.clone(), account_type).await);
    results.push(test_get_mark_price(connector, symbol.clone()).await);
    results.push(test_get_open_interest(connector, symbol.clone(), account_type).await);
    results.push(test_get_long_short_ratio(connector, symbol.clone(), account_type).await);

    results
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER SUPERTRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Combined supertrait required by all positions tests.
pub trait PositionsConnector: Positions + ExchangeIdentity {}

impl<T: Positions + ExchangeIdentity> PositionsConnector for T {}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_positions
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch open positions for `symbol` and sanity-check each entry.
///
/// An empty result is valid — the account may have no open positions.
/// Each returned position is validated with `assert_position_sane`.
pub async fn test_get_positions(
    connector: &(dyn PositionsConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_positions";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    let query = PositionQuery {
        symbol: Some(symbol),
        account_type,
    };

    match connector.get_positions(query).await {
        Ok(positions) => {
            for pos in &positions {
                if let Err(reason) = assert_position_sane(pos) {
                    return TestResult::fail(
                        NAME, exchange,
                        start.elapsed().as_millis() as u64,
                        reason,
                    );
                }
            }
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_positions unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_positions failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_funding_rate
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch the current funding rate for `symbol` and validate it is reasonable.
///
/// Checks:
/// - `-1.0 < rate < 1.0` — sane funding rate range (0.01% = 0.0001 typical).
/// - `next_funding_time > 0` if the field is present.
pub async fn test_get_funding_rate(
    connector: &(dyn PositionsConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_funding_rate";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    match connector.get_funding_rate(&symbol.to_concat(), account_type).await {
        Ok(fr) => {
            if fr.rate.is_nan() || fr.rate.is_infinite() {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("funding rate is NaN or infinite: {}", fr.rate),
                );
            }
            if fr.rate <= -1.0 || fr.rate >= 1.0 {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("funding rate out of reasonable range: {}", fr.rate),
                );
            }
            if let Some(nft) = fr.next_funding_time {
                if nft <= 0 {
                    return TestResult::fail(
                        NAME, exchange,
                        start.elapsed().as_millis() as u64,
                        format!("next_funding_time must be positive, got {nft}"),
                    );
                }
            }
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_funding_rate unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_funding_rate failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_mark_price
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch the mark price for `symbol` and validate it is positive.
pub async fn test_get_mark_price(
    connector: &(dyn PositionsConnector + Send + Sync),
    symbol: Symbol,
) -> TestResult {
    const NAME: &str = "test_get_mark_price";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    match connector.get_mark_price(&symbol.to_concat()).await {
        Ok(mp) => {
            if mp.mark_price.is_nan()
                || mp.mark_price.is_infinite()
                || mp.mark_price <= 0.0
            {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("mark_price invalid: {}", mp.mark_price),
                );
            }
            // Index price, if present, should also be positive.
            if let Some(idx) = mp.index_price {
                if idx.is_nan() || idx.is_infinite() || idx <= 0.0 {
                    return TestResult::fail(
                        NAME, exchange,
                        start.elapsed().as_millis() as u64,
                        format!("index_price invalid: {idx}"),
                    );
                }
            }
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_mark_price unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_mark_price failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_open_interest
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch open interest for `symbol` and validate it is positive.
///
/// For popular symbols (BTC, ETH) open interest should always be > 0.
pub async fn test_get_open_interest(
    connector: &(dyn PositionsConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_open_interest";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    match connector.get_open_interest(&symbol.to_concat(), account_type).await {
        Ok(oi) => {
            if oi.open_interest.is_nan()
                || oi.open_interest.is_infinite()
                || oi.open_interest < 0.0
            {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("open_interest invalid: {}", oi.open_interest),
                );
            }
            // For major pairs open interest should be non-zero.
            let base_upper = symbol.base.to_uppercase();
            let is_major = matches!(base_upper.as_str(), "BTC" | "ETH" | "SOL" | "BNB");
            if is_major && oi.open_interest == 0.0 {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("open_interest is 0 for major symbol {symbol}"),
                );
            }
            // USD value, if present, must be non-negative.
            if let Some(v) = oi.open_interest_value {
                if v.is_nan() || v.is_infinite() || v < 0.0 {
                    return TestResult::fail(
                        NAME, exchange,
                        start.elapsed().as_millis() as u64,
                        format!("open_interest_value invalid: {v}"),
                    );
                }
            }
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_open_interest unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_open_interest failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_long_short_ratio
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch the long/short ratio for `symbol` and validate both ratios are in [0, 1].
pub async fn test_get_long_short_ratio(
    connector: &(dyn PositionsConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_long_short_ratio";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    match connector.get_long_short_ratio(&symbol.to_concat(), account_type).await {
        Ok(lsr) => {
            if lsr.long_ratio < 0.0 || lsr.long_ratio.is_nan() {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("long_ratio invalid: {}", lsr.long_ratio),
                );
            }
            if lsr.short_ratio < 0.0 || lsr.short_ratio.is_nan() {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("short_ratio invalid: {}", lsr.short_ratio),
                );
            }
            // The two ratios should approximately sum to 1.0 (within 1%).
            let sum = lsr.long_ratio + lsr.short_ratio;
            if (sum - 1.0).abs() > 0.01 {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!(
                        "long_ratio + short_ratio = {sum:.4}, expected ~1.0 \
                         (long={}, short={})",
                        lsr.long_ratio, lsr.short_ratio
                    ),
                );
            }
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_long_short_ratio unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_long_short_ratio failed: {err}"))
        }
    }
}
