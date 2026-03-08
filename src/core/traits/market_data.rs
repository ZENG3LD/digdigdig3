//! # MarketDataCore - минимальные публичные рыночные данные
//!
//! Только методы, которые есть на 100% бирж.
//! Расширенные методы (get_tickers, get_recent_trades, etc.) - в биржевых коннекторах.

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, Kline, OrderBook, Price, Symbol, SymbolInfo, Ticker,
};

use super::ExchangeIdentity;

/// Минимальные публичные рыночные данные
///
/// **6 методов** - есть на всех биржах без исключений.
///
/// # Авторизация
/// **НЕ ТРЕБУЕТСЯ** - все методы публичные
///
/// # Расширенные методы
/// Следующие методы НЕ в этом трейте (реализуются в биржевых коннекторах):
/// - `get_tickers()` - все тикеры
/// - `get_recent_trades()` - последние сделки
/// - `get_open_interest()` - OI (futures)
/// - `get_mark_price()` - mark price (futures)
#[async_trait]
pub trait MarketData: ExchangeIdentity {
    /// Получить текущую цену символа
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price>;

    /// Получить книгу ордеров
    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook>;

    /// Получить свечи (klines)
    ///
    /// `end_time` — optional Unix timestamp in **milliseconds** (ms).
    /// When provided, the exchange should return bars whose open time is
    /// *before or at* this timestamp (i.e. walk backwards in time).
    /// Connectors that do not support this parameter should accept it with
    /// a leading underscore (`_end_time`) and ignore it.
    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>>;

    /// Получить 24h тикер
    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker>;

    /// Проверить соединение (ping)
    async fn ping(&self) -> ExchangeResult<()>;

    /// Получить информацию о всех торговых символах биржи
    ///
    /// Возвращает список всех доступных символов для данного типа аккаунта.
    /// Коннекторы, не поддерживающие этот метод, возвращают `UnsupportedOperation`.
    async fn get_exchange_info(&self, account_type: AccountType) -> ExchangeResult<Vec<SymbolInfo>> {
        let _ = account_type;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_exchange_info not implemented for this connector".to_string(),
        ))
    }
}
