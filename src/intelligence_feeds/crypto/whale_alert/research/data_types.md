# Whale Alert - Data Types Catalog

**NOTE:** Whale Alert is NOT a market data provider. It is a blockchain transaction tracking and alerting service.

This service focuses exclusively on **on-chain transaction data** with rich attribution and metadata.

---

## Standard Market Data

- [ ] Current Price
- [ ] Bid/Ask Spread
- [ ] 24h Ticker Stats (high, low, volume, change%)
- [ ] OHLC/Candlesticks
- [ ] Level 2 Orderbook
- [ ] Recent Trades
- [ ] Volume (24h, intraday)

**NOT AVAILABLE** - Whale Alert does not provide market/price data.

Note: Transaction data includes USD valuation (`unit_price_usd`) at the time of transaction, but this is NOT a price feed.

---

## Historical Data

- [ ] Historical prices
- [ ] Minute bars
- [ ] Daily bars
- [ ] Tick data
- [ ] Adjusted prices

**NOT AVAILABLE** - Whale Alert does not provide historical price data.

---

## Derivatives Data (Crypto/Futures)

- [ ] Open Interest
- [ ] Funding Rates
- [ ] Liquidations
- [ ] Long/Short Ratios
- [ ] Mark Price
- [ ] Index Price
- [ ] Basis

**NOT AVAILABLE** - Whale Alert does not provide derivatives data.

Note: While liquidations are a concept in derivatives, Whale Alert doesn't specifically track exchange liquidations. It tracks on-chain transactions which may include liquidation-related transfers.

---

## Options Data

- [ ] Options Chains
- [ ] Implied Volatility
- [ ] Greeks
- [ ] Open Interest
- [ ] Historical option prices

**NOT AVAILABLE** - Whale Alert does not provide options data.

---

## Fundamental Data (Stocks)

- [ ] Company Profile
- [ ] Financial Statements
- [ ] Earnings
- [ ] Dividends
- [ ] Stock Splits
- [ ] Analyst Ratings
- [ ] Insider Trading
- [ ] Institutional Holdings
- [ ] Financial Ratios
- [ ] Valuation Metrics

**NOT AVAILABLE** - Whale Alert focuses on crypto blockchain data only.

---

## On-chain Data (Crypto) - **PRIMARY OFFERING**

This is Whale Alert's core specialty:

### Transaction Data
- [x] **Blockchain Transactions** (all types)
- [x] **Large Value Transfers** ("whale" movements)
- [x] **Transaction Hash/ID**
- [x] **Block Height and Index**
- [x] **Timestamp** (Unix timestamp)
- [x] **Transaction Fees** (amount, symbol, USD value)

### Transaction Types
- [x] **Transfer** - Standard value transfers between addresses
- [x] **Mint** - Token/coin creation events
- [x] **Burn** - Token/coin destruction (sent to unrecoverable address)
- [x] **Freeze** - Assets frozen (cannot be transferred)
- [x] **Unfreeze** - Assets unfrozen (can be transferred again)
- [x] **Lock** - Assets locked (time-locked or contract-locked)
- [x] **Unlock** - Assets unlocked

### Address Data
- [x] **Address Hash** - Wallet identifier
- [x] **Address Balance** (post-transaction)
- [x] **Address Locked Balance** (non-transferable amount)
- [x] **Address Frozen Status** (boolean)
- [x] **Address Transaction History** (30-day depth)

### Address Attribution (**UNIQUE FEATURE**)
- [x] **Owner Attribution** - Entity identification (400+ entities)
- [x] **Owner Type** - Entity classification
- [x] **Address Type** - Wallet category
- [x] **Confidence Scores** (0-1 scale for attribution accuracy)

**Known Owner Types:**
- `exchange` - Cryptocurrency exchange
- `unknown` - Unidentified owner
- (Additional types exist but not fully documented - likely include: institution, miner, defi_protocol, etc.)

**Address Types:**
- `hot_wallet` - Exchange hot wallet (higher sell risk)
- `cold_wallet` - Cold storage wallet
- `deposit_wallet` - Deposit receiving wallet
- `exchange_wallet` - General exchange wallet
- `burn_address` - Burn address (tokens unrecoverable)
- `mixer_wallet` - Mixer/tumbler wallet
- `coinjoin` - CoinJoin address (privacy-focused)
- `fraud_wallet` - Known fraud/scam wallet
- `unknown` - Unclassified address

### Multi-Currency Transactions
- [x] **Sub-transactions** - Transactions can contain multiple currencies
- [x] **Per-Currency Breakdown** - Each currency tracked separately
- [x] **Symbol and Amount** - Individual currency amounts
- [x] **USD Valuation** - USD value at time of transaction

### Value Metrics
- [x] **Transaction Amount** (in native cryptocurrency)
- [x] **USD Value** (at time of transaction)
- [x] **Unit Price USD** (price per token/coin at transaction time)
- [x] **Minimum Value Filtering** (WebSocket: min $100k USD)

### Block Data
- [x] **Block Height/Number**
- [x] **Block Timestamp**
- [x] **Complete Block Data** (all transactions in a block)
- [x] **Block Height Range** (oldest and newest available blocks)

### Supported Blockchains (11+)
- [x] Bitcoin (bitcoin)
- [x] Ethereum (ethereum)
- [x] Algorand (algorand)
- [x] Bitcoin Cash (bitcoincash)
- [x] Dogecoin (dogecoin)
- [x] Litecoin (litecoin)
- [x] Polygon (polygon)
- [x] Solana (solana)
- [x] Ripple (ripple)
- [x] Cardano (cardano)
- [x] Tron (tron)

**Additional blockchains can be requested.**

### Supported Cryptocurrencies

**Bitcoin blockchain:**
- BTC, USDT, EURT

**Ethereum blockchain:**
- ETH, USDT, USDC, WBTC, DAI, LINK, UNI, AAVE, SHIB, MATIC, and hundreds more ERC-20 tokens

**Tron blockchain:**
- USDD, TRX, BTT, USDT, USDC, TUSD, USDJ, WBTC

**Dogecoin blockchain:**
- DOGE

**...and more across all supported blockchains**

### NOT Included (On-chain)
- [ ] Gas Prices - Not tracked
- [ ] NFT Data - Not a focus (may capture NFT transfers but not metadata)
- [ ] Smart Contract Code - Not provided
- [ ] Contract Events (general) - Only value-relevant events

---

## Macro/Economic Data

- [ ] Interest Rates
- [ ] GDP
- [ ] Inflation (CPI, PPI, PCE)
- [ ] Employment
- [ ] Economic Calendar

**NOT AVAILABLE** - Whale Alert focuses on blockchain data only.

---

## Forex Specific

- [ ] Currency Pairs
- [ ] Bid/Ask Spreads
- [ ] Pip precision
- [ ] Cross rates
- [ ] Historical FX rates

**NOT AVAILABLE** - Whale Alert focuses on crypto blockchain data only.

---

## Metadata & Reference

### Blockchain Metadata
- [x] **Supported Blockchains List** - Via `/status` endpoint
- [x] **Supported Currencies per Blockchain** - Via `/status` endpoint
- [x] **Blockchain Connection Status** - "connected" status indicator
- [x] **Block Height Range** - Oldest and newest available blocks

### Address Attribution Database
- [x] **400+ Entities** - Exchanges, institutions, protocols
- [x] **Entity Names** - Human-readable names (e.g., "Binance", "Coinbase")
- [x] **Entity Types** - Classification system
- [x] **Confidence Scores** - Attribution accuracy (0-1 scale)

### NOT Included
- [ ] Market Hours - Not applicable (blockchain 24/7)
- [ ] Trading Calendars - Not applicable
- [ ] Timezone Info - All timestamps are Unix (UTC)
- [ ] Sector/Industry Classifications - Not applicable

---

## News & Sentiment

### Social Media Alerts (WebSocket only)
- [x] **Whale Alert Social Posts** - Real-time Twitter/Telegram posts
- [x] **Post Text** - Full post content
- [x] **Post URLs** - Links to original posts (Twitter, Telegram)
- [x] **Related Blockchain** - Which blockchain the post refers to
- [x] **Timestamp** - When posted

### NOT Included
- [ ] General Crypto News - Only Whale Alert's own posts
- [ ] Press Releases - Not provided
- [ ] Broad Social Sentiment - Only Whale Alert content
- [ ] Analyst Reports - Not provided

---

## Unique/Custom Data - **WHAT MAKES WHALE ALERT SPECIAL**

### 1. Address Attribution System
**Whale Alert's proprietary database of 400+ entities:**
- Exchanges (Binance, Coinbase, Kraken, etc.)
- Mining pools
- DeFi protocols
- Institutional wallets
- Known scam/fraud addresses
- Mixer services
- Treasury wallets (Tether, Circle, etc.)

**Confidence scoring:** Each attribution has a confidence score (0-1 scale)

### 2. Real-time "Whale" Detection
**Automatically identifies and alerts on large transactions:**
- Minimum $100,000 USD threshold (WebSocket)
- Captures market-moving transactions
- Filters noise (sub-$10 transactions grouped)
- Real-time alerting (Priority tier: fastest in market)

### 3. Multi-blockchain Coverage
**Standardized format across 11+ blockchains:**
- Same JSON schema regardless of blockchain protocol
- Consistent field names and types
- Unified API for all chains

### 4. Transaction Classification
**7 transaction types** (not just "transfer"):
- Transfer
- Mint (token creation)
- Burn (token destruction)
- Freeze/Unfreeze (Tether, USDC, etc.)
- Lock/Unlock (time-locks, vesting)

### 5. Historical Depth (Quantitative Tier)
**30 days of transaction history:**
- Query by block height
- Stream transactions from any starting point
- Filter by symbol, type, value
- Address transaction history

### 6. Social Media Integration
**Real-time Whale Alert posts:**
- Same content as Twitter @whale_alert
- Same content as Telegram @whale_alert_io
- Delivered via WebSocket
- Includes post URLs for verification

### 7. Scientifically Validated
**Academic rigor:**
- Collaboration with universities on blockchain analysis
- Research papers published
- Validated predictions (Bybit hack, FTX collapse)
- Trusted by institutional traders

---

## Data Quality & Coverage

### Accuracy
- **Source:** Direct blockchain monitoring (full nodes)
- **Validation:** Multi-source verification
- **Attribution:** Proprietary database with confidence scoring
- **Price Data:** Captured at transaction time (historical reference)

### Completeness
- **All major blockchains:** 11+ covered
- **All major tokens:** Hundreds of ERC-20, TRC-20, etc.
- **All transaction types:** Transfer, mint, burn, freeze, lock, unlock
- **24/7 Coverage:** Blockchain never sleeps

### Timeliness
- **Real-time:** Immediate detection of new transactions
- **Priority Alerts:** Up to 1 minute faster than standard
- **Latency:** Milliseconds to seconds from blockchain confirmation
- **Historical:** 30-day depth (Quantitative tier)

### Update Frequency
- **Real-time:** As blocks are mined/confirmed
- **Bitcoin:** ~10 minutes per block
- **Ethereum:** ~12 seconds per block
- **Solana:** <1 second per block
- **Varies by blockchain**

---

## Use Cases

### Trading Signals
- Large exchange outflows (bullish signal?)
- Large exchange inflows (bearish signal?)
- Whale accumulation patterns
- Exchange wallet depletion
- Treasury movements (Tether mints/burns)

### Risk Management
- Early detection of exchange hacks (large unexpected transfers)
- Fraud wallet activity monitoring
- Mixer usage detection (potential illicit activity)
- Exchange solvency monitoring (wallet balances)

### Market Research
- Institutional flow analysis
- Exchange dominance tracking
- DeFi protocol flows
- Stablecoin supply changes (mints/burns)

### Compliance & Forensics
- Transaction tracing
- Address ownership investigation
- Fraud detection
- AML (Anti-Money Laundering) monitoring

### Algorithm Development
- ML models for transaction pattern recognition
- Predictive models (transaction → price impact)
- Anomaly detection algorithms
- Sentiment indicators based on whale behavior

---

## What Whale Alert Does NOT Provide

**Market Data:**
- No price feeds (use exchange APIs)
- No orderbook data
- No trading volumes
- No candlestick/OHLC data

**Derivatives:**
- No futures data
- No options data
- No funding rates
- No liquidation data (exchange liquidations)

**News:**
- No general crypto news
- No press releases
- No third-party content

**Analytics (calculated metrics):**
- No moving averages
- No technical indicators
- No market cap calculations
- No volume analysis

**Whale Alert provides RAW blockchain transaction data with rich attribution metadata.**

Combine with other data sources (exchange APIs, news feeds, price data) for comprehensive analysis.

---

## Summary Table

| Category | Available | Notes |
|----------|-----------|-------|
| **Market Data** | No | Use exchange APIs instead |
| **On-chain Transactions** | ✅ YES | Core offering |
| **Address Attribution** | ✅ YES | 400+ entities, unique feature |
| **Multi-blockchain** | ✅ YES | 11+ chains, standardized format |
| **Historical Blockchain** | ✅ YES | 30 days (Quantitative tier) |
| **Real-time Alerts** | ✅ YES | WebSocket (Custom/Priority tiers) |
| **Social Media Feed** | ✅ YES | Whale Alert's own posts only |
| **Transaction Classification** | ✅ YES | 7 types: transfer, mint, burn, freeze, unfreeze, lock, unlock |
| **USD Valuation** | ✅ YES | At time of transaction (not a price feed) |
| **Derivatives Data** | No | Not applicable |
| **News Aggregation** | No | Only Whale Alert's posts |
| **Price Feeds** | No | Use market data APIs |
| **Technical Indicators** | No | Calculate from other data sources |

---

## Integration Recommendations

**For Comprehensive Trading System:**

1. **Whale Alert** - On-chain transaction monitoring and attribution
2. **Exchange API (Binance, etc.)** - Price, orderbook, trading
3. **CoinGecko/CoinMarketCap** - Market cap, volume aggregation
4. **News API (CryptoPanic, etc.)** - News and sentiment
5. **Derivatives API (Coinglass, etc.)** - Funding, OI, liquidations

**Whale Alert complements but does not replace other data sources.**
