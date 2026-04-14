# Alpaca - Data Coverage

## Geographic Coverage

### Regions Supported

**Trading (Brokerage Services):**
- North America: **Yes** (US only)
- Europe: **No** (paper trading only)
- Asia: **No** (paper trading only)
- Other: **No** (paper trading only)

**Paper Trading:**
- Global: **Yes** (anyone can create paper-only account)

**Live Trading:**
- US Residents: **Yes** (individuals and businesses)
- Non-US: **No** (cannot open live brokerage accounts)

**Market Data:**
- Global: **Yes** (anyone can subscribe, no geographic restrictions for data)

### Country-Specific

**Live Trading Accounts:**
- US: **Yes** (primary market)
- Canada: **No**
- UK: **No**
- Europe: **No**
- Asia: **No**
- Other: **No**

**Paper Trading:**
- All countries: **Yes** (email signup only, no verification)

### Restricted Regions

**Blocked countries:**
- Not explicitly documented
- Likely standard OFAC sanctions list (Iran, North Korea, Syria, Crimea, Cuba)
- No specific documentation on blocked regions for market data

**VPN detection:**
- Not documented
- Paper trading likely allows VPN
- Live trading may flag VPN usage during KYC

**Geo-fencing:**
- Not documented
- Market data API accessible globally
- Live trading restricted to US residents only

---

## Markets/Exchanges Covered

### Stock Markets

**US Exchanges (Full Coverage):**
- NYSE: **Yes** (New York Stock Exchange)
- NASDAQ: **Yes**
- AMEX: **Yes** (American Stock Exchange)
- ARCA: **Yes** (NYSE Arca)
- BATS: **Yes** (Cboe BZX)
- IEX: **Yes** (Investors Exchange)
- Other US exchanges: **Yes** (via SIP feed on paid tier)

**Feed Options:**
- **IEX Feed (Free):** IEX exchange only (~2.5% of US market volume)
- **SIP Feed (Paid $99/mo):** All US exchanges consolidated (100% market volume)
  - CTA (Consolidated Tape Association) - Tape A/B
  - UTP (Unlisted Trading Privileges) - Tape C

**International Stock Markets:**
- UK (LSE): **No**
- Europe (Euronext, XETRA, etc.): **No**
- Japan (TSE): **No**
- China (SSE, SZSE): **No**
- Hong Kong (HKEX): **No**
- India (NSE, BSE): **No**
- Canada (TSX): **No**
- Australia (ASX): **No**

**Coverage:** **US markets only** - No international stocks

### Crypto Exchanges (Data Aggregator)

Alpaca aggregates crypto data from:
- **Alpaca Crypto:** Alpaca's own crypto exchange
- **Kraken:** Major crypto exchange
- **Others:** Not specified (likely limited to these two)

**Note:** This is NOT a full crypto aggregator like CoinGecko or CoinMarketCap. Only 2 exchanges covered.

**Crypto symbols supported:**
- Major coins: BTC, ETH, LTC, BCH, etc.
- Total: Not specified (likely 50-100 pairs)
- Focus: Major USD trading pairs (BTC/USD, ETH/USD, etc.)

### Forex Brokers

**NOT a forex aggregator** - Alpaca provides basic forex rates via `/v1beta1/forex/rates`

**Currency pairs:**
- Majors: Likely (EUR/USD, GBP/USD, USD/JPY, etc.)
- Minors: Limited
- Exotics: Limited

**Note:** Not a comprehensive forex data provider. Use dedicated forex APIs (Oanda, FXCM) for serious FX trading.

### Futures/Options Exchanges

**Futures:** **NOT supported** - No futures trading or data

**Options (OPRA Feed):**
- CBOE: **Yes** (Chicago Board Options Exchange)
- Other options exchanges: **Yes** (all OPRA members)
- Coverage: All US equity options

**Option data includes:**
- All strikes and expirations for supported underlyings
- Chains for stocks with `options_enabled: true`
- Greeks, implied volatility, latest trade/quote

---

## Instrument Coverage

### Stocks

**Total symbols:**
- US stocks: **~8,000+** (all actively traded US equities)
- ETFs: **~3,000+**
- Total: **~11,000+ tradable symbols**

**Coverage:**
- NYSE: All listed stocks
- NASDAQ: All listed stocks
- AMEX: All listed stocks
- OTC: **Limited** (some OTC stocks, not comprehensive)
- Penny stocks: **Yes** (if listed on major exchanges)
- Fractional shares: **2,000+ symbols** (major stocks and ETFs)

**Asset classes:**
- US Equities: **Yes**
- ETFs: **Yes**
- ADRs: **Yes** (American Depositary Receipts for foreign companies)
- REITs: **Yes**
- Closed-end funds: **Yes**

**Filter via API:**
- `/v2/assets?status=active&asset_class=us_equity`
- Filter by exchange, tradable, shortable, marginable, fractionable

### Options

**Underlying symbols with options:**
- Filter: `options_enabled: true` in assets endpoint
- Coverage: **1,000+ underlying stocks**
- Major stocks and ETFs have option contracts

**Option contracts:**
- Total contracts: **Tens of thousands** (all strikes × expirations for covered underlyings)
- Expirations: Weekly, monthly, quarterly
- Strikes: All available strikes for each expiration

**Option chain depth:**
- `/v2/option_contracts` endpoint lists all available contracts
- Filter by underlying, expiration date, strike range, type (call/put)

### Crypto

**Total coins:**
- Not explicitly documented
- Estimate: **50-100 trading pairs** (major coins)
- Focus: USD pairs (BTC/USD, ETH/USD, etc.)

**Spot pairs:**
- BTC/USD, ETH/USD, LTC/USD, BCH/USD, etc.
- Other pairings: Limited (mostly USD pairs)

**Futures/Perpetuals:**
- **NOT supported** - Spot trading only

**Stablecoins:**
- USDT, USDC: Likely available for trading
- Used as quote currency for some pairs

### Forex

**Currency pairs:**
- Total: **Not specified** (likely 20-50 major pairs)
- Majors: EUR/USD, GBP/USD, USD/JPY, etc. (likely 7 pairs)
- Minors: EUR/GBP, AUD/NZD, etc. (limited)
- Exotics: USD/TRY, USD/ZAR, etc. (very limited)

**Note:** NOT a comprehensive forex provider. Use for quick currency conversions, not serious FX trading.

### Commodities

**Direct commodities trading:** **NOT supported**

**Commodity ETFs:** **Yes**
- GLD (Gold ETF)
- USO (Oil ETF)
- SLV (Silver ETF)
- DBA (Agriculture ETF)
- etc.

**Access:** Trade commodity exposure via ETFs, not direct futures

### Indices

**Direct index trading:** **NOT supported**

**Index ETFs:** **Yes**
- SPY (S&P 500)
- QQQ (Nasdaq 100)
- DIA (Dow Jones)
- IWM (Russell 2000)
- VTI (Total Stock Market)
- etc.

**Index data:**
- Not directly available as separate instruments
- Track via ETFs that follow indices

**Crypto indices:**
- NOT supported (no crypto index data)

---

## Data History

### Historical Depth

**Stocks:**
- Daily bars: **7+ years** (back to ~2016-2017)
- Minute bars: **5 years** (back to 2016 minimum)
- Tick data: **NOT available**
- Trades: **7+ years**
- Quotes: **7+ years**

**Crypto:**
- Daily/minute bars: **6+ years**
- Trades: **6+ years**
- Orderbook: **Current only** (no historical orderbook snapshots documented)

**Options:**
- Historical bars: **Available** (depth not specified, likely 2-3 years)
- Historical trades: **Available**
- Historical quotes: **Available**
- Greeks history: **Not documented** (likely snapshot only, not historical Greeks)

**Forex:**
- Historical rates: **Limited documentation** (likely several years)

### Granularity Available

**Tick data:**
- **NOT available** - No raw tick-by-tick data

**Sub-second bars:**
- **NOT available** - Minimum 1 minute

**1-minute bars:**
- **Yes** - Available back 5 years (2016+)
- All stocks, crypto, options

**5-minute bars:**
- **Yes** - Calculated from 1-minute data or direct API

**15-minute bars:**
- **Yes**

**30-minute bars:**
- **Yes**

**Hourly bars:**
- **Yes** (1Hour, 4Hour)

**Daily bars:**
- **Yes** - 7+ years of history

**Weekly/Monthly bars:**
- **Yes** (1Week timeframe supported)
- Monthly: Can be calculated from daily data

**Custom timeframes:**
- **Yes** (via SDK, e.g., 45-minute bars)
- Use TimeFrame constructor in Python/JS SDKs

### Real-time vs Delayed

**Real-time:**
- **Free tier:** Real-time via WebSocket (IEX exchange only)
- **Paid tier:** Real-time via WebSocket (all exchanges, SIP feed)
- **REST API:** "Latest" endpoints provide near-real-time (within seconds)

**Delayed:**
- **Free tier REST:** 15-minute delay for historical data
- **Paid tier REST:** No delay (real-time)

**Snapshot:**
- `/snapshots` endpoint: Near-real-time (within seconds)
- Includes latest trade, quote, minute bar, daily bar

**By feed:**
- **IEX (free):** Real-time WebSocket, 15-min delayed REST
- **SIP (paid):** Real-time WebSocket and REST
- **OPRA options (paid):** Real-time Greeks and IV
- **OPRA indicative (free):** Delayed options data

---

## Update Frequency

### Real-time Streams (WebSocket)

**Price updates:**
- **Every trade:** Real-time (millisecond latency)
- **Every quote:** Real-time bid/ask updates
- **Frequency:** Depends on market activity (can be hundreds/second for active stocks)

**Orderbook (crypto only):**
- **Snapshot + delta:** Full orderbook sent initially, then updates
- **Update frequency:** Real-time (every orderbook change)

**Bars:**
- **Minute bars:** Sent when minute closes (every 60 seconds)
- **Daily bars:** Updated intraday (every few seconds during market hours)

**Trades:**
- **Latency:** <100ms typical (from exchange to client)
- **Batching:** Server may batch multiple trades in single message array

### Scheduled Updates

**Corporate actions:**
- **Frequency:** Daily
- **Timing:** Available morning after declaration date
- **Source:** Third-party data vendor (ingested by Alpaca)

**News:**
- **Frequency:** Real-time
- **Sources:** Benzinga, others
- **Latency:** Seconds to minutes after publication

**Fundamentals (limited):**
- **NOT provided** - No earnings, financial statements
- **Corporate actions:** Dividends (quarterly/annual), splits (as announced)

**Economic data:**
- **NOT provided** by Alpaca

### Market Hours

**Regular trading hours (EST):**
- Monday-Friday: 9:30 AM - 4:00 PM
- Real-time data during these hours

**Extended hours:**
- Pre-market: 4:00 AM - 9:30 AM
- After-hours: 4:00 PM - 8:00 PM
- **BOATS feed (paid):** Real-time extended hours
- **Overnight feed (free):** 15-minute delayed extended hours

**Crypto (24/7):**
- Trading: 24 hours, 7 days a week
- Data: Continuous real-time updates

**Weekends:**
- Stocks: No trading, no data updates
- Crypto: Trading and data available 24/7

---

## Data Quality

### Accuracy

**Source:**
- **Stocks:** Direct from exchanges via CTA/UTP feeds (SIP)
  - Free tier: Direct from IEX
  - Paid tier: Consolidated from all US exchanges
- **Options:** Direct from OPRA (Options Price Reporting Authority)
- **Crypto:** Alpaca and Kraken exchanges
- **News:** Third-party (Benzinga, others)

**Validation:**
- **Yes** - Data validated by Alpaca before distribution
- Trade conditions, quote conditions included

**Corrections:**
- **Automatic** - Trade corrections sent via WebSocket `corrections` channel
- **Cancellations** - Trade cancels sent via `cancelErrors` channel
- **Updated bars** - Late trades update bars via `updatedBars` channel

### Completeness

**Missing data:**
- **Rare** for major stocks (SIP feed)
- **Possible** for low-volume stocks during off-hours
- **Gaps:** Handled by API (no data = no records returned)

**Backfill:**
- **Available** - Historical data can be requested at any time
- No documented limits on backfill requests (within rate limits)

**Extended hours:**
- **Paid tier:** Full coverage via BOATS
- **Free tier:** 15-min delayed overnight feed

**Halts:**
- **Trading status channel:** Real-time halt and resumption notifications
- **LULD bands:** Limit Up-Limit Down bands sent in real-time

### Timeliness

**Latency (real-time streams):**
- **Stocks:** <100ms typical (exchange → Alpaca → client)
- **Options:** <100ms (OPRA feed)
- **Crypto:** <50ms (direct from Alpaca/Kraken)

**Delay (free tier REST):**
- **15 minutes** for historical data
- **Latest endpoints:** Near-real-time (seconds)

**Market hours:**
- **Covered fully:** 4:00 AM - 8:00 PM EST (extended hours included on paid tier)
- **After-hours:** Available on paid tier (BOATS feed)

**Corporate actions:**
- **Next business day** after declaration (no real-time corporate action updates)

---

## Coverage Limitations

### What's NOT Covered

1. **International stocks** - US only
2. **Futures/derivatives** - No futures, perpetuals, swaps
3. **Comprehensive forex** - Limited currency pairs
4. **Level 2 orderbook for stocks** - Only crypto has orderbook
5. **Tick-by-tick data** - Only aggregated bars
6. **Fundamentals** - No financials, earnings, ratios
7. **Economic data** - No GDP, CPI, employment
8. **On-chain crypto** - No DEX, wallet, gas data
9. **Sentiment data** - No social sentiment, analyst opinions
10. **OTC markets** - Limited OTC coverage

### Best Coverage

Alpaca has **excellent coverage** for:
- ✅ US stocks (all exchanges via SIP)
- ✅ US options (full OPRA feed)
- ✅ Real-time WebSocket streams
- ✅ Historical bars (7+ years)
- ✅ Paper trading (free, unlimited)
- ✅ Commission-free trading
- ✅ Extended hours data (paid tier)
- ✅ Crypto spot trading (24/7)

### Adequate Coverage

Alpaca has **adequate coverage** for:
- ⚠️ Crypto (limited to 2 exchanges)
- ⚠️ News (real-time, but limited sources)
- ⚠️ Corporate actions (next-day updates)
- ⚠️ Forex (basic rates only)

### Poor Coverage

Alpaca has **poor/no coverage** for:
- ❌ International equities
- ❌ Fundamental data
- ❌ Economic indicators
- ❌ Sentiment analysis
- ❌ Level 2 stock orderbook
- ❌ Futures/derivatives

---

## Comparison to Competitors

| Feature | Alpaca | Polygon.io | Alpha Vantage | IEX Cloud |
|---------|--------|------------|---------------|-----------|
| **US Stocks** | ✅ Excellent | ✅ Excellent | ✅ Good | ✅ Good |
| **Options** | ✅ Full OPRA | ✅ Full OPRA | ❌ No | ⚠️ Limited |
| **Crypto** | ⚠️ Limited | ✅ Comprehensive | ⚠️ Limited | ❌ No |
| **Forex** | ⚠️ Basic | ✅ Good | ✅ Good | ❌ No |
| **Fundamentals** | ❌ No | ✅ Yes | ✅ Yes | ✅ Yes |
| **News** | ✅ Yes | ✅ Yes | ❌ No | ✅ Yes |
| **Trading** | ✅ Built-in | ❌ No | ❌ No | ❌ No |
| **Free Tier** | ✅ Generous | ⚠️ Limited | ✅ Good | ⚠️ Limited |
| **Paid Price** | $99/mo | $199+/mo | $50+/mo | $0-100+/mo |

**Alpaca's unique value:** **Integrated trading + data** in one platform
