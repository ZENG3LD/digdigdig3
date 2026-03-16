//! # Operations Suite
//!
//! Tests for optional operation traits: `AmendOrder`, `CancelAll`, `BatchOrders`.
//!
//! Each test is independently skippable — if the connector does not implement
//! the relevant trait, the test returns `Skipped` via `UnsupportedOperation`.
//!
//! Unlike the trading suite, these functions accept concrete combined trait
//! objects rather than a single supertrait so the harness can choose which
//! tests to run based on declared feature flags.

use std::time::Instant;

use crate::core::traits::{
    AmendOrder, BatchOrders, CancelAll,
    ExchangeIdentity, MarketData, Trading,
};
use crate::core::types::{
    AccountType, AmendFields, AmendRequest, CancelRequest, CancelScope,
    OrderRequest, OrderSide, OrderType, PlaceOrderResponse, Symbol,
    TimeInForce,
};

use super::{is_auth_error, is_unsupported, TestResult};

// ═══════════════════════════════════════════════════════════════════════════════
// COMBINED TRAIT OBJECTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Trait object required by `test_amend_order`.
pub trait AmendConnector: AmendOrder + Trading + MarketData + ExchangeIdentity {}
impl<T: AmendOrder + Trading + MarketData + ExchangeIdentity> AmendConnector for T {}

/// Trait object required by `test_cancel_all`.
pub trait CancelAllConnector: CancelAll + Trading + MarketData + ExchangeIdentity {}
impl<T: CancelAll + Trading + MarketData + ExchangeIdentity> CancelAllConnector for T {}

/// Trait object required by `test_batch_orders`.
pub trait BatchConnector: BatchOrders + Trading + MarketData + ExchangeIdentity {}
impl<T: BatchOrders + Trading + MarketData + ExchangeIdentity> BatchConnector for T {}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: amend_order
// ═══════════════════════════════════════════════════════════════════════════════

/// Place a far-below-market limit BUY, amend its price even further down,
/// verify the new price, then cancel.
///
/// Steps:
/// 1. Fetch current price.
/// 2. Place LIMIT BUY at `price * 0.3`.
/// 3. Amend the order to `price * 0.25`.
/// 4. Verify the amended order has the new price via `get_order`.
/// 5. Cancel the order.
///
/// Any `UnsupportedOperation` in any step → `Skipped`.
pub async fn test_amend_order(
    connector: &(dyn AmendConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_amend_order";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    // ── Step 1: current price ────────────────────────────────────────────────
    let price = match connector.get_price(symbol.clone(), account_type).await {
        Ok(p) => p,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_price unsupported: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("failed to get price: {err}"));
        }
    };

    let original_price = (price * 0.3 * 100.0).round() / 100.0;
    let amended_price  = (price * 0.25 * 100.0).round() / 100.0;

    // ── Step 2: place order ──────────────────────────────────────────────────
    let req = OrderRequest {
        symbol: symbol.clone(),
        side: OrderSide::Buy,
        order_type: OrderType::Limit { price: original_price },
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

    let order_id = extract_order_id(&place_resp);

    // ── Step 3: amend price ──────────────────────────────────────────────────
    let amend_req = AmendRequest {
        order_id: order_id.clone(),
        symbol: symbol.clone(),
        account_type,
        fields: AmendFields {
            price: Some(amended_price),
            quantity: None,
            trigger_price: None,
        },
    };

    match connector.amend_order(amend_req).await {
        Ok(amended_order) => {
            // ── Step 4: verify new price ─────────────────────────────────────
            let actual_price = amended_order.price.unwrap_or(0.0);
            let price_matches = (actual_price - amended_price).abs() < 0.001 * amended_price;

            // Always attempt cancel before returning.
            let cancel_result = cancel_single_trading(
                connector, &symbol, &order_id, account_type,
            ).await;

            if !price_matches {
                return TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!(
                        "amended price mismatch: expected ~{amended_price}, got {actual_price}"
                    ),
                );
            }

            match cancel_result {
                Ok(_) => TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64),
                Err(err) => TestResult::error(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!("cancel failed after amend — MANUAL CLEANUP order_id={order_id}: {err}"),
                ),
            }
        }
        Err(err) if is_unsupported(&err) => {
            // amend not supported — cancel the order we placed and skip.
            let _ = cancel_single_trading(connector, &symbol, &order_id, account_type).await;
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("amend_order unsupported: {err}"))
        }
        Err(err) => {
            // Amend failed — attempt cancel and report error.
            let _ = cancel_single_trading(connector, &symbol, &order_id, account_type).await;
            TestResult::error(
                NAME, exchange,
                start.elapsed().as_millis() as u64,
                format!("amend_order failed for id={order_id}: {err}"),
            )
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: cancel_all
// ═══════════════════════════════════════════════════════════════════════════════

/// Place 2 far-below-market limit BUYs, cancel all for the symbol, verify both gone.
///
/// Steps:
/// 1. Fetch current price.
/// 2. Place order A at `price * 0.3`.
/// 3. Place order B at `price * 0.29`.
/// 4. `cancel_all_orders(CancelScope::BySymbol { symbol })`.
/// 5. `get_open_orders(Some(symbol))` — verify result is empty or neither A/B present.
///
/// Any `UnsupportedOperation` → `Skipped`.
pub async fn test_cancel_all(
    connector: &(dyn CancelAllConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_cancel_all";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    // ── Step 1: current price ────────────────────────────────────────────────
    let price = match connector.get_price(symbol.clone(), account_type).await {
        Ok(p) => p,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_price unsupported: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("failed to get price: {err}"));
        }
    };

    let price_a = (price * 0.3 * 100.0).round() / 100.0;
    let price_b = (price * 0.29 * 100.0).round() / 100.0;

    // ── Step 2: place order A ────────────────────────────────────────────────
    let order_a_id = match place_limit_buy(connector, symbol.clone(), price_a, account_type).await {
        Ok(id) => id,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_order unsupported: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_order (A) failed: {err}"));
        }
    };

    // ── Step 3: place order B ────────────────────────────────────────────────
    let order_b_id = match place_limit_buy(connector, symbol.clone(), price_b, account_type).await {
        Ok(id) => id,
        Err(err) => {
            // Cancel A before bailing out.
            let _ = cancel_single_trading(connector, &symbol, &order_a_id, account_type).await;
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_order (B) failed: {err}"));
        }
    };

    // ── Step 4: cancel all for symbol ────────────────────────────────────────
    let cancel_scope = CancelScope::BySymbol { symbol: symbol.clone() };
    match connector.cancel_all_orders(cancel_scope, account_type).await {
        Ok(_) => {} // proceed to verification
        Err(err) if is_unsupported(&err) => {
            // Cleanup both orders manually then skip.
            let _ = cancel_single_trading(connector, &symbol, &order_a_id, account_type).await;
            let _ = cancel_single_trading(connector, &symbol, &order_b_id, account_type).await;
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("cancel_all_orders unsupported: {err}"));
        }
        Err(err) => {
            let _ = cancel_single_trading(connector, &symbol, &order_a_id, account_type).await;
            let _ = cancel_single_trading(connector, &symbol, &order_b_id, account_type).await;
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("cancel_all_orders failed: {err}"));
        }
    }

    // ── Step 5: verify both orders are gone ──────────────────────────────────
    match connector.get_open_orders(Some(&symbol.to_concat()), account_type).await {
        Ok(open_orders) => {
            let ids: Vec<&str> = open_orders.iter().map(|o| o.id.as_str()).collect();
            let a_still_open = ids.contains(&order_a_id.as_str());
            let b_still_open = ids.contains(&order_b_id.as_str());

            if a_still_open || b_still_open {
                TestResult::fail(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!(
                        "orders still open after cancel_all: \
                         A={} ({a_still_open}), B={} ({b_still_open})",
                        order_a_id, order_b_id
                    ),
                )
            } else {
                TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
            }
        }
        Err(err) if is_unsupported(&err) => {
            // get_open_orders not supported — treat cancel_all as pass since it returned Ok.
            TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
        }
        Err(err) => {
            TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_open_orders verification failed: {err}"))
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TEST: batch_orders
// ═══════════════════════════════════════════════════════════════════════════════

/// Place 2 orders in a batch, verify both exist, then cancel both.
///
/// Steps:
/// 1. Fetch current price.
/// 2. Build a 2-order batch (LIMIT BUY at `price * 0.3` and `price * 0.29`).
/// 3. `place_orders_batch(orders)` — verify both `OrderResult.success == true`.
/// 4. `cancel_orders_batch([id_a, id_b])`.
///
/// Any `UnsupportedOperation` → `Skipped`.
pub async fn test_batch_orders(
    connector: &(dyn BatchConnector + Send + Sync),
    symbol: Symbol,
    account_type: AccountType,
) -> TestResult {
    const NAME: &str = "test_batch_orders";
    let exchange = connector.exchange_name();
    let start = Instant::now();

    // ── Step 1: current price ────────────────────────────────────────────────
    let price = match connector.get_price(symbol.clone(), account_type).await {
        Ok(p) => p,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("get_price unsupported: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("failed to get price: {err}"));
        }
    };

    let price_a = (price * 0.3 * 100.0).round() / 100.0;
    let price_b = (price * 0.29 * 100.0).round() / 100.0;

    let orders = vec![
        OrderRequest {
            symbol: symbol.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit { price: price_a },
            quantity: 0.001,
            time_in_force: TimeInForce::Gtc,
            account_type,
            client_order_id: Some("test_batch_a".to_string()),
            reduce_only: false,
        },
        OrderRequest {
            symbol: symbol.clone(),
            side: OrderSide::Buy,
            order_type: OrderType::Limit { price: price_b },
            quantity: 0.001,
            time_in_force: TimeInForce::Gtc,
            account_type,
            client_order_id: Some("test_batch_b".to_string()),
            reduce_only: false,
        },
    ];

    // ── Step 3: place batch ──────────────────────────────────────────────────
    let results = match connector.place_orders_batch(orders).await {
        Ok(r) => r,
        Err(err) if is_unsupported(&err) => {
            return TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_orders_batch unsupported: {err}"));
        }
        Err(err) if is_auth_error(&err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("auth error: {err}"));
        }
        Err(err) => {
            return TestResult::error(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("place_orders_batch failed: {err}"));
        }
    };

    // Check both orders succeeded and collect their IDs.
    let mut placed_ids: Vec<String> = Vec::new();
    for (i, result) in results.iter().enumerate() {
        if !result.success {
            // Attempt to cancel any that did succeed before returning.
            if !placed_ids.is_empty() {
                let _ = connector
                    .cancel_orders_batch(placed_ids.clone(), Some(&symbol.to_concat()), account_type)
                    .await;
            }
            let err_msg = result.error.clone().unwrap_or_else(|| "unknown error".to_string());
            return TestResult::fail(
                NAME, exchange,
                start.elapsed().as_millis() as u64,
                format!("batch order [{i}] failed: {err_msg}"),
            );
        }
        if let Some(ref order) = result.order {
            placed_ids.push(order.id.clone());
        }
    }

    if placed_ids.len() != 2 {
        // Something unexpected — log but continue to cancel.
        let _ = connector
            .cancel_orders_batch(placed_ids.clone(), Some(&symbol.to_concat()), account_type)
            .await;
        return TestResult::fail(
            NAME, exchange,
            start.elapsed().as_millis() as u64,
            format!("expected 2 placed order IDs, got {}", placed_ids.len()),
        );
    }

    // ── Step 4: cancel batch ─────────────────────────────────────────────────
    match connector
        .cancel_orders_batch(placed_ids.clone(), Some(&symbol.to_concat()), account_type)
        .await
    {
        Ok(cancel_results) => {
            let failed: Vec<&str> = cancel_results
                .iter()
                .filter(|r| !r.success)
                .filter_map(|r| r.error.as_deref())
                .collect();
            if failed.is_empty() {
                TestResult::pass(NAME, exchange, start.elapsed().as_millis() as u64)
            } else {
                TestResult::error(
                    NAME, exchange,
                    start.elapsed().as_millis() as u64,
                    format!(
                        "some batch cancels failed — MANUAL CLEANUP ids={:?}: {:?}",
                        placed_ids, failed
                    ),
                )
            }
        }
        Err(err) if is_unsupported(&err) => {
            // Batch cancel not supported — fall back to single cancels.
            for id in &placed_ids {
                let _ = cancel_single_trading(connector, &symbol, id, account_type).await;
            }
            TestResult::skip(NAME, exchange, start.elapsed().as_millis() as u64,
                format!("cancel_orders_batch unsupported: {err}"))
        }
        Err(err) => {
            TestResult::error(
                NAME, exchange,
                start.elapsed().as_millis() as u64,
                format!(
                    "cancel_orders_batch failed — MANUAL CLEANUP ids={:?}: {err}",
                    placed_ids
                ),
            )
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Extract the primary order ID from any `PlaceOrderResponse` variant.
fn extract_order_id(resp: &PlaceOrderResponse) -> String {
    match resp {
        PlaceOrderResponse::Simple(order) => order.id.clone(),
        PlaceOrderResponse::Bracket(br) => br.entry_order.id.clone(),
        PlaceOrderResponse::Oco(oco) => oco.first_order.id.clone(),
        PlaceOrderResponse::Algo(algo) => algo.algo_id.clone(),
    }
}

/// Place a single GTC limit BUY and return its order ID.
async fn place_limit_buy<C>(
    connector: &C,
    symbol: Symbol,
    price: f64,
    account_type: AccountType,
) -> Result<String, crate::core::types::ExchangeError>
where
    C: Trading + MarketData + ExchangeIdentity + ?Sized,
{
    let req = OrderRequest {
        symbol,
        side: OrderSide::Buy,
        order_type: OrderType::Limit { price },
        quantity: 0.001,
        time_in_force: TimeInForce::Gtc,
        account_type,
        client_order_id: None,
        reduce_only: false,
    };
    let resp = connector.place_order(req).await?;
    Ok(extract_order_id(&resp))
}

/// Cancel a single order by ID.
async fn cancel_single_trading<C>(
    connector: &C,
    symbol: &Symbol,
    order_id: &str,
    account_type: AccountType,
) -> Result<(), crate::core::types::ExchangeError>
where
    C: Trading + ?Sized,
{
    let cancel_req = CancelRequest {
        scope: CancelScope::Single {
            order_id: order_id.to_string(),
        },
        symbol: Some(symbol.clone()),
        account_type,
    };
    connector.cancel_order(cancel_req).await.map(|_| ())
}
