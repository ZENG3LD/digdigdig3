//! # TradingV2 — Thin trading trait (V2 architecture)
//!
//! One `place_order` method handles all order types via `OrderTypeV2`.
//! Connectors match only the variants they support natively; everything
//! else returns `ExchangeError::UnsupportedOperation`.
//!
//! ## Design Rules
//! - NO default implementations
//! - NO composition (no looping over base methods)
//! - Connectors are STRICT: unsupported variant → UnsupportedOperation

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, Order,
    OrderHistoryFilter, OrderRequest, CancelRequest, PlaceOrderResponse,
};

use super::ExchangeIdentity;

/// Core trading — 24/24 exchanges.
///
/// All order types go through `place_order` via `OrderTypeV2` enum.
/// Connectors match the variants they support; unmatched variants
/// return `ExchangeError::UnsupportedOperation`.
///
/// # Strict Non-Composition Rule
/// Connectors MUST NOT simulate unsupported variants by composing base methods.
/// - A connector without native batch cancel MUST NOT loop `cancel_order`.
/// - A connector without native Bracket MUST NOT submit 3 separate orders.
/// If the exchange has no endpoint for it, return `UnsupportedOperation`.
#[async_trait]
pub trait TradingV2: ExchangeIdentity {
    /// Place an order of any type.
    ///
    /// Connectors inspect `req.order_type` and match the variants they support.
    ///
    /// Returns `PlaceOrderResponse::Simple` for scalar orders, or the
    /// appropriate composite variant for Bracket/OCO/Algo orders.
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse>;

    /// Cancel one order, a batch, all orders, or all orders for a symbol.
    ///
    /// The scope is determined by `req.scope` (`CancelScope` enum).
    /// Connectors that only support single-cancel MUST return
    /// `UnsupportedOperation` for Batch/All/BySymbol scopes — never loop.
    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order>;

    /// Get the current state of a single order by ID.
    ///
    /// `symbol` is required by most exchanges; provide it when available.
    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Get all currently open orders, optionally filtered to a single symbol.
    ///
    /// `symbol = None` fetches open orders across all symbols.
    /// Not all exchanges support symbol-less open order queries — those that
    /// don't MUST return `UnsupportedOperation` for `None`, not an empty vec.
    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;

    /// Get closed / filled / cancelled order history with filtering.
    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;
}
