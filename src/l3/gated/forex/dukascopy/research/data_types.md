# Dukascopy - Data Types Catalog

## Overview

Dukascopy specializes in **forex and CFD market data** with exceptional historical tick-level granularity. As a Swiss bank and data provider, they focus on traditional forex rather than crypto derivatives or options.

**Specialty**: Historical tick data dating back to 2003+ for major forex pairs.

---

## Standard Market Data

### Price Data
- [x] **Current Price** (real-time bid/ask)
  - Access: JForex SDK (IHistory.getLastTick())
  - Precision: 5 decimals (most pairs), 3 decimals (JPY pairs)
  - Update frequency: Sub-second (tick-by-tick)

- [x] **Bid/Ask Spread**
  - Access: ITick.getBid(), ITick.getAsk()
  - Spread calculation: ask - bid
  - Typical spreads: 0.1-2.0 pips (varies by pair and liquidity)

- [x] **24h Ticker Stats** (not directly available)
  - High: Calculate from historical bars
  - Low: Calculate from historical bars
  - Volume: Available per bar (IBar.getVolume())
  - Change%: Calculate from historical data

### OHLC/Candlesticks
- [x] **Bars/Candles** (comprehensive period support)
  - Access: IHistory.getBars()
  - Intervals: TICK, 10s, 30s, 1m, 5m, 10m, 15m, 30m, 1h, 4h, 1d, 1w, 1mo
  - Fields: open, high, low, close, volume
  - Format: IBar interface
  - OfferSide: BID, ASK (separate data streams)

### Order Book Data
- [x] **Level 2 Orderbook** (10 levels)
  - Access: ITick.getAsks(), ITick.getBids()
  - Depth: Top 10 price levels
  - Fields: price, volume per level
  - Update: Real-time with each tick
  - Format: double[] arrays

- [ ] **Full Market Depth** (not available)
  - Dukascopy provides top 10 levels only

### Trade Data
- [x] **Recent Trades** (tick data)
  - Access: IHistory.getTicks()
  - Fields: time, bid, ask, bidVolume, askVolume
  - Granularity: Every price change
  - Historical: Full tick history available

### Volume
- [x] **Volume** (available per bar)
  - Access: IBar.getVolume()
  - Type: Tick volume (number of price changes, not notional)
  - Granularity: Per bar/candle
  - Real-time: Yes

---

## Historical Data

### Historical Price Data
- [x] **Historical prices** (exceptional depth)
  - Depth: 2003+ for major forex pairs (20+ years)
  - Varies by instrument:
    - Major forex: 2003+
    - Crypto: 2017+
    - Stocks: Varies (recent years)
  - Access: IHistory interface, binary downloads

### Granularity
- [x] **Tick data** (primary strength)
  - Available: Yes (full tick-by-tick history)
  - Format: .bi5 binary files (hourly), ITick objects
  - Fields: timestamp, bid, ask, bidVolume, askVolume
  - Storage: LZMA-compressed binary

- [x] **Minute bars**
  - Available: Yes
  - Timeframes: 1m, 5m, 10m, 15m, 30m
  - Historical depth: Same as tick data
  - Calculated from ticks

- [x] **Hourly/Daily bars**
  - Available: Yes
  - Timeframes: 1h, 4h, 1d, 1w, 1mo
  - Historical depth: Full history
  - Precision: High (from tick data)

### Adjusted Prices
- [x] **Adjusted prices** (for stocks/ETFs)
  - Available: Yes (for applicable instruments)
  - Adjustments: Splits, dividends
  - Forex: N/A (no adjustments needed)

---

## Derivatives Data (Crypto/Futures)

**Not Applicable** - Dukascopy focuses on spot forex and CFDs, not crypto derivatives.

- [ ] **Open Interest** - Not available
- [ ] **Funding Rates** - Not available (no perpetual futures)
- [ ] **Liquidations** - Not available
- [ ] **Long/Short Ratios** - Not available
- [ ] **Mark Price** - Not available
- [ ] **Index Price** - Not available
- [ ] **Basis** - Not available

**Alternative**: Dukascopy offers spot crypto pairs (BTC/USD, ETH/USD, etc.) but not derivatives analytics.

---

## Options Data

**Not Available** - Dukascopy does not provide options data.

- [ ] **Options Chains** - Not available
- [ ] **Implied Volatility** - Not available
- [ ] **Greeks** - Not available
- [ ] **Open Interest** - Not available
- [ ] **Historical option prices** - Not available

---

## Fundamental Data (Stocks)

**Limited** - Dukascopy is primarily a forex/CFD provider, not a fundamental data provider.

Available for stock CFDs:
- [x] **Company Profile** (basic info only)
  - Name, ticker, sector (limited)
  - Access: Instrument metadata

Not available:
- [ ] **Financial Statements** - Not provided
- [ ] **Earnings** - Not provided
- [ ] **Dividends** - Not provided (CFDs don't have dividends)
- [ ] **Stock Splits** - Not provided
- [ ] **Analyst Ratings** - Not provided
- [ ] **Insider Trading** - Not provided
- [ ] **Institutional Holdings** - Not provided
- [ ] **Financial Ratios** - Not provided
- [ ] **Valuation Metrics** - Not provided

**Recommendation**: Use dedicated fundamental data providers (Polygon, Alpha Vantage, etc.) for stocks.

---

## On-chain Data (Crypto)

**Not Available** - Dukascopy provides crypto price data only, not on-chain analytics.

- [ ] **Wallet Balances** - Not available
- [ ] **Transaction History** - Not available
- [ ] **DEX Trades** - Not available
- [ ] **Token Transfers** - Not available
- [ ] **Smart Contract Events** - Not available
- [ ] **Gas Prices** - Not available
- [ ] **Block Data** - Not available
- [ ] **NFT Data** - Not available

**Alternative**: Dukascopy offers spot crypto pairs (BTC, ETH, LTC, etc.) for price data only.

---

## Macro/Economic Data (Economics)

**Not Available** - Dukascopy does not provide economic indicators.

- [ ] **Interest Rates** - Not provided
- [ ] **GDP** - Not provided
- [ ] **Inflation** - Not provided
- [ ] **Employment** - Not provided
- [ ] **Retail Sales** - Not provided
- [ ] **Industrial Production** - Not provided
- [ ] **Consumer Confidence** - Not provided
- [ ] **PMI** - Not provided
- [ ] **Economic Calendar** - Not provided (some economic events may affect forex prices)

**Recommendation**: Use FRED, Trading Economics, or news APIs for economic data.

---

## Forex Specific

**Primary Strength** - Dukascopy excels in forex data.

### Currency Pairs
- [x] **Currency Pairs** (comprehensive coverage)
  - Total: 100+ forex pairs
  - Majors: 7 pairs (EUR/USD, GBP/USD, USD/JPY, USD/CHF, USD/CAD, AUD/USD, NZD/USD)
  - Minors: 30+ pairs (EUR/GBP, EUR/JPY, GBP/JPY, etc.)
  - Exotics: 60+ pairs (USD/TRY, EUR/HUF, etc.)
  - Metals: 50+ combinations (XAU/USD, XAG/EUR, etc.)

### Precision & Spreads
- [x] **Bid/Ask Spreads**
  - Real-time spreads: Yes
  - Historical spreads: Yes (from tick data)
  - Typical: 0.1-2.0 pips (majors)

- [x] **Pip precision**
  - Most pairs: 5 decimals (0.00001)
  - JPY pairs: 3 decimals (0.001)
  - Pipette: 1/10th of pip (5th decimal)

- [x] **Cross rates**
  - Available: Yes
  - Calculation: Via JFUtils.convert()
  - Real-time: Yes

### Historical FX Rates
- [x] **Historical rates** (exceptional depth)
  - Tick-level: 2003+ (major pairs)
  - Minute/hourly/daily: Full history
  - Format: Binary (.bi5) or SDK access

### Conversion Utilities
- [x] **Currency conversion** (JFUtils interface)
  - Methods: convert(), getRate(), convertPipToCurrency()
  - Real-time: Yes
  - Historical: Yes (via historical bars)

---

## Metadata & Reference

### Instrument Information
- [x] **Symbol/Instrument Lists**
  - Access: SDK (Instrument enum)
  - Total: 1,200+ instruments
  - Categories: Forex, crypto, stocks, indices, commodities, bonds

- [x] **Exchange Information**
  - Broker: Dukascopy Bank SA
  - Liquidity: Aggregated from multiple sources
  - Execution: ECN model

### Market Hours
- [x] **Market Hours** (forex-specific)
  - Forex: 24/5 (Sunday 22:00 GMT - Friday 22:00 GMT)
  - Pre-market: N/A (forex is continuous)
  - After-hours: N/A
  - Trading halts: Weekends only

- [x] **Trading Calendars**
  - Holidays: Limited info
  - Half-days: N/A (forex trades 24/5)
  - Maintenance: Occasional weekend maintenance

### Timezone & Metadata
- [x] **Timezone Info**
  - Standard: UTC+0 / GMT
  - Consistency: All timestamps in UTC
  - Daylight saving: Not affected

- [x] **Sector/Industry Classifications** (limited)
  - Available for stocks/ETFs
  - Basic categorization only

---

## News & Sentiment

**Not Available** - Dukascopy does not provide news or sentiment data.

- [ ] **News Articles** - Not provided
- [ ] **Press Releases** - Not provided
- [ ] **Social Sentiment** - Not provided
- [ ] **Analyst Reports** - Not provided

**Alternative**: JForex platform includes economic calendar and news widgets, but not accessible via API.

---

## Unique/Custom Data

**What makes Dukascopy special:**

### 1. Historical Tick Data (Primary Strength)
- **Free access** to tick-level data back to 2003+
- **Compressed binary format** (.bi5) for efficient storage
- **Hourly granularity** (one file per hour)
- **High precision**: 5 decimals for most pairs
- **No authentication** required for downloads
- **20+ years** of forex history

**Use cases**:
- High-frequency backtesting
- Microstructure analysis
- Spread analysis
- Volume profile studies

### 2. Custom Bar Types (via JForex SDK)
- **Renko bars**: Price-based bars (fixed price movement)
- **Kagi bars**: Trend-following bars
- **Line break bars**: Reversal-based bars
- **Point and figure bars**: Price action charts
- **Range bars**: Fixed price range bars
- **Custom feed descriptors**: Build your own bar types

**Access**:
```java
// Renko bars
IFeedDescriptor renkoDescriptor = new RenkoFeedDescriptor(
    Instrument.EURUSD,
    PriceRange.TWO_PIPS,
    OfferSide.BID
);

// Get renko bars
List<IBar> renkoBars = history.getFeedData(renkoDescriptor, from, to);
```

### 3. Multi-Level Order Book (10 levels)
- **Top 10 price levels** for bid and ask
- **Real-time updates** with each tick
- **Historical availability** via tick data
- **Volume per level**

**Unique**: Most free data providers offer only top-of-book (1 level).

### 4. Dual-Side Data (BID vs ASK)
- **Separate data streams** for BID and ASK
- **Different OHLC values** depending on side
- **Spread analysis**: Compare bid/ask bars
- **Execution modeling**: Choose side based on order type

**Example**:
```java
// Bid-side candles (for sell orders)
List<IBar> bidBars = history.getBars(Instrument.EURUSD, Period.ONE_MIN, OfferSide.BID, from, to);

// Ask-side candles (for buy orders)
List<IBar> askBars = history.getBars(Instrument.EURUSD, Period.ONE_MIN, OfferSide.ASK, from, to);
```

### 5. Tick Volume
- **Number of price changes** per bar
- **Not notional volume** (forex has no centralized volume)
- **Useful for liquidity analysis**

### 6. Swiss Banking Data Quality
- **Regulated by FINMA** (Swiss financial regulator)
- **Audited data**: Swiss banking standards
- **Reliability**: 20+ years of consistent data
- **No gaps**: Comprehensive tick coverage

### 7. Conversion Utilities
- **Cross-currency conversion**: Convert amounts between instruments
- **Pip value calculation**: Calculate pip value in any currency
- **Real-time rates**: Get exchange rates programmatically

**Use cases**:
- Multi-currency portfolio management
- Risk calculation in account currency
- Position sizing across different pairs

---

## Data Format Summary

| Data Type | Format | Access Method | Historical Depth | Real-time |
|-----------|--------|---------------|------------------|-----------|
| Tick data | ITick object / .bi5 binary | SDK / Binary download | 2003+ | Yes |
| OHLC bars | IBar object | SDK | 2003+ | Yes |
| Order book | double[] arrays | SDK | 2003+ (via ticks) | Yes |
| Custom bars | IBar object | SDK | Historical | Yes |
| Instrument metadata | Instrument enum | SDK | N/A | N/A |

---

## Instrument Coverage by Category

### Forex (Primary)
- **Total**: 100+ pairs
- **Coverage**: Excellent (major, minor, exotic)
- **Historical depth**: 2003+ (majors)
- **Quality**: Very high

### Cryptocurrencies
- **Total**: 33 instruments
- **Coverage**: Major coins (BTC, ETH, LTC)
- **Pairs**: vs USD, EUR, GBP, CHF, JPY
- **Historical depth**: 2017+
- **Quality**: High

### Commodities
- **Total**: 13 instruments
- **Agricultural**: 6 (Corn, Wheat, Soybeans, etc.)
- **Energy**: 4 (Crude Oil, Natural Gas, etc.)
- **Metals**: 3 (Gold, Silver, Copper)
- **Quality**: High

### Stock Indices
- **Total**: 22 indices
- **Americas**: 6 (S&P500, Nasdaq, Dow, etc.)
- **Asia**: 6 (Nikkei, Hang Seng, etc.)
- **Europe**: 9 (DAX, FTSE, CAC, etc.)
- **Africa**: 1
- **Quality**: High

### Stocks (CFDs)
- **Total**: 600+ stocks
- **US**: 608 stocks
- **UK**: 102 stocks
- **Japan**: 55 stocks
- **Other markets**: Limited
- **Quality**: Good (major stocks only)

### ETFs
- **Total**: 70+ ETFs
- **US**: 62 ETFs
- **France**: 3 ETFs
- **Hong Kong**: 4 ETFs
- **Germany**: 1 ETF
- **Quality**: Good

### Bonds
- **Total**: 3 instruments
- **Types**: Euro Bund, UK Gilts, US T-Bond
- **Quality**: Good

---

## Missing Data Types

**Not Available from Dukascopy**:
- Options data (chains, Greeks, IV)
- Crypto derivatives (funding, liquidations, OI)
- Economic indicators (GDP, CPI, NFP)
- Fundamental data (earnings, financials, ratios)
- News & sentiment
- On-chain data (for crypto)
- Social data
- Alternative data

**Recommendation**: Combine Dukascopy (for forex/tick data) with:
- Polygon (for stocks fundamentals)
- CoinGlass (for crypto derivatives)
- FRED (for economic data)
- News APIs (for sentiment)

---

## Summary: Dukascopy's Strengths

1. **Historical Tick Data**: Unmatched free access (2003+)
2. **Forex Coverage**: Comprehensive (100+ pairs)
3. **Data Quality**: Swiss banking standards
4. **Granularity**: Tick to monthly
5. **Custom Bar Types**: Renko, Kagi, P&F, etc.
6. **Multi-Level Order Book**: 10 levels (rare for free)
7. **Dual-Side Data**: BID/ASK separation
8. **Free Access**: No API fees, generous limits

**Best Use Cases**:
- Forex backtesting (tick-level)
- Spread analysis
- Liquidity studies
- Microstructure research
- Educational projects
- Forex algorithmic trading
