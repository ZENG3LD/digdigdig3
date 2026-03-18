# digdigdig3 Roadmap

Checkbox status tracks real validation against live data or real accounts — not just compile-time success.

---

## Phase 0: Precision Guard (DONE)

f64 → Decimal conversion at Trading trait boundary. Prevents IEEE-754 drift from corrupting order prices.

- [x] Research: f64 accumulation errors, CCXT approach, sub-tick drift analysis
- [x] Core functions: `safe_price()` (round), `safe_qty()` (floor) in `core/utils/precision.rs`
- [x] `PrecisionCache`: thread-safe per-symbol tick/step cache with RwLock
- [x] `SymbolInfo.tick_size` field added
- [x] 23 parsers extract real tick_size from exchange APIs
- [x] PrecisionCache wired into all 19 CEX connectors (place_order, amend_order, batch_orders)
- [x] 17 unit tests for precision functions + cache
- [x] Broker/DEX connectors: Paradex, OANDA, Alpaca, Zerodha wired to PrecisionCache
- [x] DEX with own precision systems: dYdX (MarketConfigCache), GMX (U256 on-chain), Lighter ({:.8} fixed)
- [x] Stubs skipped: Jupiter, Raydium (place_order → UnsupportedOperation)

## Phase 0.5: DEX/Swap Connector Gaps (DONE)

Complete unfinished DEX/swap connectors — wire existing endpoints, fix bugs, implement missing swap APIs.
Trading trait is mandatory (CoreConnector supertrait) — AMMs return UnsupportedOperation by design.
Real swap execution lives in struct methods + optional on-chain features.

### GMX (DONE)
- [x] Fix `get_funding_rate` — was wrongly stubbed, now parses `/markets/info` fundingFactorPerSecond
- [x] Implement `get_positions` via Subsquid GraphQL
- [x] Implement `get_open_orders` via Subsquid GraphQL
- [x] Implement `get_order_history` via Subsquid GraphQL
- [x] Implement `get_order` via Subsquid single-item lookup
- [x] Wire ERC-20 `balanceOf` into `get_balance` via EvmProvider
- [x] Fix ExchangeRouter address (0x7C68→0x87d6)

### Jupiter (DONE)
- [x] Fix Ultra Swap bug: correct HTTP method (GET) + requestId wiring
- [x] Update deprecated balances → `/holdings/{address}` endpoint
- [x] Wire holdings into Account trait `get_balance`
- [x] Implement Trigger API (limit orders): create, cancel, bulk cancel, query
- [x] Implement Recurring API (DCA orders): create, cancel, query
- [x] Add token search/trending/category/recent wrapper methods

### Raydium (DONE)
- [x] Implement `get_swap_quote()` wrapper over SwapQuoteBaseIn/Out
- [x] Implement `build_swap_transaction()` wrapper over SwapQuoteBaseIn/Out
- [x] Fix WebSocket: dynamic pool lookup via REST fallback + cache
- [x] Fix WebSocket race condition: broadcast channel init in new()
- [x] Add 8 missing wrappers: token list, farm, pool, RPCs, priority fees

### Uniswap (DONE)
- [x] Wire `POST /swap` → `parse_swap_transaction()` (typed SwapTransaction struct)
- [x] Fix WebSocket prices: decimal scaling (raw wei → human-readable)
- [x] Fix `get_token_balance()`: dynamic decimals query
- [x] Fix `volume_24h` in ticker: now queries poolDayDatas
- [x] Surface TVL from pool queries
- [x] Wire `POST /approval` endpoint

### dYdX (DONE)
- [x] Wire TakeProfit conditional orders via ConditionalPlan + TriggerDirection
- [x] Wire long-term orders via TimeInForce::Gtd → build_place_long_term_order_tx
- [x] Implement cancel_all_orders helper (serial cancel with sequence tracking)

### Lighter (DONE)
- [x] Research ECgFp5+Poseidon2 from Go/TS SDKs + crate security audit
- [x] Port complete crypto stack to Rust (2850 lines, zero third-party crates)
  - Goldilocks GF(p), GFp5 quintic extension, ECgFp5 curve, Poseidon2, Schnorr
  - Ported from official `elliottech/poseidon_crypto` + `pornin/ecgfp5` reference
- [x] Wire into auth.rs: real signing replaces broken k256 stubs
- [x] Fix `get_order`: now searches active + inactive orders
- [x] Wire auth token into authenticated read endpoints

---

## Phase 0.9: Fix Broken Testnet Flags (DONE)

Testnet pipeline fix: `Credentials.testnet` → `ConnectorFactory` → `TestHarness` + fix 9 broken connectors.

**Pipeline plumbing (commit 6c61d70):**
- [x] Added `testnet: bool` to `Credentials` struct + `with_testnet()` builder
- [x] `ConnectorFactory::create_public(id, testnet)` — passes testnet to all Pattern A connectors
- [x] `ConnectorFactory::create_authenticated` — extracts `credentials.testnet` and forwards
- [x] `TestHarness::create_public(id, testnet)` — updated signature
- [x] `env_loader` — reads `{EXCHANGE}_TESTNET=true|1` from `.env`

**Connector fixes:**
- [x] **BingX** — stored testnet flag, documented VST pair routing (no separate URLs)
- [x] **Bitget** — stores testnet, selects TESTNET URLs, injects `X-CHANNEL-API-CODE: paptrading` header
- [x] **Bitfinex** — stores testnet, documented paper-trading via TEST-prefixed symbols
- [x] **HTX** — documented no public testnet, flag stored for future use
- [x] **Bithumb** — returns `UnsupportedOperation` when `testnet=true` (no testnet exists)
- [x] **Angel One** — returns `UnsupportedOperation` when `testnet=true` (no sandbox)
- [x] **Dhan** — documented: sandbox is token-based, not URL-based (correct by design)
- [x] **IB** — added `paper()` constructor + `with_testnet()` builder (port 4004)
- [x] **DefiLlama** — `is_testnet()` now returns stored value instead of hardcoded `false`

**Still open (not blocking Phase 1):**
- [ ] **Binance** — Verify testnet URL `testapi.binance.vision` vs `testnet.binance.vision/api` (may already be correct — both may work)
- [ ] **GateIO** — Verify `api-testnet.gateapi.io` domain still active after `.io` → `.ws` migration

---

## Phase 0.95: Implement `get_user_trades` (DONE)

All 24 connectors now implement `Trading::get_user_trades` — fills/trades history
is available for dig3x3 to wrap as a thin parser layer.

### CEX Major (7)
- [x] **Binance** — `GET /api/v3/myTrades` (spot) + `GET /fapi/v1/userTrades` (futures)
- [x] **Bybit** — `GET /v5/execution/list` (unified, cursor-based)
- [x] **OKX** — `GET /api/v5/trade/fills` (3d) + `GET /api/v5/trade/fills-history` (3mo)
- [x] **KuCoin** — `GET /api/v1/fills` (spot) + `GET /api/v1/futures/fills` (futures)
- [x] **Kraken** — `POST /0/private/TradesHistory` (offset-based)
- [x] **Coinbase** — `GET /api/v3/brokerage/orders/historical/fills` (cursor-based)
- [x] **Gate.io** — `GET /api/v4/spot/my_trades` + `GET /api/v4/futures/{settle}/my_trades`

### CEX Secondary (11)
- [x] **BingX** — `GET /openApi/swap/v2/trade/fillHistory`
- [x] **Bitget** — `GET /api/v2/spot/trade/fills` + `GET /api/v2/mix/order/fills`
- [x] **Bitfinex** — `POST /v2/auth/r/trades/hist`
- [x] **HTX** — `GET /v1/order/matchresults` (spot)
- [x] **MEXC** — `GET /api/v3/myTrades`
- [x] **Phemex** — `GET /api-data/g-orders/tradeHistory`
- [x] **Crypto.com** — `POST private/get-trades`
- [x] **Upbit** — extract from order details (requires order_id)
- [x] **Deribit** — `GET /api/v2/private/get_user_trades_by_currency`
- [x] **Bitstamp** — `POST /api/v2/user_transactions/`
- [x] **Gemini** — `POST /v1/mytrades`

### DEX (5)
- [x] **HyperLiquid** — `POST /info` action=`userFills` (no auth, address-based)
- [x] **dYdX v4** — `GET /v4/fills` (indexer, address-based)
- [x] **Lighter** — `GET /api/v1/trades` (auth token required)
- [x] **Paradex** — `GET /v1/fills` (JWT+StarkKey)
- [x] **GMX** — Subsquid GraphQL `tradeActions` query (address-based)

### Stocks/Brokers (1)
- [x] **Alpaca** — `GET /v2/account/activities/FILL` (API key header)

---

## Phase 0.96: Implement `FundingHistory` + `AccountLedger` traits (DONE)

Two new traits added to core — funding rate payment history and full account ledger/transaction log.
Researched per-exchange endpoints individually, not copy-pasted from Binance.

### Core types added
- `FundingPayment` — symbol, funding_rate, position_size, payment, asset, timestamp
- `FundingFilter` — symbol, start_time, end_time, limit
- `LedgerEntry` — id, asset, amount, balance, entry_type, description, ref_id, timestamp
- `LedgerEntryType` — Trade, Deposit, Withdrawal, Funding, Fee, Rebate, Transfer, Liquidation, Settlement, Other
- `LedgerFilter` — asset, entry_type, start_time, end_time, limit

### FundingHistory implementations (12 connectors)
- [x] **Binance** — `GET /fapi/v1/income` (type=FUNDING)
- [x] **Bybit** — `GET /v5/account/transaction-log` (type=SETTLEMENT)
- [x] **OKX** — `GET /api/v5/account/bills` (instType=SWAP, type=8)
- [x] **KuCoin** — `GET /api/v1/funding-history`
- [x] **Kraken** — `POST /0/private/Ledgers` (type=rollover)
- [x] **Gate.io** — `GET /api/v4/futures/{settle}/funding_payments`
- [x] **Bitfinex** — `POST /v2/auth/r/ledgers/hist` (category=28 funding)
- [x] **Deribit** — `GET /api/v2/private/get_transaction_log` (type=delivery)
- [x] **Phemex** — funding-fees endpoint
- [x] **HyperLiquid** — `POST /info` action=`userFunding`
- [x] **dYdX v4** — `GET /v4/historicalFunding`
- [x] **Paradex** — `GET /v1/funding-payments`

### AccountLedger implementations (12 connectors)
- [x] **Binance** — `GET /fapi/v1/income` (all types)
- [x] **Bybit** — `GET /v5/account/transaction-log`
- [x] **OKX** — `GET /api/v5/account/bills` + `GET /api/v5/account/bills-archive`
- [x] **KuCoin** — `GET /api/v1/accounts/ledgers`
- [x] **Kraken** — `POST /0/private/Ledgers`
- [x] **Gate.io** — `GET /api/v4/wallet/ledger`
- [x] **Bitfinex** — `POST /v2/auth/r/ledgers/hist`
- [x] **Deribit** — `GET /api/v2/private/get_transaction_log`
- [x] **Bitget** — `GET /api/v2/spot/account/bills`
- [x] **Bitstamp** — `POST /api/v2/user_transactions/`
- [x] **Crypto.com** — `POST private/get-transactions`
- [x] **Alpaca** — `GET /v2/account/activities`

### Not applicable (default UnsupportedOperation)
- GMX, Lighter — no funding history endpoint (on-chain settlement only)
- BingX, HTX, MEXC, Upbit, Coinbase, Gemini — no ledger/funding endpoint in public API
- Tinkoff, OANDA, IB, Dukascopy — broker connectors, deferred

---

## Phase 1: Crypto CEX/DEX Execution Testing

Testing order placement, cancellation, account queries on real exchanges.
Organized by testnet availability — test the free ones first.

---

### Phase 1A: Free Testnet Connectors (zero cost, test everything)

Full trading simulation available. No real money required.

**Binance** (spot: `testnet.binance.vision/api`, futures: `testnet.binancefuture.com`)
Keys: GitHub OAuth at testnet.binance.vision — instant, no Binance account needed.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures — testnet.binancefuture.com)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates (`wss://testnet.binance.vision/stream`)
- [ ] WS: private balance updates

**Bybit** (`api-testnet.bybit.com`, WS: `wss://stream-testnet.bybit.com`)
Keys: email signup at testnet.bybit.com → request test coins (10,000 USDT + 1 BTC, 24h limit).
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (perps/futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**OKX** (same URL `okx.com` + header `x-simulated-trading: 1`)
Keys: OKX account → Demo Trading → Personal Center → Demo Trading API.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures/options)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**KuCoin** (`openapi-sandbox.kucoin.com`, WS: `wss://ws-api-sandbox.kucoin.com`)
Keys: separate account at sandbox.kucoin.com — virtual BTC/ETH/KCS issued on registration.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**Kraken** (futures only: `demo-futures.kraken.com`)
Keys: any email/password at demo-futures.kraken.com — no email verification, $50K virtual USD.
Note: Kraken spot has no testnet — spot trading test requires real account (Phase 1B).
- [ ] REST: place_order futures (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates (`wss://demo-futures.kraken.com/ws/v1`)
- [ ] WS: private balance updates

**Gemini** (`api.sandbox.gemini.com`, WS: `wss://api.sandbox.gemini.com`)
Keys: register at exchange.sandbox.gemini.com — auto-verified with test funds. 2FA bypass: header `GEMINI-SANDBOX-2FA: 9999999`.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**Bitstamp** (`sandbox.bitstamp.net/api/v2/`)
Keys: existing Bitstamp account → Settings → API Access → New API Key (note: may require live account with KYC first).
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: amend_order
- [ ] WS: private order updates
- [ ] WS: private balance updates

**Deribit** (`test.deribit.com/api/v2`, WS: `wss://test.deribit.com/ws/api/v2`)
Keys: register at test.deribit.com — no email verification, no KYC, virtual funds auto-credited.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (options/futures/perps)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**Phemex** (`testnet-api.phemex.com`, WS: `wss://testnet.phemex.com/ws`)
Keys: register at testnet.phemex.com — 0.5 BTC virtual on registration.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (perps)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**Bitget** (demo: `api.bitget.com` + header `paptrading: 1`, WS: `wspap.bitget.com`)
Keys: Bitget account → Demo mode → Personal Center → API Key Management → Create Demo API Key.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (futures)
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**BingX** (VST virtual pairs on live endpoint: `BTC-VST`, `ETH-VST`)
Keys: BingX account (free signup) — new accounts auto-receive 100,000 VST.
- [ ] REST: place_order (LIMIT, MARKET) on VST pairs
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance (VST balance)
- [ ] REST: get_positions
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] REST: batch_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

**dYdX v4** (testnet indexer: `indexer.v4testnet.dydx.exchange/v4`, WS: `wss://indexer.v4testnet.dydx.exchange/v4/ws`)
Keys: Cosmos wallet + faucet at faucet.v4testnet.dydx.exchange (300 USDC Dv4TNT drip, no mainnet deposit needed).
- [ ] REST/gRPC: place_order (LIMIT, MARKET)
- [ ] REST/gRPC: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions (perps)
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates

**Lighter** (`testnet.zklighter.elliot.ai`, WS: `wss://testnet.zklighter.elliot.ai/stream`)
Keys: account via testnet.app.lighter.xyz + test funds via Lighter Discord (no automated faucet as of 2026).
- [ ] place_order_signed() — ZK-native signed order (internal method)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions
- [ ] Expose place_order_signed() through standard Trading trait

**Paradex** (testnet: `api.testnet.paradex.trade/v1`, WS: `wss://ws.api.testnet.paradex.trade/v1`)
Keys: StarkNet wallet + testnet USDC via Paradex Discord #developers. Testnet launched March 2025.
- [ ] REST: place_order (StarkNet signing)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions

**OANDA** (practice: `api-fxpractice.oanda.com`, streaming: `stream-fxpractice.oanda.com`)
Keys: free registration at oanda.com → My Account → My Services → Manage API Access. No credit card. Rate limit: 120 req/s.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: get_positions
- [ ] REST: amend_order
- [ ] WS: streaming prices
- [ ] WS: streaming account updates

**Alpaca** (`paper-api.alpaca.markets`, WS: `wss://stream.data.alpaca.markets`)
Keys: signup at app.alpaca.markets (email only) → Paper Trading → API Keys. Default constructor already uses paper URL.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order
- [ ] REST: get_open_orders
- [ ] REST: get_order_history
- [ ] REST: get_balance
- [ ] REST: amend_order
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates
- [ ] WS: private balance updates

---

### Phase 1B: Free Data Only (no testnet trading, validate market data)

No free trading simulation available. Market data endpoints are public and free.
Test public REST + WebSocket data feeds here. Trading requires real accounts (deferred to Phase 1C or later).

**Coinbase** (public market data only — no testnet for Advanced Trade)
Note: `api-public.sandbox.exchange.coinbase.com` exists (Exchange sandbox, static responses). Not yet wired.
- [ ] REST: get_klines — validate on real data
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] WS: subscribe_klines
- [ ] WS: subscribe_orderbook

**Gate.io** (futures testnet only: `fx-api-testnet.gateio.ws/api/v4` — verify domain after gateio.ws migration)
- [ ] REST: get_klines (spot)
- [ ] REST: get_ticker (spot)
- [ ] REST: get_orderbook (spot)
- [ ] REST: place_order futures (testnet)
- [ ] REST: get_balance (testnet)
- [ ] WS: subscribe_orderbook

**HTX** (no testnet — validate public market data only)
- [ ] REST: get_klines — validate on real data
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] WS: subscribe_klines
- [ ] WS: subscribe_orderbook

**MEXC** (public API only — no spot API sandbox, futures institutional-only)
- [ ] REST: get_klines — validate on real data
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] WS: subscribe_klines
- [ ] WS: subscribe_orderbook

**Upbit** (full public Quotation API — no auth required)
- [ ] REST: get_klines — validate on real data
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] WS: subscribe_orderbook

**Bitfinex** (public data + 2 paper trading pairs: `tTESTBTC:TESTUSD`, `tTESTBTC:TESTUSDT`)
- [ ] REST: get_klines — validate on real data
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] REST: place_order on paper pairs (TESTBTCTESTUSD)
- [ ] REST: get_balance (paper sub-account)
- [ ] WS: subscribe_orderbook

**Crypto.com** (public market data; UAT sandbox is institutional-only)
- [ ] REST: get_klines — validate on real data
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] WS: subscribe_klines
- [ ] WS: subscribe_orderbook

**HyperLiquid** (testnet exists but requires $5 mainnet deposit for faucet)
Testnet: `api.hyperliquid-testnet.xyz`. Third-party faucets (Chainstack, QuickNode) may bypass deposit requirement.
- [ ] REST: get_klines — validate on real data (public, no auth)
- [ ] REST: get_ticker
- [ ] REST: get_orderbook
- [ ] WS: subscribe_orderbook
- [ ] (defer trading to when faucet is resolved)

**Free data-only providers** (no testnet needed — validate public endpoints)
- [ ] Finnhub: REST get_ticker, get_klines with free API key (60 calls/min)
- [ ] Polygon: REST get_klines, get_ticker with free API key (5 calls/min)
- [ ] FRED: REST economic series query with free API key (120 req/min)
- [ ] DefiLlama: REST get_protocols, get_pools (no auth at all)
- [ ] MOEX ISS: REST get_ticker, get_klines (no auth at all)
- [ ] JQuants: REST historical stock prices (12-week delay free tier)
- [ ] KRX: REST KOSPI/KOSDAQ market data

---

### Phase 1C: Paid / Real Account Required (deferred)

These require either a real brokerage account (KYC) or paid API access. Deferred until accounts are obtained.

**Indian brokers** — all require Indian KYC and real brokerage account
- [ ] Zerodha (Kite): connect with OAuth token, validate get_open_orders
- [ ] Upstox: connect with OAuth token (sandbox exists Jan 2025 but orders-only, no market data)
- [ ] Angel One: connect with JWT + TOTP, validate get_balance (no testnet — returns UnsupportedOperation when testnet=true)
- [ ] Fyers: connect with JWT, validate get_order_history (paid API Bridge for paper trading)
- [ ] Dhan: sandbox available — moved to Phase 1A since developer.dhanhq.co requires no brokerage account

**Tinkoff/T-Bank** — requires Russian T-Bank account (full gRPC sandbox once account open)
- [ ] place_order (virtual rubles)
- [ ] get_balance, get_positions on sandbox account

**GMX v2** — Arbitrum Sepolia testnet not actively maintained; use real ETH for proper testing
- [ ] Wire EvmProvider to connector
- [ ] REST: place_order (requires EVM wallet)
- [ ] REST: cancel_order
- [ ] REST: get_positions
- [ ] REST: get_balance

**Coinglass** — paid API from $29/month
- [ ] liquidations endpoint
- [ ] open interest endpoint
- [ ] funding rates endpoint

---

## Phase 1.5: AnyConnector Private Trait Delegation (dig3x3 prerequisite)

Wire remaining traits onto `AnyConnector` dispatch enum:
- [ ] `Trading` — `get_user_trades()`, `get_order_history()`, `get_open_orders()`, `get_order()`
- [ ] `Account` — `get_balance()`, `get_account_info()`, `get_fees()`
- [ ] `Positions` — `get_positions()`, `get_funding_rate()`, `get_closed_pnl()`, `get_funding_rate_history()`
- [ ] `CustodialFunds` — `get_funds_history()`, `get_deposit_address()`, `withdraw()`
- [ ] `AccountTransfers` — `transfer()`, `get_transfer_history()`

These are already implemented on individual connectors — just need match-arm delegation in AnyConnector similar to how MarketData is done.

**Blocked by**: Nothing (pure mechanical work)
**Blocks**: dig3x3 integration (can use `Arc<dyn Trading>` directly via AnyConnector downcast)

---

## Phase 1.6: Generic Test Harness Validation

Test harness code already exists in `src/testing/` (~2664 lines across market_data, trading, account, positions suites). This phase is about actually running it with real testnet keys.

- [ ] Set up `.env` with testnet API keys for Phase 1A exchanges (Binance, Bybit, OKX, KuCoin, Deribit, Phemex, Gemini, Bitstamp, Kraken, OANDA, Alpaca, dYdX, Lighter, Paradex, Bitget, BingX)
- [ ] Run market_data suite against all Phase 1A exchanges (public endpoints — should work without auth keys)
- [ ] Run trading suite against Phase 1A exchanges with testnet keys (place/cancel/amend orders)
- [ ] Run account suite against Phase 1A exchanges (balance, order history)
- [ ] Run positions suite against Phase 1A futures exchanges (Binance futures, Bybit perps, OKX futures, Kraken futures, Deribit, Phemex, dYdX, Lighter, Paradex)
- [ ] Fix any parser bugs discovered during harness runs
- [ ] Run market_data suite against Phase 1B exchanges (HTX, MEXC, Upbit, Coinbase, Gate.io, HyperLiquid public)

---

## Phase 2: Crypto DataFeed Gaps

Methods that exist but were never validated on real data.

### All CEX (19 active connectors)
- [ ] REST: get_orderbook — validate for all 19 connectors
- [ ] REST: get_recent_trades — implement for the 18 missing, then validate all
- [ ] WS: subscribe_orderbook — validate for all 19 connectors
- [ ] WS: subscribe_klines — validate where supported

### REST ticker (all CEX except Bitstamp and Gemini)
- [ ] Binance: REST get_ticker
- [ ] Bybit: REST get_ticker
- [ ] OKX: REST get_ticker
- [ ] KuCoin: REST get_ticker
- [ ] Kraken: REST get_ticker
- [ ] Coinbase: REST get_ticker
- [ ] Gate.io: REST get_ticker
- [ ] Bitfinex: REST get_ticker
- [ ] MEXC: REST get_ticker
- [ ] HTX: REST get_ticker
- [ ] Bitget: REST get_ticker
- [ ] BingX: REST get_ticker
- [ ] Crypto.com: REST get_ticker
- [ ] Upbit: REST get_ticker
- [ ] Deribit: REST get_ticker
- [ ] HyperLiquid: REST get_ticker
- [ ] dYdX v4: REST get_ticker
- [ ] Lighter: REST get_ticker
- [ ] MOEX ISS: REST get_ticker

### DEX data gaps
- [ ] dYdX: REST get_orderbook — validate on real data
- [ ] dYdX: WS subscribe_orderbook (v4_orderbook channel) — validate
- [ ] Lighter: REST get_orderbook — validate on real data
- [ ] GMX: get_orderbook — impossible (oracle pricing, no orderbook by design)
- [ ] Jupiter: get_klines — impossible (no historical OHLCV API, need Birdeye/CoinGecko)
- [ ] Jupiter: get_orderbook — impossible (aggregator, no native book)

---

## Phase 3: On-Chain Provider Testing

Test chain providers against live nodes. None of this has been exercised.

### EVM (Ethereum and compatible chains)
- [ ] Connect to Ethereum mainnet RPC (QuickNode, Infura, or public)
- [ ] get_height — latest block number
- [ ] get_native_balance — ETH balance for address
- [ ] erc20_balance() — ERC-20 token balance via eth_call
- [ ] get_logs — Transfer event filter over a block range
- [ ] EvmDecoder: decode_block() on a real block (ERC-20 transfers, Uniswap swaps)
- [ ] Log subscription via WebSocket provider

### Bitcoin
- [ ] Connect to Bitcoin RPC (QuickNode or public Electrum-compatible)
- [ ] get_height — current chain tip
- [ ] get_raw_mempool — unconfirmed tx list
- [ ] BitcoinDecoder: decode_block() on a real block (UTXO analysis, coinbase detection)

### Solana
- [ ] Connect to mainnet-beta RPC
- [ ] get_height — current slot
- [ ] get_native_balance — SOL balance for address
- [ ] SolanaDecoder: decode_transaction() on a real tx (SPL transfers, Raydium swap)
- [ ] Account subscription via WebSocket

### Cosmos (Osmosis as test target)
- [ ] Connect to Osmosis LCD endpoint
- [ ] get_all_balances — token balances for address
- [ ] get_pools — liquidity pool list
- [ ] CosmosDecoder: decode_tx_events() on a real tx (IBC packet, governance vote)
- [ ] Tendermint WebSocket subscription

### Aptos
- [ ] Connect to Aptos mainnet REST API
- [ ] get_height — current ledger version
- [ ] get_native_balance — APT balance
- [ ] Module event subscription
- [ ] Coin transfer decode

### StarkNet
- [ ] Connect to StarkNet mainnet RPC
- [ ] get_height — latest block
- [ ] Contract call via starknet_call
- [ ] Event monitoring for a known contract

### Sui
- [ ] Connect to Sui mainnet RPC
- [ ] get_height — latest checkpoint
- [ ] get_native_balance — SUI balance
- [ ] Move event subscription
- [ ] SuiDecoder: DeepBook event decode

### TON
- [ ] Connect to TON mainnet (toncenter or lite-server)
- [ ] get_height — masterchain seqno
- [ ] Jetton transfer detection
- [ ] DEX op-code decode from a real transaction

---

## Phase 4: Authenticated DataFeed Testing

Lower priority — providers requiring real accounts or payment that weren't covered in Phase 1C.

### Stock Brokers (real accounts required)
- [ ] Interactive Brokers: paper trading — IBKR Lite account (KYC, no minimum) + local TWS/Gateway running
- [ ] Futu: connect with OpenD daemon running locally, validate get_balance (Futu ID required)
- [ ] Tinkoff: sandbox after T-Bank account obtained — place sandbox order, get_balance, get_positions

### Intelligence Feeds
- [ ] Etherscan: get_transactions and get_token_transfers with free API key (api-sepolia.etherscan.io for testnet)
- [ ] CryptoCompare: multi-exchange price aggregation with free API key (100K calls/month)
- [ ] Messari: crypto prices and metrics with free API key (20 req/min)

---

## Phase 5: Extended Connectors

New connectors and trait completions.

### Missing CEX
- [ ] AscendEX
- [ ] BitMart
- [ ] CoinEx
- [ ] WOO X
- [ ] XT.com
- [ ] LBank
- [ ] HashKey
- [ ] WhiteBIT
- [ ] BTSE
- [ ] BigONE
- [ ] ProBit

### Jupiter (complete implementation)
- [ ] Implement get_klines (currently stub returning UnsupportedOperation)
- [ ] Implement get_orderbook (currently stub)
- [ ] Implement place_order through standard Trading trait
- [ ] Wire SolanaProvider for transaction submission

### EventProducer trait implementations
- [ ] EVM: implement EventProducer — emit OnChainEvent from log subscriptions
- [ ] Solana: implement EventProducer — emit OnChainEvent from account/tx subscriptions
- [ ] Bitcoin: implement EventProducer — emit OnChainEvent from block scanning
- [ ] Cosmos: implement EventProducer — emit OnChainEvent from Tendermint events

### Optional Trait Overrides (per-connector explicit impl blocks)
- [ ] MarginTrading: Binance, Bybit, OKX (all support margin borrow/repay)
- [ ] EarnStaking: Binance, Bybit, OKX (earn/flexible savings products)
- [ ] ConvertSwap: Binance, Bybit (dust conversion, instant swap)
- [ ] VaultManager: GMX, HyperLiquid, Paradex (vault deposit/withdraw)
- [ ] LiquidityProvider: Jupiter, Raydium (LP position management)
- [ ] StakingDelegation: CosmosProvider-backed connectors (dYdX, Osmosis)
- [ ] TriggerOrders: Binance, Bybit, OKX (TP/SL conditional orders)
- [ ] MarketMakerProtection: Binance, Bybit, Deribit (MMP config, mass quoting)
- [ ] BlockTradeOtc: Deribit (OTC block trade creation)
- [ ] CopyTrading: Bybit, Bitget, OKX (follow/unfollow traders)

### Infrastructure
- [ ] Interactive Brokers: wire proper brokerage execution (currently aggregator/Web API mode only)
- [ ] MOEX ISS: verify WebSocket stream works (currently listed as untested on WS)
- [ ] India broker WebSocket: Zerodha, Upstox, Angel One, Fyers — add WS implementations
