# Twelvedata - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: Yes (comprehensive - US, Canada)
- **Europe**: Yes (90+ exchanges including major markets)
- **Asia**: Yes (major markets - Japan, China, Hong Kong, Singapore, India, etc.)
- **Oceania**: Yes (Australia, New Zealand)
- **South America**: Yes (Brazil, Argentina, etc.)
- **Africa**: Limited (South Africa, Egypt)
- **Middle East**: Limited (Saudi Arabia, UAE)

### Country-Specific Coverage

#### Major Markets
- **United States**: Yes (NYSE, NASDAQ, AMEX - comprehensive)
- **United Kingdom**: Yes (LSE - London Stock Exchange)
- **Japan**: Yes (TSE - Tokyo Stock Exchange)
- **China**: Yes (SSE, SZSE - Shanghai, Shenzhen)
- **Hong Kong**: Yes (HKEX)
- **India**: Yes (NSE, BSE - National Stock Exchange, Bombay)
- **Germany**: Yes (XETRA, Frankfurt)
- **France**: Yes (Euronext Paris)
- **Canada**: Yes (TSX - Toronto Stock Exchange)
- **Australia**: Yes (ASX - Australian Securities Exchange)
- **South Korea**: Yes (KRX)
- **Switzerland**: Yes (SIX Swiss Exchange)
- **Netherlands**: Yes (Euronext Amsterdam)
- **Italy**: Yes (Borsa Italiana)
- **Spain**: Yes (BME Spanish Exchanges)
- **Brazil**: Yes (B3 - Brasil Bolsa Balcão)
- **Russia**: Limited (MOEX - Moscow Exchange, data availability may vary due to sanctions)
- **Singapore**: Yes (SGX)
- **Taiwan**: Yes (TWSE)
- **South Africa**: Yes (JSE)

#### Total Coverage
- **Countries**: 90+ countries
- **Stock Exchanges**: 90+ exchanges
- **Crypto Exchanges**: 180+ exchanges

### Restricted Regions
- **Blocked countries**: None explicitly documented (API accessible globally)
- **VPN detection**: No (not mentioned in documentation)
- **Geo-fencing**: No (API available worldwide)
- **Sanctions compliance**: Twelvedata complies with international sanctions (some symbols may be unavailable)

**Note**: API is globally accessible, but data availability for certain countries/symbols may be limited by data provider agreements or sanctions.

## Markets/Exchanges Covered

### Stock Markets

#### United States
- **NYSE** (New York Stock Exchange): Yes - full coverage
- **NASDAQ**: Yes - full coverage
- **AMEX** (American Stock Exchange): Yes
- **BATS**: Yes (CBOE BZX Exchange)
- **OTC Markets**: Limited/No (not explicitly mentioned)

#### Europe
- **LSE** (London Stock Exchange): Yes
- **XETRA** (Germany): Yes
- **Euronext** (Amsterdam, Brussels, Dublin, Lisbon, Paris): Yes
- **BME** (Spain): Yes
- **Borsa Italiana** (Italy): Yes
- **SIX Swiss Exchange** (Switzerland): Yes
- **Oslo Børs** (Norway): Yes
- **OMX Stockholm** (Sweden): Yes
- **OMX Copenhagen** (Denmark): Yes
- **OMX Helsinki** (Finland): Yes
- **OMX Iceland**: Yes
- **Warsaw Stock Exchange** (Poland): Yes
- **Prague Stock Exchange** (Czech Republic): Yes
- **Athens Exchange** (Greece): Yes
- **MOEX** (Moscow Exchange): Limited

#### Asia-Pacific
- **TSE** (Tokyo Stock Exchange): Yes
- **SSE** (Shanghai Stock Exchange): Yes
- **SZSE** (Shenzhen Stock Exchange): Yes
- **HKEX** (Hong Kong Exchanges): Yes
- **NSE** (India): Yes
- **BSE** (Bombay Stock Exchange): Yes
- **KRX** (Korea Exchange): Yes
- **ASX** (Australian Securities Exchange): Yes
- **SGX** (Singapore Exchange): Yes
- **TWSE** (Taiwan Stock Exchange): Yes
- **SET** (Stock Exchange of Thailand): Yes
- **IDX** (Indonesia Stock Exchange): Yes
- **PSE** (Philippine Stock Exchange): Yes

#### Americas (Non-US)
- **TSX** (Toronto Stock Exchange): Yes
- **TSXV** (TSX Venture Exchange): Yes
- **B3** (Brazil): Yes
- **BMV** (Mexican Stock Exchange): Yes
- **BCBA** (Buenos Aires Stock Exchange): Yes

#### Middle East & Africa
- **JSE** (Johannesburg Stock Exchange): Yes
- **Tadawul** (Saudi Stock Exchange): Yes
- **DFM** (Dubai Financial Market): Yes
- **EGX** (Egyptian Exchange): Yes

**Total**: 90+ stock exchanges globally

### Crypto Exchanges (180+ Supported)

#### Major Centralized Exchanges
- **Binance**: Yes (largest coverage)
- **Coinbase**: Yes
- **Kraken**: Yes
- **Bitfinex**: Yes
- **Huobi**: Yes
- **OKX**: Yes
- **Bybit**: Yes
- **KuCoin**: Yes
- **Gate.io**: Yes
- **Gemini**: Yes
- **Bitstamp**: Yes
- **Bittrex**: Yes
- **Poloniex**: Yes
- **Crypto.com**: Yes

#### Decentralized Exchanges (DEX)
**Not explicitly supported** - Twelvedata focuses on centralized exchange spot prices, not DEX data.

For DEX data, use specialized providers like Bitquery, The Graph, or Dune Analytics.

**Total**: 180+ crypto exchanges

### Forex Brokers
**Not exchange-based** - Forex data is aggregated from interbank/broker feeds, not specific brokers.

- **Coverage**: 200+ currency pairs
- **Data source**: Aggregated institutional/interbank rates
- **No specific broker names** listed

### Futures/Options Exchanges
**Limited/Not Explicitly Supported**

- **CME** (Chicago Mercantile Exchange): Not explicitly mentioned
- **CBOE** (Chicago Board Options Exchange): Not explicitly mentioned
- **Eurex**: Not explicitly mentioned

**Note**: Twelvedata focuses on spot prices (stocks, forex, crypto) and does not have comprehensive derivatives exchange coverage. No futures/options chain data available.

## Instrument Coverage

### Stocks
- **Total symbols**: ~60,000+ (estimated across all exchanges)
- **US stocks**: ~10,000+ (NYSE, NASDAQ, AMEX)
- **International stocks**: ~50,000+ (90+ exchanges)
- **OTC**: Limited/No (not explicitly mentioned)
- **Penny stocks**: Yes (included if listed on major exchanges)
- **ADRs** (American Depositary Receipts): Yes
- **Foreign listings on US exchanges**: Yes

### Crypto
- **Total coins/tokens**: Thousands (exact number not specified)
- **Spot pairs**: Thousands across 180+ exchanges
  - BTC/USD, ETH/USD, BTC/USDT, etc.
  - Altcoin pairs (e.g., ADA/USD, SOL/USD, DOGE/USD)
  - Cross-crypto pairs (e.g., ETH/BTC, LTC/BTC)
- **Futures**: No (not supported)
- **Perpetuals**: No (not supported)
- **Options**: No (not supported)

**Focus**: Spot crypto prices only, no derivatives.

### Forex
- **Currency pairs**: 200+ pairs
- **Majors** (7 pairs): Yes
  - EUR/USD, GBP/USD, USD/JPY, USD/CHF, AUD/USD, USD/CAD, NZD/USD
- **Minors**: ~30+ pairs
  - EUR/GBP, EUR/JPY, GBP/JPY, AUD/JPY, EUR/CHF, etc.
- **Exotics**: ~170+ pairs
  - USD/TRY, EUR/TRY, USD/ZAR, USD/MXN, USD/BRL, etc.
- **Cross rates**: Calculated on-the-fly via `/time_series/cross` (5 credits)

### ETFs
- **Total ETFs**: ~5,000+ (US + international)
- **US ETFs**: ~3,000+ (major coverage)
- **International ETFs**: ~2,000+
- **Types**: Equity, bond, commodity, currency, leveraged, inverse, sector, thematic

**Examples**: SPY, QQQ, IWM, GLD, TLT, EEM, VTI, VOO, etc.

### Mutual Funds
- **Total funds**: ~20,000+ (estimated)
- **US mutual funds**: Comprehensive
- **International funds**: Limited
- **Fund families**: Vanguard, Fidelity, American Funds, T. Rowe Price, etc.

### Commodities
- **Total commodity pairs**: 50+ (estimated)
- **Metals**:
  - Gold (XAU/USD, XAU/EUR, etc.)
  - Silver (XAG/USD, XAG/EUR)
  - Platinum (XPT/USD)
  - Palladium (XPD/USD)
  - Copper (XCU/USD)
- **Energy**:
  - Crude Oil WTI (CL)
  - Brent Crude Oil (BZ)
  - Natural Gas (NG)
  - Heating Oil
  - Gasoline
- **Agriculture**:
  - Corn (ZC)
  - Wheat (ZW)
  - Soybeans (ZS)
  - Coffee (KC)
  - Sugar (SB)
  - Cotton (CT)
  - Cocoa (CC)

### Bonds
- **Coverage**: Limited (not a primary focus)
- **Government bonds**: Some coverage (US Treasuries, etc.)
- **Corporate bonds**: Limited

**Note**: For comprehensive bond data, use specialized fixed-income providers.

### Indices
- **Total indices**: 100+ global indices
- **US Indices**:
  - S&P 500 (SPX)
  - Nasdaq 100 (NDX)
  - Dow Jones Industrial Average (DJI)
  - Russell 2000 (RUT)
  - S&P 400 MidCap (MID)
  - Wilshire 5000
- **International Indices**:
  - FTSE 100 (FTSE) - UK
  - DAX (GDAXI) - Germany
  - CAC 40 (FCHI) - France
  - Nikkei 225 (N225) - Japan
  - Hang Seng Index (HSI) - Hong Kong
  - Shanghai Composite (SSEC) - China
  - Sensex (BSESN) - India
  - ASX 200 (AXJO) - Australia
  - IBOVESPA (BVSP) - Brazil
- **Crypto Indices**:
  - BTC Dominance
  - Crypto Total Market Cap
  - DeFi Index (if available)

## Data History

### Historical Depth by Asset Type

#### Stocks
- **US major stocks**: From 1980s-1990s (or listing date if later)
- **Example**: AAPL from 1980, MSFT from 1986
- **International stocks**: Varies (typically from 2000s for most)
- **Newer listings**: From IPO date onward
- **Daily data**: Decades for most major stocks
- **Intraday data**: 1-2 years (Basic plan), 5+ years (Grow+), unlimited (Pro+)

#### Crypto
- **Bitcoin**: From ~2010-2011 (exchange listing date)
- **Ethereum**: From ~2015-2016
- **Altcoins**: From listing date on supported exchanges
- **Varies by exchange**: Different exchanges have different historical depths
- **Daily data**: Back to coin inception
- **Intraday data**: 1-2 years typical

#### Forex
- **Major pairs**: Decades (back to 1990s or earlier)
- **Minor pairs**: Typically from 2000s
- **Exotic pairs**: Varies (often from 2010s)
- **Daily data**: Very long history (20-30+ years for majors)
- **Intraday data**: 1-2 years (Basic), 5+ years (Grow+), unlimited (Pro+)

#### ETFs
- **Established ETFs**: From inception date
- **SPY** (oldest ETF): From 1993
- **Most ETFs**: From 2000s onward
- **Daily data**: Full history from inception
- **Intraday data**: 1-2 years (Basic), 5+ years (Grow+)

#### Commodities
- **Gold/Silver**: Decades of history
- **Oil/Gas**: Decades of history
- **Agriculture**: Long history (20+ years typical)

#### Fundamentals
- **Financial statements**: Back to 1980s-1990s for major US companies
- **Earnings**: Complete history (often 10-20+ years)
- **Dividends**: Full dividend history
- **Analyst ratings**: Recent years (typically 2-5 years)

### Granularity Available

#### Intraday Intervals
- [x] **1-minute bars**: Yes (depth: 1-2 years Basic, 5+ Grow, unlimited Pro+)
- [x] **5-minute bars**: Yes (same depth as 1-minute)
- [x] **15-minute bars**: Yes
- [x] **30-minute bars**: Yes
- [x] **45-minute bars**: Yes
- [x] **1-hour bars**: Yes
- [x] **2-hour bars**: Yes
- [x] **4-hour bars**: Yes

**Intraday depth by plan**:
- Basic: 1-2 years
- Grow: 5+ years
- Pro/Ultra: Unlimited (full history)

#### Daily & Longer
- [x] **Daily bars**: Yes (decades for most assets)
- [x] **Weekly bars**: Yes (full history)
- [x] **Monthly bars**: Yes (full history)

#### Tick Data
- [ ] **Individual ticks**: No (not available)
- **Alternative**: Use 1-minute bars (smallest interval)

### Real-time vs Delayed

#### By Plan Tier

**Basic Plan (Free)**:
- Real-time: **No** (delayed/limited)
- Delayed: **Yes** (delay varies by asset type)
- Snapshot: **Yes** (current values, not streaming)

**Grow Plan**:
- Real-time: **No** (focus on historical/fundamentals)
- Delayed: **Yes**
- Snapshot: **Yes**

**Pro Plan**:
- Real-time: **Yes** (true real-time data)
- WebSocket: **Yes** (~170ms latency)
- Extended hours: **Yes** (US stocks, pre/post-market)

**Ultra Plan**:
- Real-time: **Yes** (same as Pro)
- WebSocket: **Yes** (more credits)
- Extended hours: **Yes**

#### Extended Hours (Pro+ plans only)
- **Pre-market**: 4:00 AM - 9:30 AM ET (US stocks)
- **After-hours**: 4:00 PM - 8:00 PM ET (US stocks)
- **Intervals**: 1min, 5min, 15min, 30min
- **Coverage**: US equities only (not international, forex, crypto)

**Note**: Crypto markets trade 24/7 (no extended hours concept).

## Update Frequency

### Real-time Streams (WebSocket, Pro+ plans)
- **Price updates**: Real-time (~170ms average latency)
- **Tick-by-tick**: Yes (every price change)
- **Stocks**: During market hours (9:30 AM - 4:00 PM ET for US)
- **Forex**: 24/5 (Monday-Friday, 24 hours)
- **Crypto**: 24/7 (continuous)

### REST API Polling
- **Not true real-time**: Polling-based, minutely updates
- **Recommended interval**: Every 1-5 minutes (to respect rate limits)
- **Latency**: Depends on polling frequency + API processing

### Scheduled Updates

#### Reference Data
- **Stock catalogs**: Updated daily every 3 hours starting from 12 AM
- **Forex pairs**: Updated daily
- **Crypto list**: Updated daily
- **Exchange info**: Occasional updates (as needed)

#### Fundamentals
- **Financial statements**: Quarterly (within days of company release)
- **Earnings**: Quarterly (within hours of release)
- **Dividends**: As announced (real-time updates to calendar)
- **Analyst ratings**: As published (typically daily updates)
- **Company profile**: As changed (infrequent updates)

#### Economic Data
**Not available** (Twelvedata does not provide economic calendar/data).

#### News
**Not available** (Twelvedata does not provide news feeds).

## Data Quality

### Accuracy
- **Source**: Direct from exchange feeds / Aggregated from institutional providers
- **Validation**: Yes (data quality checks in place)
- **Corrections**: Automatic (price adjustments for splits/dividends)
- **Corporate actions**: Automatically applied when `adjust` parameter used

### Source Details

#### Stocks
- **Direct exchange feeds**: For major exchanges (NYSE, NASDAQ, etc.)
- **Data vendors**: For international exchanges
- **Adjustments**: Splits and dividends automatically applied (optional)

#### Forex
- **Interbank rates**: Aggregated from institutional sources
- **Not broker-specific**: Institutional-grade pricing
- **Spreads**: Indicative spreads (not live broker quotes)

#### Crypto
- **Exchange APIs**: Direct from 180+ crypto exchanges
- **Aggregated**: Prices from specific exchanges (exchange parameter can specify)
- **No aggregated index**: Each exchange reported separately

#### Fundamentals
- **SEC filings**: For US companies (via EDGAR)
- **Company reports**: Quarterly/annual filings
- **Third-party data vendors**: For processed fundamental data
- **Historical accuracy**: High quality back to 1980s for major companies

### Completeness
- **Missing data**: Rare for major symbols, occasional for smaller/international stocks
- **Gaps**: Handled gracefully (null values returned)
- **Backfill**: Available (can request historical data to fill gaps)
- **Pre-IPO data**: Not available (data starts from listing date)

### Null Value Handling
- **Expected behavior**: Many fields may return `null` when unavailable
- **Examples**:
  - `day_volume` may be null for some instruments
  - `fifty_two_week.high` may be null for newly listed stocks
  - Fundamental fields may be null for non-US or small-cap stocks
- **Best practice**: Always check for null before using values

### Timeliness

#### Real-time (Pro+ plans)
- **Latency**: ~170ms average (WebSocket)
- **During market hours**: Continuous updates
- **After hours**: Extended hours data available (US stocks)

#### Delayed (Basic/Grow plans)
- **Delay**: Varies by asset type (not explicitly specified)
- **Typical**: 15-minute delay or end-of-day updates
- **Not suitable for**: Trading (use Pro+ for trading applications)

#### Market Hours Coverage
- **Regular hours**: Fully covered (9:30 AM - 4:00 PM ET for US)
- **Extended hours** (Pro+): Pre-market (4 AM - 9:30 AM) + After-hours (4 PM - 8 PM ET)
- **International**: Varies by exchange (local market hours)
- **Forex**: 24/5 (Monday-Friday)
- **Crypto**: 24/7/365

### Data Adjustments

#### Stock Splits
- **Automatic**: Applied when `adjust=splits` parameter used
- **Historical prices**: Retroactively adjusted
- **Split information**: Available via `/splits` endpoint

#### Dividends
- **Automatic**: Applied when `adjust=dividends` parameter used
- **Ex-dividend adjustment**: Prices adjusted on ex-date
- **Dividend history**: Available via `/dividends` endpoint

#### Corporate Actions
- **Mergers/Acquisitions**: Symbol changes handled
- **Delistings**: Data available until delisting date
- **Ticker changes**: Both old and new tickers supported (with redirect)

## Geographic Restrictions Summary

| Region | Stock Coverage | Forex | Crypto | Real-time (Pro+) | Notes |
|--------|---------------|-------|--------|------------------|-------|
| **North America** | Excellent | Yes | Yes | Yes | US, Canada full coverage |
| **Europe** | Excellent | Yes | Yes | Yes | 90+ exchanges, all major markets |
| **Asia** | Good | Yes | Yes | Yes | Major exchanges covered |
| **Oceania** | Good | Yes | Yes | Yes | Australia, New Zealand |
| **South America** | Limited | Yes | Yes | Yes | Brazil primary, others limited |
| **Africa** | Very Limited | Limited exotics | Yes | Yes | South Africa primary |
| **Middle East** | Limited | Limited exotics | Yes | Yes | Saudi, UAE primary |

## Coverage Gaps

### Not Available
1. **Options chains** - No options data
2. **Futures contracts** - No futures data
3. **Level 2 order book** - No depth beyond bid/ask
4. **Tick-by-tick trades** - Minimum 1-minute bars
5. **On-chain crypto data** - Spot prices only, no blockchain data
6. **Economic indicators** - No macro data (GDP, CPI, etc.)
7. **News feeds** - No news articles
8. **Social sentiment** - No sentiment analysis
9. **DEX data** - Centralized exchanges only
10. **OTC markets** - No OTC/pink sheet coverage
11. **Derivatives analytics** - No funding rates, liquidations, OI for crypto
12. **Broker-specific forex** - Institutional rates only, not retail broker quotes

### Limited Coverage
1. **Bonds** - Basic coverage, not comprehensive
2. **Commodities futures** - Spot-like prices, not actual futures contracts
3. **Small-cap international** - Focus on large/mid-caps
4. **Penny stocks** - Available if on major exchanges, but limited
5. **Mutual funds** - US-focused, international limited
6. **Preferred stocks** - Available but limited data vs common stock

## Best Use Cases by Coverage

| Use Case | Twelvedata Coverage | Rating |
|----------|---------------------|--------|
| **US Stock Trading** | Excellent | ⭐⭐⭐⭐⭐ |
| **International Stocks** | Good | ⭐⭐⭐⭐ |
| **Forex Trading/Analysis** | Excellent | ⭐⭐⭐⭐⭐ |
| **Crypto Spot Prices** | Excellent | ⭐⭐⭐⭐⭐ |
| **Crypto Derivatives** | None | ❌ |
| **Stock Fundamentals** | Excellent (US) | ⭐⭐⭐⭐⭐ |
| **ETF Analysis** | Excellent | ⭐⭐⭐⭐⭐ |
| **Technical Analysis** | Excellent (100+ indicators) | ⭐⭐⭐⭐⭐ |
| **Options Trading** | None | ❌ |
| **Futures Trading** | None | ❌ |
| **On-chain Analysis** | None | ❌ |
| **Economic Data** | None | ❌ |
| **News/Sentiment** | None | ❌ |

## Recommendation

**Twelvedata is best for:**
- Multi-asset spot price monitoring (stocks, forex, crypto)
- US stock fundamental analysis
- Technical analysis across all asset types
- Building charting/dashboard applications
- Historical data analysis
- Real-time price streaming (Pro+ plans)

**NOT suitable for:**
- Options trading platforms
- Futures trading
- Crypto derivatives analysis
- On-chain blockchain analytics
- Economic forecasting (no macro data)
- News-driven trading (no news feed)
