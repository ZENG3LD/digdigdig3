# digdigdig3 — Testing Plan (post-coverage-sweep)

Status snapshot: commit `ecb0ed5` (2026-05-19). After the research-driven sweep,
~30 missing REST endpoints are implemented. Next phases target WS coverage,
trading paths, and stability.

## Where we are

- **Unit tests**: 778 PASS, 0 FAIL, 0 ignored. 0 warnings under `-D warnings`.
- **Live e2e_smoke matrix**: REST coverage is ~10–11 of 13 methods OK on each
  major CEX. WS coverage is uneven — Spot streams largely work; **all futures
  streams (Mark/Funding/Liq/OI/AggTrade) are ERR because e2e_smoke routes them
  through Spot account_type, which lands on the wrong WS endpoint**.
- **TRUSTED set** (REST+WS both fully populated, zero issues): Bitstamp,
  Binance, Coinbase, OKX, Kraken, GateIO, Bitget, KuCoin, Gemini, Bitfinex,
  CryptoCom, Deribit, Lighter — moving target as fixes land.

## Phase 0 — finished (this commit)

Reference. Do not redo.

- coverage matrix in `examples/e2e_smoke.rs` (CLI: `--market`, `--trading`, `--exchange`, `--json-out`)
- `WebSocketError::NotSupported(String)` + eager-return in `transport.rs`
- `ExchangeError::UnsupportedOperation` (TODO) vs `NotSupported` (wire-not-present) convention
- All 30+ TODO_Unsupported REST endpoints implemented from live docs

## Phase 1 — WS futures account_type routing (HIGHEST PRIORITY, next session)

**Goal**: every futures-only WS stream (Mark/Funding/Liq/OI/AggTrade) connects
to the correct exchange endpoint and the matrix shows OK for each supported
cell.

**Root cause**: `examples/e2e_smoke.rs` subscribes all WS streams with
`AccountType::Spot`. Bigger CEX (Binance, Bybit, OKX, …) host futures market
data on a **separate WS URL** (e.g. `wss://fstream.binance.com/ws`,
`wss://stream.bybit.com/v5/public/linear`). The `hub.connect_websocket()` call
must select that URL based on the **account_type** the caller passes.

### Tasks

1. In `examples/e2e_smoke.rs`, for the WS test block, branch by stream type:
   ```rust
   let account_for_stream = match stream_kind {
       StreamKind::Ticker | StreamKind::Trade | StreamKind::Orderbook | StreamKind::Kline(_)
           => AccountType::Spot,
       StreamKind::MarkPrice | StreamKind::FundingRate | StreamKind::Liquidation
       | StreamKind::OpenInterest | StreamKind::AggTrade
           => AccountType::FuturesCross,
   };
   hub.connect_websocket(id, account_for_stream, false).await?;
   let ws = hub.ws(id, account_for_stream)?;
   ```
   This means each exchange needs **two** WS connections in the matrix run —
   Spot + Futures — but they fire in parallel inside `tokio::join!` so
   wall-time stays at ~25s.

2. Verify per-exchange WS impls accept `AccountType::FuturesCross` and route
   to the futures URL. The connectors with separate spot/futures WS are:
   - Binance (`stream.binance.com:9443` vs `fstream.binance.com`)
   - Bybit (`/v5/public/spot` vs `/v5/public/linear`)
   - GateIO (`api.gateio.ws/ws/v4` vs `fx-ws.gateio.ws/v4/ws/usdt`)
   - HTX (`api.huobi.pro/ws` vs `api.hbdm.com/linear-swap-ws`)
   - MEXC (`wbs-api.mexc.com/ws` vs `contract.mexc.com/edge`)
   - KuCoin (`ws-api-spot.kucoin.com` vs `ws-api-futures.kucoin.com`)
   - OKX uses one endpoint for both — only the instId differs (`BTC-USDT` vs `BTC-USDT-SWAP`)
   - Deribit, BingX, Bitget, CryptoCom — single endpoint, but channel names differ

3. For exchanges where the public liquidation feed does not exist (CryptoCom,
   Deribit, HyperLiquid, KuCoin, MEXC) — they already return
   `WebSocketError::NotSupported` from `subscribe_frame`; the matrix will tag
   them `--` rather than `ERR`.

### Pass criteria

- `cargo run --example e2e_smoke --release` shows **OK** for MarkPrice / FundingRate / OpenInterest / AggTrade on the 8 main CEX (Binance, BingX, Bitget, Bybit, GateIO, HTX, KuCoin, OKX).
- Liquidation cells are **OK** on Binance/Bybit/GateIO/HTX/OKX/BingX, **NotSupported** (`--`) on CryptoCom/Deribit/HyperLiquid/KuCoin/MEXC.
- No regression in Spot WS streams.

## Phase 2 — Bybit / Binance WS bid/ask + Orderbook empty-snapshot

**Bybit WS Ticker** currently surfaces events but `bid_price`/`ask_price = None`.
Spec confirms `tickers.{sym}` carries `bid1Price`/`ask1Price`. Most likely the
parser reads the wrong field path; one focused diff in
`src/l3/open/crypto/cex/bybit/protocol.rs::parse_ticker`.

**Binance/MEXC/HTX/Bitget Orderbook snapshot** sub returns events but parser
drops them because the payload has no `s`/`symbol` field — the symbol comes
from the channel name (`btcusdt@depth20@100ms`). Fix is **in transport**: the
`UniversalWsTransport` dispatcher must carry the subscription's resolved
symbol forward to the parser when the payload omits it.

Concrete change site: `src/core/websocket/transport.rs::dispatch_frame()` —
when topic_registry resolves to a `(StreamKind::Orderbook, account)`, attach
the original `StreamSpec.symbol` from the subscriptions map. Parser then
populates `OrderBook.symbol` from that context.

## Phase 3 — dYdX WS dispatch

`v4_orderbook` returns events but they decode into `OrderbookDelta` with empty
arrays. dYdX Indexer WS uses `subscribed` (initial snapshot) + `channel_data`
(delta) — both must be parsed. Right now we only handle one type.

Same module: `src/l3/open/crypto/dex/dydx/websocket.rs`. Add explicit branches
for `type: "subscribed"` (`contents` = `OrdersInitialMessage` snapshot) and
`type: "channel_data"` (`contents` = `OrdersUpdateMessage` delta). Emit
`StreamEvent::OrderbookSnapshot` for the former and `OrderbookDelta` for the
latter.

## Phase 4 — Trading matrix (with ENV credentials)

`e2e_smoke --trading` already reads ENV. The implementation calls only
**read-only** trading methods:

- `get_balances`
- `get_account_info`
- `get_open_orders`
- `get_user_trades` (recent fills)
- `get_positions` (futures)
- `get_fees`

It must **never** call `place_order` / `cancel_order` / `transfer` / `withdraw`
— those are excluded by design (matrix only validates connectivity + parser,
not destructive paths).

ENV vars per exchange follow the convention `{EXCHANGE_UPPER}_API_KEY` +
`{EXCHANGE_UPPER}_API_SECRET` (+ `_PASSPHRASE` where applicable for OKX,
KuCoin, Crypto.com). Trading section is `Skipped` for any exchange where
required ENV is absent.

Pass criteria: on any exchange where credentials are present, all 6 read-only
methods return OK with non-empty/non-default fields.

## Phase 5 — MOEX, KRX, Polymarket cleanup

- MOEX WS bid/ask = None outside RU — needs IP-based skip or a clear `Skipped("requires RU ingress")` in the matrix when run from non-RU.
- KRX Data Marketplace path works for `get_klines`. `get_ticker` derives from latest kline. Verify `get_price` / `get_recent_trades` paths return real Korean stock data when symbol is a 6-digit code like `005930` (Samsung). For now the matrix targets BTC and KRX is auto-skipped for that symbol.
- Polymarket — `discover_active_token_id` works for REST `get_ticker`/`get_orderbook`. WS `Polymarket ClobWebSocket does not impl` — implement basic public market-channel subscribe (no auth required).

## Phase 6 — Stability / soak

- Run `e2e_smoke` 10× in a row, no flakes.
- Memory-bound test: 24h run of MarketFeed (TBD) subscribing to 20 exchanges × 3 streams; RSS should stabilise.
- Reconnect: kill a WS connection, verify auto-reconnect + auto-resubscribe within 5s.
- Rate limits: each REST endpoint should respect the exchange's quota; run a 1000-rps test against ourselves and watch for 429s + correct backoff.

## How an agent should pick up from here

1. `git log --oneline -20` in digdigdig3 to see fresh history. Look for `ecb0ed5` (sweep) and downstream commits.
2. `cd digdigdig3 && cargo test --lib` — must be 778/0/0 (or higher). If lower, regression has landed since.
3. `cargo run --example e2e_smoke --release > /tmp/e2e.txt 2>&1` then `grep -A 50 'MARKET COVERAGE MATRIX' /tmp/e2e.txt`. The matrix is the source of truth for what is broken.
4. Pick the topmost unfinished phase from this file. Don't start Phase 2 if Phase 1 hasn't landed — futures-routing fixes shift the matrix substantially.

## Operating principles (stored here so they don't get lost in CLAUDE.md churn)

- **Every Unsupported needs a research-agent before code.** No more guessing endpoint URLs or channel names. The 6 research-agents in this sweep cost ~30 min total in wall-time (parallel) and saved hours of debug.
- **Don't deflect with doc-comments**. If a field is `None`, find out why with live docs and fix the wire path. The previous "doc-comment closes the issue" approach left ~10 bid/ask None cells unaddressed across the codebase.
- **Differentiate `UnsupportedOperation` vs `NotSupported` always.** First is a TODO, second is reality. Audits depend on this.
- **Parallel test harnesses.** All e2e_smoke per-exchange work must run as `tokio::spawn` + `join_all` — one hang must not block the others.
- **`NoSupported` reasons cite the alternative.** "Use REST endpoint X" or "WS-only via channel Y" — never a bare "not supported".
