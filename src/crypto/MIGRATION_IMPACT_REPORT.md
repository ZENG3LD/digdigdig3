# Migration Impact Report: V1 Traits → V2 Traits

**Date**: 2026-03-12
**Scope**: `digdigdig3/src/` — full search of all connector files

---

## 1. Executive Summary

The codebase currently has two coexisting trait generations:

| Generation | Traits | Status |
|------------|--------|--------|
| **V1** (old) | `Trading`, `Account`, `Positions`, `ExchangeAuth`, `BatchOperations`, `AdvancedOrders`, `MarginTrading`, `Transfers` | Active in ALL 50+ connectors. Never replaced. |
| **V2** (new) | `TradingV2`, `AccountV2`, `PositionsV2`, `CancelAllV2`, `AmendOrderV2`, `BatchOrdersV2`, `AccountTransfersV2`, `CustodialFundsV2`, `SubAccountsV2`, `Authenticated` | **Defined but implemented by ZERO connectors.** Dead code. |

**The V2 traits are fully designed but no connector implements any of them yet.** Migration is greenfield.

---

## 2. V1 Trait Usage — Who Implements What

### 2.1 `Trading` (V1)

The `Trading` trait defines: `market_order`, `limit_order`, `cancel_order`, `get_order`, `get_open_orders`.

**All 50+ connectors implement `impl Trading for <Connector>`. This is the most-implemented trait in the system.**

Full list of connectors implementing `Trading`:

**Crypto CEX (19):**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\binance\connector.rs:532`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bybit\connector.rs:423`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\okx\connector.rs:358`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\kucoin\connector.rs:565`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\kraken\connector.rs:367`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\coinbase\connector.rs:498`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\gateio\connector.rs:505`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bitfinex\connector.rs:356`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bitstamp\connector.rs:373`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\gemini\connector.rs:344`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\mexc\connector.rs:522`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\htx\connector.rs:501`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bitget\connector.rs:441`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bingx\connector.rs:362`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\phemex\connector.rs:486`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\crypto_com\connector.rs:335`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\upbit\connector.rs:427`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\deribit\connector.rs:472`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\vertex\connector.rs:424`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bithumb\connector.rs:337`

**Crypto DEX (4):**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\paradex\connector.rs:500`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\lighter\connector.rs:407`

**Stocks — India (5):**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\zerodha\connector.rs:292`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\upstox\connector.rs:464`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\fyers\connector.rs:357`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\dhan\connector.rs:433`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\angel_one\connector.rs:578`

**Stocks — US (5):**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\us\polygon\connector.rs:395`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\us\finnhub\connector.rs:330`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\us\tiingo\connector.rs:345`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\us\twelvedata\connector.rs:262`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\us\alpaca\connector.rs:496`

**Stocks — Other:**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\russia\tinkoff\connector.rs:426`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\russia\moex\connector.rs:298`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\korea\krx\connector.rs:371`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\japan\jquants\connector.rs:266`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\china\futu\connector.rs:137`

**Forex (3):**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\forex\oanda\connector.rs:400`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\forex\dukascopy\connector.rs:337`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\forex\alphavantage\connector.rs:243`

**Aggregators/Feeds (5):**
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\aggregators\yahoo\connector.rs:300`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\aggregators\cryptocompare\connector.rs:228`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\intelligence_feeds\crypto\coinglass\connector.rs:364`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\intelligence_feeds\economic\fred\connector.rs:1493`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\onchain\analytics\whale_alert\connector.rs:337`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\onchain\analytics\bitquery\connector.rs:481`

**Total: 50+ connectors implement `Trading`.**
Note: most non-trading connectors (data feeds, aggregators) implement it with stub bodies returning `UnsupportedOperation`.

---

### 2.2 `Account` (V1)

Same scope as `Trading` — every connector also implements `impl Account for <Connector>`.

All the same files as `Trading` — 50+ connectors. The `Account` V1 trait defines:
- `get_balance(asset: Option<Asset>, account_type: AccountType) -> Vec<Balance>`
- `get_account_info(account_type: AccountType) -> AccountInfo`

Data-only connectors (Polygon, Finnhub, Yahoo, FRED, etc.) stub these with `UnsupportedOperation`.

---

### 2.3 `Positions` (V1)

Same scope — 50+ connectors implement `impl Positions for <Connector>`. Key file refs:
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\binance\connector.rs:726`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\cex\bybit\connector.rs:674`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\crypto\dex\paradex\connector.rs:631`
- All other connectors listed above at their respective line numbers.

---

### 2.4 `ExchangeAuth` (V1)

Usage is narrower — `ExchangeAuth` is the auth-signing trait. It is referenced:
- **Defined in**: `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\auth.rs:100`
- **Imported in factory**: `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\factory.rs:58` — imports `Credentials` (not the trait itself)
- **`ExchangeAuth` trait itself**: no `impl ExchangeAuth for` found in connectors. Each exchange has its own auth struct (e.g. `BinanceAuth`, `KuCoinAuth`) that implement the trait internally within `auth.rs` files.
- **`SignatureLocation`**: used only in the default implementation in `auth.rs:112` — not referenced by connector code directly.
- **`AuthRequest`**: used only in trait definition and `auth.rs` itself — not imported by any connector outside of the `auth.rs` files within each exchange.

---

## 3. Extension Traits (V1) — Implementation Coverage

The extension traits are defined in:
`c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\extensions.rs`

### Critical Finding: ZERO connectors implement any V1 extension trait

```
grep "impl BatchOperations for|impl AdvancedOrders for|impl MarginTrading for|impl Transfers for"
→ NO MATCHES
```

**`BatchOperations`** — defined with default sequential implementations (calls `create_limit_order` — which is itself a **bug**: `Trading` has `limit_order`, not `create_limit_order`). Never implemented by any connector.

**`AdvancedOrders`** — all methods have default `UnsupportedOperation` returns. Never implemented.

**`MarginTrading`** — all methods have default `UnsupportedOperation` returns. Never implemented.

**`Transfers`** — `transfer()` is required (no default), `get_transfer_history()` defaults to `UnsupportedOperation`. Never implemented.

**Conclusion**: V1 extension traits are **pure dead code**. They can be deleted immediately without breaking any connector.

**Additional bug in `extensions.rs`**: `BatchOperations::create_orders_batch` calls `self.create_limit_order(...)` at line 57, but `Trading` defines `limit_order`, not `create_limit_order`. This code does not compile if any connector tried to implement `BatchOperations`.

---

## 4. V2 Traits — Implementation Coverage

### Finding: ZERO connectors implement any V2 trait

```
grep "impl TradingV2 for|impl AccountV2 for|impl PositionsV2 for|impl CancelAllV2 for|
      impl AmendOrderV2 for|impl BatchOrdersV2 for|impl Authenticated for"
→ NO MATCHES
```

V2 traits defined in:
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\trading_v2.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\account_v2.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\positions_v2.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\operations_v2.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\auth_v2.rs`

All V2 traits are exported from `mod.rs` via `pub use`. `CoreConnectorV2` composite trait is defined. But the migration is **0% complete** at the connector level.

---

## 5. `OrderType` Enum Usage

`OrderType::` is used heavily across parsers and connectors. Variants observed in real code:

| Variant | File Examples |
|---------|--------------|
| `OrderType::Market` | `zerodha/parser.rs:153`, `alpaca/parser.rs:261`, `zerodha/connector.rs:375` |
| `OrderType::Limit` | `zerodha/parser.rs:188`, `alpaca/parser.rs:262`, `upstox/parser.rs:261` |
| `OrderType::StopLoss` | `alpaca/parser.rs:263,265` |
| `OrderType::StopLossLimit` | `alpaca/parser.rs:264` |

`OrderType` is referenced across parsers in:
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\zerodha\parser.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\us\alpaca\parser.rs`
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\stocks\india\upstox\parser.rs`
- And many more crypto parsers — full output is 24KB (see full grep results).

**`OrderType` is actively used. Must be preserved or migrated carefully.**

---

## 6. `TimeInForce` Enum Usage

Used pervasively across parsers and connector method bodies. Key observations:

| Variant | Usage Pattern |
|---------|---------------|
| `TimeInForce::GTC` | By far the most common — appears in 40+ files. Default for most parsers. |
| `TimeInForce::IOC` | `vertex/auth.rs`, `vertex/connector.rs`, `fyers/parser.rs`, `alpaca/parser.rs` |
| `TimeInForce::FOK` | `oanda/connector.rs`, `vertex/auth.rs`, `alpaca/parser.rs` |
| `TimeInForce::PostOnly` | `vertex/auth.rs:140` — used in Vertex EIP-712 signature generation |

Notable usage patterns:
- **Two import styles** used in real code:
  - `crate::core::TimeInForce::GTC` (most parsers)
  - `crate::core::types::TimeInForce::GTC` (`dhan/parser.rs:329`, `dhan/parser.rs:373`)
- `vertex/auth.rs` uses `TimeInForce::PostOnly` for generating blockchain expiration timestamps — this is crypto-sensitive business logic, not just a label.

**`TimeInForce` is deeply embedded. Must be preserved with identical variants.**

---

## 7. `Credentials` Struct Usage

`Credentials::new(api_key, api_secret)` and `Credentials { ... }` are used in:
- Auth unit tests in every exchange's `auth.rs` file (test scaffolding)
- `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\factory.rs:58` — factory imports `Credentials` and uses it to construct connectors
- Various `auth.rs` test blocks across all connectors

**`Credentials` is actively used by the factory and all auth test code. It must remain stable or be aliased.**

---

## 8. `AuthRequest` and `SignatureLocation` Usage

- **`AuthRequest<'a>`**: Used only in `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\auth.rs` (definition). No connector imports it directly via `use` — each exchange's `auth.rs` uses it internally within the same auth module.
- **`SignatureLocation`**: Used only at `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\auth.rs:112` (in the `ExchangeAuth` default impl). Zero external references.

**Both are used minimally. `SignatureLocation` is nearly dead — only the default impl in `auth.rs` references it.**

---

## 9. Connector Manager Analysis

### What `AnyConnector` Currently Delegates

File: `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\connector.rs`

Currently, `AnyConnector` only delegates **two** traits:
1. `ExchangeIdentity` — fully delegated (all methods: `exchange_id`, `is_testnet`, `supported_account_types`)
2. `MarketData` — fully delegated (all methods: `get_price`, `get_orderbook`, `get_klines`, `get_ticker`, `ping`)

**Trading, Account, Positions are explicitly marked TODO in `aggregator.rs`:**
```rust
// NOTE: These methods require Trading trait to be implemented on AnyConnector.
// TODO: Uncomment when Trading trait is delegated in connector.rs
```

The connector manager knows about `Trading`/`Account` only through commented-out aggregator methods. There is no delegation yet.

### What `ConnectorFactory` Uses

`c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\factory.rs` imports `Credentials` from `crate::core::traits` and constructs connectors using it.

### Pool and Aggregator

- `ConnectorPool` (DashMap-backed, lock-free) stores `Arc<AnyConnector>`
- `ConnectorAggregator` wraps the pool and provides high-level API — but only for `MarketData` methods. All trading/account methods are commented out pending `AnyConnector` delegation.

---

## 10. Types That Are Shared and Must NOT Be Touched

These types are used across V1 and V2 code — they appear in both old trait signatures and new V2 trait signatures. Changing them would break everything simultaneously:

| Type | Used By |
|------|---------|
| `Symbol` | `Trading` V1, `TradingV2`, `MarketData`, parsers |
| `OrderSide` | `Trading` V1, `TradingV2`, parsers, `extensions.rs` |
| `AccountType` | All traits V1 + V2, `AnyConnector`, `ConnectorAggregator` |
| `OrderType` | V1 `Order` struct, parsers in 20+ connectors |
| `TimeInForce` | V1 `Order` struct, parsers in 40+ connectors |
| `Order` | `Trading` V1, `TradingV2`, connector returns everywhere |
| `Balance` | `Account` V1, `AccountV2` |
| `AccountInfo` | `Account` V1, `AccountV2` |
| `Position` | `Positions` V1, `PositionsV2` |
| `FundingRate` | `Positions` V1, `PositionsV2` |
| `Price`, `Quantity` | All trading traits |
| `ExchangeId` | `ExchangeIdentity`, `ConnectorPool`, `AnyConnector` |
| `ExchangeResult`, `ExchangeError` | Every method in every trait |
| `Credentials` | `ExchangeAuth` V1, `ConnectorFactory` |

---

## 11. What Is Actively Used vs Dead Code

### Actively Used (cannot touch without breaking compiles):

- `Trading`, `Account`, `Positions` V1 traits — implemented by 50+ connectors each
- `ExchangeIdentity`, `MarketData` — implemented and delegated in `AnyConnector`
- `ExchangeAuth`, `Credentials`, `AuthRequest` — used in every exchange's `auth.rs`
- `OrderType`, `TimeInForce`, `OrderSide`, `Symbol`, `AccountType` — used in every parser
- `CoreConnector` composite (V1) — used in `mod.rs` and referenced in comments
- `BatchOperations`, `AdvancedOrders`, `MarginTrading`, `Transfers` traits — **defined but never implemented** (dead code, but the trait definitions are exported)

### Dead Code (safe to delete or ignore):

- **V1 extension trait implementations** — none exist
- `BatchOperations::create_orders_batch` default impl — **broken** (calls `create_limit_order` which doesn't exist)
- `SignatureLocation` — only referenced in a default impl, zero external usage
- All V2 traits (`TradingV2`, `AccountV2`, `PositionsV2`, etc.) — defined, exported, but zero implementations

---

## 12. Migration Plan: Minimum Changes to Replace V1 with V2

### Prerequisite: V2 type completeness

Before any connector can implement V2, these V2-specific types must exist in `core/types`:
- `OrderRequest`, `CancelRequest`, `PlaceOrderResponse` — required by `TradingV2`
- `OrderHistoryFilter` — required by `TradingV2::get_order_history`
- `BalanceQuery`, `FeeInfo` — required by `AccountV2`
- `PositionModification`, `PositionQuery` — required by `PositionsV2`
- `ExchangeCredentials` — required by `Authenticated::set_credentials`
- V2 operations types: `AmendRequest`, `CancelScope`, `CancelAllResponse`, `OrderResult`, etc.

Check `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\types\` for which of these already exist.

### Step-by-step migration (connector-by-connector):

**For each connector**, the migration is:**

1. **Add** `impl TradingV2 for <Connector>` — 5 methods using `OrderRequest`/`CancelRequest` fat enums
2. **Add** `impl AccountV2 for <Connector>` — 3 methods (`get_balance` signature changes, adds `get_fees`)
3. **Add** `impl PositionsV2 for <Connector>` (for futures-capable connectors) — 3 methods with new `PositionQuery`/`PositionModification`
4. **Add** `impl Authenticated for <Connector>` — 3 methods for credential management
5. **Optionally add** optional operation traits: `CancelAllV2`, `AmendOrderV2`, `BatchOrdersV2`

**The V1 implementations can coexist during migration** — both `impl Trading` and `impl TradingV2` can exist on the same struct simultaneously. This allows gradual connector-by-connector rollout.

### The AnyConnector change (the high-impact step):

Once all connectors implement V2 traits, add delegation in `connector.rs`:
```rust
use crate::core::traits::{TradingV2, AccountV2, PositionsV2};
impl TradingV2 for AnyConnector { ... }   // delegate_all! macro
impl AccountV2 for AnyConnector { ... }
impl PositionsV2 for AnyConnector { ... }
```
And uncomment the trading/account sections in `aggregator.rs`.

### What to delete AFTER migration:

1. `Trading`, `Account`, `Positions` V1 impls in each connector (50+ files × 3 = 150 impl blocks)
2. `extensions.rs` entirely (dead code, broken default impl)
3. `ExchangeAuth` trait and `AuthRequest`/`SignatureLocation` (replace with `Authenticated`)
4. `CoreConnector` composite trait (replace with `CoreConnectorV2`)
5. V1 exports from `core/traits/mod.rs`

### What to NOT delete:

- `Credentials` struct — still needed by `ConnectorFactory` until factory is updated to use `ExchangeCredentials`
- `OrderType`, `TimeInForce`, `OrderSide` — core data types, not tied to V1 vs V2
- `ExchangeIdentity`, `MarketData`, `WebSocketConnector` — these are not being replaced

---

## 13. Risk Assessment

| Risk | Severity | Notes |
|------|----------|-------|
| V2 types may be incomplete | High | Check `core/types/` for `OrderRequest`, `PlaceOrderResponse`, `BalanceQuery` etc. If missing, V2 connectors cannot compile |
| Method signature changes | Medium | V1 `cancel_order(symbol, id, account_type)` vs V2 `cancel_order(req: CancelRequest)` — parsers/tests referencing old signatures need updates |
| `TimeInForce::PostOnly` in Vertex auth | High | Vertex `auth.rs` uses `TimeInForce` variant in EIP-712 signature math. If `TimeInForce` is renamed or restructured, Vertex trading breaks |
| `Credentials` in factory | Medium | `ConnectorFactory` currently builds connectors with `Credentials::new`. Must update factory when switching to `Authenticated::set_credentials` |
| AnyConnector `Trading`/`Account` delegation | Low | Currently missing from `AnyConnector`. The TODO comments in aggregator confirm this is known. Easy to add once connectors implement V2. |
| Extension trait broken default | None | `BatchOperations::create_orders_batch` calls nonexistent `create_limit_order` — but since no connector implements `BatchOperations`, this dead code never compiles. Delete `extensions.rs` without risk. |

---

## 14. File Reference Index

| File | Relevance |
|------|-----------|
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\mod.rs` | Re-exports all V1 and V2 traits; `CoreConnector` and `CoreConnectorV2` definitions |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\trading.rs` | V1 `Trading` trait (5 methods) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\trading_v2.rs` | V2 `TradingV2` trait (5 methods, fat enum approach) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\account.rs` | V1 `Account` trait (2 methods) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\account_v2.rs` | V2 `AccountV2` trait (3 methods, adds `get_fees`) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\positions.rs` | V1 `Positions` trait (3 methods) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\positions_v2.rs` | V2 `PositionsV2` trait (3 methods, fat enum modify_position) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\extensions.rs` | V1 extension traits — ALL DEAD CODE, broken |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\auth.rs` | V1 `ExchangeAuth`, `Credentials`, `AuthRequest`, `SignatureLocation` |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\auth_v2.rs` | V2 `Authenticated`, `CredentialKind` |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\core\traits\operations_v2.rs` | V2 optional operation traits (6 traits) |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\connector.rs` | `AnyConnector` — delegates only `ExchangeIdentity` + `MarketData` today |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\factory.rs` | `ConnectorFactory` — uses V1 `Credentials` |
| `c:\Users\VA PC\CODING\ML_TRADING\nemo\digdigdig3\src\connector_manager\aggregator.rs` | `ConnectorAggregator` — Trading/Account methods commented out pending delegation |
