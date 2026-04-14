# yahoo - Data Types Catalog

## Standard Market Data

- [x] Current Price (real-time with 15-20s delay for some exchanges)
- [x] Bid/Ask Spread (available for most liquid symbols)
- [x] 24h Ticker Stats (high, low, volume, change%)
- [x] OHLC/Candlesticks (intervals: 1m, 2m, 5m, 15m, 30m, 60m, 90m, 1h, 1d, 5d, 1wk, 1mo, 3mo)
- [ ] Level 2 Orderbook (NOT available - Yahoo only provides best bid/ask)
- [x] Recent Trades (NOT individual trades, but aggregated volume data)
- [x] Volume (24h, intraday, with pre/post market breakdown)
- [x] Pre-Market Data (price, volume, change for supported exchanges)
- [x] Post-Market Data (price, volume, change for supported exchanges)

## Historical Data

- [x] Historical prices (depth: Varies by symbol, typically 10-50+ years for major stocks)
- [x] Minute bars (available: Yes, 1m data limited to last 7 days)
- [x] Intraday bars (2m, 5m, 15m, 30m, 60m - last 60 days only)
- [x] Daily bars (depth: Decades for major symbols, typically to IPO date)
- [x] Weekly bars (depth: Full history available)
- [x] Monthly bars (depth: Full history available)
- [ ] Tick data (NOT available - no individual tick data)
- [x] Adjusted prices (splits and dividends automatically adjusted)
- [x] Unadjusted prices (available via events parameter)
- [x] Dividend history (dates, amounts, yield)
- [x] Stock split history (dates, ratios)
- [x] Capital gains distributions (for funds)

## Derivatives Data (Crypto/Futures)

**Not applicable for most data** - Yahoo Finance focuses on spot prices, not derivatives analytics.

- [ ] Open Interest (NOT available - no futures/options OI data)
- [ ] Funding Rates (NOT available - no perpetual futures data)
- [ ] Liquidations (NOT available - no exchange liquidation data)
- [ ] Long/Short Ratios (NOT available)
- [ ] Mark Price (NOT available)
- [ ] Index Price (NOT available)
- [ ] Basis (NOT available - no futures-spot spread)

**Note:** Yahoo Finance is NOT a derivatives data provider. For futures/crypto derivatives, use specialized providers.

## Options Data (if applicable)

- [x] Options Chains (all strikes, all expirations for equity options)
- [x] Implied Volatility (per option contract)
- [x] Greeks (delta, gamma, theta, vega, rho - calculated by Yahoo)
- [x] Open Interest (per strike/expiration)
- [x] Volume (daily option contract volume)
- [x] Bid/Ask (option contract bid/ask prices)
- [ ] Historical option prices (NOT available via API - current data only)
- [x] Expiration dates (all available expirations for a symbol)
- [x] Strike prices (all strikes for each expiration)
- [x] Contract names (standardized OCC symbols)

## Fundamental Data (Stocks)

- [x] Company Profile (name, sector, industry, description, website, employees)
- [x] Company Officers (CEO, CFO, etc. with titles and compensation)
- [x] Financial Statements - Annual (income statement, balance sheet, cash flow)
- [x] Financial Statements - Quarterly (income statement, balance sheet, cash flow)
- [x] Financial Statements - TTM (trailing twelve months data)
- [x] Earnings (EPS actual vs estimates, historical and future)
- [x] Earnings Dates (upcoming and historical earnings release dates)
- [x] Revenue (historical and estimated future revenue)
- [x] Earnings Estimates (analyst consensus EPS/revenue estimates)
- [x] Earnings Trend (revision trends, growth estimates)
- [x] Dividends (history, dates, amounts, yield, payout ratio)
- [x] Ex-Dividend Date (upcoming and historical)
- [x] Stock Splits (historical split dates and ratios)
- [x] Analyst Ratings (buy/hold/sell counts, price targets)
- [x] Analyst Price Targets (mean, median, high, low targets)
- [x] Upgrade/Downgrade History (analyst rating changes over time)
- [x] Insider Trading (insider buys/sells with dates, amounts, prices)
- [x] Insider Holders (insiders and their positions)
- [x] Institutional Holdings (major institutional holders and positions)
- [x] Mutual Fund Holdings (funds holding the stock)
- [x] Major Holders Breakdown (% owned by institutions, insiders, public)
- [x] Share Purchase Activity (net insider buying/selling)
- [x] Financial Ratios (P/E, P/B, P/S, ROE, ROA, debt/equity, current ratio, etc.)
- [x] Valuation Metrics (enterprise value, EV/EBITDA, PEG ratio, market cap)
- [x] Profitability Metrics (gross margin, operating margin, profit margin)
- [x] Cash Flow Metrics (free cash flow, operating cash flow)
- [x] Balance Sheet Metrics (total assets, total liabilities, book value per share)
- [x] SEC Filings (links to 10-K, 10-Q, 8-K, etc. on SEC website)
- [x] Calendar Events (earnings dates, dividend dates, ex-dividend dates)

## Fund-Specific Data (ETFs, Mutual Funds)

- [x] Fund Profile (category, family, inception date, legal type)
- [x] Fund Performance (1m, 3m, 6m, 1y, 3y, 5y returns)
- [x] Fund Top Holdings (largest positions with weights)
- [x] Fund Sector Weightings (% allocation by sector)
- [x] Fund Asset Allocation (stocks, bonds, cash percentages)
- [x] Fund Bond Holdings (top bond positions for bond funds)
- [x] Fund Bond Ratings (credit quality breakdown)
- [x] Fund Equity Holdings (equity style box data)
- [x] Fund Fees (expense ratio, management fees)
- [x] Fund Yield (dividend yield, SEC yield)
- [x] Fund Total Assets (AUM - assets under management)
- [x] Fund Turnover Ratio (portfolio turnover percentage)
- [x] Fund Manager Info (manager names, tenure)
- [x] Fund Category (Morningstar category classification)

## On-chain Data (Crypto)

**Not Available** - Yahoo Finance does NOT provide blockchain/on-chain data.

- [ ] Wallet Balances (NOT available)
- [ ] Transaction History (NOT available)
- [ ] DEX Trades (NOT available)
- [ ] Token Transfers (NOT available)
- [ ] Smart Contract Events (NOT available)
- [ ] Gas Prices (NOT available)
- [ ] Block Data (NOT available)
- [ ] NFT Data (NOT available)

**Note:** Yahoo Finance only provides centralized exchange crypto prices (spot), not on-chain analytics.

## Macro/Economic Data (Economics)

**Limited** - Yahoo Finance provides some economic data via index symbols.

- [x] Interest Rates (via symbols like ^TNX, ^IRX, ^TYX for treasury yields)
- [ ] GDP (NOT available - no direct GDP endpoints)
- [ ] Inflation (NOT available - no CPI/PPI data)
- [ ] Employment (NOT available - no NFP/unemployment data)
- [ ] Retail Sales (NOT available)
- [ ] Industrial Production (NOT available)
- [ ] Consumer Confidence (NOT available)
- [ ] PMI (NOT available)
- [ ] Economic Calendar (NOT available - no scheduled release calendar)

**Note:** For comprehensive economic data, use FRED API or specialized economic data providers.

## Forex Specific

- [x] Currency Pairs (majors, minors, exotics via =X suffix symbols)
- [x] Bid/Ask Spreads (available for major pairs)
- [x] Pip precision (full precision available in responses)
- [x] Cross rates (all currency pairs available)
- [x] Historical FX rates (daily, weekly, monthly historical data)
- [ ] Intraday FX (limited - 60 days for <1d intervals)

**Supported Currency Pairs Examples:**
- EURUSD=X (Euro/US Dollar)
- GBPUSD=X (British Pound/US Dollar)
- USDJPY=X (US Dollar/Japanese Yen)
- AUDUSD=X (Australian Dollar/US Dollar)
- USDCAD=X (US Dollar/Canadian Dollar)
- USDCHF=X (US Dollar/Swiss Franc)
- NZDUSD=X (New Zealand Dollar/US Dollar)
- EURGBP=X (Euro/British Pound)
- EURJPY=X (Euro/Japanese Yen)
- And many more cross pairs

## Cryptocurrency Data

- [x] Crypto Spot Prices (via -USD, -USDT suffix symbols)
- [x] Crypto Market Cap (included in quote data)
- [x] Crypto Volume (24h volume)
- [x] Crypto Circulating Supply (available for major coins)
- [ ] Crypto Total Supply (limited availability)
- [ ] Crypto Max Supply (limited availability)
- [x] Historical Crypto Prices (limited history, typically 2-7 years depending on coin)
- [ ] Crypto Dominance (NOT available)
- [ ] Crypto Fear & Greed Index (NOT available)

**Supported Crypto Symbols Examples:**
- BTC-USD (Bitcoin)
- ETH-USD (Ethereum)
- XRP-USD (Ripple)
- ADA-USD (Cardano)
- DOGE-USD (Dogecoin)
- SOL-USD (Solana)
- MATIC-USD (Polygon)
- BNB-USD (Binance Coin)
- And hundreds more

## Commodities Data

- [x] Commodity Futures Prices (via =F suffix symbols)
- [x] Commodity Historical Data (futures contract history)
- [x] Energy Commodities (crude oil, natural gas, gasoline, heating oil)
- [x] Metals (gold, silver, copper, platinum, palladium)
- [x] Agricultural (corn, wheat, soybeans, coffee, sugar, cotton)
- [ ] Commodity Spot Prices (only futures prices available)
- [ ] Commodity Inventory Data (NOT available)

**Supported Commodity Symbols Examples:**
- GC=F (Gold Futures)
- SI=F (Silver Futures)
- CL=F (Crude Oil Futures)
- NG=F (Natural Gas Futures)
- HG=F (Copper Futures)
- PL=F (Platinum Futures)
- ZC=F (Corn Futures)
- ZW=F (Wheat Futures)
- ZS=F (Soybean Futures)
- KC=F (Coffee Futures)
- SB=F (Sugar Futures)
- CT=F (Cotton Futures)

## Indices Data

- [x] Major Global Indices (S&P 500, Nasdaq, Dow, FTSE, DAX, Nikkei, etc.)
- [x] Index Components (NOT directly available - requires separate data source)
- [x] Index Historical Data (decades of historical index values)
- [x] Index Real-Time Prices (with typical exchange delay)
- [x] Sector Indices (technology, healthcare, financials, etc.)
- [x] International Indices (global coverage)

**Supported Index Symbols Examples:**
- ^GSPC (S&P 500)
- ^DJI (Dow Jones Industrial Average)
- ^IXIC (Nasdaq Composite)
- ^RUT (Russell 2000)
- ^FTSE (FTSE 100)
- ^GDAXI (DAX)
- ^N225 (Nikkei 225)
- ^HSI (Hang Seng)
- ^BSESN (BSE Sensex)
- ^VIX (CBOE Volatility Index)

## Metadata & Reference

- [x] Symbol/Instrument Lists (via search and screener endpoints)
- [x] Exchange Information (exchange name, timezone, currency)
- [x] Market Hours (regular hours, pre-market, after-hours times)
- [x] Trading Calendars (holiday schedules embedded in data)
- [x] Timezone Info (exchange timezone data)
- [x] Sector/Industry Classifications (GICS sectors and industries)
- [x] Quote Type (equity, ETF, index, currency, cryptocurrency, future, option, etc.)
- [x] Symbol Lookup (search by name or partial ticker)
- [x] Currency Codes (ISO currency codes for all symbols)
- [x] Market State (PRE, REGULAR, POST, CLOSED)
- [x] Tradeable Status (whether symbol is actively trading)
- [x] Exchange Codes (NMS, NYQ, PCX, NGM, etc.)

## News & Sentiment (if applicable)

- [x] News Articles (headlines, summaries, links)
- [ ] Full Article Content (NOT available - links to external sites)
- [x] Press Releases (included in news feed)
- [ ] Social Sentiment (NOT available - no Twitter/Reddit sentiment)
- [ ] Analyst Reports (links only, not full reports)
- [x] News Provider Attribution (source names provided)
- [x] News Timestamps (publication dates/times)
- [x] Symbol-Specific News (news filtered by ticker)

**Note:** News is limited and not comprehensive. For dedicated news feeds, use specialized news APIs.

## ESG Data

- [x] ESG Scores (environmental, social, governance ratings)
- [x] ESG Peer Comparison (percentile rankings vs peers)
- [x] ESG Category Breakdown (detailed subcategory scores)
- [x] ESG Related Controversy (controversy scores)
- [x] ESG Performance (historical ESG score changes)

**Coverage:** Available for many large-cap stocks, limited for small-cap.

## Trending & Discovery

- [x] Trending Symbols (by region: US, GB, HK, AU, etc.)
- [x] Most Active (highest volume symbols)
- [x] Gainers (top % gainers)
- [x] Losers (top % losers)
- [x] Predefined Screeners (day gainers, day losers, most active, undervalued, growth tech, etc.)
- [x] Search/Discovery (fuzzy search by name or ticker)
- [x] Recommendations by Symbol (similar/related symbols)

## Screener Capabilities

- [x] Custom Screeners (POST endpoint with filters)
- [x] Market Cap Filters (min/max market cap)
- [x] Price Filters (min/max price)
- [x] Volume Filters (min/max volume)
- [x] Sector Filters (filter by sector)
- [x] Industry Filters (filter by industry)
- [x] Exchange Filters (filter by exchange)
- [x] Quote Type Filters (equity, ETF, etc.)
- [x] Dividend Yield Filters (min/max yield)
- [x] P/E Ratio Filters (min/max P/E)
- [x] Beta Filters (min/max beta)
- [x] Analyst Rating Filters (buy/hold/sell counts)
- [x] Pagination (up to 250 results per page)
- [x] Sorting (by any field)

## Unique/Custom Data

**What makes Yahoo Finance special as an aggregator:**

### 1. Multi-Asset Coverage
Unlike specialized providers, Yahoo Finance aggregates:
- Stocks (global, 100+ exchanges)
- Cryptocurrencies (hundreds of coins)
- Forex (all major and minor pairs)
- Commodities (futures contracts)
- Indices (global coverage)
- Options (US equity options)
- ETFs & Mutual Funds (thousands)
- Bonds (treasury yields)

**All in one free API** - no other free provider offers this breadth.

### 2. Historical Depth
Many symbols have **decades of historical data** for free:
- Major stocks: Back to IPO (often 30-50+ years)
- Indices: Back to index inception (S&P 500 to 1927)
- No historical data cost (unlike many providers)

### 3. Fundamental Data Richness
For free, Yahoo Finance provides:
- Complete financial statements (annual, quarterly, TTM)
- Insider trading details
- Institutional holdings
- SEC filing links
- Analyst estimates and ratings
- Options chains with Greeks

Most providers charge for this level of fundamental data.

### 4. Global Exchange Coverage
Supports stocks from **100+ global exchanges**:
- Americas: US, Canada, Brazil, Mexico
- Europe: UK, Germany, France, Italy, Spain, Netherlands, Switzerland
- Asia: Japan, China, Hong Kong, India, South Korea, Taiwan, Singapore
- Oceania: Australia, New Zealand
- Middle East: Saudi Arabia, UAE

### 5. Pre/Post Market Data
Provides pre-market and post-market data for US stocks:
- Pre-market price, volume, change (4:00 AM - 9:30 AM ET)
- Post-market price, volume, change (4:00 PM - 8:00 PM ET)
- Market state indicator

### 6. Real-Time WebSocket
Unlike most free providers, offers real-time WebSocket streaming:
- Sub-second latency for supported exchanges
- Protobuf-encoded for efficiency
- No explicit connection limits
- Free access

### 7. No Registration Required
Most endpoints work without:
- API key
- Account registration
- Email verification
- Credit card

**Completely anonymous access** for public data.

## Data Limitations

### What Yahoo Finance Does NOT Provide:

1. **No Level 2 Order Book** - Only best bid/ask, not full depth
2. **No Individual Trades** - Aggregated volume only, not trade-by-trade
3. **No Tick Data** - Lowest granularity is 1-minute bars (7 days only)
4. **No Derivatives Analytics** - No funding rates, OI, liquidations for crypto
5. **No On-Chain Data** - No blockchain/wallet data
6. **No Comprehensive Economic Calendar** - No scheduled release calendar
7. **No Real-Time News** - News is delayed and limited
8. **No Social Sentiment** - No Twitter/Reddit/StockTwits integration
9. **No Backtesting Features** - Just raw data, no built-in backtesting
10. **No Guaranteed Uptime** - Unofficial API, can break anytime

## Data Quality & Reliability

### Accuracy
- Source: Aggregated from exchanges and data vendors
- Validation: Generally accurate for mainstream symbols
- Corrections: Delayed (errors may persist for hours)
- **Trust Level:** High for major symbols, lower for obscure symbols

### Completeness
- Missing data: Common for small-cap/international stocks
- Gaps: Historical gaps possible for delisted symbols
- Backfill: Generally good, but not guaranteed
- **Completeness:** 95%+ for large-cap US, 70-90% for others

### Timeliness
- Latency: 15-20 seconds typical for "real-time" quotes
- Delay: Some exchanges 15-minute delayed (depends on exchange agreement)
- Market hours: Data available during and after trading hours
- **Freshness:** Good for most use cases, not suitable for HFT

## Coverage Summary by Asset Class

| Asset Class | Coverage | Historical Depth | Real-Time | Fundamentals | Options |
|-------------|----------|------------------|-----------|--------------|---------|
| US Stocks | Excellent (all major exchanges) | Decades | Yes (15-20s delay) | Yes (extensive) | Yes |
| International Stocks | Good (100+ exchanges) | Varies | Yes (may be delayed) | Limited | No |
| Crypto | Good (major coins) | 2-7 years | Yes | No | No |
| Forex | Excellent (all pairs) | Decades | Yes | N/A | No |
| Commodities | Good (major futures) | Years | Yes | No | No |
| Indices | Excellent (global) | Decades | Yes | N/A | No |
| ETFs | Excellent (US+intl) | Since inception | Yes | Yes (holdings) | Some |
| Mutual Funds | Excellent (US) | Years | No (NAV daily) | Yes (holdings) | No |
| Bonds | Limited (treasuries only) | Decades (yields) | Yes (yields) | No | No |
| Options | Excellent (US equities) | Current only | Yes | N/A | Yes (Greeks) |

## Total Symbol Coverage (Estimate)

- **Total Symbols:** 100,000+ (across all asset classes)
- **US Stocks:** ~8,000 (NYSE, Nasdaq, AMEX, OTC)
- **International Stocks:** ~40,000+ (100+ exchanges globally)
- **Cryptocurrencies:** ~500+ (major and mid-cap coins)
- **Forex Pairs:** ~100+ (majors, minors, exotics)
- **Commodities:** ~50+ (futures contracts)
- **Indices:** ~500+ (global indices)
- **ETFs:** ~10,000+ (US and international)
- **Mutual Funds:** ~20,000+ (US funds)
- **Options:** Millions (all strikes/expirations for optionable stocks)
