# Dhan - Data Types Catalog

## Standard Market Data

- [x] **Current Price (LTP)** - Last Traded Price via REST and WebSocket
- [x] **Bid/Ask Spread** - 5-level, 20-level, 200-level depth available
- [x] **24h Ticker Stats** - Not available (Indian markets: 6h15m trading day)
- [x] **Daily Stats** - Open, High, Low, Close, Volume via Quote API
- [x] **OHLC/Candlesticks** - Intervals: 1m, 5m, 15m, 25m, 60m, 1D
  - Intraday: 1m, 5m, 15m, 25m, 60m (last 5 years)
  - Daily: From instrument inception (10+ years for many stocks)
- [x] **Level 2 Orderbook** - 5-level bid/ask (via Quote API or WebSocket)
- [x] **Deep Orderbook**:
  - 20-level market depth (NSE Equity & Derivatives only)
  - 200-level market depth (NSE Equity & Derivatives only)
- [x] **Recent Trades** - Trade history by order ID or date range
- [x] **Volume** - Daily volume, intraday volume (per candle)
- [x] **Last Traded Quantity (LTQ)** - Available in ticker feed
- [x] **Last Traded Time (LTT)** - Timestamp of last trade

## Historical Data

- [x] **Historical prices** - Depth: From instrument inception
  - NSE stocks: ~20+ years for blue chips
  - Derivatives: From contract listing date
- [x] **Minute bars** - 1m, 5m, 15m, 25m, 60m (last 5 years)
- [x] **Daily bars** - From inception (10-20+ years)
- [ ] **Tick data** - Not available via API
- [x] **Adjusted prices** - Corporate actions adjusted (splits, dividends, bonuses)
- [x] **Historical Open Interest** - For F&O instruments (last 5 years intraday)
- [x] **90-day query window** - Intraday data fetched in 90-day chunks maximum
- [x] **Expired contracts data** - Available for expired F&O contracts

## Derivatives Data (Equity Derivatives - NSE F&O)

Dhan provides extensive derivatives analytics:

- [x] **Open Interest (OI)** - Current and historical
  - Total OI per contract
  - OI changes (buildup/unwinding)
  - OI in option chains
- [x] **Futures Data**:
  - Index futures (Nifty, Bank Nifty, Fin Nifty, Midcap Nifty, etc.)
  - Stock futures
  - Basis (futures - spot spread) - Can be calculated
- [ ] **Funding Rates** - N/A (not applicable to equity derivatives)
- [ ] **Liquidations** - N/A (no forced liquidations in Indian equity markets)
- [ ] **Long/Short Ratios** - Not provided by API
- [x] **Mark Price** - LTP serves as mark price
- [x] **Index Price** - Available for index derivatives (Nifty, Bank Nifty, etc.)

## Options Data

Comprehensive options analytics available:

- [x] **Options Chains** - All strikes for given expiry
  - Multiple expiry dates supported
  - Expiries extending to 2026 and beyond
- [x] **Implied Volatility (IV)** - Per strike in option chain
- [x] **Greeks** - Delta, Gamma, Theta, Vega, Rho
  - Available in option chain API response
- [x] **Open Interest per Strike** - Current OI for each strike
- [x] **Historical option prices** - OHLC data for option contracts
- [x] **Option Volume** - Trading volume per strike
- [x] **Bid/Ask for Options** - Top 5 levels in quote, up to 200 levels in depth
- [x] **Option Chain Snapshot** - Real-time snapshot of entire chain
- [x] **Put/Call Data** - Organized by strike (calls and puts)
- [x] **Expiry Dates** - Multiple expiries available:
  - Weekly expiries (Nifty, Bank Nifty, Fin Nifty)
  - Monthly expiries
  - Far-month expiries (2026 and beyond)

**Rate Limit Note**: Option chain API limited to 1 request per 3 seconds (OI updates slowly).

## Fundamental Data (Stocks)

**NOT AVAILABLE** via Dhan API.

Dhan is a trading-focused broker, not a fundamental data provider.

Not available:
- [ ] Company Profile
- [ ] Financial Statements
- [ ] Earnings (EPS, revenue)
- [ ] Dividends
- [ ] Stock Splits (historical data only available)
- [ ] Analyst Ratings
- [ ] Insider Trading
- [ ] Institutional Holdings
- [ ] Financial Ratios
- [ ] Valuation Metrics

**Workaround**: Use separate fundamental data provider (e.g., NSE/BSE websites, Bloomberg, CapitalIQ).

## On-chain Data (Crypto)

**NOT APPLICABLE** - Dhan is an Indian stock broker, not a crypto exchange.

- [ ] Wallet Balances
- [ ] Transaction History
- [ ] DEX Trades
- [ ] Token Transfers
- [ ] Smart Contract Events
- [ ] Gas Prices
- [ ] Block Data
- [ ] NFT Data

## Macro/Economic Data (Economics)

**NOT AVAILABLE** via Dhan API.

Dhan does not provide macroeconomic data.

- [ ] Interest Rates
- [ ] GDP
- [ ] Inflation (CPI, PPI)
- [ ] Employment
- [ ] Retail Sales
- [ ] Industrial Production
- [ ] Consumer Confidence
- [ ] PMI
- [ ] Economic Calendar

**Workaround**: Use RBI (Reserve Bank of India), NSE economic calendar, or dedicated providers.

## Forex Specific

**NOT APPLICABLE** - Dhan does not offer forex trading.

Indian brokers are restricted to:
- Equity cash
- Equity derivatives (F&O)
- Commodities (via MCX)
- Currency derivatives (limited pairs on NSE)

Currency derivatives available on NSE:
- [x] **Currency Futures** - USD/INR, EUR/INR, GBP/INR, JPY/INR
- [x] **Currency Options** - Limited pairs

But NOT spot forex (no spot USD/INR trading for retail).

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - CSV files per exchange segment
  - `GET /v2/instrument/NSE_EQ` - NSE Equity instruments
  - `GET /v2/instrument/NSE_FNO` - NSE F&O instruments
  - `GET /v2/instrument/BSE_EQ` - BSE Equity instruments
  - `GET /v2/instrument/MCX_COMM` - MCX Commodity instruments
- [x] **Exchange Information** - Via instrument lists (exchange segment, security ID)
- [x] **Market Hours** - Not via API (standard Indian market hours apply)
  - Pre-market: 9:00 AM - 9:15 AM IST
  - Regular: 9:15 AM - 3:30 PM IST
  - Post-market: 3:40 PM - 4:00 PM IST
- [x] **Trading Calendars** - Not via API (refer to NSE/BSE/MCX calendars)
- [x] **Timezone Info** - IST (UTC+5:30)
- [x] **Sector/Industry Classifications** - Limited (available in instrument lists)
- [x] **Security IDs** - Unique IDs for each instrument (used in all API calls)
- [x] **ISIN codes** - Available in instrument CSV files
- [x] **Lot sizes** - For F&O instruments in instrument list
- [x] **Tick sizes** - Standard exchange tick sizes apply

## News & Sentiment

**NOT AVAILABLE** via Dhan API.

- [ ] News Articles
- [ ] Press Releases
- [ ] Social Sentiment
- [ ] Analyst Reports

**Workaround**: Integrate with news providers (NSE announcements, MoneyControl, etc.).

## Unique/Custom Data

**What makes Dhan special:**

### 1. **200-Level Market Depth**
- **Unique in Indian retail broking**
- Deepest orderbook access (200 price levels)
- Available for NSE Equity and NSE Derivatives
- Real-time via WebSocket
- Separate WebSocket endpoint: `wss://full-depth-api.dhan.co/twohundreddepth`
- **Limitation**: Only 1 instrument per connection (due to data volume)

### 2. **20-Level Market Depth**
- More accessible than 200-level
- Up to 50 instruments per connection
- Real-time via WebSocket
- Endpoint: `wss://depth-api-feed.dhan.co/twentydepth`

### 3. **Super Orders Data**
- Track bracket orders with trailing SL via API
- Entry leg, target leg, stop loss leg status
- Trailing stop loss adjustments visible
- Unique to Dhan's advanced order types

### 4. **Forever Orders (GTT) Data**
- Good-Till-Triggered orders valid for 365 days
- Single and OCO (One-Cancels-Other) orders
- Query status via API
- Similar to GTT on other platforms but via API

### 5. **5-Year Intraday Historical Data**
- **Industry-leading depth**: 5 years of minute-level data
- Most Indian brokers: 1-2 years max
- Intervals: 1m, 5m, 15m, 25m, 60m
- Includes Open Interest for F&O

### 6. **Binary WebSocket Format**
- High-performance binary encoding (Little Endian)
- Lower latency than JSON
- Requires binary parsing (struct unpacking)
- Trade-off: Complexity vs performance

### 7. **Full Packet Feed**
- Single subscription includes: LTP + Quote + OI + Market Depth
- Reduces subscription complexity
- Request Code: 19

### 8. **Live Order Updates via WebSocket**
- Real-time order status changes
- Faster than polling REST API
- Dedicated WebSocket channel (Request Code: 5)

## Trading-Related Data (Unique to Broker APIs)

### Portfolio & Holdings
- [x] **Holdings** - Delivered stocks (T1, T2, delivered)
  - Quantity, average price, current price
  - P&L calculation
  - Collateral status
- [x] **Positions** - Intraday and carry-forward positions
  - Day P&L, overall P&L
  - Buy/sell quantities
  - Average prices
  - Net positions
- [x] **Position Conversion** - Intraday ↔ Delivery conversion tracking

### Funds & Margins
- [x] **Available Margin** - Trading balance
- [x] **Used Margin** - Blocked for open positions/orders
- [x] **Collateral Value** - From pledged holdings
- [x] **Ledger Report** - Credit/debit transactions
  - Date range queries supported
  - All transaction types

### Order & Trade Data
- [x] **Order Book** - All orders for the day
  - Order ID, status, type, price, quantity
  - Filled/pending/cancelled
- [x] **Trade Book** - Executed trades
  - Trade ID, order ID, fill price, quantity
  - Trade time, charges
- [x] **Order History by Date** - Historical order data
- [x] **Trade History by Date** - Historical trade data

### EDIS Data
- [x] **EDIS Status** - Electronic Delivery Instruction Slip status
- [x] **T-PIN Status** - CDSL T-PIN generation status
- [x] **Holdings Eligible for Sale** - After EDIS completion

## Data Format & Delivery

### REST API Response Format
- **Format**: JSON
- **Encoding**: UTF-8
- **Structure**:
  ```json
  {
    "data": { ... },
    "status": "success"
  }
  ```
  Or error:
  ```json
  {
    "errorType": "...",
    "errorCode": "...",
    "errorMessage": "..."
  }
  ```

### WebSocket Response Format
- **Request Format**: JSON
- **Response Format**: Binary (Little Endian)
- **Parsing**: Requires struct unpacking (fixed-size packets)
- **Performance**: ~10x faster than JSON (less overhead)

### Historical Data Format
- **Candles**: Array of OHLCV objects
  ```json
  [
    {
      "timestamp": 1234567890000,
      "open": 100.50,
      "high": 101.00,
      "low": 100.00,
      "close": 100.75,
      "volume": 1000000
    }
  ]
  ```
- **OI** (for derivatives): Additional field in candle
  ```json
  {
    "timestamp": 1234567890000,
    "open": 100.50,
    "high": 101.00,
    "low": 100.00,
    "close": 100.75,
    "volume": 1000000,
    "open_interest": 5000000
  }
  ```

### Instrument List Format
- **Format**: CSV
- **Columns**:
  - SecurityId (Dhan's unique ID)
  - ISIN
  - Symbol
  - Name
  - Expiry (for derivatives)
  - Strike (for options)
  - OptionType (CE/PE)
  - LotSize
  - TickSize
  - Exchange
  - Segment

## Data Quality & Reliability

### Accuracy
- **Source**: Direct from exchange (NSE, BSE, MCX)
- **Real-time**: Tick-by-tick (no artificial delays for retail)
- **Validation**: Exchange-validated data
- **Corrections**: Automatic corporate action adjustments

### Completeness
- **Missing data**: Rare (exchange feeds highly reliable)
- **Gaps**: Automatically handled by exchange
- **Backfill**: Historical data complete from inception

### Timeliness
- **Latency**:
  - WebSocket: <50ms typical (from exchange to API)
  - REST quotes: <200ms typical
- **Delay**: None (real-time for all users)
- **Market hours**: Full coverage (pre-market, regular, post-market)

## Data Availability Summary

| Data Type | Available | Method | Free? | Notes |
|-----------|-----------|--------|-------|-------|
| Real-time LTP | ✅ | REST + WS | Conditional | Quote API or WebSocket |
| Real-time OHLC | ✅ | REST + WS | Conditional | Quote packet |
| 5-level Depth | ✅ | REST + WS | Conditional | Market depth |
| 20-level Depth | ✅ | WS only | Conditional | NSE only |
| 200-level Depth | ✅ | WS only | Conditional | NSE only, 1 instrument/conn |
| Daily Historical | ✅ | REST | Conditional | From inception |
| Intraday Historical | ✅ | REST | Conditional | Last 5 years, 1m to 60m |
| Option Chains | ✅ | REST | Conditional | With Greeks, OI |
| Open Interest | ✅ | REST + WS | Conditional | F&O only |
| Corporate Actions | ✅ | Implicit | N/A | Adjusted in prices |
| Fundamentals | ❌ | N/A | N/A | Not provided |
| News | ❌ | N/A | N/A | Not provided |
| Economic Data | ❌ | N/A | N/A | Not provided |

**Conditional Free**: Free if 25+ trades/month, else Rs. 499/month

## Segments & Instrument Coverage

### NSE (National Stock Exchange)
- **NSE_EQ** (Cash Market):
  - ~2,000 equity stocks
  - ETFs, InvITs, REITs
  - Preference shares
- **NSE_FNO** (Derivatives):
  - ~200+ stock futures
  - ~300+ stock options
  - Index futures: Nifty 50, Bank Nifty, Fin Nifty, Midcap Nifty, etc.
  - Index options: Same indices
  - Currency derivatives: USD/INR, EUR/INR, GBP/INR, JPY/INR

### BSE (Bombay Stock Exchange)
- **BSE_EQ** (Cash Market):
  - ~5,000 equity stocks (more than NSE)
  - ETFs
  - Preference shares
  - Lower liquidity than NSE for most stocks

### MCX (Multi Commodity Exchange)
- **MCX_COMM** (Commodities):
  - Metals: Gold, Silver, Copper, Zinc, Lead, Nickel, Aluminum
  - Energy: Crude Oil, Natural Gas
  - Agriculture: Not typically available for retail via API

## Data Access Best Practices

1. **Use WebSocket for live data**: More efficient than polling
2. **Cache historical data**: Avoid repeated API calls
3. **Use Full Packet (Code 19)**: Single subscription for all data types
4. **Distribute across connections**: 5 connections × 5,000 instruments = 25,000 coverage
5. **Query option chains sparingly**: 1 req/3sec limit
6. **Fetch instrument lists daily**: Updated regularly for new contracts
7. **Monitor data staleness**: Check timestamps on received data
8. **Handle corporate actions**: Prices adjusted automatically, track announcements separately
