# Dhan - Data Coverage

## Geographic Coverage

### Regions Supported
- North America: No
- Europe: No
- Asia: Yes (India only)
- Other: No

### Country-Specific
- **India: Yes** (Primary and only market)
  - Full coverage of Indian stock markets (NSE, BSE)
  - Commodity markets (MCX)
  - Currency derivatives (NSE Currency segment)
- US: No
- UK: No
- Japan: No
- China: No

### Restricted Regions
- **Blocked countries**: None explicitly stated, but service is India-focused
- **Regulatory restrictions**: Dhan is an Indian broker, requires Indian residency for account opening
- **VPN detection**: Not applicable (requires valid Indian KYC)
- **Geo-fencing**: No API geo-fencing, but account requires Indian address/PAN card
- **International users**: NRI (Non-Resident Indians) and OCI cardholders can open accounts

### India-Specific Requirements
- **PAN Card**: Mandatory for account opening
- **Aadhaar**: For e-KYC verification
- **Indian bank account**: Required for fund transfers
- **Mobile number**: Indian mobile number for OTP authentication

## Markets/Exchanges Covered

### Stock Markets
- **NSE (National Stock Exchange)**: Yes ✅
  - Cash market (NSE_EQ)
  - Derivatives market (NSE_FNO)
  - Currency derivatives
- **BSE (Bombay Stock Exchange)**: Yes ✅
  - Cash market (BSE_EQ)
  - Limited derivatives (BSE F&O exists but less liquid)
- **MCX (Multi Commodity Exchange)**: Yes ✅
  - Commodity futures (MCX_COMM)

### International Exchanges
- **NYSE**: No ❌
- **NASDAQ**: No ❌
- **LSE**: No ❌
- **TSE (Tokyo)**: No ❌
- **HKEX**: No ❌
- **Other international exchanges**: No ❌

**Note**: Dhan is a domestic Indian broker. For US stocks, Indian investors typically use INDmoney, Vested, or other platforms offering international investing.

### Commodity Exchanges (India)
- **MCX**: Yes ✅
  - Metals: Gold, Silver, Copper, Zinc, Lead, Nickel, Aluminum
  - Energy: Crude Oil, Natural Gas
  - Agriculture: Limited availability for retail
- **NCDEX**: No ❌ (Agriculture commodities exchange - not covered by Dhan)

### Currency Markets
- **NSE Currency Segment**: Yes ✅
  - USD/INR (most liquid)
  - EUR/INR
  - GBP/INR
  - JPY/INR
- **Spot Forex**: No ❌ (not permitted for Indian retail traders)

## Instrument Coverage

### Stocks (Equities)

#### NSE Equity (NSE_EQ)
- **Total symbols**: ~2,000 actively traded stocks
- **Large Cap**: All Nifty 50 stocks ✅
- **Mid Cap**: All Nifty Midcap 150 stocks ✅
- **Small Cap**: All Nifty Smallcap 250 stocks ✅
- **Micro Cap**: Yes, many illiquid small caps
- **ETFs**: Yes (Nifty ETF, Bank Nifty ETF, Gold ETF, etc.)
- **InvITs (Infrastructure Investment Trusts)**: Yes
- **REITs (Real Estate Investment Trusts)**: Yes
- **Preference Shares**: Yes (limited)

#### BSE Equity (BSE_EQ)
- **Total symbols**: ~5,000+ listed stocks
- **More stocks than NSE**: Yes (BSE lists smaller companies)
- **Liquidity**: Lower than NSE for most stocks
- **Unique listings**: Some stocks only listed on BSE

#### OTC / Unlisted
- **OTC stocks**: No ❌
- **Unlisted shares**: No ❌ (via trading account)
- **IPO applications**: Yes ✅ (via Dhan platform, not API)

#### Penny Stocks
- **Available**: Yes (stocks trading < Rs. 10)
- **Risk**: High (exchange circuit limits apply)

### Derivatives (Equity F&O)

#### NSE Futures & Options (NSE_FNO)

**Index Futures**:
- Nifty 50
- Bank Nifty
- Fin Nifty
- Nifty Midcap 50
- Nifty IT
- Nifty Bank (same as Bank Nifty)
- Total: 10+ index futures

**Stock Futures**:
- ~200+ individual stock futures
- Only large-cap and some mid-cap stocks
- Lot sizes vary by stock

**Index Options**:
- Nifty 50 (most liquid)
- Bank Nifty (high premium)
- Fin Nifty (weekly expiries)
- Midcap Nifty
- Total: 10+ index options

**Stock Options**:
- ~300+ individual stock options
- Less liquid than index options

**Expiry Schedule**:
- **Weekly expiries**: Nifty (Thursday), Bank Nifty (Wednesday), Fin Nifty (Tuesday)
- **Monthly expiries**: Last Thursday of month
- **Quarterly expiries**: Available
- **Far-month contracts**: Up to 2026+ (visible in option chain API)

#### Currency Derivatives (NSE Currency)
- **USD/INR**: Futures and Options
- **EUR/INR**: Futures and Options
- **GBP/INR**: Futures and Options
- **JPY/INR**: Futures and Options
- **Cross-currency pairs**: Limited (EUR/USD, GBP/USD via INR pairs)

### Commodities (MCX_COMM)

#### Metals
- **Gold** (Gold, Gold Mini, Gold Guinea) ✅
- **Silver** (Silver, Silver Mini, Silver Micro) ✅
- **Copper** ✅
- **Zinc** ✅
- **Lead** ✅
- **Nickel** ✅
- **Aluminum** ✅

#### Energy
- **Crude Oil** (WTI, Brent-equivalent) ✅
- **Natural Gas** ✅

#### Agriculture
- **Limited for retail**: Most agri commodities restricted
- **Available**: Mentha Oil, Cardamom (limited liquidity)

### Crypto
- **Cryptocurrency**: No ❌
- **Crypto derivatives**: No ❌
- **Crypto CFDs**: No ❌

**Note**: Cryptocurrency trading is in regulatory gray area in India. Not offered by traditional brokers like Dhan.

### Forex (Spot)
- **Spot FX trading**: No ❌ (not permitted for Indian retail)
- **Only currency derivatives**: Yes (NSE Currency segment)

## Data History

### Historical Depth

#### Equities (Stocks)
- **Daily data**: From instrument inception
  - Nifty 50 stocks: 20+ years (since 1990s for some)
  - Mid/Small caps: 10-15 years typical
  - Recently listed stocks: From listing date
- **Intraday data**: Last 5 years
  - 1m, 5m, 15m, 25m, 60m intervals
  - All actively traded stocks

#### Derivatives (F&O)
- **Daily data**: From contract listing date
  - Index futures/options: Since F&O started (2000 for Nifty)
  - Stock F&O: Since individual stock was added to F&O
- **Intraday data**: Last 5 years
  - Including Open Interest data
- **Expired contracts**: Historical data available for expired F&O contracts

#### Commodities (MCX)
- **Daily data**: From MCX inception (2003+)
  - Gold, Silver: 15+ years
  - Other commodities: 10+ years
- **Intraday data**: Last 5 years

### Granularity Available
- **Tick data**: No ❌ (not available via API)
- **1-second bars**: No ❌
- **1-minute bars**: Yes ✅ (last 5 years)
- **5-minute bars**: Yes ✅ (last 5 years)
- **15-minute bars**: Yes ✅ (last 5 years)
- **25-minute bars**: Yes ✅ (last 5 years, unique interval)
- **60-minute bars**: Yes ✅ (last 5 years)
- **Daily bars**: Yes ✅ (from inception, 10-20+ years)
- **Weekly bars**: Can be constructed from daily
- **Monthly bars**: Can be constructed from daily

### Real-time vs Delayed
- **Real-time**: Yes ✅ (all users, no delay)
  - Tick-by-tick via WebSocket
  - Live quotes via REST
- **Delayed**: No (Dhan provides only real-time)
  - No 15-minute delayed data tier
  - All data is real-time for authenticated users
- **Snapshot**: Yes (via Quote API, point-in-time snapshot)

### Data Availability Windows
- **Intraday historical**: 90-day query limit per request
  - Can fetch last 5 years by making multiple requests
- **Daily historical**: No query limit
  - Can fetch entire history in single request
- **WebSocket**: Real-time only (no historical replay)

## Update Frequency

### Real-time Streams (WebSocket)

#### Price Updates
- **Frequency**: Tick-by-tick (every trade)
- **Latency**: <50ms typical (from exchange to API)
- **Throttling**: No (server pushes at market speed)

#### Orderbook (Market Depth)
- **5-level depth**: Real-time updates (snapshot + delta)
- **20-level depth**: Real-time updates
- **200-level depth**: Real-time updates
- **Update frequency**: On every orderbook change

#### Trades
- **Trade updates**: Every trade immediately
- **Volume**: Cumulative volume updated on each trade

#### Open Interest (Derivatives)
- **Update frequency**: Every few seconds (OI updates slower than price)
- **Why slower**: OI calculated at exchange, not every trade affects OI

### REST API (Polling)

#### Quote API
- **Limitation**: 1 request per second
- **Use case**: Snapshot data, not for continuous streaming
- **Recommendation**: Use WebSocket for real-time, Quote for snapshots

#### Historical Data API
- **Rate limit**: 5 requests/second
- **Use case**: Backfilling historical data, not real-time
- **Daily limit**: 100,000 requests/day

#### Option Chain API
- **Rate limit**: 1 request per 3 seconds
- **Reason**: OI updates slowly, no need for high-frequency polling

### Scheduled Updates

#### Corporate Actions
- **Frequency**: As announced by exchange
- **Adjustment**: Historical prices adjusted automatically
- **Types**:
  - Stock splits
  - Bonuses
  - Dividends
  - Rights issues

#### Instrument Lists
- **Update frequency**: Daily (new contracts, expirations)
- **Recommendation**: Fetch daily at market open

#### Market Holidays
- **Updates**: As per exchange calendar
- **Source**: NSE/BSE/MCX official calendars (not provided via API)

## Data Quality

### Accuracy
- **Source**: Direct from exchange (NSE, BSE, MCX)
  - Not aggregated or third-party
  - Official exchange data feed
- **Validation**: Exchange-validated
  - Circuit limits enforced
  - Bad ticks filtered by exchange
- **Corrections**: Automatic
  - Exchange corrects erroneous trades
  - Dhan API reflects corrected data

### Completeness
- **Missing data**: Rare
  - Exchange feeds highly reliable
  - 99.9%+ uptime during market hours
- **Gaps**: Minimal
  - Market halts (circuit breakers) reflected
  - Trading halts for corporate actions
- **Backfill**: Available
  - Historical data complete from inception
  - No gaps in historical data

### Timeliness
- **Latency**: <50ms typical (exchange → Dhan → API user)
  - WebSocket: Lowest latency
  - REST: ~100-200ms (includes network + processing)
- **Delay**: None (real-time for all users)
- **Market hours coverage**: Full coverage
  - Pre-market: 9:00-9:15 AM IST
  - Regular: 9:15 AM-3:30 PM IST
  - Post-market: 3:40-4:00 PM IST
  - After-hours: No trading (data not applicable)

### Data Adjustments
- **Corporate actions**: Automatically adjusted
  - Stock splits: Historical prices divided
  - Bonuses: Quantities adjusted
  - Dividends: Prices adjusted (ex-dividend)
- **Adjustment visibility**: Not explicitly marked in API
  - Adjusted prices returned by default
  - Unadjusted prices: Not available via API

## Specific Coverage Notes

### Nifty 50 Stocks
All Nifty 50 constituents fully covered:
- Real-time data ✅
- Historical data (20+ years) ✅
- Derivatives (futures + options) ✅
- Deep orderbook (200-level) ✅

### Bank Nifty Stocks
All Bank Nifty constituents fully covered:
- Major banks: HDFC Bank, ICICI Bank, SBI, Axis Bank, Kotak Mahindra, etc.
- Index derivatives (weekly expiries) ✅

### ETFs
- **Nifty ETF**: Yes (e.g., Nippon India ETF Nifty BeES)
- **Bank Nifty ETF**: Yes
- **Gold ETF**: Yes (multiple issuers)
- **International ETFs**: Limited (some US index ETFs listed on NSE)
- **Thematic ETFs**: Yes (IT, Pharma, Auto, etc.)

### New Listings
- **IPO stocks**: Available for trading post-listing
- **Data from listing day**: Yes
- **Historical data**: From listing date onwards

### Delisted Stocks
- **Historical data**: Available up to delisting date
- **Post-delisting**: No data (stock no longer trades)

### Suspended Stocks
- **Trading halted**: Data updates stop during suspension
- **Resumes**: Data resumes when trading resumes

## Geographic Restrictions (for API Users)

### Account Opening
- **Indian residents**: Full access
- **NRIs (Non-Resident Indians)**: Yes, can open NRI trading account
- **OCIs (Overseas Citizens of India)**: Yes
- **PIOs (Persons of Indian Origin)**: Yes
- **Foreign nationals**: No (not permitted by SEBI regulations)

### API Access from Outside India
- **VPN usage**: Not restricted (API accessible globally)
- **IP whitelisting**: Required for Order APIs (from Jan 2026)
  - Can whitelist international IPs
  - Static IP requirement
- **Data access**: Available globally (no geo-blocking)

### Regulatory Compliance
- **SEBI regulations**: All trading subject to SEBI rules
- **FPI (Foreign Portfolio Investor) limits**: Applicable
- **Repatriation rules**: For NRIs (profits can be repatriated)

## Coverage Comparison (Indian Brokers)

| Feature | Dhan | Zerodha | Upstox | Angel One |
|---------|------|---------|--------|-----------|
| NSE Equity | ✅ | ✅ | ✅ | ✅ |
| BSE Equity | ✅ | ✅ | ✅ | ✅ |
| NSE F&O | ✅ | ✅ | ✅ | ✅ |
| MCX Commodities | ✅ | ✅ | ✅ | ✅ |
| 200-level depth | ✅ | ❌ | ❌ | ❌ |
| 5-year intraday | ✅ | ❌ (1 year) | ❌ (1 year) | ❌ (1 year) |
| WebSocket connections | 5 | 3 | 3 | 1 |
| Order API (req/sec) | 25 | 10 | 10 | 10 |

**Dhan offers superior data coverage and access compared to competitors.**

## Coverage Gaps

### What Dhan Does NOT Cover:
- ❌ International stocks (US, UK, etc.)
- ❌ Cryptocurrencies
- ❌ Spot forex (only currency derivatives)
- ❌ NCDEX commodities (agriculture)
- ❌ Corporate bonds
- ❌ Government bonds (G-Secs)
- ❌ Mutual funds via API (available on platform, not API)
- ❌ Fundamental data (financials, ratios, etc.)
- ❌ News feeds
- ❌ Economic calendar
- ❌ Analyst recommendations

### Workarounds:
- **US stocks**: Use INDmoney, Vested, or other international brokers
- **Crypto**: Use WazirX, CoinDCX, or international exchanges
- **Bonds**: Use direct RBI platform or other bond platforms
- **Fundamentals**: Use Screener.in, Tijori Finance, or Bloomberg
- **News**: Use MoneyControl, ET Markets, or NSE announcements

## Data Retention

### Historical Data
- **Daily data**: Retained indefinitely (from inception)
- **Intraday data**: Last 5 years retained
- **Older intraday**: Not available (>5 years)

### Trade History
- **Your trades**: Retained for 5+ years (regulatory requirement)
- **Ledger**: Retained for 5+ years
- **Order book**: Historical orders available (via API for recent data)

## Market Coverage Completeness

### NSE Coverage: 100%
- All NSE listed securities available
- All NSE F&O contracts available
- All NSE Currency derivatives available

### BSE Coverage: 100%
- All BSE listed securities available
- Limited BSE F&O (low liquidity)

### MCX Coverage: ~80%
- Major contracts covered (Gold, Silver, Crude, Natural Gas)
- Some agriculture contracts not available for retail

## Conclusion

**Dhan provides comprehensive coverage of Indian markets:**
- ✅ **Complete coverage**: All NSE, BSE, MCX instruments
- ✅ **Best-in-class depth**: 200-level orderbook (unique)
- ✅ **Extensive history**: 5 years intraday, 20+ years daily
- ✅ **Real-time data**: No delays, tick-by-tick via WebSocket
- ✅ **Geographic**: India-focused, requires Indian residency
- ❌ **Not international**: No US/international stocks
- ❌ **No crypto**: Cryptocurrency not supported

**Best for**: Indian market traders and algorithms focusing on NSE/BSE equities and derivatives.
