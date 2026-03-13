# WAVE4 Endpoint Gap Analysis — Consolidated Summary

**Date:** 2026-03-13
**Source batches:** BATCH1–BATCH5, BATCH6_7_8, BATCH9_10, BATCH11_12, BATCH13, BATCH14_19, BATCH20_23, BATCH24_27, BATCH28_30, SWAP_DEX_REPORT
**Scope:** ~90 connectors across CEX, DEX/Swap, Onchain, Stocks/US, Stocks/India, Forex, Aggregators, Crypto intelligence, Economic feeds, Conflict/Humanitarian, Space, Aviation, Maritime, Sanctions/Legal, Corporate/Trade, Demographics/Governance/News

---

## QUICK SUMMARY — TOTAL GAP COUNTS BY CATEGORY

| Category | Approximate Missing Endpoints | Priority |
|----------|-------------------------------|----------|
| Trade/fill history (myTrades, fills, executions) | 30+ across 15 CEX connectors | P0 |
| Market data: recent trades / public fills | 25+ across 15 CEX connectors | P0 |
| Market data: open interest | 15+ across 10 CEX connectors | P1 |
| Market data: funding rate history | 15+ across 10 CEX connectors | P1 |
| Market data: mark price / index price | 15+ across 10 CEX connectors | P1 |
| Batch amend orders | 8 across Binance, Bybit, OKX, Gate.io, HTX | P1 |
| Margin/lending (borrow, repay, interest) | 80+ across 8 CEX connectors | P2 |
| Earn/staking/savings | 40+ across 5 CEX connectors | P2 |
| Convert/swap endpoints | 15+ across Binance, OKX, KuCoin | P2 |
| Copy trading | 30+ across Bitget, BingX, OKX | P2 |
| WebSocket streams (missing channels) | 60+ across 12 connectors | P1 |
| DEX trading stubs (need wallet signing) | 7 connectors (dYdX, GMX, Jupiter, Lighter, Paradex, Raydium, Uniswap) | P1 |
| Intel feed: missing data endpoints | 50+ across 20 data providers | P3 |
| Bugs / wrong paths found | 12 confirmed bugs | P0 |

---

---

## P0: CRITICAL GAPS (Must-have for any real trading or correctness)

### P0.1 — Trade / Fill History (myTrades, fills, executions)

Every real trading system needs to reconcile executed trades. These are **entirely missing** as endpoint variants.

| Connector | Missing Endpoint | Path |
|-----------|-----------------|------|
| Binance Spot | `SpotMyTrades` | `GET /api/v3/myTrades` |
| Binance Futures | `FuturesUserTrades` | `GET /fapi/v1/userTrades` |
| Bybit | `Executions` | `GET /v5/order/execution` |
| OKX Spot | Fill history | `GET /api/v5/trade/fills` |
| OKX Futures | Fill history | `GET /api/v5/trade/fills-history` |
| Kraken | Trade history | `POST /0/private/TradesHistory` |
| Coinbase | Fill history | `GET /api/v3/brokerage/orders/historical/fills` |
| KuCoin Spot | `SpotFills` | `GET /api/v1/fills` |
| KuCoin Futures | `FuturesFills` | `GET /api/v1/fills` |
| HTX | `MatchResults` (by order) | `GET /v1/order/orders/{id}/matchresults` |
| Gate.io Spot | My trades | `GET /spot/my_trades` |
| Gate.io Futures | Trade fills | `GET /futures/usdt/my_trades` |
| MEXC | Trade list | `GET /api/v3/myTrades` |
| BingX Spot | `myTrades` | `GET /openApi/spot/v1/trade/myTrades` |
| BingX Swap | `allFillOrders` | `GET /openApi/swap/v2/trade/allFillOrders` |
| Bitget Futures | `FuturesFillHistory` | `GET /api/v2/mix/order/fill-history` |
| Phemex | Trade history | `GET /exchange/order/v2/tradingList` |
| Upbit | Trade history | `GET /v1/orders/closed` |
| Dhan | Trade history | `GET /v2/trades` |
| Angel One | Trade book | via SmartAPI |
| Alpaca | (covered via `StockTrades` per symbol — P1 for single-symbol variant) | |
| Paradex | Fills | `GET /v1/fills` (partial — not all variants) |

**Total missing: ~25 fill/trade-history endpoints across CEX connectors**

---

### P0.2 — Recent Public Trades / Market Trades

Needed to build real-time tape, confirm last price, feed to indicators.

| Connector | Missing Endpoint | Path |
|-----------|-----------------|------|
| Binance Spot | `SpotRecentTrades` | `GET /api/v3/trades` |
| Binance Spot | `SpotHistoricalTrades` | `GET /api/v3/historicalTrades` |
| Binance Futures | `FuturesRecentTrades` | `GET /fapi/v1/trades` |
| KuCoin Spot | `SpotRecentTrades` | `GET /api/v1/market/histories` |
| KuCoin Futures | `FuturesTradeHistory` | `GET /api/v1/trade/history` |
| Bitget Spot | `SpotRecentFills` | `GET /api/v2/spot/market/fills` |
| Gate.io Spot | `SpotTrades` | `GET /spot/trades` |
| Gate.io Futures | Futures trades | `GET /futures/usdt/trades` |
| MEXC | Recent trades | `GET /api/v3/trades` |

**Total missing: ~15 public trade endpoints**

---

### P0.3 — Confirmed Bugs / Wrong Endpoint Paths

These are bugs in existing `endpoints.rs` files — the URL is wrong and calls will fail or return incorrect data.

| Connector | Bug Type | Detail |
|-----------|----------|--------|
| **Polygon** | Wrong paths (3) | `Dividends` → `/stocks/v1/dividends` should be `/vX/reference/dividends`. `Splits` → wrong path. `FinancialRatios` → wrong path |
| **OpenSky** | Auth broken | Migrated from username/password Basic Auth to OAuth2 client credentials — connector likely non-functional |
| **UCDP** | Wrong endpoint path | `StateConflict` uses `/stateconflict/{version}` — correct path is `/ucdpprioconflict/{version}` |
| **UCDP** | Stale version numbers | Hardcoded `24.1` throughout — current versions are `25.1` (yearly) and `26.0.1` (gedevents) |
| **AviationStack** | HTTP instead of HTTPS | Base URL uses `http://` — should be `https://` |
| **UN OCHA** | Stale API version | Uses v1 base (`/api/v1`) — API migrated to `/api/v2` |
| **Sentinel Hub** | Wrong catalog path | Uses `/api/v1/catalog/search` — correct STAC path is `/api/v1/catalog/1.0.0/` |
| **IMF PortWatch** | Unverified paths | `/portwatch/v1/...` endpoints are inferred — actual ArcGIS REST paths may differ |
| **Binance Futures account** | Stale version | `FuturesAccount` uses `/fapi/v2/account` — v3 is current (`/fapi/v3/account`) |
| **ReliefWeb** | Stale API version | Uses `/v1` — current is `/v2` |
| **Space-Track** | Hardcoded query strings | Architecture issue — all query predicates hardcoded, makes adding new classes impossible without new enum variants |
| **Launch Library 2** | Stale API version + wrong paths | Uses v2.2.0 (current is v2.3.0); `/agency/` should be `/agencies/`, `/space_station/` should be `/spacestation/` |

---

### P0.4 — Blockers (connectors that are likely non-functional)

| Connector | Issue | Severity |
|-----------|-------|----------|
| **OpenSky** | OAuth2 migration — basic auth no longer works | FULL BLOCKER |
| **UN Population** | Core `/data/indicators/{...}/locations/{...}` endpoint missing — connector can list indicators/locations but cannot actually fetch any data | FULL BLOCKER |
| **Bitquery** | WS subscriptions not wired (GraphQL subscriptions written but not connected to `WebSocketConnector`) | PARTIAL BLOCKER |

---

---

## P1: IMPORTANT GAPS (Needed for derivatives trading and real-time monitoring)

### P1.1 — Open Interest

Required for market sentiment, position sizing, and derivatives trading decisions.

| Connector | Missing Endpoint |
|-----------|-----------------|
| Binance Futures | `GET /fapi/v1/openInterest` + `GET /futures/data/openInterestHist` |
| Bybit | `GET /v5/market/open-interest` |
| OKX | Open interest endpoint |
| KuCoin Futures | `GET /api/v1/contracts/risk-limit/{symbol}` (open interest embedded) |
| HTX Futures | `GET /linear-swap-api/v1/swap-open-interest` |
| Bitget Futures | `GET /api/v2/mix/market/open-interest` |
| BingX Swap | `GET /openApi/swap/v2/quote/openInterest` |
| Gate.io Futures | Open interest endpoint |
| Deribit | Open interest per instrument |
| Phemex | Open interest |

**Affected connectors: Binance, Bybit, OKX, KuCoin, HTX, Bitget, BingX, Gate.io, Deribit, Phemex (~10)**

---

### P1.2 — Funding Rate History

Required for backtesting perpetual futures strategies and funding arbitrage.

| Connector | Missing Endpoint |
|-----------|-----------------|
| KuCoin Futures | `GET /api/v1/contract/funding-rates` |
| HTX Futures | `GET /linear-swap-api/v3/swap-funding-rate-history` |
| Bitget Futures | `GET /api/v2/mix/market/history-fund-rate` |
| BingX Swap | `GET /openApi/swap/v2/quote/fundingRateHistory` |
| Gate.io Futures | Funding rate history |
| Deribit | Funding rate history |
| Phemex | Funding rate history |
| Lighter DEX | `GET /api/v1/fundings` (implemented but not wired as trait) |

**Affected connectors: KuCoin, HTX, Bitget, BingX, Gate.io, Deribit, Phemex, Lighter (~8)**

---

### P1.3 — Mark Price / Index Price

Required for accurate PnL calculation, liquidation price estimation, and derivatives risk management.

| Connector | Missing Endpoints |
|-----------|-----------------|
| KuCoin Futures | Mark price, Index price, Premium index (3 endpoints) |
| Binance Futures | `GET /fapi/v1/premiumIndex` (mark price + funding rate combined) |
| Bybit | Mark/index/premium klines (3 kline variants) |
| HTX Futures | `GET /linear-swap-ex/market/index` + mark price klines |
| Bitget Futures | `GET /api/v2/mix/market/symbol-price` (mark + index + last) |
| BingX Swap | `GET /openApi/swap/v2/quote/premiumIndex` |
| Gate.io Futures | Index price |
| MEXC Futures | Mark price |

**Affected connectors: KuCoin, Binance, Bybit, HTX, Bitget, BingX, Gate.io, MEXC (~8)**

---

### P1.4 — Batch Amend Orders

Required for high-frequency market making — amend multiple orders in one API call.

| Connector | Missing Endpoint | Path |
|-----------|-----------------|------|
| Binance Futures | `FuturesBatchAmend` | `PATCH /fapi/v1/batchOrders` |
| Bybit | `BatchAmendOrders` | `POST /v5/order/amend-batch` |
| OKX | Batch amend | `POST /api/v5/trade/amend-batch-orders` |
| Gate.io Futures | Batch amend | `POST /futures/usdt/batch_amend_orders` |
| HTX | (batch orders endpoint missing entirely for spot) | `POST /v1/order/batch-orders` |

---

### P1.5 — Missing WebSocket Streams / Channels

Required for real-time data without polling.

#### CEX connectors with missing WS channels:

| Connector | Missing Channels |
|-----------|-----------------|
| **Bitget** | ALL WS channels missing: ticker, orderbook, trades, candles, funding rate (public); orders, account, positions (private) |
| **BingX** | ALL WS channels missing: ticker, orderbook, trades, klines (public); account updates, orders, positions (private) |
| **Gemini** | Market data WS channels not fully typed |
| **Phemex** | WS channel variants missing |
| **Deribit** | WS subscription channels not fully typed |
| **Binance** | `PUT /api/v3/userDataStream` (keepalive) + `DELETE` (close) missing |
| **Bybit** | Multiple WS subscription channels missing |
| **Alpaca** | Crypto WS URL missing; stream enum variants empty for stocks/trading WS |
| **Tiingo** | WS URLs stored but stream enum variants empty (IEX, Forex, Crypto) |
| **Polygon** | 4 WS categories: options, forex, crypto, indices — all missing |
| **Finnhub** | WS URL stored but no stream enum variants |

#### DEX / onchain connectors with missing WS:

| Connector | Issue |
|-----------|-------|
| **Bitquery** | GraphQL subscriptions written but not wired to `WebSocketConnector` trait |
| **Whale Alert** | WS URL stored (`wss://leviathan.whale-alert.io/ws`) but `WebSocketConnector` not implemented |
| **Uniswap** | ETH WS URL stored but no subscription connector |

---

### P1.6 — DEX Trading Stubs (Wallet Signing Required)

All DEX `place_order`/`cancel_order` operations are stubbed with `UnsupportedOperation`. These require wallet signing crates.

| Connector | Signing Type | Crate Needed | Effort |
|-----------|-------------|--------------|--------|
| **Lighter** | ECDSA secp256k1 L2 tx | `k256` (0.13) | LOW — REST API complete, just signing |
| **Paradex** | StarkNet ECDSA for JWT auto-refresh | `starknet-rs` (0.7) | LOW — everything else works |
| **Jupiter** | Solana tx signing | `solana-sdk` (1.18) | MEDIUM |
| **Raydium** | Solana tx signing | `solana-sdk` (1.18) | MEDIUM |
| **Uniswap** | EVM EIP-712 + Permit2 | `alloy` (0.1+) | MEDIUM |
| **GMX** | EVM EIP-712 + contract calls | `alloy` (0.1+) | HIGH |
| **dYdX** | Cosmos gRPC + protobuf | `tonic` + `prost` | HIGH — different transport |

---

### P1.7 — Closed PnL History

Required for performance reporting and tax accounting.

| Connector | Missing Endpoint |
|-----------|-----------------|
| Bybit | `GET /v5/position/closed-pnl` |
| Binance Futures | `GET /fapi/v1/income` (income history includes realized PnL) |
| KuCoin Futures | (no closed PnL endpoint modeled) |
| Gate.io Futures | Closed positions history |
| OKX | Closed positions history |

---

### P1.8 — Long/Short Ratio & Sentiment Data

Required for derivatives market microstructure analysis.

| Connector | Missing Endpoints |
|-----------|-----------------|
| Binance Futures | `topLongShortAccountRatio`, `topLongShortPositionRatio`, `globalLongShortAccountRatio`, `takerlongshortRatio` (4 endpoints) |
| Bybit | `GET /v5/market/account-ratio` |
| Bitfinex | `stats1` endpoint (open interest, longs/shorts, funding stats) |
| OKX | Long/short ratio endpoint |

---

---

## P2: NICE-TO-HAVE GAPS

### P2.1 — Margin / Lending (Borrow, Repay, Interest)

Large surface area, not needed for basic spot/futures trading but required for margin strategies.

**Affected connectors with ENTIRE margin sub-system missing:**

| Connector | Approximate Missing Endpoints |
|-----------|-------------------------------|
| Binance | 20+ (margin borrow/repay, account, all orders, history, interest) |
| HTX | 14 (isolated + cross margin borrow/repay/balance, 7 each) |
| KuCoin | 10 (borrow, repay, interest, risk limit, margin symbols) |
| Bitget | 10 (cross + isolated borrow/repay/orders/interest) |
| Gate.io | 8 (margin loans, repayment, account info) |
| Bybit | 5 (borrow history, repay liability, collateral info) |
| Bitfinex | 14 (entire P2P lending sub-system: offers, loans, credits, funding trades) |
| Phemex | Margin endpoints |

**Total missing: ~80 margin/lending endpoints**

---

### P2.2 — Earn / Staking / Savings

Required for yield strategies and portfolio management.

| Connector | Missing |
|-----------|---------|
| Binance | 20+ (Simple Earn flexible/locked subscribe/redeem, ETH staking, SOL staking, history) |
| KuCoin | 3 (savings products, subscribe, redeem) |
| Bitget | 3 (loan borrow, hour interest, savings products) |
| OKX | Earn/savings endpoints |
| Gate.io | Earn products |

**Total missing: ~35 earn/staking endpoints**

---

### P2.3 — Convert / Swap Endpoints

Required for dust conversion and instant swap features.

| Connector | Missing Endpoints |
|-----------|-----------------|
| Binance | 6 (convert exchangeInfo, assetInfo, getQuote, acceptQuote, tradeFlow, orderStatus) |
| Binance | 4 (dust convert, dust-to-BTC, convert-transfer, history) |
| OKX | Convert endpoints |
| KuCoin | Convert/swap endpoints |

**Total missing: ~15 convert/swap endpoints**

---

### P2.4 — Copy Trading

| Connector | Missing |
|-----------|---------|
| Bitget | 15+ (futures trader APIs: followers, current/history orders; follower APIs: settings, close positions, query traders; spot copy trading entirely missing) |
| BingX | 4 (follower positions, trader profits, copy settings, stop copying) |
| OKX | Copy trading endpoints |

**Total missing: ~25 copy trading endpoints**

---

### P2.5 — Stop / Plan / Conditional Orders

Required for TP/SL order types without manual monitoring.

| Connector | Missing |
|-----------|---------|
| KuCoin Spot | Stop orders place/cancel/list (3 endpoints) |
| KuCoin Futures | Futures stop orders (3 endpoints) |
| Bitget Spot | Plan orders place/cancel/current/history (4 endpoints) |
| Bitget Futures | Cancel plan order, current plan orders, history plan orders (3 endpoints) |
| HTX | Algo order cancel, open list, history, specific (5 endpoints) |
| Binance Spot | `cancelReplace`, `amendKeepPriority`, OTO, OPO, OPOCO (5 new order types) |

**Total missing: ~25 conditional order endpoints**

---

### P2.6 — Wallet / Asset Management Gaps

| Connector | Missing |
|-----------|---------|
| Binance | Coin config (all coins info, networks, fees), funding wallet, wallet balance, dust log |
| Bybit | Exchange record, delivery record, coin balance, withdraw/deposit records for specific assets |
| KuCoin | Transfer quotas, flex transfer, withdrawal cancellation, withdrawal quotas |
| HTX | Inter-account transfer (`/v1/account/transfer`) |
| Gate.io | Sub-account balance, sub-account transfers |

---

### P2.7 — Stock Market Gaps (US Stocks)

#### Polygon (25+ missing, including 3 wrong paths):
- **Wrong paths (BUGS):** `Dividends`, `Splits`, `FinancialRatios` all have incorrect URL paths
- **Missing:** Full options data surface (8 endpoints), indices snapshots/aggregates (2), forex data (3), crypto data (3), gainers/losers snapshots, reference conditions/exchanges/events, 4 WebSocket categories

#### Finnhub (19 missing):
- ETF family: holdings, profile, country, sector (4)
- Mutual fund family: holdings, profile, sector, country (4)
- Bond family: price, profile, yield-curve (3)
- IPO calendar, earnings surprise, as-reported financials, social sentiment, earnings transcripts (2), crypto profile, investment theme, country data, filing similarity index

#### Alpaca (17 missing):
- Watchlist CRUD (6), position close operations (2), account configuration (2), single-symbol market data variants (3), options chain, options contract detail, most-actives screener, auctions data

#### TwelveData:
- `real_time_price`, `complex_data` batch endpoint, mutual funds list, bonds list, missing 50+ technical indicators

#### Tiingo:
- List daily tickers, list IEX all, list forex tickers, forex metadata (4 endpoints)

---

### P2.8 — India Stocks Gaps

| Connector | Key Missing |
|-----------|-------------|
| **Zerodha (Kite)** | Portfolio margins, basket orders, GTT order list/delete/modify, instruments master file download |
| **Dhan** | Options chain endpoint, kill switch, trade history |
| **Fyers** | Position conversion, basket orders, net position |
| **Upstox** | Market quotes for multiple instruments, historical data v3 |
| **Angel One** | WebSocket market data stream channels not fully typed |

---

---

## P3: INTEL FEED GAPS

### P3.1 — Economic Data Feeds

| Connector | Key Gaps | Priority |
|-----------|----------|----------|
| **IMF DataMapper** | Entire WEO forecast API missing (4 endpoints: indicators, countries, regions, data) | HIGH |
| **CBR (Russia)** | RUONIA rate, deposit rates, refinancing rate history (3 endpoints) | MEDIUM |
| **World Bank** | Multi-country indicator batch fetch, sub-national data | LOW |
| **BIS** | 4 minor structural SDMX endpoints (categorisation, agencyscheme, etc.) | LOW |
| **ECB** | 4 structural SDMX endpoints (categoryscheme, contentconstraint, etc.) | LOW |
| **FRED** | Complete — 0 gaps | NONE |
| **DBnomics** | Complete — 0 gaps | NONE |
| **ECOS** | Complete — 0 gaps | NONE |
| **Eurostat** | 4 catalogue endpoints (DCAT-AP, RSS, metabase bulk) | LOW |

---

### P3.2 — Governance & Sanctions

| Connector | Key Gaps | Priority |
|-----------|----------|----------|
| **OpenCorporates** | `control_statements/search` (beneficial ownership chains), officer direct lookup, filing detail, statements subsystem, industry codes | HIGH |
| **GLEIF** | `relationship-records` (Level 2 ownership — who owns whom), BIC maps, reporting exceptions, fuzzy completions | HIGH |
| **EU Parliament** | `vote-results` (roll-call votes — entirely missing), parliamentary questions, activities, adopted texts | HIGH |
| **UK Parliament** | `RegisteredInterests` (financial transparency), full amendment tracking, parties composition, posts (govt/opposition) | HIGH |
| **UK Companies House** | 6 search variants, disqualifications, specific-item endpoints for charges/filings/PSC | MEDIUM |
| **OpenSanctions** | Entity adjacency (`/entities/{id}/adjacent`), statements endpoint | MEDIUM |
| **INTERPOL** | Yellow notice detail/images, UN notice type disambiguation | MEDIUM |
| **OFAC** | Bulk screening (`POST /screen/bulk`) | LOW |

---

### P3.3 — Humanitarian / Conflict

| Connector | Key Gaps | Priority |
|-----------|----------|----------|
| **UCDP** | Dyadic dataset missing; `StateConflict` path WRONG (BUG); version numbers stale | HIGH (BUG) |
| **UN OCHA** | API migrated to v2 (BUG); conflict events (ACLED), national risk (INFORM), food prices (WFP) all missing | HIGH (BUG) |
| **UN Population** | Core data endpoint missing — effectively non-functional; auth token flow missing | HIGH (BLOCKER) |
| **ACLED** | CAST forecasts (predictive conflict alerts) + deleted records (2 endpoints) | MEDIUM |
| **UNHCR** | Asylum applications endpoint (distinct from decisions) | MEDIUM |

---

### P3.4 — Space / Aviation / Maritime

| Connector | Key Gaps | Priority |
|-----------|----------|----------|
| **Space-Track** | `cdm_public` (conjunction/collision data — key safety data), `gp_history` (historical TLEs), `boxscore`, `satcat_change`, `OMM`; ARCHITECTURAL BUG: hardcoded query strings | HIGH |
| **OpenSky** | OAuth2 auth broken (BLOCKER); airports endpoint | HIGH (BLOCKER) |
| **NASA** | DONKI CMEAnalysis, notifications (high-value for alerts), MPC, RBE, HSS, WEP; Mars Rover Photos; Earth Imagery; NEO Browse (11+ endpoints) | MEDIUM |
| **AIS (Datalastic)** | `vessel_inradius` (geospatial area search — chokepoint monitoring), `vessel_bulk` (batch efficiency), `vessel_pro_est` (satellite estimated position) | MEDIUM |
| **NGA Warnings** | ASAM reports (piracy/hostile acts — HIGH value), MODU positions, World Port Index | MEDIUM |
| **AviationStack** | HTTP→HTTPS bug; timetable, flightsfuture, airplanes endpoints | MEDIUM (BUG) |
| **Sentinel Hub** | Batch processing, OGC WMS/WCS/WFS; catalog path WRONG | MEDIUM (BUG) |
| **Launch Library 2** | Stale API version; wrong path names; 6 missing endpoints (launcher, pad, location, expedition, docking, payload) | LOW |
| **Wikipedia** | 4 analytics subsystems missing (unique-devices, edits, editors, registered-users); base URL too narrow | LOW |

---

### P3.5 — Onchain / DEX Intelligence

| Connector | Key Gaps | Priority |
|-----------|----------|----------|
| **Bitquery** | WS GraphQL subscriptions not wired to connector; would enable real-time DEX trade streams | HIGH |
| **Whale Alert** | WS real-time alert stream not implemented | MEDIUM |
| **Etherscan** | Does not implement standard digdigdig3 traits — standalone only | LOW |

---

---

## IMPLEMENTATION BATCHES (Recommended Execution Order)

### Implementation Batch A — Bugs First (1–2 days)

Fix broken connectors before adding features:

1. **Polygon** — Fix 3 wrong paths: `Dividends`, `Splits`, `FinancialRatios`
2. **OpenSky** — Migrate auth from Basic to OAuth2 client credentials
3. **UCDP** — Fix `StateConflict` path + update version numbers to `25.1`
4. **AviationStack** — Change base URL from `http://` to `https://`
5. **UN OCHA** — Update base URL from `/api/v1` to `/api/v2` + add v2 endpoints
6. **ReliefWeb** — Update base URL from `/v1` to `/v2`
7. **Sentinel Hub** — Fix catalog path from `/catalog/search` to `/catalog/1.0.0/`
8. **Binance Futures** — Add `FuturesAccountV3` variant for `/fapi/v3/account`
9. **Launch Library 2** — Fix path names + update to v2.3.0

---

### Implementation Batch B — Trade History (2–3 days)

Add `myTrades` / fill history variants to top-priority CEX connectors:

1. Binance: `SpotMyTrades` + `FuturesUserTrades`
2. Bybit: `Executions`
3. OKX: Fill history endpoints
4. Kraken: Trade history
5. KuCoin: `SpotFills` + `FuturesFills`
6. Gate.io: Spot + futures my trades
7. HTX: Fill history by order
8. MEXC: `myTrades`
9. Bitget: `FuturesFillHistory`
10. BingX: `myTrades` + `allFillOrders`

---

### Implementation Batch C — Open Interest + Funding Rate History (2–3 days)

Add OI and funding rate history to all perps connectors:

1. Binance Futures: `OpenInterest`, `OpenInterestHist`
2. Bybit: `OpenInterest`
3. OKX: Open interest
4. KuCoin Futures: Open interest + funding rate history
5. HTX Futures: `swap-open-interest` + funding rate history
6. Bitget: `open-interest` + `history-fund-rate`
7. BingX: `openInterest` + `fundingRateHistory`
8. Gate.io: Futures OI + funding rate history

---

### Implementation Batch D — Mark / Index Price (1–2 days)

Add mark price and index price to perps connectors (required for liquidation calculations):

1. Binance: `premiumIndex` (mark + funding combined)
2. KuCoin Futures: mark price + index price + premium index (3 variants)
3. HTX Futures: index price + mark price klines
4. Bitget: `symbol-price` (mark + index + last)
5. BingX: `premiumIndex`
6. Gate.io, MEXC, Phemex, Deribit: mark price endpoints

---

### Implementation Batch E — Batch Amend + Advanced Order Types (1–2 days)

1. Binance Futures: `FuturesBatchAmend` (`PATCH /fapi/v1/batchOrders`)
2. Bybit: `BatchAmendOrders` (`POST /v5/order/amend-batch`)
3. OKX: `BatchAmendOrders`
4. KuCoin: Stop orders (spot + futures)
5. Bitget: Plan orders (spot + futures)
6. HTX: Algo order management (cancel/list/history)
7. BingX: `SwapQueryOrder` + force orders

---

### Implementation Batch F — WebSocket Channels (3–5 days)

Priority order:

1. **Bitquery** — Wire existing GraphQL subscription queries to `WebSocketConnector` trait
2. **Whale Alert** — Implement `WebSocketConnector` for real-time alerts
3. **Bitget** — Implement all WS public channels (ticker, books, trades, candles)
4. **BingX** — Implement all WS channels
5. **Binance** — Add keepalive/close listen key variants
6. **Alpaca** — Add crypto WS URL + stream enum variants
7. **Tiingo** — Wire stored WS URLs to stream enum variants
8. **Polygon** — Add options/forex/crypto/indices WS categories

---

### Implementation Batch G — DEX Wallet Signing (5–10 days, new crates)

Ordered by effort (easiest first):

1. **Lighter** — Add `k256` crate, implement ECDSA secp256k1 signing for L2 orders
2. **Paradex** — Add `starknet-rs`/`starknet-crypto`, implement JWT auto-refresh
3. **Jupiter** — Add `solana-sdk`, implement quote + sign + submit flow
4. **Raydium** — Add `solana-sdk`, same pattern as Jupiter
5. **Uniswap** — Add `alloy`, implement EIP-712 + Permit2 + broadcast
6. **GMX** — Add `alloy`, implement ExchangeRouter contract interaction
7. **dYdX** — Add `tonic` + `prost`, implement Cosmos gRPC MsgPlaceOrder

---

### Implementation Batch H — Intel Feed Data Completeness (5–10 days)

Priority order:

1. **UN Population** — Add core `/data/indicators/{...}/locations/{...}` endpoint + auth token flow (currently non-functional)
2. **IMF DataMapper** — Add 4 WEO endpoints (indicators, countries, regions, data)
3. **EU Parliament** — Add `vote-results` + parliamentary questions
4. **UK Parliament** — Add `RegisteredInterests` + amendment tracking
5. **GLEIF** — Add `relationship-records` (Level 2 ownership data)
6. **OpenCorporates** — Add `control_statements/search` + beneficial ownership
7. **Space-Track** — Add `cdm_public` (collision data) + `gp_history` + refactor to flexible query builder
8. **OpenSanctions** — Add entity adjacency endpoint
9. **CBR** — Add RUONIA rate + deposit rates

---

---

## CONNECTOR COVERAGE SUMMARY

### CEX Coverage (from batch summaries)

| Exchange | Est. Coverage | Key Missing Categories |
|----------|---------------|------------------------|
| Binance | ~60% | myTrades, open interest, margin, earn, convert |
| Bybit | ~55% | Executions, closed PnL, open interest, batch amend, margin |
| OKX | ~55% | Fill history, open interest, batch amend, margin, convert |
| Kraken | ~50% | Trade history, margin, WS stream variants |
| Coinbase | ~50% | Fill history, WS stream variants |
| KuCoin | ~49% | Fills, stop orders, mark price, margin, earn |
| Bitfinex | ~49% | Entire lending subsystem, statistics API, derivatives collateral |
| HTX | ~44% | Margin (14 endpoints), algo order mgmt, futures market data |
| Gate.io | ~52% | Recent trades, my trades, futures data, margin |
| MEXC | ~51% | Recent trades, myTrades, futures data |
| Bitget | ~45% | Fill history, plan orders, open interest, margin, WS (all missing) |
| BingX | ~48% | Query order, myTrades, OI, funding history, WS (all missing), Coin-M |
| Crypto.com | ~60% | Advanced order cancel variants, account ledger, earn |
| Gemini | ~55% | WS channel variants, advanced order types |
| Phemex | ~50% | Open interest, funding history, WS channels |

### Stocks Coverage

| Connector | Coverage | Key Missing |
|-----------|----------|-------------|
| Alpaca | ~65% | Watchlists, position close, account config, single-symbol variants |
| Finnhub | ~72% | ETF, mutual funds, bonds, social sentiment, transcripts |
| Polygon | ~58% (3 BUGS) | Options entire surface, wrong paths, WS categories |
| Tiingo | ~75% | WS stream variants, list endpoints |
| TwelveData | ~70% | real_time_price, complex_data, funds/bonds |
| Angel One | ~60% | WS stream channels |
| Dhan | ~65% | Options chain, trade history |
| Fyers | ~60% | Position conversion, basket orders |
| Upstox | ~65% | Market quotes batch, historical v3 |
| Zerodha | ~60% | GTT orders, portfolio margins, basket orders |

### Intel Feeds Coverage

| Category | Overall Coverage | Critical Gaps |
|----------|-----------------|---------------|
| Economic (FRED, DBnomics, ECOS) | ~95% | IMF DataMapper missing |
| Economic (CBR, central banks) | ~75% | Specialty rates missing |
| Conflict/Humanitarian | ~70% | UCDP bug, UN OCHA v2 migration, UN Population blocker |
| Space | ~65% | OpenSky auth broken, Space-Track architecture issue |
| Aviation | ~80% | OpenSky blocker, AviationStack HTTP bug |
| Maritime | ~75% | AIS vessel_inradius, NGA ASAM, IMF PortWatch unverified |
| Governance | ~40% | Vote results, registered interests, beneficial ownership |
| Sanctions/Legal | ~70% | OpenSanctions adjacency, OpenCorporates statements |
| Corporate/Trade | ~50% | GLEIF relationships, OpenCorporates beneficial ownership |
| Demographics | ~55% | UN Population blocker, WHO dimensions |

---

## CRATE DEPENDENCY SUMMARY (New crates needed for P1+ DEX work)

| Crate | Version | Required For |
|-------|---------|-------------|
| `k256` | `0.13` | Lighter (secp256k1 ECDSA L2 tx signing) |
| `starknet-crypto` | `0.6+` | Paradex JWT generation from StarkNet key |
| `solana-sdk` | `1.18` | Jupiter + Raydium swap execution |
| `alloy` | `0.1+` | Uniswap + GMX swap execution (preferred over `ethers`) |
| `tonic` | `0.12` | dYdX gRPC Node API |
| `prost` | `0.12` | dYdX protobuf message encoding |

**Currently sufficient (no new crates needed) for all P0 bug fixes and trade history additions.**

---

*Generated from WAVE4 gap analysis — 14 batch reports covering ~90 connectors*
