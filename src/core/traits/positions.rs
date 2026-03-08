//! # PositionsCore - минимальное управление позициями (Futures)
//!
//! Только методы, которые есть на 100% futures бирж.
//! Расширенные методы (margin mode, add margin, etc.) - в биржевых коннекторах.

use async_trait::async_trait;

use crate::core::types::{AccountType, ExchangeResult, FundingRate, Position, Symbol};

use super::ExchangeIdentity;

/// Минимальное управление позициями
///
/// **3 метода** - есть на всех futures биржах без исключений.
///
/// # Авторизация
/// **ТРЕБУЕТСЯ** - все методы приватные
///
/// # Применимость
/// Только для `AccountType::FuturesCross` и `AccountType::FuturesIsolated`.
/// Для Spot вернет ошибку `UnsupportedOperation`.
///
/// # Расширенные методы
/// Следующие методы НЕ в этом трейте (реализуются в биржевых коннекторах):
/// - `close_position_market()` - закрыть позицию (можно через market_order)
/// - `set_leverage()` - установить leverage
/// - `get_leverage()` - получить leverage
/// - `modify_position_tpsl()` - TP/SL позиции
/// - `get_position_mode()` / `set_position_mode()` - OneWay/Hedge
/// - `get_margin_mode()` / `set_margin_mode()` - Cross/Isolated
/// - `add_margin()` - добавить маржу (Isolated)
/// - `get_funding_rate_history()` - история funding
#[async_trait]
pub trait Positions: ExchangeIdentity {
    /// Получить позиции
    async fn get_positions(
        &self,
        symbol: Option<Symbol>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>>;

    /// Получить текущий funding rate
    async fn get_funding_rate(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate>;

    /// Установить leverage
    async fn set_leverage(
        &self,
        symbol: Symbol,
        leverage: u32,
        account_type: AccountType,
    ) -> ExchangeResult<()>;
}
