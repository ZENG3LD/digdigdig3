# Deribit L2 Orderbook Capabilities

Research date: 2026-04-16
Source: `docs/research/l2-per-exchange-capabilities-2026.md` (section 17)
Official docs: https://docs.deribit.com/

---

## Summary

| Property | Value |
|----------|-------|
| WS depth levels | 1, 10, 20 |
| REST depth | configurable (default 5, no explicit max stated) |
| Update speeds | `raw` / `100ms` / `agg2` |
| Price aggregation | Yes — `group` param (BTC and ETH differ) |
| Checksum | No |
| Sequence field | `change_id` + `prev_change_id` |
| Instrument types | BTC/ETH perpetuals, options, dated futures — same channel format |

---

## A. WebSocket Channels

Two channel variants are available.

### Aggregated (grouped) channel

```
book.{instrument_name}.{group}.{depth}.{interval}
```

**`instrument_name`** — e.g. `BTC-PERPETUAL`, `ETH-25MAR25`, `BTC-25MAR25-50000-C`

**`group`** — minimum price increment for grouping:

| Underlying | Valid group values |
|------------|-------------------|
| BTC | `none`, `1`, `2`, `5`, `10` |
| ETH | `none`, `5`, `10`, `25`, `100`, `250` |

- `none` = no grouping (raw price levels)
- ETH group value is divided by 100 (e.g., `group=5` means $0.05 grouping) [UNVERIFIED — source doc notes this but it should be confirmed against actual API behavior]

**`depth`** — number of price levels per side:
- Valid values: **1, 10, 20**

**`interval`** — update speed:
- `raw`: every individual change, lowest latency — **requires authorized connection**
- `100ms`: changes batched every 100ms
- `agg2`: approximately 1-second batches

Example subscription:
```
book.BTC-PERPETUAL.none.20.100ms
book.ETH-PERPETUAL.25.10.raw
```

### Simple channel (no grouping)

```
book.{instrument_name}.{interval}
```

- No `group` or `depth` parameter
- Same `interval` values: `raw`, `100ms`, `agg2`
- Returns all available price levels [UNVERIFIED — exact depth behavior of ungrouped simple channel not explicitly documented]

---

## B. REST Depth

- Endpoint: `GET /public/get_order_book`
- Parameter: `depth` — number of price levels to return
- Default: **5**
- Maximum: not explicitly stated in docs [UNVERIFIED — likely large; test empirically for exact cap]

---

## C. Update Types

Every subscription delivers two phases:

1. **Initial message** — full snapshot of all current price levels at the requested depth
2. **Subsequent messages** — delta updates (changed levels only)

Each entry in the delta has a `type` field:
- `"new"` — new price level added
- `"change"` — existing price level quantity changed
- `"delete"` — price level removed (qty went to zero)

Local book maintenance is required for `raw` and `100ms` intervals when using delta channels.

---

## D. Update Speed Details

| Interval | Latency | Notes |
|----------|---------|-------|
| `raw` | Event-by-event | Requires authenticated/authorized WS connection |
| `100ms` | 100ms batches | No auth required |
| `agg2` | ~1 second batches | No auth required; lower resource usage |

`raw` is the lowest-latency option but requires an authorized connection — an API key with appropriate permissions must be used on the WebSocket.

---

## E. Price Aggregation

The `group` parameter in the aggregated channel controls the minimum price tick for displayed levels.

- Setting `group=none` gives raw ungrouped levels (equivalent to the simple channel)
- Higher group values coarsen the book (fewer levels visible, each representing a wider price band)
- BTC and ETH have **different sets of valid group values** — submitting an unsupported value returns an error [UNVERIFIED — assumed error behavior; actual response not confirmed]
- No runtime reconfiguration — aggregation is set at subscription time; to change it, unsubscribe and resubscribe with a new channel name

---

## F. Checksum

**Not provided.** Deribit does not include CRC32 or any checksum field in orderbook messages. Local book consistency must be maintained via sequence tracking alone.

---

## G. Sequence and Ordering

- `change_id`: monotonically increasing integer assigned to each orderbook update
- `prev_change_id`: the `change_id` of the immediately preceding update

Gap detection: if `prev_change_id` of incoming message does not match the `change_id` of the last processed message, a gap has occurred — re-subscribe or fetch a fresh REST snapshot.

Both snapshot and delta messages carry `change_id`. The initial snapshot's `prev_change_id` is typically `0` or absent.

---

## H. Instrument Type Differences

All Deribit instrument types (perpetuals, dated futures, options) use the **same channel format**. Key differences:

| Aspect | BTC instruments | ETH instruments |
|--------|----------------|----------------|
| Valid `group` values | `none`, `1`, `2`, `5`, `10` | `none`, `5`, `10`, `25`, `100`, `250` |
| ETH group unit interpretation | N/A | Group divided by 100 (e.g., 5 = $0.05) [UNVERIFIED] |
| Options subscription | Per-instrument or all options for underlying | Same |

Options note: Deribit allows subscribing to all options for an underlying simultaneously using a wildcard-style subscription [UNVERIFIED — exact subscription syntax not captured in source].

---

## Implementation Notes

- For maintaining a local L2 book, prefer `100ms` interval unless ultra-low latency is required (avoids auth requirement of `raw`)
- Use `prev_change_id` continuity check on every incoming delta — do not apply out-of-sequence updates
- On gap detection: re-subscribe (server will re-send snapshot) rather than attempting to reconstruct state
- `depth=20` with `100ms` covers most use cases; `raw` + `depth=10` is typical for HFT/arbitrage scenarios

---

## Items Marked UNVERIFIED

1. ETH group-value interpretation (divided by 100) — source doc states it but needs live API confirmation
2. Simple channel (no group/depth params) behavior — exact depth returned is not documented precisely
3. REST endpoint maximum `depth` value — default is 5, actual cap unknown
4. Error response when unsupported `group` value is submitted — assumed error, not confirmed
5. Options wildcard subscription syntax — mentioned in source but exact format not documented

---

## Source Reference

- Section 17 in `docs/research/l2-per-exchange-capabilities-2026.md`
- Official API: https://docs.deribit.com/
