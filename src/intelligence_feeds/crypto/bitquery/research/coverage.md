# Bitquery - Data Coverage

## Geographic Coverage

### Regions Supported
- **Global**: Yes - Bitquery is blockchain data (no geographic restrictions on data access)
- **North America**: Yes
- **Europe**: Yes
- **Asia**: Yes
- **Africa**: Yes
- **South America**: Yes
- **Australia/Oceania**: Yes

**Note**: Bitquery provides blockchain data which is globally distributed by nature. There are no regional data boundaries.

---

### Country-Specific Access

**Data Access**: Available worldwide (blockchain data is borderless)

**API Access Restrictions**:
- No explicit country blocks mentioned in documentation
- OFAC/sanctions compliance likely applies (U.S. sanctioned countries)
- VPN detection: Not mentioned (likely not enforced for data access)
- Geo-fencing: No (blockchain data is global)

**Potential Restrictions** (not confirmed, but typical for crypto services):
- North Korea: Likely blocked
- Iran: Possibly restricted
- Syria: Possibly restricted
- Other sanctioned territories: May be restricted

**Recommendation**: Check terms of service or contact support if accessing from sanctioned regions.

---

### Restricted Regions
- **Blocked countries**: Not explicitly documented
- **VPN detection**: Not mentioned (likely allowed)
- **Geo-fencing**: No (API is globally accessible)
- **Compliance**: Standard crypto industry compliance (OFAC, AML)

---

## Markets/Exchanges Covered

**IMPORTANT**: Bitquery is NOT a centralized exchange aggregator. It provides **on-chain blockchain data**.

### Decentralized Exchanges (DEXs) - **PRIMARY COVERAGE**

Bitquery tracks 40+ DEX protocols across multiple blockchains:

#### Ethereum DEXs
- **Uniswap** (V1, V2, V3, V4)
- **SushiSwap**
- **Balancer** (V1, V2)
- **Curve Finance**
- **1inch** (aggregator)
- **0x Protocol**
- **Kyber Network**
- **Bancor**
- **IDEX**
- **Airswap**
- **dYdX** (on-chain portion)
- **Loopring**
- **DODO**
- **Mooniswap**
- **Matcha** (via 0x)

#### Binance Smart Chain (BSC) DEXs
- **PancakeSwap** (V1, V2, V3)
- **BakerySwap**
- **BiSwap**
- **ApeSwap**
- **MDEX**
- **SushiSwap** (BSC)
- **Ellipsis Finance**
- **Venus**

#### Polygon DEXs
- **QuickSwap**
- **SushiSwap** (Polygon)
- **Balancer** (Polygon)
- **Curve** (Polygon)
- **Uniswap V3** (Polygon)
- **DODO** (Polygon)

#### Arbitrum DEXs
- **Uniswap V3**
- **SushiSwap**
- **Camelot**
- **Balancer**
- **Curve**
- **GMX** (perpetuals - on-chain data)

#### Optimism DEXs
- **Uniswap V3**
- **Velodrome**
- **Curve**
- **Balancer**

#### Base DEXs
- **Uniswap V3**
- **Aerodrome**
- **SushiSwap**
- **BaseSwap**

#### Solana DEXs
- **Raydium**
- **Orca**
- **Serum**
- **Jupiter** (aggregator)
- **Saber**
- **Marinade**

#### Avalanche DEXs
- **Trader Joe**
- **Pangolin**
- **SushiSwap** (Avalanche)
- **Curve** (Avalanche)

#### Other Chain DEXs
- **Fantom**: SpookySwap, SpiritSwap, Curve
- **Cronos**: VVS Finance, MM Finance
- **Celo**: Ubeswap, Mobius
- **Moonbeam**: Stellaswap, Beamswap

**Total**: 40+ DEX protocols tracked across all supported chains

---

### NFT Marketplaces Covered

#### Ethereum NFT Marketplaces
- **OpenSea** (Seaport protocol)
- **Blur**
- **LooksRare**
- **Rarible**
- **X2Y2**
- **Foundation**
- **SuperRare**
- **Zora**
- **Gem** (aggregator)
- **Genie** (aggregator)

#### Solana NFT Marketplaces
- **Magic Eden** (Metaplex protocol)
- **Solanart**
- **OpenSea** (Solana)

#### Polygon NFT Marketplaces
- **OpenSea** (Polygon)

#### Multi-chain
- Any marketplace using standard protocols (Seaport, Metaplex, etc.)

---

### Centralized Exchanges (CEXs) - **NOT COVERED**

**Bitquery does NOT provide CEX data**:
- No Binance orderbook/trades
- No Coinbase trades
- No Kraken data
- No CEX futures/perpetuals
- No CEX liquidations

**Exception**: On-chain deposits/withdrawals to CEX addresses can be tracked via wallet monitoring.

---

### Traditional Stock Markets - **NOT COVERED**

- NYSE: No
- NASDAQ: No
- LSE (London): No
- TSE (Tokyo): No
- SSE/SZSE (China): No
- NSE/BSE (India): No

**Bitquery is blockchain-only.**

---

### Forex Brokers - **NOT COVERED**

No traditional forex data. Only crypto DEX pairs involving stablecoins (e.g., USDT/USDC).

---

### Futures/Options Exchanges - **NOT COVERED**

- CME: No
- CBOE: No
- ICE: No

**Exception**: On-chain derivatives protocols (dYdX, GMX) are partially covered via smart contract events.

---

## Instrument Coverage

### Cryptocurrencies (Tokens)

#### Total Tokens
- **Estimated**: 1,000,000+ tokens across all chains
- **EVM tokens**: 500,000+ (ERC-20, ERC-721, ERC-1155)
- **Solana tokens**: 50,000+ (SPL tokens)
- **Other chains**: Varies by chain

**Coverage**: All tokens with on-chain activity (any transfer, trade, or contract interaction)

#### Major Tokens Covered
- **Bitcoin**: BTC (native)
- **Ethereum**: ETH (native), all ERC-20 tokens
- **Stablecoins**: USDT, USDC, DAI, BUSD, FRAX, TUSD, USDP, etc.
- **DeFi tokens**: UNI, AAVE, COMP, MKR, SNX, CRV, BAL, SUSHI, etc.
- **Meme coins**: DOGE, SHIB, PEPE, FLOKI, etc.
- **Layer 2 tokens**: ARB, OP, MATIC, etc.
- **Wrapped tokens**: WETH, WBTC, renBTC, etc.

#### Token Standards Supported
- **ERC-20** (fungible tokens)
- **ERC-721** (NFTs)
- **ERC-1155** (multi-token standard)
- **BEP-20** (BSC tokens)
- **SPL** (Solana tokens)
- **TRC-20** (Tron tokens)

---

### Crypto Trading Pairs

#### Spot Pairs (DEX)
- **Total pairs**: 100,000+ DEX pairs across all chains
- **Example**: ETH/USDT, WBTC/ETH, UNI/USDC, etc.
- **Coverage**: All pairs with liquidity on tracked DEXs

#### Perpetuals/Futures (On-chain)
- **Limited coverage**: Only on-chain perpetual protocols
- **Examples**: GMX, dYdX (on-chain portion)
- **Not covered**: CEX perpetuals (Binance, Bybit, etc.)

---

### NFTs

#### Total NFT Collections
- **Ethereum**: 100,000+ collections
- **Solana**: 10,000+ collections
- **Polygon**: 5,000+ collections
- **Other chains**: Varies

#### Major NFT Collections Covered
- **Ethereum**: BAYC, CryptoPunks, Azuki, Doodles, Clone X, Moonbirds, etc.
- **Solana**: DeGods, y00ts, Okay Bears, etc.
- **All collections with on-chain transfers/trades tracked**

---

### Forex - **NOT COVERED**

Traditional forex pairs (EUR/USD, GBP/USD, etc.) are NOT available.

**Exception**: Crypto stablecoin pairs (USDT/USDC, DAI/USDC) are available via DEX trades.

---

### Stocks - **NOT COVERED**

- US stocks: No
- International stocks: No
- OTC: No
- Penny stocks: No

**Exception**: Tokenized stocks (e.g., Mirror Protocol synthetic stocks on Terra Classic) may be tracked if on supported chains.

---

### Commodities - **NOT COVERED**

Traditional commodities (gold, oil, wheat) are NOT available.

**Exception**: Tokenized commodities (e.g., PAXG - tokenized gold, petroleum tokens) are covered if on-chain.

---

### Indices - **NOT COVERED**

- S&P 500: No
- NASDAQ: No
- FTSE: No
- Nikkei: No

**Exception**: Crypto indices (e.g., DeFi Pulse Index - DPI) are covered via DEX trades.

---

## Data History

### Historical Depth

| Blockchain | Genesis Date | Bitquery Coverage Start | Depth |
|------------|--------------|-------------------------|-------|
| **Bitcoin** | Jan 2009 | Jan 2009 | 15+ years |
| **Ethereum** | Jul 2015 | Jul 2015 | 9+ years |
| **Binance Smart Chain** | Sep 2020 | Sep 2020 | 4+ years |
| **Polygon** | May 2020 | May 2020 | 4+ years |
| **Solana** | Mar 2020 | Mar 2020 | 4+ years |
| **Arbitrum** | May 2021 | May 2021 | 3+ years |
| **Optimism** | Nov 2021 | Nov 2021 | 3+ years |
| **Base** | Jul 2023 | Jul 2023 | 1+ year |
| **Other chains** | Various | From genesis | Full history |

**General Rule**: Bitquery provides full blockchain history from genesis block for all supported chains.

---

### Granularity Available

| Granularity | Available | Since When | Notes |
|-------------|-----------|------------|-------|
| **Tick data** | No | N/A | Not applicable (blockchain uses blocks) |
| **Block-level** | Yes | From genesis | Native granularity (~12s for Ethereum) |
| **Transaction-level** | Yes | From genesis | Individual transactions |
| **1-second** | Yes (aggregated) | Any period | Via timestamp grouping |
| **1-minute bars** | Yes (aggregated) | Any period | Via GraphQL grouping |
| **5-minute bars** | Yes | Any period | Via grouping |
| **Hourly** | Yes | Any period | Via grouping |
| **Daily** | Yes | Any period | Via grouping |
| **Weekly/Monthly** | Yes | Any period | Via grouping |

**Note**: Bitquery doesn't pre-compute OHLCV bars. You construct them via GraphQL queries with time-based grouping.

---

### Real-time vs Delayed

| Data Type | Delay | Notes |
|-----------|-------|-------|
| **Real-time (dataset: realtime)** | <1 second | Sub-second latency after block inclusion |
| **Archive (dataset: archive)** | 1-5 seconds | Near real-time (minimal indexing delay) |
| **Mempool** | Real-time | Pending transactions (pre-block) |
| **No artificial delays** | - | All data is as fresh as possible |

**Free tier**: Same latency as paid (no artificial delays on free plan)

**Commercial tier**: Same latency (no priority for real-time data)

---

## Update Frequency

### Real-time Streams (WebSocket Subscriptions)

| Data Type | Update Frequency | Notes |
|-----------|------------------|-------|
| **Blocks** | Every block | ~12s (Ethereum), ~3s (BSC), ~0.4s (Solana) |
| **Transactions** | Real-time | As transactions are confirmed |
| **DEX Trades** | Real-time | Every trade on DEXs |
| **Transfers** | Real-time | Every token transfer |
| **Mempool** | Real-time | As transactions enter mempool |
| **Events** | Real-time | As smart contract events are emitted |

---

### Scheduled Updates

**Not applicable** - Bitquery provides real-time blockchain data, not scheduled batch updates.

**Exception**: Historical data is continuously indexed (backfilling for new chains or protocols).

---

### Data Refresh Rates

| Query Type | Refresh Rate | Notes |
|------------|--------------|-------|
| **HTTP queries (archive dataset)** | On-demand | Query returns latest indexed data |
| **WebSocket subscriptions (realtime)** | Push-based | Server pushes new data as it arrives |
| **Mempool** | Real-time | Updated as mempool changes |

---

## Data Quality

### Accuracy

| Aspect | Quality | Notes |
|--------|---------|-------|
| **Source** | Direct from blockchain nodes | First-party data (not aggregated) |
| **Validation** | Yes | Data validated against blockchain consensus |
| **Corrections** | Automatic | Reorgs handled automatically |
| **Decoding** | Automatic | Events/calls decoded via ABI |

**Data Sources**:
- Direct blockchain node connections
- Full archive nodes for historical data
- No third-party aggregation (except for multi-chain queries)

---

### Completeness

| Aspect | Status | Notes |
|--------|--------|-------|
| **Missing data** | Rare | Possible during reorgs or node issues |
| **Gaps** | Handled automatically | Reindexing if gaps detected |
| **Backfill** | Available immediately | Full history from genesis |
| **Reorgs** | Handled | Blockchain reorganizations processed |

**Known Limitations**:
- Some smart contract events require ABI for decoding (may show as raw data if ABI unknown)
- Privacy chains (Zcash shielded, Monero) have limited data (only public transactions)

---

### Timeliness

| Metric | Value | Notes |
|--------|-------|-------|
| **Latency (realtime)** | <1 second | After block inclusion |
| **Latency (archive)** | 1-5 seconds | Indexing delay |
| **Delay** | ~2 seconds typical | From block production to query availability |
| **Market hours** | 24/7/365 | Blockchains never close |

---

### Data Integrity

- **Immutability**: Blockchain data is immutable (past data doesn't change, except reorgs)
- **Reorgs**: Handled automatically (data updated if chain reorgs)
- **Verification**: All data verifiable against blockchain

---

## Coverage Limitations

### What Bitquery Does NOT Cover

1. **Centralized Exchange Data**
   - No CEX trades, orderbooks, liquidations
   - No CEX futures/perpetuals
   - No CEX deposit/withdrawal data (except on-chain tracking)

2. **Off-chain Data**
   - No Layer 2 off-chain transactions (unless settled on-chain)
   - No Lightning Network transactions
   - No state channels

3. **Private/Shielded Transactions**
   - Zcash shielded transactions (only transparent txs)
   - Monero transactions (privacy by design)
   - Other privacy chains with hidden transaction data

4. **Traditional Finance**
   - No stocks, bonds, forex, commodities
   - No economic indicators
   - No traditional derivatives

5. **Social Data**
   - No Twitter sentiment
   - No news articles
   - No social media analytics (unless on-chain, e.g., Lens Protocol)

---

## Multi-chain Coverage Summary

| Chain Type | Chains Covered | Data Depth | Real-time |
|------------|----------------|------------|-----------|
| **EVM Chains** | 20+ chains | Full history | Yes |
| **Bitcoin-like (UTXO)** | 7 chains | Full history | Yes |
| **Solana** | 1 chain | From Mar 2020 | Yes |
| **Cosmos** | 1 chain | From launch | Yes |
| **Cardano** | 1 chain | From launch | Yes |
| **Tron** | 1 chain | From launch | Yes |
| **Other** | 10+ chains | From launch | Yes |

**Total**: 40+ blockchains

---

## Geographic Data Availability

**Blockchain data has no geographic boundaries**:
- All chains accessible globally
- Data reflects global blockchain state
- No regional price differences (same blockchain worldwide)

**Example**: Ethereum DEX trade in Tokyo is same data as in New York.

---

## Coverage Expansion

### Newly Added Chains (2024-2025)
- Base (Coinbase L2) - Added 2023
- zkSync Era - Added 2023
- Polygon zkEVM - Added 2023
- Linea - Added 2023
- Scroll - Added 2024
- Mantle - Added 2024

**Bitquery actively adds new chains**. Check docs for latest additions.

---

## Use Case Coverage

| Use Case | Coverage | Notes |
|----------|----------|-------|
| **DEX Trading Analytics** | Excellent | All major DEXs covered |
| **NFT Marketplace Tracking** | Excellent | OpenSea, Blur, Magic Eden, etc. |
| **Wallet Tracking** | Excellent | All on-chain wallets |
| **Token Analytics** | Excellent | All tokens with on-chain activity |
| **Smart Contract Monitoring** | Excellent | All contracts with events/calls |
| **MEV Analysis** | Good | Mempool + transaction ordering data |
| **Portfolio Tracking** | Excellent | All on-chain holdings |
| **Tax/Accounting** | Good | Transaction history available |
| **CEX Trading** | None | Not supported |
| **Traditional Finance** | None | Not supported |

---

## Summary

**Bitquery excels at**:
- Multi-chain on-chain data (40+ blockchains)
- DEX trading analytics
- NFT marketplace tracking
- Token holder analysis
- Smart contract monitoring
- Real-time blockchain streaming

**Bitquery does NOT provide**:
- Centralized exchange data
- Traditional finance data (stocks, forex, commodities)
- Off-chain transaction data
- Social sentiment or news
