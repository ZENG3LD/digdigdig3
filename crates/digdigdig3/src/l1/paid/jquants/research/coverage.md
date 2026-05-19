# JQuants - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: No
- Europe: No
- Asia: **Yes (Japan only)**
- Other: No

### Country-Specific
- US: No
- UK: No
- Japan: **Yes (exclusive focus)**
- India: No
- China: No
- South Korea: No
- Hong Kong: No
- Singapore: No

**JQuants provides data exclusively for the Japanese stock market.**

### Restricted Regions
- Blocked countries: Not explicitly documented
- VPN detection: Not documented
- Geo-fencing: Not documented (likely available globally, Japan-focused data)

## Markets/Exchanges Covered

### Stock Markets

**Japan - Tokyo Stock Exchange (TSE) / Japan Exchange Group (JPX):**
- ✅ **Prime Market** (formerly TSE 1st Section)
- ✅ **Standard Market** (formerly TSE 2nd Section)
- ✅ **Growth Market** (formerly Mothers/JASDAQ)
- ❌ Tokyo Pro Market (not included)

**Japan - Osaka Exchange (OSE):**
- ✅ **Derivatives** (futures and options only)
  - TOPIX Futures
  - Nikkei 225 Futures
  - Index Options (TOPIX, Nikkei 225)

**Other Japanese Exchanges:**
- ❌ Nagoya Stock Exchange
- ❌ Fukuoka Stock Exchange
- ❌ Sapporo Stock Exchange

### Crypto Exchanges (if aggregator)
- Not applicable (stock market only)

### Forex Brokers (if aggregator)
- Not applicable (stock market only)

### Futures/Options Exchanges
- ✅ **Osaka Exchange (OSE)** - Equity derivatives
  - TOPIX Futures
  - Nikkei 225 Futures (regular, mini, micro)
  - TOPIX Index Options
  - Nikkei 225 Index Options
- ❌ CME (Nikkei futures traded in Chicago - not covered)
- ❌ SGX (Nikkei futures traded in Singapore - not covered)
- ❌ CBOE (not covered)

## Instrument Coverage

### Stocks
- **Total symbols**: ~3,900 (approximate, all TSE listed stocks)
  - Prime Market: ~1,800 companies
  - Standard Market: ~1,450 companies
  - Growth Market: ~500 companies
- **US stocks**: 0 (Japan only)
- **International**: 0 (no foreign listings data)
- **OTC**: No (only exchange-listed)
- **Penny stocks**: Not applicable (JPY currency)
- **REITs**: Not explicitly included (check listed/info endpoint)

### Stock Categories Covered
- **Large-cap**: TOPIX Core30, TOPIX Large70
- **Mid-cap**: TOPIX Mid400
- **Small-cap**: TOPIX Small
- **Growth**: Growth Market listings
- **All sectors**: 17-sector and 33-sector classifications

### Crypto
- Not applicable (stock market only)

### Forex
- Not applicable (stock market only)

### Commodities
- Not directly available
- Note: Some commodity-related stocks covered (e.g., trading companies, energy sector)

### Indices
- ✅ **TOPIX** (Tokyo Stock Price Index)
- ✅ **TOPIX Core30** (30 largest stocks)
- ✅ **TOPIX Large70** (next 70 large stocks)
- ✅ **TOPIX Mid400** (mid-cap)
- ✅ **TOPIX Small** (small-cap)
- ✅ **Growth Market 250 Index** (formerly Mothers Index)
- ✅ **Sector indices** (17 sectors)
- ❌ Nikkei 225 index values (only futures/options)
- ❌ JPX-Nikkei 400
- ❌ Custom indices

**Note**: Nikkei 225 is owned by Nikkei, Inc. and requires separate licensing for index values. JQuants provides Nikkei 225 **derivatives** but not the spot index.

## Data History

### Historical Depth by Tier

| Tier | Historical Depth | Start Date |
|------|------------------|------------|
| Free | 2 years (12-week delay) | ~2022 |
| Light | 5 years | ~2019 |
| Standard | 10 years | ~2014 |
| Premium | All available | May 7, 2008 onwards |

### Earliest Available Data

**Premium tier**:
- **Stock prices**: From **May 7, 2008** (most symbols)
  - Earlier for some long-listed stocks
  - Later for newly listed stocks (from IPO date)
- **Financial statements**: Varies by company (typically from listing date)
- **Indices**: From May 7, 2008
- **Derivatives**: From product launch date (varies)
- **Market structure data**: From May 7, 2008

### Granularity Available

| Granularity | Available | From When | Tier/Plan |
|-------------|-----------|-----------|-----------|
| **Tick data** | ✅ (Jan 2026) | TBD | Add-on plan |
| **1-minute bars** | ✅ (Jan 2026) | TBD | Add-on plan |
| **5-minute bars** | ❌ | - | Not available |
| **Hourly bars** | ❌ | - | Not available |
| **Daily bars** | ✅ | May 7, 2008 | All tiers |
| **Weekly/Monthly** | Calculated from daily | - | Calculate client-side |

**Note**: Minute and tick data added January 2026, historical depth TBD.

### Real-time vs Delayed

| Tier | Delay | Use Case |
|------|-------|----------|
| Free | **12 weeks** | Learning, historical research |
| Light | **None** (current) | Current analysis, backtesting |
| Standard | **None** (current) | Professional analysis |
| Premium | **None** (current) | Comprehensive analysis |

**Update timing**:
- Daily data: Available ~30 minutes after market close (16:30 JST update)
- Intraday: Not real-time streaming; must poll REST API
- Minute/tick: Historical data via REST API (not live stream)

**Snapshot vs Real-time**:
- **No real-time streaming**: All data via REST API polling
- **Daily snapshots**: Most data updated once per day
- **Intraday polling**: Minute bars available via add-on (poll every minute)

## Update Frequency

### Real-time Streams
- **Not available**: JQuants has no WebSocket/streaming
- All data accessed via REST API polling

### Daily Updates (Polling Schedule)

| Data Type | Update Time (JST) | Frequency | Delay from Event |
|-----------|-------------------|-----------|------------------|
| Daily stock prices | 16:30 | Daily | ~30 min after close |
| Morning session prices | 12:00 | Daily | ~30 min after AM close |
| Indices (TOPIX, etc.) | 16:30 | Daily | ~30 min after close |
| Futures/Options | 27:00 (3:00 AM) | Daily | ~3 hours after night session |
| Short selling data | 16:30 | Daily | Same day |
| Breakdown trading | 18:00 | Daily | Same day |
| Financial statements | 18:00 / 24:30 | Ad-hoc | Same day (prelim/final) |
| Dividend announcements | 12:00-19:00 | Ad-hoc | Intraday updates (hourly) |
| Earnings calendar | ~19:00 | Daily | Next business day forecast |

### Weekly Updates

| Data Type | Update Day/Time | Frequency | Delay |
|-----------|-----------------|-----------|-------|
| Trading by investor type | Thursday 18:00 | Weekly | 4 business days lag |
| Margin trading outstanding | Tuesday 16:30 | Weekly | 2 business days lag |

### Annual Updates

| Data Type | Update Timing | Frequency |
|-----------|---------------|-----------|
| Trading calendar | End of March | Annually | Following year's calendar |

### Scheduled Updates (Corporate Actions)
- **Earnings**: Quarterly (ad-hoc, company-specific)
- **Dividends**: Semi-annual or annual (ad-hoc, company-specific)
- **Financial statements**: Quarterly and annual (ad-hoc)

## Data Quality

### Accuracy
- **Source**: **Direct from Tokyo Stock Exchange** (Japan Exchange Group)
  - Not third-party aggregated
  - Official exchange data
  - Primary source quality
- **Validation**: Exchange-validated before publication
- **Corrections**: Automatic from exchange
  - Retroactive corrections published when errors detected
  - Historical data may be revised

### Completeness
- **Missing data**: Rare (only during exchange downtime)
- **Gaps**: Minimal
  - Expected gaps: Weekends, holidays (use trading calendar)
  - Unexpected gaps: Exchange system issues (rare)
- **Backfill**: Available
  - Historical data accessible via date range queries
  - Pagination for large result sets

### Timeliness

| Data Type | Latency | Target Update Time | Reliability |
|-----------|---------|-------------------|-------------|
| Daily prices | ~30 minutes | 16:30 JST | Very high |
| Morning prices | ~30 minutes | 12:00 JST | Very high |
| Indices | ~30 minutes | 16:30 JST | Very high |
| Futures/Options | ~3 hours | 03:00 JST | High |
| Financial statements | Same day | 18:00 / 24:30 JST | High |
| Trading by type | 4 business days | Thursday 18:00 | High |
| Margin trading | 2 business days | Tuesday 16:30 | High |

**Note**: Timing may vary; "The timing of updates may be changed without notice" per official docs.

### Market Hours Coverage

**Tokyo Stock Exchange (TSE)**:
- **Regular trading**:
  - Morning session: 9:00-11:30 JST
  - Afternoon session: 12:30-15:00 JST
  - Lunch break: 11:30-12:30 JST
- **After-hours**: Not available via JQuants

**Osaka Exchange (OSE) Derivatives**:
- **Day session**: 9:00-15:15 JST
- **Night session**: 16:30-03:00 (next day) JST
  - Night session data updated at 03:00 JST (27:00 in 24h+ notation)

**Coverage**:
- ✅ Regular trading hours fully covered
- ✅ Morning/afternoon sessions separate (Premium tier)
- ✅ Night session derivatives covered
- ❌ Pre-market trading: Not applicable (TSE has no pre-market)
- ❌ After-hours trading: Not applicable (TSE closes at 15:00)

## Symbol Coverage Details

### By Market Segment

| Market | Approx. Symbols | Coverage |
|--------|-----------------|----------|
| Prime Market | ~1,800 | ✅ Full |
| Standard Market | ~1,450 | ✅ Full |
| Growth Market | ~500 | ✅ Full |
| **Total TSE** | **~3,900** | **✅ Full** |

### By Sector (33-sector classification)

All 33 sectors covered, including:
- Fishery, Agriculture & Forestry
- Mining
- Construction
- Foods
- Textiles & Apparels
- Pulp & Paper
- Chemicals
- Pharmaceutical
- Oil & Coal Products
- Rubber Products
- Glass & Ceramics Products
- Iron & Steel
- Nonferrous Metals
- Metal Products
- Machinery
- Electric Appliances
- Transportation Equipment
- Precision Instruments
- Other Products
- Electric Power & Gas
- Land Transportation
- Marine Transportation
- Air Transportation
- Warehousing & Harbor Transportation Services
- Information & Communication
- Wholesale Trade
- Retail Trade
- Banks
- Securities & Commodity Futures
- Insurance
- Other Financing Business
- Real Estate
- Services

### By Size Category

| Category | Coverage |
|----------|----------|
| TOPIX Core30 | ✅ Full (30 stocks) |
| TOPIX Large70 | ✅ Full (70 stocks) |
| TOPIX Mid400 | ✅ Full (400 stocks) |
| TOPIX Small | ✅ Full (remaining ~1,500 stocks) |

### Excluded Instruments
- ❌ Foreign stocks listed on TSE (if any)
- ❌ Unlisted stocks (OTC, private)
- ❌ Tokyo Pro Market (professional investors only)
- ❌ Delisted stocks (no historical data after delisting)
- ❌ Warrants, structured products
- ❌ ETFs (unclear if included; check listed/info endpoint)

## Derivatives Coverage

### Futures Products
- ✅ TOPIX Futures (regular, mini)
- ✅ Nikkei 225 Futures (regular, mini, micro)
- ❌ Individual stock futures
- ❌ Bond futures (JGB)
- ❌ Commodity futures

### Options Products
- ✅ TOPIX Index Options
- ✅ Nikkei 225 Index Options
- ❌ Individual stock options
- ❌ Currency options

### Contract Months
- All available contract months for covered products
- Historical data from product launch

## Data Standards & Formats

### Dates & Times
- **Format**: YYYY-MM-DD for dates, HH:MM:SS for times
- **Timezone**: JST (Japan Standard Time, UTC+9)
- **No daylight saving**: Japan does not observe DST

### Currency
- **All prices**: JPY (Japanese Yen)
- **No currency conversion**: Raw JPY values only

### Number Formats
- **Prices**: Floating-point (e.g., 2530.50)
- **Volumes**: Integers (e.g., 12345678)
- **Monetary values**: Integers in JPY (e.g., 31234567890)

### Text Encoding
- **UTF-8**: All Japanese text
- **Language**: Company names, sectors, markets in Japanese
- **English**: Company names also provided in English (CompanyNameEnglish)

### Accounting Standards
- **Primary**: Japanese GAAP
- **Secondary**: IFRS, US GAAP (some companies)
  - Fields may be blank for non-GAAP reporters

## Comparison to Other Data Sources

| Feature | JQuants | Bloomberg | Reuters | Yahoo Finance |
|---------|---------|-----------|---------|---------------|
| **Japan stocks** | ✅ Official | ✅ Yes | ✅ Yes | ✅ Yes |
| **Data source** | ✅ Exchange direct | Third-party | Third-party | Third-party |
| **Real-time** | ❌ No | ✅ Yes (paid) | ✅ Yes (paid) | Delayed 20min |
| **Historical depth** | 2008+ (Premium) | Decades | Decades | Limited |
| **Free tier** | ✅ Yes (delayed) | ❌ No | ❌ No | ✅ Yes |
| **Pricing** | $0-110/mo | $$$$ | $$$$ | Free |
| **API quality** | Good | Excellent | Excellent | Fair |
| **Japan-specific data** | ✅ Excellent | Good | Good | Limited |

## Coverage Gaps & Limitations

### What's Missing
1. **Real-time streaming**: No WebSocket, must poll REST API
2. **Order book data**: No bid/ask depth
3. **Other Asian markets**: No coverage of China, Korea, Taiwan, etc.
4. **Global indices**: No S&P 500, FTSE, DAX, etc.
5. **Foreign stocks**: No ADRs, no cross-listings
6. **ETFs/REITs**: Unclear coverage (check endpoint)
7. **Pre-market/after-hours**: Not applicable (TSE structure)
8. **News/sentiment**: No news feed or sentiment scores
9. **Analyst estimates**: No consensus estimates, ratings
10. **Economic data**: No GDP, inflation, employment data

### Geographic Limitation
**Japan-only focus**: If you need multi-country coverage, JQuants must be combined with other providers:
- US: Polygon, Alpha Vantage, IEX Cloud
- Europe: Refinitiv, Bloomberg
- Asia-Pacific: Requires multiple providers

### Temporal Limitation
**Historical depth**: Premium tier starts May 2008
- For earlier data (e.g., 1980s-2000s), need alternative sources
- Bloomberg/Reuters have decades of history

### Frequency Limitation
**No high-frequency data**: Minute bars (via add-on) are the finest granularity
- For sub-second data, need exchange co-location or HFT vendor

## Recommendations for Coverage

### Ideal Use Cases
- ✅ **Japan equity research**: Perfect fit
- ✅ **Japanese market backtesting**: Excellent historical data
- ✅ **Fundamental analysis**: Comprehensive financial statements
- ✅ **Market structure studies**: Investor flows, margin data unique to Japan
- ✅ **End-of-day trading**: Daily updates sufficient
- ✅ **Academic research**: Official exchange data, affordable tiers

### Not Ideal For
- ❌ **Global portfolio**: Need multiple providers for other countries
- ❌ **Real-time trading**: No streaming, polling only
- ❌ **High-frequency trading**: Limited granularity, no microsecond data
- ❌ **News-driven trading**: No news/sentiment feed
- ❌ **Intraday scalping**: Daily updates too slow (even with minute bars)

### Complementary Services
Combine JQuants with:
- **News**: Bloomberg, Reuters, Nikkei newswire
- **Economic data**: FRED (Japan economic indicators)
- **Global markets**: Polygon (US), other regional providers
- **Sentiment**: Social media APIs, alternative data providers
- **Alternative data**: Satellite imagery, credit card data, etc.

## Coverage Verification

To verify exact symbol coverage:
1. Query `/v1/listed/info` endpoint (no code/date params)
2. Returns all currently listed stocks
3. Filter by MarketCode (Prime/Standard/Growth)
4. Count symbols per sector, market, size category

Current coverage (as of your query date) will be in the response.

## Historical Coverage Notes

### Survivorship Bias
- Delisted stocks: No longer appear in listed/info
- Historical prices: May still be accessible if you know the code
- Recommendation: Maintain historical symbol master to avoid survivorship bias

### Corporate Actions
- Stock splits: Handled via AdjustmentFactor
- Mergers/delistings: Symbol may become inactive
- Ticker changes: May result in new Code, check ISIN or other identifiers

### Data Revisions
- Financial statements: May be revised after initial disclosure
- Historical prices: Rarely revised (only for exchange errors)
- Adjustments: Retroactive when splits/dividends occur

## Summary

JQuants provides **comprehensive, official coverage of the Japanese stock market**:
- ✅ **All TSE listed stocks** (~3,900 symbols)
- ✅ **All market segments** (Prime, Standard, Growth)
- ✅ **All sectors** (33-sector classification)
- ✅ **Derivatives** (TOPIX/Nikkei futures and options)
- ✅ **Historical depth** up to May 2008 (Premium)
- ✅ **Fundamentals** (financial statements, dividends, earnings)
- ✅ **Market structure** (investor flows, margin, short selling)
- ❌ **Japan only** (no other countries)
- ❌ **No real-time streaming** (REST polling only)
- ❌ **No order book** (data-only, no trading)

**Geographic focus**: 100% Japan, 0% rest of world.

**Ideal for**: Japanese equity specialists, Asia-focused quants, academic researchers, Japan market structure analysts.

**Not suitable for**: Global multi-asset traders, real-time HFT, news-driven strategies (without additional data sources).
