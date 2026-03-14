# Connector Maturity Matrix

Generated: 2026-03-14
Auditor: Static analysis of `connector.rs`, `websocket.rs`, `auth.rs`, and `tests.*` files.

## Legend

| Symbol | Meaning |
|--------|---------|
| ✅ | Implemented + has test file |
| 🟡 | Implemented, no dedicated test file |
| ⬜ | Stub / returns `UnsupportedOperation` |
| ❌ | Missing entirely (file/method does not exist) |

---

## How to Read the Columns

- **Klines** — `get_klines()` REST implementation
- **Ticker** — `get_ticker()` REST implementation
- **OB** — `get_orderbook()` REST implementation
- **Trades** — `get_recent_trades()` REST implementation
- **WS** — `websocket.rs` exists with real channels
- **Place** — `place_order()` real implementation (not stub)
- **Cancel** — `cancel_order()` real implementation
- **OpenOrd** — `get_open_orders()` real implementation
- **OrdHist** — `get_order_history()` real implementation
- **Balances** — `get_account_info()` real implementation
- **Positions** — `get_positions()` real implementation
- **Auth** — signing method used
- **Tests** — test files and count

---

## Crypto CEX

> **Note on disabled connectors:**
> - **Vertex** — PERMANENTLY DISABLED (exchange shut down 2025-08-14, acquired by Ink Foundation). Code retained for reference only.
> - **Bithumb** — TEMPORARILY DISABLED (SSL hangs, 403 geo-blocking). Code complete but disabled in module.
> - **Phemex** — REMOVED FROM LIVE WATCHLIST (HTTP 403 on WS upgrade). Code retained, REST may still work.
> - **GMX** — REMOVED FROM LIVE WATCHLIST (no real WS API, REST polling only).

| Connector | Klines | Ticker | OB | Trades | WS | Place | Cancel | OpenOrd | OrdHist | Balances | Positions | Auth | Tests |
|-----------|--------|--------|-----|--------|-----|-------|--------|---------|---------|----------|-----------|------|-------|
| **Binance** | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Bybit** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **OKX** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Gate.io** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA512 | none |
| **KuCoin** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Kraken** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | SHA256+HMAC | none |
| **Coinbase** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | JWT ES256 | none |
| **Bitfinex** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA384 | none |
| **Bitget** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **BingX** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Phemex** ⚠️ | 🟡 | 🟡 | 🟡 | ⬜ | 🟡¹ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **MEXC** | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Gemini** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | ⬜² | HMAC-SHA384 | none |
| **Bitstamp** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | ⬜² | HMAC-SHA256 | none |
| **HTX** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Bithumb** ⚠️ | ✅ | ✅ | ✅ | ⬜ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | HMAC-SHA256 | 26 (disabled) |
| **Upbit** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡³ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | ⬜² | JWT | none |
| **Deribit** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **Crypto.com** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡⁴ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | HMAC-SHA256 | none |
| **HyperLiquid** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | EIP-712 | none |
| **Vertex** ⛔ | ✅ | ✅ | ✅ | ⬜ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | EIP-712 | 20 (disabled) |

> ¹ Phemex WS code exists but 403-blocked at network level.
> ² Spot-only exchange — positions are inherently `UnsupportedOperation`.
> ³ Upbit WS has partial `UnsupportedOperation` stubs for some channels.
> ⁴ Crypto.com WS has a partial `UnsupportedOperation` stub for one method.

### Missing CEX (mentioned in task spec, not implemented)

| Connector | Status |
|-----------|--------|
| AscendEX | ❌ Missing |
| BigONE | ❌ Missing |
| ProBit | ❌ Missing |
| BitMart | ❌ Missing |
| CoinEx | ❌ Missing |
| DigiFinex | ❌ Missing |
| WOO X | ❌ Missing |
| XT.com | ❌ Missing |
| LBank | ❌ Missing |
| HashKey | ❌ Missing |
| WhiteBIT | ❌ Missing |
| BTSE | ❌ Missing |

---

## Crypto DEX

| Connector | Klines | Ticker | OB | Trades | WS | Place | Cancel | OpenOrd | OrdHist | Balances | Positions | Auth/Signing | Chain Wired | Tests |
|-----------|--------|--------|-----|--------|-----|-------|--------|---------|---------|----------|-----------|------|-------------|-------|
| **dYdX v4** | 🟡 | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | Cosmos wallet (gRPC) | gRPC channel | none |
| **GMX** ⚠️ | 🟡 | 🟡 | 🟡 | ⬜ | 🟡¹ | 🟡 | 🟡 | ⬜ | ⬜ | 🟡 | 🟡 | On-chain EVM tx | ❌ not wired | none |
| **Lighter** | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡² | 🟡² | 🟡 | 🟡 | 🟡 | 🟡 | ZK-native crypto | ❌ not wired | none |
| **Paradex** ⚠️ | 🟡 | 🟡 | 🟡 | ⬜ | 🟡³ | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | 🟡 | StarkNet | optional (`onchain-starknet` feature) | none |
| **Jupiter** | ⬜ | 🟡 | ⬜ | ⬜ | 🟡 | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | ⬜ | Solana wallet | ❌ not wired | none |

> ¹ GMX has no real WS API — websocket.rs exists but streams are REST-polled. Removed from live watchlist.
> ² Lighter's `place_order` / `cancel_order` via trait are stub (`UnsupportedOperation`); real signed operations are in `place_order_signed()` / `cancel_order_signed()`.
> ³ Paradex WS disabled (per-symbol attribution impossible). Code retained.

---

## Crypto Swap (AMM DEX)

| Connector | Klines | Ticker | OB | WS | Swap/Place | Cancel | Account | Auth/Signing | Chain Wired | Tests |
|-----------|--------|--------|-----|----|------------|--------|---------|------|-------------|-------|
| **Uniswap v3** | 🟡 | 🟡 | 🟡 | 🟡 | ⬜¹ | ⬜ | ⬜ | k256 / EIP-712 | EVM — private key optional | none |
| **Raydium** | 🟡 | 🟡 | ⬜ | 🟡 | ⬜¹ | ⬜ | ⬜ | Solana keypair | `SolanaProvider` wired | none |

> ¹ AMMs do not have traditional `place_order` / account management — all trading operations return `UnsupportedOperation` as by design.

---

## Stocks — US

| Connector | Klines | Ticker | OB | WS | Trading | Account | Positions | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|-----------|------|-------|
| **Alpaca** | 🟡 | 🟡 | 🟡¹ | 🟡 | 🟡 | 🟡 | 🟡 | API key header | none |
| **Polygon** | 🟡 | 🟡 | 🟡 | 🟡 | ⬜ | ⬜ | ⬜ | API key (data only) | none |
| **Tiingo** | 🟡 | 🟡 | ⬜² | 🟡 | ⬜ | ⬜ | ⬜ | Bearer token | none |
| **TwelveData** | 🟡 | 🟡 | ⬜² | 🟡 | ⬜ | ⬜ | ⬜ | API key (data only) | none |
| **Finnhub** | 🟡 | 🟡 | 🟡 | 🟡 | ⬜ | ⬜ | ⬜ | API key (data only) | none |

> ¹ Alpaca `get_orderbook()` returns `UnsupportedOperation` — Alpaca does not provide L2 order book via REST.
> ² Tiingo and TwelveData have no orderbook endpoint — returns `UnsupportedOperation`.

### Missing US Stocks Brokers (mentioned in task spec)

| Connector | Status |
|-----------|--------|
| Interactive Brokers (TWS/Gateway) | ❌ Missing (IB exists only as an *aggregator*, not a broker connector) |
| Schwab | ❌ Missing |
| Fidelity | ❌ Missing |

---

## Stocks — India

| Connector | Klines | Ticker | OB | WS | Trading | Account | Positions | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|-----------|------|-------|
| **Zerodha (Kite)** | ⬜¹ | 🟡 | ⬜ | ❌ | 🟡 | 🟡 | 🟡 | OAuth2 token | none |
| **Upstox** | 🟡 | 🟡 | ⬜ | ❌ | 🟡 | 🟡 | 🟡 | OAuth2 token | none |
| **Angel One (SmartAPI)** | 🟡 | 🟡 | ⬜ | ❌ | 🟡 | 🟡 | 🟡 | JWT + TOTP | none |
| **Dhan** | 🟡 | 🟡 | ⬜ | 🟡 | 🟡 | 🟡 | 🟡 | Access token | none |
| **Fyers** | 🟡 | 🟡 | ⬜ | ❌ | 🟡 | 🟡 | ⬜ | JWT OAuth2 | 31 tests |

> ¹ Zerodha `get_klines()` returns `UnsupportedOperation` — Kite Historical API requires separate subscription.

---

## Stocks — Russia

| Connector | Klines | Ticker | OB | WS | Trading | Account | Positions | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|-----------|------|-------|
| **MOEX ISS** | 🟡 | 🟡 | ⬜¹ | 🟡 | ⬜ | ⬜ | ⬜ | No auth (public) | 24 tests |
| **Tinkoff Invest** | 🟡 | 🟡 | ⬜ | ❌ | 🟡 | 🟡 | 🟡 | Bearer token + gRPC | 28 tests |

> ¹ MOEX ISS is a data-only public API — no trading, no account, no real-time orderbook.

---

## Stocks — Japan

| Connector | Klines | Ticker | OB | WS | Trading | Account | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|------|-------|
| **J-Quants** | 🟡 | 🟡 | ⬜¹ | ❌ | ⬜ | ⬜ | Refresh/ID token | none |

> ¹ Data provider only — no OB, no trading.

---

## Stocks — Korea

| Connector | Klines | Ticker | OB | WS | Trading | Account | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|------|-------|
| **KRX (Korea Exchange)** | 🟡 | 🟡 | ⬜¹ | ❌ | ⬜ | ⬜ | API key | 32 tests |

> ¹ Data provider only — no trading API.

---

## Stocks — China

| Connector | Klines | Ticker | OB | WS | Trading | Account | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|------|-------|
| **Futu OpenD** | 🟡 | 🟡 | ⬜¹ | ❌ | 🟡² | 🟡 | OpenD proto | 6 tests |

> ¹ OB via `proto_call()` returns `UnsupportedOperation` until OpenD is connected.
> ² Trading routes through OpenD protocol; most order types except basic market/limit return `UnsupportedOperation`.

---

## Forex

| Connector | Klines | Ticker | OB | WS/Stream | Trading | Account | Auth | Tests |
|-----------|--------|--------|-----|-----------|---------|---------|------|-------|
| **Alpha Vantage** | 🟡 | ⬜¹ | ⬜ | ❌ | ⬜ | ⬜ | API key | 10 tests |
| **Dukascopy** | 🟡 | 🟡 | ⬜ | ❌ | ⬜ | ⬜ | No auth (public) | none |
| **OANDA** | 🟡 | 🟡 | ⬜ | 🟡 (streaming) | 🟡 | 🟡 | Bearer token | none |

> ¹ Alpha Vantage `get_ticker()` returns `UnsupportedOperation` — no real-time quote endpoint.

---

## Aggregators

| Connector | Klines | Ticker | OB | WS | Trading | Account | Auth | Tests |
|-----------|--------|--------|-----|-----|---------|---------|------|-------|
| **CryptoCompare** | 🟡 | 🟡 | ⬜¹ | 🟡 | ⬜ | ⬜ | API key | none |
| **DeFiLlama** | ⬜² | 🟡 | ⬜ | ❌ | ⬜ | ⬜ | No auth (public) | none |
| **IB (Web API)** | 🟡 | 🟡 | ⬜³ | 🟡 | ⬜³ | 🟡 | OAuth2 | none |
| **Yahoo Finance** | 🟡 | 🟡 | ⬜ | 🟡 | ⬜ | ⬜ | No auth (scrape) | none |

> ¹ CryptoCompare has no standard orderbook endpoint in connector.
> ² DeFiLlama `get_klines()` returns `UnsupportedOperation` — no candle endpoint, TVL/price only.
> ³ IB Web API connector is limited — no trading routes implemented, `UnsupportedOperation` for most.

---

## Onchain Analytics

| Connector | Primary Data | WS | Auth | Tests |
|-----------|-------------|-----|------|-------|
| **Etherscan** | Block/tx explorer (5 files) | ❌ | API key | none |
| **BitQuery** | GraphQL multi-chain queries | 🟡 | OAuth2 | none |
| **Whale Alert** | Large transaction alerts | 🟡 | API key | none |

> All onchain analytics return `UnsupportedOperation` for market data traits — they expose domain-specific methods instead.

---

## Prediction Markets

| Connector | Klines | Ticker | WS | Trading | Auth | Tests |
|-----------|--------|--------|----|---------|------|-------|
| **Polymarket** | 🟡 | 🟡 | 🟡 | ⬜¹ | CLOB API key | none |
| **PredictIt** (intel feed) | ⬜ | ⬜ | ❌ | ⬜ | No auth | none |

> ¹ Polymarket trading requires Polygon wallet signing — not wired.

---

## Intelligence Feeds (85 total)

All intelligence feeds share a common pattern:
- **Market data traits** (`get_klines`, `get_ticker`, `place_order`, etc.) return `UnsupportedOperation` or `NotApplicable`
- **Domain-specific methods** are implemented per-connector (e.g., FRED's `get_series()`, GDELT's `get_articles()`)
- **No test files** unless noted

### Summary by Category

| Category | Connectors | WS/Streaming | Domain Methods | Depth (avg lines) |
|----------|-----------|--------------|----------------|-------------------|
| **Academic** | arxiv, semantic_scholar | ❌ | Search, papers, citations | ~317 |
| **Aviation** | adsb_exchange, aviationstack, opensky, wingbits | ❌ | Flight tracking, status | ~314 |
| **Conflict** | acled, gdelt, reliefweb, ucdp, unhcr | ❌ | Conflict events, displacement | ~394 |
| **Corporate** | gleif, opencorporates, uk_companies_house | ❌ | Legal entity, company data | ~355 |
| **Crypto Intel** | coingecko, coinglass | ❌ | Market cap, liquidations, OI | ~815 |
| **Cyber** | abuseipdb, alienvault_otx, censys, cloudflare_radar, nvd, ripe_ncc, shodan, urlhaus, virustotal | ❌ | Threat intel, CVEs, IP data | ~316 |
| **Demographics** | un_ocha, un_population, who, wikipedia | ❌ | Population, health, articles | ~352 |
| **Economic** | bis, boe, bundesbank, cbr, dbnomics, ecb, ecos, eurostat, fred, imf, oecd, worldbank | ❌ | Macro data, rates, series | ~395 |
| **Environment** | gdacs, global_forest_watch, nasa_eonet, nasa_firms, noaa, nws_alerts, open_weather_map, openaq, usgs_earthquake | ❌ | Disasters, weather, AQ | ~337 |
| **FAA Status** | faa_status | ❌ | NOTAM, airport status | ~273 |
| **Feodo Tracker** | feodo_tracker | ❌ | Botnet C2 IPs | ~273 |
| **Financial Intel** | alpha_vantage, finnhub, newsapi, openfigi | ❌ | News, fundamentals, ticker lookup | ~426 |
| **Governance** | eu_parliament, uk_parliament | ❌ | Parliamentary data | ~380 |
| **Hacker News** | hacker_news | ❌ | HN posts/comments | ~276 |
| **Maritime** | ais, aisstream, imf_portwatch, nga_warnings | ❌¹ | Vessel tracking, port data | ~306 |
| **C2 Intel** | c2intel_feeds | ❌ | Aggregated intel feed | ~182 |
| **RSS Proxy** | rss_proxy | ❌ | RSS feed aggregation | ~339 |
| **Sanctions** | interpol, ofac, opensanctions | ❌ | Sanctions lists, wanted | ~401 |
| **Space** | launch_library, nasa, sentinel_hub, space_track, spacex | ❌ | Launches, satellites, imagery | ~360 |
| **Trade** | comtrade, eu_ted | ❌ | Trade flows, tenders | ~351 |
| **US Gov** | bea, bls, census, congress, eia, fbi_crime, sam_gov, sec_edgar, usaspending | ❌ | Gov data, filings, energy | ~323 |
| **Prediction** (intel) | predictit | ❌ | Prediction market prices | ~276 |

> ¹ `aisstream` has a WebSocket-based feed — the only streaming connector in the intelligence feeds category.

---

## Overall Maturity Summary

### Crypto CEX (21 connectors)

| Tier | Count | Names |
|------|-------|-------|
| Full implementation (all 12 capabilities) | 19 | Binance, Bybit, OKX, Gate.io, KuCoin, Kraken, Coinbase, Bitfinex, Bitget, BingX, MEXC, Gemini, Bitstamp, HTX, Upbit, Deribit, Crypto.com, HyperLiquid, Phemex |
| Disabled (code complete, infra blocked) | 2 | Bithumb (26 tests), Vertex (20 tests, shut down) |
| Missing trades | 18 | Most — only Binance and MEXC have `get_recent_trades` |
| Missing (not implemented) | 12 | AscendEX, BigONE, ProBit, BitMart, CoinEx, DigiFinex, WOO, XT, LBank, HashKey, WhiteBIT, BTSE |

### Crypto DEX (5 connectors)

| Connector | Status |
|-----------|--------|
| dYdX v4 | Full implementation, gRPC wired |
| Lighter | Full implementation, signing exists |
| GMX | Full, no WS (design constraint), open/order history stub |
| Paradex | Full, StarkNet optional feature |
| Jupiter | Partial — klines/orderbook/trading are stubs, ticker + WS work |

### Crypto Swap (2 connectors)

Both Uniswap and Raydium are data-read connectors — trading is by design `UnsupportedOperation` (AMM swaps don't map to the trading trait). Data (klines, ticker) implemented. Chain providers optionally wired.

### Stocks (16 connectors across 5 regions)

| Region | Trading | Data | Tested |
|--------|---------|------|--------|
| US (Alpaca, Polygon, Tiingo, TwelveData, Finnhub) | Alpaca only | All 5 | None |
| India (Zerodha, Upstox, AngelOne, Dhan, Fyers) | All 5 | All 5 | Fyers (31 tests) |
| Russia (MOEX, Tinkoff) | Tinkoff only | Both | MOEX (24), Tinkoff (28) |
| Japan (J-Quants) | None (data only) | Yes | None |
| Korea (KRX) | None (data only) | Yes | 32 tests |
| China (Futu) | Partial (OpenD) | Yes | 6 tests |

### Forex (3 connectors)

- OANDA: most complete (klines, ticker, trading, streaming)
- Dukascopy: data only (klines, ticker)
- Alpha Vantage: klines only, 10 tests

### Aggregators (4 connectors)

All data-only by design. IB (as aggregator) and Yahoo have WS. CryptoCompare has WS. DeFiLlama REST-only.

### Intelligence Feeds (85 connectors)

All 85 are fully structured (auth.rs, connector.rs, endpoints.rs, parser.rs, mod.rs) but none have test files. All return `UnsupportedOperation` for standard trading/market-data traits. Domain-specific methods are the real API surface. Only `aisstream` has WebSocket streaming.

---

## Key Gaps / Action Items

1. **No test files for any active CEX connector** — 19 production CEX connectors have zero tests. The only connectors with tests are disabled ones (Bithumb, Vertex) or non-crypto (MOEX, Tinkoff, KRX, Fyers).

2. **`get_recent_trades` missing for 18 of 21 CEX connectors** — only Binance and MEXC implement this.

3. **Websocket coverage gap in stocks** — 9 of 16 stock connectors have no WebSocket file (Zerodha, Upstox, AngelOne, Fyers, Tinkoff, J-Quants, KRX, Futu).

4. **Jupiter is severely underimplemented** — klines, orderbook, all trading ops are stubs. Only ticker and WS work.

5. **12 requested CEX connectors are completely missing** — AscendEX, BigONE, ProBit, BitMart, CoinEx, DigiFinex, WOO, XT, LBank, HashKey, WhiteBIT, BTSE.

6. **Interactive Brokers missing as a broker** — only exists as an aggregator connector (IB Web API), not a proper brokerage integration.

7. **Chain signing not wired for GMX** — trading requires EVM wallet but provider is not attached in connector.

8. **Lighter's `place_order` trait is stub** — real signed orders exist only via `place_order_signed()` internal method, not exposed through the trading trait.

9. **Intelligence feeds have zero test coverage** — 85 connectors, 0 tests.

10. **Futu OpenD fully dependent on local daemon** — all methods return `UnsupportedOperation` until OpenD binary is running locally.
