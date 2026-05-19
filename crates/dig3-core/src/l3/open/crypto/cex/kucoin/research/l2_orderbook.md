# KuCoin L2 Orderbook API Capabilities

Research date: 2026-04-16
Sources: KuCoin official docs (docs-new + legacy docs), GitHub issues

---

## 1. API Architecture Overview

KuCoin exposes **two parallel API generations**:

| Generation | REST Base | WS Base (Spot) | WS Base (Futures) |
|---|---|---|---|
| **Classic** (legacy) | `https://api.kucoin.com` | `wss://ws-api-spot.kucoin.com` | `wss://ws-api-futures.kucoin.com` |
| **Pro** (new, unified) | `https://api.kucoin.com` | `wss://x-push-spot.kucoin.com` | `wss://x-push-futures.kucoin.com` |

Futures REST base is always `https://api-futures.kucoin.com` (separate host).

Both Classic and Pro WebSocket APIs coexist. The Pro API uses a different channel encoding style (`obu` channel with `depth` param vs. `/spotMarket/level2Depth5:{symbol}` style topics in Classic).

---

## 2. WebSocket Channels — Classic API

### 2.1 Spot — Increment (Full Depth Delta)

**Topic:** `/market/level2:{symbol}` (comma-separated for multi-symbol, max 100)

**Subject:** `trade.l2update`

**Type:** Delta only (no snapshots from WS; must fetch REST snapshot first)

**Update speed:** Real-time; pushed only when market changes occur

**Subscription:**
```json
{
  "id": 1545910660739,
  "type": "subscribe",
  "topic": "/market/level2:BTC-USDT",
  "response": true
}
```

**Message format:**
```json
{
  "topic": "/market/level2:BTC-USDT",
  "type": "message",
  "subject": "trade.l2update",
  "data": {
    "sequenceStart": 1545896669105,
    "sequenceEnd":   1545896669106,
    "symbol": "BTC-USDT",
    "time": 1733298926000,
    "changes": {
      "asks": [["18906", "0.00331", "14103845"]],
      "bids": [["18900", "1.50000", "14103844"]]
    }
  }
}
```

**Field definitions:**

| Field | Type | Description |
|---|---|---|
| `sequenceStart` | number | First sequence number in this batch |
| `sequenceEnd` | number | Last sequence number in this batch |
| `symbol` | string | Trading pair |
| `time` | number | Millisecond timestamp |
| `changes.asks` | array | Ask updates: `[price, size, sequence]` |
| `changes.bids` | array | Bid updates: `[price, size, sequence]` |

**Changes array element (3 fields):**

| Index | Value | Notes |
|---|---|---|
| 0 | price (string) | Price level |
| 1 | size (string) | Quantity; `"0"` = remove this level |
| 2 | sequence (string) | Last modification sequence for this price level — NOT used for batch ordering |

**Note on sequence in changes entries:** The third element in each `[price, size, sequence]` triplet is the sequence of the last modification at that specific price level, not a per-message counter. When multiple updates occur at the same price within a batch, only the latest is pushed.

**Gap detection condition:**
```
sequenceStart(new) <= sequenceEnd(old) + 1
AND
sequenceEnd(new) > sequenceEnd(old)
```
If the gap is <= 500 sequences, re-fetch REST snapshot to resync. No documented CRC32 or checksum.

**Calibration procedure (5 steps):**
1. Subscribe to `/market/level2:{symbol}` and begin caching messages
2. Fetch full orderbook snapshot via REST (`GET /api/v3/market/orderbook/level2?symbol=BTC-USDT`)
3. Record the snapshot's `sequence` value
4. Replay cached WS messages; discard any where `sequenceEnd <= snapshot.sequence`
5. Apply remaining updates in order; size=0 means remove price level; price=0 means ignore update but still advance sequence

---

### 2.2 Spot — Level 5 Best Bid/Ask (Snapshot)

**Topic:** `/spotMarket/level2Depth5:{symbol}` (comma-separated, max 100 symbols)

**Subject:** `level2`

**Type:** Full snapshot of top 5; pushed on change only

**Update speed:** 100ms

**Message format:**
```json
{
  "topic": "/spotMarket/level2Depth5:BTC-USDT",
  "type": "message",
  "subject": "level2",
  "data": {
    "asks": [["price", "size"], ...],
    "bids": [["price", "size"], ...],
    "timestamp": 1729822226746
  }
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `asks` | array | Top 5 ask levels `[price, size]` |
| `bids` | array | Top 5 bid levels `[price, size]` |
| `timestamp` | number | Millisecond timestamp |

No sequence number. No checksum. Purely snapshot — no need for gap detection or local orderbook maintenance.

---

### 2.3 Spot — Level 50 Best Bid/Ask (Snapshot)

**Topic:** `/spotMarket/level2Depth50:{symbol}` (comma-separated, max 100 symbols)

**Subject:** `level2`

**Type:** Full snapshot of top 50; pushed on change only

**Update speed:** 100ms

**Message format:** Same structure as Level 5 but with up to 50 levels.

```json
{
  "topic": "/spotMarket/level2Depth50:BTC-USDT",
  "type": "message",
  "subject": "level2",
  "data": {
    "asks": [["price", "size"], ...],
    "bids": [["price", "size"], ...],
    "timestamp": 1729822226746
  }
}
```

No sequence number. No checksum. Same snapshot semantics as Level 5.

---

### 2.4 Futures — Increment (Full Depth Delta)

**Topic:** `/contractMarket/level2:{symbol}`

**Subject:** `level2`

**Type:** Delta only (all depth, incremental)

**Update speed:** Real-time; pushed only when market changes occur

**Subscription:**
```json
{
  "id": 1545910660739,
  "type": "subscribe",
  "topic": "/contractMarket/level2:XBTUSDTM",
  "response": true
}
```

**Message format:**
```json
{
  "topic": "/contractMarket/level2:XBTUSDTM",
  "type": "message",
  "subject": "level2",
  "sn": 1709400450243,
  "data": {
    "sequence": 1709400450243,
    "change": "90631.2,sell,2",
    "timestamp": 1731897467182
  }
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `sn` | number | Message-level sequence number |
| `data.sequence` | number | Order book sequence number (same as `sn`) |
| `data.change` | string | `"price,side,quantity"` — comma-delimited string |
| `data.timestamp` | number | Millisecond timestamp |

**Change string format:** `"price,side,quantity"`
- `side`: `"buy"` or `"sell"`
- `quantity = 0`: remove price level from book

**IMPORTANT difference from Spot:** Futures uses a single `change` (string) per message — NOT a `changes` object with `asks`/`bids` arrays. Each WS message contains exactly one price level update.

**Gap detection:** Monitor `sequence` continuity. Max allowed gap before re-fetch: 500.

**Calibration procedure (6 steps):**
1. Cache incoming WS delta messages
2. Fetch snapshot via REST (`GET https://api-futures.kucoin.com/api/v1/level2/snapshot?symbol=XBTUSDM`)
3. Replay cached messages sequentially
4. Discard messages where `sequence <= snapshot.sequence`
5. Apply size updates; `quantity = 0` removes the price level
6. If gap > 500, re-fetch REST snapshot

No checksum in futures L2 increment either.

---

### 2.5 Futures — Level 5 Best Bid/Ask (Snapshot)

**Topic:** `/contractMarket/level2Depth5:{symbol}`

**Subject:** `level2`

**Type:** Full snapshot of top 5; pushed on change only

**Update speed:** 100ms

**Message format:**
```json
{
  "topic": "/contractMarket/level2Depth5:XBTUSDTM",
  "type": "message",
  "subject": "level2",
  "sn": 1709400450243,
  "data": {
    "bids": [[price, size], ...],
    "asks": [[price, size], ...],
    "sequence": 1709400450243,
    "timestamp": 1731897467182,
    "ts": 1731897467182
  }
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `sn` | number | Message sequence number |
| `data.sequence` | number | Orderbook sequence |
| `data.bids` | array | Top 5 bid levels `[price, size]` |
| `data.asks` | array | Top 5 ask levels `[price, size]` |
| `data.timestamp` | number | Millisecond timestamp |
| `data.ts` | number | Millisecond timestamp (duplicate) |

Note: Futures Level 5 includes `sequence` and `sn`; Spot Level 5 has only `timestamp` (no sequence).

---

### 2.6 Futures — Level 50 Best Bid/Ask (Snapshot)

**Topic:** `/contractMarket/level2Depth50:{symbol}`

**Subject:** `level2`

**Type:** Full snapshot of top 50; pushed on change only

**Update speed:** 100ms

Same message structure as Futures Level 5 but with up to 50 levels.

---

## 3. WebSocket Channels — Pro API (New)

The Pro API uses a unified `obu` (orderbook update) channel at `wss://x-push-spot.kucoin.com` and `wss://x-push-futures.kucoin.com`.

**Subscription:**
```json
{
  "channel": "obu",
  "tradeType": "SPOT",
  "symbol": "BTC-USDT",
  "depth": "5",
  "rpiFilter": 0
}
```

**`depth` values:** `"1"` (BBO), `"5"`, `"50"`, `"increment"`

**`rpiFilter`:** `0` = NoneRPI only; `1` = NoneRPI + RPI (futures only)

**Update speeds by depth:**

| Depth | Spot | Futures |
|---|---|---|
| `"1"` (BBO) | Real-time | Real-time |
| `"5"` | 100ms | 100ms |
| `"50"` | 100ms | 100ms |
| `"increment"` | Real-time | Real-time |

**Message format:**
```json
{
  "T": "obu.SPOT",
  "dp": "5",
  "t": "snapshot",
  "P": 1729822226746123456,
  "d": {
    "s": "BTC-USDT",
    "O": 14610502970,
    "C": 14610502970,
    "M": 1729822226746123,
    "a": [["price", "size"], ...],
    "b": [["price", "size"], ...]
  }
}
```

**Top-level fields:**

| Field | Type | Description |
|---|---|---|
| `T` | string | Message type (`"obu.SPOT"` or `"obu.FUTURES"`) |
| `dp` | string | Depth level requested |
| `t` | string | `"snapshot"` or `"delta"` |
| `P` | number | Nanosecond timestamp |

**Data (`d`) fields:**

| Field | Type | Description |
|---|---|---|
| `s` | string | Symbol |
| `O` | number | Opening sequence number of batch |
| `C` | number | Closing sequence number of batch |
| `M` | number | Microsecond timestamp |
| `a` | array | Ask levels `[price, size]` (or `[price, size, rpi]` if rpiFilter=1) |
| `b` | array | Bid levels `[price, size]` |

**Snapshot vs Delta in Pro API:**
- For depth `"1"`, `"5"`, `"50"`: `t = "snapshot"`, and `O == C` (single point in time)
- For depth `"increment"`: `t = "delta"` for updates, `t = "snapshot"` for initial; `O` may differ from `C`

**Pro API gap detection:**
```
sequenceStart(new) <= sequenceEnd(old) + 1
AND
sequenceEnd(new) > sequenceEnd(old)
```

---

## 4. REST Orderbook Endpoints

### 4.1 Spot — Part OrderBook (No Auth)

**Classic API endpoint:**
- `GET /api/v1/market/orderbook/level2_20` — returns 20 levels each side
- `GET /api/v1/market/orderbook/level2_100` — returns 100 levels each side

**Pro API endpoint:**
- `GET /api/v3/market/orderbook/level2?symbol=BTC-USDT&size=20` (size: 20 or 100)

**Parameters:**

| Param | Required | Values | Notes |
|---|---|---|---|
| `symbol` | Yes | e.g. `BTC-USDT` | Query param |
| `size` / path suffix | No default in path | `20`, `100` | Classic: path; Pro: query |

**Response:**
```json
{
  "code": "200000",
  "data": {
    "time": 1729176273859,
    "sequence": "14610502970",
    "bids": [["price", "size"], ...],
    "asks": [["price", "size"], ...]
  }
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `time` | number | Millisecond timestamp |
| `sequence` | string | Orderbook sequence number (string) |
| `bids` | array | Bid levels `[price, size]` |
| `asks` | array | Ask levels `[price, size]` |

**Auth:** Not required (public endpoint). Recommended over full orderbook for lighter traffic.

---

### 4.2 Spot — Full OrderBook (Auth Required)

**Endpoint:** `GET /api/v3/market/orderbook/level2?symbol=BTC-USDT`

**Parameters:**

| Param | Required | Description |
|---|---|---|
| `symbol` | Yes | Trading pair |

**Response:** Same structure as Part OrderBook but with all price levels.

**Auth:** Required (API key with General permission or higher).

**Rate limiting:** Strict; uses more server resources. Not recommended for continuous polling.

---

### 4.3 Futures — Full OrderBook (Snapshot)

**Endpoint:** `GET https://api-futures.kucoin.com/api/v1/level2/snapshot`

**Parameters:**

| Param | Required | Description |
|---|---|---|
| `symbol` | Yes | e.g. `XBTUSDM` |

**Response:**
```json
{
  "code": "200000",
  "data": {
    "sequence": 1697895963339,
    "symbol": "XBTUSDM",
    "bids": [[66968, 2], [66964.8, 25596]],
    "asks": [[66968.1, 13501], [66968.7, 2032]],
    "ts": 1729168101216000000
  }
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `sequence` | number | Orderbook sequence number (numeric, not string) |
| `symbol` | string | Trading pair |
| `bids` | array | Bid levels `[price, size]` (numeric, not string) |
| `asks` | array | Ask levels `[price, size]` (numeric, not string) |
| `ts` | number | Timestamp in **nanoseconds** |

**Auth:** Not required (public endpoint).

**Rate Limit Weight:** 3

**Note:** Futures prices/sizes are **numbers** (not strings), unlike Spot which uses strings.

---

### 4.4 Futures — Part OrderBook

**Endpoint:** `GET https://api-futures.kucoin.com/api/v1/level2/depth20` or `GET https://api-futures.kucoin.com/api/v1/level2/depth100`

Depth levels available: **20** and **100**.

**Auth:** Not required (public).

---

## 5. Update Speed Summary

| Channel | Speed | Configurable? |
|---|---|---|
| Spot WS Increment (`/market/level2`) | Real-time | No |
| Spot WS Depth5 (`/spotMarket/level2Depth5`) | 100ms | No |
| Spot WS Depth50 (`/spotMarket/level2Depth50`) | 100ms | No |
| Futures WS Increment (`/contractMarket/level2`) | Real-time | No |
| Futures WS Depth5 (`/contractMarket/level2Depth5`) | 100ms | No |
| Futures WS Depth50 (`/contractMarket/level2Depth50`) | 100ms | No |
| Pro WS BBO (`depth="1"`) | Real-time | No |
| Pro WS Depth5/50 | 100ms | No |
| Pro WS Increment | Real-time | No |

**Update speed is not configurable.** No 10ms, 50ms, or 250ms options.

---

## 6. Price Aggregation

**None documented.** KuCoin L2 orderbook data is aggregated by price (same price level merged into one entry), but there is no user-configurable tick size or price grouping parameter. No separate "aggregated" vs "raw" depth distinction is exposed via API parameters.

---

## 7. Checksum

**Not provided by KuCoin.** Neither Spot nor Futures L2 channels include a CRC32 or any other checksum field. There is no orderbook integrity verification mechanism documented in the official API. Integrity must be ensured via sequence number gap detection and REST snapshot re-synchronization.

---

## 8. Sequence Number Handling

### Spot Classic WS (Increment)

- `sequenceStart` / `sequenceEnd` per WS message batch
- Each entry in `changes.asks`/`changes.bids` has a third element (the sequence of last modification at that price) — this is NOT for ordering the batch
- Continuity check: `sequenceStart(new) <= sequenceEnd(old) + 1` AND `sequenceEnd(new) > sequenceEnd(old)`
- If gap detected: re-fetch REST snapshot

### Futures Classic WS (Increment)

- Single `sequence` number per message (also exposed as `sn` at message top level)
- Sequential; monotonically increasing
- Gap tolerance: 500 (documented)
- If gap > 500: re-fetch REST snapshot

### Pro WS (All depths)

- `O` (opening sequence) and `C` (closing sequence) per message
- For snapshots: `O == C`
- For deltas: `O` may < `C` (batch of updates)
- Same continuity check as Classic Spot

### REST Snapshot sequences

| Endpoint | Sequence type |
|---|---|
| Spot Part/Full | `string` (e.g. `"14610502970"`) |
| Futures Full | `number` (e.g. `1697895963339`) |

---

## 9. Spot vs Futures Differences

| Aspect | Spot (Classic) | Futures (Classic) |
|---|---|---|
| Increment WS topic | `/market/level2:{symbol}` | `/contractMarket/level2:{symbol}` |
| Increment format | `changes: { asks: [[p,s,seq],...], bids: [...] }` | `change: "price,side,qty"` (single string) |
| Updates per message | Multiple (batch) | Single price level |
| Depth5 topic | `/spotMarket/level2Depth5:{symbol}` | `/contractMarket/level2Depth5:{symbol}` |
| Depth50 topic | `/spotMarket/level2Depth50:{symbol}` | `/contractMarket/level2Depth50:{symbol}` |
| Depth5 sequence | Not included | `sequence` + `sn` fields |
| REST snapshot host | `api.kucoin.com` | `api-futures.kucoin.com` |
| REST snapshot timestamp | `time` (milliseconds) | `ts` (nanoseconds) |
| REST prices/sizes type | strings | numbers |
| REST sequence type | string | number |
| REST full endpoint path | `/api/v3/market/orderbook/level2` | `/api/v1/level2/snapshot` |
| REST part endpoint | `/api/v1/market/orderbook/level2_20` or `level2_100` | `/api/v1/level2/depth20` or `depth100` |
| REST full auth required | Yes | No |
| Max WS symbols per sub | 100 | Not specified (likely same) |

---

## 10. Channel Summary Table

| Channel | Topic | Depth | Type | Speed | Auth |
|---|---|---|---|---|---|
| Spot Increment | `/market/level2:{sym}` | Full | Delta | Real-time | No |
| Spot Depth5 | `/spotMarket/level2Depth5:{sym}` | 5 | Snapshot | 100ms | No |
| Spot Depth50 | `/spotMarket/level2Depth50:{sym}` | 50 | Snapshot | 100ms | No |
| Futures Increment | `/contractMarket/level2:{sym}` | Full | Delta | Real-time | No |
| Futures Depth5 | `/contractMarket/level2Depth5:{sym}` | 5 | Snapshot | 100ms | No |
| Futures Depth50 | `/contractMarket/level2Depth50:{sym}` | 50 | Snapshot | 100ms | No |
| Pro Spot/Futures `obu` | channel=`obu`, depth=1/5/50/increment | 1/5/50/full | Snap/Delta | Real-time or 100ms | No |

---

## Sources

- [KuCoin Pro WS Orderbook channel (obu)](https://www.kucoin.com/docs-new/3470221w0)
- [KuCoin Classic Spot WS Level2Depth5 endpoint](https://www.kucoin.com/docs-new/3470069w0)
- [KuCoin Classic Futures WS Level2Depth5 endpoint](https://www.kucoin.com/docs-new/3470083w0)
- [KuCoin Classic Spot WS Level2 Increment endpoint](https://www.kucoin.com/docs-new/3470068w0)
- [KuCoin Classic Futures WS Level2 Increment endpoint](https://www.kucoin.com/docs/websocket/futures-trading/public-channels/level2-market-data)
- [KuCoin Spot WS Level2 Market Data (legacy docs)](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/level2-market-data)
- [KuCoin Spot REST Get Part OrderBook](https://www.kucoin.com/docs-new/rest/spot-trading/market-data/get-part-orderbook)
- [KuCoin Spot REST Get Full OrderBook](https://www.kucoin.com/docs-new/rest/spot-trading/market-data/get-full-orderbook)
- [KuCoin Futures REST Get Full OrderBook](https://www.kucoin.com/docs-new/rest/futures-trading/market-data/get-full-orderbook)
- [KuCoin Futures REST Get Part OrderBook Level2](https://www.kucoin.com/docs/rest/futures-trading/market-data/get-part-order-book-level-2)
- [KuCoin GitHub L2 calibration issue #305](https://github.com/Kucoin/kucoin-api-docs/issues/305)
