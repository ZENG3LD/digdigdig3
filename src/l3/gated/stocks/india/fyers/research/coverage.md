# Fyers - Data Coverage

## Geographic Coverage

### Regions Supported
- **North America:** No
- **Europe:** No
- **Asia:** Yes (India only)
- **Other:** No

### Country-Specific
- **India:** Yes (Primary and only market)
- **US:** No
- **UK:** No
- **Japan:** No
- **China:** No
- **Other:** No

### Restricted Regions
- **Blocked countries:** Not officially documented (requires Indian trading account)
- **VPN detection:** Not specified
- **Geo-fencing:** Not specified
- **Account requirements:** Indian PAN card, Indian bank account, KYC compliance

**Note:** Fyers is an India-focused broker. API access requires active Fyers trading account with Indian regulatory compliance.

---

## Markets/Exchanges Covered

### Indian Stock Exchanges

| Exchange | Full Name | Segments Covered | Status |
|----------|-----------|------------------|--------|
| **NSE** | National Stock Exchange of India | CM, FO, CD | ✅ Full |
| **BSE** | Bombay Stock Exchange | CM, FO | ✅ Full |
| **MCX** | Multi Commodity Exchange | COMM | ✅ Full |
| **NCDEX** | National Commodity & Derivatives Exchange | COMM | ✅ Full |

### Segment Details

**NSE (National Stock Exchange):**
- **CM (Capital Market):** Equity, ETFs
- **FO (Futures & Options):** Index F&O, Stock F&O
- **CD (Currency Derivatives):** Currency Futures & Options (USDINR, EURINR, GBPINR, JPYINR)

**BSE (Bombay Stock Exchange):**
- **CM (Capital Market):** Equity, ETFs
- **FO (Futures & Options):** Index F&O, Stock F&O (limited compared to NSE)

**MCX (Multi Commodity Exchange):**
- **COMM (Commodities):** Metals (Gold, Silver, Copper), Energy (Crude Oil, Natural Gas), Agriculture

**NCDEX (National Commodity & Derivatives Exchange):**
- **COMM (Commodities):** Agricultural commodities (Wheat, Soybean, Cotton, etc.)

### Stock Markets (International)
- **US:** NYSE, NASDAQ - ❌ No
- **UK:** LSE - ❌ No
- **Japan:** TSE - ❌ No
- **China:** SSE, SZSE - ❌ No
- **Europe:** Euronext, DAX - ❌ No
- **Australia:** ASX - ❌ No

### Crypto Exchanges
**Not Applicable** - Fyers does not support cryptocurrency trading.

- Binance - ❌ No
- Coinbase - ❌ No
- Kraken - ❌ No

### Forex Brokers
**Not Applicable** - Only currency derivatives (futures/options) via NSE CD segment, not spot forex.

### Futures/Options Exchanges
- **NSE F&O:** ✅ Yes (Index & Stock F&O)
- **BSE F&O:** ✅ Yes (Limited compared to NSE)
- **CME:** ❌ No
- **CBOE:** ❌ No
- **Eurex:** ❌ No

---

## Instrument Coverage

### Equities (Stocks)

**Total Coverage:**
- **Total symbols:** ~7,000+ (NSE + BSE combined)
- **NSE stocks:** ~2,000+ actively traded
- **BSE stocks:** ~5,000+ listed
- **Actively traded:** ~1,500-2,000 (high liquidity)

**Categories:**
- **Large Cap:** ✅ All Nifty 50, Sensex 30 companies
- **Mid Cap:** ✅ Nifty Midcap 100, BSE Midcap
- **Small Cap:** ✅ Nifty Smallcap, BSE Smallcap
- **OTC (Over-the-Counter):** ❌ Not supported
- **Penny stocks:** ✅ Yes (BE, T, Z series - high risk)

**Series Types (NSE):**
- **EQ:** Main equity segment (most liquid)
- **BE:** Book Entry segment (T+2 settlement)
- **SM:** SME (Small and Medium Enterprises)
- **T, Z:** Trade-to-Trade, additional surveillance (restricted)

**ETFs & Mutual Funds:**
- **ETFs:** ✅ Yes (Nifty ETFs, Gold ETFs, etc.)
- **Mutual Funds:** ✅ Holdings visible (not tradable via API)

### Futures & Options (Derivatives)

**Index Derivatives:**
- **Nifty 50 F&O:** ✅ Yes
- **Bank Nifty F&O:** ✅ Yes
- **Fin Nifty F&O:** ✅ Yes
- **Sensex F&O:** ✅ Yes (BSE)
- **Other Indices:** ✅ Nifty IT, Pharma, Midcap, etc.

**Stock F&O:**
- **Coverage:** ~180+ stocks with F&O (NSE)
- **Lot Sizes:** Vary by stock (e.g., Reliance: 250, SBIN: 1,500)
- **Expiry:** Weekly (Nifty, Bank Nifty), Monthly (stocks)

**Futures:**
- **Total Futures:** 200+ (index + stock futures)
- **Expiry Months:** Current, Near, Far (up to 3 months)
- **Contract Sizes:** Standardized by exchange

**Options:**
- **Index Options:** ✅ Nifty, Bank Nifty, Fin Nifty, Sensex
- **Stock Options:** ✅ 180+ stocks
- **Strike Prices:** Multiple strikes per expiry
- **Expiry:** Weekly (index), Monthly (stock)
- **Types:** CE (Call), PE (Put)

**Total Derivative Instruments:**
- **Futures:** ~200+
- **Options:** ~10,000+ (all strikes, all expiries combined)

### Crypto
**Not Applicable** - Cryptocurrency not supported.

- Total coins: ❌ None
- Spot pairs: ❌ None
- Futures: ❌ None
- Perpetuals: ❌ None

### Forex (Currency Derivatives)

**Currency Pairs (NSE CD Segment):**
- **Majors vs INR:** 4 pairs
  - USDINR (US Dollar - Indian Rupee)
  - EURINR (Euro - Indian Rupee)
  - GBPINR (British Pound - Indian Rupee)
  - JPYINR (Japanese Yen - Indian Rupee)
- **Cross Currency Pairs:** ❌ Not supported (only INR pairs)
- **Spot Forex:** ❌ Not supported (only derivatives)

**Currency Derivatives Available:**
- **Currency Futures:** ✅ All 4 pairs
- **Currency Options:** ✅ All 4 pairs (CE & PE)

**Total Currency Instruments:**
- **Futures:** ~12 (4 pairs × 3 months)
- **Options:** ~500+ (all strikes, all expiries)

### Commodities

**MCX (Multi Commodity Exchange):**
- **Metals:**
  - Gold (GOLD, GOLDM, GOLDPETAL)
  - Silver (SILVER, SILVERM, SILVERMIC)
  - Copper (COPPER, COPPERMIC)
  - Aluminum, Zinc, Lead, Nickel
- **Energy:**
  - Crude Oil (CRUDEOIL, CRUDEOILM)
  - Natural Gas (NATURALGAS, NATURALGAS_M)
- **Agriculture (Limited on MCX):**
  - Cotton, Cardamom, Mentha Oil

**NCDEX (Agricultural Commodities):**
- **Grains:** Wheat, Rice, Barley
- **Oilseeds:** Soybean, Mustard Seed, Castor Seed
- **Pulses:** Chana (Chickpea), Tur (Pigeon Pea)
- **Spices:** Turmeric, Coriander, Jeera (Cumin)
- **Others:** Cotton, Guar Seed, Kapas

**Total Commodity Coverage:**
- **MCX:** 20+ commodities
- **NCDEX:** 25+ agricultural commodities
- **Total Instruments:** ~200+ (various contract months)

### Indices

**NSE Indices:**
- **Nifty 50:** ✅ Yes
- **Nifty Bank:** ✅ Yes
- **Nifty Fin:** ✅ Yes
- **Nifty IT:** ✅ Yes
- **Nifty Pharma:** ✅ Yes
- **Nifty Auto:** ✅ Yes
- **Nifty FMCG:** ✅ Yes
- **Nifty Midcap 100/150/Select:** ✅ Yes
- **Nifty Smallcap 50/100/250:** ✅ Yes

**BSE Indices:**
- **Sensex (BSE 30):** ✅ Yes
- **BSE 100, 200, 500:** ✅ Yes
- **BSE Midcap, Smallcap:** ✅ Yes

**Crypto Indices:**
- ❌ Not applicable

**Total Indices:** 50+ (spot indices, derivatives available for major indices)

---

## Data History

### Historical Depth

**Equities:**
- **From year:** Varies by symbol (older stocks: 20+ years, newer: from listing)
- **Typical depth:** 10+ years for major stocks
- **Recent IPOs:** From listing date

**Derivatives:**
- **From year:** Limited by contract duration
- **Futures:** Max ~3 months back (per contract)
- **Options:** Limited historical data (often daily only for older dates)

**Commodities:**
- **From year:** Varies (10+ years for major commodities)
- **Typical depth:** 5-10 years

**Currency Derivatives:**
- **From year:** ~10 years (since NSE CD segment inception)

### Granularity Available

| Timeframe | Equity | Derivatives | Commodities | Available? |
|-----------|--------|-------------|-------------|------------|
| Tick data | ❌ (WebSocket only) | ❌ (WebSocket only) | ❌ (WebSocket only) | WebSocket TBT |
| 1-minute | ✅ | ⚠️ Limited | ✅ | REST API |
| 5-minute | ✅ | ⚠️ Limited | ✅ | REST API |
| 15-minute | ✅ | ⚠️ Limited | ✅ | REST API |
| 1-hour | ✅ | ⚠️ Limited | ✅ | REST API |
| Daily | ✅ | ✅ | ✅ | REST API |
| Weekly | ✅ | ✅ | ✅ | REST API |
| Monthly | ✅ | ✅ | ✅ | REST API |

**Depth by Timeframe:**
- **1-minute bars:** Available (limited by symbol, typically recent months/years)
- **5-minute bars:** ✅ Yes (several years for equities)
- **Hourly:** ✅ Yes
- **Daily:** ✅ Yes (10+ years for major stocks)
- **Weekly/Monthly:** ✅ Yes (calculated from daily)

**Options Historical Data Limitation:**
- Intraday (1m, 5m, 15m): ⚠️ May be unavailable or limited
- Daily: ✅ Available
- Users report difficulty getting intraday options data for backtesting

### Real-time vs Delayed

- **Real-time:** ✅ Yes (default for all data via API)
- **Delayed:** ❌ No delayed data tier
- **Snapshot:** ✅ Yes (via quotes endpoint)

**Latency:**
- **REST API:** <1 second (near real-time)
- **WebSocket:** <100ms (tick-by-tick, true real-time)
- **Order execution:** <50ms (market orders)

---

## Update Frequency

### Real-time Streams (WebSocket)

**Price Updates:**
- **Frequency:** Tick-by-tick (every trade)
- **Latency:** <100ms
- **Mode:** Full (all fields), Lite (LTP only), Depth (orderbook)

**Orderbook:**
- **Type:** Snapshot + delta (TBT WebSocket)
- **Levels:** Top 5 bid/ask
- **Update:** On change (real-time)

**Trades:**
- **Type:** Tick-by-tick (TBT WebSocket)
- **Frequency:** Every trade event
- **Latency:** <100ms

**Order/Position Updates:**
- **Frequency:** Real-time (on status change)
- **Latency:** <50ms (order execution)
- **Channels:** Dedicated Order WebSocket

### Scheduled Updates

**Fundamentals:**
- ❌ Not available via API

**Economic Data:**
- ❌ Not available via API

**News:**
- ❌ Not available via API

**Symbol Master:**
- **Frequency:** Daily (updated before market open)
- **Format:** CSV download
- **Time:** ~8:00 AM IST (before market open)

**Holdings:**
- **Frequency:** End of day (T+1 settlement)
- **Update:** After market close

**Corporate Actions:**
- Reflected in adjusted prices (splits, dividends)
- No separate corporate action feed

---

## Data Quality

### Accuracy

**Source:**
- **Direct from exchange:** ✅ Yes
  - NSE, BSE, MCX, NCDEX feed directly to brokers
- **Aggregated:** ❌ No (not multi-exchange aggregator)
- **Calculated:** Only for derived fields (P&L, averages)

**Validation:**
- **Exchange-level:** ✅ Yes (data validated by exchanges)
- **Broker-level:** ✅ Yes (Fyers validates API requests)

**Corrections:**
- **Automatic:** ✅ Yes (exchange corrections reflected)
- **Manual:** ❌ Not applicable

### Completeness

**Missing Data:**
- **Common:** ❌ Rare (exchange data is comprehensive)
- **Rare cases:** Market halts, technical issues (exchange-level)

**Gaps:**
- **How handled:** N/A (continuous data from exchanges)
- **Trading halts:** Reflected in data (no updates during halt)

**Backfill:**
- **Available:** Not officially documented
- **On request:** Not specified

### Timeliness

**Latency:**
- **Real-time (WebSocket):** <100ms
- **REST API:** <1 second
- **Order execution:** <50ms (market orders)

**Delay:**
- **Typical:** None (real-time)
- **Market hours:** 9:15 AM - 3:30 PM IST (NSE/BSE equity)
- **Pre-market:** 9:00 AM - 9:08 AM IST (NSE)
- **Commodity hours:** Extended (MCX morning + evening sessions)

**Market Hours Coverage:**
- **Regular hours:** ✅ Fully covered
- **Pre-market:** ✅ Yes (NSE)
- **After-hours:** ❌ Not applicable (no after-hours trading in India)
- **Commodity sessions:** ✅ Both morning & evening

---

## Market Hours (IST - UTC+5:30)

### NSE/BSE Equity (CM Segment)

**Pre-Opening Session (NSE):**
- **Order Entry:** 9:00 AM - 9:08 AM
- **Order Matching:** 9:08 AM - 9:15 AM

**Regular Market Hours:**
- **Start:** 9:15 AM
- **End:** 3:30 PM
- **Duration:** 6 hours 15 minutes

**Post-Market Session:**
- ❌ Not applicable (no post-market trading)

### NSE/BSE Derivatives (F&O Segment)

**Regular Market Hours:**
- **Start:** 9:15 AM
- **End:** 3:30 PM
- **Duration:** 6 hours 15 minutes

### Currency Derivatives (NSE CD)

**Regular Market Hours:**
- **Start:** 9:00 AM
- **End:** 5:00 PM
- **Duration:** 8 hours

### MCX Commodities

**Morning Session:**
- **Start:** 9:00 AM
- **End:** 5:00 PM
- **Duration:** 8 hours

**Evening Session:**
- **Start:** 5:00 PM
- **End:** 11:30 PM (or 11:55 PM during US daylight saving)
- **Duration:** ~6.5 hours

**Note:** MCX has dual sessions for most commodities.

### NCDEX Commodities

**Morning Session:**
- **Start:** 10:00 AM
- **End:** 5:00 PM
- **Duration:** 7 hours

### Trading Holidays

**2026 Holiday Calendar:**
- 15 weekday holidays for NSE/BSE
- Muhurat Trading on Diwali (evening session only)
- MCX holidays may be partial (morning session only)

**Holiday Resources:**
- NSE Holidays: https://www.nseindia.com/resources/exchange-communication-holidays
- BSE Holidays: https://www.bseindia.com/static/markets/marketinfo/listholi.aspx
- Fyers Calendar: https://fyers.in/holiday-calendar/

---

## Symbol Coverage Examples

### Top Traded Equities (All Covered)
- Reliance Industries (RELIANCE)
- State Bank of India (SBIN)
- HDFC Bank (HDFCBANK)
- Infosys (INFY)
- TCS (TCS)
- ICICI Bank (ICICIBANK)
- Tata Motors (TATAMOTORS)
- Bajaj Finance (BAJFINANCE)
- All Nifty 50 constituents
- All Sensex 30 constituents

### Index F&O (All Covered)
- Nifty 50 (weekly & monthly)
- Bank Nifty (weekly & monthly)
- Fin Nifty (weekly & monthly)
- Nifty IT, Pharma, Auto, FMCG
- Sensex (monthly)

### Stock F&O (Sample - 180+ Total)
- Reliance, SBIN, HDFC Bank, Infosys, TCS
- ICICI Bank, Axis Bank, Kotak Bank
- Tata Motors, M&M, Bajaj Auto
- ITC, HUL, Asian Paints
- (All F&O stocks on NSE)

### Commodities (Sample)
- MCX: Gold, Silver, Crude Oil, Natural Gas, Copper
- NCDEX: Soybean, Wheat, Chana, Turmeric, Cotton

### Currency Derivatives
- USDINR, EURINR, GBPINR, JPYINR

---

## Coverage Limitations

### Not Covered by Fyers API

1. **International Markets**
   - US stocks (NYSE, NASDAQ)
   - European stocks (LSE, Euronext, DAX)
   - Asian markets (TSE, HKEX, SSE)

2. **Cryptocurrency**
   - Spot trading
   - Crypto derivatives
   - DeFi protocols

3. **Spot Forex**
   - Only currency derivatives (futures/options)
   - No spot FX pairs

4. **Fixed Income**
   - Government bonds (G-Secs)
   - Corporate bonds
   - T-Bills

5. **Alternative Investments**
   - REITs (limited - some traded on NSE)
   - InvITs
   - SGBs (Sovereign Gold Bonds) - limited

6. **OTC Markets**
   - Over-the-counter stocks
   - Unlisted securities

7. **Pre-IPO / Private Markets**
   - Not accessible

---

## Competitive Comparison (India Brokers)

| Feature | Fyers | Zerodha | Upstox | Angel One |
|---------|-------|---------|--------|-----------|
| **Exchanges** | NSE, BSE, MCX, NCDEX | NSE, BSE, MCX, NCDEX | NSE, BSE, MCX | NSE, BSE, MCX, NCDEX |
| **Equity** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **F&O** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Commodities** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **Currency** | ✅ Full | ✅ Full | ✅ Full | ✅ Full |
| **API Cost** | Free | Rs 2,000/mo | Free | Free |
| **Real-time Data** | Free | Paid add-on | Free | Free |
| **WebSocket** | Free, 5k symbols | Paid, 3k symbols | Limited | Available |

**Fyers Advantage:** Completely free API with generous WebSocket limits.

---

## Notes

1. **India-focused broker** - Only Indian exchanges (NSE, BSE, MCX, NCDEX)
2. **No international markets** - US, Europe, Asia not covered
3. **No cryptocurrency** - Crypto trading not supported
4. **Real-time data included** - No delayed data tier
5. **Symbol master updated daily** - Download before market open
6. **Options historical data limited** - Intraday may not be available
7. **Free API is rare** - Most competitors charge monthly fees
8. **High WebSocket limits** - 5,000 symbols (V3) vs competitors' 200-3,000
9. **Fast order execution** - <50ms for market orders
10. **Comprehensive F&O coverage** - Specialization in derivatives trading
