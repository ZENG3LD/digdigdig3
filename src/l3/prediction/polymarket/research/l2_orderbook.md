# Polymarket — L2 Orderbook Capabilities

## Summary

Polymarket is a prediction market running on Polygon using a hybrid-decentralized CLOB: off-chain order matching with on-chain settlement via an ERC-1155 Exchange smart contract. Prices are probabilities in the range `[0.01, 0.99]` denominated in USDC. Each binary market has two ERC-1155 tokens (YES and NO). The orderbook API operates on **token IDs** (ERC-1155 token identifiers), not traditional exchange symbol strings.

WebSocket delivers a full snapshot on subscribe, then event-driven delta updates per order change. No sequence numbers; ordering is by timestamp only. No checksum on deltas (only on the `book` snapshot event). REST delivers full depth with no pagination. Bulk REST (`POST /books`) is available.

---

## Symbol / Identifier System

Polymarket uses a three-layer identifier hierarchy:

| Level | Field | Description | Example |
|-------|-------|-------------|---------|
| Event | `event.id` / `event.slug` | Groups related markets | `"trump-2024"` |
| Market | `condition_id` | Blockchain condition ID (0x + 64 hex) | `"0x1234...abcd"` |
| Token | `token_id` | ERC-1155 token ID (YES or NO outcome) | `"71321045679..."` (decimal string) |

For all orderbook and pricing API calls, the **`token_id`** is required — not the `condition_id`.

Each market has exactly two tokens. The YES token and NO token always satisfy `price_yes + price_no ≈ 1.0` (minus small spreads).

**Token ID format:** Long decimal integer string (not hex). Example:
```
71321045679452212359400135306397382751538180628418768730384491070510793884442
```

**Condition ID format:** `0x` + 64 hex characters. Example:
```
0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1
```

### How to get token IDs

```
GET https://gamma-api.polymarket.com/markets?slug=<event-slug>
```

Response includes `clobTokenIds: ["<yes_token_id>", "<no_token_id>"]`.

Or from the CLOB API:
```
GET https://clob.polymarket.com/markets/<condition_id>
```

Response includes `tokens: [{token_id, outcome: "Yes"}, {token_id, outcome: "No"}]`.

---

## REST Endpoints

### Base URL

```
https://clob.polymarket.com
```

No authentication required for all read endpoints listed below.

### GET /book — Single Orderbook Snapshot

```
GET https://clob.polymarket.com/book?token_id=<TOKEN_ID>
```

**Parameters:**

| Name | Location | Type | Required | Description |
|------|----------|------|----------|-------------|
| `token_id` | query | string | yes | ERC-1155 token ID |

**Response: `OrderBookSummary`**

```json
{
  "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
  "asset_id": "71321045679452212359400135306397382751538180628418768730384491070510793884442",
  "timestamp": "1714000000000",
  "hash": "a1b2c3d4e5f6...",
  "bids": [
    {"price": "0.73", "size": "150"},
    {"price": "0.72", "size": "80"}
  ],
  "asks": [
    {"price": "0.75", "size": "200"},
    {"price": "0.76", "size": "60"}
  ],
  "min_order_size": "1",
  "tick_size": "0.01",
  "neg_risk": false,
  "last_trade_price": "0.74"
}
```

**Field descriptions:**

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Market condition ID (blockchain) |
| `asset_id` | string | Token ID queried |
| `timestamp` | string | Snapshot generation time (Unix ms) |
| `hash` | string | MD5/hash of full orderbook state (integrity check) |
| `bids` | array | Buy side, sorted price descending |
| `asks` | array | Sell side, sorted price ascending |
| `min_order_size` | string | Minimum order size in USDC |
| `tick_size` | string | Minimum price increment (e.g. `"0.01"` or `"0.001"`) |
| `neg_risk` | bool | Whether this is a negative-risk multi-outcome market |
| `last_trade_price` | string | Most recent trade price |

**Notes:**
- No `depth` limit parameter — full book always returned
- Prices and sizes are strings (to preserve decimal precision)
- Prices may omit leading zero: `".48"` instead of `"0.48"` — normalize on parse
- Price range: `0.01` to `0.99` (never 0.00 or 1.00 while active)
- Known issue (2025): `/book` can return stale "ghost" data (bid=0.01, ask=0.99) for some markets; `/price` returns live data

**Errors:**
- `400` — Invalid token ID
- `404` — No orderbook for token ID
- `500` — Internal error

### POST /books — Bulk Orderbook Snapshots

```
POST https://clob.polymarket.com/books
Content-Type: application/json

[{"token_id": "TOKEN_1"}, {"token_id": "TOKEN_2"}, ...]
```

Returns array of `OrderBookSummary` objects (same structure as `/book`).

Rate limit: 500 req/10s (versus 1500 req/10s for `/book`).

### GET /midpoint — Mid Price

```
GET https://clob.polymarket.com/midpoint?token_id=<TOKEN_ID>
```

Response:
```json
{"mid": 0.74}
```

`mid` is a number (not string) in this response.

Rate limit: 1,500 req/10s.

### GET /price — Best Bid or Best Ask

```
GET https://clob.polymarket.com/price?token_id=<TOKEN_ID>&side=<BUY|SELL>
```

Response:
```json
{"price": "0.73"}
```

`BUY` returns best ask (price you pay), `SELL` returns best bid (price you receive).

### GET /spread — Bid-Ask Spread

```
GET https://clob.polymarket.com/spread?token_id=<TOKEN_ID>
```

Response:
```json
{"spread": "0.02"}
```

### GET /last-trade-price — Last Trade

```
GET https://clob.polymarket.com/last-trade-price?token_id=<TOKEN_ID>
```

Response:
```json
{"price": "0.74"}
```

### GET /tick-size — Minimum Price Increment

```
GET https://clob.polymarket.com/tick-size?token_id=<TOKEN_ID>
```

Response:
```json
{"minimum_tick_size": 0.01}
```

Note: response value is a number, not a string.

Tick sizes change dynamically as price approaches extremes. Common values: `0.01` (most markets), `0.001` (high-confidence markets near 0.99 or 0.01).

### GET /prices-history — Price Klines

```
GET https://clob.polymarket.com/prices-history?market=<TOKEN_ID>&interval=<INTERVAL>&fidelity=<N>
```

| Param | Values | Notes |
|-------|--------|-------|
| `interval` | `1m`, `1h`, `6h`, `1d`, `1w`, `all` | Time bucket |
| `fidelity` | integer, max 1000 | Number of data points |

Response:
```json
{"history": [{"t": 1714000000, "p": 0.74}, ...]}
```

`t` = Unix seconds, `p` = price (0.0–1.0).

---

## WebSocket Channels

### Connection Details

```
wss://ws-subscriptions-clob.polymarket.com/ws/market   (public, market data)
wss://ws-subscriptions-clob.polymarket.com/ws/user     (authenticated, user orders/trades)
```

### Subscription Message

Send immediately after connecting:

```json
{
  "type": "market",
  "assets_ids": ["TOKEN_ID_1", "TOKEN_ID_2"],
  "initial_dump": true,
  "level": 2,
  "custom_feature_enabled": false
}
```

**Subscription fields:**

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `type` | string | yes | — | Must be `"market"` |
| `assets_ids` | array | yes | — | Token IDs to subscribe to |
| `initial_dump` | bool | no | `true` | Send full book snapshot immediately on subscribe |
| `level` | int | no | `2` | Subscription depth level: 1, 2, or 3 |
| `custom_feature_enabled` | bool | no | `false` | Enables `best_bid_ask`, `new_market`, `market_resolved` events |

**Depth level semantics:** The documentation defines levels 1, 2, 3 but does not specify the exact number of price levels each returns. Level 2 is the default and sufficient for standard orderbook maintenance.

### Dynamic Subscribe/Unsubscribe (no reconnect needed)

```json
{
  "operation": "subscribe",
  "assets_ids": ["NEW_TOKEN_ID"],
  "level": 2
}
```

```json
{
  "operation": "unsubscribe",
  "assets_ids": ["TOKEN_ID_TO_REMOVE"]
}
```

### Keepalive

Send literal string `"PING"` every 10 seconds. Server responds with literal string `"PONG"`. Connection drops without periodic pings.

---

## WebSocket Event Types

### 1. `book` — Full Orderbook Snapshot

Sent on subscribe (if `initial_dump: true`) and after trade executions.

```json
{
  "event_type": "book",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "bids": [
    {"price": "0.73", "size": "150"},
    {"price": "0.72", "size": "80"}
  ],
  "asks": [
    {"price": "0.75", "size": "200"},
    {"price": "0.76", "size": "60"}
  ],
  "timestamp": "1714000000000",
  "hash": "a1b2c3d4e5f6..."
}
```

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | string | `"book"` |
| `asset_id` | string | Token ID |
| `market` | string | Condition ID |
| `bids` | array | All bid levels, price descending |
| `asks` | array | All ask levels, price ascending |
| `timestamp` | string | Unix milliseconds |
| `hash` | string | Orderbook state hash (use for integrity check) |

**Use:** Replace entire local book. This is the canonical reset point.

### 2. `price_change` — Incremental Level Update

Sent on every order add/cancel/fill that changes a price level.

```json
{
  "event_type": "price_change",
  "market": "0x5f65177b...",
  "price_changes": [
    {
      "asset_id": "71321045...",
      "price": "0.73",
      "size": "200",
      "side": "BUY",
      "hash": "0xabc...",
      "best_bid": "0.73",
      "best_ask": "0.75"
    }
  ],
  "timestamp": "1714000000001"
}
```

Note: the existing codebase uses `changes` (not `price_changes`) as the array field name — verify against live data; documentation uses `price_changes`.

**Per-change fields:**

| Field | Type | Description |
|-------|------|-------------|
| `asset_id` | string | Token ID for this level change |
| `price` | string | Price level affected |
| `size` | string | **New aggregate size** at this level. `"0"` means level removed |
| `side` | string | `"BUY"` (bid) or `"SELL"` (ask) |
| `hash` | string | Hash of the specific order causing this change |
| `best_bid` | string | Optional: current best bid after update |
| `best_ask` | string | Optional: current best ask after update |

**Use:** Apply to local book: set `book[side][price] = size`; if `size == "0"`, remove the level.

### 3. `last_trade_price` — Trade Execution

```json
{
  "event_type": "last_trade_price",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "price": "0.74",
  "size": "219.217767",
  "side": "BUY",
  "fee_rate_bps": "0",
  "timestamp": "1714000000322",
  "transaction_hash": "0x..."
}
```

| Field | Type | Description |
|-------|------|-------------|
| `price` | string | Trade execution price |
| `size` | string | Trade size in outcome tokens |
| `side` | string | Taker side: `"BUY"` or `"SELL"` |
| `fee_rate_bps` | string | Fee in basis points (often `"0"`) |
| `transaction_hash` | string | Optional Polygon tx hash |

Note: trade events arrive alongside `price_change` events (the trade removes liquidity from the book).

### 4. `tick_size_change` — Price Increment Change

```json
{
  "event_type": "tick_size_change",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "old_tick_size": "0.01",
  "new_tick_size": "0.001",
  "timestamp": "1714000000000"
}
```

Occurs when probability approaches extremes (market becoming near-certain). After this event, all existing levels at the old granularity remain valid; new orders use the new tick size.

### 5. `best_bid_ask` — Top-of-Book Only (requires `custom_feature_enabled: true`)

```json
{
  "event_type": "best_bid_ask",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "best_bid": "0.73",
  "best_ask": "0.75",
  "spread": "0.02",
  "timestamp": "1714000000000"
}
```

Lightweight alternative when full book depth not needed.

### 6. `new_market` — Market Created (requires `custom_feature_enabled: true`)

```json
{
  "event_type": "new_market",
  "id": "market_id",
  "question": "Will X happen?",
  "market": "0x...",
  "slug": "url-slug",
  "assets_ids": ["yes_token", "no_token"],
  "outcomes": ["Yes", "No"],
  "active": true,
  "order_price_min_tick_size": "0.01",
  "timestamp": "1714000000000"
}
```

### 7. `market_resolved` — Market Settled (requires `custom_feature_enabled: true`)

```json
{
  "event_type": "market_resolved",
  "id": "market_id",
  "market": "0x...",
  "assets_ids": ["yes_token", "no_token"],
  "winning_asset_id": "yes_token",
  "winning_outcome": "Yes",
  "timestamp": "1714000000000"
}
```

---

## Snapshot vs Delta

### Protocol

```
1. Connect to wss://ws-subscriptions-clob.polymarket.com/ws/market
2. Subscribe with initial_dump: true (default)
3. Receive book event → replace entire local book
4. Receive price_change events → apply incremental updates
5. Receive book event again after trades → resync local book
```

### When `book` snapshots are sent

- On subscribe (if `initial_dump: true`)
- After trade execution (book event follows last_trade_price event)
- Possibly on reconnect (after sending new subscription)

### Applying `price_change` deltas

```
for each change in price_changes:
    if change.size == "0":
        remove book[change.side][change.price]
    else:
        book[change.side][change.price] = change.size
```

The `size` field is always the **new total aggregate size** at that level, not a diff. There are no signed +/- quantities.

### Gap handling

There are **no sequence numbers**. If messages are missed (network drop), the local book may be stale. Recovery:

1. On reconnect, re-subscribe with `initial_dump: true`
2. Wait for `book` snapshot event
3. Replace local book entirely
4. Resume delta application

Alternatively, poll `GET /book` REST endpoint at any time for an authoritative snapshot.

### Recommended maintenance pattern

```
local_book = {}
on_book(snapshot):
    local_book = snapshot

on_price_change(changes):
    for change in changes:
        if size == "0":
            del local_book[side][price]
        else:
            local_book[side][price] = size

on_disconnect:
    resubscribe with initial_dump=true
    wait for book event
```

---

## Update Speed

- **Trigger:** Event-driven, not time-based. Updates fire immediately on each order placement, cancellation, or fill.
- **No rate configuration:** There is no throttle or batch interval parameter.
- **Frequency in practice:** Liquid markets can have dozens of updates per second during active trading; thin markets may have minutes between updates.
- **Timestamps:** All WebSocket timestamps are Unix milliseconds (string format).

---

## Price Aggregation

Polymarket does not have an explicit "tick grouping" or "aggregation depth" feature equivalent to exchange APIs. However:

- **Tick size** (`minimum_tick_size`) controls the price grid. All levels are multiples of tick size.
- Default tick size is `0.01` (1 cent, i.e., 1% probability increments).
- For markets with price near 0.01 or 0.99, tick size may be reduced to `0.001`.
- The `tick_size_change` WebSocket event notifies when this changes.
- All bid/ask levels in the book are aligned to the current tick size.
- Price range: `[0.01, 0.99]`. Prices exactly 0.00 or 1.00 are not valid while market is active.

**No client-side aggregation API.** If you want grouping (e.g., 0.05 increments), you must do it yourself client-side.

---

## Checksum

- **REST `/book`:** Response includes a `hash` field — an MD5-like hash of the full orderbook state at the snapshot time. Use for integrity verification of the REST snapshot.
- **WebSocket `book` event:** Also includes `hash` field. Same semantics.
- **WebSocket `price_change` event:** Individual changes include a per-order `hash` (identifies the specific order that caused the change), not a cumulative book hash.
- **No rolling/incremental checksum** for delta stream. If you need to verify local book correctness, fetch REST `/book`, compute or compare the `hash` field.

---

## Sequence / Ordering

- **No sequence numbers** on any endpoint (REST or WebSocket).
- **Ordering guarantee:** None explicitly. Use `timestamp` (Unix ms) for ordering, but simultaneous events (same ms) have no defined order.
- **Gap detection:** Not possible from the API. Must rely on reconnect + snapshot resync.
- **Duplicate detection:** Not possible from the API. `price_change` events do not carry unique event IDs.
- **`price_change.hash`** identifies the causative order, not the event itself. Can be used to deduplicate order-level changes but not stream events.

---

## Rate Limits (CLOB API)

| Endpoint | Limit |
|----------|-------|
| `/book`, `/price`, `/midpoint` | 1,500 req / 10s |
| `/books` (bulk POST) | 500 req / 10s |
| General CLOB | 9,000 req / 10s |
| `POST /order` | 3,500 req/10s burst, 36,000 req/10 min sustained |

Limits use sliding windows. Requests are throttled (queued/delayed) rather than hard-rejected when exceeded.

No documented WebSocket connection limit per IP. Standard practice is one connection per client.

---

## Authentication

| Layer | Method | Used for |
|-------|--------|---------|
| L0 | None | All read endpoints (book, price, markets, WebSocket market channel) |
| L1 | EIP-712 wallet signature | Deriving CLOB API credentials |
| L2 | HMAC-SHA256 | Placing/cancelling orders, user WebSocket channel |

**WebSocket market channel:** No authentication required.

**WebSocket user channel:** Requires L2 credentials in subscription message:
```json
{
  "type": "user",
  "auth": {
    "apiKey": "uuid",
    "secret": "base64",
    "passphrase": "passphrase"
  }
}
```

Authentication headers for REST trading endpoints:
- `POLY_ADDRESS` — wallet address
- `POLY_SIGNATURE` — EIP-712 or HMAC signature
- `POLY_TIMESTAMP` — Unix seconds
- `POLY_NONCE` — random nonce
- `POLY_API_KEY` — API key UUID
- `POLY_PASSPHRASE` — API passphrase

---

## Raw Examples

### REST /book response

```json
{
  "market": "0x5f65177b394277fd294cd75650044e32ba009a95022d88a0c1d565897d72f8f1",
  "asset_id": "71321045679452212359400135306397382751538180628418768730384491070510793884442",
  "timestamp": "1714000000000",
  "hash": "a1b2c3d4e5f6789012345678",
  "bids": [
    {"price": "0.73", "size": "150"},
    {"price": "0.72", "size": "80"},
    {"price": "0.70", "size": "400"}
  ],
  "asks": [
    {"price": "0.75", "size": "200"},
    {"price": "0.76", "size": "60"},
    {"price": "0.78", "size": "250"}
  ],
  "min_order_size": "1",
  "tick_size": "0.01",
  "neg_risk": false,
  "last_trade_price": "0.74"
}
```

### WebSocket subscription

```json
{"type": "market", "assets_ids": ["71321045..."], "initial_dump": true, "level": 2}
```

### WebSocket `book` event

```json
{
  "event_type": "book",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "bids": [{"price": "0.73", "size": "150"}, {"price": "0.72", "size": "80"}],
  "asks": [{"price": "0.75", "size": "200"}, {"price": "0.76", "size": "60"}],
  "timestamp": "1714000000000",
  "hash": "a1b2c3d4e5f6..."
}
```

### WebSocket `price_change` event

```json
{
  "event_type": "price_change",
  "market": "0x5f65177b...",
  "price_changes": [
    {
      "asset_id": "71321045...",
      "price": "0.73",
      "size": "200",
      "side": "BUY",
      "hash": "0xabc123...",
      "best_bid": "0.73",
      "best_ask": "0.75"
    }
  ],
  "timestamp": "1714000000001"
}
```

### WebSocket `last_trade_price` event

```json
{
  "event_type": "last_trade_price",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "price": "0.74",
  "size": "219.217767",
  "side": "BUY",
  "fee_rate_bps": "0",
  "timestamp": "1750428146322",
  "transaction_hash": "0xdeadbeef..."
}
```

### WebSocket `tick_size_change` event

```json
{
  "event_type": "tick_size_change",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "old_tick_size": "0.01",
  "new_tick_size": "0.001",
  "timestamp": "1714000000000"
}
```

### WebSocket `best_bid_ask` event (custom_feature_enabled: true)

```json
{
  "event_type": "best_bid_ask",
  "asset_id": "71321045...",
  "market": "0x5f65177b...",
  "best_bid": "0.73",
  "best_ask": "0.75",
  "spread": "0.02",
  "timestamp": "1714000000000"
}
```

---

## Implementation Notes (Codebase-Specific)

### Price string quirk
Polymarket occasionally sends prices without a leading zero: `".48"` instead of `"0.48"`. The codebase handles this in `normalize_price_in_place()` and `deserialize_string_to_f64()`. This must be applied to all price fields from both REST and WebSocket.

### `price_changes` vs `changes` field name
The official docs use `price_changes` as the array field name in `price_change` events. The current `WsPriceChange` struct in `parser.rs` uses `changes`. Verify against live WS data — the field name may differ from documentation.

### `book` events are resyncs after trades
After every `last_trade_price` event, a new `book` snapshot event is sent. This makes resync automatic for trade-driven book changes. Delta-only changes (add/cancel) arrive as `price_change` without a subsequent `book` event.

### `asset_id` in `price_change`
In `price_change`, the `asset_id` is inside each element of the `price_changes` array (per-change), not at the top level of the event. This allows a single `price_change` event to carry updates for multiple tokens (though in practice typically one).

---

## Known Limitations

1. **No sequence numbers** — gap detection is impossible without reconnect+snapshot
2. **No per-level depth limit** — full book always returned (REST and WS)
3. **Stale REST `/book` issue** — known 2025 bug where `/book` returns phantom 0.01/0.99 book; `/price` returns accurate best-bid/ask
4. **Tick size ambiguity** — tick can change without warning (only via WS event); REST clients must poll `/tick-size` separately
5. **Update frequency not configurable** — purely event-driven, no throttle parameter

---

## Sources

- [Polymarket API Docs — Get Order Book](https://docs.polymarket.com/api-reference/market-data/get-order-book.md)
- [Polymarket API Docs — WebSocket Market Channel](https://docs.polymarket.com/api-reference/wss/market.md)
- [Polymarket API Docs — Rate Limits](https://docs.polymarket.com/api-reference/rate-limits.md)
- [Polymarket API Docs — Introduction](https://docs.polymarket.com/api-reference/introduction.md)
- [Polymarket API Docs — Tick Size](https://docs.polymarket.com/api-reference/market-data/get-tick-size.md)
- [Polymarket API Docs — WebSocket User Channel](https://docs.polymarket.com/api-reference/wss/user.md)
- [Polymarket Rust CLOB Client (GitHub)](https://github.com/Polymarket/rs-clob-client)
- [Known stale /book issue (GitHub)](https://github.com/Polymarket/py-clob-client/issues/180)
- [Polymarket API Guide 2026 — pm.wiki](https://pm.wiki/learn/polymarket-api)
