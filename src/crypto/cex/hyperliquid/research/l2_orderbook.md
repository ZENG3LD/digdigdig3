# HyperLiquid L2 Orderbook API Capabilities

Researched: 2026-04-16
Source: Official HyperLiquid GitBook docs + Chainstack, Dwellir, QuickNode references

---

## 1. REST Endpoint — l2Book

### Endpoint

```
POST https://api.hyperliquid.xyz/info
Content-Type: application/json
```

Testnet: `https://api.hyperliquid-testnet.xyz/info`

### Request Body

```json
{
  "type": "l2Book",
  "coin": "BTC",
  "nSigFigs": 5,
  "mantissa": null
}
```

| Field       | Type    | Required | Description |
|-------------|---------|----------|-------------|
| `type`      | string  | Yes      | Must be `"l2Book"` |
| `coin`      | string  | Yes      | Asset identifier — perps: `"BTC"`, `"ETH"` etc; spot: `"@{index}"` or `"PURR/USDC"` |
| `nSigFigs`  | integer | No       | Price aggregation: `2`, `3`, `4`, `5`, or `null` (full precision) |
| `mantissa`  | integer | No       | Only allowed when `nSigFigs == 5`. Values: `1`, `2`, `5` |

### Response Structure

```json
{
  "coin": "BTC",
  "time": 1754450974231,
  "levels": [
    [
      { "px": "113377.0", "sz": "7.6699", "n": 17 },
      { "px": "113376.0", "sz": "2.1000", "n": 5 }
    ],
    [
      { "px": "113397.0", "sz": "0.11543", "n": 3 },
      { "px": "113398.0", "sz": "1.5000", "n": 8 }
    ]
  ]
}
```

| Field    | Type   | Description |
|----------|--------|-------------|
| `coin`   | string | Asset identifier |
| `time`   | integer | Snapshot timestamp in milliseconds |
| `levels` | array  | Two-element array: `[bids, asks]` |
| `levels[0]` | array | Bids sorted descending (best first = highest price) |
| `levels[1]` | array | Asks sorted ascending (best first = lowest price) |
| `px`     | string | Price at this level |
| `sz`     | string | Total aggregated size at this price |
| `n`      | integer | Number of individual orders at this price level |

### Depth Limit

**Maximum 20 levels per side** (bids and asks each). This limit is fixed and cannot be changed via REST.

---

## 2. WebSocket Channel — l2Book

### Connection URL

```
wss://api.hyperliquid.xyz/ws          (mainnet)
wss://api.hyperliquid-testnet.xyz/ws  (testnet)
```

### Subscribe

```json
{
  "method": "subscribe",
  "subscription": {
    "type": "l2Book",
    "coin": "BTC",
    "nSigFigs": 5,
    "mantissa": null
  }
}
```

### Unsubscribe

```json
{
  "method": "unsubscribe",
  "subscription": {
    "type": "l2Book",
    "coin": "BTC",
    "nSigFigs": 5,
    "mantissa": null
  }
}
```

### Subscription Parameters

| Field      | Type    | Required | Description |
|------------|---------|----------|-------------|
| `type`     | string  | Yes      | `"l2Book"` |
| `coin`     | string  | Yes      | Asset identifier (same format as REST) |
| `nSigFigs` | integer | No       | `2`, `3`, `4`, `5`, or omit for full precision |
| `mantissa` | integer | No       | Only when `nSigFigs == 5`; values: `1`, `2`, `5` |

### Message Format (Incoming)

```json
{
  "channel": "l2Book",
  "data": {
    "coin": "BTC",
    "time": 1754450974231,
    "levels": [
      [
        { "px": "113377.0", "sz": "7.6699", "n": 17 }
      ],
      [
        { "px": "113397.0", "sz": "0.11543", "n": 3 }
      ]
    ]
  }
}
```

The `WsBook` type in the official TypeScript SDK:

```typescript
type WsBook = {
  coin: string;
  levels: [Array<WsLevel>, Array<WsLevel>];
  time: number;
}

type WsLevel = {
  px: string;   // price
  sz: string;   // size
  n: number;    // order count
}
```

### Update Model

- **Always full snapshots** — every message contains the complete current state of the book (up to 20 levels per side)
- **No delta/diff updates** — no incremental patches, no need for local book reconstruction
- **Push frequency**: "Snapshot feed, pushed on each block that is at least 0.5 seconds since last push"
- **Minimum interval**: ~500ms between pushes (block-rate limited)
- **On reconnect**: The initial message after subscription is a full snapshot; any data missed during disconnection is captured in this snapshot

---

## 3. WebSocket l2Book via POST Request

You can also query l2Book as a one-shot request over an existing WebSocket connection (instead of a subscription):

```json
{
  "method": "post",
  "id": 123,
  "request": {
    "type": "info",
    "payload": {
      "type": "l2Book",
      "coin": "ETH",
      "nSigFigs": 5,
      "mantissa": null
    }
  }
}
```

Response arrives on the `"post"` channel:

```json
{
  "channel": "post",
  "data": {
    "id": 123,
    "response": {
      "type": "info",
      "payload": { /* same as REST l2Book response */ }
    }
  }
}
```

---

## 4. Price Aggregation — nSigFigs and Mantissa

When `nSigFigs` is set, prices are bucketed/rounded to that many significant figures. This reduces the number of distinct levels returned and is useful for coarser orderbook views.

### nSigFigs Values

| Value  | Description |
|--------|-------------|
| `null` | Full precision — each distinct price level reported individually |
| `2`    | 2 significant figures (most aggressive aggregation) |
| `3`    | 3 significant figures |
| `4`    | 4 significant figures |
| `5`    | 5 significant figures (finest aggregation with bucketing) |

### Mantissa (only when nSigFigs == 5)

When `nSigFigs` is `5`, the `mantissa` parameter controls the multiplier for price bucketing:

| Value | Effect |
|-------|--------|
| `1`   | Bucket by 1x unit |
| `2`   | Bucket by 2x unit |
| `5`   | Bucket by 5x unit |
| `null`| Default (no mantissa adjustment) |

**Example**: For BTC at ~$113,000 with `nSigFigs=5`, prices are reported to 5 sig figs (~$1 buckets). With `mantissa=5`, they are grouped into $5 buckets.

If `nSigFigs` is set to 2–4, the `mantissa` field is NOT allowed (will cause an error).

---

## 5. BBO Channel (Best Bid/Offer)

A lighter alternative to l2Book for just the top-of-book:

### Subscribe

```json
{
  "method": "subscribe",
  "subscription": {
    "type": "bbo",
    "coin": "BTC"
  }
}
```

### Response Format (`WsBbo`)

```json
{
  "channel": "bbo",
  "data": {
    "coin": "BTC",
    "time": 1754450974231,
    "levels": [
      { "px": "113377.0", "sz": "7.6699", "n": 17 },
      { "px": "113397.0", "sz": "0.11543", "n": 3 }
    ]
  }
}
```

`levels` is a 2-element array: `[best_bid, best_ask]`. Either can be `null` if no orders exist on that side.

---

## 6. Spot vs Perps Differences

### Coin Naming Convention

| Market Type | Format | Examples |
|-------------|--------|---------|
| Perpetuals  | Human-readable ticker | `"BTC"`, `"ETH"`, `"SOL"`, `"HYPE"` |
| Spot        | `@{index}` (index from `spotMeta` universe) | `"@1"`, `"@107"`, `"@142"`, `"@166"` |
| Spot (exception) | `"PURR/USDC"` only | `"PURR/USDC"` |

**Important**: `"BTC"` always means the BTC perpetual. Spot BTC is `"@142"`. There is no overlap between formats.

To discover spot coin indices, query:
```json
POST /info  {"type": "spotMeta"}
```
The `universe` array index corresponds to the `@index` number.

### Functional Differences

- The l2Book endpoint and WebSocket subscription work identically for both spot and perps
- Only the `coin` identifier format differs
- Both return the same `WsBook` / response structure
- Both support the same `nSigFigs` and `mantissa` parameters

---

## 7. Checksum

**No checksum mechanism exists** in the HyperLiquid l2Book API. Since every message is a full snapshot (not a delta), there is no need for checksum validation — the received book state IS the authoritative current state.

---

## 8. Sequence Numbers / Gap Detection

**No sequence numbers** are provided in l2Book messages. The `time` field (millisecond timestamp) is the only ordering field.

Since the feed is full snapshots (not deltas), gap detection is not necessary — each message is independently valid.

If a gap occurs during WebSocket disconnection, the reconnection snapshot will contain current state.

---

## 9. Update Speed

| Mechanism | Update Frequency |
|-----------|-----------------|
| WebSocket l2Book | Per-block, minimum ~500ms between pushes |
| REST l2Book | On-demand (rate limited) |
| gRPC StreamL2Book | Per-block (complete snapshot each block, no 500ms throttle) |

The WebSocket feed is block-rate throttled with a minimum 0.5 second interval between pushes. There is no configurable update speed for the public WebSocket.

For higher-frequency needs, the gRPC streaming API (`StreamL2Book` via `OrderBookStreaming` service) delivers a complete snapshot every block without the 500ms throttle. gRPC is NOT available via JSON-RPC or WebSocket — requires dedicated gRPC infrastructure.

---

## 10. Rate Limits

### REST Info Endpoint

| Limit | Value |
|-------|-------|
| Weight quota | 1,200 weight/minute per IP |
| l2Book weight | **2** per request |
| allMids weight | 2 |
| Most other info requests | 20 |
| Effective l2Book rate | Up to 600 requests/minute |

### WebSocket

| Limit | Value |
|-------|-------|
| Max connections per IP | 10 concurrent |
| New connections per minute | 30 |
| Max subscriptions per IP | 1,000 (shared across all connections) |
| Max messages per minute | 2,000 (across all connections) |
| Inflight POST messages | 100 simultaneous |

**Critical**: The 1,000 subscription limit is per-IP, NOT per-connection. 10 connections do not give 10,000 subscriptions.

With 239+ perpetual markets, subscribing to l2Book for all perps uses 239 slots. Adding other channels (trades, bbo) further reduces available slots.

---

## 11. Summary Table

| Capability | Value |
|-----------|-------|
| REST endpoint | `POST /info` with `"type": "l2Book"` |
| WS channel | `l2Book` |
| Depth levels | Max 20 per side (fixed) |
| Update model | Full snapshots only (no deltas) |
| WS push interval | Per-block, min ~500ms |
| nSigFigs values | `null`, `2`, `3`, `4`, `5` |
| mantissa values | `1`, `2`, `5` (only when nSigFigs==5) |
| Checksum | None |
| Sequence numbers | None |
| Spot coin format | `@{index}` or `"PURR/USDC"` |
| Perps coin format | Ticker string (`"BTC"`, `"ETH"`) |
| BBO channel | `bbo` (top-of-book only) |
| REST weight | 2 per request |
| WS subscriptions limit | 1,000 per IP |
| gRPC streaming | Available (per-block, no throttle, separate infra) |

---

## Sources

- [HyperLiquid WebSocket Subscriptions (Official)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/subscriptions)
- [HyperLiquid Info Endpoint (Official)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/info-endpoint)
- [HyperLiquid Rate Limits (Official)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/rate-limits-and-user-limits)
- [HyperLiquid WebSocket (Official)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket)
- [HyperLiquid Post Requests (Official)](https://hyperliquid.gitbook.io/hyperliquid-docs/for-developers/api/websocket/post-requests)
- [Chainstack — l2Book Reference](https://docs.chainstack.com/reference/hyperliquid-info-l2-book)
- [QuickNode — L2 Order Book Dataset](https://www.quicknode.com/docs/hyperliquid/datasets/l2-book)
- [Dwellir — WebSocket Subscription Limits](https://www.dwellir.com/blog/hyperliquid-websocket-subscription-limits)
- [Dwellir — WebSocket API Quick Reference](https://www.dwellir.com/docs/hyperliquid/websocket-api)
