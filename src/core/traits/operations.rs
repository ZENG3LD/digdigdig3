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

use async_trait::async_trait;
use serde_json::Value;

use crate::core::types::{
    AccountType, ExchangeResult, Order,
    AmendRequest, CancelScope, CancelAllResponse, OrderRequest, OrderResult,
    TransferRequest, TransferHistoryFilter, TransferResponse,
    DepositAddress, WithdrawRequest, WithdrawResponse, FundsRecord,
    FundsHistoryFilter, SubAccountOperation, SubAccountResult,
    MarginBorrowResponse, MarginRepayResponse, MarginInterestRecord,
    EarnProduct, EarnPosition, ConvertQuote,
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
    ) -> ExchangeResult<CancelAllResponse>;
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
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order>;
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
    ) -> ExchangeResult<Vec<OrderResult>>;

    /// Cancel multiple orders in a single native batch request.
    ///
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
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse>;

    /// Get the history of internal transfers.
    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>>;
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
    ) -> ExchangeResult<SubAccountResult>;
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARGIN TRADING
// ═══════════════════════════════════════════════════════════════════════════════

/// Margin borrowing and repayment operations.
///
/// Available on exchanges that support cross/isolated margin accounts:
/// Binance, Bybit, OKX, KuCoin, GateIO, HTX, Bitget, BingX, Phemex, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait MarginTrading: Send + Sync {
    /// Borrow an asset for margin trading.
    async fn margin_borrow(
        &self,
        asset: &str,
        amount: f64,
        account_type: AccountType,
    ) -> ExchangeResult<MarginBorrowResponse> {
        let _ = (asset, amount, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "margin_borrow".to_string(),
        ))
    }

    /// Repay a margin loan.
    async fn margin_repay(
        &self,
        asset: &str,
        amount: f64,
        account_type: AccountType,
    ) -> ExchangeResult<MarginRepayResponse> {
        let _ = (asset, amount, account_type);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "margin_repay".to_string(),
        ))
    }

    /// Get margin interest history for an asset (or all assets if `None`).
    async fn get_margin_interest(
        &self,
        asset: Option<&str>,
    ) -> ExchangeResult<Vec<MarginInterestRecord>> {
        let _ = asset;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_margin_interest".to_string(),
        ))
    }

    /// Get margin account details.
    async fn get_margin_account(
        &self,
        account_type: AccountType,
    ) -> ExchangeResult<Value> {
        let _ = account_type;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_margin_account".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EARN / STAKING
// ═══════════════════════════════════════════════════════════════════════════════

/// Earn and staking product management (flexible savings, locked products, etc.).
///
/// Available on: Binance Earn, Bybit Earn, OKX Earn, KuCoin Pool-X, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait EarnStaking: Send + Sync {
    /// List available earn products, optionally filtered by asset.
    async fn get_earn_products(
        &self,
        asset: Option<&str>,
    ) -> ExchangeResult<Vec<EarnProduct>> {
        let _ = asset;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_earn_products".to_string(),
        ))
    }

    /// Subscribe (deposit) to an earn product.
    async fn subscribe_earn(
        &self,
        product_id: &str,
        amount: f64,
    ) -> ExchangeResult<Value> {
        let _ = (product_id, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "subscribe_earn".to_string(),
        ))
    }

    /// Redeem (withdraw) from an earn product.
    async fn redeem_earn(
        &self,
        product_id: &str,
        amount: f64,
    ) -> ExchangeResult<Value> {
        let _ = (product_id, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "redeem_earn".to_string(),
        ))
    }

    /// Get current earn positions.
    async fn get_earn_positions(&self) -> ExchangeResult<Vec<EarnPosition>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_earn_positions".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVERT / SWAP
// ═══════════════════════════════════════════════════════════════════════════════

/// Instant asset conversion (quote → accept → settle) and dust collection.
///
/// Available on: Binance Convert, Bybit Convert, OKX Convert, KuCoin Convert, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait ConvertSwap: Send + Sync {
    /// Request a conversion quote from `from_asset` to `to_asset`.
    async fn get_convert_quote(
        &self,
        from_asset: &str,
        to_asset: &str,
        amount: f64,
    ) -> ExchangeResult<ConvertQuote> {
        let _ = (from_asset, to_asset, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_convert_quote".to_string(),
        ))
    }

    /// Accept a previously obtained quote.
    async fn accept_convert_quote(&self, quote_id: &str) -> ExchangeResult<Value> {
        let _ = quote_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "accept_convert_quote".to_string(),
        ))
    }

    /// Get conversion history within an optional time range.
    async fn get_convert_history(
        &self,
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> ExchangeResult<Vec<Value>> {
        let _ = (start_time, end_time);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_convert_history".to_string(),
        ))
    }

    /// Convert small dust balances into a base asset (e.g. BNB, BGB).
    async fn convert_dust(&self, assets: Vec<String>) -> ExchangeResult<Value> {
        let _ = assets;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "convert_dust".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// COPY TRADING
// ═══════════════════════════════════════════════════════════════════════════════

/// Copy-trading — follow lead traders and mirror their positions.
///
/// Available on: Bybit, Bitget, OKX, BingX, MEXC, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait CopyTrading: Send + Sync {
    /// List top lead traders, optionally limited to `limit` results.
    async fn get_lead_traders(&self, limit: Option<u32>) -> ExchangeResult<Vec<Value>> {
        let _ = limit;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_lead_traders".to_string(),
        ))
    }

    /// Start following a lead trader.
    async fn follow_trader(&self, trader_id: &str) -> ExchangeResult<Value> {
        let _ = trader_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "follow_trader".to_string(),
        ))
    }

    /// Stop following a lead trader.
    async fn stop_following(&self, trader_id: &str) -> ExchangeResult<()> {
        let _ = trader_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "stop_following".to_string(),
        ))
    }

    /// Get currently mirrored copy positions.
    async fn get_copy_positions(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_copy_positions".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LIQUIDITY PROVIDER (DEX AMM)
// ═══════════════════════════════════════════════════════════════════════════════

/// Liquidity pool management for DEX AMMs.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait LiquidityProvider: Send + Sync {
    /// Create a new LP position in a pool.
    async fn create_lp_position(
        &self,
        pool_id: &str,
        amount_a: f64,
        amount_b: f64,
    ) -> ExchangeResult<Value> {
        let _ = (pool_id, amount_a, amount_b);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "create_lp_position".to_string(),
        ))
    }

    /// Add liquidity to an existing LP position.
    async fn add_liquidity(
        &self,
        position_id: &str,
        amount_a: f64,
        amount_b: f64,
    ) -> ExchangeResult<Value> {
        let _ = (position_id, amount_a, amount_b);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "add_liquidity".to_string(),
        ))
    }

    /// Remove a percentage of liquidity from an LP position.
    async fn remove_liquidity(
        &self,
        position_id: &str,
        percentage: f64,
    ) -> ExchangeResult<Value> {
        let _ = (position_id, percentage);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "remove_liquidity".to_string(),
        ))
    }

    /// Collect accumulated fees from an LP position.
    async fn collect_fees(&self, position_id: &str) -> ExchangeResult<Value> {
        let _ = position_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "collect_fees".to_string(),
        ))
    }

    /// Get all active LP positions.
    async fn get_lp_positions(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_lp_positions".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// VAULT MANAGER
// ═══════════════════════════════════════════════════════════════════════════════

/// Managed vault deposits and withdrawals (e.g. Paradex vaults, HyperLiquid vaults).
///
/// Available on: Paradex, dYdX MegaVault, HyperLiquid vaults.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait VaultManager: Send + Sync {
    /// List available vaults.
    async fn get_vaults(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_vaults".to_string(),
        ))
    }

    /// Deposit into a vault.
    async fn deposit_vault(&self, vault_id: &str, amount: f64) -> ExchangeResult<Value> {
        let _ = (vault_id, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "deposit_vault".to_string(),
        ))
    }

    /// Withdraw from a vault.
    async fn withdraw_vault(&self, vault_id: &str, amount: f64) -> ExchangeResult<Value> {
        let _ = (vault_id, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "withdraw_vault".to_string(),
        ))
    }

    /// Get current vault positions.
    async fn get_vault_positions(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_vault_positions".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// STAKING DELEGATION (ON-CHAIN / COSMOS)
// ═══════════════════════════════════════════════════════════════════════════════

/// On-chain validator delegation for PoS chains (Cosmos, Ethereum staking, etc.).
///
/// Available on: dYdX (Cosmos), Paradex (StarkEx), and other L1/L2 connectors.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait StakingDelegation: Send + Sync {
    /// Delegate tokens to a validator.
    async fn delegate(&self, validator: &str, amount: f64) -> ExchangeResult<Value> {
        let _ = (validator, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "delegate".to_string(),
        ))
    }

    /// Undelegate (unbond) tokens from a validator.
    async fn undelegate(&self, validator: &str, amount: f64) -> ExchangeResult<Value> {
        let _ = (validator, amount);
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "undelegate".to_string(),
        ))
    }

    /// Get all active delegations.
    async fn get_delegations(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_delegations".to_string(),
        ))
    }

    /// Claim accumulated staking rewards.
    async fn claim_staking_rewards(&self) -> ExchangeResult<Value> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "claim_staking_rewards".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BLOCK TRADE / OTC
// ═══════════════════════════════════════════════════════════════════════════════

/// Block trade and OTC desk operations.
///
/// Available on: Deribit, Bybit OTC, OKX OTC, Kraken OTC, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait BlockTradeOtc: Send + Sync {
    /// Create a block trade request.
    async fn create_block_trade(&self, params: Value) -> ExchangeResult<Value> {
        let _ = params;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "create_block_trade".to_string(),
        ))
    }

    /// Verify/validate a block trade before execution.
    async fn verify_block_trade(&self, params: Value) -> ExchangeResult<Value> {
        let _ = params;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "verify_block_trade".to_string(),
        ))
    }

    /// Execute a verified block trade by ID.
    async fn execute_block_trade(&self, trade_id: &str) -> ExchangeResult<Value> {
        let _ = trade_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "execute_block_trade".to_string(),
        ))
    }

    /// Get block trade history.
    async fn get_block_trades(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_block_trades".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARKET MAKER PROTECTION
// ═══════════════════════════════════════════════════════════════════════════════

/// Market Maker Protection (MMP) — bulk quoting and circuit-breaker config.
///
/// Available on: Deribit, Bybit, OKX, Paradex, Lighter, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait MarketMakerProtection: Send + Sync {
    /// Get current MMP configuration.
    async fn get_mmp_config(&self) -> ExchangeResult<Value> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_mmp_config".to_string(),
        ))
    }

    /// Update MMP configuration (delta/vega limits, time window, etc.).
    async fn set_mmp_config(&self, config: Value) -> ExchangeResult<()> {
        let _ = config;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "set_mmp_config".to_string(),
        ))
    }

    /// Reset MMP state (re-enable quoting after a trigger).
    async fn reset_mmp(&self) -> ExchangeResult<()> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "reset_mmp".to_string(),
        ))
    }

    /// Submit a mass quote (multiple bid/ask pairs in one request).
    async fn mass_quote(&self, quotes: Vec<Value>) -> ExchangeResult<Vec<Value>> {
        let _ = quotes;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "mass_quote".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRIGGER ORDERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Advanced trigger/conditional orders (TP/SL, stop orders beyond basic Trading trait).
///
/// Available on: Bybit, OKX, Bitget, BingX, Phemex, MEXC, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait TriggerOrders: Send + Sync {
    /// Place a trigger/conditional order.
    async fn place_trigger_order(&self, params: Value) -> ExchangeResult<Value> {
        let _ = params;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "place_trigger_order".to_string(),
        ))
    }

    /// Cancel a trigger order by ID.
    async fn cancel_trigger_order(&self, order_id: &str) -> ExchangeResult<()> {
        let _ = order_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "cancel_trigger_order".to_string(),
        ))
    }

    /// Get open or historical trigger orders, optionally filtered by symbol.
    async fn get_trigger_orders(
        &self,
        symbol: Option<&str>,
    ) -> ExchangeResult<Vec<Value>> {
        let _ = symbol;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_trigger_orders".to_string(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PREDICTION MARKET
// ═══════════════════════════════════════════════════════════════════════════════

/// Prediction market event trading (probability-based outcome markets).
///
/// Available on: Polymarket (via CLOB), Kalshi, Manifold, etc.
///
/// Default implementations return `UnsupportedOperation`.
#[async_trait]
pub trait PredictionMarket: Send + Sync {
    /// Get available prediction events.
    async fn get_prediction_events(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_prediction_events".to_string(),
        ))
    }

    /// Get the order book for a specific event.
    async fn get_event_orderbook(&self, event_id: &str) -> ExchangeResult<Value> {
        let _ = event_id;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_event_orderbook".to_string(),
        ))
    }

    /// Place an order on a prediction market event.
    async fn place_prediction_order(&self, params: Value) -> ExchangeResult<Value> {
        let _ = params;
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "place_prediction_order".to_string(),
        ))
    }

    /// Get current prediction market positions.
    async fn get_prediction_positions(&self) -> ExchangeResult<Vec<Value>> {
        Err(crate::core::types::ExchangeError::UnsupportedOperation(
            "get_prediction_positions".to_string(),
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
