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

// V2 traits — coexist with V1 traits
pub mod trading_v2;
pub mod account_v2;
pub mod positions_v2;
pub mod operations_v2;
pub mod auth_v2;

pub use identity::ExchangeIdentity;
pub use market_data::MarketData;
pub use trading::Trading;
pub use account::Account;
pub use positions::Positions;
pub use websocket::{WebSocketConnector, WebSocketExt};
pub use auth::{Credentials, AuthRequest, SignatureLocation, ExchangeAuth};

// V2 trait re-exports
pub use trading_v2::TradingV2;
pub use account_v2::AccountV2;
pub use positions_v2::PositionsV2;
pub use operations_v2::{
    CancelAllV2, AmendOrderV2, BatchOrdersV2,
    AccountTransfersV2, CustodialFundsV2, SubAccountsV2,
};
pub use auth_v2::{Authenticated, CredentialKind};

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

// ═══════════════════════════════════════════════════════════════════════════════
// V2 COMPOSITE TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Full V2 core connector — combines all V2 thin traits.
///
/// Used for generic code that works with any exchange using the V2 architecture.
/// Optional operation traits (`CancelAllV2`, `AmendOrderV2`, etc.) are NOT
/// included here because they are not universally implemented.
///
/// # Example
/// ```ignore
/// async fn check_portfolio<C: CoreConnectorV2>(conn: &C) {
///     let balance = conn.get_balance(BalanceQuery { asset: None, account_type: AccountType::Spot }).await?;
///     let orders = conn.get_open_orders(None, AccountType::Spot).await?;
/// }
/// ```
pub trait CoreConnectorV2:
    ExchangeIdentity + MarketData + TradingV2 + AccountV2 + Send + Sync
{
}

impl<T> CoreConnectorV2 for T where
    T: ExchangeIdentity + MarketData + TradingV2 + AccountV2 + Send + Sync
{
}
