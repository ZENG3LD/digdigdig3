# Twelvedata - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Latest trade price via `/price` endpoint
- [x] **Bid/Ask Spread** - Available in `/quote` endpoint
- [x] **24h Ticker Stats** - High, low, volume, change%, change via `/quote`
- [x] **OHLC/Candlesticks** - Via `/time_series` endpoint
  - Intervals: 1min, 5min, 15min, 30min, 45min, 1h, 2h, 4h, 1day, 1week, 1month
  - Max 5000 bars per request
  - Adjustable for splits/dividends
- [ ] **Level 2 Orderbook** - NOT AVAILABLE (no depth data)
- [x] **Recent Trades** - Implied (not explicit endpoint, use time series 1min)
- [x] **Volume** - 24h volume in quote, historical volume in time series
- [x] **VWAP** - Via technical indicators (Volume Weighted Moving Average)

## Historical Data

- [x] **Historical prices** - Via `/time_series` endpoint
  - **Depth**: 1980s-1990s for many assets (varies by symbol)
  - **Stocks**: Often back to listing date
  - **Forex**: Decades of history
  - **Crypto**: Back to coin inception
- [x] **Minute bars** - Available via `interval=1min`
  - **Depth**: 1-2 years on Basic plan, 5+ years on Grow+, unlimited on Pro+
- [x] **Daily bars** - Available via `interval=1day`
  - **Depth**: Decades for major stocks
- [ ] **Tick data** - NOT AVAILABLE (no individual tick data)
- [x] **Adjusted prices** - Splits and dividends via `adjust` parameter
  - Options: splits, dividends, or none
  - Corporate actions automatically applied when enabled

## Derivatives Data (Crypto/Futures)

**NOT AVAILABLE** - Twelvedata does not provide derivatives-specific data:

- [ ] Open Interest (total, by exchange)
- [ ] Funding Rates (current, historical)
- [ ] Liquidations (real-time events)
- [ ] Long/Short Ratios
- [ ] Mark Price
- [ ] Index Price
- [ ] Basis (futures - spot spread)

**Note**: Twelvedata covers spot crypto prices but not derivatives metrics like funding rates or liquidations.

## Options Data

**LIMITED/NOT AVAILABLE** - No explicit options data endpoints documented:

- [ ] Options Chains (strikes, expirations)
- [ ] Implied Volatility
- [ ] Greeks (delta, gamma, theta, vega)
- [ ] Open Interest (per strike)
- [ ] Historical option prices

**Note**: Twelvedata focuses on equities, forex, and crypto spot data, not options.

## Fundamental Data (Stocks)

### Company Information
- [x] **Company Profile** - Name, sector, industry, description via `/profile` (Grow+ plan)
  - CEO, headquarters, employees
  - Sector/industry classification
  - Website, phone, address
- [x] **Logo** - Company logo via `/logo` (1 credit, all plans)

### Financial Statements (Grow+ plan required)
- [x] **Income Statement** - Revenue, expenses, net income via `/income_statement`
  - Annual and quarterly data
  - Historical back to 1980s-1990s for major companies
  - Line items: revenue, COGS, gross profit, operating income, net income, EPS
- [x] **Balance Sheet** - Assets, liabilities, equity via `/balance_sheet`
  - Current and historical snapshots
  - Total assets, total liabilities, shareholder equity
  - Working capital, debt levels
- [x] **Cash Flow Statement** - Operating, investing, financing via `/cash_flow`
  - Operating cash flow
  - CapEx (capital expenditures)
  - Free cash flow
  - Financing activities

### Earnings
- [x] **Earnings History** - EPS actual, EPS estimate via `/earnings` (20 credits)
  - Quarterly and annual earnings
  - Surprise % (actual vs estimate)
  - Historical earnings back several years
- [x] **Earnings Calendar** - Upcoming earnings dates via `/earnings_calendar`
  - Scheduled earnings release dates
  - Estimated EPS

### Dividends
- [x] **Dividend History** - Payment dates, amounts via `/dividends` (20 credits)
  - Ex-dividend date
  - Payment date
  - Dividend amount
  - Yield calculation
  - Frequency (quarterly, annual, etc.)
- [x] **Dividends Calendar** - Upcoming dividend dates via `/dividends_calendar`

### Corporate Actions
- [x] **Stock Splits** - Historical splits via `/splits`
  - Split ratio (e.g., 2-for-1)
  - Split date
  - Adjusted price calculations
- [x] **Splits Calendar** - Upcoming splits via `/splits_calendar`
- [x] **IPO Calendar** - Upcoming IPOs via `/ipo_calendar`
  - IPO date
  - Price range
  - Exchange

### Valuation & Metrics
- [x] **Financial Ratios** - P/E, P/B, ROE, debt/equity via `/statistics`
  - Price-to-Earnings (P/E)
  - Price-to-Book (P/B)
  - Return on Equity (ROE)
  - Debt-to-Equity
  - Current Ratio
  - Quick Ratio
- [x] **Valuation Metrics** - Via `/statistics`
  - Market capitalization
  - Enterprise value
  - EV/EBITDA
  - PEG ratio
- [x] **Market Cap** - Current market capitalization via `/market_cap`
- [x] **52-week High/Low** - Via `/quote` endpoint
  - 52-week high price
  - 52-week low price
  - % from 52-week high/low

### Analyst Coverage
- [x] **Analyst Ratings** - Buy/sell/hold recommendations via `/recommendations` (high demand)
  - Strong buy, buy, hold, sell, strong sell counts
  - Consensus rating
  - Historical rating changes
- [x] **Price Targets** - Analyst price targets via `/price_target` (high demand)
  - Average target price
  - High/low targets
  - Number of analysts
- [x] **Earnings Estimates** - Analyst EPS forecasts via `/earning_estimate`
  - Current quarter estimate
  - Current year estimate
  - Next quarter/year estimates
- [x] **Revenue Estimates** - Revenue forecasts via `/revenue_estimate`
- [x] **EPS Trends** - Earnings per share trends via `/eps_trend`
- [x] **EPS Revisions** - Estimate changes via `/eps_revisions`
  - Upward revisions
  - Downward revisions
- [x] **Growth Estimates** - Future growth projections via `/growth_estimates`
- [x] **Analyst Ratings Snapshot** - Consensus summary via `/analyst_ratings_snapshot`

### Ownership & Shareholders
- [x] **Insider Trading** - Insider buys/sells via `/insider-transactions`
  - Transaction date
  - Insider name and title
  - Transaction type (buy/sell)
  - Shares and value
- [x] **Institutional Holdings** - Major shareholders via `/institutional-holders`
  - Institution name
  - Shares held
  - % ownership
  - Value
- [x] **Fund Holdings** - Mutual fund ownership via `/fund-holders`
- [x] **Direct Holdings** - Direct shareholders via `/direct-holders`

### Management
- [x] **Key Executives** - Management team via `/key_executives`
  - Name, title, compensation
  - CEO, CFO, COO, etc.

### Regulatory & Tax
- [x] **SEC Filings (EDGAR)** - 10-K, 10-Q, 8-K via `/edgar-filings-archive` (Grow+ plan)
  - Filing type
  - Filing date
  - Document links
- [x] **Tax Information** - Tax details via `/tax_info`
- [x] **Sanctioned Entities** - Restricted entities via `/sanctioned_entities`

### Additional
- [x] **Last Changes** - Recent data updates via `/last_changes`

## On-chain Data (Crypto)

**NOT AVAILABLE** - Twelvedata provides crypto price data only, not on-chain metrics:

- [ ] Wallet Balances
- [ ] Transaction History
- [ ] DEX Trades (Uniswap, PancakeSwap, etc.)
- [ ] Token Transfers (ERC-20, BEP-20)
- [ ] Smart Contract Events
- [ ] Gas Prices
- [ ] Block Data
- [ ] NFT Data

**Note**: For on-chain data, use specialized providers like Etherscan, Bitquery, or Dune Analytics.

## Macro/Economic Data

**NOT AVAILABLE** - Twelvedata does not provide macroeconomic data:

- [ ] Interest Rates (Fed Funds, Treasury yields)
- [ ] GDP (quarterly, annual)
- [ ] Inflation (CPI, PPI, PCE)
- [ ] Employment (NFP, unemployment rate, claims)
- [ ] Retail Sales
- [ ] Industrial Production
- [ ] Consumer Confidence
- [ ] PMI (Manufacturing, Services)
- [ ] Economic Calendar (upcoming releases)

**Note**: For economic data, use FRED (Federal Reserve Economic Data) or specialized economic data providers.

## Forex Specific

- [x] **Currency Pairs** - 200+ pairs (majors, minors, exotics) via `/forex_pairs`
  - Major pairs: EUR/USD, GBP/USD, USD/JPY, USD/CHF, AUD/USD, USD/CAD, NZD/USD
  - Minor pairs: EUR/GBP, EUR/JPY, GBP/JPY, etc.
  - Exotic pairs: USD/TRY, EUR/TRY, USD/ZAR, etc.
- [x] **Bid/Ask Spreads** - Real-time bid/ask via `/quote`
- [x] **Pip precision** - Configurable decimal precision (0-11) via `dp` parameter
- [x] **Cross rates** - Calculate exotic pairs on-the-fly via `/time_series/cross` (5 credits)
  - Automatically calculates cross rates for pairs not directly available
- [x] **Historical FX rates** - Decades of history via `/time_series`
- [x] **Exchange Rates** - Current conversion rates via `/exchange_rate`
- [x] **Currency Conversion** - Convert amounts via `/currency_conversion`

## Crypto Specific

- [x] **Crypto Pairs** - Thousands of pairs across 180+ exchanges via `/cryptocurrencies`
  - BTC/USD, ETH/USD, ETH/BTC, etc.
  - Spot pairs only (no futures/perpetuals)
- [x] **Exchange Coverage** - 180+ crypto exchanges via `/cryptocurrency_exchanges`
  - Binance, Coinbase, Kraken, Bitfinex, Huobi, etc.
- [x] **Historical Crypto Prices** - Back to coin inception
- [x] **24h Stats** - Volume, high, low, change via `/quote`
- [ ] **Funding Rates** - NOT AVAILABLE (see derivatives section)
- [ ] **Liquidations** - NOT AVAILABLE

## ETF Specific

- [x] **ETF Catalog** - List of ETFs via `/etf`
  - Symbol, name, currency, exchange
  - ISIN, CUSIP, FIGI identifiers
- [x] **ETF Comprehensive Data** - All ETF info via `/etf-all-data` (high demand)
- [x] **ETF Performance** - Returns, volatility via `/etf-performance` (high demand)
  - 1-year, 3-year, 5-year returns
  - Volatility metrics
  - Sharpe ratio
- [x] **ETF Composition** - Holdings breakdown via `/etf-composition` (high demand)
  - Top holdings (stocks/bonds)
  - Sector allocation
  - Geographic allocation
  - % of assets in top 10
- [x] **ETF Families** - Fund families via `/etf-family-list`
  - Vanguard, iShares (BlackRock), SPDR, etc.
- [x] **ETF Types** - Classification via `/etf-type-list`
  - Equity, bond, commodity, currency, etc.

## Mutual Fund Specific

- [x] **Mutual Fund Catalog** - List of funds via `/funds`
- [x] **Fund Ratings** - Morningstar-style ratings via `/mf-ratings`
- [x] **Purchase Info** - Minimums, fees via `/mf-purchase-info`
  - Minimum investment
  - Expense ratio
  - Front-end load, back-end load
- [x] **Sustainability Metrics** - ESG scores via `/mf-sustainability`
  - Environmental score
  - Social score
  - Governance score

## Commodities

- [x] **Commodity Pairs** - Metals, energy, agriculture via `/commodities`
  - **Metals**: Gold (XAU/USD), Silver (XAG/USD), Platinum, Palladium
  - **Energy**: Crude Oil (WTI, Brent), Natural Gas
  - **Agriculture**: Corn, Wheat, Soybeans, Coffee, Sugar, Cotton
- [x] **Commodity Time Series** - Historical prices via `/time_series`
- [x] **Commodity Quotes** - Real-time prices via `/quote`

## Bonds

- [x] **Bond Catalog** - List of bonds via `/bonds`
  - Fixed income instruments
  - Government and corporate bonds
- [x] **Bond Prices** - Via `/time_series` and `/quote` (limited data)

**Note**: Bond data may be limited compared to stocks/forex/crypto.

## Indices

- [x] **Global Indices** - Major indices worldwide
  - **US**: S&P 500 (SPX), Nasdaq 100 (NDX), Dow Jones (DJI), Russell 2000 (RUT)
  - **International**: FTSE 100 (FTSE), DAX (GDAXI), Nikkei 225 (N225), Hang Seng (HSI)
  - **Crypto**: BTC Dominance, Crypto Total Market Cap
- [x] **Index Time Series** - Historical index values via `/time_series`
- [x] **Index Quotes** - Real-time index levels via `/quote`

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - Comprehensive catalogs via `/stocks`, `/forex_pairs`, `/cryptocurrencies`, `/etf`, `/funds`, `/commodities`, `/bonds`
  - Updated daily every 3 hours starting from 12 AM
- [x] **Exchange Information** - Name, MIC code, country, timezone via `/exchanges`
  - All US exchanges
  - 90+ international stock exchanges
  - 180+ crypto exchanges
- [x] **Market Hours** - Trading schedule via `/exchange_schedule` (Ultra+ plan, 100 credits)
  - Regular trading hours
  - Pre-market hours (US stocks, Pro+ plan)
  - After-hours (US stocks, Pro+ plan)
  - Session times
- [x] **Trading Calendars** - Market holidays, half-days
  - Market state (open/closed) via `/market_state`
  - Time to open/close
- [x] **Timezone Info** - Exchange timezones via `/exchanges`
  - IANA timezone identifiers
  - Local exchange time, UTC conversion
- [x] **Sector/Industry Classifications** - Via `/profile` endpoint
  - GICS sectors
  - Industry groups
- [x] **Instrument Types** - Available asset classes via `/instrument_type`
  - Common Stock, Preferred Stock, ETF, Mutual Fund, Index, etc.
- [x] **Countries** - ISO codes, capitals, currencies via `/countries`
- [x] **Symbol Search** - Search by ticker, ISIN, FIGI, Composite FIGI via `/symbol_search`
- [x] **Cross-Listings** - All exchanges where security trades via `/cross_listings` (Grow+ plan, 40 credits)
- [x] **Earliest Timestamp** - Data availability start date via `/earliest_timestamp`

## Technical Indicators (100+ Available)

### Overlap Studies
- [x] **Bollinger Bands (BBANDS)** - Upper, middle, lower bands (high demand)
- [x] **Percent B** - Position within Bollinger Bands (high demand)
- [x] **Exponential Moving Average (EMA)** - Various periods (high demand)
- [x] **Simple Moving Average (SMA)** - Various periods (high demand)
- [x] **Weighted Moving Average (WMA)**
- [x] **Double EMA (DEMA)** - Faster response
- [x] **Triple EMA (TEMA)** - Even faster
- [x] **MESA Adaptive MA (MAMA)** - Adaptive period
- [x] **Kaufman Adaptive MA (KAMA)** - Volatility adaptive
- [x] **Volume Weighted MA (VWMA)**

### Momentum Indicators
- [x] **Relative Strength Index (RSI)** - Overbought/oversold (high demand)
- [x] **MACD** - MACD, signal, histogram (high demand)
- [x] **Stochastic Oscillator (STOCH)** - %K, %D lines (high demand)
- [x] **Stochastic RSI (STOCHRSI)** - RSI-based stochastic
- [x] **Commodity Channel Index (CCI)** - Cycle identification
- [x] **Average Directional Index (ADX)** - Trend strength (high demand)
- [x] **Williams %R** - Overbought/oversold
- [x] **Rate of Change (ROC)** - Momentum measurement
- [x] **Momentum (MOM)** - Price momentum
- [x] **Percentage Price Oscillator (PPO)** - MACD variant
- [x] **TRIX** - Triple smoothed EMA
- [x] **Ultimate Oscillator (ULTOSC)** - Multi-period momentum

### Volume Indicators
- [x] **On Balance Volume (OBV)** - Cumulative volume flow
- [x] **Accumulation/Distribution (AD)** - Money flow indicator
- [x] **AD Oscillator (ADOSC)** - AD momentum
- [x] **Money Flow Index (MFI)** - Volume-weighted RSI

### Volatility Indicators
- [x] **Average True Range (ATR)** - Volatility measurement
- [x] **Normalized ATR (NATR)** - ATR as percentage
- [x] **True Range (TR)** - Single-period range
- [x] **Standard Deviation (STDDEV)** - Price volatility

### Price Transforms
- [x] **Average Price (AVGPRICE)** - (O+H+L+C)/4
- [x] **Median Price (MEDPRICE)** - (H+L)/2
- [x] **Typical Price (TYPPRICE)** - (H+L+C)/3
- [x] **Weighted Close Price (WCLPRICE)** - (H+L+C+C)/4

### Other Indicators
- [x] **Parabolic SAR (SAR)** - Stop and reverse
- [x] **SuperTrend** - Trend following

**All indicators support**:
- Customizable time periods
- Multiple series types (open, high, low, close)
- Various intervals
- Historical calculations (up to 5000 data points)

## News & Sentiment

**NOT AVAILABLE** - Twelvedata does not provide:

- [ ] News Articles
- [ ] Press Releases
- [ ] Social Sentiment
- [ ] Analyst Reports (only ratings/estimates, not full reports)

**Note**: For news/sentiment, use specialized providers like Benzinga, Finnhub, or NewsAPI.

## Unique/Custom Data

### What makes Twelvedata special:

1. **Multi-Asset Unified API**
   - Single API for stocks, forex, crypto, ETFs, commodities, bonds, indices
   - Consistent interface across all asset types
   - Same technical indicators work on all assets

2. **Extensive Historical Fundamentals**
   - Financial statements back to 1980s-1990s for major companies
   - Decades of earnings and dividend history
   - Long-term financial ratios

3. **100+ Technical Indicators**
   - Comprehensive indicator library
   - All indicators work on all assets
   - Customizable parameters

4. **Cross-Rate Calculation**
   - On-the-fly exotic pair calculation (5 credits)
   - No need for multiple API calls
   - Accurate cross-rate time series

5. **Extended Hours Data** (Pro+ plans)
   - US pre-market: 4 AM - 9:30 AM ET
   - US after-hours: 4 PM - 8 PM PT
   - Full trading day coverage

6. **Flexible Output Formats**
   - JSON (default)
   - CSV with configurable delimiters
   - Pandas DataFrame (Python SDK)

7. **Market State API**
   - Real-time open/closed status
   - Time-to-open/time-to-close calculations
   - Works across all global exchanges

8. **FIGI/ISIN/CUSIP Support** (Ultra+ plans)
   - Financial Instrument Global Identifier (FIGI)
   - Composite FIGI for multi-exchange instruments
   - ISIN and CUSIP identifiers
   - Symbol search supports all identifier types

9. **Batch Efficiency**
   - 120 symbols per request
   - 1 credit per 100 symbols (vs 1 credit per symbol)
   - Significant cost savings for multi-symbol queries

10. **WebSocket Real-Time** (Pro+ plans)
    - ~170ms average latency
    - Multi-asset streaming in single connection
    - Stocks, forex, crypto simultaneously

## Data NOT Available (Compared to Specialized Providers)

**Compared to Crypto-Specific Providers** (e.g., Coinglass, Glassnode):
- [ ] Liquidation heatmaps
- [ ] Funding rate aggregations
- [ ] Long/short ratios
- [ ] Open interest
- [ ] On-chain metrics (wallet balances, transactions, etc.)

**Compared to Options Data Providers** (e.g., CBOE, TastyWorks):
- [ ] Options chains
- [ ] Implied volatility surfaces
- [ ] Greeks (delta, gamma, theta, vega)
- [ ] Options flow/volume

**Compared to Economic Data Providers** (e.g., FRED):
- [ ] Macroeconomic indicators (GDP, CPI, unemployment)
- [ ] Central bank rates
- [ ] Economic calendars

**Compared to News/Sentiment Providers** (e.g., Benzinga, Finnhub):
- [ ] Real-time news articles
- [ ] Social sentiment analysis
- [ ] Press releases

**Compared to Crypto On-Chain Providers** (e.g., Etherscan, Bitquery):
- [ ] Blockchain transaction data
- [ ] Smart contract events
- [ ] DEX trade data
- [ ] Token transfers

## Coverage Summary

| Asset Type | Symbols | Historical Depth | Real-time | Fundamentals |
|------------|---------|------------------|-----------|--------------|
| **US Stocks** | ~10,000+ | Decades | Yes (Pro+) | Extensive |
| **International Stocks** | ~50,000+ (90+ exchanges) | Varies | Yes (Pro+) | Limited |
| **Forex** | 200+ pairs | Decades | Yes (Pro+) | N/A |
| **Crypto** | Thousands (180+ exchanges) | Since inception | Yes (Pro+) | N/A |
| **ETFs** | ~5,000+ | Decades | Yes (Pro+) | Yes |
| **Mutual Funds** | ~20,000+ | Varies | Daily | Yes |
| **Commodities** | 50+ | Decades | Yes | N/A |
| **Bonds** | Limited | Varies | Limited | Limited |
| **Indices** | 100+ global | Decades | Yes | N/A |

## Data Quality Notes

1. **Null values expected**: Some fields may be null when data unavailable - defensive programming required
2. **Catalog updates**: Daily (every 3 hours from 12 AM)
3. **Real-time latency**: ~170ms (WebSocket), minutely (REST polling)
4. **Adjustments**: Price data adjustable for splits/dividends
5. **Decimal precision**: Configurable 0-11 decimal places
6. **Timezone support**: Exchange local, UTC, or IANA identifiers
