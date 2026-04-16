# MEXC L2 / Orderbook Capabilities

**Source docs:**
- Spot v3: https://mexcdevelop.github.io/apidocs/spot_v3_en/ / https://www.mexc.com/api-docs/spot-v3/
- Futures v1: https://mexcdevelop.github.io/apidocs/contract_v1_en/ / https://www.mexc.com/api-docs/futures/
- Protobuf schemas: https://github.com/mexcdevelop/websocket-proto

---

## 1. REST Order Book Endpoints

### Spot — `GET /api/v3/depth`

| Parameter | Type     | Required | Values / Notes                    |
|-----------|----------|----------|-----------------------------------|
| `symbol`  | string   | YES      | e.g. `BTCUSDT`                    |
| `limit`   | integer  | NO       | Default: 100. Max: **5000**       |

**Response:**
```json
{
  "lastUpdateId": 12345678,
  "bids": [["41234.50", "0.123"], ...],
  "asks": [["41235.00", "0.456"], ...]
}
```

- `bids` / `asks`: `[price_string, quantity_string]`
- `lastUpdateId`: used to align with WS stream (maps to `version`/`toVersion`)
- **No explicit valid-values list** for `limit` beyond default=100, max=5000

**Rate limit:** Weight 1 (IP-based)

---

### Futures — `GET /api/v1/contract/depth/{symbol}`

| Parameter | Type   | Required | Values / Notes               |
|-----------|--------|----------|------------------------------|
| `symbol`  | string | YES      | e.g. `BTC_USDT`              |
| `limit`   | int    | NO       | "Depth tier" — doc does not enumerate valid values explicitly |

**Response:**
```json
{
  "asks": [[6859.5, 3251, 1], ...],
  "bids": [[6859.4, 179, 4], ...],
  "version": 96801927,
  "timestamp": 1587442022003
}
```

- Each entry: `[price, order_count, quantity]` (3-element array, differs from spot's 2-element)
- `version`: monotonically increasing integer — used for WS gap detection
- **Rate limit:** 20 requests per 2 seconds

---

### Futures — `GET /api/v1/contract/depth_commits/{symbol}/{limit}`

Packet loss recovery endpoint. Returns latest N incremental depth commits sorted ascending by version.

| Path segment | Notes                                   |
|--------------|-----------------------------------------|
| `{symbol}`   | e.g. `BTC_USDT`                         |
| `{limit}`    | e.g. `1000` (max verified value in docs)|

**Response:**
```json
{
  "success": true,
  "code": 0,
  "data": [
    {"asks": [], "bids": [[3818.91, 272, 1]], "version": 26457599299},
    ...
  ]
}
```

Use case: when WS version gap detected, fetch these commits, skip all with `version <= localLastVersion`, then apply sequentially from `version == localLastVersion + 1`.

---

## 2. WebSocket Channels

### Spot WebSocket

**Endpoint:** `wss://wbs-api.mexc.com/ws`

**Subscription format:**
```json
{"method": "SUBSCRIPTION", "params": ["<channel>"]}
```

**Unsubscription format:**
```json
{"method": "UNSUBSCRIPTION", "params": ["<channel>"]}
```

**Limits:** Max 30 subscriptions per connection. Connection valid up to 24 hours. Disconnect if no data for 60s.

**Keepalive:** Send `{"method": "ping"}` every 30 seconds.

#### Spot Depth Channels (3 variants)

| Channel pattern | Type | Update speed | Levels | Notes |
|----------------|------|-------------|--------|-------|
| `spot@public.increase.depth.v3.api.pb@{SYMBOL}` | **Incremental delta** (protobuf) | Not specified explicitly | Full depth | fromVersion/toVersion for gap detection |
| `spot@public.limit.depth.v3.api.pb@{SYMBOL}@{LEVEL}` | **Snapshot** (protobuf) | Not specified explicitly | 5, 10, or 20 | Periodic limited-level snapshot |
| `spot@public.aggre.depth.v3.api.pb@{100ms\|10ms}@{SYMBOL}` | **Aggregated incremental** (protobuf) | **100ms or 10ms** | Full depth | Has price aggregation; fromVersion+toVersion |

**Note on `aggre.depth`:** The `@100ms` or `@10ms` suffix in the channel name selects the update speed. This is the only update-speed-configurable channel.

**Note on naming:** Non-protobuf JSON variants also exist without the `.pb` suffix (e.g. `spot@public.increase.depth.v3.api@{SYMBOL}`), used in older integrations.

#### Spot Depth Protobuf Structures

**`PublicIncreaseDepthsV3Api`** (incremental delta):
```protobuf
message PublicIncreaseDepthsV3Api {
  repeated PublicIncreaseDepthV3ApiItem asks = 1;
  repeated PublicIncreaseDepthV3ApiItem bids = 2;
  string eventType = 3;
  string version   = 4;   // absolute version of this snapshot level
}
message PublicIncreaseDepthV3ApiItem {
  string price    = 1;
  string quantity = 2;
}
```

**`PublicLimitDepthsV3Api`** (level snapshot):
```protobuf
message PublicLimitDepthsV3Api {
  repeated PublicLimitDepthV3ApiItem asks = 1;
  repeated PublicLimitDepthV3ApiItem bids = 2;
  string eventType = 3;
  string version   = 4;
}
```

**`PublicAggreDepthsV3Api`** (aggregated incremental):
```protobuf
message PublicAggreDepthsV3Api {
  repeated PublicAggreDepthV3ApiItem asks = 1;
  repeated PublicAggreDepthV3ApiItem bids = 2;
  string eventType    = 3;
  string fromVersion  = 4;   // first version in this batch
  string toVersion    = 5;   // last version in this batch
}
```

**`PushDataV3ApiWrapper`** (outer envelope):
```protobuf
message PushDataV3ApiWrapper {
  string  channel   = 1;
  oneof body {
    // fields 301-315
    PublicIncreaseDepthsV3Api      increaseDepths     = 302;
    PublicLimitDepthsV3Api         limitDepths        = 303;
    PublicAggreDepthsV3Api         aggreDepths        = 313;
    PublicIncreaseDepthsBatchV3Api increaseDepthBatch = 312;
    // ... other message types
  }
  optional string symbol    = 3;
  optional string symbolId  = 4;
  optional int64 createTime = 5;
  optional int64 sendTime   = 6;
}
```

**`PublicIncreaseDepthsBatchV3Api`** (multi-symbol batch):
```protobuf
message PublicIncreaseDepthsBatchV3Api {
  repeated PublicIncreaseDepthsV3Api items = 1;
  string eventType = 2;
}
```

---

### Futures WebSocket

**Endpoint:** `wss://contract.mexc.com/edge`

**Subscription format:**
```json
{"method": "sub.depth",      "param": {"symbol": "BTC_USDT"}}
{"method": "sub.depth.full", "param": {"symbol": "BTC_USDT", "limit": 20}}
```

**Keepalive:** Send ping every 10–20 seconds. Disconnects after 60s of inactivity.

**Data format:** JSON (default), with optional gzip compression.

#### Futures Depth Channels

| Method (subscribe) | Push channel | Type | Update freq | Notes |
|-------------------|-------------|------|------------|-------|
| `sub.depth` | `push.depth` | **Incremental delta** | ~200ms | Default compress=true (since 2025-04-09) |
| `sub.depth.full` | `push.depth` | **Full snapshot** then incremental | ~200ms | Sends full snapshot on subscribe + deltas after |
| `sub.depth.step` | `push.depth.step` | **Stepped/aggregated** | ~200ms | Aggregated by step size; includes market-level price info |

**Sub.depth.full parameters:**

| Param | Type | Values | Default |
|-------|------|--------|---------|
| `symbol` | string | e.g. `BTC_USDT` | required |
| `limit` | int | 5, 10, 20 | 20 |

**Sub.depth compress parameter** (incremental):
```json
{"method": "sub.depth", "param": {"symbol": "BTC_USDT", "compress": false}}
```
- `compress: true` (default since 2025-04-09) — merges/aggregates incremental data
- `compress: false` — raw unmerged increments

#### Futures Push Response Format

**`push.depth`** (incremental):
```json
{
  "channel": "push.depth",
  "data": {
    "asks": [[6859.5, 3251, 1]],
    "bids": [],
    "version": 96801927
  },
  "symbol": "BTC_USDT",
  "ts": 1587442022003
}
```

- `asks`/`bids`: `[price, order_count, quantity]`
- `version`: monotonically increasing; each message = `prev_version + 1` under normal conditions
- `ts`: server timestamp in milliseconds
- Quantity = 0 → remove that price level from local book

---

## 3. Update Speed

| Market | Channel | Speed | Configurable? |
|--------|---------|-------|---------------|
| Spot | `aggre.depth` | **10ms or 100ms** | YES — suffix in channel name |
| Spot | `increase.depth` | Not documented explicitly | NO |
| Spot | `limit.depth` | Not documented explicitly | NO |
| Futures | `push.depth` | ~200ms | NO |
| Futures | `push.depth.step` | ~200ms | NO |

The only configurable speed is on the spot `aggre.depth` channel via the `@10ms` or `@100ms` suffix.

---

## 4. Price Aggregation

| Market | Channel | Aggregation |
|--------|---------|-------------|
| Spot | `aggre.depth.v3.api.pb@100ms` or `@10ms` | YES — price levels merged/aggregated. No user-configurable tick-size in docs. |
| Futures | `push.depth.step` | YES — "stepped" by minimum notional step. Includes `bidMarketLevelPrice` / `askMarketLevelPrice`. |
| Futures | `push.depth` with `compress: true` | Partial aggregation/merging of incremental updates. |

No documentation found for a user-specified tick-size / price precision parameter on any channel.

---

## 5. Checksum

**No checksum field found** in any documented MEXC orderbook channel (spot or futures). The protobuf schemas and JSON response examples contain no checksum or CRC field.

---

## 6. Sequence / Ordering

### Spot

Uses `fromVersion` / `toVersion` string fields (in `aggre.depth`) and `version` string (in `increase.depth` / `limit.depth`).

**Gap detection rule** (from official docs):
- `fromVersion` of next message must equal `toVersion + 1` of previous message
- If not: **packet loss** — must reinitialize from snapshot

**Snapshot alignment:**
1. Subscribe to WS, buffer updates, note `fromVersion` of first message
2. Fetch REST snapshot: `GET /api/v3/depth?symbol={SYMBOL}&limit=5000`, note `lastUpdateId`
3. If `lastUpdateId < fromVersion` of first buffered update → fetch snapshot again
4. Discard buffered updates where `toVersion <= lastUpdateId`
5. Apply all remaining updates in order

**Quantity semantics:** Values are **absolute** (not deltas). `quantity = 0` means remove that level.

### Futures

Uses integer `version` field (long).

**Gap detection rule:**
- Each message: `version == prev_version + 1`
- Gap detected → use `depth_commits` REST endpoint for recovery

**Snapshot alignment:**
1. Fetch `GET /api/v1/contract/depth/{symbol}?limit=1000`, save `version` as `localLastVersion`
2. Subscribe to `sub.depth`
3. On each `push.depth` message: if `version > localLastVersion` → apply update
4. On gap: fetch `GET /api/v1/contract/depth_commits/{symbol}/1000`
5. Skip commits with `version <= localLastVersion`
6. Apply sequentially from `version == localLastVersion + 1`

---

## 7. Spot vs Futures — Protocol Differences

| Feature | Spot (v3) | Futures/Contract (v1) |
|---------|-----------|----------------------|
| WS endpoint | `wss://wbs-api.mexc.com/ws` | `wss://contract.mexc.com/edge` |
| Serialization | **Protobuf** (`.pb` channels) OR JSON | **JSON** (+ optional gzip) |
| Subscribe method | `SUBSCRIPTION` with `params` array | `sub.<channel>` with `param` object |
| Depth entry format | `[price_str, qty_str]` (2 fields) | `[price, order_count, qty]` (3 fields) |
| Version type | **String** (`"version"`, `"fromVersion"`, `"toVersion"`) | **Integer/Long** (`version`) |
| Sequence mechanism | `fromVersion` + `toVersion` pair per batch | Single `version` per message (+1 monotonic) |
| Snapshot on subscribe | No (must fetch REST manually) | Optional via `sub.depth.full` |
| Packet loss recovery | Re-fetch REST snapshot | `depth_commits` REST endpoint |
| Update speed | 10ms / 100ms (aggre.depth only) | ~200ms fixed |
| Compression | Protobuf binary encoding | Optional gzip (`compress` param) |
| Aggregated depth | `aggre.depth` channel | `push.depth.step` channel |
| Max REST depth | 5000 levels | Not enumerated in docs |
| Batch multi-symbol | `PublicIncreaseDepthsBatchV3Api` | Not documented |

---

## 8. WebSocket Connection Limits (Spot)

- Max 30 subscriptions per connection
- Connection max lifetime: 24 hours
- Disconnect if no messages for 60 seconds
- Disconnect if no subscription within 30 seconds of connect
- Ping/pong: send `{"method": "ping"}` every 30s

---

## Sources

- [MEXC Spot API v3 — Introduction (GitHub Pages)](https://mexcdevelop.github.io/apidocs/spot_v3_en/)
- [MEXC Spot API — Websocket Market Streams](https://www.mexc.com/api-docs/spot-v3/websocket-market-streams)
- [MEXC Spot API — Market Data Endpoints](https://www.mexc.com/api-docs/spot-v3/market-data-endpoints)
- [MEXC Contract API v1 — Introduction (GitHub Pages)](https://mexcdevelop.github.io/apidocs/contract_v1_en/)
- [MEXC Futures API — WebSocket API](https://www.mexc.com/api-docs/futures/websocket-api)
- [MEXC Futures API — Market Endpoints](https://www.mexc.com/api-docs/futures/market-endpoints)
- [MEXC WebSocket Protobuf Schemas](https://github.com/mexcdevelop/websocket-proto)
- [MEXC V3 WebSocket Service Replacement Announcement](https://www.mexc.com/announcements/article/mexc-v3-websocket-service-replacement-announcement-17827791522393)
- [CCXT MEXC Docs](https://docs.ccxt.com/exchanges/mexc)
