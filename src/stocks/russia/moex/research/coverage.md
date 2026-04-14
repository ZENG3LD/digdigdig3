# MOEX - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: Limited (only US stocks traded on MOEX)
- **Europe**: Limited (European stocks traded on MOEX)
- **Asia**: Yes (Russia, limited Asian stocks)
- **Russia**: Primary focus (comprehensive coverage)
- **CIS**: Yes (Kazakhstan, Belarus, other CIS countries via MOEX)

### Country-Specific
- **Russia**: Yes (primary market, comprehensive)
- **US**: Limited (ADRs/foreign stocks traded on MOEX)
- **UK**: Limited (foreign stocks traded on MOEX)
- **Japan**: Limited (foreign stocks traded on MOEX)
- **China**: Limited (foreign stocks traded on MOEX)
- **Germany**: Limited (foreign stocks traded on MOEX)
- **CIS Countries**: Yes (Kazakhstan, Belarus, etc.)

### Restricted Regions
- **Blocked countries**: None (data access not geo-restricted)
- **VPN detection**: Not applicable (public data API)
- **Geo-fencing**: No restrictions on data access
- **Trading restrictions**: Trading may be restricted for non-Russian residents (not covered in this research)

**Note**: ISS market data API is accessible globally. Trading restrictions may apply for actual trading APIs.

## Markets/Exchanges Covered

### Stock Markets
**MOEX is the exchange**, not an aggregator. Coverage includes:

- **Russia**: Moscow Exchange (MOEX) - Primary market
  - Main board (T+ settlement)
  - Small cap board
  - Preferred shares
  - ADRs/GDRs traded on MOEX
  - Foreign stocks listed on MOEX
- **Foreign stocks**: Limited to those traded on MOEX

**Not covered** (other Russian exchanges):
- St. Petersburg Stock Exchange (SPB Exchange) - Not included
- Other regional exchanges - Not included

### Trading Boards
**500+ boards** across different markets:
- **TQBR**: T+ main board (Акции и ДР)
- **SMAL**: Small cap equities
- **TQTF**: Exchange-traded funds (ETFs)
- **TQIF**: Investment funds
- **TQOB**: Corporate bonds
- **TQCB**: Government bonds (OFZ)
- **SPEQ**: Special quotation segment
- Many others (settlement modes, currency denominations, etc.)

### Bond Markets
- **Government bonds** (OFZ - Облигации федерального займа)
- **Corporate bonds**
- **Municipal bonds**
- **Eurobonds**
- **Structured notes**

### Currency Markets
- **FX Spot** (SELT market)
- **Currency forwards**
- **Currency swaps**

### Derivatives Markets
- **Futures** (FORTS market)
  - Index futures (IMOEX, RTSI)
  - Currency futures (USD, EUR, CNY)
  - Commodity futures (gold, silver, oil)
  - Stock futures
- **Options**
  - Index options
  - Stock options
  - Currency options
  - Commodity options

### Commodities
- **Precious metals** (gold, silver)
- **Agricultural products** (Agro market)
- **Energy** (limited)

### Money Market
- **Repo operations**
- **Deposits**
- **Interbank lending**

### OTC Markets
- **OTC with CCP** (central counterparty)
- **NSD OTC** (National Settlement Depository)

## Instrument Coverage

### Stocks
- **Total symbols**: ~700-800 stocks actively traded
- **Russian stocks**: ~600-700 (majority)
- **Foreign stocks**: ~100-200 (ADRs, foreign listings on MOEX)
- **Blue chips**: ~50-100 (highly liquid)
- **Mid-caps**: ~200-300
- **Small-caps**: ~300-400 (including SMAL board)
- **OTC**: Limited OTC stocks
- **Penny stocks**: Yes (low-priced stocks available)

**Major Russian stocks**:
- SBER (Sberbank)
- GAZP (Gazprom)
- LKOH (Lukoil)
- ROSN (Rosneft)
- YNDX (Yandex)
- GMKN (Norilsk Nickel)
- MGNT (Magnit)
- VTBR (VTB Bank)
- ALRS (Alrosa)
- NVTK (Novatek)
- TATN (Tatneft)
- AFLT (Aeroflot)

### Bonds
- **Total bonds**: ~1000+ issues
- **Government bonds (OFZ)**: ~50-100 series
- **Corporate bonds**: ~800-900 issues
- **Municipal bonds**: ~50-100 issues
- **Eurobonds**: ~100-200 issues
- **Maturity range**: Short-term (< 1 year) to long-term (20+ years)

### Derivatives
- **Futures**: ~150-200 active contracts
  - Index futures: 10-20
  - Stock futures: 50-100
  - Currency futures: 10-20
  - Commodity futures: 10-20
- **Options**: ~500-1000 active series
  - Stock options: Multiple strikes and expirations
  - Index options: IMOEX, RTSI
  - Currency options: USD, EUR

### ETFs
- **Total ETFs**: ~20-50 funds
- **Equity ETFs**: Russian market, sector ETFs
- **Bond ETFs**: Government and corporate bond funds
- **Commodity ETFs**: Gold, silver funds

### Currencies
- **Currency pairs**: ~20-30 pairs
- **Majors**: USD/RUB, EUR/RUB, EUR/USD
- **Emerging**: CNY/RUB, HKD/RUB, KZT/RUB, BYN/RUB
- **Exotics**: Limited exotic pairs

### Indices
- **Total indices**: ~100+ indices
- **Main indices**:
  - IMOEX (Moscow Exchange Index) - Main benchmark
  - RTSI (RTS Index) - Dollar-denominated
- **Sector indices**: Financials, Energy, Consumer, Telecom, Metals, etc.
- **Bond indices**: Government bond indices, corporate bond indices
- **Volatility indices**: Russian market volatility
- **Custom indices**: Thematic and strategic indices

## Data History

### Historical Depth
- **Russian stocks**: From 1997+ for major indices (IMOEX, RTSI)
  - Blue chips: 1997-2000+
  - Mid-caps: 2000-2010+
  - Small-caps: 2010+
  - Varies by listing date
- **Bonds**: From listing date (varies widely)
- **Derivatives**: From 2001+ (FORTS market launch)
- **Currency**: From 2000+
- **Indices**: From 1997+ (IMOEX, RTSI)

### Specific Historical Depths
- **IMOEX Index**: From September 22, 1997
- **RTSI Index**: From September 1, 1995
- **Major stocks** (SBER, GAZP, LKOH): 20+ years
- **Net Flow 2 analytics**: From 2007 for select stocks (SBER, GAZP, LKOH, GMKN, MGNT, ROSN, VTBR, ALRS, SBERP, AFLT)

### Granularity Available
- [x] **Tick data**: Via Full Order Book product (institutional, paid)
- [x] **1-minute bars**: Yes (via candles endpoint)
- [x] **10-minute bars**: Yes
- [x] **Hourly bars**: Yes
- [x] **Daily bars**: Yes (extensive history, 20+ years for major stocks)
- [x] **Weekly bars**: Yes
- [x] **Monthly bars**: Yes
- [x] **Quarterly bars**: Yes

### Historical Data Access
- **Free tier**: Unlimited historical depth (all history available)
- **Paid tier**: Same as free (no additional historical data)
- **Archives**: Bulk downloads available (yearly, monthly, daily)
- **Backfill**: Historical data corrections applied by exchange

## Real-time vs Delayed

### Real-time (Paid Subscription)
- **Market data**: 0 delay (sub-second latency)
- **Trades**: Real-time
- **Quotes**: Real-time
- **Orderbook**: Real-time (10x10 or 5x5 depth)
- **Indices**: 1-second calculation intervals (real-time)
- **Latency**: < 1 second typical

### Delayed (Free Tier)
- **Market data**: 15-minute delay
- **Trades**: 15-minute delay
- **Quotes**: 15-minute delay
- **Orderbook**: Not available (requires subscription)
- **Indices**: 15-minute delay
- **End-of-day data**: No delay (available after market close)

### Snapshot
- **Intraday snapshots**: Available via REST API
- **Real-time snapshots**: For paid subscribers
- **Delayed snapshots**: For free tier (15-minute old)

## Update Frequency

### Real-time Streams (WebSocket, Paid)
- **Price updates**: Sub-second (as trades occur)
- **Orderbook**: Continuous updates (snapshot + delta or full)
- **Trades**: Real-time (as executed)
- **Indices**: 1-second calculation intervals (IMOEX: "1s" frequency)
- **Market statistics**: Per-trade updates

### Scheduled Updates
- **End-of-day data**: Daily after market close (18:40 MSK)
- **Historical data**: Daily updates (overnight processing)
- **Corporate actions**: As announced (real-time)
- **Financial reports**: Quarterly, semi-annually, annually (as published)
- **Credit ratings**: As updated by rating agencies
- **News and events**: Real-time

### Polling Intervals (REST API)
- **Free tier**: Recommend 1 request per minute per symbol (to avoid rate limits)
- **Paid tier**: Can poll more frequently (rate limits not documented)
- **Best practice**: Use WebSocket for real-time, REST for historical

## Data Quality

### Accuracy
- **Source**: Direct from Moscow Exchange (authoritative source)
- **Validation**: Exchange-validated data
- **Corrections**: Official exchange corrections applied
- **Reconciliation**: Daily end-of-day reconciliation
- **Audit trail**: Exchange-grade data integrity

### Completeness
- **Missing data**: Rare (exchange data is complete)
- **Gaps**: Only during:
  - Market closures (holidays, weekends)
  - Trading halts (circuit breakers, security-specific halts)
  - Technical issues (rare)
- **Backfill**: Historical corrections applied retroactively
- **Corporate actions**: Fully tracked (splits, dividends, coupons)

### Timeliness
- **Real-time latency**: < 1 second for paid subscribers
- **Delayed latency**: 15 minutes for free tier
- **Historical data**: Updated daily (overnight)
- **Market hours**: Fully covered (09:50 - 18:40 MSK main session)
- **Pre-market/After-hours**: Limited (not primary focus)

### Reliability
- **Uptime**: 99%+ (exchange-grade infrastructure)
- **Data integrity**: Exchange-validated (high reliability)
- **Disaster recovery**: Exchange-level redundancy
- **Consistency**: Multi-block response format ensures consistency
- **Version tracking**: `dataversion` block tracks data versions

## Trading Hours & Sessions

### Main Trading Session
- **Regular hours**: 09:50 - 18:40 MSK (Moscow Time, UTC+3)
- **Pre-market**: 09:30 - 09:50 MSK (limited)
- **After-hours**: 18:50 - 19:00 MSK (limited, closing auction)

### Market Sessions
- **Normal session** (N): Main trading session
- **Opening auction**: 09:50
- **Closing auction**: 18:40
- **Evening session**: Limited (bonds, derivatives)

### Holidays
- **Trading calendar**: Available via API (`/iss/rms/engines/[engine]/objects/settlementscalendar`)
- **Russian holidays**: Moscow Exchange follows Russian federal holidays
- **Half-days**: Rare (pre-holiday shortened sessions)

### Timezone
- **Moscow Time**: UTC+3 (no daylight saving since 2014)
- **Fixed timezone**: Year-round UTC+3

## Market Segments

### Equities Market (shares)
- **Total listed**: ~700-800 stocks
- **Blue chips**: ~50-100 (high liquidity)
- **Mid-caps**: ~200-300
- **Small-caps**: ~300-400
- **Foreign stocks**: ~100-200

### Bond Market
- **Government bonds (OFZ)**: ~50-100 series
- **Corporate bonds**: ~800-900 issues
- **Municipal bonds**: ~50-100 issues
- **Total bond market**: ~1000+ issues

### Derivatives Market (FORTS)
- **Futures**: ~150-200 active contracts
- **Options**: ~500-1000 active series
- **Total derivatives**: ~1000+ instruments

### Currency Market (SELT)
- **Currency pairs**: ~20-30 pairs
- **Daily turnover**: Significant (multi-billion USD equivalent)

### Money Market
- **Repo operations**: Active interbank market
- **Deposits**: Money market deposits
- **Turnover**: Significant institutional activity

## Liquidity Profile

### Highly Liquid
- **Blue chip stocks**: SBER, GAZP, LKOH, ROSN, YNDX (millions of shares daily)
- **OFZ bonds**: Government bonds (high liquidity)
- **Index futures**: IMOEX, RTSI futures (very active)
- **Major currency pairs**: USD/RUB, EUR/RUB (high volumes)

### Moderately Liquid
- **Mid-cap stocks**: Moderate daily volumes (thousands to hundreds of thousands)
- **Corporate bonds**: Varies by issuer (blue chip corporates liquid)
- **Stock futures**: Individual stock futures (moderate activity)

### Low Liquidity
- **Small-cap stocks**: Low daily volumes (may have wide spreads)
- **Municipal bonds**: Limited trading
- **Exotic currency pairs**: Low volumes
- **OTC instruments**: Limited liquidity

## Data Coverage by Engine

### Stock Engine (1)
- **Markets**: shares, bonds, ndm, otc, ccp, index
- **Instruments**: ~2000+ (stocks, bonds, ETFs, indices)
- **History**: From 1997+ for major instruments

### Currency Engine (3)
- **Markets**: selt, fixing
- **Instruments**: ~20-30 currency pairs
- **History**: From 2000+

### Futures Engine (4)
- **Markets**: forts, options
- **Instruments**: ~1000+ (futures + options)
- **History**: From 2001+

### State Engine (2)
- **Markets**: Government securities placement
- **Instruments**: OFZ, government bills
- **History**: Extensive

### Commodity Engine (5)
- **Markets**: Commodity trading
- **Instruments**: Agricultural products, limited commodities
- **History**: From market launch

### Others
- Money market, OTC, Agro, Interventions, Quotes (specialized markets)

## Corporate Information Coverage

### Company Data
- **Total companies**: ~700+ issuers
- **IFRS reports**: Major companies (quarterly, annual)
- **RSBU reports**: Russian accounting (all Russian companies)
- **Credit ratings**: Company and security level
- **Corporate actions**: Comprehensive (dividends, splits, meetings, coupons)

### Financial Statements
- **IFRS**: International standards (full and short versions)
- **RSBU**: Russian accounting standards
- **Frequency**: Quarterly, semi-annually, annually
- **Historical depth**: Multi-year history
- **Industry averages**: IFRS industry indicators available

### Credit Ratings
- **Rating agencies**: Multiple agencies (Russian and international)
- **Company ratings**: Current and historical
- **Security ratings**: Bond issue ratings
- **Aggregated ratings**: Consolidated view from multiple agencies

## Summary

### Geographic Coverage
- **Primary**: Russia (comprehensive)
- **Secondary**: CIS countries (limited)
- **Foreign**: Limited to foreign stocks traded on MOEX
- **Access**: Global (no geo-restrictions on data API)

### Market Coverage
- **Single exchange**: Moscow Exchange (not an aggregator)
- **Multi-asset**: Equities, bonds, derivatives, FX, commodities, money market
- **Comprehensive**: 11 engines, 120+ markets, 500+ boards
- **Total instruments**: ~3000+ across all asset classes

### Historical Coverage
- **Depth**: From 1997+ for major indices, varies by instrument
- **Granularity**: Tick to quarterly bars
- **Free access**: Unlimited historical data (no cost)
- **Archives**: Bulk downloads available

### Data Quality
- **Source**: Exchange-authoritative
- **Reliability**: 99%+ uptime expected
- **Completeness**: Comprehensive (exchange-validated)
- **Timeliness**: Real-time (paid) or 15-minute delay (free)

### Strengths
- Comprehensive Russian market coverage
- Extensive free historical data
- Multi-asset class support
- High data quality (exchange-grade)
- Rich corporate information (CCI)

### Limitations
- Single exchange (MOEX only, not aggregated)
- 15-minute delay for free tier
- Limited foreign stock coverage
- Documentation mostly in Russian
- Rate limits not documented
