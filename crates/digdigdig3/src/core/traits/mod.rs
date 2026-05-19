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
//! ## Optional operation traits (part of CoreConnector)
//!
//! - `CancelAll` - native cancel-all endpoint
//! - `AmendOrder` - native amend/modify order
//! - `BatchOrders` - native batch placement/cancellation
//! - `AccountTransfers` - internal account transfers
//! - `CustodialFunds` - deposits and withdrawals
//! - `SubAccounts` - sub-account management
//! - `FundingHistory` - historical funding payments
//! - `AccountLedger` - full account ledger
//! - `Authenticated` - credential-aware connectors

mod identity;
mod market_data;
mod market_data_public;
mod market_data_public_stubs;
mod trading;
mod account;
mod positions;
mod websocket;
mod auth;
mod operations;
mod operations_stubs;
mod has_capabilities;
pub mod has_capabilities_stubs;

pub use identity::ExchangeIdentity;
pub use market_data::MarketData;
pub use market_data_public::MarketDataPublic;
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
    FundingHistory, AccountLedger,
};
pub use has_capabilities::HasCapabilities;
pub use crate::core::websocket::CapabilityProvider;
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
    ExchangeIdentity
    + MarketData
    + MarketDataPublic
    + Trading
    + Account
    + Positions
    + CancelAll
    + AmendOrder
    + BatchOrders
    + AccountTransfers
    + CustodialFunds
    + SubAccounts
    + FundingHistory
    + AccountLedger
    + HasCapabilities
    + Send
    + Sync
    + 'static
{
    /// Downcast to a concrete connector for exchange-specific inherent methods.
    ///
    /// Example:
    /// ```ignore
    /// let conn = pool.get(&ExchangeId::Binance)?;
    /// if let Some(binance) = conn.as_any().downcast_ref::<BinanceConnector>() {
    ///     let basis = binance.get_basis_history("BTCUSDT", "PERPETUAL", "5m", None, None, None).await?;
    /// }
    /// ```
    fn as_any(&self) -> &dyn std::any::Any;
}

impl<T> CoreConnector for T where
    T: ExchangeIdentity
        + MarketData
        + MarketDataPublic
        + Trading
        + Account
        + Positions
        + CancelAll
        + AmendOrder
        + BatchOrders
        + AccountTransfers
        + CustodialFunds
        + SubAccounts
        + FundingHistory
        + AccountLedger
        + HasCapabilities
        + Send
        + Sync
        + 'static
{
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
