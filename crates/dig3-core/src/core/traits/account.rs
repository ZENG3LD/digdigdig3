//! # Account — Core account operations
//!
//! Core account operations universal to 24/24 exchanges.

use async_trait::async_trait;

use crate::core::types::{
    AccountCapabilities, AccountInfo, AccountType, Balance, BalanceQuery, ExchangeResult, FeeInfo,
};

use super::ExchangeIdentity;

/// Core account — 24/24 exchanges.
///
/// All account-level read operations. Write operations (transfers, withdrawals,
/// sub-accounts) are in their own optional traits in `operations`.
///
/// Authentication is **required** for all methods in this trait.
#[async_trait]
pub trait Account: ExchangeIdentity {
    /// Get asset balances, optionally filtered to a single asset and/or account type.
    ///
    /// `query.asset = None` returns all assets with non-zero balance.
    /// `query.asset = Some("BTC")` returns only the BTC balance entry (as a 1-element vec
    /// or an empty vec if no BTC is held).
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>>;

    /// Get account metadata — permissions, commission rates, balance summary.
    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo>;

    /// Get the fee schedule (maker/taker rates) for this account.
    ///
    /// `symbol = None` returns the account-wide default fee tier.
    /// `symbol = Some("BTC/USDT")` returns symbol-specific fees (some exchanges
    /// allow per-symbol fee negotiation).
    ///
    /// DEX AMMs use protocol fee models not translatable to maker/taker — they return
    /// `UnsupportedOperation`.
    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo>;

    /// Returns the connector's account capabilities.
    ///
    /// The default implementation returns permissive defaults.
    /// Connectors with specific limitations should override this method.
    fn account_capabilities(&self, account_type: AccountType) -> AccountCapabilities {
        let _ = account_type;
        AccountCapabilities::permissive()
    }
}
