# Futu OpenAPI - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: Yes (US, Canada)
- **Europe**: No (not currently supported)
- **Asia**: Yes (Hong Kong, China A-shares, Singapore, Japan)
- **Oceania**: Yes (Australia)
- **Southeast Asia**: Yes (Malaysia)
- **Other**: No

### Country-Specific Markets

| Country | Market Code | Supported | Account Required | Trading |
|---------|-------------|-----------|-----------------|---------|
| **Hong Kong** | HK | Yes | Futubull/moomoo HK | Yes |
| **United States** | US | Yes | Futubull US/moomoo US | Yes |
| **China** | CN (SH/SZ) | Yes (A-shares) | China Connect access | Yes (limited) |
| **Singapore** | SG | Yes (futures) | moomoo SG | Yes (futures) |
| **Japan** | JP | Yes (futures) | Approved account | Yes (futures) |
| **Australia** | AU | Yes | moomoo AU | Yes |
| **Malaysia** | MY | Yes | moomoo MY | Yes |
| **Canada** | CA | Yes | moomoo CA | Yes |

### Restricted Regions
- **Blocked countries**: Varies by regulatory requirements (check with broker)
- **VPN detection**: Not explicitly implemented, but account must match jurisdiction
- **Geo-fencing**: Account opening restricted by country (must comply with local regulations)
- **Trading restrictions**: US citizens cannot open certain non-US accounts, vice versa

### Data Access by Region
- **Hong Kong users**: Access HK, US, A-shares (with appropriate subscriptions)
- **US users**: Access US, HK (with appropriate subscriptions)
- **China mainland users**: Special benefits (free HK LV2, free A-share LV1)
- **Other regions**: Depends on account type and broker entity

## Markets/Exchanges Covered

### Stock Markets

| Market | Exchanges | Supported | Quote Levels | Trading |
|--------|-----------|-----------|--------------|---------|
| **US** | NYSE, NASDAQ, AMEX | Yes | LV1 (free), TotalView (paid) | Yes |
| **Hong Kong** | HKEX (Main Board, GEM) | Yes | LV1 (free), LV2 (free mainland/paid) | Yes |
| **China A-Shares** | SSE, SZSE (via Connect) | Yes | LV1 (free mainland/paid) | Yes (via Connect) |
| **Australia** | ASX | Yes | Standard | Yes |
| **Malaysia** | Bursa Malaysia | Yes | Standard | Yes |
| **Canada** | TSX, TSXV | Yes | Standard | Yes |

**Not Supported**:
- European exchanges (LSE, Euronext, DAX, etc.)
- Japanese stocks (only futures)
- Singaporean stocks (only futures)
- Other Asian markets (Korea, Taiwan, India, etc.)
- Latin American markets
- Middle Eastern markets
- African markets

### Futures Exchanges

| Market | Exchanges | Supported | Coverage |
|--------|-----------|-----------|----------|
| **Hong Kong** | HKEX | Yes | HSI, MHI, HHI futures |
| **US** | CME, CBOT, NYMEX, COMEX | Yes | Index, commodity, currency futures |
| **Singapore** | SGX | Yes | Nikkei, FTSE China A50 futures |
| **Japan** | OSE, TSE | Yes | Nikkei futures |

**CME Group Products**:
- E-mini S&P 500 (ES)
- E-mini NASDAQ (NQ)
- E-mini Dow (YM)
- E-mini Russell 2000 (RTY)
- Crude Oil (CL)
- Gold (GC)
- Natural Gas (NG)
- Eurodollar, Treasury futures
- Currency futures (EUR, JPY, GBP, etc.)

### Options Exchanges

| Market | Exchanges | Supported | Coverage |
|--------|-----------|-----------|----------|
| **US** | CBOE, ISE, AMEX, etc. | Yes | Equity options, ETF options, Index options |
| **Hong Kong** | HKEX | Yes | Equity options (select stocks) |

**US Options**:
- Equity options (AAPL, GOOGL, TSLA, etc.)
- ETF options (SPY, QQQ, IWM, etc.)
- Index options (SPX, NDX, RUT)
- Weekly options (standard and weeklies)

**HK Options**:
- HSI options (Hang Seng Index)
- Equity options (00700, 00941, etc.)
- Limited underlying securities

### Crypto Exchanges
**Not Supported**: Futu does not support cryptocurrency markets. For crypto, use exchanges like Binance, Coinbase, Kraken.

### Forex Brokers
**Not Directly Supported**: Futu does not provide spot forex (EUR/USD, etc.). However, currency futures are available via CME.

## Instrument Coverage

### Stocks

| Market | Total Symbols | Coverage |
|--------|--------------|----------|
| **US Stocks** | ~8,000+ | NYSE, NASDAQ, AMEX listed securities |
| **HK Stocks** | ~2,600+ | HKEX Main Board + GEM |
| **A-Shares** | ~4,000+ | SSE + SZSE (via Connect only) |
| **Australia** | ~2,000+ | ASX listed |
| **Canada** | ~3,000+ | TSX + TSXV |
| **Malaysia** | ~900+ | Bursa Malaysia |

**Coverage Details**:
- **Large Cap**: Comprehensive (all major stocks)
- **Mid Cap**: Comprehensive
- **Small Cap**: Comprehensive
- **Micro Cap**: Limited (depends on exchange listing)
- **OTC Markets**: **No** (Pink Sheets, OTCQX not supported)
- **Penny Stocks**: Yes (if exchange-listed, not OTC)
- **ADRs**: Yes (US-listed ADRs available)
- **REITs**: Yes (treated as stocks)

### ETFs

| Market | Coverage | Examples |
|--------|----------|----------|
| **US ETFs** | Comprehensive | SPY, QQQ, IWM, GLD, TLT, ARKK, VOO, VTI |
| **HK ETFs** | Comprehensive | Tracker Fund (2800), A50 China (2823) |
| **Leveraged/Inverse** | Yes | TQQQ, SQQQ, UPRO, SPXU |
| **Sector ETFs** | Yes | XLF, XLE, XLK, XLV, etc. |

### Warrants (Hong Kong)

- **Covered Warrants**: Yes (extensive HK coverage)
- **Callable Bull/Bear Certificates (CBBC)**: Yes (HK)
- **Inline Warrants**: Yes (HK)
- **Issuers**: All major issuers (Goldman Sachs, Morgan Stanley, UBS, etc.)
- **Underlying**: HSI, HK stocks, commodities, forex

### Options

| Market | Underlying Types | Strike Coverage |
|--------|-----------------|-----------------|
| **US Options** | Stocks, ETFs, Indices | All available strikes |
| **HK Options** | HSI, Select stocks | All available strikes |

**Expirations**:
- Standard monthly expirations
- Weekly expirations (US)
- Quarterly expirations (index options)
- LEAPS (long-dated options up to 2+ years)

**Chains**:
- Full option chains via `get_option_chain()`
- All strikes and expirations
- Greeks calculated

### Futures

| Category | Instruments | Coverage |
|----------|-------------|----------|
| **Index Futures** | E-mini S&P, NASDAQ, Dow, Russell, HSI, Nikkei | Full |
| **Commodity Futures** | Gold, Silver, Crude Oil, Natural Gas, Corn, Wheat | CME products |
| **Currency Futures** | EUR, JPY, GBP, AUD, CAD, etc. | CME products |
| **Interest Rate Futures** | Treasury, Eurodollar | CME products |

**Contract Types**:
- Main contracts (front month)
- Continuous contracts (auto-roll)
- Individual contract months

### Indices (Quote Only, Not Tradable)

| Market | Indices | Coverage |
|--------|---------|----------|
| **US** | S&P 500, NASDAQ, Dow, Russell 2000 | Yes |
| **Hong Kong** | Hang Seng, Hang Seng Tech, China Enterprises | Yes |
| **China** | SSE Composite, SZSE Component | Yes |

**Note**: Indices are quote-only. Trade via futures or ETFs.

### Bonds
**Not Supported**: Corporate bonds, government bonds not available. Treasury futures available.

### Commodities (Spot)
**Not Supported**: Spot gold, spot oil not available. Trade via futures or commodity ETFs.

## Data History

### Historical Depth

| Data Type | Market | Depth | Notes |
|-----------|--------|-------|-------|
| **Daily Candlesticks** | All | **Up to 20 years** | Depends on listing date |
| **Intraday Bars (1m-1h)** | All | Varies | Usually 1-3 months real-time history |
| **Tick-by-Tick** | All | Real-time only | Historical ticks not available |

**Examples**:
- `US.AAPL` daily bars: From 2003 (20 years) to present
- `HK.00700` daily bars: From 2004 (listing date) to present
- `US.AAPL` 1-minute bars: Last ~3 months
- Older securities: May have full history from listing

**Adjustment Types**:
- **前复权 (Forward Adjusted)**: Adjusts historical prices forward from split/dividend events
- **后复权 (Backward Adjusted)**: Adjusts historical prices backward
- **不复权 (Unadjusted)**: Raw prices without adjustment

### Granularity Available

| Interval | Available | Depth | Historical Quota Cost |
|----------|-----------|-------|----------------------|
| **Tick Data** | Real-time only | N/A | No (subscription-based) |
| **1-minute** | Yes | ~3 months | Yes (1 quota/security/30 days) |
| **3-minute** | Yes | ~3 months | Yes |
| **5-minute** | Yes | ~3 months | Yes |
| **15-minute** | Yes | ~6 months | Yes |
| **30-minute** | Yes | ~6 months | Yes |
| **1-hour** | Yes | ~1 year | Yes |
| **Daily** | Yes | **20 years** | Yes |
| **Weekly** | Yes | **20 years** | Yes |
| **Monthly** | Yes | **20 years** | Yes |

**Extended Hours (US)**:
- Pre-market: 4:00 AM - 9:30 AM ET (1h and below intervals)
- After-hours: 4:00 PM - 8:00 PM ET (1h and below intervals)
- Overnight: 8:00 PM - 4:00 AM ET (1h and below intervals)
- Requires `extended_time=True` parameter

### Real-time vs Delayed

| Data Type | Free Tier | Paid Tier | Notes |
|-----------|-----------|-----------|-------|
| **HK Stocks (LV1)** | Real-time | Real-time | Free for all |
| **HK Stocks (LV2)** | Real-time (mainland) / Delayed or Paid | Real-time | Free for mainland China users |
| **US Stocks (Basic)** | Real-time | Real-time | Free basic quotes |
| **US Nasdaq TotalView** | Not available | Real-time | Requires paid subscription |
| **US Options** | Real-time (if >$3K) | Real-time | Free if qualified |
| **A-Shares** | Real-time (mainland) | Real-time | Free for mainland China |

**No 15-minute delay**: Futu provides real-time data (subject to quote level subscription), not delayed data.

## Update Frequency

### Real-time Streams (After Subscription)

| Data Type | Update Frequency | Latency | Push Method |
|-----------|-----------------|---------|-------------|
| **Price Updates** | On change | <100ms (typical) | Server push (callback) |
| **Order Book** | On change | <100ms | Server push |
| **Trades (Ticker)** | Per trade | <50ms | Server push |
| **Candlesticks** | On bar update/close | Real-time | Server push |
| **Time Frame** | Per minute | <1 second | Server push |
| **Broker Queue (HK)** | On change | <100ms | Server push |

**Push Architecture**:
- Server-initiated push (no polling)
- TCP persistent connection
- Protocol Buffers for efficiency
- Callback handlers receive updates immediately

### Scheduled Updates (Non-real-time)

| Data Type | Update Frequency | Notes |
|-----------|-----------------|-------|
| **Fundamentals** | Quarterly, Annual | Limited API coverage |
| **Corporate Actions** | As announced | Rehab data updated |
| **IPO List** | Daily | New IPOs added as announced |
| **Sector/Plate Data** | Daily | Updated overnight |
| **Stock Basic Info** | Daily | Listing changes, suspensions |

## Data Quality

### Accuracy
- **Source**: Direct from exchanges (primary source)
  - HK: HKEX
  - US: NYSE, NASDAQ, AMEX
  - A-shares: SSE, SZSE (via HKEX Connect)
- **Validation**: Exchange-validated data
- **Corrections**: Automatic (exchange-provided corrections applied)
- **Corporate Actions**: Automatically applied (splits, dividends)

### Completeness
- **Missing Data**: Rare (high-quality exchange feeds)
- **Gaps**: Identified by `is_blank` flag (time frame data)
- **Backfill**: Historical data backfilled to 20 years (daily)
- **Delisted Securities**: Basic info available, quotes unavailable

### Timeliness
- **Latency**: <100ms for real-time quotes (typical)
  - "Order execution as fast as 0.0014s" (claimed performance)
- **Delay**: Zero delay (real-time, not T+15 delayed)
- **Market Hours**: Fully covered
  - Regular hours: Full coverage
  - Pre/post market: Supported (US)
  - After-hours: Supported (US)

### Data Integrity
- **Quote Authority**: Ensures compliance with exchange rules
- **Subscription System**: Prevents unauthorized access
- **Rate Limits**: Prevents abuse, ensures fair access
- **Error Handling**: Clear error messages for invalid requests

## Market Hours Coverage

### Hong Kong (HKT = UTC+8)

| Session | Time (HKT) | API Support |
|---------|------------|-------------|
| **Pre-Opening** | 9:00 - 9:30 | Auction data available |
| **Morning Session** | 9:30 - 12:00 | Full support |
| **Lunch Break** | 12:00 - 13:00 | No trading |
| **Afternoon Session** | 13:00 - 16:00 | Full support |
| **After-Hours** | 16:00 - 16:10 | Closing auction |

### United States (ET = UTC-5/-4)

| Session | Time (ET) | API Support | Parameter |
|---------|-----------|-------------|-----------|
| **Pre-Market** | 4:00 - 9:30 AM | Yes | `extended_time=True` |
| **Regular Trading** | 9:30 AM - 4:00 PM | Yes | Default |
| **After-Hours** | 4:00 - 8:00 PM | Yes | `extended_time=True` |
| **Overnight** | 8:00 PM - 4:00 AM | Yes | `session=Session.OVERNIGHT` |

**Session Parameter**:
- `RTH`: Regular trading hours only
- `ETH`: Extended hours (pre + after)
- `OVERNIGHT`: Overnight session
- `ALL`: 24-hour quotes

### China A-Shares (CST = UTC+8)

| Session | Time (CST) | API Support |
|---------|------------|-------------|
| **Pre-Opening** | 9:15 - 9:25 | Auction |
| **Morning Session** | 9:30 - 11:30 | Full support |
| **Lunch Break** | 11:30 - 13:00 | No trading |
| **Afternoon Session** | 13:00 - 15:00 | Full support |

## Trading Days Coverage

### Trading Calendars
- **`get_trading_days()`**: Returns all trading days and holidays
- **Markets Supported**: HK, US, CN, SG, JP, AU, MY, CA
- **Data Includes**:
  - Full trading days
  - Half-days (morning/afternoon only)
  - Market holidays
- **Date Range**: Query any date range (past and future)

### Holiday Handling
- Holidays identified as non-trading days
- Half-days marked with `MORNING` or `AFTERNOON` flag
- No data available on non-trading days (expected behavior)

## Coverage Limitations

### Not Covered

**Geographic**:
- No European markets (UK, Germany, France, etc.)
- No other Asian markets (Korea, Taiwan, India, Indonesia, Thailand)
- No Latin American markets
- No Middle Eastern markets
- No African markets

**Asset Classes**:
- No spot forex (only currency futures)
- No cryptocurrencies
- No corporate/government bonds (only treasury futures)
- No spot commodities (only commodity futures)
- No OTC derivatives

**Data Types**:
- No news feeds
- No social sentiment
- No fundamental data via API (limited)
- No earnings calendars
- No economic calendars
- No analyst reports (text)

## Unique Coverage Advantages

### Multi-Market Single API
- HK + US + A-shares + SG + JP + AU + MY + CA
- Single subscription quota across all markets
- Unified API structure
- Seamless multi-market strategies

### Hong Kong Specifics
- Broker queue data (unique)
- Warrant coverage (extensive)
- Dark pool status
- Capital flow analysis

### US Extended Hours
- Pre-market (4 AM - 9:30 AM)
- After-hours (4 PM - 8 PM)
- Overnight (8 PM - 4 AM)
- Full 24-hour quote support

### China A-Shares Access
- Via HK-Shanghai/Shenzhen Connect
- Simpler than direct access
- Same API as HK/US

### Paper Trading
- Full simulated accounts
- Real market data + simulated execution
- Test strategies risk-free

## Data Costs

### Free Data
- HK LV1 quotes (all users)
- HK LV2 quotes (mainland China users only)
- US basic quotes
- US options (if >$3K assets)
- A-share LV1 (mainland China users only)
- Historical data (quota-limited, not priced)

### Paid Data
- HK LV2 quotes (~$10-20/month, non-mainland users)
- US Nasdaq TotalView (~$30-50/month)
- A-share LV1 (non-mainland users, pricing varies)
- CME futures data (pricing varies)

### No Per-Call Charges
- **Unlimited API requests** (within rate limits)
- **No data volume charges**
- **No tick/bar charges**
- **No quote snapshot fees**

## Comparison with Competitors

| Feature | Futu | Interactive Brokers | Alpaca | Polygon |
|---------|------|-------------------|--------|---------|
| **Markets** | HK, US, CN, SG, JP, AU, MY, CA | Global (>150 countries) | US only | US only |
| **Stocks** | Yes (8+ markets) | Yes (global) | Yes (US) | Yes (US) |
| **Options** | Yes (HK, US) | Yes (global) | Yes (US) | Limited |
| **Futures** | Yes (limited) | Yes (global) | No | No |
| **Crypto** | No | Yes | Yes | Yes |
| **Forex** | No (futures only) | Yes (spot) | Yes | No |
| **Historical Depth** | 20 years | Varies | 5 years | Varies |
| **Real-time** | Yes | Yes | Yes | Yes (paid) |
| **API Cost** | Free | Free | Free | Free/Paid tiers |
| **Data Cost** | $0-50/mo | $0-105/mo | Free | $0-199/mo |
| **Paper Trading** | Yes | Yes | Yes | No |

**Futu's Strengths**:
- Strong Asia coverage (HK, CN, SG, JP)
- Single API for multiple markets
- Free real-time data (with subscriptions)
- Excellent for Asian + US strategies

**Futu's Weaknesses**:
- No European markets
- Limited futures coverage (vs IBKR)
- No crypto support
- No spot forex
