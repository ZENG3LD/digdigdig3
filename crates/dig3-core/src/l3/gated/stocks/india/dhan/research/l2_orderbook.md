# Dhan — L2 Orderbook Capabilities

## Summary

Dhan (DhanHQ) provides **three tiers of market depth** data via their v2 API:

| Tier | Levels | Delivery | Max Instruments |
|------|--------|----------|-----------------|
| Standard Depth | 5 levels | REST snapshot + WebSocket (Full packet) | 1000 (REST), 5000×5 conn (WS) |
| 20-Level Depth | 20 levels | WebSocket streaming only | 50 per connection |
| Full Market Depth | 200 levels | WebSocket streaming only | 1 per connection |

- **Exchange coverage**: NSE Equity and NSE Derivatives only. BSE is NOT supported for 20 or 200 depth.
- **5-level depth**: Available on all exchanges via REST and Live Market Feed WebSocket.
- All WebSocket responses are **binary, Little Endian**. Requests are JSON.
- Depth is delivered as **snapshot per update** (not delta/incremental), with separate bid and ask packets.

---

## WebSocket Channels

### 1. Live Market Feed (5-Level Depth)

General-purpose real-time feed. Includes 5-level depth in the "Full" packet.

```
wss://api-feed.dhan.co?version=2&token=<ACCESS_TOKEN>&clientId=<CLIENT_ID>&authType=2
```

- Up to **5 simultaneous connections** per user
- Up to **5,000 instruments per connection**
- Max **100 instruments per subscription message**
- Subscription uses `RequestCode: 15` (Full packet with depth = `RequestCode: 8` response)

**Subscribe request:**
```json
{
    "RequestCode": 15,
    "InstrumentCount": 2,
    "InstrumentList": [
        {"ExchangeSegment": "NSE_EQ", "SecurityId": "1333"},
        {"ExchangeSegment": "NSE_FNO", "SecurityId": "49081"}
    ]
}
```

**Disconnect request:**
```json
{"RequestCode": 12}
```

---

### 2. 20-Level Market Depth WebSocket

Dedicated WebSocket for 20-level depth streaming, NSE instruments only.

```
wss://depth-api-feed.dhan.co/twentydepth?token=<ACCESS_TOKEN>&clientId=<CLIENT_ID>&authType=2
```

- Up to **50 instruments per connection**
- All 50 can be sent in a single subscription JSON message
- Separate bid and ask packets (response codes 41 and 51)

**Subscribe request:**
```json
{
    "RequestCode": 23,
    "InstrumentCount": 1,
    "InstrumentList": [
        {"ExchangeSegment": "NSE_EQ", "SecurityId": "1333"}
    ]
}
```

---

### 3. 200-Level Full Market Depth WebSocket

Dedicated WebSocket for full depth streaming, NSE instruments only.

```
wss://full-depth-api.dhan.co/twohundreddepth?token=<ACCESS_TOKEN>&clientId=<CLIENT_ID>&authType=2
```

- Only **1 instrument per connection**
- Same packet structure as 20-level but with 200 price levels

**Subscribe request:**
```json
{
    "RequestCode": 23,
    "ExchangeSegment": "NSE_EQ",
    "SecurityId": "1333"
}
```

---

## REST Endpoints

Base URL: `https://api.dhan.co/v2`

### Authentication Headers (all endpoints)

| Header | Required | Description |
|--------|----------|-------------|
| `access-token` | Yes | Access Token generated via Dhan (daily renewal) |
| `client-id` | Yes | User-specific identifier from Dhan |
| `Content-Type` | Yes | `application/json` |
| `Accept` | Yes | `application/json` |

---

### Market Quote (5-Level Depth Snapshot)

```
POST /marketfeed/quote
```

Returns 5-level market depth snapshot for multiple instruments at once.

**Request body:**
```json
{
    "NSE_EQ": [11536, 1333],
    "NSE_FNO": [49081, 49082],
    "BSE_EQ": [500325]
}
```

- Exchange segment as key, array of Security IDs as value
- Up to **1,000 instruments per request**
- Rate limit: **1 request per second**

**Response fields per instrument:**

| Field | Type | Description |
|-------|------|-------------|
| `last_price` | float | Last traded price |
| `volume` | int | Daily trading volume |
| `oi` | int | Open Interest (derivatives only) |
| `upper_circuit_limit` | float | Price ceiling |
| `lower_circuit_limit` | float | Price floor |
| `net_change` | float | Change from previous close |
| `average_price` | float | Volume-weighted average price |
| `buy_quantity` | int | Total pending buy quantity |
| `sell_quantity` | int | Total pending sell quantity |
| `depth.buy[0..4]` | array | 5 bid levels |
| `depth.sell[0..4]` | array | 5 ask levels |

Each depth level:
```json
{
    "quantity": 500,
    "orders": 3,
    "price": 2345.50
}
```

---

### LTP Only

```
POST /marketfeed/ltp
```

Returns only Last Traded Price. Same request format as `/marketfeed/quote`.

---

### OHLC Data

```
POST /marketfeed/ohlc
```

Returns OHLC + LTP (no depth). Same request format.

---

## Depth Levels

| Level | Access Method | Instruments | Exchange Support |
|-------|---------------|-------------|-----------------|
| 5-level | REST `/marketfeed/quote` | 1000 per request | All (NSE, BSE, MCX, etc.) |
| 5-level | WebSocket Live Feed (Full packet) | 5000 per connection | All |
| 20-level | WebSocket `/twentydepth` | 50 per connection | NSE EQ + FNO only |
| 200-level | WebSocket `/twohundreddepth` | 1 per connection | NSE EQ + FNO only |

BSE instruments: 5-level only (via REST or general WS feed).

---

## Binary Packet Formats

All WebSocket responses are binary, Little Endian.

### Live Market Feed Response Header (8 bytes)

| Bytes | Type | Size | Field |
|-------|------|------|-------|
| 0 | byte | 1 | Feed Response Code |
| 1-2 | int16 | 2 | Payload message length |
| 3 | byte | 1 | Exchange Segment |
| 4-7 | int32 | 4 | Security ID |

### Live Market Feed: Full Packet (Response Code 8)

Header (8 bytes) + Quote fields (55 bytes) + 5-level Depth (100 bytes) = 163 bytes total

**Quote portion (offset 8):**

| Offset | Type | Size | Field |
|--------|------|------|-------|
| 8 | float32 | 4 | Last Traded Price |
| 12 | int16 | 2 | Last Traded Quantity |
| 14 | int32 | 4 | Last Trade Time (Unix epoch) |
| 18 | float32 | 4 | Average Trade Price (VWAP) |
| 22 | int32 | 4 | Volume |
| 26 | int32 | 4 | Total Sell Quantity |
| 30 | int32 | 4 | Total Buy Quantity |
| 34 | float32 | 4 | Day Open |
| 38 | float32 | 4 | Day Close (previous close) |
| 42 | float32 | 4 | Day High |
| 46 | float32 | 4 | Day Low |
| 50 | int32 | 4 | Open Interest |
| 54 | int32 | 4 | Highest OI (NSE_FNO only) |
| 58 | int32 | 4 | Lowest OI (NSE_FNO only) |

**Depth portion (offset 62), 5 levels × 20 bytes each:**

| Offset within level | Type | Size | Field |
|--------------------|------|------|-------|
| 0 | int32 | 4 | Bid Quantity |
| 4 | int32 | 4 | Ask Quantity |
| 8 | int16 | 2 | Number of Bid Orders |
| 10 | int16 | 2 | Number of Ask Orders |
| 12 | float32 | 4 | Bid Price |
| 16 | float32 | 4 | Ask Price |

---

### 20-Level / 200-Level Depth Response Header (12 bytes)

| Bytes | Type | Size | Field |
|-------|------|------|-------|
| 0-1 | int16 | 2 | Message length |
| 2 | byte | 1 | Feed Response Code (41 = Bid, 51 = Ask) |
| 3 | byte | 1 | Exchange Segment |
| 4-7 | int32 | 4 | Security ID |
| 8-11 | uint32 | 4 | Message sequence / row count |

**Per depth level (16 bytes each):**

| Offset | Type | Size | Field |
|--------|------|------|-------|
| 0 | float64 | 8 | Price |
| 8 | uint32 | 4 | Quantity |
| 12 | uint32 | 4 | Order count |

**Total payload sizes:**
- 20-level: 12 + (20 × 16) = 332 bytes per side (bid or ask)
- 200-level: 12 + (200 × 16) = 3,212 bytes per side

**Feed Response Codes:**
- `41` — Bid (buy side) data
- `51` — Ask (sell side) data
- `50` — Disconnection packet

---

## Update Speed

- **WebSocket streams**: Real-time push, server sends updates as exchange ticks arrive.
- **Server ping interval**: Every 10 seconds. Client must respond (auto-pong from library).
- **Connection timeout**: 40 seconds without client response → server closes connection.
- **Snapshot vs delta**: Dhan sends **full snapshots** per update, not incremental deltas. Each bid/ask packet contains all 20 (or 200) levels.
- **REST quote**: Snapshot at request time only, not streaming. Rate: 1 req/sec.

---

## Tier Requirements

| Feature | Requirement |
|---------|-------------|
| Trading APIs (orders, positions) | Free with Dhan account; static IP required |
| Data API (live feed, intraday, historical) | ₹499/month + taxes (recurring, billed every 30 days) |
| 20-Level WebSocket depth | Included in Data API subscription (₹499/month) |
| 200-Level WebSocket depth | Included in Data API subscription (₹499/month) |
| API Key generation | Profile → DhanHQ Trading APIs (web or app) |
| API Key validity | 1 year; access token must be refreshed daily |

**Authentication flow:**
1. Generate API Key + Secret (1-year validity) from Dhan profile
2. Use key + secret + credentials to obtain daily access token
3. Pass `access-token` + `client-id` headers on REST or as WS query params

**Static IP requirement:** Mandatory for all Order APIs (place, modify, cancel). Not required for market data read-only access.

---

## Rate Limits

| API Category | Limit |
|-------------|-------|
| Non-Trading APIs | 20 req/sec |
| Order APIs | 25 req/sec |
| Data APIs | 10 req/sec |
| Quote APIs (`/marketfeed/*`) | 1 req/sec |
| Orders per day | 5,000 max |
| WebSocket connections | 5 per user |
| Instruments per WS connection (Live Feed) | 5,000 |
| Instruments per subscribe message (Live Feed) | 100 |
| Instruments per 20-depth WS connection | 50 |
| Instruments per 200-depth WS connection | 1 |

---

## Raw Examples

### REST: Market Depth Request

```bash
curl -X POST "https://api.dhan.co/v2/marketfeed/quote" \
  -H "access-token: eyJhbGciO..." \
  -H "client-id: 1000012345" \
  -H "Content-Type: application/json" \
  -H "Accept: application/json" \
  -d '{"NSE_EQ": [1333, 11536]}'
```

### REST: Market Depth Response (partial)

```json
{
  "1333": {
    "last_price": 2345.50,
    "volume": 1234567,
    "average_price": 2340.25,
    "net_change": 12.50,
    "buy_quantity": 15000,
    "sell_quantity": 18000,
    "upper_circuit_limit": 2580.00,
    "lower_circuit_limit": 2110.00,
    "depth": {
      "buy": [
        {"price": 2345.50, "quantity": 500, "orders": 3},
        {"price": 2345.00, "quantity": 1200, "orders": 8},
        {"price": 2344.50, "quantity": 750, "orders": 5},
        {"price": 2344.00, "quantity": 2000, "orders": 12},
        {"price": 2343.50, "quantity": 3000, "orders": 15}
      ],
      "sell": [
        {"price": 2346.00, "quantity": 400, "orders": 2},
        {"price": 2346.50, "quantity": 900, "orders": 6},
        {"price": 2347.00, "quantity": 1500, "orders": 9},
        {"price": 2347.50, "quantity": 2200, "orders": 11},
        {"price": 2348.00, "quantity": 3500, "orders": 14}
      ]
    }
  }
}
```

### WebSocket: 20-Level Depth Subscribe

```json
{
    "RequestCode": 23,
    "InstrumentCount": 2,
    "InstrumentList": [
        {"ExchangeSegment": "NSE_EQ", "SecurityId": "1333"},
        {"ExchangeSegment": "NSE_FNO", "SecurityId": "49081"}
    ]
}
```

### Binary Response Parsing (Rust pseudocode for 20-depth)

```rust
// Response header: 12 bytes
let msg_len = i16::from_le_bytes([buf[0], buf[1]]);
let response_code = buf[2]; // 41 = bid, 51 = ask
let exchange_seg = buf[3];
let security_id = i32::from_le_bytes([buf[4], buf[5], buf[6], buf[7]]);
let row_count = u32::from_le_bytes([buf[8], buf[9], buf[10], buf[11]]);

// Depth levels: 16 bytes each, starting at offset 12
for i in 0..20 {
    let offset = 12 + i * 16;
    let price = f64::from_le_bytes(buf[offset..offset+8].try_into().unwrap());
    let qty   = u32::from_le_bytes(buf[offset+8..offset+12].try_into().unwrap());
    let count = u32::from_le_bytes(buf[offset+12..offset+16].try_into().unwrap());
}
```

### Instrument Security IDs (examples)

| Symbol | SecurityId | Exchange |
|--------|-----------|---------|
| RELIANCE | 1333 | NSE_EQ |
| INFY | 1594 | NSE_EQ |
| TCS | 11536 | NSE_EQ |

Security IDs are Dhan-internal numeric IDs (not ISIN). The full instrument master can be downloaded from Dhan's data portal.

---

## Sources

- [Full Market Depth — DhanHQ v2 Docs](https://dhanhq.co/docs/v2/full-market-depth/)
- [Market Quote — DhanHQ v2 Docs](https://dhanhq.co/docs/v2/market-quote/)
- [Live Market Feed — DhanHQ v2 Docs](https://dhanhq.co/docs/v2/live-market-feed/)
- [DhanHQ Python Client — GitHub](https://github.com/dhan-oss/DhanHQ-py)
- [How Data API Subscription Works — Dhan Support](https://dhan.co/support/platforms/dhanhq-api/how-does-the-dhanhq-data-api-subscription-work/)
- [API Rate Limits — Dhan Support](https://dhan.co/support/platforms/dhanhq-api/what-are-the-api-rate-limits-for-dhan/)
- [20-Depth Announcement — MadeForTrade](https://madefortrade.in/t/introducing-20-depth-market-data-on-dhanhq-data-apis/40679)
- [dhan-20depth reference implementation — GitHub](https://github.com/marketcalls/dhan-20depth)
