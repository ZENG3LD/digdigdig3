//! # Market Data Suite
//!
//! Tests for the `MarketData` trait: ping, price, ticker, orderbook, klines.

use crate::core::traits::MarketData;
use crate::core::types::{AccountType, Symbol};
use super::{TestResult, assert_kline_sane, assert_orderbook_sane, assert_price_sane, assert_ticker_sane, is_unsupported};

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Parse a `&str` symbol into a `Symbol`.
///
/// Tries `/`, `-`, `_` separators. Falls back to storing the whole string as
/// `raw` so connectors that use `symbol.raw()` still work.
fn parse_symbol(s: &str) -> Symbol {
    if let Some(sym) = Symbol::parse(s) {
        return sym;
    }
    // Unknown format — store raw string; connectors use symbol.raw() to get it.
    Symbol::with_raw("", "", s.to_string())
}

// ═══════════════════════════════════════════════════════════════════════════════
// ENTRY POINT
// ═══════════════════════════════════════════════════════════════════════════════

/// Run all market data tests for the given connector and symbol.
///
/// `symbol` is a human-readable string such as `"BTC/USDT"` or `"BTC-USD"`.
/// It will be parsed into a `Symbol` before each call.
///
/// Returns one `TestResult` per test function. Tests that hit
/// `UnsupportedOperation` are returned as `Skipped`, not `Error`.
pub async fn run_all(
    connector: &dyn MarketData,
    symbol: &str,
    account_type: AccountType,
) -> Vec<TestResult> {
    let name = connector.exchange_name().to_string();
    let mut results = vec![];
    results.push(test_ping(connector, &name).await);
    results.push(test_get_price(connector, &name, symbol, account_type).await);
    results.push(test_get_ticker(connector, &name, symbol, account_type).await);
    results.push(test_get_orderbook(connector, &name, symbol, account_type).await);
    results.push(test_get_klines(connector, &name, symbol, account_type).await);
    results
}

// ═══════════════════════════════════════════════════════════════════════════════
// INDIVIDUAL TESTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that the exchange is reachable by calling `ping()`.
async fn test_ping(
    connector: &dyn MarketData,
    exchange: &str,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_ping";
    let duration = || start.elapsed().as_millis() as u64;

    match connector.ping().await {
        Ok(()) => TestResult::pass(test_name, exchange, duration()),
        Err(e) if is_unsupported(&e) => {
            TestResult::skip(test_name, exchange, duration(), e.to_string())
        }
        Err(e) => TestResult::error(test_name, exchange, duration(), e.to_string()),
    }
}

/// Validate `get_price`: must return a positive finite value.
async fn test_get_price(
    connector: &dyn MarketData,
    exchange: &str,
    symbol: &str,
    account_type: AccountType,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_price";
    let duration = || start.elapsed().as_millis() as u64;

    match connector.get_price(parse_symbol(symbol), account_type).await {
        Ok(price) => match assert_price_sane(price, "get_price") {
            Ok(()) => TestResult::pass(test_name, exchange, duration()),
            Err(reason) => TestResult::fail(test_name, exchange, duration(), reason),
        },
        Err(e) if is_unsupported(&e) => {
            TestResult::skip(test_name, exchange, duration(), e.to_string())
        }
        Err(e) => TestResult::error(test_name, exchange, duration(), e.to_string()),
    }
}

/// Validate `get_ticker`: check last_price, bid/ask sanity, and non-negative volume.
async fn test_get_ticker(
    connector: &dyn MarketData,
    exchange: &str,
    symbol: &str,
    account_type: AccountType,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_ticker";
    let duration = || start.elapsed().as_millis() as u64;

    match connector.get_ticker(parse_symbol(symbol), account_type).await {
        Ok(ticker) => {
            // Assert via shared helper which covers: symbol non-empty, last_price > 0,
            // volume_24h >= 0, bid > 0 if present, ask > 0 if present.
            if let Err(reason) = assert_ticker_sane(&ticker) {
                return TestResult::fail(test_name, exchange, duration(), reason);
            }

            // Additional cross-field check: bid < ask when both are present.
            if let (Some(bid), Some(ask)) = (ticker.bid_price, ticker.ask_price) {
                if bid >= ask {
                    return TestResult::fail(
                        test_name,
                        exchange,
                        duration(),
                        format!("ticker: bid ({bid}) >= ask ({ask}) — crossed or equal"),
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

/// Validate `get_orderbook`: sorted sides, positive prices/quantities, positive spread.
async fn test_get_orderbook(
    connector: &dyn MarketData,
    exchange: &str,
    symbol: &str,
    account_type: AccountType,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_orderbook";
    let duration = || start.elapsed().as_millis() as u64;

    match connector
        .get_orderbook(parse_symbol(symbol), Some(10), account_type)
        .await
    {
        Ok(ob) => {
            // Must have at least one side populated.
            if ob.bids.is_empty() && ob.asks.is_empty() {
                return TestResult::fail(
                    test_name,
                    exchange,
                    duration(),
                    "orderbook: both bids and asks are empty".to_string(),
                );
            }

            match assert_orderbook_sane(&ob) {
                Ok(()) => TestResult::pass(test_name, exchange, duration()),
                Err(reason) => TestResult::fail(test_name, exchange, duration(), reason),
            }
        }
        Err(e) if is_unsupported(&e) => {
            TestResult::skip(test_name, exchange, duration(), e.to_string())
        }
        Err(e) => TestResult::error(test_name, exchange, duration(), e.to_string()),
    }
}

/// Validate `get_klines`: at least one candle returned, OHLCV sanity, timestamps in order.
async fn test_get_klines(
    connector: &dyn MarketData,
    exchange: &str,
    symbol: &str,
    account_type: AccountType,
) -> TestResult {
    let start = std::time::Instant::now();
    let test_name = "test_get_klines";
    let duration = || start.elapsed().as_millis() as u64;

    match connector
        .get_klines(parse_symbol(symbol), "1h", Some(10), account_type, None)
        .await
    {
        Ok(klines) => {
            if klines.is_empty() {
                return TestResult::fail(
                    test_name,
                    exchange,
                    duration(),
                    "get_klines returned empty vec".to_string(),
                );
            }

            // Per-kline OHLCV sanity checks.
            for (i, kline) in klines.iter().enumerate() {
                if let Err(reason) = assert_kline_sane(kline) {
                    return TestResult::fail(
                        test_name,
                        exchange,
                        duration(),
                        format!("kline[{i}]: {reason}"),
                    );
                }
            }

            // Timestamps must be monotone (either all ascending or all descending).
            if klines.len() >= 2 {
                let ascending = klines[1].open_time >= klines[0].open_time;
                for i in 1..klines.len() {
                    let in_order = if ascending {
                        klines[i].open_time >= klines[i - 1].open_time
                    } else {
                        klines[i].open_time <= klines[i - 1].open_time
                    };
                    if !in_order {
                        return TestResult::fail(
                            test_name,
                            exchange,
                            duration(),
                            format!(
                                "klines: timestamps not monotone at index {i}: {} after {}",
                                klines[i].open_time,
                                klines[i - 1].open_time
                            ),
                        );
                    }
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
