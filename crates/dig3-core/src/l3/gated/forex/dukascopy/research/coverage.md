# Dukascopy - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: Yes (US, Canada)
- **Europe**: Yes (Full coverage, Swiss-based)
- **Asia**: Yes (Japan, Hong Kong, Singapore, etc.)
- **Middle East**: Yes (Limited)
- **Africa**: Yes (Limited, mainly South Africa)
- **South America**: Yes (Limited)
- **Oceania**: Yes (Australia, New Zealand)

**Primary Focus**: Global forex markets (24/5 coverage)

### Country-Specific Markets

**Stocks/Indices Available**:
- **US**: Yes (major stocks, indices)
- **UK**: Yes (FTSE, major stocks)
- **Japan**: Yes (Nikkei, major stocks)
- **Germany**: Yes (DAX, major stocks)
- **France**: Yes (CAC, major stocks)
- **Switzerland**: Yes (SMI, major stocks)
- **Hong Kong**: Yes (Hang Seng, major stocks)
- **Australia**: Yes (ASX indices)
- **India**: Limited
- **China**: Limited (mainly via Hong Kong)

**Forex Coverage**: Worldwide (100+ currency pairs)

### Restricted Regions

- **Blocked countries**: Limited (mainly compliance-related restrictions)
  - North Korea, Iran, Syria (typical sanctions)
  - Country-specific restrictions may apply based on Swiss regulations
- **VPN detection**: Not aggressively enforced for data access
- **Geo-fencing**: Minimal for data APIs (stricter for trading accounts)

**Note**: Binary tick data downloads have no geographic restrictions. JForex SDK and trading accounts may have country-specific requirements.

---

## Markets/Exchanges Covered

### Stock Markets

**US Markets**:
- **NYSE**: Yes (major stocks as CFDs)
- **NASDAQ**: Yes (major stocks as CFDs)
- **AMEX**: Limited
- **Total US stocks**: 608

**European Markets**:
- **LSE** (London): Yes (102 stocks)
- **Euronext**: Yes (France, Netherlands, Belgium)
- **Deutsche Börse** (Germany): Yes (DAX components)
- **SIX** (Switzerland): Yes (SMI components)
- **Borsa Italiana** (Italy): Limited
- **BME** (Spain): Limited

**Asian Markets**:
- **TSE** (Tokyo): Yes (55 stocks)
- **HKEX** (Hong Kong): Yes (major stocks)
- **SGX** (Singapore): Limited
- **ASX** (Australia): Yes (indices)

**Other Markets**:
- **NSE/BSE** (India): Limited
- **SSE/SZSE** (China): Limited
- **JSE** (South Africa): Yes (index)

**Total Stock Coverage**: 600+ stocks (CFDs)
**Focus**: Major blue-chip stocks, not full exchange coverage

### Forex Brokers/Liquidity Providers

**Dukascopy's Model**: ECN (Electronic Communication Network)
**Liquidity Sources**: Aggregated from multiple banks and liquidity providers

**Not an aggregator**: Dukascopy is a broker/data provider, not a multi-broker aggregator.

**Data Source**: Dukascopy's own trading infrastructure (Swiss bank)

### Futures/Options Exchanges

**Not Applicable**:
- **CME**: No direct coverage (CFDs on CME products available)
- **CBOE**: No options data
- **Eurex**: No direct coverage (CFDs on Eurex products available)
- **ICE**: No direct coverage

**CFD Coverage**: Dukascopy offers CFDs on futures contracts, not direct futures access.

### Cryptocurrency Exchanges

**Not an Aggregator**: Dukascopy offers crypto CFDs, not direct exchange data.

**Crypto Coverage**: 33 cryptocurrency pairs
- Bitcoin (BTC)
- Ethereum (ETH)
- Litecoin (LTC)
- Other major coins

**Pairs**: vs USD, EUR, GBP, CHF, JPY

**Data Source**: Dukascopy's crypto CFD prices (not direct exchange feeds)

---

## Instrument Coverage

### Forex

**Total Currency Pairs**: 100+

**Majors** (7 pairs):
- EUR/USD ✓
- GBP/USD ✓
- USD/JPY ✓
- USD/CHF ✓
- USD/CAD ✓
- AUD/USD ✓
- NZD/USD ✓

**Minors** (~30 pairs):
- EUR/GBP, EUR/JPY, EUR/CHF, EUR/AUD, EUR/NZD, EUR/CAD
- GBP/JPY, GBP/CHF, GBP/AUD, GBP/NZD, GBP/CAD
- AUD/JPY, AUD/NZD, AUD/CAD, AUD/CHF
- NZD/JPY, NZD/CAD, NZD/CHF
- CAD/JPY, CAD/CHF
- CHF/JPY
- And more...

**Exotics** (~60 pairs):
- USD/TRY, USD/ZAR, USD/MXN, USD/RUB, USD/PLN, USD/HUF, USD/CZK
- EUR/TRY, EUR/ZAR, EUR/MXN, EUR/RUB, EUR/PLN, EUR/HUF, EUR/CZK
- GBP/TRY, GBP/ZAR, GBP/MXN, GBP/PLN
- Scandinavian: EUR/NOK, EUR/SEK, EUR/DKK, USD/NOK, USD/SEK, USD/DKK
- And more...

**Metals vs Currencies** (~50 combinations):
- XAU/USD, XAU/EUR, XAU/GBP, XAU/CHF, XAU/JPY, XAU/AUD
- XAG/USD, XAG/EUR, XAG/GBP, XAG/CHF, XAG/JPY, XAG/AUD
- XPT/USD, XPD/USD (Platinum, Palladium)
- And more cross combinations

### Cryptocurrencies

**Total Coins**: 33 instruments

**Major Coins**:
- Bitcoin (BTC): vs USD, EUR, GBP, CHF, JPY
- Ethereum (ETH): vs USD, EUR, GBP, CHF, JPY
- Litecoin (LTC): vs USD, EUR, GBP

**Coverage**: Major coins only, not altcoins
**Type**: CFDs (not spot or futures)
**Pairs**: Primarily vs major fiat currencies

### Commodities

**Total**: 13 instruments

**Metals** (3):
- Gold (XAU): vs multiple currencies
- Silver (XAG): vs multiple currencies
- Copper (HG): vs USD

**Energy** (4):
- Crude Oil (WTI): CL
- Brent Crude: BRENT
- Natural Gas: NG
- Heating Oil: HO (if available)

**Agriculture** (6):
- Corn: CORN
- Wheat: WHEAT
- Soybeans: SOYBEAN
- Sugar: SUGAR (if available)
- Coffee: COFFEE (if available)
- Cotton: COTTON (if available)

**Type**: CFDs on futures contracts

### Indices

**Total**: 22 stock indices

**Americas** (6):
- S&P 500: SPX500
- NASDAQ 100: NAS100
- Dow Jones: USA30
- Russell 2000: USA2000 (if available)
- Canada: CAN (TSX)
- Mexico: MEX (IPC)

**Asia** (6):
- Japan: Nikkei 225 (JPN225)
- Hong Kong: Hang Seng (HKG33)
- China: China A50
- Singapore: SGP (STI)
- Australia: AUS200
- India: IND (Nifty 50, if available)

**Europe** (9):
- Germany: DAX (GER30)
- UK: FTSE 100 (GBR100)
- France: CAC 40 (FRA40)
- Spain: IBEX 35 (ESP35)
- Italy: FTSE MIB (ITA40)
- Netherlands: AEX (NED25)
- Switzerland: SMI (SUI20)
- Euro Stoxx 50: EUSTX50
- Poland: WIG20 (if available)

**Africa** (1):
- South Africa: SA40 (FTSE/JSE Top 40)

**Type**: CFDs on indices

### Stocks (CFDs)

**Total**: 600+ stocks

**US Stocks** (608):
- Major tech: AAPL, MSFT, GOOGL, AMZN, META, NVDA, TSLA
- Financials: JPM, BAC, WFC, GS, MS
- Consumer: WMT, HD, MCD, NKE, SBUX
- Healthcare: JNJ, PFE, UNH, ABBV
- Energy: XOM, CVX, COP
- And 590+ more

**UK Stocks** (102):
- Major: BP, HSBC, RDS (Shell), GSK, AZN, LLOY, VOD
- FTSE 100 components
- And more

**Japanese Stocks** (55):
- Major: Toyota, Sony, SoftBank, Honda, Mitsubishi
- Nikkei components
- And more

**Other Markets**: Limited coverage

**Type**: CFDs (not direct equity ownership)
**Focus**: Large-cap, liquid stocks

### ETFs

**Total**: 70+ ETFs

**US ETFs** (62):
- Broad market: SPY, QQQ, IWM, DIA
- Sector: XLF, XLE, XLK, XLV, XLI, XLP, XLU
- Bond: TLT, AGG, LQD
- Commodity: GLD, SLV, USO
- International: EEM, EFA, VWO
- And more

**France ETFs** (3):
- CAC 40 ETFs

**Hong Kong ETFs** (4):
- Hang Seng ETFs

**Germany ETFs** (1):
- DAX ETF

**Type**: CFDs on ETFs

### Bonds

**Total**: 3 instruments

**Government Bonds**:
- Euro Bund: EUR bond futures
- UK Gilts: GBP government bonds
- US T-Bond: USD treasury bonds

**Type**: CFDs on bond futures
**Limited Coverage**: Only major government bonds

---

## Data History

### Historical Depth

**Forex** (Varies by pair):
- **Majors**: From 2003+ (20+ years)
  - EUR/USD: Full history from 2003
  - GBP/USD: Full history from 2003
  - USD/JPY: Full history from 2003
- **Minors**: From 2005-2010 (15+ years)
- **Exotics**: From 2010-2015 (10+ years)

**Cryptocurrencies**:
- **BTC**: From 2017+ (7+ years)
- **ETH**: From 2017+ (7+ years)
- **LTC**: From 2017+ (7+ years)

**Stocks**:
- **US**: Varies (recent years, typically 5-10 years)
- **UK**: Varies (recent years)
- **Japan**: Varies (recent years)

**Indices**:
- **Major indices**: 10-20 years (varies)
- **Regional indices**: 5-10 years

**Commodities**:
- **Metals**: 10+ years
- **Energy**: 10+ years
- **Agriculture**: 10+ years

**Best Coverage**: Major forex pairs (2003+)

### Granularity Available

**Tick Data**:
- Available: **Yes** (primary strength)
- Format: .bi5 binary files (hourly)
- From when: 2003+ (major pairs)
- Historical tick data: **Free and unlimited**

**Intraday Bars**:
- **1-second**: Yes (via SDK, calculated from ticks)
- **10-second**: Yes
- **30-second**: Yes
- **1-minute**: Yes (from 2003+)
- **5-minute**: Yes
- **15-minute**: Yes
- **30-minute**: Yes

**Hourly/Daily**:
- **1-hour**: Yes (full history)
- **4-hour**: Yes (full history)
- **Daily**: Yes (full history, 2003+)
- **Weekly**: Yes (calculated)
- **Monthly**: Yes (calculated)

### Real-time vs Delayed

**Real-time**:
- Available: **Yes** (via JForex SDK, FIX API)
- Free tier: **Yes** (demo account provides real-time data)
- Latency: Sub-second (depends on connection)

**Delayed**:
- Not applicable (Dukascopy provides real-time only)
- No 15-minute delayed feeds

**Snapshot**:
- Available: Yes (getLastTick(), current bar)

---

## Update Frequency

### Real-time Streams

**Tick Updates**:
- Frequency: Every price change (tick-by-tick)
- Typical interval: 100-500ms during active hours
- Latency: <100ms (JForex SDK), <50ms (FIX API)

**Order Book**:
- Type: Snapshot (10 levels)
- Update: With each tick
- Not delta-based (full snapshot each update)

**Trades**:
- Real-time: Yes
- Each trade = new tick

### Scheduled Updates

**Not Applicable**:
- **Fundamentals**: Not provided
- **Economic data**: Not provided (affects forex prices but not provided as data)
- **News**: Not provided via API

**Instrument Metadata**:
- Static: Updated when new instruments added (rare)

---

## Data Quality

### Accuracy

**Source**:
- **Direct from Dukascopy**: Swiss bank's own trading infrastructure
- **Not aggregated**: Dukascopy's ECN liquidity
- **Audited**: Swiss banking standards (FINMA regulated)

**Validation**:
- Yes (Swiss banking compliance)
- Data integrity checks
- Tick-level accuracy

**Corrections**:
- Automatic (bad ticks filtered)
- Rare manual corrections if needed

**Reputation**: Very high quality, trusted for backtesting

### Completeness

**Missing Data**:
- **Rare**: Dukascopy has excellent data continuity
- **Weekends**: No data (forex market closed)
- **Holidays**: Market-dependent (some pairs affected)

**Gaps**:
- How handled: Minimal gaps in liquid pairs
- Backfill: Available (historical data complete)

**Tick Coverage**:
- Comprehensive during market hours
- Every price change recorded

### Timeliness

**Latency**:
- **JForex SDK**: <100ms for real-time ticks
- **FIX API**: <50ms for market data
- **Binary downloads**: N/A (historical)

**Delay**:
- Real-time: No delay (live market data)
- Typical: Sub-second updates

**Market Hours Coverage**:
- **Forex**: 24/5 (Sunday 22:00 GMT - Friday 22:00 GMT)
- **Stocks**: Exchange hours only
- **Crypto**: 24/7 (for crypto CFDs)

---

## Data Gaps & Anomalies

### Known Gaps

**Weekend Gaps**:
- Forex: Closes Friday 22:00 GMT, opens Sunday 22:00 GMT
- 48-hour gap every weekend
- Expected and normal

**Flash Crash Events**:
- Recorded accurately (not filtered)
- CHF flash crash (2015): Fully recorded
- Useful for analyzing extreme events

**System Maintenance**:
- Rare (Swiss bank infrastructure)
- Announced in advance
- Minimal impact

### Data Anomalies

**Spike Filtering**:
- Bad ticks: Filtered automatically
- Valid spikes: Kept (e.g., news events)

**Volume Anomalies**:
- Tick volume varies (not notional)
- Low liquidity hours: Fewer ticks

---

## Comparison with Other Providers

| Provider | Forex Depth | Tick Data | Free? | Real-time | Quality |
|----------|-------------|-----------|-------|-----------|---------|
| Dukascopy | 2003+ | Yes (free) | Yes | Yes | Very High |
| OANDA | 2004+ | No | Limited | Yes | High |
| Interactive Brokers | 10 years | Via API | No | Yes | High |
| Alpha Vantage | Limited | No | Yes | Yes | Medium |
| Polygon | N/A (stocks) | No | Limited | Yes | High |

**Dukascopy Advantages**:
- Free unlimited historical tick data
- 20+ years of forex history
- Swiss bank data quality
- No API fees

---

## Best Use Cases

### Ideal For:
- **Forex backtesting**: Tick-level accuracy, 20+ years
- **Spread analysis**: Historical bid/ask spreads
- **Microstructure research**: Tick-by-tick data
- **Order book studies**: 10-level depth
- **Algorithmic trading**: High-quality historical data
- **Academic research**: Free, comprehensive data

### Not Ideal For:
- **Stock fundamentals**: Limited fundamental data
- **Options trading**: No options data
- **Crypto derivatives**: No funding/OI data
- **Economic indicators**: Not provided
- **News sentiment**: Not provided

---

## Summary

**Strengths**:
- Forex: Exceptional (100+ pairs, 2003+, tick data)
- Data Quality: Very high (Swiss bank)
- Historical Depth: 20+ years (major pairs)
- Granularity: Tick to monthly
- Free Access: Unlimited historical ticks
- Real-time: Free with demo account

**Limitations**:
- Stocks: Limited (CFDs, major stocks only)
- Fundamentals: Minimal
- Options: Not available
- Economic Data: Not provided
- News: Not provided
- Crypto: CFDs only (not derivatives analytics)

**Overall**: Best-in-class for forex historical data and backtesting.
