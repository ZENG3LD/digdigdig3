# AlphaVantage - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - GLOBAL_QUOTE, CURRENCY_EXCHANGE_RATE
- [x] **Bid/Ask Spread** - Included in GLOBAL_QUOTE
- [x] **24h Ticker Stats** - GLOBAL_QUOTE (high, low, volume, change%)
- [x] **OHLC/Candlesticks** - All TIME_SERIES functions
  - [x] Intraday: 1min, 5min, 15min, 30min, 60min (Premium)
  - [x] Daily
  - [x] Weekly
  - [x] Monthly
- [ ] **Level 2 Orderbook** - Not available
- [x] **Recent Trades** - Not direct endpoint, but price history available
- [x] **Volume** - Included in all time series data
- [x] **Extended Hours** - Pre-market and after-hours (Premium)

## Historical Data

- [x] **Historical prices** (depth: 20+ years for stocks, varies for other assets)
- [x] **Minute bars** (Premium only via TIME_SERIES_INTRADAY)
- [x] **5-minute bars** (Premium)
- [x] **15-minute bars** (Premium)
- [x] **30-minute bars** (Premium)
- [x] **60-minute bars** (Premium)
- [x] **Daily bars** (20+ years)
- [x] **Weekly bars** (20+ years)
- [x] **Monthly bars** (20+ years)
- [ ] **Tick data** - Not available
- [x] **Adjusted prices** - TIME_SERIES_DAILY_ADJUSTED (splits, dividends)

## Derivatives Data (Crypto/Futures)

**Not applicable** - AlphaVantage does not cover:
- [ ] Open Interest
- [ ] Funding Rates
- [ ] Liquidations
- [ ] Long/Short Ratios
- [ ] Mark Price
- [ ] Index Price
- [ ] Basis (futures - spot spread)

**Note**: AlphaVantage focuses on spot markets, not derivatives.

## Options Data (Premium)

- [x] **Options Chains** - REALTIME_OPTIONS (strikes, expirations)
- [x] **Implied Volatility** - Included in REALTIME_OPTIONS
- [x] **Greeks** - Delta, gamma, theta, vega via REALTIME_OPTIONS
- [x] **Open Interest** - Per strike in options data
- [x] **Historical option prices** - HISTORICAL_OPTIONS (15+ years depth)
- [x] **Real-time options** - REALTIME_OPTIONS (Premium)

**Coverage**: US stock options

## Fundamental Data (Stocks)

- [x] **Company Profile** - COMPANY_OVERVIEW
  - [x] Name, sector, industry, description
  - [x] Exchange, currency, country
  - [x] Market cap, shares outstanding
  - [x] IPO date, fiscal year end
- [x] **Financial Statements**
  - [x] Income Statement - INCOME_STATEMENT (annual/quarterly)
  - [x] Balance Sheet - BALANCE_SHEET (annual/quarterly)
  - [x] Cash Flow - CASH_FLOW (annual/quarterly)
- [x] **Earnings** - EARNINGS
  - [x] EPS history
  - [x] Revenue
  - [x] Surprises (actual vs estimate)
  - [x] Fiscal dates
- [x] **Dividends** - DIVIDENDS
  - [x] Dividend history
  - [x] Ex-dividend date
  - [x] Payment date
  - [x] Amount
  - [x] Dividend yield (in COMPANY_OVERVIEW)
- [x] **Stock Splits** - SPLITS
  - [x] All historical splits
  - [x] Split ratio
  - [x] Split date
- [ ] **Analyst Ratings** - Not available
- [x] **Insider Trading** - INSIDER_TRANSACTIONS
  - [x] Insider name, title
  - [x] Transaction type (buy/sell)
  - [x] Shares, price
  - [x] Filing date
- [ ] **Institutional Holdings** - Not available
- [x] **Financial Ratios** - COMPANY_OVERVIEW
  - [x] P/E ratio (trailing, forward)
  - [x] PEG ratio
  - [x] P/B ratio (Price to Book)
  - [x] Price to Sales
  - [x] EV to Revenue
  - [x] EV to EBITDA
  - [x] Profit margin
  - [x] Operating margin
  - [x] Return on Assets (ROA)
  - [x] Return on Equity (ROE)
  - [x] Revenue per share
  - [x] Quarterly earnings growth
  - [x] Quarterly revenue growth
  - [x] Analyst target price
  - [x] Beta
  - [x] 52-week high/low
- [x] **Valuation Metrics** - Included in COMPANY_OVERVIEW
- [x] **Shares Outstanding** - SHARES_OUTSTANDING (historical)
- [x] **ETF Profile** - ETF_PROFILE
  - [x] ETF holdings
  - [x] Expense ratio
  - [x] ETF family
  - [x] Asset allocation

## On-chain Data (Crypto)

**Not applicable** - AlphaVantage does not provide on-chain analytics:
- [ ] Wallet Balances
- [ ] Transaction History (blockchain)
- [ ] DEX Trades
- [ ] Token Transfers
- [ ] Smart Contract Events
- [ ] Gas Prices
- [ ] Block Data
- [ ] NFT Data

**Note**: Crypto coverage limited to price data and ratings (FCAS score).

## Macro/Economic Data (Economics)

- [x] **Interest Rates**
  - [x] Federal Funds Rate - FEDERAL_FUNDS_RATE
  - [x] Treasury Yields - TREASURY_YIELD
    - [x] 3-month
    - [x] 2-year
    - [x] 5-year
    - [x] 7-year
    - [x] 10-year
    - [x] 30-year
- [x] **GDP**
  - [x] Real GDP - REAL_GDP (quarterly)
  - [x] Real GDP per Capita - REAL_GDP_PER_CAPITA (annual)
- [x] **Inflation**
  - [x] CPI - CPI (Consumer Price Index, monthly)
  - [x] Inflation Rate - INFLATION (annual)
  - [ ] PPI - Not available
  - [ ] PCE - Not available
- [x] **Employment**
  - [x] Unemployment Rate - UNEMPLOYMENT (monthly)
  - [x] Nonfarm Payroll - NONFARM_PAYROLL (monthly)
  - [ ] Initial Jobless Claims - Not explicitly documented
- [x] **Retail Sales** - RETAIL_SALES (monthly)
- [x] **Durable Goods** - DURABLES (Durable Goods Orders, monthly)
- [ ] **Consumer Confidence** - Not available
- [ ] **PMI** - Not available
- [ ] **Industrial Production** - Not available
- [ ] **Economic Calendar** - Not available (but EARNINGS_CALENDAR, IPO_CALENDAR for stocks)

**Coverage**: Primarily US economic indicators

## Forex Specific

- [x] **Currency Pairs** - 182 physical currencies supported
  - [x] Majors (USD, EUR, JPY, GBP, CHF, CAD, AUD, NZD)
  - [x] Minors (crosses without USD)
  - [x] Exotics (emerging market currencies)
- [x] **Bid/Ask Spreads** - CURRENCY_EXCHANGE_RATE includes bid/ask
- [x] **Pip precision** - Standard forex precision in responses
- [x] **Cross rates** - Any currency pair combination (182 × 182 possible pairs)
- [x] **Historical FX rates**
  - [x] Intraday (Premium) - FX_INTRADAY
  - [x] Daily - FX_DAILY
  - [x] Weekly - FX_WEEKLY
  - [x] Monthly - FX_MONTHLY
- [x] **Real-time exchange rates** - CURRENCY_EXCHANGE_RATE

**Supported Physical Currencies**: 182 total (see coverage.md for complete list)

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - LISTING_STATUS (US stocks)
- [x] **Exchange Information** - Included in COMPANY_OVERVIEW
- [x] **Market Hours** - MARKET_STATUS (global markets)
  - [x] Open/closed status
  - [x] Market type (equity, forex, crypto)
  - [x] Region
  - [x] Local open/close times
  - [x] Current status
- [ ] **Trading Calendars** - Not available (holidays, half-days)
- [ ] **Timezone Info** - Included in metadata of responses
- [x] **Sector/Industry Classifications** - In COMPANY_OVERVIEW
- [x] **Symbol Search** - SYMBOL_SEARCH (fuzzy search with match score)

## News & Sentiment

- [x] **News Articles** - NEWS_SENTIMENT
  - [x] Title, summary, URL
  - [x] Source
  - [x] Publication time
  - [x] Ticker relevance
- [x] **Sentiment Analysis** - NEWS_SENTIMENT
  - [x] Overall sentiment (positive/negative/neutral)
  - [x] Sentiment score
  - [x] Ticker-specific sentiment
  - [x] Relevance score
- [x] **Topics/Categories** - Filter by topic (earnings, IPO, M&A, etc.)
- [ ] **Press Releases** - Included in news feed
- [ ] **Social Sentiment** - Not available
- [ ] **Analyst Reports** - Not available
- [x] **Earnings Call Transcripts** - EARNINGS_CALL_TRANSCRIPT
- [x] **Top Movers** - TOP_GAINERS_LOSERS
  - [x] Top gainers
  - [x] Top losers
  - [x] Most actively traded

## Commodities

- [x] **Metals**
  - [ ] Gold - Not explicit function (may use commodity index)
  - [ ] Silver - Not explicit function
  - [x] Copper - COPPER
  - [x] Aluminum - ALUMINUM
- [x] **Energy**
  - [x] WTI Crude Oil - WTI
  - [x] Brent Crude Oil - BRENT
  - [x] Natural Gas - NATURAL_GAS
- [x] **Agriculture**
  - [x] Wheat - WHEAT
  - [x] Corn - CORN
  - [x] Cotton - COTTON
  - [x] Sugar - SUGAR
  - [x] Coffee - COFFEE
- [x] **Composite Index**
  - [x] All Commodities - ALL_COMMODITIES

**Intervals**: Daily, weekly, monthly, quarterly, annual (varies by commodity)

## Cryptocurrencies

- [x] **Crypto Spot Prices**
  - [x] Intraday (Premium) - CRYPTO_INTRADAY
  - [x] Daily - DIGITAL_CURRENCY_DAILY
  - [x] Weekly - DIGITAL_CURRENCY_WEEKLY
  - [x] Monthly - DIGITAL_CURRENCY_MONTHLY
- [x] **Real-time Crypto** - CURRENCY_EXCHANGE_RATE (supports BTC, ETH, etc.)
- [x] **Crypto Rating** - CRYPTO_RATING
  - [x] FCAS score (Fundamental Crypto Asset Score)
  - [x] Developer score
  - [x] Market maturity
  - [x] Utility score
- [x] **Market data** - Volume, market cap in responses
- [ ] **On-chain metrics** - Not available

**Supported**: Major cryptocurrencies (see documentation for digital currency list)

## Technical Indicators (50+)

**All indicators work with**: Stocks, Forex, Crypto

### Moving Averages
- [x] SMA - Simple Moving Average
- [x] EMA - Exponential Moving Average
- [x] WMA - Weighted Moving Average
- [x] DEMA - Double Exponential Moving Average
- [x] TEMA - Triple Exponential Moving Average
- [x] TRIMA - Triangular Moving Average
- [x] KAMA - Kaufman Adaptive Moving Average
- [x] MAMA - MESA Adaptive Moving Average
- [x] VWAP - Volume Weighted Average Price (Premium)
- [x] T3 - T3 Moving Average

### Momentum Indicators
- [x] MACD - Moving Average Convergence Divergence (Premium)
- [x] MACDEXT - MACD with Controllable MA Type
- [x] RSI - Relative Strength Index
- [x] STOCH - Stochastic Oscillator
- [x] STOCHF - Stochastic Fast
- [x] STOCHRSI - Stochastic RSI
- [x] WILLR - Williams %R
- [x] ADX - Average Directional Movement Index
- [x] ADXR - Average Directional Movement Index Rating
- [x] APO - Absolute Price Oscillator
- [x] PPO - Percentage Price Oscillator
- [x] MOM - Momentum
- [x] BOP - Balance of Power
- [x] CCI - Commodity Channel Index
- [x] CMO - Chande Momentum Oscillator
- [x] ROC - Rate of Change
- [x] ROCR - Rate of Change Ratio
- [x] AROON - Aroon Indicator
- [x] AROONOSC - Aroon Oscillator
- [x] MFI - Money Flow Index
- [x] TRIX - Triple Exponential Moving Average Oscillator
- [x] ULTOSC - Ultimate Oscillator
- [x] DX - Directional Movement Index

### Volatility Indicators
- [x] BBANDS - Bollinger Bands
- [x] ATR - Average True Range
- [x] NATR - Normalized Average True Range
- [x] TRANGE - True Range
- [x] SAR - Parabolic SAR

### Volume Indicators
- [x] OBV - On Balance Volume
- [x] AD - Accumulation/Distribution Line
- [x] ADOSC - Accumulation/Distribution Oscillator

### Directional Movement
- [x] MINUS_DI - Minus Directional Indicator
- [x] PLUS_DI - Plus Directional Indicator
- [x] MINUS_DM - Minus Directional Movement
- [x] PLUS_DM - Plus Directional Movement

### Hilbert Transform (Cycle Indicators)
- [x] HT_TRENDLINE - Instantaneous Trendline
- [x] HT_SINE - Sine Wave
- [x] HT_TRENDMODE - Trend vs Cycle Mode
- [x] HT_DCPERIOD - Dominant Cycle Period
- [x] HT_DCPHASE - Dominant Cycle Phase
- [x] HT_PHASOR - Phasor Components

### Price Indicators
- [x] MIDPOINT - Midpoint Price
- [x] MIDPRICE - Midpoint Price over Period

**Total**: 50+ indicators, works across all asset classes

## Unique/Custom Data

**What makes AlphaVantage special:**

### 1. Comprehensive Multi-Asset Coverage
Single API for:
- ✅ Stocks (200,000+ global)
- ✅ Forex (182 currencies)
- ✅ Crypto (major coins)
- ✅ Commodities (energy, metals, agriculture)
- ✅ Economic indicators
- ✅ Options

**Unique advantage**: No need to integrate multiple APIs for different asset classes.

### 2. Pre-Computed Technical Indicators (50+)
Most APIs provide raw OHLCV and you compute indicators client-side. AlphaVantage:
- ✅ Server-side computation of 50+ indicators
- ✅ Saves client CPU/memory
- ✅ Consistent calculations
- ✅ Works across stocks, forex, crypto

### 3. Fundamental Data Integration
Rare for data APIs to combine market data + fundamentals:
- ✅ Financial statements (income, balance sheet, cash flow)
- ✅ Earnings history and estimates
- ✅ Company metrics and ratios
- ✅ Insider transactions
- ✅ Dividends and splits

### 4. Economic Indicators
Few stock APIs include economic data:
- ✅ GDP, CPI, unemployment
- ✅ Treasury yields, interest rates
- ✅ Retail sales, durable goods
- ✅ Nonfarm payroll

**Use case**: Macro-driven trading strategies

### 5. News Sentiment Analysis (AI-Powered)
- ✅ Real-time news feed
- ✅ AI sentiment scoring
- ✅ Ticker relevance scores
- ✅ Topic filtering (M&A, IPO, earnings, etc.)
- ✅ Global news sources

### 6. Options Data (Premium)
- ✅ Real-time options chains
- ✅ Greeks (delta, gamma, theta, vega)
- ✅ Implied volatility
- ✅ 15+ years historical options

**Rare**: Most free/cheap APIs don't include options.

### 7. Crypto FCAS Ratings
- ✅ Fundamental Crypto Asset Score
- ✅ Developer activity
- ✅ Market maturity
- ✅ Utility metrics

**Unique**: Fundamental analysis for crypto (not just price).

### 8. NASDAQ-Licensed Data
- ✅ Regulatory compliant
- ✅ Licensed by SEC, FINRA
- ✅ Reliable for commercial use

**Important**: Many "free" APIs use unlicensed data sources.

### 9. Deep Historical Coverage
- ✅ 20+ years stock/forex history
- ✅ 15+ years options history
- ✅ Adjusted for splits/dividends

### 10. Simple, Function-Based API
- ✅ Single base URL
- ✅ Consistent parameter naming
- ✅ Easy to learn and integrate
- ✅ JSON or CSV output

### 11. MCP Support (2026)
- ✅ Native AI assistant integration
- ✅ Claude, ChatGPT compatible
- ✅ Natural language queries

**Innovative**: First major financial API with MCP support.

## Data NOT Available

To set clear expectations:

- ❌ **WebSocket** - No real-time streaming
- ❌ **Level 2 Orderbook** - No bid/ask depth beyond top of book
- ❌ **Tick data** - No individual trade ticks
- ❌ **Futures/Derivatives** - Only spot markets (+ options)
- ❌ **On-chain crypto data** - No blockchain analytics
- ❌ **Social sentiment** - Only news sentiment
- ❌ **Analyst reports** - Only ratings/estimates in fundamentals
- ❌ **Institutional holdings** - Not available
- ❌ **Full economic calendar** - Have earnings/IPO calendars, not general events
- ❌ **Trading execution** - Data only, no orders
- ❌ **Portfolio management** - No account features
- ❌ **Screening/Scanning** - No complex screeners (but TOP_GAINERS_LOSERS)
- ❌ **Charting** - Raw data only, no chart images
- ❌ **Alerts** - No server-side alerts

## Summary: AlphaVantage's Data Breadth

| Category | Coverage | Depth | Unique Features |
|----------|----------|-------|-----------------|
| **Stocks** | ⭐⭐⭐⭐⭐ (200,000+) | 20+ years | Comprehensive fundamentals |
| **Forex** | ⭐⭐⭐⭐⭐ (182 currencies) | 20+ years | All major, minor, exotic pairs |
| **Crypto** | ⭐⭐⭐ (Major coins) | Limited | FCAS ratings |
| **Commodities** | ⭐⭐⭐ (Key commodities) | Varies | Energy, metals, agriculture |
| **Options** | ⭐⭐⭐⭐ (US stocks) | 15+ years | Greeks, IV (Premium) |
| **Economic** | ⭐⭐⭐⭐ (US indicators) | Decades | GDP, CPI, employment, rates |
| **Technical** | ⭐⭐⭐⭐⭐ (50+ indicators) | N/A | Pre-computed, multi-asset |
| **Fundamental** | ⭐⭐⭐⭐ (US stocks) | Comprehensive | Financials, earnings, ratios |
| **News** | ⭐⭐⭐⭐ (Global) | Real-time | AI sentiment analysis |
| **Real-time** | ⭐⭐⭐ (Premium only) | N/A | No WebSocket, polling only |

**Overall Rating**: ⭐⭐⭐⭐ (4/5)

**Strengths**: Breadth of coverage, fundamentals, indicators, multi-asset
**Weaknesses**: No WebSocket, premium required for intraday, free tier very limited
