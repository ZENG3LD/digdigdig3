# digdigdig3 — Complete Library Inventory

**Version:** 0.1.5
**Description:** Multi-exchange connector library — unified async Rust API for 40+ crypto exchanges, stock brokers, forex providers, and 88 intelligence feeds
**Date of Audit:** 2026-03-14

---

## 1. Top-Level Structure

```
digdigdig3/src/
├── lib.rs                     # Root: pub mod + re-exports
├── core/                      # Core infrastructure (traits, types, transport, utils)
├── connector_manager/         # Runtime connector pool, registry, factory
├── crypto/
│   ├── cex/                   # 21 centralized exchanges (2 disabled)
│   ├── dex/                   # 5 decentralized exchanges (3 partially disabled)
│   └── swap/                  # 2 swap protocols
├── stocks/
│   ├── china/                 # 1 broker (Futu)
│   ├── india/                 # 5 brokers
│   ├── japan/                 # 1 data provider
│   ├── korea/                 # 1 exchange
│   ├── russia/                # 2 providers
│   └── us/                    # 5 providers
├── forex/                     # 3 forex providers
├── aggregators/               # 4 multi-asset aggregators
├── intelligence_feeds/        # 80+ feeds across 23 categories
├── onchain/                   # 3 on-chain analytics providers
└── prediction/                # 1 prediction market (Polymarket)
```

**Total .rs source files:** 846
**Total lines of code (src/):** ~30,268

---

## 2. Cargo.toml Metadata

| Field | Value |
|-------|-------|
| name | digdigdig3 |
| version | 0.1.5 |
| edition | 2021 |
| license | MIT OR Apache-2.0 |
| repository | https://github.com/ZENG3LD/digdigdig3 |

---

## 3. Feature Flags

| Feature | Dependencies Enabled | Notes |
|---------|---------------------|-------|
| `default` | `onchain-evm` | Default includes EVM on-chain |
| `onchain-evm` | `alloy` (provider-ws, rpc-types) | Ethereum/EVM chain providers |
| `onchain-ethereum` | → `onchain-evm` | Backward-compat alias |
| `onchain-solana` | `solana-sdk`, `solana-client`, `solana-account-decoder` | Solana on-chain |
| `onchain-cosmos` | `cosmrs` (bip32) | Cosmos ecosystem (dYdX, Osmosis) |
| `onchain-starknet` | `starknet-crypto` | StarkNet chain |
| `onchain-bitcoin` | `bitcoin` v0.32 (serde, std) | Bitcoin JSON-RPC |
| `onchain-sui` | none (pure reqwest REST) | Sui JSON-RPC |
| `onchain-ton` | none (pure reqwest REST) | TON REST — no C++ FFI |
| `onchain-aptos` | none (pure reqwest REST) | Aptos REST — avoids tokio_unstable |
| `starknet` | → `onchain-starknet` | Legacy alias |
| `websocket` | none | WebSocket enablement flag |
| `grpc` | `tonic` (tls, tls-native-roots), `prost` | gRPC transport |
| `k256-signing` | `k256` (ecdsa-core, ecdsa) | k256 ECDSA signing |

---

## 4. Core Infrastructure

### 4.1 Traits (`src/core/traits/`)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 93 | Trait re-exports |
| `account.rs` | 41 | `Account` trait (balance, account_info) |
| `auth.rs` | 200 | `ExchangeAuth`, `Authenticated`, `CredentialKind`, `Credentials`, `AuthRequest`, `SignatureLocation` |
| `event_stream.rs` | 211 | `EventProducer`, `EventFilter` — streaming event traits |
| `identity.rs` | 72 | `ExchangeIdentity` — name, id, type |
| `market_data.rs` | 80 | `MarketData` — price, orderbook, klines, ticker, ping |
| `operations.rs` | 727 | Optional traits: `CancelAll`, `AmendOrder`, `BatchOrders`, `AccountTransfers`, `CustodialFunds`, `SubAccounts` |
| `positions.rs` | 148 | `Positions` — positions, funding_rate, set_leverage |
| `trading.rs` | 98 | `Trading` — market_order, limit_order, cancel, get_order, open_orders |
| `websocket.rs` | 142 | `WebSocketConnector`, `WebSocketExt` |
| **Total** | **1812** | |

### 4.2 Types (`src/core/types/`)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 24 | Type re-exports |
| `common.rs` | 488 | `ExchangeId`, `ExchangeType`, `AccountType`, `Symbol`, `ExchangeError`, `ExchangeResult`, `ExchangeCredentials`, `SymbolInfo` |
| `market_data.rs` | 166 | `Kline`, `Ticker`, `OrderBook`, `PublicTrade`, `FundingRate` |
| `trading.rs` | 1208 | `Order`, `OrderSide`, `OrderType`, `OrderStatus`, `Position`, `Balance`, etc. |
| `responses.rs` | 415 | `PlaceOrderResponse`, `OrderResult`, `CancelAllResponse`, `FeeInfo`, `TransferResponse`, etc. |
| `websocket.rs` | 331 | `ConnectionStatus`, `StreamType`, `SubscriptionRequest`, `StreamEvent`, update events |
| `onchain.rs` | 547 | `ChainId`, `OnChainEvent`, `OnChainEventType`, `TokenAmount`, `TokenInfo`, etc. |
| **Total** | **3179** | |

### 4.3 Chain Providers (`src/core/chain/`)

| File | Lines | Chain | Feature Flag |
|------|-------|-------|--------------|
| `mod.rs` | 78 | All | — |
| `provider.rs` | 199 | Abstract `ChainProvider` trait | — |
| `evm.rs` | 467 | Ethereum/EVM | `onchain-evm` |
| `solana.rs` | 410 | Solana | `onchain-solana` |
| `cosmos.rs` | 1410 | Cosmos ecosystem | `onchain-cosmos` |
| `bitcoin_chain.rs` | 672 | Bitcoin | `onchain-bitcoin` |
| `aptos_chain.rs` | 851 | Aptos L1 | `onchain-aptos` |
| `starknet_chain.rs` | 604 | StarkNet | `onchain-starknet` |
| `sui_chain.rs` | 923 | Sui | `onchain-sui` |
| `ton_chain.rs` | 929 | TON | `onchain-ton` |
| `decoders/evm_decoder.rs` | — | EVM TX decoder | — |
| `decoders/solana_decoder.rs` | — | Solana TX decoder | — |
| `decoders/cosmos_decoder.rs` | — | Cosmos TX decoder | — |
| `decoders/bitcoin_decoder.rs` | — | Bitcoin TX decoder | — |
| **Chain total** | **6543** | | |

**Chains supported:** EVM, Solana, Cosmos, Bitcoin, Aptos, StarkNet, Sui, TON (8 chains)

### 4.4 HTTP Transport (`src/core/http/`)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 14 | Re-exports |
| `client.rs` | 689 | `HttpClient` — async reqwest wrapper with auth, retry, rate limiting |
| `graphql.rs` | 89 | `GraphQlClient` — GraphQL over HTTP |
| **Total** | **792** | |

### 4.5 WebSocket Transport (`src/core/websocket/`)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 9 | Re-exports |
| `base_websocket.rs` | 675 | Base WebSocket implementation with reconnect, ping/pong |
| **Total** | **684** | |

### 4.6 gRPC Transport (`src/core/grpc/`) — feature `grpc`

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 18 | Re-exports |
| `client.rs` | 100 | `GrpcClient` — tonic-based gRPC client |
| **Total** | **118** | |

### 4.7 Utils (`src/core/utils/`)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 19 | Re-exports |
| `crypto.rs` | 86 | `hmac_sha256`, `hmac_sha256_hex`, `hmac_sha384`, `hmac_sha512`, `sha256`, `sha512` |
| `encoding.rs` | 43 | `encode_base64`, `encode_hex`, `encode_hex_lower` |
| `time.rs` | 45 | `timestamp_millis`, `timestamp_seconds`, `timestamp_iso8601` |
| `rate_limiter.rs` | 1003 | `SimpleRateLimiter`, `WeightRateLimiter` |
| **Total** | **1196** | |

---

## 5. Connector Manager (`src/connector_manager/`)

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | 67 | Re-exports |
| `connector.rs` | 1101 | `ConnectorHandle` — unified dynamic dispatch wrapper |
| `registry.rs` | 1946 | `ConnectorRegistry` — maps exchange IDs to connector instances |
| `factory.rs` | 1002 | `ConnectorFactory` — builds connectors from config |
| `config.rs` | 1039 | `ConnectorConfig` — per-exchange config structs |
| `pool.rs` | 685 | `ConnectorPool` — DashMap-based concurrent pool |
| `aggregator.rs` | 844 | `ConnectorAggregator` — multi-exchange data fan-out |
| `macros.rs` | 241 | Macro helpers for connector registration |
| **Total** | **6925** | |

---

## 6. Crypto: CEX Connectors (`src/crypto/cex/`)

**Total: 21 connectors** (2 disabled: Vertex permanently, Bithumb temporarily; Phemex and GMX in code but WS disabled)

| Exchange | .rs Files | connector.rs Lines | Traits Implemented | WS | Status |
|----------|-----------|--------------------|--------------------|----|--------|
| **binance** | 6 | 2138 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **bybit** | 6 | 2254 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **okx** | 6 | 1733 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **kucoin** | 6 | 2405 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **kraken** | 6 | 1471 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, CustodialFunds, SubAccounts | YES | Active |
| **coinbase** | 6 | 1377 | Identity, MarketData, Trading, Account, Positions, CancelAll, CustodialFunds | YES | Active |
| **hyperliquid** | 7 | 1682 | Identity, MarketData, Trading, Account, Positions, AmendOrder, BatchOrders, CancelAll, AccountTransfers | YES | Active |
| **bingx** | 6 | 1821 | Identity, MarketData, Trading, Account, Positions, CancelAll, BatchOrders, AmendOrder, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **bitfinex** | 6 | 1444 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds | YES | Active |
| **bitget** | 6 | 2499 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **bitstamp** | 6 | 948 | Identity, MarketData, Trading, Account, CancelAll, AmendOrder, CustodialFunds | YES | Active |
| **crypto_com** | 6 | 1678 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, CustodialFunds, SubAccounts | YES | Active |
| **deribit** | 6 | 1425 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, CustodialFunds | YES | Active |
| **gateio** | 6 | 2353 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **gemini** | 6 | 879 | Identity, MarketData, Trading, Account, Positions, CancelAll, CustodialFunds | YES | Active |
| **htx** | 6 | 1762 | Identity, MarketData, Trading, Account, Positions, CancelAll, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **mexc** | 6 | 1568 | Identity, MarketData, Trading, Account, CancelAll, BatchOrders, AccountTransfers, CustodialFunds, SubAccounts | YES | Active |
| **upbit** | 6 | 1098 | Identity, MarketData, Trading, Account, CancelAll, CustodialFunds, AmendOrder | YES | Active |
| **phemex** | 6 | 1723 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, AccountTransfers, CustodialFunds, SubAccounts | YES | WS disabled (geo-block) |
| **bithumb** | 8 | 988 | Identity, MarketData, Trading, Account, Positions, CustodialFunds | YES | DISABLED (SSL/geo-block) |
| **vertex** | 8 | 800 | Identity, MarketData, Trading, Account, Positions | YES | PERMANENTLY DISABLED (exchange shut down 2025-08-14) |

**All CEX connectors have research/ directories and WebSocket implementations.**

---

## 7. Crypto: DEX Connectors (`src/crypto/dex/`)

**Total: 5 connectors** (3 disabled from live watchlist due to WebSocket/data issues)

| Exchange | .rs Files | Traits Implemented | WS | Status |
|----------|-----------|--------------------|-----|--------|
| **dydx** | 8 | Identity, MarketData, Account, Positions, Trading | YES | Active |
| **gmx** | 7 | Identity, MarketData, Trading, Account, Positions | YES | WS disabled (no real-time WebSocket, on-chain only) |
| **jupiter** | 6 | Identity, MarketData, Trading, Account | YES | Active |
| **lighter** | 6 | Identity, MarketData, Trading, Account, Positions | YES | Active |
| **paradex** | 6 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder, BatchOrders | YES | WS disabled (global channel, per-symbol unreliable) |

---

## 8. Crypto: Swap Protocols (`src/crypto/swap/`)

**Total: 2 connectors**

| Protocol | .rs Files | Traits Implemented | WS | Status |
|----------|-----------|--------------------|-----|--------|
| **raydium** | 6 | Identity, MarketData, Trading, Account | YES | Active (Solana) |
| **uniswap** | 7 | Identity, MarketData, Trading, Account | YES | Active (EVM) |

---

## 9. Stocks (`src/stocks/`)

**Total: 15 connectors** across 6 regions

### US Stocks (`src/stocks/us/`)

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **alpaca** | 6 | Identity, MarketData, Trading, Account, Positions, CancelAll, AmendOrder | YES | no |
| **finnhub** | 6 | Identity, MarketData, Trading, Account, Positions | YES | no |
| **polygon** | 6 | Identity, MarketData, Trading, Account, Positions | YES | no |
| **tiingo** | 6 | Identity, MarketData, Trading, Account, Positions | YES | YES |
| **twelvedata** | 6 | Identity, MarketData, Trading, Account, Positions | YES | YES |

### India Stocks (`src/stocks/india/`)

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **angel_one** | 5 | Identity, MarketData, Trading, Account, Positions, AmendOrder | no | YES |
| **dhan** | 6 | Identity, MarketData, Trading, Account, Positions, AmendOrder | YES | YES |
| **fyers** | 6 | Identity, MarketData, Trading, Account, AmendOrder, BatchOrders | no | YES |
| **upstox** | 5 | Identity, MarketData, Trading, Account, Positions, AmendOrder | no | YES |
| **zerodha** | 5 | Identity, MarketData, Trading, Account, Positions, AmendOrder | no | YES |

### Russia Stocks (`src/stocks/russia/`)

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **moex** | 7 | Identity, MarketData, Trading, Account, Positions | YES | YES |
| **tinkoff** | 7 | Identity, MarketData, Trading, Account, Positions, AmendOrder | no | YES |

### China Stocks (`src/stocks/china/`)

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **futu** | 6 | Identity, MarketData, Trading, Account, Positions, AmendOrder | no | YES |

### Korea Stocks (`src/stocks/korea/`)

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **krx** | 6 | Identity, MarketData, Trading, Account, Positions | no | YES |

### Japan Stocks (`src/stocks/japan/`)

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **jquants** | 5 | Identity, MarketData, Trading, Account, Positions | no | YES |

---

## 10. Forex (`src/forex/`)

**Total: 3 providers**

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **alphavantage** | 6 | Identity, MarketData, Trading, Account, Positions | no | YES |
| **dukascopy** | 5 | Identity, MarketData, Trading, Account, Positions | no | YES |
| **oanda** | 6 | Identity, MarketData, Trading, Account, Positions, AmendOrder | no | no |

---

## 11. Aggregators (`src/aggregators/`)

**Total: 4 providers**

| Provider | .rs Files | Traits Implemented | WS | Research |
|----------|-----------|--------------------|-----|----------|
| **cryptocompare** | 6 | Identity, MarketData, Trading (stub), Account (stub), Positions (stub) | YES | YES |
| **defillama** | 5 | Identity, MarketData | no | no |
| **ib** | 6 | Identity, MarketData | YES | YES |
| **yahoo** | 6 | Identity, MarketData, Trading (stub), Account (stub), Positions (stub) | YES | YES |

---

## 12. Intelligence Feeds (`src/intelligence_feeds/`)

**Total: 80 feed providers** across 20 domain categories + 4 infrastructure modules

### Infrastructure Modules

| Module | Files | Purpose |
|--------|-------|---------|
| `feed_manager/factory.rs` | 614 lines | `FeedFactory` — builds feed connectors |
| `feed_manager/registry.rs` | 849 lines | `FeedRegistry` — maps FeedId to providers |
| `feed_manager/feed_id.rs` | 379 lines | `FeedId` enum — all 80+ feed IDs |
| `feed_manager/mod.rs` | 47 lines | Re-exports |
| `rss_proxy/` | 5 files | RSS/Atom feed proxy connector |
| `c2intel_feeds/` | 5 files | C2 intelligence feed aggregator |

### Academic Feeds (`academic/`)
- **arxiv** — arXiv preprint API
- **semantic_scholar** — Semantic Scholar research API

### Aviation Feeds (`aviation/`)
- **adsb_exchange** — ADS-B Exchange live aircraft tracking
- **aviationstack** — AviationStack flight data API
- **opensky** — OpenSky Network ADS-B data
- **wingbits** — Wingbits aviation intelligence

### Conflict / Geopolitical (`conflict/`)
- **acled** — Armed Conflict Location & Event Data
- **gdelt** — GDELT Global Database of Events
- **reliefweb** — OCHA ReliefWeb humanitarian data
- **ucdp** — Uppsala Conflict Data Program
- **unhcr** — UNHCR refugee/displacement data

### Corporate Data (`corporate/`)
- **gleif** — Global LEI Foundation (legal entity identifiers)
- **opencorporates** — OpenCorporates company registry
- **uk_companies_house** — UK Companies House

### Crypto Intelligence (`crypto/`)
- **coingecko** — CoinGecko market data
- **coinglass** — Coinglass liquidations/OI data

### Cyber Intelligence (`cyber/`)
- **abuseipdb** — AbuseIPDB IP reputation
- **alienvault_otx** — AlienVault OTX threat intelligence
- **censys** — Censys internet device scanning
- **cloudflare_radar** — Cloudflare Radar internet trends
- **nvd** — NIST National Vulnerability Database
- **ripe_ncc** — RIPE NCC internet registry
- **shodan** — Shodan device intelligence
- **urlhaus** — URLhaus malicious URL tracker
- **virustotal** — VirusTotal malware analysis

### Demographics (`demographics/`)
- **un_ocha** — UN OCHA humanitarian data
- **un_population** — UN Population Division
- **who** — World Health Organization
- **wikipedia** — Wikipedia statistics API

### Economic Data (`economic/`)
- **bis** — Bank for International Settlements
- **boe** — Bank of England
- **bundesbank** — Deutsche Bundesbank
- **cbr** — Central Bank of Russia
- **dbnomics** — DBnomics aggregated economic data
- **ecb** — European Central Bank
- **ecos** — Bank of Korea ECOS
- **eurostat** — Eurostat EU statistics
- **fred** — Federal Reserve FRED
- **imf** — International Monetary Fund
- **oecd** — OECD statistics
- **worldbank** — World Bank data API

### Environment / Disaster (`environment/`)
- **gdacs** — Global Disaster Alert and Coordination System
- **global_forest_watch** — Global Forest Watch deforestation
- **nasa_eonet** — NASA Earth Observatory Natural Event Tracker
- **nasa_firms** — NASA FIRMS active fire data
- **noaa** — NOAA weather and climate
- **nws_alerts** — National Weather Service alerts
- **open_weather_map** — OpenWeatherMap API
- **openaq** — OpenAQ air quality
- **usgs_earthquake** — USGS earthquake data

### FAA Status (`faa_status/`) — 5-file standalone connector

### Financial News (`financial/`)
- **alpha_vantage** — Alpha Vantage financial data
- **finnhub** — Finnhub financial news
- **newsapi** — NewsAPI news aggregator
- **openfigi** — OpenFIGI financial instrument IDs

### Feodo Tracker (`feodo_tracker/`) — 5-file standalone connector (botnet C&C tracker)

### Governance (`governance/`)
- **eu_parliament** — EU Parliament voting data
- **uk_parliament** — UK Parliament data

### Hacker News (`hacker_news/`) — 5-file standalone connector

### Maritime Intelligence (`maritime/`)
- **ais** — AIS vessel tracking
- **aisstream** — AISStream live vessel data
- **imf_portwatch** — IMF PortWatch shipping data
- **nga_warnings** — NGA maritime safety warnings

### NWS Alerts (`nws_alerts/`) — research directory present, implementation pending

### Prediction Markets (`prediction/`)
- **predictit** — PredictIt political prediction market

### Sanctions (`sanctions/`)
- **interpol** — INTERPOL wanted persons
- **ofac** — OFAC sanctions lists (US Treasury)
- **opensanctions** — OpenSanctions aggregated lists

### Space (`space/`)
- **launch_library** — Launch Library 2 rocket launches
- **nasa** — NASA open data APIs
- **sentinel_hub** — Sentinel Hub satellite imagery
- **space_track** — Space-Track.org orbital data
- **spacex** — SpaceX launch data

### Trade (`trade/`)
- **comtrade** — UN Comtrade international trade statistics
- **eu_ted** — EU TED (Tenders Electronic Daily)

### US Government (`us_gov/`)
- **bea** — Bureau of Economic Analysis
- **bls** — Bureau of Labor Statistics
- **census** — US Census Bureau
- **congress** — Congress.gov legislative data
- **eia** — Energy Information Administration
- **fbi_crime** — FBI Crime Data API
- **sam_gov** — SAM.gov federal contracts
- **sec_edgar** — SEC EDGAR filings
- **usaspending** — USASpending.gov federal spending

---

## 13. On-Chain Analytics (`src/onchain/`)

### Analytics Providers (`src/onchain/analytics/`)

| Provider | .rs Files | Traits Implemented |
|----------|-----------|--------------------|
| **bitquery** | 6 | Identity, MarketData, Trading (stub), Account (stub), Positions (stub) |
| **whale_alert** | 6 | Identity, MarketData, Trading (stub), Account (stub), Positions (stub) |

### Ethereum Chain (`src/onchain/ethereum/`)

| Provider | .rs Files | Traits Implemented |
|----------|-----------|--------------------|
| **etherscan** | 5 | Identity (partial) |

---

## 14. Prediction Markets (`src/prediction/`)

**Total: 1 connector**

| Provider | .rs Files | Traits Implemented | WS |
|----------|-----------|--------------------|-----|
| **polymarket** | 5 | connector.rs present | YES |

---

## 15. Tests

| Location | Type | Exchanges |
|----------|------|-----------|
| `src/crypto/cex/bithumb/_tests_integration.rs` | Integration (disabled) | Bithumb |
| `src/crypto/cex/bithumb/_tests_websocket.rs` | WS test (disabled) | Bithumb |
| `src/crypto/cex/vertex/_tests_integration.rs` | Integration (disabled) | Vertex |
| `src/crypto/cex/vertex/_tests_websocket.rs` | WS test (disabled) | Vertex |
| `src/forex/alphavantage/tests.rs` | Unit tests | AlphaVantage |
| `src/stocks/china/futu/tests.rs` | Unit tests | Futu |
| `src/stocks/india/fyers/tests.rs` | Unit tests | Fyers |
| `src/stocks/korea/krx/tests.rs` | Unit tests | KRX |
| `src/stocks/russia/moex/tests.rs` | Unit tests | MOEX |
| `src/stocks/russia/tinkoff/tests.rs` | Unit tests | Tinkoff |

**Note:** No root-level `tests/` directory. Tests are inline in `src/` modules. Bithumb and Vertex tests are stored inside connector directories (disabled) to prevent CI runs.

---

## 16. Statistics Summary

| Category | Count |
|----------|-------|
| CEX exchanges | 21 (19 active + 2 disabled) |
| DEX exchanges | 5 (3 active + 2 WS-disabled) |
| Swap protocols | 2 |
| Stock brokers/providers | 15 |
| Forex providers | 3 |
| Multi-asset aggregators | 4 |
| Intelligence feed providers | 80 |
| On-chain analytics | 3 |
| Prediction markets | 1 |
| **TOTAL connectors** | **134** |
| Total .rs source files | 846 |
| Total lines of code (src/) | ~30,268 |
| Supported blockchain chains | 8 (EVM, Solana, Cosmos, Bitcoin, Aptos, StarkNet, Sui, TON) |
| Core traits | 13 |

### Breakdown by .rs File Count

| Category | .rs Files |
|----------|-----------|
| Core infrastructure | ~80 |
| Connector manager | 8 |
| CEX (21 × ~6) | ~130 |
| DEX (5 × ~7) | ~35 |
| Swap (2 × ~6) | ~12 |
| Stocks (15 × ~6) | ~90 |
| Forex (3 × ~6) | ~18 |
| Aggregators (4 × ~6) | ~24 |
| Intelligence feeds (80 × 5 + infra) | ~420 |
| Onchain (3 × 6) | ~18 |
| Prediction (1 × 5) | 5 |
| Category mod.rs files | ~6 |
| **Total** | **~846** |

---

## 17. Architecture Notes

- **Pattern:** Traits + Utils (no config-based auth injection)
- **Each connector module:** `endpoints.rs`, `auth.rs`, `parser.rs`, `connector.rs`, `websocket.rs`, `mod.rs`
- **Core trait hierarchy:** `ExchangeIdentity` → `MarketData` → `Trading` → `Account` → `Positions` → optional traits
- **`CoreConnector`** = combined supertrait (`ExchangeIdentity + MarketData + Trading + Account + Positions`)
- **Authentication:** Each connector owns its signing logic — no shared auth config struct
- **Rate limiting:** `SimpleRateLimiter` (token bucket) and `WeightRateLimiter` (weight-based) in `core/utils/rate_limiter.rs`
- **Reference implementation:** `src/crypto/cex/kucoin/` — most complete example with all optional traits

---

*Generated by audit on 2026-03-14. Update when adding new connectors.*
