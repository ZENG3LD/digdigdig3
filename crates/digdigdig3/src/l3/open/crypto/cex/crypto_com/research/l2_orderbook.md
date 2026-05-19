# Crypto.com Exchange L2 Orderbook Capabilities

**Source**: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html
**API Version**: Exchange v1 (unified Spot + Derivatives)
**Research Date**: 2026-04-16

---

## 1. WebSocket Channels

### Channel Name Format

```
book.{instrument_name}.{depth}
```

**Examples:**
- `book.BTCUSD-PERP.10`
- `book.BTC_USDT.50`
- `book.ETH_USDT.150`

**Deprecated (removed):**
- `book.{instrument_name}` (no explicit depth) — removed; default depth was 50 during transition

### Subscription Request Format

```json
{
  "id": 1,
  "method": "subscribe",
  "params": {
    "channels": ["book.BTCUSD-PERP.10"],
    "book_subscription_type": "SNAPSHOT_AND_UPDATE"
  },
  "nonce": 1587523073344
}
```

The `book_subscription_type` parameter is passed inside `params` alongside `channels`.

### Valid Depth Levels

| Depth | Notes |
|-------|-------|
| `10`  | Confirmed valid (used in REST examples) |
| `50`  | Former default; confirmed valid (transition default) |
| `150` | Confirmed valid (referenced in legacy Spot v2.x docs, carries over) |

**Note:** The documentation states REST `depth` is "up to 50" for `public/get-book`. For WebSocket, both `10` and `150` are explicitly referenced in documentation. The exact full set of allowed integer values for WS is not exhaustively enumerated in official docs — confirmed values are `10`, `50`, `150`.

---

## 2. Subscription Types (Snapshot vs Delta)

### `book_subscription_type` Parameter Values

| Value | Mode | Frequency | Notes |
|-------|------|-----------|-------|
| `SNAPSHOT` | Full snapshots only | **500ms** | Delivers complete book at each interval |
| `SNAPSHOT_AND_UPDATE` | Snapshot + delta updates | **100ms** | Recommended; delivers snapshot then incremental deltas |

**Breaking change (2025-02-27 08:00 UTC):**
- `SNAPSHOT` at 100ms was removed. Now `SNAPSHOT` = 500ms only.
- Users who need 100ms must use `SNAPSHOT_AND_UPDATE` (delta mode).
- During transition, users on removed 100ms snapshot received 500ms instead.

### Delta Mode Behavior

- On subscribe: receives a full snapshot first
- Subsequent messages: incremental delta updates at 100ms
- When book has no changes: empty delta is sent (instead of the previous fixed 500ms heartbeat snapshot)
- "The fixed 500ms delta full book snapshot heartbeat is replaced with empty delta in the case of no book changes"

### Snapshot Subscription Response Fields (result.data array)

```json
{
  "id": -1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "instrument_name": "BTCUSD-PERP",
    "subscription": "book.BTCUSD-PERP.50",
    "channel": "book",
    "depth": 50,
    "data": [
      {
        "bids": [
          ["50113.500000", "0.400000", "0"],
          ["50113.000000", "0.100000", "0"]
        ],
        "asks": [
          ["50126.000000", "0.400400", "0"],
          ["50130.000000", "1.279000", "0"]
        ],
        "t": 1587523234180,
        "tt": 1587523234171,
        "u": 408159616,
        "cs": 1021955186
      }
    ]
  }
}
```

**Note on subscription field:** As of a changelog update, the `subscription` field is now explicit with depth — e.g., `"subscription": "book.BTC_USD.50"` instead of the old `"book.BTC_USD"`.

---

## 3. Response Data Fields

### `data[]` Object Fields

| Field | Type | Description |
|-------|------|-------------|
| `bids` | array | Array of bid levels: `[price, quantity, num_orders]` (all strings) |
| `asks` | array | Array of ask levels: `[price, quantity, num_orders]` (all strings) |
| `t`   | integer | Exchange timestamp of the update (milliseconds) |
| `tt`  | integer | Timestamp at which the trade/update was applied (nanoseconds or ms — exact unit TBD from docs) |
| `u`   | integer | Update sequence number — monotonically increasing; used for ordering and gap detection |
| `cs`  | integer | Checksum — CRC32 value for order book integrity verification |

**Known issue:** The third element in each bid/ask entry (`num_orders`) currently returns `0` — documented as a known limitation.

### Price/Quantity Format

All numeric values are returned as **strings** (wrapped in double quotes), e.g. `"50113.500000"`. This applies to both REST and WebSocket responses.

### Array Format per Level

```
[price_string, quantity_string, num_orders_string]
```

Best bid/ask are first in their respective arrays.

---

## 4. Update Speed

| Mode | Speed |
|------|-------|
| `SNAPSHOT` | 500ms |
| `SNAPSHOT_AND_UPDATE` (delta) | 100ms |

**Not configurable** beyond these two modes. There is no per-connection speed tuning.

---

## 5. Price Aggregation

No price aggregation is documented for the `book.{instrument_name}.{depth}` channel. The exchange delivers raw order book levels without price grouping/bucketing options in the API.

---

## 6. Checksum

**Field:** `cs` (integer, CRC32)

- Present in both snapshot and delta messages
- Allows clients to verify local book state integrity after applying updates
- Checksum validation is optional on the client side
- The exact algorithm (which levels are included, byte ordering) is documented in the official docs but was not fully extractable during research — the pattern follows CRC32 over top N bid/ask levels (common pattern: top 25 bids + 25 asks concatenated as `price:qty|price:qty|...`)
- Third-party library `cryptofeed` notes Crypto.com as "Snapshots only" for validation (as of their audit), meaning checksum/sequence-gap recovery may be less critical if using snapshot mode

---

## 7. Sequence Numbers / Ordering

**Field:** `u` (update ID / sequence number)

- Monotonically increasing integer
- Present in each orderbook message
- Used to detect missed/out-of-order updates in delta mode
- **Gap handling:** If a gap is detected in `u` values, clients should re-subscribe to get a fresh snapshot
- Documentation mentions "book delta sequence number handling and re-subscription" was clarified in a changelog update

**Field ordering in outer message** (consistent across all market data subscriptions):
```
id → method → code → instrument_name → subscription → channel → ...
```

---

## 8. REST Endpoint

### `public/get-book`

**URL:** `GET https://api.crypto.com/exchange/v1/public/get-book`

**Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | Yes | e.g., `BTCUSD-PERP`, `BTC_USDT` |
| `depth` | integer | Yes | Number of bids and asks to return; documented "up to 50"; values `10` confirmed in examples |

**Response:**

```json
{
  "code": 0,
  "method": "public/get-book",
  "result": {
    "instrument_name": "BTCUSD-PERP",
    "depth": 10,
    "data": [
      {
        "bids": [["50113.500000", "0.400000", "0"], ...],
        "asks": [["50126.000000", "0.400400", "0"], ...],
        "t": 1587523234180,
        "tt": 1587523234171,
        "u": 408159616,
        "cs": 1021955186
      }
    ]
  }
}
```

**Depth limit for REST:** "up to 50" per documentation. Requesting larger depths (e.g., 150) may not be supported on the REST endpoint even if valid on WebSocket.

---

## 9. Spot vs Derivatives Differences

The Exchange v1 API is a **unified API** for both Spot and Derivatives in a single wallet system. There is no separate book channel for spot vs. derivatives — both use the same `book.{instrument_name}.{depth}` channel format.

**Instrument naming conventions differ:**
- Spot instruments: `BTC_USDT`, `ETH_USDT` (underscore-separated pair)
- Perpetual futures: `BTCUSD-PERP`, `ETHUSD-PERP`
- Dated futures: `BTCUSD-230331` (date-suffixed)

**Behavior is identical** for both categories. The Exchange v1 API was created by merging the old Derivatives v1 API with new Spot capabilities — it is a superset of the old Derivatives API.

**Legacy note:** The old Spot v2.x API used depth values of `10` or `150`. The old Derivatives API (pre-v1) documented "up to 50". Exchange v1 inherits both patterns.

---

## 10. WebSocket Connection Details

**Market Data WebSocket URL:**
```
wss://stream.crypto.com/exchange/v1/market
```

**User API WebSocket URL** (for authenticated subscriptions):
```
wss://stream.crypto.com/exchange/v1/user
```

**Heartbeat:** Server sends heartbeat every 30 seconds; client must respond with `public/respond-heartbeat` using matching `id` within 5 seconds.

**Rate limits:** WebSocket rate limits are pro-rated based on the calendar-second the connection was opened. Recommended: add 1-second sleep before sending requests after connect.

---

## 11. Summary Table

| Capability | Value |
|------------|-------|
| WS channel format | `book.{instrument_name}.{depth}` |
| Valid depths (WS) | `10`, `50`, `150` (confirmed) |
| Valid depths (REST) | Up to `50` per docs; `10` in examples |
| Subscription modes | `SNAPSHOT` (500ms), `SNAPSHOT_AND_UPDATE` (100ms delta) |
| Default depth (deprecated) | `50` |
| Snapshot speed | 500ms |
| Delta speed | 100ms |
| Checksum field | `cs` (CRC32 integer) |
| Sequence field | `u` (update ID, monotonic) |
| Timestamp field | `t` (ms), `tt` (ns or ms) |
| Price format | String |
| 3rd array element (num_orders) | Always `"0"` (known issue) |
| Price aggregation | Not available |
| Speed configurable | No (fixed to 2 modes) |
| Spot vs Derivatives | Same channel, different instrument names |
| WS URL | `wss://stream.crypto.com/exchange/v1/market` |

---

## Sources

- [Crypto.com Exchange v1 API Documentation](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Crypto.com Exchange Institutional API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index-insto-8556ea5c-4dbb-44d4-beb0-20a4d31f63a7.html)
- [Crypto.com Exchange Derivatives API](https://exchange-docs.crypto.com/derivatives/index.html)
- [Tardis.dev Crypto.com Data Details](https://docs.tardis.dev/historical-data-details/crypto-com)
- [cryptofeed Book Validation Docs](https://github.com/bmoscon/cryptofeed/blob/master/docs/book_validation.md)
