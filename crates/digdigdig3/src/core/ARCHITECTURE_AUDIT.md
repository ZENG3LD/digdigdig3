# Architecture Audit: Thin-Trait + Fat-Enum Design
# digdigdig3 Multi-Exchange Connector Library

Generated: 2026-03-12
Source files audited: 9 trait files, 6 type files, 2 design documents, 2 connector examples.

---

## Table of Contents

1. [Core Design Philosophy](#core-design-philosophy)
2. [Trait Inventory — Full Method Signatures](#trait-inventory)
3. [Type Inventory — Fat Enums and Structs](#type-inventory)
4. [Capability Matrix Summary](#capability-matrix-summary)
5. [Minimal vs Maximal Connector](#minimal-vs-maximal-connector)
6. [Example Connectors: Binance (maximal) vs Polygon (minimal)](#example-connectors)
7. [Architecture Findings and Observations](#architecture-findings)

---

## Core Design Philosophy

Source: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/THIN_TRAIT_FAT_ENUM_DESIGN.md`

### Two Founding Principles

**1. Thin traits.** Each trait contains the fewest methods representing semantically distinct
operations. There is no separate `market_order` and `limit_order` — there is one `place_order`
whose behavior is determined by the `OrderType` variant in the fat enum. A connector either
supports a variant or returns `UnsupportedOperation`. That decision is made at runtime by matching
the enum, not at compile time by the method signature.

**2. Fat enums.** All complexity lives in enum variants with fields. An enum definition IS the
capability documentation. Reading `OrderType` shows the full universe of order types across all
24 exchanges. The enum IS the capability matrix, encoded in types.

### The Strict Non-Composition Rule

> Connectors MUST NEVER compose base methods to simulate missing features.
> If an exchange has no native endpoint for a capability, the connector returns
> `ExchangeError::UnsupportedOperation`. All composition belongs in higher layers.

This is a correctness requirement, not a style preference. Silent composition:
- Destroys atomicity guarantees (a real batch cancel is atomic; a loop is not)
- Corrupts rate-limit semantics (one batch request vs N individual requests)
- Deceives callers about what they are paying for in latency and cost

### Auth as Internal Implementation Detail

Auth is NOT a public trait surface. Each connector struct is responsible for its own signing.
The only public auth surface is:
- `ExchangeCredentials` enum — what credentials look like from outside
- `Authenticated` marker trait — so the connector manager can enforce "this connector has creds"

---

## Trait Inventory

File locations: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/`

### Trait Hierarchy Overview

```
ExchangeIdentity                    (root — all connectors)
    └── MarketData                  (public data — all connectors)
    └── Trading                     (order placement — trading connectors)
    └── Account                     (account info — trading connectors)
    └── Positions                   (futures — 22/24 connectors)

Optional operation traits (supertrait shown):
    └── CancelAll: Trading          (22/24)
    └── AmendOrder: Trading         (18/24)
    └── BatchOrders: Trading        (17/24)
    └── AccountTransfers: Account   (17/20 applicable)
    └── CustodialFunds: Account     (18/20 custodial)
    └── SubAccounts: Account        (~12/24)

Orthogonal:
    └── Authenticated               (marker — credential-aware connectors)
    └── WebSocketConnector          (streaming data)
    └── WebSocketExt                (blanket impl over WebSocketConnector)

Composite alias:
    └── CoreConnector = ExchangeIdentity + MarketData + Trading + Account + Positions
```

---

### `ExchangeIdentity`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/identity.rs`

Coverage: 100% — ALL connectors implement this trait.
Supertraits: `Send + Sync`

```rust
pub trait ExchangeIdentity: Send + Sync {
    fn exchange_id(&self) -> ExchangeId;
    fn exchange_name(&self) -> &'static str;       // default: exchange_id().as_str()
    fn is_testnet(&self) -> bool;
    fn supported_account_types(&self) -> Vec<AccountType>;
    fn exchange_type(&self) -> ExchangeType;       // default: exchange_id().exchange_type()
    fn metrics(&self) -> ConnectorStats;           // default: ConnectorStats::default()
}
```

**Input/Output types:**
| Method | Input | Output |
|--------|-------|--------|
| `exchange_id` | `&self` | `ExchangeId` |
| `exchange_name` | `&self` | `&'static str` |
| `is_testnet` | `&self` | `bool` |
| `supported_account_types` | `&self` | `Vec<AccountType>` |
| `exchange_type` | `&self` | `ExchangeType` |
| `metrics` | `&self` | `ConnectorStats` |

**Note:** `exchange_name` and `exchange_type` have default implementations; `metrics` defaults to
zeroed `ConnectorStats`. Only `exchange_id`, `is_testnet`, and `supported_account_types` are REQUIRED.

---

### `MarketData`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/market_data.rs`

Coverage: All connectors (both trading and data-only providers).
Supertraits: `ExchangeIdentity`
Auth required: NO — all methods are public.

```rust
#[async_trait]
pub trait MarketData: ExchangeIdentity {
    async fn get_price(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Price>;

    async fn get_orderbook(
        &self,
        symbol: Symbol,
        depth: Option<u16>,
        account_type: AccountType,
    ) -> ExchangeResult<OrderBook>;

    async fn get_klines(
        &self,
        symbol: Symbol,
        interval: &str,
        limit: Option<u16>,
        account_type: AccountType,
        end_time: Option<i64>,
    ) -> ExchangeResult<Vec<Kline>>;

    async fn get_ticker(
        &self,
        symbol: Symbol,
        account_type: AccountType,
    ) -> ExchangeResult<Ticker>;

    async fn ping(&self) -> ExchangeResult<()>;

    async fn get_exchange_info(
        &self,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<SymbolInfo>>;   // default: UnsupportedOperation
}
```

**Input/Output types:**
| Method | Input | Output |
|--------|-------|--------|
| `get_price` | `Symbol`, `AccountType` | `ExchangeResult<Price>` (= `f64`) |
| `get_orderbook` | `Symbol`, `Option<u16>`, `AccountType` | `ExchangeResult<OrderBook>` |
| `get_klines` | `Symbol`, `&str`, `Option<u16>`, `AccountType`, `Option<i64>` | `ExchangeResult<Vec<Kline>>` |
| `get_ticker` | `Symbol`, `AccountType` | `ExchangeResult<Ticker>` |
| `ping` | `&self` | `ExchangeResult<()>` |
| `get_exchange_info` | `AccountType` | `ExchangeResult<Vec<SymbolInfo>>` |

**Note:** `get_exchange_info` has a default implementation that returns `UnsupportedOperation`.
The other 5 methods are required. `end_time` in `get_klines` is an optional pagination cursor
(Unix ms); connectors that do not support it should accept it with `_end_time` and ignore it.

---

### `Trading`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/trading.rs`

Coverage: 24/24 trading exchanges (NOT data-only providers — they return UnsupportedOperation).
Supertraits: `ExchangeIdentity`
Auth required: YES — all methods require credentials.

```rust
#[async_trait]
pub trait Trading: ExchangeIdentity {
    async fn place_order(&self, req: OrderRequest) -> ExchangeResult<PlaceOrderResponse>;

    async fn cancel_order(&self, req: CancelRequest) -> ExchangeResult<Order>;

    async fn get_order(
        &self,
        symbol: &str,
        order_id: &str,
        account_type: AccountType,
    ) -> ExchangeResult<Order>;

    async fn get_open_orders(
        &self,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;

    async fn get_order_history(
        &self,
        filter: OrderHistoryFilter,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<Order>>;
}
```

**Input/Output types:**
| Method | Input | Output |
|--------|-------|--------|
| `place_order` | `OrderRequest` | `ExchangeResult<PlaceOrderResponse>` |
| `cancel_order` | `CancelRequest` | `ExchangeResult<Order>` |
| `get_order` | `&str`, `&str`, `AccountType` | `ExchangeResult<Order>` |
| `get_open_orders` | `Option<&str>`, `AccountType` | `ExchangeResult<Vec<Order>>` |
| `get_order_history` | `OrderHistoryFilter`, `AccountType` | `ExchangeResult<Vec<Order>>` |

**Fat enum dispatch:** `place_order` inspects `req.order_type` (an `OrderType` fat enum) and
`cancel_order` inspects `req.scope` (a `CancelScope` fat enum). Connectors match only the
variants they support natively.

**Strict rule for `get_open_orders`:** `symbol = None` fetches across all symbols. Exchanges
that do NOT support symbol-less open-order queries MUST return `UnsupportedOperation` for `None`,
not an empty `Vec`.

---

### `Account`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/account.rs`

Coverage: 24/24 trading exchanges (data providers return UnsupportedOperation).
Supertraits: `ExchangeIdentity`
Auth required: YES — all methods require credentials.

```rust
#[async_trait]
pub trait Account: ExchangeIdentity {
    async fn get_balance(&self, query: BalanceQuery) -> ExchangeResult<Vec<Balance>>;

    async fn get_account_info(&self, account_type: AccountType) -> ExchangeResult<AccountInfo>;

    async fn get_fees(&self, symbol: Option<&str>) -> ExchangeResult<FeeInfo>;
}
```

**Input/Output types:**
| Method | Input | Output |
|--------|-------|--------|
| `get_balance` | `BalanceQuery` | `ExchangeResult<Vec<Balance>>` |
| `get_account_info` | `AccountType` | `ExchangeResult<AccountInfo>` |
| `get_fees` | `Option<&str>` | `ExchangeResult<FeeInfo>` |

**Notes:**
- `get_fees` returns `UnsupportedOperation` for on-chain AMMs — these use
  protocol fee models not translatable to maker/taker.
- `get_balance` with `query.asset = None` returns all non-zero balances; `Some("BTC")` returns
  only the BTC entry.

---

### `Positions`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/positions.rs`

Coverage: 22/24 — Bitstamp (spot-only) and Gemini (spot-only) do NOT implement this trait.
Supertraits: `ExchangeIdentity`
Auth required: YES — all methods require credentials.

```rust
#[async_trait]
pub trait Positions: ExchangeIdentity {
    async fn get_positions(&self, query: PositionQuery) -> ExchangeResult<Vec<Position>>;

    async fn get_funding_rate(
        &self,
        symbol: &str,
        account_type: AccountType,
    ) -> ExchangeResult<FundingRate>;

    async fn modify_position(&self, req: PositionModification) -> ExchangeResult<()>;
}
```

**Input/Output types:**
| Method | Input | Output |
|--------|-------|--------|
| `get_positions` | `PositionQuery` | `ExchangeResult<Vec<Position>>` |
| `get_funding_rate` | `&str`, `AccountType` | `ExchangeResult<FundingRate>` |
| `modify_position` | `PositionModification` | `ExchangeResult<()>` |

**Fat enum dispatch:** `modify_position` inspects a `PositionModification` fat enum with 6
variants. Connectors match only what they support natively.

---

### `CancelAll`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`

Coverage: 43/43 active connectors — dYdX v4 (Cosmos tx-based, no bulk cancel) returns UnsupportedOperation by design.
Supertraits: `Trading`
Rule: Connectors implement this ONLY if the exchange has a native cancel-all endpoint. No looping
over `cancel_order` is permitted.

```rust
#[async_trait]
pub trait CancelAll: Trading {
    async fn cancel_all_orders(
        &self,
        scope: CancelScope,
        account_type: AccountType,
    ) -> ExchangeResult<CancelAllResponse>;
}
```

**Input/Output:** `CancelScope` (must be `All` or `BySymbol`), `AccountType` → `CancelAllResponse`

---

### `AmendOrder`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`

Coverage: 18/24 — Binance Futures, Bybit, OKX, KuCoin, GateIO, Bitfinex, MEXC, HTX, Bitget,
BingX, Phemex, CryptoCom, Deribit, HyperLiquid, Lighter, Paradex, dYdX, Upbit.
Supertraits: `Trading`

```rust
#[async_trait]
pub trait AmendOrder: Trading {
    async fn amend_order(&self, req: AmendRequest) -> ExchangeResult<Order>;
}
```

**Input/Output:** `AmendRequest` → `ExchangeResult<Order>`

---

### `BatchOrders`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`

Coverage: 17/24 — Binance, Bybit, OKX, KuCoin, GateIO, Bitfinex, MEXC, HTX, Bitget, BingX,
Phemex, CryptoCom, Deribit, HyperLiquid, Lighter, Paradex, dYdX.
Supertraits: `Trading`
Rule: ONLY when exchange has a native batch endpoint (one HTTP request for multiple orders).
NO sequential loops are permitted even as a fallback.

```rust
#[async_trait]
pub trait BatchOrders: Trading {
    async fn place_orders_batch(
        &self,
        orders: Vec<OrderRequest>,
    ) -> ExchangeResult<Vec<OrderResult>>;

    async fn cancel_orders_batch(
        &self,
        order_ids: Vec<String>,
        symbol: Option<&str>,
        account_type: AccountType,
    ) -> ExchangeResult<Vec<OrderResult>>;

    fn max_batch_place_size(&self) -> usize;

    fn max_batch_cancel_size(&self) -> usize;
}
```

**Input/Output:**
| Method | Input | Output |
|--------|-------|--------|
| `place_orders_batch` | `Vec<OrderRequest>` | `ExchangeResult<Vec<OrderResult>>` |
| `cancel_orders_batch` | `Vec<String>`, `Option<&str>`, `AccountType` | `ExchangeResult<Vec<OrderResult>>` |
| `max_batch_place_size` | `&self` | `usize` |
| `max_batch_cancel_size` | `&self` | `usize` |

**Note:** Individual failures within a batch are represented in `OrderResult::success = false`
rather than returning `Err` for the whole batch (partial success is common exchange behavior).

---

### `AccountTransfers`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`

Coverage: 17/20 applicable — DEX/non-custodial exchanges excluded.
Supertraits: `Account`

```rust
#[async_trait]
pub trait AccountTransfers: Account {
    async fn transfer(&self, req: TransferRequest) -> ExchangeResult<TransferResponse>;

    async fn get_transfer_history(
        &self,
        filter: TransferHistoryFilter,
    ) -> ExchangeResult<Vec<TransferResponse>>;
}
```

**Input/Output:**
| Method | Input | Output |
|--------|-------|--------|
| `transfer` | `TransferRequest` | `ExchangeResult<TransferResponse>` |
| `get_transfer_history` | `TransferHistoryFilter` | `ExchangeResult<Vec<TransferResponse>>` |

---

### `CustodialFunds`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`

Coverage: 18/20 custodial — DEX/non-custodial excluded.
Supertraits: `Account`

```rust
#[async_trait]
pub trait CustodialFunds: Account {
    async fn get_deposit_address(
        &self,
        asset: &str,
        network: Option<&str>,
    ) -> ExchangeResult<DepositAddress>;

    async fn withdraw(&self, req: WithdrawRequest) -> ExchangeResult<WithdrawResponse>;

    async fn get_funds_history(
        &self,
        filter: FundsHistoryFilter,
    ) -> ExchangeResult<Vec<FundsRecord>>;
}
```

**Input/Output:**
| Method | Input | Output |
|--------|-------|--------|
| `get_deposit_address` | `&str`, `Option<&str>` | `ExchangeResult<DepositAddress>` |
| `withdraw` | `WithdrawRequest` | `ExchangeResult<WithdrawResponse>` |
| `get_funds_history` | `FundsHistoryFilter` | `ExchangeResult<Vec<FundsRecord>>` |

---

### `SubAccounts`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/operations.rs`

Coverage: ~12/24 — Binance, Bybit, OKX, KuCoin, GateIO, MEXC, HTX, Bitget, BingX, Phemex,
Kraken, Bitfinex. DEX connectors never implement this.
Supertraits: `Account`

```rust
#[async_trait]
pub trait SubAccounts: Account {
    async fn sub_account_operation(
        &self,
        op: SubAccountOperation,
    ) -> ExchangeResult<SubAccountResult>;
}
```

**Input/Output:** `SubAccountOperation` fat enum → `ExchangeResult<SubAccountResult>`

---

### `Authenticated`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/auth.rs`

A marker trait — connectors that only support public endpoints (data providers) do NOT implement it.

```rust
pub trait Authenticated: Send + Sync {
    fn set_credentials(&mut self, creds: ExchangeCredentials);
    fn is_authenticated(&self) -> bool;
    fn credential_type(&self) -> Option<CredentialKind>;
}
```

**Input/Output:**
| Method | Input | Output |
|--------|-------|--------|
| `set_credentials` | `ExchangeCredentials` | `()` |
| `is_authenticated` | `&self` | `bool` |
| `credential_type` | `&self` | `Option<CredentialKind>` |

---

### `WebSocketConnector`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/websocket.rs`

Supertraits: `Send + Sync`
Note: NOT a supertrait of any core trait — implemented independently by connectors that support streaming.

```rust
#[async_trait]
pub trait WebSocketConnector: Send + Sync {
    async fn connect(&mut self, account_type: AccountType) -> WebSocketResult<()>;
    async fn disconnect(&mut self) -> WebSocketResult<()>;
    fn connection_status(&self) -> ConnectionStatus;
    async fn subscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>;
    async fn unsubscribe(&mut self, request: SubscriptionRequest) -> WebSocketResult<()>;
    fn event_stream(&self) -> Pin<Box<dyn Stream<Item = WebSocketResult<StreamEvent>> + Send>>;
    fn active_subscriptions(&self) -> Vec<SubscriptionRequest>;
    fn has_subscription(&self, request: &SubscriptionRequest) -> bool;  // default
    fn ping_rtt_handle(&self) -> Option<Arc<TokioMutex<u64>>>;          // default: None
}
```

### `WebSocketExt`
Blanket impl over all `WebSocketConnector` — convenience subscription helpers.
Provides: `subscribe_ticker`, `subscribe_trades`, `subscribe_orderbook`, `subscribe_klines`,
`subscribe_orders`, `subscribe_balance`, `subscribe_positions`.

---

### `CoreConnector` (composite alias)
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/mod.rs`

```rust
pub trait CoreConnector:
    ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync {}

// Blanket implementation:
impl<T> CoreConnector for T where
    T: ExchangeIdentity + MarketData + Trading + Account + Positions + Send + Sync {}
```

Used for generic code that needs to work with any full-featured exchange connector.
Data-only providers (e.g. Polygon) do NOT satisfy `CoreConnector` because they return
`UnsupportedOperation` from `Trading`, `Account`, and `Positions` — they satisfy the trait
bounds syntactically but violate the semantic contract.

---

## Type Inventory

File locations: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/`

### `OrderType` — Primary Fat Enum
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/trading.rs`

This is the central fat enum. Reading it is reading the capability matrix for order types.

```
Variant          Coverage    Fields
──────────────────────────────────────────────────────────────────
Market           24/24       (none)
Limit            24/24       price: Price
StopMarket       19/24       stop_price: Price
StopLimit        19/24       stop_price: Price, limit_price: Price
TrailingStop     10/24       callback_rate: f64, activation_price: Option<Price>
Oco              7/24        price: Price, stop_price: Price, stop_limit_price: Option<Price>
Bracket          9/24        price: Option<Price>, take_profit: Price, stop_loss: Price
Iceberg          8/24        price: Price, display_quantity: Quantity
Twap             7/24        duration_seconds: u64, interval_seconds: Option<u64>
PostOnly         20/24       price: Price
Ioc              21/24       price: Option<Price>
Fok              17/24       price: Price
Gtd              8/24        price: Price, expire_time: Timestamp
ReduceOnly       19/24       price: Option<Price>
```

---

### `CancelScope` — Fat Enum for cancel dispatch
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/trading.rs`

```
Variant     Coverage    Fields
────────────────────────────────────────────────────────────
Single       24/24      order_id: String
Batch        17/24      order_ids: Vec<String>
All          22/24      symbol: Option<Symbol>
BySymbol     22/24      symbol: Symbol
```

---

### `PositionModification` — Fat Enum for position mutations
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/trading.rs`

```
Variant          Coverage    Fields
──────────────────────────────────────────────────────────────────────────────────
SetLeverage      19/24       symbol: Symbol, leverage: u32, account_type: AccountType
SetMarginMode    16/24       symbol: Symbol, margin_type: MarginType, account_type: AccountType
AddMargin        12/24       symbol: Symbol, amount: Quantity, account_type: AccountType
RemoveMargin     10/24       symbol: Symbol, amount: Quantity, account_type: AccountType
ClosePosition    22/24       symbol: Symbol, account_type: AccountType
SetTpSl          15/24       symbol: Symbol, take_profit: Option<Price>, stop_loss: Option<Price>, account_type: AccountType
```

---

### `SubAccountOperation` — Fat Enum
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/trading.rs`

```
Variant      Coverage    Fields
────────────────────────────────────────────────────────────────────────────────────────
Create       ~10/24      label: String
List         ~12/24      (none)
Transfer     ~10/24      sub_account_id: String, asset: Asset, amount: Quantity, to_sub: bool
GetBalance   ~10/24      sub_account_id: String
```

---

### `PlaceOrderResponse` — Fat Enum for order placement response
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/responses.rs`

```
Variant     Contents            Used for
──────────────────────────────────────────────────────────────────────────
Simple      Order               Market, Limit, StopMarket, StopLimit,
                                TrailingStop, PostOnly, IOC, FOK, GTD,
                                ReduceOnly, Iceberg
Bracket     BracketResponse     Bracket (entry + TP + SL)
Oco         OcoResponse         OCO (2-leg pair)
Algo        AlgoOrderResponse   TWAP and other algorithmic orders
```

---

### `ExchangeCredentials` — Auth Fat Enum
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/trading.rs`

```
Variant                 Coverage    Fields
──────────────────────────────────────────────────────────────────────────────
HmacSha256              12/24       api_key: String, api_secret: String
HmacWithPassphrase       3/24       api_key, api_secret, passphrase: String
HmacSha512               1/24       api_key: String, api_secret: String
HmacSha384               1/24       api_key: String, api_secret: String
JwtEs256                 1/24       api_key: String, private_key_pem: String
JwtHmac                  1/24       api_key: String, secret: String
OAuth2                   1/24       access_token: String, refresh_token: Option<String>
EthereumWallet           2/24       private_key_hex: String, address: Option<String>
SolanaKeypair            1/24       private_key_b58: String
StarkKey                 2/24       stark_private_key: String, ethereum_address: Option<String>
CosmosWallet             1/24       mnemonic: String, derivation_path: Option<String>
```

---

### `CredentialKind` — Auth descriptor enum (in traits/auth.rs)
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/traits/auth.rs`

Used for capability discovery (does this connector accept Ethereum wallet credentials?).
Contains the same variants as `ExchangeCredentials` but without data fields — it is a pure
descriptor used in `Authenticated::credential_type()`.

Variants: `HmacSha256`, `HmacWithPassphrase`, `HmacSha512`, `HmacSha384`, `JwtEs256`,
`JwtHmac`, `OAuth2`, `EthereumWallet`, `SolanaKeypair`, `StarkKey`, `CosmosWallet`

---

### `ExchangeId` — Exchange identifier enum
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/common.rs`

Comprehensive identifier for all supported sources:
- **CEX crypto:** Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, GateIO, Bitfinex, Bitstamp,
  Gemini, MEXC, HTX, Bitget, BingX, Phemex, CryptoCom, Upbit, Deribit, HyperLiquid
- **DEX:** Lighter, Paradex, Dydx
- **Prediction markets:** Polymarket
- **Data providers (stocks/crypto/forex/econ):** Polygon, Finnhub, Tiingo, Twelvedata,
  Coinglass, CryptoCompare, WhaleAlert, Bitquery, DefiLlama, Oanda, AlphaVantage,
  Dukascopy, AngelOne, Zerodha, Fyers, Dhan, Upstox, Alpaca, JQuants, Tinkoff, Moex,
  Krx, Fred, Bls, YahooFinance, Ib
- **Custom(u16)**

**Disabled:** Bithumb (infrastructure issues), Vertex (shut down Aug 2025), Futu (TCP + protobuf — incompatible)

---

### `AccountType`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/common.rs`

```
Spot | Margin | FuturesCross | FuturesIsolated
```

---

### `StreamType` — WebSocket stream type enum
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/websocket.rs`

```
Public:   Ticker, Trade, Orderbook, OrderbookDelta, Kline { interval: String }, MarkPrice, FundingRate
Private:  OrderUpdate, BalanceUpdate, PositionUpdate
```

---

### `StreamEvent` — WebSocket event fat enum
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/websocket.rs`

```
Public events:
  Ticker(Ticker)
  Trade(PublicTrade)
  OrderbookSnapshot(OrderBook)
  OrderbookDelta { bids, asks, timestamp }
  Kline(Kline)
  MarkPrice { symbol, mark_price, index_price, timestamp }
  FundingRate { symbol, rate, next_funding_time, timestamp }

Private events:
  OrderUpdate(OrderUpdateEvent)
  BalanceUpdate(BalanceUpdateEvent)
  PositionUpdate(PositionUpdateEvent)
```

---

### `ExchangeError`
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/core/types/common.rs`

```
Http(String)
Network(String)
Parse(String)
ParseError(String)      <- duplicate of Parse, should be consolidated
Api { code: i32, message: String }
RateLimit
RateLimitExceeded { retry_after: Option<u64>, message: String }
Auth(String)
InvalidCredentials(String)
PermissionDenied(String)
InvalidRequest(String)
NotSupported(String)    <- near-duplicate of UnsupportedOperation
UnsupportedOperation(String)
Timeout(String)
NotFound(String)
```

**Note:** `Parse` and `ParseError` are duplicates. `NotSupported` and `UnsupportedOperation`
are functionally equivalent — only `UnsupportedOperation` is used by connectors; `NotSupported`
appears to be a legacy variant.

---

### Key Plain Structs

| Struct | File | Purpose |
|--------|------|---------|
| `Order` | trading.rs | Single order state |
| `OrderRequest` | trading.rs | Input to `place_order` |
| `CancelRequest` | trading.rs | Input to `cancel_order` (contains `CancelScope`) |
| `AmendRequest` | trading.rs | Input to `amend_order` (contains `AmendFields`) |
| `OrderHistoryFilter` | trading.rs | Input to `get_order_history` |
| `PositionQuery` | trading.rs | Input to `get_positions` |
| `BalanceQuery` | trading.rs | Input to `get_balance` |
| `TransferRequest` | trading.rs | Input to `transfer` |
| `WithdrawRequest` | trading.rs | Input to `withdraw` |
| `FundsHistoryFilter` | trading.rs | Input to `get_funds_history` |
| `Position` | trading.rs | Open futures position |
| `Balance` | trading.rs | Asset balance |
| `AccountInfo` | trading.rs | Account permissions + commission |
| `Kline` | market_data.rs | OHLCV candle |
| `Ticker` | market_data.rs | 24h market stats |
| `OrderBook` | market_data.rs | Bid/ask depth snapshot |
| `FundingRate` | market_data.rs | Perpetual funding rate |
| `Symbol` | common.rs | base/quote pair + optional raw string |
| `ConnectorStats` | common.rs | Runtime HTTP/rate metrics |
| `SymbolInfo` | trading.rs | Exchange symbol metadata |
| `OrderResult` | responses.rs | Single result within a batch operation |
| `CancelAllResponse` | responses.rs | Result of `cancel_all_orders` |
| `BracketResponse` | responses.rs | 3-leg bracket order result |
| `OcoResponse` | responses.rs | 2-leg OCO order result |
| `AlgoOrderResponse` | responses.rs | TWAP / algo order task result |
| `DepositAddress` | responses.rs | On-chain deposit address |
| `WithdrawResponse` | responses.rs | Withdrawal submission result |
| `FundsRecord` | responses.rs | Enum: Deposit or Withdrawal record |
| `FeeInfo` | responses.rs | Maker/taker fee rates |
| `TransferResponse` | responses.rs | Internal transfer result |

---

## Capability Matrix Summary

Source: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/TRADING_CAPABILITY_MATRIX.md`

### Tier Classification

| Tier | Count | Trait assignment | Examples |
|------|-------|------------------|---------|
| UNIVERSAL (22–24/24) | Market, Limit, single cancel, get open, get history, balances | Core trait methods — must implement | All 24 |
| COMMON (15–21/24) | StopMarket, StopLimit, CancelAll, Amend, Batch, Funding, Positions | Optional operation traits | Most CEX |
| SPECIALIZED (8–14/24) | TrailingStop, OCO, GTD, SetLeverage, MarginMode, Deposit/Withdraw | Optional trait extensions | Larger CEX + derivatives |
| RARE (3–7/24) | Bracket, TWAP, SubAccount Create/List | Exchange-specific traits | Deribit, OKX, BingX |
| ULTRA-RARE (1–2/24) | Block trades, MassQuote, ZK privacy | Not in traits — handled at exchange level | Deribit, Paradex |

---

## Minimal vs Maximal Connector

Source: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/TRADING_CAPABILITY_MATRIX.md`
Section 3: Exchange Rankings

### Maximum Exchange: Deribit (Rank 1, ~48/55 capabilities)

Defines the "all optional traits implemented" reference:
- Full `OrderType` suite: Market, Limit, StopMarket, StopLimit, TrailingStop, TP, SL, OCO,
  Bracket/OTOCO, OTO, Market-Limit
- All `TimeInForce`: GTC, IOC, FOK, PostOnly, GTD
- Full order management: `CancelAll`, `AmendOrder`, `BatchOrders` (cancel batch), cancel by symbol
- Full `Positions`: GetPos, Close, FundingRate, LiqPrice, SetLeverage (NOTE: no SetLeverage on Deribit)
- Full `Account`: balances, fees, transfer, deposit, withdraw, history
- Full `SubAccounts`: create, list, transfer
- Advanced: Iceberg, BlockTrades, BlockRFQ, MassQuote
- Auth: OAuth2 client_credentials (`ExchangeCredentials::OAuth2` — Deribit uses `public/auth` flow)

OKX is the runner-up (Rank 2, ~45/55 capabilities) and is cited as "top CEX runner-up" with
identical coverage on most dimensions except GTD and sub-account list.

### Minimum Exchange — CEX (Rank 24): Upbit (~20/55 capabilities)

Defines the absolute semantic floor for the core traits:
- `OrderType`: Market and Limit only — NO StopMarket, StopLimit, TrailingStop, TP/SL, OCO, Bracket
- `TimeInForce`: IOC, FOK, PostOnly — no GTD, GTC only implicit
- `Trading` but NOT `CancelAll` (no batch cancel-all endpoint)
- No `BatchOrders`
- No `AmendOrder` (no true amend)
- No `Positions` (spot-only exchange)
- No `AccountTransfers` (no internal transfer)
- No `SubAccounts`
- Has `CustodialFunds` (deposit/withdraw supported)
- Auth: JWT-HMAC

In `Trading::place_order`, an Upbit connector matches only:
```rust
OrderType::Market => { /* ... */ }
OrderType::Limit { price } => { /* ... */ }
_ => Err(ExchangeError::UnsupportedOperation(...))
```

### Minimum Connector — Data Provider: Polygon (structural minimum)

Data-only providers define the absolute structural floor:
- Implements `ExchangeIdentity` + `MarketData` only (functionally)
- Implements `Trading`, `Account`, `Positions` syntactically (required for `CoreConnector`) but
  every method returns `ExchangeError::UnsupportedOperation`
- No `CancelAll`, `AmendOrder`, `BatchOrders`, `AccountTransfers`, `CustodialFunds`, `SubAccounts`
- No `Authenticated` (API key passed at construction, auth not user-configurable)

---

## Example Connectors

### Binance — Maximal CEX Connector
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/crypto/cex/binance/connector.rs`

Ranked #3 overall (~44/55 capabilities). Full `CoreConnector` implementation.

**Struct fields:**
- `http: HttpClient` — shared HTTP client
- `auth: Option<BinanceAuth>` — optional credentials (None = public-only mode)
- `urls: BinanceUrls` — mainnet/testnet URL sets
- `testnet: bool`
- `weight_limiter: Arc<Mutex<WeightRateLimiter>>` — 6000 weight/minute rate limiter

**Construction:** `BinanceConnector::new(credentials: Option<Credentials>, testnet: bool)`
Also: `BinanceConnector::public(testnet)` — convenience constructor, no credentials.

**ExchangeIdentity implementation:**
```rust
fn exchange_id(&self)              -> ExchangeId::Binance
fn is_testnet(&self)               -> self.testnet
fn supported_account_types(&self)  -> [Spot, Margin, FuturesCross, FuturesIsolated]
fn exchange_type(&self)            -> ExchangeType::Cex
fn metrics(&self)                  -> ConnectorStats { http_requests, http_errors,
                                       last_latency_ms, rate_used, rate_max, ... }
```

**MarketData implementation:** All 6 methods are fully implemented.
- `get_price`: dispatches to SpotPrice or FuturesPrice endpoint based on `AccountType`
- `get_orderbook`: dispatches to SpotOrderbook or FuturesOrderbook
- `get_klines`: dispatches to SpotKlines or FuturesKlines; supports `end_time` pagination
- `get_ticker`: dispatches to SpotTicker or FuturesTicker
- `ping`: calls Ping endpoint
- `get_exchange_info`: dispatches to SpotExchangeInfo or FuturesExchangeInfo, returns only
  `status == "TRADING"` symbols

Binance also exposes a non-trait method `get_klines_paginated` for multi-page historical fetches.

**Trading::place_order — match arms:**
```rust
match req.order_type {
    OrderType::Market => {
        // POST SpotCreateOrder or FuturesCreateOrder
        // Returns PlaceOrderResponse::Simple(order)
    }
    OrderType::Limit { price } => {
        // POST SpotCreateOrder or FuturesCreateOrder with timeInForce="GTC"
        // Returns PlaceOrderResponse::Simple(order)
    }
    _ => Err(ExchangeError::UnsupportedOperation(
        format!("{:?} order type not supported on {:?}", req.order_type, self.exchange_id())
    ))
}
```

**Note:** This is a partial implementation — only Market and Limit are handled. The connector
correctly returns `UnsupportedOperation` for all other variants (StopMarket, TrailingStop, OCO,
Bracket, Iceberg, TWAP, PostOnly, IOC, FOK, GTD, ReduceOnly) even though Binance supports many
of them. This appears to be work-in-progress.

**Trading::cancel_order — match arms:**
```rust
match req.scope {
    CancelScope::Single { ref order_id } => {
        // DELETE SpotCancelOrder or FuturesCancelOrder
        // Returns the cancelled Order
    }
    _ => Err(ExchangeError::UnsupportedOperation(
        format!("{:?} cancel scope not supported on {:?}", req.scope, self.exchange_id())
    ))
}
```

Only `CancelScope::Single` is handled. `Batch`, `All`, `BySymbol` return `UnsupportedOperation`
(despite Binance having native CancelAll — indicates these would be in a separate `CancelAll`
trait implementation that does not yet exist).

**Trading::get_order_history:** Returns `UnsupportedOperation("not yet implemented")` — explicitly
stubbed as work-in-progress.

**Account implementation:**
- `get_balance`: dispatches to SpotAccount or FuturesAccount; uses `omitZeroBalances` param for Spot.
  Parses with `BinanceParser::parse_balances` or `parse_futures_balances`.
- `get_account_info`: similar dispatch; manually extracts `canTrade`, `canWithdraw`, `canDeposit`,
  `commissionRates` from JSON response.
- `get_fees`: Returns `UnsupportedOperation("not yet implemented")` — stubbed.

**Positions implementation:**
- `get_positions`: validates account type (returns `UnsupportedOperation` for Spot/Margin);
  calls FuturesPositions endpoint.
- `get_funding_rate`: validates account type; calls FundingRate endpoint with `limit=1`.
- `modify_position — match arms:**
```rust
match req {
    PositionModification::SetLeverage { ref symbol, leverage, account_type } => {
        // POST FuturesSetLeverage
        // Returns Ok(())
    }
    _ => Err(ExchangeError::UnsupportedOperation(
        format!("{:?} not supported on {:?}", req, self.exchange_id())
    ))
}
```

Only `SetLeverage` is handled. `SetMarginMode`, `AddMargin`, `RemoveMargin`, `ClosePosition`,
`SetTpSl` all return `UnsupportedOperation` — work-in-progress stubs.

**Rate limiting:** Weight-based `WeightRateLimiter` (6000/minute). Updates from server via
`X-MBX-USED-WEIGHT-1M` response header. Per-endpoint weights hard-coded in `weights` module.

---

### Polygon — Minimal Data-Only Connector
File: `c:/Users/VA PC/CODING/ML_TRADING/nemo/digdigdig3/src/stocks/us/polygon/connector.rs`

The canonical example of a read-only data provider connector.

**Struct fields:**
- `http: HttpClient`
- `auth: PolygonAuth` — always required (API key at construction), not user-configurable at runtime
- `urls: PolygonUrls`
- `_realtime: bool` — flag (prefixed `_` = currently unused)
- `rate_limiter: Arc<Mutex<WeightRateLimiter>>` — 5 req/min for free tier

**Construction:** `PolygonConnector::new(credentials: Credentials, realtime: bool)` — credentials
always required, no public/anonymous mode.

**ExchangeIdentity implementation:**
```rust
fn exchange_id(&self)              -> ExchangeId::Polygon
fn is_testnet(&self)               -> false  // Polygon has no testnet
fn supported_account_types(&self)  -> [AccountType::Spot]  // compatibility default
fn exchange_type(&self)            -> ExchangeType::Cex    // NOTE: should be DataProvider
```

**Note:** `exchange_type()` returns `ExchangeType::Cex` but `ExchangeId::Polygon.exchange_type()`
returns `ExchangeType::DataProvider`. The connector overrides this with an incorrect value — the
default delegation to `exchange_id().exchange_type()` (which returns `DataProvider`) would be
more accurate. The explicit override `ExchangeType::Cex` with comment "Data provider" is a bug
or documentation error.

**MarketData implementation:** All 6 methods are fully implemented.
- `get_price`: calls `PreviousClose` endpoint (free tier compatible); extracts `"c"` (close price)
  from first result in array. Uses only `symbol.base` as ticker (stock ticker = base asset only).
- `get_orderbook`: calls `SingleSnapshot` endpoint; parses via `PolygonParser::parse_orderbook`.
- `get_klines`: calls `Aggregates` endpoint with date range (default: last 30 days); maps
  `interval` string to Polygon's `timespan` + `multiplier` params; ignores `end_time` parameter.
- `get_ticker`: calls `SingleSnapshot` endpoint.
- `ping`: calls `MarketStatus` endpoint; checks `"status" == "OK"`.
- `get_exchange_info`: calls `Tickers` endpoint with `market=stocks&active=true&limit=1000`;
  returns `Vec<SymbolInfo>` where each entry uses ticker as `base_asset`, USD as `quote_asset`.

**Trading implementation — ALL methods return UnsupportedOperation:**
```rust
impl Trading for PolygonConnector {
    async fn place_order(&self, _req: OrderRequest) -> ExchangeResult<PlaceOrderResponse> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Trading is not supported."
        ))
    }
    // cancel_order, get_order, get_open_orders, get_order_history — same pattern
}
```

**Account implementation — ALL methods return UnsupportedOperation:**
```rust
impl Account for PolygonConnector {
    async fn get_balance(&self, _query: BalanceQuery) -> ExchangeResult<Vec<Balance>> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Account operations are not supported."
        ))
    }
    // get_account_info, get_fees — same pattern
}
```

**Positions implementation — ALL methods return UnsupportedOperation:**
```rust
impl Positions for PolygonConnector {
    async fn get_positions(&self, _query: PositionQuery) -> ExchangeResult<Vec<Position>> {
        Err(ExchangeError::UnsupportedOperation(
            "Polygon is a data provider, not an exchange. Position operations are not supported."
        ))
    }
    // get_funding_rate, modify_position — same pattern
}
```

**Optional traits NOT implemented:** `CancelAll`, `AmendOrder`, `BatchOrders`,
`AccountTransfers`, `CustodialFunds`, `SubAccounts`, `Authenticated`, `WebSocketConnector`.

---

## Architecture Findings and Observations

### Confirmed Design Correctness

1. **Fat-enum dispatch works cleanly.** The `match req.order_type { ... _ => UnsupportedOperation }`
   pattern is consistently applied in both connectors. The enum IS the capability documentation.

2. **Non-composition rule is enforced.** Binance's `cancel_order` returns `UnsupportedOperation`
   for `CancelScope::Batch` and `CancelScope::All` rather than looping `Single`. The rule is upheld.

3. **Thin trait surface is genuinely thin.** Five methods in `Trading`, three in `Account`,
   three in `Positions`. Contrast with the `TRADING_CAPABILITY_MATRIX.md`'s proposed `BaseTrade`
   which merges balance, fees, and trading into one trait — the final design correctly splits
   these into separate `Trading` and `Account` traits.

4. **Auth is correctly internal.** Neither connector exposes signing in the trait surface.
   `BinanceAuth` and `PolygonAuth` are private struct members. Auth headers are applied inside
   the `get()`/`post()`/`delete()` private methods.

5. **Rate limiting is per-connector, not per-trait.** Each connector manages its own
   `WeightRateLimiter` with exchange-specific weights. This is correct — rate limit semantics
   differ dramatically across exchanges.

### Divergences from Design Document

1. **`ExchangeCredentials` naming discrepancy.** The design document (`THIN_TRAIT_FAT_ENUM_DESIGN.md`)
   defines `HmacSha256WithPassphrase` as a variant name; the actual implementation in
   `types/trading.rs` uses `HmacWithPassphrase`. Similarly, the design doc has `KrakenDual` which
   does NOT appear in the actual `ExchangeCredentials` enum — Kraken is mapped to `HmacSha512`.

2. **`CredentialKind` vs `CredentialType`.** The design document defines `CredentialType` as the
   descriptor enum name; the actual code uses `CredentialKind` in `traits/auth.rs`.

3. **`Authenticated` trait has extra methods in implementation.** The design doc defines
   `Authenticated` with only `credential_type() -> CredentialType`. The actual implementation has
   three methods: `set_credentials`, `is_authenticated`, and `credential_type`.

4. **Binance is work-in-progress, not "maximal".** The TRADING_CAPABILITY_MATRIX.md ranks Binance
   as #3 with ~44/55 capabilities, but the actual `BinanceConnector` only implements 2 of 14
   `OrderType` variants, stubs `get_order_history` and `get_fees` as `UnsupportedOperation`,
   and only handles `SetLeverage` in `modify_position`. The connector is functionally partial.

### Issues Identified

1. **`ExchangeError` duplicate variants:** `Parse` and `ParseError` are both present and have
   identical `#[error]` strings ("Parse error: {0}"). One should be removed.

2. **`ExchangeError::NotSupported` vs `UnsupportedOperation`:** Both exist and are used
   inconsistently. Connectors use `UnsupportedOperation` exclusively; `NotSupported` appears to
   be a legacy variant that should be removed or aliased.

3. **`PolygonConnector::exchange_type()` returns wrong value.** Returns `ExchangeType::Cex`
   instead of `ExchangeType::DataProvider`. The default delegation in `ExchangeIdentity` would
   return the correct value. The explicit override should be removed.

4. **`CoreConnector` semantic gap.** Polygon satisfies `CoreConnector` trait bounds at compile
   time (it implements all required traits), but violates the semantic contract at runtime (all
   `Trading`, `Account`, `Positions` methods return `UnsupportedOperation`). The type system
   cannot distinguish a "real" CoreConnector from a data provider pretending to be one. Consider
   adding separate `DataConnector` and `TradingConnector` top-level markers, or checking
   `exchange_type() == DataProvider` at the callsite.

5. **Binance `CancelAll`, `BatchOrders`, etc. not yet implemented.** The trait structs exist in
   `operations.rs` with coverage of 22/24 and 17/24 respectively, but there is no
   `impl CancelAll for BinanceConnector` block in the connector file. These would be in separate
   `impl` blocks per the optional trait design, but they are absent.

6. **`_realtime` field in PolygonConnector is unused** (prefixed with `_`). This should either
   be removed or connected to routing logic (e.g. real-time WebSocket vs REST polling).

### Summary: What the Architecture Gets Right

- The thin-trait fat-enum pattern cleanly separates "what operation" (method) from "how the
  operation is parameterized" (enum variant). This is the key architectural win.
- Optional operation traits (`CancelAll`, `AmendOrder`, `BatchOrders`, etc.) as separate trait
  objects are the correct way to express optional capabilities without polluting the core interface.
- The `WebSocketConnector` being orthogonal to the REST trait hierarchy is correct — not all
  streaming and REST clients are co-located.
- Per-connector internal rate limiting is the right model — centralized rate limiting would
  require exchange-specific knowledge in the shared layer.
- The `PlaceOrderResponse` fat enum (Simple / Bracket / Oco / Algo) correctly handles the fact
  that composite order types return multiple orders, not one.
