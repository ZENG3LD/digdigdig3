# Dukascopy - Complete Endpoint Reference

## IMPORTANT NOTE

Dukascopy does NOT provide a traditional REST API. Instead, data access is through:
1. **JForex SDK (Java)** - Primary official method
2. **FIX 4.4 Protocol** - Professional trading
3. **Direct binary file downloads** - Historical tick data
4. **Third-party wrappers** - Community-built REST/WebSocket interfaces

This document covers all available access methods.

---

## Category: JForex SDK - IHistory Interface Methods

The IHistory interface is the primary way to access historical data programmatically.

**Access Method**: Java SDK
**Documentation**: https://www.dukascopy.com/client/javadoc3/com/dukascopy/api/IHistory.html
**Requirements**: JForex SDK, Demo or Live account
**Thread Safety**: All methods are thread-safe

### Bar Data Methods

| Method | Parameters | Return Type | Description | Sync/Async | Notes |
|--------|-----------|-------------|-------------|------------|-------|
| getBar() | Instrument, Period, OfferSide, int shift | IBar | Get single bar by shift | Sync | shift 0=current, 1=previous |
| getBars() | Instrument, Period, OfferSide, long from, long to | List\<IBar\> | Get bars by time range | Sync | Inclusive boundaries |
| getBars() | Instrument, Period, OfferSide, Filter, int before, long time, int after | List\<IBar\> | Get bars centered on time | Sync | With filtering |
| readBars() | Instrument, Period, OfferSide, long from, long to, LoadingDataListener, LoadingProgressListener | void | Load bars async | Async | Background loading |

**Periods Available**:
- TICK (tick data)
- TEN_SECS, THIRTY_SECS, ONE_MIN, FIVE_MINS, TEN_MINS, FIFTEEN_MINS, THIRTY_MINS
- ONE_HOUR, FOUR_HOURS
- DAILY, WEEKLY, MONTHLY

**OfferSide**: BID, ASK

**IBar Fields**:
- getTime(): long (timestamp in ms)
- getOpen(): double
- getHigh(): double
- getLow(): double
- getClose(): double
- getVolume(): double

### Tick Data Methods

| Method | Parameters | Return Type | Description | Sync/Async | Notes |
|--------|-----------|-------------|-------------|------------|-------|
| getLastTick() | Instrument | ITick | Get most recent tick | Sync | Current market price |
| getTick() | Instrument, int shift | ITick | Get tick by shift | Sync | shift 0=last, 1=previous |
| getTicks() | Instrument, long from, long to | List\<ITick\> | Get ticks by time range | Sync | Inclusive boundaries |
| getTicks() | Instrument, int before, long time, int after | List\<ITick\> | Get ticks centered on time | Sync | 1-second intervals |
| readTicks() | Instrument, long from, long to, LoadingDataListener, LoadingProgressListener | void | Load ticks async | Async | Background loading |

**ITick Fields**:
- getTime(): long (timestamp in ms)
- getBid(): double
- getAsk(): double
- getBidVolume(): double
- getAskVolume(): double
- getAsks(): double[] (top 10 ask prices)
- getBids(): double[] (top 10 bid prices)

### Feed Data Methods

| Method | Parameters | Return Type | Description | Sync/Async | Notes |
|--------|-----------|-------------|-------------|------------|-------|
| getFeedData() | IFeedDescriptor, int shift | ITimedData | Get feed data by shift | Sync | Custom feed types |
| readFeedData() | ITailoredFeedDescriptor, long from, long to, ITailoredFeedListener, LoadingProgressListener | void | Load feed data async | Async | Type-safe loading |

**Custom Feed Types Supported**:
- Renko bars
- Kagi bars
- Line break bars
- Point and figure bars
- Range bars

### Timing Utility Methods

| Method | Parameters | Return Type | Description | Notes |
|--------|-----------|-------------|-------------|-------|
| getTimeOfLastTick() | Instrument | long | Time of last received tick | Returns -1 if none |
| getBarStart() | Period, long time | long | Get bar start time containing time | |
| getNextBarStart() | Period, long barTime | long | Get next bar start time | |
| getPreviousBarStart() | Period, long barTime | long | Get previous bar start time | |
| getStartTimeOfCurrentBar() | Instrument, Period | long | Get current bar start time | |
| getTimeForNBarsBack() | Period, long to, int numberOfBars | long | Calculate time N bars back | |
| getTimeForNBarsForward() | Period, long from, int numberOfBars | long | Calculate time N bars forward | |

### First Data Availability (IDataService)

| Method | Parameters | Return Type | Description | Notes |
|--------|-----------|-------------|-------------|-------|
| getTimeOfFirstCandle() | Instrument, Period | long | First candle timestamp | Check data availability |
| getTimeOfFirstRenko() | Instrument, ... | long | First renko bar timestamp | |
| getTimeOfFirstKagi() | Instrument, ... | long | First kagi bar timestamp | |
| getTimeOfFirstLineBreak() | Instrument, ... | long | First line break timestamp | |
| getTimeOfFirstPointAndFigure() | Instrument, ... | long | First P&F timestamp | |
| getTimeOfFirstRangeBar() | Instrument, ... | long | First range bar timestamp | |

---

## Category: Direct Binary File Downloads

**Access Method**: HTTP GET (no auth required)
**Format**: LZMA-compressed binary (.bi5)
**Data**: Tick-level data (bid, ask, volumes)

### Historical Tick Data Download

| Type | URL Pattern | Description | Auth? | Free? | Rate Limit | Notes |
|------|-------------|-------------|-------|-------|------------|-------|
| GET | https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5 | Hourly tick data | No | Yes | Yes (undocumented) | LZMA compressed |

**URL Parameters**:
- `{SYMBOL}`: Instrument name (e.g., EURUSD, XAUUSD)
- `{YYYY}`: 4-digit year (e.g., 2024)
- `{MM}`: 2-digit month, 0-indexed (00=Jan, 11=Dec)
- `{DD}`: 2-digit day (00-31)
- `{HH}`: 2-digit hour (00-23)

**File Format**:
- Compression: LZMA
- Record size: 20 bytes per tick
- Fields per tick (all big-endian):
  - Timestamp offset: 4 bytes (unsigned int, milliseconds from hour start)
  - Ask price: 4 bytes (unsigned int, multiply by point value)
  - Bid price: 4 bytes (unsigned int, multiply by point value)
  - Ask volume: 4 bytes (float)
  - Bid volume: 4 bytes (float)

**Example**:
```
URL: https://datafeed.dukascopy.com/datafeed/EURUSD/2024/01/15/14h_ticks.bi5
Description: EUR/USD ticks for Jan 15, 2024, 14:00-14:59 UTC
```

---

## Category: FIX 4.4 Protocol

**Access Method**: FIX 4.4 over SSL
**Documentation**: https://www.dukascopy.com/swiss/docs/Dukascopy_FIXAPI-8.0.1.pdf
**Requirements**: Live account, USD 100,000 minimum deposit, IP registration

### Connection Endpoints

| Service | Host | Port | Protocol | Description |
|---------|------|------|----------|-------------|
| Trading Gateway | (provided on registration) | 10443 | SSL/TLS | Order management |
| Data Feed | (provided on registration) | 9443 | SSL/TLS | Market data |

### FIX Message Types (Selected)

**Session Management**:
- Logon (A): Initial authentication
- Logout (5): Session termination
- Heartbeat (0): Keep-alive (default 30s interval)
- TestRequest (1): Connection test
- ResendRequest (2): Replay messages
- Reject (3): Message rejection
- SequenceReset (4): Sequence number reset

**Market Data**:
- MarketDataRequest (V): Subscribe to market data
- MarketDataSnapshotFullRefresh (W): Market data snapshot
- MarketDataIncrementalRefresh (X): Market data updates

**Order Management**:
- NewOrderSingle (D): Place new order
- OrderCancelRequest (F): Cancel order
- OrderCancelReplaceRequest (G): Modify order
- ExecutionReport (8): Order status/fill
- OrderStatusRequest (H): Query order status

**Position Management**:
- PositionReport (AP): Position information
- RequestForPositions (AN): Query positions

### Rate Limits (FIX API)

| Limit Type | Value | Notes |
|-----------|-------|-------|
| Max orders per second | 16 | Hard limit |
| Max open positions | 100 | Per account |
| Connection attempts | 5 per minute | Per server, per IP |
| Session timeout | 2 hours | If not restored |
| Heartbeat interval | 30 seconds | Default (configurable) |

---

## Category: Third-Party REST API (Unofficial)

**Source**: https://github.com/ismailfer/dukascopy-api-websocket
**Type**: Community wrapper around JForex SDK
**Technology**: Spring Boot
**Status**: Unofficial, unsupported by Dukascopy

### Market Data Endpoints

| Method | Endpoint | Description | Auth? | Parameters | Notes |
|--------|----------|-------------|-------|------------|-------|
| GET | http://localhost:7080/api/v1/history | Historical OHLCV data | Config file | instID, timeFrame, from, to | See below |

**Parameters for /api/v1/history**:

| Name | Type | Required | Values | Description |
|------|------|----------|--------|-------------|
| instID | string | Yes | EURUSD, GBPUSD, etc. | Instrument identifier |
| timeFrame | string | Yes | 1SEC, 10SEC, 1MIN, 5MIN, 10MIN, 15MIN, 1HOUR, DAILY | Candle interval |
| from | timestamp | Yes | Unix milliseconds | Start time (inclusive) |
| to | timestamp | No | Unix milliseconds | End time (inclusive) |

**Response Format** (JSON Array):
```json
[
  {
    "symbol": "EURUSD",
    "open": 1.1234,
    "high": 1.1245,
    "low": 1.1230,
    "close": 1.1240,
    "volume": 12345.67,
    "ticks": 150,
    "time": 1234567890000,
    "period": "ONE_MIN"
  }
]
```

### Order Management Endpoints

| Method | Endpoint | Description | Parameters | Response |
|--------|----------|-------------|------------|----------|
| GET | http://localhost:7080/api/v1/position | Get position info | clientOrderID or dukasOrderID | Position details |
| POST | http://localhost:7080/api/v1/position | Open position | See below | Order confirmation |
| PUT | http://localhost:7080/position | Modify position | clientOrderID/dukasOrderID, takeProfitPips, stopLossPips | Update confirmation |
| DELETE | http://localhost:7080/position | Close position | clientOrderID or dukasOrderID | Close confirmation |

**POST /api/v1/position Parameters**:

| Name | Type | Required | Values | Description |
|------|------|----------|--------|-------------|
| instID | string | Yes | EURUSD, etc. | Instrument |
| clientOrderID | string | Yes | Unique ID | Client order reference |
| orderSide | string | Yes | Buy, Sell | Order direction |
| orderType | string | Yes | Market, Limit | Order type |
| quantity | number | Yes | > 0 | Lot size |
| price | number | No | > 0 | Limit price (for Limit orders) |
| slippage | number | No | >= 0 | Allowed slippage |

**Order Response (Success)**:
```json
{
  "symbol": "EURUSD",
  "clientOrderID": "order123",
  "dukasOrderID": "DK12345678",
  "fillQty": 0.1,
  "fillPrice": 1.1234,
  "orderSuccess": true
}
```

**Order Response (Rejection)**:
```json
{
  "symbol": "EURUSD",
  "clientOrderID": "order123",
  "fillQty": 0.0,
  "fillPrice": 0.0,
  "orderSuccess": false,
  "rejectReason": "Insufficient margin"
}
```

### Authentication (Third-Party Wrapper)

**Method**: Configuration file (application.properties)
**Format**:
```properties
dukascopy.username=YOUR_DEMO_USERNAME
dukascopy.password=YOUR_PASSWORD
dukascopy.demo=true  # or false for live
```

---

## Category: Conversion & Utility

**Access Method**: JForex SDK - JFUtils interface

### Conversion Methods

| Method | Parameters | Return Type | Description | Notes |
|--------|-----------|-------------|-------------|-------|
| convert() | double amount, Instrument from, Instrument to | double | Convert amount between instruments | Requires subscription |
| convertPipToCurrency() | Instrument, Currency | double | Convert pip value to currency | Useful for risk calc |
| getRate() | Currency from, Currency to | double | Get exchange rate | Direct conversion |

**Overloads**:
- convert() with precision parameter
- convert() with OfferSide parameter (BID/ASK)

**Requirements**:
- Instruments must be subscribed
- At least one intermediate currency pair needed for indirect conversions

---

## Summary Table: Access Methods

| Method | Use Case | Auth | Free? | Complexity | Best For |
|--------|----------|------|-------|------------|----------|
| JForex SDK | Live/historical data, trading | Demo/Live account | Yes (demo) | High (Java) | Official integration |
| FIX API | Professional trading | Live account + $100k | No | High | Institutional |
| Binary Downloads | Bulk historical ticks | None | Yes | Medium | Backtesting |
| Third-Party REST | Simple REST integration | Config file | Yes | Low | Prototyping |
| Third-Party WebSocket | Real-time data | Config file | Yes | Low | Live feeds |

---

## Important Notes

1. **No Official REST API**: Dukascopy focuses on Java SDK and FIX protocol
2. **Free Historical Data**: Available via binary downloads and demo SDK access
3. **Rate Limiting**: Applied but limits not publicly documented
4. **Community Tools**: Many wrappers exist but are unofficial
5. **Commercial Use**: Requires separate agreement with Dukascopy
6. **Time Synchronization**: Required for FIX API (GMT/UTC)
7. **IP Registration**: Required for FIX API connections

---

## Data Coverage

- **Instruments**: 1,200+ (forex, crypto, stocks, commodities, indices)
- **Historical Depth**: Varies by instrument (forex often back to 2003+)
- **Granularity**: Tick to monthly
- **Update Frequency**: Real-time (sub-second for ticks)
- **Timezone**: UTC+0 for all data
