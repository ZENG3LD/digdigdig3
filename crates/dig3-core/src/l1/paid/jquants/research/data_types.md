# JQuants - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Available via daily quotes (Close price)
- [x] **Bid/Ask Spread** - Not available (data-only API, no order book)
- [x] **24h Ticker Stats** - Not available in standard format
- [x] **OHLC/Candlesticks** - Available (intervals: daily, minute via add-on)
  - Daily bars: All tiers
  - Minute bars: Add-on plan (January 2026)
  - Intraday bars: Via minute bars add-on
- [ ] **Level 2 Orderbook** - Not available (no order book data)
- [x] **Recent Trades** - Available via tick data add-on (January 2026)
- [x] **Volume** - Daily volume included in OHLC
- [x] **Morning/Afternoon Session Data** - Premium tier only

## Historical Data

- [x] **Historical prices** - Depth varies by tier:
  - Free: 2 years (12-week delay)
  - Light: 5 years
  - Standard: 10 years
  - Premium: All available (from May 7, 2008 onwards)
- [x] **Minute bars** - Add-on plan (January 2026), historical depth TBD
- [x] **Daily bars** - All tiers (depth varies)
- [x] **Tick data** - Add-on plan (January 2026)
- [x] **Adjusted prices** - Splits and dividends adjustment included
  - AdjustmentFactor field
  - AdjustmentOpen, AdjustmentHigh, AdjustmentLow, AdjustmentClose

## Derivatives Data (Futures & Options)

Available on **Premium tier only**:

- [x] **Futures Prices (OHLC)** - TOPIX futures, Nikkei 225 futures
  - Whole day OHLC
  - Day session OHLC
  - Night session OHLC
- [x] **Options Prices (OHLC)** - Index options (TOPIX, Nikkei 225)
  - Strike prices
  - Implied volatility
  - Theoretical prices
- [x] **Open Interest** - Included in futures/options data
- [ ] **Funding Rates** - Not applicable (stock market, not crypto)
- [ ] **Liquidations** - Not applicable (no margin liquidations data)
- [ ] **Long/Short Ratios** - Not directly available
- [x] **Mark Price** - Settlement price provided
- [x] **Index Price** - TOPIX and other indices (Standard tier+)
- [ ] **Basis** - Not directly provided (can calculate from futures/spot)

### Available Futures Products
- TOPIX Futures
- Nikkei 225 Futures
- Mini-TOPIX
- Nikkei 225 Mini
- Nikkei 225 Micro

### Available Options Products
- TOPIX Index Options
- Nikkei 225 Index Options

## Options Data (Index Options)

Available on **Standard tier** (basic) and **Premium tier** (full):

- [x] **Options Chains** - Standard tier (index options endpoint)
- [x] **Implied Volatility** - Premium tier (derivatives/options)
- [x] **Greeks** - Not directly provided (must calculate)
- [x] **Open Interest** - Premium tier (derivatives/options)
- [x] **Historical option prices** - Premium tier

## Fundamental Data (Stocks)

- [x] **Company Profile**
  - Company name (Japanese and English)
  - Sector classification (17-sector and 33-sector)
  - Industry classification
  - Market segment (Prime, Standard, Growth)
  - Scale category (TOPIX Core30, Large70, Mid400, Small)
- [x] **Financial Statements** - All tiers (12-week delay on Free)
  - Income Statement (Net Sales, Operating Profit, Ordinary Profit, Profit)
  - Balance Sheet (Total Assets, Equity, Equity-to-Asset Ratio)
  - Cash Flow (Operating, Investing, Financing Activities)
  - Available for: Quarterly, Annual
  - Standards: Japanese GAAP (IFRS/US GAAP fields may be blank)
  - Premium tier: Full BS/PL/CF data
- [x] **Earnings** - Financial statements endpoint
  - EPS (basic and diluted)
  - Revenue, guidance (if disclosed)
  - Quarterly and annual results
- [x] **Dividends** - Premium tier
  - Cash dividend announcements
  - Dividend per share
  - Interim/final dividends
  - Commemorative/special dividends
  - Record date, ex-date, payable date
  - Forecast and actual dividends
- [x] **Stock Splits** - Included in adjustment factor
- [ ] **Analyst Ratings** - Not available
- [ ] **Insider Trading** - Not available
- [ ] **Institutional Holdings** - Not available
- [x] **Financial Ratios** - Some calculated in statements
  - Equity-to-Asset Ratio
  - Book Value Per Share
  - EPS
  - (Other ratios must be calculated from raw financial data)
- [ ] **Valuation Metrics** - Not directly provided (calculate from financials)

## Market Structure Data (Japan-Specific)

### Trading by Type of Investors (Standard tier+)
- [x] **Investor Type Breakdown** - Weekly data
  - Individuals
  - Foreigners
  - Securities companies
  - Investment trusts
  - Business corporations
  - Other corporations
  - Banks/Trust banks
  - Life insurance companies
  - Non-life insurance companies
  - Pension funds
  - Other financial institutions

### Margin Trading Data (Standard tier+)
- [x] **Margin Trading Outstanding** - Weekly data
  - Margin buy outstanding
  - Margin sell outstanding
  - Lending outstanding

### Short Selling Data (Standard tier+)
- [x] **Short Sale Value and Ratio** - Daily data by sector
  - Short selling with restrictions
  - Short selling without restrictions
  - Regular selling (excluding short sales)
  - Aggregated by 33-sector codes

### Breakdown Trading Data (Premium tier)
- [x] **Detail Breakdown Trading** - Granular trading analysis
  - Detailed investor type breakdowns
  - Trading value analysis

## On-chain Data (Crypto)

- [ ] Not applicable (stock market API)

## Macro/Economic Data (Economics)

- [ ] Not available (Japan stock market focus only)
- [ ] Economic calendar not included
- Note: For Japanese economic data, use FRED or other macro data sources

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - Listed issue master endpoint
  - All TSE listed stocks
  - Stock code, company name (JP/EN)
  - Sector codes (17 and 33 classifications)
- [x] **Exchange Information** - Tokyo Stock Exchange (TSE) focus
  - Market codes: Prime Market, Standard Market, Growth Market
  - Osaka Exchange (OSE) for derivatives
- [x] **Market Hours** - Not directly in API (known constants)
  - TSE Regular: 9:00-11:30 (morning), 12:30-15:00 (afternoon) JST
  - OSE Night: 16:30-03:00 (next day) JST
- [x] **Trading Calendars** - Trading calendar endpoint
  - Business days
  - Holidays
  - Half-day trading indicators
  - Updated annually around March
- [x] **Timezone Info** - All timestamps in JST (Japan Standard Time, UTC+9)
- [x] **Sector/Industry Classifications**
  - SECTOR17: 17 broad categories
  - SECTOR33: 33 detailed categories
  - Tokyo Stock Exchange official classifications

### Sector Classifications

**17-Sector Codes** (examples):
- 1: Food
- 2: Energy
- 3: Construction
- 4: Chemicals
- 5: Pharmaceutical
- 6: Steel
- 7: Machinery
- 8: Electric Appliances
- 9: Precision Instruments
- 10: Automotive
- 11: Trading Companies
- 12: Retail
- 13: Banks
- 14: Finance (excl. Banks)
- 15: Real Estate
- 16: Transportation
- 17: Utilities

**33-Sector Codes**: More granular breakdowns (see `/v1/listed/info` for full list)

## News & Sentiment

- [ ] **News Articles** - Not available
- [ ] **Press Releases** - Not available directly (use financial statements disclosures)
- [ ] **Social Sentiment** - Not available
- [ ] **Analyst Reports** - Not available

## Unique/Custom Data

**What makes JQuants special?**

1. **Official TSE Data** - Direct from Japan Exchange Group
   - Not aggregated or third-party
   - Exchange-official quality and timeliness

2. **Japan-Specific Classifications**
   - 17-sector and 33-sector codes (TSE official)
   - TOPIX scale categories (Core30, Large70, Mid400, Small)
   - Market segments (Prime, Standard, Growth)

3. **Investor Type Breakdowns** - Unique to Japanese market
   - Weekly trading by investor nationality/type
   - Tracks foreign vs domestic flows
   - Institutional vs retail breakdown

4. **Margin Trading Outstandings** - Weekly updates
   - Margin buy/sell positions
   - Japan-specific margin system data

5. **Short Selling by Sector** - Daily aggregated short sales
   - Broken down by 33 sectors
   - Restricted vs unrestricted short sales

6. **Morning/Afternoon Session Splits** - Premium tier
   - Separate OHLC for AM/PM sessions
   - Unique to Asian markets with lunch breaks

7. **Cash Dividend Granularity** - Premium tier
   - Interim/final classification
   - Commemorative/special dividend flags
   - Deemed dividend calculations

8. **Japanese GAAP Focus** - Financial statements
   - Native Japanese accounting standards
   - IFRS/US GAAP as secondary

9. **TOPIX-Specific Data** - Standard tier
   - TOPIX prices and components
   - Scale categorizations based on TOPIX
   - TOPIX futures and options

10. **Index Options** - Standard tier
    - TOPIX and Nikkei 225 options
    - Osaka Exchange derivatives

## Data Availability Matrix

| Data Type | Free | Light | Standard | Premium | Add-on Required |
|-----------|------|-------|----------|---------|-----------------|
| Daily stock prices | ✅ (12w delay) | ✅ | ✅ | ✅ | No |
| Morning/afternoon prices | ❌ | ❌ | ❌ | ✅ | No |
| Minute bars | ❌ | ❌ | ❌ | ❌ | ✅ |
| Tick data | ❌ | ❌ | ❌ | ❌ | ✅ |
| Listed info | ✅ (12w delay) | ✅ | ✅ | ✅ | No |
| Financial statements | ✅ (12w delay) | ✅ | ✅ | ✅ (full) | No |
| Earnings calendar | ✅ | ✅ | ✅ | ✅ | No |
| Trading calendar | ✅ (12w delay) | ✅ | ✅ | ✅ | No |
| TOPIX/indices | ❌ | ❌ | ✅ | ✅ | No |
| Index options | ❌ | ❌ | ✅ | ✅ | No |
| Futures prices | ❌ | ❌ | ❌ | ✅ | No |
| Options prices (full) | ❌ | ❌ | ❌ | ✅ | No |
| Trading by investor type | ❌ | ❌ | ✅ | ✅ | No |
| Margin trading | ❌ | ❌ | ✅ | ✅ | No |
| Short selling | ❌ | ❌ | ✅ | ✅ | No |
| Breakdown trading | ❌ | ❌ | ❌ | ✅ | No |
| Cash dividends | ❌ | ❌ | ❌ | ✅ | No |

## Missing Data Types (Not Available)

- Real-time order book (L2 depth)
- Bid/ask quotes
- Real-time trade stream (via WebSocket)
- Analyst ratings/estimates
- Insider trading data
- Institutional ownership changes
- News/press releases
- Social sentiment
- Economic indicators
- Cross-market correlation data
- ADR/foreign listing data
- Corporate actions (beyond splits/dividends)
- Shareholder meetings data
- Credit ratings
- Bond data (corporate/government)
- ETF holdings/constituents

## Data Quality Notes

### Accuracy
- Source: **Direct from Tokyo Stock Exchange** (official)
- Validation: Exchange-validated
- Corrections: Automatic from exchange

### Completeness
- Missing data: Rare (exchange downtime only)
- Gaps: Minimal (weekends, holidays expected)
- Backfill: Available via historical endpoints

### Timeliness (Update Schedule)

| Data Type | Update Time (JST) | Delay |
|-----------|-------------------|-------|
| Daily prices | 16:30 | ~30 min after market close |
| Morning prices | 12:00 | ~30 min after morning session |
| Financial statements | 18:00 / 24:30 | Same day (prelim/final) |
| Indices | 16:30 | ~30 min after close |
| Futures/Options | 27:00 (3:00 AM) | ~3 hours after night session |
| Trading by type | Thursday 18:00 | 4 business days lag (weekly) |
| Margin trading | Tuesday 16:30 | 2 business days lag (weekly) |
| Short selling | 16:30 | Same day |
| Dividends | 12:00-19:00 | Intraday updates |
| Earnings calendar | ~19:00 | Next business day forecast |

### Standards
- Dates: YYYY-MM-DD format
- Timestamps: JST (UTC+9)
- Numbers: Floating-point (prices), integers (volumes)
- Currency: JPY (Japanese Yen)
- Accounting: Japanese GAAP primary, IFRS/US GAAP secondary

## Recommended Use Cases by Data Type

### For Daily Trading/Analysis
- Daily OHLC (Light tier minimum)
- Volume data
- Sector classifications

### For Backtesting
- Historical adjusted prices (Standard/Premium for depth)
- Financial statements
- Dividend history

### For Fundamental Analysis
- Financial statements (quarterly/annual)
- Company profiles
- Sector classifications
- Earnings calendar

### For Market Structure Research
- Trading by investor type (Standard tier)
- Margin trading data (Standard tier)
- Short selling data (Standard tier)
- Breakdown trading (Premium tier)

### For Derivatives Trading
- Futures prices (Premium tier)
- Options chains and IV (Premium tier)
- Index prices (Standard tier)

### For High-Frequency/Intraday
- Minute bars (Add-on)
- Tick data (Add-on)
- Morning/afternoon sessions (Premium tier)

## Data Format Examples

### Stock Price (OHLC)
```json
{
  "Date": "2024-01-15",
  "Code": "7203",
  "Open": 2500.0,
  "High": 2550.0,
  "Low": 2480.0,
  "Close": 2530.0,
  "Volume": 12345678,
  "TurnoverValue": 31234567890,
  "AdjustmentFactor": 1.0,
  "AdjustmentOpen": 2500.0,
  "AdjustmentHigh": 2550.0,
  "AdjustmentLow": 2480.0,
  "AdjustmentClose": 2530.0
}
```

### Listed Info
```json
{
  "Date": "2024-01-15",
  "Code": "7203",
  "CompanyName": "トヨタ自動車株式会社",
  "CompanyNameEnglish": "Toyota Motor Corporation",
  "Sector17Code": "10",
  "Sector17CodeName": "自動車",
  "Sector33Code": "3350",
  "Sector33CodeName": "自動車",
  "ScaleCategory": "TOPIX Core30",
  "MarketCode": "0111",
  "MarketCodeName": "プライム"
}
```

### Financial Statement
```json
{
  "DisclosedDate": "2024-01-15",
  "Code": "7203",
  "FiscalYear": "2024-03-31",
  "FiscalQuarter": "Q3",
  "NetSales": 9000000000000,
  "OperatingProfit": 800000000000,
  "OrdinaryProfit": 850000000000,
  "Profit": 600000000000,
  "EarningsPerShare": 420.50,
  "TotalAssets": 50000000000000,
  "Equity": 25000000000000,
  "EquityToAssetRatio": 50.0,
  "BookValuePerShare": 15000.00
}
```

## Summary

JQuants provides **comprehensive Japanese stock market data** with:
- ✅ Daily and intraday OHLC
- ✅ Fundamentals (financials, earnings, dividends)
- ✅ Derivatives (futures, options) on Premium
- ✅ Market structure (investor flows, margin, short selling)
- ✅ Official TSE/OSE exchange data
- ✅ Historical depth up to May 2008
- ❌ No real-time streaming (REST only)
- ❌ No order book data
- ❌ No news/sentiment
- ❌ No analyst ratings

**Best for**: Backtesting, fundamental analysis, market structure research, end-of-day trading systems, academic research on Japanese equities.

**Not suitable for**: Real-time trading, HFT, order book analysis, global multi-market data (Japan only).
