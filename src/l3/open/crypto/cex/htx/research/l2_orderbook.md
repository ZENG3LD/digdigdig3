# HTX (Huobi) — L2 Orderbook Capabilities

Research date: 2026-04-16
Source: `docs/research/l2-per-exchange-capabilities-2026.md` (section 12)
Official docs: https://huobiapi.github.io/docs/spot/v1/en/

> Items marked **[UNVERIFIED]** are inferred from docs structure or partial sources and should be validated against the live API before implementing.

---

## Summary

| Property | Value |
|----------|-------|
| WS depth levels (spot) | 5, 10, 20, 150, 400 (mbp); step0–step5 (traditional) |
| WS depth levels (futures) | Similar to spot [UNVERIFIED] |
| REST max depth | 150 (step0) / 20 (step1–step5) |
| Snapshot WS | Yes — mbp.refresh channel |
| Delta WS | Yes — mbp incremental channel |
| Update speed | 100ms (most levels); tick-by-tick for mbp level 5 |
| Checksum | No |
| Sequence field | `seqNum` (mbp), `version` (traditional depth) |

---

## A. WebSocket Channels

HTX exposes **two parallel depth systems**. They are independent and should not be mixed.

### System 1 — Market By Price (MBP) Incremental

Channel pattern:
```
market.{symbol}.mbp.{levels}
```

| Levels | Update Speed | WS Endpoint |
|--------|-------------|-------------|
| 5 | tick-by-tick (fastest) | `wss://api.huobi.pro/feed` only |
| 10 | 100ms | standard WS endpoint |
| 20 | 100ms | standard WS endpoint |
| 150 | 100ms | standard WS endpoint |
| 400 | 100ms | standard WS endpoint (added 2021-08-26) |

**Key notes:**
- Level 5 is available ONLY at the dedicated feed endpoint `wss://api.huobi.pro/feed` — it is NOT available on the default endpoint.
- Levels 10/20/150/400 use 100ms batched updates.
- MBP is an **incremental delta** system — a local book must be maintained.

### MBP Refresh Channel (Re-synchronization Snapshots)

Channel pattern:
```
market.{symbol}.mbp.refresh.{levels}
```

- Sends periodic **full snapshots** for re-synchronization after gaps or reconnects.
- This is the only way to initialize a local MBP book via WebSocket (no initial snapshot on subscribe).
- Workflow: subscribe to refresh → get full snapshot → align `seqNum` → apply buffered/ongoing incrementals.

### System 2 — Traditional Depth (Snapshot-based)

Channel pattern:
```
market.{symbol}.depth.{type}
```

| Type | Levels | Aggregation | Update Speed |
|------|--------|-------------|-------------|
| step0 | 150 | None (raw prices) | 100ms |
| step1 | 20 | Coarse level 1 | 100ms |
| step2 | 20 | Coarse level 2 | 100ms |
| step3 | 20 | Coarse level 3 | 100ms |
| step4 | 20 | Coarse level 4 | 100ms |
| step5 | 20 | Coarsest | 100ms |

- Traditional depth pushes **full snapshots** each tick — no local book maintenance required.
- step0 = no price grouping (finest granularity, 150 levels).
- step1–step5 = progressively coarser price grouping, 20 levels each.
- Aggregation is configured at subscription time; cannot be changed at runtime without re-subscribing.

---

## B. REST Depth

Endpoint: `GET /market/depth`

| Parameter | Values |
|-----------|--------|
| `symbol` | e.g., `btcusdt` |
| `type` | `step0` through `step5` |
| `depth` | `5`, `10`, or `20` |

**Behavior by type:**
- `step0`: returns up to **150** price levels (no aggregation).
- `step1`–`step5`: returns up to **20** price levels (with aggregation).
- The `depth` parameter (5/10/20) sub-selects within the returned levels. [UNVERIFIED: whether `depth` applies to step0 or only step1–step5]

---

## C. Update Types

| Channel | Update Type | Local Book Required |
|---------|------------|---------------------|
| `mbp.{levels}` | Delta (incremental) | Yes |
| `mbp.refresh.{levels}` | Full snapshot | No (used to seed local book) |
| `depth.{type}` | Full snapshot (re-sent each tick) | No |

**MBP local book maintenance procedure:**
1. Subscribe to `mbp.refresh.{levels}` and `mbp.{levels}` simultaneously.
2. Buffer all incoming incremental messages.
3. Wait for a refresh snapshot; record its `seqNum`.
4. Discard buffered messages with `seqNum` <= snapshot's `seqNum`.
5. Apply remaining buffered messages in order, then continue applying live incrementals.
6. Use `seqNum` to detect gaps; request a new refresh snapshot if a gap is detected.

---

## D. Update Speed

| Channel / Scenario | Speed |
|--------------------|-------|
| MBP level 5 (at `wss://api.huobi.pro/feed`) | Tick-by-tick (every trade/change) |
| MBP levels 10/20/150/400 | 100ms |
| Traditional depth (step0–step5) | 100ms |
| MBP refresh snapshots | Periodic (interval unspecified) [UNVERIFIED] |

---

## E. Price Aggregation (step0–step5)

Price aggregation is available **only via the traditional depth system** (System 2):

| Step | Levels | Grouping |
|------|--------|---------|
| step0 | 150 | No aggregation — raw exchange prices |
| step1 | 20 | Finest grouping |
| step2 | 20 | Coarser |
| step3 | 20 | Coarser |
| step4 | 20 | Coarser |
| step5 | 20 | Coarsest grouping |

**Notes:**
- Grouping step values (the actual price increment per step) are **symbol-dependent** and are NOT documented as fixed values. They are determined by HTX based on the symbol's price range. [UNVERIFIED: exact step sizes per symbol]
- MBP channels (System 1) do NOT support price aggregation — they always return raw prices.
- Aggregation is a subscription-time parameter; changing it requires re-subscribing.

---

## F. Checksum

**Not provided.** HTX does not include CRC32 or any checksum field in depth messages.

To detect book corruption, use `seqNum` continuity checks (MBP) or re-request a refresh snapshot.

---

## G. Sequence / Ordering

### MBP channels

- Field: `seqNum` (integer, monotonically increasing).
- Each incremental message has a `seqNum`.
- The refresh snapshot also carries a `seqNum`.
- Validation: messages must be applied in ascending `seqNum` order. A gap (missing seqNum) means data loss — must re-sync via a fresh refresh snapshot.
- Deduplication: discard any message with `seqNum` <= last applied `seqNum`.

### Traditional depth channels

- Field: `version` (integer).
- Since these are full snapshots, ordering is informational — the latest version wins.
- No gap detection needed (each message is self-contained).

---

## H. Spot vs Futures Differences

| Property | Spot | USDT-M Swaps | Coin-M Swaps |
|----------|------|-------------|-------------|
| WS endpoint | `wss://api.huobi.pro` | Separate endpoint | Separate endpoint |
| Channel prefix | `market.*` | Different prefix [UNVERIFIED] | Different prefix [UNVERIFIED] |
| Docs URL | huobiapi.github.io/docs/spot/v1/en/ | huobiapi.github.io/docs/usdt_swap/ | huobiapi.github.io/docs/coin_margined_swap/ |
| Depth levels | 5/10/20/150/400 (mbp); step0–step5 | Similar [UNVERIFIED] | Similar [UNVERIFIED] |
| Tick-by-tick level 5 | Yes (at /feed endpoint) | [UNVERIFIED] | [UNVERIFIED] |
| Checksum | No | No [UNVERIFIED] | No [UNVERIFIED] |
| seqNum field | Yes (mbp) | [UNVERIFIED] | [UNVERIFIED] |

**Important:** Spot, USDT-M Swaps, and Coin-M Swaps each have **entirely separate documentation, WebSocket endpoints, and channel naming conventions.** Do not assume spot channel names or endpoints apply to futures/swaps without verifying against the respective docs.

---

## Implementation Notes

1. **Two distinct booking systems** — pick one per use case:
   - MBP incremental: lowest latency, requires local book + seqNum tracking, refresh channel for init.
   - Traditional depth: simplest (snapshots only), use when no local book maintenance is desired.

2. **Level 5 tick-by-tick** requires connecting to `wss://api.huobi.pro/feed`, not the main WS endpoint. This is a separate connection.

3. **No checksum** — implement seqNum gap detection for MBP as the primary integrity mechanism.

4. **step0 vs MBP raw** — both provide unaggregate prices, but step0 is snapshot-based (simpler) while MBP is delta-based (lower latency). For deep books (150+ levels), only MBP 150/400 or step0 REST applies.

5. **Futures require separate connector logic** — different endpoints, possibly different channel formats. Verify against USDT swap and Coin-M swap docs independently.

---

## Sources

- HTX/Huobi Spot API: https://huobiapi.github.io/docs/spot/v1/en/
- USDT-M Swap docs: https://huobiapi.github.io/docs/usdt_swap/
- Coin-M Swap docs: https://huobiapi.github.io/docs/coin_margined_swap/
- Extracted from `docs/research/l2-per-exchange-capabilities-2026.md` section 12
