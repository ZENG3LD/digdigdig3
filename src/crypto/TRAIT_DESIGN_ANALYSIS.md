# V5 Trait Design Analysis
# digdigdig3 — Multi-Exchange Connector Library

Generated: 2026-03-12
Based on: TRADING_CAPABILITY_MATRIX.md + matrix_batch1/2/3.md (24 exchanges)

---

## Strict Rule (repeated for clarity)

> Connectors MUST NEVER compose base methods to simulate missing features.
> If an exchange has no native endpoint for something, the connector returns
> `ExchangeError::UnsupportedOperation`. All composition belongs in higher layers.

This rule is the lens through which every design decision below is evaluated.

---

## 1. Trading Trait — Is It Correctly Minimal?

### Current five methods

```
market_order        cancel_order
limit_order         get_order
                    get_open_orders
```

### Verdict on each method

**market_order — 24/24 Y. CORRECT. Keep.**

**limit_order — 24/24 Y. CORRECT. Keep.**

**cancel_order — 24/24 Y. CORRECT. Keep.**

**get_open_orders — 24/24 Y. CORRECT. Keep.**

**get_order — Table C shows 23 Y + 1 P (Lighter returns it only via list
filtering). That is effectively universal. CORRECT. Keep.**

### What is MISSING from Trading that is universal

**`get_order_history` — 24/24 Y. THIS IS MISSING FROM THE BASE TRAIT.**

Table C, column GetHistory: 24/24 exchanges support it, including Upbit (spot
minimal floor), Jupiter (DEX), GMX (via GraphQL). Every single exchange that
can place orders also exposes order history. This is not a composition — it is
a native endpoint on every platform. It MUST be in the base Trading trait.

Its absence from the current trait is an oversight. A consumer of digdigdig3
that calls `get_order_history` would today need to downcast to an exchange-
specific type or use an extension trait, which defeats the purpose of a unified
abstraction.

### What SHOULD be removed from Trading

Nothing should be removed. All five current methods are universal.

However, the signature of `market_order` and `limit_order` deserves scrutiny.
Having two separate methods for order types (instead of one `place_order` with
an `OrderType` parameter) means every new order type variant requires a new
method on the base trait. The matrix proposal in section 4 of
TRADING_CAPABILITY_MATRIX.md already proposes a unified `place_order(req:
OrderRequest)`. This is the correct design — `OrderRequest` carries the order
type enum, and the parser/connector rejects unsupported types with
`UnsupportedOperation`. The current split into `market_order` / `limit_order`
is premature type-level duplication.

### Proposed fix

```rust
pub trait Trading: ExchangeIdentity {
    // Unified placement: OrderRequest.order_type = Market | Limit | ...
    // Connector returns UnsupportedOperation for any type it cannot natively execute.
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<Order>;

    async fn cancel_order(&self, symbol: &str, order_id: &str,
                          account_type: AccountType) -> ExchangeResult<Order>;

    async fn get_order(&self, symbol: &str, order_id: &str,
                       account_type: AccountType) -> ExchangeResult<Order>;

    async fn get_open_orders(&self, symbol: Option<&str>,
                             account_type: AccountType) -> ExchangeResult<Vec<Order>>;

    // ADDED — universally supported, was missing:
    async fn get_order_history(&self, filter: OrderHistoryFilter,
                               account_type: AccountType) -> ExchangeResult<Vec<Order>>;
}
```

---

## 2. Account Trait — Is It Correctly Minimal?

### Current two methods

```
get_balance(asset: Option<Asset>, account_type) -> Vec<Balance>
get_account_info(account_type) -> AccountInfo
```

### Verdict on get_balance

Table E, column Balances: 23 Y + 1 P (GMX is partial — requires on-chain
`balanceOf` calls). Given that even GMX can produce a balance value (just via
a different mechanism), this is effectively 24/24. CORRECT. Keep.

### Verdict on get_account_info

`get_account_info` is currently in the trait but the matrix does not have a
direct "account info" column. Looking at what account_info would contain:
fee tier, trading limits, account status, VIP level. These are exchange-
specific fields. Returning an opaque `AccountInfo` struct that is mostly empty
for minimal exchanges (Upbit, GMX, Jupiter) is fine, but it begs the question
of what the base contract actually guarantees.

If `AccountInfo` is defined as containing only fields that are universally
available, it can stay. If it tries to carry fee schedules, leverage limits,
and IP whitelist status, those belong in extension traits.

Recommendation: keep `get_account_info` but define `AccountInfo` minimally —
only fields that every exchange can populate (account_type, available balance
summary). Exchange-specific details go in connector-specific structs.

### Should get_fees() be in base Account? — 23/24

Table E, column Fees: 17 Y + 6 P = 23/24 non-N. Only GMX (N/A — fees
embedded in on-chain calculation) is missing. 23/24 is very strong.

**YES, `get_fees()` should be in the base Account trait.**

Reasoning:
- 23 of 24 exchanges have a native fee endpoint
- The 6 "partial" responses all have fees embedded in order/trade responses —
  a connector can surface them
- GMX is the sole exception and GMX's fee data is available via REST market
  info endpoint — it qualifies as Y at the exchange level even if the
  *account-specific* fee tier does not exist (fees are uniform in GMX V2)
- Fee information is essential for any order sizing calculation; hiding it
  behind an extension trait forces consumers to feature-detect for something
  nearly every exchange provides

The method signature: `get_fees(symbol: Option<&str>) -> ExchangeResult<FeeInfo>`
where `FeeInfo` contains `{ maker_rate: Decimal, taker_rate: Decimal }` at
minimum. Exchanges that only return symbol-level fees can accept `None` as
"use the default tier".

### Proposed fix

```rust
pub trait Account: ExchangeIdentity {
    async fn get_balance(&self, asset: Option<&str>,
                         account_type: AccountType) -> ExchangeResult<Vec<Balance>>;

    async fn get_account_info(&self, account_type: AccountType)
        -> ExchangeResult<AccountInfo>;

    // ADDED — 23/24 exchanges support this natively:
    async fn get_fees(&self, symbol: Option<&str>)
        -> ExchangeResult<FeeInfo>;
}
```

---

## 3. Positions Trait — Is It Correctly Scoped?

### Context: the denominator is 22, not 24

Bitstamp and Upbit are spot-only; their N/A entries mean positions questions
are structurally inapplicable. Jupiter and GMX are also N/A for set_leverage
and margin_mode. The effective denominator for futures capability is 18–22
depending on the capability.

### get_positions — 18 Y + 2 P / 22

Strong majority. Appropriate in a FuturesPositions trait (not base Trading).
Spot-only exchanges implement Trading and Account but not Positions. This
scoping is correct.

### get_funding_rate — 17 Y / 22

17 of 22 futures-capable exchanges have a native funding rate endpoint.
The 5 missing are: Bitget (N), HTX (public only — no private account funding),
MEXC (N), and a couple of others. "Public only" funding rate data is relevant
here: Table D shows that the endpoint may be public (no auth required) but
still exists as a native call. The 17/22 count uses strict "Y" and several
more are partial. Given that funding rates are central to perpetual futures
trading, 17/22 is sufficient to keep it in the Positions trait rather than
an extension. KEEP in Positions.

However, there is a distinction between:
- **`get_funding_rate(symbol)` — current rate**: available on ~17/22
- **`get_funding_history(symbol)` — historical payments**: a separate, less
  universal capability

The current trait only has `get_funding_rate`. This is fine. The history
variant belongs in an extension.

### set_leverage — 15 Y + 2 P / 22

15 Y + 2 P = 17/22. That is 77% of futures exchanges.

The 5 exchanges that cannot natively set leverage: Deribit (N — leverage is
implicit via margin allocation), dYdX V4 (N — leverage is position size /
equity), Jupiter (N/A), GMX (N — leverage is implicit), Bitfinex (P — per-
order parameter only, no account-level set).

**This is the key design question: does 77% qualify for the base Positions
trait or an extension?**

Analysis:
- Deribit and dYdX are major, widely-used exchanges
- For Deribit, `set_leverage` must return `UnsupportedOperation` because there
  is no Deribit endpoint — leverage is implicit
- For dYdX, same: no explicit leverage endpoint exists
- Putting `set_leverage` in base Positions requires these connectors to always
  return `UnsupportedOperation`, which is a runtime failure on a very common
  operation
- A consumer who needs to set leverage before trading perpetuals must either
  check capabilities or receive runtime errors

**Recommendation: move `set_leverage` to an extension trait `LeverageControl`.**

This makes the capability opt-in. Connectors that have native leverage APIs
implement `LeverageControl`. Connectors without it simply don't implement it.
The consumer uses `connector.as_leverage_control()` and handles None gracefully.

Same logic applies marginally to `get_funding_rate` — but 17/22 is stronger
than 15/22, and funding rate is read-only (not an action that can fail with
misleading behavior). Keep funding rate in base Positions.

### Proposed Positions trait

```rust
pub trait Positions: ExchangeIdentity {
    async fn get_positions(&self, symbol: Option<&str>,
                           account_type: AccountType) -> ExchangeResult<Vec<Position>>;

    async fn get_funding_rate(&self, symbol: &str,
                              account_type: AccountType) -> ExchangeResult<FundingRate>;

    // REMOVED: set_leverage — moved to LeverageControl extension
}

// Extension: 15Y+2P of 22 futures exchanges
pub trait LeverageControl: Positions {
    async fn set_leverage(&self, symbol: &str, leverage: u32,
                          account_type: AccountType) -> ExchangeResult<()>;
}
```

---

## 4. Extension Traits — Validation

### BatchOperations — THE COMPOSITION VIOLATION

**Current design: BatchOperations has a DEFAULT IMPL that loops sequentially.**

This is a direct violation of the strict rule. Here is why:

The strict rule says: if an exchange does not have a native batch API endpoint,
return `UnsupportedOperation`. A default implementation that calls
`cancel_order` N times in a loop is precisely the kind of composition that is
forbidden. It silently "works" for exchanges without batch support, making
callers believe they are getting a real batch operation when they are actually
issuing N sequential HTTP calls with N independent round trips.

This matters because:
1. The caller cannot distinguish "this exchange has a real batch cancel that
   atomically removes 20 orders" from "this is looping your 20 sequential
   cancels through our fake impl"
2. Rate limiting behavior is completely different
3. Atomic guarantees differ — a real batch cancel either succeeds entirely or
   fails; a loop can partially succeed
4. Throughput expectations differ by an order of magnitude

**Fix: Remove all default implementations from BatchOperations. Every method
MUST be explicit. Exchanges without native batch endpoints do not implement
this trait at all, and callers get a compile-time `None` from downcasting.**

Table C, column Batch (max): 14 Y + 3 P = 17/24 have native batch creation.
That is still a strong majority. BatchOperations is worth keeping as an
extension — just without the fake default impl.

```rust
// CORRECT — no default implementations
pub trait BatchOperations: Trading {
    /// Native batch order placement.
    /// Only implement if the exchange has a real /batch-orders endpoint.
    async fn create_orders_batch(&self, orders: Vec<OrderRequest>)
        -> ExchangeResult<Vec<OrderResult>>;

    /// Native batch cancel.
    /// Only implement if the exchange has a real cancel-batch endpoint.
    async fn cancel_orders_batch(&self, cancels: Vec<CancelRequest>)
        -> ExchangeResult<Vec<CancelResult>>;
}
```

Exchanges that implement BatchOperations (native batch create): Binance (F
only, 5), Bybit (20/10), OKX (20), KuCoin (S:5), Gate.io (S:10), Bitfinex
(75 mixed), MEXC (S:20/F:50), HTX (10), Bitget (50), BingX (unspec.),
Crypto.com (10), HyperLiquid (unspec.), Lighter (50), Paradex (10),
dYdX (partial via Cosmos tx batching).

Exchanges that do NOT implement BatchOperations: Bitstamp, Kraken (S:15 for
same-pair is the one partial), Coinbase (batch cancel only, not place),
Gemini, Phemex, Upbit, Deribit, Jupiter, GMX (multicall ≠ batch API),
dYdX (Cosmos tx can batch but not a dedicated batch-place endpoint).

### AdvancedOrders — Method list vs matrix

Current methods: `create_trailing_stop`, `create_stop_limit_order`,
`create_oco_order`. All default to `UnsupportedOperation`.

**Problem 1: `create_stop_limit_order` should NOT be in AdvancedOrders.**

Table A StopLimit: 16 Y + 3 P = 19/24 non-N. That is 79% support. Stop-limit
is one of the most common order types across all exchange categories. It is
more prevalent than trailing stop (10/24), OCO (7/24), or bracket (9/24).
Stop-limit belongs either in the base `place_order` call (via `OrderType`
enum) or in a `ConditionalOrders` trait — not in `AdvancedOrders` alongside
rarities like OCO and trailing stop.

Similarly, StopMarket: 19/24 non-N. TP/SL: 19/24 non-N.

Stop, TP, and SL order types should be covered by `OrderType` enum variants in
`OrderRequest` rather than separate trait methods. The connector rejects
unsupported variants with `UnsupportedOperation`. This keeps `place_order`
universal (it always exists) while allowing the `OrderType` to be exchange-
specific.

**Problem 2: What truly belongs in AdvancedOrders?**

Looking at the matrix, "advanced" should mean capabilities that are rare
enough to warrant their own trait interface AND that have genuinely distinct
call signatures from a regular order:

| Type         | Count      | Distinct signature? | Belongs? |
|--------------|------------|---------------------|----------|
| TrailingStop | 10/24      | Yes (offset/percent) | Yes |
| OCO          | 7/24       | Yes (two linked orders) | Yes |
| Bracket      | 9/24 non-N | Yes (entry+TP+SL one call) | Yes |
| Iceberg      | 8/24       | Yes (visible_qty param) | Yes |
| TWAP         | 7/24       | Yes (duration, slices) | Yes |
| StopLimit    | 19/24      | No — just OrderType::StopLimit | No |
| StopMarket   | 19/24      | No — just OrderType::StopMarket | No |
| TP/SL        | 19/24      | No — just OrderType::TakeProfit/StopLoss | No |

**Proposed AdvancedOrders:**

```rust
pub trait AdvancedOrders: Trading {
    async fn place_trailing_stop(&self, req: TrailingStopRequest)
        -> ExchangeResult<Order>;

    async fn place_oco(&self, req: OcoRequest)
        -> ExchangeResult<OcoResponse>;

    async fn place_bracket(&self, req: BracketRequest)
        -> ExchangeResult<BracketResponse>;

    async fn place_iceberg(&self, req: IcebergRequest)
        -> ExchangeResult<Order>;

    async fn place_twap(&self, req: TwapRequest)
        -> ExchangeResult<AlgoOrderResponse>;
    // No default impls. All UnsupportedOperation if not implemented.
}
```

Note: AdvancedOrders methods may each be implemented independently. A
connector that has TWAP but no OCO still implements the AdvancedOrders trait
but its `place_oco` returns `UnsupportedOperation`.

### MarginTrading and Transfers

**MarginTrading: borrow_margin, repay_margin, get_margin_info, set_margin_type**

Table D MarginMode: 13 Y + 4 P / 22. That is majority but not dominant. Set
margin mode is meaningful on CEX futures.

However `borrow_margin` / `repay_margin` are specifically **spot margin
lending** operations. Looking at the matrix — these are relevant only for
exchanges that support spot margin (Bitfinex, Kraken spot margin, Binance
Margin account). That is roughly 5–8 exchanges. This is too narrow for a
shared extension trait.

Recommendation: Split MarginTrading:
- `set_margin_type` (isolated/cross) → move to `LeverageControl` (already
  proposed above, alongside `set_leverage`)
- `borrow_margin` / `repay_margin` → move to a `SpotMarginLending` trait
  (ultra-specialized, 5/24)
- `get_margin_info` → could be part of Account or Positions depending on
  whether it is per-account or per-position

**Transfers: `transfer` (required), `get_transfer_history` (default UnsupportedOperation)**

Table E InternalTransfer: 16 Y + 1 P / 20 applicable. Strong majority. The
current design is reasonable.

However, the `transfer` method being marked as "required" in an extension trait
creates a contradiction: if the trait is optional (extension), all its methods
should either be optional or have sane defaults. Marking one method as required
while others default to UnsupportedOperation within the same extension trait
is logically inconsistent.

Recommendation: Rename to `AccountTransfers`. All methods required (no
defaults). Connectors that don't support it simply don't implement it. This
affects: Jupiter (N/A), GMX (N/A), Paradex (N), Upbit (N — single account
type), Coinbase (limited).

---

## 5. Auth Trait — Is It Flexible Enough?

### Current model

```rust
trait ExchangeAuth: Send + Sync {
    fn sign_request(&self, credentials: &Credentials, req: &mut AuthRequest)
        -> ExchangeResult<()>;
    fn signature_location(&self) -> SignatureLocation; // Headers or QueryParams
}

struct Credentials {
    api_key: String,
    api_secret: String,
    passphrase: Option<String>,
}
```

### Problem: 10 distinct auth mechanisms in the matrix

From Table H:

| Category | Exchanges | Current model handles? |
|----------|-----------|------------------------|
| HMAC-SHA256 (key+secret) | 12 exchanges | YES |
| HMAC-SHA256 + passphrase | OKX, KuCoin, Bitget | YES (passphrase field) |
| HMAC-SHA512 | Gate.io | NO — different algorithm |
| HMAC-SHA384 | Bitfinex | NO — different algorithm |
| JWT (ECDSA P-256) | Coinbase | NO — requires PEM private key, not secret |
| JWT (HMAC signed payload) | Upbit | Partially — but JWT generation logic differs |
| OAuth2 client_credentials | Deribit | NO — requires token refresh flow |
| Ethereum ECDSA (secp256k1) | HyperLiquid, GMX | NO — requires wallet private key |
| Solana Ed25519 | Jupiter | NO — requires Solana keypair |
| STARK EC | Lighter, Paradex | NO — requires STARK key |
| Cosmos secp256k1 | dYdX V4 | NO — requires mnemonic/Cosmos wallet |

The current `Credentials` struct only models HMAC-style exchanges. 8 of 24
exchanges cannot be correctly represented.

### Why the current `sign_request` signature is insufficient

`sign_request` receives `&mut AuthRequest` and must modify it in place. This
works well for HMAC (add headers or query params). But:

- OAuth2 (Deribit): requires an async HTTP call to `public/auth` first to
  obtain a token, then injects the Bearer token. `sign_request` is synchronous
  — it cannot perform network I/O
- On-chain signing (HyperLiquid, Jupiter, GMX, Paradex, dYdX): the "request"
  is not an HTTP request with headers — it is a typed action struct that must
  be serialized and signed with cryptographic primitives. The `AuthRequest`
  abstraction does not model this
- Per-action signing (HyperLiquid): every action carries an ECDSA signature
  in its JSON body, not just the HTTP request headers. The signing is
  interleaved with request construction

### How the auth trait should evolve

The key insight is that auth is NOT uniform across exchange types. There are
two fundamentally different authorization paradigms:

**Paradigm A: HTTP request signing** (CEX norm)
The auth layer decorates HTTP requests with API key + signature headers/params.
The connector builds the request, the auth layer signs it, then it is sent.

**Paradigm B: Transaction signing** (DEX norm)
The auth layer signs typed action payloads (not raw HTTP requests). The
"signed transaction" IS the request body. The connector does not send a
plain HTTP request and attach a signature — it signs a domain object and
submits the signed envelope.

These two paradigms require different abstract interfaces.

**Recommended evolution:**

```rust
/// Marker trait for all auth implementations.
pub trait ExchangeAuth: Send + Sync {}

/// Paradigm A: signs HTTP requests (CEX exchanges).
/// Works for all HMAC variants, JWT-based exchanges, OAuth2 (with caching).
pub trait HttpRequestSigner: ExchangeAuth {
    /// Mutates the request to add auth headers/params.
    /// For OAuth2 (Deribit): implementations hold a cached token and refresh
    /// as needed. The fn signature remains sync but the token refresh must
    /// have been done beforehand via `ensure_token_valid()`.
    fn sign_http_request(&self, req: &mut AuthRequest) -> ExchangeResult<()>;
}

/// Paradigm B: signs typed action payloads (DEX exchanges).
/// Works for HyperLiquid (Ethereum ECDSA), Jupiter (Solana Ed25519),
/// Lighter/Paradex (STARK), dYdX (Cosmos).
pub trait ActionSigner: ExchangeAuth {
    /// Signs a typed action payload and returns the signed envelope.
    /// The `payload` bytes are the canonical serialization of the action
    /// (exchange-specific format). Returns the signature bytes.
    fn sign_action(&self, payload: &[u8]) -> ExchangeResult<Vec<u8>>;

    /// Returns the public key / address associated with this signer.
    fn public_address(&self) -> &str;
}

/// For OAuth2 exchanges (Deribit): token lifecycle management.
pub trait OAuthCredentials: HttpRequestSigner {
    /// Ensures a valid access token is cached. Must be called before sign_http_request.
    async fn ensure_token_valid(&self) -> ExchangeResult<()>;
    fn access_token(&self) -> Option<&str>;
}
```

**Concrete Credentials types** (not a single unified struct):

```rust
// For: Binance, Bybit, MEXC, BingX, Bitstamp, HTX, Phemex, Gemini, Crypto.com
pub struct HmacCredentials {
    pub api_key: String,
    pub api_secret: String,
    pub passphrase: Option<String>,  // OKX, KuCoin, Bitget
    pub algorithm: HmacAlgorithm,    // Sha256, Sha512, Sha384
}

// For: Coinbase (ES256 JWT), Upbit (HMAC-signed JWT)
pub struct JwtCredentials {
    pub key_id: String,
    pub private_key_pem: String,  // PEM for Coinbase ES256; raw secret for Upbit
    pub algorithm: JwtAlgorithm,
}

// For: Deribit
pub struct OAuth2Credentials {
    pub client_id: String,
    pub client_secret: String,
}

// For: HyperLiquid, GMX
pub struct EthereumCredentials {
    pub private_key_hex: String,  // secp256k1 private key
}

// For: Jupiter
pub struct SolanaCredentials {
    pub private_key_bytes: Vec<u8>,  // Ed25519 keypair
    pub api_key: Option<String>,     // x-api-key for REST reads
}

// For: Lighter, Paradex
pub struct StarkCredentials {
    pub private_key: String,  // STARK field element
    pub public_key: Option<String>,
}

// For: dYdX V4
pub struct CosmosCredentials {
    pub mnemonic: String,
    pub chain_id: String,
}
```

The `ExchangeAuth` bound on connector structs changes from accepting a single
`Credentials` to accepting the appropriate concrete type for that exchange.
Each exchange connector is generic over its specific credentials type, not
over a universal opaque `Credentials`.

---

## 6. Missing Traits

### get_order_history — 24/24, completely absent

Already covered in section 1. This is the most significant omission. Must be
added to the base `Trading` trait.

### cancel_all_orders — 20 Y + 2 P / 24

Table C CancelAll: 20 Y + 2 P = 22/24 non-N. Only GMX (N) and dYdX V4 (N —
`MsgBatchCancel` requires explicit order IDs, not a cancel-all primitive) lack
this.

22/24 qualifies this as COMMON, not specialized. It should be in an extension
trait (not base — because 2 exchanges genuinely lack it), but it should be in
a MANDATORY extension rather than an optional specialized one.

Recommendation: create a `OrderManagement` extension trait that sits one level
above `Trading`:

```rust
pub trait OrderManagement: Trading {
    /// Cancel all open orders, optionally filtered by symbol.
    async fn cancel_all_orders(&self, symbol: Option<&str>,
                               account_type: AccountType)
        -> ExchangeResult<CancelAllResponse>;

    /// Modify an existing order's price and/or quantity.
    /// 18/24 exchanges support native amend.
    async fn amend_order(&self, req: AmendRequest)
        -> ExchangeResult<Order>;
}
```

Exchanges implementing `OrderManagement.cancel_all_orders`: 22/24.
Exchanges implementing `OrderManagement.amend_order`: 13 Y + 5 P = 18/24.

GMX and dYdX implement `amend_order` (GMX via `updateOrder` on-chain; dYdX
cannot truly amend but replaces). For strict compliance:
- GMX: `cancel_all_orders` → `UnsupportedOperation`, `amend_order` → Y
- dYdX: both → `UnsupportedOperation`

These two implementing `amend_order` but not `cancel_all_orders` is fine
because `OrderManagement` is an extension they partially implement.

Actually a cleaner decomposition: separate the two methods into separate traits
given they have different support levels:

```rust
// 22/24 — very high coverage
pub trait CancelAll: Trading {
    async fn cancel_all_orders(&self, symbol: Option<&str>,
                               account_type: AccountType)
        -> ExchangeResult<CancelAllResponse>;
}

// 18/24 — high coverage
pub trait AmendOrder: Trading {
    async fn amend_order(&self, req: AmendRequest,
                         account_type: AccountType)
        -> ExchangeResult<Order>;
}
```

### CancelBySymbol — 16 Y + 1 P / 24

Present in many connectors but not universal. Extension trait. Exchanges like
Kraken (cancel by txid only), Coinbase (no cancel-by-symbol), and several DEXes
don't have it. Not enough coverage for a separate trait — should be a method
in the `CancelAll` trait as an optional second method.

### What the matrix shows that has no trait

Beyond what is already discussed:

**`amend_order` / `modify_order` — 18/24**: Already proposed above as
`AmendOrder` trait. Currently no trait for this common operation.

**`get_position_funding_history` — partial**: Available on many exchanges as
the account funding payment history. Not currently modeled. Should be in
a `FundingHistory` extension under `Positions`.

**`add_remove_margin` — 11 Y + 1 P / 22**: `MarginTrading` extension covers
this with `borrow_margin`/`repay_margin`, but those are spot margin concepts.
For futures, it is `add_margin(symbol, amount)`. These are different operations
with different semantics. Should be a separate `MarginAdjustment` trait:

```rust
pub trait MarginAdjustment: Positions {
    async fn add_margin(&self, symbol: &str, amount: Decimal,
                        account_type: AccountType) -> ExchangeResult<()>;
    async fn remove_margin(&self, symbol: &str, amount: Decimal,
                           account_type: AccountType) -> ExchangeResult<()>;
}
```

---

## 7. Proposed Final Trait Hierarchy

### Design Principles

1. **Core traits** = methods every compliant connector MUST implement. Not
   implementing a core trait method is a compile error.
2. **Extension traits** = optional capabilities. Connectors implement them if
   and only if the exchange has a native endpoint. No default implementations
   except where explicitly noted with strong justification.
3. **UnsupportedOperation** is only used within a trait that a connector HAS
   implemented, for methods within that trait that the exchange partially
   supports (e.g., `AdvancedOrders` implemented but `place_twap` not
   supported by this particular exchange variant).
4. **Never** implement an extension trait with a "fake" implementation that
   composes lower-level primitives.

### Tier 0: Foundation

```rust
/// Identity — every connector must implement.
pub trait ExchangeIdentity: Send + Sync {
    fn exchange_id(&self) -> ExchangeId;
    fn name(&self) -> &str;
    fn account_types(&self) -> &[AccountType];
}
```

### Tier 1: Universal Core Traits (all 24 exchanges)

```rust
/// Core trading — 24/24 exchanges.
/// Market + Limit are always available via OrderType enum in OrderRequest.
/// The connector returns UnsupportedOperation for OrderType variants it
/// cannot natively execute (e.g., StopMarket on Upbit).
pub trait Trading: ExchangeIdentity {
    async fn place_order(&self, req: OrderRequest)
        -> ExchangeResult<Order>;

    async fn cancel_order(&self, symbol: &str, order_id: &str,
                          account_type: AccountType)
        -> ExchangeResult<Order>;

    async fn get_order(&self, symbol: &str, order_id: &str,
                       account_type: AccountType)
        -> ExchangeResult<Order>;

    async fn get_open_orders(&self, symbol: Option<&str>,
                             account_type: AccountType)
        -> ExchangeResult<Vec<Order>>;

    async fn get_order_history(&self, filter: OrderHistoryFilter,
                               account_type: AccountType)
        -> ExchangeResult<Vec<Order>>;
}

/// Core account — 24/24 exchanges (23/24 strict + GMX partial balance).
pub trait Account: ExchangeIdentity {
    async fn get_balance(&self, asset: Option<&str>,
                         account_type: AccountType)
        -> ExchangeResult<Vec<Balance>>;

    async fn get_account_info(&self, account_type: AccountType)
        -> ExchangeResult<AccountInfo>;

    async fn get_fees(&self, symbol: Option<&str>)
        -> ExchangeResult<FeeInfo>;
}
```

### Tier 2: High-Coverage Extension Traits (15–22 / 24)

These are not required but nearly universal. Well-behaved connectors should
implement them. Consumers can safely assume most connectors will have them.

```rust
/// Cancel all open orders — 22/24 support.
/// Missing: GMX (on-chain no cancel-all), dYdX V4 (requires explicit IDs).
pub trait CancelAll: Trading {
    async fn cancel_all_orders(&self, symbol: Option<&str>,
                               account_type: AccountType)
        -> ExchangeResult<CancelAllResponse>;
}

/// Amend existing order — 18/24 support (13Y + 5P).
/// Not supported: Upbit, MEXC (spot), HTX (spot), Bitfinex (cancel+replace),
/// KuCoin (HF alter is cancel+replace), some DEXes.
pub trait AmendOrder: Trading {
    async fn amend_order(&self, req: AmendRequest,
                         account_type: AccountType)
        -> ExchangeResult<Order>;
}

/// Native batch order placement — 17/24 support (14Y + 3P).
/// No default implementation. If exchange has no batch endpoint: don't implement.
pub trait BatchOrders: Trading {
    async fn place_orders_batch(&self, orders: Vec<OrderRequest>)
        -> ExchangeResult<Vec<OrderResult>>;

    async fn cancel_orders_batch(&self, cancels: Vec<CancelRequest>)
        -> ExchangeResult<Vec<CancelResult>>;
}

/// Futures/perpetuals positions — 22/24 applicable (spot-only excluded).
pub trait Positions: ExchangeIdentity {
    async fn get_positions(&self, symbol: Option<&str>,
                           account_type: AccountType)
        -> ExchangeResult<Vec<Position>>;

    async fn get_funding_rate(&self, symbol: &str,
                              account_type: AccountType)
        -> ExchangeResult<FundingRate>;
}

/// Account transfers (spot/futures/margin) — 17/20 applicable exchanges.
/// Not applicable to non-custodial DEXes (Jupiter, GMX, dYdX bridges are not
/// internal transfers).
pub trait AccountTransfers: Account {
    async fn transfer(&self, req: TransferRequest)
        -> ExchangeResult<TransferResponse>;

    async fn get_transfer_history(&self, filter: TransferHistoryFilter)
        -> ExchangeResult<Vec<Transfer>>;
}
```

### Tier 3: Specialized Extension Traits (8–15 / 22 futures, or 7–14 / 24 total)

```rust
/// Leverage and margin mode control — 15Y+2P / 22 futures exchanges.
/// NOT in base Positions because Deribit (rank 1 exchange) and dYdX lack it.
pub trait LeverageControl: Positions {
    async fn set_leverage(&self, symbol: &str, leverage: u32,
                          account_type: AccountType)
        -> ExchangeResult<()>;

    async fn set_margin_mode(&self, symbol: &str, mode: MarginMode,
                             account_type: AccountType)
        -> ExchangeResult<()>;
}

/// Add or remove collateral from isolated position — 11Y+1P / 22.
pub trait MarginAdjustment: Positions {
    async fn add_margin(&self, symbol: &str, amount: Decimal,
                        account_type: AccountType)
        -> ExchangeResult<()>;

    async fn remove_margin(&self, symbol: &str, amount: Decimal,
                           account_type: AccountType)
        -> ExchangeResult<()>;
}

/// Advanced order types — selectively supported.
/// A connector implementing this trait returns UnsupportedOperation
/// for methods its exchange lacks.
pub trait AdvancedOrders: Trading {
    /// TrailingStop — 10/24
    async fn place_trailing_stop(&self, req: TrailingStopRequest)
        -> ExchangeResult<Order>;

    /// OCO (One-Cancels-Other) — 7/24
    async fn place_oco(&self, req: OcoRequest)
        -> ExchangeResult<OcoResponse>;

    /// Bracket (Entry + TP + SL in one native call) — 9/24
    async fn place_bracket(&self, req: BracketRequest)
        -> ExchangeResult<BracketResponse>;

    /// Iceberg — 8/24
    async fn place_iceberg(&self, req: IcebergRequest)
        -> ExchangeResult<Order>;

    /// TWAP algo — 7/24
    async fn place_twap(&self, req: TwapRequest)
        -> ExchangeResult<AlgoOrderResponse>;
}

/// Custodial deposits and withdrawals — ~18/24 custodial exchanges.
/// DEXes are structurally non-custodial; return UnsupportedOperation is wrong,
/// they simply do not implement this trait.
pub trait CustodialFunds: Account {
    async fn get_deposit_address(&self, asset: &str, network: Option<&str>)
        -> ExchangeResult<DepositAddress>;

    async fn withdraw(&self, req: WithdrawRequest)
        -> ExchangeResult<WithdrawResponse>;

    async fn get_deposit_withdraw_history(&self, filter: FundsHistoryFilter)
        -> ExchangeResult<Vec<FundsRecord>>;
}

/// Sub-account management — ~12/22 non-DEX exchanges.
pub trait SubAccounts: Account {
    async fn create_subaccount(&self, params: SubAccountParams)
        -> ExchangeResult<SubAccount>;

    async fn list_subaccounts(&self)
        -> ExchangeResult<Vec<SubAccount>>;

    async fn transfer_to_subaccount(&self, req: SubAccountTransfer)
        -> ExchangeResult<TransferResponse>;
}
```

### Tier 4: Rare Extension Traits (2–8 / 24)

These are valid traits but should NOT be in the core library. They belong in
an `extensions` module and are considered unstable API surface.

```rust
/// Spot margin lending — ~5 exchanges (Bitfinex, Kraken, Binance Margin,
/// Gate.io margin, HTX margin account).
pub mod extensions {
    pub trait SpotMarginLending: Account {
        async fn borrow_margin(&self, asset: &str, amount: Decimal)
            -> ExchangeResult<BorrowResponse>;
        async fn repay_margin(&self, asset: &str, amount: Decimal)
            -> ExchangeResult<()>;
        async fn get_margin_loan_info(&self, asset: Option<&str>)
            -> ExchangeResult<MarginInfo>;
    }

    /// On-chain DEX specific — not part of HTTP connector abstraction.
    /// GMX, HyperLiquid liquidity provision.
    pub trait LiquidityPool {
        async fn deposit_to_pool(&self, req: PoolDepositRequest)
            -> ExchangeResult<PoolReceipt>;
        async fn withdraw_from_pool(&self, req: PoolWithdrawRequest)
            -> ExchangeResult<PoolReceipt>;
    }
}
```

### Minimum Exchange (Upbit)

Upbit implements ONLY:
- `ExchangeIdentity`
- `Trading` (place_order: Market+Limit only; cancel; get_order; get_open; get_history)
- `Account` (balance; account_info; fees — partial but present in order responses)
- `CancelAll` (cancel_all_orders supported; cancel_by_symbol supported)

Upbit does NOT implement: `Positions`, `AmendOrder`, `BatchOrders`,
`AccountTransfers` (no transfer between account types),
`LeverageControl`, `AdvancedOrders`, `CustodialFunds` (yes it does! Upbit
supports deposit address, withdraw, deposit/withdraw history), `SubAccounts`.

Correction: Upbit DOES implement `CustodialFunds` despite its minimal trading
profile. The matrix shows DepositAddr: Y, Withdraw: Y, Dep/Wdw History: Y.

### Maximum Exchange (Deribit)

Deribit implements ALL of:
- `ExchangeIdentity`
- `Trading` (all order types via OrderType enum)
- `Account`
- `CancelAll`
- `AmendOrder`
- `BatchOrders` — actually NO for Deribit. Table C shows Deribit Batch: N.
  Deribit has `mass_quote` for market makers but no general batch order
  placement. So Deribit does NOT implement `BatchOrders`.
- `Positions`
- `AccountTransfers`
- `LeverageControl` — actually NO for Deribit. SetLev: N in Table D.
  Deribit uses implicit leverage via margin allocation. So Deribit does NOT
  implement `LeverageControl`.
- `MarginAdjustment` — NO for Deribit. AddRemMargin: N in Table D.
- `AdvancedOrders` (trailing, OCO, bracket, iceberg; NOT TWAP)
- `CustodialFunds`
- `SubAccounts`

The fact that the "maximum exchange" (Deribit) does not implement `BatchOrders`
or `LeverageControl` confirms these traits are correctly placed as optional
extensions rather than universal requirements.

---

## Summary: Changes Required to Current Implementation

| Area | Issue | Action |
|------|-------|--------|
| `Trading` | Missing `get_order_history` (24/24) | ADD to base trait |
| `Trading` | Split `market_order`/`limit_order` → unified `place_order(OrderRequest)` | REFACTOR |
| `Account` | Missing `get_fees` (23/24) | ADD to base trait |
| `Positions` | `set_leverage` not universal (15/22) | MOVE to `LeverageControl` extension |
| `BatchOperations` | Default impl loops sequentially (violates strict rule) | REMOVE all default impls |
| `AdvancedOrders` | Contains `create_stop_limit_order` which is not advanced | REMOVE; StopLimit → `OrderType` enum |
| `ExchangeAuth` | Models HMAC only; 10 auth mechanisms unsupported | REDESIGN with `HttpRequestSigner` + `ActionSigner` |
| `Transfers` | "Required" method in optional extension is contradictory | Rename to `AccountTransfers`, all methods required, no defaults |
| Missing | No `CancelAll` trait (22/24) | CREATE |
| Missing | No `AmendOrder` trait (18/24) | CREATE |
| Missing | No `MarginAdjustment` trait (11/22) | CREATE |
| Missing | No `CustodialFunds` trait | CREATE |
| Missing | `AdvancedOrders` missing Iceberg (8/24) and TWAP (7/24) | ADD methods |

---

## Appendix: Per-Exchange Trait Implementation Table

| Exchange | Trading | Account | CancelAll | AmendOrder | BatchOrders | Positions | LeverageControl | AdvancedOrders | AccountTransfers | CustodialFunds | SubAccounts |
|----------|---------|---------|-----------|------------|-------------|-----------|-----------------|----------------|------------------|----------------|-------------|
| Binance | Y | Y | Y | Y | Y(F) | Y | Y | Trailing,Iceberg | Y | Y | Y(broker) |
| Bybit | Y | Y | Y | Y | Y | Y | Y | Trailing,Iceberg | Y | Y | N(no create/list) |
| OKX | Y | Y | Y | Y | Y | Y | Y | All | Y | Y | Y(partial) |
| KuCoin | Y | Y | Y | Y(HF) | Y | Y | P(risk-level) | Iceberg | Y | Y | N |
| Kraken | Y | Y | Y | P | Y(S:15) | Y | Y | Trailing,Iceberg | Y | Y | N |
| Coinbase | Y | Y | P(batch) | P | N | Y(INTX) | P | Bracket,TWAP | Y | N | N |
| Gate.io | Y | Y | Y | Y | Y | Y | Y | Iceberg | Y | Y | N(partial) |
| Bitfinex | Y | Y | Y | Y | Y | Y | P | Trailing,Iceberg,OCO | Y | N | N(partial) |
| Bitstamp | Y | Y | Y | P | N | N | N | N | P | Y | N(institutional) |
| MEXC | Y | Y | Y | N | Y | Y | Y | N | Y | N | N(partial) |
| HTX | Y | Y | Y | N | Y | Y | Y | Trailing | Y | N | N |
| Bitget | Y | Y | Y | Y(F) | Y | Y | Y | Trailing,Bracket | Y | N | N |
| Gemini | Y | Y | Y | N | N | P | N | N | Y | Y | Y |
| BingX | Y | Y | Y | Y(F) | Y | Y | Y | Trailing,TWAP | Y | Y | Y |
| Phemex | Y | Y | Y | Y | N | Y | Y | Trailing,Bracket,Iceberg | Y | Y | N(partial) |
| Crypto.com | Y | Y | Y | Y | Y | Y | Y | OCO,Bracket | Y | Y | N(UI only) |
| Upbit | Y | Y | Y | P | N | N | N | N | N | Y | N |
| Deribit | Y | Y | Y | Y | N | Y | N | Trailing,OCO,Bracket,Iceberg | Y | Y | Y |
| HyperLiquid | Y | Y | P | Y | Y | Y | Y | TWAP | Y | N | P |
| Lighter | Y | Y | Y | Y | Y | Y | Y | TWAP,OCO(P) | Y | Y | Y |
| Jupiter | Y | Y | Y | N | N | P | N/A | N | N/A | N/A | N/A |
| GMX | Y | P | N | Y | Y(multicall) | Y | N | N | N/A | N/A | N/A |
| Paradex | Y | Y | Y | Y | Y | P | P | TWAP | N | N/A | N(partial) |
| dYdX V4 | Y | Y | N | N | P | Y | N | TWAP | Y | N/A | Y |
