//! # V5 Core Traits
//!
//! ## Architecture
//!
//! ```text
//! CoreConnector<C>           - universal methods (Identity + MarketData + Trading + Account + Positions)
//!      │
//!      └── BinanceConnector  - core + all Binance-specific directly
//!      └── KuCoinConnector   - core + all KuCoin-specific directly
//! ```
//!
//! ## Principles
//!
//! 1. **Core traits are minimal** — only what 100% of exchanges support
//! 2. **No UnsupportedOperation in core** — all core methods work everywhere
//! 3. **Extensions in exchange connectors** — directly as struct methods
//!
//! ## Core traits
//!
//! | Trait | Methods | Description |
//! |-------|---------|-------------|
//! | `ExchangeIdentity` | 5 | Basic identification |
//! | `MarketData` | 5 | Public data (price, orderbook, klines, ticker, ping) |
//! | `Trading` | 5 | Trading (place_order, cancel_order, get_order, get_open_orders, get_order_history) |
//! | `Account` | 3 | Account (balance, account_info, fees) |
//! | `Positions` | 3 | Futures (positions, funding_rate, modify_position) |
//!
//! ## Optional operation traits (not universally implemented)
//!
//! - `CancelAll` - native cancel-all endpoint
//! - `AmendOrder` - native amend/modify order
//! - `BatchOrders` - native batch placement/cancellation
//! - `AccountTransfers` - internal account transfers
//! - `CustodialFunds` - deposits and withdrawals
//! - `SubAccounts` - sub-account management
//! - `Authenticated` - credential-aware connectors

mod identity;
mod market_data;
mod trading;
mod account;
mod positions;
mod websocket;
mod auth;
mod operations;
pub mod event_stream;

pub use identity::ExchangeIdentity;
pub use market_data::MarketData;
pub use trading::Trading;
pub use account::Account;
pub use positions::Positions;
pub use websocket::{WebSocketConnector, WebSocketExt};
pub use auth::{
    Authenticated, CredentialKind,
    // Backward compat — used by connector constructors and auth implementations
    Credentials, AuthRequest, SignatureLocation, ExchangeAuth,
};
pub use operations::{
    CancelAll, AmendOrder, BatchOrders,
    AccountTransfers, CustodialFunds, SubAccounts,
    MarginTrading, EarnStaking, ConvertSwap, CopyTrading,
    LiquidityProvider, VaultManager, StakingDelegation, BlockTradeOtc,
    MarketMakerProtection, TriggerOrders, PredictionMarket,
    FundingHistory, AccountLedger,
};
pub use event_stream::{EventProducer, EventFilter};

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITE TRAIT
// ═══════════════════════════════════════════════════════════════════════════════

/// Full core connector
///
/// Combines all core traits.
/// Used for generic code that works with any exchange.
///
/// # Example
/// ```ignore
/// async fn check_balance<C: CoreConnector>(conn: &C) -> Result<()> {
///     let balance = conn.get_balance(BalanceQuery { asset: None, account_type: AccountType::Spot }).await?;
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
