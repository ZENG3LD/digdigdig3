# digdigdig3 Roadmap

Checkbox status tracks real validation against live data or real accounts — not just compile-time success.

---

## What's Done This Session

- L2 orderbook research for all L3/open connectors (18 CEX + 3 DEX + Polymarket)
- `OrderbookCapabilities` implemented and wired for all L3/open connectors
- WS integration tests written for all 18 L3/open CEX connectors
- 14 of 18 pass live; 4 blocked by geo-restriction (Upbit, Bitfinex, Bitstamp, Crypto.com)
- READMEs written for all L3/open connectors
- Full codebase restructured to l1/l2/l3 + open/gated layout

---

## Phase 0: Precision Guard (DONE)

f64 to Decimal conversion at Trading trait boundary. Prevents IEEE-754 drift from corrupting order prices.

- [x] Research: f64 accumulation errors, CCXT approach, sub-tick drift analysis
- [x] Core functions: `safe_price()` (round), `safe_qty()` (floor) in `core/utils/precision.rs`
- [x] `PrecisionCache`: thread-safe per-symbol tick/step cache with RwLock
- [x] `SymbolInfo.tick_size` field added
- [x] 23 parsers extract real tick_size from exchange APIs
- [x] PrecisionCache wired into all 18 CEX connectors (place_order, amend_order, batch_orders)
- [x] 17 unit tests for precision functions + cache
- [x] Broker/DEX connectors: Paradex, OANDA, Alpaca, Zerodha wired to PrecisionCache
- [x] DEX with own precision systems: dYdX (MarketConfigCache), Lighter ({:.8} fixed)
- [x] Paradex: wired to PrecisionCache

## Phase 0.5: DEX Connector Gaps (DONE)

Complete unfinished DEX connectors — wire existing endpoints, fix bugs, implement missing APIs.

### dYdX (DONE)
- [x] Wire TakeProfit conditional orders via ConditionalPlan + TriggerDirection
- [x] Wire long-term orders via TimeInForce::Gtd to build_place_long_term_order_tx
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

Testnet pipeline fix: `Credentials.testnet` to `ConnectorFactory` to `TestHarness` + fix 9 broken connectors.

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
- [x] **Angel One** — returns `UnsupportedOperation` when `testnet=true` (no sandbox)
- [x] **Dhan** — documented: sandbox is token-based, not URL-based (correct by design)
- [x] **IB** — added `paper()` constructor + `with_testnet()` builder (port 4004)
- [x] **DefiLlama** — `is_testnet()` now returns stored value instead of hardcoded `false`

**Still open (not blocking Phase 1):**
- [ ] **Binance** — Verify testnet URL `testapi.binance.vision` vs `testnet.binance.vision/api` (may already be correct — both may work)
- [ ] **GateIO** — Verify `api-testnet.gateapi.io` domain still active after `.io` to `.ws` migration

---

## Phase 0.95: Implement `get_user_trades` (DONE)

All 24 connectors now implement `Trading::get_user_trades`.

### CEX Major (7)
- [x] **Binance** — `GET /api/v3/myTrades` (spot) + `GET /fapi/v1/userTrades` (futures)
- [x] **Bybit** — `GET /v5/execution/list` (unified, cursor-based)
- [x] **OKX** — `GET /api/v5/trade/fills` (3d) + `GET /api/v5/trade/fills-history` (3mo)
- [x] **KuCoin** — `GET /api/v1/fills` (spot) + `GET /api/v1/futures/fills` (futures)
- [x] **Kraken** — `POST /0/private/TradesHistory` (offset-based)
- [x] **Coinbase** — `GET /api/v3/brokerage/orders/historical/fills` (cursor-based)
- [x] **Gate.io** — `GET /api/v4/spot/my_trades` + `GET /api/v4/futures/{settle}/my_trades`

### CEX Secondary (9)
- [x] **BingX** — `GET /openApi/swap/v2/trade/fillHistory`
- [x] **Bitget** — `GET /api/v2/spot/trade/fills` + `GET /api/v2/mix/order/fills`
- [x] **Bitfinex** — `POST /v2/auth/r/trades/hist`
- [x] **HTX** — `GET /v1/order/matchresults` (spot)
- [x] **MEXC** — `GET /api/v3/myTrades`
- [x] **Crypto.com** — `POST private/get-trades`
- [x] **Upbit** — extract from order details (requires order_id)
- [x] **Deribit** — `GET /api/v2/private/get_user_trades_by_currency`
- [x] **Bitstamp** — `POST /api/v2/user_transactions/`
- [x] **Gemini** — `POST /v1/mytrades`

### DEX (4)
- [x] **HyperLiquid** — `POST /info` action=`userFills` (no auth, address-based)
- [x] **dYdX v4** — `GET /v4/fills` (indexer, address-based)
- [x] **Lighter** — `GET /api/v1/trades` (auth token required)
- [x] **Paradex** — `GET /v1/fills` (JWT+StarkKey)

### Stocks/Brokers (1)
- [x] **Alpaca** — `GET /v2/account/activities/FILL` (API key header)

---

## Phase 0.96: Implement `FundingHistory` + `AccountLedger` traits (DONE)

Two new traits added to core — funding rate payment history and full account ledger/transaction log.

### Core types added
- `FundingPayment` — symbol, funding_rate, position_size, payment, asset, timestamp
- `FundingFilter` — symbol, start_time, end_time, limit
- `LedgerEntry` — id, asset, amount, balance, entry_type, description, ref_id, timestamp
- `LedgerEntryType` — Trade, Deposit, Withdrawal, Funding, Fee, Rebate, Transfer, Liquidation, Settlement, Other
- `LedgerFilter` — asset, entry_type, start_time, end_time, limit

### FundingHistory implementations (11 connectors)
- [x] **Binance** — `GET /fapi/v1/income` (type=FUNDING)
- [x] **Bybit** — `GET /v5/account/transaction-log` (type=SETTLEMENT)
- [x] **OKX** — `GET /api/v5/account/bills` (instType=SWAP, type=8)
- [x] **KuCoin** — `GET /api/v1/funding-history`
- [x] **Kraken** — `POST /0/private/Ledgers` (type=rollover)
- [x] **Gate.io** — `GET /api/v4/futures/{settle}/funding_payments`
- [x] **Bitfinex** — `POST /v2/auth/r/ledgers/hist` (category=28 funding)
- [x] **Deribit** — `GET /api/v2/private/get_transaction_log` (type=delivery)
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
- Lighter — no funding history endpoint (on-chain settlement only)
- BingX, HTX, MEXC, Upbit, Coinbase, Gemini — no ledger/funding endpoint in public API
- Tinkoff, OANDA, IB, Dukascopy — broker connectors, deferred

---

## Phase 1A: L3/open CEX — WebSocket Orderbook Tests (IN PROGRESS)

18 CEX connectors in `src/l3/open/crypto/cex/`. All have `OrderbookCapabilities` implemented.
WS integration tests written and run against live feeds.

Status: 14 pass, 4 geo-blocked.

**Passing (14):**
- [x] Binance — WS orderbook live
- [x] Bybit — WS orderbook live
- [x] OKX — WS orderbook live
- [x] KuCoin — WS orderbook live
- [x] Kraken — WS orderbook live
- [x] Coinbase — WS orderbook live
- [x] Gate.io — WS orderbook live
- [x] HTX — WS orderbook live
- [x] MEXC — WS orderbook live
- [x] BingX — WS orderbook live
- [x] Bitget — WS orderbook live
- [x] Deribit — WS orderbook live
- [x] HyperLiquid — WS orderbook live
- [x] Gemini — WS orderbook live

**Geo-blocked (4) — need VPN or proxy to validate:**
- [ ] Upbit — geo-restricted (South Korea)
- [ ] Bitfinex — geo-restricted (US/certain regions)
- [ ] Bitstamp — geo-restricted (certain regions)
- [ ] Crypto.com — geo-restricted (certain regions)

---

## Phase 1B: L3/open DEX — WebSocket Orderbook Tests

3 DEX connectors in `src/l3/open/crypto/dex/`. All have `OrderbookCapabilities` implemented.

- [ ] dYdX v4 — WS orderbook (v4_orderbook channel, indexer feed)
- [ ] Lighter — WS orderbook (testnet.zklighter.elliot.ai/stream)
- [ ] Paradex — WS orderbook (ws.api.testnet.paradex.trade/v1)

Note: Polymarket (`src/l3/open/prediction/`) is REST-only by design — no WS orderbook needed.

---

## Phase 2: L3/open Execution Testing (Testnets)

Order placement, cancellation, account queries on real testnet endpoints.
All connectors in `src/l3/open/`. No real money required for any item in this phase.

### CEX with free testnets

**Binance** (spot: `testnet.binance.vision/api`, futures: `testnet.binancefuture.com`)
Keys: GitHub OAuth at testnet.binance.vision — instant, no Binance account needed.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions (futures)
- [ ] REST: amend_order, cancel_all_orders, batch_orders
- [ ] WS: private order updates, balance updates

**Bybit** (`api-testnet.bybit.com`, WS: `wss://stream-testnet.bybit.com`)
Keys: email signup at testnet.bybit.com, request 10,000 USDT test coins.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions
- [ ] REST: amend_order, cancel_all_orders, batch_orders
- [ ] WS: private order updates, balance updates

**OKX** (same URL + header `x-simulated-trading: 1`)
Keys: OKX account to Demo Trading to Personal Center to Demo Trading API.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions
- [ ] REST: amend_order, cancel_all_orders, batch_orders
- [ ] WS: private order updates, balance updates

**KuCoin** (`openapi-sandbox.kucoin.com`, WS: `wss://ws-api-sandbox.kucoin.com`)
Keys: separate account at sandbox.kucoin.com — virtual BTC/ETH/KCS on registration.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions (futures)
- [ ] REST: amend_order, cancel_all_orders, batch_orders
- [ ] WS: private order updates, balance updates

**Kraken** (futures only: `demo-futures.kraken.com`)
Keys: any email at demo-futures.kraken.com — no email verification, $50K virtual USD.
- [ ] REST: place_order futures (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates (`wss://demo-futures.kraken.com/ws/v1`)

**Gemini** (`api.sandbox.gemini.com`, WS: `wss://api.sandbox.gemini.com`)
Keys: register at exchange.sandbox.gemini.com — auto-verified with test funds. 2FA bypass: header `GEMINI-SANDBOX-2FA: 9999999`.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history, get_balance
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates, balance updates

**Bitstamp** (`sandbox.bitstamp.net/api/v2/`)
Keys: existing Bitstamp account to Settings to API Access (may require KYC first).
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history, get_balance
- [ ] REST: amend_order
- [ ] WS: private order updates, balance updates

**Deribit** (`test.deribit.com/api/v2`, WS: `wss://test.deribit.com/ws/api/v2`)
Keys: register at test.deribit.com — no email verification, no KYC, virtual funds auto-credited.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions (options/futures/perps)
- [ ] REST: amend_order, cancel_all_orders
- [ ] WS: private order updates, balance updates

**Bitget** (demo: `api.bitget.com` + header `paptrading: 1`, WS: `wspap.bitget.com`)
Keys: Bitget account to Demo mode to API Key Management to Create Demo API Key.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions (futures)
- [ ] REST: amend_order, cancel_all_orders, batch_orders
- [ ] WS: private order updates, balance updates

**BingX** (VST virtual pairs: `BTC-VST`, `ETH-VST` on live endpoint)
Keys: BingX account (free signup) — new accounts auto-receive 100,000 VST.
- [ ] REST: place_order on VST pairs (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance (VST), get_positions
- [ ] REST: amend_order, cancel_all_orders, batch_orders
- [ ] WS: private order updates, balance updates

**OANDA** (practice: `api-fxpractice.oanda.com`, streaming: `stream-fxpractice.oanda.com`)
Keys: free registration at oanda.com to My Account to Manage API Access. No credit card.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions
- [ ] REST: amend_order
- [ ] WS: streaming prices, streaming account updates

**Alpaca** (`paper-api.alpaca.markets`, WS: `wss://stream.data.alpaca.markets`)
Keys: signup at app.alpaca.markets (email only) to Paper Trading to API Keys.
- [ ] REST: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history, get_balance
- [ ] REST: amend_order, cancel_all_orders
- [ ] WS: private order updates, balance updates

### DEX with free testnets

**dYdX v4** (testnet: `indexer.v4testnet.dydx.exchange/v4`)
Keys: Cosmos wallet + faucet at faucet.v4testnet.dydx.exchange (300 USDC drip, no mainnet deposit).
- [ ] REST/gRPC: place_order (LIMIT, MARKET)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions
- [ ] REST: cancel_all_orders
- [ ] WS: private order updates

**Lighter** (`testnet.zklighter.elliot.ai`)
Keys: account via testnet.app.lighter.xyz + test funds via Lighter Discord.
- [ ] place_order_signed() via ZK-native signing
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions
- [ ] Expose place_order_signed() through standard Trading trait

**Paradex** (`api.testnet.paradex.trade/v1`)
Keys: StarkNet wallet + testnet USDC via Paradex Discord #developers.
- [ ] REST: place_order (StarkNet signing)
- [ ] REST: cancel_order, get_open_orders, get_order_history
- [ ] REST: get_balance, get_positions

---

## Phase 3: L3/gated Connector Validation (Deferred)

Connectors in `src/l3/gated/` that require real accounts or institutional API access.
Deferred until accounts are obtained.

**Indian brokers** — require Indian KYC and real brokerage account
- [ ] Zerodha (Kite): connect with OAuth token, validate get_open_orders
- [ ] Upstox: connect with OAuth token (sandbox exists but orders-only, no market data)
- [ ] Angel One: connect with JWT + TOTP, validate get_balance (no testnet)
- [ ] Fyers: connect with JWT, validate get_order_history
- [ ] Dhan: sandbox at developer.dhanhq.co — no brokerage account required

**Tinkoff/T-Bank** — requires Russian T-Bank account (full gRPC sandbox once account open)
- [ ] place_order (virtual rubles)
- [ ] get_balance, get_positions on sandbox account

**Coinbase** — public market data works; trading sandbox (`api-public.sandbox.exchange.coinbase.com`) has static responses only
- [ ] REST: get_klines, get_ticker, get_orderbook — validate on real data
- [ ] WS: subscribe_klines, subscribe_orderbook

**Gate.io** — futures testnet only (`fx-api-testnet.gateio.ws/api/v4` — verify domain after gateio.ws migration)
- [ ] REST: get_klines, get_ticker, get_orderbook (spot — real data)
- [ ] REST: place_order futures (testnet)
- [ ] WS: subscribe_orderbook

---

## Phase 4: L2 Connector Validation

Connectors in `src/l2/`. Market data endpoints — no trading.

- [ ] Finnhub: REST get_ticker, get_klines with free API key (60 calls/min)
- [ ] Polygon: REST get_klines, get_ticker with free API key (5 calls/min)
- [ ] FRED: REST economic series query with free API key (120 req/min)
- [ ] DefiLlama: REST get_protocols, get_pools (no auth)
- [ ] MOEX ISS: REST get_ticker, get_klines (no auth)
- [ ] JQuants: REST historical stock prices (12-week delay free tier)
- [ ] KRX: REST KOSPI/KOSDAQ market data

---

## Phase 5: L1 Connector Validation

Connectors in `src/l1/`. On-chain providers — connect to live nodes, validate read endpoints.
On-chain write operations (transaction submission) are handled by dig2chain, not here.

### EVM (Ethereum and compatible chains)
- [ ] Connect to Ethereum mainnet RPC (QuickNode, Infura, or public)
- [ ] get_height — latest block number
- [ ] get_native_balance — ETH balance for address
- [ ] erc20_balance() — ERC-20 token balance via eth_call
- [ ] get_logs — Transfer event filter over a block range
- [ ] EvmDecoder: decode_block() on a real block (ERC-20 transfers, DEX swaps)
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
- [ ] SolanaDecoder: decode_transaction() on a real tx (SPL transfers, DEX swap)
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

## Phase 6: Extended Features

Demand-driven additions. Nothing here is scheduled — items get promoted to earlier phases when needed.

### AnyConnector private trait delegation (dig3x3 prerequisite)

Wire remaining traits onto `AnyConnector` dispatch enum:
- [ ] `Trading` — `get_user_trades()`, `get_order_history()`, `get_open_orders()`, `get_order()`
- [ ] `Account` — `get_balance()`, `get_account_info()`, `get_fees()`
- [ ] `Positions` — `get_positions()`, `get_funding_rate()`, `get_closed_pnl()`, `get_funding_rate_history()`
- [ ] `CustodialFunds` — `get_funds_history()`, `get_deposit_address()`, `withdraw()`
- [ ] `AccountTransfers` — `transfer()`, `get_transfer_history()`

These are already implemented on individual connectors — just need match-arm delegation in AnyConnector similar to how MarketData is done.

### Optional trait overrides (per-connector explicit impl blocks)
- [ ] MarginTrading: Binance, Bybit, OKX (margin borrow/repay)
- [ ] EarnStaking: Binance, Bybit, OKX (earn/flexible savings)
- [ ] ConvertSwap: Binance, Bybit (dust conversion, instant swap)
- [ ] VaultManager: HyperLiquid, Paradex (vault deposit/withdraw)
- [ ] StakingDelegation: CosmosProvider-backed connectors (dYdX, Osmosis)
- [ ] TriggerOrders: Binance, Bybit, OKX (TP/SL conditional orders)
- [ ] MarketMakerProtection: Binance, Bybit, Deribit (MMP config, mass quoting)
- [ ] BlockTradeOtc: Deribit (OTC block trade creation)
- [ ] CopyTrading: Bybit, Bitget, OKX (follow/unfollow traders)

### EventProducer trait implementations
- [ ] EVM: emit OnChainEvent from log subscriptions
- [ ] Solana: emit OnChainEvent from account/tx subscriptions
- [ ] Bitcoin: emit OnChainEvent from block scanning
- [ ] Cosmos: emit OnChainEvent from Tendermint events

### Infrastructure
- [ ] Interactive Brokers: wire proper brokerage execution (currently Web API mode only)
- [ ] MOEX ISS: verify WebSocket stream works (currently listed as untested on WS)
- [ ] India broker WebSocket: Zerodha, Upstox, Angel One, Fyers — add WS implementations
