# dig3 Station — Architecture Design

**Status**: design / pending implementation
**Date**: 2026-05-20
**Predecessor work**: Waves 1-10 (raw ExchangeHub validated against 18 TRUSTED crypto exchanges)
**MLC reference**: explored in `mylittlechart/` (research summary in commit history)

## 1. Goal

Build a consumer-facing high-level layer on top of the existing `ExchangeHub`, exposed via a fluent builder, with opt-in features:
- Persistence (bars + trades binary, snapshots json.gz)
- REST/cache layer (LRU + TTL)
- Warm-start (cache replay before live feed catches up)
- Multiplexed subscriptions with deferred-unsub and RAII handles
- Reconnect + gap-heal via REST backfill
- Orderbook tracker (L2 reconstruction from snapshot+delta)
- Three-level bar loader (instant disk + Phase A fresh + Phase B gap heal)
- Telemetry (`metrics` + `tracing`, optional Prometheus endpoint)
- Declarative `SubscriptionSet` across exchanges → one merged stream

MLC is the inspiration but not a migration target — it stays pinned on 0.1.32. We are free to break API.

## 2. Workspace layout

Single workspace with three crates:

```
digdigdig3/
├── Cargo.toml                                   # [workspace] members = [core, station, cli]
├── crates/
│   ├── dig3-core/                              # existing code, renamed
│   │   ├── Cargo.toml                          # name = "digdigdig3-core"
│   │   └── src/
│   │       ├── connector_manager/              # ExchangeHub, factory, pool, ws_pool, registry
│   │       ├── core/                           # types, traits, websocket transport, utils, storage primitives
│   │       └── l1/ l2/ l3/                     # connectors (47 exchanges)
│   ├── dig3-station/                           # NEW: consumer-facing layer
│   │   ├── Cargo.toml                          # name = "digdigdig3-station"
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── builder.rs                      # StationBuilder
│   │       ├── station.rs                      # Station, the runtime object
│   │       ├── subscription/
│   │       │   ├── mod.rs
│   │       │   ├── set.rs                      # SubscriptionSet declarative API
│   │       │   ├── handle.rs                   # SubscriptionHandle (RAII)
│   │       │   ├── multiplexer.rs              # per-(exchange,stream,account) actor
│   │       │   └── refcount.rs                 # atomic ref + deferred unsub
│   │       ├── persistence/
│   │       │   ├── mod.rs
│   │       │   ├── bars.rs                     # binary fixed-size 48 bytes/bar
│   │       │   ├── trades.rs                   # append log + index
│   │       │   ├── snapshots.rs                # json.gz periodic OB dumps
│   │       │   ├── retention.rs                # days + size
│   │       │   └── replay.rs                   # read_range() helper
│   │       ├── cache/
│   │       │   ├── mod.rs
│   │       │   ├── shared_map.rs               # SharedMap<StreamKey, Arc<RwLock<Series>>>
│   │       │   ├── rest_lru.rs                 # REST ticker/orderbook TTL LRU
│   │       │   └── warm_start.rs               # boot-time replay from disk
│   │       ├── reconnect/
│   │       │   ├── mod.rs
│   │       │   ├── policy.rs                   # exp/none/custom
│   │       │   └── gap_heal.rs                 # REST backfill on Reconnected event
│   │       ├── orderbook_tracker/
│   │       │   ├── mod.rs
│   │       │   └── ladder.rs                   # snapshot+delta → ordered tree → top-N
│   │       ├── bar_loader/
│   │       │   ├── mod.rs
│   │       │   ├── ring.rs                     # in-memory ring (10k bars default)
│   │       │   └── three_level.rs              # disk → phase A → phase B
│   │       ├── telemetry/
│   │       │   ├── mod.rs
│   │       │   ├── metrics.rs                  # `metrics` crate counters
│   │       │   └── prometheus.rs               # optional, feature-gated
│   │       └── error.rs
│   └── dig3-cli/                               # NEW: bin
│       ├── Cargo.toml                          # name = "digdigdig3-cli", bin "dig3"
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── watch.rs                    # dig3 watch trades binance btc-usdt
│           │   ├── persist.rs                  # dig3 persist start/stop/status
│           │   ├── replay.rs                   # dig3 replay --from --to --out
│           │   ├── matrix.rs                   # dig3 matrix run (== e2e_smoke)
│           │   ├── inspect.rs                  # dig3 inspect symbols binance
│           │   ├── capture.rs                  # dig3 capture liq --duration 3600
│           │   └── benchmark.rs                # dig3 benchmark cache --duration 60
│           └── config.rs                       # TOML/CLI parsing
└── examples/                                   # legacy examples stay; new ones under each crate
```

### Crate dependency direction

```
dig3-cli  ─→  dig3-station  ─→  dig3-core
                                  ↑
                              examples/e2e_smoke (existing) still uses core directly
```

dig3-core has **no** dependency on station. dig3-station has **no** dependency on cli.

### Feature flags

`dig3-station/Cargo.toml`:
```toml
[features]
default       = ["persistence", "cache", "multiplex", "reconnect"]
full          = ["default", "orderbook-tracker", "bar-loader", "metrics", "prometheus"]
minimal       = []

persistence       = ["dep:zstd", "dep:bincode", "dep:crc32fast"]
cache             = []
multiplex         = []
reconnect         = []
orderbook-tracker = []
bar-loader        = []
metrics           = ["dep:metrics"]
prometheus        = ["metrics", "dep:metrics-exporter-prometheus"]
```

A consumer that only wants persistence + multiplex:
```toml
digdigdig3-station = { version = "0.3", default-features = false, features = ["persistence", "multiplex"] }
```

## 3. Public API surface (dig3-station)

### 3.1 Top-level builder

```rust
use digdigdig3_station::{Station, Persistence, Cache, Reconnect, Telemetry, OrderbookTracker, BarLoader};
use digdigdig3_core::core::types::{ExchangeId, AccountType};

let station = Station::builder()
    // — connectivity (passes through to ExchangeHub) —
    .with_exchanges([Binance, Bybit, OKX])              // pre-register
    .testnet(false)
    .connect(ConnectMode::Lazy)                          // Lazy | Eager | OnResubscribe

    // — feature toggles, each opt-in —
    .persistence(Persistence::on()                       // .off() = no disk
        .root("./dig3_storage/")
        .bars(true)
        .trades(true)
        .snapshots(true)                                 // OB periodic json.gz
        .orderbook_deltas(false)                         // opt-in only, very heavy
        .retention(Retention::days(30).max_size_mb(2048))
        .flush_interval(Duration::from_secs(5)))

    .cache(Cache::on()                                   // .off() = pass-through
        .shared_map_capacity(10_000)                     // bars/trades per series
        .rest_ttl_ticker(Duration::from_secs(1))
        .rest_ttl_orderbook(Duration::from_millis(500))
        .rest_ttl_symbol_metadata(Duration::from_secs(3600))
        .rest_lru_max_entries(1024)
        .warm_start(WarmStart::LastN(300)))              // .None = no replay

    .reconnect(Reconnect::on()
        .min_backoff(Duration::from_secs(1))
        .max_backoff(Duration::from_secs(30))
        .ws_silence_timeout(Duration::from_secs(60))
        .gap_heal(GapHeal::OnReconnect))                 // .None | .OnReconnect | .Custom

    .multiplex(Multiplex::on()                           // .off() = 1 conn per sub
        .deferred_unsub(Duration::from_secs(30))
        .control_channel(ControlChannel::Unbounded))

    .orderbook_tracker(OrderbookTracker::on()            // feature-gated
        .max_depth(50)
        .seed_from_rest(true)                            // sequenced REST→WS bootstrap
        .checksum_validation(true))

    .bar_loader(BarLoader::on()                          // feature-gated
        .ring_capacity(10_000)
        .phase_a_fresh(300)
        .phase_b_max_pages(20)
        .phase_b_page_size(500))

    .telemetry(Telemetry::on()
        .tracing_target("dig3")
        .metrics(MetricsBackend::Default))               // None | Default | Prometheus(addr)

    // — backpressure (single knob) —
    .broadcast_capacity(65_536)
    .on_lag(LagPolicy::DropOldest)                       // tokio default; not configurable for MVP

    .build().await?;
```

### 3.2 Subscription — declarative SubscriptionSet

```rust
use digdigdig3_station::SubscriptionSet;

// Multi-exchange, multi-symbol, multi-stream in one declaration
let set = SubscriptionSet::new()
    .add(Binance, "BTC-USDT", AccountType::FuturesCross,
         [Stream::Ticker, Stream::Trade, Stream::Orderbook, Stream::MarkPrice])
    .add(Binance, "ETH-USDT", AccountType::FuturesCross,
         [Stream::Trade, Stream::Orderbook])
    .add(Bybit,   "BTC-USDT", AccountType::FuturesCross,
         [Stream::Trade, Stream::Liquidation])
    .with_warm_start_override(Some(WarmStart::LastN(100)));   // optional override of builder default

// Returns RAII handle.  All actors spawn, REST seeds run, then WS subs go through.
let handle = station.subscribe(set).await?;

// One merged stream of Events from all (exchange, symbol, stream) tuples.
while let Some(event) = handle.recv().await {
    match event {
        Event::Trade { exchange, symbol, price, qty, side, ts_ms, .. } => { ... }
        Event::Orderbook { exchange, symbol, bids, asks, ts_ms, .. } => { ... }
        Event::Reconnected { exchange, downtime_ms } => { ... }
        Event::GapHealed { exchange, symbol, from_ts, to_ts, n_events } => { ... }
        Event::Error { exchange, kind, msg } => { ... }
        _ => {}
    }
}

drop(handle);   // → atomic refcount decrement → deferred unsub after grace
```

Subscription handle traits:
```rust
impl SubscriptionHandle {
    pub async fn recv(&mut self) -> Option<Event>;
    pub fn try_recv(&mut self) -> Result<Event, TryRecvError>;
    pub fn into_stream(self) -> impl Stream<Item = Event>;
    pub fn into_receiver(self) -> broadcast::Receiver<Event>;

    /// Read-only view of current cache state (no refcount increment).
    pub fn snapshot(&self) -> &SharedMap;

    /// Active subscriptions in this handle (for debugging).
    pub fn active(&self) -> &[ActiveSub];

    /// Add a stream to an existing subscription without dropping the handle.
    pub async fn add(&mut self, exchange: ExchangeId, symbol: &str, ...) -> Result<()>;
    pub async fn remove(&mut self, exchange: ExchangeId, symbol: &str, ...) -> Result<()>;
}
```

### 3.3 Direct cache access (read-only tap)

```rust
let tap = station.tap();             // free, no refcount

// Read current state of any cached series (returns None if not subscribed)
let bars = tap.bars(Binance, "BTC-USDT", AccountType::Spot, "1m").await;
let last_trade = tap.last_trade(Binance, "BTC-USDT").await;
let book = tap.orderbook(Binance, "BTC-USDT", AccountType::FuturesCross).await;
```

### 3.4 Persistence replay (offline mode)

```rust
let replay = station.replay()
    .from_ms(1_000_000_000_000)
    .to_ms(2_000_000_000_000)
    .filter(Binance, "BTC-USDT", AccountType::Spot, [Stream::Trade])
    .open().await?;

// Same Event shape, but historical
while let Some(event) = replay.next().await? { ... }
```

## 4. Internal structure

### 4.1 SharedMap pattern (borrowed from MLC, hardened)

```rust
pub(crate) struct SharedMap {
    // Per-series in-memory state, behind Arc for cheap cloning across panels/tasks
    bars: Arc<RwLock<HashMap<BarKey, Arc<RwLock<BarSeries>>>>>,
    trades: Arc<RwLock<HashMap<TradeKey, Arc<RwLock<TradeSeries>>>>>,
    orderbooks: Arc<RwLock<HashMap<OrderbookKey, Arc<RwLock<OrderbookSeries>>>>>,
    tickers: Arc<RwLock<HashMap<TickerKey, Arc<RwLock<TickerSeries>>>>>,
}
```

Two paths into a series:
1. **Write path** (single producer per key): the multiplexer actor for that StreamKey owns mut access. Receives upstream Events from ExchangeHub WS, writes into the series, broadcasts "changed" signal.
2. **Read path** (many consumers): `Arc<RwLock<Series>>` clone. Consumer holds it and reads at their own pace. No copy through channels — channel only carries "ts X changed".

### 4.2 Multiplexer (one actor per StreamKey, fix MLC anti-pattern)

```rust
pub(crate) struct StreamKey {
    exchange: ExchangeId,
    stream_kind: StreamKind,
    account_type: AccountType,
}

pub(crate) struct MultiplexerActor {
    key: StreamKey,
    hub: Arc<ExchangeHub>,                                 // dig3-core
    ref_table: Arc<DashMap<Symbol, u32>>,                  // sym → consumer count
    deferred_unsub: Arc<DashMap<Symbol, Instant>>,         // sym → unsub deadline
    cmd_rx: mpsc::UnboundedReceiver<MuxCmd>,               // UNBOUNDED for control (fix MLC)
    upstream_rx: broadcast::Receiver<StreamEvent>,         // from hub.ws()
    fan_out: broadcast::Sender<Event>,                     // to consumers
}

enum MuxCmd {
    Subscribe(Symbol, oneshot::Sender<Result<()>>),
    Unsubscribe(Symbol),
    Shutdown,
}
```

### 4.3 SubscriptionHandle — atomic RAII

```rust
pub struct SubscriptionHandle {
    inner: Arc<HandleInner>,
    rx: broadcast::Receiver<Event>,
}

struct HandleInner {
    set: SubscriptionSet,
    multiplexers: Vec<(StreamKey, mpsc::UnboundedSender<MuxCmd>)>,
}

impl Drop for HandleInner {
    fn drop(&mut self) {
        for (key, tx) in &self.multiplexers {
            for sym in self.set.symbols_for(*key) {
                let _ = tx.send(MuxCmd::Unsubscribe(sym));
            }
        }
    }
}
```

One Arc, one Drop, no drift between data-refcount and ws-refcount.

### 4.4 Persistence — bytes on disk

Layout:
```
dig3_storage/
├── bars/
│   └── binance/spot/btcusdt/1m.bin             # fixed binary, append, 48 bytes/bar
├── trades/
│   └── binance/spot/btcusdt/
│       ├── 2026-05-20.dat                      # date-rotated trade log
│       └── 2026-05-20.idx                      # ts→offset for random seek
├── snapshots/
│   └── binance/futures/btcusdt/
│       └── 2026-05-20T15-32-00.ob.json.gz      # one per N seconds (configurable)
├── meta/
│   ├── binance/symbols.json                    # symbol metadata cache
│   └── _retention.lock                         # for cleanup coordination
└── _index.db                                   # sled key-value: stream → last_ts, file inventory
```

Format choices:
- **Bars**: `#[repr(C)] BarRecord { ts: i64, o,h,l,c: f64, v,trades: u32 }` = 48 bytes. File header: `[u32 MAGIC, u32 VERSION, u32 BAR_COUNT, u32 CRC32]`. Atomic write via tmpfile + rename.
- **Trades**: append-only binary `[u64 ts_ns, f64 price, f64 qty, u8 side, u64 trade_id]` = 41 bytes per record. Sidecar `.idx` writes `[u64 ts_ms, u64 offset]` every 1024 records (sparse index for fast `read_range`).
- **Snapshots**: full JSON serialization of `OrderBook` snapshot, gzipped. Replayable via standard parser. Period configurable (default 10s).
- **Index** (`_index.db`): sled. Keys: `b"stream/{exchange}/{symbol}/{type}"`, values: `bincode(StreamMeta { last_ts, file_inventory })`. Used by retention sweeper and replay seek.

Write path is non-blocking:
```
multiplexer → tokio::sync::mpsc::UnboundedSender → persistence actor → spawn_blocking(io)
```

### 4.5 Cache — REST LRU with TTL

```rust
pub(crate) struct RestCache {
    inner: Arc<Mutex<lru::LruCache<RestKey, CachedEntry>>>,
    ttl_by_kind: HashMap<RestKind, Duration>,
}

struct CachedEntry { value: Value, expires_at: Instant }

impl RestCache {
    pub async fn get_or_fetch<F, Fut, T>(
        &self,
        key: RestKey,
        kind: RestKind,
        fetcher: F,
    ) -> Result<T>
    where F: FnOnce() -> Fut, Fut: Future<Output = Result<T>>;
}
```

Wrap each `hub.rest(id).get_ticker(...)` call site in `cache.get_or_fetch`. Cache hit returns instant; miss → fetcher → store.

### 4.6 Warm-start

On `station.subscribe(set)`:
1. For each (exchange, symbol, stream) in set:
   1. If persistence on and `warm_start = LastN(n)`: read last n records from disk → populate SharedMap[key].
   2. Send `Event::WarmStartLoaded { key, count }` to handle's rx.
2. Then spawn/wire multiplexer (which connects WS, processes events).
3. Consumer sees historical events first, then live.

`Event::WarmStartLoaded` lets consumer know when transition happens.

### 4.7 Gap-heal

When multiplexer's WS reconnects (after disconnect), it knows `last_ts` per symbol (tracked in MultiplexerActor state). After reconnect, before unsubscribing/reconnecting, fire:
- REST `get_recent_trades(symbol, from=last_ts)` for Trade stream
- REST `get_klines(symbol, interval, from=last_ts)` for Bar stream
- Orderbook: snapshot+resync (depth re-bootstrap, no delta replay)

Backfilled events go into SharedMap and emit through `fan_out` with `was_backfilled = true` flag.

### 4.8 Three-level bar loader

Phase 0 (instant): `BarLoader::open(key)` reads disk → returns historical bars from file → consumer renders immediately.

Phase A (300 fresh): one REST `get_klines(limit=300, ending=now)` → merge with disk → emit `Event::BarsLoaded { phase: A }`.

Phase B (gap heal): if disk had bars but the most recent disk bar is older than `2 * interval`, paginate backward up to 20 × 500 = 10000 bars from disk-end to phase-A-start. Emit `Event::BarsLoaded { phase: B }`.

WS subscribes after Phase A starts (continuous live updates).

### 4.9 Orderbook tracker

```rust
pub(crate) struct OrderbookLadder {
    bids: BTreeMap<NotNanF64, f64>,                 // price → size, sorted desc via reverse iter
    asks: BTreeMap<NotNanF64, f64>,                 // sorted asc
    last_update_id: u64,
}

impl OrderbookLadder {
    pub fn apply_snapshot(&mut self, snap: OrderbookSnapshot);
    pub fn apply_delta(&mut self, delta: OrderbookDelta) -> Result<(), GapDetected>;
    pub fn top_n(&self, n: usize) -> (Vec<Level>, Vec<Level>);
    pub fn mid(&self) -> Option<f64>;
}
```

On gap detected → request fresh REST snapshot (via gap-heal path).

## 5. CLI surface (dig3-cli)

```
dig3 watch <stream> <exchange> <symbol> [--account spot|cross|isolated] [--duration N]
   ↳ Subscribes to a stream and prints events to stdout. Live.
   ↳ Examples:
       dig3 watch trades binance btc-usdt
       dig3 watch orderbook bybit eth-usdt --account cross --duration 60
       dig3 watch liquidation binance --all-symbols

dig3 persist start [--config path/to/dig3.toml]
   ↳ Boots Station with config from TOML, persists subscribed streams, runs forever.
dig3 persist status
   ↳ Shows storage size, last-write per stream, retention state.
dig3 persist cleanup [--dry-run]
   ↳ Applies retention manually.

dig3 replay --from <iso8601> --to <iso8601> --filter <exchange>:<symbol>:<stream> [--out json|csv]
   ↳ Reads from disk, emits events.

dig3 matrix [--all] [--exchange X] [--json-out path]
   ↳ Runs the e2e_smoke equivalent — port of examples/e2e_smoke.rs as a CLI cmd.

dig3 inspect symbols <exchange>
   ↳ Lists known symbols for an exchange (uses meta/ cache if available).
dig3 inspect capabilities <exchange>
   ↳ Prints HasCapabilities::capabilities() — what's declared supported.

dig3 capture liq --exchanges X,Y --symbols A,B --duration N
   ↳ Standalone liquidation capture (port of liq_capture.rs).

dig3 benchmark cache --duration 60
   ↳ Reports REST cache hit rate / latency.
```

Config file (`dig3.toml`):
```toml
[connectivity]
exchanges = ["Binance", "Bybit", "OKX"]
testnet = false
connect = "lazy"

[persistence]
enabled = true
root = "./dig3_storage"
bars = true
trades = true
snapshots = true
orderbook_deltas = false
retention_days = 30
retention_size_mb = 2048
flush_interval_secs = 5

[cache]
enabled = true
shared_map_capacity = 10000
rest_ttl_ticker_ms = 1000
rest_ttl_orderbook_ms = 500
rest_ttl_symbol_metadata_secs = 3600
warm_start = { kind = "last_n", n = 300 }

[reconnect]
enabled = true
min_backoff_secs = 1
max_backoff_secs = 30
ws_silence_timeout_secs = 60
gap_heal = "on_reconnect"

[multiplex]
enabled = true
deferred_unsub_secs = 30
control_channel = "unbounded"

[telemetry]
tracing_target = "dig3"
metrics = "default"
prometheus_addr = "127.0.0.1:9898"  # optional, requires `prometheus` feature

[subscriptions]
[[subscriptions.entry]]
exchange = "Binance"
symbol = "BTC-USDT"
account_type = "futures_cross"
streams = ["ticker", "trade", "orderbook", "mark_price"]

[[subscriptions.entry]]
exchange = "Bybit"
symbol = "ETH-USDT"
account_type = "futures_cross"
streams = ["trade", "liquidation"]
```

## 6. Implementation phases

### MVP (phase 1) — minimal useful station

Goal: subscribe with one builder call, persist trades+bars, basic RAII, REST cache.

- [ ] Workspace split: rename `digdigdig3` → `crates/dig3-core/`, set up workspace toml
- [ ] Create `crates/dig3-station/` with empty lib + Cargo.toml
- [ ] Create `crates/dig3-cli/` with `dig3 watch` only (binary skeleton)
- [ ] `Station::builder()` fluent API stub (all .feature() chainable, but most are no-ops)
- [ ] `SubscriptionSet` struct + `station.subscribe(set)` → `SubscriptionHandle`
- [ ] SharedMap with Trade + Bar series only (no OB yet)
- [ ] Multiplexer per (exchange, stream, account) for Trade + Kline streams
- [ ] RAII `SubscriptionHandle` Drop → Unsubscribe via unbounded mpsc
- [ ] Persistence: binary trades append-only + binary bars; no snapshots; no retention
- [ ] REST cache LRU with TTL for `get_ticker`
- [ ] `dig3 watch trades <exchange> <symbol>` works end-to-end via station

Compile gate: `cargo check --workspace --all-features` clean, 0 warnings.
Acceptance: `dig3 watch trades binance btc-usdt --duration 30` prints trades from cache+live, persists to `./dig3_storage/trades/...`.

### Phase 2 — reconnect, multiplex, deferred-unsub

- [ ] Reconnect policy plumbing (already exists in transport, expose through Station builder)
- [ ] Deferred-unsub queue + 30s grace
- [ ] Two consumers on same StreamKey share one upstream sub (multiplex actually multiplexes)
- [ ] Telemetry: `metrics` crate counters per (exchange, stream) — subscribe/unsubscribe/events_in/events_out/lag

Acceptance: 5 simultaneous `SubscriptionHandle`s on same (Binance, BTC-USDT, Trade) → 1 upstream WS sub. Last handle dropped → 30s grace → upstream unsub.

### Phase 3 — warm-start + gap-heal

- [ ] Warm-start LastN: read disk → SharedMap → emit historical events on subscribe
- [ ] Gap-heal: on reconnect, REST backfill from last_ts → emit with `was_backfilled = true`
- [ ] Three-level bar loader

Acceptance: `dig3 watch trades binance btc-usdt --warm-start 100` shows 100 historical events instantly, then transitions to live.

### Phase 4 — orderbook tracker

- [ ] OrderbookLadder L2 reconstruction
- [ ] Sequenced REST seed → WS subscribe (no delta gap)
- [ ] Checksum validation where available (OKX, Bitget, GateIO)
- [ ] Snapshots persistence (json.gz every N seconds)

Acceptance: `dig3 watch orderbook binance btc-usdt --depth 20` shows ordered top-20 levels with `mid()` price; survives reconnect; persists snapshots.

### Phase 5 — retention + cleanup

- [ ] Days-based retention
- [ ] Size-based retention (max_size_mb)
- [ ] Background cleanup task
- [ ] `dig3 persist cleanup --dry-run`

Acceptance: 7-day retention with 100MB cap properly evicts oldest files.

### Phase 6 — full CLI + Prometheus

- [ ] All `dig3` subcommands wired
- [ ] Prometheus endpoint via feature flag
- [ ] `dig3 inspect`, `dig3 capabilities`, `dig3 benchmark`, `dig3 replay`

Acceptance: CLI usable as primary entrypoint; Prometheus scrape works.

## 7. Open design questions (mark before implementation)

1. **Persistence atomicity on crash**: bars use tmp+rename. Trades use append. If process crashes mid-append, last record may be torn. Solution: CRC32 per-record OR fsync after each record (slow). Pick CRC.
2. **SharedMap memory pressure**: 10k bars × 5 timeframes × 50 symbols × 18 exchanges ≈ 45M records × 48 bytes ≈ 2 GB. Implement LRU eviction at series level — keep only N most-recently-accessed series in memory. The rest evicted but disk persisted; warm-load on next access.
3. **Subscription update semantics**: `handle.add(...)` triggers a fresh WS subscribe. Should `remove(...)` defer-unsub or immediate? Default: defer (consistent with builder default).
4. **Stream type explosion**: 9 WS streams × N exchanges. Each multiplexer handles ONE (exchange, stream, account). For 18 exchanges × 9 streams × 3 account types that's 486 potential actors. Lazy spawn — actor only exists if subscribed.
5. **Cancel-on-drop vs explicit unsubscribe**: Drop unsubscribes; consumer can also `handle.unsubscribe(...).await` for explicit reasoning about timing.

## 8. What this is NOT

- Not a replacement for MLC. MLC stays pinned, runs independently, won't migrate.
- Not a strategy engine. No order routing, no signal generation. Pure data plane.
- Not a trading order manager (yet). The hub already has Trading + Account traits; station phase 7+ may wrap them with similar builder pattern but out of scope for now.
- Not an aggregator. No cross-exchange consolidated feed. Each (exchange, symbol) is independent.

## 9. Why this is a sound architecture

1. **Layered**: dig3-core stays untouched; station is pure addition; cli is consumer of station. Each crate has a clear job.
2. **Opt-in everything**: features are flags, none mandatory. Embedded users (someone building their own UI) only pay for what they use.
3. **Borrows the strong MLC patterns** (SharedMap, deferred-unsub, sequenced REST→WS) and fixes the weak ones (try_send drop, dual refcount drift).
4. **Industry-standard tools where they exist**: `tracing`, `metrics`, `lru`, `sled`. No reinventing wheels.
5. **Declarative config**: TOML for CLI, builder for embedded. Same options, two surfaces.
6. **Composable**: SubscriptionSet declares everything, Station resolves wiring. No ad-hoc subscribe_X methods that drift.
7. **Time-aware**: every Event carries ts; gap-heal closes the inevitable reconnect window; warm-start gives instant data.
