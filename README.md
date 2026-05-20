# digdigdig3

Multi-exchange connector toolkit for Rust. **47 exchanges** covered (crypto CEX + DEX +
forex + stocks + prediction + data providers), **18 TRUSTED** (all major CEX with full
futures coverage). Single unified async API.

**Version:** 0.3.3 · **Edition:** Rust 2021 · **License:** MIT OR Apache-2.0
**Repository:** https://github.com/ZENG3LD/digdigdig3

## Workspace layout

Three crates, all on the same version pin (uzor-style):

| Crate | Purpose |
|---|---|
| **`digdigdig3`** | Pure connector library. `ExchangeHub` + REST/WS connectors + symbol normalization + capabilities. No persistence, no caching. |
| **`digdigdig3-station`** | High-level builder over `ExchangeHub`. Persistence (binary append + sparse idx), in-memory ring, REST cache, warm-start, **auto-heal on WS disconnect** (kline only), multiplex (N consumers share one WS), orderbook tracker, replay, cure. |
| **`digdigdig3-cli`** | `dig3` binary — `watch trades/orderbook/kline/...` + `dig3-inspect` post-mortem analyzer. |

## Quick start — connector library

```rust
use digdigdig3::{ExchangeHub, ExchangeId, AccountType, Symbol, sym};

let hub = ExchangeHub::new();
hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;
let conn = hub.rest(ExchangeId::Binance).unwrap();

// Three equivalent ways to pass a symbol:
let ticker = conn.get_ticker("BTCUSDT".into(), AccountType::Spot).await?;        // raw
let sym = Symbol::new("BTC", "USDT");
let ticker = conn.get_ticker((&sym).into(), AccountType::Spot).await?;            // canonical
let ticker = conn.get_ticker(sym!("BTCUSDT"), AccountType::Spot).await?;          // macro

println!("BTC = {}", ticker.last_price);
```

## Quick start — Station (high-level)

```rust
use digdigdig3_station::{Station, SubscriptionSet, Stream, ExchangeId, AccountType,
                         PersistenceConfig, GapHealConfig};

let station = Station::builder()
    .storage_root("./dig3_storage")
    .persistence(PersistenceConfig::on())   // binary append per kind
    .warm_start(300)                          // emit last-300 from disk before live
    .gap_heal(GapHealConfig::on())            // auto-heal kline on WS disconnect
    .build().await?;

let mut handle = station.subscribe(
    SubscriptionSet::new()
        .add(ExchangeId::Binance, "BTC-USDT", AccountType::Spot, [Stream::Kline("1m".into())])
).await?;

while let Some(event) = handle.recv().await {
    println!("{event:?}");
}
```

## Quick start — CLI

```bash
cargo install digdigdig3-cli       # binary: `dig3`

# Live trade tape, persisted to ./dig3_storage/trades/...
dig3 watch trades binance BTC-USDT --duration 30

# Live L2 ladder, top-10
dig3 watch orderbook binance BTC-USDT --depth 10

# Live klines + auto-heal on WS disconnect (REST get_klines refresh)
dig3 --gap-heal true watch kline binance BTC-USDT --interval 1m --duration 60

# Warm-start: emit last 300 bars from disk before live stream
dig3 --warm-start 300 watch kline binance BTC-USDT --interval 1m

# Post-mortem analysis of a persisted .dat
dig3-inspect kline ./dig3_storage/klines_1m/binance/spot/btcusdt/2026-05-20.dat
```

Global flags: `--storage-root <path>` (or `DIG3_STORAGE_ROOT`), `--warm-start N`, `--persist BOOL`, `--gap-heal BOOL`.

## TRUSTED 18 (all major crypto CEX + 4 DEX)

Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit, Coinbase, CryptoCom, Deribit, Dydx,
GateIO, HTX, HyperLiquid, Kraken, KuCoin, Lighter, MEXC, OKX.

Full futures coverage (mark/funding/OI/liquidation/aggTrade), bid/ask via primary
channel or parallel REST orderbook fetch, WS reconnect + replay, dedicated multi-symbol
liquidation capture.

Outside TRUSTED — wire-not-present (documented, won't be patched):
- CryptoCompare: CCCAGG free tier doesn't expose BID/ASK.
- MOEX: RU IP required for FAST/CEDR streams.
- Polymarket: ClobWebSocket not yet implemented.
- Dukascopy: tick-data-only, no public live REST.
- Auth-gated venues (Alpaca/Tinkoff/Polygon/IB/...): skip without ENV creds.

## Architecture

### Hub-first API

`ExchangeHub` is the sole public entry point for multi-connector operations.
Pool internals are `pub(crate)` — consumers cannot bypass.

| Method | Purpose |
|---|---|
| `connect_full(id, accounts, testnet)` | REST + WS for an exchange |
| `connect_full_validated(...)` | Same but rejects exchanges without ValidationStamp |
| `connect_public(id, testnet)` | REST only (warm-start, gap-heal) |
| `connect_websocket(id, account, testnet)` | WS only |
| `rest(id) -> Option<Arc<dyn CoreConnector>>` | Typed REST dispatch |
| `ws(id, account) -> Option<Arc<dyn WebSocketConnector>>` | WS dispatch |
| `capabilities(id) -> Option<ConnectorCapabilities>` | Discover what an exchange supports |
| `shutdown(id)` | Releases REST + WS |

### SymbolInput — raw or canonical, per call

Every per-symbol method accepts `SymbolInput<'_>`. Raw → zero-allocation passthrough.
Canonical `Symbol{base, quote}` → normalized inside via `SymbolNormalizer::to_exchange`
(22 per-exchange sub-modules: Binance `"BTCUSDT"`, OKX `"BTC-USDT"`, Gate.io `"BTC_USDT"`,
Bitfinex `"tBTCUSD"`, Deribit `"BTC-PERPETUAL"`, etc.).

### WebSocket: `UniversalWsTransport`

All connectors share `UniversalWsTransport<P: WsProtocol>` — connect/reconnect/backoff,
ping scheduler, subscription replay, frame dispatch with required `tracing::warn!` on
unmatched topics (no silent drops).

Each exchange implements thin `WsProtocol` (~400-900 LOC) vs the old bespoke loops
(800-1500 LOC each).

### Station: auto-heal on WS disconnect

`digdigdig3-station` adds high-level concerns over the raw hub:

- **Persistence** — binary append-only `.dat` + sparse `.idx`, fixed record size per
  data class (trades 48 B, bars 64 B, ticker 72 B, OB snapshot 808 B, ...). UTC day
  rotation. Layout: `<storage_root>/<kind>/<exchange>/<account>/<symbol>/<YYYY-MM-DD>.dat`.
- **Warm-start** — emit last-N from disk (or REST if disk empty) before live stream.
- **Auto-heal on WS disconnect** — three triggers (silence timeout, stream end, stream err),
  each runs full cycle: REST `get_klines` → `upsert_by_ts` (last-write-wins overwrite of
  broken in-flight bars) → `unsubscribe` + `subscribe` → re-attach broadcast receiver.
  Mirrors `mylittlechart::live_data::ws_manager` pattern.
- **Multiplex** — N consumers share one underlying WS subscription per `SeriesKey`.
- **Orderbook tracker** + replay + cure (dedup/gap/integrity) modules.

### Capabilities = empirical truth

`HasCapabilities::validation_status() -> Option<ValidationStamp>` exposes per-method
validation from the `e2e_smoke` harness, embedded as `data/validation_snapshot.json`.

## Feature flags (digdigdig3 core)

| Feature | Default | Purpose |
|---|---|---|
| `onchain-evm` | yes | k256 + sha3 for HyperLiquid EIP-712 signing |
| `onchain-cosmos` | no | cosmrs for dYdX |
| `onchain-starknet` | no | starknet-crypto for Lighter |
| `grpc` | no | tonic transport (Tinkoff) |
| `websocket` | yes | WS enablement |

Removing `onchain-evm` cuts ~52 transitive deps (only needed for HyperLiquid private trading).

## Architecture invariants

- **No `_ => Ok(None)` catch-alls** in WS dispatch. Unmatched topic → `tracing::warn!`.
- **No `std::sync::Mutex` across `.await`** — tokio sync only.
- **Symbol normalization lives outside connectors.** Connectors take raw exchange-native strings.
- **Capabilities derived from `topic_registry`**, not free-form flags — cannot drift from reality.
- **`UnsupportedOperation` vs `NotSupported`** are distinct: first = TODO, second = wire-not-present.

## Validation gate

```bash
RUSTFLAGS="-D warnings" cargo check --workspace --all-targets --all-features
cargo test -p digdigdig3 --lib --all-features          # 818 pass (1 pre-existing dYdX fail)
cargo test -p digdigdig3-station --tests                # 75 pass
cargo run --example e2e_smoke --release -p digdigdig3   # live API matrix
```

## Documentation

- `CLAUDE.md` — architectural principles, scope, test plan
- `examples/exchange_hub_demo.rs` — minimal hub usage
- `examples/e2e_smoke.rs` — full 47-exchange validation matrix

## License

MIT OR Apache-2.0
