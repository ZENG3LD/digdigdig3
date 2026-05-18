# digdigdig3 (dig3)

Multi-exchange connector library — single `ExchangeHub` async pool exposing all connectors with self-declared capabilities.

## Architectural principles

### 1. Hub-first API surface

`ExchangeHub` is the **sole public entry point** for all multi-connector operations. All pool/factory internals are `pub(crate)` — external code cannot bypass the hub.

#### PUBLIC types (`digdigdig3::connector_manager::*`)
- `ExchangeHub` — single entry for all operations
- `AuthType`, `ConnectorCategory`, `ConnectorMetadata`, `Features` — read-only registry metadata
- All types in `core::types::*` and traits in `core::traits::*`
- `core::websocket::{StreamSpec, StreamKind, KlineInterval, SupportLevel}` — stream construction
- `core::utils::SymbolNormalizer`, `core::utils::validation_snapshot::validation_for`

#### INTERNAL (`pub(crate)`)
- `ConnectorPool`, `WebSocketPool` — pool internals accessed only through hub
- `ConnectorFactory` — connector construction, only called from hub
- `ConnectorRegistry` — static metadata accessor, only used for test harness

#### Hub API
- `hub.connect_full(id, &[AccountType], testnet)` — wires REST + WS
- `hub.connect_public(id, testnet)` — REST only
- `hub.connect_websocket(id, account_type, testnet)` — WS only
- `hub.rest(id) -> Option<Arc<dyn CoreConnector>>` — typed dispatch
- `hub.ws(id, account_type) -> Option<Arc<dyn WebSocketConnector>>` — WS dispatch
- `hub.shutdown(id)` — releases REST + WS
- `hub.list_connected() -> Vec<ExchangeId>` — all REST-connected exchanges
- `hub.is_connected(id) -> bool` — check REST connectivity
- `hub.capabilities(id) -> Option<ConnectorCapabilities>` — capability query

### 2. Raw exchange symbols inside connectors

**Connectors accept and emit exchange-native symbol strings.** Binance gets `"BTCUSDT"`, OKX gets `"BTC-USDT"`, Gate.io gets `"BTC_USDT"`. No internal "canonical Symbol{base, quote}" massaging.

Symbol translation is a **separate utility** (`src/core/utils/symbol_normalizer.rs`). 22 in-scope exchanges each have a per-exchange sub-module with `to_exchange` + `from_exchange` rules. Callers that want canonical → raw use the normalizer explicitly:
```rust
let raw = SymbolNormalizer::to_exchange(ExchangeId::Binance, &Symbol::new("BTC", "USDT"), AccountType::Spot)?;
// raw == "BTCUSDT"
conn.get_ticker(&raw, AccountType::Spot).await?;
```

This separates concerns:
- **Connector** = wire protocol shim, knows only its exchange's native format
- **Normalizer** = canonical ↔ raw translation, lives in `core::utils::symbol_normalizer` (22 sub-modules)
- **Consumer** = chooses whether to feed canonical (via normalizer) or raw

### SymbolInput — raw or canonical, per-call

Every per-symbol connector method takes `SymbolInput<'_>`:

```rust
pub enum SymbolInput<'a> {
    Raw(&'a str),            // "tBTCUSD" — used as-is
    Canonical(&'a Symbol),   // &Symbol::new("BTC","USD") — normalized inside connector
}
```

Three call styles, all valid:

```rust
// 1. Raw, terse — use exchange-native string directly
conn.get_ticker("tBTCUSD".into(), AccountType::Spot).await?;

// 2. Canonical — exchange-agnostic
let sym = Symbol::new("BTC", "USD");
conn.get_ticker((&sym).into(), AccountType::Spot).await?;

// 3. Macro
conn.get_ticker(sym!("tBTCUSD"), AccountType::Spot).await?;          // Raw
conn.get_ticker(sym!(&canonical_symbol), AccountType::Spot).await?;  // Canonical
```

Inside the connector, `SymbolInput::resolve(exchange, account_type) -> Cow<'_, str>` dispatches. Raw → identity (zero allocation). Canonical → SymbolNormalizer.

For long-lived contexts (e.g. `StreamSpec.symbol`), use `OwnedSymbolInput` with same Raw/Canonical variants.

Per-call dispatch (not compile-time): caller can mix Raw and Canonical in a loop over multiple exchanges without picking a different method name.

Per-exchange normalization rules are in `src/core/utils/symbol_normalizer.rs` (22 sub-modules).

### 3. Capabilities self-declared AND empirically validated

Two-level capability surface:

- **Declared** — `HasCapabilities::capabilities() -> ConnectorCapabilities` (71 flags) declared per-connector at impl time.
- **Derived** — `CapabilityProvider::supports(StreamKind, AccountType) -> SupportLevel` automatically derived from `WsProtocol::topic_registry()` (cannot drift from reality).
- **Empirical** — `HasCapabilities::validation_status() -> Option<ValidationStamp>` exposes per-method/stream validation from the `e2e_smoke` harness. Embedded snapshot at `data/validation_snapshot.json` (22 entries). `hub.connect_full_validated(...)` rejects exchanges without a valid stamp — use for production flows that require confirmed data quality.

### 4. WebSocket: UniversalWsTransport, no bespoke loops

`UniversalWsTransport<P: WsProtocol>` in `src/core/websocket/transport.rs` owns:
- connect/reconnect/backoff
- ping scheduler
- subscription registry + replay on reconnect
- frame dispatch (NO `_ => Ok(None)` catch-alls allowed)
- tracing on every frame (`target: "dig3::ws::frame"`)
- unmatched topic warning (`target: "dig3::ws::unmatched"`)

Each exchange implements only `WsProtocol` (`endpoint`, `ping_frame`, `subscribe_frame`, `topic_registry`, `extract_topic`). Approximate cost: ~150 LOC websocket.rs wrapper + ~400-900 LOC protocol.rs.

Old `base_websocket.rs` is dead — do not extend or reference. UniversalWsTransport supersedes it.

### 5. Async-first, never block the runtime

- `tokio::sync::Mutex` only. `std::sync::Mutex` across `.await` is forbidden.
- Blocking I/O wrapped in `spawn_blocking`.
- Sync sleeps banned (`std::thread::sleep` → `tokio::time::sleep`).
- Rate limiter loops MUST yield (Lighter busy-spin bug 2d254e8 is the cautionary tale).

## Test plan — three layers

### Layer 1: Compile gate (every commit)

```
cd digdigdig3
chcp.com 65001 > $null 2>&1
$env:RUSTFLAGS="-D warnings"
cargo check --all-targets --all-features
```

0 errors, 0 warnings. Mandatory.

### Layer 2: Unit tests (per-module)

Each `*/parser.rs` has fixture-based tests (captured exchange payloads → assert parsed struct fields). Each `*/protocol.rs` has registry+frame extraction tests. Each new fix requires the regression test.

```
cargo test --lib --all-features
```

### Layer 3: Live e2e_smoke (validation gate)

`examples/e2e_smoke.rs` — parallel async coverage **matrix**: per exchange, 13 REST methods × 9 WS streams. Each cell tagged `OK / EMPT / ERR / TIME / -- / SKIP`. Full matrix on 43 exchanges runs in ~25s.

CLI:
```
cargo run --example e2e_smoke --release --              # market only, all exchanges
cargo run --example e2e_smoke --release -- --exchange Binance
cargo run --example e2e_smoke --release -- --trading    # private paths, reads ENV creds
cargo run --example e2e_smoke --release -- --all
cargo run --example e2e_smoke --release -- --json-out report.json
```

REST methods tested: ping, price, ticker, orderbook, klines, recent_trades, exchange_info, funding_rate, open_interest, mark_price, long_short_ratio, liquidations, premium_index.

WS streams tested: Ticker, Trade, Orderbook, Kline, MarkPrice, FundingRate, Liquidation, OpenInterest, AggTrade.

Five bug classes detected by the strict inspector:
- **Class 1**: REST connect fails (auth/network/symbol unknown)
- **Class 2**: subscribed but silent (registry/format gap)
- **Class 3**: events flowing BUT typed struct has zero/default fields (parser bug)
- **Class 4**: WS event content issues — timestamp in seconds-not-ms / future-tz / bid==ask / bid>ask / WRONG_TYPE routing
- **TRUSTED**: REST+WS both populated AND zero Class-1..4 issues

Inspector flags timestamp_unit_bug (ts < now/100 → seconds-not-ms), timestamp_future_bug (ts > now+60s → timezone bug), ts_missing (== 0).

Must run in parallel: `tokio::spawn` per exchange + `join_all`, never sequential. One hang must not stall the harness. Each exchange task capped at 60s.

```
cd digdigdig3
cargo build --example e2e_smoke --release
target\release\examples\e2e_smoke.exe > e2e_report.txt 2>&1
```

Outputs: human report (stdout) + regenerated `data/validation_snapshot.json`.

Validation gate: a connector is **TRUSTED** only when the matrix reports every declared capability OK with no Class-1..4 issues. The connector's `capabilities()` should ONLY claim what the matrix confirms.

### Unsupported convention — `UnsupportedOperation` vs `NotSupported`

Strict naming so future audits know what to research and what to leave alone:

| Variant | Meaning | When to use |
|---|---|---|
| `ExchangeError::UnsupportedOperation(reason)` | **TODO_Implement** — exchange supports the endpoint, we have not written the code | placeholder during scaffolding; tracked as a real regression by e2e_smoke |
| `ExchangeError::NotSupported(reason)` | **Wire-not-present** — the exchange itself does not expose this method publicly | document the alternative (WS-only feed, history-only, auth-tier-required); never resolves to TODO |
| `WebSocketError::NotSupported(reason)` | Same, for WS subscribe paths. The blanket `subscribe` in `transport.rs` eagerly returns this from `subscribe_frame()` before queueing, so callers see the error immediately (not after a 5s silent timeout) | apply when the channel does not exist in the exchange's WS spec |

Reasons MUST cite the alternative (e.g. `"NotSupported: Binance does not expose realtime WS open interest — use REST GET /fapi/v1/openInterest with polling"`). This shows up unmodified in the e2e matrix and saves a future agent the round-trip to the docs.

### deep_smoke / e2e_smoke history

The harness was renamed from `deep_smoke` to `e2e_smoke` (commit 4866465) — it is end-to-end against live exchange APIs, not a smoke layer. Old artefact paths (`deep_smoke_*.txt`) are still excluded in `Cargo.toml` for legacy reasons.

## Scope of development

### In scope
- L3-open crypto (CEX + DEX + Polymarket) — primary consumer surface
- Public market data (klines/ticker/orderbook/trades/funding/OI) over REST + WS
- Trading + Account + Positions traits per exchange (gated by API keys)
- Capability discovery + empirical validation
- `ExchangeHub` as single consumer-facing API
- **Validated subset**: 22 connectors (L3-open CEX 18 + DEX 2 + Pred 1 + MOEX 1). Functional validation complete — see `data/validation_snapshot.json`.
- **L1/L2-paid + L3-gated** (21 exchanges): compile-validated only; functional validation deferred until API keys available.

### Out of scope (deferred to other crates / future)
- On-chain monitoring → `dig2chain`
- High-frequency execution paths beyond current trait surface
- Per-exchange UI / dashboard (consumer = `mylittlechart`)
- Symbol normalization INSIDE connectors (use external `SymbolNormalizer` utility)
- Legacy `base_websocket.rs` and old bespoke WS loops — replaced by `UniversalWsTransport`

### Known gaps (post-coverage-sweep state, May 2026)

After the research-driven coverage sweep (commit `ecb0ed5`) every L3-open exchange has ~10-11 of 13 REST methods OK. Remaining work is **WS-side**:

- **WS futures streams subscribed via Spot account_type fail** — e2e_smoke currently passes `AccountType::Spot` to `hub.connect_websocket()` for all streams. `MarkPrice / FundingRate / Liquidation / OpenInterest / AggTrade` live on **separate WS endpoints** (e.g. `wss://fstream.binance.com/ws` for Binance, `/v5/public/linear` for Bybit). Need to route futures streams through `AccountType::FuturesCross` so the hub picks the futures endpoint. This is the next-up task — see `docs/testing-plan.md`.
- **WS Orderbook ERR on several CEX** — Binance/HTX/MEXC/Bitget. The subscribe channel is correct, but the parser drops the snapshot because the symbol is not in the payload (it's encoded in the channel name). Need to track `channel → symbol` in subscription context inside `UniversalWsTransport`.
- **Bybit WS Ticker bid/ask = None** — `tickers.{sym}` payload carries `bid1Price`/`ask1Price`; parser extracts them via `parse_ws_ticker` but they're not propagating to the `StreamEvent::Ticker`. Likely a field-name mismatch after the WS rewrite.
- **dYdX WS `Trade`/`Orderbook` ERR** — `subscribe_frame` returns OK but dispatch maps `v4_orderbook` content to `OrderbookDelta` with empty arrays. Needs the order-book snapshot/delta merge logic that the Indexer WS guide describes.
- **MOEX WS** — `bid/ask both None` on the FAST/CEDR stream when running outside RU IP space. REST connect OK; WS event rate unreliable from non-RU networks.
- **L1/L2-paid + L3-gated connectors** (21 exchanges) — return `Auth` error without credentials. e2e_smoke `--trading` reads ENV (`{EXCHANGE}_API_KEY` / `{EXCHANGE}_API_SECRET` etc.). Functional validation runs only when ENV is populated.

### Closed gaps (historical, do not re-investigate)

- HyperLiquid `get_ticker` — universe loader added (`OnceCell<HashMap<String, usize>>` in `HyperliquidConnector`), `metaAndAssetCtxs` flat ctx array parser.
- Upbit ticker timestamp — parser falls back to `trade_timestamp → now()` (commit 02f2d15).
- MOEX REST timezone — `SYSTIME` parsed as Moscow local via `FixedOffset::east(3*3600)`.
- Bitfinex / GateIO / HyperLiquid REST `timestamp = 0` — parsers stamp `now_ms()` when wire format has no timestamp field.
- Yahoo REST `regularMarketTime` — multiplied ×1000 (was treated as ms but is seconds).
- CryptoCompare bitmask parser — idx starts at 5 (after FLAGS), reads BID/OFFER bits properly.
- Gemini synthetic Ticker — `parse_ws_l2_ticker` builds Ticker from `changes` top-of-book + `trades[-1].price`.
- Lighter `last_updated_at` — divided by 1000 (µs → ms).
- HyperLiquid Ticker WS — switched from `activeAssetCtx` to `bbo` channel; activeAssetCtx entry removed from registry.
- BingX, MEXC bookTicker channel migrations.
- KRX — switched from broken Open API stub to live Data Marketplace scrape (`data.krx.co.kr/comm/bldAttendant/getJsonData.cmd` with browser User-Agent).
- Dukascopy — `get_ticker`/`get_price` honestly return `NotSupported` (no public live-quote REST endpoint).
- All 12 `UnsupportedOperation` regressions from the 0.2.x refactor — implemented (mark_price ×9, open_interest ×11, recent_trades ×7, long_short_ratio ×3).

## Per-module conventions

### Connector module layout

```
src/{level}/{tier}/{category}/{name}/
  ├── mod.rs          — pub re-exports
  ├── endpoints.rs    — URL constants, endpoint enum, symbol formatting helpers (callable but optional — caller may pass raw symbols directly)
  ├── auth.rs         — signing implementation
  ├── parser.rs       — JSON → typed struct (test fixtures required)
  ├── connector.rs    — trait implementations (CoreConnector + optional)
  ├── protocol.rs     — WsProtocol impl (NEW, post-Wave 2)
  └── websocket.rs    — thin wrapper over UniversalWsTransport<XProtocol> (~150 LOC)
```

Reference: `src/l3/open/crypto/cex/bitget/` (Wave 1 pilot).

### Trait composition

`CoreConnector` (mega-trait, blanket impl in `src/core/traits/mod.rs`) composes 15 sub-traits:
- ExchangeIdentity + MarketData + MarketDataPublic + Trading + Account + Positions
- + CancelAll + AmendOrder + BatchOrders + AccountTransfers + CustodialFunds + SubAccounts
- + FundingHistory + AccountLedger + HasCapabilities
- + Send + Sync + 'static
- + `as_any()` escape hatch for exchange-specific inherent methods

Do not add new sub-traits without aligning with the capability struct + hub plumbing.

### Capability struct

`ConnectorCapabilities` in `src/core/types/capabilities.rs` (71 fields). When you add a method, ALSO add a flag, AND fill it explicitly in EVERY L3-open crypto connector's `HasCapabilities::capabilities()`. There is NO default — compile fails if a CoreConnector implementor omits the declaration. This is by design (prevents drift).

`MarketDataCapabilities` (4 ws_* flags) is legacy — deprecated in favor of `CapabilityProvider::supports(StreamKind, AccountType) -> SupportLevel`. Do not extend the bool-flag list.

## Commands

```bash
# Compile gate
cd digdigdig3
$env:RUSTFLAGS="-D warnings"
cargo check --all-targets --all-features

# Unit tests
cargo test --lib --all-features

# Full validation smoke (live API, parallel async, ~10s for 48 exchanges)
cargo build --example e2e_smoke --release
target\release\examples\e2e_smoke.exe

# Quick hub demo (3 exchanges)
cargo run --example exchange_hub_demo --release
```

## File pointers

- Architecture entry: `src/connector_manager/hub.rs` (ExchangeHub)
- WS framework: `src/core/websocket/{transport.rs, protocol.rs, topic_registry.rs, stream_kind.rs}`
- Trait composition: `src/core/traits/mod.rs`
- Capability struct: `src/core/types/capabilities.rs`
- Reference WS migration: `src/l3/open/crypto/cex/bitget/{protocol.rs, websocket.rs}`
- Validation harness: `examples/e2e_smoke.rs` + `examples/exchange_hub_demo.rs`
- Plans: `docs/plans/wave0-foundation.md`, `docs/plans/smoke_v8_findings_spec.md`, `docs/plans/ws-rest-inventory.md`

## Gotchas

- Cargo.toml is v0.2.2 (v0.2.3 anticipated post-θ.6 bump). README.md matches. Trust CLAUDE.md and code for architecture facts.
- Windows codepage: prefix Windows-native commands with `chcp.com 65001 > $null 2>&1;` for UTF-8.
- NEVER chain git commands with `&&`. Separate `git add` / `git commit` calls.
- digdigdig3 is a git submodule with its own `.git`. `cd digdigdig3` before any git command.
- Do NOT bump version unless explicitly asked.
- Do NOT push to remote unless explicitly asked.
