# Whale Alert - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: Yes (global service, no restrictions)
- Europe: Yes
- Asia: Yes
- South America: Yes
- Africa: Yes
- Oceania: Yes
- **Global:** Yes - blockchain data is inherently global

### Country-Specific
- **No country-specific restrictions** - Whale Alert monitors blockchains globally
- Data coverage is blockchain-based, not country-based
- All regions have equal access to data

### Restricted Regions
- Blocked countries: Not publicly documented (standard compliance likely applies)
- VPN detection: Not applicable (API key-based, not IP-based)
- Geo-fencing: Not documented
- **Note:** API access may be subject to standard US export restrictions and sanctions

**Blockchain data is borderless** - Whale Alert monitors global blockchain networks.

---

## Markets/Exchanges Covered

### Stock Markets
- US: Not applicable
- UK: Not applicable
- Other: Not applicable

**Whale Alert does NOT cover stock markets.**

### Crypto Exchanges (Attribution Data)

Whale Alert monitors addresses belonging to **400+ entities**, including major exchanges:

**Confirmed Exchanges in Attribution Database:**
- Binance (global leader)
- Coinbase (US-based)
- Kraken
- Huobi
- Bitfinex
- Gemini
- Bitstamp
- OKX
- KuCoin
- Gate.io
- Bybit
- ...and 390+ more entities

**Exchange Coverage:**
- Hot wallets
- Cold wallets
- Deposit wallets
- Withdrawal processing wallets
- Institutional custody wallets

**Note:** Whale Alert doesn't aggregate exchange data. It monitors blockchain transactions and attributes addresses to known exchanges.

### Forex Brokers
Not applicable - Whale Alert focuses on cryptocurrency blockchains only.

### Futures/Options Exchanges
Not directly tracked - Whale Alert monitors on-chain data, not exchange-level derivatives.

---

## Blockchain Coverage

### Supported Blockchains (11+ as of 2026)

| Blockchain | Symbol | Status | Notes |
|------------|--------|--------|-------|
| Bitcoin | BTC | ✅ Connected | ~10 min blocks |
| Ethereum | ETH | ✅ Connected | ~12 sec blocks, ERC-20 tokens |
| Algorand | ALGO | ✅ Connected | Fast finality |
| Bitcoin Cash | BCH | ✅ Connected | Bitcoin fork |
| Dogecoin | DOGE | ✅ Connected | Meme coin, ~1 min blocks |
| Litecoin | LTC | ✅ Connected | ~2.5 min blocks |
| Polygon | MATIC | ✅ Connected | Ethereum sidechain, low fees |
| Solana | SOL | ✅ Connected | High throughput, <1 sec blocks |
| Ripple | XRP | ✅ Connected | Payment network |
| Cardano | ADA | ✅ Connected | ~20 sec blocks |
| Tron | TRX | ✅ Connected | ~3 sec blocks, TRC-20 tokens |

**Additional blockchains can be requested** - contact Whale Alert for custom blockchain support.

### Blockchain Data Depth

**Historical Depth (Enterprise API):**
- 30 days of transaction history
- Query by block height
- Oldest and newest block heights available via `/status` endpoint

**Real-time Coverage:**
- All supported blockchains monitored 24/7
- Immediate detection of new blocks and transactions

---

## Instrument Coverage

### Stocks
- Total symbols: 0 (not covered)

**Not applicable** - Whale Alert is crypto-only.

### Crypto

**Total Coverage:**
- **Coins:** 11+ native blockchain currencies (BTC, ETH, SOL, TRX, etc.)
- **Tokens:** Hundreds to thousands (ERC-20, TRC-20, etc.)
- **Spot pairs:** Not applicable (Whale Alert doesn't track trading pairs)
- **Futures:** Not applicable (on-chain data only)
- **Perpetuals:** Not applicable

**Confirmed Cryptocurrencies:**

**Bitcoin blockchain:**
- BTC (native)
- USDT (Omni layer)
- EURT

**Ethereum blockchain (ERC-20 tokens):**
- ETH (native)
- USDT (Tether)
- USDC (Circle)
- WBTC (Wrapped Bitcoin)
- DAI (MakerDAO)
- LINK (Chainlink)
- UNI (Uniswap)
- AAVE
- SHIB (Shiba Inu)
- MATIC (Polygon native token on Ethereum)
- ...and hundreds more ERC-20 tokens

**Tron blockchain (TRC-20 tokens):**
- TRX (native)
- USDD (Tron stablecoin)
- BTT (BitTorrent)
- USDT (Tether TRC-20)
- USDC (Circle TRC-20)
- TUSD
- USDJ
- WBTC

**Other blockchains:**
- DOGE (Dogecoin native)
- ALGO (Algorand native)
- BCH (Bitcoin Cash native)
- LTC (Litecoin native)
- MATIC (Polygon native)
- SOL (Solana native)
- XRP (Ripple native)
- ADA (Cardano native)

**Total estimated coverage:** 500+ cryptocurrencies across 11+ blockchains

### Forex
Not applicable - crypto only.

### Commodities
Not applicable - crypto only.

### Indices
Not applicable - crypto only.

---

## Data History

### Historical Depth

**Enterprise API (Quantitative tier):**
- **Transaction history:** 30 days
- **Query method:** By block height, address, transaction hash
- **Access:** REST API

**Developer API v1 (deprecated):**
- **Transaction history:** Limited (not clearly specified)
- **Query method:** By timestamp, transaction hash

**Historical Data Product:**
- **Custom depth:** Purchase by year ($1,990/year of data)
- **Format:** Bulk datasets (not API)
- **Use case:** Model training, backtesting

**Blockchain Coverage Start Dates (estimated):**
- Bitcoin: Data likely from 2017-2018+ (when Whale Alert launched)
- Ethereum: Data likely from 2017-2018+
- Newer blockchains (Solana, Cardano): From time they were added (varies)

**Note:** Exact historical start dates not publicly documented. Contact Whale Alert for specific blockchain history.

### Granularity Available

**Transaction-level granularity:**
- ✅ Individual transactions (finest granularity)
- ✅ Block-level queries (all transactions in a block)
- ✅ Address-level queries (all transactions for an address)
- ❌ Time-based candles (not applicable - this is transaction data, not price data)

**Time Resolution:**
- Real-time (as blocks are mined)
- Block-by-block (depends on blockchain: Bitcoin ~10min, Ethereum ~12sec, Solana <1sec)

### Real-time vs Delayed

**Real-time:**
- ✅ WebSocket alerts (Custom Alerts, Priority Alerts)
- ✅ Enterprise REST API (query latest blocks)
- **Delay:** Seconds from blockchain confirmation
- **Priority Alerts:** Up to 1 minute faster than Custom Alerts

**Delayed:**
- ❌ No delayed tier - all data is real-time or historical

**Snapshot:**
- Block snapshots (query specific block)
- Address snapshots (current balance via attribution endpoint)

---

## Update Frequency

### Real-time Streams (WebSocket)

**Transaction Alerts:**
- **Frequency:** As transactions occur
- **Delivery:** Real-time (within seconds of blockchain confirmation)
- **Priority Alerts:** 1 minute faster than Custom Alerts (typically <10 seconds from confirmation)
- **Filtering:** Only transactions meeting criteria (min $100k USD)

**Social Media Alerts:**
- **Frequency:** As Whale Alert posts to Twitter/Telegram
- **Delivery:** Real-time

### Blockchain-Specific Block Times

| Blockchain | Block Time | Real-time Latency |
|------------|------------|-------------------|
| Bitcoin | ~10 minutes | 10-15 minutes from broadcast |
| Ethereum | ~12 seconds | 15-30 seconds from broadcast |
| Solana | <1 second | 1-5 seconds from broadcast |
| Tron | ~3 seconds | 5-10 seconds from broadcast |
| Polygon | ~2 seconds | 3-10 seconds from broadcast |
| Dogecoin | ~1 minute | 1-2 minutes from broadcast |
| Litecoin | ~2.5 minutes | 3-5 minutes from broadcast |
| Cardano | ~20 seconds | 20-40 seconds from broadcast |
| Ripple | ~3-5 seconds | 5-15 seconds from broadcast |
| Algorand | ~4.5 seconds | 5-15 seconds from broadcast |
| Bitcoin Cash | ~10 minutes | 10-15 minutes from broadcast |

**Note:** Latency includes blockchain confirmation time + Whale Alert processing + network delivery.

### Scheduled Updates

**Not applicable** - blockchain data is event-driven, not scheduled.

- Fundamentals: N/A
- Economic data: N/A
- News: Real-time (social alerts)

---

## Data Quality

### Accuracy

**Source:**
- ✅ Direct from blockchain (full node monitoring)
- ❌ Not aggregated from third parties
- ✅ Multi-source verification

**Validation:**
- Scientific validation (peer-reviewed research)
- Multiple blockchain nodes
- Proven track record (early warnings for Bybit hack, FTX collapse)

**Price Data:**
- USD valuation captured at transaction time
- Source: Not documented (likely major exchange average or oracle)
- Purpose: Historical reference, not trading prices

**Attribution:**
- Proprietary database of 400+ entities
- Confidence scores (0-1 scale) for uncertain attributions
- Continuous updates and verification

### Completeness

**Transaction Coverage:**
- All transactions on supported blockchains
- No sampling - full blockchain monitoring
- Small transactions (<$10 USD) may be grouped in v1 API

**Missing Data:**
- Rare: Blockchain data is complete by nature
- Gaps: Only during blockchain reorganizations (reorgs) or node issues
- Backfill: 30-day historical available (Enterprise API)

**Address Attribution:**
- 400+ entities covered
- Unknown addresses marked as "unknown" (not missing)
- Confidence scores indicate attribution uncertainty

### Timeliness

**Latency:**
- **Priority Alerts:** <10 seconds typical (up to 1 minute faster than standard)
- **Custom Alerts:** 10-60 seconds typical
- **REST API:** Real-time queries (seconds from latest block)

**Delay:**
- Blockchain confirmation time (varies by chain)
- Network propagation (~1-5 seconds)
- Processing and enrichment (~1-10 seconds)

**Market Hours:**
- ✅ 24/7/365 - blockchains never close
- No downtime for weekends or holidays
- Continuous monitoring

---

## Attribution Coverage

### Entity Types Covered

**Exchanges (majority of 400+ entities):**
- Centralized exchanges (CEX)
- Decentralized exchange wallets (DEX smart contracts)
- P2P platforms
- Derivatives platforms

**Other Entity Types:**
- Mining pools
- DeFi protocols (lending, staking, AMM)
- Stablecoin treasuries (Tether, Circle, etc.)
- Custodians and institutional wallets
- Payment processors
- Mixer/tumbler services
- Known scam/fraud addresses
- ICO/token sale addresses
- Foundation wallets
- Developer wallets

### Attribution Accuracy

**Confidence Scoring:**
- 0.0 to 1.0 scale
- Higher score = higher confidence
- Multiple attributions possible (e.g., shared wallets)

**Typical Confidence Levels:**
- 0.95-1.0: High confidence (official exchange hot wallet)
- 0.80-0.95: Good confidence (likely owned by entity)
- 0.50-0.80: Moderate confidence (possible connection)
- <0.50: Low confidence (speculative)

### Unknown Addresses

**Percentage of unknown addresses:**
- Not publicly disclosed
- Likely majority (millions of retail/personal wallets)
- Focus on high-value and exchange addresses

**Unknown != Missing:**
- Unknown addresses still tracked
- Full transaction data available
- Just no owner attribution

---

## Regional Coverage Notes

### Global Blockchain Coverage

**Whale Alert monitors global blockchains:**
- Bitcoin (global, no borders)
- Ethereum (global)
- All supported chains are borderless

**Exchange Attribution:**
- Global exchanges (Binance, OKX, etc.)
- US exchanges (Coinbase, Gemini, Kraken)
- Asian exchanges (Huobi, Bybit)
- European exchanges (Bitstamp, Bitfinex)

### API Access Restrictions

**Not documented explicitly, but likely:**
- Standard US export controls apply
- Sanctioned countries may be blocked
- OFAC compliance for US-based service

**Recommendation:** Check terms of service for specific restrictions.

---

## Comparison to Other Data Providers

### vs Exchange APIs (Binance, Coinbase)
- **Exchange APIs:** Trading data, orderbooks, prices
- **Whale Alert:** On-chain transactions, address attribution
- **Overlap:** None - complementary data sources

### vs Block Explorers (Etherscan, Blockchain.com)
- **Block Explorers:** Raw blockchain data, single chain
- **Whale Alert:** Multi-chain, enriched with attribution, filtered for large transactions
- **Advantage:** Address attribution, real-time alerts, standardized format

### vs On-chain Analytics (Glassnode, Nansen)
- **Analytics Platforms:** Metrics, charts, on-chain indicators
- **Whale Alert:** Raw transaction feed, real-time alerts
- **Advantage:** Real-time speed (Priority Alerts), simple API, transaction-level detail

### vs Market Data (CoinGecko, CoinMarketCap)
- **Market Data:** Prices, volumes, market cap, exchanges
- **Whale Alert:** On-chain transactions
- **Overlap:** None - completely different data types

---

## Use Case Coverage

### Supported Use Cases

✅ **Whale Watching:**
- Large transaction monitoring
- Exchange inflow/outflow tracking
- Institutional movement detection

✅ **Trading Signals:**
- Exchange deposit alerts (potential sell pressure)
- Exchange withdrawal alerts (accumulation signals)
- Treasury movements (stablecoin mints/burns)

✅ **Risk Management:**
- Exchange hack detection (unexpected large outflows)
- Fraud wallet monitoring
- Address screening (AML/compliance)

✅ **Market Research:**
- Wallet behavior analysis
- Exchange flow patterns
- Blockchain adoption metrics

✅ **Algorithmic Trading:**
- ML model input (transaction patterns)
- Predictive models (transaction → price impact)
- Anomaly detection

### Not Supported Use Cases

❌ **Price Trading:**
- Use exchange APIs for OHLC, orderbook, trades

❌ **Technical Analysis:**
- Use market data providers for indicators

❌ **Fundamental Analysis:**
- Use CoinGecko, Messari for tokenomics

❌ **Derivatives Trading:**
- Use Coinglass, exchange APIs for funding, OI, liquidations

---

## Summary Table

| Category | Coverage | Notes |
|----------|----------|-------|
| **Geographic** | Global | Blockchain data is borderless |
| **Blockchains** | 11+ | Bitcoin, Ethereum, Solana, Tron, etc. |
| **Cryptocurrencies** | 500+ | Native coins + tokens (ERC-20, TRC-20, etc.) |
| **Exchanges (Attribution)** | 400+ | All major global exchanges |
| **Historical Depth** | 30 days | Enterprise API (REST) |
| **Historical Depth (bulk)** | Full history | Historical Data product ($1,990/year) |
| **Real-time Latency** | <10 sec | Priority Alerts |
| **Update Frequency** | Per block | Varies by blockchain (12 sec to 10 min) |
| **Address Attribution** | 400+ entities | Confidence scoring |
| **Market Hours** | 24/7/365 | Blockchains never close |
| **Data Quality** | High | Direct blockchain monitoring, validated |

---

## Recommendations

**For Global Coverage:**
- Whale Alert covers all major blockchains globally - no regional gaps

**For Comprehensive Analysis:**
- Combine Whale Alert (on-chain) + Exchange APIs (prices) + Derivatives APIs (funding, OI)

**For Historical Analysis:**
- Enterprise API: 30-day rolling window (sufficient for recent patterns)
- Historical Data: Purchase full archives for deep backtesting

**For Real-time Trading:**
- Priority Alerts: Fastest delivery (institutional grade)
- Custom Alerts: Sufficient for most traders

**For Attribution Needs:**
- Focus on 400+ known entities
- Unknown addresses still have full transaction data
- Use confidence scores to filter low-confidence attributions
