# KRX - Data Types Catalog

## Standard Market Data

- [x] Current Price (delayed by 1+ business day)
- [x] 24h Ticker Stats (high, low, volume, change%)
- [x] OHLC/Candlesticks (intervals: **daily only**)
- [ ] Level 2 Orderbook (not available via public API)
- [x] Daily Trading Volume
- [x] Market Capitalization
- [ ] Real-time bid/ask (not available via public API)
- [ ] Intraday data (not available - daily granularity only)

## Historical Data

- [x] Historical daily prices (depth: from listing date)
- [ ] Minute bars (not available via public API)
- [x] Daily bars (depth: full history from listing)
- [ ] Tick data (not available via public API)
- [x] Adjusted prices (splits and dividends reflected)
- [x] Corporate actions history (splits, dividends)

**Granularity:** Daily only (no intraday through public API)

**Historical Depth:**
- KOSPI stocks: From listing date (decades of data for older companies)
- KOSDAQ stocks: From listing date
- KONEX stocks: From listing date (newer market, less history)

## Derivatives Data

- [ ] Not applicable (KRX Derivatives Market not covered by standard public API)
- [ ] Open Interest (futures/options - separate API/commercial)
- [ ] Funding Rates (not applicable to stock market)
- [ ] Liquidations (not applicable)

**Note:** KRX operates a derivatives market, but public API focuses on stock market data. Derivatives data requires commercial data feed.

## Options Data

- [ ] Options Chains (not available via public API)
- [ ] Implied Volatility (not available)
- [ ] Greeks (not available)
- [ ] Open Interest per strike (not available)
- [ ] Historical option prices (not available)

**Note:** Options data exists on KRX but requires commercial data services.

## Fundamental Data (Stocks)

### Available via KRX API

- [x] Company Profile (name, ticker, ISIN)
- [x] Sector/Industry Classification
- [x] Listing Date
- [x] Number of Listed Shares
- [x] Market Capitalization (calculated)
- [x] Stock Type (common, preferred)
- [x] Market Type (KOSPI, KOSDAQ, KONEX)
- [x] Corporate Registration Number

### Limited Availability

- [x] Basic Financial Ratios (P/E, P/B - may require calculation from DART data)
- [x] Dividends (history available but may need DART integration)
- [x] Stock Splits (historical data available)
- [ ] Earnings (limited - better through DART API)
- [ ] Financial Statements (limited - use DART API instead)
- [ ] Analyst Ratings (not available)
- [ ] Insider Trading (not available via this API)
- [ ] Institutional Holdings (partial data available)

### Recommended: Use DART API for Comprehensive Fundamentals

**DART (Data Analysis, Retrieval and Transfer System):**
- URL: https://opendart.fss.or.kr/
- Purpose: Electronic disclosure system for Korean companies
- Coverage: All listed companies (KOSPI, KOSDAQ, KONEX)
- Data: Financial statements, earnings, disclosures, filings
- API: Free with registration

**Integration:** For complete fundamental analysis, combine KRX (price data) + DART (fundamental data)

## Trading Statistics

### Investor Type Analysis

- [x] Trading Value by Investor Type
  - Individual investors (개인)
  - Institutional investors (기관)
  - Foreign investors (외국인)
  - Breakdown by buy/sell/net
- [x] Trading Volume by Investor Type
- [x] Foreign Ownership Ratio
- [x] Institutional Ownership

### Market Activity

- [x] Program Trading Data
- [x] Short Selling Data
  - Short position by stock
  - Short selling volume
  - Short selling value
- [x] Daily Trading Value (total market)
- [x] Daily Trading Volume (total market)

## Metadata & Reference

- [x] Symbol/Instrument Lists
  - KOSPI listed stocks
  - KOSDAQ listed stocks
  - KONEX listed stocks
  - ETFs
- [x] Exchange Information
- [x] Market Hours (regular: 09:00-15:30 KST)
- [x] Trading Calendars (holidays, market closures)
- [x] Timezone Info (Asia/Seoul, GMT+9, no DST)
- [x] Sector/Industry Classifications (Korean standard)
- [x] ISIN Codes
- [x] Stock Code Mappings

## Index Data

- [x] KOSPI Index (Korea Composite Stock Price Index)
- [x] KOSDAQ Index
- [x] KRX 100
- [x] KOSPI 200
- [x] Sector Indices
- [x] Thematic Indices
- [x] Historical Index Values
- [x] Index Constituent Lists

## Sector/Industry Data

- [x] Sector Performance
- [x] Industry Classification
- [x] Sector Statistics (market cap, volume, etc.)
- [x] Sector Constituent Lists
- [x] Industry Benchmarks

## Corporate Actions

- [x] Stock Splits
- [x] Reverse Splits
- [x] Dividends (cash and stock)
- [x] Rights Issues
- [x] Mergers & Acquisitions (basic info)
- [x] Delisting Information
- [x] Listing Information (IPOs)

## News & Sentiment

- [ ] News Articles (not available via KRX API)
- [ ] Press Releases (limited - use DART)
- [ ] Social Sentiment (not available)
- [ ] Analyst Reports (not available)

**Recommendation:** Use third-party providers or DART for news/sentiment

## Economic Data

- [ ] Not provided by KRX API
- [ ] For Korean economic data, use Bank of Korea (한국은행) APIs
- [ ] For global economic data, use providers like FRED, World Bank, etc.

## Forex Data

- [ ] Not applicable (KRX is stock exchange)
- [ ] For Korean Won (KRW) forex rates, use Bank of Korea or forex providers

## Market Microstructure (Limited)

- [x] Trading Halts (information may be available)
- [x] Circuit Breakers (status information)
- [ ] Odd Lot Trading (not documented)
- [ ] Block Trades (not available via public API)
- [ ] Dark Pool Activity (not applicable/not public)

## ETF-Specific Data

- [x] ETF Listings (listed on KRX)
- [x] ETF Prices (same as stocks)
- [x] ETF Trading Volume
- [ ] NAV (Net Asset Value) - may require separate lookup
- [ ] ETF Holdings - typically available from ETF provider
- [ ] Creation/Redemption Data - not public API

## ESG Data

- [ ] ESG Ratings (not available via KRX API)
- [ ] Carbon Emissions (not available)
- [ ] Sustainability Scores (not available)

**Note:** ESG data for Korean companies may be available through specialized providers or DART filings

## Alternative Data

- [ ] Satellite Imagery (not applicable)
- [ ] Social Media Sentiment (not available)
- [ ] Web Traffic (not available)
- [ ] Credit Card Data (not available)

**Note:** Alternative data requires third-party providers

## Unique/Custom Data

### What makes KRX data special:

**1. Korean Market Focus**
- Comprehensive coverage of KOSPI, KOSDAQ, KONEX
- Korean-specific market structure data
- Local investor behavior patterns (individual vs institutional)

**2. Investor Type Breakdown**
- Detailed tracking of foreign investor flows
- Institutional investor activity
- Retail investor sentiment (via trading patterns)
- **Unique insight:** Korean market has distinct retail investor culture

**3. Program Trading Transparency**
- Program trading volume disclosed
- Algorithmic trading activity visible
- Market impact analysis possible

**4. Short Selling Transparency**
- Comprehensive short selling data
- Daily short position reports
- Short selling restrictions tracking
- **More transparent than many markets**

**5. Chaebol Coverage**
- Data on major Korean conglomerates (Samsung, Hyundai, LG, SK, etc.)
- Inter-company relationships
- Cross-holdings data (via DART integration)

**6. Government Data Integration**
- Official government data portal integration
- Regulatory filing connections (DART)
- Authoritative source (exchange-operated)

**7. Market Structure**
- Pre-market auction data
- Closing auction data
- VI (Volatility Interruption) events
- Market stabilization measures

## Data Quality Notes

### Strengths
- Official exchange data (authoritative)
- Complete coverage of all listed stocks
- No sampling - full market data
- Corporate action adjustments included
- Long historical depth

### Limitations
- Delayed data (+1 business day)
- Daily granularity only
- No real-time feeds via public API
- Limited fundamental data (use DART)
- Korean language prevalent in some fields

## Data Fields Available

### Stock Price Data (OHLCV)
```
TRD_DD      - Trade Date (YYYYMMDD)
TDD_OPNPRC  - Opening Price
TDD_HGPRC   - High Price
TDD_LWPRC   - Low Price
TDD_CLSPRC  - Close Price
ACC_TRDVOL  - Accumulated Trade Volume
ACC_TRDVAL  - Accumulated Trade Value
FLUC_TP_CD  - Fluctuation Type Code (up/down/unchanged)
CMPPRVDD_PRC - Compared to Previous Day Price
FLUC_RT     - Fluctuation Rate (%)
```

### Stock Information
```
ISU_CD      - Issue Code (ISIN)
ISU_SRT_CD  - Issue Short Code (ticker)
ISU_NM      - Issue Name (company name)
MKT_NM      - Market Name (KOSPI/KOSDAQ/KONEX)
SECUGRP_NM  - Security Group Name
STOCK_KIND  - Stock Kind (common/preferred)
LIST_DD     - Listing Date
LIST_SHRS   - Listed Shares
MKTCAP      - Market Capitalization
```

### Trading Statistics
```
INVSTRY_NM   - Investor Name (type)
BUY_TRDVOL   - Buy Trade Volume
BUY_TRDVAL   - Buy Trade Value
SELL_TRDVOL  - Sell Trade Volume
SELL_TRDVAL  - Sell Trade Value
NET_TRDVOL   - Net Trade Volume
NET_TRDVAL   - Net Trade Value
```

## Coverage Summary

| Data Type | Coverage | Granularity | Historical Depth | Real-time |
|-----------|----------|-------------|------------------|-----------|
| Stock Prices | All listed stocks | Daily | From listing | No (+1 day) |
| Volume | All listed stocks | Daily | From listing | No (+1 day) |
| Market Cap | All listed stocks | Daily | From listing | No (+1 day) |
| Investor Type | All listed stocks | Daily | Several years | No (+1 day) |
| Indices | All KRX indices | Daily | Full history | No (+1 day) |
| Short Selling | Most stocks | Daily | Recent years | No (+1 day) |
| Fundamentals | Limited | Varies | Varies | No |
| Options/Futures | Not available | N/A | N/A | No |
| Intraday | Not available | N/A | N/A | No |

## Recommended Data Combinations

### For Price Analysis
- KRX API: OHLCV, volume, market cap
- Sufficient for technical analysis and backtesting

### For Fundamental Analysis
- KRX API: Price data, sector classification
- DART API: Financial statements, earnings, disclosures
- Combined: Complete fundamental analysis

### For Market Sentiment
- KRX API: Investor type trading (foreign/institutional/retail)
- DART API: Corporate disclosures
- Third-party: News sentiment

### For Quantitative Research
- KRX API: Historical prices, volume, investor flows
- DART API: Financial ratios, earnings
- Bank of Korea: Economic indicators
- Combined: Multi-factor models
