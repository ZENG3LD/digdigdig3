# Bitstamp L2 Order Book Capabilities

Research date: 2026-04-16
Official docs: https://www.bitstamp.net/websocket/v2/ | https://www.bitstamp.net/api/

---

## 1. WebSocket Channels

### Connection Endpoint

```
wss://ws.bitstamp.net
```

### Subscription Format

```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "order_book_btcusd"
  }
}
```

To unsubscribe: same message with `"event": "bts:unsubscribe"`.

### Available Order Book Channels

| Channel Pattern | Type | Depth | Behavior |
|---|---|---|---|
| `order_book_{pair}` | L2 Snapshot | Top 100 bids + Top 100 asks | Full snapshot on every change |
| `diff_order_book_{pair}` | L2 Delta | Full book | Incremental diffs since last broadcast |
| `live_orders_{pair}` | L3 Events | Full book | Per-order lifecycle events (add/modify/cancel) |

**Channel name examples:**
- `order_book_btcusd`
- `diff_order_book_btcusd`
- `live_orders_btcusd`

Currency pair is lowercase, concatenated (e.g. `btcusd`, `ethusd`, `btceur`).

---

### Channel: `order_book_{pair}` — Live Order Book (L2 Snapshot)

**Behavior:** Emits a full snapshot of top 100 bids and top 100 asks whenever the book changes. Each message is self-contained — no local state maintenance required. Considered the most reliable channel, always consistent with Bitstamp's own trading terminal.

**Incoming message format:**

```json
{
  "event": "data",
  "channel": "order_book_btcusd",
  "data": {
    "timestamp": "1605558814",
    "microtimestamp": "1605558814000000",
    "bids": [
      ["3284.06000000", "0.16927410"],
      ["3284.05000000", "1.00000000"]
    ],
    "asks": [
      ["3289.00000000", "3.16123001"],
      ["3291.99000000", "0.22000000"]
    ]
  }
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `timestamp` | string | Unix timestamp in seconds |
| `microtimestamp` | string | Unix timestamp in microseconds (precision: 1 µs) |
| `bids` | array of [price, amount] | Top 100 bids, sorted highest-price-first, strings |
| `asks` | array of [price, amount] | Top 100 asks, sorted lowest-price-first, strings |

All numeric values are returned as strings. Up to 8 decimal places.

---

### Channel: `diff_order_book_{pair}` — Live Full Order Book (L2 Delta)

**Behavior:** Emits incremental diffs — only the changes since the last broadcast. Covers the full depth of the book (not just top 100). Known to occasionally diverge and produce crossed books (bids > asks) after extended operation — requires careful local state management.

**No initial snapshot provided.** Clients must:
1. Subscribe to `diff_order_book_{pair}`
2. Fetch initial snapshot via REST: `GET /api/v2/order_book/{pair}/?group=0` (full depth)
3. Apply diffs on top of the snapshot
4. Discard diffs with `microtimestamp` older than the snapshot

Snapshot messages (from third-party providers bridging the gap) are marked with:
```json
{ "event": "snapshot", "generated": true }
```

**Fields:** Same as `order_book` channel (timestamp, microtimestamp, bids, asks). Each entry in bids/asks that is returned represents a change — removed levels have amount `"0"`.

---

### Channel: `live_orders_{pair}` — Live Orders (L3)

**Behavior:** Per-order lifecycle events. Provides the order ID alongside price and amount. Equivalent to L3 feed.

Fields include `id` (order_id), `price`, `amount`, `order_type`, `datetime`, `microtimestamp`.

**Note:** Bitstamp's event stream for this channel frequently omits data and events can appear out of order (e.g., cancel before add). No initial snapshot provided — same bootstrapping requirement as `diff_order_book`.

---

## 2. REST Order Book Endpoint

### Endpoint

```
GET https://www.bitstamp.net/api/v2/order_book/{currency_pair}/
```

**Example:**
```
GET https://www.bitstamp.net/api/v2/order_book/btcusd/?group=1
```

### Parameters

| Parameter | Type | Required | Values | Description |
|---|---|---|---|---|
| `currency_pair` | path | yes | e.g. `btcusd`, `ethusd` | Trading pair, lowercase concatenated |
| `group` | query | no | `0`, `1`, `2` | Price aggregation/grouping mode (see below) |

### `group` Parameter Values

| Value | Behavior |
|---|---|
| `0` | Orders not grouped — each individual order shown at its exact price |
| `1` | Orders grouped by price (default) — aggregated by price level |
| `2` | Orders grouped by price with top 100 levels only |

### Response Format

```json
{
  "timestamp": "1518634712",
  "microtimestamp": "1518634712000000",
  "bids": [
    ["3284.06000000", "0.16927410"],
    ["3284.05000000", "1.00000000"]
  ],
  "asks": [
    ["3289.00000000", "3.16123001"],
    ["3291.99000000", "0.22000000"]
  ]
}
```

**Fields:**

| Field | Type | Description |
|---|---|---|
| `timestamp` | string | Unix timestamp in seconds |
| `microtimestamp` | string | Unix timestamp in microseconds |
| `bids` | array of [price, amount] | Buy orders, sorted highest-price-first |
| `asks` | array of [price, amount] | Sell orders, sorted lowest-price-first |

With `group=0`: Returns the full ungrouped order book (all individual orders).
With `group=1` (default): Returns price-aggregated book.

**Note:** Parameter is named `group` in official Bitstamp docs, not `depth`. Some third-party documentation incorrectly refers to it as `depth`.

---

## 3. Update Speed

- **WebSocket `order_book`:** No configurable update speed. Updates are event-driven — a message is emitted on every book change. No throttle or interval options.
- **WebSocket `diff_order_book`:** Same — event-driven, no configurable speed.
- **REST:** Poll-based. Rate limit: 600 requests per 10 minutes per IP (standard public tier). Higher limits available via bespoke agreement.

---

## 4. Price Aggregation

- **REST `group` parameter:** Controls aggregation. `group=0` = no aggregation (raw orders), `group=1` = grouped by price level (default), `group=2` = grouped top 100.
- **WebSocket `order_book`:** Always delivers price-level aggregated top 100 — no raw order-level data, no configurable aggregation.
- **WebSocket `diff_order_book`:** Delivers full depth price-level aggregated diffs — no configurable aggregation parameter.
- **No dynamic grouping options** (e.g., no "group by 10 USD" style aggregation on the wire). Only the three fixed `group` modes on REST.

---

## 5. Checksum

- **Status:** Not confirmed present in official Bitstamp WebSocket v2 documentation for `order_book` or `diff_order_book` channels.
- The C# client library `bitstamp-client-websocket` (Marfusios) lists "Orderbook L2 (100 lvl)", "Orderbook L3 (100 ord)", and "Orderbook L2 (diffs)" as supported, but no checksum field is documented in official sources.
- Some third-party sources mention a CRC32 checksum field, but this is **not confirmed** from official Bitstamp documentation. Treat as unverified.
- **Recommendation:** Assume no checksum. Validate by comparing `order_book` snapshots against locally reconstructed `diff_order_book` state periodically.

---

## 6. Sequence / Ordering

### microtimestamp

- Present in **all** order book channel messages (both WebSocket and REST).
- Type: string representing Unix timestamp in **microseconds**.
- Example: `"1605558814000000"` (microseconds since epoch).
- Used for ordering events and determining freshness of diffs vs. REST snapshots.

### Sync Procedure for `diff_order_book`

The correct bootstrap procedure (no initial snapshot from WS):

1. Start buffering `diff_order_book_{pair}` messages (do NOT apply yet).
2. Fetch REST snapshot: `GET /api/v2/order_book/{pair}/?group=0`
3. Note the `microtimestamp` of the REST snapshot.
4. Discard all buffered diffs with `microtimestamp <= snapshot_microtimestamp`.
5. Apply remaining buffered diffs to the snapshot state.
6. Continue applying live diffs as they arrive.

### Event Ordering Notes

- `live_orders` channel: events are known to arrive out of order. Cancel events can appear before the corresponding add event.
- `order_book` channel: stateless snapshots, no ordering concern.
- `diff_order_book` channel: microtimestamp-ordered, but gaps are possible. If a gap is detected (microtimestamp jumps unexpectedly), re-bootstrap from REST.

---

## 7. Special Notes

### `order_book` vs `diff_order_book` Reliability

Bitstamp's own official examples and independent data providers (e.g., CryptoChassis) recommend `order_book` over `diff_order_book` for correctness:
- `order_book` is always consistent with Bitstamp's trading terminal.
- `diff_order_book` can diverge and produce crossed books (bids > asks) after extended periods.
- If using `diff_order_book`, implement periodic re-sync from REST snapshot.

### No `detail_order_book` Channel

Despite some documentation references, there is no confirmed separate `detail_order_book` channel in Bitstamp WebSocket v2. The three confirmed channels are `order_book`, `diff_order_book`, and `live_orders`.

### Reconnect Handling

The WebSocket server can send `"bts:request_reconnect"` events. Clients must reinitialize the connection and re-bootstrap order book state.

### String Numeric Values

All price and amount values are returned as **strings** (not numbers), for precision. Parse to decimal/BigDecimal for arithmetic.

### Gap Recovery REST Endpoints

Bitstamp provides REST endpoints for order event replay:
- `POST /api/v2/order_data/` — historical public order events for a specific market (WS gap recovery)
- `POST /api/v2/account_order_data/` — historical order events for authenticated user

### Rate Limits (REST)

- Standard: 600 requests per 10 minutes per IP
- Higher limits available by agreement with Bitstamp

---

## Summary Table

| Feature | Value |
|---|---|
| WS Endpoint | `wss://ws.bitstamp.net` |
| L2 Snapshot Channel | `order_book_{pair}` (top 100, full snapshot per update) |
| L2 Delta Channel | `diff_order_book_{pair}` (full depth diffs, no initial snapshot) |
| L3 Channel | `live_orders_{pair}` (per-order events, unreliable ordering) |
| REST Endpoint | `GET /api/v2/order_book/{pair}/?group={0,1,2}` |
| REST Max Depth | Full book (`group=0`), or top 100 price levels (`group=2`) |
| Update Speed | Event-driven, not configurable |
| Price Aggregation | REST `group` param (0/1/2); WS always aggregated by price level |
| Checksum | Not confirmed in official docs |
| Microtimestamp | Yes — microsecond precision Unix timestamp string, all channels |
| Timestamp | Yes — second precision Unix timestamp string, all channels |
| Sequence Numbers | No explicit sequence numbers; microtimestamp used for ordering |
| Initial WS Snapshot | Only `order_book` provides it; `diff_order_book` requires REST bootstrap |

---

## Sources

- [Bitstamp WebSocket v2 Official Docs](https://www.bitstamp.net/websocket/v2/)
- [Bitstamp REST API Official Docs](https://www.bitstamp.net/api/)
- [Bitstamp Live Order Book Example](https://www.bitstamp.net/s/webapp/examples/order_book_v2.html)
- [Bitstamp Diff Order Book Example](https://www.bitstamp.net/s/webapp/examples/diff_order_book_v2.html)
- [Tardis.dev Bitstamp Historical Data Details](https://docs.tardis.dev/historical-data-details/bitstamp)
- [CryptoChassis Medium: Order Book Part II - Bitstamp](https://medium.com/open-crypto-market-data-initiative/walking-on-the-thin-ice-of-orderbooks-part-ii-bitstamp-f2c156e62d2a)
- [CCXT Issue #8106: Bitstamp order book corruption](https://github.com/ccxt/ccxt/issues/8106)
- [node-bitstamp npm](https://www.npmjs.com/package/node-bitstamp)
- [Marfusios bitstamp-client-websocket (C#)](https://github.com/Marfusios/bitstamp-client-websocket)
