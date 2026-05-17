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

Symbol translation is a **separate utility** (`src/core/utils/symbol_normalizer.rs` — to be built). Callers that want one canonical Symbol → per-exchange raw use the normalizer explicitly:
```rust
let raw = SymbolNormalizer::for_exchange(ExchangeId::Binance, &Symbol::new("BTC", "USDT"));
// raw == "BTCUSDT"
conn.get_ticker(&raw).await?;
```

This separates concerns:
- **Connector** = wire protocol shim, knows only its exchange's native format
- **Normalizer** = canonical ↔ raw translation, lives outside connectors
- **Consumer** = chooses whether to feed canonical (via normalizer) or raw

### 3. Capabilities self-declared AND empirically validated

Two-level capability surface:

- **Declared** — `HasCapabilities::capabilities() -> ConnectorCapabilities` (71 flags) declared per-connector at impl time.
- **Derived** — `CapabilityProvider::supports(StreamKind, AccountType) -> SupportLevel` automatically derived from `WsProtocol::topic_registry()` (cannot drift from reality).
- **Empirical** (TBD) — `validation_status: Option<ValidationStamp>` populated by smoke-validation harness, asserting "this method has been observed to return real data on date X". Required before consumers trust `capabilities()`.

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

### Layer 3: Live deep_smoke (validation gate)

`examples/deep_smoke.rs` — parallel async harness covering EVERY exchange. Per-target row:
- REST: connect + `get_ticker(BTC/USDT)` + assert real fields (last_price > 0, volume > 0, recent timestamp)
- WS: subscribe to ticker, collect 5s window, **inspect first event content** (not just count)
- Three bug classes detected:
  - **A**: connection fails (auth/network/symbol unknown)
  - **B**: subscribed but silent (registry/format gap)
  - **C**: events flowing BUT typed struct has zero/default fields (parser bug)

Must run in parallel: `tokio::spawn` per exchange + `join_all`, never sequential. One hang must not stall the harness. Each task capped at 25s.

```
cd digdigdig3
cargo build --example deep_smoke --release
target\release\examples\deep_smoke.exe 2>&1 | tee deep_smoke_report.txt
```

Validation gate: a connector is considered "validated" only when Layer 3 reports REST+WS green with non-default data. The connector's `capabilities()` should ONLY claim what Layer 3 confirms.

## Scope of development

### In scope
- L3-open crypto (CEX + DEX + Polymarket) — primary consumer surface
- Public market data (klines/ticker/orderbook/trades/funding/OI) over REST + WS
- Trading + Account + Positions traits per exchange (gated by API keys)
- Capability discovery + empirical validation
- `ExchangeHub` as single consumer-facing API

### Out of scope (deferred to other crates / future)
- On-chain monitoring → `dig2chain`
- High-frequency execution paths beyond current trait surface
- Per-exchange UI / dashboard (consumer = `mylittlechart`)
- Symbol normalization INSIDE connectors (use external `SymbolNormalizer` utility)
- Legacy `base_websocket.rs` and old bespoke WS loops — replaced by `UniversalWsTransport`

### Currently broken / known gaps (priority order)
1. **Class C parser bugs** (8 connectors emitting StreamEvents with default/zero fields): GateIO ts=0, Upbit symbol reversed + stale ts, Bitfinex/KuCoin symbol="", BingX/Coinbase/Gemini REST volume=0, CryptoCom wrong stream variant
2. **MOEX factory bug** — `create_websocket(Moex)` returns UnsupportedOperation despite working `MoexWebSocket::new_public()` existing
3. **5 silent streams** (Bitstamp, Deribit, Dydx, MOEX, YahooFinance) — symbol format gaps
4. **HTX server-pong reply** — known limitation (commit e214995), framework reconnect compensates
5. **README.md stale** (refs deleted traits + old architecture)
6. **L1/L2/L3-gated connectors untested** (43 total, only 14 CEX production-tested)

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
cargo build --example deep_smoke --release
target\release\examples\deep_smoke.exe

# Quick hub demo (3 exchanges)
cargo run --example exchange_hub_demo --release
```

## File pointers

- Architecture entry: `src/connector_manager/hub.rs` (ExchangeHub)
- WS framework: `src/core/websocket/{transport.rs, protocol.rs, topic_registry.rs, stream_kind.rs}`
- Trait composition: `src/core/traits/mod.rs`
- Capability struct: `src/core/types/capabilities.rs`
- Reference WS migration: `src/l3/open/crypto/cex/bitget/{protocol.rs, websocket.rs}`
- Validation harness: `examples/deep_smoke.rs` + `examples/exchange_hub_demo.rs`
- Plans: `docs/plans/wave0-foundation.md`, `docs/plans/smoke_v8_findings_spec.md`, `docs/plans/ws-rest-inventory.md`

## Gotchas

- README.md at root is **stale** (v0.1.20, references deleted traits MarginTrading/EarnStaking/etc and old `base_websocket.rs`). Cargo.toml is v0.2.0. Do not trust README for architecture facts; trust CLAUDE.md and code.
- Windows codepage: prefix Windows-native commands with `chcp.com 65001 > $null 2>&1;` for UTF-8.
- NEVER chain git commands with `&&`. Separate `git add` / `git commit` calls.
- digdigdig3 is a git submodule with its own `.git`. `cd digdigdig3` before any git command.
- Do NOT bump version unless explicitly asked.
- Do NOT push to remote unless explicitly asked.
