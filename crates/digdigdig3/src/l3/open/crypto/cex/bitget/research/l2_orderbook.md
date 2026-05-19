# Bitget L2 Orderbook Capabilities

Research date: 2026-04-16
Source: `docs/research/l2-per-exchange-capabilities-2026.md` (section 13)
Official docs:
- https://www.bitget.com/api-doc/spot/websocket/public/Order-Book
- https://www.bitget.com/api-doc/contract/websocket/public/Order-Book-Channel

---

## WS Channels

Four named channels, identical for spot and futures:

| Channel  | Depth         | Default Speed | Update Model                        |
|----------|---------------|---------------|-------------------------------------|
| `books1` | 1 (BBO)       | 100ms / 20ms* | Periodic snapshot                   |
| `books5` | 5 levels      | 150ms         | Periodic snapshot                   |
| `books15`| 15 levels     | 150ms         | Periodic snapshot                   |
| `books`  | All levels    | 150ms         | First push = snapshot, then delta   |

*`books1` was optimized to **20ms** as of 2026-01-07 for high-liquidity symbols:
BTCUSDT, ETHUSDT, XRPUSDT, SOLUSDT, SUIUSDT, DOGEUSDT, ADAUSDT, PEPEUSDT, LINKUSDT, HBARUSDT
(Classic Account v2 only). All other symbols remain at 100ms. [UNVERIFIED: cutoff list may expand]

Spot vs futures distinction is made via the `instType` subscription parameter, not separate channel names.

---

## REST Depth

- Spot endpoint: `GET /api/v2/spot/market/orderbook`
- Futures endpoint: `GET /api/v2/mix/market/depth`
- Max limit (spot): **150 levels**
- Max limit (futures): not explicitly stated in source — check official docs [UNVERIFIED]

---

## Update Speed Summary

| Channel  | Speed (top symbols) | Speed (others)  |
|----------|---------------------|-----------------|
| `books1` | 20ms                | 100ms           |
| `books5` | 150ms               | 150ms           |
| `books15`| 150ms               | 150ms           |
| `books`  | 150ms               | 150ms           |

No configurable speed parameter — speed is determined by channel choice.
There is no sub-10ms or tick-by-tick mode available [UNVERIFIED: no mention in source].

---

## Price Aggregation

**Not supported** via WebSocket. No grouping parameter exists in any channel.
REST also does not expose a grouping parameter (source does not mention one).

---

## Checksum

**Yes — CRC32 (32-bit signed integer)**

- Covers **top 25 bids and 25 asks** from local book state
- If subscribed depth has fewer than 25 levels on either side, use all available
- If depth > 25, truncate to 25 per side before computing

Algorithm (interleaved format):

```
bid1[price:amount]:ask1[price:amount]:bid2[price:amount]:ask2[price:amount]:...
```

- Use the **original price string** as received — do NOT strip trailing zeros or reformat
- Result is a signed 32-bit CRC32 integer

This matches OKX's checksum format (both use top-25, interleaved, original strings).

---

## Sequence / Ordering

- Field: **`seqId`** — monotonically increasing sequence ID per message
- Present in both snapshot and delta pushes from the `books` channel
- For `books1` / `books5` / `books15` (snapshot-only channels): use `seqId` for staleness detection only; no gap-filling needed since each message is complete state
- For `books` (delta channel): `seqId` must be contiguous — gaps indicate missed updates; re-subscribe and re-initialize from snapshot on gap [UNVERIFIED: gap recovery procedure not explicitly documented in source]

---

## Spot vs Futures

| Property           | Spot                                    | Futures                                         |
|--------------------|-----------------------------------------|-------------------------------------------------|
| Channel names      | `books`, `books1`, `books5`, `books15`  | Same names                                      |
| Subscription param | `instType` = SPOT                       | `instType` = USDT-FUTURES / USDC-FUTURES / COIN-FUTURES / SUSDT-FUTURES |
| REST endpoint      | `/api/v2/spot/market/orderbook`         | `/api/v2/mix/market/depth`                      |
| REST max depth     | 150                                     | [UNVERIFIED — check futures docs]               |
| Checksum           | Yes (CRC32, top 25)                     | Yes (same algorithm) [UNVERIFIED for futures]   |
| `seqId` field      | Yes                                     | Yes                                             |
| Update speeds      | Same as table above                     | Same as table above                             |

Futures subtypes: USDT-FUTURES, USDC-FUTURES, COIN-FUTURES, SUSDT-FUTURES — all use the same channel names, distinguished only by `instType`.

---

## Local Book Maintenance (books channel)

1. Subscribe to `books`
2. First message received has `action: "snapshot"` (or equivalent) — initialize local book from it
3. All subsequent messages are deltas — apply price-level changes (qty=0 means remove level)
4. After each delta applied, verify CRC32 against top 25 bids/asks
5. On checksum mismatch or `seqId` gap: re-subscribe, discard local state, reinitialize from next snapshot

For `books1` / `books5` / `books15`: no local book maintenance needed — each message is a complete state at that depth.

---

## Items Marked UNVERIFIED

- `books1` 20ms optimization: symbol list may have changed since 2026-01-07 cutoff
- Futures REST max depth: not stated in source
- Gap recovery procedure for `books` delta channel: re-subscription assumed but not explicitly confirmed
- CRC32 applicability to futures (vs spot only): assumed same but not confirmed in source
- Whether `seqId` gaps are detectable via a single field or require additional prev-seq tracking
- No source confirmation that `books` channel includes an explicit `action` field ("snapshot"/"delta") — behavior described but field name not confirmed

---

## Sources

- Bitget Spot Orderbook WS: https://www.bitget.com/api-doc/spot/websocket/public/Order-Book
- Bitget Contract Orderbook WS: https://www.bitget.com/api-doc/contract/websocket/public/Order-Book-Channel
- Parent research doc: `docs/research/l2-per-exchange-capabilities-2026.md` section 13
