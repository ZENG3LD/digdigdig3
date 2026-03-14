# Audit: Existing Analytics & Monitoring Capabilities

**Date:** 2026-03-14
**Scope:** All analytics, monitoring, and intelligence connectors in the codebase.
**Working Directory:** `digdigdig3/src/`

---

## 1. Summary Table

| Connector | Path | Category | Transport | Key Data | Status |
|-----------|------|----------|-----------|----------|--------|
| **Bitquery** | `onchain/analytics/bitquery/` | Blockchain analytics | GraphQL (REST + WS URL) | DEX trades, token transfers, balance updates, blocks, txns, smart contract events | FULL |
| **Whale Alert** | `onchain/analytics/whale_alert/` | Whale monitor | REST + WebSocket | Large-txn alerts, address attribution, block data | FULL + WS |
| **Etherscan** | `onchain/ethereum/etherscan/` | Block explorer | REST | ETH/ERC20 balances, txns, gas oracle, ETH price, contract ABI | FULL |
| **Coinglass** | `intelligence_feeds/crypto/coinglass/` | Derivatives analytics | REST + WS | Liquidations, OI, funding rates, L/S ratios, orderbook analytics, ETF flows, on-chain transfers | FULL |
| **CoinGecko** | `intelligence_feeds/crypto/coingecko/` | Price aggregator | REST only | Coin prices, market caps, trending, DeFi global, stablecoin data | FULL |
| **DefiLlama** | `aggregators/defillama/` | DeFi aggregator | REST | TVL (all chains/protocols), token prices, stablecoins, yields, DEX volumes, fees | FULL |

---

## 2. Detailed Connector Capabilities

### 2.1 Bitquery (`onchain/analytics/bitquery/`)

**Type:** GraphQL multi-chain blockchain data provider
**Auth:** OAuth Bearer token
**Rate limit:** 10 req/min (free), configurable for commercial

**Methods (all custom, no standard MarketData/Trading):**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_dex_trades(network, protocol, buy, sell, limit)` | `Vec<DexTrade>` | DEX trade history — archive dataset |
| `get_realtime_dex_trades(network, protocol, limit)` | `Vec<DexTrade>` | DEX trades — realtime dataset |
| `get_token_transfers(network, currency, sender, receiver, limit)` | `Vec<TokenTransfer>` | ERC20 token transfer events |
| `get_balance_updates(network, address, limit)` | `Vec<BalanceUpdate>` | On-chain balance deltas for address |
| `get_blocks(network, limit)` | `Vec<BlockData>` | Block metadata (number, time, gas, coinbase) |
| `get_transactions(network, hash, from, to, limit)` | `Vec<TransactionData>` | Transaction data with receipt |
| `get_smart_contract_events(network, contract, event_name, limit)` | `Vec<SmartContractEvent>` | Smart contract event logs with arguments |

**Key data types:**
- `DexTrade` — buy/sell sides (amount, price, priceInUSD, currency, buyer, seller), DEX info (protocolName, protocolFamily), block/tx refs
- `TokenTransfer` — amount, sender, receiver, currency (symbol, name, contract, decimals), type (ERC20/NFT), NFT tokenId
- `BalanceUpdate` — address, amount, update type, currency, tx/block refs
- `BlockData` — number, time, hash, gasLimit, gasUsed, baseFee, coinbase, txCount, size
- `TransactionData` — hash, from, to, value, gas, gasPrice, gasUsed, nonce, type, receipt (status, gasUsed, effectiveGasPrice)
- `SmartContractEvent` — log (signature, name, contract), typed arguments (name, type, value)

**Networks supported:** 20+ EVM + Solana + Bitcoin + Cosmos (Bitquery multi-chain)

**WebSocket:** URL stored in `BitqueryUrls.websocket` — not wired into a `WebSocketConnector` implementation. No WebSocket connector struct exists yet.

---

### 2.2 Whale Alert (`onchain/analytics/whale_alert/`)

**Type:** Blockchain whale transaction tracker
**Auth:** API key (query param)
**Versions:** Enterprise API v2 (default) + Developer API v1 (deprecated, partial)

**REST Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_status()` | `StatusResponse` | Supported blockchains map |
| `get_blockchain_status(blockchain)` | `BlockchainStatus` | Block height range for blockchain |
| `get_transaction(blockchain, hash)` | `WhaleTransaction` | Single tx by hash (v2 only) |
| `get_transactions(blockchain, start_height, symbol, type, limit)` | `Vec<WhaleTransaction>` | Transactions from block height |
| `get_block(blockchain, height)` | `WhaleBlock` | Full block with all transactions |
| `get_address_transactions(blockchain, address)` | `Vec<WhaleTransaction>` | 30-day address tx history |
| `get_address_attributions(blockchain, address)` | `Vec<OwnerAttribution>` | Owner/type attribution for address |

**Key data types:**
- `WhaleTransaction` — height, index_in_block, timestamp, hash, fee, fee_symbol, fee_symbol_price, `Vec<SubTransaction>`
- `SubTransaction` — symbol, unit_price_usd, transaction_type, inputs/outputs (`Vec<Address>`)
- `Address` — amount, address, balance, locked, is_frozen, owner, owner_type, address_type
- `OwnerAttribution` — owner (label), owner_type, address_type, confidence (0-1 float)
- `WhaleBlock` — blockchain, height, timestamp, transaction_count, `Vec<WhaleTransaction>`
- `StatusResponse` — `HashMap<blockchain, Vec<symbol>>`

**WebSocket (WIRED, production-quality):**
- URL: `wss://leviathan.whale-alert.io/ws?api_key=KEY`
- Subscription types: `subscribe_alerts` (min $100K USD threshold) + `subscribe_socials`
- `WhaleAlert` — channel_id, timestamp, blockchain, tx_type, from, to, `Vec<AlertAmount>`, text, full `WhaleTransaction`
- `SocialAlert` — blockchain, text, urls
- Implements `WebSocketConnector` trait with reconnect, broadcast channel (1000 capacity)
- Supports filtering by blockchain, symbol, tx_type, min_value_usd

**Blockchains:** BTC, ETH, SOL, POLY, XRP, ADA, TRX, and more

---

### 2.3 Etherscan (`onchain/ethereum/etherscan/`)

**Type:** Ethereum block explorer API
**Auth:** API key (query param `apikey`)
**Networks:** Mainnet + Sepolia testnet

**Methods:**

| Method | Returns | Description |
|--------|---------|-------------|
| `get_balance(address)` | `String` (Wei) | ETH balance for address |
| `get_multi_balance(&[address])` | `Vec<EthBalance>` | Batch ETH balances (up to 20) |
| `get_transactions(address, start_block, end_block, page, limit)` | `Vec<EthTransaction>` | Normal tx list |
| `get_internal_transactions(address, ...)` | `Vec<EthTransaction>` | Internal tx list |
| `get_token_transfers(address, contract_address, page, limit)` | `Vec<TokenTransfer>` | ERC20 transfer events |
| `get_eth_price()` | `EthPrice` | ETH/USD + ETH/BTC with timestamps |
| `get_eth_supply()` | `String` (Wei) | Total ETH supply |
| `get_chain_size()` | `String` | Blockchain size in bytes |
| `get_token_supply(contract)` | `String` | ERC20 total supply |
| `get_gas_oracle()` | `GasOracle` | Safe/Propose/Fast gas prices in Gwei |
| `get_latest_block_number()` | `String` (hex) | Current block height |
| `get_block_reward(block)` | `BlockReward` | Miner reward + uncle rewards |
| `get_contract_abi(address)` | `String` (JSON) | Smart contract ABI |

**Key data types:**
- `EthTransaction` — blockNumber, timestamp, hash, from, to, value, gas, gasPrice, gasUsed, isError, input
- `TokenTransfer` — blockNumber, timestamp, hash, from, to, value, tokenName, tokenSymbol, tokenDecimal, contractAddress
- `EthPrice` — ethbtc, ethbtc_timestamp, ethusd, ethusd_timestamp
- `GasOracle` — lastBlock, safeGasPrice, proposeGasPrice, fastGasPrice, suggestBaseFee, gasUsedRatio
- `BlockReward` — blockNumber, timestamp, blockMiner, blockReward, `Vec<UncleReward>`

**No WebSocket.** No standard `CoreConnector` trait implementations (only `ExchangeError` types used).

---

### 2.4 Coinglass (`intelligence_feeds/crypto/coinglass/`)

**Type:** Crypto derivatives analytics
**Auth:** API key in `CG-Api-Key` header
**Rate limits:** 30/min (Hobbyist), 80/min (Startup), configurable via `WeightRateLimiter`

**Endpoint categories (all implemented):**

| Category | Endpoints | Key Methods |
|----------|-----------|-------------|
| Market Discovery | SupportedCoins, SupportedExchangePairs, PairsMarkets, CoinsMarkets | `get_supported_coins()` |
| Liquidations | LiquidationHistory, LiquidationHeatmap, LiquidationMap, LiquidationMaxPain | `get_liquidation_history(symbol, interval, limit)` |
| Open Interest | OpenInterestOhlc, OpenInterestAggregated, OpenInterestHistory, OpenInterestVolRatio, OpenInterestByCoin | `get_open_interest_ohlc(symbol, interval, limit)` |
| Funding Rates | FundingRateHistory, FundingRateCurrent, FundingRateAggregated | `get_funding_rate_history(symbol, exchange, limit)` |
| Long/Short | LongShortRateHistory, LongShortAccountRatio, LongShortGlobalAccountRatio, TopLongShortPositionRatio, TopLongShortAccountRatio, TakerBuySellVolume | `get_long_short_ratio(symbol, interval, limit)` |
| Order Book Analytics | BidAskRange, OrderbookHeatmap, LargeOrders | — |
| Volume & Flows | CumulativeVolumeDelta, NetFlowIndicator, FootprintChart | — |
| Options | OptionsMaxPain, OptionsOiHistory, OptionsVolumeHistory | — |
| On-Chain | ExchangeReserve, ExchangeBalanceHistory, Erc20Transfers, WhaleTransfers, TokenUnlocks, TokenVesting | — |
| ETF | BtcEtfFlow, EthEtfFlow, SolEtfFlow, XrpEtfFlow, HkEtfFlow, GrayscalePremium | — |
| HyperLiquid | HyperLiquidWhaleAlert, HyperLiquidWhalePositions, HyperLiquidWalletPositions, HyperLiquidPositionDistribution | — |
| Technical Indicators | Rsi, MovingAverage | — |

**Key data types:**
- `LiquidationData` — timestamp, symbol, side (long/short), price, quantity, value_usd, exchange
- `OpenInterestOhlc` — timestamp, open, high, low, close (aggregated OI across exchanges)
- `FundingRateData` — timestamp, symbol, exchange, funding_rate, next_funding_time
- `LongShortRatio` — timestamp, long_rate, short_rate, long_account, short_account

**Note:** Many endpoint paths are defined (50+) but only 4 custom methods are wired through to parsers. The rest of the endpoints need connector method implementations.

---

### 2.5 CoinGecko (`intelligence_feeds/crypto/coingecko/`)

**Type:** Cryptocurrency price aggregator
**Auth:** Optional API key (`x-cg-demo-api-key` / `x-cg-pro-api-key`)
**WebSocket:** None (CoinGecko REST-only)

**Endpoints defined:**

| Category | Endpoints |
|----------|-----------|
| Simple | SimplePrice |
| Coins | CoinsList, CoinDetail, CoinMarketChart, CoinsMarkets, CoinTickers |
| Search | Search, SearchTrending |
| Global | Global, GlobalDefi |
| Exchanges | Exchanges, ExchangeDetail |

**Provides:** coin prices, market caps, volume, ATH, 24h change, trending, global market data, DeFi sector stats, exchange listings

---

### 2.6 DefiLlama (`aggregators/defillama/`)

**Type:** DeFi multi-chain aggregator
**Auth:** Optional API key for Pro tier
**WebSocket:** None

**Endpoint categories:**

| Category | Endpoints |
|----------|-----------|
| Protocols | Protocols, Protocol, ProtocolTvl |
| TVL | TvlAll (per chain), ChainTvl |
| Prices | PricesCurrent, PricesHistorical, PricesFirst |
| Stablecoins | Stablecoins, Stablecoin, StablecoinCharts, StablecoinChain |
| Yields | YieldPools, YieldPoolChart |
| Fees/Revenue | ProtocolFees |
| Volumes | DexVolumes |
| Pro Only | ProAnalytics |

**Provides:** Protocol TVL, historical TVL per chain, token prices (cross-chain by contract), stablecoin market cap/peg tracking, yield pool APYs, DEX volume, protocol fee revenue

---

## 3. Intelligence Feeds Inventory (Full)

The `intelligence_feeds/` module contains 88 feed connectors across 18 categories. The `FeedId` enum enumerates all of them. Categories most relevant to trading analytics:

### Crypto (5 connectors — all in FeedId)
- `CoinGecko` — price aggregator
- `Coinglass` — derivatives analytics
- `Bitquery` — blockchain analytics (also in `onchain/analytics/`)
- `Etherscan` — Ethereum explorer (also in `onchain/ethereum/`)
- `WhaleAlert` — whale monitor (also in `onchain/analytics/`)

### Financial (4 connectors)
- `AlphaVantage` (`intelligence_feeds/financial/alpha_vantage/`) — stocks, forex, crypto, indicators
- `Finnhub` (`intelligence_feeds/financial/finnhub/`) — stocks, news, fundamentals
- `NewsApi` (`intelligence_feeds/financial/newsapi/`) — financial news
- `OpenFigi` (`intelligence_feeds/financial/openfigi/`) — financial instrument identifiers

### Economic (12 connectors)
- `Fred` — US Federal Reserve economic data (St. Louis Fed)
- `Ecb`, `Boe`, `Bundesbank`, `Cbr` — central bank data
- `Bis` — Bank for International Settlements
- `Imf` — International Monetary Fund
- `WorldBank` — World Bank indicators
- `Oecd` — OECD statistics
- `Eurostat` — EU statistics
- `DBnomics` — multi-source economic data aggregator
- `Ecos` — South Korean central bank

### Cyber/Security (9 connectors) — relevant for DeFi risk intelligence
- `Shodan` — internet-connected devices
- `VirusTotal` — malware / phishing address scanning
- `Censys` — network infrastructure analysis
- `AbuseIpdb` — IP reputation
- `OtxAlienVault` — threat intelligence
- `Nvd` — vulnerability database
- `CloudflareRadar` — internet traffic patterns
- `RipeNcc` — network address registry
- `Urlhaus` — malicious URL tracking

### Sanctions (3 connectors) — critical for compliance
- `Ofac` — US Treasury sanctions list
- `Interpol` — international law enforcement alerts
- `OpenSanctions` — aggregated global sanctions

---

## 4. What Already EXISTS vs What's MISSING

### EXISTS (working connectors with full implementations)

| Capability | Connector | Notes |
|-----------|-----------|-------|
| DEX trade analytics (multi-chain) | Bitquery | GraphQL, 20+ chains |
| Whale transaction tracking (REST) | Whale Alert | Enterprise v2 + deprecated v1 |
| Whale transaction tracking (WebSocket) | Whale Alert WS | Real-time, min $100K threshold |
| On-chain balance updates | Bitquery | Per-address history |
| Smart contract event monitoring | Bitquery | Any contract, any event |
| Ethereum account balances | Etherscan | Single + batch |
| Ethereum transaction history | Etherscan | Normal + internal |
| ERC20 transfer history | Etherscan | Per-address + per-contract filter |
| Gas oracle (Gwei prices) | Etherscan | Safe/Propose/Fast |
| ETH price (USD + BTC) | Etherscan | REST, not real-time |
| Contract ABI fetching | Etherscan | By address |
| Liquidation data (futures) | Coinglass | History, heatmap, map, max-pain |
| Open Interest OHLC | Coinglass | Aggregated cross-exchange |
| Funding rate history | Coinglass | Per exchange or aggregated |
| Long/Short ratio | Coinglass | Account + position ratios |
| DeFi TVL | DefiLlama | Per protocol + per chain |
| Token prices (multi-chain) | DefiLlama | Cross-chain contract addresses |
| Stablecoin peg tracking | DefiLlama | Market cap + peg deviation |
| Yield pool APYs | DefiLlama | All DeFi protocols |
| DEX volume analytics | DefiLlama | Protocol-level volume |
| Crypto market caps | CoinGecko | All coins |
| Trending coins | CoinGecko | Real-time |
| Exchange rate data | CoinGecko | Per exchange |

### MISSING — Endpoints defined but connector methods not wired

| Capability | Connector | Gap |
|-----------|-----------|-----|
| Coinglass orderbook heatmap | Coinglass | Endpoint defined, no method |
| Coinglass CVD | Coinglass | Endpoint defined, no method |
| Coinglass ETF flows (BTC/ETH/SOL/XRP) | Coinglass | Endpoints defined, no methods |
| Coinglass Hyperliquid whale positions | Coinglass | Endpoints defined, no methods |
| Coinglass options analytics (max pain, OI, volume) | Coinglass | Endpoints defined, no methods |
| Coinglass token unlocks/vesting | Coinglass | Endpoints defined, no methods |
| Coinglass exchange reserves | Coinglass | Endpoints defined, no methods |
| Coinglass on-chain ERC20/whale transfers | Coinglass | Endpoints defined, no methods |

### MISSING — No connector exists at all

| Capability | Possible Source | Notes |
|-----------|-----------------|-------|
| Mempool monitoring | Blocknative, Alchemy WS | Zero connectors — critical gap |
| Real-time DEX liquidity depth | Uniswap V3 subgraph | Bitquery can approximate via GraphQL |
| NFT whale tracking | Bitquery | Could reuse existing connector |
| MEV/sandwich tracking | EigenPhi, Zeromev API | No connector |
| Cross-chain bridge flows | Li.Fi, Socket.tech | No connector |
| Exchange wallet inflow/outflow | CryptoQuant, Glassnode | No connector |
| On-chain MVRV / NVT ratio | Glassnode | No connector (paid) |
| Address clustering / entity resolution | Arkham, Chainalysis | No connector |
| Solana program event monitoring | Helius, QuickNode | No connector |
| Token launch detection | Bitquery realtime | Possible via existing connector |
| Rug pull detection | No structured connector | Pattern: LP removal + sell pressure |
| DEX whale alerts (Uniswap, Raydium) | Bitquery realtime | Possible via existing connector |
| Governance votes (on-chain) | Tally, Snapshot APIs | No connector |
| Protocol hack / exploit alerts | Forta, DeFiLlama hacks | No connector (DefiLlama partial) |

---

## 5. WebSocket Capabilities Summary

| Connector | WS Implemented | WS Available | Details |
|-----------|---------------|-------------|---------|
| Whale Alert | YES, FULL | YES | `wss://leviathan.whale-alert.io/ws` — full WebSocketConnector impl |
| Bitquery | NO (WS URL stored) | YES | URL in `BitqueryUrls.websocket` — not wired into connector |
| Coinglass | NO | YES | `wss://open-ws.coinglass.com/ws-api` — URL in `CoinglassUrls.ws` |
| Etherscan | NO | NO | REST-only service |
| CoinGecko | NO | NO | REST-only service |
| DefiLlama | NO | NO | REST-only service |

---

## 6. Key Structural Observations

1. **No dedicated analytics trait exists** — all analytics/monitoring connectors implement the standard `CoreConnector` traits (MarketData, Trading, Account, Positions) as `UnsupportedOperation` stubs. The actual data is exposed as struct methods. There is no `OnChainAnalytics`, `WhaleMonitor`, or `DerivativesAnalytics` trait.

2. **FeedId enum is the registry** — `intelligence_feeds/feed_manager/feed_id.rs` lists 88 connectors. The 5 crypto-specific analytics connectors (Bitquery, Coinglass, CoinGecko, Etherscan, WhaleAlert) are listed under `crypto` category in this enum despite some being in `onchain/` path.

3. **Bitquery + Coinglass are the most powerful analytics connectors** — Bitquery for raw on-chain data across 20+ chains; Coinglass for derivatives market microstructure across all major CEXes.

4. **Whale Alert is the only connector with both REST and working WebSocket** — the WS connector uses `broadcast::channel(1000)` and implements `WebSocketConnector` trait properly.

5. **Coinglass has the most endpoint coverage** (50+ endpoint variants) but the fewest wired connector methods (4 methods). Significant surface area to expose.

6. **No mempool connector exists anywhere** — this is the largest gap for latency-sensitive analytics.

7. **Sanctions and cyber feeds** (OFAC, VirusTotal, Shodan) could be used for address screening and smart contract risk scoring — not currently exposed as a composed service.

---

## 7. File Reference

| File | Description |
|------|-------------|
| `onchain/analytics/bitquery/connector.rs` | Bitquery connector — 7 custom methods |
| `onchain/analytics/bitquery/parser.rs` | Bitquery types: DexTrade, TokenTransfer, BalanceUpdate, BlockData, TransactionData, SmartContractEvent |
| `onchain/analytics/bitquery/endpoints.rs` | GraphQL query builders |
| `onchain/analytics/whale_alert/connector.rs` | Whale Alert REST — 7 methods |
| `onchain/analytics/whale_alert/websocket.rs` | Whale Alert WS — full impl, WhaleAlert + SocialAlert types |
| `onchain/analytics/whale_alert/parser.rs` | Whale Alert types: WhaleTransaction, SubTransaction, Address, OwnerAttribution |
| `onchain/ethereum/etherscan/connector.rs` | Etherscan — 12 methods |
| `onchain/ethereum/etherscan/parser.rs` | Etherscan types: EthTransaction, TokenTransfer, EthPrice, GasOracle, BlockReward |
| `intelligence_feeds/crypto/coinglass/connector.rs` | Coinglass — 4 wired methods + 50+ endpoint stubs |
| `intelligence_feeds/crypto/coinglass/endpoints.rs` | Full endpoint enum with all 50+ variants |
| `intelligence_feeds/crypto/coinglass/parser.rs` | Coinglass types: LiquidationData, OpenInterestOhlc, FundingRateData, LongShortRatio |
| `intelligence_feeds/crypto/coingecko/connector.rs` | CoinGecko connector |
| `intelligence_feeds/crypto/coingecko/endpoints.rs` | CoinGecko endpoint enum |
| `aggregators/defillama/connector.rs` | DefiLlama — TVL, prices, stablecoins, yields, volumes |
| `aggregators/defillama/endpoints.rs` | DefiLlama endpoint enum + URL routing |
| `intelligence_feeds/feed_manager/feed_id.rs` | FeedId enum — all 88 connectors |
| `AUDIT_ONCHAIN_MONITORS_AND_PROVIDERS.md` | Prior audit: ChainProvider needs per connector |
