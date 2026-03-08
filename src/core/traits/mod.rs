//! # V5 Core Traits - Минимальный набор
//!
//! ## Архитектура V6
//!
//! ```text
//! CoreConnector<C>           - минимальные универсальные методы (15 методов)
//!      │
//!      └── KuCoinConnector   - core + все KuCoin-специфичное напрямую
//!      └── BinanceConnector  - core + все Binance-специфичное напрямую
//! ```
//!
//! ## Принципы
//!
//! 1. **Core трейты минимальны** - только то, что есть на 100% бирж
//! 2. **Нет UnsupportedOperation** - все core методы работают везде
//! 3. **Расширения в биржевых коннекторах** - напрямую как методы структуры
//!
//! ## Core трейты
//!
//! | Трейт | Методов | Описание |
//! |-------|---------|----------|
//! | `ExchangeIdentity` | 5 | Базовая идентификация |
//! | `MarketData` | 5 | Публичные данные (price, orderbook, klines, ticker, ping) |
//! | `Trading` | 5 | Торговля (market, limit, cancel, get_order, open_orders) |
//! | `Account` | 2 | Аккаунт (balance, account_info) |
//! | `Positions` | 3 | Futures (positions, funding_rate, set_leverage) |
//!
//! **Итого: ~20 методов** (вместо ~50+ в старой версии)
//!
//! ## Расширенные методы (в биржевых коннекторах)
//!
//! - `get_tickers()`, `get_recent_trades()`, `get_exchange_info()`, `get_open_interest()`
//! - `modify_order()`, `cancel_all_orders()`, `get_order_history()`, `get_trades()`
//! - `create_stop_loss()`, `create_take_profit()`, `create_order_with_tpsl()`
//! - `get_my_trades()`, `get_deposit_address()`, transfers
//! - `close_position_market()`, margin mode, position mode, add margin
//! - WebSocket subscriptions
//! - Биржево-специфичные (sub-accounts, internal transfers, etc.)

mod identity;
mod market_data;
mod trading;
mod account;
mod positions;
mod websocket;
mod auth;

pub use identity::ExchangeIdentity;
pub use market_data::MarketData;
pub use trading::Trading;
pub use account::Account;
pub use positions::Positions;
pub use websocket::{WebSocketConnector, WebSocketExt};
pub use auth::{Credentials, AuthRequest, SignatureLocation, ExchangeAuth};

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITE TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Полный core коннектор
///
/// Комбинирует все core трейты.
/// Используется для generic кода, работающего с любой биржей.
///
/// # Пример
/// ```ignore
/// async fn check_balance<C: CoreConnector>(conn: &C) -> Result<()> {
///     let balance = conn.get_balance(None, AccountType::Spot).await?;
///     let price = conn.get_price(Symbol::new("BTC", "USDT"), AccountType::Spot).await?;
///     Ok(())
/// }
/// ```
pub trait CoreConnector:
    ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync
{
}

impl<T> CoreConnector for T where
    T: ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync
{
}
