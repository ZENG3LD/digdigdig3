# Phase α — Symbol Decoupling

## Goal

Connectors accept and emit **raw exchange-native symbol strings only**.
No internal `Symbol{base, quote}` massaging inside connector methods.
Symbol translation moves to `SymbolNormalizer` in `src/core/utils/symbol_normalizer.rs`.

After Phase α:
- `BinanceConnector::get_ticker("BTCUSDT", AccountType::Spot)` — caller passes raw.
- `OkxConnector::get_ticker("BTC-USDT-SWAP", AccountType::FuturesCross)` — caller passes raw.
- `GateIoConnector::get_ticker("BTC_USDT", AccountType::Spot)` — caller passes raw.
- `format_symbol` helpers remain as **public exports in each `endpoints.rs`** for use by the normalizer; connectors no longer call them internally.
- `StreamSpec.symbol: String` (raw) replaces `StreamSpec.symbol: Symbol`.
- `SubscriptionRequest.symbol: Symbol` is preserved at the public WS trait boundary but its `raw` field is always set to the exchange-native string; connectors read `spec.symbol` (which is `String`) from `StreamSpec`, not `SubscriptionRequest`.

---

## 1. SymbolNormalizer API

### Location
`src/core/utils/symbol_normalizer.rs`

### Design decision
**Option B — single central match** over trait dispatch. Per-exchange format logic is 1–5 LOC (typically one `format!`). 22-arm match is large but transparent, auditable, and eliminates cross-crate coupling. No new traits, no blanket impls.

### Error type

```rust
// src/core/utils/symbol_normalizer.rs

#[derive(Debug, thiserror::Error)]
pub enum NormalizerError {
    #[error("unknown exchange: {0:?}")]
    UnknownExchange(ExchangeId),
    #[error("invalid format for {exchange:?}: '{raw}'")]
    InvalidFormat { exchange: ExchangeId, raw: String },
    #[error("account type {account_type:?} not supported for {exchange:?}")]
    UnsupportedAccountType { exchange: ExchangeId, account_type: AccountType },
    #[error("symbol requires full instrument name (e.g. Deribit options): {msg}")]
    RequiresRawInstrument { msg: String },
}
```

### Struct + methods

```rust
pub struct SymbolNormalizer;

impl SymbolNormalizer {
    /// Canonical Symbol → exchange-native raw string.
    ///
    /// `account_type` is required because Binance spot≠coin-margined,
    /// OKX spot≠SWAP, KuCoin spot≠futures, etc.
    pub fn to_exchange(
        id: ExchangeId,
        sym: &Symbol,
        account_type: AccountType,
    ) -> Result<String, NormalizerError>;

    /// Exchange-native raw string → canonical Symbol.
    ///
    /// Exchanges always emit valid format, so this is infallible for
    /// well-formed exchange-sourced strings. Returns Err only if `raw`
    /// cannot be parsed as a known pattern for this exchange.
    pub fn from_exchange(
        id: ExchangeId,
        raw: &str,
        account_type: AccountType,
    ) -> Result<Symbol, NormalizerError>;

    /// Cheap pattern check — does `raw` match this exchange's known format?
    /// Used for validation before sending to API.
    pub fn is_valid_for(
        id: ExchangeId,
        raw: &str,
        account_type: AccountType,
    ) -> bool;
}
```

### Per-exchange rule table for `to_exchange`

| Exchange | AccountType | Rule | Example in → out |
|----------|-------------|------|-----------------|
| Binance | Spot/Margin | `concat(base, quote)` uppercase | BTC/USDT → `BTCUSDT` |
| Binance | FuturesCross/FuturesIsolated (USDT-M) | `concat(base, quote)` uppercase | BTC/USDT → `BTCUSDT` |
| Binance | CoinMargined (future AccountType::Options treated as coin-M) | `concat(base, USD)_PERP` | BTC/USD → `BTCUSD_PERP` |
| Bybit | Spot | `concat(base, quote)` uppercase | BTC/USDT → `BTCUSDT` |
| Bybit | FuturesCross/FuturesIsolated | `concat(base, quote)` uppercase | BTC/USDT → `BTCUSDT` |
| OKX | Spot/Margin | `BASE-QUOTE` | BTC/USDT → `BTC-USDT` |
| OKX | FuturesCross/FuturesIsolated | `BASE-QUOTE-SWAP` | BTC/USDT → `BTC-USDT-SWAP` |
| KuCoin | Spot/Margin | `BASE-QUOTE` | BTC/USDT → `BTC-USDT` |
| KuCoin | FuturesCross/FuturesIsolated + USDT quote | `XBTUSDTM` (BTC→XBT) | BTC/USDT → `XBTUSDTM` |
| KuCoin | FuturesCross/FuturesIsolated + USD quote | `XBTUSDM` (BTC→XBT) | BTC/USD → `XBTUSDM` |
| Kraken | Spot/Margin | `XBTUSD` (BTC→XBT, no separator) | BTC/USD → `XBTUSD` |
| Kraken | FuturesCross/FuturesIsolated | `PI_XBTUSD` (BTC→XBT, `PI_` prefix) | BTC/USD → `PI_XBTUSD` |
| Coinbase | Spot | `BASE-QUOTE` | BTC/USDT → `BTC-USDT` |
| Gate.io | Spot/Margin/Futures | `BASE_QUOTE` underscore | BTC/USDT → `BTC_USDT` |
| Gemini | Spot | `basequote` all lowercase | BTC/USD → `btcusd` |
| MEXC | Spot | `BASE_QUOTE` | BTC/USDT → `BTC_USDT` |
| MEXC | FuturesCross/FuturesIsolated | `BASE_USDT` | BTC/USDT → `BTC_USDT` |
| HTX | Spot | `basequote` all lowercase | BTC/USDT → `btcusdt` |
| HTX | FuturesCross | `BASE-USDT` (uppercase, hyphen) | BTC/USDT → `BTC-USDT` |
| Bitget | Spot | `BASEQUOTE` uppercase | BTC/USDT → `BTCUSDT` |
| Bitget | FuturesCross/FuturesIsolated | `BASEUSDTPERP` | BTC/USDT → `BTCUSDTPERP` |
| BingX | Spot | `BASE-QUOTE` | BTC/USDT → `BTC-USDT` |
| BingX | FuturesCross | `BASE-USDT` | BTC/USDT → `BTC-USDT` |
| Crypto.com | Spot | `BASE_QUOTE` | BTC/USD → `BTC_USD` |
| Crypto.com | FuturesCross | `BASEUSD-PERP` | BTC/USD → `BTCUSD-PERP` |
| Upbit | Spot only | `QUOTE-BASE` **reversed** | BTC/KRW → `KRW-BTC` |
| Bitfinex | Spot/Margin | `tBASEQUOTE` (USDT→USD) | BTC/USDT → `tBTCUSD` |
| Bitfinex | FuturesCross/FuturesIsolated | `tBASEF0:QUOTEF0` (USDT→UST) | BTC/USDT → `tBTCF0:USTF0` |
| Bitstamp | Spot only | `basequote` all lowercase | BTC/USD → `btcusd` |
| Deribit | FuturesCross + USD quote | `BASE-PERPETUAL` | BTC/USD → `BTC-PERPETUAL` |
| Deribit | FuturesCross + USDC quote | `BASE_USDC-PERPETUAL` | SOL/USDC → `SOL_USDC-PERPETUAL` |
| Deribit | AccountType::Options | `Err(RequiresRawInstrument)` — caller must pass `BTC-30MAY26-50000-C` directly | — |
| HyperLiquid | FuturesCross/FuturesIsolated | `BASE` uppercase coin only | BTC/USDT → `BTC` |
| HyperLiquid | Spot | `@{index}` if index known, else `BASE/QUOTE` passthrough | HYPE/USDC → `@107` |
| dYdX v4 | FuturesCross (only) | `BASE-USD` | BTC/USDC → `BTC-USD` |
| Lighter | FuturesCross/FuturesIsolated | `BASE` uppercase | ETH/USDC → `ETH` |
| Lighter | Spot | `BASE/QUOTE` with slash | ETH/USDC → `ETH/USDC` |
| Polymarket | — | market slug or condition ID passthrough | no canonical conversion |
| MOEX | Spot (stocks) | `BASE` uppercase (ticker, no quote) | SBER/RUB → `SBER` |

**Notes on tricky cases:**
- **Upbit**: reversed format is now transparent to callers — they pass `KRW-BTC` directly, normalizer produces it from `Symbol{base:"BTC", quote:"KRW"}`.
- **Bitfinex**: USDT→USD and USDT→UST mappings live only in the normalizer; connector never sees `Symbol`.
- **Deribit options**: normalizer returns `NormalizerError::RequiresRawInstrument`; caller must build the raw instrument string (`BTC-30MAY26-50000-C`) and pass it directly to the connector.
- **HyperLiquid spot**: the `@index` numeric mapping (`symbol_to_market_id`) stays in `endpoints.rs` and is called by the normalizer.
- **Polymarket**: no canonical symbol → raw conversion exists; callers always pass the condition ID or market slug as the raw string.

### `from_exchange` key rules (parsing exchange responses)

| Exchange | Parse rule |
|----------|-----------|
| Binance | split at known quote suffixes (USDT, BTC, ETH, BNB, BUSD) — use `get_exchange_info` cache ideally |
| Bybit | split at known quote suffixes |
| OKX | split on first `-`, strip trailing `-SWAP` → base/quote |
| KuCoin spot | split on `-` |
| KuCoin futures | strip `USDTM` → base=XBT→BTC, quote=USDT; strip `USDM` → USD |
| Kraken | call existing `parse_response_symbol` from `endpoints.rs` |
| Coinbase | split on `-` |
| Gate.io | split on `_` |
| Gemini | no separator — requires exchange info to split |
| MEXC | split on `_` |
| HTX spot | no separator — requires exchange info |
| HTX futures | split on `-` |
| Bitget spot | no separator — requires exchange info |
| BingX | split on `-` |
| Crypto.com spot | split on `_` |
| Upbit | split on `-`, reversed order: `KRW-BTC` → base=BTC, quote=KRW |
| Bitfinex | strip `t` prefix, split at 3-char boundary or `:F0` suffix |
| Bitstamp | no separator — requires exchange info |
| Deribit | existing `parse_currency` + `parse_instrument_kind` from `endpoints.rs` |
| HyperLiquid | uppercase string = base, quote = USDC (perp) |
| dYdX | split on `-`, quote always USD |
| Lighter perp | string = base, quote = USDC |
| Lighter spot | split on `/` |
| MOEX | string = base, quote = RUB |

For exchanges with no separator (Gemini, Bitstamp, HTX spot, Bitget spot), `from_exchange` returns `NormalizerError::InvalidFormat` unless `raw` contains a known separator variant. Connectors at those exchanges must use `get_exchange_info` to build a local lookup table and call `Symbol::with_raw(base, quote, raw)` when parsing responses.

---

## 2. Trait Signature Diffs

### Current state of trait methods taking `Symbol`

Surveying all trait files:

| Trait | Method | Old sig | New sig | Notes |
|-------|--------|---------|---------|-------|
| `MarketData` | `get_price` | `symbol: Symbol, account_type: AccountType` | `symbol: &str, account_type: AccountType` | Owned → borrow |
| `MarketData` | `get_orderbook` | `symbol: Symbol, depth: Option<u16>, account_type: AccountType` | `symbol: &str, depth: Option<u16>, account_type: AccountType` | |
| `MarketData` | `get_klines` | `symbol: Symbol, interval: &str, limit: Option<u16>, account_type: AccountType, end_time: Option<i64>` | `symbol: &str, interval: &str, limit: Option<u16>, account_type: AccountType, end_time: Option<i64>` | |
| `MarketData` | `get_ticker` | `symbol: Symbol, account_type: AccountType` | `symbol: &str, account_type: AccountType` | |
| `MarketData` | `get_exchange_info` | `account_type: AccountType` | no change (no symbol param) | |
| `MarketDataPublic` | `get_recent_trades` | `symbol: &Symbol, limit: Option<u32>, account_type: AccountType` | `symbol: &str, limit: Option<u32>, account_type: AccountType` | |
| `MarketDataPublic` | `get_liquidation_history` | `symbol: Option<&Symbol>, ...` | `symbol: Option<&str>, ...` | |
| `MarketDataPublic` | `get_open_interest_history` | `symbol: &Symbol, period: &str, ...` | `symbol: &str, period: &str, ...` | |
| `MarketDataPublic` | `get_premium_index` | `symbol: Option<&Symbol>, ...` | `symbol: Option<&str>, ...` | |
| `MarketDataPublic` | `get_long_short_ratio_history` | `symbol: &Symbol, period: &str, ...` | `symbol: &str, period: &str, ...` | |
| `MarketDataPublic` | `get_mark_price_klines` | `symbol: &Symbol, interval: &str, ...` | `symbol: &str, interval: &str, ...` | |
| `MarketDataPublic` | `get_index_price_klines` | `symbol: &Symbol, interval: &str, ...` | `symbol: &str, interval: &str, ...` | |
| `MarketDataPublic` | `get_funding_rate_history` | `symbol: &Symbol, ...` | `symbol: &str, ...` | |
| `Trading` | `place_order` | `req: OrderRequest` | no change — `OrderRequest.symbol: String` already | verify `OrderRequest` has no `Symbol` field |
| `Trading` | `cancel_order` | `req: CancelRequest` | no change — verify `CancelRequest.symbol: Option<String>` | |
| `Trading` | `get_order` | `symbol: &str, order_id: &str, account_type: AccountType` | **already `&str`** — no change | |
| `Trading` | `get_open_orders` | `symbol: Option<&str>, account_type: AccountType` | **already `&str`** — no change | |
| `Trading` | `get_order_history` | `filter: OrderHistoryFilter, account_type: AccountType` | no change — verify filter has no `Symbol` field | |
| `Trading` | `get_user_trades` | `filter: UserTradeFilter, account_type: AccountType` | no change — verify filter | |
| `Positions` | `get_positions` | `query: PositionQuery` | no change — verify `PositionQuery.symbol: Option<String>` | |
| `Positions` | `get_funding_rate` | `symbol: &str, account_type: AccountType` | **already `&str`** — no change | |
| `Positions` | `get_open_interest` | `symbol: &str, account_type: AccountType` | **already `&str`** — no change | |
| `Positions` | `get_mark_price` | `symbol: &str` | **already `&str`** — no change | |
| `Positions` | `get_closed_pnl` | `symbol: Option<&str>, ...` | **already `&str`** — no change | |
| `Positions` | `get_long_short_ratio` | `symbol: &str, account_type: AccountType` | **already `&str`** — no change | |
| `Account` | `get_fees` | `symbol: Option<&str>` | **already `&str`** — no change | |
| `WebSocketConnector` | `subscribe` | `request: SubscriptionRequest` | no change — `SubscriptionRequest.symbol: Symbol` kept at public boundary; inner `StreamSpec.symbol` changes to `String` | |

**Summary of actual breaking changes:**
- `MarketData`: 4 methods (`get_price`, `get_orderbook`, `get_klines`, `get_ticker`) change from `symbol: Symbol` (owned) to `symbol: &str`.
- `MarketDataPublic`: 8 methods change from `symbol: &Symbol` or `symbol: Option<&Symbol>` to `symbol: &str` or `symbol: Option<&str>`.
- `StreamSpec.symbol`: `Symbol` → `String` (internal type only, not a public trait).
- **Total trait-level breaks: 12 method signatures.**

The `Trading`, `Positions`, `Account`, and optional operation traits already use `&str` or opaque request structs — verify `OrderRequest`, `CancelRequest`, `PositionQuery`, `OrderHistoryFilter`, `UserTradeFilter` do not embed `Symbol` (if they do, that is a separate breaking change — flag to implementer for audit).

### Callsite estimate per connector

Each connector implements: 4 (MarketData) + 8 (MarketDataPublic, subset implemented) + protocol.rs subscribe_frame callsite.
Estimate: ~12–18 callsites per connector × 22 connectors = **~265–400 callsites** total.
Plus `mli-collector` + `examples/` consumer callsites: ~30–50 more.

---

## 3. StreamSpec Change

### Current
```rust
pub struct StreamSpec {
    pub kind: StreamKind,
    pub symbol: Symbol,        // ← CHANGE THIS
    pub account_type: AccountType,
    pub depth: Option<u32>,
    pub speed_ms: Option<u32>,
}
```

### New
```rust
pub struct StreamSpec {
    pub kind: StreamKind,
    pub symbol: String,        // raw exchange-native string
    pub account_type: AccountType,
    pub depth: Option<u32>,
    pub speed_ms: Option<u32>,
}
```

### Cascade effects

**`TryFrom<SubscriptionRequest> for StreamSpec`** (`stream_spec.rs:26`):
- `SubscriptionRequest.symbol: Symbol` is kept (public-facing type).
- Conversion: `symbol: req.symbol.raw().unwrap_or(&req.symbol.to_concat()).to_string()`.
- Callers that set `Symbol::with_raw("","", "BTCUSDT")` (as mli-collector already does at line 122) get the raw string directly.
- Callers that set `Symbol::new("BTC", "USDT")` without a raw string get `req.symbol.to_concat()` as a last-resort fallback — this is intentionally wrong-for-some-exchanges to force callers to migrate.

**`From<StreamSpec> for SubscriptionRequest`** (`stream_spec.rs:42`):
- Reverse: `symbol: Symbol::with_raw("", "", spec.symbol.clone())`.

**`WsProtocol::subscribe_frame(spec: &StreamSpec)`**:
- All 22 protocol implementations change from `format_symbol(&spec.symbol.base, ...)` to `spec.symbol.as_str()` directly. No more format logic in the frame builder.

**`WebSocketExt` convenience methods** (`websocket.rs:125–173`):
- `subscribe_ticker(symbol: Symbol)` → `subscribe_ticker(symbol: String)`.
- `subscribe_trades(symbol: String)` etc.
- Private stream methods (`subscribe_orders`, `subscribe_balance`, `subscribe_positions`) pass an empty string `""` instead of `Symbol::empty()`.

---

## 4. Per-Connector Difficulty Rating

| # | Exchange | Path | Difficulty | Notes |
|---|----------|------|------------|-------|
| 1 | Binance | `cex/binance/` | Low | `concat(base, quote)` — same for spot+futures USDT-M. 20+ `format_symbol` callsites in connector.rs. Mechanical. |
| 2 | Bybit | `cex/bybit/` | Low | `concat(base, quote)` always. `format_symbol(symbol: &Symbol, _)` already takes `&Symbol` not `(&str,&str)` — slight signature diff. |
| 3 | OKX | `cex/okx/` | Low | `BASE-QUOTE` spot, `BASE-QUOTE-SWAP` futures. Clear, well-tested. |
| 4 | KuCoin | `cex/kucoin/` | Medium | BTC→XBT in futures, `USDTM`/`USDM` suffix logic. `pre_connect_hook` for dynamic WS endpoint unaffected. |
| 5 | Kraken | `cex/kraken/` | Medium | BTC→XBT + `PI_` prefix for futures. Existing `parse_response_symbol` in endpoints.rs can stay; tricky `from_exchange` parsing. |
| 6 | Coinbase | `cex/coinbase/` | Low | `BASE-QUOTE` always. |
| 7 | Gate.io | `cex/gateio/` | Low | `BASE_QUOTE` underscore always. Known ts=0 parser bug is separate. |
| 8 | Gemini | `cex/gemini/` | Low | `basequote` lowercase no separator. Simple concat. |
| 9 | MEXC | `cex/mexc/` | Low | Spot `BASE_QUOTE`, futures `BASE_USDT`. |
| 10 | HTX | `cex/htx/` | Low | Spot lowercase concat, futures `BASE-USDT`. |
| 11 | Bitget | `cex/bitget/` | Low | Spot uppercase concat, futures `BASEUSDTPERP`. Reference WS migration crate. |
| 12 | BingX | `cex/bingx/` | Low | `BASE-QUOTE` always. |
| 13 | Crypto.com | `cex/crypto_com/` | Low | Spot `BASE_QUOTE`, futures `BASEUSD-PERP`. |
| 14 | Upbit | `cex/upbit/` | Medium | **Reversed**: `QUOTE-BASE`. Previously guessed inside connector → root cause of reversed symbol bug. After Phase α: caller passes `KRW-BTC` directly, bug eliminated. |
| 15 | Bitfinex | `cex/bitfinex/` | Medium | `t` prefix, USDT→USD mapping, futures `F0` suffix. Previously failed silently on empty symbol. Bug eliminated. |
| 16 | Bitstamp | `cex/bitstamp/` | Low | `basequote` lowercase. Spot-only. Silent WS bug is separate. |
| 17 | Deribit | `cex/deribit/` | High | `BTC-PERPETUAL`, `SOL_USDC-PERPETUAL`, options require full instrument name. `format_symbol` currently returns `ExchangeResult<String>` (fallible). Normalizer mirrors this: returns `Err(RequiresRawInstrument)` for options. Connector methods must handle the `_` account type arm. |
| 18 | HyperLiquid | `cex/hyperliquid/` | Medium | Perp = coin name (`BTC`), spot = `@index`. `symbol_to_market_id` table stays in endpoints.rs, called by normalizer. |
| 19 | dYdX v4 | `dex/dydx/` | Low | Always `BASE-USD`. Existing `normalize_symbol` in endpoints.rs reused by normalizer. |
| 20 | Lighter | `dex/lighter/` | Medium | Perp = coin name, spot = `BASE/QUOTE`. `symbol_to_market_id` table stays in endpoints.rs for WS channel IDs. |
| 21 | Polymarket | `prediction/polymarket/` | Low | No canonical symbol. Callers always pass raw condition ID/slug. `to_exchange` returns passthrough or `Err(UnsupportedAccountType)`. |
| 22 | MOEX | `l2/free/moex/` | Low | `BASE` ticker only. Separate known factory bug (WS returns UnsupportedOperation). |

---

## 5. Migration Order

### Sub-phase α.1 — Core infrastructure (sequential, must merge first)

Files to modify:

1. `src/core/traits/market_data.rs` — change 4 method signatures (`Symbol`/`&Symbol` → `&str`).
2. `src/core/traits/market_data_public.rs` — change 8 method signatures.
3. `src/core/websocket/stream_spec.rs` — `StreamSpec.symbol: Symbol` → `String`; update both `TryFrom<SubscriptionRequest>` and `From<StreamSpec>`.
4. `src/core/traits/websocket.rs` — `WebSocketExt` convenience methods: change `symbol: Symbol` params to `symbol: String`.
5. `src/core/utils/mod.rs` — add `pub mod symbol_normalizer;` + re-export.

Files to create:

6. `src/core/utils/symbol_normalizer.rs` — `NormalizerError` enum + `SymbolNormalizer` struct with `to_exchange`, `from_exchange`, `is_valid_for`. Skeleton only: all arms `todo!()` or `unimplemented!()` — flesh out per-exchange arms in α.2.

After α.1 merges, the crate will **not compile** until all connectors are updated. This is expected. α.2 agents work against the α.1 branch.

### Sub-phase α.2 — Per-connector migration (parallel batches)

Each connector agent does exactly:
1. Remove `format_symbol(...)` calls in `connector.rs` methods — use the `symbol: &str` param directly.
2. Update `protocol.rs` / `websocket.rs` `subscribe_frame` to use `spec.symbol.as_str()`.
3. Fix any `_tests_websocket.rs` fixture symbol strings to raw format.
4. Fill the corresponding `ExchangeId` arm in `SymbolNormalizer::to_exchange` + `from_exchange` + `is_valid_for` (coordinate via PR; alternatively each agent writes to a per-exchange sub-function called from the central match — avoids merge conflicts).

**Batch A** (Low difficulty — 7 agents in parallel):
Binance, Bybit, OKX, Coinbase, Gate.io, Gemini, BingX

**Batch B** (Low-Medium — 7 agents in parallel):
KuCoin, Kraken, MEXC, HTX, Bitget, Bitstamp, Crypto.com

**Batch C** (Medium — 5 agents in parallel):
Upbit, Bitfinex, HyperLiquid, dYdX, Lighter

**Batch D** (High/Special — 3 agents in parallel):
Deribit, Polymarket, MOEX

Each agent also updates the `SymbolNormalizer` arms for their exchange. To avoid git conflicts on `symbol_normalizer.rs`, the recommended pattern is:

```rust
// symbol_normalizer.rs
pub fn to_exchange(id: ExchangeId, sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> {
    match id {
        ExchangeId::Binance => binance::to_exchange(sym, account_type),
        ExchangeId::Bybit => bybit::to_exchange(sym, account_type),
        // ...
        _ => Err(NormalizerError::UnknownExchange(id)),
    }
}

mod binance {
    use super::*;
    pub fn to_exchange(sym: &Symbol, account_type: AccountType) -> Result<String, NormalizerError> { ... }
}
```

Each batch-agent adds one `mod <exchange>` block — no conflicts because each block is in a different section of the file.

### Sub-phase α.3 — Consumer + examples update

1. `examples/deep_smoke.rs` — update all `Symbol::new(...)` constructions feeding connectors to `SymbolNormalizer::to_exchange(...)` calls.
2. `examples/exchange_hub_demo.rs` — same.
3. Any `tests/` files using `get_ticker(Symbol::new(...))` → `get_ticker("BTCUSDT", ...)`.
4. `WebSocketExt::subscribe_ticker(symbol: String)` callsites in examples.

### Sub-phase α.4 — Compile gate + validation

```bash
cd digdigdig3
$env:RUSTFLAGS="-D warnings"
cargo check --all-targets --all-features
cargo test --lib --all-features
cargo build --example deep_smoke --release
```

Do not run `deep_smoke.exe` as gating step for α.4 — live API calls are a post-merge concern. Compile gate + unit tests are sufficient.

---

## 6. SymbolNormalizer Registry Strategy

**Chosen: Option B — single central match with per-exchange sub-modules.**

Architecture:
```
src/core/utils/symbol_normalizer.rs
  ├── pub enum NormalizerError
  ├── pub struct SymbolNormalizer
  ├── impl SymbolNormalizer { to_exchange, from_exchange, is_valid_for }
  ├── mod binance { fn to_exchange / from_exchange / is_valid_for }
  ├── mod bybit   { ... }
  ├── mod okx     { ... }
  ├── ...22 sub-modules total...
```

Each sub-module is `pub(super)` (not exported). The public API is only `SymbolNormalizer::{to_exchange, from_exchange, is_valid_for}`. Sub-modules import from their exchange's `endpoints.rs` for helper functions (e.g., `kraken::endpoints::parse_response_symbol`).

**Why not trait dispatch (Option A):**
- Would require a new `SymbolMapper` trait implemented in 22 different exchange modules.
- Each connector module would need to export it, coupling the normalizer to connector internals.
- The per-exchange format logic is too small (1–5 LOC each) to justify the abstraction overhead.
- Central match allows the full per-exchange table to be reviewed in one file — critical for correctness audits.

---

## 7. Backward Compat Strategy

**No-compat — hard break at v0.3.0.**

Rationale:
- `mli-collector` already uses `Symbol::with_raw("", "", raw_string)` pattern at `subscriber.rs:122` and `exchange_hub_fetcher.rs:72`. It constructs `Symbol` with an empty base/quote and the raw string. After Phase α: `SubscriptionRequest.symbol.raw()` gives the raw string, `TryFrom` extracts it into `StreamSpec.symbol: String`. mli-collector's WS subscription path **already works** with the new model.
- mli-collector REST path (`exchange_hub_fetcher.rs:83`) calls `conn.get_klines(sym, ...)` where `sym = Symbol::with_raw("","",symbol)`. After Phase α the method takes `&str` — update to `conn.get_klines(symbol, ...)` where `symbol` is the raw string from config.
- Total mli-collector changes: ~15–20 callsites in `exchange_hub_fetcher.rs` + `subscriber.rs`. One dedicated implementer agent handles this in α.3.

No `#[deprecated]` wrappers. Old `Symbol`-taking signatures are removed entirely.

---

## 8. Consumer Impact — mli-collector

### Files affected
- `mylittleindicators/src/data_loader/exchange_hub_fetcher.rs` — REST path
- `mylittleindicators/crates/mli-collector/src/subscriber.rs` — WS subscription path

### Current pattern (already partially correct)

`subscriber.rs:122`:
```rust
let symbol = Symbol::with_raw("", "", sub.symbol.clone());
let req = SubscriptionRequest { symbol, stream_type, account_type, ... };
```
This already stores the raw string. After α.1 the `TryFrom<SubscriptionRequest> for StreamSpec` extracts `symbol.raw()` → `StreamSpec.symbol: String`. No change needed in subscriber.rs WS subscription block.

`exchange_hub_fetcher.rs:72`:
```rust
let sym = Symbol::with_raw("", "", symbol.to_string());
// then: conn.get_klines(sym, "1m", ...) — takes Symbol (owned) today
```
After α.1: `get_klines` takes `symbol: &str`. Change to `conn.get_klines(symbol, "1m", ...)` directly (the `symbol: &str` param from `RestFetcher::fetch`). The `sym` construction is deleted.

Same pattern applies for `get_orderbook`, `get_recent_trades`, `get_funding_rate_history`, etc. — each takes `symbol: &str`, so just pass the raw string from the fetcher's `symbol: &str` param.

### Estimated mli-collector changes
- ~12 trait method callsites in `exchange_hub_fetcher.rs`
- 0 changes needed in `subscriber.rs` WS path (already raw-string-first)
- `WebSocketExt` convenience methods: if called anywhere in mli, update `Symbol` args to `String`

---

## 9. Risks and Open Questions

### Risk 1: OrderRequest / CancelRequest / filter types embedding Symbol
**Check**: Verify `OrderRequest`, `CancelRequest`, `PositionQuery`, `OrderHistoryFilter`, `UserTradeFilter` in `src/core/types/trading.rs` for `Symbol` fields. If present, those are additional breaking changes in α.1.
**Mitigation**: Implementer of α.1 must read `src/core/types/trading.rs` fully before making changes.

### Risk 2: Connector inherent methods on BinanceConnector etc.
Binance connector has inherent method `get_klines_paginated(symbol: Symbol, ...)` at `connector.rs:199`. This is **not** a trait method — it takes owned `Symbol` today. Must be migrated to `symbol: &str` in the Binance agent's α.2 work. Search for similar inherent methods in other connectors.

### Risk 3: Bybit format_symbol signature difference
`bybit/endpoints.rs:295`: `pub fn format_symbol(symbol: &Symbol, _account_type: AccountType) -> String` — takes `&Symbol` not `(&str, &str, AccountType)`. The normalizer sub-module for Bybit must replicate this logic (`symbol.base + symbol.quote` uppercase concat) without calling through to the old signature.

### Risk 4: Deribit's fallible format_symbol
`deribit/endpoints.rs:239`: `pub fn format_symbol(...) -> ExchangeResult<String>` — returns `Result`. The normalizer's `to_exchange` for Deribit must propagate this as `Err(NormalizerError::RequiresRawInstrument)` for `AccountType::Options`. Deribit connector methods that took `symbol: Symbol` and called `format_symbol?` must change to accepting `symbol: &str` and trusting the caller supplied a valid instrument name.

### Risk 5: WS protocol files using format_symbol
Grep of the codebase shows `format_symbol` appears in `protocol.rs` files (e.g., `mexc/protocol.rs`, `kucoin/protocol.rs`, `okx/protocol.rs`). These must also be updated in α.2 — the connector-level agent for each exchange covers `protocol.rs` as well.

### Risk 6: Merge conflicts on symbol_normalizer.rs in parallel batch
Mitigated by the sub-module pattern (each agent adds one `mod exchange {}` block, no arm conflicts). Agents must coordinate that the outer `match id { ... }` arms are added without overlapping lines. Recommended: α.1 agent adds all 22 arms as `ExchangeId::X => X::to_exchange(sym, account_type),` pointing to stub sub-modules returning `Err(NormalizerError::UnknownExchange(id))`. Each α.2 agent then only fills in their sub-module — zero conflict.

### Risk 7: deep_smoke uses Symbol::new internally
`examples/deep_smoke.rs` constructs `Symbol::new("BTC","USDT")` and passes to trait methods. After α.1 these are `&str` params. The α.3 agent must update all examples to pass raw strings, which requires knowing the correct raw format per exchange — use `SymbolNormalizer::to_exchange` in the examples, or hardcode known-correct raw strings per exchange (acceptable for smoke tests).

### Open question: Lighter numeric market IDs in WS
Lighter's WS channel uses numeric market IDs (`0`, `1`, `2`...) derived from `symbol_to_market_id`. The `protocol.rs` `subscribe_frame` currently calls this to get the channel number. After α.2: `spec.symbol` will be `"ETH"` (raw). `subscribe_frame` must call `symbol_to_market_id("ETH")` directly — this stays in `endpoints.rs`, called from `protocol.rs`. No change to the mapping table itself.

### Open question: where does mli-collector get the right raw symbol?
Config-driven: `CollectorConfig` stores raw exchange symbols as strings (already). The `sub.symbol` field is already the raw string (e.g., `"BTCUSDT"` for Binance). This is confirmed by `subscriber.rs:122` which passes `sub.symbol.clone()` as the raw string. No config format change needed.

---

## Files to Create

- `src/core/utils/symbol_normalizer.rs` — `NormalizerError` + `SymbolNormalizer` + 22 per-exchange sub-modules

## Files to Modify (α.1)

- `src/core/traits/market_data.rs` — 4 method sigs
- `src/core/traits/market_data_public.rs` — 8 method sigs
- `src/core/websocket/stream_spec.rs` — `StreamSpec.symbol: String` + conversion impls
- `src/core/traits/websocket.rs` — `WebSocketExt` convenience method params
- `src/core/utils/mod.rs` — add `symbol_normalizer` module

## Files to Modify (α.2, per-connector — 22 × ~3 files each)

For each of 22 connectors:
- `{connector}/connector.rs` — remove `format_symbol` calls, use `symbol: &str` params
- `{connector}/protocol.rs` or `websocket.rs` — `subscribe_frame` uses `spec.symbol.as_str()`
- `{connector}/_tests_websocket.rs` — fixture raw strings

Plus:
- `src/core/utils/symbol_normalizer.rs` — per-exchange sub-module filled in

## Files to Modify (α.3)

- `examples/deep_smoke.rs`
- `examples/exchange_hub_demo.rs`
- `mylittleindicators/src/data_loader/exchange_hub_fetcher.rs`
- Any `tests/` files using `Symbol`-taking trait methods

---

## Estimated Complexity

**High** — large blast radius (22 connectors, 12 trait changes, 1 new utility module, 1 consumer crate). Individual connector changes are **Low** each. The sequencing risk (α.1 breaks everything until α.2 is complete) is the primary execution risk.

Recommended: merge α.1 to a feature branch, run all α.2 batches against that branch, merge α.2 batches sequentially (A then B then C then D to keep the crate compilable on the feature branch incrementally), then α.3, then merge feature branch to main.
