# CryptoCompare - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: Yes
- Europe: Yes
- Asia: Yes
- South America: Yes
- Africa: Yes
- Oceania: Yes
- **Global:** Yes (worldwide coverage)

### Country-Specific
CryptoCompare aggregates data from global exchanges, so coverage is worldwide. Not country-specific.

- US: Yes (Coinbase, Kraken, Gemini, etc.)
- UK: Yes (various exchanges)
- Japan: Yes (bitFlyer, etc.)
- India: Yes (various exchanges)
- China: Limited (mainland exchanges restricted, but data available)
- South Korea: Yes (Upbit, Bithumb, etc.)
- Singapore: Yes (various exchanges)
- EU countries: Yes (Bitstamp, etc.)

### Restricted Regions
- Blocked countries: None (API accessible globally)
- VPN detection: No
- Geo-fencing: No
- **Note:** CryptoCompare is a data aggregator, not an exchange. No trading restrictions apply.

### Regional Notes
- API accessible from anywhere
- No regional endpoints (single global API)
- Data includes exchanges from all regions
- Some exchanges in aggregation may have regional restrictions (but data still available)

## Markets/Exchanges Covered

CryptoCompare aggregates data from **170-316 exchanges** (sources vary).

### Major Centralized Exchanges (CEX)

**Tier 1 (Highest Volume):**
- Binance - Yes
- Coinbase - Yes
- Kraken - Yes
- Bitfinex - Yes
- Bitstamp - Yes
- Gemini - Yes
- KuCoin - Yes
- Huobi - Yes
- OKX - Yes
- Bybit - Yes

**Tier 2 (High Volume):**
- Bittrex - Yes
- Poloniex - Yes
- Gate.io - Yes
- Crypto.com - Yes
- FTX - No (defunct)
- Bitso - Yes
- Liquid - Yes
- HitBTC - Yes
- CEX.IO - Yes

**Regional Exchanges:**
- bitFlyer (Japan) - Yes
- Upbit (South Korea) - Yes
- Bithumb (South Korea) - Yes
- Coincheck (Japan) - Yes
- Zaif (Japan) - Yes
- BTCBOX (Japan) - Yes
- QuadrigaCX (Canada) - No (defunct)

### Decentralized Exchanges (DEX)
Limited DEX coverage. CryptoCompare primarily focuses on CEX data.

- Uniswap - Limited (aggregate data, not full DEX analytics)
- PancakeSwap - Limited
- SushiSwap - Limited
- Curve - Limited
- **Note:** For comprehensive DEX data, use specialized providers (Bitquery, Dune Analytics)

### Derivatives Exchanges
Limited coverage. Spot data is primary focus.

- Binance Futures - Limited
- Bybit (derivatives) - Limited
- Deribit - Limited
- **Note:** CryptoCompare focuses on spot markets. For derivatives analytics, use exchange APIs or Coinglass.

### CCCAGG (Aggregate Index)
CryptoCompare's proprietary aggregate index combines data from **170+ exchanges** to provide volume-weighted average prices.

**Exchanges in CCCAGG include:**
- All major tier 1 exchanges
- Selected tier 2 exchanges
- Weighted by volume and reliability
- Updated in real-time

## Instrument Coverage

### Cryptocurrencies
- **Total coins:** 5,700+ cryptocurrencies
- **Active trading:** ~3,000+ with significant volume
- **Coverage includes:**
  - All major coins (BTC, ETH, BNB, XRP, ADA, SOL, etc.)
  - Mid-cap altcoins
  - Small-cap coins
  - Newly listed tokens (updated regularly)
  - Stablecoins (USDT, USDC, DAI, BUSD, etc.)
  - DeFi tokens
  - NFT-related tokens
  - Meme coins

### Trading Pairs
- **Total pairs:** 260,000+ trading pairs
- **Spot pairs:** 260,000+
- **Futures pairs:** Limited coverage
- **Perpetuals:** Limited coverage
- **Margin pairs:** Data available where exchanges provide it

### Pair Categories
- **Crypto-to-Crypto:** Yes (BTC/ETH, ETH/BNB, etc.)
- **Crypto-to-Fiat:** Yes (BTC/USD, ETH/EUR, etc.)
- **Crypto-to-Stablecoin:** Yes (BTC/USDT, ETH/USDC, etc.)
- **Cross-exchange pairs:** Aggregated via CCCAGG

### Major Coins Coverage (Top 100 by Market Cap)
- Bitcoin (BTC) - Full history since 2010
- Ethereum (ETH) - Full history since 2015
- BNB - Full history
- XRP - Full history
- Cardano (ADA) - Full history
- Solana (SOL) - Full history since launch
- Dogecoin (DOGE) - Full history
- Polygon (MATIC) - Full history
- Polkadot (DOT) - Full history
- **All top 100 coins:** Comprehensive coverage

### Altcoin Coverage
- **Top 100:** Excellent coverage (full history, all major exchanges)
- **Top 500:** Good coverage (most exchanges, full daily history)
- **Top 1000:** Fair coverage (major exchanges, daily history)
- **Beyond top 1000:** Limited (smaller exchanges, may have gaps)

### Stablecoins
- USDT (Tether) - Full coverage
- USDC (USD Coin) - Full coverage
- BUSD (Binance USD) - Full coverage
- DAI - Full coverage
- USDD - Full coverage
- TUSD - Full coverage
- USDP - Full coverage

### DeFi Tokens
- UNI (Uniswap) - Full coverage
- AAVE - Full coverage
- COMP (Compound) - Full coverage
- MKR (Maker) - Full coverage
- SNX (Synthetix) - Full coverage
- LINK (Chainlink) - Full coverage
- CRV (Curve) - Full coverage
- **100+ DeFi tokens:** Good coverage

### Stock Markets
CryptoCompare does NOT cover traditional stock markets.

- US stocks (NYSE, NASDAQ, AMEX) - No
- International stocks - No
- **Note:** Use Polygon, Alpha Vantage for stock data

### Forex
CryptoCompare does NOT cover traditional forex pairs.

- Traditional forex (EUR/USD, GBP/USD, etc.) - No
- Crypto-to-fiat pairs (BTC/USD, ETH/EUR) - Yes (this is crypto, not forex)
- **Note:** Use OANDA, Forex.com APIs for traditional forex

### Commodities
CryptoCompare does NOT cover traditional commodities.

- Metals (Gold, Silver) - No
- Energy (Oil, Gas) - No
- Agriculture - No
- **Note:** Use specialized commodity data providers

### Indices
- Crypto indices: Yes (CCCAGG proprietary index)
- Traditional indices (S&P500, Nasdaq) - No
- Crypto market cap index - Yes (via aggregated data)
- DeFi index - Limited (can be calculated from component coins)
- **Bitcoin Dominance:** Can be calculated from available data

## Data History

### Historical Depth

**By Granularity:**

| Granularity | Free Tier | Paid Tier | Enterprise | Notes |
|-------------|-----------|-----------|------------|-------|
| **Daily** | Full history | Full history | Full history | BTC: back to 2010, ETH: back to 2015 |
| **Hourly** | Full history | Full history | Full history | Same as daily |
| **Minute** | 7 days | 1 year | Unlimited | Limited on free tier |
| **Tick/Trade** | Not available | Not available | Available | Raw trade data |

**By Coin:**

| Coin | Daily Data Depth | Notes |
|------|------------------|-------|
| Bitcoin (BTC) | 2010 - present | ~14 years of data |
| Ethereum (ETH) | 2015 - present | ~9 years of data |
| Litecoin (LTC) | 2013 - present | ~11 years of data |
| XRP | 2013 - present | ~11 years of data |
| New coins | From listing | Data starts when coin lists on exchange |

**Historical Coverage:**
- **Major coins (top 50):** Full history from coin inception or exchange listing
- **Mid-cap coins:** Full daily/hourly history, limited minute history
- **Small-cap coins:** Daily history available, hourly may have gaps
- **Newly listed:** Data starts from listing date

### Granularity Available

| Granularity | Availability | Free Tier Depth | API Endpoint |
|-------------|--------------|-----------------|--------------|
| **Tick data** | Enterprise only | Not available | Enterprise API |
| **1-second bars** | Not available | - | - |
| **1-minute bars** | Yes | 7 days | `/data/histominute` |
| **5-minute bars** | Yes (aggregate) | 7 days | `/data/histominute` + aggregate |
| **15-minute bars** | Yes (aggregate) | 7 days | `/data/histominute` + aggregate |
| **Hourly bars** | Yes | Full history | `/data/histohour` |
| **4-hour bars** | Yes (aggregate) | Full history | `/data/histohour` + aggregate |
| **Daily bars** | Yes | Full history | `/data/histoday` |
| **Weekly bars** | Yes (aggregate) | Full history | `/data/histoday` + aggregate |
| **Monthly bars** | Yes (aggregate) | Full history | `/data/histoday` + aggregate |

**Aggregation:**
- Use `aggregate` parameter to create custom intervals
- Example: `aggregate=5` on minute data = 5-minute bars
- Example: `aggregate=7` on daily data = weekly bars

### Real-time vs Delayed

- **Real-time:** Yes (with ~10-second server-side cache)
- **Delayed:** No (all data is real-time, subject to cache)
- **Snapshot:** Yes (current price endpoints)

**Cache Duration:**
- Current prices: 10 seconds
- Ticker data: 10 seconds
- Historical data: Longer cache (immutable once period closes)
- News: Real-time to 1 minute

**WebSocket:**
- Real-time updates (no delay)
- Trade stream: Immediate
- Ticker stream: Real-time
- Orderbook: Real-time (paid tier)

## Update Frequency

### Real-time Streams (WebSocket)

| Data Type | Update Frequency | Notes |
|-----------|------------------|-------|
| **Trades** | Immediate | Every trade pushed instantly |
| **Ticker** | Real-time | On every price change |
| **Aggregate Ticker** | Real-time | CCCAGG updated on every trade |
| **Orderbook** | Real-time | Snapshot + deltas (paid tier) |
| **OHLC** | Per interval | Pushed when interval closes |
| **Volume** | Real-time | Updated on every trade |

### REST API Polling

| Endpoint | Cache Duration | Recommended Poll Frequency |
|----------|----------------|---------------------------|
| `/data/price` | 10 seconds | Every 10-15 seconds |
| `/data/pricemultifull` | 10 seconds | Every 10-15 seconds |
| `/data/histominute` | 1 minute | Every 1-2 minutes |
| `/data/histohour` | 1 hour | Every hour |
| `/data/histoday` | 1 day | Daily |
| `/data/v2/news/` | 1 minute | Every 1-5 minutes |
| `/data/social/coin/latest` | 1 hour | Every hour |

**Note:** Polling faster than cache duration wastes rate limit quota.

### Scheduled Updates

| Data Type | Update Frequency | Notes |
|-----------|------------------|-------|
| **News** | Real-time | As published by sources |
| **Social stats** | Hourly | Updated every hour |
| **Blockchain data** | Daily | Updated once per day (after day closes) |
| **Coin list** | Weekly | New coins added weekly |
| **Exchange list** | Monthly | Updated as exchanges added/removed |

### Historical Data Updates

- **Closed periods:** Immutable (daily/hourly bars don't change once period closes)
- **Current period:** Updates continuously until period closes
- **Backfill:** Historical data may be backfilled if gaps found
- **Corrections:** Rare, but can occur if exchange data corrected

## Data Quality

### Accuracy

- **Source:** Direct from exchanges (API connections)
- **Aggregation:** CCCAGG uses volume-weighted averaging
- **Validation:** Automated checks for outliers
- **Corrections:** Automated and manual correction processes

**CCCAGG Calculation:**
```
Price = Sum(Exchange_Price * Exchange_Volume) / Sum(Exchange_Volume)
```

**Quality by Exchange Tier:**
- **Tier 1 exchanges:** Excellent (Binance, Coinbase, Kraken)
- **Tier 2 exchanges:** Good (most verified exchanges)
- **Small exchanges:** Fair (may have occasional gaps or outliers)

### Completeness

- **Missing data:** Rare for major coins, occasional for small-cap coins
- **Gaps:** May occur during exchange downtime or maintenance
- **Backfill:** Available (historical gaps can be filled via support request)

**Common Gaps:**
- New coin listings (data starts from listing, no pre-listing data)
- Exchange outages (gaps during downtime)
- Delisted coins (data stops at delisting)
- Small exchanges going offline

**Handling Missing Data:**
- Use CCCAGG (aggregate) for most reliable data
- Falls back to alternate exchanges if primary source unavailable
- Historical gaps: contact support for backfill

### Timeliness

- **Latency:** <1 second for WebSocket, ~10 seconds for REST (due to cache)
- **Delay:** No intentional delay (real-time data)
- **Market hours:** 24/7 coverage (crypto never sleeps)

**REST API Latency:**
- First request: Fetches fresh data from exchanges (~500ms-2s)
- Cached request: Instant (served from cache)
- Cache expires: 10 seconds (then fresh fetch)

**WebSocket Latency:**
- Trade stream: <100ms from exchange to client
- Ticker stream: <500ms
- Aggregate stream: <1s (time to aggregate across exchanges)

### Reliability

- **Uptime:** High (99%+ SLA for enterprise)
- **Redundancy:** Multiple data sources per symbol
- **Failover:** Automatic failover to alternate exchanges

**Known Issues:**
- Occasional API slowdowns during extreme volatility
- WebSocket disconnects (rare, but recommend auto-reconnect)
- Rate limit errors if not managed properly

## Coverage Comparison

| Provider | Coins | Exchanges | Pairs | Historical Depth (Daily) | Special Features |
|----------|-------|-----------|-------|--------------------------|------------------|
| **CryptoCompare** | 5,700+ | 170-316 | 260,000+ | Full history | CCCAGG aggregate, social data |
| CoinGecko | 14,000+ | 600+ | Similar | 1-2 years (free) | More coins, DeFi focus |
| CoinMarketCap | 10,000+ | 500+ | Similar | Limited (free) | Market cap focus |
| Binance API | 600+ | 1 (Binance) | 2,000+ | Full history | Direct exchange, no aggregation |
| Messari | 500+ | Limited | Limited | Full history | Fundamental data, research |

**CryptoCompare Strengths:**
- Excellent aggregation (CCCAGG)
- Long historical depth for major coins
- Social media metrics
- News aggregation
- Consistent API across all exchanges

**CryptoCompare Weaknesses:**
- Fewer total coins than CoinGecko/CMC
- Limited DEX coverage
- No derivatives analytics
- No on-chain analytics

## Recommendations

### Best Use Cases:
1. **Multi-exchange price aggregation** - Use CCCAGG for reliable average price
2. **Historical analysis** - Excellent daily/hourly data depth for major coins
3. **Social sentiment tracking** - Unique multi-platform social data
4. **News monitoring** - Aggregated crypto news from multiple sources
5. **Portfolio tracking** - Wide coverage of coins and exchanges

### Not Recommended For:
1. **Derivatives trading** - Use exchange APIs or Coinglass
2. **DEX analytics** - Use Bitquery or Dune Analytics
3. **On-chain analysis** - Use blockchain explorers or specialized providers
4. **Traditional finance** - Use Polygon, Alpha Vantage, or Bloomberg

### Coverage Summary:

| Category | Coverage | Quality | Notes |
|----------|----------|---------|-------|
| **Spot Crypto** | Excellent | Excellent | Core strength |
| **Historical Data** | Excellent | Excellent | Full history for major coins |
| **Exchange Coverage** | Excellent | Good | 170-316 exchanges |
| **Derivatives** | Poor | N/A | Not a focus |
| **DEX Data** | Poor | N/A | Use specialized providers |
| **Social Data** | Excellent | Good | Unique offering |
| **News** | Good | Good | Multi-source aggregation |
| **On-chain** | Poor | Fair | Basic blockchain stats only |

**Overall:** CryptoCompare is excellent for spot crypto market data, historical analysis, and social metrics. Not suitable for derivatives, DEX, or on-chain analytics.
