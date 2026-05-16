//! # Operations — Optional operation traits
//!
//! These traits represent capabilities that are near-universal but not
//! available on every exchange. Each is implemented only by exchanges
//! that support the operation natively.
//!
//! ## NO DEFAULT IMPLEMENTATIONS
//! These traits have NO default implementations.
//! Connectors either implement the trait (native support) or do not.
//! There is no silent sequential fallback that masks missing capability.
//!
//! ## Traits in this module
//!
//! | Trait | Coverage | Supertraits |
//! |-------|----------|-------------|
//! | `CancelAll` | 22/24 | `Trading` |
//! | `AmendOrder` | 18/24 | `Trading` |
//! | `BatchOrders` | 17/24 | `Trading` |
//! | `AccountTransfers` | 17/20 applicable | `Account` |
//! | `CustodialFunds` | 18/20 custodial | `Account` |
//! | `SubAccounts` | ~12/24 | `Account` |
//! | `FundingHistory` | ~16/24 | `Send + Sync` |
//! | `AccountLedger` | ~14/24 | `Send + Sync` |

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, Order,
    AmendRequest, CancelScope, CancelAllResponse, OrderRequest, OrderResult,
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord,
    FundsHistoryFilter, SubAccountOperation, SubAccountResult,
    FundingPayment, FundingFilter,
    LedgerEntry, LedgerFilter,
};

use super::{Trading, Account};

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders — optionally scoped to a symbol.
///
/// All exchanges except dYdX (Cosmos tx-based, no bulk cancel API in v4).
///
/// Connectors implement this trait ONLY if the exchange has a native
/// cancel-all endpoint. No looping over `cancel_order` is permitted.
#[async_trait]
pub trait CancelAll: Trading {
    /// Cancel orders matching the given scope.
    ///
    /// `scope` must be `CancelScope::All` or `CancelScope::BySymbol`.
    /// Other scopes are handled by `Trading::cancel_order`.
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse> {
        let _ = (scope, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "cancel_all_orders not supported".into(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER
// ═══════════════════════════════════════════════════════════════════════════════

/// Amend (modify) a live order in-place without cancel+replace.
///
/// 18/24: Binance Futures, Bybit, OKX, KuCoin, GateIO, Bitfinex, MEXC, HTX,
/// Bitget, BingX, Phemex, CryptoCom, Deribit, HyperLiquid, Lighter,
/// Paradex, dYdX, Upbit.
///
/// Connectors that implement this trait have a native modify/amend endpoint.
/// Connectors that DON'T implement this trait simply do not have the trait —
/// callers must cancel+replace manually at the application layer if needed.
#[async_trait]
pub trait AmendOrder: Trading {
    /// Modify a live order's price, quantity, and/or trigger price.
    ///
    /// At least one field in `req.fields` must be `Some`.
    /// The connector rejects requests where no field changes.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order> {
        let _ = req;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "amend_order not supported".into(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Native batch order placement and cancellation.
///
/// 17/24: Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex, MEXC, HTX, Bitget,
/// BingX, Phemex, CryptoCom, Deribit, HyperLiquid, Lighter, Paradex, dYdX.
///
/// Connectors implement this trait ONLY when the exchange has a native
/// batch endpoint (one HTTP request for multiple orders).
/// NO sequential loops are permitted even as a fallback.
#[async_trait]
pub trait BatchOrders: Trading {
    /// Place multiple orders in a single native batch request.
    ///
    /// Returns one `OrderResult` per input order, in the same order.
    /// Individual failures are represented in `OrderResult::success = false`
    /// rather than returning an `Err` for the whole batch (partial success is
    /// a common exchange behavior).
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>> {
        let _ = orders;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "place_orders_batch not supported".into(),
        ))
    }

    /// Cancel multiple orders in a single native batch request.
    ///
    /// Use `CancelRequest` with `CancelScope::Batch` to pass order IDs.
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>> {
        let _ = (order_ids, symbol, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "cancel_orders_batch not supported".into(),
        ))
    }

    /// Maximum number of orders allowed in a single batch place request.
    ///
    /// Returns the exchange-imposed limit. Callers must split larger batches.
    fn max_batch_place_size(&self) -> usize {
        0
    }

    /// Maximum number of orders allowed in a single batch cancel request.
    fn max_batch_cancel_size(&self) -> usize {
        0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal transfers between account types (Spot ↔ Futures ↔ Margin).
///
/// 17/20 applicable exchanges (non-custodial DEX excluded):
/// Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex, Gemini, MEXC, HTX,
/// Bitget, BingX, Phemex, CryptoCom, Upbit, Deribit, HyperLiquid, Kraken.
#[async_trait]
pub trait AccountTransfers: Account {
    /// Transfer an asset between two account types.
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse> {
        let _ = req;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "transfer not supported".into(),
        ))
    }

    /// Get the history of internal transfers.
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>> {
        let _ = filter;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_transfer_history not supported".into(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management for custodial exchanges.
///
/// 18/20 custodial exchanges (DEX/non-custodial excluded):
/// Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, GateIO, Bitfinex,
/// Bitstamp, Gemini, MEXC, HTX, Bitget, BingX, Phemex, CryptoCom,
/// Upbit, Deribit.
#[async_trait]
pub trait CustodialFunds: Account {
    /// Get the deposit address for an asset on a given network.
    ///
    /// `network = None` returns the default / primary network address.
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress> {
        let _ = (asset, network);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_deposit_address not supported".into(),
        ))
    }

    /// Submit a withdrawal request.
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse> {
        let _ = req;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "withdraw not supported".into(),
        ))
    }

    /// Get deposit and/or withdrawal history.
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>> {
        let _ = filter;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_funds_history not supported".into(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB ACCOUNTS
// ═══════════════════════════════════════════════════════════════════════════════

/// Sub-account management — create, list, transfer, get balances.
///
/// ~12/24: Binance, Bybit, OKX, KuCoin, GateIO, MEXC, HTX, Bitget,
/// BingX, Phemex, Kraken, Bitfinex.
///
/// Sub-accounts are a CEX-only concept. DEX connectors never implement this.
#[async_trait]
pub trait SubAccounts: Account {
    /// Perform a sub-account operation (create, list, transfer, get balance).
    ///
    /// All operations are unified through `SubAccountOperation` to minimize
    /// trait surface. The result type `SubAccountResult` carries the
    /// relevant fields for each operation type.
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult> {
        let _ = op;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "sub_account_operation not supported".into(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING HISTORY
// ═══════════════════════════════════════════════════════════════════════════════

/// Historical funding payment records for perpetual futures positions.
///
/// ~16/24: all perpetual futures exchanges — Binance, Bybit, OKX, KuCoin,
/// GateIO, Bitget, BingX, Phemex, MEXC, HTX, CryptoCom, Deribit,
/// HyperLiquid, Paradex, dYdX, Lighter.
///
/// Default implementation returns `UnsupportedOperation`.
/// Connectors that expose a native funding payment history endpoint override
/// `get_funding_payments`.
#[async_trait]
pub trait FundingHistory: Send + Sync {
    /// Get historical funding payments for the account.
    ///
    /// `filter.symbol = None` returns payments across all symbols (if the
    /// exchange supports global queries). `account_type` selects the futures
    /// account tier (FuturesCross, FuturesIsolated).
    async fn get_funding_payments(
        &self,
        filter: FundingFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<FundingPayment>> {
        let _ = (filter, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_funding_payments not implemented".into(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT LEDGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Full account ledger — chronological log of all balance changes.
///
/// ~14/24: Binance, Bybit, OKX, KuCoin, Kraken, GateIO, Bitfinex, Bitget,
/// BingX, Phemex, Deribit, HyperLiquid, Paradex, dYdX.
///
/// Default implementation returns `UnsupportedOperation`.
/// Connectors that expose a native ledger/transaction-log endpoint override
/// `get_ledger`.
#[async_trait]
pub trait AccountLedger: Send + Sync {
    /// Get the account ledger — all balance change entries matching the filter.
    ///
    /// `filter.asset = None` returns entries for all assets.
    /// `filter.entry_type = None` returns all entry types.
    /// `account_type` selects which account sub-type to query.
    async fn get_ledger(
        &self,
        filter: LedgerFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<LedgerEntry>> {
        let _ = (filter, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_ledger not implemented".into(),
        ))
    }
}
