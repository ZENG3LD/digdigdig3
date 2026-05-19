# Tiingo - Data Types Catalog

## Standard Market Data

### Stocks (US & International)
- [x] **Current Price** - Real-time via IEX WebSocket and REST
- [x] **Bid/Ask Spread** - Top-of-book via IEX WebSocket
- [x] **24h Ticker Stats** - Daily high, low, volume, change%
- [x] **OHLC/Candlesticks**
  - Intervals: 1min, 5min, 15min, 30min, 1hour, 4hour, daily, weekly, monthly, annually
  - Historical: 50+ years for daily, limited for intraday
  - Adjusted for splits and dividends
- [ ] **Level 2 Orderbook** - Not available (IEX provides top-of-book only)
- [x] **Recent Trades** - Real-time trade feed via WebSocket
- [x] **Volume** - Intraday and cumulative volume
- [x] **Pre-market/After-hours** - Available via IEX (afterHours parameter)

### Crypto
- [x] **Current Price** - Top-of-book via REST and WebSocket
- [x] **Bid/Ask Spread** - Top-of-book quotes
- [x] **24h Ticker Stats** - Daily metrics across 40+ exchanges
- [x] **OHLC/Candlesticks**
  - Intervals: 1min, 5min, 15min, 1hour, 1day, etc.
  - Historical depth varies by exchange/pair
  - Aggregated across 40+ exchanges
- [ ] **Level 2 Orderbook** - Not available (top-of-book only)
- [x] **Recent Trades** - Real-time updates via WebSocket
- [x] **Volume** - Per-exchange and aggregated volume

### Forex
- [x] **Current Price** - Top-of-book from tier-1 banks
- [x] **Bid/Ask Spread** - Institutional-grade quotes
- [x] **OHLC/Candlesticks**
  - Intervals: 1min, 5min, 15min, 30min, 1hour, 4hour, 1day
  - Historical FX data available
- [x] **Mid Price** - (bid + ask) / 2
- [ ] **Level 2 Orderbook** - Not available (FX is OTC, top-of-book only)

---

## Historical Data

### End-of-Day (EOD) Stock Data
- [x] **Historical prices** - **50+ years** for US stocks
- [x] **Daily bars** - OHLCV + adjusted prices
- [x] **Adjusted prices** - Split-adjusted, dividend-adjusted
  - adjOpen, adjHigh, adjLow, adjClose, adjVolume
  - Raw prices also available
  - divCash (dividend cash amount)
  - splitFactor (split ratio)
- [x] **Weekly/Monthly/Annual bars** - Resampled from daily data
- [x] **Symbol metadata** - Name, exchange, description, start/end dates

### Intraday Data (IEX)
- [x] **Minute bars** - 1min, 5min, 15min, 30min intervals
- [x] **Hourly bars** - 1hour, 4hour intervals
- [x] **Tick data** - Not available (aggregated bars only)
- [x] **Historical depth** - Limited (not specified, likely weeks to months)
- [x] **Real-time intraday** - Yes, via IEX REST and WebSocket

### Crypto Historical
- [x] **Historical prices** - Depth varies by exchange/pair
- [x] **Minute/hourly bars** - 1min, 5min, 15min, 1hour, etc.
- [x] **Daily bars** - Aggregated across exchanges
- [x] **Multi-exchange data** - 40+ exchanges, ticker-specific availability

### Forex Historical
- [x] **Historical FX rates** - Depth not specified (years of data likely)
- [x] **Intraday bars** - 1min, 5min, 15min, 30min, 1hour, 4hour
- [x] **Daily bars** - EOD FX rates
- [x] **Institutional-grade** - Direct from tier-1 banks and FX dark pools

---

## Derivatives Data (Crypto/Futures)

**Not explicitly available** - Tiingo focuses on spot markets.

- [ ] **Open Interest** - Not available
- [ ] **Funding Rates** - Not available
- [ ] **Liquidations** - Not available
- [ ] **Long/Short Ratios** - Not available
- [ ] **Mark Price** - Not available
- [ ] **Index Price** - Not available
- [ ] **Basis** - Not available

**Note**: Tiingo does not provide derivatives/futures analytics. For crypto derivatives, use specialized providers (e.g., Binance, Bybit, CoinGlass).

---

## Options Data

**Not available** - Tiingo does not provide options data.

- [ ] **Options Chains** - Not available
- [ ] **Implied Volatility** - Not available
- [ ] **Greeks** - Not available
- [ ] **Open Interest** - Not available
- [ ] **Historical option prices** - Not available

**Note**: For options data, use providers like CBOE, Tradier, or IEX Cloud (full version).

---

## Fundamental Data (Stocks)

Tiingo provides comprehensive fundamentals for **5,500+ equities** with **80+ indicators**.

### Company Profile
- [x] **Company Information**
  - Name, ticker symbol
  - Sector, industry classification
  - Description
  - Exchange, country
  - Start/end dates

### Financial Statements
- [x] **Income Statement**
  - Revenue, COGS, gross profit
  - Operating income, EBITDA, EBIT
  - Net income, EPS
  - Quarterly and annual periods
- [x] **Balance Sheet**
  - Total assets, liabilities, equity
  - Current assets/liabilities
  - Cash, debt, inventory
  - Quarterly and annual periods
- [x] **Cash Flow Statement**
  - Operating cash flow
  - Investing cash flow
  - Financing cash flow
  - Free cash flow
  - Quarterly and annual periods

### Earnings & Dividends
- [x] **Earnings** - EPS (reported and diluted), revenue, quarterly/annual
- [x] **Dividends** - Dividend history, yield, payout ratio
- [x] **Stock Splits** - Split history with splitFactor
- [ ] **Earnings guidance** - Not explicitly mentioned
- [ ] **Analyst estimates** - Not available

### Daily-Updated Metrics
- [x] **Market Capitalization** - Daily updates
- [x] **Enterprise Value** - Daily updates
- [x] **Valuation Ratios**
  - P/E ratio (price-to-earnings)
  - P/B ratio (price-to-book)
  - P/S ratio (price-to-sales)
  - EV/EBITDA, EV/Sales
- [x] **Profitability Metrics**
  - ROE (return on equity)
  - ROA (return on assets)
  - ROIC (return on invested capital)
  - Gross margin, operating margin, net margin
- [x] **Financial Health**
  - Debt/Equity ratio
  - Current ratio, quick ratio
  - Interest coverage
- [x] **Dividend Yield** - Updated daily
- [x] **80+ indicators total** - Comprehensive coverage

### Historical Fundamentals
- [x] **5 years** - Free tier
- [x] **15+ years** - Paid tiers
- [x] **Quarterly updates** - Financial statements
- [x] **Annual updates** - Financial statements
- [x] **Daily updates** - Market-based metrics (market cap, ratios)

### Statement Formats
- [x] **As-Reported** - Raw data from SEC filings
- [x] **Standardized** - Normalized format for comparisons
- [x] **Quarterly** - 10-Q filings
- [x] **Annual** - 10-K filings

### Coverage
- **5,500+ US equities**
- **80+ fundamental indicators**
- **20+ years of history** (varies by metric)
- **Quarterly and annual periods**

### Not Available
- [ ] **Analyst Ratings** - Not available
- [ ] **Insider Trading** - Not available
- [ ] **Institutional Holdings** - Not available
- [ ] **Short Interest** - Not available
- [ ] **Analyst Estimates** - Not available

---

## On-chain Data (Crypto)

**Not available** - Tiingo does not provide blockchain/on-chain analytics.

- [ ] **Wallet Balances** - Not available
- [ ] **Transaction History** - Not available
- [ ] **DEX Trades** - Not available
- [ ] **Token Transfers** - Not available
- [ ] **Smart Contract Events** - Not available
- [ ] **Gas Prices** - Not available
- [ ] **Block Data** - Not available
- [ ] **NFT Data** - Not available

**Note**: Tiingo focuses on centralized exchange crypto prices. For on-chain data, use providers like Bitquery, The Graph, or Etherscan.

---

## Macro/Economic Data

**Not available** - Tiingo does not provide economic/macro data.

- [ ] **Interest Rates** - Not available
- [ ] **GDP** - Not available
- [ ] **Inflation (CPI, PPI)** - Not available
- [ ] **Employment (NFP, unemployment)** - Not available
- [ ] **Retail Sales** - Not available
- [ ] **Industrial Production** - Not available
- [ ] **Consumer Confidence** - Not available
- [ ] **PMI** - Not available
- [ ] **Economic Calendar** - Not available

**Note**: For economic data, use FRED (Federal Reserve Economic Data), Alpha Vantage, or Trading Economics.

---

## Metadata & Reference

### Symbol Lists
- [x] **Stock Tickers** - List of supported tickers (32,000+ US equities, Chinese stocks)
- [x] **ETF Tickers** - 33,000+ ETFs and mutual funds
- [x] **Crypto Tickers** - 2,100-4,100+ crypto pairs from 40+ exchanges
- [x] **Forex Tickers** - 140+ currency pairs
- [x] **Bulk Download** - supported_tickers.zip from apimedia

### Exchange Information
- [x] **Exchange Names** - NYSE, NASDAQ, AMEX, etc.
- [x] **Currency** - Trading currency for each ticker
- [x] **Start/End Dates** - Data availability range per ticker
- [ ] **Market Hours** - Not explicitly provided
- [ ] **Trading Calendars** - Not explicitly provided
- [ ] **Holidays** - Not explicitly provided

### Sector/Industry Classifications
- [x] **Sector** - Sector classification for equities
- [x] **Industry** - Industry classification
- [ ] **GICS/NAICS codes** - Not explicitly mentioned

### Timezone Info
- [ ] **Not explicitly documented** - Likely UTC for timestamps

---

## News & Sentiment

### News Data
- [x] **Financial News Articles** - Curated news feed
- [x] **Ticker-specific news** - Filter by tickers
- [x] **Source filtering** - Filter by news source domains
- [x] **Tag/keyword search** - Filter by tags/keywords
- [x] **Date range filtering** - startDate, endDate
- [x] **Pagination** - limit, offset parameters
- [x] **Sorting** - Sort by publishedDate
- [x] **Bulk download** - Bulk news files available

### News Sources
- [x] **Curated sources** - Tiingo curates reputable financial news
- [x] **Source domains** - Filter by specific domains (e.g., washingtonpost.com)
- [ ] **Press releases** - Not explicitly separated
- [ ] **SEC filings** - Not included (use fundamentals API for 10-K/10-Q)

### Sentiment
- [ ] **Social Sentiment** - Not available
- [ ] **News Sentiment Scores** - Not available
- [ ] **Analyst Reports** - Not available

**Note**: Tiingo provides raw news articles but not sentiment analysis. For sentiment, use providers like Sentdex, StockTwits, or custom NLP.

---

## Unique/Custom Data

**What makes Tiingo special?**

### 1. IEX Real-time Intraday Data
- **Unique**: IEX exchange data at intraday frequency (free tier)
- **Benefit**: Real-time stock data without expensive exchange fees
- **Use Case**: Intraday trading, real-time dashboards, backtesting with intraday data

### 2. Multi-Asset Platform (Stocks + Crypto + Forex)
- **Unique**: Single API for stocks, crypto, AND forex
- **Benefit**: Unified authentication, consistent API design across asset classes
- **Use Case**: Multi-asset portfolios, cross-market analysis

### 3. Institutional-Grade Forex from Tier-1 Banks
- **Unique**: Direct connections to tier-1 banks and FX dark pools
- **Benefit**: High-quality FX quotes (not retail broker feeds)
- **Use Case**: Professional FX trading, arbitrage, institutional applications

### 4. 50+ Years of Stock Historical Data
- **Unique**: Extremely deep historical coverage (50+ years)
- **Benefit**: Long-term backtesting, research, historical analysis
- **Use Case**: Quantitative research, long-term strategies, academic studies

### 5. Comprehensive Fundamentals (80+ Indicators)
- **Unique**: 80+ fundamental indicators with 15+ years history
- **Benefit**: Deep fundamental analysis without multiple providers
- **Use Case**: Value investing, fundamental screeners, DCF models

### 6. WebSocket Firehose (Microsecond Resolution)
- **Unique**: Microsecond-resolution WebSocket firehose (free tier included)
- **Benefit**: Ultra-low-latency real-time data for HFT-adjacent applications
- **Use Case**: High-frequency strategies, market microstructure research

### 7. Adjusted Prices with Split/Dividend Data
- **Unique**: Both raw and adjusted prices with explicit divCash and splitFactor
- **Benefit**: Accurate backtesting, dividend-adjusted returns
- **Use Case**: Backtesting, portfolio analytics, dividend strategies

### 8. 40+ Crypto Exchanges Aggregated
- **Unique**: Aggregates data from 40+ crypto exchanges
- **Benefit**: Comprehensive crypto coverage, cross-exchange arbitrage
- **Use Case**: Crypto trading, arbitrage, market analysis

### 9. Transparent Pricing (No Hidden Fees)
- **Unique**: Clear tier structure, no surprise charges
- **Benefit**: Predictable costs, budget planning
- **Use Case**: Startups, cost-conscious developers

### 10. Free Tier with Real Production Features
- **Unique**: Free tier includes WebSocket, fundamentals, all data types
- **Benefit**: Full-featured testing without credit card
- **Use Case**: Learning, prototyping, personal projects

---

## Coverage Summary

| Data Type | Coverage | Historical Depth | Update Frequency | Free Tier |
|-----------|----------|------------------|------------------|-----------|
| **US Stocks (EOD)** | 32,000+ equities | 50+ years | Daily | Yes |
| **US Stocks (Intraday)** | IEX coverage | Limited (weeks/months) | Real-time | Yes (WebSocket) |
| **ETFs/Mutual Funds** | 33,000+ | 50+ years | Daily | Yes |
| **Chinese Stocks** | Included | Varies | Daily | Yes |
| **Crypto** | 2,100-4,100+ pairs | Varies by exchange | Real-time | Yes (WebSocket) |
| **Forex** | 140+ pairs | Years | Real-time | Yes (WebSocket) |
| **Fundamentals** | 5,500+ equities | 5-15+ years | Quarterly/Annual/Daily | 5yr (free), 15yr (paid) |
| **News** | Curated sources | Archives available | Real-time | Yes |

---

## Data Types NOT Available

### Missing from Tiingo:
1. **Options data** (chains, Greeks, IV)
2. **Derivatives analytics** (futures, perpetuals, funding rates)
3. **On-chain crypto data** (blockchain, DEX, wallets)
4. **Economic/macro data** (GDP, CPI, NFP, etc.)
5. **Analyst ratings/estimates**
6. **Insider trading data**
7. **Institutional holdings**
8. **Short interest**
9. **Social sentiment**
10. **Level 2 orderbook** (only top-of-book)

### Workarounds:
- **Options**: Use CBOE, Tradier, or IEX Cloud
- **Derivatives**: Use exchange APIs (Binance, Bybit) or CoinGlass
- **On-chain**: Use Bitquery, The Graph, or Etherscan
- **Economic**: Use FRED, Alpha Vantage, Trading Economics
- **Analyst data**: Use FactSet, Bloomberg, or Seeking Alpha
- **Sentiment**: Use Sentdex, StockTwits, or custom NLP
- **Level 2**: Use direct exchange feeds (paid)

---

## Recommended Use Cases

Tiingo is **best suited** for:
1. **Multi-asset portfolios** (stocks + crypto + forex)
2. **Intraday stock backtesting** (IEX real-time/historical)
3. **Fundamental analysis** (80+ indicators, 15+ years)
4. **Long-term historical analysis** (50+ years EOD data)
5. **Real-time dashboards** (WebSocket firehose)
6. **Crypto aggregation** (40+ exchanges in one API)
7. **Institutional-grade FX** (tier-1 bank quotes)
8. **Cost-effective prototyping** (generous free tier)
9. **Transparent pricing** (no hidden fees)
10. **Python/R quantitative research** (official SDKs)

Tiingo is **NOT ideal** for:
1. **Options trading** (no options data)
2. **Crypto derivatives** (no futures/perpetuals analytics)
3. **On-chain analysis** (no blockchain data)
4. **Economic forecasting** (no macro data)
5. **Analyst-driven strategies** (no ratings/estimates)
6. **Ultra-HFT** (WebSocket is fast but not direct exchange feed)
7. **Level 2 orderbook strategies** (only top-of-book)

---

## Data Quality Notes

- **Adjusted prices**: High quality, explicit split/dividend tracking
- **Fundamentals**: Sourced from SEC filings (10-K, 10-Q)
- **IEX data**: Direct from IEX exchange (reputable source)
- **Forex**: Tier-1 banks and FX dark pools (institutional-grade)
- **Crypto**: Aggregated from 40+ exchanges (may have exchange-specific gaps)
- **News**: Curated sources (quality over quantity)
- **Historical depth**: Excellent for stocks (50+ years), varies for crypto/forex
