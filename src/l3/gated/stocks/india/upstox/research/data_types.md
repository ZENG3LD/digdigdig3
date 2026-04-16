# Upstox - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - LTP (Last Traded Price) endpoint, real-time
- [x] **Bid/Ask Spread** - Full market depth with bid/ask prices
- [x] **24h Ticker Stats** - High, low, volume, change%, open, close
- [x] **OHLC/Candlesticks** - Multiple intervals:
  - Minutes: 1-300 (configurable, e.g., 1, 3, 5, 10, 15, 30, 60)
  - Hours: 1-5
  - Days: 1
  - Weeks: 1
  - Months: 1
- [x] **Level 2 Orderbook** - Market depth (bids/asks)
  - Standard: 5 levels
  - Upstox Plus: 30 levels (Full D30 mode)
- [x] **Recent Trades** - Trade history for the day
- [x] **Volume** - 24h volume, total buy/sell quantities
- [x] **Open Interest** - For F&O instruments, OI day high/low

---

## Historical Data

- [x] **Historical prices** - Depth:
  - Daily bars: From year 2000 (24+ years)
  - Intraday bars: From January 2022
- [x] **Minute bars** - Available: Yes
  - 1-300 minute intervals
  - From January 2022
  - Max 1 month for 1-15 min intervals
  - Max 1 quarter for >15 min intervals
- [x] **Hourly bars** - Available: Yes
  - 1-5 hour intervals
  - From January 2022
  - Max 1 quarter historical depth
- [x] **Daily bars** - Available: Yes
  - From year 2000
  - Max 1 decade per request
- [x] **Weekly bars** - Available: Yes
  - From year 2000
  - Unlimited historical depth
- [x] **Monthly bars** - Available: Yes
  - From year 2000
  - Unlimited historical depth
- [x] **Tick data** - Available: No (not explicitly offered)
- [x] **Adjusted prices** - For splits, dividends (via equity corporate actions)
- [x] **Intraday data** - Current trading day real-time candles

---

## Derivatives Data (Futures & Options)

**Upstox supports NSE F&O, BSE F&O, MCX futures:**

- [x] **Open Interest** - Total, by instrument
  - Current OI
  - OI day high
  - OI day low
  - Historical OI (via candle data - index 6)
- [x] **Funding Rates** - Not applicable (Indian markets don't have perpetual contracts)
- [ ] **Liquidations** - Not applicable (no crypto perpetuals)
- [ ] **Long/Short Ratios** - Not available
- [ ] **Mark Price** - Not explicitly available
- [ ] **Index Price** - Available for indices (NSE_INDEX, BSE_INDEX)
- [ ] **Basis** - Can be calculated (futures price - spot price)

---

## Options Data

**Full option chain support for NSE, BSE (not MCX):**

- [x] **Options Chains** - Strikes, expirations
  - All strikes for given expiry
  - Call and Put data
  - Multiple expiry dates
- [x] **Implied Volatility** - Yes (included in option chain)
- [x] **Greeks** - Delta, gamma, theta, vega, rho
  - Available in option chain API
  - Real-time via WebSocket (option_greeks mode)
- [x] **Open Interest** - Per strike (call and put)
- [x] **Historical option prices** - Via historical candle API
- [x] **Put/Call Ratio** - Can be calculated from chain data
- [x] **Bid/Ask for options** - Market depth available
- [x] **Option volume** - Trading volume per strike
- [x] **Underlying spot price** - Included in option data
- [x] **Probability of Profit (POP)** - Not explicitly available

**Option Chain Fields:**
- Strike price
- Call and Put data
- LTP, close price, volume, OI
- Bid/ask prices and quantities
- Previous OI
- Delta, gamma, theta, vega, rho
- Implied volatility (IV)

---

## Fundamental Data (Stocks)

**Limited fundamental data available:**

- [x] **Company Profile** - Basic info
  - Company name
  - ISIN
  - Sector (via instrument file)
  - Trading symbol
- [ ] **Financial Statements** - Not available via API
- [ ] **Earnings** - Not available
- [ ] **Dividends** - Not directly available (may need external source)
- [ ] **Stock Splits** - Not directly available (may need external source)
- [ ] **Analyst Ratings** - Not available
- [ ] **Insider Trading** - Not available
- [ ] **Institutional Holdings** - Not available
- [ ] **Financial Ratios** - Not available
- [ ] **Valuation Metrics** - Not available

**Note:** Upstox is primarily a trading platform, not a fundamental data provider. For fundamental data, integrate with specialized providers (e.g., NSE corporate filings, BSE data, third-party fundamental APIs).

---

## On-chain Data (Crypto)

**Not Applicable** - Upstox does not support cryptocurrency trading or data.

- [ ] Wallet Balances
- [ ] Transaction History
- [ ] DEX Trades
- [ ] Token Transfers
- [ ] Smart Contract Events
- [ ] Gas Prices
- [ ] Block Data
- [ ] NFT Data

---

## Macro/Economic Data (Economics)

**Not Available** - Upstox focuses on Indian equity and derivatives markets.

- [ ] Interest Rates
- [ ] GDP
- [ ] Inflation
- [ ] Employment
- [ ] Retail Sales
- [ ] Industrial Production
- [ ] Consumer Confidence
- [ ] PMI
- [ ] Economic Calendar

**Alternative:** Use dedicated economic data providers (e.g., RBI data, MOSPI, global providers like FRED, Trading Economics).

---

## Forex Specific

**Limited** - Upstox supports currency derivatives (USD-INR, EUR-INR, GBP-INR, JPY-INR futures on NSE), not spot forex.

- [x] **Currency Pairs** - Currency futures available
  - USD-INR
  - EUR-INR
  - GBP-INR
  - JPY-INR
  - (as futures contracts on NSE)
- [x] **Bid/Ask Spreads** - Available for currency futures
- [x] **Historical FX rates** - Via currency futures historical data
- [ ] **Spot forex** - Not available (only futures)
- [ ] **Cross rates** - Calculate from futures
- [ ] **Pip precision** - Futures tick size available

---

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - Comprehensive
  - Complete instruments JSON file
  - Exchange-specific files (NSE, BSE, MCX)
  - Segment-specific filtering
  - Instrument types (EQ, FUT, CE, PE, INDEX)
- [x] **Exchange Information** - Yes
  - NSE, BSE, MCX coverage
  - Segment details (NSE_EQ, NSE_FO, BSE_EQ, BSE_FO, MCX_FO, etc.)
- [x] **Market Hours** - Implicit via market status
  - Regular trading hours
  - Pre-market session (for NSE F&O from Dec 2025)
  - Post-market
- [x] **Trading Calendars** - Implicit
  - Market holidays (check market status)
  - Half-days (via market status)
- [x] **Timezone Info** - IST (Asia/Kolkata, UTC+5:30)
- [x] **Sector/Industry Classifications** - Basic
  - Industry type in instrument metadata
  - Limited classification depth

**Instrument File Fields:**
- instrument_key (unique ID)
- exchange_token
- trading_symbol
- name (company/instrument name)
- last_price
- expiry (for derivatives)
- strike (for options)
- tick_size
- lot_size
- instrument_type (EQ, FUT, CE, PE, INDEX)
- option_type (Call/Put for options)
- exchange (NSE, BSE, MCX)
- segment (NSE_EQ, NSE_FO, BSE_EQ, BSE_FO, MCX_FO, etc.)
- isin (for equities)

---

## News & Sentiment

**Not Available** via Upstox API.

- [ ] News Articles
- [ ] Press Releases
- [ ] Social Sentiment
- [ ] Analyst Reports

**Alternative:** Integrate third-party news/sentiment providers (e.g., Bloomberg, Reuters, NewsAPI, StockTwits, Twitter API).

---

## Trading-Specific Data

- [x] **Order Book** - User's orders for the day
- [x] **Trade History** - Historical trades
- [x] **Positions** - Current positions
  - Day positions (buy/sell)
  - Overnight positions
  - P&L (realized and unrealized)
- [x] **Holdings** - Long-term holdings
  - Quantity, average price
  - Current value, P&L
  - Collateral info
- [x] **Funds & Margins** - Account balance
  - Available margin
  - Used margin
  - Equity and commodity segments
  - Combined funds (from July 2025)
- [x] **Trade Charges** - Brokerage breakdown
  - Brokerage
  - GST
  - STT
  - Transaction fees
  - Clearing charges
  - SEBI fees
  - Stamp duty
  - DP charges
- [x] **P&L Reports** - Trade-wise profit/loss
  - By segment (EQ, FO, COM, CD)
  - By financial year
  - Date range filtering
- [x] **GTT Orders** - Good Till Trigger orders
  - Multi-leg strategies (ENTRY, TARGET, STOPLOSS)
  - Order details, status
- [x] **Margin Requirements** - Calculate margin for instruments

---

## Real-time Streaming Data (WebSocket)

### Market Data Feed
- [x] **LTPC Mode:**
  - Last traded price
  - Last trade time
  - Last trade quantity
  - Close price (previous day)
- [x] **Full Mode (5 depth):**
  - LTPC data
  - Market depth (5 bid/ask levels)
  - OHLC
  - Volume, OI
  - Total buy/sell quantities
  - Circuit limits
  - OI day high/low
- [x] **Full D30 Mode (30 depth):**
  - All Full mode data
  - 30 levels of market depth (Plus users only)
- [x] **Option Greeks Mode:**
  - LTPC
  - Delta, theta, gamma, vega, rho
  - Implied volatility
  - Underlying spot price

### Portfolio Feed
- [x] **Order Updates:**
  - Order status changes
  - Filled quantity updates
  - Rejection reasons
  - Order timestamps
- [x] **GTT Order Updates:**
  - GTT status changes
  - Rule triggers
  - Multi-leg order updates
- [x] **Position Updates:**
  - Real-time P&L
  - Quantity changes
  - Buy/sell updates
- [x] **Holdings Updates:**
  - Holdings changes
  - Collateral updates
  - Current value updates

---

## Unique/Custom Data

### What makes Upstox special?

1. **Indian Market Focus:**
   - Deep integration with NSE, BSE, MCX
   - Support for Indian-specific instruments (equity, F&O, commodity, currency futures)
   - ISIN-based instrument identification for equities

2. **WebSocket with Protocol Buffers:**
   - Binary format for efficient data transmission
   - Lower latency compared to JSON
   - Suitable for high-frequency data streaming

3. **Comprehensive Option Chain:**
   - Full Greeks (delta, gamma, theta, vega, rho)
   - Implied volatility
   - Real-time updates via WebSocket
   - Not available for MCX (only NSE, BSE)

4. **GTT Orders:**
   - Good Till Trigger (GTT) order type
   - Multi-leg strategies (ENTRY, TARGET, STOPLOSS)
   - One-year validity
   - Trigger-based execution

5. **Multi-Order APIs (Beta):**
   - Batch order placement (up to 200 orders)
   - Cancel all open orders (by segment/tag)
   - Exit all positions (up to 200 positions)
   - Optimized for algorithmic trading

6. **Historical Depth:**
   - Daily data from year 2000 (24+ years)
   - Intraday from January 2022
   - Flexible intervals (1-300 minutes, 1-5 hours, daily, weekly, monthly)

7. **Instrument Metadata:**
   - Comprehensive instrument files
   - Daily BOD updates (~6 AM IST)
   - Exchange tokens, lot sizes, tick sizes
   - Segment and type classifications

8. **Margin Trading Facility (MTF):**
   - MTF positions tracking
   - Dedicated API for MTF instruments
   - List of MTF-eligible instruments

9. **Indian Market Hours:**
   - Support for pre-market (NSE F&O)
   - After-market orders (AMO)
   - Market status tracking (open, closed, pre_open, post_close)

10. **Brokerage & Charges Breakdown:**
    - Detailed charge calculation
    - Segment-wise breakdown
    - Tax components (GST, STT, stamp duty)
    - SEBI fees, clearing charges

---

## Data Not Available

**Missing data types (need external sources):**
- Fundamental data (financials, earnings, ratios)
- Corporate actions (detailed dividends, splits)
- News and sentiment
- Economic indicators
- Analyst ratings and research
- Insider trading data
- Institutional holdings
- Mutual fund data
- Crypto/blockchain data
- Spot forex (only currency futures)
- Global markets (US, EU, etc.)

---

## Data Coverage Summary

| Category | Coverage | Notes |
|----------|----------|-------|
| Equities | Excellent | All NSE, BSE stocks |
| Futures | Excellent | NSE, BSE, MCX |
| Options | Excellent | NSE, BSE (not MCX) |
| Indices | Good | NSE_INDEX, BSE_INDEX |
| Commodities | Good | MCX futures |
| Currency | Limited | Only futures (NSE) |
| Fundamental | Poor | Minimal data |
| News | None | Not available |
| Economic | None | Not available |
| Crypto | None | Not supported |

---

## Data Quality

### Accuracy
- **Source:** Direct from NSE, BSE, MCX exchanges
- **Validation:** Exchange-validated data
- **Corrections:** Automatic from exchange

### Completeness
- **Missing data:** Rare (exchange downtime only)
- **Gaps:** Handled via exchange (non-trading days, market holidays)
- **Backfill:** Available via historical APIs

### Timeliness
- **Latency:** <100ms for WebSocket real-time data
- **Delay:** No delay (real-time data)
- **Market hours:** Full coverage during trading hours
- **Pre-market:** Supported for NSE F&O (from Dec 2025)
- **After-market:** AMO support

---

## Recommended Use Cases

**Best For:**
- Algorithmic trading on Indian markets
- Options trading with Greeks
- Intraday trading with real-time data
- Portfolio tracking and management
- Multi-strategy trading (GTT, multi-order APIs)
- Historical backtesting (24+ years daily, 4+ years intraday)

**Not Suitable For:**
- Fundamental analysis (need external data)
- News-based trading (no news API)
- Global market trading (India-only)
- Cryptocurrency trading
- Spot forex trading (only futures available)

---

## Integration Recommendations

**Combine Upstox with:**
1. **Fundamental Data:** NSE/BSE corporate filings, Screener.in, Trendlyne, Tickertape
2. **News:** NewsAPI, Bloomberg, Reuters, Economic Times API
3. **Economic Data:** RBI data portal, MOSPI, Trading Economics
4. **Global Markets:** IEX Cloud, Alpha Vantage, Yahoo Finance
5. **Alternative Data:** Social sentiment (Twitter, StockTwits), satellite imagery, web scraping
