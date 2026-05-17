# dig3 Spec — Smoke v8 Findings

## 1. Summary

- **dig3 version**: 0.2.0 (`ExchangeHub` v0.2.0)
- **mli-collector-smoke version**: v8
- **Run date**: 2026-05-16 (smoke_run8.log, smoke_data/smoke_report.json)
- **Total duration**: 279s (REST 5s + WS 274s)
- **Exchanges**: 20/20 connected

### REST totals (308 endpoint calls)
| Outcome | Count |
|---|---|
| OK | 88 |
| Unsupported | 90 |
| Skipped | 123 |
| Auth required | 1 |
| Errors | 4 |

### WebSocket totals (841 subscription attempts)
| Outcome | Count |
|---|---|
| Subscribed (with data) | 57 |
| Subscribed (silent — 0 events) | 186 |
| Failed | 14 |
| Unsupported by exchange | 89 |
| Auth required | 0 |
| Symbol format errors | 0 |
| Rate limit | 0 |
| Dropped | 93 |
| Skipped | 402 |

### Throughput
- **220 335 total events** in ~4 min (274 s WS window) ≈ 800 events/s sustained.

### Source artifacts
- Log: `c:\Users\VA PC\CODING\ML_TRADING\nemo\mylittleindicators\crates\mli-collector\smoke_run8.log`
- JSON: `c:\Users\VA PC\CODING\ML_TRADING\nemo\mylittleindicators\crates\mli-collector\smoke_data\smoke_report.json`

---

## 2. REST issues — dig3-side bugs

Extracted from `grep "REST]" smoke_run8.log | grep -E "Other|Parse|HttpError"`.

### 2.1 HTX `get_ticker` (futures_cross) → ParseError "Invalid close price"

- **File**: `digdigdig3/src/l3/open/crypto/cex/htx/parser.rs::parse_ticker`
- **Line**: 99 (`tick["close"]` extraction fails)
- **Root cause**: HTX futures (linear-swap) `/market/detail/merged` returns ticker payload where the field is **not** `"close"` — it is `"close"` for spot only. For futures the field name differs (commonly `"last_px"`, or the response is wrapped differently in the `tick` envelope).
- **Symptom**: `tick["close"].as_f64()` returns `None` → `ExchangeError::Parse("Invalid close price")`.
- **Action**:
  1. Verify the exact HTX futures ticker response shape (`/linear-swap-ex/market/detail/merged` for futures_cross).
  2. Either:
     - Branch parser by `account_type` and read the correct field per-variant, OR
     - Try `close`/`last_px`/`price` in fallback order.
  3. Add a per-account-type unit test using a captured futures payload.

### 2.2 Gate.io `get_klines` (futures_cross) → ParseError "Kline is not an array"

- **File**: `digdigdig3/src/l3/open/crypto/cex/gateio/parser.rs::parse_klines`
- **Line**: 178 (`.as_array()` returns `None`)
- **Root cause**: Gate.io futures klines endpoint (`/api/v4/futures/{settle}/candlesticks`) returns objects (`{"t":…, "v":…, "c":…, "h":…, "l":…, "o":…}`), NOT bare arrays like spot (`/api/v4/spot/candlesticks`).
- **Symptom**: Parser expects `[ts, vol, close, high, low, open]` array; gets object → fails.
- **Action**:
  1. In `parse_klines`, detect element shape (`is_array` vs `is_object`).
  2. For object form, read fields `t`, `o`, `h`, `l`, `c`, `v`, `sum` (volume).
  3. Add fixture-based test for both spot bare-array and futures object forms.

### 2.3 BingX `get_open_interest` (futures_cross) → HttpError 109400 "symbol must end with -USDT or -USDC"

- **File**: `digdigdig3/src/l3/open/crypto/cex/bingx/connector.rs::get_open_interest`
- **Line**: 1232–1272 (specifically line 1247: `params.insert("symbol", symbol.to_string())`)
- **Root cause**: This method receives `symbol: &str` directly from the caller and inserts it **as-is** into `params` without calling `endpoints::format_symbol`. Mli passes `BTCUSDT` (raw concatenated), BingX swap endpoint requires `BTC-USDT`.
  - Note: `endpoints::format_symbol` (endpoints.rs:267) already produces correct `BTC-USDT` format — it is simply not used here.
- **Action**:
  1. Either change the signature to accept `&Symbol` (preferred — consistent with other methods), then call `format_symbol(&s.base, &s.quote, account_type)`.
  2. Or if keeping `&str`, normalize: if symbol has no `-`, split on USDT/USDC and reinsert hyphen.
  3. Audit ALL BingX connector methods that take `symbol: &str` for the same pattern (likely several).

### 2.4 OKX `get_liquidation_history` (futures_cross) → "Either parameter uly or instFamily is required"

- **File**: `digdigdig3/src/l3/open/crypto/cex/okx/connector.rs::get_liquidation_history`
- **Line**: 2257–2269 (trait impl); internally calls `get_liquidation_orders` at line 436.
- **Root cause**: OKX `/api/v5/public/liquidation-orders` for SWAP/FUTURES/OPTION **requires** either `uly` (underlying, e.g. `BTC-USD`) or `instFamily` (e.g. `BTC-USDT`). The trait method passes `instFamily=None` to `get_liquidation_orders`. `instId` alone is insufficient.
- **Action**: In `get_liquidation_history`, derive `instFamily` from the symbol:
  ```rust
  let inst_family = symbol.map(|s|
      format!("{}-{}", s.base.to_uppercase(), s.quote.to_uppercase())
  );
  self.get_liquidation_orders(
      inst_type,
      inst_family.as_deref(),  // ← was None
      inst_id.as_deref(),
      Some("filled"), None, None, limit
  ).await
  ```
  Caveat: for SWAP `instFamily` is e.g. `BTC-USDT`; for COIN-margined SWAP it would be `BTC-USD`. Use `account_type` to pick.

### 2.5 Deribit Options basic methods → "wrong format param instrument_name"

- **File**: `digdigdig3/src/l3/open/crypto/cex/deribit/endpoints.rs::format_symbol`
- **Status**: **Documented limitation, NOT a bug.** Already filtered upstream in smoke v8 — log line 27 confirms: *"Deribit Options REST skipped — basic methods require specific contract instrument_name, not generic Symbol"*.
- **Background**: Generic `Symbol::new("BTC", "USD")` resolves to `BTC-PERPETUAL` (a perp), but if the caller forces `account_type=Options`, Deribit's options endpoints need a concrete contract like `BTC-30MAY26-50000-C`. Generic `Symbol` does not encode expiry/strike/side.
- **Recommended dig3 action** — pick ONE:
  - **(a)** When `account_type=Options` and `symbol` has no `raw()` (no explicit instrument), return `ExchangeError::UnsupportedOperation("Deribit options require specific instrument_name (e.g. BTC-30MAY26-50000-C); generic Symbol not supported")`. **This is the cleanest fix** — gives mli/clients a typed signal instead of an HTTP 400.
  - **(b)** Add an `OptionsSymbol` variant (`Symbol::option { base, expiry, strike, side }`) with a builder. Heavier change; needs trait signature update.
  - **(c)** Accept current state but document in trait docs that Options instruments need `Symbol::with_raw("BTC-30MAY26-50000-C")` and otherwise return `UnsupportedOperation`.
- **Default recommendation**: **(a)** for v0.2.1, **(b)** for v0.3.

---

## 3. WebSocket issues — dig3-side bugs

### 3.1 Bitget — "Trying to work with closed connection" (60 hits across spot + futures_cross)

- **File**: `digdigdig3/src/l3/open/crypto/cex/bitget/websocket.rs`
- **Symptoms**:
  - smoke matrix shows **Bitget `spot` + `futures_cross` `Failed`** on EVERY stream type (ticker, trade, orderbook, orderbook_delta, kline, …).
  - 60 occurrences of `closed connection` errors in the log.
- **Diagnosis**: smoke subscribes to many stream types **sequentially in one connection**. After the first few subscribe calls, the underlying WS is dropped (likely no ping → server closes after timeout, OR our code closes after first error and doesn't reopen). All subsequent subscriptions on that handle fail with "Trying to work with closed connection".
- **Note**: file already references `last_ping`/`pong` infrastructure (lines 151–187, 276), so partial heartbeat code exists. Likely missing pieces:
  - Outbound ping is not being **sent** on schedule (timer task may not be spawned).
  - OR the close-handler does not flip status back to `Connecting` to trigger reconnect.
  - OR the broadcast sender is `take()`-ed on close (line 317) which makes all subsequent `subscribe()` calls fail because there's no live channel.
- **Action**:
  1. Audit the ping task — verify it actually fires every 30s with the right Bitget-specific frame (Bitget needs the literal string `"ping"` for public WS).
  2. Implement auto-reconnect: on close, mark `Disconnected`, recreate the broadcast channel, reconnect, **replay all active subscriptions**.
  3. Make `subscribe()` either lazily connect or return `ConnectionClosed` (typed) instead of silently inserting into a dead channel.
  4. Add a regression test: 30 sequential subscribes on one Bitget connection over a 60s window must all succeed.

### 3.2 Lighter — REST & WS hang the tokio runtime

- **Files**:
  - `digdigdig3/src/l3/open/crypto/dex/lighter/connector.rs::get_klines` (line 403)
  - Likely also other REST methods + WS path
- **Symptom**: smoke v8 explicitly skips Lighter with `WARN: Lighter REST audit skipped — dig3 connector blocks tokio (sync code in get_klines)` (log line 28). WS skipped for the same reason (line 42).
- **Diagnosis**: `get_klines` body itself looks async, but the call chain likely contains:
  - synchronous gRPC / blocking HTTP client somewhere in `LighterEndpoint::Candlesticks` resolution or `get_market_id`,
  - or a `std::sync::Mutex` held across `.await`,
  - or a sync init step (e.g. fetching the market list from gRPC the first time) that is not properly `spawn_blocking`-wrapped.
- **Action**:
  1. **Audit ALL Lighter REST methods** (`get_klines`, `get_orderbook`, `get_ticker`, `get_recent_trades`, `ping`, `get_market_id`) for:
     - `std::thread::sleep` → replace with `tokio::time::sleep`
     - `std::sync::Mutex` held across `await` → replace with `tokio::sync::Mutex` or scope-release before await
     - blocking HTTP / gRPC clients → migrate to async client OR wrap entrypoints in `tokio::task::spawn_blocking`
     - blocking channel ops (`recv()` instead of `recv().await`)
  2. **Audit the Lighter WS path** identically.
  3. Add a test that runs `tokio::time::timeout(Duration::from_secs(5), connector.get_klines(...))` — must NOT hang.

### 3.3 Silent WS streams (186 hits) — subscribe OK but 0 events

Subscribe call succeeds, but mli receives zero `StreamEvent`s during the observation window. Either the connector ignores incoming frames, parses them and drops them silently, or the channel mapping is wrong.

Per-stream silent producers extracted from `smoke_run8.log` STREAM AVAILABILITY MATRIX (lines 1336–1493):

| Stream | Silent exchanges/accounts |
|---|---|
| `ticker` | binance:spot, binance:futures_cross, htx:futures_cross, upbit:spot |
| `trade` | mexc:spot, binance:spot, binance:futures_cross, htx:spot, htx:futures_cross, dydx:futures_cross, bitstamp:spot, gateio:futures_cross, gemini:spot, upbit:spot |
| `orderbook` | mexc:spot, coinbase:spot, okx:spot, binance:spot, bitfinex:spot, binance:futures_cross, bybit:futures_cross, kucoin:futures_cross, deribit:options, dydx:futures_cross, bybit:spot, gateio:futures_cross, okx:futures_cross, gemini:spot, upbit:spot, crypto_com:spot, kraken:spot |
| `orderbook_delta` | binance:spot, gateio:spot, bitfinex:spot, binance:futures_cross, hyperliquid:futures_cross, bitstamp:spot, gateio:futures_cross |
| `kline:1m` | okx:spot, binance:spot, binance:futures_cross, bybit:futures_cross, htx:spot, htx:futures_cross, kucoin:futures_cross, deribit:options, dydx:futures_cross, bybit:spot, okx:futures_cross |
| `mark_price` | binance:futures_cross |
| `funding_rate` | binance:futures_cross, bybit:futures_cross |
| `liquidation` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, htx:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `open_interest` | binance:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `long_short_ratio` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `agg_trade` | okx:spot, binance:spot, gateio:spot, binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, bybit:spot, gateio:futures_cross, okx:futures_cross, kraken:spot |
| `composite_index` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `mark_price_kline:1m` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `index_price_kline:1m` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `premium_index_kline:1m` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `index_price` | binance:futures_cross, bybit:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `insurance_fund` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `basis` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `option_greeks` | deribit:options |
| `auction_event` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `market_warning` | okx:spot, binance:spot, gateio:spot, binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, bybit:spot, gateio:futures_cross, okx:futures_cross, kraken:spot |
| `block_trade` | coinbase:spot, okx:spot, binance:spot, gateio:spot, binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, bybit:spot, gateio:futures_cross, okx:futures_cross, kraken:spot |
| `orderbook_l3` | okx:spot, binance:spot, gateio:spot, binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, bybit:spot, gateio:futures_cross, okx:futures_cross, kraken:spot |
| `settlement_event` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `risk_limit` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `predicted_funding` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `funding_settlement` | binance:futures_cross, bybit:futures_cross, hyperliquid:futures_cross, kucoin:futures_cross, gateio:futures_cross, okx:futures_cross |
| `historical_volatility` / `volatility_index` | (unsupported by deribit, not silent — see §3.4) |

**Worst offenders (concentrated bugs, not "exchange has no channel")**:

| Connector | Pattern |
|---|---|
| **Binance spot + futures_cross** | Silent on ticker, trade, orderbook, orderbook_delta, kline:1m, mark_price, funding_rate, liquidation, open_interest, long_short_ratio, agg_trade, ALL extended streams. Subscribes succeed but no events delivered. **High-priority audit.** |
| **HTX spot + futures_cross** | Silent on ticker (futures only), trade, kline. Combined with REST ticker parse bug (§2.1). |
| **MEXC spot + futures_cross** | Silent on trade, agg_trade. Mexc has a known frame-format quirk (gzip-deflate); likely the dig3 frame decoder is dropping/mis-parsing. |
| **Kucoin futures_cross** | Silent on orderbook, kline, liquidation, open_interest, long_short_ratio, all extended futures streams. |
| **OKX futures_cross + spot** | Silent on orderbook, kline:1m, agg_trade, and ALL futures extended streams (funding_rate, open_interest, long_short_ratio, mark/index/premium klines, …). Subscribe acks look fine but events never flow. |
| **Gateio futures_cross** | Silent on orderbook_delta, trade, orderbook, kline, mark/funding, liquidation, OI, LSR, all extended. |
| **Bybit futures_cross + spot** | Silent on orderbook, kline:1m, agg_trade, funding_rate, OI, LSR, all extended. |
| **Hyperliquid futures_cross** | Silent on orderbook_delta, all extended futures streams. |
| **Deribit options** | Silent on orderbook, kline:1m, option_greeks. |
| **Other (single-stream silent)** | upbit:spot ticker/trade, bitstamp:spot trade/orderbook_delta, gemini:spot trade/orderbook_delta, dydx:futures_cross trade, coinbase:spot orderbook, bitfinex:spot orderbook/orderbook_delta. |

**Likely root causes** (per category):
1. **Subscribe ack consumed but data frames not routed** — incoming frame handler matches on event/channel name; if the channel ID/format differs from the subscribe payload (common in Binance: `btcusdt@trade` lowercase vs `BTCUSDT` uppercase), frames are received but dropped silently before reaching `event_stream()`.
2. **Frame decoder swallows errors** — gzip/deflate (HTX, OKX, KuCoin) or protobuf wrappers; on decode error the message is dropped instead of surfacing via `WebSocketError`.
3. **Wrong subscription topic** — for futures-extended streams (mark price kline, premium index, insurance fund, basis, …) the connector may use the same handler as plain kline; payload arrives on a topic the handler does not match.
4. **`subscribe()` returns Ok without confirming server-side ack** — broker queues the topic locally but never sends to the wire, OR the server NACKs and we ignore it.

**Action plan per connector**:
1. Enable `tracing::trace` inside `handle_message()` on the silent connectors during a re-run to capture raw frames.
2. For each silent (exchange, stream) pair, diff the actual incoming topic string vs the topic our `subscribe()` registered.
3. Ensure unknown frames are NOT silently dropped — emit `WebSocketError::UnknownFrame { topic, len }` so future smokes see them.
4. Add unit tests that feed captured frame payloads into `handle_message` and assert events emitted.

**Order of attack** (impact-sorted): Binance → OKX → Bybit → Gateio → HTX → KuCoin → MEXC → Hyperliquid → Deribit options → singletons.

### 3.4 UnsupportedByExchange (89 hits) — NOT a bug, informative only

The exchange itself returns "channel does not exist" (or our connector knows statically that the channel is unavailable). Examples per smoke matrix:

- **HTX futures_cross**: orderbook_l3, agg_trade, market_warning, block_trade, mark_price, funding_rate, liquidation, OI, LSR, composite_index, mark/index/premium klines, index_price, insurance_fund, basis, auction_event, settlement_event, risk_limit, predicted_funding, funding_settlement
- **MEXC spot**: orderbook_delta, agg_trade, market_warning, block_trade, orderbook_l3, kline:1m (also unsupported)
- **Bitfinex spot**: agg_trade, market_warning, block_trade
- **Bitstamp spot**: agg_trade, market_warning, block_trade
- **Coinbase spot**: agg_trade, market_warning, orderbook_l3
- **Upbit spot**: orderbook_delta, agg_trade, market_warning, block_trade, orderbook_l3
- **Crypto.com spot**: agg_trade, market_warning, block_trade, orderbook_l3
- **DyDx futures_cross**: orderbook_delta, market_warning, block_trade, …
- **Deribit options**: mark_price, funding_rate, liquidation, OI, LSR, mark/index/premium klines, index_price, insurance_fund, basis, auction_event, settlement_event, risk_limit, predicted_funding, funding_settlement, historical_volatility, volatility_index, market_warning, block_trade, orderbook_l3, agg_trade, …

**Do not touch.** These are correct `UnsupportedOperation` returns. They feed into the capability-flag gap discussed in §4.

---

## 4. Capability flags — extension needed

### Current state

`MarketDataCapabilities` (`digdigdig3/src/core/types/capabilities.rs`, lines 873–878) exposes only 4 WS flags on the public surface:

```rust
pub has_ws_klines: bool,
pub has_ws_trades: bool,
pub has_ws_orderbook: bool,
pub has_ws_ticker: bool,
pub has_ws_mark_price: bool,
pub has_ws_funding_rate: bool,
```

### Gap

mli smoke must subscribe blindly and discover unsupported streams only via runtime `UnsupportedByExchange` errors (the 89 hits in §3.4). With richer capability flags, mli can short-circuit at planning time and never attempt unsupported subscriptions.

### Required new flags (add to both `MarketDataCapabilities` and `MarketDataCapabilitiesExt`)

```rust
// Trade variants
pub has_ws_agg_trade: bool,
pub has_ws_block_trade: bool,

// Orderbook variants
pub has_ws_orderbook_l3: bool,
pub has_ws_orderbook_delta: bool,   // (currently inferred — make explicit)

// Funding / settlement
pub has_ws_predicted_funding: bool,
pub has_ws_funding_settlement: bool,
pub has_ws_settlement: bool,

// Risk / insurance
pub has_ws_risk_limit: bool,
pub has_ws_insurance_fund: bool,

// Index / basis / mark variants
pub has_ws_index_price: bool,
pub has_ws_basis: bool,
pub has_ws_composite_index: bool,
pub has_ws_mark_price_kline: bool,
pub has_ws_index_price_kline: bool,
pub has_ws_premium_index_kline: bool,

// Options-specific
pub has_ws_option_greeks: bool,
pub has_ws_volatility_index: bool,
pub has_ws_historical_volatility: bool,

// Market lifecycle
pub has_ws_auction_event: bool,
pub has_ws_market_warning: bool,
```

**Total**: 19 new flags. Defaults: `false` for the conservative-defaults preset, `true` for the maximalist preset (use grep on existing `Default`/`default()` impls to find all four preset blocks at lines ~52, ~74, ~96, ~115, ~134 in capabilities.rs).

### Per-connector wiring

Each `<exchange>/connector.rs` `market_data_capabilities()` impl must set new flags accurately. Reference the smoke v8 matrix:
- `Working (events)` → `true`
- `Silent (0 events)` → `true` (channel exists, separate bug — see §3.3)
- `Unsupported by exchange` → `false`
- `Failed` (only Bitget) → `true` once §3.1 is fixed

---

## 5. Auth-required REST (1 hit) — informative only

- `binance/futures_cross.get_force_orders` → `AuthRequired { msg: "Invalid credentials: API-key format invalid." }`
- **Not a bug**: Binance `/fapi/v1/forceOrders` requires SIGNED auth even though the data is public-flavoured (liquidations).
- **Action (docs only)**: in `MarketDataPublic` trait docs, annotate that `get_force_orders` / liquidation endpoints on Binance require credentials. Consider adding a `requires_auth: bool` field to the capability struct OR a separate `auth_required_for_public_data: Vec<MethodKind>` list.

---

## 6. Follow-up improvements (non-blocking)

1. **Tracing instrumentation**: add `tracing::trace!` inside every connector's `handle_message`/`on_frame` so silent-stream diagnosis becomes a log-grep instead of a code-read. Gate behind feature `dig3-ws-trace`.
2. **Unified WS reconnect strategy**: extract a reusable `ReconnectingTransport` (jittered backoff, subscription replay, ping scheduler) into `core/websocket/` to fix Bitget AND future similar bugs once.
3. **Capability discovery API**: replace plain `has_ws_*: bool` with a queryable trait method:
   ```rust
   fn supports_stream(&self, kind: StreamKind, account: AccountType) -> SupportLevel;
   // SupportLevel ∈ { Native, NotImplemented, UnsupportedByExchange, RequiresAuth }
   ```
4. **WS frame error policy**: never silently drop unknown topics — surface as `WebSocketError::UnknownFrame { topic }`.
5. **Smoke-regression hooks**: ship a `cargo run --bin dig3-smoke -- --exchange binance` short-form that mli can wrap, so dig3 PRs can self-validate against the same harness mli uses.

---

## 7. File / line index — bugs at a glance

| # | Bug | File (relative to `digdigdig3/`) | Approx line |
|---|---|---|---|
| 2.1 | HTX ticker parser — futures close field | `src/l3/open/crypto/cex/htx/parser.rs::parse_ticker` | 99 |
| 2.2 | Gate.io klines parser — futures object form | `src/l3/open/crypto/cex/gateio/parser.rs::parse_klines` | 178 |
| 2.3 | BingX format_symbol — `get_open_interest` skips formatter | `src/l3/open/crypto/cex/bingx/connector.rs::get_open_interest` | 1232–1272 (esp. 1247) |
| 2.4 | OKX `instFamily`/`uly` missing | `src/l3/open/crypto/cex/okx/connector.rs::get_liquidation_history` | 2257–2269 (helper at 436) |
| 2.5 | Deribit Options generic Symbol → HTTP 400 | `src/l3/open/crypto/cex/deribit/endpoints.rs::format_symbol` | TBD (file exists; specific function line not located in this pass) |
| 3.1 | Bitget WS — no auto-reconnect / ping not sending | `src/l3/open/crypto/cex/bitget/websocket.rs` | 151–187 (ping state), 276–321 (close handler) |
| 3.2 | Lighter blocks tokio runtime | `src/l3/open/crypto/dex/lighter/connector.rs` | 403 (`get_klines`), audit full file |
| 3.3 | Silent WS streams (Binance/OKX/Bybit/HTX/KuCoin/Gateio/MEXC/Hyperliquid/Deribit) | `src/l3/open/crypto/cex/<exchange>/websocket.rs::handle_message` | TBD — needs per-connector trace |
| 4 | Capability flag expansion (19 new bool flags) | `src/core/types/capabilities.rs` | 28–137 (presets), 873–878 (Ext struct) |

---

## Files NOT modified by this spec

This is documentation only. No dig3 source changes were made. No mli changes are in scope — see separate mli-side spec (TBD) for collector-side improvements (planning-time capability checks, smarter skip lists, trace logging).
