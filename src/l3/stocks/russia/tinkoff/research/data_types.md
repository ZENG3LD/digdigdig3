# Tinkoff Invest API - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Last trade price via GetLastPrices
- [x] **Bid/Ask Spread** - Order book via GetOrderBook (L2 depth: 1-50 levels)
- [x] **24h Ticker Stats** - Not available (MOEX doesn't provide 24h stats like crypto)
- [x] **OHLC/Candlesticks** - GetCandles (intervals: 5s, 10s, 30s, 1m, 2m, 3m, 5m, 10m, 15m, 30m, 1h, 2h, 4h, 1d, 1w, 1mo)
- [x] **Level 2 Orderbook** - GetOrderBook (bids/asks depth configurable: 1, 10, 20, 30, 40, 50)
- [x] **Recent Trades** - GetLastTrades (anonymous trades from last hour)
- [x] **Volume** - Included in candle data (volume in lots)
- [x] **Trading Status** - GetTradingStatus (NORMAL_TRADING, BREAK, AUCTION, etc. - 17 states)
- [x] **Close Prices** - GetClosePrices (session closing prices)

## Historical Data

- [x] **Historical prices** - Depth: up to 10 years (from 1970-01-01 for some instruments)
- [x] **Minute bars** - Available: Yes (1m, 2m, 3m, 5m, 10m, 15m, 30m)
  - 1-minute: 1 day depth (2400 candles)
  - 5-minute: 1 week depth (2400 candles)
  - 15-minute: 3 weeks depth (2400 candles)
- [x] **Second bars** - Available: Yes (5s, 10s, 30s)
  - 5-second: 200 minutes depth (2500 candles)
  - 10-second: 200 minutes depth (1250 candles)
  - 30-second: 20 hours depth
- [x] **Daily bars** - Depth: 6 years (2400 candles max)
- [x] **Weekly bars** - Depth: 5 years (300 candles max)
- [x] **Monthly bars** - Depth: 10 years (120 candles max)
- [x] **Tick data** - Not available via API
- [x] **Adjusted prices** - Yes (splits and dividends accounted for)

## Derivatives Data (Futures/Options)

### Futures
- [x] **Futures contracts** - Full support via FutureBy, Futures methods
- [x] **Mark Price** - Not explicitly provided (use last price or calculate)
- [x] **Margin requirements** - GetFuturesMargin (guarantee collateral amount)
- [x] **Contract specifications** - Included in futures instrument data
- [x] **Expiration dates** - Part of futures metadata
- [x] **Settlement data** - Available via operations/broker reports

### Options
- [x] **Options chains** - OptionsBy (filter by underlying asset)
- [x] **Strike prices** - Included in option metadata
- [x] **Expiration dates** - Part of option specification
- [x] **Implied Volatility** - Not provided by API
- [x] **Greeks (delta, gamma, theta, vega)** - Not provided by API
- [x] **Open Interest** - Not provided via API (may be in instrument data)
- [x] **Historical option prices** - GetCandles works for options (use instrument_uid)
- [x] **Underlying asset tracking** - basic_asset_uid parameter

**Note**: Advanced options analytics (IV, Greeks) not provided - must calculate client-side.

## Fundamental Data (Stocks)

### Company Information
- [x] **Company profile** - GetBrandBy (company name, sector, description)
- [x] **Sector/Industry** - Part of instrument metadata
- [x] **Exchange listing** - Trading venue in instrument data
- [x] **Country** - GetCountries provides reference data
- [x] **ISIN/FIGI** - Part of instrument identification

### Financial Data
- [x] **Dividends** - GetDividends (payment events, amounts, ex-dates)
- [x] **Stock splits** - Historical adjustments in price data
- [x] **Financial statements** - Not provided (income, balance sheet, cash flow not available)
- [x] **Earnings (EPS, revenue)** - Not provided
- [x] **Financial ratios** - Not provided (P/E, P/B, ROE, etc.)
- [x] **Analyst ratings** - Not provided
- [x] **Insider trading** - Not provided
- [x] **Institutional holdings** - Not provided

### Bond-Specific Data
- [x] **Coupon payments** - GetBondCoupons (schedule, amounts, payment dates)
- [x] **Accrued interest** - GetAccruedInterests (NKD - накопленный купонный доход)
- [x] **Yield to maturity** - Not provided (must calculate)
- [x] **Bond ratings** - Not provided via API
- [x] **Maturity date** - Part of bond metadata
- [x] **Face value (nominal)** - Included in bond specification

**Note**: Tinkoff API focuses on trading data, not fundamental analysis. Limited fundamental data compared to dedicated financial data providers.

## Trading & Account Data

### Portfolio & Positions
- [x] **Portfolio holdings** - GetPortfolio (stocks, bonds, ETFs, currencies, futures, options)
- [x] **Current positions** - GetPositions (securities, futures, options + blocked amounts)
- [x] **Position P&L** - Expected yield in portfolio data
- [x] **Average entry price** - Average position price (FIFO and regular)
- [x] **Blocked amounts** - Securities/funds blocked in orders
- [x] **Available balance** - GetWithdrawLimits (liquid funds for withdrawal)
- [x] **Margin status** - GetMarginAttributes (leverage, liquidity, requirements)

### Operations & Transactions
- [x] **Trade history** - GetOperations (trades, commissions, dividends, etc.)
- [x] **Commission fees** - Included in operations data
- [x] **Dividends received** - Part of operations history
- [x] **Tax withholding** - GetDividendsForeignIssuer (foreign dividends + tax)
- [x] **Broker statements** - GetBrokerReport (official trade confirmations)
- [x] **Money movements** - Deposits, withdrawals in operations

### Orders & Execution
- [x] **Active orders** - GetOrders (pending orders with status)
- [x] **Order history** - GetOperations with filters
- [x] **Order execution details** - Order state, fills, partial fills
- [x] **Stop orders** - GetStopOrders (active conditional orders)
- [x] **Order types supported** - Market, Limit, Best Price, Stop-Loss, Take-Profit, Stop-Limit

## Real-Time Streaming Data

### Market Data Streams (via WebSocket/gRPC)
- [x] **Real-time candles** - MarketDataStream with candle subscription
- [x] **Real-time trades** - Anonymous trade feed
- [x] **Real-time order book** - L2 updates (snapshot, not delta)
- [x] **Real-time trading status** - Instrument status changes
- [x] **Real-time last price** - Price updates on trades

### Account Streams
- [x] **Portfolio updates** - PortfolioStream (holdings changes)
- [x] **Position updates** - PositionsStream (position changes)
- [x] **Order execution events** - TradesStream (fills, partial fills)

## Metadata & Reference Data

### Instruments
- [x] **Stock list** - Shares (~1,900 on MOEX as of 2022)
- [x] **Bond list** - Bonds (~655)
- [x] **ETF list** - Etfs (~105)
- [x] **Futures list** - Futures (~284)
- [x] **Options list** - Options (available, count varies)
- [x] **Currency pairs** - Currencies (~21 pairs)
- [x] **Instrument search** - FindInstrument (search by query)
- [x] **Instrument details** - Full specifications (lot size, price step, currency, etc.)

### Trading Information
- [x] **Trading schedules** - TradingSchedules (exchange hours by date range)
- [x] **Market hours** - Regular, pre-market, after-hours in schedule
- [x] **Trading calendars** - Holidays, half-days via trading schedule
- [x] **Timezone info** - UTC timestamps throughout API
- [x] **Exchange information** - Real exchange (MOEX, RTS) in instrument data
- [x] **Price limits** - Daily limit up/down in order book data

### User & Account
- [x] **User accounts** - GetAccounts (all trading accounts with types/statuses)
- [x] **Account types** - Tinkoff, IIS (Individual Investment Account), Invest Box
- [x] **User tariff** - GetUserTariff (commission rates, service fees)
- [x] **Qualification status** - GetInfo (qualified investor status)
- [x] **Margin attributes** - Leverage, liquidity, margin requirements

### Favorites & Personalization
- [x] **User favorites** - GetFavorites (saved instruments)
- [x] **Edit favorites** - EditFavorites (add/remove)

## Asset Information

- [x] **Assets** - GetAssets, GetAssetBy (asset-level data)
- [x] **Brands** - GetBrands, GetBrandBy (company/brand information)
- [x] **Countries** - GetCountries (ISO codes + metadata)

## On-chain Data (Crypto)

**NOT APPLICABLE** - Tinkoff is traditional broker, not crypto exchange.

- [ ] Wallet balances
- [ ] Transactions
- [ ] DEX trades
- [ ] Token transfers
- [ ] Smart contract events
- [ ] Gas prices
- [ ] Block data
- [ ] NFT data

## Macro/Economic Data (Economics)

**NOT DIRECTLY PROVIDED** - Focus is on instrument trading data.

- [ ] Interest rates (not via API, but instruments like OFZ bonds reflect rates)
- [ ] GDP
- [ ] Inflation metrics
- [ ] Employment data
- [ ] Retail sales
- [ ] Industrial production
- [ ] Consumer confidence
- [ ] PMI
- [ ] Economic calendar (not provided - use external source)

**Workaround**: Trade economic-sensitive instruments (government bonds, currency pairs) which reflect macro conditions.

## News & Sentiment

**NOT PROVIDED**

- [ ] News articles
- [ ] Press releases
- [ ] Social sentiment
- [ ] Analyst reports

**Recommendation**: Use external news APIs (Tinkoff Journal, Russian business news sources).

## Unique/Custom Data

### What makes Tinkoff special?

1. **Russian Market Focus**
   - Deep coverage of MOEX (Moscow Exchange) instruments
   - Russian bonds with coupon/accrued interest data
   - Ruble-denominated instruments
   - Access to Russian government bonds (OFZ)
   - IIS (Individual Investment Account) support

2. **Integrated Brokerage**
   - Direct access to actual trading account
   - Real portfolio, not just data feed
   - Margin trading support with real-time margin calculations
   - Multiple account types (standard, IIS)

3. **Comprehensive Trading**
   - Full order lifecycle (market, limit, stop orders)
   - Best Price orders (unique order type)
   - Stop orders with GTC/GTD expiration
   - Order replacement (cancel + create atomically)

4. **Sandbox Environment**
   - Complete testing environment
   - Virtual funds via SandboxPayIn
   - Identical API to production
   - No risk testing for strategies

5. **Dynamic Rate Limiting**
   - Fair usage system (active traders get more)
   - No hard caps for high-volume users
   - Scales with business value

6. **gRPC Protocol**
   - Modern protocol (not just REST)
   - Bidirectional streaming
   - Protocol Buffers efficiency
   - Strong typing via proto contracts

7. **Qualified Investor Support**
   - API respects qualification status
   - Access to restricted instruments
   - Compliance built into API

8. **Tax Reporting**
   - Foreign dividend tax reporting
   - Broker statements via API
   - Operations history for tax filing

9. **Granular Time Intervals**
   - Sub-minute candles (5s, 10s, 30s)
   - Unusual intervals (2m, 3m, 10m)
   - Very high granularity for HFT

10. **Multi-Currency Support**
    - RUB, USD, EUR, CNY, other currencies
    - Currency exchange via API
    - Portfolio in multiple currencies

## Data Coverage Summary

| Category | Coverage | Quality | Notes |
|----------|----------|---------|-------|
| **Market Data** | ⭐⭐⭐⭐⭐ Excellent | Real-time | Full coverage, all instruments |
| **Historical Data** | ⭐⭐⭐⭐⭐ Excellent | Complete | Up to 10 years, multiple intervals |
| **Trading** | ⭐⭐⭐⭐⭐ Excellent | Full | All order types, real execution |
| **Portfolio** | ⭐⭐⭐⭐⭐ Excellent | Real-time | Live account data, positions, P&L |
| **Fundamentals** | ⭐⭐ Limited | Basic | Dividends, coupons only - no financials |
| **News/Sentiment** | ⭐ None | N/A | Not provided |
| **Economic Data** | ⭐ None | N/A | Not provided |
| **Options Analytics** | ⭐⭐ Limited | Basic | No Greeks, IV - calculate yourself |
| **Derivatives** | ⭐⭐⭐⭐ Good | Complete | Futures/options trading, margin data |

## Comparison with Other Data Providers

| Feature | Tinkoff | Polygon.io | Alpha Vantage | IEX Cloud |
|---------|---------|------------|---------------|-----------|
| **Focus** | Russian stocks + trading | US stocks | Global stocks | US stocks |
| **Real-time** | Yes (free) | Yes (paid) | No (15min delay free) | Yes (paid) |
| **Historical** | 10 years | 20+ years | 20+ years | 5 years |
| **Fundamentals** | Limited | Excellent | Good | Limited |
| **Trading** | Full support | No | No | No |
| **Price** | Free | $199+/mo | Free/$25/mo | $9+/mo |

**Tinkoff advantage**: Only one with integrated trading execution + real portfolio data.

## Data Freshness

### Real-Time Data
- **Market data**: Real-time (no delay)
- **Order book**: Real-time L2 snapshots
- **Trades**: Real-time anonymous feed
- **Portfolio**: Real-time updates (PortfolioStream)
- **Order execution**: Real-time events (TradesStream)

### Near Real-Time
- **Trading status**: Updates on status changes (seconds)
- **Position changes**: Updates on trades/operations

### Scheduled/Delayed
- **Dividends**: Announced by companies (days before ex-date)
- **Coupon payments**: Schedule known in advance
- **Financial reports**: Not provided (get from company IR)

### Historical
- **Candle data**: Available with slight delay (after candle closes)
- **Operations history**: Available within minutes of execution
- **Broker reports**: Generated on request

## Missing Data Types (Limitations)

### Not Available via API:
1. **Financial statements** (income, balance sheet, cash flow)
2. **Earnings reports** (EPS, revenue, guidance)
3. **Analyst ratings** and price targets
4. **Insider trading** activity
5. **Institutional holdings** (13F filings equivalent)
6. **News feed** (corporate news, press releases)
7. **Economic calendar** (GDP, inflation, jobs data)
8. **Options Greeks** (delta, gamma, vega, theta, rho)
9. **Implied volatility** (for options)
10. **Options open interest** (per strike)
11. **Tick-by-tick data** (only trades, not all quotes)
12. **L3 order book data** (only L2)
13. **Corporate actions** (other than dividends/splits)
14. **Short interest** data
15. **Dark pool activity**

### Workarounds:
- **Fundamentals**: Use Moex.com, InvestFunds.ru, or company IR sites
- **News**: Tinkoff Journal, Interfax, Reuters Russia
- **Economic data**: CBR (Central Bank of Russia), Rosstat
- **Options analytics**: Calculate Greeks yourself using pricing models
- **Tick data**: Use second-level candles (5s, 10s) as approximation

## Summary Checklist

**Available Data** (100% coverage):
- [x] Real-time market data (prices, trades, order book)
- [x] Historical candles (multiple intervals, up to 10 years)
- [x] Trading execution (orders, stops, fills)
- [x] Portfolio & positions (real-time)
- [x] Account information (balance, margin, limits)
- [x] Instrument metadata (specs, schedules, search)
- [x] Bonds (coupons, accrued interest)
- [x] Dividends (payment schedule)
- [x] Streaming data (WebSocket/gRPC)

**Limited Data**:
- [~] Fundamentals (only dividends, no financials)
- [~] Options (trading only, no Greeks/IV)
- [~] Analytics (basic data, must calculate indicators)

**Not Available**:
- [ ] Company financials
- [ ] News/sentiment
- [ ] Economic indicators
- [ ] Advanced options analytics (must calculate)
