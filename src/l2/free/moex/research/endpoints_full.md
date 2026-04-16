# MOEX - Complete Endpoint Reference

Base URL: `https://iss.moex.com/iss`

**Format Support**: All endpoints support `.json`, `.xml`, `.csv`, `.html` formats via extension or query parameter.

## Category: Securities & Instruments

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /securities | List all securities traded on MOEX | Yes (delayed) | No | Returns all instruments |
| GET | /securities/[security] | Get instrument specification | Yes (delayed) | No | Detailed security info |
| GET | /securities/[security]/indices | List indices containing security | Yes | No | Index membership |
| GET | /securities/[security]/aggregates | Aggregated trading summaries by date/market | Yes | No | Historical aggregates |

### Parameters for Securities Endpoints
| Name | Type | Required | Description |
|------|------|----------|-------------|
| securities.columns | string | No | Comma-separated column names to filter response |
| start | int | No | Pagination offset |
| limit | int | No | Number of results (default varies by endpoint) |

## Category: Current Market Data (Real-time/Delayed)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /engines/[engine]/markets/[market]/securities | Instruments in current trading session | Yes (delayed) | No | All securities for market |
| GET | /engines/[engine]/markets/[market]/securities/[security] | Specific instrument current data | Yes (delayed) | No | Price, volume, bid/ask |
| GET | /engines/[engine]/markets/[market]/trades | All trades for market | Yes (delayed) | No | Recent trades |
| GET | /engines/[engine]/markets/[market]/orderbook | Best quotes for all instruments | Paid | Yes | Level 1 orderbook |
| GET | /engines/[engine]/markets/[market]/securities/[security]/trades | Trades for specific instrument | Yes (delayed) | No | Instrument trade history |
| GET | /engines/[engine]/markets/[market]/securities/[security]/orderbook | Order book for instrument | Paid | Yes | Requires subscription |
| GET | /engines/[engine]/markets/[market]/boards/[board]/securities | Instruments by trading mode | Yes (delayed) | No | Board-specific data |
| GET | /engines/[engine]/markets/[market]/boards/[board]/securities/[security] | Instrument data by board | Yes (delayed) | No | Board-specific details |
| GET | /engines/[engine]/markets/[market]/boards/[board]/trades | All trades for board | Yes (delayed) | No | Board trade history |
| GET | /engines/[engine]/markets/[market]/boards/[board]/orderbook | Best quotes for board | Paid | Yes | Board orderbook |
| GET | /engines/[engine]/markets/[market]/boards/[board]/securities/[security]/trades | Instrument trades by board | Yes (delayed) | No | Detailed trades |
| GET | /engines/[engine]/markets/[market]/boards/[board]/securities/[security]/orderbook | Instrument orderbook by board | Paid | Yes | Full depth |

### Common Market Data Parameters
| Name | Type | Required | Description |
|------|------|----------|-------------|
| securities.columns | string | No | Filter response columns |
| marketdata.columns | string | No | Filter market data columns |
| start | int | No | Pagination offset |
| limit | int | No | Number of results |

## Category: Historical Data (OHLC/Candles)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /engines/[engine]/markets/[market]/securities/[security]/candles | Candles by default board group | Yes | No | OHLC bars |
| GET | /engines/[engine]/markets/[market]/securities/[security]/candleborders | Date range for available candles | Yes | No | Metadata |
| GET | /engines/[engine]/markets/[market]/boards/[board]/securities/[security]/candles | Candles by trading board | Yes | No | Board-specific OHLC |
| GET | /engines/[engine]/markets/[market]/boards/[board]/securities/[security]/candleborders | Board candle date range | Yes | No | Metadata |
| GET | /engines/[engine]/markets/[market]/boardgroups/[boardgroup]/securities/[security]/candles | Candles by board group | Yes | No | Group OHLC |
| GET | /engines/[engine]/markets/[market]/boardgroups/[boardgroup]/securities/[security]/candleborders | Group candle date range | Yes | No | Metadata |

### Candles Parameters
| Name | Type | Required | Default | Description |
|------|------|----------|---------|-------------|
| from | date | Yes | - | Start date (YYYY-MM-DD) |
| till | date | No | today | End date (YYYY-MM-DD) |
| interval | int | No | 24 | Interval in minutes: 1, 10, 60, 24, 7*24, 31*24, 4*31*24 |
| start | int | No | 0 | Pagination offset |

**Interval Values**:
- 1 = 1 minute
- 10 = 10 minutes
- 60 = 1 hour
- 24 = 1 day (deprecated in favor of daily historical endpoints)
- Text codes also supported: m1, m10, H1, D1, W1, M1, Q1

## Category: Historical Trading Data

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /history/engines/[engine]/markets/[market]/securities | History for all securities by date | Yes | No | Daily data |
| GET | /history/engines/[engine]/markets/[market]/securities/[security] | Historical data for one security | Yes | No | Date range support |
| GET | /history/engines/[engine]/markets/[market]/boards/[board]/securities | Board history by date | Yes | No | Board-specific |
| GET | /history/engines/[engine]/markets/[market]/boards/[board]/securities/[security] | Security board history | Yes | No | Filtered by board |
| GET | /history/engines/[engine]/markets/[market]/boardgroups/[boardgroup]/securities | Board group history | Yes | No | Group data |
| GET | /history/engines/[engine]/markets/[market]/boardgroups/[boardgroup]/securities/[security] | Security group history | Yes | No | Filtered by group |
| GET | /history/engines/[engine]/markets/[market]/dates | Available historical dates | Yes | No | Metadata |
| GET | /history/engines/[engine]/markets/[market]/securities/[security]/dates | Security-specific date ranges | Yes | No | Metadata |

### Historical Data Parameters
| Name | Type | Required | Description |
|------|------|----------|-------------|
| date | date | No | Specific date (YYYY-MM-DD) |
| from | date | No | Start date |
| till | date | No | End date |
| start | int | No | Pagination offset |
| limit | int | No | Number of records |

## Category: Historical Sessions (Stock Market)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /history/engines/[engine]/markets/[market]/sessions | Available sessions | Yes | No | Stock market only |
| GET | /history/engines/[engine]/markets/[market]/sessions/[session]/securities | All securities for session/date | Yes | No | Session data |
| GET | /history/engines/[engine]/markets/[market]/sessions/[session]/securities/[security] | Security session history | Yes | No | Detailed |
| GET | /history/engines/[engine]/markets/[market]/sessions/[session]/boards/[board]/securities | Board session data | Yes | No | By board |
| GET | /history/engines/[engine]/markets/[market]/sessions/[session]/boards/[board]/securities/[security] | Security board session | Yes | No | Filtered |

## Category: Historical Yields (Bonds)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /history/engines/[engine]/markets/[market]/yields | Calculated yields by date | Yes | No | Bond data |
| GET | /history/engines/[engine]/markets/[market]/yields/[security] | Yield history for security | Yes | No | Time series |
| GET | /history/engines/[engine]/markets/[market]/boards/[board]/yields | Yields by board | Yes | No | Board-specific |
| GET | /history/engines/[engine]/markets/[market]/boards/[board]/yields/[security] | Board yield history | Yes | No | Filtered |

## Category: Archives (Bulk Downloads)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /archives/engines/[engine]/markets/[market]/[datatype]/years | Years with archive files | Yes | No | datatype: securities or trades |
| GET | /archives/engines/[engine]/markets/[market]/[datatype]/[period] | Archive file links | Yes | No | period: yearly, monthly, daily |
| GET | /archives/engines/[engine]/markets/[market]/[datatype]/years/[year]/months | Months with archives | Yes | No | Monthly availability |

**Note**: Monthly archives only available for last 30 days.

## Category: Listings (Non-traded Instruments)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /history/engines/[engine]/markets/[market]/listing | Non-traded instruments | Yes | No | IPO, delisted |
| GET | /history/engines/[engine]/markets/[market]/boards/[board]/listing | Listing by board | Yes | No | Board-specific |
| GET | /history/engines/[engine]/markets/[market]/boardgroups/[boardgroup]/listing | Listing by group | Yes | No | Group-specific |

## Category: Market Turnovers & Statistics

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /turnovers | Aggregate market turnovers | Yes | No | All markets |
| GET | /engines/[engine]/turnovers | Current session turnovers by market | Yes | No | Engine-specific |
| GET | /engines/[engine]/markets/[market]/turnovers | Market turnover value | Yes | No | Single market |
| GET | /engines/[engine]/markets/[market]/secstats | Intermediate daily results | Yes | No | Intraday stats |

## Category: Indices

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /statistics/engines/stock/markets/index/analytics | Stock market indices | Yes | No | All indices |
| GET | /statistics/engines/stock/markets/index/analytics/[indexid] | Index analytical data by date | Yes | No | Time series |
| GET | /statistics/engines/stock/markets/index/analytics/[indexid]/tickers | All tickers for index | Yes | No | Constituents |
| GET | /statistics/engines/stock/markets/index/analytics/[indexid]/tickers/[ticker] | Ticker information | Yes | No | Component details |
| GET | /statistics/engines/stock/markets/index/bulletins | Index bulletins | Yes | No | Official reports |

## Category: Derivatives Analytics

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /statistics/engines/futures/markets/options/assets | Option series list | Yes | No | All option series |
| GET | /statistics/engines/futures/markets/options/assets/[asset] | Specific option series | Yes | No | Series details |
| GET | /statistics/engines/futures/markets/options/assets/[asset]/volumes | Option series volume | Yes | No | Trading volumes |
| GET | /statistics/engines/futures/markets/options/assets/[asset]/turnovers | Option series turnover | Yes | No | Value traded |
| GET | /statistics/engines/futures/markets/options/assets/[asset]/openpositions | Option open positions | Yes | No | Open interest |
| GET | /statistics/engines/futures/markets/options/assets/[asset]/optionboard | Option board data | Yes | No | Full option chain |
| GET | /statistics/engines/futures/markets/forts/series | Futures list | Yes | No | All futures |
| GET | /statistics/engines/futures/markets/[market]/openpositions | Base assets for open positions | Yes | No | OI by asset |
| GET | /statistics/engines/futures/markets/[market]/openpositions/[asset] | Open positions for asset | Yes | No | Detailed OI |
| GET | /analyticalproducts/futoi/securities | Futures open interest all | Yes | No | Aggregated |
| GET | /analyticalproducts/futoi/securities/[security] | Futures OI for security | Yes | No | Specific instrument |

## Category: Currency & Rates

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /statistics/engines/currency/markets/selt/rates | Central Bank rates | Yes | No | Official CBR rates |
| GET | /statistics/engines/currency/markets/fixing | MOEX fixings | Yes | No | Daily fixings |
| GET | /statistics/engines/currency/markets/fixing/[security] | Security fixings | Yes | No | Specific pair |
| GET | /statistics/engines/futures/markets/indicativerates/securities | Indicative forex rates | Yes | No | All pairs |
| GET | /statistics/engines/futures/markets/indicativerates/securities/[security] | Indicative rate | Yes | No | Specific rate |

## Category: Yield Curves (Zero-Coupon)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /engines/[engine]/markets/zcyc | Zero-coupon yield curve | Yes | No | Bond curves |
| GET | /engines/[engine]/zcyc | Yield curve data | Yes | No | Alternative endpoint |
| GET | /sdfi/curves | Swap curves reference | Yes | No | SDFI market |
| GET | /sdfi/curves/[curveid] | Specific swap curve | Yes | No | Curve details |

## Category: Stock Market Statistics

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /statistics/engines/stock/markets/shares/correlations | Stock correlation coefficients | Yes | No | Market correlations |
| GET | /statistics/engines/stock/splits | Stock splits and consolidations | Yes | No | Corporate actions |
| GET | /statistics/engines/stock/splits/[security] | Security split data | Yes | No | Specific stock |
| GET | /statistics/engines/stock/deviationcoeffs | Deviation criteria indicators | Yes | No | Volatility metrics |
| GET | /statistics/engines/stock/quotedsecurities | Securities with market quotes | Yes | No | Active securities |
| GET | /statistics/engines/stock/currentprices | Current security prices | Yes | No | Latest prices |
| GET | /statistics/engines/stock/capitalization | Market capitalization | Yes | No | Total market cap |
| GET | /statistics/engines/stock/markets/bonds/monthendaccints | Month-end accrued interest | Yes | No | Bond interest |
| GET | /statistics/engines/stock/markets/bonds/aggregates | Bond market aggregates | Yes | No | Summary stats |

## Category: Aggregated Totals

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /history/engines/stock/totals/boards | Boards for aggregated info | Yes | No | Available boards |
| GET | /history/engines/stock/totals/securities | Aggregated stock data | Yes | No | All securities |
| GET | /history/engines/stock/totals/boards/[board]/securities | Aggregated by board | Yes | No | Board-specific |
| GET | /history/engines/stock/totals/boards/[board]/securities/[security] | Aggregated security data | Yes | No | Specific instrument |

## Category: Analytics Products

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /analyticalproducts/netflow2/securities | Net flow for all securities | Yes | No | Since 2007 for major stocks |
| GET | /analyticalproducts/netflow2/securities/[security] | Net flow for security | Yes | No | Money flow analysis |

## Category: OTC Markets

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /history/otc/providers/nsd/markets | OTC aggregate markets list | Yes | No | NSD provider |
| GET | /history/otc/providers/nsd/markets/[market]/daily | Daily OTC data | Yes | No | Daily aggregates |
| GET | /history/otc/providers/nsd/markets/[market]/monthly | Monthly OTC data | Yes | No | Monthly aggregates |

## Category: Reference Data 2.0

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /referencedata/engines/[engine]/markets/all/securitieslisting | Trading availability | Yes | No | Complete reference |
| GET | /referencedata/engines/stock/markets/all/securities | Complete instrument list | Yes | No | All stock securities |
| GET | /referencedata/engines/stock/markets/all/shorts | Short instruments list | Yes | No | Shortable stocks |
| GET | /referencedata/engines/futures/markets/[market]/securities | Derivatives instruments | Yes | No | Futures/options list |
| GET | /referencedata/engines/futures/markets/[market]/params | Derivatives parameters | Yes | No | Contract specs |
| GET | /referencedata/engines/futures/markets/[market]/risks | Derivatives risk parameters | Yes | No | Margin requirements |

## Category: Trading Systems & Markets Metadata

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /index | Global ISS reference directories | Yes | No | API metadata |
| GET | /engines | Available trading systems | Yes | No | All engines |
| GET | /engines/[engine] | Trading system description | Yes | No | Engine details |
| GET | /engines/[engine]/markets | Markets within trading system | Yes | No | Available markets |
| GET | /engines/[engine]/markets/[market] | Market description | Yes | No | Market details |
| GET | /engines/[engine]/markets/[market]/boards | Trading mode reference | Yes | No | All boards |
| GET | /engines/[engine]/markets/[market]/boards/[board] | Trading mode description | Yes | No | Board details |
| GET | /engines/[engine]/markets/[market]/boardgroups | Board group reference | Yes | No | All groups |
| GET | /engines/[engine]/markets/[market]/boardgroups/[boardgroup] | Board group description | Yes | No | Group details |

## Category: Security Groups & Collections

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /securitygroups | Security groups listing | Yes | No | All groups |
| GET | /securitygroups/[securitygroup] | Specific security group | Yes | No | Group details |
| GET | /securitygroups/[securitygroup]/collections | Collections within group | Yes | No | Group collections |
| GET | /securitygroups/[securitygroup]/collections/[collection] | Specific collection | Yes | No | Collection details |
| GET | /securitygroups/[securitygroup]/collections/[collection]/securities | Instruments in collection | Yes | No | Collection members |

## Category: Corporate Information Services (CCI)

### Company Information
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/info-nsd/companies | Organizations reference | Yes | No | All companies |
| GET | /cci/info-nsd/companies/[company] | Organization info | Yes | No | Company details |
| GET | /cci/info/companies | Enhanced org reference | Yes | No | Full data |
| GET | /cci/info/companies/[company] | Enhanced org info | Yes | No | Search by ID/INN/OGRN |
| GET | /cci/info/companies/industry-codes | Industry classifications | Yes | No | Sector codes |

### Financial Reports - IFRS
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/accounting/msfo-full/reports | IFRS full reports | Yes | No | All reports |
| GET | /cci/accounting/msfo-full/periods/[period]/reports | IFRS by period | Yes | No | Filtered by period |
| GET | /cci/accounting/msfo-full/companies/[company]/reports | Company IFRS reports | Yes | No | Company-specific |
| GET | /cci/accounting/msfo-full/companies/[company]/periods/[period]/reports | Company IFRS by period | Yes | No | Filtered |
| GET | /cci/accounting/msfo-full/indicators | IFRS indicators | Yes | No | Financial metrics |
| GET | /cci/accounting/msfo-full/companies/[company]/indicators | Company IFRS indicators | Yes | No | Company metrics |
| GET | /cci/accounting/msfo-full/industry-indicators/reports | Industry-average IFRS | Yes | No | Sector benchmarks |

### Financial Reports - Russian Accounting (RSBU)
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/accounting/rsbu/reports | Russian accounting reports | Yes | No | All RSBU reports |
| GET | /cci/accounting/rsbu/periods/[period]/reports | RSBU by period | Yes | No | Period filtered |
| GET | /cci/accounting/rsbu/companies/[company]/reports | Company RSBU reports | Yes | No | Company-specific |

### Credit Ratings
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/rating/companies | Current org ratings | Yes | No | Latest ratings |
| GET | /cci/rating/companies/[company_id] | Organization ratings | Yes | No | Search by ID/INN/OGRN |
| GET | /cci/rating/history/companies | Historical org ratings | Yes | No | Rating history |
| GET | /cci/rating/securities | Current security ratings | Yes | No | Bond ratings |
| GET | /cci/rating/securities/[security_id] | Security ratings | Yes | No | Specific issue |
| GET | /cci/rating/agg/companies | Aggregated org ratings | Yes | No | Consolidated ratings |
| GET | /cci/rating/agg/securities | Aggregated security ratings | Yes | No | Consolidated bond ratings |

### Corporate Actions
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/corp-actions | Corporate actions | Yes | No | All actions |
| GET | /cci/corp-actions/[corp_action_id] | Specific action | Yes | No | Action details |
| GET | /cci/corp-actions/meetings | Meeting actions | Yes | No | Shareholder meetings |
| GET | /cci/corp-actions/coupons | Coupon actions | Yes | No | Bond coupons |
| GET | /cci/corp-actions/dividends | Dividend actions | Yes | No | Dividend payments |

### Affiliate Reporting
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/reporting/affiliates/reports | Affiliate reports | Yes | No | All reports |
| GET | /cci/reporting/affiliates/companies/[company]/reports | Company affiliate reports | Yes | No | Company-specific |

### Corporate Information
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/reporting/corp-info/reports | Corporate info reports | Yes | No | All reports |
| GET | /cci/reporting/corp-info/companies/[company]/reports | Company corp info | Yes | No | Company-specific |

### Consensus Forecasts
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/consensus/shares-price | Share price forecasts | Yes | No | Analyst consensus |
| GET | /cci/consensus/shares-price/[security] | Security consensus | Yes | No | Specific stock |

### IR Calendar
| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /cci/calendars/ir-calendar | IR event calendar | Yes | No | Upcoming events |

## Category: News & Events

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /sitenews | Exchange news | Yes | No | All news items |
| GET | /sitenews/[news_id] | Specific news item | Yes | No | News details |
| GET | /events | Exchange events | Yes | No | All events |
| GET | /events/[event_id] | Specific event | Yes | No | Event content |

## Category: Risk Management (RMS)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /rms/engines/[engine]/objects/irr | Risk indicators | Yes | No | Interest rate risk |
| GET | /rms/engines/[engine]/objects/settlementscalendar | Settlement calendar | Yes | No | Trading days |
| GET | /rms/engines/[engine]/objects/[object] | Risk parameters | Yes | No | Static and dynamic |

## Category: Collateral & State Rates

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /statistics/engines/[engine]/markets/[market] | Collateral revaluation rates | Yes | No | Margin rates |
| GET | /statistics/engines/[engine]/markets/[market]/securities | Collateral by instrument | Yes | No | Security-specific |
| GET | /statistics/engines/state/rates | State rates data | Yes | No | Government rates |
| GET | /statistics/engines/state/markets/repo/cboper | Central bank rates | Yes | No | CB weighted rates |
| GET | /statistics/engines/state/markets/repo/dealers | Repo dealers data | Yes | No | Dealer information |

## Category: Field Descriptions (Metadata)

| Method | Endpoint | Description | Free? | Auth? | Notes |
|--------|----------|-------------|-------|-------|-------|
| GET | /turnovers/columns | Turnover field descriptions | Yes | No | Column metadata |
| GET | /engines/[engine]/markets/[market]/securities/columns | Securities field descriptions | Yes | No | Data dictionary |
| GET | /engines/[engine]/markets/[market]/trades/columns | Trades field descriptions | Yes | No | Trade fields |
| GET | /engines/[engine]/markets/[market]/orderbook/columns | Orderbook field descriptions | Yes | No | Orderbook fields |
| GET | /history/engines/[engine]/markets/[market]/listing/columns | Listing field descriptions | Yes | No | Listing fields |

## Common URL Patterns

**Engine values**: stock, state, currency, futures, commodity, interventions, offboard, agro, otc, quotes, money

**Market values** (examples):
- Stock engine: shares, bonds, ndm, otc, ccp, index
- Currency engine: selt, fixing
- Futures engine: forts, options

**Board values**: TQBR (T+ main board), SMAL (small cap), TQTF (ETFs), etc. (500+ boards total)

**Data type values** (archives): securities, trades

**Period values** (archives): yearly, monthly, daily

## Response Format Examples

### Example: Get candles for SBER stock
```
GET /iss/engines/stock/markets/shares/boards/TQBR/securities/SBER/candles.json?from=2026-01-20&interval=60
```

### Example: Get current market data for all shares
```
GET /iss/engines/stock/markets/shares/securities.json?securities.columns=SECID,SHORTNAME,PREVPRICE,LAST,CHANGE
```

### Example: Get historical data for GAZP
```
GET /iss/history/engines/stock/markets/shares/boards/TQBR/securities/GAZP.json?from=2026-01-01&till=2026-01-26
```

### Example: Get list of available engines
```
GET /iss/engines.json
```

## Notes on Pagination

Most endpoints support pagination via `start` parameter (offset). Default page size varies by endpoint. Use `start=100` to get next page, `start=200` for third page, etc.

## Notes on Column Filtering

Use `.columns` parameter to filter response fields:
- `securities.columns=SECID,SHORTNAME,LAST`
- `marketdata.columns=LAST,BID,ASK,VOLUME`
- Multiple column sets can be filtered in same request

## Total Endpoint Count

**~400+ unique endpoint patterns** across all categories, with extensive parameterization creating thousands of possible queries.
