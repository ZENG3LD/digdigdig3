# BingX L2 Orderbook Capabilities

Extracted from: `docs/research/l2-per-exchange-capabilities-2026.md`
Research date: 2026-04-16
Official docs: https://bingx-api.github.io/docs/

---

## Summary

| Property | Value |
|----------|-------|
| WS endpoint | `wss://open-api-ws.bingx.com/market` |
| WS compression | GZIP (mandatory — all WS responses) |
| Snapshot WS | Yes (initialization) |
| Incremental WS | Yes (delta updates) |
| Update speed | 100ms (incremental); real-time BBO (separate) |
| Checksum | Not confirmed |
| Sequence fields | `U` (first update ID), `u` (last update ID) |
| Price aggregation | None confirmed |
| Spot REST endpoint | `GET /openApi/spot/v1/market/depth` |
| Futures REST endpoint | `GET /openApi/swap/v2/quote/depth` |

---

## A. WebSocket Channels

- **WS endpoint:** `wss://open-api-ws.bingx.com/market`
- **Compression:** All WS responses are GZIP-compressed — clients MUST decompress before parsing
- **Depth levels:** 5, 10, 20 confirmed; full list may differ — check live docs [UNVERIFIED: exact complete set]
- **Spot depth levels added:** 2023-11-22 (incremental depth support for spot)
- **Max subscriptions per connection:** 200 (as of 2024-08-20 update)

### Available channels (inferred from docs structure):

| Channel type | Description |
|--------------|-------------|
| Incremental depth (`depth@100ms`) | Delta updates at 100ms intervals |
| Full depth snapshot | Complete book snapshot for initialization |
| BBO (real-time) | Best bid/offer, separate from depth channels |

Note: Exact channel subscription string format (e.g. `{symbol}@depth{levels}@100ms` vs named channel) is [UNVERIFIED — confirm in live BingX docs].

---

## B. REST Depth

| Market | Endpoint |
|--------|----------|
| Spot | `GET /openApi/spot/v1/market/depth` |
| Perpetual Futures | `GET /openApi/swap/v2/quote/depth` |

- REST depth limit parameters: [UNVERIFIED — not stated in source research]
- Used to fetch initial snapshot before applying WS incremental updates

---

## C. Update Types

- **Snapshot:** Full book state — used for initialization
- **Incremental (delta):** Changed price levels only, at 100ms intervals (`depth@100ms`)
- Recommended flow: subscribe to incremental WS → fetch REST snapshot to seed local book → apply deltas using `U`/`u` sequence continuity

---

## D. Update Speed

| Mode | Speed |
|------|-------|
| Incremental depth | 100ms |
| Real-time BBO | Real-time (event-driven, separate channel) |
| Full snapshot | [UNVERIFIED — frequency not stated in source] |

---

## E. Price Aggregation

- **None confirmed** in available documentation [UNVERIFIED]
- No configurable price grouping parameter documented for WS channels

---

## F. Checksum

- **Not confirmed** in available documentation [UNVERIFIED]
- No CRC32 or other integrity check mentioned in source research
- Integrity must be maintained via sequence number continuity (`U`/`u` fields)

---

## G. Sequence / Ordering

| Field | Meaning |
|-------|---------|
| `U` | First update ID in this event |
| `u` | Last update ID in this event |

- Gap detection: `U` of new event should equal `u + 1` of previous event
- No explicit `prevSeqId` or checksum — sequence gaps require re-snapshot from REST
- Same `U`/`u` field naming as Binance / Gate.io diff depth pattern

---

## H. Spot vs Futures Differences

| Property | Spot | Perpetual Futures |
|----------|------|-------------------|
| REST endpoint | `/openApi/spot/v1/market/depth` | `/openApi/swap/v2/quote/depth` |
| WS endpoint | `wss://open-api-ws.bingx.com/market` | Same (market-level endpoint) [UNVERIFIED] |
| Depth channel support | Yes (added 2023-11-22) | Yes |
| Account variants | Standard | Standard + Pro [UNVERIFIED — separate endpoints?] |

- "Perpetual Futures standard and pro" listed as separate account types — exact API surface differences [UNVERIFIED]
- Max 200 WS subscriptions per connection applies to all markets (post 2024-08-20)

---

## Notes & Unverified Items

The following items were flagged as uncertain in source research or are inferred from partial docs:

1. [UNVERIFIED] Exact complete set of supported depth levels (only 5, 10, 20 confirmed)
2. [UNVERIFIED] Exact WS channel subscription string format
3. [UNVERIFIED] Snapshot channel update frequency (how often full re-sends occur)
4. [UNVERIFIED] Whether checksum exists but was undocumented in crawled sources
5. [UNVERIFIED] Exact REST depth limit parameter name and maximum value
6. [UNVERIFIED] Whether Standard vs Pro perpetual futures accounts use different WS endpoints
7. [UNVERIFIED] Price aggregation — absence confirmed only from reviewed docs, not exhaustive

All unverified items should be validated directly against https://bingx-api.github.io/docs/ before implementing `OrderbookCapabilities` struct for BingX.

---

## Sources

- `docs/research/l2-per-exchange-capabilities-2026.md` (section 14)
- [BingX API Docs](https://bingx-api.github.io/docs/)
