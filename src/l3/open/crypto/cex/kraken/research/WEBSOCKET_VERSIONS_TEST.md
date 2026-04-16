# Kraken WebSocket API Versions Test Results

**Test Date:** 2026-01-21
**Purpose:** Find which Kraken WebSocket API version actually works and returns real-time data

## Executive Summary

**RESULT:** Both v1 and v2 APIs work, but with different protocols and symbol formats.

- **v1 API** (`wss://ws.kraken.com`) - WORKS with both `XBT/USD` and `BTC/USD` symbols
- **v2 API** (`wss://ws.kraken.com/v2`) - WORKS with `BTC/USD` only (XBT/USD rejected)

## Available WebSocket Endpoints

### Production Endpoints

| Version | Type | URL |
|---------|------|-----|
| v1 | Public | `wss://ws.kraken.com` |
| v1 | Private (Auth) | `wss://ws-auth.kraken.com` |
| v2 | Public | `wss://ws.kraken.com/v2` |
| v2 | Private (Auth) | `wss://ws-auth.kraken.com/v2` |

### Beta Endpoints

| Version | Type | URL |
|---------|------|-----|
| v1 | Public | `wss://beta-ws.kraken.com` |
| v1 | Private (Auth) | `wss://beta-ws-auth.kraken.com` |
| v2 | Public | `wss://beta-ws.kraken.com/v2` |
| v2 | Private (Auth) | `wss://beta-ws-auth.kraken.com/v2` |

## Test Results

### Test 1: Kraken WebSocket v1 - XBT/USD

**URL:** `wss://ws.kraken.com`

**Subscribe Message:**
```json
{
  "event": "subscribe",
  "pair": ["XBT/USD"],
  "subscription": {
    "name": "ticker"
  }
}
```

**Result:** ✅ **WORKING**

**Response Examples:**
```json
// System status
{"event":"systemStatus","version":"1.9.6","status":"online","connectionID":12014973084084585052}

// Subscription confirmation
{
  "channelID": 119930888,
  "channelName": "ticker",
  "event": "subscriptionStatus",
  "pair": "XBT/USD",
  "status": "subscribed",
  "subscription": {"name": "ticker"}
}

// Ticker data (array format)
[
  119930888,
  {
    "a": ["88969.50000", 2, "2.87234330"],
    "b": ["88969.40000", 0, "0.10056199"],
    "c": ["88974.50000", "0.00300000"],
    "v": ["0.00000000", "2728.48755483"],
    "p": ["0.00000", "89909.40020"],
    "t": [0, 83774],
    "l": ["..."]
  }
]

// Heartbeat
{"event":"heartbeat"}
```

**Messages Received:** 16 messages in 10 seconds (including ticker updates and heartbeats)

**Ticker Update Frequency:** Real-time updates on every trade

---

### Test 2: Kraken WebSocket v1 - BTC/USD

**URL:** `wss://ws.kraken.com`

**Subscribe Message:**
```json
{
  "event": "subscribe",
  "pair": ["BTC/USD"],
  "subscription": {
    "name": "ticker"
  }
}
```

**Result:** ✅ **WORKING**

**Notes:**
- v1 accepts both `BTC/USD` and `XBT/USD` and maps them to the same pair
- Response still shows "pair": "XBT/USD" even when subscribing with "BTC/USD"
- Same channel ID (119930888) for both symbols
- Identical data stream

**Messages Received:** 15 messages in 10 seconds

---

### Test 3: Kraken WebSocket v2 - BTC/USD

**URL:** `wss://ws.kraken.com/v2`

**Subscribe Message:**
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD"]
  }
}
```

**Result:** ✅ **WORKING**

**Response Examples:**
```json
// Status message
{
  "channel": "status",
  "type": "update",
  "data": [{
    "version": "2.0.10",
    "system": "online",
    "api_version": "v2",
    "connection_id": 8488217682369066242
  }]
}

// Subscription confirmation
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "event_trigger": "trades",
    "snapshot": true,
    "symbol": "BTC/USD"
  },
  "success": true,
  "time_in": "2026-01-21T01:32:51.439219Z",
  "time_out": "2026-01-21T01:32:51.439244Z"
}

// Snapshot data
{
  "channel": "ticker",
  "type": "snapshot",
  "data": [{
    "symbol": "BTC/USD",
    "bid": 88974.1,
    "bid_qty": 0.06970000,
    "ask": 88974.2,
    "ask_qty": 0.27020276,
    "last": 88949.6,
    "volume": 2728.96214095,
    "vwap": 89909.2,
    "low": 87784.4,
    "high": 91960.0,
    "change": 1162.1,
    "change_pct": 1.32
  }]
}

// Update data
{
  "channel": "ticker",
  "type": "update",
  "data": [{
    "symbol": "BTC/USD",
    "bid": 88974.1,
    "bid_qty": 0.20857076,
    "ask": 88974.2,
    "ask_qty": 0.00413477,
    "last": 88974.2,
    "volume": 2728.96382817,
    "vwap": 89909.2,
    "low": 87784.4,
    "high": 91960.0,
    "change": 1166.7,
    "change_pct": 1.33
  }]
}

// Heartbeat
{"channel":"heartbeat"}
```

**Messages Received:** 15 messages in 10 seconds

**Ticker Update Frequency:** Real-time updates on every trade

**Key Features:**
- JSON objects instead of arrays
- Numbers as actual JSON numbers (not strings)
- RFC3339 timestamps
- More readable field names
- Snapshot + update pattern

---

### Test 4: Kraken WebSocket v2 - XBT/USD

**URL:** `wss://ws.kraken.com/v2`

**Subscribe Message:**
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["XBT/USD"]
  }
}
```

**Result:** ❌ **FAILED**

**Error Response:**
```json
{
  "error": "Currency pair not supported XBT/USD",
  "method": "subscribe",
  "success": false,
  "symbol": "XBT/USD",
  "time_in": "2026-01-21T01:33:03.979334Z",
  "time_out": "2026-01-21T01:33:03.979359Z"
}
```

**Notes:**
- v2 API does NOT support XBT/USD symbol format
- v2 uses BTC instead of XBT for Bitcoin
- This is documented as part of v2's improvements to use more readable symbols

---

## API Version Comparison

### v1 vs v2 Key Differences

| Feature | v1 | v2 |
|---------|----|----|
| **Endpoint** | `wss://ws.kraken.com` | `wss://ws.kraken.com/v2` |
| **Protocol** | Event-based (legacy) | Method-based (FIX-like) |
| **Subscribe Event** | `"event": "subscribe"` | `"method": "subscribe"` |
| **Symbol Format** | `XBT/USD` or `BTC/USD` | `BTC/USD` only |
| **Data Format** | Arrays (channel ID + data) | JSON objects |
| **Numbers** | Strings | Native JSON numbers |
| **Timestamps** | Unix timestamps | RFC3339 strings |
| **Field Names** | Short (a, b, c, v, p) | Descriptive (bid, ask, volume) |
| **Snapshot** | First update | Explicit snapshot message |
| **Version** | 1.9.6 | 2.0.10 |

### v1 Protocol Structure

**Subscribe:**
```json
{
  "event": "subscribe",
  "pair": ["XBT/USD"],
  "subscription": {
    "name": "ticker"
  }
}
```

**Ticker Update:**
```json
[
  channelID,
  {
    "a": ["price", wholeLotVolume, "volume"],  // ask
    "b": ["price", wholeLotVolume, "volume"],  // bid
    "c": ["price", "volume"],                   // close
    "v": ["today", "last24h"],                  // volume
    "p": ["today", "last24h"],                  // vwap
    "t": [today, last24h],                      // trades
    "l": ["today", "last24h"],                  // low
    "h": ["today", "last24h"],                  // high
    "o": ["today", "last24h"]                   // open
  },
  "channelName",
  "pair"
]
```

### v2 Protocol Structure

**Subscribe:**
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD"]
  }
}
```

**Ticker Snapshot:**
```json
{
  "channel": "ticker",
  "type": "snapshot",
  "data": [{
    "symbol": "BTC/USD",
    "bid": 88974.1,
    "bid_qty": 0.06970000,
    "ask": 88974.2,
    "ask_qty": 0.27020276,
    "last": 88949.6,
    "volume": 2728.96214095,
    "vwap": 89909.2,
    "low": 87784.4,
    "high": 91960.0,
    "change": 1162.1,
    "change_pct": 1.32
  }]
}
```

**Ticker Update:**
```json
{
  "channel": "ticker",
  "type": "update",
  "data": [{
    "symbol": "BTC/USD",
    "bid": 88974.1,
    "bid_qty": 0.20857076,
    "ask": 88974.2,
    "ask_qty": 0.00413477,
    "last": 88974.2,
    "volume": 2728.96382817,
    "vwap": 89909.2,
    "low": 87784.4,
    "high": 91960.0,
    "change": 1166.7,
    "change_pct": 1.33
  }]
}
```

## Symbol Format Reference

### Bitcoin Symbol Variations

| API | Accepted Format | Notes |
|-----|----------------|-------|
| v1 | `XBT/USD` | Recommended |
| v1 | `BTC/USD` | Maps to XBT/USD |
| v2 | `BTC/USD` | Only format |
| v2 | `XBT/USD` | ❌ Rejected |

### Symbol Discovery

**For v1:**
- Use REST API endpoint `/0/public/AssetPairs`
- Field `wsname` provides the WebSocket symbol format
- Example: `XXBTZUSD` REST symbol → `XBT/USD` WebSocket symbol

**For v2:**
- Use WebSocket `instrument` channel
- Subscribe to get list of tradable instruments
- Symbols already in readable format (e.g., `BTC/USD`)

## Recommendations

### For New Implementations

**Use v2 API** for the following reasons:

1. **Better Developer Experience:**
   - Readable field names
   - Native JSON numbers (no string parsing)
   - Proper timestamps (RFC3339)
   - Clear snapshot/update distinction

2. **Industry Standard:**
   - FIX-like protocol design
   - Familiar to financial developers
   - Better documentation

3. **Future-Proof:**
   - Active development continues on v2
   - v1 is in maintenance mode
   - New features only in v2

### v2 Implementation Checklist

- [ ] Use `wss://ws.kraken.com/v2` endpoint
- [ ] Use `BTC/USD` symbol format (not XBT)
- [ ] Subscribe with `method: "subscribe"` format
- [ ] Handle both `snapshot` and `update` message types
- [ ] Parse numbers as JSON numbers (not strings)
- [ ] Use `channel` field to route messages
- [ ] Implement heartbeat monitoring
- [ ] Handle error responses with `success: false`

### v1 Fallback Support

If v2 is not working or you need v1 compatibility:

- [ ] Use `wss://ws.kraken.com` endpoint
- [ ] Use `XBT/USD` symbol format
- [ ] Subscribe with `event: "subscribe"` format
- [ ] Handle array-based ticker updates
- [ ] Parse all numbers from strings
- [ ] Use channel ID for message routing
- [ ] Map short field names (a=ask, b=bid, etc.)

## Connection Requirements

Both v1 and v2 require:

- **TLS with SNI (Server Name Indication)**
- **JSON message encoding**
- **Reconnect limit:** No faster than once every 5 seconds
- **Heartbeat monitoring:** Server sends periodic heartbeats
- **Keep-alive:** Authenticated connections need at least one private subscription active

## Test Environment

**Test Script:** `test_kraken_ws.py`

**Dependencies:**
- Python 3.12
- `websockets` library
- `asyncio` for async operations

**Test Duration:** 10 seconds per configuration

**Test Date:** 2026-01-21 06:32 UTC+5

## Sources

- [Kraken WebSocket API FAQ](https://support.kraken.com/articles/360022326871-kraken-websocket-api-frequently-asked-questions)
- [Kraken API Center](https://docs.kraken.com/)
- [Kraken WebSockets v1 Documentation](https://docs.kraken.com/websockets/)
- [Kraken WebSockets v2 Reference](https://docs.kraken.com/websockets-v2/)
- [Spot WebSockets Introduction](https://docs.kraken.com/api/docs/guides/spot-ws-intro/)
- [WebSockets v2 API Blog](https://docs.kraken.com/api/blog/ws-v2/)
- [Ticker v1 Documentation](https://docs.kraken.com/api/docs/websocket-v1/ticker/)
- [Ticker v2 Documentation](https://docs.kraken.com/api/docs/websocket-v2/ticker/)

## Conclusion

**BOTH v1 and v2 APIs WORK and deliver real-time data.**

The initial problem with v2 "ignoring subscriptions" was likely due to:
1. Using wrong symbol format (`XBT/USD` instead of `BTC/USD`)
2. Not waiting for the subscription confirmation
3. Not handling the snapshot message type

**Recommended approach:** Use v2 API with `BTC/USD` symbol format for best developer experience and future compatibility.
