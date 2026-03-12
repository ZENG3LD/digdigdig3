# Thin-Trait + Fat-Enum Architecture for digdigdig3
# Multi-Exchange Connector Library — Complete Design Document

Generated: 2026-03-12
Based on: TRADING_CAPABILITY_MATRIX.md + TRAIT_DESIGN_ANALYSIS.md + matrix_batch1/2/3.md (24 exchanges)

---

## Core Design Philosophy

### The Two Principles

**1. Thin traits.** Each trait has the fewest methods that are semantically distinct operations.
There is no `market_order` AND `limit_order` — there is one `place_order` whose behavior
is determined by the `OrderType` variant in the fat enum. A connector either supports a variant
or it returns `UnsupportedOperation`. That decision is made at runtime by matching the enum,
not at compile time by the method signature.

**2. Fat enums.** All complexity lives in enum variants with fields. An enum definition IS the
capability documentation. Reading `OrderType` shows the full universe of order types across all
24 exchanges. You do not need a capability matrix — the enum IS the matrix, encoded in types.

### The Strict Non-Composition Rule

> Connectors MUST NEVER compose base methods to simulate missing features.
> If an exchange has no native endpoint for a capability, the connector returns
> `ExchangeError::UnsupportedOperation`. All composition belongs in higher layers.

This is not a style preference. It is a correctness requirement. Silent composition:
- Destroys atomicity guarantees (a real batch cancel is atomic; a loop is not)
- Corrupts rate-limit semantics (one batch request vs N individual requests)
- Deceives callers about what they are paying for in latency and cost

### Auth as Internal Implementation Detail

Auth is NOT a public trait. Each connector struct is responsible for its own signing.
The connector manager receives an `ExchangeCredentials` enum and passes it to the connector
that knows how to use it. There is no `sign_request` trait method — signing is an internal
detail of each connector's HTTP/gRPC/on-chain call path.

The only public auth surface is:
- `ExchangeCredentials` enum — what credentials look like from the outside
- `Authenticated` marker trait — so the connector manager can accept "this connector has creds"

---

## Part 1: Credential Enums

### ExchangeCredentials

```rust
/// All credential variants across 24 exchanges.
/// Each connector accepts exactly the variant(s) it requires.
/// Passing the wrong variant yields ExchangeError::InvalidCredentials.
#[derive(Clone)]
pub enum ExchangeCredentials {
    /// HMAC-SHA256 (standard CEX pattern).
    /// Binance, Bybit, MEXC, BingX, Bitstamp, Phemex, Crypto.com, Gemini
    HmacSha256 {
        api_key: String,
        api_secret: String,
    },

    /// HMAC-SHA256 with passphrase (3-factor auth).
    /// OKX, KuCoin, Bitget
    HmacSha256WithPassphrase {
        api_key: String,
        api_secret: String,
        passphrase: String,
    },

    /// HMAC-SHA512.
    /// Gate.io (payload includes body hash + path + timestamp)
    HmacSha512 {
        api_key: String,
        api_secret: String,
    },

    /// HMAC-SHA384.
    /// Bitfinex (signs path + nonce + body)
    HmacSha384 {
        api_key: String,
        api_secret: String,
    },

    /// Kraken Spot uses SHA512, Futures uses SHA256.
    /// Kraken is the only exchange that uses different algorithms per product.
    KrakenDual {
        api_key: String,
        api_secret: String,
        /// "spot" or "futures" — selects the signing algorithm
        product: String,
    },

    /// JWT signed with ECDSA P-256 (ES256).
    /// Coinbase Advanced Trade API (CDP private key)
    JwtEs256 {
        key_id: String,
        /// PEM-encoded EC private key
        private_key_pem: String,
    },

    /// JWT with HMAC-SHA256 signed payload.
    /// Upbit — JWT payload is built from API key + query hash, then signed
    JwtHmacSha256 {
        api_key: String,
        api_secret: String,
    },

    /// OAuth2 client_credentials flow.
    /// Deribit — calls public/auth to get access_token, refreshes when expired
    OAuth2ClientCredentials {
        client_id: String,
        client_secret: String,
    },

    /// Ethereum ECDSA wallet signing (secp256k1).
    /// HyperLiquid (per-action signing), GMX (on-chain contract calls only)
    EthereumWallet {
        /// 32-byte hex private key, with or without 0x prefix
        private_key_hex: String,
    },

    /// Solana keypair (Ed25519).
    /// Jupiter — signs Solana transactions; optional REST API key for read endpoints
    SolanaKeypair {
        /// 64-byte base58-encoded keypair, or 32-byte base58 seed
        private_key_base58: String,
        /// Optional x-api-key header for REST read endpoints
        api_key: Option<String>,
    },

    /// STARK elliptic curve key.
    /// Lighter (ZK-rollup), Paradex (Starknet L2)
    StarkKey {
        /// STARK field element as hex string
        private_key: String,
        /// Derived public key (optional — can be computed from private)
        public_key: Option<String>,
        /// Ethereum address for on-chain verification (Paradex requires this)
        ethereum_address: Option<String>,
    },

    /// Cosmos SDK wallet.
    /// dYdX V4 — broadcasts MsgPlaceOrder/MsgCancelOrder via gRPC
    CosmosWallet {
        mnemonic: String,
        /// Chain ID, e.g. "dydx-mainnet-1"
        chain_id: String,
        /// Bech32 prefix, e.g. "dydx"
        account_prefix: String,
    },
}
```

### Authenticated Marker Trait

```rust
/// Marker trait. A connector that implements this carries credentials
/// and can make authenticated API calls.
/// The connector_manager uses this bound when building authenticated connectors.
pub trait Authenticated: Send + Sync {
    /// Returns the credential type this connector uses.
    /// Used by the connector manager to validate credential compatibility.
    fn credential_type(&self) -> CredentialType;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CredentialType {
    HmacSha256,
    HmacSha256WithPassphrase,
    HmacSha512,
    HmacSha384,
    KrakenDual,
    JwtEs256,
    JwtHmacSha256,
    OAuth2ClientCredentials,
    EthereumWallet,
    SolanaKeypair,
    StarkKey,
    CosmosWallet,
}
```

---

## Part 2: Core Type Enums (Fat Enums)

### OrderType — The Full Universe

```rust
/// ALL order type variants that exist across the 24-exchange universe.
/// A connector matches on the variants it supports and returns
/// ExchangeError::UnsupportedOperation for any variant it cannot natively execute.
///
/// Support counts (from capability matrix):
///   Market         — 24/24  universal
///   Limit          — 24/24  universal
///   StopMarket     — 19/24  common
///   StopLimit      — 19/24  common
///   TakeProfit     — 19/24  common
///   TakeProfitLimit— ~15/24 common
///   StopLoss       — 19/24  common
///   StopLossLimit  — ~15/24 common
///   TrailingStop   — 10/24  specialized
///   OCO            —  7/24  specialized
///   Bracket        —  9/24  specialized (most are partial)
///   Iceberg        —  8/24  specialized
///   Twap           —  7/24  specialized
///   BestPrice      —  1/24  Upbit-specific
#[derive(Debug, Clone)]
pub enum OrderType {
    /// Execute immediately at the best available market price.
    /// 24/24 exchanges. Required field: quantity.
    /// Note: Upbit market BUY uses quote_amount (cost), not base quantity.
    Market {
        /// Base asset quantity to trade. For Upbit market buy, use quote_amount instead.
        quantity: Option<Decimal>,
        /// Quote asset amount to spend (Upbit market buy, Coinbase market buy).
        quote_amount: Option<Decimal>,
    },

    /// Execute at the specified price or better.
    /// 24/24 exchanges.
    Limit {
        quantity: Decimal,
        price: Decimal,
        time_in_force: TimeInForce,
        /// Some exchanges support post-only as a TIF variant; others as a flag.
        post_only: bool,
        /// Only execute against existing liquidity (reduce position). Futures only.
        reduce_only: bool,
    },

    /// Trigger a market order when the price crosses the stop_price.
    /// 19/24 exchanges. Missing: Upbit, Bitstamp, Gemini, Jupiter, GMX (native).
    StopMarket {
        quantity: Decimal,
        /// Trigger price. When the market reaches this price, a market order fires.
        stop_price: Decimal,
        reduce_only: bool,
    },

    /// Trigger a limit order when the price crosses the stop_price.
    /// 19/24 exchanges.
    StopLimit {
        quantity: Decimal,
        /// Price that triggers the order.
        stop_price: Decimal,
        /// Price of the limit order that is placed after trigger.
        limit_price: Decimal,
        time_in_force: TimeInForce,
        reduce_only: bool,
    },

    /// Take-profit order: trigger when price moves in profit direction.
    /// 19/24 exchanges.
    TakeProfit {
        quantity: Decimal,
        /// Price at which the TP triggers.
        trigger_price: Decimal,
        /// If Some, a limit order is placed at this price after trigger.
        /// If None, a market order fires after trigger.
        order_price: Option<Decimal>,
        reduce_only: bool,
    },

    /// Stop-loss order: trigger when price moves against position.
    /// 19/24 exchanges.
    StopLoss {
        quantity: Decimal,
        /// Price at which the SL triggers.
        trigger_price: Decimal,
        /// If Some, a limit order is placed at this price after trigger.
        /// If None, a market order fires after trigger.
        order_price: Option<Decimal>,
        reduce_only: bool,
    },

    /// Trailing stop: stop_price adjusts dynamically as price moves favorably.
    /// 10/24 exchanges: Binance(F), Bybit, OKX, Kraken, Bitfinex, HTX, Bitget(F), BingX(F), Phemex(F), Deribit.
    TrailingStop {
        quantity: Decimal,
        /// Fixed price offset from the trailing reference price.
        /// Use either offset_amount or offset_percent, not both.
        offset_amount: Option<Decimal>,
        /// Percentage offset (0.01 = 1%).
        offset_percent: Option<Decimal>,
        /// Activation price: trailing only begins after market reaches this price.
        /// Optional — not all exchanges support activation price.
        activation_price: Option<Decimal>,
        reduce_only: bool,
    },

    /// One-Cancels-Other: two linked orders; when one fills, the other is cancelled.
    /// 7/24 exchanges: OKX, Bitfinex, Crypto.com (Advanced), Deribit, Lighter(partial), HyperLiquid(partial).
    Oco {
        /// The limit/stop order on one side.
        leg1: OcoLeg,
        /// The limit/stop order on the other side.
        leg2: OcoLeg,
    },

    /// Bracket (OTOCO): entry order + simultaneous TP and SL.
    /// 9/24 exchanges (most partial): Coinbase, Phemex(F), Deribit(OTOCO), Crypto.com(OTOCO), Bitget(F OTOCO), Kraken(OTO, partial), Lighter(partial).
    Bracket {
        quantity: Decimal,
        /// Entry order parameters.
        entry: BracketEntry,
        /// Take-profit leg: triggers when price moves in profit direction.
        take_profit: BracketLeg,
        /// Stop-loss leg: triggers when price moves against position.
        stop_loss: BracketLeg,
    },

    /// Iceberg: order with a visible portion; hidden quantity is revealed as visible fills.
    /// 8/24 exchanges: Binance(S), OKX, KuCoin, Kraken(S), Gate.io, Bitfinex(HIDDEN), Phemex(S), Deribit.
    Iceberg {
        quantity: Decimal,
        price: Decimal,
        /// Quantity visible in the order book at any time.
        visible_quantity: Decimal,
        time_in_force: TimeInForce,
    },

    /// TWAP: time-weighted average price algorithm that slices order over time.
    /// 7/24 exchanges: OKX, Coinbase(native TWAP), BingX, HyperLiquid, Lighter, Paradex, dYdX V4.
    Twap {
        quantity: Decimal,
        /// Total duration over which slices execute, in seconds.
        duration_secs: u64,
        /// Number of equal-sized slices. If None, the exchange decides.
        slices: Option<u32>,
        /// Optional price limit; if the market moves beyond this, execution pauses.
        price_limit: Option<Decimal>,
    },

    /// Best-price order: execute at the best available price (Upbit-specific).
    /// 1/24 exchanges: Upbit only.
    BestPrice {
        quantity: Decimal,
        time_in_force: TimeInForce,
    },
}

/// A single leg in an OCO order.
#[derive(Debug, Clone)]
pub struct OcoLeg {
    pub order_kind: OcoLegKind,
    pub price: Decimal,
    /// Optional trigger price for stop legs.
    pub trigger_price: Option<Decimal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OcoLegKind {
    Limit,
    StopLimit,
    StopMarket,
}

/// Entry parameters for a bracket order.
#[derive(Debug, Clone)]
pub enum BracketEntry {
    Market,
    Limit { price: Decimal },
    StopMarket { trigger_price: Decimal },
    StopLimit { trigger_price: Decimal, limit_price: Decimal },
}

/// A TP or SL leg in a bracket order.
#[derive(Debug, Clone)]
pub struct BracketLeg {
    pub trigger_price: Decimal,
    /// If Some, a limit order fires at this price. If None, a market order fires.
    pub limit_price: Option<Decimal>,
}
```

### TimeInForce

```rust
/// Time-in-force variants across all 24 exchanges.
///
/// Support counts:
///   GoodTilCancelled — 23/24 (Upbit implicit only)
///   ImmediateOrCancel — 21/24
///   FillOrKill        — 19/24
///   PostOnly          — 22/24
///   GoodTilDate       — 11/24 (7Y + 4P)
///   GoodTilBlock      —  1/24 (dYdX V4 only)
#[derive(Debug, Clone)]
pub enum TimeInForce {
    /// Remain open until manually cancelled. Universal.
    GoodTilCancelled,

    /// Fill immediately; cancel any unfilled portion. 21/24 exchanges.
    ImmediateOrCancel,

    /// Fill entire quantity immediately or cancel the whole order. 19/24 exchanges.
    FillOrKill,

    /// Only execute as a maker order; cancel if it would take. 22/24 exchanges.
    /// Note: HyperLiquid calls this ALO (Add Liquidity Only).
    PostOnly,

    /// Cancel after a specific date/time. 11/24 exchanges.
    /// KuCoin uses GTT (Good-Till-Time) with a `cancelAfter` seconds field.
    /// Bitfinex accepts a datetime string.
    /// dYdX V4 uses GTBT (Good-Til-Block-Time).
    GoodTilDate {
        /// UTC timestamp in milliseconds.
        expires_at_ms: u64,
    },

    /// Cancel after a specific block number (dYdX V4 only).
    GoodTilBlock {
        block_height: u64,
    },
}
```

### OrderSide

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderSide {
    Buy,
    Sell,
}
```

### AccountType

```rust
/// Account sub-type within an exchange.
/// Connectors return UnsupportedOperation for account types they don't offer.
///
/// Most exchanges have Spot. Futures/Perpetual/Derivatives are applicable to 22/24.
/// Unified account (OKX, Bybit) spans all products in a single margin pool.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AccountType {
    /// Standard spot trading account.
    Spot,
    /// USD-margined perpetual futures.
    PerpetualUsd,
    /// Coin-margined perpetual futures (inverse contracts).
    PerpetualCoin,
    /// Traditional expiry futures.
    Futures,
    /// Options trading.
    Options,
    /// Spot margin lending account (Binance Margin, Kraken margin, Bitfinex exchange margin).
    Margin,
    /// Unified account spanning all product types (OKX Unified, Bybit Unified).
    Unified,
    /// Portfolio margin account with cross-product margin relief (Binance Portfolio, Deribit PM).
    Portfolio,
    /// Funding / Earn / Savings wallet (Binance earn, KuCoin pool).
    Funding,
    /// Institution / brokerage sub-account.
    Institutional,
}
```

### CancelScope

```rust
/// Scope of a cancel_all operation.
/// Fat enum: connectors match on the variants they support.
#[derive(Debug, Clone)]
pub enum CancelScope {
    /// Cancel all open orders regardless of symbol or account type.
    All,
    /// Cancel all open orders for a specific trading pair.
    BySymbol { symbol: String },
    /// Cancel all open orders of a specific type (limit, stop, etc.).
    ByOrderType { order_type_kind: OrderTypeKind },
    /// Cancel all open orders by client order ID prefix/group tag.
    ByClientGroup { group_id: String },
    /// Cancel all open orders for a specific account type.
    ByAccountType { account_type: AccountType },
}

/// Discriminant for order type, used in cancel/filter operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderTypeKind {
    Market,
    Limit,
    StopMarket,
    StopLimit,
    TakeProfit,
    StopLoss,
    TrailingStop,
    Oco,
    Bracket,
    Iceberg,
    Twap,
}
```

### AmendFields

```rust
/// Which fields to modify on an existing order.
/// 18/24 exchanges support some form of order amendment.
/// A connector returns UnsupportedOperation for fields it cannot natively amend.
#[derive(Debug, Clone, Default)]
pub struct AmendFields {
    /// New price. None = do not change.
    pub price: Option<Decimal>,
    /// New quantity. None = do not change.
    pub quantity: Option<Decimal>,
    /// New trigger price (for stop/TP/SL orders). None = do not change.
    pub trigger_price: Option<Decimal>,
    /// New take-profit price (Bybit supports inline TP/SL amend).
    pub take_profit_price: Option<Decimal>,
    /// New stop-loss price.
    pub stop_loss_price: Option<Decimal>,
}
```

### OrderHistoryFilter

```rust
/// Filter for get_order_history. 24/24 exchanges support order history.
/// All fields are optional; unrecognized filter fields are silently ignored by the connector.
#[derive(Debug, Clone, Default)]
pub struct OrderHistoryFilter {
    /// Restrict to a specific trading pair.
    pub symbol: Option<String>,
    /// Start of time range (Unix ms). If None, exchange uses its default lookback.
    pub start_ms: Option<u64>,
    /// End of time range (Unix ms). If None, up to now.
    pub end_ms: Option<u64>,
    /// Maximum number of records to return. Connector clamps to exchange's own max.
    pub limit: Option<u32>,
    /// Pagination cursor from the previous response.
    pub cursor: Option<String>,
    /// Filter by order status.
    pub status: Option<OrderStatusFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatusFilter {
    Filled,
    Cancelled,
    Expired,
    All,
}
```

### TransferRequest

```rust
/// Internal transfer between account types within one exchange.
/// 17/24 exchanges support internal transfers.
/// DEXes (Jupiter, GMX) return UnsupportedOperation — bridge-based only.
#[derive(Debug, Clone)]
pub struct TransferRequest {
    pub asset: String,
    pub amount: Decimal,
    pub from: AccountType,
    pub to: AccountType,
    /// Optional: destination sub-account ID (for sub-account transfers on exchanges
    /// that merge internal transfers and sub-account transfers into one endpoint).
    pub to_sub_account: Option<String>,
    /// Optional client-assigned transfer ID for idempotency.
    pub client_transfer_id: Option<String>,
}
```

### MarginMode

```rust
/// Position margin mode. 13/22 futures exchanges support switching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarginMode {
    /// Single shared margin pool across all positions.
    Cross,
    /// Each position has its own isolated margin; loss capped at allocated margin.
    Isolated,
    /// Portfolio margin: advanced cross-product netting (Binance Portfolio, Deribit PM).
    Portfolio,
}
```

### PositionModification

```rust
/// Variant-based position modification operations.
/// Connectors return UnsupportedOperation for variants they lack native endpoints for.
#[derive(Debug, Clone)]
pub enum PositionModification {
    /// Set account-level or symbol-level leverage multiplier. 15Y+2P of 22 exchanges.
    SetLeverage {
        symbol: String,
        leverage: u32,
        account_type: AccountType,
    },

    /// Switch cross / isolated / portfolio margin mode. 13Y+4P of 22 exchanges.
    SetMarginMode {
        symbol: String,
        mode: MarginMode,
        account_type: AccountType,
    },

    /// Add margin to an isolated position. 11Y+1P of 22 exchanges.
    AddMargin {
        symbol: String,
        amount: Decimal,
        account_type: AccountType,
    },

    /// Remove margin from an isolated position.
    RemoveMargin {
        symbol: String,
        amount: Decimal,
        account_type: AccountType,
    },

    /// Close an entire position at market price using a dedicated close endpoint.
    /// 5Y explicitly (OKX, GMX, BingX, Bitget, Crypto.com); many others via reduceOnly market.
    ClosePosition {
        symbol: String,
        account_type: AccountType,
    },
}
```

### WithdrawRequest

```rust
/// Withdrawal request for custodial exchanges. 14/24 exchanges support withdrawals.
/// DEXes return UnsupportedOperation — they are non-custodial.
#[derive(Debug, Clone)]
pub struct WithdrawRequest {
    pub asset: String,
    pub amount: Decimal,
    /// Destination blockchain address.
    pub destination_address: String,
    /// Blockchain network/chain (e.g., "ETH", "BSC", "TRON").
    pub network: String,
    /// Optional memo/tag for coins that require it (XRP, EOS, ATOM).
    pub memo: Option<String>,
    /// Optional client-assigned ID for idempotency.
    pub client_id: Option<String>,
}
```

---

## Part 3: Trait Definitions

### Identity (foundation — every connector)

```rust
use std::borrow::Cow;

/// Every connector must implement ExchangeIdentity.
/// This is the only required trait. All others are optional.
pub trait ExchangeIdentity: Send + Sync + 'static {
    fn exchange_id(&self) -> ExchangeId;
    fn name(&self) -> &'static str;
    /// Account types this connector instance is configured to access.
    fn account_types(&self) -> &[AccountType];
    /// Exchange category for routing decisions.
    fn exchange_kind(&self) -> ExchangeKind;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExchangeId {
    Binance, Bybit, Okx, KuCoin, Kraken, Coinbase, GateIo, Bitfinex,
    Bitstamp, Mexc, Htx, Bitget, Gemini, BingX, Phemex, CryptoCom,
    Upbit, Deribit, HyperLiquid, Lighter, Jupiter, Gmx, Paradex, DydxV4,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExchangeKind {
    /// Traditional centralized exchange with custodial funds.
    CentralizedExchange,
    /// Centralized layer-1 chain with non-custodial mechanics (HyperLiquid).
    L1Chain,
    /// ZK-rollup DEX on Ethereum.
    ZkRollupDex,
    /// DEX on Solana.
    SolanaDex,
    /// DEX on Arbitrum/Avalanche EVM.
    EvmDex,
    /// DEX on Starknet L2.
    StarknetDex,
    /// DEX on Cosmos app-chain.
    CosmosDex,
}
```

### Trading (24/24 — universal core)

```rust
/// Core trading trait. ALL 24 exchanges implement this.
///
/// The `order_type` field in OrderRequest carries ALL complexity.
/// A connector matches on the variants it supports and returns
/// ExchangeError::UnsupportedOperation for any variant it cannot natively execute.
/// The connector does NOT compose unsupported types from primitives.
pub trait Trading: ExchangeIdentity {
    /// Place a single order of any supported type.
    /// The connector matches order_type and returns UnsupportedOperation for
    /// any variant it lacks a native endpoint for.
    async fn place_order(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType,
        account_type: AccountType,
        client_order_id: Option<&str>,
    ) -> ExchangeResult<Order>;

    /// Cancel a single order by exchange-assigned order ID.
    /// 24/24 exchanges.
    async fn cancel_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Retrieve a single order by its exchange-assigned ID.
    /// 23Y + 1P (Lighter returns via list filtering) / 24. Effectively universal.
    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    /// Get all currently open orders, optionally filtered by symbol.
    /// 24/24 exchanges.
    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;

    /// Get historical/closed orders with optional filter.
    /// 24/24 exchanges. THIS WAS MISSING FROM THE PREVIOUS DESIGN.
    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;

    // -------------------------------------------------------------------------
    // Capability discovery — optional but recommended
    // -------------------------------------------------------------------------

    /// Returns the set of OrderType variants this connector supports natively.
    /// Default: empty (caller must probe or consult documentation).
    /// Override to enable static capability checking without runtime probe.
    fn supported_order_types(&self) -> Vec<OrderTypeKind> {
        vec![OrderTypeKind::Market, OrderTypeKind::Limit]
    }

    /// Returns the set of TimeInForce variants this connector supports.
    fn supported_time_in_force(&self) -> Vec<TimeInForce> {
        vec![
            TimeInForce::GoodTilCancelled,
            TimeInForce::ImmediateOrCancel,
        ]
    }
}
```

### Account (24/24 — universal core)

```rust
/// Core account information trait. ALL 24 exchanges implement this.
pub trait Account: ExchangeIdentity {
    /// Get balances for all assets, or a specific asset.
    /// 23Y + 1P (GMX partial — ERC-20 balanceOf) / 24. Effectively universal.
    async fn get_balance(
        &self,
        asset: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Balance>>;

    /// Get basic account information (status, tier, account type).
    /// AccountInfo contains only fields universally available across exchanges.
    async fn get_account_info(
        &self,
        account_type: AccountType,
    ) -> ExchangeResult<AccountInfo>;

    /// Get trading fee rates (maker/taker).
    /// 17Y + 6P = 23/24 non-N. The only missing exchange is GMX (fees embedded in on-chain calc).
    /// THIS WAS MISSING FROM THE PREVIOUS DESIGN.
    async fn get_fees(
        &self,
        symbol: Option<&str>,
    ) -> ExchangeResult<FeeInfo>;
}
```

### CancelAll (22/24 — high coverage extension)

```rust
/// Cancel-all-orders capability.
/// 20Y + 2P = 22/24. Missing: GMX (on-chain, no cancel-all primitive), dYdX V4 (requires explicit IDs).
///
/// The `scope` enum handles all cancel-all variants:
///   - Cancel everything: CancelScope::All
///   - Cancel by symbol: CancelScope::BySymbol
///   - Cancel by order type: CancelScope::ByOrderType
///   - Cancel by group tag: CancelScope::ByClientGroup
///
/// Connectors return UnsupportedOperation for CancelScope variants they lack.
pub trait CancelAll: Trading {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResult>;
}

#[derive(Debug, Clone)]
pub struct CancelAllResult {
    /// Number of orders successfully cancelled.
    pub cancelled_count: u64,
    /// Order IDs that were cancelled, if the exchange returns them.
    pub cancelled_ids: Vec<String>,
    /// Errors for individual orders that could not be cancelled (partial failures).
    pub errors: Vec<PartialCancelError>,
}

#[derive(Debug, Clone)]
pub struct PartialCancelError {
    pub order_id: String,
    pub reason: String,
}
```

### AmendOrder (18/24 — high coverage extension)

```rust
/// Modify an existing order's price and/or quantity.
/// 13Y + 5P = 18/24.
///
/// Not supported: Upbit (cancel_and_new only), MEXC (spot), HTX (spot),
/// Bitfinex (cancel+replace semantics), KuCoin spot (HF: cancel+recreate internally),
/// dYdX V4, Jupiter.
///
/// The AmendFields struct carries all modifiable fields.
/// Connectors return UnsupportedOperation for fields they cannot natively amend
/// (e.g., KuCoin HF can amend price but not quantity).
pub trait AmendOrder: Trading {
    async fn amend_order(
        &self,
        symbol: &str,
        order_id: &str,
        fields: AmendFields,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;
}
```

### BatchOrders (17/24 — high coverage extension)

```rust
/// Native batch order operations.
/// 14Y + 3P = 17/24. NO DEFAULT IMPLEMENTATIONS — every method is a real endpoint.
///
/// Exchanges with native batch create: Binance(F:5), Bybit(20/10), OKX(20),
/// KuCoin(S:5), Gate.io(S:10), Bitfinex(75 mixed), MEXC(S:20/F:50), HTX(10),
/// Bitget(50), BingX, Crypto.com(10), HyperLiquid, Lighter(50), Paradex(10).
///
/// NOT implemented by: Bitstamp, Coinbase, Gemini, Phemex, Upbit, Deribit,
/// Jupiter, GMX (multicall != batch-place API), dYdX V4 (partial Cosmos batching).
///
/// If an exchange has no batch-place endpoint, it does NOT implement this trait.
/// The rule: never simulate batch with a loop.
pub trait BatchOrders: Trading {
    /// Place multiple orders in a single native request.
    /// Returns results in the same order as input; failed items carry the error inline.
    async fn place_orders_batch(
        &self,
        orders: Vec<BatchOrderRequest>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<BatchOrderResult>>;

    /// Cancel multiple orders in a single native request.
    async fn cancel_orders_batch(
        &self,
        cancels: Vec<BatchCancelRequest>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<BatchCancelResult>>;

    /// Returns the maximum batch size for place operations on this exchange.
    /// Callers MUST split larger batches into chunks of this size.
    fn max_batch_place_size(&self) -> usize;

    /// Returns the maximum batch size for cancel operations.
    fn max_batch_cancel_size(&self) -> usize;
}

#[derive(Debug, Clone)]
pub struct BatchOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub client_order_id: Option<String>,
}

#[derive(Debug, Clone)]
pub enum BatchOrderResult {
    Ok(Order),
    Err { client_order_id: Option<String>, error: String },
}

#[derive(Debug, Clone)]
pub struct BatchCancelRequest {
    pub symbol: String,
    pub order_id: String,
}

#[derive(Debug, Clone)]
pub enum BatchCancelResult {
    Ok(Order),
    Err { order_id: String, error: String },
}
```

### Positions (22/24 futures-capable — extension)

```rust
/// Futures/perpetuals position management.
/// Implemented by 22/24 exchanges. Bitstamp and Upbit are spot-only — they do not implement this.
///
/// All position modification operations are routed through the PositionModification fat enum.
/// A connector matches on the variants it supports and returns UnsupportedOperation for others.
pub trait Positions: ExchangeIdentity {
    /// Get current open positions. 18Y + 2P / 22 futures-capable.
    async fn get_positions(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Position>>;

    /// Get the current funding rate for a perpetual contract. 17Y / 22.
    async fn get_funding_rate(
        &self,
        symbol: &str,
    ) -> ExchangeResult<FundingRate>;

    /// Modify a position attribute (leverage, margin mode, margin amount, close).
    /// The PositionModification enum routes to the appropriate exchange endpoint.
    /// Connector returns UnsupportedOperation for variants lacking native endpoints.
    async fn modify_position(
        &self,
        modification: PositionModification,
    ) -> ExchangeResult<PositionModificationResult>;

    // -------------------------------------------------------------------------
    // Capability discovery
    // -------------------------------------------------------------------------

    /// Which PositionModification variants this connector supports natively.
    fn supported_position_modifications(&self) -> Vec<PositionModificationKind> {
        vec![]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PositionModificationKind {
    SetLeverage,
    SetMarginMode,
    AddMargin,
    RemoveMargin,
    ClosePosition,
}

#[derive(Debug, Clone)]
pub enum PositionModificationResult {
    LeverageSet { symbol: String, leverage: u32 },
    MarginModeSet { symbol: String, mode: MarginMode },
    MarginAdded { symbol: String, added: Decimal },
    MarginRemoved { symbol: String, removed: Decimal },
    PositionClosed { symbol: String, pnl: Option<Decimal> },
}
```

### AccountTransfers (17/24 — extension)

```rust
/// Internal transfer between account sub-types within one exchange.
/// 16Y + 1P / 20 applicable (excludes non-custodial DEXes).
///
/// NOT implemented by: Jupiter (N/A), GMX (N/A), Paradex (N), Upbit (N — single account type),
/// Coinbase (limited — only portfolio moves, not full transfer freedom).
pub trait AccountTransfers: Account {
    async fn transfer(
        &self,
        req: TransferRequest,
    ) -> ExchangeResult<TransferResult>;

    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<Transfer>>;
}

#[derive(Debug, Clone, Default)]
pub struct TransferHistoryFilter {
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub asset: Option<String>,
}

#[derive(Debug, Clone)]
pub struct TransferResult {
    pub transfer_id: String,
    pub status: TransferStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransferStatus {
    Success,
    Pending,
    Failed,
}

#[derive(Debug, Clone)]
pub struct Transfer {
    pub transfer_id: String,
    pub from: AccountType,
    pub to: AccountType,
    pub asset: String,
    pub amount: Decimal,
    pub timestamp_ms: u64,
    pub status: TransferStatus,
}
```

### CustodialFunds (18/24 custodial exchanges — extension)

```rust
/// Deposit addresses, withdrawals, and deposit/withdraw history.
/// ~18 of 24 exchanges support this. Non-custodial DEXes (Jupiter, GMX, HyperLiquid,
/// Paradex, dYdX) do NOT implement this trait — they are bridge-based.
///
/// The WithdrawRequest fat struct carries all exchange-specific withdrawal fields.
/// Connectors use only the fields they support and ignore the rest.
pub trait CustodialFunds: Account {
    /// Get a deposit address for the given asset and network.
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress>;

    /// Request a withdrawal to an external address.
    async fn withdraw(
        &self,
        req: WithdrawRequest,
    ) -> ExchangeResult<WithdrawResult>;

    /// Get deposit and withdrawal history with optional filters.
    async fn get_deposit_withdraw_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>>;
}

#[derive(Debug, Clone)]
pub struct DepositAddress {
    pub asset: String,
    pub network: String,
    pub address: String,
    pub memo: Option<String>,
    pub address_tag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WithdrawResult {
    pub withdrawal_id: String,
    pub status: WithdrawStatus,
    pub fee: Option<Decimal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WithdrawStatus {
    Submitted,
    Processing,
    Success,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Default)]
pub struct FundsHistoryFilter {
    pub asset: Option<String>,
    pub start_ms: Option<u64>,
    pub end_ms: Option<u64>,
    pub limit: Option<u32>,
    pub cursor: Option<String>,
    pub record_type: Option<FundsRecordType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FundsRecordType {
    Deposit,
    Withdrawal,
    Both,
}

#[derive(Debug, Clone)]
pub struct FundsRecord {
    pub id: String,
    pub record_type: FundsRecordType,
    pub asset: String,
    pub amount: Decimal,
    pub fee: Option<Decimal>,
    pub address: Option<String>,
    pub network: Option<String>,
    pub status: String,
    pub timestamp_ms: u64,
    pub tx_hash: Option<String>,
}
```

### SubAccounts (12/24 non-DEX exchanges — extension)

```rust
/// Sub-account management. ~12/22 non-DEX exchanges support some form.
///
/// The SubAccountOperation fat enum routes all operations.
/// Connectors return UnsupportedOperation for operations they lack native endpoints for.
///
/// Support breakdown:
///   Create:   8Y + 4P / 22
///   List:     9Y / 22
///   Transfer: 15Y / 22
pub trait SubAccounts: Account {
    /// Execute a sub-account operation.
    /// The SubAccountOperation enum routes to the appropriate endpoint.
    async fn subaccount_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult>;
}

#[derive(Debug, Clone)]
pub enum SubAccountOperation {
    /// Create a new sub-account. 8Y+4P / 22.
    Create {
        label: String,
        /// Permissions to assign. Exchange-specific.
        permissions: Vec<String>,
    },
    /// List all sub-accounts. 9Y / 22.
    List,
    /// Transfer funds between master and sub-account. 15Y / 22.
    Transfer {
        asset: String,
        amount: Decimal,
        /// None = transfer from master to sub-account named below.
        from_sub_account: Option<String>,
        /// None = transfer to master.
        to_sub_account: Option<String>,
    },
    /// Get balances of a sub-account.
    GetBalance {
        sub_account_id: String,
        asset: Option<String>,
    },
}

#[derive(Debug, Clone)]
pub enum SubAccountResult {
    Created(SubAccount),
    Listed(Vec<SubAccount>),
    Transferred { transfer_id: String },
    Balance(Vec<Balance>),
}

#[derive(Debug, Clone)]
pub struct SubAccount {
    pub id: String,
    pub label: Option<String>,
    pub created_at_ms: Option<u64>,
    pub status: Option<String>,
}
```

### AdvancedOrders (selectively supported — extension)

```rust
/// Advanced order algorithms with structurally distinct call signatures.
/// A connector that implements this trait returns UnsupportedOperation for
/// methods its exchange does not natively support.
///
/// NOTE: Connectors should only implement this trait if they support at least
/// ONE of these advanced types natively. Implementing the full trait and
/// returning UnsupportedOperation for every method is allowed but discouraged
/// in favor of returning a capability list via `supported_advanced_types()`.
///
/// Support per method:
///   TrailingStop — 10/24
///   Oco          —  7/24
///   Bracket      —  9/24 (most partial/exchange-specific)
///   Iceberg      —  8/24
///   Twap         —  7/24
pub trait AdvancedOrders: Trading {
    /// Place a trailing stop order.
    /// Exchanges: Binance(F), Bybit, OKX, Kraken, Bitfinex, HTX, Bitget(F), BingX(F), Phemex(F), Deribit.
    async fn place_trailing_stop(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType, // Must be OrderType::TrailingStop
        account_type: AccountType,
        client_order_id: Option<&str>,
    ) -> ExchangeResult<Order>;

    /// Place an OCO (One-Cancels-Other) order pair.
    /// Exchanges: OKX, Bitfinex, Crypto.com (Advanced), Deribit, Lighter(partial), HyperLiquid(partial).
    async fn place_oco(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType, // Must be OrderType::Oco
        account_type: AccountType,
        client_order_id: Option<&str>,
    ) -> ExchangeResult<OcoOrderResult>;

    /// Place a bracket order (entry + TP + SL in one native call).
    /// Exchanges: Coinbase, Phemex(F), Deribit(OTOCO), Crypto.com(OTOCO), Bitget(F OTOCO), Kraken(OTO, partial).
    async fn place_bracket(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType, // Must be OrderType::Bracket
        account_type: AccountType,
        client_order_id: Option<&str>,
    ) -> ExchangeResult<BracketOrderResult>;

    /// Place an iceberg order with visible quantity.
    /// Exchanges: Binance(S), OKX, KuCoin, Kraken(S), Gate.io, Bitfinex(HIDDEN), Phemex(S), Deribit.
    async fn place_iceberg(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType, // Must be OrderType::Iceberg
        account_type: AccountType,
        client_order_id: Option<&str>,
    ) -> ExchangeResult<Order>;

    /// Place a TWAP algorithmic order.
    /// Exchanges: OKX, Coinbase(native), BingX, HyperLiquid, Lighter, Paradex, dYdX V4.
    async fn place_twap(
        &self,
        symbol: &str,
        side: OrderSide,
        order_type: OrderType, // Must be OrderType::Twap
        account_type: AccountType,
        client_order_id: Option<&str>,
    ) -> ExchangeResult<AlgoOrderResult>;

    /// Returns which advanced types this connector supports.
    fn supported_advanced_types(&self) -> Vec<OrderTypeKind> {
        vec![]
    }
}

#[derive(Debug, Clone)]
pub struct OcoOrderResult {
    pub order_list_id: String,
    pub leg1: Order,
    pub leg2: Order,
}

#[derive(Debug, Clone)]
pub struct BracketOrderResult {
    pub list_id: String,
    pub entry: Order,
    pub take_profit: Option<Order>,
    pub stop_loss: Option<Order>,
}

#[derive(Debug, Clone)]
pub struct AlgoOrderResult {
    pub algo_order_id: String,
    pub status: AlgoOrderStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlgoOrderStatus {
    Running,
    Paused,
    Completed,
    Cancelled,
    Failed,
}
```

---

## Part 4: Response Types (Common)

```rust
use rust_decimal::Decimal;

#[derive(Debug, Clone)]
pub struct Order {
    pub order_id: String,
    pub client_order_id: Option<String>,
    pub exchange_id: ExchangeId,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type_kind: OrderTypeKind,
    pub status: OrderStatus,
    pub quantity: Decimal,
    pub filled_quantity: Decimal,
    pub price: Option<Decimal>,
    pub avg_fill_price: Option<Decimal>,
    pub created_at_ms: u64,
    pub updated_at_ms: Option<u64>,
    pub account_type: AccountType,
    /// Exchange-specific fields that do not map to the common model.
    pub raw: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderStatus {
    Pending,
    Open,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Expired,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
    pub total: Decimal,
    pub account_type: AccountType,
    /// USD-equivalent value, if the exchange provides it.
    pub usd_value: Option<Decimal>,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub exchange_id: ExchangeId,
    pub account_type: AccountType,
    /// VIP/tier level, if the exchange exposes it.
    pub fee_tier: Option<String>,
    /// Whether the account can trade.
    pub can_trade: bool,
    /// Whether the account can withdraw.
    pub can_withdraw: bool,
}

#[derive(Debug, Clone)]
pub struct FeeInfo {
    /// Maker fee rate as a decimal fraction (0.001 = 0.1%).
    pub maker_rate: Decimal,
    /// Taker fee rate.
    pub taker_rate: Decimal,
    /// Symbol this applies to, if symbol-specific.
    pub symbol: Option<String>,
    /// Fee tier name/label.
    pub tier: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Position {
    pub symbol: String,
    pub side: PositionSide,
    pub size: Decimal,
    pub entry_price: Decimal,
    pub mark_price: Option<Decimal>,
    pub liquidation_price: Option<Decimal>,
    pub unrealized_pnl: Option<Decimal>,
    pub realized_pnl: Option<Decimal>,
    pub margin: Option<Decimal>,
    pub leverage: Option<u32>,
    pub margin_mode: Option<MarginMode>,
    pub account_type: AccountType,
    pub funding_rate: Option<Decimal>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PositionSide {
    Long,
    Short,
    Both, // Some exchanges use a single bidirectional position
}

#[derive(Debug, Clone)]
pub struct FundingRate {
    pub symbol: String,
    /// Current funding rate (e.g., 0.0001 = 0.01%).
    pub rate: Decimal,
    /// Timestamp of the next funding settlement in Unix ms.
    pub next_settlement_ms: Option<u64>,
    /// Timestamp this rate was captured.
    pub timestamp_ms: u64,
}

/// Standard error type for all connector operations.
#[derive(Debug, Clone)]
pub enum ExchangeError {
    /// The connector does not have a native endpoint for this operation.
    /// Do NOT use this for temporary failures — only for structural absence of the feature.
    UnsupportedOperation { exchange: ExchangeId, operation: String },

    /// Authentication failed or credentials are invalid.
    AuthError { message: String },

    /// The exchange rejected the request (e.g., invalid symbol, invalid price).
    InvalidRequest { message: String, code: Option<String> },

    /// Rate limit exceeded. Retry after `retry_after_ms` if provided.
    RateLimited { retry_after_ms: Option<u64> },

    /// The exchange returned an error response.
    ExchangeError { code: String, message: String },

    /// Network or transport failure.
    NetworkError { message: String },

    /// Response from the exchange could not be parsed.
    ParseError { message: String },

    /// An order was requested that does not exist.
    OrderNotFound { order_id: String },

    /// Insufficient balance or margin.
    InsufficientFunds { required: Option<Decimal>, available: Option<Decimal> },

    /// Credentials supplied are the wrong type for this exchange.
    InvalidCredentials { expected: CredentialType, provided: String },
}

pub type ExchangeResult<T> = Result<T, ExchangeError>;
```

---

## Part 5: Per-Exchange Implementation Table

The columns show which traits each exchange implements. A "Y" means the exchange has a
native endpoint and the connector MUST implement that trait. "N" means the exchange structurally
lacks the capability and the connector MUST NOT implement the trait (not even with
UnsupportedOperation — just don't impl the trait at all).

"P" means partial: the connector implements the trait but some enum variants within it
return `UnsupportedOperation`.

For `AdvancedOrders`, the cell lists which methods are Y vs N within the trait.

| Exchange | Trading | Account | CancelAll | AmendOrder | BatchOrders | Positions | AccountTransfers | CustodialFunds | SubAccounts | AdvancedOrders |
|----------|---------|---------|-----------|------------|-------------|-----------|-----------------|----------------|-------------|----------------|
| **Binance** | Y | Y | Y | Y | Y (F:5, batch cancel Y) | Y | Y | Y | Y (broker) | Trailing(F), Iceberg(S) |
| **Bybit** | Y | Y | Y | Y | Y (20/10) | Y | Y | Y | N (no create/list, transfer only) | Trailing, Iceberg |
| **OKX** | Y | Y | Y | Y | Y (20, incl batch amend) | Y | Y | Y | Y (partial: create+transfer, no list) | Trailing, OCO, Bracket, Iceberg, TWAP |
| **KuCoin** | Y | Y | Y | P (HF only; spot: cancel+recreate) | Y (S:5) | Y | Y | Y | N (transfer only) | Iceberg |
| **Kraken** | Y | Y | Y | P (WS amend preferred; F: editorder) | Y (S:15 same-pair) | Y | Y | Y | N | Trailing(F), Iceberg(S), Bracket(P via OTO) |
| **Coinbase** | Y | Y | P (batch_cancel, no all endpoint) | P (limit GTC only) | N | Y (INTX only) | Y | N | N | Bracket, TWAP |
| **Gate.io** | Y | Y | Y | Y | Y (S:10) | Y | Y | P (no deposit addr) | N (partial transfer, no create) | Iceberg |
| **Bitfinex** | Y | Y | Y | Y | Y (75 ops mixed) | Y | Y | N | N (partial) | Trailing, OCO, Iceberg |
| **Bitstamp** | Y | Y | Y | P (cancel_and_new only) | N | N | P (limited) | Y | N (institutional only) | N |
| **MEXC** | Y | Y | Y | N | Y (S:20/F:50) | Y | Y | N | N (partial) | N |
| **HTX** | Y | Y | Y | N | Y (10) | Y | Y | N | N | Trailing (spot algo only) |
| **Bitget** | Y | Y | Y | P (F only) | Y (50) | Y | Y | N | N | Trailing(F), Bracket(F OTOCO) |
| **Gemini** | Y | Y | Y | N | N | P (get positions only) | Y | Y | Y (create+list+transfer) | N |
| **BingX** | Y | Y | Y | P (F only) | Y | Y | Y | Y | Y | Trailing(F), TWAP |
| **Phemex** | Y | Y | Y | Y | N | Y | Y | Y | N (transfer only) | Trailing(F), Bracket(F 5-order), Iceberg(S) |
| **Crypto.com** | Y | Y | Y | Y | Y (10) | Y | Y | Y | N (UI only, no API) | OCO (Advanced API), Bracket (OTOCO) |
| **Upbit** | Y | Y | Y | P (cancel_and_new only) | N | N | N | Y | N | N |
| **Deribit** | Y | Y | Y | Y | N | Y | Y | Y | Y | Trailing, OCO, Bracket (OTOCO), Iceberg |
| **HyperLiquid** | Y | Y | P (no cancel-by-symbol) | Y | Y | Y | Y | N | P (list+transfer, no create) | TWAP |
| **Lighter** | Y | Y | Y | Y | Y (50) | Y | Y | Y | Y | TWAP, OCO(P) |
| **Jupiter** | Y | Y | Y | N | N | P (get only, no perp position mgmt) | N/A | N/A | N/A | N |
| **GMX** | Y | P | N | Y (on-chain updateOrder) | P (multicall != batch-place API) | Y | N/A | N/A | N/A | N |
| **Paradex** | Y | Y | Y | Y | Y (10) | P (get positions only, leverage P) | N | N/A | N (partial) | TWAP |
| **dYdX V4** | Y | Y | N | N | P (Cosmos tx batching, not REST) | Y | Y | N/A | Y (auto-create, list, transfer) | TWAP |

### Notes on "P" entries

**Coinbase CancelAll P**: Has `POST /brokerage/orders/batch_cancel` which takes order IDs —
this is not a true "cancel all" but a batch cancel. `CancelScope::All` → `UnsupportedOperation`.
`CancelScope::BySymbol` with known order IDs can be composed by the execution layer, not the connector.

**KuCoin AmendOrder P**: High-Frequency (HF) trading endpoint supports true amend. Classic
endpoint does cancel+recreate internally but exposes it as amend. The connector should implement
`AmendOrder` only for HF accounts.

**GMX BatchOrders P**: GMX uses Solidity `multicall` on the ExchangeRouter contract, which
batches multiple on-chain calls. This is structurally different from a REST batch-place endpoint.
It could be modeled as `BatchOrders` with `max_batch_place_size = 10`, but the semantics
(on-chain gas batching vs exchange matching engine atomicity) differ enough to treat as partial.

**HyperLiquid CancelAll P**: Supports `cancelByCloid` (by client order ID) but no global
cancel-all. `CancelScope::All` → `UnsupportedOperation`. `CancelScope::ByClientGroup` works.

---

## Part 6: Auth Implementation Per Exchange

Each connector struct holds its credentials internally. The signing logic is private.

| Exchange | Credentials Variant | Signing Algorithm | Request Method |
|----------|---------------------|-------------------|----------------|
| Binance | `HmacSha256` | HMAC-SHA256 | Header `X-MBX-APIKEY` + query `signature` + `timestamp` |
| Bybit | `HmacSha256` | HMAC-SHA256 | 4 custom headers: `X-BAPI-API-KEY`, `X-BAPI-SIGN`, `X-BAPI-TIMESTAMP`, `X-BAPI-RECV-WINDOW` |
| OKX | `HmacSha256WithPassphrase` | HMAC-SHA256 | 4 headers: `OK-ACCESS-KEY`, `OK-ACCESS-SIGN`, `OK-ACCESS-TIMESTAMP`, `OK-ACCESS-PASSPHRASE` |
| KuCoin | `HmacSha256WithPassphrase` | HMAC-SHA256 (passphrase also HMAC signed) | `KC-API-KEY`, `KC-API-SIGN`, `KC-API-TIMESTAMP`, `KC-API-PASSPHRASE` |
| Kraken | `KrakenDual` | SHA512 (spot) / SHA256 (futures) | `API-Key` + `API-Sign`; nonce in body |
| Coinbase | `JwtEs256` | ECDSA P-256 (ES256 JWT) | `Authorization: Bearer <JWT>` (JWT valid 2 min) |
| Gate.io | `HmacSha512` | HMAC-SHA512 | `KEY` + `SIGN` headers; payload = `method\npath\nquery\nbody_hash\ntimestamp` |
| Bitfinex | `HmacSha384` | HMAC-SHA384 | `bfx-apikey`, `bfx-signature`, `bfx-nonce`; payload = path + nonce + body |
| Bitstamp | `HmacSha256` | HMAC-SHA256 | Headers; all private calls form-encoded POST |
| MEXC | `HmacSha256` | HMAC-SHA256 | `X-MEXC-APIKEY` header + `signature` query param (Binance-compatible) |
| HTX | `HmacSha256` | HMAC-SHA256 | All params in query string including `AccessKeyId`, `Timestamp`, `Signature` |
| Bitget | `HmacSha256WithPassphrase` | HMAC-SHA256 | `ACCESS-KEY`, `ACCESS-SIGN`, `ACCESS-TIMESTAMP`, `ACCESS-PASSPHRASE` |
| Gemini | `HmacSha256` | HMAC-SHA256 | Payload base64 in `X-GEMINI-PAYLOAD` header; nonce required |
| BingX | `HmacSha256` | HMAC-SHA256 | `X-BX-APIKEY` header + `signature` param (Binance-compatible) |
| Phemex | `HmacSha256` | HMAC-SHA256 | `x-phemex-access-token`, `x-phemex-request-expiry`, `x-phemex-request-signature` |
| Crypto.com | `HmacSha256` | HMAC-SHA256 | `api_key` + `sig` in JSON request body (not headers) |
| Upbit | `JwtHmacSha256` | HMAC-SHA256 signed JWT payload | `Authorization: Bearer {jwt_token}` |
| Deribit | `OAuth2ClientCredentials` | — (Bearer token) | `public/auth` → `access_token` → Bearer; auto-refresh before expiry |
| HyperLiquid | `EthereumWallet` | secp256k1 ECDSA | Signs typed action hash per request; two schemes: L1 action vs user-signed |
| Lighter | `StarkKey` | STARK ZK signature | Signs each order/tx via `SignerClient`; REST reads use `auth` token header |
| Jupiter | `SolanaKeypair` | Ed25519 | Signs Solana transactions; `x-api-key` header for REST reads |
| GMX | `EthereumWallet` | secp256k1 ECDSA | On-chain only; signs `ExchangeRouter` contract calls; REST is read-only |
| Paradex | `StarkKey` | STARK EC curve | STARK `[r,s]` signature per order + `signature_timestamp`; JWT Bearer for REST reads |
| dYdX V4 | `CosmosWallet` | Cosmos secp256k1 | `MsgPlaceOrder`/`MsgCancelOrder` via gRPC; Indexer REST is fully public |

### OAuth2 Caching (Deribit)

The Deribit connector MUST cache the access_token and auto-refresh before expiry.
The connector holds a `tokio::sync::RwLock<Option<TokenCache>>` internally.
Before each authenticated call, it checks `now + buffer_secs > token_expiry_ms` and
refreshes if needed. This is an internal concern — the `Authenticated` trait does not
expose token lifecycle.

```rust
struct TokenCache {
    access_token: String,
    expires_at_ms: u64,
}

// Inside DeribitConnector — private implementation detail
async fn ensure_authenticated(&self) -> ExchangeResult<String> {
    let cache = self.token_cache.read().await;
    if let Some(ref t) = *cache {
        if chrono::Utc::now().timestamp_millis() as u64 + 5_000 < t.expires_at_ms {
            return Ok(t.access_token.clone());
        }
    }
    drop(cache);
    // Refresh token...
    self.refresh_token().await
}
```

---

## Part 7: Migration Path

### From Current Design to Thin-Trait + Fat-Enum

**Step 1: Introduce the fat enums (non-breaking)**

Add `OrderType` enum, `CancelScope` enum, `AmendFields`, `PositionModification` to a new
`digdigdig3/src/crypto/types.rs`. These are new types — existing code is unaffected.

**Step 2: Refactor Trading trait (breaking)**

Replace `market_order` + `limit_order` → `place_order(&self, symbol, side, OrderType, ...)`.
Each existing connector that handled market/limit now matches on `OrderType::Market` and
`OrderType::Limit`. Any connector that was in `AdvancedOrders` for `stop_limit` moves the
handling into `Trading::place_order` matching on `OrderType::StopLimit`.

**Step 3: Add missing universal methods (additive)**

Add `get_order_history` to `Trading` trait.
Add `get_fees` to `Account` trait.
All existing connectors implement these — they all have history and fee endpoints.

**Step 4: Remove BatchOperations default impl (breaking)**

The sequential loop default implementation is deleted.
Exchanges without native batch endpoints (Bitstamp, Coinbase, Gemini, Phemex, Upbit, Deribit,
Jupiter) must now NOT implement `BatchOrders` at all, rather than inheriting a fake impl.
The connector_manager layer (the consumer) is responsible for chunking/looping if it needs
sequential simulation.

**Step 5: Consolidate Positions (breaking)**

Replace `set_leverage` in base `Positions` with `modify_position(PositionModification)`.
The `PositionModification` enum handles leverage, margin mode, add/remove margin, and close.
Deribit and dYdX V4 implement `Positions` but their `modify_position` returns
`UnsupportedOperation` for `SetLeverage` and `SetMarginMode` — these exchanges have no
native lever/mode endpoint.

**Step 6: Auth overhaul (breaking)**

Replace single `Credentials` struct with `ExchangeCredentials` enum.
Replace `ExchangeAuth` trait with `Authenticated` marker.
Each connector's constructor now accepts the specific `ExchangeCredentials` variant it requires.
Gate.io's connector constructor: `fn new(creds: ExchangeCredentials) -> Result<Self>` — internally
matches on `ExchangeCredentials::HmacSha512` and returns `InvalidCredentials` for anything else.

**Step 7: Remove extension trait violations**

Delete all `default impl` blocks in extension traits.
Rename `BatchOperations` → `BatchOrders`.
Rename `Transfers` → `AccountTransfers` with all methods required, no defaults.
Split `MarginTrading` → keep only `modify_position` in `Positions` fat enum.
Move spot margin lending (`borrow_margin`/`repay_margin`) to `extensions::SpotMarginLending`
(ultra-specialized, not in core library).

### File Layout After Migration

```
digdigdig3/src/crypto/
├── mod.rs
├── types.rs              # All fat enums: OrderType, CancelScope, AmendFields,
│                         # PositionModification, ExchangeCredentials, etc.
├── errors.rs             # ExchangeError, ExchangeResult<T>
├── traits/
│   ├── mod.rs
│   ├── identity.rs       # ExchangeIdentity, ExchangeId, ExchangeKind
│   ├── trading.rs        # Trading (place_order, cancel, get_order, get_open, history)
│   ├── account.rs        # Account (balance, info, fees)
│   ├── cancel_all.rs     # CancelAll (cancel_all_orders with CancelScope)
│   ├── amend_order.rs    # AmendOrder (amend_order with AmendFields)
│   ├── batch_orders.rs   # BatchOrders (place_batch, cancel_batch)
│   ├── positions.rs      # Positions (get_positions, funding_rate, modify_position)
│   ├── transfers.rs      # AccountTransfers (transfer, history)
│   ├── custodial.rs      # CustodialFunds (deposit_addr, withdraw, history)
│   ├── subaccounts.rs    # SubAccounts (SubAccountOperation fat enum)
│   └── advanced_orders.rs # AdvancedOrders (trailing, oco, bracket, iceberg, twap)
├── cex/
│   ├── mod.rs
│   ├── binance/
│   ├── bybit/
│   └── ...
├── dex/
│   ├── mod.rs
│   ├── hyperliquid/
│   ├── lighter/
│   └── ...
└── THIN_TRAIT_FAT_ENUM_DESIGN.md   # This document
```

---

## Part 8: Design Invariants (Enforcement Checklist)

These invariants must be verified for every new connector and every trait modification:

### Invariant 1: No Composition in Connectors

> If an exchange does not have a native endpoint for a capability, the connector returns
> `ExchangeError::UnsupportedOperation`. It does NOT implement the capability by calling
> other methods.

Verify: Search for any connector method body that calls another trait method.
The only legitimate cross-method call within a connector is within a single ordered operation
(e.g., OAuth2 token refresh before a request) — not user-facing capability simulation.

### Invariant 2: No Default Implementations in Extension Traits

> Extension traits (CancelAll, AmendOrder, BatchOrders, Positions, AccountTransfers,
> CustodialFunds, SubAccounts, AdvancedOrders) MUST NOT have default method implementations.

The only trait with default methods is `Trading::supported_order_types()` and similar
capability-discovery methods, which return empty/conservative defaults and are purely informational.

Verify: `grep -n "fn.*{" traits/**/*.rs | grep -v "fn supported_"` — no default bodies.

### Invariant 3: Trait Membership Matches Reality

> If a connector implements a trait, it means the exchange has a native endpoint for
> at least one method in that trait. If the exchange lacks ALL methods in a trait,
> the connector MUST NOT implement the trait.

Verify: The per-exchange table in Part 5 is the source of truth. Any deviation requires
updating the table AND documenting why the matrix data was incorrect.

### Invariant 4: ExchangeCredentials Variant Matches Exchange

> Each connector's constructor accepts `ExchangeCredentials` and returns
> `Err(ExchangeError::InvalidCredentials)` if the wrong variant is provided.
> It NEVER silently ignore wrong credential types.

Verify: Every connector's `new(creds: ExchangeCredentials)` function has an explicit
`match` that returns `InvalidCredentials` for all non-applicable variants.

### Invariant 5: Fat Enum Variants Are Self-Documenting

> Every `OrderType` variant, every `CancelScope` variant, every `PositionModification`
> variant must have a doc comment citing how many of the 24 exchanges support it.

Verify: `rustdoc` renders the enum — each variant's doc comment includes the support count.

---

## Appendix: Exchange-to-Credential Variant Quick Reference

| Exchange | `ExchangeCredentials` variant |
|----------|-------------------------------|
| Binance | `HmacSha256` |
| Bybit | `HmacSha256` |
| OKX | `HmacSha256WithPassphrase` |
| KuCoin | `HmacSha256WithPassphrase` |
| Kraken | `KrakenDual` |
| Coinbase | `JwtEs256` |
| Gate.io | `HmacSha512` |
| Bitfinex | `HmacSha384` |
| Bitstamp | `HmacSha256` |
| MEXC | `HmacSha256` |
| HTX | `HmacSha256` |
| Bitget | `HmacSha256WithPassphrase` |
| Gemini | `HmacSha256` |
| BingX | `HmacSha256` |
| Phemex | `HmacSha256` |
| Crypto.com | `HmacSha256` |
| Upbit | `JwtHmacSha256` |
| Deribit | `OAuth2ClientCredentials` |
| HyperLiquid | `EthereumWallet` |
| Lighter | `StarkKey` |
| Jupiter | `SolanaKeypair` |
| GMX | `EthereumWallet` |
| Paradex | `StarkKey` |
| dYdX V4 | `CosmosWallet` |
