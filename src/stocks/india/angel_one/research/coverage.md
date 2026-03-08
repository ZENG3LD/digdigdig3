# Angel One SmartAPI - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: No
- **Europe**: No
- **Asia**: Yes (India only)
- **Other**: No

### Country-Specific
- **India**: Yes (primary and only market)
- **US**: No
- **UK**: No
- **Japan**: No
- **Other countries**: No

**Geographic Focus**: Angel One SmartAPI is exclusively for Indian markets. All supported exchanges are located in India.

### Restricted Regions
- **Blocked countries**: Not explicitly documented (likely follows Indian regulations)
- **VPN detection**: Not documented
- **Geo-fencing**: May apply based on Indian securities regulations
- **Access**: Requires Indian trading account with Angel One
- **KYC**: Indian KYC required (PAN card, Aadhaar, etc.)

## Markets/Exchanges Covered

### Stock Markets
- **India - NSE (National Stock Exchange)**: Yes
  - Equity cash segment
  - ~2000+ listed stocks
  - Indices (NIFTY 50, NIFTY Bank, etc.)
- **India - BSE (Bombay Stock Exchange)**: Yes
  - Equity cash segment
  - ~5000+ listed stocks
  - Indices (SENSEX, BSE 500, etc.)
- **US Markets (NYSE, NASDAQ)**: No
- **UK Markets (LSE)**: No
- **Japan Markets (TSE)**: No
- **China Markets (SSE, SZSE)**: No
- **Other International Markets**: No

### Derivatives Exchanges
- **India - NFO (NSE Futures & Options)**: Yes
  - Index futures (NIFTY, BANKNIFTY, FINNIFTY, etc.)
  - Stock futures
  - Index options
  - Stock options
- **India - BFO (BSE Futures & Options)**: Yes
  - SENSEX futures
  - Stock futures
  - Index options
  - Stock options
- **CME (Chicago Mercantile Exchange)**: No
- **CBOE (Chicago Board Options Exchange)**: No
- **Eurex**: No

### Commodities Exchanges
- **India - MCX (Multi Commodity Exchange)**: Yes
  - Precious metals (Gold, Silver)
  - Base metals (Copper, Zinc, Lead, Nickel, Aluminum)
  - Energy (Crude Oil, Natural Gas)
  - Agricultural commodities
- **India - NCDEX (National Commodity & Derivatives Exchange)**: Yes
  - Agricultural commodities
  - Spices
  - Pulses
  - Oilseeds
- **COMEX (US)**: No
- **LME (London Metal Exchange)**: No

### Currency Exchange
- **India - CDS (Currency Derivatives Segment)**: Yes
  - USD/INR futures
  - EUR/INR futures
  - GBP/INR futures
  - JPY/INR futures
  - Cross-currency futures
- **Spot Forex**: No (only currency derivatives/futures)

### Crypto Exchanges (Not Applicable)
Angel One SmartAPI does not support cryptocurrency trading.

- **Binance**: No
- **Coinbase**: No
- **Kraken**: No
- **Any crypto exchange**: No

**Note**: Cryptocurrency trading is in regulatory gray area in India. Angel One focuses on traditional regulated markets.

## Instrument Coverage

### Stocks

#### NSE (National Stock Exchange)
- **Total symbols**: ~2,000+ actively traded stocks
- **Large-cap**: All NIFTY 50 stocks
- **Mid-cap**: NIFTY Midcap 100 stocks
- **Small-cap**: NIFTY Smallcap 250 stocks
- **Sectoral stocks**: All sectors covered
- **OTC**: No
- **Penny stocks**: Some available (if listed on NSE)

#### BSE (Bombay Stock Exchange)
- **Total symbols**: ~5,000+ listed stocks
- **Large-cap**: All SENSEX 30 stocks
- **Mid-cap & Small-cap**: Extensive coverage
- **SME Platform**: Yes (BSE SME stocks)
- **OTC**: No
- **Penny stocks**: More extensive coverage than NSE

### Derivatives

#### Index Futures & Options
- **NIFTY 50**: Futures & Options (weekly, monthly expirations)
- **BANKNIFTY**: Futures & Options (weekly, monthly expirations)
- **FINNIFTY**: Futures & Options (weekly expirations)
- **SENSEX**: Futures & Options
- **BANKEX**: Futures & Options
- **Other Indices**: Multiple sectoral indices

#### Stock Futures & Options
- **NFO Stock F&O**: ~200+ stocks
- **BFO Stock F&O**: Selected stocks
- **Contract types**: Futures (1-3 month expirations), Options (multiple strikes per expiry)

### Commodities

#### Precious Metals
- **Gold**: Futures (various contract sizes)
- **Silver**: Futures (various contract sizes)
- **Platinum**: Limited availability
- **Palladium**: Limited availability

#### Base Metals
- **Copper**: Yes
- **Zinc**: Yes
- **Lead**: Yes
- **Nickel**: Yes
- **Aluminum**: Yes

#### Energy
- **Crude Oil**: Futures
- **Natural Gas**: Futures
- **Other energy commodities**: Limited

#### Agricultural
- **Grains**: Wheat, Corn, etc. (NCDEX)
- **Oilseeds**: Soybean, Mustard, etc. (NCDEX)
- **Spices**: Turmeric, Jeera, Pepper, etc. (NCDEX)
- **Pulses**: Various (NCDEX)

### Currency

#### Major Pairs (All vs INR)
- **USD/INR**: Yes (futures)
- **EUR/INR**: Yes (futures)
- **GBP/INR**: Yes (futures)
- **JPY/INR**: Yes (futures)

#### Cross Pairs
- **EUR/USD**: Yes (cross-currency futures)
- **GBP/USD**: Yes (cross-currency futures)
- **USD/JPY**: Yes (cross-currency futures)

**Note**: All currency trading is in futures contracts, not spot.

### Indices

#### NSE Indices
- **NIFTY 50**: Yes (real-time OHLC)
- **NIFTY Bank**: Yes
- **NIFTY IT**: Yes
- **NIFTY Pharma**: Yes
- **NIFTY Auto**: Yes
- **NIFTY FMCG**: Yes
- **NIFTY Metal**: Yes
- **NIFTY Midcap 100**: Yes
- **NIFTY Smallcap 250**: Yes
- **Sectoral indices**: 50+ indices
- **Total NSE indices**: 100+ indices

#### BSE Indices
- **SENSEX**: Yes
- **BSE 500**: Yes
- **BSE Midcap**: Yes
- **BSE Smallcap**: Yes
- **Sectoral indices**: Multiple
- **Total BSE indices**: 20+ indices

#### MCX Indices
- **MCX Commodity indices**: Yes

#### Total Indices Coverage
**120+ indices** across NSE, BSE, and MCX with real-time OHLC data.

### Mutual Funds
- **Mutual Fund Trading**: Yes (order placement supported)
- **Real-time NAV**: Not via API (use fund house websites)

### Bonds & Fixed Income
**Not available** via SmartAPI.

- **Government Bonds**: No
- **Corporate Bonds**: No
- **T-Bills**: No

## Instrument Master Statistics

Based on instrument master file, typical coverage:

| Segment | Approximate Count | Notes |
|---------|-------------------|-------|
| NSE Equity | ~2,000 | Actively traded stocks |
| BSE Equity | ~5,000 | Including SME segment |
| NFO Futures | ~250 | Index + stock futures |
| NFO Options | ~50,000+ | Multiple strikes, expirations |
| BFO Derivatives | ~1,000+ | Limited compared to NFO |
| MCX Commodities | ~100+ | Various contract months |
| CDS Currency | ~50+ | Various expiries |
| NCDEX Commodities | ~50+ | Agricultural focus |

**Total instruments**: 60,000+ (heavily weighted towards options with multiple strikes)

## Data History

### Historical Depth

#### Equity (NSE, BSE)
- **From year**: Varies by stock (generally 2010-2015 onwards for most stocks)
- **Typical depth**: 5-10+ years
- **Daily data**: Up to 2000 days (~5.5 years max per request)
- **Intraday data**: Limited by interval (30 days for 1-minute)

#### Derivatives (NFO, BFO)
- **From year**: Contract-specific (from listing date)
- **Expiry data**: Only until contract expiration
- **Expired contracts**: **NOT available** (important limitation)
- **Current contracts**: Full history from listing

#### Commodities (MCX, NCDEX)
- **From year**: Contract-specific
- **Historical depth**: Similar to derivatives

#### Currency (CDS)
- **From year**: Contract-specific
- **Historical depth**: From contract listing to expiry

### Granularity Available

| Interval | Available | Max History | Max Candles | Notes |
|----------|-----------|-------------|-------------|-------|
| Tick data | No | N/A | N/A | Not provided |
| 1-minute | Yes | 30 days | 8000 | ONE_MINUTE |
| 3-minute | Yes | 60 days | 8000 | THREE_MINUTE |
| 5-minute | Yes | 100 days | 8000 | FIVE_MINUTE |
| 10-minute | Yes | 100 days | 8000 | TEN_MINUTE |
| 15-minute | Yes | 200 days | 8000 | FIFTEEN_MINUTE |
| 30-minute | Yes | 200 days | 8000 | THIRTY_MINUTE |
| Hourly | Yes | 400 days | 8000 | ONE_HOUR |
| Daily | Yes | 2000 days | 8000 | ONE_DAY |
| Weekly | No | N/A | N/A | Can construct from daily |
| Monthly | No | N/A | N/A | Can construct from daily |

**Max candles per request**: 8,000 (applies to all intervals)

**Important**: Historical data limits are per request. For longer history, make multiple requests with different date ranges.

### Real-time vs Delayed

- **Real-time data**: Yes (all market data is real-time)
- **Delayed data**: No (no delayed data feed option)
- **Snapshot**: Yes (current quote/LTP available)
- **Streaming**: Yes (via WebSocket V2)

**Note**: All data provided by Angel One SmartAPI is real-time with no delay. This is a significant advantage over some data providers that charge for real-time access.

## Update Frequency

### Real-time Streams (WebSocket)

| Data Type | Update Frequency | Notes |
|-----------|------------------|-------|
| LTP (Mode 1) | Every tick | Sub-second updates |
| Quote (Mode 2) | Every tick | Sub-second updates |
| Snap Quote (Mode 3) | Every tick | Sub-second updates |
| Depth 20 (Mode 4) | Every tick | Sub-second updates |
| Order Book | On change | 5 or 20 levels |
| Trades | Real-time | As trades execute |

**Latency**: Typically <100ms from exchange (not guaranteed, depends on network)

### REST API Polling

Not recommended for real-time data (use WebSocket instead).

- **Quote API**: No specified rate limit for queries (use responsibly)
- **Historical API**: Not for real-time (returns closed candles)

### Scheduled Updates

- **Fundamentals**: Not provided
- **Economic data**: Not provided
- **News**: Not provided
- **Corporate actions**: Reflected in adjusted prices (no separate feed)

## Data Quality

### Accuracy

- **Source**: Direct from exchange (NSE, BSE, MCX, etc.)
- **Validation**: Exchange-validated data
- **Corrections**: Automatic via exchange feeds
- **Data provider**: Angel One acts as broker-aggregator (data from exchanges)

### Completeness

- **Missing data**: Rare (exchange data is comprehensive)
- **Gaps**: Market holidays, circuit breaker halts reflected
- **Backfill**: Available for most stocks (subject to historical depth limits)
- **Corporate actions**: Adjusted prices for splits, bonuses, dividends

### Timeliness

- **Latency**: Sub-second for WebSocket (typically <100ms)
- **Delay**: None (real-time data)
- **Market hours**: Covered fully during trading sessions
- **After-hours**: Limited (AMO orders possible, no live quotes)

## Market Hours Coverage

### NSE Equity
- **Regular session**: 9:15 AM - 3:30 PM IST (Mon-Fri)
- **Pre-open**: 9:00 AM - 9:15 AM IST
- **Post-close**: After 3:30 PM (no trading, only settlements)
- **Real-time data during**: Regular session + pre-open

### NFO (Derivatives)
- **Regular session**: 9:15 AM - 3:30 PM IST (Mon-Fri)
- **Same as equity**: Aligned with NSE equity hours

### BSE Equity
- **Regular session**: 9:15 AM - 3:30 PM IST (Mon-Fri)
- **Aligned with NSE**: Same trading hours

### MCX (Commodities)
- **Morning session**: 9:00 AM - 5:00 PM IST (varies by commodity)
- **Evening session**: 5:00 PM - 11:30/11:55 PM IST (for select commodities)
- **Extended hours**: Some commodities trade nearly 24 hours

### CDS (Currency)
- **Session**: 9:00 AM - 5:00 PM IST (Mon-Fri)

**Timezone**: All times in IST (Indian Standard Time, UTC+5:30)

**Market Holidays**: Follow NSE/BSE/MCX holiday calendars (not provided via API)

## Data Restrictions

### Redistribution
- **Data redistribution**: **Prohibited**
- **Personal use**: Allowed (for trading via Angel One account)
- **Application use**: Allowed (for personal trading applications)
- **Commercial redistribution**: Not allowed without separate agreement

### Exchange Compliance
- **NSE/BSE data policies**: Apply (no redistribution)
- **Vendor license**: Not included (this is for personal trading use)

### Rate Limits
See `tiers_and_limits.md` for comprehensive rate limit documentation.

## Special Coverage Features

### 1. Depth 20 Order Book
- **20 levels of depth** (unique among Indian broker APIs)
- **Exchanges**: NSE, BSE, NFO, BFO
- **Real-time**: Sub-second updates
- **Free**: Included at no additional cost

### 2. 120+ Indices
- **NSE indices**: 100+
- **BSE indices**: 20+
- **MCX indices**: Select commodity indices
- **Real-time OHLC**: All indices

### 3. Free Historical Data
- **All segments**: NSE, BSE, NFO, BFO, MCX, CDS, NCDEX
- **No cost**: Completely free (unlike competitors)
- **Depth**: Up to 2000 days for daily candles
- **Quality**: Exchange-sourced, adjusted for corporate actions

### 4. AMO (After Market Orders)
- **Place orders post-market**: Yes (executed next trading day)
- **Supported segments**: Equity, F&O
- **Order types**: Limit, Market, SL

## Coverage Gaps

### What's NOT Covered

1. **International Markets**: Only Indian markets (no US, UK, etc.)
2. **Cryptocurrency**: No crypto trading or data
3. **Expired F&O Contracts**: Historical data unavailable after expiry
4. **Fundamental Data**: No company financials, earnings, ratios
5. **News Feed**: No news or announcements
6. **Economic Calendar**: No macro data or event calendar
7. **Options Analytics**: No IV, Greeks, volatility surface
8. **Tick Data**: No tick-by-tick data (only aggregated candles)
9. **Level 3 Data**: Order book depth limited to 20 levels
10. **Bonds**: No bond trading or data
11. **IPOs**: IPO data not via API (use Angel One app)
12. **Mutual Fund NAV**: NAV not real-time via API

### Workarounds

- **Fundamental data**: Use NSE/BSE websites, Screener.in, or dedicated fundamental APIs
- **News**: Integrate separate news API (MoneyControl, Economic Times, etc.)
- **Economic calendar**: Use Investing.com, TradingView, or forex calendars
- **Options analytics**: Calculate IV/Greeks client-side using Black-Scholes
- **Expired contracts**: No workaround (permanent limitation)
- **International markets**: Use separate broker/API for global markets

## Summary Statistics

| Metric | Count/Details |
|--------|---------------|
| **Countries Covered** | 1 (India only) |
| **Stock Exchanges** | 2 (NSE, BSE) |
| **Derivatives Exchanges** | 2 (NFO, BFO) |
| **Commodity Exchanges** | 2 (MCX, NCDEX) |
| **Currency Exchange** | 1 (CDS) |
| **Total Exchange Segments** | 7 |
| **Equity Symbols** | ~7,000 (NSE + BSE) |
| **Total Instruments** | 60,000+ (including all F&O strikes) |
| **Indices Coverage** | 120+ |
| **Historical Depth (Daily)** | Up to 2000 days (~5.5 years) |
| **Historical Depth (1-min)** | 30 days |
| **Max Candles/Request** | 8,000 |
| **Order Book Depth** | 20 levels (Depth 20 mode) |
| **WebSocket Token Limit** | 1,000 symbols |
| **Real-time Data** | Yes (all data) |
| **Data Cost** | FREE |

## Competitive Comparison

### Angel One vs Zerodha Kite vs Upstox

| Feature | Angel One | Zerodha Kite | Upstox |
|---------|-----------|--------------|--------|
| **API Cost** | Free | ₹2,000/month | Free |
| **Historical Data Cost** | Free | Limited/Paid | Limited |
| **Order Book Depth** | 20 levels | 5 levels | 5 levels |
| **Indices Coverage** | 120+ | Limited | Limited |
| **WebSocket** | Free | Included | Included |
| **Rate Limit (Orders)** | 20/sec | 10/sec | 10/sec |
| **Exchanges** | NSE, BSE, NFO, BFO, MCX, CDS, NCDEX | NSE, BSE, NFO, BFO, MCX, CDS | NSE, BSE, NFO, BFO, MCX, CDS |

**Angel One Advantages**:
1. Free API (vs Zerodha's ₹2,000/month)
2. Free historical data for all segments
3. 20-level order book depth (unique)
4. Higher order rate limits (20/sec vs 10/sec)
5. 120+ indices coverage

**Angel One Limitations**:
1. India-only (like competitors)
2. No fundamental data (like competitors)
3. No tick data (like competitors)
4. No expired F&O data (like competitors)

## Recommendations

### Best Use Cases
1. **Algorithmic Trading**: Excellent coverage for Indian markets
2. **Multi-asset Strategies**: Equity + Derivatives + Commodities
3. **Market Microstructure Analysis**: Depth 20 unique feature
4. **Historical Backtesting**: Free comprehensive historical data
5. **Real-time Trading**: WebSocket with low latency

### Not Suitable For
1. **Global Diversification**: India-only coverage
2. **Fundamental Analysis**: No fundamental data
3. **News Trading**: No news feed
4. **Crypto Trading**: No cryptocurrency support
5. **Bond Trading**: No fixed income instruments

### Recommended Combinations
- **Angel One + Alpha Vantage**: Cover India + US markets
- **Angel One + Screener.in**: Add fundamental data
- **Angel One + NewsAPI**: Add news feed
- **Angel One + TradingView**: Enhanced charting + global coverage
