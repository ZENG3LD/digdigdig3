# OKX L2 Orderbook Capabilities

Research date: 2026-04-16
Source: Extracted from `docs/research/l2-per-exchange-capabilities-2026.md`
Official docs: https://www.okx.com/docs-v5/en/

---

## 1. WebSocket Channels

Five distinct orderbook channels, each with fixed characteristics:

| Channel | Depth (levels/side) | Speed | Type | Access Tier |
|---------|---------------------|-------|------|-------------|
| `bbo-tbt` | 1 (BBO only) | 10ms | Snapshot | All users |
| `books5` | 5 | 100ms | Snapshot | All users |
| `books` | 400 | 100ms | Incremental (snapshot + delta) | All users |
| `books50-l2-tbt` | 50 | 10ms | Incremental (snapshot + delta) | VIP4+ |
| `books-l2-tbt` | 400 | 10ms | Incremental (snapshot + delta) | VIP5+ |

### Channel Behavior Details

- `bbo-tbt` and `books5`: each message is a complete state — no delta, fresh snapshot every push.
- `books`, `books50-l2-tbt`, `books-l2-tbt`: first push on subscription is a full snapshot (`"action": "snapshot"`), subsequent pushes are deltas (`"action": "update"`).
- Incremental channels send initial full snapshot (up to their max depth) on subscribe.

---

## 2. REST Order Book

- Endpoint: `GET /api/v5/market/books`
- Parameter: `sz` (size) — number of levels per side
- Valid `sz` range: 1 to **400**
- Default `sz`: UNVERIFIED — needs docs check
- Separate lite endpoint: `GET /api/v5/market/books-lite` — top 5 levels only (no `sz` parameter)

---

## 3. Update Speed per Channel

| Channel | Update Speed |
|---------|-------------|
| `bbo-tbt` | 10ms |
| `books50-l2-tbt` | 10ms |
| `books-l2-tbt` | 10ms |
| `books5` | 100ms |
| `books` | 100ms |

Speed is fixed per channel — there is no separate speed parameter to configure.

---

## 4. Price Aggregation

- **Not available** via standard WS channels.
- No grouping or price rounding parameter exposed in the orderbook subscription.
- UNVERIFIED — needs docs check for any aggregated depth endpoint or parameter.

---

## 5. Checksum

### Supported
Yes — CRC32 checksums are provided in incremental channels (`books`, `books50-l2-tbt`, `books-l2-tbt`).

### Fields in Message
- `checksum`: the CRC32 value (32-bit signed integer)
- `seqId`: sequence ID of the current update
- `prevSeqId`: sequence ID of the previous update (use to detect gaps)

### Algorithm
1. Take the **top 25 bids** and **top 25 asks** from the local reconstructed orderbook.
2. Interleave them alternately: `bid1[price:qty] : ask1[price:qty] : bid2[price:qty] : ask2[price:qty] : ...`
3. Use the **original price string as received** (e.g., `"0.5000"`, not `"0.5"`) — do NOT trim trailing zeros.
4. Compute CRC32 over the resulting concatenated string.
5. Interpret result as a **32-bit signed integer**.

### Levels Covered
- Top 25 bids + top 25 asks (50 total levels).
- If local book has fewer than 25 levels on either side, use what is available.

### Channels Without Checksum
- `bbo-tbt` (snapshot-only, 1 level) — UNVERIFIED whether checksum is present.
- `books5` (snapshot-only, 5 levels) — UNVERIFIED whether checksum is present.

---

## 6. Sequence / Ordering

| Field | Description |
|-------|-------------|
| `seqId` | Monotonically increasing sequence ID for the current push |
| `prevSeqId` | Expected `seqId` of the immediately preceding message |

### Gap Detection
- On each delta: verify that `prevSeqId` of the new message equals `seqId` of the previous message.
- If a gap is detected, discard local book and re-subscribe to get a fresh snapshot.

### Snapshot vs Delta Identification
- `"action": "snapshot"` — initial full book; replace local state entirely.
- `"action": "update"` — delta; apply changes to local book (qty = 0 means remove level).

---

## 7. Spot vs Futures Differences

| Aspect | Spot | Futures / Swap / Options |
|--------|------|--------------------------|
| Channel names | Same (`books`, `books5`, etc.) | Same (`books`, `books5`, etc.) |
| Subscription `instType` arg | `SPOT` | `FUTURES`, `SWAP`, `OPTION` |
| Available channels | All 5 channels | All 5 channels |
| VIP tier requirements | Same | Same |
| Max REST depth (`sz`) | 400 | 400 |
| Checksum | Yes (incremental channels) | Yes (incremental channels) |
| Sequence fields | `seqId`, `prevSeqId` | `seqId`, `prevSeqId` |

All five orderbook channels are available across instrument types (Spot, Futures, Perpetual Swaps, Options). The `instType` field in the subscription `arg` object is the primary differentiator.

UNVERIFIED — needs docs check: whether Options have any restrictions on depth levels or channel access tiers compared to Spot/Swap.

---

## 8. Connection Notes (UNVERIFIED — needs docs check)

- WebSocket endpoint: `wss://ws.okx.com:8443/ws/v5/public` (public market data)
- Business endpoint: `wss://ws.okx.com:8443/ws/v5/business` (used for some channels — UNVERIFIED which)
- Max subscriptions per connection: UNVERIFIED — needs docs check
- Heartbeat mechanism: UNVERIFIED — needs docs check

---

## Summary Card

```
OKX L2 — Quick Reference
─────────────────────────────────────────────────────
WS Channels (5 total):
  bbo-tbt          depth=1    10ms   snapshot    all users
  books5           depth=5    100ms  snapshot    all users
  books            depth=400  100ms  incremental all users
  books50-l2-tbt   depth=50   10ms   incremental VIP4+
  books-l2-tbt     depth=400  10ms   incremental VIP5+

REST /api/v5/market/books:
  sz: 1–400 (exact default: UNVERIFIED)
  books-lite: top 5 only

Checksum: YES (CRC32, top 25 bids + 25 asks, interleaved price:qty)
Sequence: seqId + prevSeqId on every incremental message
Price Aggregation: NOT supported via WS
Spot vs Futures: same channels, different instType arg
```
