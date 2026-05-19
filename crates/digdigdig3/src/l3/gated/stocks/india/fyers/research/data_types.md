# Fyers - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Last Traded Price (LTP)
- [x] **Bid/Ask Spread** - Best bid/ask prices and sizes
- [x] **24h Ticker Stats** - Not 24h (market hours only), but includes:
  - Open, High, Low, Close
  - Previous Close
  - Change (points and percentage)
  - Total Volume
- [x] **OHLC/Candlesticks** - Historical and intraday
  - Intervals: 1m, 2m, 3m, 5m, 10m, 15m, 30m, 45m, 60m, 120m, 180m, 240m, 1D, 1W, 1M
- [x] **Level 2 Orderbook** - Market Depth (top 5 bid/ask levels)
  - Bid/Ask prices, volumes, order counts
  - Total buy/sell quantities
- [x] **Recent Trades** - Via WebSocket (tick-by-tick)
- [x] **Volume** - Total volume, last traded quantity

---

## Historical Data

- [x] **Historical prices** - Depth varies by symbol
  - Equity: Multiple years (varies by symbol)
  - Derivatives: Limited by contract duration
  - Options: May be limited to daily only
- [x] **Minute bars** - Available
  - 1min, 2min, 3min, 5min, 10min, 15min, 30min, 45min, 60min
- [x] **Daily bars** - Multiple years
- [ ] **Tick data** - Not available via REST (TBT WebSocket only)
- [x] **Adjusted prices** - Splits and dividends adjusted
  - Corporate actions reflected in historical data

---

## Derivatives Data (F&O)

### Futures & Options (India Markets)

- [x] **Open Interest** - Available (via quotes endpoint)
- [ ] **Funding Rates** - Not applicable (not perpetual futures)
- [ ] **Liquidations** - Not applicable (broker-managed margin)
- [ ] **Long/Short Ratios** - Not provided
- [ ] **Mark Price** - Not applicable (exchange-based settlement)
- [ ] **Index Price** - Spot index prices available
- [ ] **Basis** - Can calculate (futures price - spot price)

### F&O Specific Data

- [x] **Futures Data:**
  - Contract specifications (lot size, expiry)
  - Current price, volume
  - Open interest
  - Historical data (limited by contract duration)
- [x] **Options Data:** (See Options Data section below)

---

## Options Data

- [x] **Options Chains** - Via symbol master and quotes
  - All strikes for given expiry
  - Multiple expiry dates
  - CE (Call) and PE (Put) options
- [x] **Implied Volatility** - Available in quotes
- [x] **Greeks** - Available (delta, gamma, theta, vega)
  - Provided by exchange (NSE/BSE)
- [x] **Open Interest** - Per strike, per expiry
- [x] **Historical option prices** - Limited availability
  - May be restricted to daily timeframe
  - Intraday data limited for options

**Note:** Options historical data has limitations compared to equities. Some users report only daily data availability.

---

## Fundamental Data (Stocks)

**Limited fundamental data available through API.**

- [x] **Company Profile** - Basic info via symbol master
  - Symbol description
  - ISIN code
  - Series (EQ, BE, etc.)
- [ ] **Financial Statements** - Not available
- [ ] **Earnings** - Not available
- [ ] **Dividends** - Not available via API
- [ ] **Stock Splits** - Reflected in adjusted prices
- [ ] **Analyst Ratings** - Not available
- [ ] **Insider Trading** - Not available
- [ ] **Institutional Holdings** - Not available
- [ ] **Financial Ratios** - Not available
- [ ] **Valuation Metrics** - Not available

**Note:** Fyers API focuses on trading data, not fundamental analysis. Use third-party data providers (e.g., Financial Modeling Prep, Alpha Vantage) for fundamentals.

---

## On-chain Data (Crypto)

**Not Applicable** - Fyers is India stock/derivatives broker, not crypto exchange.

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

**Not Available** - Fyers API does not provide macro/economic data.

- [ ] Interest Rates
- [ ] GDP
- [ ] Inflation (CPI, PPI, PCE)
- [ ] Employment Data
- [ ] Retail Sales
- [ ] Industrial Production
- [ ] Consumer Confidence
- [ ] PMI
- [ ] Economic Calendar

**Note:** Use specialized providers (FRED, Trading Economics, etc.) for economic data.

---

## Forex Specific

**Currency Derivatives Available** (NSE Currency Derivatives segment)

- [x] **Currency Pairs** - Major pairs via NSE CD segment
  - USDINR, EURINR, GBPINR, JPYINR
  - Futures and Options
- [x] **Bid/Ask Spreads** - Available
- [x] **Pip precision** - Standard for currency trading
- [ ] **Cross rates** - Not directly (only INR pairs)
- [x] **Historical FX rates** - Historical data for currency futures

**Note:** Spot forex not available (only currency derivatives via NSE).

---

## Commodities

**MCX & NCDEX Coverage**

- [x] **Metals** - Gold, Silver, Copper, etc.
- [x] **Energy** - Crude Oil, Natural Gas
- [x] **Agriculture** - Wheat, Soybean, Cotton, etc.
- [x] **Futures Contracts** - All MCX/NCDEX commodities
- [x] **Historical Data** - Available for commodity futures
- [x] **Real-time Quotes** - Available via WebSocket/REST

---

## Metadata & Reference Data

- [x] **Symbol/Instrument Lists** - Symbol master CSV
  - NSE, BSE, MCX, NCDEX
  - All segments (CM, FO, CD, COMM)
  - Daily updated
- [x] **Exchange Information** - Via market status endpoint
  - Exchange names
  - Segment details
- [x] **Market Hours** - Via market status endpoint
  - Regular hours: 9:15 AM - 3:30 PM (NSE/BSE equity)
  - Pre-opening: 9:00 AM - 9:08 AM (NSE)
  - MCX: Morning 9:00 AM - 5:00 PM, Evening 5:00 PM - 11:30/11:55 PM
- [x] **Trading Calendars** - Holidays available
  - NSE/BSE holiday calendar
  - MCX/NCDEX holiday calendar
  - Fyers holiday calendar: https://fyers.in/holiday-calendar/
- [x] **Timezone Info** - IST (Indian Standard Time, UTC+5:30)
- [x] **Sector/Industry Classifications** - Limited (basic symbol info only)

---

## Account/Portfolio Data

- [x] **Profile** - User profile details
  - Client ID, name, email, mobile
  - PAN, DP ID
- [x] **Funds** - Available balance
  - Capital market balance
  - Commodity market balance
  - Collateral, payout
- [x] **Holdings** - Equity and mutual fund holdings
  - Symbol, quantity, average price
  - Current value, P&L
  - T1 holdings, pledge quantity
- [x] **Positions** - Current day positions
  - Net quantity, side (long/short)
  - Average buy/sell price
  - Realized/unrealized P&L
- [x] **Orders** - Order book
  - All orders (pending, completed, cancelled)
  - Order details (type, status, quantity, price)
- [x] **Trades** - Trade book
  - Executed trades
  - Trade price, quantity, time

---

## Order & Trading Data

- [x] **Order Placement** - All order types
  - Market, Limit, Stop, Stop-Limit
  - Product types: Intraday, CNC, Margin, CO, BO
- [x] **Order Modification** - Modify pending orders
- [x] **Order Cancellation** - Cancel pending orders
- [x] **Basket Orders** - Up to 10 orders in one request
- [x] **Multi-leg Orders** - 2-3 leg strategies
  - Spreads, straddles, strangles
- [x] **Position Conversion** - Intraday to CNC, etc.
- [x] **Position Exit** - Close positions
- [x] **Real-time Order Updates** - Via Order WebSocket
- [x] **Real-time Trade Updates** - Via Order WebSocket

---

## News & Sentiment

**Not Available** - Fyers API does not provide news or sentiment data.

- [ ] News Articles
- [ ] Press Releases
- [ ] Social Sentiment
- [ ] Analyst Reports

**Note:** Use third-party providers (NewsAPI, Benzinga, etc.) for news and sentiment.

---

## E-DIS (Electronic Delivery of Securities)

- [x] **TPIN Generation** - For CDSL authorization
- [x] **Holdings Status** - E-DIS transaction status
- [x] **CDSL Integration** - Submit holdings for delivery

**Purpose:** Authorize delivery of shares from demat account for selling.

---

## Market Status & Utility Data

- [x] **Market Status** - Real-time exchange status
  - NSE, BSE, MCX, NCDEX
  - Segment-wise status (open/closed)
  - Current market timings
- [x] **Symbol Master** - Complete instrument list
  - All exchanges and segments
  - Updated daily
  - CSV format download
- [x] **Instrument Details** - From symbol master
  - Symbol, description, ISIN
  - Lot size, tick size
  - Expiry (for derivatives)
  - Strike price, option type (for options)

---

## Unique/Custom Data

### What Makes Fyers Special?

**1. Free API Access**
- Completely free API for account holders
- No monthly subscription fee
- Rare among Indian brokers

**2. F&O Specialization**
- Strong focus on Futures & Options
- Multi-leg order support (spreads, straddles)
- Bracket Orders (BO) and Cover Orders (CO)

**3. High Rate Limits**
- 100,000 requests per day
- 10x increase in V3 (from 10,000)
- Generous for free tier

**4. WebSocket Capabilities**
- Multiple WebSocket types (Data, Order, TBT)
- Up to 5,000 symbol subscriptions (V3)
- Lite mode for bandwidth optimization
- Binary TBT feed with Protobuf

**5. Fast Order Execution**
- Orders execute under 50 milliseconds
- Real-time order/trade/position updates

**6. E-DIS Integration**
- Electronic Delivery Instruction Slip
- CDSL integration for seamless selling

**7. Multi-Exchange Support**
- NSE, BSE (equity & derivatives)
- MCX, NCDEX (commodities)
- Currency derivatives (NSE CD)

**8. Symbol Master**
- Daily updated CSV downloads
- All instruments across all exchanges
- Complete metadata (lot size, tick size, ISIN)

---

## Data Coverage Summary

| Category | Coverage | Quality | Notes |
|----------|----------|---------|-------|
| **Equities** | Excellent | High | All NSE/BSE stocks |
| **Futures** | Excellent | High | Index & stock futures |
| **Options** | Excellent | High | Full chains, Greeks, OI |
| **Commodities** | Excellent | High | MCX/NCDEX coverage |
| **Currency** | Good | High | INR pairs only (futures/options) |
| **Historical Data** | Good | Medium | Equity excellent, options limited |
| **Real-time Data** | Excellent | High | Tick-by-tick, <1s latency |
| **Market Depth** | Excellent | High | Top 5 levels |
| **Order Book** | Excellent | High | Real-time updates |
| **Fundamentals** | Poor | Low | Very limited |
| **News** | None | N/A | Not available |
| **Economic Data** | None | N/A | Not available |

---

## Data Availability by Asset Class

### Equities (NSE/BSE)
- Real-time quotes: ✅
- Market depth: ✅
- Historical intraday: ✅
- Historical daily: ✅ (multiple years)
- Volume data: ✅
- Corporate actions: ✅ (adjusted prices)

### Index Futures
- Real-time quotes: ✅
- Market depth: ✅
- Open interest: ✅
- Historical data: ✅ (limited by contract duration)
- Rollover data: ⚠️ (manual calculation)

### Index Options
- Real-time quotes: ✅
- Market depth: ✅
- Options chains: ✅
- Greeks (IV, delta, gamma, theta, vega): ✅
- Open interest: ✅
- Historical data: ⚠️ (may be daily only)

### Stock Futures
- Real-time quotes: ✅
- Market depth: ✅
- Open interest: ✅
- Historical data: ✅

### Stock Options
- Real-time quotes: ✅
- Market depth: ✅
- Options chains: ✅
- Greeks: ✅
- Historical data: ⚠️ (limited)

### Commodities (MCX/NCDEX)
- Real-time quotes: ✅
- Market depth: ✅
- Historical data: ✅
- All major commodities: ✅

### Currency Derivatives
- Real-time quotes: ✅
- Market depth: ✅
- Historical data: ✅
- INR pairs only: ⚠️

---

## Data Update Frequency

### Real-time Streams (WebSocket)
- **Price updates:** <1 second (tick-by-tick)
- **Orderbook updates:** Real-time (on change)
- **Order updates:** Immediate (on status change)
- **Trade updates:** Immediate (on execution)
- **Position updates:** Real-time (on change)

### REST API Polling
- **Quotes:** On-demand (up to 10/sec)
- **Depth:** On-demand (up to 10/sec)
- **Positions:** On-demand
- **Orders:** On-demand

### Scheduled Updates
- **Symbol Master:** Daily (updated before market open)
- **Holdings:** End of day (T+1 for delivery)
- **Fundamentals:** N/A (not provided)

---

## Data Quality Notes

1. **Real-time data is exchange-quality** (direct from NSE/BSE/MCX)
2. **Historical data depth varies** by symbol and timeframe
3. **Options historical data limited** (may be daily only for older dates)
4. **Adjusted prices for corporate actions** (splits, dividends)
5. **No pre-market/after-hours data** (only regular market hours)
6. **Fundamental data not available** (use third-party providers)
7. **News and sentiment not provided** (use external services)
8. **Symbol master updated daily** (may lag for newly listed symbols)

---

## Missing Data Types

**What Fyers API Does NOT Provide:**

1. **Fundamental Analysis Data**
   - Financial statements, earnings, ratios
   - Use: Financial Modeling Prep, Alpha Vantage, EOD Historical Data

2. **News & Sentiment**
   - News articles, social sentiment
   - Use: NewsAPI, Benzinga, StockTwits

3. **Economic Calendar**
   - Macro economic releases
   - Use: Trading Economics, Investing.com

4. **Global Markets**
   - US stocks, international exchanges
   - Fyers focuses on Indian markets only

5. **Cryptocurrency**
   - Spot or derivatives crypto trading
   - Use: Binance, Coinbase, etc.

6. **Spot Forex**
   - Only currency derivatives (futures/options)
   - For spot forex: OANDA, FXCM, etc.

7. **Insider Trading Data**
   - Promoter/insider transactions
   - Available on NSE/BSE websites

8. **Analyst Ratings**
   - Buy/sell recommendations
   - Use: Tip Ranks, Zacks, etc.

---

## Recommended Data Stack

**For comprehensive India market trading:**

| Data Type | Provider |
|-----------|----------|
| Trading (Equity, F&O, Commodities) | **Fyers API** ✅ |
| Fundamentals | Financial Modeling Prep, Screener.in |
| News | NewsAPI, Moneycontrol |
| Economic Calendar | Trading Economics |
| Technical Indicators | Calculate from OHLC data |
| Insider Trading | NSE/BSE websites |
| Analyst Ratings | TipRanks, Investing.com |

---

## Notes

1. **Fyers excels at trading data** (real-time, historical, order management)
2. **Limited fundamental data** - need third-party providers
3. **No news/sentiment** - integrate external news APIs
4. **India markets only** - NSE, BSE, MCX, NCDEX
5. **Real-time data is high-quality** and low-latency
6. **Historical data depth varies** - test before production
7. **Options data excellent for real-time** but limited historical
8. **Symbol master updated daily** - download and cache
9. **WebSocket preferred for real-time** (more efficient than REST)
10. **Free API is major advantage** compared to competitors
