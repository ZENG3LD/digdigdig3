# CryptoCompare - Data Types Catalog

## Standard Market Data

- [x] Current Price - Single symbol, multiple symbols, full data
- [x] Bid/Ask Spread - Available in full ticker data (CURRENT, CURRENTAGG)
- [x] 24h Ticker Stats - High, low, volume, change%, open
- [x] OHLC/Candlesticks - Intervals: 1m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 12h, 1d, 1w, 1M
- [x] Level 2 Orderbook - Paid tier only (WebSocket Channel 16)
- [x] Recent Trades - Real-time via WebSocket (Channel 0)
- [x] Volume - 24h, hourly, daily, by exchange and aggregated

### Available Price Endpoints
- `/data/price` - Simple current price
- `/data/pricemulti` - Matrix of prices (multiple from/to symbols)
- `/data/pricemultifull` - Full ticker data (OHLCV, volume, market cap, change%)
- `/data/generateAvg` - Volume-weighted average across exchanges
- `/data/dayAvg` - Daily average based on hourly VWAP

### Ticker Data Fields
From `pricemultifull` and WebSocket `CURRENT`/`CURRENTAGG`:
- Last price, bid, ask
- 24h high, low, open
- 24h volume (base and quote currency)
- Today volume (from 00:00 GMT)
- Hourly volume
- Last trade volume and value
- Change (absolute and percentage)
- Last market where trade occurred

## Historical Data

- [x] Historical prices - Depth: Full history (varies by granularity)
- [x] Minute bars - Available: Yes (7 days free, 1 year paid, unlimited enterprise)
- [x] Hourly bars - Depth: Full history (years)
- [x] Daily bars - Depth: Full history (10+ years for major coins)
- [ ] Tick data - Available: Paid tier only (enterprise)
- [x] Adjusted prices - Not applicable (crypto doesn't have splits/dividends)

### Historical Endpoints
- `/data/pricehistorical` - Price at specific timestamp (end of day GMT)
- `/data/histoday` - Daily OHLCV bars
- `/data/histohour` - Hourly OHLCV bars
- `/data/histominute` - Minute OHLCV bars
- `/data/v2/histoday` - Enhanced daily bars (v2)
- `/data/v2/histohour` - Enhanced hourly bars (v2)
- `/data/v2/histominute` - Enhanced minute bars (v2)

### Historical Data Depth
| Granularity | Free Tier | Paid Tier | Enterprise |
|-------------|-----------|-----------|------------|
| Daily | Full history | Full history | Full history |
| Hourly | Full history | Full history | Full history |
| Minute | 7 days | 1 year | Unlimited |
| Tick/Trade | Not available | Not available | Available |

### OHLCV Fields
- `time` - Timestamp (Unix seconds)
- `open` - Open price
- `high` - High price
- `low` - Low price
- `close` - Close price
- `volumefrom` - Volume in base currency
- `volumeto` - Volume in quote currency
- Additional fields in v2: conversionType, conversionSymbol

## Derivatives Data (Crypto/Futures)

CryptoCompare is primarily a spot data aggregator. Derivatives data is limited.

- [ ] Open Interest - Not available
- [ ] Funding Rates - Not available
- [ ] Liquidations - Not available
- [ ] Long/Short Ratios - Not available
- [ ] Mark Price - Not available
- [ ] Index Price - CCCAGG index available (proprietary aggregate)
- [ ] Basis (futures - spot spread) - Not available

**Note:** For derivatives data, use exchange-specific APIs (Binance, Bybit, etc.) or specialized providers (Coinglass, Glassnode).

## Options Data

- [ ] Options Chains - Not available
- [ ] Implied Volatility - Not available
- [ ] Greeks - Not available
- [ ] Open Interest (per strike) - Not available
- [ ] Historical option prices - Not available

**Note:** CryptoCompare does not provide options data.

## Fundamental Data (Stocks)

CryptoCompare is crypto-focused. No stock fundamental data.

- [ ] Company Profile - Not applicable
- [ ] Financial Statements - Not applicable
- [ ] Earnings - Not applicable
- [ ] Dividends - Not applicable
- [ ] Stock Splits - Not applicable
- [ ] Analyst Ratings - Not applicable
- [ ] Insider Trading - Not applicable
- [ ] Institutional Holdings - Not applicable
- [ ] Financial Ratios - Not applicable
- [ ] Valuation Metrics - Not applicable

**Note:** For stocks, use Polygon, Alpha Vantage, or similar providers.

## On-chain Data (Crypto)

Limited on-chain data available.

- [x] Blockchain statistics - Daily stats (transactions, hashrate, etc.)
- [ ] Wallet Balances - Not available
- [ ] Transaction History - Not available
- [ ] DEX Trades - Not available
- [ ] Token Transfers - Not available
- [ ] Smart Contract Events - Not available
- [ ] Gas Prices - Not available
- [ ] Block Data - Limited (via blockchain endpoints)
- [ ] NFT Data - Not available

### Available Blockchain Endpoints
- `/data/blockchain/list` - List of supported blockchains
- `/data/blockchain/histo/day` - Daily blockchain statistics
- `/data/blockchain/latest` - Latest blockchain data
- `/data/blockchain/mining/calculator` - Mining profitability calculator

### Blockchain Data Fields
From `/data/blockchain/histo/day`:
- Block time
- Block size
- Transaction count
- Hashrate (for PoW coins)
- Difficulty
- Block reward
- Network total
- Active addresses (limited)

**Note:** For comprehensive on-chain data, use Bitquery, Dune Analytics, or blockchain explorers.

## Macro/Economic Data (Economics)

CryptoCompare does not provide macroeconomic data.

- [ ] Interest Rates - Not available
- [ ] GDP - Not available
- [ ] Inflation - Not available
- [ ] Employment - Not available
- [ ] Retail Sales - Not available
- [ ] Industrial Production - Not available
- [ ] Consumer Confidence - Not available
- [ ] PMI - Not available
- [ ] Economic Calendar - Not available

**Note:** For economic data, use FRED API, Trading Economics, or similar providers.

## Forex Specific

CryptoCompare does not provide traditional forex data (EUR/USD, GBP/USD, etc.).

- [ ] Currency Pairs (traditional forex) - Not available
- [ ] Bid/Ask Spreads (forex) - Not available
- [ ] Pip precision - Not applicable
- [ ] Cross rates - Not applicable
- [ ] Historical FX rates - Not applicable

**Note:** Some crypto pairs like BTC/USD can be considered crypto-to-fiat, but not traditional forex.

## Metadata & Reference

- [x] Symbol/Instrument Lists - Complete coin list with metadata
- [x] Exchange Information - All exchanges and their trading pairs
- [x] Market Hours - Not applicable (crypto trades 24/7)
- [ ] Trading Calendars - Not applicable
- [ ] Timezone Info - Timestamps in Unix seconds (UTC)
- [x] Sector/Industry Classifications - Coin categories/tags (limited)

### Metadata Endpoints
- `/data/all/coinlist` - All coins with metadata
- `/data/all/exchanges` - All exchanges and trading pairs
- `/data/blockchain/list` - Supported blockchains

### Coin Metadata Fields
From `/data/all/coinlist`:
- `Id` - Internal ID
- `Symbol` - Coin symbol (BTC, ETH)
- `CoinName` - Full name (Bitcoin, Ethereum)
- `FullName` - Display name
- `Algorithm` - Mining algorithm (if PoW)
- `ProofType` - Consensus mechanism
- `TotalCoinSupply` - Max supply
- `ImageUrl` - Logo image URL
- `Url` - CryptoCompare page URL
- `SortOrder` - Ranking by market cap
- Additional: BuiltOn, SmartContractAddress (for tokens)

### Exchange Metadata
From `/data/all/exchanges`:
- Exchange name
- Trading pairs available
- Internal ID

## News & Sentiment

- [x] News Articles - Aggregated from multiple sources
- [ ] Press Releases - Not separate category
- [x] Social Sentiment - Reddit, Twitter, Facebook, GitHub metrics
- [ ] Analyst Reports - Not available

### News Endpoints
- `/data/v2/news/` - Latest news articles
- `/data/news/feeds` - Available news feeds/sources
- `/data/news/categories` - News categories

### News Data Fields
- `id` - Article ID
- `guid` - Unique identifier
- `published_on` - Timestamp
- `imageurl` - Article image
- `title` - Article title
- `url` - Article URL
- `source` - News source
- `body` - Article body/excerpt
- `tags` - Related coins/topics
- `categories` - Article categories
- `lang` - Language
- `source_info` - Source metadata

### Social Stats Endpoints
- `/data/social/coin/latest` - Latest social metrics
- `/data/social/coin/histo/day` - Daily historical social stats
- `/data/social/coin/histo/hour` - Hourly historical social stats

### Social Data Fields
- **Reddit:**
  - `subscribers` - Subreddit subscribers
  - `active_users` - Active users
  - `posts_per_hour` - Posting frequency
  - `posts_per_day`
  - `comments_per_hour`
  - `comments_per_day`
  - `community_creation` - Subreddit creation date

- **Twitter:**
  - `followers` - Account followers
  - `following` - Following count
  - `lists` - Lists mentioning account
  - `favourites` - Tweets favorited
  - `statuses` - Total tweets
  - `account_creation` - Account creation date
  - `Points` - CryptoCompare social score

- **Facebook:**
  - `likes` - Page likes
  - `talking_about` - People talking about
  - `Points` - Social score

- **GitHub:**
  - `stars` - Repository stars
  - `forks` - Repository forks
  - `subscribers` - Repository watchers
  - `issues` - Open issues
  - `closed_issues` - Closed issues
  - `contributors` - Number of contributors
  - `created_at` - Repository creation

- **Aggregated:**
  - `CryptoCompare.Points` - Overall social score
  - `CryptoCompare.Followers`
  - `CryptoCompare.Posts`
  - `CryptoCompare.Comments`

## Top Lists & Rankings

- [x] Top Exchanges - By volume for a pair
- [x] Top Pairs - By volume for a coin
- [x] Top Coins by Volume - 24h total volume
- [x] Top Coins by Market Cap - Market cap ranking

### Top List Endpoints
- `/data/top/exchanges` - Top exchanges for pair
- `/data/top/exchanges/full` - Full data on top exchanges
- `/data/top/pairs` - Top trading pairs for symbol
- `/data/top/volumes` - Top coins by 24h volume
- `/data/top/mktcapfull` - Top coins by market cap (full data)
- `/data/top/totalvolfull` - Top coins by total volume across all markets

## Unique/Custom Data

### What makes CryptoCompare special?

#### 1. CCCAGG (CryptoCompare Aggregate Index)
- **Proprietary index** aggregating data from 170+ exchanges
- Volume-weighted average price
- More accurate than single-exchange prices
- Available via `e=CCCAGG` parameter or WebSocket Channel 5
- Used as benchmark by many applications

#### 2. Multi-Exchange Aggregation
- Aggregates data from 170-316 exchanges (sources vary)
- Single API for all major exchanges
- Consistent format across exchanges
- Historical data going back years

#### 3. Social Metrics
- **Unique multi-platform social data**
- Reddit, Twitter, Facebook, GitHub combined
- Proprietary "Points" scoring system
- Historical social stats (hourly and daily)
- Useful for sentiment analysis

#### 4. News Aggregation
- Multiple crypto news sources in one API
- Filtered by coin, category, language
- Real-time updates
- Tagged with related coins

#### 5. Comprehensive Coverage
- 5,700+ cryptocurrencies
- 260,000+ trading pairs
- Both spot and (limited) derivatives
- Metadata for all coins (algorithm, supply, etc.)

#### 6. Blockchain Statistics
- Historical blockchain data (hashrate, difficulty, transactions)
- Mining calculator
- Network statistics

#### 7. Long Historical Depth
- Daily data going back to coin inception
- Major coins have 10+ years of daily data
- Hourly data for full history (most coins)

## Data Not Available

For completeness, here's what CryptoCompare does NOT provide:

### Not Available:
- Traditional forex pairs (EUR/USD, etc.)
- Stock market data
- Futures/derivatives specific data (funding, OI, liquidations)
- Options data
- Macroeconomic indicators
- Full on-chain analytics (use Bitquery, Dune)
- NFT market data
- DEX-specific analytics
- Detailed order flow / Level 3 orderbook
- Raw tick data (free tier)

### Use Alternative Providers:
- **Derivatives:** Coinglass, exchange APIs
- **On-chain:** Bitquery, Dune Analytics, Etherscan
- **Stocks:** Polygon, Alpha Vantage
- **Forex:** OANDA, Forex.com APIs
- **Economic:** FRED, Trading Economics
- **NFTs:** OpenSea API, Moralis

## Summary of Available Data

| Category | Available | Quality | Notes |
|----------|-----------|---------|-------|
| Current Prices | Yes | Excellent | Multi-exchange aggregation |
| Historical OHLCV | Yes | Excellent | Full history (limited minute data free) |
| Trades (real-time) | Yes | Good | WebSocket only |
| Orderbook | Paid only | Good | Level 2, paid tier |
| News | Yes | Good | Multi-source aggregation |
| Social Stats | Yes | Excellent | Unique multi-platform data |
| Blockchain Stats | Yes | Good | Basic blockchain metrics |
| Exchange Metadata | Yes | Excellent | Comprehensive exchange list |
| Coin Metadata | Yes | Excellent | All coins with details |
| Derivatives | No | N/A | Use exchange APIs |
| On-chain | Limited | Fair | Use specialized providers |
| Economic Data | No | N/A | Use FRED, etc. |

**CryptoCompare excels at:** Aggregated spot market data, social metrics, historical data, multi-exchange coverage.

**Not suitable for:** Derivatives analytics, deep on-chain analysis, traditional finance data.
