//! # Responses V2 — Response types for the V2 thin-trait surface
//!
//! These types are returned by the V2 traits. They are separate from the
//! original response types to allow coexistence with the V1 connector layer.

use serde::{Deserialize, Serialize};
use super::{Quantity, Asset, Timestamp, Order};

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER RESULTS (BATCH)
// ═══════════════════════════════════════════════════════════════════════════════

/// Result for a single order within a batch operation.
///
/// Used in `BatchOrdersV2::place_orders_batch` and `cancel_orders_batch`
/// to represent individual success/failure within the batch.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResult {
    /// The order that was placed or cancelled (populated on success).
    pub order: Option<Order>,

    /// Client-assigned order ID (if provided in the request).
    pub client_order_id: Option<String>,

    /// Whether this individual order operation succeeded.
    pub success: bool,

    /// Error message if the individual order failed.
    pub error: Option<String>,

    /// Exchange-assigned error code if the individual order failed.
    pub error_code: Option<i32>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL ALL RESPONSE
// ═══════════════════════════════════════════════════════════════════════════════

/// Response from `CancelAllV2::cancel_all_orders`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllResponse {
    /// Number of orders successfully cancelled.
    pub cancelled_count: u32,

    /// Number of orders that failed to cancel (e.g. already filled).
    pub failed_count: u32,

    /// Detailed per-order results (populated if the exchange returns them).
    /// Empty if the exchange only returns aggregate counts.
    pub details: Vec<OrderResult>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// COMPOSITE ORDER RESPONSES
// ═══════════════════════════════════════════════════════════════════════════════

/// Response from placing a Bracket order (`OrderTypeV2::Bracket`).
///
/// 9/24: Bybit, OKX, Phemex, Bitget, BingX, Deribit, HyperLiquid, Paradex, dYdX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BracketResponse {
    /// The entry order (limit or market).
    pub entry_order: Order,

    /// The take-profit order attached to the entry.
    pub tp_order: Order,

    /// The stop-loss order attached to the entry.
    pub sl_order: Order,
}

/// Response from placing an OCO order (`OrderTypeV2::Oco`).
///
/// 7/24: Binance Spot, Gemini, Kraken, KuCoin Spot, GateIO, OKX, HTX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcoResponse {
    /// The first leg of the OCO pair (limit side).
    pub first_order: Order,

    /// The second leg of the OCO pair (stop side).
    pub second_order: Order,

    /// Exchange-assigned OCO list ID (links both legs together).
    pub list_id: Option<String>,
}

/// Response from placing a TWAP or other algo order (`OrderTypeV2::Twap`).
///
/// 7/24: Binance (algo), Bybit (algo), OKX (algo), KuCoin (algo),
/// Bitget (algo), BingX (algo), HyperLiquid.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgoOrderResponse {
    /// Exchange-assigned algorithm order / task ID.
    pub algo_id: String,

    /// Current algo order status (e.g. "Running", "Paused", "Completed").
    pub status: String,

    /// Number of sub-orders already executed (if available).
    pub executed_count: Option<u32>,

    /// Total number of sub-orders planned (if available).
    pub total_count: Option<u32>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSFER RESPONSE
// ═══════════════════════════════════════════════════════════════════════════════

/// Response from `AccountTransfersV2::transfer`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferResponse {
    /// Exchange-assigned transfer / transaction ID.
    pub transfer_id: String,

    /// Transfer status (e.g. "Successful", "Pending", "Failed").
    pub status: String,

    /// Asset transferred.
    pub asset: Asset,

    /// Amount transferred.
    pub amount: Quantity,

    /// Unix timestamp (ms) when the transfer was processed.
    pub timestamp: Option<Timestamp>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CUSTODIAL FUNDS RESPONSES
// ═══════════════════════════════════════════════════════════════════════════════

/// Deposit address for `CustodialFundsV2::get_deposit_address`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepositAddress {
    /// On-chain address string (EVM 0x, Solana base58, etc.).
    pub address: String,

    /// Destination tag or memo — required for XRP, XLM, EOS, etc.
    pub tag: Option<String>,

    /// Blockchain network identifier (e.g. "ERC20", "TRC20", "BEP20").
    pub network: Option<String>,

    /// Asset this address is for.
    pub asset: Asset,

    /// Unix timestamp (ms) when the address was issued (if available).
    pub created_at: Option<Timestamp>,
}

/// Response from `CustodialFundsV2::withdraw`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResponse {
    /// Exchange-assigned withdrawal ID.
    pub withdraw_id: String,

    /// Withdrawal status (e.g. "Pending", "Processing", "Completed", "Failed").
    pub status: String,

    /// Estimated or actual on-chain transaction hash (available after broadcast).
    pub tx_hash: Option<String>,
}

/// A single deposit or withdrawal record from `CustodialFundsV2::get_funds_history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FundsRecord {
    /// An inbound deposit record.
    Deposit {
        /// Exchange-assigned deposit record ID.
        id: String,
        /// Asset deposited.
        asset: Asset,
        /// Amount received (after any fees).
        amount: Quantity,
        /// On-chain transaction hash.
        tx_hash: Option<String>,
        /// Blockchain network.
        network: Option<String>,
        /// Status (e.g. "Credited", "Pending").
        status: String,
        /// Unix timestamp (ms) when the deposit was credited.
        timestamp: Timestamp,
    },

    /// An outbound withdrawal record.
    Withdrawal {
        /// Exchange-assigned withdrawal record ID.
        id: String,
        /// Asset withdrawn.
        asset: Asset,
        /// Amount sent (before exchange fee).
        amount: Quantity,
        /// Exchange fee charged for the withdrawal.
        fee: Option<Quantity>,
        /// Destination address.
        address: String,
        /// Destination tag / memo.
        tag: Option<String>,
        /// On-chain transaction hash.
        tx_hash: Option<String>,
        /// Blockchain network.
        network: Option<String>,
        /// Status (e.g. "Completed", "Pending", "Failed").
        status: String,
        /// Unix timestamp (ms) when the withdrawal was submitted.
        timestamp: Timestamp,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// FEE INFO
// ═══════════════════════════════════════════════════════════════════════════════

/// Fee schedule returned by `AccountV2::get_fees`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeInfo {
    /// Maker fee rate as a fraction (e.g. 0.001 = 0.1%).
    pub maker_rate: f64,

    /// Taker fee rate as a fraction (e.g. 0.001 = 0.1%).
    pub taker_rate: f64,

    /// Optional: symbol these fees apply to (None = account-wide default).
    pub symbol: Option<String>,

    /// Optional: VIP / fee tier level label (e.g. "VIP1", "Regular").
    pub tier: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// UNIFIED ORDER PLACEMENT RESPONSE
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified response from `TradingV2::place_order` — wraps all order variants.
///
/// Most orders return `Simple(Order)`. Composite order types (Bracket, OCO, Algo)
/// use their dedicated variants to carry all constituent orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaceOrderResponse {
    /// A single order was placed (Market, Limit, StopMarket, StopLimit,
    /// TrailingStop, PostOnly, IOC, FOK, GTD, ReduceOnly, Iceberg).
    Simple(Order),

    /// A bracket order was placed (entry + TP + SL).
    Bracket(BracketResponse),

    /// An OCO order pair was placed.
    Oco(OcoResponse),

    /// An algorithmic order (TWAP, etc.) was submitted.
    Algo(AlgoOrderResponse),
}
