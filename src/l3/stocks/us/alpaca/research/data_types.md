# Alpaca - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Latest trade price (REST `/latest/trades`, WebSocket `trades` channel)
- [x] **Bid/Ask Spread** - Latest quote (REST `/latest/quotes`, WebSocket `quotes` channel)
- [x] **24h Ticker Stats** - Not directly (must calculate from bars/trades)
- [x] **OHLC/Candlesticks** - Comprehensive (REST `/bars`, WebSocket `bars`, `dailyBars`)
  - Intervals: 1Min, 5Min, 15Min, 30Min, 1Hour, 4Hour, 1Day, 1Week
  - Custom timeframes supported via SDK
- [x] **Level 2 Orderbook** - Crypto only (REST `/latest/orderbooks`, WebSocket `orderbooks`)
  - **Not available for stocks** (no orderbook depth data)
- [x] **Recent Trades** - Historical and real-time (REST `/trades`, WebSocket `trades`)
- [x] **Volume** - Included in bars (OHLCV), trades, and snapshots
- [x] **VWAP** - Volume-weighted average price in bar data (`vw` field)
- [x] **Trade Count** - Number of trades per bar (`n` field)

## Historical Data

- [x] **Historical prices** - Depth: **7+ years** (stocks), **6+ years** (crypto)
  - Stocks: Back to 2016 minimum
  - Daily bars: 7+ years available
- [x] **Minute bars** - Available back to **5 years** (2016+)
  - Timeframes: 1Min, 5Min, 15Min, 30Min supported
- [x] **Daily bars** - Depth: **7+ years**
- [ ] **Tick data** - **NOT available** (only aggregated bars, no raw tick-by-tick)
- [x] **Adjusted prices** - Splits and dividends adjustments supported
  - Adjustment parameter: `raw`, `split`, `dividend`, `all`
  - Corporate actions API provides split/dividend data

## Derivatives Data (Futures/Perpetuals)

**NOT APPLICABLE** - Alpaca is a stock/options/crypto broker, not a derivatives exchange.

- [ ] Open Interest - Not available (stocks don't have OI)
- [ ] Funding Rates - Not available (no perpetual futures)
- [ ] Liquidations - Not available (no futures)
- [ ] Long/Short Ratios - Not available
- [ ] Mark Price - Not available (stocks use market price)
- [ ] Index Price - Not available (but index data available via stock symbols like SPY)
- [ ] Basis - Not available (no futures-spot spread)

**Note:** Options data available (see Options Data section below)

## Options Data

- [x] **Options Chains** - Strikes, expirations, contract symbols
  - Endpoint: `/v1beta1/options/snapshots/{underlying_symbol}`
  - Filter by underlying symbol
- [x] **Implied Volatility** - Latest IV per contract in chain
- [x] **Greeks** - Full Greeks support
  - Delta, Gamma, Theta, Vega, Rho
  - Real-time (paid tier) or indicative (free tier)
- [x] **Open Interest** - Per strike (if available in OPRA feed)
- [x] **Historical option prices** - Bars, trades, quotes
  - `/v1beta1/options/bars` - Historical OHLCV
  - `/v1beta1/options/trades` - Trade executions
  - `/v1beta1/options/quotes` - Bid/ask history
- [x] **Latest option data** - Snapshots include latest trade and quote
- [x] **Options contracts list** - `/v2/option_contracts` endpoint

**Tiers:**
- **Free tier:** Indicative data (delayed, not real-time Greeks)
- **Paid tier ($99/mo):** Real-time OPRA feed with accurate Greeks and IV

## Fundamental Data (Stocks)

**LIMITED** - Alpaca focuses on trading and market data, not fundamentals.

- [x] **Company Profile** - Basic info via assets endpoint (name, symbol, exchange)
- [ ] **Financial Statements** - **NOT available** (no income statement, balance sheet, cash flow)
- [ ] **Earnings** - **NOT available** (no EPS, revenue, guidance data)
- [x] **Dividends** - Corporate actions API includes dividend announcements
  - Dividend amount, ex-date, record date, payable date
- [x] **Stock Splits** - Corporate actions API includes split data
  - Split ratio, ex-date, record date
- [ ] **Analyst Ratings** - **NOT available**
- [ ] **Insider Trading** - **NOT available**
- [ ] **Institutional Holdings** - **NOT available**
- [ ] **Financial Ratios** - **NOT available** (no P/E, P/B, ROE, debt/equity, etc.)
- [ ] **Valuation Metrics** - **NOT available**

**Workaround:** Use third-party fundamental data providers (Polygon, Alpha Vantage, etc.)

## On-chain Data (Crypto)

**NOT APPLICABLE** - Alpaca provides crypto trading and price data, not on-chain analytics.

- [ ] Wallet Balances - Not available (not a blockchain explorer)
- [ ] Transaction History - Not available (only trade history on Alpaca/Kraken)
- [ ] DEX Trades - Not available (only CEX data from Alpaca and Kraken)
- [ ] Token Transfers - Not available
- [ ] Smart Contract Events - Not available
- [ ] Gas Prices - Not available
- [ ] Block Data - Not available
- [ ] NFT Data - Not available

**Crypto data available:**
- Spot trading prices (Alpaca + Kraken exchanges)
- Historical bars, trades, quotes
- Orderbook depth (REST and WebSocket)
- 6+ years historical data

**Workaround:** Use blockchain-specific APIs (Etherscan, Bitquery, Covalent, etc.)

## Macro/Economic Data

**NOT APPLICABLE** - Alpaca does not provide economic data.

- [ ] Interest Rates - Not available
- [ ] GDP - Not available
- [ ] Inflation (CPI, PPI, PCE) - Not available
- [ ] Employment (NFP, unemployment) - Not available
- [ ] Retail Sales - Not available
- [ ] Industrial Production - Not available
- [ ] Consumer Confidence - Not available
- [ ] PMI - Not available
- [ ] Economic Calendar - Not available

**Workaround:** Use FRED API, Trading Economics, Alpha Vantage, etc.

## Forex Specific

**LIMITED** - Alpaca has basic forex rates, not full FX trading.

- [x] **Currency Pairs** - Basic forex rates via `/v1beta1/forex/rates`
- [x] **Bid/Ask Spreads** - Not explicitly available
- [ ] **Pip precision** - Not specified
- [x] **Cross rates** - Calculated rates available
- [x] **Historical FX rates** - Limited historical data

**Note:** Alpaca is primarily a US stock/options/crypto broker. For serious FX trading, use dedicated forex brokers (Oanda, FXCM, etc.)

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - `/v2/assets` endpoint
  - Filter by asset class (us_equity, crypto)
  - Filter by exchange (NYSE, NASDAQ, AMEX, etc.)
  - Filter by status (active, inactive)
  - Fields: symbol, name, tradable, marginable, shortable, fractionable, options_enabled
- [x] **Exchange Information** - Asset data includes exchange field
  - US equity exchanges: NYSE, NASDAQ, AMEX, ARCA, BATS
  - Crypto: Alpaca, Kraken
- [x] **Market Hours** - `/v2/clock` endpoint
  - Current market status (open/closed)
  - Next open/close times
  - Timestamp in market timezone
- [x] **Trading Calendars** - `/v2/calendar` endpoint
  - Market days from 1970 to 2029
  - Early closures (e.g., day before Thanksgiving)
  - Specific open/close times per day
- [x] **Timezone Info** - All timestamps in UTC (RFC-3339 format)
  - Market hours endpoint provides timezone-aware data
- [ ] **Sector/Industry Classifications** - **NOT available** (assets don't include sector/industry)

**Workaround for sectors:** Use third-party APIs (Polygon, Alpha Vantage, Yahoo Finance)

## News & Sentiment

- [x] **News Articles** - `/v1beta1/news` endpoint
  - Filter by symbol(s)
  - Filter by date range
  - Sources: Benzinga, others
  - Fields: headline, author, summary, content, url, images, symbols
  - Limit 1-50 articles per request
  - Pagination supported
- [ ] **Press Releases** - Included in news feed (not separate)
- [ ] **Social Sentiment** - **NOT available** (no Twitter/Reddit sentiment analysis)
- [ ] **Analyst Reports** - **NOT available**

**Real-time news:**
- WebSocket stream available for real-time news updates
- Same sources as REST API

## Unique/Custom Data

**What makes Alpaca special:**

### 1. **Integrated Trading + Data Platform**
- Single API for both market data and trading execution
- Paper trading with real-time data (free)
- Commission-free trading (stocks, ETFs, options, crypto)

### 2. **Extended Hours Trading Data**
- **BOATS Feed:** Blue Ocean ATS for extended evening hours
- **Overnight Feed:** 15-minute delayed extended hours data (free tier)
- Pre-market and after-hours data included

### 3. **Fractional Shares Support**
- Trade as little as $1 worth of 2,000+ US equities
- Fractional data in positions/account endpoints
- Market and day orders only (no limit orders on fractionals)

### 4. **Options Trading Up to Level 3**
- Complex options strategies (multi-leg orders)
- Up to 4 legs per order
- Greeks and IV data (real-time on paid tier)

### 5. **Corporate Actions Integration**
- Dividends: Declaration, ex-date, record, payable dates
- Stock splits: Forward and reverse splits with ratios
- Mergers and spinoffs
- Automatic position adjustments

### 6. **Screener Endpoint**
- `/v1beta1/screener/stocks/movers`
- Top gainers, losers, most active stocks
- Quick discovery of market movers

### 7. **Logos API**
- `/v1beta1/logos/{symbol}`
- Company logo URLs for UI integration
- Useful for building trading interfaces

### 8. **Multi-Asset Support**
- Stocks: US equities and ETFs
- Options: Up to Level 3 strategies
- Crypto: 24/7 spot trading
- Forex: Basic currency rates (limited)

### 9. **Real-time WebSocket Streams**
- **Market Data:** Trades, quotes, bars (minute and daily), orderbook (crypto)
- **Trading Updates:** Real-time order fills, cancellations, position changes
- **Advanced Channels:** Trading status (halts), LULD bands, trade corrections, imbalances

### 10. **Developer-First Platform**
- API-first design (no manual trading interface required)
- Comprehensive SDKs (Python, JavaScript, community Rust)
- Paper trading environment identical to live
- OAuth2 for third-party integrations

### 11. **Account/Portfolio Data**
- Real-time account value (equity, cash, buying power)
- Portfolio history endpoint (equity curve over time)
- Account activities log (all transactions)
- Position tracking with unrealized P&L

### 12. **Test Stream**
- 24/7 test WebSocket stream (`wss://stream.data.alpaca.markets/v2/test`)
- Use symbol "FAKEPACA" for testing
- No need to wait for market hours

### 13. **Fixed Income & Forex**
- `/v1beta1/fixed-income` - Bond pricing data (limited documentation)
- `/v1beta1/forex/rates` - Currency exchange rates

## Data NOT Available on Alpaca

**Avoid using Alpaca for:**

1. **Fundamental analysis** - No financial statements, earnings, ratios
2. **Sentiment analysis** - No social media sentiment, analyst opinions
3. **Economic data** - No GDP, CPI, employment data
4. **On-chain crypto** - No DEX data, wallet tracking, gas prices
5. **Futures/Derivatives** - No futures, perpetuals, funding rates (options only)
6. **International stocks** - US markets only (no European, Asian stocks)
7. **Level 2 orderbook for stocks** - Only crypto has orderbook depth
8. **Tick-by-tick data** - Only aggregated bars (no raw tick data)
9. **Sector/Industry data** - Assets don't include classifications

## Data Coverage Summary

| Data Type | Available? | Quality | Tiers |
|-----------|-----------|---------|-------|
| **US Stocks** | ✅ Excellent | 7+ years, IEX (free) / SIP (paid) | Free: IEX, Paid: All exchanges |
| **US Options** | ✅ Good | Indicative (free) / Real-time OPRA (paid) | Requires paid for real Greeks |
| **Crypto** | ✅ Good | 6+ years, Alpaca + Kraken | Same on free and paid |
| **News** | ✅ Good | Real-time, Benzinga + others | Free and paid |
| **Corporate Actions** | ✅ Good | Dividends, splits, mergers | Free and paid |
| **Extended Hours** | ✅ Good | BOATS feed (paid), 15-min delay (free) | Paid better |
| **Forex** | ⚠️ Limited | Basic rates only | Not a focus |
| **Fixed Income** | ⚠️ Limited | Minimal docs | Not a focus |
| **Fundamentals** | ❌ None | Use third-party API | Not available |
| **Sentiment** | ❌ None | Use third-party API | Not available |
| **Economic Data** | ❌ None | Use FRED, etc. | Not available |
| **Futures** | ❌ None | Use CME, etc. | Not available |
| **International Stocks** | ❌ None | Use IEX Cloud, etc. | Not available |

## Recommended Use Cases

**Alpaca is BEST for:**
- ✅ US stock algorithmic trading
- ✅ Options trading strategies
- ✅ Crypto spot trading (24/7)
- ✅ Paper trading / backtesting
- ✅ Real-time market data streaming
- ✅ Developer-first API integration
- ✅ Commission-free trading

**Alpaca is NOT ideal for:**
- ❌ Fundamental stock analysis (missing financials)
- ❌ Economic/macro trading (no economic data)
- ❌ International equity markets
- ❌ Futures/perpetuals trading
- ❌ Deep market microstructure (no L2 orderbook for stocks)
- ❌ Sentiment-driven strategies (no sentiment data)
