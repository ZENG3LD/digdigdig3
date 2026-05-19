# dig3 — Testing Plan (post-Wave-10)

Status snapshot: `7d800a0` (2026-05-20). 10 debug waves complete, all 18 major
crypto exchanges TRUSTED. Next work is the Station layer (see
`docs/plans/station-architecture.md`).

## Current state

- **Unit tests**: 821 PASS, 1 FAIL (pre-existing dydx `test_signing_key_from_invalid_bytes`, unrelated to our work).
- **`cargo check --all-targets --all-features` with `RUSTFLAGS=-D warnings`**: clean.
- **Live `e2e_smoke` matrix**: TRUSTED 18 stable across consecutive runs.

## TRUSTED 18

Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit, Coinbase, CryptoCom, Deribit, Dydx, GateIO, HTX, HyperLiquid, Kraken, KuCoin, Lighter, MEXC, OKX.

Full coverage of:
- All futures channels (mark/funding/OI/liquidation/aggTrade) on every L3-open CEX that exposes them
- Bid/ask populated via primary WS channel OR parallel REST orderbook fetch
- WebSocket reconnect, ping-pong, subscription replay
- NotSupported with citation propagated as `--` cell

## Outside TRUSTED — all documented (DO NOT re-investigate)

- **CryptoCompare**: CCCAGG free tier wire-not-present for BID/ASK (verified by live curl). Other cells (orderbook UnsupportedOp, exch_info rate-limit, WS_trade/kline silent) are wire-not-present or quiet-channel.
- **MOEX**: RU IP required for FAST/CEDR streams. Geo-locked from non-RU.
- **Polymarket**: ClobWebSocket not implemented (NotSupported); REST partial.
- **Dukascopy**: tick-data-only; no public live REST.
- **Auth-gated venues**: Alpaca, AngelOne, Coinglass, Dhan, Finnhub, Futu, Fyers, Ib, JQuants, Krx, Oanda, Polygon, Tiingo, Tinkoff, Twelvedata, Upstox, Zerodha — skip without ENV creds. Validate via `e2e_smoke --trading` when keys present.

## Validation tooling

| Harness | Path | Use |
|---|---|---|
| Full matrix | `examples/e2e_smoke.rs` | 13 REST × 9 WS per exchange, parallel, per-exchange 90s cap |
| Liquidation capture | `examples/liq_capture.rs` | `--exchanges X,Y --symbol BTC-USDT --duration 7200` |
| Raw Bybit liq probe | `examples/bybit_liq_raw.rs` | Direct `tokio-tungstenite`, bypasses our transport. Used in Wave 10 to prove Bybit channel cadence (51 events/hr across 5 symbols). |
| Bitstamp trade probe | `examples/bitstamp_trade_capture.rs` | Used in Wave 8 to find double-connect bug. |
| MarketFeed demo | `examples/feed_demo.rs` | Early high-level API; will be superseded by Station in Phase 1. |

## What's next: Station layer

The connector library (Layer 1) is done. Next iteration adds:

- **Layer 2 — dig3-station**: high-level consumer-facing fluent builder with opt-in persistence / cache / multiplex / reconnect / gap-heal / orderbook-tracker / telemetry.
- **Layer 3 — dig3-cli**: `dig3` binary with watch/persist/replay/matrix/inspect/capture/benchmark subcommands.

See `docs/plans/station-architecture.md` for the architectural design and
`docs/plans/station-phase-1-plan.md` for the immediate Phase 1 implementation steps.

The workspace will be reorganized:

```
digdigdig3/
├── crates/dig3-core/      ← existing code, renamed
├── crates/dig3-station/   ← NEW Layer 2
└── crates/dig3-cli/       ← NEW Layer 3
```

## Validation gates that must continue passing

After every workspace-altering change:
1. `cargo check --workspace --all-targets --all-features` with `RUSTFLAGS=-D warnings` — clean.
2. `cargo test --workspace` — 821 pass (with the same single pre-existing dydx failure).
3. `cargo run --example e2e_smoke --release` — TRUSTED still 18 (snapshot variance ±2 OK due to market activity).
4. `cargo run --bin dig3 -- watch trades binance btc-usdt --duration 30` (Phase 1 acceptance) — prints live trades + persists to disk.

## Operating principles

- **Every Unsupported needs a research-agent before code.** No more guessing endpoint URLs or channel names. The pattern in waves 1-10 was: research-agent looks up live docs → code-agent applies the targeted fix → live capture verifies.
- **Don't deflect with doc-comments**. If a field is `None`, find out why with live docs and fix the wire path.
- **Differentiate `UnsupportedOperation` vs `NotSupported` always.** First is a TODO, second is reality. Audits depend on this.
- **Parallel test harnesses.** All e2e_smoke per-exchange work must run as `tokio::spawn` + `join_all` — one hang must not block the others.
- **`NotSupported` reasons cite the alternative.** "Use REST endpoint X" or "WS-only via channel Y" — never a bare "not supported".
- **Don't trust agent summary — verify via capture.** Waves 8-10 each found bugs where prior agents claimed "market quiet" but raw captures proved channels worked once parser/lifecycle bug was fixed.
