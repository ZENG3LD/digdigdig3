# Paradex — L2 Orderbook Capabilities

## WebSocket Channels

### Endpoints

**Production:** `wss://ws.api.prod.paradex.trade/v1`
**Testnet:** `wss://ws.api.testnet.paradex.trade/v1`

All WebSocket communication uses **JSON-RPC 2.0** protocol.

---

### Channel: `order_book` (Throttled Snapshot + Delta Feed)

Public channel delivering orderbook updates (snapshot + incremental deltas) at a configured refresh rate.

**Channel name format:**
```
order_book.{MARKET}
```

Supports `ALL` as the market symbol to subscribe to all markets at once:
```
order_book.ALL
```

**Subscription request:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "refresh_rate": "50ms",
    "price_tick": "0_1",
    "depth": 15
  },
  "id": 1
}
```

**Subscription parameters:**

| Parameter | Type | Required | Values | Description |
|-----------|------|----------|--------|-------------|
| `channel` | string | yes | `order_book.{MARKET}` | Market symbol or `order_book.ALL` |
| `refresh_rate` | string | no | `"50ms"` or `"100ms"` | Throttle interval for updates |
| `price_tick` | string | no | e.g., `"0_1"` (represents 0.1) | Price level aggregation granularity |
| `depth` | integer | no | max observed: `15` | Number of price levels |

**Note on `depth`:** The documented maximum depth for the WebSocket channel is **15 levels**. Deeper books require polling the REST endpoint.

---

### Channel: `order_book_deltas` (Unthrottled Delta-Only Feed)

A separate channel providing raw unthrottled delta updates — no periodic snapshots. Recommended for latency-sensitive applications that maintain a full local book.

**Channel name format:**
```
order_book_deltas.{MARKET}
```

**Subscription parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `channel` | string | yes | `order_book_deltas.{MARKET}` |

Note: This channel does **not** accept `refresh_rate` or `price_tick` parameters — it is a raw unfiltered stream. The `OrderBookDeltas` enum in the Rust SDK (`snow-avocado/paradex-rs`) confirms this.

---

### Channel: `bbo` (Best Bid/Offer — Event-Driven)

Event-driven best bid/offer feed with **no throttling** — fires only when price or size changes.

**Channel name format:**
```
bbo.{MARKET}
```

**Published message:**
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "bbo.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "bid": "65432.4",
      "bid_size": "1.111",
      "ask": "65432.5",
      "ask_size": "1.234",
      "seq_no": 12345678,
      "timestamp": 1681759756789
    }
  }
}
```

BBO carries `seq_no` for ordering. No depth, no aggregation options.

---

## REST Endpoints

### GET /v1/orderbook/{market}

Get a full REST snapshot of the orderbook for a given market.

**URL:**
```
GET https://api.prod.paradex.trade/v1/orderbook/{market}
```

**Path parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market` | string | yes | Market symbol e.g. `BTC-USD-PERP` |

**Query parameters:**

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `depth` | integer | no | 20 | Number of price levels to return |
| `price_tick` | string | no | — | Price aggregation tick size |

No documented upper bound on `depth` — defaults to 20, higher values accepted. No pagination (single response).

**Response schema (`responses.AskBidArray`):**
```json
{
  "market": "BTC-USD-PERP",
  "asks": [["65432.5", "1.234"], ["65432.6", "2.456"]],
  "bids": [["65432.4", "1.111"], ["65432.3", "2.222"]],
  "best_ask_api": ["65432.5", "1.234"],
  "best_ask_interactive": ["65432.5", "1.234"],
  "best_bid_api": ["65432.4", "1.111"],
  "best_bid_interactive": ["65432.4", "1.111"],
  "last_updated_at": 1681759756789,
  "seq_no": 12345678
}
```

**Response fields:**

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market symbol |
| `asks` | `[[price, size], ...]` | Ask levels sorted ascending by price |
| `bids` | `[[price, size], ...]` | Bid levels sorted descending by price |
| `best_ask_api` | `[price, size]` | Best ask excluding RPI orders |
| `best_ask_interactive` | `[price, size]` | Best ask including RPI (UI-facing) |
| `best_bid_api` | `[price, size]` | Best bid excluding RPI orders |
| `best_bid_interactive` | `[price, size]` | Best bid including RPI (UI-facing) |
| `last_updated_at` | integer (ms) | Timestamp of last book update |
| `seq_no` | integer | Sequence number matching WS stream |

---

### GET /v1/orderbook/{market}/interactive

Same as above but returns the **interactive orderbook** including RPI (Retail Price Improvement) orders.

**URL:**
```
GET https://api.prod.paradex.trade/v1/orderbook/{market}/interactive
```

Parameters and response schema are identical to `/v1/orderbook/{market}`. The distinction is that RPI orders — which are normally hidden from the standard API book — are included here.

---

### GET /v1/bbo/{market}/interactive

Returns the interactive best bid/offer (including RPI).

**URL:**
```
GET https://api.prod.paradex.trade/v1/bbo/{market}/interactive
```

---

## Snapshot vs Delta

### Unified Model in `order_book` Channel

The `order_book` WebSocket channel delivers **both** full snapshots and incremental deltas using the same message structure. The `update_type` field distinguishes them.

**Published message format (both snapshot and delta use this structure):**
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "update_type": "s",
      "seq_no": 12345678,
      "last_updated_at": 1681759756789,
      "inserts": [
        { "price": "65432.5", "side": "SELL", "size": "1.234" },
        { "price": "65432.4", "side": "BUY",  "size": "1.111" }
      ],
      "updates": [
        { "price": "65432.3", "side": "BUY", "size": "3.500" }
      ],
      "deletes": [
        { "price": "65432.2", "side": "BUY", "size": "2.222" }
      ]
    }
  }
}
```

**`update_type` values:**

| Value | Meaning | Action |
|-------|---------|--------|
| `"s"` | Full snapshot | Discard local book, rebuild from scratch using `inserts` |
| `"d"` | Incremental delta | Apply `inserts`, `updates`, `deletes` to existing local book |

**Data arrays (both snapshot and delta):**

| Field | Description |
|-------|-------------|
| `inserts` | New price levels to add |
| `updates` | Existing price levels with changed size |
| `deletes` | Price levels to remove |
| `side` | `"BUY"` or `"SELL"` |
| `price` | Price as string (decimal) |
| `size` | Size as string (decimal) |

### Maintaining a Local Book

```
1. Wait for first message with update_type == "s"
2. Reset local book, populate from inserts array
3. Record seq_no as last_seq_no
4. For each subsequent message:
   a. If update_type == "s": reset and rebuild from inserts
   b. If update_type == "d":
      - If seq_no != last_seq_no + 1: gap detected → discard book, wait for next "s"
      - Apply deletes (remove levels)
      - Apply updates (change sizes)
      - Apply inserts (add levels)
      - Update last_seq_no = seq_no
```

### `order_book_deltas` Channel

This channel delivers **delta-only** messages without periodic snapshots. To use it:
1. Fetch an initial snapshot via the REST endpoint `GET /v1/orderbook/{market}`
2. Note the `seq_no` from the REST response
3. Apply all WS deltas with `seq_no > rest_seq_no`

---

## Update Speed

| Channel | Update Mechanism | Speed |
|---------|-----------------|-------|
| `order_book` | Throttled snapshots + deltas | 50ms or 100ms (configurable via `refresh_rate`) |
| `order_book_deltas` | Raw unthrottled deltas | Event-driven, no artificial delay |
| `bbo` | Event-driven BBO only | No throttling, fires on every change |

**Recommendation from official docs:** For fastest full orderbook depth, subscribe to `order_book_deltas` instead of `order_book`. For price-only strategies, `bbo` is optimal.

**Message queue limit:** The server will disconnect a client if **2,000+ messages accumulate unprocessed** in the connection buffer. Fast message draining is required.

---

## Price Aggregation

Price aggregation (tick grouping) is supported in both REST and WebSocket:

**REST:** `price_tick` query parameter on `/v1/orderbook/{market}`

**WebSocket:** `price_tick` subscription parameter on `order_book` channel

**Format:** The `price_tick` value uses underscores as decimal separators — e.g., `"0_1"` means 0.1, `"1_0"` means 1.0, `"10"` means 10.

When `price_tick` is specified, multiple orders at nearby prices are collapsed into single aggregated levels at multiples of the tick size.

The `order_book_deltas` channel does **not** support `price_tick` — it delivers raw, unaggregated price levels.

---

## Checksum

**No checksum mechanism is documented** in the Paradex API. There is no CRC32 or hash field in the orderbook message schema for integrity verification.

Integrity is maintained solely through:
1. Sequence numbers (`seq_no`) for gap detection
2. Full snapshot (`update_type: "s"`) for re-synchronization

---

## Sequence / Ordering

### `seq_no` Field

Present in all orderbook messages (REST and WebSocket):

| Source | seq_no Behavior |
|--------|----------------|
| REST `/v1/orderbook` | Returns current `seq_no` of the book state |
| WS `order_book` | Monotonically increasing per market |
| WS `order_book_deltas` | Monotonically increasing per market |
| WS `bbo` | Monotonically increasing per market |

### Gap Detection

The `seq_no` increments by exactly 1 between consecutive messages. A gap is detected when:
```
received_seq_no != last_seq_no + 1
```

**Gap handling:**
1. Discard the local order book (state is now unreliable)
2. Await the next snapshot message (`update_type: "s"`) from the `order_book` channel
3. Or: re-fetch via REST `GET /v1/orderbook/{market}` and re-seed the delta stream

### Stale/Duplicate Updates

If `received_seq_no <= last_seq_no`, the update is stale/duplicate and should be discarded.

### First Message on Subscription

The server sends an initial snapshot (`update_type: "s"`) upon subscription to the `order_book` channel. Do not apply deltas until this snapshot is received.

---

## Account Type Differences

### API Orderbook vs Interactive Orderbook

Paradex operates a **tiered liquidity model** with two distinct orderbook views:

| Aspect | Standard API Orderbook | Interactive Orderbook |
|--------|----------------------|----------------------|
| REST endpoint | `/v1/orderbook/{market}` | `/v1/orderbook/{market}/interactive` |
| WS channel | `order_book.{MARKET}` | No separate WS channel |
| RPI orders | **Excluded** | **Included** |
| Book state | Always uncrossed | May appear crossed (RPI at better prices) |
| Use case | Algorithmic trading, bots | UI display, retail traders |

**RPI (Retail Price Improvement) Orders:**
- RPI orders are placed by market makers to provide better pricing to retail users
- They are **hidden from API traders** and do not match against API-submitted orders
- They are **visible only to non-algorithmic users** via the interactive endpoints
- The interactive orderbook may appear "crossed" (bid > ask) due to RPI orders sitting inside the spread
- `best_bid_api` vs `best_bid_interactive` in REST responses reflect this split

### Cross Margin vs Isolated Margin Markets

Markets are classified by `market_kind` in the `GET /v1/markets` response:

| `market_kind` | Description |
|---------------|-------------|
| `cross` | Standard perpetuals using shared cross-margin collateral pool |
| `isolated` | Perpetuals with per-position isolated margin (launched April 2025) |
| `isolated_margin` | Variant of isolated margin |

**Orderbook behavior:** The orderbook itself is **not separated** by margin type. `BTC-USD-PERP` has a single order book regardless of whether a trader uses cross or isolated margin. The `market_kind` distinction affects margin accounting, not the order book feed.

**Authentication for isolated margin:** Orders for isolated margin markets require specifying the isolated sub-account address (`on_behalf_of_account` field in the order request). Orderbook subscriptions are the same regardless.

---

## Raw Examples

### Subscribe to order_book with throttling

```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "refresh_rate": "50ms",
    "depth": 15
  },
  "id": 1
}
```

### Snapshot message (update_type = "s")

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "update_type": "s",
      "seq_no": 12345678,
      "last_updated_at": 1681759756789,
      "inserts": [
        { "price": "65432.5", "side": "SELL", "size": "1.234" },
        { "price": "65432.6", "side": "SELL", "size": "2.456" },
        { "price": "65432.7", "side": "SELL", "size": "3.789" },
        { "price": "65432.4", "side": "BUY",  "size": "1.111" },
        { "price": "65432.3", "side": "BUY",  "size": "2.222" },
        { "price": "65432.2", "side": "BUY",  "size": "3.333" }
      ],
      "updates": [],
      "deletes": []
    }
  }
}
```

### Delta message (update_type = "d")

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "update_type": "d",
      "seq_no": 12345679,
      "last_updated_at": 1681759756839,
      "inserts": [
        { "price": "65432.8", "side": "SELL", "size": "1.500" }
      ],
      "updates": [
        { "price": "65432.5", "side": "SELL", "size": "0.800" }
      ],
      "deletes": [
        { "price": "65432.3", "side": "BUY", "size": "2.222" }
      ]
    }
  }
}
```

### BBO subscription and message

```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": { "channel": "bbo.BTC-USD-PERP" },
  "id": 2
}
```

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "bbo.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "bid": "65432.4",
      "bid_size": "1.111",
      "ask": "65432.5",
      "ask_size": "0.800",
      "seq_no": 12345679,
      "timestamp": 1681759756839
    }
  }
}
```

### REST snapshot (with price aggregation)

```
GET https://api.prod.paradex.trade/v1/orderbook/BTC-USD-PERP?depth=20&price_tick=1_0
```

```json
{
  "market": "BTC-USD-PERP",
  "asks": [
    ["65433.0", "4.690"],
    ["65434.0", "2.100"]
  ],
  "bids": [
    ["65432.0", "4.666"],
    ["65431.0", "1.800"]
  ],
  "best_ask_api": ["65433.0", "4.690"],
  "best_ask_interactive": ["65432.0", "0.500"],
  "best_bid_api": ["65432.0", "4.666"],
  "best_bid_interactive": ["65433.0", "0.300"],
  "last_updated_at": 1681759756789,
  "seq_no": 12345679
}
```

Note: In this example the interactive book appears crossed (`best_bid_interactive > best_ask_interactive`) due to RPI orders.

---

## Summary Table

| Capability | Value |
|-----------|-------|
| WebSocket orderbook channel | `order_book.{MARKET}` |
| Delta-only channel | `order_book_deltas.{MARKET}` |
| BBO channel | `bbo.{MARKET}` |
| WS depth | 15 levels max |
| REST depth default | 20 levels |
| REST depth max | Not documented (no hard cap) |
| Throttle options | 50ms or 100ms |
| Unthrottled stream | `order_book_deltas` (event-driven) |
| Price aggregation | `price_tick` param (REST + `order_book` WS) |
| Snapshot type field | `update_type: "s"` |
| Delta type field | `update_type: "d"` |
| Sequence field | `seq_no` (integer, monotonic per market) |
| Checksum | None |
| Gap handling | Await next snapshot (`"s"`) or re-fetch REST |
| RPI orders in book | API book: excluded; Interactive book: included |
| Market type variation | Single book per symbol; margin type = separate accounting only |
| Protocol | JSON-RPC 2.0 |
| Auth required | No (public channels) |
| Connection rate limit | 20 conn/sec, 600 conn/min per IP |
| Message queue limit | 2,000 unprocessed messages → disconnect |

---

## Sources

- [SUB order_book — Paradex Docs](https://docs.paradex.trade/ws/web-socket-channels/order-book/order-book)
- [WebSocket API Introduction — Paradex Docs](https://docs.paradex.trade/ws/general-information/introduction)
- [Advanced API Trader Best Practices — Paradex Docs](https://docs.paradex.trade/trading/api-best-practices)
- [Get market orderbook — Paradex REST Docs](https://docs.paradex.trade/api/prod/markets/get-orderbook)
- [Get market interactive orderbook — Paradex REST Docs](https://docs.paradex.trade/api/prod/markets/get-orderbook-interactive)
- [WebSocket Rate Limits — Paradex Docs](https://docs.paradex.trade/ws/general-information/rate-limits)
- [Subscription Channels — Paradex Docs](https://docs.paradex.trade/ws/general-information/subscription-channels)
- [paradex-py SDK — Python Examples](https://github.com/tradeparadex/paradex-py/blob/main/examples/connect_ws_api.py)
- [paradex-rs — Rust Client (snow-avocado)](https://github.com/snow-avocado/paradex-rs)
- [paradex-docs — GitHub Source](https://github.com/tradeparadex/paradex-docs)
- [Retail Price Improvement — Paradex Docs](https://docs.paradex.trade/trading/rpi)
