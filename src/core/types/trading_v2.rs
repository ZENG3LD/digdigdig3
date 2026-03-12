//! # Trading V2 Types — Fat Enums + Unified Requests
//!
//! Fat-enum architecture: all complexity lives in enum variants.
//! Every connector matches only the variants it supports natively;
//! unmatched variants return `ExchangeError::UnsupportedOperation`.
//!
//! ## Design Rules
//! - NO default implementations — every connector must be explicit
//! - NO composition — connectors NEVER loop over base methods to simulate missing features
//! - ALL complexity in enum variants — reading the enum IS reading the capability matrix

use serde::{Deserialize, Serialize};
use super::{AccountType, Price, Quantity, Asset, Timestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER TYPE V2 — fat enum covering all 24 exchanges
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified order type enum — covers all order variants across 24 exchanges.
///
/// A connector matches only the variants it supports natively.
/// For unsupported variants it returns `ExchangeError::UnsupportedOperation`.
///
/// Reading this enum IS reading the capability matrix for order types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OrderTypeV2 {
    /// Plain market order — executes at best available price.
    ///
    /// 24/24 exchanges support this.
    Market,

    /// Limit order — executes at `price` or better.
    ///
    /// 24/24 exchanges support this.
    Limit {
        /// Limit price. Mandatory.
        price: Price,
    },

    /// Stop market — triggers a market order when `stop_price` is reached.
    ///
    /// 19/24: Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, GateIO, Bitfinex,
    /// Bitstamp, MEXC, HTX, Bitget, BingX, Phemex, CryptoCom, Deribit,
    /// HyperLiquid, Paradex, dYdX.
    StopMarket {
        /// Price at which the stop triggers and a market order is placed.
        stop_price: Price,
    },

    /// Stop limit — triggers a limit order when `stop_price` is reached.
    ///
    /// 19/24: same exchanges as StopMarket minus DEX-only (GMX, Jupiter, Uniswap,
    /// Raydium + some CEX spot-only).
    StopLimit {
        /// Price at which the stop triggers.
        stop_price: Price,
        /// Limit price of the order that gets placed after trigger.
        limit_price: Price,
    },

    /// Trailing stop — follows best price by `callback_rate` percent.
    ///
    /// 10/24: Binance Futures, Bybit, OKX, KuCoin Futures, Bitget, BingX,
    /// Phemex, Deribit, HyperLiquid, Paradex.
    TrailingStop {
        /// Distance from peak price as a percentage (e.g. 1.0 = 1%).
        callback_rate: f64,
        /// Optional price at which trailing tracking begins.
        activation_price: Option<Price>,
    },

    /// OCO (One-Cancels-the-Other) — a limit order paired with a stop order.
    /// When one fills or triggers, the other is automatically cancelled.
    ///
    /// 7/24: Binance Spot, Gemini, Kraken, KuCoin Spot, GateIO, OKX, HTX.
    Oco {
        /// Limit side price (must be above market for sell, below for buy).
        price: Price,
        /// Stop trigger price.
        stop_price: Price,
        /// Limit price after the stop triggers (None = market after trigger).
        stop_limit_price: Option<Price>,
    },

    /// Bracket order — entry + attached TP + SL, all in one atomic request.
    ///
    /// 9/24: Bybit, OKX, Phemex, Bitget, BingX, Deribit, HyperLiquid,
    /// Paradex, dYdX.
    Bracket {
        /// Entry limit price (None = market entry).
        price: Option<Price>,
        /// Take-profit trigger price.
        take_profit: Price,
        /// Stop-loss trigger price.
        stop_loss: Price,
    },

    /// Iceberg order — large order split into smaller visible chunks.
    ///
    /// 8/24: Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex, Bitstamp, Deribit.
    Iceberg {
        /// Full order price.
        price: Price,
        /// Size of each visible slice placed on the book.
        display_quantity: Quantity,
    },

    /// TWAP (Time-Weighted Average Price) — algorithmic order splitting
    /// execution over a time window.
    ///
    /// 7/24: Binance (algo), Bybit (algo), OKX (algo), KuCoin (algo),
    /// Bitget (algo), BingX (algo), HyperLiquid.
    Twap {
        /// Total duration to split execution over, in seconds.
        duration_seconds: u64,
        /// Optional sub-order interval in seconds. Exchange default if None.
        interval_seconds: Option<u64>,
    },

    /// Post-Only limit — rejected if it would immediately match.
    /// Guarantees maker fee.
    ///
    /// 20/24: all except GMX, Uniswap, Raydium, Jupiter (AMM / no maker/taker).
    PostOnly {
        /// Limit price.
        price: Price,
    },

    /// Immediate-Or-Cancel — fills what it can immediately, cancels the rest.
    ///
    /// 21/24: all except GMX, Uniswap, Raydium (AMM semantics don't apply).
    Ioc {
        /// Limit price (None = market price for IOC market sweep).
        price: Option<Price>,
    },

    /// Fill-Or-Kill — must fill in full immediately or the entire order is cancelled.
    ///
    /// 17/24: Binance, Bybit, OKX, KuCoin, Kraken, GateIO, Bitfinex, Bitstamp,
    /// Gemini, MEXC, HTX, Bitget, Phemex, Deribit, HyperLiquid, Paradex, dYdX.
    Fok {
        /// Limit price (mandatory — FOK with market price is rare).
        price: Price,
    },

    /// Good-Till-Date — limit order that expires at `expire_time`.
    ///
    /// 8/24: Binance, Bybit, OKX, KuCoin, Kraken, Bitget, Deribit, Paradex.
    Gtd {
        /// Limit price.
        price: Price,
        /// Unix timestamp (ms) after which the order is cancelled.
        expire_time: Timestamp,
    },

    /// Reduce-Only limit — only allowed to reduce an open position.
    ///
    /// 19/24: all futures-capable exchanges.
    /// Returns `UnsupportedOperation` for spot-only exchanges.
    ReduceOnly {
        /// Limit price (None = market).
        price: Option<Price>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// TIME IN FORCE V2
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified time-in-force for the V2 trait surface.
///
/// Used in `OrderRequest` alongside `OrderTypeV2` when TIF is separate from
/// order type (some exchanges encode PostOnly as TIF, others as order type).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum TimeInForceV2 {
    /// Good-Till-Cancel — remains open until filled or explicitly cancelled.
    ///
    /// 24/24 exchanges.
    #[default]
    Gtc,

    /// Immediate-Or-Cancel — fill what is possible now, cancel the remainder.
    ///
    /// 21/24 exchanges (not GMX, Uniswap, Raydium).
    Ioc,

    /// Fill-Or-Kill — fill entirely now or cancel entirely.
    ///
    /// 17/24 exchanges.
    Fok,

    /// Post-Only — reject if the order would cross the spread (taker fill).
    ///
    /// 20/24 exchanges (not GMX, Uniswap, Raydium, Jupiter).
    PostOnly,

    /// Good-Till-Date — cancel at `expire_time` specified in the order.
    ///
    /// 8/24 exchanges (Binance, Bybit, OKX, KuCoin, Kraken, Bitget, Deribit, Paradex).
    Gtd,

    /// Good-Till-Block — cancel after a specific blockchain block height.
    ///
    /// 3/24: dYdX v4, Paradex, Lighter (all Cosmos/Starknet-based L2s).
    GoodTilBlock {
        /// Block height after which the order expires.
        block_height: u64,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER REQUEST
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified order placement request for `TradingV2::place_order`.
///
/// The connector inspects `order_type` and matches the variants it supports.
/// `time_in_force` is separate because some exchanges encode TIF as part of
/// order type while others use a dedicated field — connectors translate as needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderRequest {
    /// Trading pair.
    pub symbol: super::Symbol,

    /// Direction — buy or sell.
    pub side: super::OrderSide,

    /// Order type with all parameters embedded.
    pub order_type: OrderTypeV2,

    /// Total quantity to trade in base asset units.
    pub quantity: Quantity,

    /// Time in force (some exchanges ignore this if encoded in `order_type`).
    pub time_in_force: TimeInForceV2,

    /// Account type — Spot, Margin, FuturesCross, FuturesIsolated.
    pub account_type: AccountType,

    /// Optional client-assigned order ID for idempotency / tracking.
    pub client_order_id: Option<String>,

    /// Reduce-only flag — only valid for futures account types.
    /// If `true` and `order_type` is not `ReduceOnly`, the connector should
    /// apply reduce-only semantics if the exchange supports it as a flag.
    pub reduce_only: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// CANCEL REQUEST
// ═══════════════════════════════════════════════════════════════════════════════

/// How many / which orders to cancel in a single request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CancelScope {
    /// Cancel a single order by its exchange-assigned ID.
    ///
    /// 24/24 exchanges.
    Single {
        /// Exchange-assigned order ID.
        order_id: String,
    },

    /// Cancel a batch of orders by their IDs.
    ///
    /// 17/24 exchanges (same set as `BatchOrdersV2`).
    Batch {
        /// List of exchange-assigned order IDs to cancel.
        order_ids: Vec<String>,
    },

    /// Cancel ALL open orders — optionally filtered to a single symbol.
    ///
    /// 22/24 exchanges (missing GMX, dYdX which have no native cancel-all).
    All {
        /// If `Some(symbol)`, only cancel orders for that symbol.
        /// If `None`, cancel all open orders across all symbols.
        symbol: Option<super::Symbol>,
    },

    /// Cancel all open orders for a specific symbol (explicit symbol scope).
    ///
    /// 22/24 exchanges (same as `All` with `Some(symbol)`).
    BySymbol {
        /// The symbol whose orders should all be cancelled.
        symbol: super::Symbol,
    },
}

/// Cancel order request for `TradingV2::cancel_order`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelRequest {
    /// What to cancel and how many.
    pub scope: CancelScope,

    /// Symbol hint — some exchanges require it even for single-order cancels.
    /// Mandatory when `scope` is `Single` on Binance, KuCoin, GateIO, etc.
    pub symbol: Option<super::Symbol>,

    /// Account type context.
    pub account_type: AccountType,
}

// ═══════════════════════════════════════════════════════════════════════════════
// AMEND REQUEST
// ═══════════════════════════════════════════════════════════════════════════════

/// Fields that can be changed on a live order via amend.
///
/// All fields are `Option` — `None` means "keep the existing value".
/// At least one field must be `Some` for a valid amend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendFields {
    /// New limit price. `None` = no change.
    pub price: Option<Price>,

    /// New quantity (total, not remaining). `None` = no change.
    pub quantity: Option<Quantity>,

    /// New trigger/stop price. `None` = no change.
    pub trigger_price: Option<Price>,
}

/// Amend (modify) a live order request for `AmendOrderV2::amend_order`.
///
/// 18/24 exchanges natively support amend (modify-in-place without cancel+replace).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmendRequest {
    /// Exchange-assigned order ID of the order to amend.
    pub order_id: String,

    /// Symbol — required by most exchanges.
    pub symbol: super::Symbol,

    /// Account type context.
    pub account_type: AccountType,

    /// The fields to change. At least one must be `Some`.
    pub fields: AmendFields,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDER HISTORY FILTER
// ═══════════════════════════════════════════════════════════════════════════════

/// Filter for `TradingV2::get_order_history`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderHistoryFilter {
    /// Optional symbol filter. `None` = all symbols (if the exchange supports it).
    pub symbol: Option<super::Symbol>,

    /// Start of time range (Unix ms). `None` = exchange default (usually 24h or 7d ago).
    pub start_time: Option<Timestamp>,

    /// End of time range (Unix ms). `None` = now.
    pub end_time: Option<Timestamp>,

    /// Maximum number of records to return. `None` = exchange default.
    pub limit: Option<u32>,

    /// Filter by order status. `None` = all closed statuses.
    pub status: Option<super::OrderStatus>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ORDERS QUERY
// ═══════════════════════════════════════════════════════════════════════════════

/// Query type for fetching orders — open vs historical.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrdersQuery {
    /// Fetch currently open orders.
    ///
    /// 24/24 exchanges.
    Open {
        /// Optional symbol scope. `None` = all symbols.
        symbol: Option<super::Symbol>,
    },

    /// Fetch order history (closed, filled, cancelled).
    ///
    /// 24/24 exchanges (parameters vary).
    History(OrderHistoryFilter),

    /// Fetch specific orders by their IDs.
    ///
    /// 15/24 exchanges support batch order lookup by ID.
    ByIds {
        /// List of order IDs.
        order_ids: Vec<String>,
        /// Symbol — required by some exchanges.
        symbol: Option<super::Symbol>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITION MODIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// All position mutation operations via a single enum for `PositionsV2::modify_position`.
///
/// The connector matches the variants it supports; returns `UnsupportedOperation`
/// for variants not natively supported by the exchange.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionModification {
    /// Change the leverage multiplier for a symbol.
    ///
    /// 19/24 futures-capable exchanges.
    SetLeverage {
        /// Trading pair.
        symbol: super::Symbol,
        /// New leverage multiplier (e.g. 10 = 10x).
        leverage: u32,
        /// Account type (FuturesCross or FuturesIsolated).
        account_type: AccountType,
    },

    /// Switch between cross-margin and isolated-margin for a symbol.
    ///
    /// 16/24: Binance, Bybit, OKX, KuCoin, GateIO, MEXC, HTX, Bitget,
    /// BingX, Phemex, CryptoCom, Deribit, HyperLiquid, Paradex, dYdX, Lighter.
    SetMarginMode {
        /// Trading pair.
        symbol: super::Symbol,
        /// Target margin mode.
        margin_type: super::MarginType,
        /// Account type context.
        account_type: AccountType,
    },

    /// Add additional margin to an isolated-margin position.
    ///
    /// 12/24: Binance, Bybit, OKX, KuCoin, GateIO, Bitget, BingX, Phemex,
    /// CryptoCom, Deribit, HyperLiquid, Paradex.
    AddMargin {
        /// Trading pair.
        symbol: super::Symbol,
        /// Amount of margin to add (in quote asset or USDT).
        amount: Quantity,
        /// Account type (must be FuturesIsolated).
        account_type: AccountType,
    },

    /// Remove margin from an isolated-margin position.
    ///
    /// 10/24: Bybit, OKX, KuCoin, GateIO, Bitget, BingX, Phemex,
    /// Deribit, HyperLiquid, Paradex.
    RemoveMargin {
        /// Trading pair.
        symbol: super::Symbol,
        /// Amount of margin to remove.
        amount: Quantity,
        /// Account type (must be FuturesIsolated).
        account_type: AccountType,
    },

    /// Close the entire position at market price.
    ///
    /// 22/24 futures-capable exchanges.
    ClosePosition {
        /// Trading pair.
        symbol: super::Symbol,
        /// Account type context.
        account_type: AccountType,
    },

    /// Set or update TP/SL prices on an open position.
    ///
    /// 15/24: Bybit, OKX, KuCoin, Bitget, BingX, Phemex, CryptoCom,
    /// Deribit, HyperLiquid, Paradex, dYdX, Lighter, GateIO, MEXC, HTX.
    SetTpSl {
        /// Trading pair.
        symbol: super::Symbol,
        /// New take-profit price. `None` = keep existing.
        take_profit: Option<Price>,
        /// New stop-loss price. `None` = keep existing.
        stop_loss: Option<Price>,
        /// Account type context.
        account_type: AccountType,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// POSITION QUERY
// ═══════════════════════════════════════════════════════════════════════════════

/// Query parameters for `PositionsV2::get_positions`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionQuery {
    /// Optional symbol filter. `None` = all open positions.
    pub symbol: Option<super::Symbol>,

    /// Account type (FuturesCross or FuturesIsolated).
    pub account_type: AccountType,
}

// ═══════════════════════════════════════════════════════════════════════════════
// BALANCE QUERY
// ═══════════════════════════════════════════════════════════════════════════════

/// Query parameters for `AccountV2::get_balance`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceQuery {
    /// Optional asset filter (e.g. "BTC", "USDT"). `None` = all assets.
    pub asset: Option<Asset>,

    /// Account type scope (Spot, Margin, FuturesCross, FuturesIsolated).
    pub account_type: AccountType,
}

// ═══════════════════════════════════════════════════════════════════════════════
// TRANSFER REQUEST
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal transfer between account types for `AccountTransfersV2::transfer`.
///
/// 17/20 custodial exchanges support this (DEX/non-custodial excluded).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferRequest {
    /// Asset to transfer (e.g. "USDT", "BTC").
    pub asset: Asset,

    /// Amount to transfer.
    pub amount: Quantity,

    /// Source account type (e.g. Spot).
    pub from_account: AccountType,

    /// Destination account type (e.g. FuturesCross).
    pub to_account: AccountType,
}

/// Filter for transfer history queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferHistoryFilter {
    /// Start of time range (Unix ms). `None` = exchange default.
    pub start_time: Option<Timestamp>,

    /// End of time range (Unix ms). `None` = now.
    pub end_time: Option<Timestamp>,

    /// Maximum records to return. `None` = exchange default.
    pub limit: Option<u32>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// SUB-ACCOUNT OPERATION
// ═══════════════════════════════════════════════════════════════════════════════

/// All sub-account operations via a single enum for `SubAccountsV2::sub_account_operation`.
///
/// ~12/24 exchanges support sub-accounts (CEX-only concept).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SubAccountOperation {
    /// Create a new sub-account.
    ///
    /// ~10/24: Binance, Bybit, OKX, KuCoin, Bitget, BingX, HTX, MEXC, GateIO, Phemex.
    Create {
        /// Display label for the sub-account.
        label: String,
    },

    /// List all sub-accounts under this master account.
    ///
    /// ~12/24: same exchanges as Create plus Kraken, Bitfinex.
    List,

    /// Transfer funds from master to sub-account or vice versa.
    ///
    /// ~10/24: same set as Create.
    Transfer {
        /// Target sub-account identifier.
        sub_account_id: String,
        /// Asset to transfer.
        asset: Asset,
        /// Amount to transfer.
        amount: Quantity,
        /// `true` = master → sub; `false` = sub → master.
        to_sub: bool,
    },

    /// Get balance of a specific sub-account.
    ///
    /// ~10/24: same set as Create.
    GetBalance {
        /// Sub-account identifier.
        sub_account_id: String,
    },
}

/// Result of a sub-account operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAccountResult {
    /// Sub-account ID (returned by Create, used in subsequent operations).
    pub id: Option<String>,

    /// Sub-account display name / label.
    pub name: Option<String>,

    /// List of sub-accounts (populated by List operation).
    pub accounts: Vec<SubAccount>,

    /// Transfer or balance result (populated by Transfer / GetBalance).
    pub transaction_id: Option<String>,
}

/// Summary of a single sub-account.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubAccount {
    /// Exchange-assigned sub-account identifier.
    pub id: String,

    /// Display name or label.
    pub name: String,

    /// Account status (e.g. "Normal", "Frozen").
    pub status: String,
}

// ═══════════════════════════════════════════════════════════════════════════════
// WITHDRAW REQUEST
// ═══════════════════════════════════════════════════════════════════════════════

/// Withdrawal request for `CustodialFundsV2::withdraw`.
///
/// 18/20 custodial exchanges support withdrawals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WithdrawRequest {
    /// Asset to withdraw (e.g. "BTC", "ETH").
    pub asset: Asset,

    /// Amount to withdraw.
    pub amount: Quantity,

    /// Destination on-chain address.
    pub address: String,

    /// Blockchain network (e.g. "ERC20", "TRC20", "BEP20").
    /// Required when an asset is available on multiple networks.
    pub network: Option<String>,

    /// Destination tag / memo — required for assets like XRP, XLM, EOS.
    pub tag: Option<String>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// FUNDS HISTORY FILTER
// ═══════════════════════════════════════════════════════════════════════════════

/// Filter for deposit / withdrawal history queries.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundsHistoryFilter {
    /// Optional asset filter. `None` = all assets.
    pub asset: Option<Asset>,

    /// Record type filter.
    pub record_type: FundsRecordType,

    /// Start of time range (Unix ms). `None` = exchange default.
    pub start_time: Option<Timestamp>,

    /// End of time range (Unix ms). `None` = now.
    pub end_time: Option<Timestamp>,

    /// Maximum records to return. `None` = exchange default.
    pub limit: Option<u32>,
}

/// Whether to query deposits, withdrawals, or both.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FundsRecordType {
    /// Fetch deposit records only.
    Deposit,

    /// Fetch withdrawal records only.
    Withdrawal,

    /// Fetch both deposits and withdrawals (not all exchanges support combined).
    Both,
}

// ═══════════════════════════════════════════════════════════════════════════════
// EXCHANGE CREDENTIALS V2
// ═══════════════════════════════════════════════════════════════════════════════

/// Unified credential enum covering all 24-exchange authentication schemes.
///
/// Auth is an internal detail — connectors consume this enum, sign requests
/// internally, and never expose the signing process through public traits.
///
/// Auth method distribution across 24 exchanges:
/// - HMAC-SHA256: 12 exchanges
/// - HMAC+passphrase: 3 exchanges (OKX, KuCoin, Bitget)
/// - HMAC-SHA512: 1 exchange (Kraken)
/// - HMAC-SHA384: 1 exchange (Coinbase legacy)
/// - JWT-ES256: 1 exchange (Coinbase Advanced Trade)
/// - JWT-HMAC: 1 exchange (Paradex)
/// - OAuth2: 1 exchange (Upstox, some India brokers)
/// - Ethereum ECDSA: 2 exchanges (HyperLiquid, GMX)
/// - Solana Ed25519: 1 exchange (Jupiter, Raydium)
/// - STARK key: 2 exchanges (Lighter, Paradex)
/// - Cosmos wallet: 1 exchange (dYdX v4)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExchangeCredentials {
    /// HMAC-SHA256 with API key + secret.
    ///
    /// 12/24: Binance, Bybit, GateIO, Bitfinex, Bitstamp, Gemini, MEXC,
    /// HTX, BingX, Phemex, CryptoCom, Upbit.
    HmacSha256 {
        /// API key provided by the exchange.
        api_key: String,
        /// Secret key used for HMAC signing.
        api_secret: String,
    },

    /// HMAC-SHA256 with API key + secret + passphrase.
    ///
    /// 3/24: OKX, KuCoin, Bitget.
    HmacWithPassphrase {
        /// API key provided by the exchange.
        api_key: String,
        /// Secret key used for HMAC signing.
        api_secret: String,
        /// Additional passphrase set at key creation time.
        passphrase: String,
    },

    /// HMAC-SHA512 — Kraken's authentication scheme.
    ///
    /// 1/24: Kraken.
    HmacSha512 {
        /// API key provided by the exchange.
        api_key: String,
        /// Secret key used for HMAC-SHA512 signing (base64-encoded).
        api_secret: String,
    },

    /// HMAC-SHA384 — used by Coinbase legacy REST API (HMAC variant).
    ///
    /// 1/24: Deribit (also uses HMAC-SHA256 variant depending on endpoint).
    HmacSha384 {
        /// API key provided by the exchange.
        api_key: String,
        /// Secret key used for HMAC-SHA384 signing.
        api_secret: String,
    },

    /// JWT signed with EC P-256 private key (ES256).
    ///
    /// 1/24: Coinbase Advanced Trade API.
    JwtEs256 {
        /// API key name (used as JWT `kid` header).
        api_key: String,
        /// PEM-encoded EC private key.
        private_key_pem: String,
    },

    /// JWT signed with HMAC-SHA256 secret.
    ///
    /// 1/24: Paradex (uses JWT + StarkKey hybrid).
    JwtHmac {
        /// API key or JWT issuer identifier.
        api_key: String,
        /// Secret used for HMAC JWT signing.
        secret: String,
    },

    /// OAuth 2.0 bearer token flow.
    ///
    /// 1/24: Upstox (Indian broker), some Angel One endpoints.
    OAuth2 {
        /// OAuth access token (short-lived, must be refreshed).
        access_token: String,
        /// Optional refresh token for token renewal.
        refresh_token: Option<String>,
    },

    /// Ethereum ECDSA wallet signing.
    ///
    /// 2/24: HyperLiquid (EIP-712), GMX (EIP-712).
    EthereumWallet {
        /// Private key as a 0x-prefixed hex string.
        private_key_hex: String,
        /// Optional wallet address (derived from key if not provided).
        address: Option<String>,
    },

    /// Solana Ed25519 keypair signing.
    ///
    /// 1/24: Jupiter, Raydium (both Solana-native).
    SolanaKeypair {
        /// Base58-encoded Solana private key (64-byte keypair).
        private_key_b58: String,
    },

    /// StarkEx / StarkNet STARK key.
    ///
    /// 2/24: Lighter (Starknet), Paradex (Starknet).
    StarkKey {
        /// StarkKey private key as a hex string.
        stark_private_key: String,
        /// Ethereum address used to derive / register the StarkKey (optional).
        ethereum_address: Option<String>,
    },

    /// Cosmos SDK wallet (Tendermint signature).
    ///
    /// 1/24: dYdX v4 (Cosmos-based).
    CosmosWallet {
        /// Mnemonic phrase for HD wallet derivation.
        mnemonic: String,
        /// Optional HD derivation path (default: m/44'/118'/0'/0/0).
        derivation_path: Option<String>,
    },
}
