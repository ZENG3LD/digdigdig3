# dYdX v4 — L2 Orderbook Capabilities

> Researched April 2026. Official docs at docs.dydx.xyz (redirected from docs.dydx.exchange).
> dYdX v4 runs on its own Cosmos-based appchain (dYdX Chain). Data is served by the Indexer
> (off-chain read-only service) or directly from full nodes via gRPC streaming.

---

## WebSocket Channels

### Endpoint URLs

| Network  | WebSocket Base URL                               |
|----------|--------------------------------------------------|
| Mainnet  | `wss://indexer.dydx.trade/v4/ws`                 |
| Testnet  | `wss://indexer.v4testnet.dydx.exchange/v4/ws`    |

### Available Channels

| Channel Name           | `channel` field value     | Description                          |
|------------------------|---------------------------|--------------------------------------|
| **Orderbook**          | `v4_orderbook`            | Bids/asks for a perpetual market     |
| Trades                 | `v4_trades`               | Executed fills                       |
| Markets                | `v4_markets`              | Market params and oracle prices      |
| Candles                | `v4_candles`              | OHLCV at configurable resolution     |
| Subaccounts            | `v4_subaccounts`          | User positions, orders, fills        |
| Parent Subaccounts     | `v4_parent_subaccounts`   | Isolated position management         |
| Block Height           | `v4_block_height`         | Current blockchain block height      |

### Orderbook Channel — Subscription

```json
{
  "type": "subscribe",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "batched": true
}
```

Fields:
- `type`: always `"subscribe"`
- `channel`: `"v4_orderbook"`
- `id`: market ticker string (e.g. `"BTC-USD"`, `"ETH-USD"`)
- `batched`: boolean; when `true` the server may batch multiple updates into a single frame to reduce message count

Unsubscribe (same structure, no `batched` field):
```json
{
  "type": "unsubscribe",
  "channel": "v4_orderbook",
  "id": "BTC-USD"
}
```

### Connection Limits per WebSocket Connection

| Channel          | Max simultaneous subscriptions |
|------------------|-------------------------------|
| `v4_orderbook`   | **32**                        |
| `v4_trades`      | 32                            |
| `v4_markets`     | 32                            |
| `v4_candles`     | 32                            |
| `v4_subaccounts` | 256                           |

### Depth

The Indexer returns up to **100 levels per side** (bids and asks independently). This is controlled by
`API_ORDERBOOK_LEVELS_PER_SIDE_LIMIT` (default = 100) in the comlink service config. The live endpoint
currently returns ~87 levels per side for liquid markets like BTC-USD.

There is **no client-controllable depth parameter** — the REST response always returns the configured
server-side limit.

---

## REST Endpoints

### Base URLs

| Network  | REST Base URL                             |
|----------|-------------------------------------------|
| Mainnet  | `https://indexer.dydx.trade/v4`           |
| Testnet  | `https://indexer.v4testnet.dydx.exchange/v4` |

### Orderbook Snapshot

```
GET /v4/orderbooks/perpetualMarket/{market}
```

Parameters:
- `market` (path, required): Market ticker, e.g. `BTC-USD`
- No depth/limit parameter — server applies `API_ORDERBOOK_LEVELS_PER_SIDE_LIMIT` (100)
- No pagination

Response format:
```json
{
  "bids": [
    { "price": "74693", "size": "0.0075" },
    { "price": "74692", "size": "0.1200" }
  ],
  "asks": [
    { "price": "74706", "size": "0.0069" },
    { "price": "74707", "size": "0.0500" }
  ]
}
```

- `price`: string (decimal, human-readable USD)
- `size`: string (decimal, base asset units)
- Bids sorted descending by price
- Asks sorted ascending by price
- Server applies `uncrossBook: true` — crossed levels are removed before response

### Other Relevant REST Endpoints

```
GET /v4/perpetualMarkets              # list all markets with metadata
GET /v4/perpetualMarkets?market={t}   # single market metadata
GET /v4/trades/perpetualMarket/{market}  # recent trades
GET /v4/height                        # current block height
GET /v4/time                          # server time
```

### Rate Limits

- REST: 100 requests / 10 seconds per IP
- Rate limit scope: per account; all subaccounts under the same account share the limit
- Current limits queryable via: `GET /dydxprotocol/clob/block_rate` on any REST node

---

## Snapshot vs Delta

### Indexer WebSocket (L2 Aggregated)

The `v4_orderbook` channel follows the standard snapshot-then-delta pattern:

**1. Initial Snapshot** — received immediately after subscription:
```json
{
  "type": "subscribed",
  "connection_id": "abc-123",
  "message_id": 1,
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "contents": {
    "bids": [
      { "price": "74693", "size": "0.0075" },
      { "price": "74692", "size": "1.2000" }
    ],
    "asks": [
      { "price": "74706", "size": "0.0069" },
      { "price": "74707", "size": "0.3000" }
    ]
  }
}
```

- `type`: `"subscribed"` — marks this as the initial full-state snapshot
- `contents.bids` / `contents.asks`: arrays of `{ "price": string, "size": string }`

**2. Incremental Updates** — ongoing delta messages after the snapshot:
```json
{
  "type": "channel_data",
  "connection_id": "abc-123",
  "message_id": 5,
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "version": "2.1.0",
  "contents": {
    "bids": [["74693", "0"], ["74690", "0.5"]],
    "asks": [["74710", "1.5"]]
  }
}
```

- `type`: `"channel_data"` — marks this as an incremental update
- `contents.bids` / `contents.asks`: arrays of `[price, size, offset]` tuples (strings)
  - `size == "0"` → **delete** this price level from local book
  - `size != "0"` → **upsert** this price level (add or replace)
  - `offset` (third element, integer string) — logical timestamp for uncrossing (see Sequence section)

### Maintaining a Local Book

Algorithm for processing `channel_data`:
1. For each `[price, size]` (or `[price, size, offset]`) in `bids`:
   - If `parseFloat(size) === 0` → remove level at that price
   - Else → upsert level, then sort bids descending
2. Same for `asks`, sort ascending
3. Track `message_id` per message for gap detection

### Full Node gRPC Streaming (L3 Order-Level)

The full node exposes an L3 stream with individual order events — not price-level aggregates.

Subscribe via `StreamOrderbookUpdatesRequest`:
```protobuf
message StreamOrderbookUpdatesRequest {
  repeated uint32 clob_pair_id = 1;
  repeated dydxprotocol.subaccounts.SubaccountId subaccount_ids = 2;
}
```

Message types in order stream:
- **`OrderPlaceV1`**: Order added to book at price level end; always followed by `OrderUpdateV1` with fill=0
- **`OrderUpdateV1`**: Updates `total_filled_quantums` (cumulative, not incremental); can arrive before `OrderPlaceV1` (safe to ignore if order not yet in local book)
- **`OrderRemoveV1`**: Order removed from book
- **`ClobMatch`** (`StreamOrderbookFill`): Trade fill event; `fill_amounts` is cumulative total

Snapshot detection for gRPC:
- Discard all messages until `StreamOrderbookUpdate` with `snapshot = true`
- The snapshot message contains complete book state per clob pair

---

## Update Speed

### Indexer WebSocket
- Latency: typically **within milliseconds of block finalization** (~1 second block time on dYdX Chain)
- Indexer WebSocket is **faster than REST** — REST is served from read replicas, ordinarily <1s behind, but may lag under load
- Throughput: Indexer processes 500–1,000 orderbook events/second

### Full Node gRPC Streaming
- Latency: sub-block, includes **optimistic (mempool) updates** before block confirmation
- `exec_mode` field distinguishes optimistic vs confirmed:
  - `0` (`execModeCheck`) — mempool/CheckTx phase, may revert
  - `7` (`execModeFinalize`) — consensus-confirmed DeliverTx, canonical
- Flush interval: configurable via `grpc-streaming-flush-interval-ms` (default 50ms)
- Max buffer: `grpc-streaming-max-batch-size` (default 2000) and `grpc-streaming-max-channel-buffer-size` (default 2000)

### Configurable Speed Options
- `batched: true/false` on WebSocket subscription — controls whether updates are batched per frame (reduces message count, slightly increases latency)
- No client-side throttle or speed selection; all available updates are streamed

---

## Price Aggregation

**The Indexer `v4_orderbook` channel delivers L2 aggregated data only** — prices are aggregated by price level (HSET keyed by `humanPrice`, value = sum of quantums at that price). There is no tick grouping or configurable aggregation.

- No tick grouping API (no "group by 10 USDC" style parameter)
- No configurable aggregation step
- Aggregation granularity matches the market's native `tickSize` / `stepBaseQuantums`

For L3 individual-order visibility, use full node gRPC streaming.

Market metadata (tick size, min order size) available from:
```
GET /v4/perpetualMarkets
```

Key fields per market: `tickSize`, `stepSize`, `minOrderBaseQuantums`, `stepBaseQuantums`, `quantumConversionExponent`.

---

## Checksum

**No checksum mechanism is provided by the dYdX v4 Indexer WebSocket or REST API.**

There is no CRC32, MD5, or other integrity field in orderbook messages. The dYdX protocol explicitly
acknowledges that the orderbook view is subjective (each node may see different states) and does not
guarantee crossed-price-free data.

For the full node gRPC stream, there is also no checksum. Integrity is maintained through the
snapshot-then-delta sequence and the `snapshot` boolean flag.

---

## Sequence / Ordering

### `message_id`

Every outgoing WebSocket message carries:
```
message_id: number  // monotonically increasing integer per connection
```

This is a **per-connection** counter starting at 1 with the `subscribed` acknowledgement. It increments
for every message sent on that connection across all channels.

Use `message_id` to detect gaps: if you receive `message_id` 5 then 7 (skipping 6), you have missed
a message and should re-subscribe to get a fresh snapshot.

### Per-Level `offset`

In `channel_data` incremental updates, each `[price, size, offset]` tuple carries an `offset` field:
- `offset` is an integer string — a logical timestamp for that specific price level update
- Used for **uncrossing the book**: when `best_bid >= best_ask`, compare offsets of the crossed levels

Uncrossing algorithm (Python pseudocode from official docs):
```python
while bids and asks and float(bids[0][0]) >= float(asks[0][0]):
    bid_offset = int(bids[0][2])
    ask_offset = int(asks[0][2])
    if bid_offset < ask_offset:
        bids.pop(0)          # bid is stale, discard
    elif bid_offset > ask_offset:
        asks.pop(0)          # ask is stale, discard
    else:                    # same offset
        if float(bids[0][1]) > float(asks[0][1]):
            asks.pop(0)      # bid has more size, discard ask
        else:
            bids.pop(0)      # ask has more size, discard bid
```

### Why the Book Can Cross

dYdX v4 has **no centralized, global orderbook**. The canonical book at any moment is whatever the
current block proposer holds in its mempool. Block proposers rotate every block (~1 second), causing
slight divergence in canonical state each block. This means:
- Crossed prices are expected and normal
- They resolve within 1–2 blocks
- If you don't need an uncrossed book, just listen to WebSocket updates

### Sequence Gaps

There are no per-channel sequence numbers (unlike e.g. Binance which has `U`/`u` fields). Gap detection
relies solely on the connection-level `message_id`. On gap: re-subscribe (will receive fresh `subscribed`
snapshot message).

---

## Account Type Differences

### Perpetuals Only — No Spot

dYdX v4 is **exclusively a perpetual futures exchange**. There are no spot markets. All orderbook
endpoints operate on perpetual markets only:
- All endpoints use `/perpetualMarket/` path prefix
- All `channel` subscriptions implicitly target perpetuals
- Collateral is USDC; settlement is position P&L, not asset transfer

### Market Sub-Types (v5.0.0+)

Since dYdX Chain v5.0.0 (January 2025), markets have a `marketType` field:

| `marketType`                        | Description                                                   |
|-------------------------------------|---------------------------------------------------------------|
| `PERPETUAL_MARKET_TYPE_CROSS`       | Cross-margin. Multiple positions per subaccount. All pre-v5 markets. |
| `PERPETUAL_MARKET_TYPE_ISOLATED`    | Isolated-margin. One position per subaccount. Added in v5.    |

**Orderbook API is identical for both market types** — same endpoints, same WebSocket channel,
same message format. The `marketType` distinction affects only margin accounting, not data delivery.

Query `marketType` per market via:
```
GET /v4/perpetualMarkets?market=BTC-USD
```

Response includes `marketType`, `tickSize`, `stepSize`, `baseAsset`, `quoteAsset`, etc.

### Subaccount Model

dYdX uses **subaccounts** (numbered 0–127 per wallet address) for position isolation. The orderbook
itself is shared and global — there is no per-account orderbook view.

---

## Raw Examples

### WebSocket Subscribe Request
```json
{
  "type": "subscribe",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "batched": true
}
```

### WebSocket Subscribed (Initial Snapshot)
```json
{
  "type": "subscribed",
  "connection_id": "0f1e2d3c-4b5a-6978-8796-a5b4c3d2e1f0",
  "message_id": 1,
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "contents": {
    "bids": [
      { "price": "74693", "size": "0.0075" },
      { "price": "74692", "size": "1.2000" },
      { "price": "74690", "size": "0.5500" }
    ],
    "asks": [
      { "price": "74706", "size": "0.0069" },
      { "price": "74707", "size": "0.3000" },
      { "price": "74710", "size": "0.8000" }
    ]
  }
}
```

### WebSocket Incremental Update (channel_data)
```json
{
  "type": "channel_data",
  "connection_id": "0f1e2d3c-4b5a-6978-8796-a5b4c3d2e1f0",
  "message_id": 7,
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "version": "2.1.0",
  "contents": {
    "bids": [
      ["74693", "0", "8468"],
      ["74689", "0.3300", "8469"]
    ],
    "asks": [
      ["74706", "0.1500", "8468"],
      ["74715", "0", "8467"]
    ]
  }
}
```

Notes on incremental update format:
- Each entry is `[price: string, size: string, offset: string]`
- `size == "0"` means delete this level
- `offset` is a logical timestamp integer (as string) for uncrossing

### REST Orderbook Response
```
GET https://indexer.dydx.trade/v4/orderbooks/perpetualMarket/BTC-USD
```
```json
{
  "bids": [
    { "price": "74693", "size": "0.0075" },
    { "price": "74692", "size": "1.2000" }
  ],
  "asks": [
    { "price": "74706", "size": "0.0069" },
    { "price": "74707", "size": "0.3000" }
  ]
}
```

### Full Node gRPC WebSocket Connection
```
ws://localhost:9092/ws?clobPairIds=0,1&subaccountIds=
```
- `clobPairIds`: CLOB pair integer IDs (BTC-USD = 0, ETH-USD = 1, etc.)
- First message type: `StreamOrderbookUpdate` with `snapshot = true`
- Subsequent messages: individual `OrderPlaceV1` / `OrderUpdateV1` / `OrderRemoveV1` / `ClobMatch`

---

## Summary Table

| Capability                         | Indexer WebSocket (`v4_orderbook`) | Indexer REST             | Full Node gRPC/WS       |
|------------------------------------|------------------------------------|--------------------------|-------------------------|
| Data type                          | L2 (price-level aggregated)        | L2 snapshot              | L3 (per-order)          |
| Snapshot                           | Yes (`type: subscribed`)           | Yes (always)             | Yes (`snapshot=true`)   |
| Incremental updates                | Yes (`type: channel_data`)         | No (poll only)           | Yes (order events)      |
| Depth per side                     | Up to 100                          | Up to 100                | Full book (all orders)  |
| Depth configurable by client       | No                                 | No                       | No (all or clob-filter) |
| Sequence numbers                   | `message_id` (connection-level)    | N/A                      | No dedicated sequence   |
| Per-level offset                   | Yes (third tuple element)          | No                       | No (order-level)        |
| Checksum                           | None                               | None                     | None                    |
| Crossed prices possible            | Yes (expected, resolves in ~1 blk) | No (uncrossed on serve)  | Yes                     |
| Tick grouping / aggregation        | None                               | None                     | None                    |
| Latency vs block                   | ~ms after finalization             | <1s (read replica lag)   | Before finalization     |
| Max simultaneous subscriptions     | 32 per connection                  | 100 req/10s (IP)         | Unlimited (per node)    |
| Spot market support                | None (perps only)                  | None (perps only)        | None (perps only)       |

---

## Sources

- [dYdX Indexer WebSocket Documentation](https://docs.dydx.xyz/indexer-client/websockets)
- [dYdX Indexer REST HTTP Documentation](https://docs.dydx.xyz/indexer-client/http)
- [dYdX Connecting / Endpoints](https://docs.dydx.xyz/interaction/endpoints)
- [Full Node gRPC Streaming](https://docs.dydx.exchange/api_integration-full-node-streaming)
- [How to Uncross the Orderbook](https://docs.dydx.exchange/api_integration-guides/how_to_uncross_orderbook)
- [Isolated Markets (v5.0.0)](https://docs.dydx.exchange/api_integration-trading/isolated_markets)
- [Indexer Architecture Deep Dive](https://docs.dydx.exchange/concepts-architecture/indexer)
- [v4-clients Python WebSocket source](https://github.com/dydxprotocol/v4-clients/blob/main/v4-client-py-v2/dydx_v4_client/indexer/socket/websocket.py)
- [v4-clients JS SocketClient source](https://github.com/dydxprotocol/v4-clients/blob/main/v4-client-js/src/clients/socket-client.ts)
- [v4-chain socks types.ts](https://github.com/dydxprotocol/v4-chain/blob/main/indexer/services/socks/src/types.ts)
- [v4-chain orderbook-controller.ts](https://github.com/dydxprotocol/v4-chain/blob/main/indexer/services/comlink/src/controllers/api/v4/orderbook-controller.ts)
- [v4-chain orderbook-levels-cache.ts](https://github.com/dydxprotocol/v4-chain/blob/main/indexer/packages/redis/src/caches/orderbook-levels-cache.ts)
- [v4-chain comlink config.ts](https://github.com/dydxprotocol/v4-chain/blob/main/indexer/services/comlink/src/config.ts)
- [Live orderbook endpoint](https://indexer.dydx.trade/v4/orderbooks/perpetualMarket/BTC-USD)
