# MOEX - Data Types Catalog

## Standard Market Data

- [x] **Current Price** - Last traded price for all securities
- [x] **Bid/Ask Spread** - Best bid and ask quotes (delayed for free, real-time for paid)
- [x] **24h Ticker Stats** - High, low, volume, value, change, change%
- [x] **OHLC/Candlesticks** - Intervals: 1m, 10m, 1h, 1d, 1w, 1mo, 1q
- [x] **Level 1 Orderbook** - Best bid/ask only (free delayed, paid real-time)
- [ ] **Level 2 Orderbook** - Requires paid subscription (10x10 for equities/bonds/FX, 5x5 for derivatives)
- [x] **Recent Trades** - Trade history with price, quantity, time, side
- [x] **Volume** - 24h volume, session volume, cumulative volume
- [x] **Market Value** - Total traded value in RUB/USD/EUR
- [x] **Number of Trades** - Trade count per session/period
- [x] **Market Capitalization** - For equities (updated regularly)

## Historical Data

- [x] **Historical Prices** - Depth: From 1997+ for major indices, varies by security
- [x] **Minute Bars** - Available: Yes (1-minute intervals)
- [x] **10-Minute Bars** - Available: Yes
- [x] **Hourly Bars** - Available: Yes
- [x] **Daily Bars** - Depth: Extensive (20+ years for major securities)
- [x] **Weekly Bars** - Available: Yes
- [x] **Monthly Bars** - Available: Yes
- [x] **Quarterly Bars** - Available: Yes
- [ ] **Tick Data** - Available: Only via Full Order Book product (institutional)
- [x] **Adjusted Prices** - Splits, dividends adjustments available
- [x] **Trading Sessions** - Historical session data (stock market)
- [x] **Archive Files** - Bulk downloads (yearly, monthly, daily)

## Derivatives Data (Futures & Options)

**MOEX Derivatives Market** (engine: futures, market: forts):

- [x] **Open Interest** - Total open positions by asset/contract
- [x] **Open Interest History** - Historical OI data
- [ ] **Funding Rates** - Not applicable (no perpetual futures like crypto)
- [ ] **Liquidations** - Not applicable (different clearing mechanism)
- [ ] **Long/Short Ratios** - Not available publicly
- [x] **Mark Price** - For derivatives
- [x] **Index Price** - For derivatives based on indices
- [x] **Settlement Price** - Daily settlement prices
- [x] **Futures Series** - All available futures contracts
- [x] **Options Series** - All available options series
- [x] **Options Trading Volume** - By series and asset
- [x] **Options Turnover** - Value traded
- [x] **Options Open Positions** - By series and strike

## Options Data

- [x] **Options Chains** - All strikes and expirations available
- [x] **Options Board** - Full option board data with strikes, calls, puts
- [ ] **Implied Volatility** - Not directly provided (calculate client-side)
- [ ] **Greeks** - Not directly provided (calculate client-side: delta, gamma, theta, vega)
- [x] **Open Interest** - Per strike and series
- [x] **Historical Option Prices** - Available for all options
- [x] **Option Series Volumes** - Trading volume by series
- [x] **Option Series Turnovers** - Value traded by series
- [x] **Base Assets** - Underlying assets for options

**Note**: MOEX provides raw option data (prices, volumes, OI). Greeks and IV must be calculated client-side using option pricing models.

## Fundamental Data (Stocks)

**Corporate Information Services (CCI)**:

- [x] **Company Profile** - Name, sector, industry, INN, OGRN, description
- [x] **Financial Statements** - IFRS and Russian accounting (RSBU)
  - [x] Income statements (full and short versions)
  - [x] Balance sheets
  - [x] Cash flow statements
  - [x] Source data
- [x] **Earnings** - Quarterly and annual reports (via financials)
- [x] **Dividends** - Dividend payments and history (via corporate actions)
- [x] **Stock Splits** - Split and consolidation data
- [x] **Analyst Ratings** - Credit ratings (company and security level)
- [ ] **Insider Trading** - Not available publicly
- [ ] **Institutional Holdings** - Not available via ISS API
- [x] **Financial Ratios** - Via IFRS indicators (industry averages available)
- [x] **Valuation Metrics** - P/E, P/B, etc. (via financial data and price)
- [x] **Industry Classifications** - Sector and industry codes
- [x] **Affiliate Reporting** - Related parties and affiliates
- [x] **Corporate Information** - Management, shareholders (limited)

## Corporate Actions & Events

- [x] **Corporate Actions** - All types:
  - [x] Shareholder meetings
  - [x] Coupon payments (bonds)
  - [x] Dividend payments
  - [x] Stock splits/consolidations
- [x] **IR Calendar** - Investor relations events
- [x] **News** - Exchange news and announcements
- [x] **Events** - Exchange events

## Indices

- [x] **Index Values** - Real-time (delayed for free) and historical
- [x] **Index Analytics** - Analytical data by date
- [x] **Index Composition** - Constituent securities (tickers)
- [x] **Index Bulletins** - Official index bulletins
- [x] **Index Changes** - Historical value and percentage changes
- [x] **RUSFAR** - RUSFAR indicator (Russian market indicator)
- [x] **Sector Indices** - Multiple sector and industry indices

**Major Indices**:
- **IMOEX** - Moscow Exchange Index (main benchmark)
- **RTSI** - RTS Index (dollar-denominated)
- Sector indices (financials, energy, consumer, etc.)
- Bond indices
- Commodity indices

## Currency & Forex Data

**Currency Market** (engine: currency):

- [x] **Currency Pairs** - Major pairs (USD/RUB, EUR/RUB, etc.)
- [x] **Bid/Ask Spreads** - Real-time quotes (delayed for free)
- [x] **Cross Rates** - Calculated cross rates
- [x] **Historical FX Rates** - Extensive history
- [x] **Central Bank Rates** - Official CBR (Central Bank of Russia) rates
- [x] **MOEX Fixings** - Daily fixing rates
- [x] **Indicative Forex Rates** - Indicative rates for derivatives

**Supported Currencies**: RUB, USD, EUR, CNY, HKD, KZT, BYN, and others

## Bonds Data

- [x] **Bond Prices** - Current and historical
- [x] **Accrued Interest** - Current and month-end accrued interest
- [x] **Yields** - Calculated yields (current and historical)
- [x] **Yield to Maturity** - Available via yields endpoints
- [x] **Coupon Schedules** - Via corporate actions (coupon payments)
- [x] **Bond Aggregates** - Market aggregate indicators
- [x] **Zero-Coupon Yield Curves** - ZCYC data
- [x] **Swap Curves** - SDFI swap curves
- [x] **Credit Ratings** - Company and security ratings

**Bond Types**:
- Government bonds (OFZ - Облигации федерального займа)
- Corporate bonds
- Municipal bonds
- Eurobonds

## Money Market & Repo

**Money Market** (engine: money):

- [x] **Repo Rates** - Central bank repo rates
- [x] **Repo Dealers** - Dealer information
- [x] **State Rates** - Government rates
- [x] **Money Market Operations** - Trading data

## OTC Markets

- [x] **OTC Markets List** - Available OTC markets (NSD provider)
- [x] **Daily OTC Data** - Daily aggregates
- [x] **Monthly OTC Data** - Monthly aggregates
- [x] **OTC with CCP** - OTC with central counterparty clearing

## Metadata & Reference

- [x] **Symbol/Instrument Lists** - Complete lists for all markets
- [x] **Exchange Information** - Engines, markets, boards structure
- [x] **Market Hours** - Trading schedules (09:50-18:40 for main market)
- [x] **Trading Calendars** - Trading days, holidays, half-days
- [x] **Settlement Calendars** - Settlement day calendars
- [x] **Timezone Info** - Moscow time (UTC+3)
- [x] **Sector/Industry Classifications** - Industry codes and classifications
- [x] **Security Groups** - Security groups and collections
- [x] **Trading Modes** - Boards (T+, T0, etc.) and board groups
- [x] **Security Types** - 90+ security types (stocks, bonds, ETFs, futures, options, etc.)
- [x] **Trading Availability** - Securities listing status
- [x] **Short Instruments List** - Securities available for short selling

## Analytics Products

- [x] **Net Flow 2** - Money flow analysis
  - Available since 2007 for major stocks: SBER, GAZP, LKOH, GMKN, MGNT, ROSN, VTBR, ALRS, SBERP, AFLT
  - Tracks institutional vs retail flows
- [x] **Futures Open Interest (FUTOI)** - Futures OI analytics
- [x] **Stock Correlations** - Correlation coefficients between stocks
- [x] **Deviation Coefficients** - Substantial deviation criteria
- [x] **Quoted Securities** - Securities with active market quotes
- [x] **Current Prices** - Current pricing reference

## Statistics & Risk Data

- [x] **Market Statistics** - Various statistical indicators
- [x] **Risk Parameters** - RMS risk indicators
- [x] **Collateral Rates** - Collateral revaluation rates
- [x] **Margin Requirements** - Derivatives margin data (via risk params)
- [x] **Complex Instruments Markup** - Complex financial instruments classification

## Trading Statistics

- [x] **Market Turnovers** - Total market turnover (daily, cumulative)
- [x] **Session Statistics** - Intermediate "Итоги дня" (daily results)
- [x] **Board Statistics** - Per-board trading stats
- [x] **Market Statistics** - Per-market aggregates

## News & Sentiment

- [x] **News Articles** - Exchange news via /sitenews
- [ ] **Press Releases** - Company press releases (limited via news)
- [ ] **Social Sentiment** - Not available
- [ ] **Analyst Reports** - Not directly available (ratings available, not full reports)
- [x] **Consensus Forecasts** - Analyst consensus for share prices

## Unique/Custom Data

**What makes MOEX special**:

1. **Russian Market Focus**
   - Comprehensive Russian equities, bonds, derivatives
   - Ruble-denominated assets
   - Russian government securities (OFZ)
   - Russian corporate bonds

2. **Dual Accounting Standards**
   - IFRS (International) financial reports
   - RSBU (Russian) accounting reports
   - Industry-average IFRS indicators

3. **Corporate Information Services (CCI)**
   - Extensive company information
   - Credit ratings (company and security level)
   - Affiliate reporting
   - Corporate actions database
   - IR calendar

4. **Net Flow 2 Analytics**
   - Institutional vs retail money flow tracking
   - Available since 2007 for major stocks
   - Unique MOEX analytical product

5. **Zero-Coupon Yield Curves**
   - ZCYC data for bond markets
   - SDFI swap curves
   - Russian fixed income analytics

6. **Multi-Engine Architecture**
   - 11 distinct trading engines
   - 120+ markets
   - 500+ trading boards
   - Comprehensive market structure data

7. **Russian Regulatory Data**
   - INN (Russian tax ID)
   - OGRN (Russian business registration)
   - Industry codes (Russian classifications)
   - Compliance with Russian regulations

8. **Consensus Forecasts**
   - Analyst consensus for Russian stocks
   - Price targets
   - Aggregated analyst views

9. **Full Order Book Product**
   - Message-by-message market data
   - Complete order book reconstruction
   - Institutional-grade market microstructure data

10. **Archive Access**
    - Bulk historical data downloads
    - Yearly, monthly, daily archives
    - Free access to extensive historical data (from 1997+)

## Data NOT Available

**Compared to Western/US Markets**:

- [ ] Real-time institutional holdings (13F filings equivalent)
- [ ] Insider trading reports (Form 4 equivalent)
- [ ] Short interest data (comprehensive)
- [ ] Dark pool data
- [ ] Options Greeks (must calculate client-side)
- [ ] Implied volatility (must calculate client-side)
- [ ] Level 3 orderbook (only 10x10 or 5x5 available)
- [ ] Social media sentiment
- [ ] Alternative data (satellite, credit card, etc.)
- [ ] Detailed analyst reports (only ratings/consensus)

**Compared to Crypto Exchanges**:

- [ ] Funding rates (no perpetual futures)
- [ ] Liquidation events (different clearing system)
- [ ] Long/Short ratios (not publicly available)
- [ ] Open interest by exchange (single exchange)

## Coverage by Asset Class

### Equities
- Russian stocks (blue chips, mid-caps, small caps)
- Preferred shares
- ADRs/GDRs traded on MOEX
- Foreign stocks traded on MOEX
- ETFs

### Fixed Income
- Government bonds (OFZ)
- Corporate bonds
- Municipal bonds
- Eurobonds
- Structured notes

### Derivatives
- Futures (equity indices, currencies, commodities)
- Options (on futures, equities, indices)
- Swaps (via SDFI)

### Currencies
- Major pairs (USD/RUB, EUR/RUB, etc.)
- Emerging market pairs (CNY/RUB, etc.)
- Cross rates

### Commodities
- Precious metals (gold, silver)
- Agricultural commodities
- Energy (limited)

### Indices
- Equity indices (IMOEX, RTSI, sector indices)
- Bond indices
- Volatility indices

## Data Formats

All data available in:
- **JSON** - Modern API format
- **XML** - Legacy format (still widely used)
- **CSV** - For bulk exports
- **HTML** - For browser viewing

Specify format via:
- URL extension: `.json`, `.xml`, `.csv`, `.html`
- Accept header: `Accept: application/json`

## Data Quality Notes

### Accuracy
- **Source**: Direct from Moscow Exchange (authoritative)
- **Validation**: Exchange-validated data
- **Corrections**: Official corrections applied
- **Reconciliation**: End-of-day reconciliation

### Completeness
- **Missing data**: Rare (exchange data is complete)
- **Gaps**: Minimal (only during market closures or halts)
- **Backfill**: Historical data corrections applied retroactively

### Timeliness
- **Real-time latency**: < 1 second (for paid subscribers)
- **Delayed latency**: 15 minutes (free tier)
- **Historical data**: Updated daily (end-of-day)
- **Corporate actions**: Updated as announced

### Reliability
- **Uptime**: High (99%+ expected for exchange infrastructure)
- **Data integrity**: Exchange-grade reliability
- **Disaster recovery**: Exchange-level redundancy

## Summary

MOEX ISS API provides **comprehensive Russian market data** including:

**Strengths**:
- Complete Russian equities, bonds, derivatives coverage
- Extensive historical data (free, from 1997+)
- Dual accounting standards (IFRS + RSBU)
- Rich corporate information (CCI)
- Unique analytics (Net Flow 2, consensus forecasts)
- Zero-coupon yield curves and swap curves
- Multi-asset class coverage (equities, bonds, FX, derivatives, commodities)
- High data quality (direct from exchange)
- Free access to most data (delayed)

**Limitations**:
- 15-minute delay for free tier
- No institutional holdings data
- No insider trading reports
- Must calculate option Greeks/IV client-side
- Limited orderbook depth without subscription (10x10 max)
- Documentation mostly in Russian
- Rate limits not documented

**Ideal for**:
- Russian market trading and analysis
- Multi-asset class strategies
- Fundamental analysis of Russian companies
- Fixed income analytics (Russian bonds)
- Historical backtesting (extensive free data)
- Academic research (free data access)
