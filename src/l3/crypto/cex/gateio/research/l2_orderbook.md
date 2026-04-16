# Gate.io L2 / Order Book API Capabilities

> Source: Gate.io API v4 official documentation (gate.com/docs)
> Researched: 2026-04-16

---

## 1. WebSocket Channels — Spot

Gate.io provides **four** spot order book WebSocket channels, accessible at `wss://api.gateio.ws/ws/v4/`.

### 1.1 `spot.book_ticker` — Best Bid/Ask

| Property       | Value               |
|----------------|---------------------|
| Update speed   | 10 ms               |
| Type           | Real-time best price only |
| Authentication | Not required        |

**Subscription payload:** `["BTC_USDT"]` (list of currency pairs)

**Response fields:**

| Field | Type    | Description                     |
|-------|---------|---------------------------------|
| `t`   | integer | Update timestamp (milliseconds) |
| `u`   | integer | Order book update ID            |
| `s`   | string  | Currency pair                   |
| `b`   | string  | Best bid price                  |
| `B`   | string  | Best bid amount                 |
| `a`   | string  | Best ask price                  |
| `A`   | string  | Best ask amount                 |

---

### 1.2 `spot.order_book_update` — Incremental Deltas (Recommended for local book)

| Property       | Value                                |
|----------------|--------------------------------------|
| Update speed   | `20ms` or `100ms` (selectable)       |
| Depth at 20ms  | 20 levels                            |
| Depth at 100ms | 100 levels                           |
| Type           | Delta (incremental); first message = full snapshot (`full: true`) |
| Authentication | Not required                         |

**Subscription payload:** `["BTC_USDT", "100ms"]` — `[currency_pair, interval]`

Valid intervals: `"20ms"`, `"100ms"`

Note: Interval and depth are linked — `20ms` gives 20-level depth, `100ms` gives 100-level depth.

**Response fields:**

| Field  | Type    | Description                                     |
|--------|---------|-------------------------------------------------|
| `t`    | integer | Update timestamp (milliseconds)                 |
| `s`    | string  | Currency pair                                   |
| `U`    | integer | First update ID in this event (inclusive)       |
| `u`    | integer | Last update ID in this event (inclusive)        |
| `l`    | string  | Depth level (e.g. `"100"`)                      |
| `full` | boolean | `true` = full snapshot; absent/false = delta    |
| `b`    | array   | Changed bid levels: `[["price", "amount"], ...]`|
| `a`    | array   | Changed ask levels: `[["price", "amount"], ...]`|

**Example subscription:**
```json
{
  "time": 1606294781,
  "channel": "spot.order_book_update",
  "event": "subscribe",
  "payload": ["BTC_USDT", "100ms"]
}
```

**Example delta message:**
```json
{
  "time": 1606294781,
  "channel": "spot.order_book_update",
  "event": "update",
  "result": {
    "t": 1606294781123,
    "full": true,
    "l": "100",
    "s": "BTC_USDT",
    "U": 48776301,
    "u": 48776306,
    "b": [["19137.74", "0.0001"]],
    "a": [["19137.75", "0.6135"]]
  }
}
```

---

### 1.3 `spot.order_book` — Periodic Snapshots

| Property       | Value                                   |
|----------------|-----------------------------------------|
| Update speed   | `100ms` or `1000ms` (selectable)        |
| Depth levels   | 5, 10, 20, 50, 100                      |
| Type           | Full snapshot on each push              |
| Authentication | Not required                            |

**Subscription payload:** `["BTC_USDT", "5", "100ms"]` — `[currency_pair, level, interval]`

Valid levels: `"5"`, `"10"`, `"20"`, `"50"`, `"100"`
Valid intervals: `"100ms"`, `"1000ms"`

**Response fields:**

| Field          | Type    | Description                                     |
|----------------|---------|-------------------------------------------------|
| `t`            | integer | Snapshot timestamp (milliseconds)               |
| `s`            | string  | Currency pair                                   |
| `l`            | string  | Depth level                                     |
| `lastUpdateId` | integer | Order book snapshot ID                          |
| `bids`         | array   | Top bids high→low: `[["price", "amount"], ...]` |
| `asks`         | array   | Top asks low→high: `[["price", "amount"], ...]` |

---

### 1.4 `spot.obu` — Order Book V2 (Fastest)

| Property            | Value                                     |
|---------------------|-------------------------------------------|
| Depth 50 update     | 20 ms                                     |
| Depth 400 update    | 100 ms                                    |
| Type                | Delta; first message = full snapshot (`full: true`) |
| Authentication      | Not required                              |

**Subscription payload:** `["ob.BTC_USDT.50"]` or `["ob.BTC_USDT.400"]`

Valid levels: `50`, `400`

**Response fields:**

| Field  | Type    | Description                                          |
|--------|---------|------------------------------------------------------|
| `t`    | integer | Timestamp (milliseconds)                             |
| `s`    | string  | Stream name (e.g. `"ob.BTC_USDT.400"`)               |
| `U`    | integer | Starting update ID of this batch                     |
| `u`    | integer | Ending update ID of this batch                       |
| `full` | boolean | `true` = full replacement snapshot; absent = delta   |
| `b`    | array   | Bid updates: `[["price", "amount"], ...]`            |
| `a`    | array   | Ask updates: `[["price", "amount"], ...]`            |

**Example (full snapshot):**
```json
{
  "result": {
    "t": 1743673026995,
    "full": true,
    "s": "ob.BTC_USDT.400",
    "u": 79072179673,
    "b": [["83705.9", "30166"]],
    "a": [["83706", "4208"]]
  }
}
```

---

## 2. WebSocket Channels — Futures (Perpetual & Delivery)

Base URL: `wss://fx-ws.gateio.ws/v4/ws/usdt` (or `/btc` for BTC-settled)

### 2.1 `futures.book_ticker` — Best Bid/Ask

| Property     | Value        |
|--------------|--------------|
| Update speed | Real-time    |
| Parameters   | `[contract]` |

**Fields:** `t` (ms), `b`/`B` (best bid price/size), `a`/`A` (best ask price/size)

---

### 2.2 `futures.order_book` — Legacy Full + Delta (Not Recommended)

| Property     | Value                                             |
|--------------|---------------------------------------------------|
| Type         | `all` event = full snapshot; `update` event = delta |
| Parameters   | `[contract, limit, interval]`                    |

Valid limits: `"1"`, `"5"`, `"10"`, `"20"`, `"50"`, `"100"`
Valid interval: `"0"` only

---

### 2.3 `futures.order_book_update` — Incremental Deltas (Recommended)

| Property             | Value                                      |
|----------------------|--------------------------------------------|
| Update speed         | `100ms` or `20ms` (selectable)             |
| Depth at 20ms        | 20 levels only                             |
| Depth at 100ms       | 20, 50, or 100 levels (selectable)         |
| Type                 | Delta; first message may be full snapshot  |
| Authentication       | Not required                               |

**Subscription payload:** `[contract, frequency, level]`

Valid frequencies: `"20ms"`, `"100ms"`
Valid levels: `"20"` (20ms only), `"20"`, `"50"`, `"100"` (100ms)

**Response fields:**

| Field  | Type    | Description                                              |
|--------|---------|----------------------------------------------------------|
| `t`    | integer | Timestamp (milliseconds)                                 |
| `s`    | string  | Contract name                                            |
| `U`    | integer | First update ID                                          |
| `u`    | integer | Last update ID                                           |
| `l`    | string  | Depth level (e.g. `"100"`)                               |
| `b`    | array   | Bid changes: `[{"p": "price", "s": "size"}, ...]`        |
| `a`    | array   | Ask changes: `[{"p": "price", "s": "size"}, ...]`        |

Note: Futures bid/ask entries use object format `{"p": "price", "s": "size"}` vs spot's array `["price", "amount"]`.

**Example delta:**
```json
{
  "time_ms": 1615366381123,
  "channel": "futures.order_book_update",
  "result": {
    "t": 1615366381417,
    "s": "BTC_USD",
    "U": 2517661101,
    "u": 2517661113,
    "b": [{"p": "54672.1", "s": "0"}],
    "a": [{"p": "54743.6", "s": "0"}],
    "l": "100"
  }
}
```

---

### 2.4 `futures.obu` — Order Book V2 (Fastest Futures)

| Property         | Value                                       |
|------------------|---------------------------------------------|
| Depth 50 update  | 20 ms                                       |
| Depth 400 update | 100 ms                                      |
| Type             | Delta; first message = full snapshot        |

**Subscription payload:** `["ob.BTC_USDT.50"]` or `["ob.BTC_USDT.400"]`
Valid levels: `50`, `400`

**Fields:** Same as `spot.obu` — `t`, `full`, `s`, `U`, `u`, `b`, `a`

---

## 3. REST API — Order Book Endpoints

### 3.1 Spot: `GET /api/v4/spot/order_book`

| Parameter       | Type    | Required | Default | Notes                                      |
|-----------------|---------|----------|---------|--------------------------------------------|
| `currency_pair` | string  | Yes      | —       | e.g. `BTC_USDT`                            |
| `interval`      | string  | No       | `"0"`   | Price aggregation precision; `"0"` = none  |
| `limit`         | integer | No       | 10      | Number of levels per side; max ~100        |
| `with_id`       | boolean | No       | false   | If true, response includes order book ID for WS sync |

**Response includes `id` field** (order book update ID) when `with_id=true`. This ID corresponds to `lastUpdateId` / `u` in WebSocket messages — required for local book initialization.

Interval valid values for spot: `"0"` (no aggregation) — additional precision tiers may be pair-dependent.

---

### 3.2 Futures: `GET /api/v4/futures/{settle}/order_book`

| Parameter  | Type    | Required | Default | Notes                                 |
|------------|---------|----------|---------|---------------------------------------|
| `settle`   | string  | Yes (path)| —      | `"usdt"` or `"btc"`                   |
| `contract` | string  | Yes      | —       | e.g. `BTC_USDT`                       |
| `interval` | string  | No       | `"0"`   | Aggregation: `"0"`, `"0.1"`, `"0.01"` |
| `limit`    | integer | No       | —       | Max ~100 levels per side              |
| `with_id`  | boolean | No       | false   | Returns order book update ID          |

**Response fields:**
- `id`: Order book update ID (only with `with_id=true`)
- `bids`: array of `{"p": price, "s": size}` (high → low)
- `asks`: array of `{"p": price, "s": size}` (low → high)

---

## 4. Update Speed Summary

| Channel                       | Market   | Speed Options     | Depth Options          |
|-------------------------------|----------|-------------------|------------------------|
| `spot.book_ticker`            | Spot     | 10 ms (fixed)     | Best bid/ask only      |
| `spot.order_book_update`      | Spot     | 20 ms, 100 ms     | 20 levels, 100 levels  |
| `spot.order_book`             | Spot     | 100 ms, 1000 ms   | 5, 10, 20, 50, 100     |
| `spot.obu` (V2)               | Spot     | 20 ms, 100 ms     | 50, 400                |
| `futures.book_ticker`         | Futures  | Real-time         | Best bid/ask only      |
| `futures.order_book`          | Futures  | (interval `"0"`)  | 1, 5, 10, 20, 50, 100  |
| `futures.order_book_update`   | Futures  | 20 ms, 100 ms     | 20 (20ms); 20/50/100 (100ms) |
| `futures.obu` (V2)            | Futures  | 20 ms, 100 ms     | 50, 400                |

---

## 5. Price Aggregation (Interval Parameter)

**Spot REST:** `interval` = `"0"` (no aggregation). Additional levels are pair-dependent and not explicitly listed.

**Futures REST:** `interval` options:
- `"0"` — no aggregation (default, raw tick precision)
- `"0.1"` — aggregate to 0.1 price increments
- `"0.01"` — aggregate to 0.01 price increments

**WebSocket:** No aggregation parameter is exposed. WS streams always deliver raw (non-aggregated) prices.

---

## 6. Checksum

**No checksum field documented** for any Gate.io v4 WebSocket order book channel (spot or futures). Gate.io uses **sequence IDs** (`U`/`u`) for gap detection and data integrity verification, not CRC32 or similar hash-based checksums.

This differs from Binance (no checksum) and Kraken (has CRC32 checksum).

---

## 7. Sequence / Ordering Fields

### Field Names

| Channel                          | Sequence Fields         |
|----------------------------------|-------------------------|
| `spot.book_ticker`               | `u` (single snapshot ID)|
| `spot.order_book_update`         | `U` (first), `u` (last) |
| `spot.order_book`                | `lastUpdateId`          |
| `spot.obu`                       | `U` (first), `u` (last) |
| `futures.order_book`             | event type (`all`/`update`) |
| `futures.order_book_update`      | `U` (first), `u` (last) |
| `futures.obu`                    | `U` (first), `u` (last) |

### Gap Detection Logic

For `order_book_update` and `obu` channels (both spot and futures):

1. **Initialize**: Call REST `GET /spot/order_book?with_id=true` (or futures equivalent) to get base snapshot with ID.
2. **Sync**: The first WebSocket message with `full: true` replaces local book entirely; update local `depth_id = u`.
3. **Continuity check**: For each subsequent delta, verify `U == local_depth_id + 1`.
   - If `true` → continuous, apply update, set `local_depth_id = u`
   - If `false` → gap detected → **unsubscribe and resubscribe** to get fresh snapshot
4. **Zero amount**: If `amount == "0"` (or `size == "0"` for futures) → remove that price level from local book.

---

## 8. Spot vs. Futures Differences

| Aspect                    | Spot                                     | Futures                                  |
|---------------------------|------------------------------------------|------------------------------------------|
| Channel namespace         | `spot.*`                                 | `futures.*`                              |
| Bid/ask entry format      | `["price", "amount"]` arrays             | `{"p": "price", "s": "size"}` objects   |
| Field names               | `b`/`a` with string arrays               | `b`/`a` with object arrays               |
| REST settle param         | Not applicable                           | `settle` path param: `usdt` or `btc`     |
| REST interval options     | `"0"` (documented)                       | `"0"`, `"0.1"`, `"0.01"`                |
| REST limit max            | ~100 (default 10)                        | ~100                                     |
| Legacy channel            | `spot.order_book` (still valid)          | `futures.order_book` (deprecated, use `futures.order_book_update`) |
| Recommended channel       | `spot.obu` (fastest) or `spot.order_book_update` | `futures.obu` (fastest) or `futures.order_book_update` |
| Futures delta channel     | n/a                                      | `futures.order_book` uses `"all"`/`"update"` event type instead of `full` boolean |

---

## 9. WebSocket Connection Details

- **Spot WS endpoint:** `wss://api.gateio.ws/ws/v4/`
- **Futures WS endpoint (USDT):** `wss://fx-ws.gateio.ws/v4/ws/usdt`
- **Futures WS endpoint (BTC):** `wss://fx-ws.gateio.ws/v4/ws/btc`
- **Authentication:** Order book channels do not require authentication (public data)
- **Subscription format:**
```json
{
  "time": 1606294781,
  "channel": "spot.order_book_update",
  "event": "subscribe",
  "payload": ["BTC_USDT", "100ms"]
}
```

---

## Sources

- [Gate Spot WebSocket API v4](https://www.gate.com/docs/developers/apiv4/ws/en/)
- [Gate Futures WebSocket v4](https://www.gate.com/docs/developers/futures/ws/en/)
- [Gate API v4 Futures REST](https://www.gate.com/docs/futures/api/index.html)
- [Gate API v4 Overview](https://www.gate.com/docs/developers/apiv4/en/)
- [Gate.io Futures API Upgrade Announcement (10ms orderbook)](https://www.gate.com/announcements/article/19776)
- [Gate.io at Tardis.dev](https://docs.tardis.dev/historical-data-details/gate-io)
