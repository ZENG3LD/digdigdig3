//! # ExchangeIdentity - базовый трейт идентификации биржи
//!
//! Этот трейт является корнем иерархии и должен быть реализован ВСЕМИ коннекторами.
//! НЕ требует авторизации.
//!
//! ## Реализация
//! - Binance: ✅
//! - Bybit: ✅
//! - OKX: ✅
//! - Hyperliquid: ✅

use crate::core::types::{
    AccountType, ConnectorStats, ExchangeId, ExchangeType, RateLimitCapabilities,
};

/// Базовая идентификация биржи
///
/// Этот трейт определяет минимальный набор методов для идентификации биржи.
/// Все коннекторы ДОЛЖНЫ реализовывать этот трейт.
///
/// # Примечания
/// - НЕ требует авторизации
/// - Все методы синхронные (не async)
/// - Должен быть Send + Sync для многопоточного использования
pub trait ExchangeIdentity: Send + Sync {
    /// Уникальный идентификатор биржи
    ///
    /// # Возвращает
    /// `ExchangeId` enum значение (Binance, Bybit, OKX, Hyperliquid, etc.)
    fn exchange_id(&self) -> ExchangeId;

    /// Человекочитаемое имя биржи
    ///
    /// # Дефолтная реализация
    /// Делегирует в `exchange_id().as_str()`
    fn exchange_name(&self) -> &'static str {
        self.exchange_id().as_str()
    }

    /// Работаем ли с тестовой сетью
    ///
    /// # Возвращает
    /// - `true` - тестнет/демо режим
    /// - `false` - продакшн
    fn is_testnet(&self) -> bool;

    /// Список поддерживаемых типов аккаунтов
    ///
    /// # Примеры
    /// - Binance: [Spot, Margin, FuturesCross, FuturesIsolated]
    /// - Bybit: [Spot, FuturesCross, FuturesIsolated]
    /// - OKX: [Spot, Margin, FuturesCross, FuturesIsolated]
    /// - Hyperliquid: [Spot, FuturesCross]
    fn supported_account_types(&self) -> Vec<AccountType>;

    /// Тип биржи (централизованная, децентрализованная, гибрид)
    ///
    /// # Дефолтная реализация
    /// Делегирует в `exchange_id().exchange_type()`
    fn exchange_type(&self) -> ExchangeType {
        self.exchange_id().exchange_type()
    }

    /// Runtime metrics snapshot for this connector.
    ///
    /// Returns HTTP request/error counters, last latency, and rate-limiter
    /// utilization. The default implementation returns zeroed metrics.
    /// Override this in connectors that have an `HttpClient` to expose live data.
    fn metrics(&self) -> ConnectorStats {
        ConnectorStats::default()
    }

    /// Static rate limit capabilities for this exchange.
    ///
    /// Returns the compile-time descriptor used to build runtime limiters.
    /// Default is `permissive()` (unlimited) — override in each connector.
    fn rate_limit_capabilities(&self) -> RateLimitCapabilities {
        RateLimitCapabilities::permissive()
    }
}
