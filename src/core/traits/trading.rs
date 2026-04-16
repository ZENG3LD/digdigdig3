//! # Trading — Core trading operations
//!
//! One `place_order` method handles all order types via `OrderType`.
//! Connectors match only the variants they support natively; everything
//! else returns `ExchangeError::UnsupportedOperation`.
//!
//! ## Design Rules
//! - NO default implementations
//! - NO composition (no looping over base methods)
//! - Connectors are STRICT: unsupported variant → UnsupportedOperation

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, Order, OrderHistoryFilter, OrderRequest, CancelRequest,
    PlaceOrderResponse, TradingCapabilities, UserTrade, UserTradeFilter,
};

use super::ExchangeIdentity;

/// Core trading — 24/24 exchanges.
///
/// All order types go through `place_order` via `OrderType` enum.
/// Connectors match the variants they support; unmatched variants
/// return `ExchangeError::UnsupportedOperation`.
///
/// # Strict Non-Composition Rule
/// Connectors MUST NOT simulate unsupported variants by composing base methods.
/// - A connector without native batch cancel MUST NOT loop `cancel_order`.
/// - A connector without native Bracket MUST NOT submit 3 separate orders.
/// If the exchange has no endpoint for it, return `UnsupportedOperation`.
#[async_trait]
pub trait Trading: ExchangeIdentity {
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

    /// Get the user's own trade fills (executions).
    ///
    /// Returns individual fill records for completed orders, optionally filtered
    /// by symbol, order ID, or time range.
    ///
    /// Default implementation returns `UnsupportedOperation` — connectors
    /// that expose a native fills/trades endpoint should override this.
    ///
    /// ~20/24: all major CEX exchanges. DEX connectors without native trade records return
    /// `UnsupportedOperation`.
    async fn get_user_trades(
        &self,
        filter: UserTradeFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<UserTrade>> {
        let _ = (filter, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_user_trades not implemented".into(),
        ))
    }

    /// Returns the connector's trading capabilities.
    ///
    /// The default implementation returns permissive defaults.
    /// Connectors with specific limitations should override this method.
    fn trading_capabilities(&self) -> TradingCapabilities {
        TradingCapabilities::permissive()
    }
}
