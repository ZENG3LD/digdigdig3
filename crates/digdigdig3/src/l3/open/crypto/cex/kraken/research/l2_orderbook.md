# Kraken L2 Orderbook API Capabilities

Research date: 2026-04-16
Sources: Official Kraken API documentation at docs.kraken.com

---

## 1. WebSocket Channels

### 1.1 Spot WebSocket v2 — `book` Channel

**Endpoint:** `wss://ws.kraken.com/v2`
**Channel name:** `book`

#### Subscription Message

```json
{
  "method": "subscribe",
  "params": {
    "channel": "book",
    "symbol": ["BTC/USD", "ETH/USD"],
    "depth": 10,
    "snapshot": true
  },
  "req_id": 1234
}
```

#### Subscription Parameters

| Parameter | Type            | Required | Valid Values              | Default |
|-----------|-----------------|----------|---------------------------|---------|
| `method`  | string          | Yes      | `"subscribe"`             | —       |
| `channel` | string          | Yes      | `"book"`                  | —       |
| `symbol`  | array[string]   | Yes      | e.g. `["BTC/USD"]`        | —       |
| `depth`   | integer         | No       | `10, 25, 100, 500, 1000`  | `10`    |
| `snapshot`| boolean         | No       | `true`, `false`           | `true`  |
| `req_id`  | integer         | No       | Any integer               | —       |

#### Snapshot Message Format

```json
{
  "channel": "book",
  "type": "snapshot",
  "data": [
    {
      "symbol": "BTC/USD",
      "bids": [
        { "price": 45283.5, "qty": 1.0 }
      ],
      "asks": [
        { "price": 45285.2, "qty": 0.1 }
      ],
      "checksum": 2114181697,
      "timestamp": "2023-10-06T17:35:55.440295Z"
    }
  ]
}
```

#### Update Message Format

```json
{
  "channel": "book",
  "type": "update",
  "data": [
    {
      "symbol": "MATIC/USD",
      "bids": [
        { "price": 0.5657, "qty": 1098.3947558 }
      ],
      "asks": [],
      "checksum": 2114181697,
      "timestamp": "2023-10-06T17:35:55.440295Z"
    }
  ]
}
```

#### Message Fields

| Field       | Type    | Description                                             |
|-------------|---------|---------------------------------------------------------|
| `channel`   | string  | Always `"book"`                                         |
| `type`      | string  | `"snapshot"` or `"update"`                             |
| `data`      | array   | Array of book objects (one per symbol)                  |
| `symbol`    | string  | Currency pair identifier (e.g. `"BTC/USD"`)            |
| `bids`      | array   | Bid price levels — `[{price, qty}]` — float values     |
| `asks`      | array   | Ask price levels — `[{price, qty}]` — float values     |
| `checksum`  | integer | CRC32 checksum of top 10 bids and asks (unsigned 32-bit)|
| `timestamp` | string  | RFC3339 format, e.g. `"2022-12-25T09:30:59.123456Z"`  |

#### Key Behaviors

- **Snapshot:** Sent once after subscription containing the full book at subscribed depth
- **Update:** Incremental changes only; `qty: 0` means the price level should be removed
- **Multiple updates per level:** A single update message may contain multiple updates to the same price level; process them in order
- **Depth truncation:** The client must truncate the local book to the subscribed depth after each update; the exchange does NOT send explicit removal messages for levels that fall out of the subscribed window

### 1.2 Futures WebSocket v1 — `book` Channel

**Endpoint:** `wss://futures.kraken.com/ws/v1`
**Feed names:** `book_snapshot` (initial), `book` (delta)

#### Subscription Message

```json
{
  "event": "subscribe",
  "feed": "book",
  "product_ids": ["PI_XBTUSD"]
}
```

#### Subscription Parameters

| Parameter     | Type           | Required | Description                         |
|---------------|----------------|----------|-------------------------------------|
| `event`       | string         | Yes      | `"subscribe"` or `"unsubscribe"`    |
| `feed`        | string         | Yes      | `"book"`                            |
| `product_ids` | array[string]  | Yes      | Futures product identifiers          |

#### Snapshot Message Format (`book_snapshot`)

```json
{
  "feed": "book_snapshot",
  "product_id": "PI_XBTUSD",
  "seq": 1,
  "timestamp": 1696614955440,
  "tickSize": null,
  "bids": [
    { "price": 45283.5, "qty": 1.0 }
  ],
  "asks": [
    { "price": 45285.2, "qty": 0.1 }
  ]
}
```

#### Delta Message Format (`book`)

```json
{
  "feed": "book",
  "product_id": "PI_XBTUSD",
  "seq": 2,
  "timestamp": 1696614955441,
  "side": "buy",
  "price": 45283.5,
  "qty": 1.5
}
```

#### Futures Message Fields

| Field        | Type            | Present In           | Description                              |
|--------------|-----------------|----------------------|------------------------------------------|
| `feed`       | string          | both                 | `"book_snapshot"` or `"book"`           |
| `product_id` | string          | both                 | Futures instrument (e.g. `"PI_XBTUSD"`) |
| `seq`        | positive integer| both                 | Monotonically increasing sequence number |
| `timestamp`  | positive integer| both                 | Unix milliseconds                        |
| `tickSize`   | string/null     | snapshot only        | Tick size (currently null)               |
| `bids`       | array           | snapshot only        | Array of `{price, qty}` objects          |
| `asks`       | array           | snapshot only        | Array of `{price, qty}` objects          |
| `side`       | string          | delta only           | `"buy"` or `"sell"`                     |
| `price`      | positive float  | delta only           | Price level of the update                |
| `qty`        | positive float  | delta only           | New quantity (0 = remove level)          |

#### Notable Futures Differences vs Spot v2

- **Delta format:** Per-level (one side, one price per message) vs spot v2 which batches multiple levels per message
- **Sequence numbers:** Futures has explicit `seq` per message; spot v2 has NO sequence numbers on book channel
- **Timestamp:** Futures uses Unix ms integer; spot v2 uses RFC3339 string
- **Depth:** Futures does NOT document fixed depth levels (full book); spot v2 has discrete depths (10/25/100/500/1000)
- **Checksum:** Futures has NO checksum; spot v2 has CRC32 checksum
- **Symbol format:** Futures uses product_id (e.g. `"PI_XBTUSD"`); spot v2 uses pair (e.g. `"BTC/USD"`)

---

## 2. REST Order Book

### 2.1 Spot REST API

**Endpoint:** `GET https://api.kraken.com/0/public/Depth`
**Authentication:** None (public endpoint)

#### Request Parameters

| Parameter | Type    | Required | Description                        |
|-----------|---------|----------|------------------------------------|
| `pair`    | string  | Yes      | Asset pair (e.g. `XBTUSD`, `XXBTZUSD`) |
| `count`   | integer | No       | Max number of asks/bids per side. Default: 100. Max: 100 |

**Notes:**
- `count` max is 100 per the REST endpoint (much lower than WebSocket's 1000)
- The REST endpoint does NOT offer the depth variety that WebSocket provides

#### Example Request

```
GET https://api.kraken.com/0/public/Depth?pair=xbteur&count=10
```

#### Response Format

```json
{
  "error": [],
  "result": {
    "XXBTZEUR": {
      "bids": [
        ["45283.50000", "1.000", 1696614955]
      ],
      "asks": [
        ["45285.20000", "0.100", 1696614955]
      ]
    }
  }
}
```

#### REST Response Fields

| Field     | Type    | Description                                |
|-----------|---------|--------------------------------------------|
| `bids`    | array   | Array of `[price, volume, timestamp]` tuples |
| `asks`    | array   | Array of `[price, volume, timestamp]` tuples |
| price     | string  | Price level (string decimal)               |
| volume    | string  | Aggregated volume at price level (string)  |
| timestamp | integer | Unix timestamp of last update              |

**Note:** REST returns string prices/volumes (not floats), sorted bids high→low, asks low→high.

---

## 3. Update Speed

- **Spot WebSocket v2:** No configurable update speed. Push-based — updates arrive as they occur. No documented frequency or aggregation window.
- **Futures WebSocket v1:** No configurable update speed. Push-based, per-event delta stream.
- **REST:** Poll-based; no push. Rate-limited by API call counter.

**No update speed (throttle interval) parameter exists on either WebSocket or REST for Kraken.**

**Heartbeat:** When subscribed to any WebSocket feed, a heartbeat is received at approximately 1 per second.

---

## 4. Price Aggregation

**None.** Kraken's `book` channel streams the raw L2 order book (individual price levels with aggregated volume per level). There is no configurable price aggregation (tick grouping) parameter on either the WebSocket or REST API.

The Spot REST API does have a separate endpoint labeled "Grouped Order Book" in the documentation navigation (referenced as distinct from `/public/Depth`), but full documentation on that endpoint was not available during this research.

---

## 5. Checksum

### Spot WebSocket v2 Only (Futures has NO checksum)

**Algorithm:** CRC32 (unsigned 32-bit integer)
**Coverage:** Top 10 price levels on each side ONLY, regardless of subscribed depth

#### Computation Steps

**Step 1: Format Asks (sorted lowest price to highest)**

For each of the top 10 ask levels:
1. Remove the decimal point from price: `"45285.2"` → `"452852"`
2. Remove all leading zeros from price: `"452852"` → `"452852"`
3. Remove the decimal point from quantity: `"0.00100000"` → `"000100000"`
4. Remove all leading zeros from quantity: `"000100000"` → `"100000"`
5. Concatenate price + quantity strings: `"452852100000"`
6. Append to the asks accumulator string

**Step 2: Format Bids (sorted highest price to lowest)**

Apply the identical formatting rules as asks (decimal removal, leading zero removal, concatenation).

**Step 3: Concatenate**

Combine the formatted asks string with the formatted bids string:
`checksum_input = asks_string + bids_string`

**Step 4: CRC32**

```
checksum = CRC32(checksum_input) as u32
```

#### Critical Implementation Notes

- **Float precision:** In WebSocket v2, prices and quantities are sent as floats. Precision must be determined from the `instrument` channel fields `price_precision` and `qty_precision` to reconstruct the exact decimal representation before stripping.
- **Parse as decimal, not float:** Use a decimal or string decoder during deserialization to preserve full precision before formatting.
- **Level removal:** Price levels with `"qty": 0` should be removed from the local book BEFORE computing the checksum.
- **Checksum is optional:** The documentation states checksum verification is optional but recommended.
- **Process all updates first:** Apply all updates in the message before computing the checksum.

#### Example

For ask price `"45285.2"` with qty `"0.00100000"`:
- Price string: `"452852"` (removed `.`)
- Qty string: `"100000"` (removed `.` and leading zeros)
- Contribution: `"452852100000"`

---

## 6. Sequence / Ordering

### Spot WebSocket v2 — `book` Channel

**NO explicit sequence numbers** on the public `book` channel.

- Messages contain only `timestamp` (RFC3339) for temporal context
- The documentation does NOT define a gap detection protocol for the book channel
- Recommended behavior on data gaps: unsubscribe and re-subscribe to receive a fresh snapshot
- Private feeds (`executions`, `openOrders`) DO have sequence numbers starting at 1 per connection per feed

**Gap detection strategy:** Use checksum. If computed checksum diverges from the received checksum after applying an update, the local book state is corrupt → re-subscribe.

### Futures WebSocket v1 — `book` Channel

**Has explicit `seq` field (positive integer)** on both snapshot and delta messages.

- Sequence is monotonically increasing per `product_id`
- Gap detection: if `received_seq != last_seq + 1` → sequence gap detected → re-subscribe
- Delta message carries only a single side/price/qty update per message

---

## 7. Spot vs Futures Differences (Summary Table)

| Feature                  | Spot WS v2                          | Futures WS v1                    |
|--------------------------|--------------------------------------|-----------------------------------|
| Endpoint                 | `wss://ws.kraken.com/v2`            | `wss://futures.kraken.com/ws/v1` |
| Channel name             | `book`                              | `book` (sub), `book_snapshot` (msg feed name) |
| Subscription field       | `channel`                           | `feed`                           |
| Symbol field             | `symbol` (e.g. `"BTC/USD"`)        | `product_id` (e.g. `"PI_XBTUSD"`) |
| Depth levels             | 10, 25, 100, 500, 1000 (selectable) | Full book (no discrete levels documented) |
| Default depth            | 10                                  | Not documented                   |
| Snapshot type            | `type: "snapshot"`                  | `feed: "book_snapshot"`          |
| Delta type               | `type: "update"` (batch, multi-level) | `feed: "book"` (single level per msg) |
| Sequence numbers         | None on book channel                | `seq` integer, per product        |
| Timestamp format         | RFC3339 string                      | Unix milliseconds (integer)      |
| Checksum                 | CRC32, top 10 levels                | None                             |
| Update speed config      | Not configurable                    | Not configurable                 |
| Price aggregation        | None                                | None                             |
| qty=0 means              | Remove price level                  | Remove price level               |

---

## 8. REST vs WebSocket Depth Comparison

| API                    | Depth Options                       | Max  |
|------------------------|-------------------------------------|------|
| Spot REST `GET /0/public/Depth` | `count` param, integer       | 100  |
| Spot WS v2 `book`      | 10, 25, 100, 500, 1000              | 1000 |
| Futures WS v1 `book`   | Full book (no depth filter documented) | Unlimited |

---

## Sources

- [Book (Level 2) — Spot WS v2](https://docs.kraken.com/api/docs/websocket-v2/book/)
- [Spot Websockets (v2) Book Checksum Guide](https://docs.kraken.com/api/docs/guides/spot-ws-book-v2/)
- [Book — Futures WS v1](https://docs.kraken.com/api/docs/futures-api/websocket/book/)
- [Get Order Book — Spot REST](https://docs.kraken.com/api/docs/rest-api/get-order-book/)
- [Spot Websockets Introduction](https://docs.kraken.com/api/docs/guides/spot-ws-intro/)
- [Futures Websockets Guide](https://docs.kraken.com/api/docs/guides/futures-websockets/)
- [Spot REST Rate Limits](https://docs.kraken.com/api/docs/guides/spot-rest-ratelimits/)
