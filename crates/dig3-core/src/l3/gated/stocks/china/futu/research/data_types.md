# Futu OpenAPI - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Last traded price
- [x] **Bid/Ask Spread** - Best bid and ask prices
- [x] **24h Ticker Stats** - High, low, volume, turnover, change%, amplitude
- [x] **OHLC/Candlesticks** - Multiple intervals (1m, 3m, 5m, 15m, 30m, 1h, 1d, 1w, 1M)
- [x] **Level 2 Order Book** - Bid/ask depth (10-60 levels depending on market and subscription)
- [x] **Recent Trades** - Tick-by-tick trade data with direction
- [x] **Volume** - 24h volume, cumulative intraday volume
- [x] **Turnover** - Trading amount/value
- [x] **Turnover Rate** - Percentage of shares traded vs outstanding
- [x] **Dark Pool Status** - Dark pool trading status (HK market)

## Historical Data

- [x] **Historical Prices** - Up to **20 years** of daily data
- [x] **Minute Bars** - 1m, 3m, 5m, 15m, 30m, 60m (available)
- [x] **Daily Bars** - Up to 20 years depth
- [x] **Weekly Bars** - Historical weekly candlesticks
- [x] **Monthly Bars** - Historical monthly candlesticks
- [ ] **Tick Data** - Real-time tick-by-tick, but historical tick data not available via API
- [x] **Adjusted Prices** - Three types: 前复权 (forward), 后复权 (backward), 不复权 (unadjusted)

## Derivatives Data (Futures/Options)

### Futures Data
- [x] **Open Interest** - Current open interest for futures contracts
- [x] **Open Interest Change** - Net change in open interest
- [x] **Last Settlement Price** - Previous day settlement price
- [x] **Contract Specifications** - Multiplier, nominal value, expiry
- [x] **Position Data** - Current position quantity and changes
- [ ] **Funding Rates** - Not applicable (not perpetual futures)
- [ ] **Liquidations** - Not provided by Futu API
- [ ] **Long/Short Ratios** - Not provided
- [x] **Mark Price** - Not explicitly, uses last price
- [x] **Index Price** - For index futures (underlying index price)
- [ ] **Basis** - Not directly provided, must calculate (futures - spot)

### Options Data
- [x] **Options Chains** - All strikes and expirations for underlying
- [x] **Implied Volatility** - IV for each option
- [x] **Greeks**:
  - [x] Delta
  - [x] Gamma
  - [x] Theta
  - [x] Vega
  - [x] Rho
- [x] **Open Interest** - Per option contract
- [x] **Historical Option Prices** - Via `request_history_kline()`
- [x] **Strike Price** - In quote data
- [x] **Contract Size** - Multiplier/lot size
- [x] **Days to Expiry** - Expiry date distance
- [x] **Option Type** - Call/Put
- [x] **Option Area Type** - American/European/Other
- [x] **Premium** - Option premium (intrinsic + time value)

## Fundamental Data (Stocks)

**Limited Coverage** - Futu API is primarily market data + trading focused.

- [ ] **Company Profile** - Not directly available via API (available in app)
- [ ] **Financial Statements** - Not available via API
- [ ] **Earnings** - Not available via API
- [ ] **Dividends** - Corporate actions include dividends (see Rehab data)
- [x] **Stock Splits** - Available via `get_rehab()` (rehabilitation data)
- [ ] **Analyst Ratings** - Not available via API
- [ ] **Insider Trading** - Not available via API
- [x] **Institutional Holdings** - `get_holding_change_list()` provides shareholding changes
- [ ] **Financial Ratios** - Not available via API
- [ ] **Valuation Metrics** - Not available via API

### Corporate Actions (Rehab Data)
- [x] **Stock Splits** - Via `get_rehab()`
- [x] **Dividends** - Cash dividends in rehab data
- [x] **Rights Issues** - In rehab data
- [x] **Bonus Shares** - In rehab data
- [x] **Adjustment Factors** - For price adjustments

## Market Metadata & Reference

- [x] **Symbol/Instrument Lists** - `get_stock_basicinfo()`, `get_plate_stock()`
- [x] **Exchange Information** - Security type, exchange, market
- [x] **Market Hours** - `get_market_state()` shows open/closed/pre-market/after-hours
- [x] **Trading Calendars** - `get_trading_days()` returns holidays, half-days
- [x] **Timezone Info** - Market-specific (HK=UTC+8, US=UTC-5/-4)
- [x] **Sector/Industry Classifications** - `get_plate_list()`, `get_plate_stock()`
- [x] **Security Status** - Trading status (normal, suspension, halt)
- [x] **Listing Date** - In `get_stock_basicinfo()`
- [x] **Delisting Status** - In `get_stock_basicinfo()`
- [x] **Lot Size** - Shares per lot (HK market specific)
- [x] **Stock ID** - Unique identifier
- [x] **Security Type** - Stock, ETF, Warrant, Option, Future, Index

## Intraday Data

- [x] **Time Frame Data** - Minute-by-minute intraday timeline
  - Current price at each minute
  - Average price
  - Cumulative volume
  - Cumulative turnover
  - Minutes since market open
  - Blank periods (no trades)
- [x] **Pre-market Data** - US extended hours (if `extended_time=True`)
- [x] **After-hours Data** - US extended hours
- [x] **Overnight Data** - US overnight session (if available)

## Broker-Level Data (Hong Kong Specific)

- [x] **Broker Queue** - Requires LV2 quote subscription
  - Broker ID and name
  - Position in queue (bid/ask side)
  - Only available for HK market
  - Requires `SubType.BROKER` subscription

## Warrants Data (Hong Kong Specific)

- [x] **Warrant Information** - `get_warrant()`
  - Underlying security
  - Warrant type (call/put)
  - Exercise price
  - Expiry date
  - Conversion ratio
  - Issuer
  - HK market only

## IPO Data

- [x] **IPO List** - `get_ipo_list()` provides:
  - Upcoming IPOs
  - Recent IPOs
  - Listing date
  - IPO price
  - Subscription period
  - Security code

## Capital Flow Data

- [x] **Capital Flow** - `get_capital_flow()` provides:
  - Main inflow (large orders)
  - Main outflow
  - Medium inflow/outflow
  - Small inflow/outflow
  - Net inflow/outflow
  - HK stocks primarily

- [x] **Capital Distribution** - `get_capital_distribution()`:
  - Distribution by investor type
  - Institutional vs retail
  - HK stocks primarily

## Plate/Sector Data

- [x] **Plate Lists** - Industry sectors, concept plates, regional plates
- [x] **Plate Hierarchy** - Parent/child plate relationships
- [x] **Plate Stock List** - All securities in a plate/sector
- [x] **Owner Plates** - Which plates a security belongs to
- [x] **Plate Types**:
  - Industry sectors
  - Concept themes
  - Regional classifications
  - Custom categories

## Trading-Related Data

### Account Data
- [x] **Account List** - All trading accounts (HK, US, simulated, etc.)
- [x] **Account Funds**:
  - Total assets
  - Net asset value
  - Securities market value
  - Cash (by currency)
  - Available funds
  - Purchasing power
  - Frozen cash
  - Withdrawable cash
  - Margin used
  - Margin available
- [x] **Account Info**:
  - Account ID
  - Account type (Cash, Margin, Universal)
  - Broker (Futu Securities, moomoo, etc.)
  - Account status (Active, Disabled)
  - Market access permissions

### Position Data
- [x] **Current Positions**:
  - Security code and name
  - Quantity held
  - Available quantity (can sell)
  - Frozen quantity (in orders)
  - Average cost price
  - Market price
  - Market value
  - P&L (profit/loss)
  - P&L %
  - Today's P&L
  - Currency

### Order Data
- [x] **Open Orders**:
  - Order ID
  - Security code
  - Order type (Limit, Market, Stop, etc.)
  - Order status (Submitted, Working, Partial Fill, etc.)
  - Price and quantity
  - Filled quantity
  - Average fill price
  - Create time
  - Update time
  - Time in force
  - Fill outside RTH setting
  - Trading session
  - Remark
  - Last error message

- [x] **Historical Orders**:
  - All fields from open orders
  - Completed/cancelled orders
  - Date range filtering

### Deal/Fill Data
- [x] **Today's Deals**:
  - Deal ID
  - Order ID
  - Security code
  - Deal quantity
  - Deal price
  - Deal time
  - Trading side (Buy/Sell)
  - Counter broker (HK only)

- [x] **Historical Deals**:
  - All fields from today's deals
  - Historical date range
  - Live trading only (not paper trading)

### Trading Limits
- [x] **Max Trade Quantities**:
  - Max buy quantity (based on buying power)
  - Max sell quantity (based on position)
  - Max sell short quantity (for margin accounts)
  - Cash available for buying
  - Margin available

- [x] **Margin Data**:
  - Margin ratio by security
  - Margin requirements
  - Long margin ratio
  - Short margin ratio
  - Detailed margin info

### Order Fees
- [x] **Order Fee Estimates**:
  - Commission
  - Platform fee
  - Handling fee
  - Exchange fee
  - Settlement fee
  - Stamp duty (HK)
  - Transaction levy
  - Total estimated fee

### Risk Data
- [x] **Risk Level** - 9-level risk indicator:
  - Level 1: Very Safe
  - Level 2: Safe
  - Level 3: Medium Safe
  - Level 4: Medium
  - Level 5: Medium Risk
  - Level 6: Risk
  - Level 7: High Risk
  - Level 8: Very High Risk
  - Level 9: Margin Call

## Market-Specific Data

### Hong Kong Market
- [x] Basic quotes (LV1 free)
- [x] Enhanced quotes (LV2 paid)
- [x] Broker queue (LV2 only)
- [x] Warrants data
- [x] Capital flow
- [x] Dark pool status
- [x] Lot trading (board lot system)

### US Market
- [x] Stocks (NYSE, NASDAQ, AMEX)
- [x] ETFs
- [x] Options (chains, Greeks)
- [x] Futures
- [x] Extended hours (pre/post market, overnight)
- [x] Different trading sessions
- [x] Deep order book (TotalView with paid subscription)

### China A-Shares
- [x] A-share stocks (via China Connect)
- [x] Limited to mainland China users (or paid subscription)
- [x] LV1 quotes
- [x] Trading via HK-Shanghai/Shenzhen Stock Connect

### Singapore Market
- [x] Singapore futures
- [x] Limited subscription (50 securities max for SF users)

### Japan Market
- [x] Japan futures
- [x] Limited subscription (50 securities max for SF users)

### Australia Market
- [x] Australian securities
- [x] Via moomoo Australia account

### Malaysia Market
- [x] Malaysian securities
- [x] Via moomoo Malaysia account

### Canada Market
- [x] Canadian securities
- [x] Via moomoo Canada account

## Data NOT Available

### Missing Market Data
- [ ] News articles/feeds
- [ ] Social sentiment
- [ ] Analyst reports (text)
- [ ] Economic calendars
- [ ] Earnings calendars
- [ ] Dividend calendars
- [ ] Corporate events (detailed)
- [ ] Historical tick data (only real-time ticks)

### Missing Fundamental Data
- [ ] Income statements
- [ ] Balance sheets
- [ ] Cash flow statements
- [ ] Financial ratios
- [ ] Valuation multiples
- [ ] Earnings estimates
- [ ] Revenue forecasts

### Missing Alternative Data
- [ ] On-chain data (N/A for stocks)
- [ ] DEX trades (N/A)
- [ ] Token transfers (N/A)
- [ ] Smart contract events (N/A)
- [ ] Gas prices (N/A)

### Missing Macro Data
- [ ] Interest rates
- [ ] GDP data
- [ ] Inflation metrics
- [ ] Employment data
- [ ] Economic indicators

## Unique/Custom Data Features

### What Makes Futu Special

1. **Broker Queue Data (HK)**:
   - Unique to HK market
   - Shows which brokers have orders at each price level
   - Helps identify institutional activity
   - Requires LV2 subscription

2. **Capital Flow Analysis**:
   - Categorizes flows by order size (large/medium/small)
   - Useful for sentiment analysis
   - HK market focus

3. **Dark Pool Status (HK)**:
   - Indicates dark pool trading activity
   - Unique market microstructure data

4. **Multi-Market Single API**:
   - HK, US, A-shares, Singapore, Japan, Australia in one API
   - Unified data format across markets
   - Single subscription quota pool

5. **Paper Trading Data**:
   - Full simulated trading with same API
   - Real market data + simulated execution
   - Risk-free strategy testing

6. **Intraday Timeline**:
   - `RT_DATA` subscription type
   - Minute-by-minute price/volume/turnover
   - Captures entire trading day timeline
   - Identifies periods without trades (blank periods)

7. **Warrant Coverage**:
   - Comprehensive HK warrant data
   - Exercise price, conversion ratio, issuer
   - Important for HK market participants

8. **IPO Data**:
   - Upcoming IPO schedule
   - Subscription periods
   - IPO pricing
   - Helps plan pre-IPO strategies

9. **Extended Hours (US)**:
   - Pre-market (4:00 AM - 9:30 AM ET)
   - Regular hours (9:30 AM - 4:00 PM ET)
   - After-hours (4:00 PM - 8:00 PM ET)
   - Overnight (8:00 PM - 4:00 AM ET)
   - Configurable via `extended_time` and `session` parameters

10. **Algorithmic Order Visibility**:
    - Can query TWAP and VWAP orders (read-only)
    - Cannot place algo orders via API (app only)
    - Helps understand institutional order flow

## Data Quality Notes

### Accuracy
- **Source**: Direct from exchanges for most data
- **Validation**: Exchange-provided data (high quality)
- **Corrections**: Automatic for corporate actions
- **Lag**: Real-time (subject to exchange rules and subscription level)

### Completeness
- **Missing Data**: Rare for subscribed securities
- **Gaps**: Handled by `is_blank` flag in time frame data
- **Backfill**: Historical data available via `request_history_kline()`

### Timeliness
- **Latency**: "Order execution as fast as 0.0014s" (claimed)
- **Quote Delay**: Real-time (LV1/LV2 subscription), no 15-minute delay
- **Market Hours**: Covered fully, including extended hours for US

## Data Format Standards

### Timestamps
- **Format**: "YYYY-MM-DD HH:mm:ss" (string)
- **Timezone**: Market-specific (HK=UTC+8, US=UTC-5/-4)
- **Unix Timestamp**: Not used (string timestamps only)

### Price Precision
- **HK Stocks**: Varies by price range (typically 3 decimals)
- **US Stocks**: 4 decimals (e.g., 150.2500)
- **US Options**: 2 decimals
- **Futures**: 8 integer digits, 9 decimal places

### Security Codes
- **Format**: "MARKET.CODE"
- **Examples**: "US.AAPL", "HK.00700", "SH.600000" (A-shares)
- **Consistency**: Same format across all markets

### Data Structures
- **SDK Return**: Python DataFrame, C# DataTable, Java List, etc.
- **Consistency**: Consistent column names across languages
- **Protocol**: Protocol Buffers (binary format internally)
