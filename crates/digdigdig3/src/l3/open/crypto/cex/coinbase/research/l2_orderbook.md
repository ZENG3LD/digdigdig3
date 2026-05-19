# Coinbase Advanced Trade — L2 Orderbook Capabilities

Research date: 2026-04-16
Source: `docs/research/l2-per-exchange-capabilities-2026.md` (section 6)
Official docs: https://docs.cdp.coinbase.com/advanced-trade/docs/ws-channels

---

## Summary

| Property | Value |
|----------|-------|
| WS channels | `level2`, `level2_batch` |
| Configurable depth | No — full book only |
| REST depth endpoint | Yes (`/api/v3/brokerage/product/book`, `limit` param) |
| Update type | Absolute quantities (not deltas) |
| Update speed | Event-driven (no fixed interval) |
| Price aggregation | No |
| Checksum | No |
| Sequence field | `sequence` |
| Markets | Spot only (Advanced Trade API) |

---

## A. WebSocket Channels

- Channel names: **`level2`** and **`level2_batch`**
- No configurable depth level — provides full book updates (all price levels)
- Message structure:
  - `type`: `"snapshot"` or `"update"`
  - `product_id`: trading pair identifier
  - `updates`: array of book change entries
- Each entry in `updates`:
  - `price_level`: price string
  - `new_quantity`: absolute quantity at that level (not a delta)
  - `event_time`: timestamp of the change
  - `side`: `"bid"` or `"offer"`
- `new_quantity = "0"`: signals removal of that price level from the book

**`level2` vs `level2_batch` distinction:** [UNVERIFIED] `level2_batch` likely batches multiple updates per message at the cost of slightly higher latency vs `level2` which may push updates individually. Official docs treat them as variants of the same channel type.

---

## B. REST Depth

- Endpoint: `GET /api/v3/brokerage/product/book`
- Parameter: `limit` — number of bid/ask levels to return
- Maximum `limit` value: [UNVERIFIED] not explicitly stated in available docs
- Returns a snapshot at the time of the request
- Requires authentication (brokerage endpoint, not public)

---

## C. Update Types

Two message types over the WebSocket connection:

| `type` field | Meaning |
|-------------|---------|
| `"snapshot"` | Initial full book state sent on subscription |
| `"update"` | Subsequent changes (may contain multiple price level changes) |

**Key characteristic:** quantities in `update` messages are **absolute** (the new total quantity at that price), not incremental deltas. This simplifies local book maintenance — no accumulation needed, just replace.

To remove a level: check `new_quantity == "0"` (or equivalent zero string).

---

## D. Update Speed

- **Event-driven** — no fixed stated interval
- Messages are pushed when the book changes, not on a timer
- No 100ms / 500ms bucket options like some other exchanges
- Latency characteristics: [UNVERIFIED] no SLA or typical latency figures found in docs

---

## E. Price Aggregation

- **Not supported** — no grouping or tick-size aggregation parameter
- Prices are at native tick resolution

---

## F. Checksum

- **Not provided**
- No CRC32 or equivalent integrity verification field in messages
- Local book correctness must be maintained via sequence field and reconnect logic

---

## G. Sequence / Ordering

- `sequence` field present in messages
- Monotonically increasing integer per product
- Use to detect gaps (missed messages) — if received sequence is not `prev_sequence + 1`, a reconnect and re-snapshot is required
- [UNVERIFIED] Whether `sequence` is per-product or global across all subscriptions on the connection

---

## H. Account Type Differences

| API | Market | Notes |
|-----|--------|-------|
| Advanced Trade API (`api.coinbase.com`) | Spot only | `level2` / `level2_batch` channels live here |
| Exchange (GDAX) API (`advanced-trade-api.coinbase.com` legacy) | Spot | Separate WebSocket with different channel names (`full`, `level2`, `ticker`) — different protocol |
| Coinbase International (INTX) | Perpetuals / Derivatives | Completely separate API — [UNVERIFIED] different WS endpoint and channel structure |

- Advanced Trade API is the current recommended API for retail and institutional spot trading
- The legacy Exchange (GDAX) API WebSocket uses a different subscription format and channel naming — do not mix with Advanced Trade channels
- Futures/derivatives via Coinbase International: [UNVERIFIED] assumed to have separate L2 channels; not covered by the Advanced Trade docs

---

## Local Book Maintenance Recipe

1. Subscribe to `level2` or `level2_batch` for the desired product
2. Receive first message with `type="snapshot"` — initialize local book from this
3. For each subsequent `type="update"`:
   - For each entry in `updates`:
     - If `new_quantity == "0"`: delete `price_level` from local book side
     - Otherwise: set `price_level` on local book side to `new_quantity`
   - Check `sequence` continuity; if gap detected, re-subscribe and re-snapshot
4. No delta accumulation needed — quantities are absolute

---

## Known Gaps / Unverified Items

| Item | Status |
|------|--------|
| `level2` vs `level2_batch` behavioral difference | UNVERIFIED |
| REST `limit` maximum value | UNVERIFIED |
| `sequence` scope (per-product vs global) | UNVERIFIED |
| Coinbase International L2 channel names and format | UNVERIFIED |
| Typical WS message latency / p99 | UNVERIFIED |
| Authentication requirement for `level2` WS subscription | UNVERIFIED (Advanced Trade WS channels typically require auth) |

---

## Source

- [Coinbase Advanced Trade WebSocket Channels](https://docs.cdp.coinbase.com/advanced-trade/docs/ws-channels)
- [Coinbase Advanced Trade API reference (CDP portal)](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-channels)
- Extracted from: `docs/research/l2-per-exchange-capabilities-2026.md`, section 6
