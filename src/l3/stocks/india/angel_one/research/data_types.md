# Angel One SmartAPI - Data Types Catalog

## Standard Market Data

- [x] **Current Price (LTP)** - Last Traded Price via quote API and WebSocket Mode 1
- [x] **Bid/Ask Spread** - Best bid/ask in Quote mode (Mode 2) and Snap Quote (Mode 3)
- [x] **24h Ticker Stats** - high, low, volume, change in Quote API
- [x] **OHLC/Candlesticks** - Historical candle data via getCandleData endpoint
  - Intervals: ONE_MINUTE, THREE_MINUTE, FIVE_MINUTE, TEN_MINUTE, FIFTEEN_MINUTE, THIRTY_MINUTE, ONE_HOUR, ONE_DAY
- [x] **Level 2 Orderbook** - 5 levels (Snap Quote Mode 3), 20 levels (Depth 20 Mode 4)
- [x] **Recent Trades** - Last traded price and quantity in real-time
- [x] **Volume** - Total volume for the day, last traded quantity
- [x] **Average Traded Price** - VWAP (Volume Weighted Average Price)
- [x] **Total Buy/Sell Quantity** - Aggregate bid/ask quantities

## Historical Data

- [x] **Historical prices** - Up to 2000 days for daily candles
- [x] **Minute bars** - ONE_MINUTE (30 days), THREE_MINUTE (60 days), FIVE_MINUTE (100 days), TEN_MINUTE (100 days), FIFTEEN_MINUTE (200 days), THIRTY_MINUTE (200 days)
- [x] **Hourly bars** - ONE_HOUR (400 days)
- [x] **Daily bars** - ONE_DAY (2000 days = ~5.5 years)
- [ ] **Tick data** - Not available
- [x] **Adjusted prices** - Historical data includes corporate actions (splits, dividends, bonuses)
- [x] **Maximum candles** - Up to 8000 candles per single API request

**Note**: Historical data for expired F&O contracts is NOT available.

## Derivatives Data (Futures & Options)

### Futures
- [x] **Open Interest** - Current open interest for futures
- [x] **Open Interest Change %** - Percentage change in OI
- [ ] **Funding Rates** - Not applicable (Indian market doesn't use funding rates like crypto perpetuals)
- [ ] **Liquidations** - Not applicable (no forced liquidations in traditional futures)
- [ ] **Long/Short Ratios** - Not provided
- [x] **Mark Price** - Last traded price serves as settlement reference
- [ ] **Index Price** - Spot index separate from futures (available via index symbols)
- [ ] **Basis** - Not directly provided (can calculate: futures price - spot price)

### Options
- [x] **Options Chains** - Available via instrument master (all strikes and expirations)
- [ ] **Implied Volatility** - Not directly provided by API
- [ ] **Greeks** - Not provided (delta, gamma, theta, vega not available)
- [x] **Open Interest** - Available for individual option contracts
- [x] **Historical option prices** - OHLC for option contracts (until expiry)
- [x] **Strike Prices** - All strikes in instrument master
- [x] **Expiration Dates** - Expiry dates in instrument master

**Options Data Availability**: Basic price and OI data available, but advanced analytics (IV, Greeks) not provided.

## Fundamental Data (Stocks)

Angel One SmartAPI focuses on trading execution and market data. Fundamental data is **NOT available** through the API.

- [ ] Company Profile
- [ ] Financial Statements
- [ ] Earnings (EPS, revenue, guidance)
- [ ] Dividends (history, yield)
- [x] **Stock Splits** - Reflected in adjusted historical prices
- [ ] Analyst Ratings
- [ ] Insider Trading
- [ ] Institutional Holdings
- [ ] Financial Ratios (P/E, P/B, ROE, etc.)
- [ ] Valuation Metrics

**For fundamental data**: Use separate data providers (e.g., NSE/BSE websites, Bloomberg, Reuters, or dedicated fundamental data APIs).

## On-chain Data (Crypto)

**Not applicable** - Angel One SmartAPI is for traditional Indian markets (stocks, derivatives, commodities, currency), not cryptocurrency.

- [ ] Wallet Balances
- [ ] Transaction History
- [ ] DEX Trades
- [ ] Token Transfers
- [ ] Smart Contract Events
- [ ] Gas Prices
- [ ] Block Data
- [ ] NFT Data

## Macro/Economic Data (Economics)

**Not available** - SmartAPI does not provide macroeconomic data.

- [ ] Interest Rates
- [ ] GDP
- [ ] Inflation (CPI, WPI)
- [ ] Employment Data
- [ ] Retail Sales
- [ ] Industrial Production
- [ ] Consumer Confidence
- [ ] PMI (Manufacturing, Services)
- [ ] Economic Calendar

**For economic data**: Use RBI (Reserve Bank of India) data, MOSPI (Ministry of Statistics), or economic data providers.

## Forex Specific

### Currency Derivatives (CDS Segment)
- [x] **Currency Pairs** - Major pairs (USD/INR, EUR/INR, GBP/INR, JPY/INR)
- [x] **Bid/Ask Spreads** - Available in quote data
- [x] **Pip precision** - Standard forex precision in quotes
- [x] **Cross rates** - Not directly provided (can calculate from USD pairs)
- [x] **Historical FX rates** - Historical candle data for currency futures

**Note**: Angel One provides currency derivatives (futures), not spot forex. All currency trading is in futures contracts on NSE CDS segment.

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - Complete instrument master JSON file
  - URL: https://margincalculator.angelone.in/OpenAPI_File/files/OpenAPIScripMaster.json
  - Updated regularly with new IPOs, F&O contracts
- [x] **Exchange Information** - Exchange segments in user profile
- [x] **Market Hours** - Not explicitly provided (standard Indian market hours apply)
  - Equity: 9:15 AM - 3:30 PM IST
  - F&O: 9:15 AM - 3:30 PM IST
  - Commodities: Varies by commodity
  - Currency: 9:00 AM - 5:00 PM IST
- [ ] **Trading Calendars** - Holidays not provided via API (use NSE/BSE holiday calendars)
- [x] **Timezone Info** - All timestamps in IST (Indian Standard Time, UTC+5:30)
- [x] **Sector/Industry Classifications** - Not in API (available in instrument master as exchange segment)
- [x] **Circuit Limits** - Upper and lower circuit limits in Snap Quote and Depth 20 modes
- [x] **52-Week High/Low** - Available in Snap Quote (Mode 3) and Depth 20 (Mode 4)
- [x] **Tick Size** - Available in instrument master file
- [x] **Lot Size** - Available in instrument master file (for F&O contracts)

## Trading Data

### Order Types
- [x] **MARKET** - Market orders (immediate execution)
- [x] **LIMIT** - Limit orders (price specified)
- [x] **STOPLOSS_LIMIT** - Stop loss with limit price
- [x] **STOPLOSS_MARKET** - Stop loss with market execution

### Order Varieties
- [x] **NORMAL** - Regular orders
- [x] **STOPLOSS** - Stop loss orders
- [x] **AMO** - After Market Orders (placed post-market, execute next day)
- [x] **ROBO** - Bracket Orders (with target and SL)

### Product Types
- [x] **DELIVERY** - Cash & Carry (CNC) for equity
- [x] **CARRYFORWARD** - Normal for F&O (NRML)
- [x] **MARGIN** - Margin Delivery
- [x] **INTRADAY** - Margin Intraday Squareoff (MIS)
- [x] **BO** - Bracket Order product type

### Order Duration
- [x] **DAY** - Day orders (valid for current trading session)
- [x] **IOC** - Immediate or Cancel

### Advanced Order Types
- [x] **GTT** - Good Till Triggered (valid for 1 year)
- [x] **OCO** - One Cancels Other (two triggers, one cancels other)
- [ ] **Cover Orders** - Mentioned but details not fully documented
- [x] **Bracket Orders** - Target profit + stop loss (ROBO variety)

## Portfolio & Account Data

### Holdings
- [x] **Long-term Holdings** - Delivery positions held overnight
- [x] **Average Price** - Average purchase price
- [x] **Current Price** - Current market price
- [x] **Quantity** - Number of shares/contracts held
- [x] **P&L** - Profit/Loss on holdings
- [x] **Total Holding Value** - Aggregate portfolio value

### Positions
- [x] **Intraday Positions** - MIS positions (must close by EOD)
- [x] **Open Positions** - F&O positions carried forward
- [x] **Position P&L** - Mark-to-market P&L
- [x] **Position Conversion** - Convert between product types

### Funds & Margins
- [x] **RMS Limits** - Risk Management System limits
- [x] **Available Margin** - Margin available for trading
- [x] **Used Margin** - Margin blocked in open positions
- [x] **Fund Balance** - Available cash balance
- [x] **Margin Calculator** - Calculate margin requirement for basket of orders

### Order & Trade Data
- [x] **Order Book** - All orders for the day
- [x] **Trade Book** - Executed trades
- [x] **Order Status** - Real-time order status
- [x] **Order History** - Status changes for individual orders
- [x] **Fill Price** - Execution price
- [x] **Fill Quantity** - Executed quantity
- [x] **Partial Fills** - Partial execution tracking

## News & Sentiment

**Not available** - SmartAPI does not provide news or sentiment data.

- [ ] News Articles
- [ ] Press Releases
- [ ] Social Sentiment
- [ ] Analyst Reports

**For news**: Use dedicated news aggregators or NSE/BSE announcements.

## Unique/Custom Data

### What Makes Angel One SmartAPI Special?

1. **Depth 20 Order Book** (WebSocket V2 Mode 4)
   - 20 levels of bid/ask data
   - Unique among Indian broker APIs
   - Superior market microstructure visibility
   - Launched in 2024 (beta → stable)

2. **Free Historical Data for All Segments**
   - NSE, BSE, NFO, BFO, MCX, CDS, NCDEX
   - Completely free (competitors charge or limit)
   - Up to 2000 days for daily data
   - 8000 candles per request

3. **120+ Indices Coverage**
   - Real-time OHLC for 120 indices across NSE, BSE, MCX
   - Broader index coverage than many competitors

4. **Margin Calculator API**
   - Real-time margin calculation for basket of positions
   - Pre-trade margin validation
   - Launched June 2025

5. **Comprehensive Exchange Coverage**
   - Equity: NSE, BSE
   - Derivatives: NFO, BFO
   - Commodities: MCX, NCDEX
   - Currency: CDS
   - All in single API

6. **High Order Rate Limits**
   - 20 orders/sec (vs 10/sec for competitors)
   - Suitable for active algo trading

7. **Real-time Order Updates via WebSocket**
   - Separate WebSocket for order status streaming
   - Immediate notification of order executions

8. **Public Instrument Master**
   - No authentication required to download symbol list
   - Regularly updated with new listings
   - JSON format for easy parsing

## Data Format Notes

### Price Format
- **REST API**: Prices in integer paise (divide by 100 for rupees)
  - Example: `50025` = ₹500.25
- **WebSocket**: Prices in integer paise
- **Historical Data**: Prices in rupees (decimal format)

### Timestamp Format
- **REST API Historical**: String format "YYYY-MM-DD HH:MM"
- **WebSocket**: Unix timestamp in milliseconds
- **Timezone**: IST (UTC+5:30)

### Symbol Format
- **Trading Symbol**: Exchange-specific format
  - NSE Equity: "SBIN-EQ"
  - BSE Equity: "SBIN" (without -EQ)
  - NFO Futures: "NIFTY26FEBFUT"
  - NFO Options: "NIFTY26FEB21000CE" (CE=Call, PE=Put)
  - MCX: "GOLD26FEBFUT"
  - CDS: "USDINR26FEB"
- **Symbol Token**: Numeric token (unique identifier)
  - Example: "3045" for SBIN on NSE

### Instrument Master Fields
```json
{
  "token": "3045",
  "symbol": "SBIN-EQ",
  "name": "STATE BANK OF INDIA",
  "expiry": "",
  "strike": "-1.000000",
  "lotsize": "1",
  "instrumenttype": "",
  "exch_seg": "NSE",
  "tick_size": "5.000000"
}
```

**Key Fields**:
- `token`: Numeric identifier for API calls
- `symbol`: Trading symbol
- `name`: Full company/instrument name
- `expiry`: Expiry date for derivatives (empty for equity)
- `strike`: Strike price for options (-1 for non-options)
- `lotsize`: Minimum trading lot (1 for equity, varies for F&O)
- `instrumenttype`: Contract type (blank for equity, "OPTIDX", "FUTIDX", etc.)
- `exch_seg`: Exchange segment (NSE, BSE, NFO, BFO, MCX, CDS, NCDEX)
- `tick_size`: Minimum price movement

## Data Availability Matrix

| Data Type | REST API | WebSocket | Historical | Free? | Notes |
|-----------|----------|-----------|------------|-------|-------|
| LTP | Yes | Yes | No | Yes | Real-time price |
| OHLC | Yes | Yes | Yes | Yes | Quote mode, historical candles |
| Volume | Yes | Yes | Yes | Yes | Included in quote/candles |
| Order Book (5 levels) | Yes | Yes | No | Yes | Snap Quote mode |
| Order Book (20 levels) | Yes | Yes | No | Yes | Depth 20 mode |
| Ticker Stats | Yes | Yes | No | Yes | High, low, open, close |
| Open Interest | Yes | Yes | Yes | Yes | For derivatives |
| Historical Candles | Yes | No | Yes | Yes | All segments, 8000 max |
| 52W High/Low | Yes | Yes | No | Yes | Snap Quote, Depth 20 |
| Circuit Limits | Yes | Yes | No | Yes | Snap Quote, Depth 20 |
| Holdings | Yes | No | No | Yes | Portfolio endpoint |
| Positions | Yes | No | No | Yes | Portfolio endpoint |
| Order Book | Yes | Yes* | No | Yes | *Via order update WebSocket |
| Trade Book | Yes | No | No | Yes | Query endpoint |
| Margin/Funds | Yes | No | No | Yes | RMS endpoint |
| Instrument List | Yes | No | No | Yes | Public JSON file |

## Coverage Segments

| Segment | Code | Type | Supported | Historical | WebSocket |
|---------|------|------|-----------|------------|-----------|
| NSE | NSE | Equity | Yes | Yes | Yes |
| BSE | BSE | Equity | Yes | Yes | Yes |
| NFO | NFO | Derivatives (F&O) | Yes | Yes | Yes |
| BFO | BFO | Derivatives (F&O) | Yes | Yes | Yes |
| MCX | MCX | Commodities | Yes | Yes | Yes |
| CDS | CDS | Currency | Yes | Yes | Yes |
| NCDEX | NCDEX | Commodities | Yes | Yes | Yes |

**All segments support**:
- Real-time quotes
- Historical data (free)
- WebSocket streaming
- Order execution

## Summary

### Strengths
1. Free historical data for all segments (huge cost advantage)
2. Depth 20 order book (unique feature)
3. 120+ indices coverage
4. High rate limits (20 orders/sec)
5. Comprehensive exchange coverage
6. Margin calculator API
7. Real-time order updates

### Limitations
1. No fundamental data
2. No options analytics (IV, Greeks)
3. No news/sentiment data
4. No macroeconomic data
5. No tick data
6. Historical data unavailable for expired F&O contracts

### Best Use Cases
- **Algorithmic trading**: Excellent for order execution
- **Market microstructure analysis**: Depth 20 provides detailed book
- **Multi-asset strategies**: Equity, derivatives, commodities, currency in one API
- **Historical backtesting**: Free comprehensive historical data
- **Real-time trading**: WebSocket for live quotes and order updates
- **Portfolio management**: Holdings, positions, P&L tracking

### Not Suitable For
- Fundamental analysis (no fundamental data)
- Options strategy analysis requiring Greeks/IV
- News-based trading (no news feed)
- Economic event trading (no economic calendar)
- Ultra-high-frequency trading (20/sec may be limiting)
