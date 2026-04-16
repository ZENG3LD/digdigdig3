# Lighter — L2 Orderbook Capabilities

## Overview

Lighter is a ZK-rollup–based perpetual (and spot) DEX on Arbitrum with a Central Limit Order Book (CLOB).
Architecture: Sequencer → Indexer → API Servers → clients. The Indexer processes Sequencer output and
serves real-time market/account state via REST and WebSocket.

Base endpoints:
- **Mainnet REST**: `https://mainnet.zklighter.elliot.ai`
- **Testnet REST**: `https://testnet.zklighter.elliot.ai`
- **Mainnet WS**: `wss://mainnet.zklighter.elliot.ai/stream`
- **Testnet WS**: `wss://testnet.zklighter.elliot.ai/stream`
- **Read-only restricted regions**: append `?readonly=true` to WS URL

---

## WebSocket Channels

### Channel: `order_book/{MARKET_INDEX}`

The primary L2 depth stream. Sends the complete set of ask/bid price levels for the given market.

**Auth required**: No (public channel)

**Subscription message**:
```json
{
  "type": "subscribe",
  "channel": "order_book/0"
}
```

**Unsubscribe message**:
```json
{
  "type": "unsubscribe",
  "channel": "order_book/0"
}
```

**Update message format**:
```json
{
  "channel": "order_book:0",
  "type": "update/order_book",
  "timestamp": 1640995200,
  "last_updated_at": 1640995200123,
  "offset": 12345,
  "order_book": {
    "code": 200,
    "asks": [
      { "price": "3025.00", "size": "2.0" },
      { "price": "3026.50", "size": "1.5" }
    ],
    "bids": [
      { "price": "3024.00", "size": "1.0" },
      { "price": "3023.00", "size": "3.5" }
    ],
    "offset": 12345,
    "nonce": 67890,
    "last_updated_at": 1640995200123,
    "begin_nonce": 67885
  }
}
```

**Field descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| `channel` | string | `"order_book:{MARKET_INDEX}"` (colon separator in response, slash in subscription) |
| `type` | string | Always `"update/order_book"` |
| `timestamp` | integer | Unix timestamp of the message |
| `last_updated_at` | integer | Timestamp of the last book update (outer, mirrors inner) |
| `offset` | integer | API-server-scoped monotonic counter; outer field mirrors `order_book.offset` |
| `order_book.code` | integer | HTTP-style status code (200 = OK) |
| `order_book.asks` | array | Ask price levels, ascending by price, as `{price, size}` strings |
| `order_book.bids` | array | Bid price levels, descending by price, as `{price, size}` strings |
| `order_book.offset` | integer | Same offset, API-server-specific |
| `order_book.nonce` | integer | Matching engine sequence number after this update |
| `order_book.last_updated_at` | integer | Millisecond timestamp of last matching engine update |
| `order_book.begin_nonce` | integer | Matching engine sequence number before this update (= prev `nonce`) |

**Update frequency**: 50 ms batches (max 20 updates/second per market)

---

### Channel: `ticker/{MARKET_INDEX}`

Best bid/offer (BBO) only — lighter-weight than the full book channel.

**Auth required**: No

**Subscription**:
```json
{
  "type": "subscribe",
  "channel": "ticker/0"
}
```

Useful when only the top-of-book quote is needed, not the full depth.

---

### Channel: `trade/{MARKET_INDEX}`

Real-time trade executions — useful for reconstructing prints and cross-referencing orderbook state.

**Auth required**: No

**Update message**:
```json
{
  "channel": "trade/0",
  "type": "update/trade",
  "timestamp": 1640995200,
  "trade_id": 12345,
  "price": "3024.66",
  "size": "1.5",
  "side": "buy",
  "is_maker_ask": true
}
```

**Update frequency**: Real-time (per trade execution)

---

### Channel: `market_stats/{MARKET_INDEX}` or `market_stats/all`

Market-level statistics: last price, 24h volume, funding rate, open interest.

**Auth required**: No

Supports `market_stats/all` to get all markets in one subscription — unique to this channel.

---

### Channel: `spot_market_stats/{MARKET_INDEX}`

Same as `market_stats` but for spot markets specifically.

**Auth required**: No

---

### Other Public Channels (Non-Orderbook)

| Channel | Description |
|---------|-------------|
| `height` | Blockchain block height |
| `account_all/{ACCOUNT_ID}` | Account overview (public read) |
| `user_stats/{ACCOUNT_ID}` | Account statistics |

### Authenticated Channels (Non-Orderbook)

| Channel | Auth | Description |
|---------|------|-------------|
| `account_market/{MARKET_ID}/{ACCOUNT_ID}` | Yes | Per-market account data + open orders |
| `account_tx/{ACCOUNT_ID}` | Yes | Transaction history |
| `account_orders/{MARKET_INDEX}/{ACCOUNT_ID}` | Yes | Market-specific orders |
| `account_all_orders/{ACCOUNT_ID}` | Yes | All orders across markets |
| `account_all_trades/{ACCOUNT_ID}` | Yes | All trades |
| `account_all_positions/{ACCOUNT_ID}` | Yes | All positions |
| `account_all_assets/{ACCOUNT_ID}` | Yes | Asset balances |
| `account_spot_avg_entry_prices/{ACCOUNT_ID}` | Yes | Spot avg entry prices |
| `notification/{ACCOUNT_ID}` | Yes | Liquidation/deleverage alerts |
| `pool_data/{ACCOUNT_ID}` | Yes | LP pool activity |
| `pool_info/{ACCOUNT_ID}` | Yes | LP pool info |

---

## REST Endpoints

### `GET /api/v1/orderBookOrders`

Returns the live resting orders in an order book (bid + ask ladder).

**Parameters**:

| Param | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `market_id` | int16 | Yes | — | Market to query |
| `limit` | int64 | Yes | — | Number of orders to return; **range 1–250** |

**Max depth**: 250 orders per side is the documented limit parameter range. The response contains individual resting orders (each with its own `order_index`, `price`, `base_amount`), not aggregated price levels.

**Weight**: 300

**Use case**: Snapshot fetch when reconnecting or for initial book seed. More granular than WS updates (individual orders vs. aggregated levels).

---

### `GET /api/v1/orderBooks`

Returns metadata for all markets (or a filtered subset). Does NOT return price levels.

**Parameters**:

| Param | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| `market_id` | int16 | No | 255 | Specific market; 255 = all |
| `filter` | string | No | `"all"` | `"all"`, `"spot"`, `"perp"` |

**Response** (per market):
```json
{
  "symbol": "ETH",
  "market_id": 0,
  "market_type": "perp",
  "base_asset_id": 0,
  "quote_asset_id": 0,
  "status": "active",
  "taker_fee": "0.0001",
  "maker_fee": "0.0000",
  "liquidation_fee": "0.01",
  "min_base_amount": "0.01",
  "min_quote_amount": "0.1",
  "supported_size_decimals": 4,
  "supported_price_decimals": 4,
  "supported_quote_decimals": 4,
  "order_quote_limit": "281474976.710655"
}
```

**Key fields for precision**:
- `supported_size_decimals`: decimal places for order sizes
- `supported_price_decimals`: decimal places for order prices
- `supported_quote_decimals`: `size_decimals + quote_decimals` combined

**Weight**: 300

---

### `GET /api/v1/orderBookDetails`

Same as `orderBooks` but includes live market statistics (last price, volume, OI, daily chart).

**Parameters**: same as `orderBooks` (`market_id`, `filter`)

**Additional fields** in response:
```json
{
  "size_decimals": 4,
  "price_decimals": 4,
  "quote_multiplier": 10000,
  "default_initial_margin_fraction": 100,
  "min_initial_margin_fraction": 100,
  "maintenance_margin_fraction": 50,
  "closeout_margin_fraction": 100,
  "last_trade_price": 3024.66,
  "daily_trades_count": 68,
  "daily_base_token_volume": 235.25,
  "daily_quote_token_volume": 93566.25,
  "daily_price_low": 3014.66,
  "daily_price_high": 3024.66,
  "daily_price_change": 3.66,
  "open_interest": 93.0,
  "daily_chart": {"1640995200": 3024.66},
  "market_config": {
    "market_margin_mode": 0,
    "insurance_fund_account_index": 281474976710655,
    "liquidation_mode": 0,
    "force_reduce_only": false,
    "trading_hours": ""
  }
}
```

**Weight**: 300

---

### `GET /api/v1/recentTrades`

Last N trades for a market. Use alongside WS `trade/` channel for book reconstruction.

**Parameters**: `market_id` (required), `limit` (optional)

**Weight**: 600

---

## Snapshot vs Delta

### Behavior on WebSocket Subscription

The `order_book/{MARKET_INDEX}` channel uses a **hybrid snapshot + incremental** model:

1. **On first subscribe**: Server sends a **complete full snapshot** of the current orderbook state. This includes ALL current price levels for both asks and bids.
2. **Subsequent messages**: Server sends **only state changes** (deltas) — price levels that changed, were added, or were removed since the last update.
3. **Re-subscribe = new snapshot**: Sending a new subscription message triggers reconnection and a fresh snapshot.

### How to Maintain a Local Book

```
Step 1: Subscribe to order_book/{market_id}
Step 2: Receive initial snapshot → set local book state, record initial nonce
Step 3: For each incremental update:
    - Verify: update.order_book.begin_nonce == local_book.nonce (previous)
    - If match: apply delta (update/remove levels where size=0, add new levels)
    - If mismatch (gap detected): re-subscribe to get fresh snapshot
Step 4: Update local nonce to update.order_book.nonce
```

### Delta Semantics

- An entry with `size > 0` means: set price level to this size (upsert)
- An entry with `size == 0` means: remove this price level from the book
- Only changed levels are sent in incremental updates

---

## Update Speed

- **WebSocket batch interval**: every **50 ms** (20 updates/second maximum)
- Updates are batched — all changes within a 50 ms window are coalesced into one message
- No configurable update frequency — 50 ms is fixed
- **REST polling**: subject to rate limits; not recommended for real-time book tracking
- **Keepalive**: clients must send at least one frame (ping or any message) every **2 minutes** or the server closes the connection
- Server supports `permessage-deflate` compression for bandwidth reduction
- Server "aggressively disconnects slow readers" — ensure consumers keep up with 50 ms cadence

---

## Price Aggregation

**No server-side price aggregation is available.**

- The WS `order_book/` channel sends raw price levels at their native tick granularity
- No "depth by grouping" (e.g., 1.0 / 5.0 / 10.0 aggregation) is offered by the API
- Price and size precision is per-market and defined in `orderBooks`/`orderBookDetails`:
  - `supported_price_decimals`: max decimal precision for prices (e.g., 4 = 0.0001)
  - `supported_size_decimals`: max decimal precision for sizes
  - `quote_multiplier`: internal multiplier for price representation in transactions
- Clients must implement their own price level grouping / tick aggregation

---

## Checksum

**No checksum mechanism is provided.**

Lighter does not publish a book checksum in WS messages or REST responses. There is no CRC32 or similar integrity check field in the orderbook update payload.

Integrity is maintained via the **nonce/begin_nonce sequence** (see below). The correctness guarantee comes from the ZK-proof layer at the protocol level, not from API-level checksums.

---

## Sequence / Ordering

### Two Separate Counters

Lighter uses **two distinct sequence mechanisms** in orderbook messages:

#### 1. `nonce` / `begin_nonce` — Matching Engine Sequence

- `nonce`: the matching engine's sequence number **after** this batch of changes was applied
- `begin_nonce`: the matching engine's sequence number **before** this batch (= previous message's `nonce`)
- Tied to the **matching engine state**, not to the API server
- **Monotonically increasing**, but may skip values (batch covers multiple matching events)
- **Continuity check**: `current.begin_nonce == previous.nonce` → data is contiguous
- **Gap detection**: if `current.begin_nonce != previous.nonce` → gap exists, re-subscribe

```
Message 1: begin_nonce=100, nonce=105   ← 5 matching events in this batch
Message 2: begin_nonce=105, nonce=108   ← OK, contiguous
Message 3: begin_nonce=110, nonce=115   ← GAP! Expected begin_nonce=108
```

#### 2. `offset` — API Server Sequence

- API-server-scoped counter, increments with each WS message on that server
- **Not continuous**: may skip values
- **Not stable across reconnections**: changes drastically when routed to a different API server
- **Do NOT use offset for gap detection** — use nonce/begin_nonce instead
- Useful only for per-session ordering of messages from the same connection

### Recommended Gap Handling

1. Track `last_nonce` from the most recently applied update
2. On new update: check `update.order_book.begin_nonce == last_nonce`
3. If mismatch: discard the local book and re-subscribe to get a fresh snapshot
4. After reconnect: always expect a fresh full snapshot (offset may be reset)

---

## Account Type Differences

### Market Types

Lighter has two market types with different `market_id` ranges:

| Type | Market ID Range | Examples |
|------|----------------|---------|
| Perp (perpetual futures) | 0–2047 | market_id=0 (ETH), market_id=1 (BTC), ... |
| Spot | 2048+ | market_id=2048 (ETH/USDC) |

### Orderbook Channel Behavior

The `order_book/{MARKET_INDEX}` channel works **identically** for both perp and spot markets — same subscription format, same message format, same nonce/begin_nonce sequence mechanism.

**Differences between perp and spot** (visible in `orderBookDetails` response):

| Field | Perp Markets | Spot Markets |
|-------|-------------|-------------|
| Funding rate | Yes (periodic, 8h equivalent) | No |
| Open interest | Yes | No |
| Leverage / margin | Yes (margin fractions, maintenance, closeout) | No leverage |
| Liquidation fee | Yes | No |
| `market_config` | Has liquidation_mode, insurance_fund | Minimal |
| `order_quote_limit` | Large (leveraged) | Asset-constrained |
| `daily_chart` | Yes | Yes |

**Spot market stats**: use `spot_market_stats/{MARKET_INDEX}` channel (vs `market_stats/` for perps) for market statistics.

### Market ID 255

`market_id=255` is used as a sentinel value meaning "all markets" in REST API filter parameters. It is not a real trading market.

### Perp-Specific Orderbook Properties

For perp markets, funding rates are computed using impact prices from the live orderbook:

> "Each minute, at a random moment, the system snapshots the premium between perp and index prices using the actual impact bid and ask from the orderbook."

The impact bid/ask are derived from the CLOB depth — meaning the live orderbook directly feeds into funding rate calculations. This is relevant for L2 consumers tracking funding exposure.

---

## Raw Examples

### Subscribe to ETH Perp Orderbook (market_id=0)

```json
{"type": "subscribe", "channel": "order_book/0"}
```

### Full Snapshot Response (on subscribe)

```json
{
  "channel": "order_book:0",
  "type": "update/order_book",
  "timestamp": 1714000000,
  "last_updated_at": 1714000000123,
  "offset": 5500,
  "order_book": {
    "code": 200,
    "asks": [
      {"price": "3025.00", "size": "2.0000"},
      {"price": "3026.00", "size": "1.5000"},
      {"price": "3027.00", "size": "5.0000"},
      {"price": "3030.00", "size": "10.0000"}
    ],
    "bids": [
      {"price": "3024.00", "size": "1.0000"},
      {"price": "3023.00", "size": "3.5000"},
      {"price": "3020.00", "size": "7.0000"},
      {"price": "3015.00", "size": "20.0000"}
    ],
    "offset": 5500,
    "nonce": 88200,
    "last_updated_at": 1714000000123,
    "begin_nonce": 0
  }
}
```

Note: On the initial snapshot `begin_nonce` may be 0 or some earlier value — the key is that subsequent incremental updates must have `begin_nonce == prev_nonce`.

### Incremental Update (~50ms later)

```json
{
  "channel": "order_book:0",
  "type": "update/order_book",
  "timestamp": 1714000000050,
  "last_updated_at": 1714000000173,
  "offset": 5501,
  "order_book": {
    "code": 200,
    "asks": [
      {"price": "3025.00", "size": "0.5000"}
    ],
    "bids": [
      {"price": "3024.00", "size": "0.0000"}
    ],
    "offset": 5501,
    "nonce": 88205,
    "last_updated_at": 1714000000173,
    "begin_nonce": 88200
  }
}
```

Interpretation:
- Ask at 3025.00: size changed from 2.0 to 0.5 (partial fill or cancel)
- Bid at 3024.00: size became 0.0 → **remove this level from local book**
- `begin_nonce=88200` matches previous `nonce=88200` → contiguous, no gap

### REST Snapshot Fetch

```
GET https://mainnet.zklighter.elliot.ai/api/v1/orderBookOrders?market_id=0&limit=250
```

Returns up to 250 resting orders (individual orders, not aggregated levels).

### REST Market Precision Lookup

```
GET https://mainnet.zklighter.elliot.ai/api/v1/orderBooks?market_id=0
```

Returns `supported_price_decimals` and `supported_size_decimals` needed for correct decimal handling.

---

## Connection & Rate Limits

### WebSocket (Per IP)

| Limit | Value |
|-------|-------|
| Max connections | 100 |
| Subscriptions per connection | 100 |
| Total subscriptions | 1,000 |
| Connection attempts per minute | 80 |
| Client messages per minute | 200 (excl. sendTx) |
| Inflight messages | 50 max |
| Unique accounts | 100 |
| Keepalive interval | every 2 minutes |

### REST (Per Account Tier)

| Tier | Limit |
|------|-------|
| Builder | 240,000 weighted req/min |
| Premium | 24,000 weighted req/min |
| Standard | 60 req/min |

Endpoint weights: `orderBookOrders` = 300, `orderBooks` = 300, `orderBookDetails` = 300.

---

## Sources

- [Lighter WebSocket Reference](https://apidocs.lighter.xyz/docs/websocket-reference)
- [Lighter API Get Started](https://apidocs.lighter.xyz/docs/get-started)
- [orderBooks endpoint](https://apidocs.lighter.xyz/reference/orderbooks)
- [orderBookDetails endpoint](https://apidocs.lighter.xyz/reference/orderbookdetails)
- [orderBookOrders endpoint](https://apidocs.lighter.xyz/reference/orderbookorders)
- [Rate Limits](https://apidocs.lighter.xyz/docs/rate-limits)
- [Lighter Documentation](https://docs.lighter.xyz)
- [Lighter Whitepaper](https://assets.lighter.xyz/whitepaper.pdf)
- [CCXT Issue #26526](https://github.com/ccxt/ccxt/issues/26526)
