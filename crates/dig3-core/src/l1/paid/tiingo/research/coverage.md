# Tiingo - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: Yes (primary focus)
  - United States: Full coverage
  - Canada: Limited (not explicitly documented)
  - Mexico: Not documented
- **Europe**: Limited (not primary focus)
- **Asia**: Yes (China specifically)
  - China: Yes (Chinese equities included)
  - Japan: Not documented
  - India: Not documented
  - Other Asian markets: Not documented
- **Other**: Limited global coverage via international stocks

### Country-Specific

#### United States
- **Coverage**: Full (primary market)
- **Exchanges**: NYSE, NASDAQ, AMEX, OTC Group
- **Equities**: 32,000+ US stocks
- **ETFs/Mutual Funds**: 33,000+
- **Historical Depth**: 50+ years
- **Real-time**: Yes (via IEX)
- **Intraday**: Yes (via IEX)

#### China
- **Coverage**: Yes
- **Exchanges**: SSE (Shanghai Stock Exchange), SZSE (Shenzhen Stock Exchange)
- **Equities**: Included in 32,000+ total (specific count not provided)
- **Historical Depth**: Not specified (likely less than US stocks)
- **Real-time**: Not specified

#### Other Countries
- **UK (LSE)**: Not explicitly documented
- **Japan (TSE)**: Not explicitly documented
- **India (NSE, BSE)**: Not explicitly documented
- **Germany (XETRA)**: Not explicitly documented
- **Canada (TSX)**: Not explicitly documented

**Note**: Tiingo primarily focuses on US markets with Chinese equities as secondary coverage. For comprehensive international coverage, use providers like IEX Cloud (global), Alpha Vantage, or Yahoo Finance.

### Restricted Regions
- **Blocked countries**: Not explicitly documented (likely follows US sanctions: Iran, North Korea, Syria, Cuba, Crimea)
- **VPN detection**: Not documented
- **Geo-fencing**: Not documented (API access likely global, subject to US export laws)
- **GDPR compliance**: Not explicitly documented

---

## Markets/Exchanges Covered

### Stock Markets

#### United States
- [x] **NYSE** (New York Stock Exchange)
- [x] **NASDAQ** (NASDAQ Stock Market)
- [x] **AMEX** (NYSE American, formerly American Stock Exchange)
- [x] **OTC Group** (OTC Markets Group: OTCQX, OTCQB, Pink Sheets)
- [x] **Total US Equities**: 32,000+

#### China
- [x] **SSE** (Shanghai Stock Exchange)
- [x] **SZSE** (Shenzhen Stock Exchange)
- [x] **Coverage**: Chinese equities included in total count

#### Other International
- [ ] **LSE** (London Stock Exchange) - Not documented
- [ ] **TSE** (Tokyo Stock Exchange) - Not documented
- [ ] **NSE/BSE** (India) - Not documented
- [ ] **TSX** (Toronto Stock Exchange) - Not documented
- [ ] **HKEX** (Hong Kong) - Not explicitly documented
- [ ] **ASX** (Australia) - Not documented

**Primary Focus**: US markets (NYSE, NASDAQ, AMEX, OTC)

---

### Crypto Exchanges (Aggregated)

Tiingo aggregates data from **40+ crypto exchanges** with **2,100-4,100+ crypto tickers**.

**Major Exchanges Included** (documented examples):
- [x] **Binance** - Largest global exchange
- [x] **Coinbase** - US-based exchange
- [x] **Kraken** - Not explicitly confirmed (likely included)
- [x] **Bitfinex** - Not explicitly confirmed (likely included)
- [x] **Bitstamp** - Not explicitly confirmed (likely included)
- [x] **Huobi** - Not explicitly confirmed (likely included)
- [x] **OKX** - Not explicitly confirmed (likely included)

**Total**: 40+ exchanges (specific list not published)

**Note**: Tiingo aggregates top-of-book data across exchanges. For exchange-specific data or to verify exact exchange list, contact Tiingo support or check ticker metadata.

---

### Forex Brokers/Sources

- **Source**: Tier-1 banks and FX dark pools (direct connections)
- **Quality**: Institutional-grade quotes
- **Coverage**: 140+ currency pairs
- **Type**: OTC (over-the-counter) market, not exchange-based
- **Exchanges/Brokers**: Not applicable (interbank FX market)

**Forex is not exchange-based**. Tiingo connects directly to:
- Tier-1 banks (major global banks that provide liquidity)
- FX dark pools (private liquidity pools)

This provides **institutional-grade FX quotes**, superior to retail broker feeds.

---

### Futures/Options Exchanges

**Not covered** - Tiingo does not provide futures or options data.

- [ ] **CME** (Chicago Mercantile Exchange) - Not covered
- [ ] **CBOE** (Chicago Board Options Exchange) - Not covered
- [ ] **ICE** (Intercontinental Exchange) - Not covered
- [ ] **EUREX** - Not covered

**Alternative**: For futures/options, use:
- CME DataMine (CME data)
- CBOE API (options data)
- IEX Cloud (some options data)
- Interactive Brokers API (futures/options)

---

## Instrument Coverage

### Stocks

#### Total Coverage
- **Total symbols**: ~65,000+ (32,000+ US equities + 33,000+ ETFs/mutual funds)
- **US stocks**: 32,000+
- **Chinese stocks**: Included (specific count not provided, estimated several thousand)
- **International stocks**: Limited (not primary focus)
- **ADRs** (American Depositary Receipts): Yes (included in US coverage)
- **OTC**: Yes (OTC Markets Group: OTCQX, OTCQB, Pink Sheets)
- **Penny stocks**: Yes (included in OTC coverage)

#### By Exchange
- **NYSE**: Full coverage
- **NASDAQ**: Full coverage
- **AMEX (NYSE American)**: Full coverage
- **OTC Group**: Full coverage (OTCQX, OTCQB, Pink)

#### ETFs & Mutual Funds
- **ETFs**: ~33,000+ (included in total)
- **Mutual Funds**: ~33,000+ (included in total)
- **CEFs** (Closed-End Funds): Yes (NAV data for 33,000 funds/CEFs)

#### Special Categories
- **REITs**: Yes (traded as stocks)
- **Preferred Stocks**: Yes (likely included)
- **Warrants**: Not documented
- **Rights**: Not documented

---

### Crypto

#### Coverage
- **Total coins/tokens**: 2,100 - 4,100+ (range from different sources)
- **Exchanges aggregated**: 40+
- **Pairs**: Thousands (coin x quote currency x exchange combinations)

#### Major Coins (Examples)
- [x] Bitcoin (BTC)
- [x] Ethereum (ETH)
- [x] Litecoin (LTC)
- [x] Ripple (XRP)
- [x] Bitcoin Cash (BCH)
- [x] And thousands more...

#### Quote Currencies
- USD, USDT, EUR, BTC, ETH, and more (depends on exchange)

#### Spot vs Derivatives
- **Spot pairs**: Yes (primary coverage)
- **Futures**: Not documented (likely not covered)
- **Perpetuals**: Not documented (likely not covered)
- **Options**: Not documented (likely not covered)

**Note**: Tiingo focuses on spot crypto markets. For crypto derivatives, use exchange APIs (Binance, Bybit, OKX) or CoinGlass.

---

### Forex

#### Coverage
- **Currency pairs**: 140+
- **Source**: Tier-1 banks and FX dark pools
- **Quality**: Institutional-grade (not retail broker feeds)

#### Major Pairs (7 pairs)
- [x] EUR/USD (Euro / US Dollar)
- [x] USD/JPY (US Dollar / Japanese Yen)
- [x] GBP/USD (British Pound / US Dollar)
- [x] USD/CHF (US Dollar / Swiss Franc)
- [x] AUD/USD (Australian Dollar / US Dollar)
- [x] USD/CAD (US Dollar / Canadian Dollar)
- [x] NZD/USD (New Zealand Dollar / US Dollar)

#### Minors & Exotics
- **Minors**: Likely included (EUR/GBP, EUR/JPY, GBP/JPY, etc.)
- **Exotics**: Likely included (USD/TRY, USD/ZAR, USD/MXN, etc.)
- **Total**: 140+ pairs (specific list not published)

#### Cross Rates
- **Calculated**: Yes (cross rates available, e.g., EUR/GBP from EUR/USD and GBP/USD)
- **Direct quotes**: Yes (from tier-1 banks)

---

### Commodities

**Not explicitly covered** - Tiingo does not provide dedicated commodities data.

- [ ] **Metals**: Gold, Silver, Platinum (not directly covered)
- [ ] **Energy**: Oil, Gas, Natural Gas (not directly covered)
- [ ] **Agriculture**: Corn, Wheat, Soybeans (not directly covered)

**Workaround**: Some commodity ETFs may be covered (e.g., GLD for gold, USO for oil).

**Alternative**: For commodities, use:
- Quandl/Nasdaq Data Link (commodities data)
- Alpha Vantage (commodities API)
- CME DataMine (futures prices)
- IEX Cloud (some commodity indices)

---

### Indices

**Not explicitly documented** - Index data not a primary focus.

- [ ] **US Indices**: S&P 500, Nasdaq Composite, Dow Jones (not directly covered)
- [ ] **International Indices**: FTSE, Nikkei, DAX (not directly covered)
- [ ] **Crypto Indices**: BTC Dominance, DeFi Index (not directly covered)

**Workaround**: Index ETFs may be covered (e.g., SPY for S&P 500, QQQ for Nasdaq).

**Alternative**: For indices, use:
- Yahoo Finance (free index data)
- Alpha Vantage (index API)
- IEX Cloud (index data)

---

## Data History

### Historical Depth

#### Stocks (EOD)
- **US Stocks**: **50+ years** (from ~1962-1970s depending on stock)
- **Chinese Stocks**: Not specified (likely since listing dates, less than US)
- **ETFs/Mutual Funds**: Varies (since inception or 50 years, whichever is shorter)
- **Coverage start**: Earliest data from 1960s-1970s

**Example**: Apple (AAPL) has data from 1980-12-12 (IPO date).

#### Intraday (IEX)
- **Historical depth**: Not explicitly specified (likely weeks to months)
- **Real-time**: Yes (current day + recent history)
- **Minute bars**: Available (1min, 5min, 15min, 30min)
- **Hourly bars**: Available (1hour, 4hour)

**Note**: IEX intraday is designed for recent data, not deep historical backtesting.

#### Crypto
- **Historical depth**: Varies by exchange and pair
- **Bitcoin**: Likely several years (since major exchange listings)
- **Altcoins**: Varies (since listing on tracked exchanges)
- **Exchanges**: 40+ exchanges, each with different history

**Note**: Crypto historical depth depends on when exchange started trading the pair.

#### Forex
- **Historical depth**: Not explicitly specified (likely years of data)
- **Major pairs**: Likely deep history (10+ years)
- **Exotics**: May have shorter history

#### Fundamentals
- **Depth**: 5 years (free tier), 15+ years (paid tiers)
- **Coverage**: 20+ years across 5,500+ equities
- **Statements**: Quarterly and annual (10-Q, 10-K filings)
- **Daily metrics**: Market cap, ratios, etc. (updated daily)

---

### Granularity Available

#### Stocks

**Daily (EOD):**
- [x] **Daily bars**: 50+ years
- [x] **Weekly bars**: Resampled from daily
- [x] **Monthly bars**: Resampled from daily
- [x] **Annual bars**: Resampled from daily

**Intraday (IEX):**
- [x] **1-minute bars**: Yes (via resampleFreq=1min)
- [x] **5-minute bars**: Yes
- [x] **15-minute bars**: Yes
- [x] **30-minute bars**: Yes
- [x] **Hourly bars**: Yes (1hour, 4hour)
- [ ] **Tick data**: Not available (aggregated bars only)

#### Crypto
- [x] **1-minute bars**: Yes
- [x] **5-minute bars**: Yes
- [x] **15-minute bars**: Yes
- [x] **Hourly bars**: Yes
- [x] **Daily bars**: Yes
- [ ] **Tick/trade data**: Not available (aggregated only)

#### Forex
- [x] **1-minute bars**: Yes
- [x] **5-minute bars**: Yes
- [x] **15-minute bars**: Yes
- [x] **30-minute bars**: Yes
- [x] **Hourly bars**: Yes (1hour, 4hour)
- [x] **Daily bars**: Yes
- [ ] **Tick data**: Not available

---

### Real-time vs Delayed

#### Stocks
- **Real-time**: Yes (via IEX exchange - free tier included)
- **Delayed**: No delay for IEX data
- **Snapshot**: Yes (current quote via REST API)
- **Streaming**: Yes (WebSocket for real-time updates)

**Note**: IEX provides real-time data. Other exchanges (NYSE, NASDAQ direct feeds) not covered (those require expensive exchange fees).

#### Crypto
- **Real-time**: Yes (top-of-book from 40+ exchanges)
- **Delayed**: No delay
- **Snapshot**: Yes (REST /crypto/top)
- **Streaming**: Yes (WebSocket crypto endpoint)

#### Forex
- **Real-time**: Yes (tier-1 bank quotes)
- **Delayed**: No delay
- **Snapshot**: Yes (REST /fx/top)
- **Streaming**: Yes (WebSocket fx endpoint)

#### Fundamentals
- **Real-time**: No (fundamentals are quarterly/annual)
- **Daily updates**: Yes (market cap, ratios updated daily)
- **Quarterly updates**: Yes (10-Q filings)
- **Annual updates**: Yes (10-K filings)

---

## Update Frequency

### Real-time Streams (WebSocket)

#### IEX (Stocks)
- **Price updates**: Microsecond resolution (firehose)
- **Orderbook**: Top-of-book only (bid/ask updates)
- **Trades**: Real-time trade feed
- **Latency**: <100ms typical (depends on network)

#### Crypto
- **Price updates**: Microsecond resolution (firehose)
- **Orderbook**: Top-of-book only (aggregated across exchanges)
- **Trades**: Real-time (last price updates)

#### Forex
- **Price updates**: Microsecond resolution (firehose)
- **Orderbook**: Top-of-book only (bid/ask from tier-1 banks)
- **Trades**: Not applicable (FX is OTC, quote-driven)

### Scheduled Updates

#### Fundamentals
- **Quarterly**: 10-Q filings (within days/weeks of filing)
- **Annual**: 10-K filings (within days/weeks of filing)
- **Daily metrics**: Market cap, ratios updated daily (EOD)

#### Economic Data
- **Not covered**: Tiingo does not provide economic calendars or macro data

#### News
- **Real-time**: News articles added as crawled (minutes to hours after publication)
- **Frequency**: Continuous (as news is published)

---

## Data Quality

### Accuracy

#### Stocks
- **Source**: Direct from exchanges (EOD), IEX exchange (intraday)
- **Validation**: Assumed (not explicitly documented)
- **Corrections**: Corporate actions (splits, dividends) applied to adjusted prices
- **Quality**: High (reputable sources)

#### Crypto
- **Source**: Aggregated from 40+ exchanges
- **Validation**: Not explicitly documented
- **Corrections**: Exchange-specific (may vary)
- **Quality**: Good (multi-exchange aggregation reduces single-exchange errors)

#### Forex
- **Source**: Tier-1 banks and FX dark pools
- **Validation**: Not explicitly documented
- **Corrections**: Not applicable (real-time quotes)
- **Quality**: Institutional-grade (superior to retail broker feeds)

#### Fundamentals
- **Source**: SEC filings (10-K, 10-Q)
- **Validation**: Standardized format available
- **Corrections**: As-reported and standardized formats
- **Quality**: High (sourced from official regulatory filings)

### Completeness

#### Missing Data
- **Stocks**: Rare (complete coverage for tracked symbols)
- **Crypto**: Common (exchange outages, delisting, low-volume pairs)
- **Forex**: Rare (tier-1 bank quotes are reliable)
- **Fundamentals**: Rare (companies may skip filings, but uncommon)

#### Gaps
- **How handled**: Not explicitly documented
- **Backfill**: Available for historical data (if gaps exist, likely filled retroactively)
- **Forward-fill**: IEX API has `forceFill` parameter (forward-fill missing bars)

#### Coverage Gaps
- **International stocks**: Limited (US + China only)
- **Options/Futures**: Not covered
- **Economic data**: Not covered
- **Level 2 orderbook**: Not covered (top-of-book only)

### Timeliness

#### Real-time
- **Latency**: <100ms typical for WebSocket (microsecond server timestamps)
- **Delay**: No artificial delay (real-time firehose)
- **Market hours**: Covered fully during exchange hours

#### Historical
- **EOD data**: Updated after market close (typically within 1-2 hours)
- **Fundamentals**: Updated within days/weeks of SEC filing
- **News**: Updated minutes to hours after publication

---

## Coverage Comparison

### Tiingo Strengths
1. **50+ years stock history** (excellent for long-term backtesting)
2. **IEX real-time intraday** (free tier, no exchange fees)
3. **Multi-asset platform** (stocks + crypto + forex in one API)
4. **Institutional-grade FX** (tier-1 banks, not retail feeds)
5. **40+ crypto exchanges** (comprehensive aggregation)
6. **Comprehensive fundamentals** (80+ indicators, 15+ years)
7. **WebSocket firehose** (microsecond resolution, free tier)
8. **Transparent pricing** (clear tiers, no hidden fees)

### Tiingo Weaknesses
1. **No options data** (use CBOE, Tradier, or IEX Cloud)
2. **No futures/derivatives** (use CME, exchange APIs)
3. **Limited international stocks** (US + China only, use IEX Cloud or Yahoo Finance for global)
4. **No on-chain crypto data** (use Bitquery, The Graph, Etherscan)
5. **No economic/macro data** (use FRED, Alpha Vantage, Trading Economics)
6. **No Level 2 orderbook** (top-of-book only, use direct exchange feeds)
7. **No analyst ratings/estimates** (use FactSet, Bloomberg, or Seeking Alpha)
8. **IEX intraday only** (no NASDAQ/NYSE direct intraday, use paid feeds)

---

## Summary

### Geographic Coverage
- **Primary**: United States (full coverage)
- **Secondary**: China (equities included)
- **Limited**: International (not primary focus)

### Exchanges Covered
- **US Stocks**: NYSE, NASDAQ, AMEX, OTC (32,000+ equities)
- **Crypto**: 40+ exchanges (2,100-4,100+ tickers)
- **Forex**: Tier-1 banks, FX dark pools (140+ pairs)

### Instrument Coverage
- **Stocks**: 32,000+ US equities, Chinese stocks
- **ETFs/Mutual Funds**: 33,000+
- **Crypto**: 2,100-4,100+ pairs
- **Forex**: 140+ currency pairs
- **Fundamentals**: 5,500+ equities, 80+ indicators

### Historical Depth
- **Stocks**: 50+ years (EOD), weeks/months (IEX intraday)
- **Crypto**: Varies by exchange/pair (years for major coins)
- **Forex**: Years (not specified exactly)
- **Fundamentals**: 5-15+ years (tier-dependent)

### Update Frequency
- **Real-time**: Microsecond WebSocket firehose (stocks, crypto, forex)
- **EOD**: Daily updates (stocks)
- **Fundamentals**: Quarterly/annual + daily metrics
- **News**: Real-time (as crawled)

### Data Quality
- **Stocks**: High (direct from exchanges, SEC filings)
- **Crypto**: Good (40+ exchange aggregation)
- **Forex**: Institutional-grade (tier-1 banks)
- **Fundamentals**: High (SEC filings)

**Overall**: Tiingo excels in US stock coverage (historical + intraday), multi-asset support, and institutional-grade FX. Weaknesses include limited international stocks, no options/futures, and no on-chain crypto data.
