//! # Responses — Response types for the trading trait surface
//!
//! These types are returned by the trading and account traits.

use serde::{Deserialize, Serialize};
use super::{Quantity, Asset, Timestamp, Order};

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER RESULTS (BATCH)
// ═══════════════════════════════════════════════════════════════════════════════

/// Result for a single order within a batch operation.
///
/// Used in `BatchOrders::place_orders_batch` and `cancel_orders_batch`
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

/// Response from `CancelAll::cancel_all_orders`.
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

/// Response from placing a Bracket order (`OrderType::Bracket`).
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

/// Response from placing an OCO order (`OrderType::Oco`).
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

/// Response from placing a TWAP or other algo order (`OrderType::Twap`).
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

/// Response from `AccountTransfers::transfer`.
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

/// Deposit address for `CustodialFunds::get_deposit_address`.
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

/// Response from `CustodialFunds::withdraw`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawResponse {
    /// Exchange-assigned withdrawal ID.
    pub withdraw_id: String,

    /// Withdrawal status (e.g. "Pending", "Processing", "Completed", "Failed").
    pub status: String,

    /// Estimated or actual on-chain transaction hash (available after broadcast).
    pub tx_hash: Option<String>,
}

/// A single deposit or withdrawal record from `CustodialFunds::get_funds_history`.
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

/// Fee schedule returned by `Account::get_fees`.
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

/// Unified response from `Trading::place_order` — wraps all order variants.
///
/// Most orders return `Simple(Order)`. Composite order types (Bracket, OCO, Algo)
/// use their dedicated variants to carry all constituent orders.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlaceOrderResponse {
    /// A single order was placed (Market, Limit, StopMarket, StopLimit,
    /// TrailingStop, PostOnly, IOC, FOK, GTD, ReduceOnly, Iceberg).
    Simple(Order),

    /// A bracket order was placed (entry + TP + SL).
    Bracket(Box<BracketResponse>),

    /// An OCO order pair was placed.
    Oco(Box<OcoResponse>),

    /// An algorithmic order (TWAP, etc.) was submitted.
    Algo(AlgoOrderResponse),
}

// ═══════════════════════════════════════════════════════════════════════════════
// MARGIN TRADING TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// Response from `MarginTrading::margin_borrow`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginBorrowResponse {
    /// Exchange-assigned transaction ID.
    pub tran_id: String,

    /// Asset borrowed.
    pub asset: String,

    /// Amount borrowed.
    pub amount: f64,
}

/// Response from `MarginTrading::margin_repay`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginRepayResponse {
    /// Exchange-assigned transaction ID.
    pub tran_id: String,

    /// Asset repaid.
    pub asset: String,

    /// Amount repaid.
    pub amount: f64,
}

/// A single margin interest record from `MarginTrading::get_margin_interest`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginInterestRecord {
    /// Asset for which interest was charged.
    pub asset: String,

    /// Interest amount charged.
    pub interest: f64,

    /// Hourly interest rate applied.
    pub interest_rate: f64,

    /// Unix timestamp (ms) when interest was charged.
    pub timestamp: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// EARN / STAKING TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// An available earn product from `EarnStaking::get_earn_products`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnProduct {
    /// Exchange-assigned product identifier.
    pub product_id: String,

    /// Asset associated with this earn product.
    pub asset: String,

    /// Estimated annual percentage yield.
    pub apy: f64,

    /// Minimum subscription amount (if any).
    pub min_amount: Option<f64>,

    /// Maximum subscription amount (if any).
    pub max_amount: Option<f64>,
}

/// An active earn position from `EarnStaking::get_earn_positions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnPosition {
    /// Exchange-assigned product identifier.
    pub product_id: String,

    /// Asset held in this earn position.
    pub asset: String,

    /// Principal amount subscribed.
    pub amount: f64,

    /// Interest accrued but not yet distributed.
    pub accrued_interest: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONVERT / SWAP TYPES
// ═══════════════════════════════════════════════════════════════════════════════

/// A conversion quote from `ConvertSwap::get_convert_quote`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvertQuote {
    /// Exchange-assigned quote identifier (used to accept the quote).
    pub quote_id: String,

    /// Asset being sold.
    pub from_asset: String,

    /// Asset being bought.
    pub to_asset: String,

    /// Amount of `from_asset` to sell.
    pub from_amount: f64,

    /// Amount of `to_asset` to receive.
    pub to_amount: f64,

    /// Conversion price (`to_asset` per unit of `from_asset`).
    pub price: f64,

    /// Unix timestamp (ms) when this quote expires.
    pub expires_at: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CLOSED PNL RECORD
// ═══════════════════════════════════════════════════════════════════════════════

/// A single closed position P&L record from `Positions::get_closed_pnl`.
///
/// ~12/24: Bybit, OKX, Binance Futures, KuCoin, GateIO, Bitget, BingX,
/// Phemex, Deribit, HyperLiquid, Paradex, dYdX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedPnlRecord {
    /// Trading pair.
    pub symbol: String,
    /// Side of the closed position (e.g. "Long", "Short").
    pub side: String,
    /// Closed size (in base asset units).
    pub closed_size: f64,
    /// Average entry price of the closed position.
    pub avg_entry_price: f64,
    /// Average exit price when the position was closed.
    pub avg_exit_price: f64,
    /// Realized P&L for this close event (in quote asset).
    pub closed_pnl: f64,
    /// Unix timestamp (ms) when the position was closed.
    pub timestamp: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// LONG/SHORT RATIO
// ═══════════════════════════════════════════════════════════════════════════════

/// Long/short ratio snapshot from `Positions::get_long_short_ratio`.
///
/// Market sentiment indicator — proportion of long vs short accounts
/// or positions for a given symbol.
///
/// ~8/24: Binance Futures, Bybit, OKX, KuCoin Futures, Bitget, BingX,
/// GateIO, HTX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LongShortRatio {
    /// Trading pair.
    pub symbol: String,
    /// Fraction of long accounts/positions (0.0–1.0).
    pub long_ratio: f64,
    /// Fraction of short accounts/positions (0.0–1.0).
    pub short_ratio: f64,
    /// Unix timestamp (ms) of the snapshot.
    pub timestamp: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDING PAYMENT
// ═══════════════════════════════════════════════════════════════════════════════

/// A single funding payment record from `FundingHistory::get_funding_payments`.
///
/// Funding payments occur periodically on perpetual futures positions.
/// Negative `payment` means the user paid; positive means the user received.
///
/// ~16/24: all perpetual futures exchanges (Binance, Bybit, OKX, KuCoin,
/// GateIO, Bitget, BingX, Phemex, MEXC, HTX, CryptoCom, Deribit,
/// HyperLiquid, Paradex, dYdX, Lighter).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingPayment {
    /// Trading pair (e.g. "BTCUSDT").
    pub symbol: String,
    /// Funding rate applied at settlement time.
    pub funding_rate: f64,
    /// Position size at the time of settlement (in base asset units).
    pub position_size: f64,
    /// Payment amount — negative = paid by user, positive = received by user.
    pub payment: f64,
    /// Settlement currency (e.g. "USDT", "BTC").
    pub asset: String,
    /// Unix timestamp (ms) of the funding settlement.
    pub timestamp: Timestamp,
}

/// Filter for `FundingHistory::get_funding_payments`.
#[derive(Debug, Clone, Default)]
pub struct FundingFilter {
    /// Optional symbol filter. `None` = all symbols.
    pub symbol: Option<String>,
    /// Start of time range (Unix ms). `None` = exchange default.
    pub start_time: Option<u64>,
    /// End of time range (Unix ms). `None` = now.
    pub end_time: Option<u64>,
    /// Maximum number of records to return. `None` = exchange default.
    pub limit: Option<u32>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// LEDGER ENTRY
// ═══════════════════════════════════════════════════════════════════════════════

/// Category of a ledger entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LedgerEntryType {
    /// Entry from a trade execution (fill).
    Trade,
    /// On-chain or external deposit credited to the account.
    Deposit,
    /// Withdrawal deducted from the account.
    Withdrawal,
    /// Funding payment (perpetual futures).
    Funding,
    /// Trading fee charged.
    Fee,
    /// Fee rebate credited (maker/VIP rebate).
    Rebate,
    /// Internal transfer between account types.
    Transfer,
    /// Forced liquidation.
    Liquidation,
    /// Settlement (options/futures expiry).
    Settlement,
    /// Any other entry type not covered above.
    Other(String),
}

/// A single entry in the account ledger from `AccountLedger::get_ledger`.
///
/// The ledger is a chronological log of all balance changes for an account.
/// Positive `amount` = credit (balance increased); negative = debit.
///
/// ~14/24: Binance, Bybit, OKX, KuCoin, Kraken, GateIO, Bitfinex, Bitget,
/// BingX, Phemex, Deribit, HyperLiquid, Paradex, dYdX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    /// Exchange-assigned ledger entry ID.
    pub id: String,
    /// Asset affected (e.g. "USDT", "BTC").
    pub asset: String,
    /// Amount of change — positive = credit, negative = debit.
    pub amount: f64,
    /// Account balance after this entry (if provided by the exchange).
    pub balance: Option<f64>,
    /// Type of this ledger entry.
    pub entry_type: LedgerEntryType,
    /// Human-readable description of the entry.
    pub description: String,
    /// Related order, trade, or transfer ID (if available).
    pub ref_id: Option<String>,
    /// Unix timestamp (ms) of the entry.
    pub timestamp: Timestamp,
}

/// Filter for `AccountLedger::get_ledger`.
#[derive(Debug, Clone, Default)]
pub struct LedgerFilter {
    /// Optional asset filter. `None` = all assets.
    pub asset: Option<String>,
    /// Optional entry type filter. `None` = all entry types.
    pub entry_type: Option<LedgerEntryType>,
    /// Start of time range (Unix ms). `None` = exchange default.
    pub start_time: Option<u64>,
    /// End of time range (Unix ms). `None` = now.
    pub end_time: Option<u64>,
    /// Maximum number of records to return. `None` = exchange default.
    pub limit: Option<u32>,
}
