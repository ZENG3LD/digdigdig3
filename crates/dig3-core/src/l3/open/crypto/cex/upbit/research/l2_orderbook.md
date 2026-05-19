# Upbit L2 / Orderbook Capabilities

> Researched: 2026-04-16
> Sources: official Upbit Developer Center (docs.upbit.com, global-docs.upbit.com)

---

## 1. WebSocket Channel

### Connection

| Item | Value |
|------|-------|
| URL (public / quotation) | `wss://api.upbit.com/websocket/v1` |
| URL (private / exchange) | `wss://api.upbit.com/websocket/v1/private` |
| Protocol | JSON over WebSocket (RFC 6455) |
| Compression | RFC 7692 permessage-deflate supported |
| Idle timeout | 120 seconds — send PING or `"PING"` text to keep alive |
| Server keepalive | Server pushes `{"status":"UP"}` every 10 seconds |

### Subscription Message Format

Requests are JSON arrays with three elements in order:

```json
[
  { "ticket": "<unique-id>" },
  {
    "type": "orderbook",
    "codes": ["KRW-BTC", "KRW-ETH"],
    "is_only_snapshot": false,
    "is_only_realtime": false
  },
  { "format": "DEFAULT" }
]
```

`format` options: `DEFAULT`, `SIMPLE`, `JSON_LIST`, `SIMPLE_LIST`

### Orderbook Channel Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | Yes | Must be `"orderbook"` |
| `codes` | array of string | Yes | Market codes, uppercase (e.g. `"KRW-BTC"`) |
| `level` | number | No | Price grouping/aggregation unit. KRW markets only. Default: 0 (no grouping). |
| `is_only_snapshot` | boolean | No | If `true`, receive only the initial snapshot, no realtime stream. Default: `false` |
| `is_only_realtime` | boolean | No | If `true`, skip snapshot, receive only realtime updates. Default: `false` |

### Depth Level Control (via codes notation)

Since **v1.1.0**, the number of orderbook levels can be controlled by appending `.{count}` to the market code:

```
"KRW-BTC.5"    → 5 levels per side
"KRW-ETH.15"   → 15 levels per side
```

Supported count values: **1, 5, 15, 30**
- Default (no suffix): **30 levels** per side
- Unsupported values fall back to 30

---

## 2. WebSocket Response Fields

### Outer Envelope

| Field | Abbreviation | Type | Description |
|-------|--------------|------|-------------|
| `type` | `ty` | string | Always `"orderbook"` |
| `code` | `cd` | string | Market code (e.g. `"KRW-BTC"`) |
| `timestamp` | `tms` | long (ms) | Server timestamp in milliseconds |
| `total_ask_size` | `tas` | double | Sum of all ask quantities across all levels |
| `total_bid_size` | `tbs` | double | Sum of all bid quantities across all levels |
| `orderbook_units` | `obu` | array | Array of price level objects (see below) |
| `level` | `lv` | number | Grouping unit applied (0 = default tick) |
| `stream_type` | `st` | string | `"SNAPSHOT"` or `"REALTIME"` |

### orderbook_units Entry

| Field | Abbreviation | Type | Description |
|-------|--------------|------|-------------|
| `ask_price` | `ap` | double | Ask (sell) price at this level |
| `bid_price` | `bp` | double | Bid (buy) price at this level |
| `ask_size` | `as` | double | Ask quantity at this level |
| `bid_size` | `bs` | double | Bid quantity at this level |

### Example Response

```json
{
  "type": "orderbook",
  "code": "KRW-BTC",
  "timestamp": 1746602359173,
  "total_ask_size": 0.68780013,
  "total_bid_size": 0.78754733,
  "orderbook_units": [
    { "ask_price": 125056000.0, "bid_price": 124743000.0, "ask_size": 0.17, "bid_size": 0.17 },
    { "ask_price": 125207000.0, "bid_price": 124332000.0, "ask_size": 0.09, "bid_size": 0.09 }
  ],
  "level": 0,
  "stream_type": "SNAPSHOT"
}
```

---

## 3. REST Orderbook Endpoint

### Endpoint

```
GET https://api.upbit.com/v1/orderbook
GET https://{region}-api.upbit.com/v1/orderbook   (Global API)
```

### Parameters

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `markets` | string | Yes | — | Comma-separated market codes, e.g. `"KRW-BTC,KRW-ETH"` |
| `count` | integer | No | 30 | Number of orderbook levels to return. Maximum: **30** |
| `level` | string/number | No | 0 | Price grouping unit. KRW markets only. 0 = no grouping |

### Response Fields

Same structure as WebSocket response. Array of objects, one per market:

```json
[
  {
    "market": "KRW-BTC",
    "timestamp": 1746602359000,
    "total_ask_size": 1.23,
    "total_bid_size": 2.34,
    "orderbook_units": [
      { "ask_price": 125000000, "bid_price": 124900000, "ask_size": 0.1, "bid_size": 0.2 }
    ],
    "level": 0
  }
]
```

### Rate Limits

- **10 calls per second per IP** (shared within the orderbook group)

---

## 4. Orderbook Instruments Endpoint

Used to discover supported price aggregation levels per market.

```
GET https://api.upbit.com/v1/orderbook/supported_levels
```

### Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `markets` | string | Yes | Comma-separated market codes |

### Response Fields (Korean API)

```json
[
  {
    "market": "KRW-BTC",
    "quote_currency": "KRW",
    "tick_size": "1000",
    "supported_levels": [0, 10000, 100000, 1000000, 10000000, 100000000]
  }
]
```

### Response Fields (Global API)

```json
[
  {
    "market": "SGD-BTC",
    "quote_currency": "SGD",
    "tick_size": "1"
  }
]
```

Note: `supported_levels` array only present in Korean KRW API. Global API returns only `tick_size`.

---

## 5. Snapshot vs Delta

| Item | Detail |
|------|--------|
| Update model | **Full snapshot on every update** — no delta/incremental messages |
| Initial message | `stream_type: "SNAPSHOT"` — full orderbook state |
| Subsequent messages | `stream_type: "REALTIME"` — still full snapshots, not diffs |
| Delta support | **None** — Upbit does not offer incremental order book updates |

Each message is a complete replacement of the orderbook state. Consumers must overwrite the previous book entirely on each message.

---

## 6. Update Speed

- **Not configurable** — no parameter to control update frequency
- No documented millisecond interval (e.g. no 100ms / 500ms / 1000ms variants)
- Updates are pushed as they occur on the exchange side
- Frequency depends on market activity

---

## 7. Price Aggregation (Level Grouping)

| Item | Detail |
|------|--------|
| Feature name | `level` (호가 모아보기, "orderbook grouping") |
| Where available | **KRW markets only** on Korean API |
| Global API | **Not supported** (no `supported_levels` field) |
| Default | `level: 0` = standard tick size (no aggregation) |
| KRW-BTC example | Supported levels: `[0, 10000, 100000, 1000000, 10000000, 100000000]` |
| Invalid level behavior | Returns empty list |
| Discovery | Use `/v1/orderbook/supported_levels` to query per-market supported levels |

In WebSocket: pass `level` as a parameter in the subscription object alongside `codes`. In REST: pass `level` as query parameter.

---

## 8. Checksum

**No checksum mechanism** — Upbit does not provide any orderbook checksum field.

Since messages are full snapshots (not deltas), there is no need to detect gaps; each message stands alone. Validation is not supported beyond timestamp ordering.

---

## 9. Sequence Numbers / Gap Detection

**No sequence number field.** Ordering relies solely on:

- `timestamp` (milliseconds) in each message

Because the update model is full-snapshot, gap detection is unnecessary — each message is self-contained. There is no `lastUpdateId`, `sequence`, or `u`/`U` style fields like Binance.

---

## 10. Symbol Format

| Market | Format | Example |
|--------|--------|---------|
| Korean won | `KRW-{BASE}` | `KRW-BTC`, `KRW-ETH` |
| Bitcoin market | `BTC-{BASE}` | `BTC-ETH`, `BTC-XRP` |
| USDT market | `USDT-{BASE}` | `USDT-BTC` |
| Global (SGD) | `SGD-{BASE}` | `SGD-BTC`, `SGD-ETH` |

Codes in WebSocket `codes` field must be **uppercase**.

---

## 11. Authentication

- **Public orderbook data**: No authentication required. A `ticket` field must be present in WebSocket subscription (any unique string), but no API key needed.
- **Private exchange WebSocket**: JWT bearer token in `Authorization` header.

---

## 12. Summary Table

| Capability | Value |
|------------|-------|
| WS channel name | `orderbook` |
| WS endpoint | `wss://api.upbit.com/websocket/v1` |
| REST endpoint | `GET /v1/orderbook` |
| Max depth levels | **30** per side |
| Depth control (WS) | `{code}.{count}` suffix; values: 1, 5, 15, 30 |
| Depth control (REST) | `count` param, max 30 |
| Update model | Full snapshot every tick |
| Delta updates | **No** |
| Snapshot field | `stream_type: "SNAPSHOT"` / `"REALTIME"` |
| Update speed | Not configurable |
| Price aggregation | `level` param — KRW markets only |
| Checksum | **No** |
| Sequence number | **No** |
| Timestamp field | `timestamp` (ms epoch) |
| Rate limit (REST) | 10 req/s per IP |
| Symbol format | `{QUOTE}-{BASE}` uppercase |

---

## Sources

- [Upbit WebSocket Orderbook (Global)](https://global-docs.upbit.com/reference/websocket-orderbook)
- [Upbit REST Orderbook (Global)](https://global-docs.upbit.com/reference/list-orderbooks)
- [Upbit Orderbook Instruments (Global)](https://global-docs.upbit.com/reference/list-orderbook-instruments)
- [Upbit REST Orderbook (Korean)](https://docs.upbit.com/reference/list-orderbooks)
- [Upbit WebSocket Guide](https://docs.upbit.com/reference/websocket-guide)
- [Tardis.dev Upbit historical data details](https://docs.tardis.dev/historical-data-details/upbit)
