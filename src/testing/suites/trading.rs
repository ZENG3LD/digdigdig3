//! # Trading Suite
//!
//! Tests for the `Trading` trait: place_order, cancel_order, get_order,
//! get_open_orders, get_order_history, get_user_trades.
//!
//! All tests in this suite require authentication. The core test is a
//! place-then-cancel roundtrip that never fills (limit buy far below market).

use std::time::Instant;

use crate::core::traits::{ExchangeIdentity, MarketData, Trading};
use crate::core::types::{
    AccountType, CancelRequest, CancelScope, OrderHistoryFilter, OrderRequest,
    OrderSide, OrderType, PlaceOrderResponse, Symbol, TimeInForce, UserTradeFilter,
};

use super::{is_auth_error, is_unsupported, TestResult};

// ═══════════════════════════════════════════════════════════════════════════════
// RUN ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Run all trading suite tests against `connector`.
///
/// `symbol` — the trading pair to use (e.g. `Symbol::new("BTC", "USDT")`).
/// `account_type` — which account type to trade on.
///
/// Returns one `TestResult` per test function.
pub async fn run_all(
    connector: &(dyn TradingWithMarketData + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> Vec<TestResult> {
    let mut results = Vec::new();

    results.push(
        test_place_cancel_roundtrip(connector, symbol.clone(), account_type).await,
    );
    results.push(test_get_open_orders(connector, symbol.clone(), account_type).await);
    results.push(test_get_order_history(connector, symbol.clone(), account_type).await);
    results.push(test_get_user_trades(connector, symbol.clone(), account_type).await);

    results
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPER SUPERTRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Combined supertrait required by all trading tests.
///
/// A connector must implement `Trading + MarketData + ExchangeIdentity`
/// to run this suite. Using a combined trait object avoids the need for
/// generic parameters in the `run_all` entry point.
pub trait TradingWithMarketData: Trading + MarketData + ExchangeIdentity {}

impl<T: Trading + MarketData + ExchangeIdentity> TradingWithMarketData for T {}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: place_cancel_roundtrip
// ═══════════════════════════════════════════════════════════════════════════════

/// Place a far-below-market limit BUY, verify it exists, then cancel it.
///
/// The order is placed at 30% of the current market price to guarantee it
/// will never fill during the test.
///
/// Steps:
/// 1. Fetch current price via `MarketData::get_price`.
/// 2. Place LIMIT BUY at `price * 0.3` with quantity 0.001.
/// 3. Verify order exists via `get_order`.
/// 4. Cancel via `cancel_order`.
///
/// If any step returns `UnsupportedOperation` the test is `Skipped`.
/// If `place_order` fails the test is `Error` — no cancel is attempted.
/// If cancel fails the test is `Error` and the order ID is included in the
/// message for manual cleanup.
pub async fn test_place_cancel_roundtrip(
    connector: &(dyn TradingWithMarketData + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_place_cancel_roundtrip";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    // ── Step 1: get current price ────────────────────────────────────────────
    let price = match connector.get_price(symbol.clone(), account_type).await {
        Ok(p) => p,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_price unsupported: {err}"));
        }
        Err(err) if is_auth_error(&err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error fetching price: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("failed to get price: {err}"));
        }
    };

    let far_price = (price * 0.3 * 100.0).round() / 100.0;

    // ── Step 2: place far-below-market limit buy ─────────────────────────────
    let req = OrderRequest {
        symbol: symbol.clone(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit { price: far_price },
        quantity: 0.001,
        time_in_force: TimeInForce::Gtc,
        account_type,
        client_order_id: None,
        reduce_only: false,
    };

    let place_resp = match connector.place_order(req).await {
        Ok(r) => r,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_order unsupported: {err}"));
        }
        Err(err) if is_auth_error(&err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error placing order: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_order failed: {err}"));
        }
    };

    // Extract the order ID from the response.
    let order_id = match &place_resp {
        PlaceOrderResponse::Simple(order) => order.id.clone(),
        PlaceOrderResponse::Bracket(br) => br.entry_order.id.clone(),
        PlaceOrderResponse::Oco(oco) => oco.first_order.id.clone(),
        PlaceOrderResponse::Algo(algo) => algo.algo_id.clone(),
    };

    // ── Step 3: verify order exists via get_order ────────────────────────────
    match connector.get_order(
        &symbol.to_concat(),
        &order_id,
        account_type,
    ).await {
        Ok(_) => {} // order confirmed
        Err(err) if is_unsupported(&err) => {
            // get_order not supported — still try to cancel
        }
        Err(err) => {
            // get_order failed but order was placed; attempt cancel and report error
            let _ = cancel_single(connector, &symbol, &order_id, account_type).await;
            return TestResult::error(
                NAME, exchange,
                start.elapsed().as_millis() as u64,
                format!("get_order failed for id={order_id}: {err}"),
            );
        }
    }

    // ── Step 4: cancel the order ─────────────────────────────────────────────
    match cancel_single(connector, &symbol, &order_id, account_type).await {
        Ok(_) => {
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("cancel_order unsupported: {err}"))
        }
        Err(err) => {
            TestResult::error(
                NAME, exchange,
                start.elapsed().as_millis() as u64,
                format!(
                    "cancel failed — MANUAL CLEANUP REQUIRED order_id={order_id}: {err}"
                ),
            )
        }
    }
}

/// Send a `CancelScope::Single` request for `order_id` on `symbol`.
async fn cancel_single(
    connector: &dyn TradingWithMarketData,
    symbol: &Symbol,
    order_id: &str,
    account_type: AccountType,
) -> Result<(), crate::core::types::ExchangeError> {
    let cancel_req = CancelRequest {
        scope: CancelScope::Single {
            order_id: order_id.to_string(),
        },
        symbol: Some(symbol.clone()),
        account_type,
    };
    connector.cancel_order(cancel_req).await.map(|_| ())
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_open_orders
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch open orders for `symbol` and verify the call succeeds.
///
/// The result may be an empty list — that is valid. What matters is that
/// the connector returns `Ok` (or `UnsupportedOperation → Skip`).
pub async fn test_get_open_orders(
    connector: &(dyn TradingWithMarketData + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_open_orders";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    match connector
        .get_open_orders(Some(&symbol.to_concat()), account_type)
        .await
    {
        Ok(_orders) => TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64),
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_open_orders unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_open_orders failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_order_history
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch recent order history for `symbol` and verify the call succeeds.
///
/// Uses a minimal filter (symbol only, no time bounds, limit=10).
/// An empty result is valid.
pub async fn test_get_order_history(
    connector: &(dyn TradingWithMarketData + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_order_history";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    let filter = OrderHistoryFilter {
        symbol: Some(symbol.clone()),
        start_time: None,
        end_time: None,
        limit: Some(10),
        status: None,
    };

    match connector.get_order_history(filter, account_type).await {
        Ok(_orders) => TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64),
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_order_history unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_order_history failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: get_user_trades
// ═══════════════════════════════════════════════════════════════════════════════

/// Fetch recent user trade fills for `symbol` and verify the call succeeds.
///
/// Many DEX connectors return `UnsupportedOperation` here — that results
/// in a `Skipped` status rather than a failure.
pub async fn test_get_user_trades(
    connector: &(dyn TradingWithMarketData + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_get_user_trades";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    let filter = UserTradeFilter {
        symbol: Some(symbol.to_concat()),
        order_id: None,
        start_time: None,
        end_time: None,
        limit: Some(10),
    };

    match connector.get_user_trades(filter, account_type).await {
        Ok(_trades) => TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64),
        Err(err) if is_unsupported(&err) => {
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_user_trades unsupported: {err}"))
        }
        Err(err) if is_auth_error(&err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"))
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_user_trades failed: {err}"))
        }
    }
}
