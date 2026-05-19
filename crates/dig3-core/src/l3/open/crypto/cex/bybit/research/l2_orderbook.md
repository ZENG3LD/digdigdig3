# Bybit L2 / Orderbook Capabilities

Researched from official Bybit v5 API documentation.
Date: 2026-04-16

Sources:
- WS: https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook
- REST: https://bybit-exchange.github.io/docs/v5/market/orderbook

---

## A. WebSocket Depth Levels and Update Speeds

### Channel Name Format

```
orderbook.{depth}.{symbol}
```

Examples: `orderbook.1.BTCUSDT`, `orderbook.50.BTCUSDT`, `orderbook.200.BTCUSDT`

### Spot

| Depth | Push Frequency |
|-------|---------------|
| 1     | 10ms           |
| 50    | 20ms           |
| 200   | 100ms          |
| 1000  | 200ms          |

### Linear (USDT/USDC Perpetual)

| Depth | Push Frequency |
|-------|---------------|
| 1     | 10ms           |
| 50    | 20ms           |
| 200   | 100ms          |
| 1000  | 200ms          |

### Inverse

| Depth | Push Frequency |
|-------|---------------|
| 1     | 10ms           |
| 50    | 20ms           |
| 200   | 100ms          |
| 1000  | 200ms          |

### Option

| Depth | Push Frequency |
|-------|---------------|
| 25    | 20ms           |
| 100   | 100ms          |

---

## B. Snapshot vs Delta

### When snapshots are sent

- Upon initial subscription (always a snapshot first)
- When the service restarts (occasionally — indicated by `u = 1`)
- When a new `snapshot` type message is received at any time (must reset local book)
- For Level 1 data specifically: if 3 seconds have elapsed without any orderbook change, a snapshot is re-pushed (with the same `u` value as the previous message)

### How the type is indicated

Field: **`type`** (top-level field in the WebSocket message)

Values:
- `"snapshot"` — full orderbook state, replace local copy
- `"delta"` — incremental update, apply to local copy

### Delta processing rules

| Condition | Action |
|-----------|--------|
| Received price level with size = `"0"` | Delete that price level from local orderbook |
| Received price level not present locally | Insert as new level |
| Received price level already present locally | Update size to new value |

**Note**: `size = 0` means the order at that price is fully filled or cancelled — remove the level entirely.

### Special case: Level 1 (depth=1)

For spot, linear, and inverse `depth=1`: **snapshot messages only**, no delta messages. The stream always sends the full top-of-book state.

---

## C. REST Orderbook

### Endpoint

```
GET /v5/market/orderbook
```

### Parameters

| Parameter | Required | Type   | Description                          |
|-----------|----------|--------|--------------------------------------|
| category  | Yes      | string | `spot`, `linear`, `inverse`, `option` |
| symbol    | Yes      | string | Uppercase symbol, e.g. `BTCUSDT`     |
| limit     | No       | integer| Number of bid/ask levels to return   |

### Valid `limit` values by category

| Category         | Valid Range | Default |
|------------------|-------------|---------|
| spot             | 1–200       | 1       |
| linear (USDT/USDC perpetual) | 1–500 | 25 |
| inverse          | 1–500       | 25      |
| option           | 1–25        | 1       |

### Max depth via REST

The endpoint returns up to the limit specified. Max values per above table (200 for spot, 500 for linear/inverse, 25 for option).

Note: Documentation mentions "1000-level orderbook data" is available for contracts and spot via WebSocket streams, but the REST endpoint caps at the limit values above.

### Response fields

| Field | Type   | Description                                      |
|-------|--------|--------------------------------------------------|
| s     | string | Symbol name                                      |
| b     | array  | Bids: `[price, size]` pairs, sorted descending   |
| a     | array  | Asks: `[price, size]` pairs, sorted ascending    |
| ts    | number | System timestamp (milliseconds)                  |
| u     | number | Update ID (sequential)                           |
| seq   | number | Cross sequence number                            |
| cts   | number | Matching engine timestamp (correlates with trade data) |

RPI (Retail Price Improvement) orders are NOT included in REST responses and are not visible via API.

---

## D. Update Speed

Update speeds are fixed per depth level — not user-configurable.

| Depth | Spot    | Linear  | Inverse | Option  |
|-------|---------|---------|---------|---------|
| 1     | 10ms    | 10ms    | 10ms    | N/A     |
| 25    | N/A     | N/A     | N/A     | 20ms    |
| 50    | 20ms    | 20ms    | 20ms    | N/A     |
| 100   | N/A     | N/A     | N/A     | 100ms   |
| 200   | 100ms   | 100ms   | 100ms   | N/A     |
| 1000  | 200ms   | 200ms   | 200ms   | N/A     |

Deeper levels push less frequently. Speed is not configurable — it is determined by the depth level subscribed to.

---

## E. Price Aggregation

The official Bybit v5 WebSocket documentation does not mention any configurable price grouping or merged/aggregated orderbook channels. No merged orderbook streams are documented. Only raw price-level data is provided.

---

## F. Checksum

The Bybit v5 orderbook WebSocket documentation **does not document a checksum field** in the orderbook stream messages. No checksum algorithm, procedure, or field is mentioned in the v5 public orderbook docs.

(Some older Bybit API versions had checksum; v5 docs reviewed here do not describe it.)

---

## G. Sequence / Ordering

### Fields

| Field | Location        | Description |
|-------|-----------------|-------------|
| `u`   | `data.u`        | Update ID — monotonically increasing per symbol. Value of `1` indicates service restart; local book must be reset. |
| `seq` | `data.seq`      | Cross sequence — comparable across different depth levels. Smaller `seq` = earlier data generation. Used to compare orderbooks at different depth subscriptions. |
| `cts` | top-level       | Matching engine timestamp (ms) — correlates with public trade channel data. |
| `ts`  | top-level       | System timestamp (ms) — when the message was generated by Bybit's system. |

### Detecting missed messages

- Use `u` (Update ID) to detect gaps: if `u` is not sequential with the previous delta, data was missed.
- If a new `snapshot` message is received unexpectedly, reset the local orderbook immediately regardless of sequence.
- If `u = 1` is received, it signals a service restart — reset local orderbook and treat the message as a fresh snapshot.
- The `seq` field allows cross-depth comparison but is not the primary gap-detection mechanism (use `u` for that).

### Ordering within messages

- **Bids (`b`)**: sorted by price descending (best bid first)
- **Asks (`a`)**: sorted by price ascending (best ask first)

### Field ordering note

For spot and futures Level 1 data, the `ts` field appears **before** the `type` field in the JSON. For other futures depth levels, `ts` appears **after** `type`. This is a documentation-noted quirk; handle by field name, not position.

---

## H. Category Differences Summary

| Feature                    | Spot        | Linear      | Inverse     | Option      |
|----------------------------|-------------|-------------|-------------|-------------|
| WS depths available        | 1, 50, 200, 1000 | 1, 50, 200, 1000 | 1, 50, 200, 1000 | 25, 100 |
| Fastest update (depth 1)   | 10ms        | 10ms        | 10ms        | N/A         |
| Depth=1 message type       | snapshot only | snapshot only | snapshot only | N/A      |
| Option-specific depths     | No          | No          | No          | 25, 100     |
| REST limit range           | 1–200       | 1–500       | 1–500       | 1–25        |
| REST default limit         | 1           | 25          | 25          | 1           |
| RPI orders in REST         | Excluded    | Excluded    | Excluded    | Excluded    |
| PreLaunch contracts        | N/A         | No feed until ContinuousTrading phase | N/A | N/A |

Key differences:
- **Option** is unique: only 2 depth levels (25 and 100), no depth=1 channel, much shallower REST max (25 levels).
- **Spot** REST max is 200, while linear/inverse REST max is 500.
- **Linear** and **Inverse** are identical in depth config and update speeds.
- All categories: depth=1 WS is snapshot-only (no deltas).

---

## Sources

- [Bybit V5 WebSocket Public - Orderbook](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook)
- [Bybit V5 Market - Get Orderbook (REST)](https://bybit-exchange.github.io/docs/v5/market/orderbook)
