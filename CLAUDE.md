# digdigdig3 (dig3)

Multi-exchange connector library covering 47 exchanges. 18 TRUSTED (all major crypto + 4 DEX, full futures coverage). Three crates in one workspace — single version pin (uzor-style), currently `0.3.11`:

- **`digdigdig3`** — pure connector library. ONLY `ExchangeHub` + REST/WS connectors + capabilities + symbol normalization. No persistence, no replay, no cure/cache infrastructure.
- **`digdigdig3-station`** — high-level builder over `ExchangeHub`. OWNS: unified `Series<T>` / `DiskStore<T>` over 27 `DataPoint` impls (9 core + 18 extended for MLI), `SeriesKey { exchange, account, symbol, kind }`, multiplexed `Station` (N consumers share one WS per StreamKey), warm-start, REST cache, replay, cure, **auto-heal on WS disconnect** (kline-only — see below). String-bearing variants (BlockTrade, AuctionEvent, MarketWarning, OrderbookL3) persist via fixed header + companion `.blob` file.
- **`digdigdig3-cli`** — `dig3` binary (watch trades/orderbook/kline/ticker/mark/funding/open-interest/liquidations/agg-trades) + `dig3-inspect` post-mortem analyzer + legacy `dig3-catcher` / `dig3-cure` bins.

## Bar-aligned non-OHLCV loader (2026-06-04, local — not bumped/published)

dig3's side of the mlq data-handoff (`nemo/docs/mlq/data-handoff-dig3-2026-06-04.md`):
deliver non-OHLCV market data as **bar-aligned historical series** so the mlq
backtester can warmup the ~130 non-OHLCV mli indicators. Three commits, all LOCAL:

- **Track A — derived kline REST (`feat(binance)` cb82864 + `fix` 808426f):**
  wired `markPriceKlines` / `indexPriceKlines` / `premiumIndexKlines` (3 new
  `BinanceEndpoint` variants + shared `get_derived_klines` helper reusing
  `parse_klines` — identical 12-elem array). New `MarketDataPublic::
  get_premium_index_klines` trait method (mark/index already existed as stubs).
  New `has_premium_index_klines` capability flag (Binance=true, false elsewhere).
  **Quirk:** `indexPriceKlines` keys the instrument as `pair`, not `symbol`
  (mark/premium use `symbol`) — found by the live e2e.
- **Track B — station bar-align loader (`feat(station)` 4764599):**
  `bar_align::load_bar_aligned(exchange, account, symbol, kind, interval, range)
  -> BarAlignedSeries` (`Klines(Vec<BarPoint>)` | `Scalar(Vec<ScalarBar>)`).
  Fill policy by stream nature (`Kind::fill_policy`): **state** streams
  (funding/OI/mark/index/LSR) carry-forward into empty bars; **flow** streams
  (liq/aggTrade) bucket-sum, gap=0. Kline-family returns native bars (paginated
  backwards over the range, deduped); scalars resample onto the interval grid.
  Flow + book streams return `StreamNotSupported` (daemon-required, Track C TODO).
- **Tests:** 5 resampler unit tests + `examples/bar_align_e2e.rs` LIVE e2e —
  **9/9 green** vs Binance USDⓈ-M (mark/index/premium klines + funding + OI +
  LSR + mark-scalar all valid & bar-grid-aligned; liq/aggTrade daemon-gated).
  Run: `cargo run -p digdigdig3-station --example bar_align_e2e`.
- **Multi-exchange coverage (2026-06-04, commit dd4a616):** after a per-venue
  internet research pass (`nemo/docs/mlq/validation/rest-history-*.md`, 1 agent
  per 3 exchanges against official docs), the non-OHLCV history methods were
  wired + live-debugged to green across **7 venues**. `examples/bar_align_matrix.rs`
  = the live gate (7 exchanges × up-to-6 streams, all PASS).

  | Venue | mark | index | premium | funding | OI | LSR |
  |---|---|---|---|---|---|---|
  | Binance | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
  | Bybit | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
  | Gate.io | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |
  | OKX | ✓ | ✓ | NS(tick) | ✓ | ✓ | ✓ |
  | HTX | ✓ | NS | ✓ | ✓ | ✓ | ✓(elite) |
  | Bitget | ✓ | ✓ | NS | ✓ | NS(snap) | ✓ |
  | Kraken-fut | ✓ | ✓ | — | ✓ | TODO | TODO |

  Live-debug findings worth remembering: Bybit derived-kline rows are
  `[t,o,h,l,c]` (no volume) — `parse_klines` must not `?`-drop on missing idx5;
  OKX index klines need the index instId (`BTC-USD`), OI needs `instId` (not
  `ccy`), LSR/OI period must be upper-cased (`1H`), funding uses before=start/
  after=end; HTX mark/premium/estimated klines live under `/index/market/history/`
  (NOT `/linear-swap-ex/`), no index-price kline exists; Kraken charts/v1 `to`-only
  returns from-genesis (must bound `from`) and its candle `time` is already ms.

- **Full rollout landed (2026-06-04, commits 669ac45 + 597102d).** All researched
  venues wired + live-debugged. `bar_align_matrix.rs` extended to **15 venues +
  taker direct-checks → PASS**. Additions since the 7-venue table above:
  - **Kraken OI/LSR** — `charts/v1/analytics/{open-interest,long-short-info}`
    (analytics_type strings live-verified); flags true.
  - **basis/taker lifted into `MarketDataPublic`** — new `TakerVolume` core type +
    `get_basis_history` / `get_taker_volume_history`. Implemented: basis on
    Binance + HTX; taker on Binance + OKX + Gate.io + Bitget (all live-green).
    `Kind::Basis` wired into the station loader (state/ffill). Bybit basis =
    wire-absent (v5/market/basis 404).
  - **Thin venues** — MEXC (mark/index/funding), Crypto.com (funding; mark/index
    are minute-tick valuations, NOT bar-aligned klines → not loader-routed),
    BingX (mark/funding; index/premium/LSR unverified→default), dYdX (funding),
    HyperLiquid (funding), Lighter (mark/funding; market_id int map), Bitfinex
    (funding/OI/LSR via status/deriv — parser indices were off-by-one, fixed),
    Deribit (funding + historical_volatility).
  - **Track C** — `bar_align_points<T: DataPoint>` resamples daemon-recorded
    streams onto the grid (recording already exists via `dig3 watch`→DiskStore).
  - **Live-debug lessons** (cargo-check-clean ≠ correct — found ONLY by live e2e):
    OKX funding before/after was swapped; OKX mark path + OI instId/period; HTX
    history endpoints live under `/index/market/history/` (not `/linear-swap-ex/`);
    Bybit derived-kline rows have no volume idx; Kraken charts `to`-only =
    from-genesis + ms `time`; Bitfinex `{key}/hist` has no leading key element.

- **Scope closed (2026-06-04, commit 0b42140):** Crypto.com klines now
  interval-bucketed (per-minute valuations → OHLC); BingX index/premium/LSR
  confirmed wire-absent via live probe (code 100400) → NotSupported; Kraken
  premium = derived mark−spot; Track-C read-path = `bar_align_from_disk`.
  **Both e2e green: NATIVE (`bar_align_matrix`, 15 venues + taker) + WASM
  (`wasm_bar_align`, headless Edge/msedgedriver, Lighter direct) 2/2.**
- **Genuinely external / out-of-contract (NOT our TODO):** Tardis historical
  archive ($350/mo, the only paywall — optional deep-history bootstrap);
  a long-running HTTP auto-serve daemon (mlq links Station directly per the
  handoff, so not needed). NOT bumped/published (awaiting command).
- **WASM test note:** Windows usernames with a space (`VA PC`) break the
  wasm-bindgen-test-runner path — run station wasm tests via 8.3 short paths
  (`VAPC~1`) in `CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUNNER` + `CHROMEDRIVER`
  (see `wasm_bar_align.rs` header). Native-only station tests are
  `cfg(not(wasm32))`-gated so the wasm test target builds.

## WASM Wave 3 (2026-05-28, in-flight — NOT yet bumped/published)

Full wasm parity for the 0.4.0 target. Local commits only, no push/publish/bump
until the whole wave is accepted. Plan: `docs/plans/wasm-wave3-master.md` +
4 research reports under `docs/research/wasm-wave3/`.

**What landed (15 commits on top of Wave 2's c9584a7..34858ba):**

- **WsProtocol hooks**: `is_server_ping` + `pong_response_frame` (server-initiated
  heartbeats), `post_connect_delay` (Crypto.com 1s mandatory connect pause).
  Binary frame decode (BingX gzip, Upbit UTF-8) handled by the pre-existing
  `decode_binary` default fallback (gzip → zlib → deflate → utf-8).
- **5 bespoke WS migrated to `UniversalWsTransport`** (Workstream B — the user's
  hard blocker): Gemini, CryptoCom, Bitfinex, BingX, Upbit. ~4,200 LOC bespoke
  loops → ~600 LOC wrappers + ~2,200 LOC protocol.rs. All now compile to wasm32.
  - Gemini + Upbit synthetic Ticker punted (return `NotSupported`) — needs a
    stateful cross-channel parser hook; Trade + Orderbook fully migrated.
  - Bitfinex chanId integer routing via `Arc<Mutex<HashMap<u64, TopicKey>>>` in
    the protocol struct + thread-local symbol passthrough to fn-pointer parsers.
  - CryptoCom server heartbeat via the new hooks + `post_connect_delay`.
- **REST override end-to-end** (Workstream A): `set_rest_base_override` was a dead
  API in Wave 2; now `ConnectorFactory::create_public` threads `Option<String>`
  into 9 wasm-eligible connectors (Binance/Bybit/OKX/Bitget/Bitstamp/Coinbase/
  Kraken/Deribit/HTX) which substitute it at every base_url call site. Unblocks
  MLC-backend-proxy + nginx-proxy patterns. `None` = current behavior (zero regression).
- **3 DEX wasm-enabled, public data only** (Workstream F): HyperLiquid (cfg fix —
  k256+sha3 compile to wasm, auth only in trading ctor), dYdX v4 + Lighter WS
  ported to `UniversalWsTransport`. REST arms for all three on wasm. Trading paths
  stay native-only by design. Lighter CORS confirmed `*` (live curl 2026-05-28).
- **OPFS DiskStore** (Workstream C): wasm-side `series/store_wasm.rs` (Option C —
  in-memory buffer + async `createWritable` flush). Native `new/flush/read_tail`
  widened to async for API parity (`append` stays sync). `Drop` keeps a sync flush.
- **Un-gated polling + gap_heal for wasm** (Workstream D). `cure` + `replay`
  DEFERRED to Wave 4 — both depend on the native sled/tokio::fs `StorageManager`;
  need a wasm StorageManager port first. Poll-only kinds (LSR, HV) gracefully
  return `StreamNotSupported` on wasm (browser fetch is `!Send`).
- **Wasm e2e test suite** (Workstream E): `wasm_rest_corsproxy`, `wasm_opfs_round_trip`
  (100 appends + read_tail), `wasm_poll_degradation`. cure/replay live tests deferred.

**HyperLiquid mis-placement** (`cex/hyperliquid/` → should be `dex/`) is a known
legacy directory mistake — refactor is a separate followup, NOT Wave 3.

### Wave 3 live-validation pass (2026-05-28) — transport deadlock fix

`cargo check` passing ≠ data flowing. A full live `e2e_smoke` run after the WS
migrations exposed that most WS connectors emitted ZERO events. Root cause was
NOT per-connector — a **transport-level deadlock**:

- The driver shares ONE `Arc<Mutex<Box<dyn WsConn>>>` between read and write
  tasks. The read task held the lock across a blocking `next_frame().await`.
  On venues that send nothing until they receive a subscribe (dYdX, Gemini,
  Bitfinex, Upbit, Lighter), the write task could never acquire the lock to
  SEND the subscribe → silent forever, no error. High-frequency venues
  (Binance/BingX/CryptoCom) masked it by releasing the lock between frames.
- Fix: read task polls `next_frame` under a 100ms timeout, releasing the lock
  each tick. Plus two smaller systemic fixes: `WsProtocol::uses_native_ping()`
  (Gemini disconnects on client Ping, Upbit can't flush Pong promptly under the
  task split) + ping deadline reset to +interval (no ping on connect); and
  replacing the sentinel `Subscribe(Ticker,"")` in `connect()` with a no-op
  `TransportCmd::Connect` (the empty symbol leaked a malformed subscribe to
  dYdX/Bitfinex). Commit 70f82b6.
- Per-connector fixes (commit b5105a8): Kraken WS needs `BASE/QUOTE` slash
  format not REST `XBTUSD` (`to_kraken_ws_symbol` normalizer added); Bitstamp
  synthetic ticker had bid/ask swapped; Lighter AggTrade misrouted to Trade
  parser → now `NotSupported` (Lighter has no aggTrade channel).

**Result: TRUSTED 5 → 15.** All 18 crypto WS connectors flow real data live
(trade/ob/kline verified green per venue). All 7 migrated WS connectors pass
their `--ignored` live tests. Wasm smoke still 2/2 in headless Chrome (the
timeout-poll is wasm-safe).

### Flag audit + channel completion (2026-05-29)

Audited every `NotSupported` vs `UnsupportedOperation` flag in all 21 WS
protocols against the OFFICIAL exchange docs (4 reports in
`docs/research/wave3-debug/flag-audit-group{1..4}.md`). Convention:
`NotSupported` = exchange has no such public WS channel (wire-absent);
`UnsupportedOperation` = exchange HAS it, we haven't implemented (TODO).
Migrating agents had mislabeled 18 channels as `NotSupported` that actually
exist — reclassified to `UnsupportedOperation` (commit beebe40), then
**implemented** the genuinely-public ones, each LIVE-verified (real data on the
wire, not just `cargo check`):

| Venue | Channel | Live result |
|---|---|---|
| Upbit | native Ticker | `last=108665000` |
| Gemini | synthetic Ticker (from l2 changes+trades, stateful) | `bid<ask<last` |
| GateIO | OpenInterest (`futures.contract_stats`, 1m) | `oi=640374112` |
| HTX | IndexPriceKline (`market.<c>.index.<p>`, separate `ws_index` endpoint) | `open=73706` |
| KuCoin | Liquidation (`/contractMarket/liquidationOrders`, PUBLIC — old "needs auth" comment was wrong) | wired (sparse) |
| Bitfinex | Liquidation (public `status` `key:liq:global`) | caught live `ETHF0 Buy 19@2013.5` |
| Lighter | Kline (`candle/<mkt>/<res>`) | `open=73665 close=73633` |

**BingX funding/forceOrder/openInterest/aggTrade**: the doc-audit agent CLAIMED
these exist (trusted an unofficial GitHub repo) — but a live raw probe returned
`code 80015 "dataType not support"` for all four. They are genuinely wire-absent
→ kept/corrected to `NotSupported` (commit 1b9fd5c). Lesson reinforced: even a
docs audit is not authoritative — the live wire is. dYdX/Crypto.com
account-scoped Balance/Position/Order updates stay `UnsupportedOperation`
(exist on `v4_subaccounts` / `user.*`, out of public-data scope).

Confirmed-correct-NotSupported (left as-is): Binance OI (REST-only), Bitget liq
(UTA-V3-only), HyperLiquid liq/aggTrade, Bitstamp/Coinbase/Kraken catch-alls.

Native after channel work: 25 test-suites ok, 0 failed, `-D warnings` clean.
Wasm Chrome smoke 2/2.

**Lesson (durable):** a connector migration is not done at `cargo check` — it
is done when a live `e2e_smoke` / `--ignored` test shows real data flowing.
Always run the live matrix before claiming a WS connector works.

Native baseline after Wave 3: 1090/0 across lib + integration + examples,
`RUSTFLAGS=-D warnings` clean. Wasm: lib + station compile clean to wasm32.

## Major refactors (0.3.4 + 0.3.5 + 0.3.6 + 0.3.7 + 0.3.8 + 0.3.9)

**0.3.9 (2026-05-23)** — `Stream::OrderbookDelta` exposed in Station (MLI Ask 1):

- New `Kind::OrderbookDelta` + `Stream::OrderbookDelta` + `Event::OrderbookDelta { exchange, symbol, point: ObDeltaPoint }`. dig3 core already parsed `StreamEvent::OrderbookDelta`; Station was the bottleneck — only the snapshot variant was exposed. Unblocks 11 book/cluster indicators (BookSlope, QueueImb, IcebergDetector, LiquiditySweep, BidAskBounceRate, MidPriceVelocity, BestLevelVolatility, LevelReplenishRate, BookChurnRate, BookDepthChange, ClQueueImb) that need delta-frequency events.
- `ObDeltaPoint`: fixed 808 B record (mirrors `ObSnapshotPoint` layout — top 25 changes per side). `(price, 0.0)` distinguishes "remove level" from zero-padded tail by only treating `(0.0, 0.0)` as padding.
- New persistence toggle `PersistenceConfig::orderbook_deltas` (default false; `on()` enables). Slug = `"orderbook_deltas"` for path layout.
- 2 new round-trip unit tests in `tests/data_point_round_trip.rs` (with-removal + empty).
- Pass-rate impact for MLI validator: 470/577 → ~481/577 (+11 indicators).

MLI Ask 2 (subscription_label) and Ask 3 (deribit_pick_atm_option) rejected — see `docs/plans/mli-asks-decision.md` for rationale (both belong to the MLI app layer, not the connector library).



**0.3.8 (2026-05-22)** — `SubscriptionSet::add_raw` passthrough for exotic instrument IDs:

- New `SubscriptionSet::add_raw(exchange, symbol, account, streams)`. The symbol is passed through to the WS connector verbatim — no `parse_symbol`, no `SymbolNormalizer::to_exchange`. Use for instrument IDs that don't fit the canonical BASE-QUOTE shape: Deribit options (`"BTC-23MAY26-86000-C"`), dated futures (`"BTCUSDT_240329"`), index symbols (`".DEFI"`, `"BTC-PERPETUAL"`).
- Existing `.add()` is unchanged — still canonical-input, still normalizer-translated. `.add` and `.add_raw` can be freely mixed in one set; `Station::subscribe` branches per-entry on the internal `Entry.is_raw` flag.
- For raw entries the internal canonical `Symbol` is built as `Symbol::with_raw("", "", raw)` (empty base/quote, raw = passthrough). Connector-side `SymbolInput::resolve` reads the raw field, so the wire format is whatever the caller passed.
- 2 new unit tests in `tests/subscribe_report.rs` (`add_raw` accepts Deribit option ID; `.add` + `.add_raw` mix in same set).
- Closes MLI ask 2026-05-22 — unblocks Deribit option subscriptions (OptionGreeks, VolatilityIndex, BlockTrade, IndexPrice) which all 4 require exotic instrument IDs the normalizer cannot translate.



**0.3.7 (2026-05-22)** — fail-closed subscribe + heal kline-only (MLI 0.3.6 OOM fix):

- `Station::subscribe(set) -> Result<SubscribeReport>`: continue-on-error. Per-stream failures (NotSupported, transport, normalize) are collected in `report.failed: Vec<FailedStream>`; `report.handle` always present for the streams that did succeed. Batch-level errors (empty set) still return `Err`.
- New `StationError::StreamNotSupported(String)` variant with `.is_not_supported()` helper. Maps both `WebSocketError::NotSupported` and `WebSocketError::UnsupportedOperation` from `transport.rs::subscribe` (which now eagerly propagates EVERY frame-construction error before queuing the cmd — previously only `NotSupported` was eager, `UnsupportedOperation` slipped through and triggered the bug).
- `acquire_or_spawn` returns `Err` BEFORE spawning the forwarder or registering the mux entry when subscribe fails. No more dead forwarders looping on heal/resub forever.
- `spawn_forwarder` heal/resub is now **kline-family only** (`Kline | MarkPriceKline | IndexPriceKline | PremiumIndexKline`). Non-kline kinds on disconnect log INFO + exit the forwarder cleanly; the transport-level auto-reconnect inside `UniversalWsTransport` is enough for those (REST cannot bridge trade/OB/ticker/mark/funding/OI/liq gaps anyway).
- On forwarder exit (kline silence-no-recovery or non-kline disconnect), mux entry is removed from `inner.muxes` if no consumers remain, so re-subscribe for the same key spawns a fresh forwarder.
- Explicit `drop(stream)` before `ws.event_stream()` re-attach on kline heal — guarantees the old BroadcastStream receiver releases before the new one subscribes.
- 4 new unit tests in `tests/subscribe_report.rs` (error variant identity, public field shape, empty-set error). 2 new live tests in `tests/subscribe_not_supported_live.rs` (Bybit MarketWarning → `failed`, mixed Trade + MarketWarning → ok=1 / failed=1, neither leaks a mux entry). Live tests gated `--ignored`; both pass in 210ms against real Bybit.
- MLI workaround `DIG3_WS_SILENCE_SECS=999999` no longer needed — 36 NotSupported subscribes on the 53-stream validator now return cleanly in `report.failed` instead of leaking until OOM.



**0.3.6 (2026-05-22)** — fixed-header + companion blob-file persistence for the 4 string-bearing types:

- `DataPoint` trait gains opt-in `encode_blob() -> Option<Vec<u8>>`, `decode_blob(header, blob) -> Option<Self>`, `blob_pointer_offset() -> Option<usize>` with `None` defaults. 23 numeric types inherit defaults and behave unchanged.
- `DiskStore<T>` opens a companion `.blob` file alongside `.dat`+`.idx` ONLY when `T::blob_pointer_offset()` is `Some(_)`. Write order: blob bytes first, header (with patched `(offset:u64, len:u32)` tail) second. Read tail opens `.blob` read-only and reconstructs string fields via `decode_blob`. Bounds check on read: out-of-range blob pointers logged + skipped.
- Layouts: `BlockTradePoint` 44 B (32 + 12 tail), `AuctionEventPoint` 36 B (24 + 12), `MarketWarningPoint` 20 B (8 + 12), `OrderbookL3Point` 44 B (32 + 12). Blob format: u16-length-prefix per string, UTF-8 bytes, no framing across records.
- `persistence::is_enabled_for` no longer special-cases the 4 string types — they persist normally when global `enabled = true`.
- 9 new integration tests in `tests/blob_persistence.rs` covering ASCII / UTF-8 / empty / 1KiB strings, full append+read_tail round-trip for BlockTrade + MarketWarning, regression that `TradePoint` does NOT create `.blob` files.



**0.3.4 (2026-05-21)** — typed StreamEvent API, structured routing keys, cleanup of empty-symbol holes:

- Every public-data `StreamEvent` variant is struct-style with explicit `symbol: String` + (klines) `interval: KlineInterval`. No tuple variants for public events. No empty-string placeholders in production code (1 doc comment exception).
- `Ticker`, `PublicTrade`, `FundingRate`, `MarkPrice`, `OpenInterest` payload structs no longer carry redundant `symbol: String`. Routing key is on the variant; payload is statistics only.
- `Order.symbol: Option<String>` — cancel/amend responses without symbol payload model `None` explicitly.
- `OrderUpdateEvent` / `PositionUpdateEvent` payload structs no longer carry `symbol`. `StreamEvent::OrderUpdate { symbol, event }` / `PositionUpdate { symbol, event }` are struct variants (BalanceUpdate stays tuple — wallet-wide).
- `MarketWarning.symbol: Option<String>` — `None` for venue-wide notifications.
- `Canonicalize` for payload structs replaced by `pub(crate)` free fns taking `symbol: String` as parameter. Public path is `StreamEvent::canonicalize()` only.
- TF newtype: `KlineInterval` propagated through `Stream::Kline(KlineInterval)`, `Kind::Kline(KlineInterval)`, `Event::Bar { timeframe: KlineInterval }`, `StreamEvent::Kline { interval: KlineInterval }` (and 3 kline-family variants), `CanonicalKline.interval: KlineInterval`. REST `get_klines(interval: &str, ...)` left as exchange-native `&str`.
- `station::event_matches_key` reads `symbol` from every public variant uniformly (was accept-all for OB before — closed cross-symbol pollination).
- `Event.symbol` is per-handle: relay task overwrites with user-input format so two handles with different inputs (`"BTC-USDT"` vs `"BTCUSDT"`) each see their own label.
- Bug fixes shipped together: GateIO `parse_kline` triple-bug (mainstream symbol/interval/mark-premium branches), Bitstamp OB symbol extraction from channel, Polygon/CryptoCompare/Alpaca empty interval, dydx signing key length validation.

**0.3.5 (2026-05-22)** — 18 additional `Stream`/`Kind`/`Event` variants exposed in `digdigdig3-station` for MLI consumer:

- Numeric (11): `IndexPrice`, `CompositeIndex`, `OptionGreeks`, `VolatilityIndex`, `HistoricalVolatility`, `Basis`, `InsuranceFund`, `SettlementEvent`, `PredictedFunding`, `FundingSettlement`, `RiskLimit`.
- Kline-family (3): `MarkPriceKline(KlineInterval)`, `IndexPriceKline(KlineInterval)`, `PremiumIndexKline(KlineInterval)`.
- String-bearing (4): `BlockTrade`, `AuctionEvent`, `MarketWarning`, `OrderbookL3` — initially stubbed `RECORD_SIZE=8` with persistence disabled; **real disk persistence shipped in 0.3.6** via header + companion `.blob` file. See 0.3.6 entry above.

## Commit chain since 0.3.3 (12 commits → 0.3.4, +2 → 0.3.5)

| Commit | Tag | What |
|---|---|---|
| `bbcdb2b` | | refactor: typed StreamEvent + drop redundant symbol from Ticker/PublicTrade |
| `60d40d3` | | test: dual-symbol routing live regression (Binance) |
| `4744f43` | | fix: Event.symbol per-handle (relay overwrites label) |
| `45eb36c` | | fix: Bitstamp live_orders OB symbol from channel name |
| `2935b93` | | fix: GateIO parse_kline triple-bug |
| `a71d949` | | fix: Polygon/CryptoCompare/Alpaca emit interval (not empty) |
| `b2fad9b` | | fix: normalization propagates symbol+interval into CanonicalEvent |
| `64471a5` | | fix: dydx signing key length validation (closes baseline test) |
| `a1e3e5f` | | refactor: drop symbol from FundingRate/MarkPrice/OpenInterest payloads |
| `beac7ed` | | refactor: MarketWarning Option + OrderUpdate struct variant + OB re-wrap cleanup |
| `15098c5` | | refactor: Order.symbol Option + Canonicalize → pub(crate) fns |
| `da7029b` | | refactor: KlineInterval newtype across Station + StreamEvent |
| `295752b` | v0.3.4 | release |
| `c326732` | v0.3.5 | feat: 18 additional Station Stream/Event variants for MLI |
| `027b5dd` | | docs(claude): summarize 0.3.4 + 0.3.5 refactor scope + commit chain |
| `6731d0e` | v0.3.6 | feat(station): fixed-header + companion `.blob` file persistence for 4 string-bearing types |
| `8251ad0` | | docs(claude): record v0.3.6 in commit chain + link release report |
| `ffe9c54` | v0.3.7 | fix(station): fail-closed subscribe + heal kline-only (MLI 0.3.6 OOM fix) |
| `d438a45` | v0.3.8 | feat(station): `SubscriptionSet::add_raw` passthrough for exotic instrument IDs |
| (next) | v0.3.9 | feat(station): `Stream::OrderbookDelta` + `ObDeltaPoint` (MLI Ask 1) |

Test baseline: 830/0 core, 75/0 station + 4 ignored live integration tests (`dual_symbol_routing`, `label_per_subscriber`). Strict (`RUSTFLAGS=-D warnings`) clean.

Workspace follows the uzor pattern: every sub-crate's `[package]` block uses `version.workspace = true`, `[workspace.dependencies]` declares `digdigdig3 = { workspace = true }` etc. — all three crates ALWAYS publish together at the same pin.

**Phase 1 + 2 complete**:
- `dig3 watch <kind> <exchange> <symbol>` works for all 9 data classes.
- Persistence: `<storage_root>/<kind>/<exchange>/<account>/<symbol>/<YYYY-MM-DD>.dat` (binary, fixed record per class) + sparse `.idx`. Day rotation.
- Warm-start: emit last-N from disk (or REST backfill if disk empty) before live.
- Multiplex: per-`SeriesKey` broadcast, ref-counted consumer drop.
- **Auto-heal on WS disconnect** (kline only, since trade/OB/etc. have no public REST endpoint to bridge live gaps). Three disconnect triggers (silence_timeout / stream_ended / stream_err), each runs full cycle: REST `get_klines` → `upsert_by_ts` (last-write-wins overwrite of broken in-flight bars) → `unsubscribe` + `subscribe` (force fresh sub state at exchange) → re-attach `event_stream()`. Pattern mirrors `mylittlechart::live_data::ws_manager`. Silence threshold default 60 s, env `DIG3_WS_SILENCE_SECS`.

Storage root resolves: `--storage-root` flag > `DIG3_STORAGE_ROOT` env > `./dig3_storage`. Harness artefacts (e2e_smoke JSON, WS frame trace) default to `target/harness_out/` when `--json-out auto` / `DIG3_WS_TRACE=1`.

## RAW core / normalized station boundary (2026-06-04)

**Invariant: dig3-core is RAW + most-complete ALWAYS. All normalization,
active-only filtering, and asset clustering live in `digdigdig3-station`
(opt-in). The raw↔non-raw boundary is localized ONLY in station.** The
connector folder (`cex/`/`dex/`/`gated/`) is OUR storage convenience and does
NOT imply a ticker's asset class — that's per-`SymbolInfo` exchange data.

- **`get_exchange_info` is RAW**: every connector returns ALL symbols (incl
  pre-launch / delisting / expired / settled / halted) with the venue-NATIVE
  `status` string verbatim. The old `status != "TRADING"` filters were removed
  everywhere (commits fbfab2b/5e48fe4/e23c634). Consumers that want a live-only
  universe call `digdigdig3_station::active_only(...)`.
- **`SymbolInfo` carries native fields + raw passthrough**: `instrument_type:
  Option<String>` (native token — okx instType, deribit kind, bybit contractType,
  alpaca `class`, tinkoff instrumentKind, ...) + `extra: serde_json::Value` (the
  full native symbol record, nothing lost). The typed fields are a convenience
  subset; `extra` is the guarantee. `#[derive(Default)]` — new construction sites
  may `..Default::default()`.
- **Station normalized-mode** (`station/src/normalize.rs`): `SymbolStatus` enum +
  `canonical_status()` (maps native tokens → Trading/Halted/PreLaunch/Closed/
  Unknown) + `active_only()` filter. Opt-in; never mutates the raw SymbolInfo.
- **Pragmatic exception (stays in core)**: lossless deterministic unit
  canonicalization — sec→ms timestamp `*1000`. Test for "may stay in core":
  reversible + lossless ⇒ core; lossy/inventive/restrictive ⇒ station.
- **Still in core, SHOULD move to station** (deferred, lower priority): the dead
  `Canonicalize`/`CanonicalEvent` machinery in `core/normalization.rs` (dead
  code, only tests/examples use it); HEURISTIC fixups — `now_ms()` stamping when
  the wire has no timestamp + Gemini synthetic ticker + CryptoCompare interval-
  multiplier loss (these INVENT data → belong in a station repair layer; the
  move is invasive — changes event timestamps — so it's a separate pass).
- Live proof: `examples/symbolinfo_raw_e2e.rs` — 7 venues, all return raw native
  status + native instrument_type + populated `extra` (Deribit 5164 = full
  un-filtered options universe).

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

Must run in parallel: `tokio::spawn` per exchange + `join_all`, never sequential. One hang must not stall the harness. Each exchange task capped at 90s (raised from 60s in Wave 10 for Bybit liq multi-symbol parallel).

WS budget per stream (`run_ws_sub` in `e2e_smoke::market`):
- Ticker / Trade / Orderbook / Kline / MarkPrice / FundingRate / AggTrade → 10s
- OpenInterest → 20s
- Liquidation → 30s (Bybit: 5 parallel symbols × 60s budget each, capped by 90s exchange wall)

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

### In scope (`digdigdig3`)
- L3-open crypto (CEX + DEX + Polymarket) — primary consumer surface
- Public market data (klines/ticker/orderbook/trades/funding/OI/liquidation/aggTrade) over REST + WS
- Trading + Account + Positions traits per exchange (gated by API keys)
- Capability discovery + empirical validation
- `ExchangeHub` as single consumer-facing API
- **Validated subset**: 18 TRUSTED connectors (full futures, mark/funding/OI/liquidation/aggTrade verified). See above for the list and which ones are wire-not-present.
- **L1/L2-paid + L3-gated** (~16 exchanges): compile-validated only; functional validation runs only when ENV creds populated (use `e2e_smoke --trading`).

### In scope (`digdigdig3-station`)
- `storage::*` — `StorageManager`, `EventLog`, `StreamKey`, `StorageConfig` (binary append-only event log per stream, day rotation, retention)
- `orderbook::*` — `OrderBookTracker` with delta-merge + gap detection
- `rest_cache::*` — generic LRU+TTL cache (used by `cache::ticker_cache` and future symbol-info caches)
- `replay::*` — `ReplayHub`, `ReplayConfig`, `ReplayRate` (historical replay of event log)
- `cure::*` — `IntegrityChecker`, `Deduper`, `GapDetector`, `RepairPipeline` (post-capture cleanup tools)
- `persistence::TradeWriter` (Phase 1 step 4 — fixed 41-byte records + sparse `.idx`)
- `cache::*` — Station-facing typed cache helpers (e.g. `ticker_cache`)
- `Station` builder + `SubscriptionSet` + `SubscriptionHandle` (Phase 1 step 6)

### Out of scope (deferred to other crates / future)
- On-chain monitoring → `dig2chain`
- High-frequency execution paths beyond current trait surface
- Per-exchange UI / dashboard (consumer = `mylittlechart`)
- Symbol normalization INSIDE connectors (use external `SymbolNormalizer` utility)
- Legacy `base_websocket.rs` and old bespoke WS loops — replaced by `UniversalWsTransport`
- Anything storage/replay/cure-related belongs in `digdigdig3-station`, NEVER in `digdigdig3` (the core crate). Core is pure transport + connectors.

### Known state (after Waves 1-10, 2026-05-20)

**TRUSTED 18 stable**: Binance, BingX, Bitfinex, Bitget, Bitstamp, Bybit, Coinbase, CryptoCom, Deribit, Dydx, GateIO, HTX, HyperLiquid, Kraken, KuCoin, Lighter, MEXC, OKX. All major centralized crypto + 4 DEX (Dydx, HyperLiquid, Lighter, plus CryptoCom CEX). Full futures coverage (mark/funding/OI/liquidation/aggTrade), bid/ask flow populated via primary channel or parallel REST orderbook fetch.

**Outside TRUSTED — wire-not-present (do NOT re-investigate)**:
- **CryptoCompare** — CCCAGG aggregate free tier doesn't expose BID/ASK; verified by live curl. `ob/l1/top` endpoint exists but paid-tier only. Matrix uses `no_bid_ask_by_design(id)` exemption for 13 such data providers.
- **MOEX** — RU IP required for FAST/CEDR streams; geo-locked from non-RU networks. WS Ticker returns `NotSupported` eagerly.
- **Polymarket** — ClobWebSocket not yet implemented; REST partial (price/orderbook use stale token_id discovery).
- **Dukascopy** — tick-data-only architecture; no public live REST endpoints. Documented `NotSupported`.
- **Auth-gated venues** (Alpaca, AngelOne, Coinglass, Dhan, Finnhub, Futu, Fyers, Ib, JQuants, Krx, Oanda, Polygon, Tiingo, Tinkoff, Twelvedata, Upstox, Zerodha) — correctly skip when ENV creds absent. Run `--trading` with creds to validate.

### Closed regressions (do NOT re-investigate)

The 10-wave debug journey closed every detectable bug. Highlights of root causes that should not be guessed at again:

- **Binance `!forceOrder@arr` parser** (Wave 6, commit `95dcf92`) — `parse_force_order_arr` was wrong: `data` is a single event object, not array. Old code did `data.as_array()` → None fallback that wrapped event as `{"o": item}`, so parser read `o.s` on the outer object (no `s` key). Plus side mapping was inverted (`SELL → Buy`). Fixed: delegate to `parse_force_order(raw)`; map `BUY → Buy`. Prefer `o.z` (accumulated filled) over `o.q` for quantity.
- **Bitstamp double-connect** (Wave 8, commit `af42698`) — `e2e_smoke::collect_ws_stream` called `ws.connect()` again after `hub.connect_websocket()` already connected, orphaning the first broadcast channel. Fixed with idempotency guard in `BitstampWebSocket::connect()` — early return if status already `Connected`.
- **Lighter parser key** (Wave 8) — `parse_trade` read `frame.get("trade")` (singular). Actual key is `"trades"` (plural array). Lighter BTC market has 266 trades/min — NOT market-quiet. Parser now returns `Vec<StreamEvent>` iterating the array.
- **Dydx subscribe race** (Wave 8) — sub insert happened AFTER frame send. v4_markets snapshot ACK arrived in milliseconds with empty symbol. Fixed by inserting `request.clone()` BEFORE send, rolling back on wire-send failure.
- **OKX kline on /ws/v5/business** (Wave 4, commit `e5dfb34`) — OKX V5 split channels disjoint between `/ws/v5/public` (tickers/marks/funding/OI/trades/books/liq) and `/ws/v5/business` (candle*, mark-price-candle*, index-candle*) on 2023-06-20. `OkxWebSocket` now holds TWO `UniversalWsTransport` instances; `is_business_kind()` routes by StreamKind; events merged via `futures_util::stream::select`.
- **Bybit WS_liq genuinely sparse** (Wave 9-10) — `examples/bybit_liq_raw.rs` raw `tokio-tungstenite` test bypassed our entire pipeline. 1-hour capture confirmed Bybit V5 `allLiquidation.{symbol}` channel works; BTCUSDT 29 liq/hr (1 per ~2.1 min), 5 symbols total 51 liq/hr. Channel correct, parser correct. Matrix uses 5 parallel symbols × 60s window for ~75-80% hit probability.
- **5s window was too short for low-freq streams** (Wave 5, commit `d590fab`) — fixed budget mapping: Trade/Ticker/Orderbook/Kline/MarkPrice/FundingRate/AggTrade → 10s, OpenInterest → 20s, Liquidation → 30s. Per-exchange wall-time cap 60s → 90s (Wave 10) to fit Bybit 5×60s parallel.

### Validation tooling (in `examples/`)

- `e2e_smoke.rs` — full 47-exchange coverage matrix
- `liq_capture.rs` — multi-exchange liquidation feed validator (`--exchanges X,Y --symbol BTC-USDT --duration 7200`)
- `bybit_liq_raw.rs` — raw tokio-tungstenite test bypassing our transport (proves channel works regardless of our code)
- `bitstamp_trade_capture.rs` — 60s trade capture harness (used to find double-connect bug)
- `feed_demo.rs` — early MarketFeed (will be superseded by Station in Phase 1)

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

## Workspace layout (Phase 1 — done)

Designed in `docs/plans/station-architecture.md`, plan in `docs/plans/station-phase-1-plan.md`, handoff snapshot in `docs/plans/handoff-2026-05-20.md`.

```
digdigdig3/
├── crates/dig3-core/      ← Layer 1: ExchangeHub + raw connectors ONLY
│   ├── src/core/{types,traits,utils,http,websocket,chain,macros,normalization}/
│   ├── src/connector_manager/   (ExchangeHub + pools + factory)
│   ├── src/l{1,2,3}/             (connector implementations)
│   └── examples/                  (e2e_smoke, exchange_hub_demo, etc.)
├── crates/dig3-station/   ← Layer 2: persistence/cache/replay/cure/OB/builder
│   ├── src/storage/        (moved from core in Wave 11C)
│   ├── src/orderbook.rs    (moved from core)
│   ├── src/rest_cache.rs   (moved from core)
│   ├── src/replay/         (moved from core)
│   ├── src/cure/           (moved from core)
│   ├── src/persistence.rs  (TradeWriter — Phase 1 step 4)
│   ├── src/cache.rs        (Station-typed cache helpers)
│   ├── src/{station,builder,subscription,error}.rs
│   └── tests/, examples/   (storage_manager, event_log, cure, replay, etc.)
└── crates/dig3-cli/       ← Layer 3: `dig3` binary + dig3-catcher + dig3-cure bins
    └── src/{main.rs,bin/{dig3_catcher,dig3_cure}.rs}
```

Sample usage:

```rust
let station = Station::builder()
    .storage_root("./dig3_storage")
    .persistence(PersistenceConfig::on())
    .build().await?;

let handle = station.subscribe(
    SubscriptionSet::new()
        .add(ExchangeId::Binance, "BTC-USDT", AccountType::Spot, [Stream::Trade])
).await?;
```

Phase 1 steps done: 1 (split) + 4 (TradeWriter) + 5 (RestCache wrapper) + 6 (subscribe wiring). Step 7 (`dig3 watch trades` end-to-end) operational since commit `2fde113`.

MLC reference architecture explored. Strong patterns borrowed (SharedMap dual-read, deferred-unsub, sequenced REST→WS, three-level bar loader). Weak patterns fixed (try_send drop, dual independent refcount drift, ad-hoc subscribe_X methods).

**Not migrating MLC** — it stays pinned on 0.1.32 and we don't owe it API compatibility.

## File pointers

- Architecture entry: `src/connector_manager/hub.rs` (ExchangeHub)
- WS framework: `src/core/websocket/{transport.rs, protocol.rs, topic_registry.rs, stream_kind.rs}`
- Trait composition: `src/core/traits/mod.rs`
- Capability struct: `src/core/types/capabilities.rs`
- Reference WS migration: `src/l3/open/crypto/cex/bitget/{protocol.rs, websocket.rs}`
- Validation harness: `examples/e2e_smoke.rs` + `examples/exchange_hub_demo.rs`
- Plans: `docs/plans/wave0-foundation.md`, `docs/plans/smoke_v8_findings_spec.md`, `docs/plans/ws-rest-inventory.md`
- Release reports: `docs/plans/release-0.3.6-report.md` (blob persistence), `docs/plans/release-0.3.7-plan.md` (subscribe fail-closed + heal kline-only)
- Persistence layout (0.3.6+): `docs/plans/fixed-header-blob-persistence.md`
- MLI feedback log: `docs/plans/mli-0.3.6-findings.md` (motivated 0.3.7), `docs/plans/mli-station-asks.md` (motivated 0.3.9), `docs/plans/mli-asks-decision.md` (rationale for accept/reject)

## Gotchas

- Cargo.toml is v0.2.2 (v0.2.3 anticipated post-θ.6 bump). README.md matches. Trust CLAUDE.md and code for architecture facts.
- Windows codepage: prefix Windows-native commands with `chcp.com 65001 > $null 2>&1;` for UTF-8.
- NEVER chain git commands with `&&`. Separate `git add` / `git commit` calls.
- digdigdig3 is a git submodule with its own `.git`. `cd digdigdig3` before any git command.
- Do NOT bump version unless explicitly asked. When asked to bump, the DEFAULT
  is ALWAYS a patch bump (`x.x.+1`) — for any change, including public-API /
  core-type / behavior breaks. A minor bump (`x.+1.x`) is done ONLY by agreement
  with the user / on their explicit demand — never decide a minor yourself.
- Do NOT push to remote unless explicitly asked.
