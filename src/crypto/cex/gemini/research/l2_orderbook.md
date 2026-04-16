# Gemini Exchange — L2 / Order Book API Capabilities

> Research date: 2026-04-16
> Sources: docs.gemini.com (official), asyncapi/spec examples

---

## 1. API Overview

Gemini offers **three distinct WebSocket APIs**, all relevant to L2 data:

| API | Endpoint | Status |
|-----|----------|--------|
| Market Data v1 | `wss://api.gemini.com/v1/marketdata/{symbol}` | Legacy, still active |
| Market Data v2 | `wss://api.gemini.com/v2/marketdata` | Current, multi-symbol |
| Fast API | `wss://ws.gemini.com` | New, recommended |

---

## 2. WebSocket Channels

### 2.1 Market Data v1 — Single-Symbol Stream

**Connection:**
```
wss://api.gemini.com/v1/marketdata/{symbol}?heartbeat=true&trades=true&bids=true&asks=true&top_of_book=false
```

**Query Parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `heartbeat` | bool | false | Enable heartbeat messages |
| `trades` | bool | true | Include trade events |
| `bids` | bool | true | Include bid changes |
| `asks` | bool | true | Include ask changes |
| `top_of_book` | bool | false | If true, only best bid/ask |

**Message Types:**

`type: "update"` — Order book or trade event batch

Fields:
- `type` (string): `"update"`
- `eventId` (integer): Monotonically increasing, persistent across reconnects
- `timestamp` (string): Seconds precision
- `timestampms` (string): Milliseconds precision
- `socket_sequence` (integer): Zero-indexed per-connection counter
- `events` (array): List of event objects

**Event object fields** (inside `events` array):
- `type`: `"change"` (orderbook), `"trade"`, `"auction"`, `"block_trade"`
- `price` (number): Price level
- `side`: `"bid"` or `"ask"`
- `reason`: `"place"`, `"trade"`, `"cancel"`, `"initial"` — `"initial"` marks snapshot events
- `remaining` (number): Quantity remaining at this price level after change
- `delta` (number): Quantity changed (negative = removal)

**Heartbeat message:**
```json
{
  "type": "heartbeat",
  "socket_sequence": 30
}
```

**Snapshot behavior:** First update message contains events with `reason: "initial"` — full state of the book. Subsequent messages are incremental deltas.

**Depth:** Full order book — no configurable limit. `top_of_book=true` restricts to best bid/ask only.

---

### 2.2 Market Data v2 — Multi-Symbol Stream

**Connection:**
```
wss://api.gemini.com/v2/marketdata
```

**Subscribe message (sent after connect):**
```json
{
  "type": "subscribe",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["BTCUSD", "ETHUSD", "ETHBTC"]
    }
  ]
}
```

**Available subscription names:**

| Name | Description |
|------|-------------|
| `l2` | Level 2 order book + trades |
| `candles_1m`, `candles_5m`, `candles_15m`, `candles_30m`, `candles_1h`, `candles_6h`, `candles_1d` | OHLCV candles |
| `mark_price` | Mark price for perpetuals |
| `funding_amount` | Funding amount |

**L2 update message (`l2_updates`):**
```json
{
  "type": "l2_updates",
  "symbol": "BTCUSD",
  "changes": [
    ["buy", "9122.04", "0.00121425"],
    ["sell", "9122.07", "0.98942292"]
  ],
  "trades": [
    {
      "type": "trade",
      "symbol": "BTCUSD",
      "eventid": 169841458,
      "timestamp": 1560976400428,
      "price": "9122.04",
      "quantity": "0.0073173",
      "side": "sell",
      "tid": 2840140800042677
    }
  ]
}
```

**Fields:**
- `type` (string): `"l2_updates"`
- `symbol` (string): Trading pair
- `changes` (array of arrays): `[side, price, quantity]` — `quantity = "0"` means price level removed
- `trades` (array): Present only in initial snapshot (last 50 trades), empty in deltas

**Snapshot behavior:** First `l2_updates` message = full book snapshot + last 50 trades. All subsequent messages = incremental changes only.

**Depth:** Full order book — not configurable. No `limit_bids` / `limit_asks` equivalent on WS v2.

**Sequence numbers:** NOT present in v2 l2_updates messages. No `socket_sequence` on this feed.

**Update speed:** Not documented (no configurable interval).

**Multi-symbol:** Yes — subscribe to multiple symbols in a single connection. Max symbols per connection: not documented.

---

### 2.3 Fast API — Next-Generation (Recommended)

**Connection:**
```
wss://ws.gemini.com[?snapshot=N&cancelOnDisconnect=true]
```

**Connection parameters:**

| Parameter | Values | Default | Description |
|-----------|--------|---------|-------------|
| `snapshot` | `-1`, `0`, positive int | `0` | `-1` = full book on connect; `N` = top N levels; `0` = no snapshot |
| `cancelOnDisconnect` | `true`/`false` | false | Cancel all open orders on disconnect |

**Performance tiers:**
- Tier 2 (Public Internet / AWS us-east-1): ~15ms p99
- Tier 1 (In-region direct): ~10ms p99
- Tier 0 (Local Zone): ~5ms p99

#### 2.3.1 L2 Partial Depth Streams

**Subscription format:** `{symbol}@depth{level}[@interval]`

| Stream | Depth | Update Interval |
|--------|-------|-----------------|
| `BTCUSD@depth5` | Top 5 levels | 1 second |
| `BTCUSD@depth10` | Top 10 levels | 1 second |
| `BTCUSD@depth20` | Top 20 levels | 1 second |
| `BTCUSD@depth5@100ms` | Top 5 levels | 100 ms |
| `BTCUSD@depth10@100ms` | Top 10 levels | 100 ms |
| `BTCUSD@depth20@100ms` | Top 20 levels | 100 ms |

**Message format:**
```json
{
  "lastUpdateId": 1234567,
  "bids": [["price", "quantity"], ...],
  "asks": [["price", "quantity"], ...]
}
```

Fields:
- `lastUpdateId` (integer): Sequence identifier for this snapshot
- `bids` (array): Top N bid levels `[price, quantity]`
- `asks` (array): Top N ask levels `[price, quantity]`

Note: Full replacement snapshot each update (not delta).

#### 2.3.2 L2 Differential Depth Streams

**Subscription format:** `{symbol}@depth[@interval]`

| Stream | Update Interval |
|--------|-----------------|
| `BTCUSD@depth` | 1 second |
| `BTCUSD@depth@100ms` | 100 ms |

**Message format:**
```json
{
  "e": "depthUpdate",
  "E": 1672531200000000000,
  "s": "BTCUSD",
  "U": 100,
  "u": 105,
  "b": [["price", "quantity"], ...],
  "a": [["price", "quantity"], ...]
}
```

Fields:
- `e` (string): Event type `"depthUpdate"`
- `E` (integer): Event time in **nanoseconds**
- `s` (string): Symbol
- `U` (integer): First update ID in this event
- `u` (integer): Last update ID in this event
- `b` (array): Bid changes `[price, quantity]`
- `a` (array): Ask changes `[price, quantity]`

**Quantity = `"0"` means price level removed.**

**Snapshot:** Use `snapshot=-1` connection parameter to receive full book on subscribe. Without snapshot, must reconstruct from scratch.

#### 2.3.3 Book Ticker

Streams best bid/ask only (not full L2). Subscription format: `{symbol}@bookTicker`

---

## 3. REST Order Book Endpoint

**Endpoint:**
```
GET https://api.gemini.com/v1/book/{symbol}
```

**Path parameters:**
- `symbol` (required): Trading pair (e.g., `BTCUSD`)

**Query parameters:**

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `limit_bids` | integer | 50 | Number of bid price levels. Set to `0` for full book. |
| `limit_asks` | integer | 50 | Number of ask price levels. Set to `0` for full book. |

**Max depth:** No documented hard cap. `limit_bids=0` / `limit_asks=0` returns the full book.

**Response format:**
```json
{
  "bids": [
    {
      "price": "9122.04",
      "amount": "0.50000000",
      "timestamp": "1560976400"
    }
  ],
  "asks": [
    {
      "price": "9125.00",
      "amount": "1.25000000",
      "timestamp": "1560976405"
    }
  ]
}
```

**Field types:** All numeric values returned as **strings** (Gemini's policy to preserve precision — not safe to parse as float).

**Fields per level:**
- `price` (string): Price level
- `amount` (string): Total quantity at this level
- `timestamp` (string): Unix timestamp (seconds) when level was last updated

---

## 4. Update Speed

| API | Update Speed | Configurable? |
|-----|-------------|---------------|
| Market Data v1 | As-fast-as-possible (event-driven) | No |
| Market Data v2 | As-fast-as-possible (event-driven) | No |
| Fast API — Partial Depth | 1s or 100ms | Yes (stream suffix) |
| Fast API — Differential | 1s or 100ms | Yes (stream suffix) |

---

## 5. Price Aggregation

**None.** All APIs return individual price levels — no configurable tick aggregation documented.

---

## 6. Checksum

**Not present** in any Gemini L2 API (v1, v2, or Fast API). No checksum field documented.

---

## 7. Sequence / Ordering

### Market Data v1

| Field | Location | Description |
|-------|----------|-------------|
| `socket_sequence` | Every message (update + heartbeat) | Zero-indexed per-connection counter. Gap = missed message → reconnect. |
| `eventId` | `update` message | Monotonically increasing across reconnects. Indicates event ordering. |

**Gap detection:** If `socket_sequence` is non-contiguous → disconnect and reconnect immediately.

**Heartbeats share sequence:** Heartbeat and update messages use the same `socket_sequence` counter.

### Market Data v2

No sequence numbers documented on `l2_updates` messages.

### Fast API

| Field | Location | Description |
|-------|----------|-------------|
| `lastUpdateId` | Partial depth messages | Snapshot ID |
| `U` | Differential depth messages | First update ID in event |
| `u` | Differential depth messages | Last update ID in event |

---

## 8. Special Notes

### API Version Hierarchy

```
Market Data v1  (legacy, single-symbol, socket_sequence)
      ↓
Market Data v2  (current, multi-symbol, l2 subscription, no sequence on l2)
      ↓
Fast API        (new, recommended, partial/differential depth, nanosecond timestamps)
```

Gemini is actively migrating integrators to Fast API. The archived v2 page states the old API "has been replaced by the new WebSocket API."

### Market Data v2 vs v1 Key Differences

| Feature | v1 | v2 |
|---------|----|----|
| Multi-symbol | No | Yes |
| Snapshot trigger | Implicit (first message) | Implicit (first message) |
| Socket sequence | Yes | No |
| Subscription format | URL params | JSON message |
| Event format | `events[]` array with `reason` | `changes[]` array `[side, price, qty]` |
| Trades in updates | Yes | Initial snapshot only |

### Fast API vs v2 Key Differences

| Feature | v2 | Fast API |
|---------|----|---------|
| Depth limit streams | No | Yes (5/10/20) |
| Configurable update rate | No | Yes (1s / 100ms) |
| Full diff depth | Implicit | Explicit `@depth` stream |
| Nanosecond timestamps | No | Yes |
| Snapshot control | No | Via `snapshot=` param |
| Update IDs (U/u) | No | Yes (diff stream) |

### Gemini Fast API Similarity to Binance

The Fast API stream subscription format (`{symbol}@depth`, `{symbol}@depth5@100ms`, fields `U`/`u`/`b`/`a`) closely mirrors Binance WebSocket Streams. This is intentional — Gemini adopted Binance-compatible stream naming for ease of migration.

---

## Sources

- [Gemini Fast API Stream Reference](https://docs.gemini.com/websocket/fast-api/streams) — last modified Feb 14, 2026
- [Gemini WebSocket Introduction (Fast API)](https://docs.gemini.com/websocket/introduction)
- [Gemini Market Data v2 About](https://docs.gemini.com/websocket/market-data/v2/about)
- [Gemini Market Data v2 Level 2 Data](https://docs.gemini.com/websocket/market-data/v2/level-2-data)
- [Gemini Market Data v2 Archived](https://docs.gemini.com/websocket/archived/v2)
- [Gemini Market Data v1 About](https://docs.gemini.com/websocket/market-data/v1/about)
- [Gemini Order Events / Event Types](https://docs.gemini.com/websocket/order-events/event-types)
- [Gemini REST Market Data](https://docs.gemini.com/rest/market-data)
- [AsyncAPI Spec — Gemini WebSocket Example](https://raw.githubusercontent.com/asyncapi/spec/v2.3.0/examples/websocket-gemini.yml)
