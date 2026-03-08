# Dukascopy - Response Formats

## Important Note

Dukascopy does NOT use JSON/REST APIs officially. Response formats are:
1. **Java Objects** (JForex SDK) - Primary method
2. **Binary Files** (.bi5 format) - Historical tick data
3. **FIX Messages** (FIX 4.4 protocol) - Professional trading
4. **JSON** (third-party REST wrapper only) - Unofficial

This document covers ALL formats.

---

## Java Objects (JForex SDK)

### ITick Interface (Tick Data)

**Java Object**:
```java
ITick tick = history.getLastTick(Instrument.EURUSD);

// Access fields
long time = tick.getTime();              // 1234567890123 (Unix ms)
double bid = tick.getBid();              // 1.12345
double ask = tick.getAsk();              // 1.12347
double bidVolume = tick.getBidVolume();  // 1500000.0
double askVolume = tick.getAskVolume();  // 1200000.0

// Order book (10 levels)
double[] bids = tick.getBids();          // [1.12345, 1.12344, 1.12343, ...]
double[] asks = tick.getAsks();          // [1.12347, 1.12348, 1.12349, ...]
```

**Field Descriptions**:

| Method | Return Type | Description | Example |
|--------|-------------|-------------|---------|
| getTime() | long | Timestamp (Unix milliseconds) | 1234567890123 |
| getBid() | double | Best bid price | 1.12345 |
| getAsk() | double | Best ask price | 1.12347 |
| getBidVolume() | double | Volume at best bid | 1500000.0 |
| getAskVolume() | double | Volume at best ask | 1200000.0 |
| getBids() | double[] | Top 10 bid prices | [1.12345, 1.12344, ...] |
| getAsks() | double[] | Top 10 ask prices | [1.12347, 1.12348, ...] |

**Equivalent JSON** (for reference):
```json
{
  "time": 1234567890123,
  "bid": 1.12345,
  "ask": 1.12347,
  "bidVolume": 1500000.0,
  "askVolume": 1200000.0,
  "bids": [1.12345, 1.12344, 1.12343, 1.12342, 1.12341, 1.12340, 1.12339, 1.12338, 1.12337, 1.12336],
  "asks": [1.12347, 1.12348, 1.12349, 1.12350, 1.12351, 1.12352, 1.12353, 1.12354, 1.12355, 1.12356]
}
```

---

### IBar Interface (OHLC Candles)

**Java Object**:
```java
IBar bar = history.getBar(Instrument.EURUSD, Period.ONE_MIN, OfferSide.BID, 0);

// Access fields
long time = bar.getTime();        // 1234567890000 (bar start time)
double open = bar.getOpen();      // 1.12340
double high = bar.getHigh();      // 1.12350
double low = bar.getLow();        // 1.12335
double close = bar.getClose();    // 1.12345
double volume = bar.getVolume();  // 150.0 (tick count)
```

**Field Descriptions**:

| Method | Return Type | Description | Example |
|--------|-------------|-------------|---------|
| getTime() | long | Bar start time (Unix ms) | 1234567890000 |
| getOpen() | double | Opening price | 1.12340 |
| getHigh() | double | Highest price | 1.12350 |
| getLow() | double | Lowest price | 1.12335 |
| getClose() | double | Closing price | 1.12345 |
| getVolume() | double | Tick volume (count) | 150.0 |

**Equivalent JSON**:
```json
{
  "time": 1234567890000,
  "open": 1.12340,
  "high": 1.12350,
  "low": 1.12335,
  "close": 1.12345,
  "volume": 150.0
}
```

**List of Bars** (getBars method):
```java
List<IBar> bars = history.getBars(Instrument.EURUSD, Period.ONE_MIN, OfferSide.BID, from, to);
```

**Equivalent JSON Array**:
```json
[
  {
    "time": 1234567890000,
    "open": 1.12340,
    "high": 1.12350,
    "low": 1.12335,
    "close": 1.12345,
    "volume": 150.0
  },
  {
    "time": 1234567950000,
    "open": 1.12345,
    "high": 1.12355,
    "low": 1.12340,
    "close": 1.12350,
    "volume": 142.0
  }
]
```

---

### Instrument Enum (Symbol Information)

**Java Object**:
```java
Instrument instrument = Instrument.EURUSD;

String name = instrument.name();                        // "EURUSD"
Currency primary = instrument.getPrimaryCurrency();     // Currency.EUR
Currency secondary = instrument.getSecondaryCurrency(); // Currency.USD
double pipValue = instrument.getPipValue();             // 0.0001
int pipScale = instrument.getPipScale();                // 4
```

**Field Descriptions**:

| Method | Return Type | Description | Example |
|--------|-------------|-------------|---------|
| name() | String | Instrument name | "EURUSD" |
| getPrimaryCurrency() | Currency | Base currency | EUR |
| getSecondaryCurrency() | Currency | Quote currency | USD |
| getPipValue() | double | Pip value | 0.0001 |
| getPipScale() | int | Decimal places | 4 |

**Equivalent JSON**:
```json
{
  "symbol": "EURUSD",
  "primaryCurrency": "EUR",
  "secondaryCurrency": "USD",
  "pipValue": 0.0001,
  "pipScale": 4
}
```

---

## Binary Format (.bi5 Files)

### File Structure

**Format**: LZMA-compressed binary
**URL Pattern**: `https://datafeed.dukascopy.com/datafeed/{SYMBOL}/{YYYY}/{MM}/{DD}/{HH}h_ticks.bi5`

**Decompressed Structure**:
- Record size: 20 bytes per tick
- Byte order: Big-endian
- Struct format: `>3I2f` (3 unsigned ints + 2 floats)

### Binary Record (20 bytes)

| Offset | Size | Type | Description | Notes |
|--------|------|------|-------------|-------|
| 0 | 4 bytes | uint32 | Timestamp offset | Milliseconds from hour start |
| 4 | 4 bytes | uint32 | Ask price (raw) | Multiply by point value |
| 8 | 4 bytes | uint32 | Bid price (raw) | Multiply by point value |
| 12 | 4 bytes | float32 | Ask volume | Base currency |
| 16 | 4 bytes | float32 | Bid volume | Base currency |

### Decoding Example (Python)

```python
import lzma
import struct
import requests
from datetime import datetime

# Download .bi5 file
url = "https://datafeed.dukascopy.com/datafeed/EURUSD/2024/00/15/14h_ticks.bi5"
response = requests.get(url)
compressed_data = response.content

# Decompress
decompressed = lzma.decompress(compressed_data)

# Parse ticks
point_value = 0.00001  # For EURUSD (5 decimals)
hour_start = datetime(2024, 1, 15, 14, 0, 0).timestamp() * 1000

ticks = []
for i in range(0, len(decompressed), 20):
    record = struct.unpack('>3I2f', decompressed[i:i+20])

    tick = {
        "time": int(hour_start + record[0]),  # Unix ms
        "ask": record[1] * point_value,
        "bid": record[2] * point_value,
        "askVolume": record[3],
        "bidVolume": record[4]
    }
    ticks.append(tick)

# Result
print(ticks[0])
```

**Output** (JSON equivalent):
```json
{
  "time": 1234567890123,
  "ask": 1.12347,
  "bid": 1.12345,
  "askVolume": 1200000.0,
  "bidVolume": 1500000.0
}
```

### Point Values (for decoding)

| Instrument Type | Decimals | Point Value | Example |
|-----------------|----------|-------------|---------|
| Most forex pairs | 5 | 0.00001 | EUR/USD, GBP/USD |
| JPY pairs | 3 | 0.001 | USD/JPY, EUR/JPY |
| Metals | 5 | 0.00001 | XAU/USD |
| Indices | 2 | 0.01 | SPX500 |

---

## FIX 4.4 Protocol

### Market Data Snapshot (MsgType=W)

**FIX Message**:
```
8=FIX.4.4|9=XXX|35=W|49=DUKASCOPY|56=CLIENT|
34=123|52=20240115-14:30:00|
55=EUR/USD|262=REQ123|268=2|
269=0|270=1.12345|271=1500000|
269=1|270=1.12347|271=1200000|
10=XXX|
```

**Field Descriptions**:

| Tag | Field Name | Value | Description |
|-----|------------|-------|-------------|
| 35 | MsgType | W | Market Data Snapshot |
| 55 | Symbol | EUR/USD | Instrument |
| 262 | MDReqID | REQ123 | Request ID |
| 268 | NoMDEntries | 2 | Number of entries |
| 269 | MDEntryType | 0/1 | 0=Bid, 1=Ask |
| 270 | MDEntryPx | 1.12345 | Price |
| 271 | MDEntrySize | 1500000 | Volume |

**Equivalent JSON**:
```json
{
  "msgType": "MarketDataSnapshot",
  "symbol": "EUR/USD",
  "reqID": "REQ123",
  "time": "2024-01-15T14:30:00Z",
  "entries": [
    {
      "type": "bid",
      "price": 1.12345,
      "size": 1500000
    },
    {
      "type": "ask",
      "price": 1.12347,
      "size": 1200000
    }
  ]
}
```

### Execution Report (MsgType=8)

**FIX Message**:
```
8=FIX.4.4|9=XXX|35=8|49=DUKASCOPY|56=CLIENT|
11=ORDER123|37=DK12345678|17=EXEC123|
150=2|39=2|55=EUR/USD|54=1|
38=100000|44=1.12345|31=1.12345|32=100000|
151=0|14=100000|6=1.12345|
60=20240115-14:30:00|10=XXX|
```

**Field Descriptions**:

| Tag | Field Name | Value | Description |
|-----|------------|-------|-------------|
| 35 | MsgType | 8 | Execution Report |
| 11 | ClOrdID | ORDER123 | Client order ID |
| 37 | OrderID | DK12345678 | Dukascopy order ID |
| 39 | OrdStatus | 2 | 2=Filled |
| 150 | ExecType | 2 | 2=Fill |
| 55 | Symbol | EUR/USD | Instrument |
| 54 | Side | 1 | 1=Buy, 2=Sell |
| 38 | OrderQty | 100000 | Order size |
| 31 | LastPx | 1.12345 | Fill price |
| 32 | LastQty | 100000 | Fill quantity |
| 14 | CumQty | 100000 | Total filled |
| 151 | LeavesQty | 0 | Remaining |

**Equivalent JSON**:
```json
{
  "msgType": "ExecutionReport",
  "clientOrderID": "ORDER123",
  "dukasOrderID": "DK12345678",
  "execID": "EXEC123",
  "symbol": "EUR/USD",
  "side": "buy",
  "orderQty": 100000,
  "orderStatus": "filled",
  "execType": "fill",
  "lastPrice": 1.12345,
  "lastQty": 100000,
  "cumQty": 100000,
  "leavesQty": 0,
  "avgPrice": 1.12345,
  "time": "2024-01-15T14:30:00Z"
}
```

---

## Third-Party REST API (Unofficial)

### GET /api/v1/history (Historical Bars)

**Request**:
```
GET http://localhost:7080/api/v1/history?instID=EURUSD&timeFrame=1MIN&from=1234567890000&to=1234567950000
```

**Response** (JSON Array):
```json
[
  {
    "symbol": "EURUSD",
    "open": 1.12340,
    "high": 1.12350,
    "low": 1.12335,
    "close": 1.12345,
    "volume": 150.0,
    "ticks": 150,
    "time": 1234567890000,
    "period": "ONE_MIN"
  },
  {
    "symbol": "EURUSD",
    "open": 1.12345,
    "high": 1.12355,
    "low": 1.12340,
    "close": 1.12350,
    "volume": 142.0,
    "ticks": 142,
    "time": 1234567950000,
    "period": "ONE_MIN"
  }
]
```

**Field Descriptions**:

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| symbol | string | Instrument identifier | "EURUSD" |
| open | number | Opening price | 1.12340 |
| high | number | Highest price | 1.12350 |
| low | number | Lowest price | 1.12335 |
| close | number | Closing price | 1.12345 |
| volume | number | Volume (tick count) | 150.0 |
| ticks | number | Number of ticks | 150 |
| time | number | Bar start time (Unix ms) | 1234567890000 |
| period | string | Period identifier | "ONE_MIN" |

---

### POST /api/v1/position (Open Order)

**Request**:
```json
{
  "instID": "EURUSD",
  "clientOrderID": "ORDER123",
  "orderSide": "Buy",
  "orderType": "Market",
  "quantity": 0.1,
  "slippage": 5
}
```

**Response (Success)**:
```json
{
  "symbol": "EURUSD",
  "clientOrderID": "ORDER123",
  "dukasOrderID": "DK12345678",
  "fillQty": 0.1,
  "fillPrice": 1.12345,
  "orderSuccess": true
}
```

**Response (Rejection)**:
```json
{
  "symbol": "EURUSD",
  "clientOrderID": "ORDER123",
  "fillQty": 0.0,
  "fillPrice": 0.0,
  "orderSuccess": false,
  "rejectReason": "Insufficient margin"
}
```

---

### GET /api/v1/position (Query Position)

**Request**:
```
GET http://localhost:7080/api/v1/position?clientOrderID=ORDER123
```

**Response**:
```json
{
  "symbol": "EURUSD",
  "clientOrderID": "ORDER123",
  "dukasOrderID": "DK12345678",
  "side": "Buy",
  "quantity": 0.1,
  "openPrice": 1.12345,
  "currentPrice": 1.12350,
  "pnl": 0.50,
  "status": "open"
}
```

---

## WebSocket (Third-Party)

### Top-of-Book Message

**WebSocket URL**: `ws://localhost:7081/ticker?topOfBook=true`

**Message**:
```json
{
  "symbol": "EURUSD",
  "bidQty": 1500000.0,
  "bid": 1.12345,
  "ask": 1.12347,
  "askQty": 1200000.0,
  "last": 1.12346,
  "spread": 0.00002,
  "spreadBps": 0.2,
  "updateTime": 1234567890123,
  "updateNumber": 12345,
  "depthLevels": 2,
  "live": true
}
```

### Order Book Message (10 Levels)

**WebSocket URL**: `ws://localhost:7081/ticker?topOfBook=false&instIDs=EURUSD`

**Message**:
```json
{
  "symbol": "EURUSD",
  "bidQty": 1500000.0,
  "bid": 1.12345,
  "ask": 1.12347,
  "askQty": 1200000.0,
  "last": 1.12346,
  "spread": 0.00002,
  "spreadBps": 0.2,
  "updateTime": 1234567890123,
  "updateNumber": 12345,
  "depthLevels": 10,
  "live": true,
  "bids": [
    {"quantity": 1500000.0, "price": 1.12345},
    {"quantity": 800000.0, "price": 1.12344},
    {"quantity": 1200000.0, "price": 1.12343},
    {"quantity": 950000.0, "price": 1.12342},
    {"quantity": 1100000.0, "price": 1.12341},
    {"quantity": 870000.0, "price": 1.12340},
    {"quantity": 1050000.0, "price": 1.12339},
    {"quantity": 920000.0, "price": 1.12338},
    {"quantity": 1180000.0, "price": 1.12337},
    {"quantity": 890000.0, "price": 1.12336}
  ],
  "asks": [
    {"quantity": 1200000.0, "price": 1.12347},
    {"quantity": 900000.0, "price": 1.12348},
    {"quantity": 1100000.0, "price": 1.12349},
    {"quantity": 850000.0, "price": 1.12350},
    {"quantity": 1050000.0, "price": 1.12351},
    {"quantity": 930000.0, "price": 1.12352},
    {"quantity": 1080000.0, "price": 1.12353},
    {"quantity": 960000.0, "price": 1.12354},
    {"quantity": 1150000.0, "price": 1.12355},
    {"quantity": 910000.0, "price": 1.12356}
  ]
}
```

---

## Error Responses

### Java SDK Exceptions

**CaptchaException**:
```java
try {
    client.connect(jnlpUrl, username, password);
} catch (CaptchaException e) {
    System.err.println("CAPTCHA required: " + e.getMessage());
}
```

**AuthenticationException**:
```java
catch (AuthenticationException e) {
    System.err.println("Authentication failed: " + e.getMessage());
}
```

### HTTP Errors (Binary Downloads)

**404 Not Found**:
```
HTTP/1.1 404 Not Found
Content-Type: text/html

File not found
```

**429 Rate Limited**:
```
HTTP/1.1 429 Too Many Requests
Retry-After: 60

Rate limit exceeded
```

### FIX Reject

**Message Reject (MsgType=3)**:
```
8=FIX.4.4|9=XXX|35=3|49=DUKASCOPY|56=CLIENT|
45=1|58=Invalid credentials|372=A|373=1|10=XXX|
```

**Equivalent JSON**:
```json
{
  "msgType": "Reject",
  "refSeqNum": 1,
  "refMsgType": "Logon",
  "reason": "Invalid credentials",
  "rejectReason": 1
}
```

---

## Summary Table

| Format | Use Case | Official? | Complexity | Best For |
|--------|----------|-----------|------------|----------|
| Java Objects | JForex SDK | Yes | Medium | Official integration |
| Binary (.bi5) | Historical ticks | Yes | High | Bulk downloads |
| FIX 4.4 | Professional trading | Yes | High | Institutional |
| JSON (REST) | Third-party | No | Low | Prototyping |
| JSON (WebSocket) | Third-party | No | Low | Real-time feeds |

---

## Code Examples

### Java SDK (Official)

```java
// Get last tick
ITick tick = history.getLastTick(Instrument.EURUSD);
System.out.println("Bid: " + tick.getBid() + ", Ask: " + tick.getAsk());

// Get bars
List<IBar> bars = history.getBars(
    Instrument.EURUSD,
    Period.ONE_MIN,
    OfferSide.BID,
    from,
    to
);

for (IBar bar : bars) {
    System.out.println("Time: " + bar.getTime() + ", Close: " + bar.getClose());
}
```

### Binary Decoding (Python)

```python
import lzma, struct, requests

url = "https://datafeed.dukascopy.com/datafeed/EURUSD/2024/00/15/14h_ticks.bi5"
data = lzma.decompress(requests.get(url).content)

for i in range(0, len(data), 20):
    ms_offset, ask_raw, bid_raw, ask_vol, bid_vol = struct.unpack('>3I2f', data[i:i+20])
    print(f"Bid: {bid_raw * 0.00001}, Ask: {ask_raw * 0.00001}")
```

### REST (Third-Party)

```bash
curl "http://localhost:7080/api/v1/history?instID=EURUSD&timeFrame=1MIN&from=1234567890000&to=1234567950000"
```

### WebSocket (Third-Party)

```javascript
const ws = new WebSocket('ws://localhost:7081/ticker?topOfBook=false&instIDs=EURUSD');

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log(`Bid: ${data.bid}, Ask: ${data.ask}`);
};
```
