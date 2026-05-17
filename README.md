# digdigdig3

Multi-exchange connector library — unified async Rust API for 22 production crypto exchanges
+ stocks/forex/prediction (validated subset). Single `ExchangeHub` async pool exposing
all connectors with self-declared, empirically-validated capabilities.

**Version:** 0.2.1
**Edition:** Rust 2021
**License:** MIT OR Apache-2.0
**Repository:** https://github.com/ZENG3LD/digdigdig3

## Quick start

```rust
use digdigdig3::{ExchangeHub, ExchangeId, AccountType, Symbol, SymbolNormalizer};

let hub = ExchangeHub::new();
hub.connect_full(ExchangeId::Binance, &[AccountType::Spot], false).await?;

let conn = hub.rest(ExchangeId::Binance).unwrap();
let raw = SymbolNormalizer::to_exchange(ExchangeId::Binance, &Symbol::new("BTC", "USDT"), AccountType::Spot)?;
let ticker = conn.get_ticker(&raw, AccountType::Spot).await?;
println!("BTC = {}", ticker.last_price);
```

## Architecture

### Hub-first API

`ExchangeHub` is THE entry point. Pool internals are `pub(crate)` — consumers cannot bypass.

| Method | Purpose |
|---|---|
| `connect_full(id, accounts, testnet)` | Wires REST + WS for an exchange |
| `connect_full_validated(...)` | Same but rejects exchanges without validation stamp |
| `rest(id) -> Option<Arc<dyn CoreConnector>>` | Typed REST dispatch |
| `ws(id, account) -> Option<Arc<dyn WebSocketConnector>>` | WS dispatch |
| `capabilities(id) -> Option<ConnectorCapabilities>` | Discover what an exchange supports |
| `list_connected() -> Vec<ExchangeId>` | Enumerate active connections |
| `shutdown(id)` | Releases REST + WS |

### Raw symbols inside connectors

Connectors take exchange-native strings (`"BTCUSDT"` for Binance, `"BTC-USDT"` for OKX,
`"BTC_USDT"` for Gate.io, `"tBTCUSD"` for Bitfinex, `"BTC-PERPETUAL"` for Deribit, etc).

Translation is a separate utility:

```rust
let raw = SymbolNormalizer::to_exchange(id, &canonical, account_type)?;
let parsed = SymbolNormalizer::from_exchange(id, &raw, account_type)?;
```

Per-exchange rules in `core::utils::symbol_normalizer` (22 exchanges).

### WebSocket: UniversalWsTransport

All 9 migrated WS connectors share `UniversalWsTransport<P: WsProtocol>` framework owning:
- connect/reconnect/backoff
- ping scheduler + subscription replay
- frame dispatch (no `_ => Ok(None)` catch-alls — unmatched topics warn)
- tracing on every frame

Each exchange implements thin `WsProtocol` (~6 methods, ~400-900 LOC) vs the old bespoke loops
(800-1500 LOC each).

### Capabilities = empirical truth

`HasCapabilities::validation_status() -> Option<ValidationStamp>` exposes per-method/stream
validation from the `deep_smoke` harness, embedded as `data/validation_snapshot.json`.

```rust
let stamp = hub.capabilities(ExchangeId::Binance).and_then(|c| c.validation);
match stamp.as_ref().and_then(|s| s.rest.get("get_ticker")) {
    Some(FieldValidation::Validated { fields_populated }) => println!("validated: {fields_populated:?}"),
    Some(FieldValidation::PopulatedButEmpty { missing_fields }) => println!("partial — missing: {missing_fields:?}"),
    Some(FieldValidation::Failed { reason }) => println!("broken: {reason}"),
    _ => println!("not tested"),
}
```

## Validated coverage (deep_smoke 2026-05-17)

22 in-scope exchanges (no API keys needed for public data):

| Group | Exchanges | REST OK | WS Connected | Real Data Flowing |
|---|---|---|---|---|
| L3-open CEX (18) | Binance, Bybit, OKX, KuCoin, Kraken, Coinbase, Gate.io, Gemini, MEXC, HTX, Bitget, BingX, Crypto.com, Upbit, Bitfinex, Bitstamp, Deribit, HyperLiquid | 15/18 | 17/18 | 13/17 |
| L3-open DEX (2) | dYdX, Lighter | 1/2 | 2/2 | 2/2 |
| L3-open Pred (1) | Polymarket | 0/1 | 0/1 | — |
| L2-free (1) | MOEX | 1/1 | 0/1 | — |

L1/L2-paid + L3-gated (21 exchanges) compile-validated only; functional validation deferred
until API keys available.

## Levels

| Level | Description | Auth |
|---|---|---|
| L1 | OHLCV/ticker only — no orderbook | Often API key |
| L2 | L1 + orderbook | Often API key |
| L3 open | Full stack, public data | None for data; key for trading |
| L3 gated | Full stack, all data | Required |

43 connectors total across all levels.

## Documentation

- `CLAUDE.md` — architectural principles + test plan + scope (full detail)
- `docs/plans/wave0-foundation.md` — UniversalWsTransport design
- `docs/plans/phase-alpha-symbol-decoupling.md` — Symbol decoupling design
- `docs/plans/smoke_v8_findings_spec.md` — original consumer feedback spec
- `examples/exchange_hub_demo.rs` — minimal hub usage
- `examples/deep_smoke.rs` — full validation harness
- `examples/full_smoke.rs` — parallel 48-exchange smoke

## Validation harness

```bash
cargo build --example deep_smoke --release
./target/release/examples/deep_smoke.exe   # Windows: deep_smoke.exe
```

Generates `deep_smoke_post_zeta.txt` and a regenerated `data/validation_snapshot.json`.

## Feature flags

| Feature | Default | Purpose |
|---|---|---|
| `onchain-evm` | yes | EVM provider (HyperLiquid signing) |
| `onchain-cosmos` | no | Cosmos provider (dYdX) |
| `onchain-starknet` | no | StarkNet provider (Lighter) |
| `grpc` | no | tonic transport (Tinkoff) |
| `websocket` | yes | WS enablement |

## Architecture invariants

- **No `_ => Ok(None)` catch-alls** in WS dispatch. Unmatched topic → `tracing::warn!`.
- **No `std::sync::Mutex` across `.await`**. Tokio sync only.
- **Symbol normalization lives outside connectors**. Connectors take raw exchange-native strings.
- **Capabilities are derived, not declared.** Topic registry determines stream support; ValidationStamp records observed reality.

## Levels of test coverage

| Layer | What | Command |
|---|---|---|
| Compile gate | 0 errors, 0 warnings | `RUSTFLAGS="-D warnings" cargo check --all-targets --all-features` |
| Unit tests | Fixture-based parser tests | `cargo test --lib --all-features` |
| Live validation | Real API + content inspection | `./target/release/examples/deep_smoke.exe` |

## Known limitations

- L3-gated connectors (Alpaca, Zerodha, OANDA, IB, ...) compile but are not functionally validated — requires API keys.
- Some L3-open methods limited by exchange API access: HyperLiquid `get_ticker` needs asset_index → coin mapping (deferred).
- Symbol normalizer assumes canonical `(base, quote)` model. Options (Deribit) require concrete instrument_name via `Symbol::with_raw`.
- HTX server-pong reply hook not in framework yet — auto-reconnect compensates.
- MOEX FAST WS may need RU ISP routing for events.

## Author / repository

- Repo: https://github.com/ZENG3LD/digdigdig3
- License: MIT OR Apache-2.0
