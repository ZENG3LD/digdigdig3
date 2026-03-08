//! # Extension Traits - опциональные расширения
//!
//! Extension traits для функциональности, которая поддерживается не всеми биржами.
//! Каждый трейт имеет дефолтную реализацию, возвращающую `UnsupportedOperation`.
//!
//! ## Трейты в этом модуле:
//! - `BatchOperations` - batch создание/отмена ордеров
//! - `AdvancedOrders` - trailing stop, stop limit, OCO
//! - `MarginTrading` - займы и погашение (CEX only)
//! - `Transfers` - переводы между аккаунтами

use async_trait::async_trait;

use crate::core::types::{
    AccountType, Asset, CreateOrderRequest, ExchangeError, ExchangeResult,
    MarginBorrowResult, MarginLoan, MarginRepayResult, MarginType, Order, OrderSide,
    Price, Quantity, Symbol, Timestamp, TransferHistory, TransferResult,
};

use super::{Account, Trading};

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH OPERATIONS
// ═══════════════════════════════════════════════════════════════════════════════

/// Batch операции с ордерами
///
/// Позволяет создавать/отменять несколько ордеров одним запросом.
///
/// # Поддержка
/// - Bybit: ✅ Нативный batch API
/// - OKX: ✅ Нативный batch API
/// - Hyperliquid: ✅ Нативный batch API
/// - Binance: ⚠️ Дефолтная реализация через цикл
///
/// # Дефолтная реализация
/// Если биржа не поддерживает нативный batch, методы выполняют
/// последовательные запросы (менее эффективно, но работает).
#[async_trait]
pub trait BatchOperations: Trading {
    /// Создать несколько ордеров одним запросом
    ///
    /// # Аргументы
    /// * `requests` - Список запросов на создание ордеров
    ///
    /// # Возвращает
    /// Список созданных ордеров
    ///
    /// # Дефолтная реализация
    /// Последовательно создает ордера через `create_limit_order`
    async fn create_orders_batch(
        &self,
        requests: Vec<CreateOrderRequest>,
    ) -> ExchangeResult<Vec<Order>> {
        let mut results = Vec::with_capacity(requests.len());
        for req in requests {
            let order = self.create_limit_order(
                req.symbol,
                req.side,
                req.quantity,
                req.price.ok_or_else(|| {
                    ExchangeError::InvalidRequest("Batch orders require price".to_string())
                })?,
                req.account_type,
            ).await?;
            results.push(order);
        }
        Ok(results)
    }

    /// Отменить несколько ордеров одним запросом
    ///
    /// # Аргументы
    /// * `order_ids` - Список ID ордеров для отмены
    /// * `symbol` - Торговая пара
    /// * `account_type` - Тип аккаунта
    ///
    /// # Возвращает
    /// Список отмененных ордеров
    ///
    /// # Дефолтная реализация
    /// Последовательно отменяет ордера через `cancel_order`
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>> {
        let mut results = Vec::with_capacity(order_ids.len());
        for order_id in order_ids {
            let order = self.cancel_order(&order_id, symbol.clone(), account_type).await?;
            results.push(order);
        }
        Ok(results)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ADVANCED ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Расширенные типы ордеров
///
/// Trailing stop, stop limit, OCO и другие специальные типы ордеров.
///
/// # Поддержка
/// - Binance: ✅ Trailing (Futures), Stop Limit, OCO (Spot)
/// - Bybit: ✅ Trailing, Stop Limit
/// - OKX: ✅ Trailing, Stop Limit, OCO
/// - Hyperliquid: ❌ Не поддерживает
///
/// # Дефолтная реализация
/// Все методы возвращают `UnsupportedOperation`
#[async_trait]
pub trait AdvancedOrders: Trading {
    /// Создать trailing stop ордер
    ///
    /// # Аргументы
    /// * `symbol` - Торговая пара
    /// * `side` - Направление (Buy/Sell)
    /// * `quantity` - Количество
    /// * `callback_rate` - Процент отката (например, 1.0 = 1%)
    /// * `activation_price` - Цена активации (опционально)
    /// * `account_type` - Тип аккаунта
    ///
    /// # Поддержка
    /// - Binance Futures: ✅
    /// - Binance Spot: ❌
    /// - Bybit: ✅
    /// - OKX: ✅
    /// - Hyperliquid: ❌
    async fn create_trailing_stop(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _callback_rate: f64,
        _activation_price: Option<Price>,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Trailing stop not supported".to_string()
        ))
    }

    /// Создать stop limit ордер
    ///
    /// # Аргументы
    /// * `symbol` - Торговая пара
    /// * `side` - Направление (Buy/Sell)
    /// * `quantity` - Количество
    /// * `stop_price` - Цена активации
    /// * `limit_price` - Цена лимитного ордера после активации
    /// * `account_type` - Тип аккаунта
    ///
    /// # Поддержка
    /// - Binance: ✅
    /// - Bybit: ✅
    /// - OKX: ✅
    /// - Hyperliquid: ❌
    async fn create_stop_limit_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _stop_price: Price,
        _limit_price: Price,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "Stop limit not supported".to_string()
        ))
    }

    /// Создать OCO ордер (One-Cancels-Other)
    ///
    /// # Аргументы
    /// * `symbol` - Торговая пара
    /// * `side` - Направление (Buy/Sell)
    /// * `quantity` - Количество
    /// * `price` - Цена лимитного ордера
    /// * `stop_price` - Цена активации стоп ордера
    /// * `stop_limit_price` - Цена лимитного ордера после активации стопа
    /// * `account_type` - Тип аккаунта
    ///
    /// # Поддержка
    /// - Binance Spot: ✅
    /// - Binance Futures: ❌
    /// - Bybit: ❌
    /// - OKX: ✅
    /// - Hyperliquid: ❌
    async fn create_oco_order(
        &self,
        _symbol: Symbol,
        _side: OrderSide,
        _quantity: Quantity,
        _price: Price,
        _stop_price: Price,
        _stop_limit_price: Option<Price>,
        _account_type: AccountType,
    ) -> ExchangeResult<Order> {
        Err(ExchangeError::UnsupportedOperation(
            "OCO orders not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARGIN TRADING
// ═══════════════════════════════════════════════════════════════════════════════

/// Маржинальная торговля
///
/// Займы и погашение для маржинальной торговли.
///
/// # Поддержка
/// - Binance: ✅
/// - Bybit: ✅
/// - OKX: ✅
/// - Hyperliquid: ❌ (DEX, нет маржинальной торговли)
///
/// # Дефолтная реализация
/// Все методы возвращают `UnsupportedOperation`
#[async_trait]
pub trait MarginTrading: Account {
    /// Взять займ
    ///
    /// # Аргументы
    /// * `asset` - Актив для займа
    /// * `amount` - Сумма займа
    /// * `symbol` - Торговая пара (для isolated margin)
    ///
    /// # Поддержка
    /// CEX only (Binance, Bybit, OKX)
    async fn borrow_margin(
        &self,
        _asset: Asset,
        _amount: Quantity,
        _symbol: Option<Symbol>,
    ) -> ExchangeResult<MarginBorrowResult> {
        Err(ExchangeError::UnsupportedOperation(
            "Margin trading not supported".to_string()
        ))
    }

    /// Погасить займ
    ///
    /// # Аргументы
    /// * `asset` - Актив для погашения
    /// * `amount` - Сумма погашения
    /// * `symbol` - Торговая пара (для isolated margin)
    ///
    /// # Поддержка
    /// CEX only (Binance, Bybit, OKX)
    async fn repay_margin(
        &self,
        _asset: Asset,
        _amount: Quantity,
        _symbol: Option<Symbol>,
    ) -> ExchangeResult<MarginRepayResult> {
        Err(ExchangeError::UnsupportedOperation(
            "Margin trading not supported".to_string()
        ))
    }

    /// Получить информацию о займах
    ///
    /// # Аргументы
    /// * `asset` - Актив (None = все активы)
    ///
    /// # Поддержка
    /// CEX only (Binance, Bybit, OKX)
    async fn get_margin_info(
        &self,
        _asset: Option<Asset>,
    ) -> ExchangeResult<Vec<MarginLoan>> {
        Err(ExchangeError::UnsupportedOperation(
            "Margin trading not supported".to_string()
        ))
    }

    /// Установить тип маржи (Cross/Isolated)
    ///
    /// # Аргументы
    /// * `symbol` - Торговая пара
    /// * `margin_type` - Тип маржи
    ///
    /// # Поддержка
    /// CEX only (Binance, Bybit, OKX)
    async fn set_margin_type(
        &self,
        _symbol: Symbol,
        _margin_type: MarginType,
    ) -> ExchangeResult<()> {
        Err(ExchangeError::UnsupportedOperation(
            "Margin type switching not supported".to_string()
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Переводы между аккаунтами
///
/// Переводы средств между типами аккаунтов (Spot -> Futures, etc.)
///
/// # Поддержка
/// - Binance: ✅
/// - Bybit: ✅
/// - OKX: ✅
/// - Hyperliquid: ✅ (transfer), ❌ (history)
#[async_trait]
pub trait Transfers: Account {
    /// Перевести средства между аккаунтами
    ///
    /// # Аргументы
    /// * `asset` - Актив для перевода
    /// * `amount` - Сумма перевода
    /// * `from_account` - Исходный тип аккаунта
    /// * `to_account` - Целевой тип аккаунта
    ///
    /// # Примеры переводов
    /// - Spot -> FuturesCross
    /// - FuturesCross -> Spot
    /// - Margin -> Spot
    async fn transfer(
        &self,
        asset: Asset,
        amount: Quantity,
        from_account: AccountType,
        to_account: AccountType,
    ) -> ExchangeResult<TransferResult>;

    /// Получить историю переводов
    ///
    /// # Аргументы
    /// * `start_time` - Начало периода (опционально)
    /// * `end_time` - Конец периода (опционально)
    /// * `limit` - Максимальное количество записей
    ///
    /// # Поддержка
    /// - Binance: ✅
    /// - Bybit: ✅
    /// - OKX: ✅
    /// - Hyperliquid: ❌
    async fn get_transfer_history(
        &self,
        _start_time: Option<Timestamp>,
        _end_time: Option<Timestamp>,
        _limit: Option<u16>,
    ) -> ExchangeResult<Vec<TransferHistory>> {
        Err(ExchangeError::UnsupportedOperation(
            "Transfer history not supported".to_string()
        ))
    }
}
