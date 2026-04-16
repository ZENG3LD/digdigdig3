# Binance L2 Orderbook API Capabilities

Research date: 2026-04-16
Source: developers.binance.com (official Binance Open Platform docs)

---

## A. WebSocket Depth — Partial Book Depth Streams

### SPOT

- **Valid depth levels:** 5, 10, 20
- **Update speeds:** 1000ms (default), 100ms
- **Channel name format:**
  - `<symbol>@depth<levels>` — 1000ms updates
  - `<symbol>@depth<levels>@100ms` — 100ms updates
- **Examples:**
  - `wss://stream.binance.com:9443/ws/bnbbtc@depth5`
  - `wss://stream.binance.com:9443/ws/ethusdt@depth20@100ms`
- **Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `lastUpdateId` | integer | Latest update ID |
| `bids` | array | `[[price, qty], ...]` — top N bid levels |
| `asks` | array | `[[price, qty], ...]` — top N ask levels |

### USD-M Futures (fstream)

- **Valid depth levels:** 5, 10, 20
- **Update speeds:** 250ms (default), 500ms, 100ms
- **Channel name format:**
  - `<symbol>@depth<levels>` — 250ms updates
  - `<symbol>@depth<levels>@500ms` — 500ms updates
  - `<symbol>@depth<levels>@100ms` — 100ms updates
- **Base URL:** `wss://fstream.binance.com`
- **Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `e` | string | Event type: `"depthUpdate"` |
| `E` | integer | Event time (ms) |
| `T` | integer | Transaction time (ms) |
| `s` | string | Symbol |
| `U` | integer | First update ID in event |
| `u` | integer | Final update ID in event |
| `pu` | integer | Final update ID from previous stream event |
| `b` | array | Bid updates `[[price, qty], ...]` |
| `a` | array | Ask updates `[[price, qty], ...]` |

Note: RPI (Retail Price Improvement) orders are excluded from this stream.

### COIN-M Futures (dstream)

- **Valid depth levels:** 5, 10, 20
- **Update speeds:** 250ms (default), 500ms, 100ms
- **Channel name format:** same as USD-M (`<symbol>@depth<levels>[@500ms|@100ms]`)
- **Base URL:** `wss://dstream.binance.com`
- **Response fields:** same as USD-M, plus `ps` (pair):

| Field | Type | Description |
|-------|------|-------------|
| `e` | string | Event type: `"depthUpdate"` |
| `E` | integer | Event time (ms) |
| `T` | integer | Transaction time (ms) |
| `s` | string | Symbol (e.g. `BTCUSD_200626`) |
| `ps` | string | Pair (e.g. `BTCUSD`) — COIN-M only |
| `U` | integer | First update ID in event |
| `u` | integer | Final update ID in event |
| `pu` | integer | Final update ID from previous stream event |
| `b` | array | Bid updates |
| `a` | array | Ask updates |

---

## B. WebSocket Diff Depth Stream

### SPOT Diff Depth

- **Channel name format:**
  - `<symbol>@depth` — 1000ms updates
  - `<symbol>@depth@100ms` — 100ms updates
- **Nature:** Delta updates (NOT full snapshots)
- **Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `e` | string | Event type: `"depthUpdate"` |
| `E` | integer | Event time (ms) |
| `s` | string | Symbol |
| `U` | integer | First update ID in event |
| `u` | integer | Final update ID in event |
| `b` | array | Bid updates `[[price, qty], ...]` |
| `a` | array | Ask updates `[[price, qty], ...]` |

Note: Spot diff depth does NOT have `T` (transaction time) or `pu` fields.

### USD-M Futures Diff Depth

- **Channel name format:**
  - `<symbol>@depth` — 250ms updates
  - `<symbol>@depth@500ms` — 500ms updates
  - `<symbol>@depth@100ms` — 100ms updates
- **Nature:** Delta updates
- **Response fields:** (same as Futures Partial Book fields above — includes `T`, `pu`)

| Field | Description |
|-------|-------------|
| `e` | `"depthUpdate"` |
| `E` | Event time (ms) |
| `T` | Transaction time (ms) |
| `s` | Symbol |
| `U` | First update ID in event |
| `u` | Final update ID in event |
| `pu` | Final update ID from the PREVIOUS stream event (for gap detection) |
| `b` | Bid updates |
| `a` | Ask updates |

Note: RPI orders are excluded. `pu` is the key field for gap detection (see Section G).

### COIN-M Futures Diff Depth

- **Channel name format:**
  - `<symbol>@depth` — 250ms default
  - `<symbol>@depth@500ms`
  - `<symbol>@depth@100ms`
- **Response fields:** same as USD-M plus `ps` (pair field)

### USD-M Futures RPI Diff Depth (special stream)

- **Channel name:** `<symbol>@rpiDepth@500ms`
- **Update speed:** 500ms only (fixed)
- **Difference:** Includes RPI (Retail Price Improvement) orders in the bid/ask data
- **Zero-quantity behavior:** Level quantity = 0 means either all orders filled/canceled OR RPI orders for that level are hidden (crossed)

---

## C. REST Order Book

### SPOT — `GET /api/v3/depth`

| Limit range | API Weight |
|-------------|-----------|
| 1–100 | 5 |
| 101–500 | 25 |
| 501–1000 | 50 |
| 1001–5000 | 250 |

- **Default limit:** 100
- **Maximum:** 5000 (if higher value is requested, only 5000 entries returned)
- **Valid values:** Any integer from 1 to 5000 (no discrete set — full range allowed)
- **Required params:** `symbol` (STRING)
- **Optional params:** `limit` (INT), `symbolStatus` (ENUM: `TRADING`, `HALT`, `BREAK`)
- **Response fields:**

| Field | Description |
|-------|-------------|
| `lastUpdateId` | Order book snapshot update ID |
| `bids` | `[[price, qty], ...]` strings |
| `asks` | `[[price, qty], ...]` strings |

- **Data source:** Memory (real-time)

### USD-M Futures — `GET /fapi/v1/depth`

- **Valid limit values:** 5, 10, 20, 50, 100, 500, 1000 (discrete set, not arbitrary)
- **Default limit:** 500

| Limit | Weight |
|-------|--------|
| 5, 10, 20, 50 | 2 |
| 100 | 5 |
| 500 | 10 |
| 1000 | 20 |

- **Response fields:**

| Field | Description |
|-------|-------------|
| `lastUpdateId` | Update ID |
| `E` | Message output time (ms) |
| `T` | Transaction time (ms) |
| `bids` | `[[price, qty], ...]` |
| `asks` | `[[price, qty], ...]` |

### COIN-M Futures — `GET /dapi/v1/depth`

- **Valid limit values:** 5, 10, 20, 50, 100, 500, 1000 (discrete set, same as USD-M)
- **Default limit:** 500
- **Weight costs:** same as USD-M (2 / 5 / 10 / 20)
- **Response fields:** same as USD-M, plus `symbol` and `pair`:

| Field | Description |
|-------|-------------|
| `lastUpdateId` | Update ID |
| `symbol` | e.g. `BTCUSD_PERP` |
| `pair` | e.g. `BTCUSD` |
| `E` | Message output time (ms) |
| `T` | Transaction time (ms) |
| `bids` | `[[price, qty], ...]` |
| `asks` | `[[price, qty], ...]` |

### USD-M Futures RPI Order Book — `GET /fapi/v1/depth` (RPI variant)

- **Valid limit values:** 1000 only
- **Includes RPI orders** in the aggregated response
- Crossed price levels are hidden

---

## D. Update Speed

### Summary Table

| Product | Partial Book WS | Diff Depth WS |
|---------|-----------------|---------------|
| Spot | 1000ms, 100ms | 1000ms, 100ms |
| USD-M Futures | 250ms, 500ms, 100ms | 250ms, 500ms, 100ms |
| COIN-M Futures | 250ms, 500ms, 100ms | 250ms, 500ms, 100ms |

### How speed is specified in channel name:

- No suffix → default speed (1000ms for Spot, 250ms for Futures)
- `@100ms` → 100ms updates
- `@500ms` → 500ms updates (Futures only)

**Examples:**
```
btcusdt@depth20          # Spot 1000ms
btcusdt@depth20@100ms   # Spot 100ms
btcusdt@depth5@500ms    # Futures 500ms
btcusdt@depth5@100ms    # Futures 100ms
```

---

## E. Price Aggregation

- **Binance does NOT support configurable price grouping on WebSocket streams.** All WS depth streams return raw price levels as-is from the matching engine.
- There is no "tick-size grouping" or "price bucket" parameter on any documented stream.
- The only aggregation endpoints are:
  - **Book Ticker** (`<symbol>@bookTicker`) — best bid/ask only, not a multi-level orderbook
  - **RPI streams** — include RPI orders aggregated into price levels, but this is not user-configurable price grouping

---

## F. Checksum

- **Binance does NOT provide orderbook checksums** in any of the documented WS or REST endpoints for Spot or Futures.
- No CRC32 or other integrity field found in the documentation.
- Integrity is instead maintained via the sequencing fields (`U`, `u`, `pu`) and the sync procedure described in Section G.

---

## G. Sequence / Ordering and Sync Procedure

### SPOT — Fields for ordering

| Field | Meaning |
|-------|---------|
| `U` | First update ID in this WS event |
| `u` | Final update ID in this WS event |
| `lastUpdateId` | Snapshot update ID (from REST) |

**No `pu` field on Spot.** Gap detection is done differently (see procedure below).

### Futures (USD-M and COIN-M) — Fields for ordering

| Field | Meaning |
|-------|---------|
| `U` | First update ID in this WS event |
| `u` | Final update ID in this WS event |
| `pu` | Final update ID from the PREVIOUS stream event |

**Gap detection:** If `pu` of current event != `u` of previous event → gap detected → reinitialize from step 1.

---

### SPOT — Local Order Book Sync Procedure (official)

1. Open WebSocket stream to `<symbol>@depth` or `<symbol>@depth@100ms`. Buffer all events. Note the `U` of the first event.
2. Fetch REST snapshot: `GET https://api.binance.com/api/v3/depth?symbol=BNBBTC&limit=5000`
3. Verify: `snapshot.lastUpdateId` must be **strictly less than** the first buffered event's `U`. If not, retry step 2.
4. Discard all buffered events where `u <= snapshot.lastUpdateId`.
5. The first event to apply must satisfy: `U <= snapshot.lastUpdateId + 1` AND `u >= snapshot.lastUpdateId + 1`.
6. Initialize local book with snapshot data. Set local update ID = `snapshot.lastUpdateId`.
7. For each subsequent event:
   - Discard if `u < local_update_id + 1`
   - Restart from step 1 if `U > local_update_id + 1` (gap detected)
   - Apply updates: price levels are absolute quantities (not deltas). Remove levels where qty = 0.

### Futures (USD-M/COIN-M) — Local Order Book Sync Procedure (official)

1. Open WebSocket stream: `wss://fstream.binance.com/stream?streams=btcusdt@depth`. Buffer events.
2. Fetch REST snapshot: `GET /fapi/v1/depth?symbol=BTCUSDT&limit=1000`
3. Discard buffered events where `u < snapshot.lastUpdateId`.
4. First valid event must satisfy: `U <= snapshot.lastUpdateId` AND `u >= snapshot.lastUpdateId`.
5. For each subsequent event: validate `pu == previous_event.u`. If not → gap → reinitialize from step 1.
6. Apply updates: quantities are absolute (not deltas). Remove levels where qty = 0. Missing price levels in local book are normal (can receive a removal for a level not present — ignore safely).

---

## H. Spot vs Futures Differences

| Feature | SPOT | USD-M Futures | COIN-M Futures |
|---------|------|---------------|----------------|
| Partial depth levels | 5, 10, 20 | 5, 10, 20 | 5, 10, 20 |
| WS update speeds | 1000ms, 100ms | 250ms, 500ms, 100ms | 250ms, 500ms, 100ms |
| Default WS speed | 1000ms | 250ms | 250ms |
| `pu` field in WS | No | Yes | Yes |
| `T` (transaction time) | No | Yes | Yes |
| `ps` (pair) field | No | No | Yes |
| REST valid limits | 1–5000 (any int) | 5,10,20,50,100,500,1000 | 5,10,20,50,100,500,1000 |
| REST default limit | 100 | 500 | 500 |
| REST max limit | 5000 | 1000 | 1000 |
| REST `E`/`T` fields | No | Yes | Yes |
| REST `symbol`/`pair` | No | No | Yes (COIN-M) |
| RPI stream available | No | Yes | Not found in docs |
| Gap detection method | `U`/`u` vs `lastUpdateId` | `pu == prev.u` | `pu == prev.u` |
| Checksum | No | No | No |
| Price aggregation | No | No | No |

---

## Sources

- [Binance Spot WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- [Binance Spot REST Market Data Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints)
- [USD-M Futures Partial Book Depth Streams](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Partial-Book-Depth-Streams)
- [USD-M Futures Diff Book Depth Streams](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Diff-Book-Depth-Streams)
- [USD-M Futures RPI Diff Book Depth Streams](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/Diff-Book-Depth-Streams-RPI)
- [USD-M Futures REST Order Book](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Order-Book)
- [USD-M Futures RPI REST Order Book](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Order-Book-RPI)
- [USD-M Futures How to Manage Local Order Book](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams/How-to-manage-a-local-order-book-correctly)
- [COIN-M Futures Partial Book Depth Streams](https://developers.binance.com/docs/derivatives/coin-margined-futures/websocket-market-streams/Partial-Book-Depth-Streams)
- [COIN-M Futures Diff Book Depth Streams](https://developers.binance.com/docs/derivatives/coin-margined-futures/websocket-market-streams/Diff-Book-Depth-Streams)
- [COIN-M Futures REST Order Book](https://developers.binance.com/docs/derivatives/coin-margined-futures/market-data/rest-api/Order-Book)
