# FRED - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: Yes (primary focus on United States)
- Europe: Yes (limited - major economies via international data)
- Asia: Yes (limited - major economies via international data)
- Other: Limited international coverage

### Country-Specific

**Primary Coverage:**
- US: Yes - **COMPREHENSIVE** (840,000+ series, 460,000+ regional series)

**International Coverage (Limited):**
- UK: Yes - GDP, inflation, interest rates, exchange rates
- Japan: Yes - GDP, inflation, interest rates, exchange rates
- India: Yes - Limited (exchange rates, some macro indicators)
- China: Yes - Limited (exchange rates, some macro indicators)
- Canada: Yes - GDP, interest rates, exchange rates
- Germany: Yes - GDP, inflation, interest rates
- France: Yes - GDP, inflation, interest rates
- Eurozone: Yes - GDP, inflation, ECB rates, exchange rates
- OECD Countries: Yes - Selected indicators via OECD source
- G7 Countries: Yes - Major indicators
- G20 Countries: Partial - Selected indicators

**Coverage Note**: International data is much less comprehensive than U.S. data. Focus is heavily on U.S. economic statistics.

### Restricted Regions

- Blocked countries: None - API is globally accessible
- VPN detection: No
- Geo-fencing: No
- Access restrictions: Based on use case (commercial use restricted), not geography

**Note**: FRED data can be accessed from anywhere, but Terms of Use apply globally.

## Markets/Exchanges Covered

### Stock Markets

**U.S. Markets:**
- NYSE: Yes (indices like S&P 500, Dow Jones)
- NASDAQ: Yes (NASDAQ Composite index)
- AMEX: Indirectly (part of broader indices)

**International Markets:**
- UK: LSE (FTSE 100 index)
- Japan: TSE (Nikkei 225)
- Germany: DAX index
- France: CAC 40
- Other: Limited coverage via international indices

**Note**: FRED provides **index-level data**, not individual stock prices or company fundamentals.

### Crypto Exchanges (if aggregator)

- Not applicable - FRED does not provide cryptocurrency data

### Forex Brokers (if aggregator)

- Not applicable - FRED provides official exchange rates, not broker data

### Futures/Options Exchanges

**Limited Coverage:**
- CME: No direct coverage (but commodity prices available)
- CBOE: VIX index available
- ICE: Some commodity prices
- NYMEX: Oil prices (WTI)

**Note**: FRED provides spot commodity prices and some derivatives indices, but not comprehensive futures/options data.

## Instrument Coverage

### Stocks

- Total symbols: N/A - FRED doesn't provide individual stock data
- U.S. stocks: Indices only (S&P 500, Dow Jones, NASDAQ, Wilshire 5000, Russell 2000)
- International: Major indices only (FTSE, Nikkei, DAX, CAC 40)
- OTC: No
- Penny stocks: No

**Stock Market Coverage**: Index-level only, no individual equities.

### Crypto

- Total coins: 0
- Spot pairs: 0
- Futures: 0
- Perpetuals: 0

**Crypto Coverage**: None - FRED does not cover cryptocurrencies.

### Forex

- Currency pairs: 170+ exchange rate series
- Majors: Yes (EUR/USD, GBP/USD, JPY/USD, CHF/USD, CAD/USD, AUD/USD, NZD/USD)
- Minors: Yes (EUR/GBP, EUR/JPY, GBP/JPY, etc.)
- Exotics: Limited (some emerging market currencies vs USD)

**Exchange Rate Sources:**
- Board of Governors of the Federal Reserve System
- Bank of England
- European Central Bank
- Bank for International Settlements

**Coverage Details:**
- Historical depth: Many series from 1970s-1980s
- Update frequency: Daily (business days)
- Format: Typically expressed as foreign currency per USD

**Popular FX Series:**
- DEXUSEU - EUR/USD
- DEXUSUK - GBP/USD
- DEXJPUS - JPY/USD
- DEXCHUS - CNY/USD
- DEXCAUS - CAD/USD

### Commodities

**Energy:**
- Oil: WTI Crude (DCOILWTICO), Brent Crude (DCOILBRENTEU)
- Natural Gas: Henry Hub prices
- Gasoline: Retail and wholesale prices
- Heating Oil: Yes
- Coal: Yes
- Electricity: Regional prices

**Metals:**
- Gold: GOLDAMGBD228NLBM (London AM fixing)
- Silver: SLVPRUSD
- Copper: PCOPPUSDM
- Platinum: Yes
- Palladium: Yes
- Aluminum: Yes
- Iron Ore: Yes

**Agriculture:**
- Corn: PCOALLUSDM
- Wheat: PWHEAMTUSDM
- Soybeans: PSOYBUSDM
- Coffee: PCOFFOTMUSDM
- Sugar: Yes
- Cotton: Yes
- Livestock: Cattle, hogs

**Other Commodities:**
- Lumber: Yes
- Rubber: Yes
- Various industrial commodities

**Coverage**: Spot prices primarily, some futures-based indices.

### Indices

**U.S. Stock Indices:**
- S&P 500: SP500
- Dow Jones Industrial Average: DJIA
- NASDAQ Composite: NASDAQCOM
- Wilshire 5000: WILL5000IND
- Russell 2000: RU2000PR
- VIX (Volatility): VIXCLS

**International Stock Indices:**
- FTSE 100 (UK)
- Nikkei 225 (Japan)
- DAX (Germany)
- CAC 40 (France)
- Hang Seng (Hong Kong)

**Economic Indices:**
- ISM Manufacturing PMI
- ISM Services PMI
- Consumer Confidence Index (University of Michigan, Conference Board)
- Leading Economic Indicators (LEI)
- Financial Stress Indices (multiple variants)

**Price Indices:**
- CPI (320+ series)
- PPI (10,000+ series)
- PCE Price Index
- GDP Deflator
- Import/Export Price Indices
- House Price Indices (Case-Shiller, FHFA, Zillow)

**Crypto:**
- BTC Dominance: No
- DeFi Index: No
- Crypto coverage: None

## Data History

### Historical Depth

**U.S. Economic Data:**
- Stocks: S&P 500 from 1927, Dow Jones from 1896
- GDP: From 1947 (quarterly), 1929 (annual)
- Inflation (CPI): From 1913
- Employment: Various series from 1930s-1940s
- Interest rates: Federal funds from 1954, some rates from earlier
- Population: Some series from 1700s
- Banking: Some series from early 1900s

**International Data:**
- Generally from 1960s-1990s depending on source
- Less historical depth than U.S. data

**Regional U.S. Data:**
- State-level: Generally from 1970s-1990s
- MSA-level: Mostly from 1990s-2000s
- County-level: Very limited, recent data

### Granularity Available

- Tick data: No
- 1-minute bars: No
- 5-minute bars: No
- Hourly: No
- Daily: Yes (financial markets, some rates, selected indicators)
- Weekly: Yes (initial jobless claims, some surveys)
- Biweekly: Yes (limited series)
- Monthly: Yes (most economic indicators) - **PRIMARY FREQUENCY**
- Quarterly: Yes (GDP, many national accounts)
- Semiannual: Yes (limited series)
- Annual: Yes (demographics, some international data)

**Depth by Frequency:**
- Daily: Financial market data from 1920s-1960s depending on series
- Monthly: Most series from 1940s-1950s, some from earlier
- Quarterly: GDP from 1947, many series from 1960s
- Annual: Some demographic series from 1700s-1800s

### Real-time vs Delayed

- Real-time: Yes - data appears shortly after official government release times
- Delayed: No intentional delay beyond official release schedules
- Snapshot: Historical snapshots via ALFRED (real-time periods)

**Update Timing:**
- Economic releases follow published calendars (e.g., Employment Situation first Friday of month)
- FRED typically updates within minutes to hours of official release
- No intraday updates (data released once per day for daily series)
- Financial market data (stocks, rates) updated after market close

**ALFRED (Archival FRED):**
- Preserves all historical revisions of data
- Can access data "as it was known" at any point in history
- Critical for research on data revisions and real-time forecasting

## Update Frequency

### Real-time Streams

- Price updates: N/A - No real-time streaming (REST API only)
- Orderbook: N/A
- Trades: N/A

**Note**: FRED is not a real-time data stream. Updates occur on scheduled release times.

### Scheduled Updates

**Daily Series:**
- Financial markets: Updated daily after market close
- Interest rates: Updated daily (business days)
- Exchange rates: Updated daily (business days)

**Weekly Series:**
- Initial jobless claims: Every Thursday morning
- Money supply: Every Thursday
- Various surveys: Weekly schedules

**Monthly Series:**
- Employment: First Friday of month (typically 8:30 AM ET)
- CPI: Mid-month (typically 8:30 AM ET)
- Retail sales: Mid-month
- Industrial production: Mid-month
- Housing starts: Mid-month
- PCE: End of month

**Quarterly Series:**
- GDP: ~1 month after quarter end (advance estimate)
- GDP revisions: Second and third estimates in following months
- Financial accounts: ~2.5 months after quarter end

**Annual Series:**
- Demographics: Irregular schedules
- Some international data: Annual releases

**Release Calendar:**
- Each series has documented release schedule
- Can access release dates via /fred/releases/dates endpoint
- Revisions common for many economic indicators

### Fundamentals

- Quarterly: GDP, corporate profits, some national accounts
- Annual: Demographics, some international data, historical revisions

### Economic data

- Daily: Financial markets, rates
- Weekly: Claims, surveys
- Monthly: Most indicators (employment, prices, production, sales)
- Quarterly: GDP, national accounts
- Annual: Demographics, long-term series

### News

- Real-time: Not applicable (FRED provides data, not news)

## Data Quality

### Accuracy

- Source: Official government agencies, central banks, international organizations
- Validation: Data validated by source agencies
- Corrections: Revisions published by source agencies, preserved in ALFRED

**Data Sources (118 total):**
- U.S. Bureau of Economic Analysis (BEA)
- U.S. Bureau of Labor Statistics (BLS)
- U.S. Census Bureau
- Board of Governors of the Federal Reserve System
- Federal Reserve Banks (12 regional banks)
- U.S. Treasury
- OECD
- World Bank
- IMF
- European Central Bank
- Bank of England
- And many more official sources

**Reliability**: Very high - data from official government and institutional sources.

### Completeness

- Missing data: Rare for current data; more common for historical data
- Gaps: Clearly indicated with "." value
- Backfill: Not available through API (FRED provides data as released by sources)

**Data Gaps:**
- Discontinued series clearly marked
- Seasonal data may have gaps (e.g., quarterly series)
- Some historical series have missing observations

### Timeliness

- Latency: Minutes to hours after official release (not instantaneous)
- Delay: Depends on source agency release schedule
- Market hours: N/A (economic data released on fixed schedules, not intraday)

**Release Timing Examples:**
- Employment report: 8:30 AM ET, first Friday of month
- CPI: 8:30 AM ET, mid-month
- GDP: 8:30 AM ET, ~1 month after quarter end
- FRED typically reflects new data within 15 minutes to 2 hours of release

### Revisions

**Common Revision Patterns:**
- GDP: Advance → Second → Third estimates (monthly revisions for 3 months)
- Employment: Revised for 2 prior months each release
- Retail sales: Revised for 1-2 prior months
- Many monthly indicators: Subject to revisions

**ALFRED Vintage Data:**
- Preserves all historical versions
- Can track revision history
- Access via realtime_start/realtime_end parameters or /series/vintagedates

## Geographic Granularity

### National (U.S.)
- Comprehensive coverage
- 380,000+ national series

### State Level
- 50 states + DC
- Employment, unemployment, GDP, income, population
- 100,000+ state-level series

### Metropolitan Statistical Area (MSA)
- 380+ MSAs covered
- Employment, unemployment, house prices, income
- 300,000+ MSA-level series

### County Level
- Very limited coverage
- Primarily population and selected indicators
- ~60,000+ county-level series

### International
- Country-level only
- No regional breakdowns for foreign countries

## Comparison to Other Data Providers

| Feature | FRED | Bloomberg | Refinitiv | Trading Economics |
|---------|------|-----------|-----------|-------------------|
| Cost | Free | $$$$$ | $$$$$ | $-$$$ |
| U.S. Economic Data | Excellent | Excellent | Excellent | Good |
| International Data | Limited | Excellent | Excellent | Excellent |
| Historical Depth | Excellent | Excellent | Excellent | Good |
| Real-time Updates | Minutes-Hours | Seconds | Seconds | Minutes |
| Individual Stocks | No | Yes | Yes | No |
| Company Fundamentals | No | Yes | Yes | Limited |
| Crypto | No | Yes | Yes | Yes |
| API Quality | Good | Excellent | Excellent | Good |
| Commercial Use | Restricted | Yes | Yes | Yes |

**FRED's Niche**: Best free source for U.S. economic data with excellent historical depth and official government sources. Not suitable for real-time trading, individual securities, or comprehensive international coverage.

## Unique Coverage Strengths

1. **U.S. Regional Data**: Unmatched free coverage of state/MSA economic indicators
2. **Historical Depth**: Some series dating back centuries
3. **Revision History**: ALFRED provides complete vintage data
4. **Official Sources**: Direct from government agencies
5. **Economic Research**: Optimized for academic and policy research
6. **Free Access**: No cost barrier for educational/non-commercial use

## Coverage Limitations

1. **No Individual Securities**: Index-level only for stocks
2. **No Real-time Streaming**: Updates on release schedules only
3. **Limited International**: Focus heavily on U.S. data
4. **No Alternative Data**: No social sentiment, satellite imagery, etc.
5. **No Crypto**: Zero cryptocurrency coverage
6. **No High-Frequency**: No intraday or minute-level data
