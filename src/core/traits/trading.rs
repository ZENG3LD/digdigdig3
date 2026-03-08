//! # TradingCore - минимальные торговые операции
//!
//! Только методы, которые есть на 100% бирж.
//! Расширенные методы (modify_order, cancel_all, order_history) - в биржевых коннекторах.

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, Order, OrderSide, Price, Quantity, Symbol,
};

use super::ExchangeIdentity;

/// Минимальные торговые операции
///
/// **5 методов** - есть на всех биржах без исключений.
///
/// # Авторизация
/// **ТРЕБУЕТСЯ** - все методы приватные
///
/// # Расширенные методы
/// Следующие методы НЕ в этом трейте (реализуются в биржевых коннекторах):
/// - `modify_order()` - Binance Spot не поддерживает
/// - `cancel_all_orders()` - есть везде, но детали разные
/// - `get_order_history()` - есть везде, но параметры разные
/// - `get_trades()` / `get_my_trades()` - fills
/// - `create_stop_loss()` / `create_take_profit()` - conditional orders
/// - `create_order_with_tpsl()` - комбинированные ордера
#[async_trait]
pub trait Trading: ExchangeIdentity {
    /// Создать market ордер
    async fn market_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Создать limit ордер
    async fn limit_order(
        &self,
        symbol: Symbol,
        side: OrderSide,
        quantity: Quantity,
        price: Price,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Отменить ордер
    async fn cancel_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Получить информацию об ордере
    async fn get_order(
        &self,
        symbol: Symbol,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Получить открытые ордера
    async fn get_open_orders(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;
}
