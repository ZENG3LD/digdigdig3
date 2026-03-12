# DEX/Swap Connector Implementation Audit

**Date:** 2026-03-12
**Scope:** 8 connectors — Hyperliquid (CEX-DEX hybrid), dYdX, Lighter, Paradex, GMX, Jupiter, Uniswap, Raydium

---

## Legend

- **REAL** — method calls `self.get/post/put/delete` (actual API call)
- **STUB(NotSupported)** — `ExchangeError::NotSupported(...)` — "not yet implemented / could be done"
- **STUB(UnsupportedOp)** — `ExchangeError::UnsupportedOperation(...)` — "exchange genuinely cannot do this via REST"
- **HARDCODED** — returns hardcoded constant values without API call (semi-real)

---

## 1. Hyperliquid

**File:** `digdigdig3/src/crypto/cex/hyperliquid/connector.rs`

**Root cause of all stubs:** EIP-712 wallet signing not yet implemented.

### place_order — 0 / 14 REAL

Single flat stub (no match arms on OrderType):

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(NotSupported) — "EIP-712 signing not yet implemented" |

**Score: 0/14** (no OrderType dispatch at all)

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(NotSupported) — "EIP-712 signing not yet implemented" |

**Score: 0/4**

### modify_position — 0 / 6 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(NotSupported) — "EIP-712 signing not yet implemented" |

**Score: 0/6**

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(NotSupported) | EIP-712 not implemented |
| get_fees | **HARDCODED** | Returns `{ maker: 0.0002, taker: 0.0005 }` — no API call, but documented as correct fixed rate |
| get_balance | STUB(NotSupported) | EIP-712 not implemented |
| get_account_info | STUB(NotSupported) | EIP-712 not implemented |
| get_positions | STUB(NotSupported) | EIP-712 not implemented |
| get_funding_rate | STUB(UnsupportedOp) | Mislabeled — HL *does* have funding data, this is a lazy stub |

### Optional Traits

- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present

### Summary

MarketData is fully implemented (price, orderbook, klines, ticker, ping, exchange_info). All Trading/Account/Positions are blocked on missing EIP-712 signer. `get_funding_rate` uses `UnsupportedOperation` but HL *does* expose funding data via the Info endpoint — this is a bug/misclassification.

---

## 2. dYdX

**File:** `digdigdig3/src/crypto/dex/dydx/connector.rs`

**Architecture:** Indexer REST API (read-only). Trading requires Cosmos SDK gRPC (MsgPlaceOrder). Account/Position data requires `address + subaccountNumber` params not exposed by generic trait.

### place_order — 0 / 14 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "Cosmos gRPC required, REST is read-only" |

**Score: 0/14** — Correct: REST truly cannot place orders on dYdX v4.

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "Cosmos gRPC required" |

**Score: 0/4** — Correct: cancel also requires gRPC.

### modify_position — 0 / 6 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires address + subaccountNumber" |

**Score: 0/6**

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(UnsupportedOp) | Requires `address + subaccountNumber` — trait has no place for those. Extended method `get_orders_for_subaccount()` exists and is REAL. |
| get_fees | STUB(UnsupportedOp) | "not yet implemented" — a fee tier endpoint does exist on dYdX Indexer, this is a lazy stub |
| get_balance | STUB(NotSupported) | "requires address parameter" — extended `get_subaccount_balances()` is REAL |
| get_account_info | STUB(NotSupported) | "requires address parameter" |
| get_positions | STUB(UnsupportedOp) | "requires address + subaccountNumber" — extended `get_subaccount_positions()` is REAL |
| get_funding_rate | STUB(UnsupportedOp) | Same wrong message ("get_positions requires address...") — copy-paste error |

### Optional Traits

- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present

### Extended Methods (REAL, not in trait)

- `get_subaccount_balances(address, subaccount_number)` — REAL API call
- `get_subaccount_positions(address, subaccount_number)` — REAL API call
- `get_orders_for_subaccount(address, subaccount_number, ...)` — REAL API call
- `get_all_markets()` — REAL API call

### Summary

MarketData is fully implemented. The trait-level stubs for account/position/trading are architecturally correct (REST is read-only, address param is missing). However `get_fees` is a lazy stub — dYdX Indexer does expose fee data. `get_funding_rate` has a copy-paste bug in its error message.

---

## 3. Lighter

**File:** `digdigdig3/src/crypto/dex/lighter/connector.rs`

**Architecture:** Phase 1 (market data) complete. Phase 2 (account) and Phase 3 (trading) pending — requires transaction signing.

### place_order — 0 / 14 REAL

| Arm | Status |
|-----|--------|
| `OrderType::Market` | STUB(UnsupportedOp) — "Trading not yet implemented (Phase 3)" |
| `OrderType::Limit { price }` | STUB(UnsupportedOp) — "Trading not yet implemented (Phase 3)" |
| `_` (catch-all) | STUB(UnsupportedOp) — "not supported on Lighter" |

**Score: 0/14**

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| `CancelScope::Single { order_id }` | STUB(UnsupportedOp) — "Trading not yet implemented (Phase 3)" |
| `_` (catch-all) | STUB(UnsupportedOp) — "not supported on Lighter" |

**Score: 0/4** — Note: `CancelScope::All` and `CancelScope::BySymbol` fall into the `_` catch-all.

### modify_position — 0 / 6 REAL

| Arm | Status |
|-----|--------|
| `PositionModification::SetLeverage { .. }` | STUB(NotSupported) — "Leverage not yet implemented (Phase 2)" |
| `_` (catch-all) | STUB(UnsupportedOp) — "not supported on Lighter" |

**Score: 0/6**

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(UnsupportedOp) | "not yet implemented" |
| get_fees | **REAL** | Calls `self.get(LighterEndpoint::OrderBooks, ...)` and parses `maker_fee`/`taker_fee` from response |
| get_balance | STUB(NotSupported) | "Account data not yet implemented (Phase 2)" |
| get_account_info | STUB(NotSupported) | "Account data not yet implemented (Phase 2)" |
| get_positions | STUB(NotSupported) | "Positions not yet implemented (Phase 2)" |
| get_funding_rate | **REAL** | Calls `self.get(LighterEndpoint::Fundings, ...)` and parses result |

### Optional Traits

- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present

### Summary

MarketData fully implemented. `get_fees` and `get_funding_rate` are the only REAL methods outside MarketData. All trading and account data blocked on Phase 2/3 (transaction signing). `get_fees` correctly makes a live API call.

---

## 4. Paradex

**File:** `digdigdig3/src/crypto/dex/paradex/connector.rs`

**Architecture:** Perpetual futures only. JWT authentication required for private endpoints. Trading implemented via StarkNet JWT (note: "production use requires StarkNet signature" per code comment, but REST POST calls are wired).

### place_order — 9 / 14 REAL

| Arm | Status |
|-----|--------|
| `OrderType::Market` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::Limit { price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::PostOnly { price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::Ioc { price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::Fok { price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::StopMarket { stop_price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::StopLimit { stop_price, limit_price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::ReduceOnly { price }` | **REAL** — `self.post(CreateOrder, ...)` |
| `OrderType::Gtd { price, expire_time }` | **REAL** — `self.post(CreateOrder, ...)` |
| `_` (catch-all for remaining types) | STUB(UnsupportedOp) |

**Score: 9/14** — The 5 remaining types (e.g., TakeProfitMarket, TrailingStop, etc.) hit the `_` catch-all.

### cancel_order — 3 / 4 REAL

| Arm | Status |
|-----|--------|
| `CancelScope::Single { order_id }` | **REAL** — `self.delete(CancelOrder, ...)` |
| `CancelScope::All { symbol }` | **REAL** — `self.cancel_all_orders(symbol)` → `self.delete(CancelAllOrders, ...)` |
| `CancelScope::BySymbol { symbol }` | **REAL** — `self.cancel_all_orders(Some(symbol))` → `self.delete(CancelAllOrders, ...)` |
| `CancelScope::Batch { .. }` | STUB(UnsupportedOp) — "use CancelAll/BySymbol instead" |

**Score: 3/4** — Batch cancel not supported by Paradex.

### modify_position — 2 / 6 REAL

| Arm | Status |
|-----|--------|
| `PositionModification::ClosePosition { symbol, account_type }` | **REAL** — `self.post(CreateOrder, reduce-only market)` |
| `PositionModification::SetTpSl { symbol, take_profit, stop_loss, account_type }` | **REAL** — `self.post(CreateOrder, ...)` for TP and/or SL |
| `PositionModification::SetLeverage { .. }` | STUB(UnsupportedOp) — "Paradex manages leverage automatically" |
| `PositionModification::SetMarginMode { .. }` | STUB(UnsupportedOp) — "uses cross-margin by default" |
| `PositionModification::AddMargin { .. }` | STUB(UnsupportedOp) — "uses auto-margin management" |
| `PositionModification::RemoveMargin { .. }` | STUB(UnsupportedOp) — "uses auto-margin management" |

**Score: 2/6** — Leverage/margin stubs are correct: Paradex does not support manual leverage or margin add/remove.

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | **REAL** | `self.get(OrdersHistory, params)` with filters |
| get_fees | **REAL** | `self.get(Markets, ...)` and parses `fee_config.api_fees.maker/taker` |
| get_balance | **REAL** | `self.get(Balances, ...)` |
| get_account_info | **REAL** | `self.get(Account, ...)` |
| get_positions | **REAL** | `self.get(Positions, ...)` |
| get_funding_rate | **REAL** | `self.get(MarketsSummary, ...)` parsing `next_funding_rate` |

### Optional Traits

- `impl CancelAll` — NOT a separate trait impl, but `cancel_all_orders()` is an extended method called internally
- `impl AmendOrder` — NOT present (a `_put` helper exists but no trait impl)
- `impl BatchOrders` — NOT present

### Summary

Most complete connector of the 8. All account/position data is real. Trading fully wired for 9 order types. The StarkNet signature comment is a concern — REST POST calls are wired but may fail in production without full StarkNet signing integration.

---

## 5. GMX

**File:** `digdigdig3/src/crypto/dex/gmx/connector.rs`

**Architecture:** REST API is read-only. All trading/account requires smart contract calls (ethers-rs/alloy) — architecturally impossible via REST.

### place_order — 0 / 14 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires blockchain wallet integration" |

**Score: 0/14** — Correct: GMX has no REST trading endpoints.

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires smart contract transaction" |

**Score: 0/4** — Correct.

### modify_position — 0 / 6 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires smart contract transactions" |

**Score: 0/6** — Correct.

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(UnsupportedOp) | "requires The Graph subgraph" — correct, REST doesn't expose this |
| get_fees | STUB(UnsupportedOp) | "uses protocol fees, not maker/taker" — correct, GMX fee model is incompatible |
| get_balance | STUB(UnsupportedOp) | "requires ERC-20 contract queries" — correct |
| get_account_info | STUB(UnsupportedOp) | "no account concept in REST API" — correct |
| get_positions | STUB(UnsupportedOp) | "requires smart contract queries" — correct |
| get_funding_rate | STUB(UnsupportedOp) | "GMX uses borrowing fees, not funding rates" — technically correct but borrowing rate data IS in the `/markets` REST endpoint |

### Optional Traits

- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present
- `impl Positions` — NOT present (no Positions trait import)

### Summary

MarketData is fully implemented (price, klines, ticker, ping, exchange_info). Orderbook correctly returns UnsupportedOp (oracle pricing). All trading/account stubs are architecturally correct for a read-only REST connector. Note: `get_funding_rate` could be partially implemented using `/markets` borrow rate data — current stub is slightly over-restrictive.

---

## 6. Jupiter

**File:** `digdigdig3/src/crypto/dex/jupiter/connector.rs`

**Architecture:** DEX aggregator on Solana. Swap execution requires Solana wallet signing. No Positions trait implemented.

### place_order — 0 / 14 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires Solana wallet integration" |

**Score: 0/14** — Correct.

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "swaps are atomic, cannot be cancelled" |

**Score: 0/4** — Correct.

### modify_position — NOT APPLICABLE

Jupiter does not implement the `Positions` trait. No `modify_position` method exists.

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(UnsupportedOp) | Correct — Jupiter has no order history REST endpoint |
| get_fees | STUB(UnsupportedOp) | Correct — fees are protocol-level, not maker/taker |
| get_balance | STUB(UnsupportedOp) | Correct — no account system, use Solana RPC |
| get_account_info | STUB(UnsupportedOp) | Correct — no account concept |
| get_positions | NOT PRESENT | Positions trait not implemented |
| get_funding_rate | NOT PRESENT | Positions trait not implemented |

### Optional Traits

- `impl Positions` — NOT present
- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present

### Summary

MarketData partially implemented: get_price (REAL), get_ticker (REAL), ping (REAL). get_orderbook and get_klines return UnsupportedOp (correct — Jupiter aggregator has no native orderbook or historical klines). All Trading/Account stubs are architecturally correct.

---

## 7. Uniswap

**File:** `digdigdig3/src/crypto/swap/uniswap/connector.rs`

**Architecture:** Uses three backends — Ethereum RPC (slot0 call), The Graph subgraph (GraphQL), and Uniswap Trading API (POST). No Positions trait. All trading requires wallet signing.

### place_order — 0 / 14 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires Ethereum wallet integration" |

**Score: 0/14** — Correct.

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "swaps are atomic, cannot be cancelled" |

**Score: 0/4** — Correct.

### modify_position — NOT APPLICABLE

Uniswap does not implement the `Positions` trait.

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(UnsupportedOp) | Correct — no REST order history |
| get_fees | STUB(UnsupportedOp) | Correct — uses pool fee tiers (0.01%/0.05%/0.30%/1.00%), not maker/taker |
| get_balance | STUB(UnsupportedOp) | Correct — no account system |
| get_account_info | STUB(UnsupportedOp) | Correct — permissionless AMM |
| get_positions | NOT PRESENT | Positions trait not implemented |
| get_funding_rate | NOT PRESENT | Positions trait not implemented |

### Optional Traits

- `impl Positions` — NOT present
- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present

### Summary

MarketData is the most sophisticated of the 8 — uses direct Ethereum RPC slot0() calls (no API key), GraphQL subgraph fallback, and simulates orderbook from pool liquidity. get_klines converts swap events to klines. All Trading/Account stubs are architecturally correct.

---

## 8. Raydium

**File:** `digdigdig3/src/crypto/swap/raydium/connector.rs`

**Architecture:** Pure AMM on Solana. REST API is public. Trading requires Solana wallet signing. No Positions trait.

### place_order — 0 / 14 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "requires Solana wallet integration" |

**Score: 0/14** — Correct.

### cancel_order — 0 / 4 REAL

| Arm | Status |
|-----|--------|
| entire method (no match) | STUB(UnsupportedOp) — "AMM swaps are atomic" |

**Score: 0/4** — Correct.

### modify_position — NOT APPLICABLE

Raydium does not implement the `Positions` trait.

### Other Methods

| Method | Status | Notes |
|--------|--------|-------|
| get_order_history | STUB(UnsupportedOp) | Correct — no REST order history |
| get_fees | STUB(UnsupportedOp) | Correct — pool fee tiers, not maker/taker |
| get_balance | STUB(UnsupportedOp) | Correct — no account system |
| get_account_info | STUB(UnsupportedOp) | Correct — permissionless AMM |
| get_positions | NOT PRESENT | Positions trait not implemented |
| get_funding_rate | NOT PRESENT | Positions trait not implemented |

### Note on get_klines and get_orderbook

| Method | Status | Notes |
|--------|--------|-------|
| get_klines | STUB(NotSupported) | "Raydium API does not provide kline data" — uses NotSupported (could be implemented) rather than UnsupportedOp — possibly incorrect classification |
| get_orderbook | STUB(UnsupportedOp) | Correct — pure AMM |

### Optional Traits

- `impl Positions` — NOT present
- `impl CancelAll` — NOT present
- `impl AmendOrder` — NOT present
- `impl BatchOrders` — NOT present

### Summary

Simple connector. get_price and get_ticker are REAL. get_orderbook correctly returns UnsupportedOp. get_klines uses NotSupported which is a misclassification — Raydium v3 does have a `/pairs` endpoint with some data, but their API genuinely doesn't serve candlestick data. All Trading/Account stubs are correct.

---

## Cross-Connector Summary Table

| Connector | place_order Real | cancel Real | modify_pos Real | get_fees | get_balance | get_acct_info | get_positions | get_funding | Optional Traits |
|-----------|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|:---:|
| Hyperliquid | 0/14 | 0/4 | 0/6 | HARDCODED | STUB(NS) | STUB(NS) | STUB(NS) | STUB(UO)* | None |
| dYdX | 0/14 | 0/4 | 0/6 | STUB(UO)** | STUB(NS) | STUB(NS) | STUB(UO) | STUB(UO) | None |
| Lighter | 0/14 | 0/4 | 0/6 | **REAL** | STUB(NS) | STUB(NS) | STUB(NS) | **REAL** | None |
| Paradex | **9/14** | **3/4** | **2/6** | **REAL** | **REAL** | **REAL** | **REAL** | **REAL** | cancel_all (method) |
| GMX | 0/14 | 0/4 | 0/6 | STUB(UO) | STUB(UO) | STUB(UO) | N/A | STUB(UO) | None |
| Jupiter | 0/14 | 0/4 | N/A | STUB(UO) | STUB(UO) | STUB(UO) | N/A | N/A | None |
| Uniswap | 0/14 | 0/4 | N/A | STUB(UO) | STUB(UO) | STUB(UO) | N/A | N/A | None |
| Raydium | 0/14 | 0/4 | N/A | STUB(UO) | STUB(UO) | STUB(UO) | N/A | N/A | None |

**Legend:** NS = NotSupported, UO = UnsupportedOperation, N/A = trait not implemented

`*` Hyperliquid `get_funding_rate` uses `UnsupportedOperation` but HL *does* expose funding data — misclassification.
`**` dYdX `get_fees` uses `UnsupportedOperation("not yet implemented")` — lazy stub, fee endpoint exists.

---

## Issues Found

### Critical Misclassifications (NotSupported used where data IS available)

1. **Hyperliquid `get_funding_rate`** — Uses `UnsupportedOperation` with message "not yet implemented". Hyperliquid exposes `fundingHistory` and `metaAndAssetCtxs` (contains `funding` field) via the Info endpoint. This is a lazy stub masquerading as a correct unsupported stub.

2. **dYdX `get_fees`** — Uses `UnsupportedOperation("not yet implemented")`. The dYdX Indexer exposes fee tiers (fee_tier_history endpoint). This is implementable.

3. **dYdX `get_funding_rate`** — Error message says "get_positions requires address and subaccountNumber" — this is a copy-paste bug. The actual funding rate IS available via the `/v4/perpetualMarkets` endpoint's `nextFundingRate` field without authentication.

4. **Raydium `get_klines`** — Uses `NotSupported` (could be implemented) instead of `UnsupportedOperation` (exchange genuinely can't). Raydium API genuinely has no kline endpoint, so `UnsupportedOperation` would be more accurate.

### Architectural Notes (correct stubs)

- **GMX, Jupiter, Uniswap, Raydium** — All Trading stubs are architecturally correct. Atomic blockchain transactions (EVM/Solana) are genuinely impossible to cancel or place via REST alone.
- **dYdX** — Trading stubs are correct (Cosmos gRPC is genuinely required). Account stubs are borderline — they're blocked on the generic trait not supporting address params, but the data is accessible via extended methods.
- **Paradex** — Best-implemented connector. The "production requires StarkNet signature" comment on place_order should be investigated — if JWT auth works for the POST calls, trading may function. If StarkNet signing is separately needed per-order, the stubs are premature.

### Correct UnsupportedOperation Usage

- GMX orderbook — oracle pricing, no orderbook concept
- Jupiter orderbook — aggregator, routes to source DEXes
- Raydium orderbook — pure AMM, no orderbook
- Jupiter klines — no historical data API
- GMX get_fees — protocol fee model genuinely incompatible with FeeInfo struct
- Uniswap get_fees — pool fee tier model genuinely incompatible
- Raydium get_fees — pool fee tier model genuinely incompatible
- GMX/Jupiter/Uniswap/Raydium balances — no account system on permissionless DEXes

---

## Recommendations by Priority

### P1 — Fix Misclassifications (easy wins)

1. **Hyperliquid `get_funding_rate`**: Implement via `InfoType::MetaAndAssetCtxs` — funding rates are in the assets context array. Should be ~20 lines.

2. **dYdX `get_funding_rate`**: Implement via `DydxEndpoint::PerpetualMarkets` — `nextFundingRate` is already fetched by `get_price/get_ticker`. Should be ~10 lines.

3. **dYdX `get_fees`**: Implement via the fee tier history endpoint or extract from perpetual markets data.

4. **dYdX `get_funding_rate` error message**: Fix copy-paste bug (message says "get_positions requires...").

5. **Raydium `get_klines`**: Change from `NotSupported` to `UnsupportedOperation` to accurately communicate "exchange doesn't support this" vs "not yet coded".

### P2 — Unblock Hyperliquid Trading

The only blocker is EIP-712 signing. Once a signer is integrated, all 5 Trading/Account/Position methods become implementable. The Info POST endpoint infrastructure is already in place.

### P3 — Paradex Production Readiness

Verify StarkNet signature requirement. If JWT auth suffices for order placement, Paradex is the only connector where `place_order` is genuinely production-ready (9/14 order types wired). If StarkNet signatures are required per-order, those 9 arms should be re-labeled NotSupported.

### P4 — dYdX Extended Methods in Trait

Consider adding optional trait methods or a `DydxAccount` sub-trait to expose `address + subaccountNumber` variants of balance/position/order queries. The extended methods exist and work — they're just unreachable via the standard trait interface.
