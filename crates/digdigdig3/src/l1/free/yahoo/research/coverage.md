# yahoo - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: Yes (US, Canada, Mexico)
- Europe: Yes (extensive coverage - 30+ countries)
- Asia: Yes (comprehensive coverage - 20+ countries)
- Oceania: Yes (Australia, New Zealand)
- Middle East: Yes (limited - Saudi Arabia, UAE, Israel)
- Africa: Yes (very limited - South Africa)
- South America: Yes (Brazil, Argentina, Chile)

### Country-Specific Coverage

**Americas:**
- US: Yes (comprehensive - NYSE, Nasdaq, AMEX, OTC)
- Canada: Yes (TSX, TSXV)
- Mexico: Yes (BMV)
- Brazil: Yes (B3)
- Argentina: Yes (MERV)
- Chile: Yes (BCS)

**Europe:**
- UK: Yes (LSE, AIM)
- Germany: Yes (XETRA, Frankfurt)
- France: Yes (Euronext Paris)
- Italy: Yes (MIL)
- Spain: Yes (BME)
- Netherlands: Yes (AMS)
- Switzerland: Yes (SWX)
- Sweden: Yes (STO)
- Norway: Yes (OSE)
- Denmark: Yes (CPH)
- Finland: Yes (HEL)
- Belgium: Yes (EBR)
- Austria: Yes (VIE)
- Ireland: Yes (ISE)
- Portugal: Yes (ELI)
- Greece: Yes (ATH)
- Poland: Yes (WAR)
- Czech Republic: Yes (PRA)
- Russia: Yes (MCX) *Note: May be restricted*
- Turkey: Yes (IST)

**Asia:**
- Japan: Yes (JPX, TSE)
- China: Yes (SSE, SZSE - limited access)
- Hong Kong: Yes (HKEX)
- India: Yes (NSE, BSE)
- South Korea: Yes (KRX, KSE)
- Taiwan: Yes (TWSE, TPEx)
- Singapore: Yes (SGX)
- Malaysia: Yes (MYX)
- Thailand: Yes (SET)
- Indonesia: Yes (IDX)
- Philippines: Yes (PSE)
- Vietnam: Yes (limited)
- Israel: Yes (TASE)
- Saudi Arabia: Yes (Tadawul)
- UAE: Yes (DFM, ADX)

**Oceania:**
- Australia: Yes (ASX)
- New Zealand: Yes (NZX)

**Africa:**
- South Africa: Yes (JSE)

### Restricted Regions
- Blocked countries: None explicitly blocked (API access worldwide)
- VPN detection: No (direct API access doesn't detect VPN)
- Geo-fencing: No geo-fencing on API endpoints
- **Note:** Some symbols may be unavailable in certain regions due to exchange data agreements

## Markets/Exchanges Covered

### Stock Markets (100+ Exchanges)

**United States:**
- NYSE (New York Stock Exchange): Yes
- NASDAQ: Yes
- AMEX (American Stock Exchange): Yes
- OTC Markets (Pink Sheets, OTCQB, OTCQX): Yes
- BATS: Yes (now part of Cboe)

**Canada:**
- TSX (Toronto Stock Exchange): Yes
- TSXV (TSX Venture Exchange): Yes
- CSE (Canadian Securities Exchange): Yes

**Europe:**
- LSE (London Stock Exchange): Yes
- AIM (Alternative Investment Market): Yes
- XETRA (Deutsche Börse): Yes
- Euronext (Paris, Amsterdam, Brussels, Lisbon): Yes
- Borsa Italiana: Yes
- BME (Bolsa de Madrid): Yes
- SIX Swiss Exchange: Yes
- OMX Nordic (Stockholm, Helsinki, Copenhagen): Yes
- Oslo Børs: Yes
- Warsaw Stock Exchange: Yes
- Prague Stock Exchange: Yes
- Moscow Exchange: Yes
- Istanbul Stock Exchange: Yes

**Asia:**
- Tokyo Stock Exchange (JPX): Yes
- Hong Kong Stock Exchange (HKEX): Yes
- Shanghai Stock Exchange (SSE): Yes
- Shenzhen Stock Exchange (SZSE): Yes
- National Stock Exchange of India (NSE): Yes
- Bombay Stock Exchange (BSE): Yes
- Korea Exchange (KRX): Yes
- Taiwan Stock Exchange (TWSE): Yes
- Singapore Exchange (SGX): Yes
- Bursa Malaysia: Yes
- Stock Exchange of Thailand: Yes
- Indonesia Stock Exchange: Yes
- Philippine Stock Exchange: Yes
- Tel Aviv Stock Exchange: Yes
- Saudi Stock Exchange (Tadawul): Yes

**Oceania:**
- Australian Securities Exchange (ASX): Yes
- New Zealand Exchange (NZX): Yes

**Americas (other):**
- B3 (Brazil): Yes
- Bolsa Mexicana de Valores: Yes
- Buenos Aires Stock Exchange: Yes
- Santiago Stock Exchange: Yes

**Africa:**
- Johannesburg Stock Exchange (JSE): Yes

### Crypto Exchanges (Data Aggregated)

Yahoo Finance aggregates crypto prices but doesn't specify individual exchange sources. Coverage includes:
- Binance: Yes (implied - major source)
- Coinbase: Yes (implied)
- Kraken: Yes (implied)
- Bitstamp: Yes (implied)
- Other major exchanges: Yes (aggregated)

**Note:** Yahoo provides aggregated crypto prices, not exchange-specific data. Individual exchange data not available.

### Forex Brokers (Data Aggregated)

Forex data is aggregated from major market data providers, not specific brokers.
- Data source: Institutional forex data feeds
- Coverage: All major, minor, and exotic currency pairs
- Individual broker data: Not available

### Futures/Options Exchanges

**Futures:**
- CME Group (Chicago Mercantile Exchange): Yes
- CBOT (Chicago Board of Trade): Yes
- NYMEX (New York Mercantile Exchange): Yes
- COMEX: Yes
- ICE Futures: Yes (limited)

**Options:**
- CBOE (Chicago Board Options Exchange): Yes
- US equity options (all exchanges): Yes
- International options: Limited

## Instrument Coverage

### Stocks
- Total symbols: ~50,000+
- US stocks: ~8,000 (NYSE, Nasdaq, AMEX, OTC)
- International stocks: ~42,000+ (100+ global exchanges)
- OTC stocks: Yes (~5,000+ pink sheets, OTCQB, OTCQX)
- Penny stocks: Yes (if traded on supported exchanges)
- ADRs (American Depositary Receipts): Yes
- Preferred stocks: Yes

**Symbol Format Examples:**
- US: AAPL, MSFT, GOOGL
- UK: BARC.L, BP.L, HSBA.L
- Germany: SAP.DE, VOW3.DE
- Japan: 7203.T (Toyota), 9984.T (SoftBank)
- Hong Kong: 0700.HK (Tencent), 9988.HK (Alibaba)
- India: RELIANCE.NS, TCS.BO

### Crypto
- Total coins: ~500+
- Major coins (top 100 by market cap): Yes (nearly all)
- Mid-cap coins: Yes (selective)
- Small-cap coins: Limited
- Stablecoins: Yes (USDT, USDC, BUSD, DAI, etc.)
- DeFi tokens: Yes (major ones)
- Meme coins: Yes (DOGE, SHIB, etc.)
- Spot pairs: ~500+ (primarily USD, USDT pairs)
- Futures: No (crypto derivatives not available)
- Perpetuals: No

**Symbol Format:** SYMBOL-USD (e.g., BTC-USD, ETH-USD)

**Coverage:**
- Bitcoin (BTC): Yes
- Ethereum (ETH): Yes
- Ripple (XRP): Yes
- Cardano (ADA): Yes
- Solana (SOL): Yes
- Dogecoin (DOGE): Yes
- Polygon (MATIC): Yes
- And 500+ more

### Forex
- Currency pairs: ~100+
- Majors (7 pairs): Yes (EUR/USD, USD/JPY, GBP/USD, USD/CHF, USD/CAD, AUD/USD, NZD/USD)
- Minors (~20 pairs): Yes (EUR/GBP, EUR/JPY, GBP/JPY, etc.)
- Exotics (~70+ pairs): Yes (USD/TRY, USD/ZAR, USD/MXN, etc.)
- Cryptocurrency pairs: No (use crypto symbols instead)

**Symbol Format:** PAIR=X (e.g., EURUSD=X, GBPUSD=X)

### Commodities
- Metals: Gold (GC=F), Silver (SI=F), Copper (HG=F), Platinum (PL=F), Palladium (PA=F)
- Energy: Crude Oil (CL=F), Brent Crude (BZ=F), Natural Gas (NG=F), Gasoline (RB=F), Heating Oil (HO=F)
- Agriculture: Corn (ZC=F), Wheat (ZW=F), Soybeans (ZS=F), Coffee (KC=F), Sugar (SB=F), Cotton (CT=F), Cocoa (CC=F)
- Livestock: Live Cattle (LE=F), Lean Hogs (HE=F), Feeder Cattle (GF=F)

**Symbol Format:** SYMBOL=F (futures contracts)

**Note:** Only futures contracts available, not spot commodity prices.

### Indices
- US indices: ~50+ (S&P 500, Nasdaq, Dow, Russell 2000, sector indices, etc.)
- International indices: ~200+ (FTSE, DAX, CAC, Nikkei, Hang Seng, etc.)
- Crypto indices: Limited (Bitcoin Dominance Index, etc.)
- Bond indices: Limited
- Commodity indices: Limited

**Symbol Format:** ^SYMBOL (e.g., ^GSPC, ^DJI, ^IXIC)

**Major Indices:**
- ^GSPC (S&P 500): Yes
- ^DJI (Dow Jones): Yes
- ^IXIC (Nasdaq Composite): Yes
- ^RUT (Russell 2000): Yes
- ^FTSE (FTSE 100): Yes
- ^GDAXI (DAX): Yes
- ^FCHI (CAC 40): Yes
- ^N225 (Nikkei 225): Yes
- ^HSI (Hang Seng): Yes
- ^BSESN (BSE Sensex): Yes
- ^VIX (CBOE Volatility Index): Yes

### ETFs & Mutual Funds
- US ETFs: ~3,000+
- International ETFs: ~1,000+
- US Mutual Funds: ~20,000+
- International Mutual Funds: Limited

**Coverage Types:**
- Equity ETFs: Yes (broad market, sector, thematic)
- Bond ETFs: Yes (government, corporate, high yield)
- Commodity ETFs: Yes (gold, oil, agriculture)
- Currency ETFs: Yes
- Leveraged ETFs: Yes (2x, 3x)
- Inverse ETFs: Yes (-1x, -2x, -3x)
- Actively Managed Funds: Yes

### Options
- US equity options: Yes (all optionable stocks)
- ETF options: Yes
- Index options: Yes (SPX, NDX, RUT, etc.)
- International options: No
- Commodity options: No (futures only)
- Forex options: No

**Coverage:**
- Total optionable symbols: ~5,000+
- Expirations: All available expirations (weekly, monthly, quarterly, LEAPS)
- Strikes: All strikes (ITM, ATM, OTM)
- Greeks: Yes (delta, gamma, theta, vega, rho)

### Bonds
- US Treasuries: Yes (yields via ^TNX, ^IRX, ^TYX symbols)
- Corporate Bonds: Limited (some ETFs)
- Municipal Bonds: No
- International Government Bonds: Limited
- Bond Funds: Yes (bond ETFs and mutual funds)

**Treasury Symbols:**
- ^IRX (13-week T-Bill): Yes
- ^FVX (5-year Treasury): Yes
- ^TNX (10-year Treasury): Yes
- ^TYX (30-year Treasury): Yes

## Data History

### Historical Depth

**Stocks:**
- US major stocks: From IPO date (often 30-50+ years)
  - Example: AAPL from 1980 (40+ years)
  - Example: MSFT from 1986 (35+ years)
- US small-cap: Varies (5-20 years typically)
- International stocks: Varies by exchange (10-30 years)
- Delisted stocks: Limited (data often removed)

**Crypto:**
- Bitcoin (BTC-USD): From ~2014 (7+ years on Yahoo Finance)
- Ethereum (ETH-USD): From ~2017 (5+ years)
- Newer coins: From listing date (1-5 years)
- **Note:** Blockchain inception dates may predate Yahoo Finance coverage

**Forex:**
- Major pairs: 20+ years
- Minor pairs: 10-20 years
- Exotic pairs: 5-10 years

**Commodities (Futures):**
- Gold, Oil, Wheat: 20+ years
- Newer contracts: From contract inception

**Indices:**
- S&P 500 (^GSPC): From 1927 (90+ years!)
- Dow Jones (^DJI): From 1896 (125+ years!)
- Nasdaq (^IXIC): From 1971 (50+ years)
- International: Varies (10-50 years)

**ETFs:**
- SPY (oldest ETF): From 1993 (30+ years)
- Newer ETFs: From launch date

**Mutual Funds:**
- Varies by fund (typically 10-40 years for major funds)

### Granularity Available

**Intraday:**
- Tick data: No (not available)
- 1-minute bars: Yes (last 7 days ONLY)
- 2-minute bars: Yes (last 60 days)
- 5-minute bars: Yes (last 60 days)
- 15-minute bars: Yes (last 60 days)
- 30-minute bars: Yes (last 60 days)
- 60-minute bars: Yes (last 730 days / 2 years)
- 90-minute bars: Yes (last 60 days)

**Daily & Higher:**
- Daily: Yes (full historical depth)
- Weekly: Yes (full historical depth)
- Monthly: Yes (full historical depth)
- Quarterly: Yes (calculated from monthly)

**CRITICAL LIMITATIONS:**
- 1m data: Only 7 days (NOT suitable for long-term backtesting)
- Intraday (<1d): Only 60 days for most intervals
- 1h data: 730 days maximum

### Real-time vs Delayed

**Real-time (15-20 second delay):**
- US stocks: Yes (exchange fee waived for retail data)
- Canadian stocks: Yes
- Most international: Yes
- Crypto: Yes (~15-20s delay from exchanges)
- Forex: Yes

**Delayed (15-minute delay):**
- Some exchanges require paid subscriptions for real-time
- Free users get 15-minute delayed data for these:
  - Some international exchanges
  - Specific exchange agreements

**Snapshot (End-of-Day):**
- Mutual funds: NAV updated once daily (after market close)
- Some bonds: Daily updates only

**Free Tier Delay:**
- Most data: 15-20 seconds (acceptable for retail)
- No paid tier from Yahoo for truly real-time data
- For sub-second data, use direct exchange feeds

## Update Frequency

### Real-time Streams (WebSocket)

**Price updates:**
- Frequency: Real-time push (as trades occur)
- Latency: ~15-20 seconds from exchange
- During market hours: Continuous updates
- Outside market hours: Pre/post market updates (if applicable)

**Orderbook:**
- Not available (only best bid/ask via quote endpoint)

**Trades:**
- Not available individually (aggregated in price/volume)

### REST API Updates

**Quote endpoint (/v7/finance/quote):**
- Update frequency: Every 15-20 seconds during market hours
- Cache: May be cached for ~15s on Yahoo's servers
- Rate limit: ~2000/hour (poll max every 1-2 seconds)

**Chart endpoint (/v8/finance/chart):**
- Update frequency: Same as quote (15-20s)
- Historical data: Static (only updates with new bars)

### Scheduled Updates

**Fundamentals:**
- Financial statements: Quarterly (after earnings release)
- Annual reports: Yearly
- Update lag: 1-2 days after company filing
- Historical: Retroactive updates if corrections made

**Economic data:**
- Treasury yields: Daily (continuous during trading hours)
- Limited economic indicators (via index symbols)

**News:**
- Update frequency: Real-time (as published)
- Lag: ~1-5 minutes from source publication

**Analyst ratings:**
- Updates: As analysts publish (sporadic)
- Aggregation lag: ~1 day

**Earnings dates:**
- Updates: As companies announce (usually quarterly)
- Calendar updates: Weekly

**Dividends:**
- Updates: As declared (ex-date typically announced ~1 month prior)

**Insider trading:**
- Updates: After SEC Form 4 filing (within 2 business days of transaction)
- Lag: ~1-3 days from actual transaction

## Data Quality

### Accuracy
- Source: Aggregated from exchanges, data vendors, and SEC filings
- Validation: Basic validation (price sanity checks)
- Corrections: Delayed (errors may persist for hours/days)
- **Trust Level:**
  - Large-cap US stocks: Very High (99.9%+ accurate)
  - Small-cap US stocks: High (99%+ accurate)
  - International stocks: Medium-High (95-99% accurate, more gaps)
  - Crypto: Medium (95-98% accurate, occasional spikes/errors)
  - Fundamentals: High (sourced from SEC, but manual entry errors possible)

### Completeness
- Missing data: Common for:
  - Small-cap/micro-cap stocks
  - International stocks (especially emerging markets)
  - Delisted/bankrupt companies (data removed)
  - Newly listed stocks (first few days may be incomplete)
- Gaps: Historical gaps possible for:
  - Trading halts (data may be missing during halt)
  - System outages (rare, but can cause gaps)
  - Delisted symbols (historical data may be removed)
- Backfill: Generally good for major symbols
- **Completeness:**
  - Large-cap US: 99%+ complete
  - Small-cap US: 95%+ complete
  - International: 85-95% complete (varies by exchange)
  - Crypto: 90-95% complete (newer coins may have gaps)

### Timeliness
- Latency: ~15-20 seconds for "real-time" quotes
  - From exchange → Yahoo servers: ~10s
  - From Yahoo servers → API response: ~5-10s
- Delay: Some exchanges 15-minute delayed (exchange data agreement)
- Market hours: Data available during and after trading hours
- **Freshness:**
  - Real-time quotes: Good (15-20s acceptable for retail traders)
  - Pre/post market: Good (updates during extended hours)
  - Fundamentals: Delayed 1-2 days (after company filing)
  - News: Delayed 1-5 minutes (from source)

**Not Suitable For:**
- High-frequency trading (HFT)
- Sub-second strategies
- Arbitrage trading
- Market making

**Suitable For:**
- Retail day trading (with 15-20s delay tolerance)
- Swing trading
- Long-term investing
- Fundamental analysis
- Backtesting (with intraday limitations)

## Coverage Gaps & Limitations

### What Yahoo Finance Does NOT Cover Well:

1. **Micro-cap stocks**: Coverage exists but data quality lower
2. **OTC Pink Sheets**: Limited data (basic quotes only, no fundamentals)
3. **Emerging market stocks**: Spotty coverage, frequent gaps
4. **Delisted stocks**: Historical data often removed
5. **Pre-IPO companies**: No coverage (obviously)
6. **Private companies**: No coverage
7. **Unlisted securities**: No coverage
8. **Exotic derivatives**: No coverage (only US equity options)
9. **Municipal bonds**: No coverage
10. **Real estate (direct)**: No coverage (only REITs)
11. **Physical commodities spot prices**: Only futures available
12. **Crypto derivatives**: No perpetuals, futures, options
13. **DeFi protocols**: No direct coverage (only tokens if traded)
14. **NFTs**: No coverage
15. **Economic calendar**: No structured calendar of releases

## Regional Restrictions & Availability

### Data Access by Region

**North America:**
- US: Full access (all markets, full data)
- Canada: Full access
- Mexico: Full access

**Europe:**
- All countries: Full access (no IP geo-blocking)
- Data quality varies by exchange

**Asia:**
- All countries: Full access
- China: Access may be restricted domestically (Great Firewall)
- Data quality varies (Japan/HK excellent, others variable)

**Other:**
- Australia/NZ: Full access
- Middle East: Full access
- Africa: Full access
- South America: Full access

**API Access:**
- No VPN required (access from any country)
- No geo-fencing on API endpoints
- RapidAPI available globally

**Note:** While API access is global, some symbol data may be restricted based on exchange data agreements (not user location).

## Summary Table: Coverage by Asset Class

| Asset Class | Symbols | Exchanges | Historical | Intraday | Fundamentals | Quality |
|-------------|---------|-----------|------------|----------|--------------|---------|
| US Stocks | ~8,000 | 4 | Decades | 60d | Excellent | Very High |
| Intl Stocks | ~42,000 | 100+ | 10-30y | 60d | Limited | Medium-High |
| Crypto | ~500+ | Aggregated | 2-7y | 60d | No | Medium |
| Forex | ~100+ | Aggregated | 20y+ | 60d | N/A | High |
| Commodities | ~50+ | CME, etc. | 20y+ | 60d | No | High |
| Indices | ~250+ | Global | Decades | 60d | N/A | Very High |
| ETFs | ~4,000 | US+Intl | Since launch | 60d | Holdings | High |
| Mutual Funds | ~20,000 | US | 10-40y | EOD only | Holdings | High |
| Options | ~5,000 base | US | Current | Real-time | Greeks | High |
| Bonds | Limited | Treasury yields | Decades | Daily | No | Medium |

## Best Use Cases by Coverage

**Excellent Coverage (Use Yahoo Finance):**
- US stock analysis (large/mid cap)
- Global index tracking
- Multi-asset portfolio tracking (stocks, crypto, forex, commodities)
- Long-term backtesting (daily data)
- Fundamental analysis (US stocks)
- Options trading (US equity options)
- ETF/mutual fund research

**Good Coverage (Yahoo Finance acceptable):**
- International stock tracking (major markets)
- Cryptocurrency price tracking
- Forex trading (major/minor pairs)
- Commodity futures tracking
- Short-term intraday analysis (60-day window)

**Poor Coverage (Use alternative sources):**
- High-frequency trading (need direct exchange feeds)
- Crypto derivatives (use Binance, Bybit, etc.)
- On-chain analysis (use Etherscan, blockchain explorers)
- Comprehensive economic calendar (use Trading Economics, FRED)
- Micro-cap/OTC detailed analysis (use OTC Markets Group)
- Municipal bonds (use EMMA, Bloomberg)
- Real-time tick data (use IEX, direct exchange feeds)
- Historical intraday >60 days (use paid providers like Polygon, Alpha Vantage)
