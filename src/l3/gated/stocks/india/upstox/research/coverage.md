# Upstox - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America:** No
- **Europe:** No
- **Asia:** Yes (India only)
- **Other:** No

### Country-Specific
- **India:** Yes (primary market)
- **US:** No
- **UK:** No
- **Japan:** No
- **China:** No
- **Singapore:** No
- **Hong Kong:** No
- **Other countries:** No

**Upstox is exclusively focused on Indian financial markets.**

### Restricted Regions
- **Blocked countries:** Not specified (likely follows Indian regulations)
- **VPN detection:** Not specified
- **Geo-fencing:** Not specified
- **Compliance:** Subject to SEBI (Securities and Exchange Board of India) regulations
- **Account requirements:** Indian KYC (Know Your Customer) required for trading

---

## Markets/Exchanges Covered

### Stock Markets

| Exchange | Supported | Segments | Notes |
|----------|-----------|----------|-------|
| **NSE** (National Stock Exchange) | Yes | NSE_EQ, NSE_FO, NSE_INDEX, NSE_COM | Primary exchange |
| **BSE** (Bombay Stock Exchange) | Yes | BSE_EQ, BSE_FO, BSE_INDEX | Second largest |
| **MCX** (Multi Commodity Exchange) | Yes | MCX_FO | Commodities |
| NYSE | No | - | Not supported |
| NASDAQ | No | - | Not supported |
| LSE | No | - | Not supported |
| TSE (Tokyo) | No | - | Not supported |
| SSE/SZSE (China) | No | - | Not supported |

**Supported Segments:**
- **NSE_EQ:** NSE Equity (Cash segment)
- **NSE_FO:** NSE Futures & Options
- **NSE_INDEX:** NSE Indices (Nifty 50, Bank Nifty, etc.)
- **NSE_COM:** NSE Commodity derivatives
- **BSE_EQ:** BSE Equity (Cash segment)
- **BSE_FO:** BSE Futures & Options
- **BSE_INDEX:** BSE Indices (Sensex, etc.)
- **BCD_FO:** BSE Currency derivatives
- **MCX_FO:** MCX Commodity futures
- **NCD_FO:** NSE Currency derivatives

### Crypto Exchanges
**Not supported** - Upstox does not provide cryptocurrency trading or data.

### Forex Brokers
**Not applicable** - Upstox is not a forex broker. Only currency futures/derivatives are available (via NSE/BSE).

### Futures/Options Exchanges
- **NSE F&O:** Yes (NSE_FO segment)
- **BSE F&O:** Yes (BSE_FO segment)
- **MCX:** Yes (commodities futures)
- **NSE Currency:** Yes (NCD_FO segment)
- **BSE Currency:** Yes (BCD_FO segment)
- **CME (US):** No
- **CBOE (US):** No
- **Eurex:** No

---

## Instrument Coverage

### Stocks

| Category | Count | Details |
|----------|-------|---------|
| **Total symbols** | ~10,000+ | NSE + BSE combined |
| **NSE stocks** | ~2,000+ | NSE_EQ segment |
| **BSE stocks** | ~8,000+ | BSE_EQ segment |
| **International stocks** | 0 | India only |
| **OTC** | No | No OTC trading |
| **Penny stocks** | Yes | Available on NSE/BSE |

**Major Stocks:**
- Large-cap: All Nifty 50, Sensex 30 constituents
- Mid-cap: Nifty Midcap, BSE Midcap
- Small-cap: Nifty Smallcap, BSE Smallcap
- Micro-cap: Yes (on BSE)

### Indices

**NSE Indices (NSE_INDEX):**
- Nifty 50
- Nifty Bank
- Nifty IT
- Nifty Auto
- Nifty Financial Services
- Nifty Midcap 50/100/150
- Nifty Smallcap 50/100/250
- Nifty Next 50
- And 50+ other sector/thematic indices

**BSE Indices (BSE_INDEX):**
- Sensex (BSE 30)
- BSE 100
- BSE 200
- BSE 500
- BSE Midcap
- BSE Smallcap
- Sector indices (Auto, Bankex, IT, etc.)

### Futures

**Equity Futures:**
- Single stock futures (NSE, BSE)
- Index futures (Nifty, Bank Nifty, Sensex, etc.)

**Commodity Futures (MCX):**
- Precious metals: Gold, Silver
- Base metals: Copper, Zinc, Lead, Nickel, Aluminium
- Energy: Crude Oil, Natural Gas
- Agri: Cotton, Castor Seed, etc.

**Currency Futures (NSE/BSE):**
- USD-INR
- EUR-INR
- GBP-INR
- JPY-INR

**Total F&O instruments:** ~5,000+ (varies with expiries)

### Options

**Equity Options:**
- Single stock options (limited stocks)
- Index options (Nifty, Bank Nifty, FinNifty, etc.)

**Option Chain Coverage:**
- Available for: NSE, BSE
- **Not available for:** MCX (option chain endpoint)
- Strikes: All available strikes for given expiry
- Expiries: Weekly, monthly

**Total Options instruments:** ~20,000+ (varies with strikes and expiries)

### Commodities

**MCX Commodities:**
- **Bullion:** Gold, Silver, Gold Mini, Silver Mini
- **Base Metals:** Copper, Zinc, Lead, Nickel, Aluminium
- **Energy:** Crude Oil, Natural Gas
- **Agriculture:** Cotton, Castor Seed, Cardamom, Crude Palm Oil, etc.

**Total commodity instruments:** ~100+ (active contracts)

### Forex (Currency Derivatives)

**NSE Currency Derivatives:**
- USD-INR (futures)
- EUR-INR (futures)
- GBP-INR (futures)
- JPY-INR (futures)

**BSE Currency Derivatives:**
- USD-INR (futures)
- EUR-INR (futures)
- GBP-INR (futures)
- JPY-INR (futures)

**Note:** Only currency futures/options are available, not spot forex.

### Bonds
**Not covered** - Government and corporate bonds not available via API.

### Mutual Funds
**Not directly available** via API. (Third-party APIs may be needed for mutual fund data.)

### ETFs
**Yes** - Exchange-Traded Funds available on NSE and BSE (traded like stocks in NSE_EQ, BSE_EQ segments).

---

## Data History

### Historical Depth

| Data Type | Start Date | Depth | Notes |
|-----------|------------|-------|-------|
| **Daily bars** | January 2000 | 24+ years | For stocks, indices |
| **Intraday bars** | January 2022 | 4+ years | Minute/hour candles |
| **Tick data** | Not available | - | No tick-by-tick data |
| **Options** | January 2000 | 24+ years | Daily historical |
| **Futures** | January 2000 | 24+ years | Daily historical |
| **Commodities** | January 2000 | 24+ years | Daily historical |

**Historical Depth by Interval:**
- **1-15 minute bars:** Max 1 month (from Jan 2022)
- **>15 minute bars:** Max 1 quarter (from Jan 2022)
- **Hourly bars:** Max 1 quarter (from Jan 2022)
- **Daily bars:** Max 1 decade per request (from Jan 2000)
- **Weekly/Monthly bars:** Unlimited (from Jan 2000)

### Granularity Available

| Granularity | Available | From When | Max Request Range |
|-------------|-----------|-----------|-------------------|
| **Tick data** | No | - | - |
| **1-minute bars** | Yes | Jan 2022 | 1 month |
| **5-minute bars** | Yes | Jan 2022 | 1 month |
| **15-minute bars** | Yes | Jan 2022 | 1 month |
| **30-minute bars** | Yes | Jan 2022 | 1 quarter |
| **1-hour bars** | Yes | Jan 2022 | 1 quarter |
| **Daily** | Yes | Jan 2000 | 1 decade |
| **Weekly** | Yes | Jan 2000 | Unlimited |
| **Monthly** | Yes | Jan 2000 | Unlimited |

**Custom Intervals (V3 API):**
- Minutes: 1-300 (any value, e.g., 1, 2, 3, 5, 7, 10, 15, 30, 60, 90, 120, etc.)
- Hours: 1-5

### Real-time vs Delayed

| Data Type | Real-time | Delayed | Free Tier |
|-----------|-----------|---------|-----------|
| **Market quotes** | Yes | No | Requires subscription |
| **WebSocket feed** | Yes | No | Requires subscription |
| **Historical data (public)** | - | - | Some endpoints public |
| **Order/Trade updates** | Yes | No | Requires subscription |

- **Real-time:** Yes (with API subscription)
- **Delayed:** No (all data is real-time or historical)
- **Snapshot:** Yes (REST API quotes are snapshots)

---

## Update Frequency

### Real-time Streams (WebSocket)

| Data Type | Update Frequency | Latency |
|-----------|------------------|---------|
| **Price updates** | Real-time (tick-by-tick) | <100ms |
| **Orderbook** | Snapshot + delta updates | <100ms |
| **Trades** | Real-time (each trade) | <100ms |
| **Option Greeks** | Real-time | <100ms |
| **Order updates** | Real-time (status changes) | <100ms |
| **Position updates** | Real-time | <100ms |

**WebSocket Feed Characteristics:**
- **Market Data Feed:** Binary Protocol Buffers (efficient, low latency)
- **Portfolio Feed:** Real-time order/position/holding updates
- **Ping/Pong:** Automatic keep-alive

### REST API Polling

**Market Data Endpoints:**
- **Rate Limit:** 50 requests/second, 500 requests/minute
- **Recommended Polling:** 1-5 seconds (for real-time quotes)
- **Data Freshness:** <1 second (exchange data)

### Scheduled Updates

| Data Type | Update Frequency | Timing |
|-----------|------------------|--------|
| **Instrument files** | Daily | ~6:00 AM IST (BOD update) |
| **Corporate actions** | As announced | Exchange updates |
| **Market holidays** | Calendar-based | Exchange calendar |

---

## Data Quality

### Accuracy
- **Source:** Direct from NSE, BSE, MCX exchanges
- **Validation:** Exchange-validated data
- **Corrections:** Automatic from exchange (corporate actions, price adjustments)
- **Third-party verification:** Not applicable (primary source data)

### Completeness
- **Missing data:** Rare (only during exchange outages)
- **Gaps:** Handled via exchange (non-trading days, market holidays, circuit breakers)
- **Backfill:** Available via historical APIs
- **Data integrity:** High (exchange-grade data)

### Timeliness
- **Latency:** <100ms for WebSocket real-time data
- **Delay:** None (real-time data)
- **Market hours:** Full coverage during trading hours
- **Pre-market:** Supported for NSE F&O (from December 2025)
- **After-market:** AMO (After-Market Order) support
- **Non-market hours:** No live data (use historical endpoints)

**Trading Hours (IST):**
- **Equity (NSE/BSE):**
  - Pre-market: 9:00 AM - 9:15 AM
  - Regular: 9:15 AM - 3:30 PM
  - Post-close: 3:40 PM - 4:00 PM (closing auction)
- **F&O (NSE):**
  - Pre-open: Available from Dec 2025
  - Regular: 9:15 AM - 3:30 PM
- **Commodities (MCX):**
  - Varies by commodity (some 24-hour trading)

---

## Coverage Limitations

### What's NOT Covered

**Geographic:**
- International markets (US, EU, Asia ex-India)
- Global stocks (AAPL, TSLA, MSFT, etc.)
- ADRs/GDRs (unless listed on NSE/BSE)

**Asset Classes:**
- Cryptocurrency (BTC, ETH, etc.)
- Spot forex (only currency futures)
- Government bonds
- Corporate bonds
- Mutual funds (direct data)
- Real estate
- Precious metals (spot) - only MCX futures
- Private equity

**Data Types:**
- Fundamental data (limited)
- News and sentiment
- Economic indicators
- Analyst research
- Insider trading data
- Institutional holdings
- Corporate governance data
- Social sentiment

**Exchanges:**
- NYSE, NASDAQ (US)
- LSE (UK)
- TSE (Japan)
- SSE, SZSE (China)
- HKEX (Hong Kong)
- SGX (Singapore)
- And all other global exchanges

### India-Specific Limitations

**Segments Not Covered:**
- Corporate bonds (direct)
- Government securities (G-Secs)
- SDLs (State Development Loans)
- T-Bills
- Repo/Reverse Repo
- Mutual funds (NAV data)

**Option Chain Limitation:**
- Available: NSE, BSE
- **Not available: MCX** (commodity options)

---

## Instrument Identification

### Instrument Key Format

**Structure:** `{SEGMENT}|{IDENTIFIER}`

**Examples:**
- Equity: `NSE_EQ|INE669E01016` (ISIN-based)
- Index: `NSE_INDEX|Nifty 50` (name-based)
- Futures: `NSE_FO|45678` (exchange token)
- Options: `NSE_FO|54321` (exchange token)
- BSE Equity: `BSE_EQ|INE002A01018` (ISIN-based)
- MCX Commodity: `MCX_FO|67890` (exchange token)

**Identifiers:**
- **Equity:** ISIN (e.g., INE669E01016)
- **F&O:** Exchange token (numeric)
- **Index:** Index name (e.g., "Nifty 50", "Sensex")

---

## Data Availability by Segment

| Segment | Instruments | Historical | Real-time | Options | WebSocket |
|---------|-------------|------------|-----------|---------|-----------|
| NSE_EQ | ~2,000 | Daily: 2000+, Intraday: 2022+ | Yes | Limited stocks | Yes |
| BSE_EQ | ~8,000 | Daily: 2000+, Intraday: 2022+ | Yes | Very limited | Yes |
| NSE_FO | ~5,000 | Daily: 2000+, Intraday: 2022+ | Yes | Yes | Yes |
| BSE_FO | ~1,000 | Daily: 2000+, Intraday: 2022+ | Yes | Yes | Yes |
| NSE_INDEX | ~100+ | Daily: 2000+, Intraday: 2022+ | Yes | No (indices) | Yes |
| BSE_INDEX | ~50+ | Daily: 2000+, Intraday: 2022+ | Yes | No (indices) | Yes |
| MCX_FO | ~100+ | Daily: 2000+, Intraday: 2022+ | Yes | No (API limit) | Yes |
| NCD_FO | ~10 | Daily: 2000+, Intraday: 2022+ | Yes | Yes | Yes |
| BCD_FO | ~10 | Daily: 2000+, Intraday: 2022+ | Yes | Yes | Yes |

---

## Summary

### Strong Coverage
- **Indian equities:** Excellent (NSE, BSE)
- **Indian F&O:** Excellent (equity, index, commodity, currency)
- **Real-time data:** Excellent (WebSocket, REST)
- **Historical data:** Excellent (24+ years daily, 4+ years intraday)
- **Options data:** Excellent (Greeks, chains, real-time)
- **Market depth:** Good (5 levels standard, 30 levels Plus)

### Weak Coverage
- **International markets:** None
- **Fundamental data:** Limited
- **News/sentiment:** None
- **Economic data:** None
- **Bonds:** None
- **Mutual funds:** Limited/None
- **Crypto:** None

### Ideal For
- Indian equity trading
- Options trading (NSE, BSE)
- Algorithmic trading on Indian markets
- Intraday/swing trading
- Futures and derivatives trading
- Portfolio tracking (Indian assets)

### Not Suitable For
- Global diversification
- Fundamental analysis (need external data)
- News-driven trading
- Cryptocurrency trading
- Forex trading (only futures available)
- Bond trading
