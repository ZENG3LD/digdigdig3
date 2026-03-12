//! # OperationsV2 — Optional operation traits (V2 architecture)
//!
//! These traits represent capabilities that are near-universal but not
//! available on every exchange. Each is implemented only by exchanges
//! that support the operation natively.
//!
//! ## NO DEFAULT IMPLEMENTATIONS
//! Unlike the V1 extension traits, these traits have NO default implementations.
//! Connectors either implement the trait (native support) or do not.
//! There is no silent sequential fallback that masks missing capability.
//!
//! ## Traits in this module
//!
//! | Trait | Coverage | Supertraits |
//! |-------|----------|-------------|
//! | `CancelAllV2` | 22/24 | `TradingV2` |
//! | `AmendOrderV2` | 18/24 | `TradingV2` |
//! | `BatchOrdersV2` | 17/24 | `TradingV2` |
//! | `AccountTransfersV2` | 17/20 applicable | `AccountV2` |
//! | `CustodialFundsV2` | 18/20 custodial | `AccountV2` |
//! | `SubAccountsV2` | ~12/24 | `AccountV2` |

use async_trait::async_trait;

use crate::core::types::{
    AccountType, ExchangeResult, Order,
    AmendRequest, CancelScope, CancelAllResponse, OrderRequest, OrderResult,
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord,
    FundsHistoryFilter, SubAccountOperation, SubAccountResult,
};

use super::{TradingV2, AccountV2};

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL V2
// ═══════════════════════════════════════════════════════════════════════════════

/// Cancel all open orders — optionally scoped to a symbol.
///
/// 22/24: missing GMX (no cancel-all endpoint) and dYdX (Cosmos tx-based,
/// no bulk cancel API in v4).
///
/// Connectors implement this trait ONLY if the exchange has a native
/// cancel-all endpoint. No looping over `cancel_order` is permitted.
#[async_trait]
pub trait CancelAllV2: TradingV2 {
    /// Cancel orders matching the given scope.
    ///
    /// `scope` must be `CancelScope::All` or `CancelScope::BySymbol`.
    /// Other scopes are handled by `TradingV2::cancel_order`.
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND ORDER V2
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
pub trait AmendOrderV2: TradingV2 {
    /// Modify a live order's price, quantity, and/or trigger price.
    ///
    /// At least one field in `req.fields` must be `Some`.
    /// The connector rejects requests where no field changes.
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// BATCH ORDERS V2
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
pub trait BatchOrdersV2: TradingV2 {
    /// Place multiple orders in a single native batch request.
    ///
    /// Returns one `OrderResult` per input order, in the same order.
    /// Individual failures are represented in `OrderResult::success = false`
    /// rather than returning an `Err` for the whole batch (partial success is
    /// a common exchange behavior).
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>>;

    /// Cancel multiple orders in a single native batch request.
    ///
    /// `cancels` is a vec of `OrderRequest`s where each must identify a
    /// single order (i.e. `order_type` is not used; only `client_order_id`
    /// or symbol+side are used as identifiers by some exchanges).
    /// Use `CancelRequest` with `CancelScope::Batch` to pass order IDs.
    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>>;

    /// Maximum number of orders allowed in a single batch place request.
    ///
    /// Returns the exchange-imposed limit. Callers must split larger batches.
    fn max_batch_place_size(&self) -> usize;

    /// Maximum number of orders allowed in a single batch cancel request.
    fn max_batch_cancel_size(&self) -> usize;
}

// ═══════════════════════════════════════════════════════════════════════════════
// ACCOUNT TRANSFERS V2
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal transfers between account types (Spot ↔ Futures ↔ Margin).
///
/// 17/20 applicable exchanges (non-custodial DEX excluded):
/// Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex, Gemini, MEXC, HTX,
/// Bitget, BingX, Phemex, CryptoCom, Upbit, Deribit, HyperLiquid, Kraken.
#[async_trait]
pub trait AccountTransfersV2: AccountV2 {
    /// Transfer an asset between two account types.
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse>;

    /// Get the history of internal transfers.
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS V2
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit and withdrawal management for custodial exchanges.
///
/// 18/20 custodial exchanges (DEX/non-custodial excluded):
/// Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, GateIO, Bitfinex,
/// Bitstamp, Gemini, MEXC, HTX, Bitget, BingX, Phemex, CryptoCom,
/// Upbit, Deribit.
#[async_trait]
pub trait CustodialFundsV2: AccountV2 {
    /// Get the deposit address for an asset on a given network.
    ///
    /// `network = None` returns the default / primary network address.
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress>;

    /// Submit a withdrawal request.
    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse>;

    /// Get deposit and/or withdrawal history.
    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB ACCOUNTS V2
// ═══════════════════════════════════════════════════════════════════════════════

/// Sub-account management — create, list, transfer, get balances.
///
/// ~12/24: Binance, Bybit, OKX, KuCoin, GateIO, MEXC, HTX, Bitget,
/// BingX, Phemex, Kraken, Bitfinex.
///
/// Sub-accounts are a CEX-only concept. DEX connectors never implement this.
#[async_trait]
pub trait SubAccountsV2: AccountV2 {
    /// Perform a sub-account operation (create, list, transfer, get balance).
    ///
    /// All operations are unified through `SubAccountOperation` to minimize
    /// trait surface. The result type `SubAccountResult` carries the
    /// relevant fields for each operation type.
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult>;
}
