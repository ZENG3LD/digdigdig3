//! # AccountCore - минимальная информация об аккаунте
//!
//! Только методы, которые есть на 100% бирж.

use async_trait::async_trait;

use crate::core::types::{AccountInfo, AccountType, Asset, Balance, ExchangeResult};

use super::ExchangeIdentity;

/// Минимальная информация об аккаунте
///
/// **2 метода** - есть на всех биржах без исключений.
///
/// # Авторизация
/// **ТРЕБУЕТСЯ** - все методы приватные
///
/// # Расширенные методы
/// Следующие методы НЕ в этом трейте (реализуются в биржевых коннекторах):
/// - `get_my_trades()` - история fills
/// - `get_deposit_address()` - адрес депозита
/// - `get_deposit_history()` - история депозитов
/// - `get_withdrawal_history()` - история выводов
#[async_trait]
pub trait Account: ExchangeIdentity {
    /// Получить баланс
    async fn get_balance(
        &self,
        asset: Option<Asset>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>>;

    /// Получить информацию об аккаунте
    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo>;
}
