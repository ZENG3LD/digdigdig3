# digdigdig3

Multi-exchange connector library — unified async Rust API for 140 connectors spanning crypto exchanges, stock brokers, forex providers, intelligence feeds, and on-chain blockchain providers.

**Version:** 0.1.5
**Edition:** Rust 2021
**License:** MIT OR Apache-2.0
**Repository:** https://github.com/ZENG3LD/digdigdig3

---

## Overview

digdigdig3 is organized around capability levels. Each level describes what a connector can do and what it requires (credentials, chain infrastructure, etc.).

| Level | Description | Auth Required | Count |
|-------|-------------|--------------|-------|
| 1a | Market data feed — klines, orderbook, ticker, trades | No | 21 crypto + 6 other |
| 1b | Extended data feed — intelligence, aggregators, analytics | No or API key | 95 |
| 2 | Authenticated data feed — order history, account info, positions | Yes | ~50 |
| 3 | Execution — place/cancel orders, manage positions | Yes | ~30 |
| 4a | On-chain transport — signing and submitting transactions for DEX connectors | Yes (private key) | 5 |
| 4b | On-chain monitoring — direct blockchain data extraction | No | 8 chains |

**Total connectors:** 140
**Total source files (.rs):** 846
**Lines of code (src/):** ~258,000
**Blockchain chains supported:** 8 (EVM, Solana, Cosmos, Bitcoin, Aptos, StarkNet, Sui, TON)

---

## Adding as a Dependency

```toml
[dependencies]
digdigdig3 = { path = "../digdigdig3" }
# or
digdigdig3 = { git = "https://github.com/ZENG3LD/digdigdig3", tag = "v0.1.5" }
```

With specific feature flags:

```toml
[dependencies]
digdigdig3 = { version = "0.1.5", features = ["grpc", "k256-signing"] }
```

---

## Level 1a: Market Data Feed (no auth required)

Public endpoints — price, orderbook, klines, ticker, trades. No credentials needed.

These connectors implement `ExchangeIdentity` + `MarketData` + `WebSocketConnector`. All public streams (ticker, orderbook, klines, trades) are available without an API key.

### Crypto CEX — Tested on real data via live bridge

The following 21 connectors were validated against live exchange data in production use.

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
| Bitfinex | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bitstamp | Tested | Tested (poll/30s) | Untested | N/A | Tested | Tested | Untested | Untested |
| Gemini | Tested | Tested (poll/15s) | Untested | N/A | Tested | Tested | Untested | Untested |
| MEXC | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| HTX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Bitget | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| BingX | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Crypto.com | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Upbit | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Deribit | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| HyperLiquid | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| dYdX v4 | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |
| Lighter | Tested | Untested | Untested | Untested | Tested | Tested | Untested | Untested |
| MOEX ISS | Tested | Untested | Untested | N/A | Tested | Tested | Untested | Untested |

Notes:
- Bitstamp and Gemini supplement WebSocket ticker with a REST poll fallback (30s and 15s intervals respectively). That REST `get_ticker` call was exercised in production.
- `get_recent_trades` (REST trades column) is implemented only for Lighter. All other connectors return `UnsupportedOperation` and are marked N/A.
- WS orderbook and WS klines channels exist in code for most connectors but were never subscribed to in a live session.

### Crypto CEX — Implemented, NOT tested on real data

| Exchange | REST | WebSocket | Status |
|----------|------|-----------|--------|
| Phemex | Yes | Yes (code exists) | HTTP 403 on WebSocket upgrade from some regions. REST may work |
| Bithumb | Yes | Yes | Disabled — SSL timeouts and geo-blocking. 26 test files exist but are disabled |
| Vertex | Yes | Yes | Permanently disabled — exchange shut down 2025-08-14. Code retained for reference |

---

## Level 1b: Extended Data Feed

These connectors expose data that does not fit the standard klines/orderbook/ticker API. They implement `ExchangeIdentity` + `MarketData` for basic identity, but their real value is in domain-specific methods (e.g., `get_series()`, `get_liquidations()`, `get_articles()`).

**None of this was tested against real APIs.** All connectors compile and are structurally complete, but no live data validation has been done.

### Crypto Intelligence

| Provider | Domain Methods | Auth |
|----------|----------------|------|
| CoinGecko | Market cap, coin details, trending, global stats | API key optional |
| Coinglass | Liquidation data, open interest, long/short ratios, funding rates, fear and greed — ~50 endpoints | API key optional (rate limits differ) |
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
| J-Quants | Japan | Yes | Yes | No | Refresh/ID token |
| KRX | Korea | Yes | Yes | No | API key |
| Dukascopy | Forex | Yes | Yes | No | No auth (public) |
| Alpha Vantage | Forex/US | Yes | No | No | API key |

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

All 19 active CEX connectors listed in Level 1a also implement these authenticated read methods:

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

All 19 active CEX connectors have zero test files. This is the primary quality gap.

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
| HyperLiquid | Yes | Yes | Yes | Yes | No | No |
| Kraken | Yes | Yes | No | No | Yes | Yes |
| Bitfinex | Yes | Yes | Yes | Yes | Yes | No |
| Coinbase | Yes | No | No | No | Yes | No |
| Deribit | Yes | Yes | No | No | Yes | No |
| Gemini | Yes | No | No | No | Yes | No |
| Upbit | Yes | Yes | No | No | Yes | No |
| Bitstamp | Yes | Yes | No | No | Yes | No |
| Phemex | Yes | Yes | No | Yes | Yes | Yes |

### DEX Execution

| Connector | Execution Notes |
|-----------|----------------|
| dYdX v4 | Order placement via Cosmos gRPC — full trading trait implemented |
| Lighter | `place_order` trait is stub (`UnsupportedOperation`). Real signed orders via `place_order_signed()` using ZK-native signing — not yet exposed through the standard `Trading` trait |
| Paradex | Full trading trait implemented, StarkNet signing via `onchain-starknet` feature |
| GMX | Trading trait implemented but chain signing is not wired — `EvmProvider` is not attached to the connector |
| Jupiter | Trading methods are stubs — `UnsupportedOperation`. Only ticker and WebSocket work |

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
- `VaultManager` — vault deposit/withdraw (GMX/Paradex/dYdX/HyperLiquid)
- `StakingDelegation` — delegate/undelegate, claim rewards
- `BlockTradeOtc` — OTC block trade creation and execution
- `MarketMakerProtection` — MMP config, mass quoting
- `TriggerOrders` — conditional order placement

---

## Level 4: On-Chain Providers

Direct blockchain connectivity layer. Two distinct purposes.

### 4a: Transport and Auth for DEX Connectors

Chain providers used by DEX/swap connectors for transaction signing and submission.

| Provider | Chain | Feature Flag | Used By | Status |
|----------|-------|-------------|---------|--------|
| `EvmProvider` | Ethereum/EVM | `onchain-evm` (default) | GMX (not wired), Uniswap | Code exists; not attached to GMX connector |
| `SolanaProvider` | Solana | `onchain-solana` | Jupiter, Raydium | Wired in both connectors |
| `CosmosProvider` | Cosmos | `onchain-cosmos` | dYdX | Wired via gRPC channel |
| `StarkNetProvider` | StarkNet | `onchain-starknet` | Paradex | Optional feature, wired |

None of these have been tested end-to-end with real transaction submission.

### 4b: On-Chain Monitoring

Direct chain data extraction — read blockchain state, subscribe to events, decode transactions without going through an exchange API.

8 chain providers implemented:

| Provider | Feature Flag | Lines | Capabilities |
|----------|-------------|-------|-------------|
| `EvmProvider` | `onchain-evm` | 467 | Log subscriptions, eth_call, pending tx mempool, block data |
| `SolanaProvider` | `onchain-solana` | 410 | Transaction subscriptions, account monitoring, program interactions |
| `CosmosProvider` | `onchain-cosmos` | 1,410 | Tendermint WebSocket, IBC transfers, governance events, staking |
| `BitcoinProvider` | `onchain-bitcoin` | 672 | Block scanning, mempool, UTXO analysis, coinbase tx detection |
| `AptosProvider` | `onchain-aptos` | 851 | Module events, resource queries, coin transfers |
| `StarkNetProvider` | `onchain-starknet` | 604 | Contract calls, event monitoring, transaction signing |
| `SuiProvider` | `onchain-sui` | 923 | Move event subscriptions, object ownership, DeepBook events |
| `TonProvider` | `onchain-ton` | 929 | Jetton transfers, message parsing, DEX op-code detection |

4 transaction decoders:

| Decoder | File | Decodes |
|---------|------|---------|
| `EvmDecoder` | `src/core/chain/decoders/evm_decoder.rs` | ERC-20 transfers, Uniswap V2/V3 swaps, liquidity events |
| `SolanaDecoder` | `src/core/chain/decoders/solana_decoder.rs` | SPL token transfers, Raydium/Jupiter swap logs |
| `CosmosDecoder` | `src/core/chain/decoders/cosmos_decoder.rs` | IBC packets, governance events, staking operations |
| `BitcoinDecoder` | `src/core/chain/decoders/bitcoin_decoder.rs` | UTXO analysis, coinbase tx identification, OP_RETURN parsing |

The `OnChainEvent` enum with 17 event types is defined in `src/core/types/onchain.rs`:

```
LargeTransfer, DexSwap, LiquidityChange, ExchangeFlow, MempoolAlert,
BridgeTransfer, NewTokenLaunch, GovernanceEvent, ValidatorAlert,
StakingEvent, NftTransfer, ContractCall, GasAlert, BlockProduced,
MempoolCongestion, ChainReorg, SlashingEvent
```

The `EventProducer` trait is defined in `src/core/traits/event_stream.rs` with `get_events()` and `poll_events()` methods. No connector currently implements this trait — the infrastructure is in place but nothing is wired to it.

**Status: None of the on-chain providers have been tested. The provider and decoder layer compiles but has not been run against live nodes.**

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

On-chain:
    EventProducer (chain_id, get_events, poll_events) — defined, 0 implementations
    ChainProvider (abstract provider trait)
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

**How it works:**
- `safe_price(f64, tick)` — converts via `Decimal::from_str(price.to_string())`, rounds to nearest tick (like CCXT)
- `safe_qty(f64, step)` — same conversion, floors to step_size (never exceeds available quantity)
- `PrecisionCache` — per-symbol HashMap loaded from `get_exchange_info()`, stores tick/step per symbol
- Fallback: raw `f64::to_string()` if symbol not in cache (backwards compatible)

**Why not just f64::to_string()?** — `100.05_f64` is stored as `100.04999...` in IEEE-754. With floor rounding, this loses a full tick. The string-path via Ryu shortest-round-trip eliminates this drift.

**tick_size sources:** 23 parsers extract real tick_size from exchange APIs (Binance PRICE_FILTER, Bybit priceFilter, OKX tickSz, etc.). Remaining connectors fall back to `price_precision` integer digits.

### Connector Manager

The connector manager (`src/connector_manager/`) provides a runtime pool for managing multiple connectors simultaneously:

| Component | Lines | Purpose |
|-----------|-------|---------|
| `ConnectorHandle` | 1,101 | Unified dynamic dispatch wrapper for any connector |
| `ConnectorRegistry` | 1,946 | Maps `ExchangeId` to connector instances |
| `ConnectorFactory` | 1,002 | Builds connectors from `ConnectorConfig` |
| `ConnectorConfig` | 1,039 | Per-exchange config structs |
| `ConnectorPool` | 685 | DashMap-based concurrent pool |
| `ConnectorAggregator` | 844 | Multi-exchange data fan-out |

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
| REST (reqwest) | default | All 140 connectors |
| WebSocket | `websocket` | All 21 CEX + DEX + major stock brokers |
| gRPC (tonic) | `grpc` | dYdX (Cosmos gRPC), Tinkoff Invest |
| GraphQL | default | BitQuery |
| On-chain EVM RPC | `onchain-evm` | GMX, Uniswap |
| On-chain Solana RPC | `onchain-solana` | Jupiter, Raydium |
| On-chain Cosmos gRPC | `onchain-cosmos` | dYdX |
| On-chain StarkNet | `onchain-starknet` | Paradex |

HTTP client: `src/core/http/client.rs` — async reqwest wrapper with auth injection, retry logic, and rate limiting (`SimpleRateLimiter` token bucket and `WeightRateLimiter`).

WebSocket base: `src/core/websocket/base_websocket.rs` — handles reconnect, ping/pong, and subscription replay on reconnect.

---

## Auth Methods

| Method | Connectors |
|--------|-----------|
| HMAC-SHA256 | Binance, Bybit, OKX, KuCoin, Gate.io, Bitget, BingX, MEXC, HTX, Bitstamp, Deribit, Crypto.com, Phemex |
| HMAC-SHA384 | Bitfinex, Gemini |
| HMAC-SHA512 | Kraken |
| HMAC + passphrase | OKX, KuCoin, Bitget (additional passphrase layer on top of HMAC) |
| JWT ES256 (EC P-256) | Coinbase, Upbit, Tinkoff, Dhan, J-Quants |
| JWT + TOTP | Angel One |
| OAuth2 / Bearer token | Upstox, Fyers, Zerodha, BitQuery, Whale Alert, OANDA, IB |
| EIP-712 (Ethereum typed data) | HyperLiquid, Uniswap, GMX |
| Ed25519 / Solana keypair | Raydium, Lighter |
| Cosmos SDK wallet | dYdX |
| StarkNet ECDSA (STARK key) | Paradex |
| API key in header | Polygon, Finnhub, Tiingo, most intelligence feeds |
| No auth | MOEX ISS, Dukascopy, DeFiLlama, most public intelligence feeds |

---

## Feature Flags

| Feature | Dependencies Enabled | Notes |
|---------|---------------------|-------|
| `default` | `onchain-evm` | EVM provider is included by default |
| `onchain-evm` | `alloy` (provider-ws, rpc-types) | Ethereum/EVM chain providers |
| `onchain-ethereum` | `onchain-evm` | Backward-compat alias |
| `onchain-solana` | `solana-sdk`, `solana-client`, `solana-account-decoder` | Solana chain |
| `onchain-cosmos` | `cosmrs` (bip32) | Cosmos ecosystem (dYdX, Osmosis) |
| `onchain-starknet` | `starknet-crypto` | StarkNet chain |
| `onchain-bitcoin` | `bitcoin` v0.32 (serde, std) | Bitcoin JSON-RPC |
| `onchain-sui` | none (pure reqwest REST) | Sui JSON-RPC |
| `onchain-ton` | none (pure reqwest REST) | TON REST — no C++ FFI |
| `onchain-aptos` | none (pure reqwest REST) | Aptos REST — avoids tokio_unstable |
| `starknet` | `onchain-starknet` | Legacy alias |
| `websocket` | — | WebSocket enablement flag |
| `grpc` | `tonic` (tls, tls-native-roots), `prost` | gRPC transport |
| `k256-signing` | `k256` (ecdsa-core, ecdsa) | k256 ECDSA signing |

---

## Statistics

| Category | Count |
|----------|-------|
| CEX connectors | 21 (19 active, 2 disabled) |
| DEX connectors | 5 (3 active, 2 WS-disabled) |
| Swap protocols | 2 |
| Stock brokers/providers | 15 |
| Forex providers | 3 |
| Aggregators | 4 |
| Intelligence feed connectors | 85 |
| On-chain analytics connectors | 3 |
| Prediction markets | 1 |
| **Total** | **139** |
| Total .rs source files | 846 |
| Lines of code (src/) | ~258,000 |
| Blockchain chains | 8 |
| Core traits | 13 |
| Connectors with zero test coverage | ~120 |

---

## Known Issues and Disabled Connectors

| Connector | Issue | Status |
|-----------|-------|--------|
| Vertex | Exchange shut down 2025-08-14, acquired by Ink Foundation | Permanently disabled. Code retained as reference. 20 test files exist but are disabled |
| Bithumb | SSL handshake timeouts and HTTP 403 geo-blocking | Temporarily disabled. Code complete. 26 test files disabled. Re-enable when access resolves |
| Phemex | HTTP 403 on WebSocket upgrade from restricted regions | REST may still work. WebSocket removed from live watchlist. Code retained |
| GMX | No real-time WebSocket API — websocket.rs does REST polling internally | Removed from live watchlist. REST data works |
| Paradex | Per-symbol WebSocket attribution is unreliable (exchange uses a global channel) | WebSocket removed from live watchlist. REST works |
| Jupiter | Severely incomplete — klines, orderbook, all trading ops are stubs | Only ticker and WebSocket currently functional |
| Lighter | `place_order` via the `Trading` trait is `UnsupportedOperation` | Real signed orders via internal `place_order_signed()` method — not exposed through standard trait yet |
| GMX | `EvmProvider` not wired — trading requires EVM wallet but provider not attached | Trading trait compiles but chain signing path is broken at runtime |
| Futu | Requires OpenD local daemon | All methods return `UnsupportedOperation` until OpenD binary is running |

---

## Roadmap and Known Gaps

Testing is the main gap. 19 active CEX connectors and 85 intelligence feed connectors have zero test files.

| Priority | Item |
|----------|------|
| High | Add unit tests for the 19 active CEX connectors (Binance first, then Bybit/OKX) |
| High | Wire `EvmProvider` to GMX connector for actual transaction submission |
| High | Expose Lighter's `place_order_signed()` through the standard `Trading` trait |
| High | Implement `get_recent_trades` for the 18 CEX connectors missing it |
| Medium | Add WebSocket implementations for India stock brokers (Zerodha, Upstox, Angel One, Fyers) |
| Medium | Implement `EventProducer` for at least EVM and Solana providers |
| Medium | Complete Jupiter — klines, orderbook, and trading need real implementation |
| Low | Add missing CEX connectors: AscendEX, BigONE, ProBit, BitMart, CoinEx, DigiFinex, WOO X, XT.com, LBank, HashKey, WhiteBIT, BTSE |
| Low | Interactive Brokers proper brokerage integration (currently only aggregator/Web API mode) |
| Low | Override optional operation traits (MarginTrading, EarnStaking, ConvertSwap, etc.) for exchanges that support them |
| Low | Test on-chain providers against live nodes |

---

*Audit date: 2026-03-14. When adding a connector, update `src/LIBRARY_INVENTORY.md` and `src/MATURITY_MATRIX.md` accordingly.*
