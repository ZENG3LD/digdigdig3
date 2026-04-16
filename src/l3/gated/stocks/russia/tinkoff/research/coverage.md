# Tinkoff Invest API - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America**: Yes (limited - US stocks via SPB Exchange)
- **Europe**: Yes (limited - European stocks via SPB Exchange)
- **Asia**: Yes (focus on Russia + some Chinese stocks)
- **Other**: Russia (PRIMARY focus - Moscow Exchange)

### Country-Specific

#### Russia (Primary Market)
- **Moscow Exchange (MOEX)**: Yes ⭐⭐⭐⭐⭐ (Complete coverage)
  - Stocks (shares): ~1,900 instruments
  - Bonds: ~655 instruments
  - ETFs: ~105 instruments
  - Futures: ~284 contracts
  - Options: Available
  - Currency pairs: ~21 pairs
- **RTS Exchange**: Yes (derivatives, indices)
- **SPB Exchange**: Yes (foreign stocks traded in Russia)

#### United States
- **US stocks**: Yes (via SPB Exchange - Saint Petersburg Exchange)
  - Access to: 10,000+ international securities
  - Major stocks: AAPL, MSFT, TSLA, GOOGL, etc.
  - Trading hours: Moscow timezone (pre-market/after-hours limited)

#### European Union
- **EU stocks**: Yes (via SPB Exchange)
  - Major European stocks accessible
  - Limited compared to native European brokers

#### China
- **Chinese stocks**: Yes (limited, via SPB Exchange and direct MOEX listings)
  - Hong Kong listings: Some access
  - A-shares: Very limited

#### Other Countries
- **UK**: Yes (via SPB Exchange - LSE stocks)
- **Japan**: Limited
- **India**: No
- **Brazil**: Limited
- **Other emerging markets**: Very limited

### Restricted Regions

- **Blocked countries**: Not explicitly documented (Russian broker, subject to Russian regulations)
- **VPN detection**: Not documented
- **Geo-fencing**: Not documented
- **Sanctions impact**: Subject to international sanctions affecting Russian financial institutions
  - US/EU investors may face restrictions
  - SWIFT disconnection impact (2022+)
  - Check current regulatory status before using

**Important**: Tinkoff Investments is a Russian broker. International access may be restricted due to:
1. Russian financial regulations
2. International sanctions on Russian financial sector
3. Payment processing limitations
4. Cross-border financial restrictions

## Markets/Exchanges Covered

### Stock Markets

#### Russian Exchanges (Primary)
- **Moscow Exchange (MOEX)**: Yes ⭐⭐⭐⭐⭐
  - Main market board (TQBR)
  - Innovation board
  - Classifieds board
  - Full API support
- **RTS Exchange**: Yes
  - Derivatives trading
  - Futures and options

#### SPB Exchange (International Access)
- **SPB Exchange**: Yes ⭐⭐⭐⭐
  - US stocks: NASDAQ, NYSE, AMEX
  - European stocks: limited selection
  - Trading in USD, EUR
  - Access to 10,000+ international securities

#### Other Stock Exchanges
- **NYSE (via SPB)**: Yes (indirect)
- **NASDAQ (via SPB)**: Yes (indirect)
- **LSE**: Limited (via SPB)
- **Euronext**: Limited (via SPB)
- **Tokyo Stock Exchange (TSE)**: No
- **Shanghai Stock Exchange (SSE)**: No
- **Shenzhen Stock Exchange (SZSE)**: No
- **Hong Kong Stock Exchange (HKEX)**: Very limited
- **BSE/NSE (India)**: No
- **ASX (Australia)**: No
- **TSX (Canada)**: Limited (via SPB)

### Crypto Exchanges (if aggregator)

**NOT APPLICABLE** - Tinkoff is traditional broker, not crypto exchange.

- Binance: No
- Coinbase: No
- Kraken: No

**Note**: Cryptocurrency trading is restricted in Russia. Tinkoff does NOT offer crypto trading.

### Forex Market

- **Currency pairs**: Yes (~21 pairs)
- **Majors**: Limited
  - USD/RUB: Yes ⭐⭐⭐⭐⭐
  - EUR/RUB: Yes ⭐⭐⭐⭐⭐
  - CNY/RUB: Yes
  - GBP/RUB: Yes
- **Minors**: Very limited (pairs involving RUB primarily)
- **Exotics**: Limited (focus on RUB crosses)

**Note**: Forex coverage is Russia-centric (RUB pairs), not full FX market like dedicated forex brokers.

### Futures/Options Exchanges

- **Moscow Exchange Derivatives Market**: Yes ⭐⭐⭐⭐⭐
  - Futures: ~284 contracts
  - Options: Available
  - Commodity futures: Yes
  - Index futures: Yes (RTS Index, etc.)
  - Currency futures: Yes
- **CME**: No (not directly accessible)
- **CBOE**: No
- **Eurex**: No
- **ICE**: No

**Note**: Russian derivatives only. No access to US/EU derivatives exchanges.

## Instrument Coverage

### Stocks

- **Total symbols**: ~12,000+ (including international via SPB)
- **Russian stocks**: ~1,900 (MOEX as of 2022)
- **International stocks**: ~10,000+ (via SPB Exchange)
- **US stocks**: Extensive (major NASDAQ/NYSE stocks)
- **European stocks**: Limited selection
- **Asian stocks**: Very limited
- **OTC**: No (Over-the-counter not supported)
- **Penny stocks**: Limited (Russian small caps available)

**Russian stock examples**:
- Sberbank (SBER)
- Gazprom (GAZP)
- Lukoil (LKOH)
- Yandex (YNDX)
- Rosneft (ROSN)
- Norilsk Nickel (GMKN)
- Magnit (MGNT)
- Tinkoff (TCS)

**US stock examples (via SPB)**:
- Apple (AAPL)
- Microsoft (MSFT)
- Tesla (TSLA)
- Amazon (AMZN)
- Google (GOOGL)
- Meta (META)
- NVIDIA (NVDA)

### Bonds

- **Total bonds**: ~655 (Russian bonds)
- **Government bonds (OFZ)**: Yes (Russian federal bonds)
- **Corporate bonds**: Yes (Russian companies)
- **Municipal bonds**: Limited
- **Eurobonds**: Limited
- **US Treasury bonds**: No (not directly)
- **Corporate bonds (US/EU)**: No

**Bond types**:
- Fixed coupon bonds
- Floating rate bonds
- Zero-coupon bonds
- Amortization bonds
- Perpetual bonds (rare)

### ETFs

- **Total ETFs**: ~105
- **Russian ETFs**: Yes (focus on Russian indices)
- **US ETFs**: No (not US-listed ETFs)
- **International ETFs**: Limited (some MOEX-listed international ETFs)

**ETF examples**:
- TMOS (Moscow Exchange index)
- VTBX (VTB index)
- SBMX (Sberbank index)
- Gold ETFs
- Bond ETFs

### Commodities

- **Direct commodities trading**: No (not spot commodities)
- **Commodity futures**: Yes
  - Gold: Yes
  - Silver: Yes
  - Oil (Brent): Yes
  - Natural Gas: Yes
  - Agricultural: Limited
- **Precious metals**: Yes (via futures)
- **Energy**: Yes (via futures)
- **Agriculture**: Limited (via futures)

### Futures

- **Total futures contracts**: ~284
- **Index futures**: Yes
  - RTS Index
  - MOEX Index
  - Blue Chip Index
- **Currency futures**: Yes
  - USD/RUB
  - EUR/RUB
  - CNY/RUB
- **Commodity futures**: Yes
  - Gold, Silver
  - Brent Crude
  - Natural Gas
- **Stock futures**: Yes (on major Russian stocks)
- **Interest rate futures**: Yes (Russian government bonds)

### Options

- **Options available**: Yes
- **Stock options**: Yes (on liquid Russian stocks)
- **Index options**: Yes
- **Currency options**: Yes
- **Total option contracts**: Not specified (varies by underlying)

**Note**: Options market less developed than US, but growing.

### Indices

- **Russian Indices**: Yes
  - MOEX Russia Index
  - RTS Index
  - Blue Chip Index
  - Sector indices
- **US Indices**: No (cannot trade SPX, NDX directly)
  - Access via US ETFs on SPB
- **International Indices**: No (direct access)

### Currencies (Forex)

- **Currency pairs**: ~21 pairs
- **Base currencies**: RUB (primary), USD, EUR, CNY
- **Pairs available**:
  - USD/RUB ⭐
  - EUR/RUB ⭐
  - EUR/USD
  - GBP/RUB
  - CNY/RUB
  - CHF/RUB
  - JPY/RUB
  - Others (RUB-based)

**Not available**: Most exotic pairs, full forex market depth

## Data History

### Historical Depth

- **Stocks**: From 1998 for some instruments (earliest: 1970-01-01 theoretical limit)
- **Bonds**: Varies by bond (issue date onwards)
- **ETFs**: Since ETF inception
- **Futures**: Limited history (contract-specific)
- **Options**: Since contract listing
- **Forex**: Currency pair dependent

**Per instrument**:
- Daily candles: Up to 6 years (2400 candles max per request)
- Weekly candles: Up to 5 years (300 candles max)
- Monthly candles: Up to 10 years (120 candles max)
- Intraday: Limited by candle interval (see table below)

### Granularity Available

| Interval | Available | Historical Depth | Max Candles | Notes |
|----------|-----------|------------------|-------------|-------|
| **5 seconds** | Yes | 200 minutes | 2500 | Very short-term |
| **10 seconds** | Yes | 200 minutes | 1250 | Very short-term |
| **30 seconds** | Yes | 20 hours | Variable | Short-term |
| **1 minute** | Yes | 1 day | 2400 | Intraday |
| **2 minutes** | Yes | 1 day | 1200 | Intraday |
| **3 minutes** | Yes | 1 day | 750 | Intraday |
| **5 minutes** | Yes | 1 week | 2400 | Intraday |
| **10 minutes** | Yes | 1 week | Variable | Intraday |
| **15 minutes** | Yes | 3 weeks | 2400 | Intraday |
| **30 minutes** | Yes | 3 weeks | 1200 | Intraday |
| **1 hour** | Yes | 3 months | 2400 | Hourly |
| **2 hours** | Yes | 3 months | 2400 | Hourly |
| **4 hours** | Yes | 3 months | 700 | Hourly |
| **1 day** | Yes | 6 years | 2400 | Daily |
| **1 week** | Yes | 5 years | 300 | Weekly |
| **1 month** | Yes | 10 years | 120 | Monthly |

**Tick data**: Not available via API (only aggregated candles and trades)

### Real-time vs Delayed

- **Real-time**: Yes ⭐⭐⭐⭐⭐ (all market data is real-time)
- **Delayed**: No (no delayed feeds offered)
- **Snapshot**: Yes (current state via GetOrderBook, GetLastPrices, etc.)

**Important**: ALL data is real-time, no 15-minute or end-of-day delays.

## Update Frequency

### Real-time Streams

- **Price updates**: Real-time (milliseconds)
- **Order book**: Snapshot-based (not tick-by-tick delta)
  - Full snapshot on each update
  - Update frequency: Variable (depends on market activity)
- **Trades**: Real-time (every trade)
- **Candles**: On candle close (e.g., every 1 minute for 1m candles)
- **Trading status**: On status change (opening, closing, break, etc.)

### Scheduled Updates

- **Dividends**: Announced by companies (days/weeks before ex-date)
- **Coupon payments**: Bond schedule (known in advance)
- **Financial reports**: Not provided via API (use external sources)
- **Trading schedules**: Can be requested for date ranges (static data)

### Market Hours

**MOEX Trading Hours** (Moscow Time, UTC+3):
- **Main trading session**: 10:00 - 18:40 (primary session)
- **Pre-market**: 07:00 - 10:00 (limited liquidity)
- **After-hours**: 18:40 - 23:50 (limited liquidity)
- **Clearing session**: 19:00 - 19:05

**SPB Exchange** (for international stocks):
- Follows Moscow timezone
- Limited to Russian trading hours
- No access to US pre-market/after-hours in US time

**Forex/Derivatives**:
- Extended hours (some 24-hour trading)

**API Access**: 24/7 (can fetch historical data anytime)

## Data Quality

### Accuracy

- **Source**: Direct from exchange (MOEX, SPB Exchange)
  - Primary data feed
  - Official exchange data
- **Validation**: Yes (exchange-validated)
- **Corrections**: Automatic (exchange adjustments)
- **Corporate actions**: Adjusted (splits, dividends)

**Data integrity**: High (direct broker access, not third-party aggregator)

### Completeness

- **Missing data**: Rare (occasional gaps during exchange issues)
- **Gaps**: Minimal (exchange outages only)
- **Backfill**: Not explicitly documented (likely limited)
- **Suspended instruments**: Data available until suspension, then stops

**Data availability**:
- ✅ All traded instruments (if on MOEX/SPB)
- ✅ All trading sessions (regular, pre-market, after-hours)
- ✅ Corporate actions reflected
- ❌ No data for instruments not available on MOEX/SPB

### Timeliness

- **Real-time latency**: <100ms typical (direct exchange connection)
- **Delayed data**: N/A (no delayed feeds)
- **Market hours coverage**: Full coverage during exchange hours
- **Off-hours access**: Historical data available 24/7

**Latency comparison**:
- Direct exchange feed: <10ms
- Tinkoff API: <100ms (estimated, includes network + processing)
- Third-party data providers: 100-500ms

## Coverage Comparison

### Tinkoff vs Other Russian Brokers

| Feature | Tinkoff | BCS | Finam | Interactive Brokers (Russia) |
|---------|---------|-----|-------|------------------------------|
| **Russian stocks** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| **International stocks** | ⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **API quality** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Free API** | Yes | Limited | No | Yes |
| **Real-time data** | Yes | Yes | Yes | Yes |

### Tinkoff vs International Brokers

| Feature | Tinkoff | Interactive Brokers | TD Ameritrade | Alpaca |
|---------|---------|---------------------|---------------|--------|
| **US stocks** | ⭐⭐⭐ (via SPB) | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ |
| **Russian stocks** | ⭐⭐⭐⭐⭐ | ⭐⭐ | ⭐ | ⭐ (none) |
| **API** | Free, gRPC | Free, various | Free (deprecated) | Free, REST |
| **Sanctions impact** | ⚠️ High | Low | Low | Low |

## Unique Coverage Characteristics

### Strengths

1. **Russian Market Leader**
   - Best coverage of MOEX instruments
   - Deep liquidity access
   - Local market expertise

2. **Integrated Ecosystem**
   - Real trading account (not just data)
   - Actual execution, not paper trading
   - Portfolio tracking included

3. **Modern API**
   - gRPC protocol (efficient)
   - Free for all clients
   - Real-time data included

4. **Derivative Markets**
   - Full futures/options coverage
   - Russian commodity access
   - Currency derivatives

5. **Bond Market**
   - Comprehensive Russian bond coverage
   - OFZ (government bonds) access
   - Corporate bonds

### Weaknesses

1. **Geographic Limitations**
   - Russian/SPB exchanges only
   - No direct US exchange access
   - Limited EU coverage
   - No Asian exchanges (except limited Chinese)

2. **Sanctions Exposure**
   - Subject to international sanctions
   - Payment processing restrictions
   - Cross-border limitations
   - Regulatory uncertainty

3. **Cryptocurrency**
   - No crypto trading (Russian regulations)
   - No crypto data

4. **Fundamental Data**
   - Limited company financials
   - No analyst ratings
   - No news feed
   - Basic dividend data only

5. **Market Hours**
   - Moscow timezone (inconvenient for US traders)
   - No true US market hours access
   - SPB hours limited

## Instrument Search & Discovery

- **Search API**: Yes (FindInstrument method)
- **Search by**:
  - Ticker
  - Company name
  - FIGI
  - ISIN
  - Instrument type filter
- **Favorites**: Yes (GetFavorites, EditFavorites)
- **Categories**: Sector classification available
- **Filtering**: By instrument type, status, exchange

## Special Instruments

### Structured Products
- **Structured notes**: Yes (INSTRUMENT_TYPE_SP)
- **Coverage**: Limited (Russian market products)

### Clearing Certificates
- **Available**: Yes (INSTRUMENT_TYPE_CLEARING_CERTIFICATE)
- **Use case**: Specialized instruments

### Investment Coins
- **Gold/silver coins**: Via commodity futures (not physical)

## Data Coverage Summary

| Category | Coverage Level | Quality | Notes |
|----------|---------------|---------|-------|
| **Russian stocks** | ⭐⭐⭐⭐⭐ 100% | Excellent | Complete MOEX coverage |
| **US stocks** | ⭐⭐⭐⭐ 70% | Good | Via SPB, major stocks only |
| **EU stocks** | ⭐⭐⭐ 40% | Adequate | Limited selection via SPB |
| **Asian stocks** | ⭐ 10% | Poor | Very limited |
| **Russian bonds** | ⭐⭐⭐⭐⭐ 100% | Excellent | Government + corporate |
| **International bonds** | ⭐ 5% | Poor | Minimal |
| **ETFs** | ⭐⭐⭐⭐ 80% | Good | Russian ETFs, some international |
| **Futures** | ⭐⭐⭐⭐⭐ 100% | Excellent | Russian derivatives |
| **Options** | ⭐⭐⭐⭐ 80% | Good | Growing market |
| **Forex** | ⭐⭐⭐ 50% | Adequate | RUB pairs focus |
| **Commodities** | ⭐⭐⭐⭐ 70% | Good | Via futures |
| **Crypto** | ⭐ 0% | N/A | Not available |

## Regulatory & Compliance

### Russian Regulations
- Subject to Central Bank of Russia oversight
- Qualified investor requirements for some instruments
- Trading restrictions on certain instruments via API

### International Considerations
- **US investors**: Check OFAC sanctions
- **EU investors**: Check EU sanctions compliance
- **Other countries**: Verify legal status

### Data Usage Rights
- **Personal use**: Allowed
- **Commercial use**: Requires registration (contact al.a.volkov@tinkoff.ru)
- **Redistribution**: Not explicitly allowed (check Terms of Service)

## Summary

**Best for**:
- ✅ Russian stock/bond trading
- ✅ MOEX derivatives
- ✅ Russian market data
- ✅ Integrated trading + data API
- ✅ Real-time Russian market access

**Not suitable for**:
- ❌ Primary US stock trading (use US broker)
- ❌ Cryptocurrency trading
- ❌ Full global market coverage
- ❌ Users in sanctioned jurisdictions
- ❌ Extensive fundamental data needs

**Unique value proposition**: Best-in-class Russian market access with modern gRPC API and free real-time data for all clients.
