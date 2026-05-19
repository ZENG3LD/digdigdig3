# dig3 Station — Phase 1 MVP Implementation Plan

**Status**: ✅ COMPLETE (commits `bc50508` → `dd4223c`, May 2026)
**Predecessor**: `station-architecture.md` (architecture design)
**Goal**: end-to-end working `dig3 watch trades binance btc-usdt` via the new Station layer, with persistence + REST cache + RAII subscription handle.

## Outcome — DONE

This single command works:

```
dig3 watch trades binance BTC-USDT --duration 30
```

Verified live:
1. ✅ Boots a Station with default features + ring CryptoProvider.
2. ✅ Subscribes via SubscriptionSet API.
3. ✅ Prints live Trade events to stdout (`<ts> <Exch> <symbol> <side> px=<n> qty=<n>`).
4. ✅ Persists each Trade to `./dig3_storage/trades/binance/spot/btcusdt/<YYYY-MM-DD>.dat` (41-byte fixed record + sparse `.idx` every 1024).
5. ✅ Duration timeout exits cleanly; handle drop closes the forwarder task.

Bonus delivered beyond original plan:
- `--storage-root <path>` + `DIG3_STORAGE_ROOT` env override.
- `DIG3_WS_TRACE=1` → `target/harness_out/ws_trace/<exch>.jsonl` default.
- `--json-out auto` → `target/harness_out/e2e_smoke_<ts>.json`.

## Commit chain

| Commit  | Step  | What landed |
|---------|-------|-------------|
| `bc50508` | Wave 11B | Architecture + Phase 1 plan + handoff snapshot docs |
| `6cefa7c` | Step 1 | Workspace split into 3 crates (core/station/cli) |
| `2fde113` | Step 6 | Station::subscribe wired; `dig3 watch trades` prints live trades |
| `5215371` | gitignore | Exclude e2e/liq/matrix run artefacts |
| `6605071` | Steps 4+5 | StationBuilder.storage_root + TradeWriter (binary append + sparse idx + day rotation) + RestCache wrapper |
| `66f3709` | refactor | Core = connectors only; station owns storage/cache/replay/cure/orderbook; bins moved to cli; tests moved to station |
| `dd4223c` | Step D | Harness artefact paths default to `target/harness_out/` |

## Per-step status

| Step | Title | Status | Notes |
|------|-------|--------|-------|
| 1 | Workspace split | ✅ done | core/station/cli + symbol rename `digdigdig3::` → `digdigdig3_core::` everywhere |
| 2 | dig3-station skeleton | ✅ done | All modules declared; persistence/cache/reconnect features wired |
| 3 | SubscriptionSet + SubscriptionHandle | ✅ done | Typed core enums (ExchangeId, AccountType, Stream); RAII via oneshot+mpsc |
| 4 | TradeWriter persistence | ✅ done | 41 B/record, sparse `.idx` every 1024, UTC day rotation. Unit test green |
| 5 | RestCache LRU + TTL | ✅ done as wrapper | `station::cache::ticker_cache(ttl)` over the (now-station-owned) `RestCache<K,V>`. Builder hook deferred to Phase 2 |
| 6 | Station::builder + .subscribe() | ✅ done | Lazy single mux per (exchange, account); writer per-entry; trade-only in Phase 1 |
| 7 | dig3-cli `watch trades` | ✅ done | `dig3 watch trades <exch> <sym> [--account] [--duration] [--persist] [--storage-root]` operational |
| 8 | Verify + commit | ✅ done | `cargo check --workspace --all-targets --all-features` clean; core 818/1 + station 50/0 |

Beyond plan:
- Step A — `storage_root` config trio (`.storage_root()` / `--storage-root` / `DIG3_STORAGE_ROOT`)
- Refactor — moved storage/orderbook/rest_cache/replay/cure from core to station (user directive: core = pure connectors)
- Step D — harness artefact defaults under `target/harness_out/`

## Step-by-step plan

### Step 1 — workspace split (no code change yet)

1. Move existing source: `digdigdig3/src/` → `digdigdig3/crates/dig3-core/src/`.
2. Move existing `Cargo.toml` (top-level) → `crates/dig3-core/Cargo.toml`. Update `[package].name = "digdigdig3-core"`.
3. Create new top-level `Cargo.toml`:
   ```toml
   [workspace]
   members = ["crates/dig3-core", "crates/dig3-station", "crates/dig3-cli"]
   resolver = "2"
   ```
4. Move `examples/` → `crates/dig3-core/examples/` (they use core directly, stay there).
5. Move `tests/` → `crates/dig3-core/tests/` (if any).
6. Move `data/` and `docs/` → keep at workspace root (shared assets).
7. Run `cargo check --workspace` — expect existing examples + lib to still compile cleanly.

Gate: `cargo check --workspace --all-features` clean, 821 tests pass (same as before).

### Step 2 — create dig3-station skeleton

1. `crates/dig3-station/Cargo.toml`:
   ```toml
   [package]
   name = "digdigdig3-station"
   version = "0.1.0"
   edition = "2021"

   [dependencies]
   digdigdig3-core = { path = "../dig3-core" }
   tokio = { version = "1", features = ["full"] }
   async-trait = "0.1"
   futures-util = "0.3"
   serde = { version = "1", features = ["derive"] }
   serde_json = "1"
   tracing = "0.1"
   dashmap = "5"
   thiserror = "1"

   # feature-gated deps
   zstd = { version = "0.13", optional = true }
   bincode = { version = "1.3", optional = true }
   crc32fast = { version = "1.4", optional = true }
   lru = { version = "0.12", optional = true }
   metrics = { version = "0.23", optional = true }
   metrics-exporter-prometheus = { version = "0.15", optional = true }
   sled = { version = "0.34", optional = true }

   [features]
   default = ["persistence", "cache", "multiplex", "reconnect"]
   full = ["default", "orderbook-tracker", "bar-loader", "metrics", "prometheus"]
   minimal = []
   persistence = ["zstd", "bincode", "crc32fast", "sled"]
   cache = ["lru"]
   multiplex = []
   reconnect = []
   orderbook-tracker = []
   bar-loader = []
   prometheus = ["metrics", "metrics-exporter-prometheus"]
   ```

2. `crates/dig3-station/src/lib.rs`:
   ```rust
   //! High-level consumer-facing layer over digdigdig3-core ExchangeHub.

   pub mod builder;
   pub mod station;
   pub mod subscription;
   pub mod error;

   #[cfg(feature = "persistence")] pub mod persistence;
   #[cfg(feature = "cache")] pub mod cache;
   #[cfg(feature = "reconnect")] pub mod reconnect;

   pub use builder::StationBuilder;
   pub use station::Station;
   pub use subscription::{SubscriptionSet, SubscriptionHandle, Stream as SubStream};
   pub use error::{StationError, Result};
   ```

3. Stub all module files with `// TODO: phase 1` comments. Make `Station::builder()` chainable but all options no-op for now.

Gate: `cargo check -p digdigdig3-station --all-features` clean.

### Step 3 — SubscriptionSet + SubscriptionHandle (no actor yet, mock events)

1. `subscription/set.rs`:
   ```rust
   pub struct SubscriptionSet {
       entries: Vec<Entry>,
   }
   struct Entry {
       exchange: ExchangeId,
       symbol: String,
       account_type: AccountType,
       streams: Vec<SubStream>,
   }
   pub enum SubStream { Ticker, Trade, Orderbook, Kline(String), MarkPrice, FundingRate, OpenInterest, Liquidation, AggTrade }
   ```

2. `subscription/handle.rs` with the RAII pattern (Drop sending Unsubscribe via unbounded mpsc).

3. `subscription/multiplexer.rs` — actor skeleton: tokio task receiving `MuxCmd::{Subscribe, Unsubscribe, Shutdown}`, holding `Arc<ExchangeHub>` from core.

For Phase 1, the multiplexer connects via `hub.connect_websocket(...)` and forwards `StreamEvent::Trade` from `hub.ws(...)` events stream into the handle's broadcast channel. Single sub per StreamKey; no multiplex yet (Phase 2).

Gate: `cargo check -p digdigdig3-station --all-features` clean.

### Step 4 — Persistence: trades binary append-only

1. `persistence/trades.rs`:
   ```rust
   pub struct TradeWriter {
       path: PathBuf,
       file: File,
       idx_file: File,
       records_since_idx: u32,
   }
   const RECORD_SIZE: usize = 41; // u64 ts_ns + f64 px + f64 qty + u8 side + u64 trade_id
   const SPARSE_IDX_EVERY: u32 = 1024;
   ```
2. `TradeWriter::append(record) -> io::Result<()>`: write 41 bytes, every 1024th record write `[u64 ts_ms, u64 offset]` to idx file.
3. `TradeWriter::flush()`: `file.sync_data()`.
4. Multi-day rotation: when day rolls over (UTC), reopen `<date>.dat`.

The multiplexer actor, on each `StreamEvent::Trade` it forwards to consumers, also calls `trade_writer.append(...)`.

Gate: integration test writes 1000 trades, reads them back via simple read loop. 1 fail tolerance for the new dydx test.

### Step 5 — REST cache LRU + TTL (minimal scope: just `get_ticker`)

1. `cache/rest_lru.rs`:
   ```rust
   pub struct RestCache {
       inner: Mutex<LruCache<RestKey, CachedEntry>>,
       ttl: HashMap<RestKind, Duration>,
   }
   ```
2. Wrapper around `hub.rest(id).get_ticker(...)` that checks cache first.

Phase 1 scope: just ticker. orderbook + symbol metadata in Phase 2.

Gate: micro-benchmark: 100 sequential get_ticker calls with cache on → only 1 network call (rest cached for ttl=1s).

### Step 6 — Station::builder + Station + .subscribe()

1. `builder.rs` fluent builder constructs a `StationConfig`, then `build().await` constructs a `Station`.
2. `Station` holds:
   - `Arc<ExchangeHub>` (from core)
   - `Option<TradeWriter>` map per (exchange, symbol)
   - `Option<RestCache>`
   - `DashMap<StreamKey, MultiplexerHandle>` for active mux actors
3. `Station::subscribe(set) -> Result<SubscriptionHandle>` lazily spawns one multiplexer per (exchange, stream, account) if not already running; sends Subscribe cmds; returns handle with merged broadcast::Receiver.

Gate: integration test: `let station = Station::builder().persistence(Persistence::on()).build().await?; let h = station.subscribe(SubscriptionSet::new().add(Binance, "BTC-USDT", Spot, [Trade])).await?; let event = h.recv().await; assert!(matches!(event, Some(Event::Trade { .. })));`

### Step 7 — dig3-cli skeleton + `dig3 watch trades`

1. `crates/dig3-cli/Cargo.toml`:
   ```toml
   [package]
   name = "digdigdig3-cli"
   version = "0.1.0"
   edition = "2021"

   [[bin]]
   name = "dig3"
   path = "src/main.rs"

   [dependencies]
   digdigdig3-station = { path = "../dig3-station", features = ["full"] }
   clap = { version = "4", features = ["derive"] }
   tokio = { version = "1", features = ["full"] }
   anyhow = "1"
   tracing-subscriber = "0.3"
   ```

2. `src/main.rs` + `commands/watch.rs`:
   - `dig3 watch trades <exchange> <symbol> [--account spot|cross|isolated] [--duration N]`
   - `dig3 watch kline <exchange> <symbol> --interval 1m`
   - `dig3 watch orderbook <exchange> <symbol> --depth 20` (Phase 4)
   - Other subcommands skeletal: `dig3 persist`, `dig3 replay`, `dig3 matrix` print "not yet implemented in Phase 1".

3. Watch command flow:
   - Parse args, build Station with defaults + persistence on (root from `--storage-root` or default `./dig3_storage`).
   - Build SubscriptionSet with one entry.
   - subscribe().
   - Print events for `--duration` secs OR until Ctrl-C.
   - drop(handle).

Gate: `cargo build --bin dig3 --release && target/release/dig3 watch trades binance btc-usdt --duration 30` runs end-to-end, prints trades, persists to disk.

### Step 8 — verification & commit

1. `cargo check --workspace --all-features` clean with `RUSTFLAGS=-D warnings`.
2. `cargo test --workspace` — 821+ tests pass.
3. New: `cargo test -p digdigdig3-station` — phase 1 integration tests.
4. Run `target/release/dig3 watch trades binance btc-usdt --duration 30` — verify trades printed + file written.
5. Run `target/release/examples/e2e_smoke` — TRUSTED still 18.
6. Single git commit on dig3 submodule: "feat(workspace): phase 1 — split into core/station/cli, MVP Station builder, dig3 watch trades".
7. Single git commit on nemo: submodule bump.

## Risks / things to verify before committing

1. **Workspace move git history**: `git mv -k src/ crates/dig3-core/src/` — verify git history preserved. If not, accept; we can blame -M30 later.
2. **examples/ visibility**: e2e_smoke.rs uses `digdigdig3::...` paths — after rename to `digdigdig3-core`, all `use digdigdig3::` becomes `use digdigdig3_core::`. ~50+ callsites in examples; sed-replace then verify.
3. **External `Cargo.toml` references**: if other workspace consumers (mlc/mli/mlt) pin `digdigdig3 = "0.2.x"` — they won't break since we're not publishing crates.io changes. But local path dependencies will break: `dependencies.digdigdig3 = { path = "../digdigdig3" }` must become `path = "../digdigdig3/crates/dig3-core"`. **Need to check**: search nemo workspace for path deps. Probably none since MLC is pinned independently.
4. **doc-tests**: `cargo test --doc` may fail if any rustdoc examples use `digdigdig3::...` paths. Update or `#[doc(hidden)]`.
5. **Cargo.lock**: regenerate. Probably big diff. OK.

## What's explicitly NOT in Phase 1 — backlog for Phase 2+

| Item | Now in | Target phase |
|------|--------|-------|
| WS multiplexing — one connection per `StreamKey`, N consumers share | `station::Station` (one mux per (exchange, account) but no consumer sharing) | Phase 2 |
| Deferred-unsub grace (drop = sleep N s, then unsub if no resubscribe) | not present (drop = immediate close on next iter) | Phase 2 |
| Reconnect override (custom backoff per Station, overrides transport default) | not present | Phase 3 |
| Warm-start (`LastN` from disk into consumer's receiver before live stream) | not present | Phase 5 |
| Gap-heal (REST backfill on reconnect to cover missed trades) | not present | Phase 3 |
| Orderbook tracker for `Stream::Orderbook` | `station::orderbook::OrderBookTracker` exists; `Station.subscribe` doesn't wire it yet | Phase 4 |
| Orderbook snapshot persistence (`.json.gz` per snapshot) | not present | Phase 4 |
| Three-level bar loader (memory ↔ disk ↔ REST) | not present | Phase 5 |
| Telemetry (counters per stream / dropped events / WS uptime) | basic `tracing` lines only | Phase 6 |
| Retention / cleanup (cron-like rotation of old days) | `station::storage::Retention` exists as types; Station never invokes it | Phase 2 |
| Prometheus exporter | not present (feature flag declared) | Phase 6 |
| `dig3 replay` / `dig3 inspect` / `dig3 capture` / `dig3 benchmark` | skeletons only | Phase 3-6 |
| `dig3 matrix` subcommand (port of `examples/e2e_smoke`) | skeleton; existing harness still works as `cargo run --example e2e_smoke` | Phase 3 |
| `dig3 persist` daemon (TOML-driven multi-stream capture) | `dig3-catcher` bin exists in cli (older API) | Phase 2 |
| `dig3 cure` (integrity / dedup / gap fix) | `dig3-cure` bin exists in cli (older API) | Phase 2 |

## Order of operations summary

1. workspace split  →  cargo check workspace clean
2. station crate skeleton  →  cargo check station clean
3. SubscriptionSet + Handle types  →  cargo check station clean
4. TradeWriter persistence  →  unit test passes
5. RestCache  →  micro-bench passes
6. Station::builder wiring  →  integration test passes
7. dig3-cli `watch trades` command  →  e2e test passes
8. Verify everything + commit
