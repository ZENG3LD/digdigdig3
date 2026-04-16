# digdigdig3

Multi-exchange connector library — unified async Rust API for crypto exchanges, stock brokers, forex providers, intelligence feeds, and DEX/on-chain query providers.

**Version:** 0.1.17
**Edition:** Rust 2021
**License:** MIT OR Apache-2.0
**Repository:** https://github.com/ZENG3LD/digdigdig3

> **Note:** On-chain monitoring (Solana, Bitcoin, TON, Sui, Aptos chain watchers, `OnChainEvent` types, `EventProducer` trait, and all transaction builders) was extracted to the [dig2chain](https://github.com/ZENG3LD/dig2chain) workspace. digdigdig3 retains only the on-chain providers needed by DEX connectors for query and signing (EVM via alloy, Cosmos via cosmrs, StarkNet via starknet-crypto).

---

## Overview

digdigdig3 is organized around capability levels. Each level describes what a connector can do and what it requires.

| Level | Description | Auth Required | Count |
|-------|-------------|--------------|-------|
| 1a | Market data feed — klines, orderbook, ticker, trades | No | 14 CEX + 7 DEX/other |
| 1b | Extended data feed — intelligence, aggregators, analytics | No or API key | ~95 |
| 2 | Authenticated data feed — order history, account info, positions | Yes | ~50 |
| 3 | Execution — place/cancel orders, manage positions | Yes | ~30 |
| 4a | On-chain transport — signing and submitting transactions for DEX connectors | Yes (private key) | 3 providers |

**Total connectors:** ~120+
**Blockchain chain monitoring:** moved to dig2chain

---

## Adding as a Dependency

```toml
[dependencies]
digdigdig3 = { path = "../digdigdig3" }
# or
digdigdig3 = { git = "https://github.com/ZENG3LD/digdigdig3", tag = "v0.1.17" }
```

With specific feature flags:

```toml
[dependencies]
digdigdig3 = { version = "0.1.17", features = ["grpc", "k256-signing"] }
```

---

## Level 1a: Market Data Feed (no auth required)

Public endpoints — price, orderbook, klines, ticker, trades. No credentials needed.

These connectors implement `ExchangeIdentity` + `MarketData` + `WebSocketConnector`.

### Crypto CEX — Tested on real data via live bridge

**Legend:**
- **Tested** — data flowed through the live bridge in production
- **Untested** — code exists and compiles, never validated on real data
- **N/A** — method not implemented (returns `UnsupportedOperation`)

| Exchange | REST klines | REST ticker | REST orderbook | REST trades | WS trades | WS ticker | WS orderbook | WS klines |
|----------|-------------|-------------|----------------|-------------|-----------|-----------|--------------|-----------|
| Binance | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bybit | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| OKX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| KuCoin | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Kraken | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Coinbase | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Gate.io | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Gemini | Tested | Tested (poll/15s) | Untested | N/A | Tested | Tested | Untested | Untested |
| MEXC | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| HTX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bitget | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| BingX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Crypto.com | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Upbit | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |

Notes:
- Gemini supplements WebSocket ticker with a REST poll fallback (15s interval). That REST `get_ticker` call was exercised in production.
- `get_recent_trades` (REST trades column) is implemented only for Lighter. All other connectors return `UnsupportedOperation` and are marked N/A.
- WS orderbook and WS klines channels exist in code for most connectors but were never subscribed to in a live session.

### Crypto CEX — Disabled

| Exchange | Issue | Status |
|----------|-------|--------|
| Bithumb | SSL timeouts and HTTP 403 geo-blocking. 26 test files exist but are disabled | Temporarily disabled |
| Vertex | Exchange shut down 2025-08-14. 20 test files disabled | Permanently disabled — code retained for reference |

### DEX / On-Chain Connectors

| Connector | Chain | Feature Flag | Notes |
|-----------|-------|-------------|-------|
| Uniswap | EVM | `onchain-evm` | Swap via Trading API + on-chain `exactInputSingle` |
| GMX | EVM | `onchain-evm` | Positions/orders via Subsquid GraphQL, funding rates from REST |
| dYdX v4 | Cosmos | `onchain-cosmos` + `grpc` | Full trading trait via Cosmos gRPC |
| Paradex | StarkNet | `onchain-starknet` | Full trading trait, StarkNet ECDSA signing |
| Jupiter | Solana | (no extra feature) | Swap via Ultra API + Trigger + Recurring. No klines/orderbook (AMM) |
| Raydium | Solana | (no extra feature) | Swap quote + transaction builder. No klines/orderbook (AMM) |
| Lighter.xyz | Custom | — | Full trading trait, native ECgFp5+Poseidon2+Schnorr signing (zero third-party crates) |

Notes:
- GMX WebSocket removed from live watchlist (no real-time WS API — internally polls REST).
- Paradex WebSocket removed from live watchlist (per-symbol attribution unreliable — exchange uses a global channel).
- Jupiter and Raydium `get_klines` / `get_orderbook` return `UnsupportedOperation` by design — these are AMM aggregators with no historical data endpoint.

---

## Level 1b: Extended Data Feed

These connectors expose data that does not fit the standard klines/orderbook/ticker API. They implement `ExchangeIdentity` + `MarketData` for basic identity, but their real value is in domain-specific methods (e.g., `get_series()`, `get_liquidations()`, `get_articles()`).

**None of this was tested against real APIs.** All connectors compile and are structurally complete, but no live data validation has been done.

### Crypto Intelligence

| Provider | Domain Methods | Auth |
|----------|----------------|------|
| CoinGecko | Market cap, coin details, trending, global stats | API key optional |
| CoinMarketCap | Price ranking, listings, metadata | API key |
| Coinglass | Liquidation data, open interest, long/short ratios, funding rates, fear and greed | API key optional |
| CryptoCompare | Multi-exchange price aggregation, historical OHLCV, social stats | API key |
| DeFiLlama | TVL by protocol/chain, yield pools, stablecoin data | No auth (public) |

### On-Chain Analytics

| Provider | Domain Methods | Transport | Auth |
|----------|----------------|-----------|------|
| BitQuery | Multi-chain GraphQL queries — transfers, DEX trades, token holders | REST + GraphQL + WebSocket | OAuth2 |
| Whale Alert | Large transaction alerts (cross-chain), blockchain monitoring | REST + WebSocket | API key |
| Etherscan | Block explorer queries — transactions, tokens, contracts, gas | REST | API key |

### Financial Data Aggregators

| Provider | Klines | Ticker | WebSocket | Notes |
|----------|--------|--------|-----------|-------|
| CryptoCompare | Yes | Yes | Yes | Multi-exchange aggregate |
| Yahoo Finance | Yes | Yes | Yes | No auth, scraping-based |
| Interactive Brokers Web API | Yes | Yes | Yes | Aggregator mode only — no brokerage execution wired |
| DeFiLlama | No | Yes | No | TVL/price only, no candles |

### Stock Data Providers (data-only, no trading)

| Provider | Region | Klines | Ticker | WebSocket | Auth |
|----------|--------|--------|--------|-----------|------|
| Polygon | US | Yes | Yes | Yes | API key |
| Finnhub | US | Yes | Yes | Yes | API key |
| Tiingo | US | Yes | Yes | Yes | Bearer token |
| TwelveData | US | Yes | Yes | Yes | API key |
| Alpha Vantage | Forex/US | Yes | No | No | API key |
| J-Quants | Japan | Yes | Yes | No | Refresh/ID token |
| KRX | Korea | Yes | Yes | No | API key |
| Dukascopy | Forex | Yes | Yes | No | No auth (public) |

Polygon, Finnhub, Tiingo, TwelveData, J-Quants, and KRX implement the trading trait interface but all trading methods return `UnsupportedOperation` — they are data providers, not brokers.

### Intelligence Feeds (85 connectors)

85 connectors organized across 21 domain categories. All are REST-only unless noted. All standard trading/market-data trait methods return `UnsupportedOperation` — domain-specific methods are the real surface area. No test files exist for any of these.

| Category | Connectors | Streaming | Sample Domain |
|----------|-----------|-----------|---------------|
| Economic | fred, ecb, imf, worldbank, oecd, bis, boe, bundesbank, cbr, dbnomics, eurostat, ecos | No | Macro data, interest rates, GDP series |
| US Government | sec_edgar, bls, bea, census, eia, usaspending, congress, fbi_crime, sam_gov | No | SEC filings, employment, energy, federal contracts |
| Cyber Intelligence | shodan, virustotal, alienvault_otx, censys, nvd, abuseipdb, urlhaus, ripe_ncc, cloudflare_radar | No | CVEs, IP reputation, threat indicators |
| Environment | noaa, usgs_earthquake, nasa_firms, nasa_eonet, gdacs, openaq, open_weather_map, global_forest_watch, nws_alerts | No | Weather, earthquakes, fires, air quality |
| Space | nasa, space_track, launch_library, spacex, sentinel_hub | No | Orbital data, launches, satellite imagery |
| Conflict/Geopolitical | gdelt, acled, ucdp, reliefweb, unhcr | No | Conflict events, displacement, humanitarian |
| Maritime | ais, aisstream, imf_portwatch, nga_warnings | aisstream only | Vessel tracking, port data, safety warnings |
| Aviation | adsb_exchange, opensky, aviationstack, wingbits | No | Live aircraft tracking, flight status |
| Sanctions | ofac, opensanctions, interpol | No | Sanctions lists, wanted persons |
| Demographics | un_population, who, wikipedia, un_ocha | No | Population, health, statistical data |
| Corporate | gleif, opencorporates, uk_companies_house | No | Legal entity identifiers, company registry |
| Governance | eu_parliament, uk_parliament | No | Parliamentary voting, legislative data |
| Financial Intel | newsapi, openfigi, finnhub (news), alpha_vantage | No | News, fundamentals, ticker lookup |
| Academic | arxiv, semantic_scholar | No | Research papers, citations |
| Trade | comtrade, eu_ted | No | International trade flows, EU tenders |
| Prediction (intel) | predictit | No | Political prediction market prices |
| Standalone feeds | faa_status, feodo_tracker, hacker_news, c2intel_feeds, rss_proxy | No | NOTAM, botnet IPs, HN posts, RSS aggregation |

`aisstream` is the only intelligence feed with WebSocket streaming.

---

## Level 2: Authenticated Data Feed

Same connector structs as Level 1a, but methods that require credentials. Includes: order history, account balances, positions, funding rate history, transfer history.

No connector in this category was tested against real credentials. All implementations are complete — they call the correct endpoints with signed requests — but have not been validated against live accounts.

### Crypto CEX (auth-required read methods)

All 14 active CEX connectors listed in Level 1a also implement these authenticated read methods:

- `get_open_orders()` — open orders for symbol or all symbols
- `get_order_history()` — historical orders with filters
- `get_order()` — single order by ID
- `get_account_info()` — account snapshot
- `get_balance()` — asset balances across account types
- `get_positions()` — open derivatives positions
- `get_funding_rate()` / `get_funding_rate_history()`
- `get_open_interest()`
- `get_mark_price()`
- `get_closed_pnl()`

Private WebSocket streams are also implemented for order updates, balance changes, and position updates.

### Stock Brokers (auth-required)

| Broker | Region | Klines | Trading | WebSocket | Auth Method |
|--------|--------|--------|---------|-----------|-------------|
| Alpaca | US | Yes | Yes | Yes | API key header |
| Zerodha (Kite) | India | No* | Yes | No | OAuth2 token |
| Upstox | India | Yes | Yes | No | OAuth2 token |
| Angel One | India | Yes | Yes | No | JWT + TOTP |
| Dhan | India | Yes | Yes | Yes | Access token |
| Fyers | India | Yes | Yes | No | JWT OAuth2 |
| Tinkoff Invest | Russia | Yes | Yes | No | Bearer token + gRPC |
| Futu OpenD | China | Yes | Partial | No | OpenD proto (requires local daemon) |

*Zerodha `get_klines` requires a separate Historical API subscription — returns `UnsupportedOperation` without it.

Futu requires the OpenD binary running locally. All methods return `UnsupportedOperation` until OpenD is connected.

### Forex (auth-required)

| Provider | Klines | Streaming | Trading | Auth |
|----------|--------|-----------|---------|------|
| OANDA | Yes | Yes (server-sent) | Yes | Bearer token |

### Test Coverage Summary

| Connector | Test Count | Type |
|-----------|-----------|------|
| Fyers | 31 tests | Unit tests in src/ |
| MOEX | 24 tests | Unit tests in src/ |
| Tinkoff | 28 tests | Unit tests in src/ |
| KRX | 32 tests | Unit tests in src/ |
| Futu | 6 tests | Unit tests in src/ |
| Alpha Vantage | 10 tests | Unit tests in src/ |
| Bithumb | 26 tests | Disabled (geo-block) |
| Vertex | 20 tests | Disabled (shut down) |

All 14 active CEX connectors have zero test files. This is the primary quality gap.

---

## Level 3: Execution

Order placement, cancellation, amendment, and account management. Always requires credentials. None of this was tested against real accounts.

### Core Trading Traits

All CEX connectors and most broker connectors implement:

| Trait | Methods |
|-------|---------|
| `Trading` | `place_order`, `cancel_order`, `get_order`, `get_open_orders`, `get_order_history` |
| `CancelAll` | `cancel_all_orders` (scope by symbol or all) |
| `AmendOrder` | `amend_order` (in-place modification without cancel/replace) |
| `BatchOrders` | `place_orders_batch`, `cancel_orders_batch` |
| `AccountTransfers` | `transfer` (between account types), `get_transfer_history` |
| `CustodialFunds` | `get_deposit_address`, `withdraw`, `get_funds_history` |
| `SubAccounts` | `sub_account_operation` (create, transfer, list) |

### CEX Execution Coverage

| Connector | CancelAll | AmendOrder | BatchOrders | AccountTransfers | CustodialFunds | SubAccounts |
|-----------|-----------|------------|-------------|-----------------|----------------|-------------|
| Binance | Yes | Yes | Yes | Yes | Yes | Yes |
| Bybit | Yes | Yes | Yes | Yes | Yes | Yes |
| OKX | Yes | Yes | Yes | Yes | Yes | Yes |
| KuCoin | Yes | Yes | Yes | Yes | Yes | Yes |
| Gate.io | Yes | Yes | Yes | Yes | Yes | Yes |
| Bitget | Yes | Yes | Yes | Yes | Yes | Yes |
| BingX | Yes | Yes | Yes | Yes | Yes | Yes |
| HTX | Yes | No | Yes | Yes | Yes | Yes |
| Crypto.com | Yes | Yes | Yes | No | Yes | Yes |
| MEXC | Yes | No | Yes | Yes | Yes | Yes |
| Kraken | Yes | Yes | No | No | Yes | Yes |
| Coinbase | Yes | No | No | No | Yes | No |
| Gemini | Yes | No | No | No | Yes | No |
| Upbit | Yes | Yes | No | No | Yes | No |

### DEX Execution

| Connector | Execution Notes |
|-----------|----------------|
| dYdX v4 | Order placement via Cosmos gRPC — full trading trait implemented |
| Lighter | Full trading trait — native ECgFp5+Poseidon2+Schnorr signing (zero third-party crates) |
| Paradex | Full trading trait, StarkNet signing via `onchain-starknet` feature |
| GMX | Trading trait + positions/orders via Subsquid GraphQL, funding rates from REST, ERC-20 balances |
| Jupiter | Swap via Ultra API + Trigger API (limit orders) + Recurring API (DCA). Trading trait stubs by design (AMM) |
| Raydium | Swap quote + transaction builder. Trading trait stubs by design (AMM) |
| Uniswap | Swap via Trading API + on-chain `exactInputSingle`. Trading trait stubs by design (AMM) |

### Stock Broker Execution

| Broker | place_order | cancel_order | amend_order | batch_orders |
|--------|-------------|--------------|-------------|--------------|
| Alpaca | Yes | Yes | Yes | No |
| Zerodha | Yes | Yes | Yes | No |
| Upstox | Yes | Yes | Yes | No |
| Angel One | Yes | Yes | Yes | No |
| Dhan | Yes | Yes | Yes | No |
| Fyers | Yes | Yes | No | Yes |
| Tinkoff | Yes | Yes | Yes | No |
| Futu | Partial | No | No | No |

### Optional Trading Traits (defined, not yet implemented in any connector)

These traits exist in `src/core/traits/operations.rs` with default `UnsupportedOperation` implementations. They are ready for connector-level override but no connector has explicit `impl` blocks for them yet:

- `MarginTrading` — margin borrow/repay, margin account info
- `EarnStaking` — earn products, subscribe/redeem
- `ConvertSwap` — convert quotes, dust conversion
- `CopyTrading` — follow/unfollow traders, copy positions
- `LiquidityProvider` — LP position management (Uniswap/Raydium/Jupiter)
- `VaultManager` — vault deposit/withdraw (GMX/Paradex/dYdX)
- `StakingDelegation` — delegate/undelegate, claim rewards
- `BlockTradeOtc` — OTC block trade creation and execution
- `MarketMakerProtection` — MMP config, mass quoting
- `TriggerOrders` — conditional order placement

---

## Level 4a: On-Chain Transport for DEX Connectors

Chain providers used internally by DEX connectors for transaction signing and query submission. These are **not** standalone chain monitors — for full blockchain event streaming, use [dig2chain](https://github.com/ZENG3LD/dig2chain).

| Provider | Chain | Feature Flag | Used By |
|----------|-------|-------------|---------|
| `EvmProvider` | Ethereum/EVM | `onchain-evm` (default) | Uniswap, GMX |
| `CosmosProvider` | Cosmos | `onchain-cosmos` | dYdX |
| `StarkNetProvider` | StarkNet | `onchain-starknet` | Paradex |

None of these have been tested end-to-end with real transaction submission.

Note: Solana chain interaction (Jupiter, Raydium) is implemented via raw WebSocket + tokio-tungstenite against the Solana RPC — no `solana-sdk` dependency needed.

---

## L2 Orderbook Capabilities

Each exchange declares its L2/orderbook capabilities via `orderbook_capabilities(account_type)`:

```rust
use digdigdig3::core::traits::WebSocketConnector;
use digdigdig3::core::types::AccountType;

let ws = BinanceWebSocket::new(None, None);
let caps = ws.orderbook_capabilities(AccountType::Spot);

println!("WS depths: {:?}", caps.ws_depths);           // [5, 10, 20]
println!("REST max: {:?}", caps.rest_max_depth);        // Some(5000)
println!("Snapshot: {}", caps.supports_snapshot);        // true
println!("Delta: {}", caps.supports_delta);              // true
println!("Checksum: {:?}", caps.checksum);               // None
println!("Channels: {}", caps.ws_channels.len());        // 8

// Pick best channel for your needs
if let Some(ch) = caps.best_channel(Some(20), true) {
    println!("Use channel: {} (delta={}, depth={:?})", ch.name, !ch.is_snapshot, ch.depth);
}
```

### Capabilities per exchange

| Exchange | WS Depths | Checksum | Aggregation | Sequence |
|----------|-----------|----------|-------------|----------|
| Binance | 5/10/20 | No | No | U/u (futures: +pu) |
| Bybit | 1/50/200/1000 | No | No | u field |
| OKX | 1/5/50/400 | CRC32 top-25 | No | seqId/prevSeqId |
| Kraken | 10/25/100/500/1000 | CRC32 top-10 | No | No (spot) |
| KuCoin | 5/50 (snapshot) | No | No | sequence |
| Coinbase | full book | No | No | sequence |
| GateIO | 5/10/20/50/100 | No | No | U/u |
| Bitfinex | 1/25/100/250 | CRC32 top-25 | P0-P4 | optional |
| Bitget | 1/5/15/full | CRC32 top-25 | No | seqId |
| HTX | 5/10/20/150/400 | No | step0-step5 | seqNum |
| MEXC | 5/10/20 | No | No | version |
| BingX | 5/10/20 | No | No | U/u |
| Bitstamp | 100 (snap)/full (delta) | No | No | microtimestamp |
| Gemini | 5/10/20 (Fast API) | No | No | socket_sequence |
| CryptoCom | 10/50/150 | CRC32 | No | u field |
| Upbit | 1/5/15/30 | No | KRW only | No |
| Deribit | 1/10/20 | No | group param | change_id |
| HyperLiquid | 20 (fixed) | No | nSigFigs | No |

---

## Architecture

### Core Trait Hierarchy

```
ExchangeIdentity (name, id, account types)
    |--- MarketData (price, orderbook, klines, ticker, ping)
    |--- Trading (place_order, cancel_order, get_order, open_orders, order_history)
    |--- Account (balance, account_info, fees)
    |--- Positions (positions, funding_rate, open_interest, mark_price, closed_pnl)

CoreConnector = ExchangeIdentity + MarketData + Trading + Account + Positions
    (blanket impl — use as generic bound for code working with any exchange)

Optional execution traits (each requires Trading or Account supertrait):
    CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts

Optional advanced traits (default UnsupportedOperation, override per-connector):
    MarginTrading, EarnStaking, ConvertSwap, CopyTrading, LiquidityProvider,
    VaultManager, StakingDelegation, BlockTradeOtc, MarketMakerProtection,
    TriggerOrders, PredictionMarket

WebSocket:
    WebSocketConnector (connect, subscribe, event_stream, active_subscriptions)
    WebSocketExt (convenience blanket: subscribe_ticker, subscribe_klines, etc.)
```

### Precision Guard (f64 → Decimal at Execution Boundary)

All DataFeed paths use `f64` for maximum performance (indicators, UI, research). At the Trading trait boundary, prices and quantities are converted to exchange-safe strings via `rust_decimal`:

```
DataFeed:  get_klines() → Vec<Kline{f64}>  → indicators/UI (fast, no overhead)
Execution: place_order(price: f64, qty: f64)
               ↓
           PrecisionCache.price(&symbol, price)  → "67543.25"
           PrecisionCache.qty(&symbol, qty)      → "0.12345"
               ↓
           Exchange API receives exact string
```

- `safe_price(f64, tick)` — converts via `Decimal::from_str(price.to_string())`, rounds to nearest tick
- `safe_qty(f64, step)` — same conversion, floors to step_size (never exceeds available quantity)
- `PrecisionCache` — per-symbol HashMap loaded from `get_exchange_info()`, stores tick/step per symbol
- Fallback: raw `f64::to_string()` if symbol not in cache

### Connector Manager

The connector manager (`src/connector_manager/`) provides a runtime pool for managing multiple connectors simultaneously:

| Component | Purpose |
|-----------|---------|
| `ConnectorHandle` | Unified dynamic dispatch wrapper for any connector |
| `ConnectorRegistry` | Maps `ExchangeId` to connector instances |
| `ConnectorFactory` | Builds connectors from `ConnectorConfig` |
| `ConnectorConfig` | Per-exchange config structs |
| `ConnectorPool` | DashMap-based concurrent pool |
| `ConnectorAggregator` | Multi-exchange data fan-out |

### Module Structure (per connector)

Every connector follows this layout:

```
src/{category}/{name}/
    mod.rs          — pub re-exports
    endpoints.rs    — URL constants, endpoint enum, symbol formatting
    auth.rs         — signing implementation (HMAC, JWT, EIP-712, etc.)
    parser.rs       — JSON response parsing, type mapping
    connector.rs    — trait implementations
    websocket.rs    — WebSocket connector implementation
```

Reference implementation: `src/crypto/cex/kucoin/` — most complete example with all optional traits.

---

## Transport Methods

| Transport | Feature Flag | Used By |
|-----------|-------------|---------|
| REST (reqwest) | default | All connectors |
| WebSocket | `websocket` | CEX + DEX + major stock brokers |
| gRPC (tonic) | `grpc` | dYdX (Cosmos gRPC), Tinkoff Invest |
| GraphQL | default | BitQuery |
| On-chain EVM RPC | `onchain-evm` | GMX, Uniswap |
| On-chain Cosmos gRPC | `onchain-cosmos` | dYdX |
| On-chain StarkNet | `onchain-starknet` | Paradex |

HTTP client: `src/core/http/client.rs` — async reqwest wrapper with auth injection, retry logic, and rate limiting (`SimpleRateLimiter` token bucket and `WeightRateLimiter`).

WebSocket base: `src/core/websocket/base_websocket.rs` — handles reconnect, ping/pong, and subscription replay on reconnect.

---

## Auth Methods

| Method | Connectors |
|--------|-----------|
| HMAC-SHA256 | Binance, Bybit, OKX, KuCoin, Gate.io, Bitget, BingX, MEXC, HTX, Crypto.com |
| HMAC-SHA384 | Gemini |
| HMAC-SHA512 | Kraken |
| HMAC + passphrase | OKX, KuCoin, Bitget (additional passphrase layer on top of HMAC) |
| JWT ES256 (EC P-256) | Coinbase, Upbit, Tinkoff, Dhan, J-Quants |
| JWT + TOTP | Angel One |
| OAuth2 / Bearer token | Upstox, Fyers, Zerodha, BitQuery, Whale Alert, OANDA, IB |
| EIP-712 (Ethereum typed data) | Uniswap, GMX |
| Ed25519 / Solana keypair | Raydium, Lighter |
| Cosmos SDK wallet | dYdX |
| StarkNet ECDSA (STARK key) | Paradex |
| API key in header | Polygon, Finnhub, Tiingo, most intelligence feeds |
| No auth | MOEX ISS, Dukascopy, DeFiLlama, most public intelligence feeds |

---

## Feature Flags

| Feature | Dependencies Enabled | Notes |
|---------|---------------------|-------|
| `default` | `onchain-evm` | EVM provider included by default |
| `onchain-evm` | `alloy` (provider-ws, rpc-types) | EVM chain provider for Uniswap/GMX |
| `onchain-ethereum` | `onchain-evm` | Backward-compat alias |
| `onchain-cosmos` | `cosmrs` (bip32) | Cosmos provider for dYdX |
| `onchain-starknet` | `starknet-crypto` | StarkNet provider for Paradex |
| `starknet` | `onchain-starknet` | Legacy alias |
| `websocket` | — | WebSocket enablement flag |
| `grpc` | `tonic` (tls, tls-native-roots), `prost` | gRPC transport |
| `k256-signing` | `k256` (ecdsa-core, ecdsa) | k256 ECDSA signing |

Removed features (extracted to dig2chain): `onchain-solana`, `onchain-bitcoin`, `onchain-sui`, `onchain-ton`, `onchain-aptos`.

---

## Known Issues and Disabled Connectors

| Connector | Issue | Status |
|-----------|-------|--------|
| Vertex | Exchange shut down 2025-08-14, acquired by Ink Foundation | Permanently disabled. Code retained as reference |
| Bithumb | SSL handshake timeouts and HTTP 403 geo-blocking | Temporarily disabled. Code complete. Re-enable when access resolves |
| GMX | No real-time WebSocket API — websocket.rs does REST polling internally | Removed from live watchlist. REST data works |
| Paradex | Per-symbol WebSocket attribution is unreliable (exchange uses a global channel) | WebSocket removed from live watchlist. REST works |
| Jupiter | Klines and orderbook impossible by design (aggregator, no historical data) | Swap APIs (Ultra, Trigger, Recurring) fully wired |
| GMX | `EvmProvider` not wired — trading requires EVM wallet but provider not attached | Positions/orders via Subsquid work. On-chain trading needs EvmProvider attachment |
| Futu | Requires OpenD local daemon | All methods return `UnsupportedOperation` until OpenD binary is running |

---

*Audit date: 2026-04-16. On-chain monitoring extracted to dig2chain. Version bumped to 0.1.17.*
