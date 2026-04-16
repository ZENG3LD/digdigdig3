# KRX - Data Coverage

## Geographic Coverage

### Regions Supported

- North America: No
- Europe: No
- Asia: Yes (South Korea only)
- Other: No

**KRX is exclusively a South Korean stock exchange.**

### Country-Specific

- **South Korea: Yes** (primary and only market)
- US: No
- UK: No
- Japan: No
- China: No
- India: No
- Other Asian countries: No

### Restricted Regions

- Blocked countries: None explicitly documented
- VPN detection: Not documented (likely minimal enforcement)
- Geo-fencing: No (API accessible globally)
- Compliance: Subject to South Korean financial regulations
- Data redistribution: Prohibited without license

**Note:** While API access is geographically unrestricted, data usage must comply with KRX terms of service and Korean financial regulations.

---

## Markets/Exchanges Covered

### Stock Markets

**KRX operates three stock markets:**

#### KOSPI (Korea Composite Stock Price Index)
- Full name: Korea Stock Exchange
- Type: Main board
- Coverage: **Yes** (fully supported)
- Listed companies: ~800 companies (as of 2020s)
- Market cap: Largest market in South Korea
- Typical listings: Large-cap, established companies (Samsung, Hyundai, LG, SK, etc.)
- Market ID parameter: `STK`

#### KOSDAQ (Korea Securities Dealers Automated Quotations)
- Type: Technology and growth companies board
- Coverage: **Yes** (fully supported)
- Listed companies: ~1,500 companies (as of 2020s)
- Typical listings: SMEs, tech companies, growth stocks
- Comparison: Similar to NASDAQ
- Market ID parameter: `KSQ`

#### KONEX (Korea New Exchange)
- Type: SME-focused market
- Coverage: **Yes** (fully supported)
- Listed companies: ~100-150 companies (as of 2024)
- Typical listings: Small businesses, startups, early-stage companies
- Launch year: 2013
- Market ID parameter: `KNX`

**Total Listed Companies (KRX combined): ~2,409 (as of December 2020)**

### International Stock Markets

- US (NYSE, NASDAQ, AMEX): No
- UK (LSE): No
- Japan (TSE): No
- China (SSE, SZSE): No
- Hong Kong (HKEX): No
- Other markets: No

**KRX API covers only Korean markets.**

### Crypto Exchanges

- Not applicable (KRX is traditional stock exchange)
- For Korean crypto data, see: Upbit, Bithumb, Coinone (separate APIs)

### Forex Brokers

- Not applicable
- For KRW forex rates, see: Bank of Korea API

### Futures/Options Exchanges

**KRX Derivatives Market:**
- Coverage via public API: **Limited**
- Derivatives products: KOSPI 200 futures/options, currency futures, commodity futures
- Public API focus: Stock market data (derivatives require commercial license)
- Real-time derivatives data: Commercial only

---

## Instrument Coverage

### Stocks

- **Total symbols: ~2,409** (all KRX markets combined, as of 2020)
- **KOSPI stocks: ~800**
  - Large-cap blue chips
  - Samsung Electronics (005930)
  - SK Hynix (000660)
  - Hyundai Motor (005380)
  - NAVER (035420)
  - Kakao (035720)
  - LG Energy Solution (373220)
- **KOSDAQ stocks: ~1,500**
  - Mid-cap and small-cap
  - Technology companies
  - Biotech and pharma
  - Growth stocks
- **KONEX stocks: ~100-150**
  - Early-stage SMEs
  - Venture companies
- **OTC (Over-the-counter): No** (not covered by standard API)
- **Penny stocks: Yes** (low-priced stocks included if listed)

**Stock Types:**
- Common stocks (보통주): Yes
- Preferred stocks (우선주): Yes
- ETFs: Yes (listed on KRX)
- ETNs: Yes (Exchange Traded Notes)
- REITs: Yes (if listed)
- DRs (Depositary Receipts): Limited

### Crypto

- Not applicable (KRX is not a crypto exchange)

### Forex

- Not applicable (use Bank of Korea or forex providers)

### Commodities

**Direct commodity trading:** Not available on KRX stock markets

**Commodity-related stocks:**
- Mining companies: Yes (if listed)
- Energy companies: Yes (if listed)
- Agricultural companies: Yes (if listed)

**KRX Derivatives Market (commercial):**
- Gold futures: Yes
- Currency futures: Yes (USD/KRW, EUR/KRW, JPY/KRW)
- Interest rate futures: Yes

### Indices

**Major Indices (all supported):**

**KOSPI Indices:**
- KOSPI (Korea Composite Stock Price Index) - Main index
- KOSPI 200 - Top 200 large-cap stocks
- KOSPI 100
- KOSPI 50
- KOSPI Large Cap
- KOSPI Mid Cap
- KOSPI Small Cap

**KOSDAQ Indices:**
- KOSDAQ Composite
- KOSDAQ 150
- KRX 300 (combines KOSPI and KOSDAQ)

**Sector Indices:**
- Electronics
- Automobiles
- Banking
- Telecommunications
- Pharmaceuticals
- Construction
- Chemicals
- Steel
- Retail
- Entertainment & Media

**Thematic Indices:**
- KRX ESG Leaders
- KRX Green New Deal
- KRX Bio
- KRX Autonomous Driving
- KRX Semiconductor

**Other:**
- KRX 100 (top 100 stocks across all markets)
- Dividend indices
- Value/Growth indices

---

## Data History

### Historical Depth

**Stocks:**
- **From year:** Varies by stock (from listing date)
- **Typical depth:**
  - KOSPI blue chips: 1980s-1990s (30-40+ years)
  - Samsung Electronics: Data from 1975 listing
  - KOSDAQ stocks: From KOSDAQ founding (1996) or listing date
  - KONEX stocks: From KONEX founding (2013) or listing date
- **Availability:** Full historical data from listing date to present

**Crypto:** Not applicable

**Forex:** Not applicable (use Bank of Korea for historical KRW rates)

**Indices:**
- KOSPI: Historical data from 1980 (base: January 4, 1980 = 100)
- KOSDAQ: From 1996 (KOSDAQ founding)
- Other indices: From index creation date

### Granularity Available

- **Tick data:** No (not available via public API)
- **1-minute bars:** No (intraday not available)
- **5-minute bars:** No
- **15-minute bars:** No
- **Hourly:** No
- **Daily:** **Yes** (from listing date)
- **Weekly:** Can be calculated from daily
- **Monthly:** Can be calculated from daily

**KRX public API provides daily granularity only.**

### Real-time vs Delayed

- **Real-time:** **No** (not available via public API)
- **Delayed:** **Yes** (+1 business day minimum)
- **Delay period:** Data for date D available on date D+1 at 1:00 PM KST
- **Snapshot:** Yes (latest available is yesterday's close)
- **Intraday:** No

**Example:**
- Trading day: Monday, January 20, 2026
- Data available: Tuesday, January 21, 2026 at 1:00 PM KST
- Delay: ~24+ hours

---

## Update Frequency

### Real-time Streams

- **Not available via public API**

**For real-time data:**
- Use third-party providers (ICE, Twelve Data)
- Direct KRX market data feed (institutional)
- Commercial data vendors

### Scheduled Updates

**Stock Data:**
- Update time: **1:00 PM KST daily**
- Update days: Business days only (Monday-Friday)
- Holidays: No updates on Korean market holidays
- Frequency: Once per day

**Fundamentals:**
- Financial statements: Quarterly, annually (via DART integration)
- Corporate actions: As announced
- Dividends: As declared
- Stock splits: As they occur

**Economic Data:**
- Not provided (use Bank of Korea)

**News:**
- Not provided (use news APIs or DART for disclosures)

**Indices:**
- Update frequency: Daily (same as stock data)
- Historical: Full daily history available

### Market Hours

**Trading Hours (KST):**
- Pre-market: 08:30 - 09:00 (order submission)
- Opening auction: 09:00 (price determination)
- Regular session: 09:00 - 15:30
- Closing auction: 15:20 - 15:30
- After-hours: 15:30 - 16:00 (off-exchange, not in API)

**API Data Covers:**
- Regular session data: Yes (09:00 - 15:30)
- Opening/closing auction: Yes (reflected in OHLC)
- Pre-market: No
- After-hours: No

**Timezone:**
- KST (Korea Standard Time)
- UTC+9 (no daylight saving time)

---

## Data Quality

### Accuracy

- **Source:** Direct from exchange (KRX official data)
- **Validation:** Exchange-validated
- **Corrections:** Automatic (corporate actions, adjustments)
- **Data provider:** Authoritative (this is the exchange itself)
- **Reliability:** Very high (official source)

**Corporate Actions:**
- Stock splits: Automatically adjusted
- Dividends: Reflected in adjusted prices
- Rights issues: Adjusted
- Mergers: Historical data preserved

### Completeness

- **Missing data:** Rare (only for exceptional events)
- **Gaps:** Minimal (only trading halts, suspensions)
- **Backfill:** Not typically needed (data complete from start)
- **Delisted stocks:** Historical data may become unavailable
- **Suspended stocks:** Latest data frozen during suspension

**Trading Halts:**
- Data during halt: Last price before halt
- Resume: Data resumes after trading resumes
- Circuit breakers: Reflected in data

### Timeliness

- **Latency:** +1 business day (public API)
- **Update reliability:** Very consistent (1:00 PM KST daily)
- **Delay variation:** Minimal (usually exactly 1 business day)
- **Market hours coverage:** Full (all trades during 09:00-15:30)
- **Weekend data:** Not available until Monday (for Friday trading)

---

## Special Considerations

### Korean Market Holidays

**KRX is closed on:**
- New Year's Day
- Lunar New Year (Seollal) - 3 days
- Independence Movement Day (March 1)
- Buddha's Birthday
- Children's Day (May 5)
- Memorial Day (June 6)
- Liberation Day (August 15)
- Chuseok (Korean Thanksgiving) - 3 days
- National Foundation Day (October 3)
- Hangul Day (October 9)
- Christmas Day

**Check:** Trading calendar API endpoint for specific dates

### Market Characteristics

**Circuit Breakers:**
- KOSPI: ±8% triggers 20-minute halt
- Individual stocks: ±30% triggers halt
- Reflected in data as gaps or unusual OHLC patterns

**Price Limits:**
- Daily price limit: ±30% from previous close
- New listings: ±30% for initial days, then increased
- Prevents extreme price movements
- Stocks hitting limit show as ceiling/floor in data

**VI (Volatility Interruption):**
- Triggered by rapid price movements
- 2-minute cooling period
- Static VI: Price moves ≥3% in 1 minute
- Dynamic VI: Price moves ≥3% from last VI reference price
- Reflected in data as price plateaus

### Data Interpretation

**Comma-Formatted Numbers:**
- All numeric fields are strings with commas
- Must parse: "12,345,678" → 12345678
- Applies to: prices, volumes, values, market caps

**Korean Language:**
- Stock names: Korean characters (삼성전자, SK하이닉스)
- Sector names: Korean (전기전자, 서비스업)
- Investor types: Korean (개인, 외국인, 기관계)
- Consider: Translation or mapping to English

**ISIN Codes:**
- Format: KR7XXXXXXX (12 digits total)
- Example: KR7005930003 (Samsung Electronics)
- Preferred: KR7005935002 (Samsung Electronics preferred)
- Use ISIN for unambiguous stock identification

---

## Coverage Gaps

### Not Available via KRX Public API

**Market data:**
- ❌ Real-time prices
- ❌ Intraday data (minute/hourly bars)
- ❌ Level 2 order book
- ❌ Tick data
- ❌ Market depth
- ❌ Bid/ask spreads (real-time)

**Advanced data:**
- ❌ Options chains
- ❌ Derivatives data (limited access)
- ❌ Dark pool activity
- ❌ High-frequency trading data

**Fundamental data:**
- ⚠️ Limited (use DART API for comprehensive fundamentals)
- ❌ Analyst ratings
- ❌ Insider trading details
- ❌ Detailed financial statements (use DART)

**Alternative data:**
- ❌ News sentiment
- ❌ Social media data
- ❌ Satellite imagery
- ❌ Credit card transactions

### Recommended Complementary APIs

**For fundamentals:**
- DART (Data Analysis, Retrieval and Transfer System)
- URL: https://opendart.fss.or.kr/
- Coverage: Financial statements, disclosures, filings

**For real-time data:**
- ICE Data Services
- Twelve Data
- Other commercial providers

**For economic data:**
- Bank of Korea (한국은행)
- Korean Statistical Information Service (KOSIS)

**For news:**
- Naver Finance
- Daum Finance
- Bloomberg Korea
- Reuters Korea

---

## Summary Table

| Category | Coverage | Quality | Timeliness | Notes |
|----------|----------|---------|------------|-------|
| KOSPI stocks | Full | Excellent | +1 day | ~800 stocks |
| KOSDAQ stocks | Full | Excellent | +1 day | ~1,500 stocks |
| KONEX stocks | Full | Excellent | +1 day | ~100-150 stocks |
| ETFs | Full | Excellent | +1 day | Listed on KRX |
| Indices | Full | Excellent | +1 day | All major indices |
| Historical | Full | Excellent | Complete | From listing date |
| Intraday | None | N/A | N/A | Use commercial feeds |
| Real-time | None | N/A | N/A | Use commercial feeds |
| Fundamentals | Limited | Good | Varies | Use DART for detail |
| Derivatives | Limited | N/A | N/A | Commercial only |
| International | None | N/A | N/A | Korea only |

---

## Use Case Fit

| Use Case | Suitable? | Notes |
|----------|-----------|-------|
| Historical backtesting | ✅ Excellent | Full daily data, accurate |
| Long-term investing | ✅ Excellent | Fundamentals + price data |
| Swing trading | ✅ Good | Daily data sufficient |
| Day trading | ❌ No | Need intraday data |
| HFT | ❌ No | Need tick data, real-time |
| Portfolio tracking | ✅ Good | Daily updates work |
| Research/Academia | ✅ Excellent | Authoritative data source |
| Quantitative analysis | ✅ Excellent | Clean, structured data |
| Real-time monitoring | ❌ No | +1 day delay |
| Multi-market arbitrage | ❌ Partial | Korea only, delayed |
