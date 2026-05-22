# MLI ↔ dig3 0.3.6 — findings from live indicator validator

**Discovery date:** 2026-05-22
**dig3 version:** 0.3.6
**Test harness:** `mli-collector-indicator-validator`
**Workload:** 53 Station subscribe calls (BTCUSDT FuturesCross on Binance +
Bybit, all 27 Stream variants).

## Critical: gap_heal cycle on NotSupported streams → OOM

### Symptom

Process memory grows unbounded. After ~2 minutes of runtime the validator
panics with `memory allocation of 80530636800 bytes failed` (80 GB).

**Note: this is not a resource exhaustion issue.** Host has 9.2 GB free
RAM (of 33 GB total) + 86 GB free disk. A single 80 GB allocation
request indicates integer overflow on a `Vec::with_capacity` /
`reserve` / equivalent — likely in the broadcast receiver or the
forwarder's event buffer.

### Root cause

For Stream variants where the exchange does not expose the channel (e.g.
Bybit `MarketWarning`, `AuctionEvent`, `RiskLimit`, `PredictedFunding`,
`FundingSettlement`, `OptionGreeks`, etc.), the initial `subscribe` returns
`Ok(())` (the Station accepts the subscription request and spawns the
forwarder), but the underlying `ws.subscribe_frame()` returns
`Unsupported operation: bybit: unsupported stream kind X`. The forwarder
task never receives any event.

After `DIG3_WS_SILENCE_SECS` (default 60 s) the forwarder hits
`silence_timeout` and the gap_heal logic kicks in:

```
ws disconnect detected → heal + resub
key=SeriesKey { exchange: Bybit, kind: MarketWarning } reason="silence_timeout"
resub cycle complete  unsub_ok=true sub_ok=true
WARN unsubscribe_frame failed  exchange=bybit  error=Unsupported operation: ...
WARN subscribe_frame failed    exchange=bybit  error=Unsupported operation: ...
```

The "resub cycle complete" log line lies — it reports `unsub_ok=true /
sub_ok=true` because the Station-level wrapper succeeded, but the inner
WS-level subscribe_frame returned NotSupported. The forwarder then
re-attaches `ws.event_stream()` and the cycle repeats every 60 seconds
for every NotSupported (exchange × stream) combo.

Each `event_stream()` call appears to spawn a new broadcast receiver
without releasing the old one (or some equivalent leak). With ~36 such
NotSupported combos (Bybit + Binance subscribed to 18 extended streams,
of which most are unsupported on these venues), the receivers accumulate
geometrically.

### Reproduction

```bash
cd mylittleindicators/crates/mli-collector
cargo build --release --bin mli-collector-indicator-validator
./target/release/mli-collector-indicator-validator.exe --duration-secs 600
```

Subscriptions list in `src/bin/indicator_validator.rs` (~line 1015)
includes 18 extended streams × 2 exchanges; most fail at the WS level.

### Impact on MLI

Without a fix, MLI cannot run the indicator validator on extended
streams in production mode. ~41 indicators across `greeks` (7),
`microstructure` (9), `index_basis` (6), `stress` (6),
`volatility_advanced` (4), and part of `risk_funding` (~9) remain
unvalidated because the validator OOMs before warmup completes.

### Workaround at consumer

Setting `DIG3_WS_SILENCE_SECS=999999` disables the gap_heal cycle and
allows the validator to run to completion. This is brittle (it also
disables legitimate kline heal) and should not be the long-term answer.

### Suggested fix in dig3

Three options, ordered by intrusiveness:

1. **Fail-closed on initial subscribe error.** If `ws.subscribe(req)`
   returns `Err(NotSupported)` at `acquire_or_spawn` time, do NOT spawn
   the forwarder — bubble the error up out of `Station::subscribe(set)`.
   The consumer sees the failure immediately and handles it (validator
   already wraps each subscribe in `match`/`warn skip`).

   Pro: simplest, matches existing eager-error convention from dig3 0.3.4.
   Con: changes `Station::subscribe` semantics — a SubscriptionSet that
   wraps multiple per-stream subscribes would no longer share a single
   handle.

2. **Track NotSupported per (exchange, kind) and skip heal.** Forwarder
   gets a flag set on first WS subscribe error. On silence_timeout, if
   the flag is set, do not heal — either exit the forwarder or stay
   parked. The Station-level handle for that stream returns no events
   ever; consumer's `event_stream` simply drains empty.

   Pro: backward-compatible with existing `Station::subscribe` API.
   Con: silent failure — consumer never knows the subscribe didn't
   actually reach the exchange unless they correlate WS warn logs.

3. **Fix the receiver leak.** Even if heal cycles forever, each cycle
   should release the previous `event_stream()` receiver. The leak is
   independent of whether heal is the right call — heal on a working
   stream that just had a brief outage should not leak either.

   Pro: addresses the actual OOM regardless of heal semantics.
   Con: doesn't address the wasted cycles (CPU + log spam).

Recommended: **(1) + (3)**. Fast-fail on first NotSupported makes the
consumer behavior explicit, and the leak fix protects against future
heal-pattern regressions.

### Diagnostic data

After ~2 minutes of runtime (estimated, before OOM):

- 53 station.subscribe calls issued
- ~17 succeeded fully (no WS error)
- ~36 wrapped a NotSupported subscribe_frame underneath
- Each of the 36 ran ~2 heal cycles before OOM
- Memory growth approximately linear with (heal cycles × NotSupported subscriptions)

## Other observations (not bugs, just notes)

- `ws.subscribe(req)` at `Station::acquire_or_spawn` succeeds even when
  the inner `subscribe_frame` returns NotSupported. This is the same
  pattern that caused the silent-stream confusion in 0.3.2 (closed by
  0.3.4 wave 11), now resurfacing through the gap_heal cycle in a
  different shape.
- All 27 Stream variants successfully compile and the per-Point disk
  store opens cleanly — the 0.3.5/0.3.6 expansion landed without
  consumer-side breakage.

## What MLI did to consume 0.3.6

- Pin bump `0.3.4 → 0.3.6` in `mylittleindicators/Cargo.toml` and
  `mli-collector/Cargo.toml`. Patch path unchanged (still
  `../digdigdig3/crates/digdigdig3`).
- No source changes were required — the lib + collector + smoke binary
  all compile clean against 0.3.6.
- Validator binary extended with 18 new event match arms, 18 new
  conversion helpers (Point → mli core type), 15 new `try_update_*`
  methods on `IndicatorState`, and 36 new subscribe combos (was 17, now
  53). Kline-trio (MarkPriceKline / IndexPriceKline / PremiumIndexKline)
  routes to `StreamKind::Bar` indicators because their payload is OHLCV.
