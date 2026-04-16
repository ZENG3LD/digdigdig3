# Bitfinex L2 Order Book Capabilities

Research date: 2026-04-16
Source: https://docs.bitfinex.com

---

## 1. WebSocket Book Channel

### Connection Endpoints

| Type | URL |
|------|-----|
| Public | `wss://api-pub.bitfinex.com/ws/2` |
| Authenticated | `wss://api.bitfinex.com/ws/2` |

### Subscribe Message

```json
{
  "event": "subscribe",
  "channel": "book",
  "symbol": "tBTCUSD",
  "prec": "P0",
  "freq": "F0",
  "len": "25",
  "subId": "optional-client-id"
}
```

All parameters except `channel` and `symbol` are optional with defaults shown.

---

## 2. Precision Levels (prec)

| Level | Description | Significant Figures |
|-------|-------------|---------------------|
| P0 | Default — most granular aggregation | 5 significant figures |
| P1 | Medium aggregation | 4 significant figures |
| P2 | Coarser aggregation | 3 significant figures |
| P3 | Very coarse | 2 significant figures |
| P4 | Least granular | 1 significant figure |
| R0 | Raw order book — no aggregation, individual orders | Order ID used instead of COUNT |

**P0-P4** are aggregated books: multiple orders at similar prices are grouped into a single price level. COUNT field indicates how many orders are in the bucket.

**R0** is the raw (unaggregated) book: each entry is a single order with its own ORDER_ID. A `len=25` subscription returns 25 individual orders, not 25 price levels.

---

## 3. Depth Levels (len)

| Value | Description |
|-------|-------------|
| `"1"` | 1 price point per side |
| `"25"` | 25 price points per side (default) |
| `"100"` | 100 price points per side |
| `"250"` | 250 price points per side |

Same valid values apply to both WebSocket (`len`) and REST (`len` query param).

---

## 4. Update Frequency (freq)

| Value | Behavior |
|-------|----------|
| `F0` | Real-time — updates sent immediately as they occur (default) |
| `F1` | Throttled — updates batched and sent every 2 seconds |

Only F0 and F1 are documented. No other frequency values are available.

---

## 5. Snapshot and Update Message Format

### Snapshot (received once on subscribe)

**Trading pair (tBTCUSD) — aggregated (P0-P4):**
```
[CHAN_ID, [[PRICE, COUNT, AMOUNT], [PRICE, COUNT, AMOUNT], ...]]
```

**Funding currency (fUSD) — aggregated:**
```
[CHAN_ID, [[RATE, PERIOD, COUNT, AMOUNT], ...]]
```

**Raw book (R0) — trading:**
```
[CHAN_ID, [[ORDER_ID, PRICE, AMOUNT], ...]]
```

**Raw book (R0) — funding:**
```
[CHAN_ID, [[OFFER_ID, PERIOD, RATE, AMOUNT], ...]]
```

### Update (subsequent messages)

**Aggregated trading:**
```
[CHAN_ID, [PRICE, COUNT, AMOUNT]]
```

**Raw trading:**
```
[CHAN_ID, [ORDER_ID, PRICE, AMOUNT]]
```

### Update Interpretation — Aggregated Books

| Condition | Action |
|-----------|--------|
| COUNT > 0 | Update price level with new AMOUNT |
| COUNT = 0, AMOUNT = 1 | Remove price level from bids |
| COUNT = 0, AMOUNT = -1 | Remove price level from asks |

### Update Interpretation — Raw Books

| Condition | Action |
|-----------|--------|
| PRICE > 0 | Update order with new AMOUNT |
| PRICE = 0 | Remove order with that ORDER_ID |

---

## 6. Trading vs Funding Pairs — Side Convention

The sign convention for AMOUNT is **inverted** between trading and funding:

| Pair Type | Positive AMOUNT | Negative AMOUNT |
|-----------|-----------------|-----------------|
| Trading (tXXXYYY) | Bid side | Ask side |
| Funding (fXXX) | Ask side (offers) | Bid side |

Funding books also have an extra `PERIOD` field (integer, number of days for the loan offer).

---

## 7. Heartbeat

Server sends heartbeat every 15 seconds:
```
[CHAN_ID, "hb"]
```

No data is lost between heartbeats — it is a keep-alive signal only.

---

## 8. Checksum

### Enabling

Send a configuration event immediately after WebSocket connection:

```json
{
  "event": "conf",
  "flags": 131072
}
```

Flag value: `OB_CHECKSUM = 131072`

### Checksum Message Format

```
[CHAN_ID, "cs", CHECKSUM_INT32]
```

Delivered after each book update (not each heartbeat).

### Coverage

Covers the **top 25 bids and top 25 asks** (50 levels total).

### Computation Procedure — Aggregated Books (P0-P4)

1. Sort bids descending by price, asks ascending by price.
2. Take top 25 bids and top 25 asks from local state.
3. Interleave bid and ask entries alternating: bid[0], ask[0], bid[1], ask[1], ...
4. For each entry extract: `price` and `amount`.
5. Build flat array: `[bid_price_0, bid_amount_0, ask_price_0, ask_amount_0, bid_price_1, ...]`
6. Concatenate values with colon separator:
   `"50968755521:0.10783801:50968615681:-0.4675:..."`
7. Compute CRC-32 of the resulting string.
8. Compare against server-provided checksum value.

### Computation Procedure — Raw Books (R0)

Same procedure but substitute `ORDER_ID` in place of `price`:
- For same-price levels: sub-sort by ORDER_ID ascending before interleaving.
- Build array: `[order_id_0, amount_0, order_id_1, amount_1, ...]`
- Concatenate with colons, apply CRC-32.

### Checksum Mismatch

If checksums do not match: re-subscribe to the channel to get a fresh snapshot.

---

## 9. Sequence Numbers (Gap Detection)

**Flag**: `SEQ_ALL = 65536` (currently beta)

```json
{
  "event": "conf",
  "flags": 65536
}
```

When enabled, sequence numbers are appended to all event messages, allowing detection of:
- Dropped packets
- Out-of-order message delivery

This feature is marked **beta** in the documentation — behavior may change.

---

## 10. Additional WebSocket Flags

Multiple flags can be combined with bitwise OR:

| Flag Name | Value | Effect |
|-----------|-------|--------|
| TIMESTAMP | 32768 | Adds millisecond timestamps to all events |
| SEQ_ALL | 65536 | Enables sequence numbers (beta) |
| OB_CHECKSUM | 131072 | Enables CRC32 checksum per book update |
| BULK_UPDATES | 536870912 | Multiple book updates arrive in a single array message |

Example combining checksum + timestamp:
```json
{
  "event": "conf",
  "flags": 163840
}
```
(131072 + 32768 = 163840)

---

## 11. REST Order Book

### Endpoint

```
GET https://api-pub.bitfinex.com/v2/book/{symbol}/{precision}
```

### Path Parameters

| Parameter | Required | Values |
|-----------|----------|--------|
| `symbol` | Yes | `tBTCUSD`, `fUSD`, etc. |
| `precision` | Yes | `P0`, `P1`, `P2`, `P3`, `P4`, `R0` |

### Query Parameters

| Parameter | Type | Default | Valid Values |
|-----------|------|---------|--------------|
| `len` | int32 | 25 | `1`, `25`, `100`, `250` |

### Response Formats

**Aggregated — trading pair (P0-P4):**
```json
[[PRICE, COUNT, AMOUNT], ...]
```

**Aggregated — funding currency (P0-P4):**
```json
[[RATE, PERIOD, COUNT, AMOUNT], ...]
```

**Raw — trading pair (R0):**
```json
[[ORDER_ID, PRICE, AMOUNT], ...]
```

**Raw — funding currency (R0):**
```json
[[OFFER_ID, PERIOD, RATE, AMOUNT], ...]
```

---

## 12. Rate Limits and Connection Limits

| Resource | Limit |
|----------|-------|
| WebSocket connections | 20 per minute (new connections) |
| Public channel subscriptions per connection | 30 |
| Authenticated connection channel reservation | 1 (for account info) |

---

## 13. Symbol Format

| Type | Format | Example |
|------|--------|---------|
| Trading pair | `t` + BASE + QUOTE | `tBTCUSD`, `tETHUSD` |
| Funding currency | `f` + CURRENCY | `fUSD`, `fBTC` |

All symbols must be uppercase.

---

## Summary Table

| Feature | Value |
|---------|-------|
| Precision levels | P0 (5 sig figs), P1, P2, P3, P4 (1 sig fig), R0 (raw) |
| Depth levels | 1, 25 (default), 100, 250 |
| Update frequency | F0 (real-time, default), F1 (2-second batching) |
| Checksum | CRC32, top 25 bids + 25 asks, flag 131072 |
| Sequence numbers | SEQ_ALL flag 65536, beta feature |
| Bulk updates | BULK_UPDATES flag 536870912 |
| Heartbeat | Every 15 seconds `[CHAN_ID, "hb"]` |
| Funding vs Trading | Inverted AMOUNT sign; PERIOD field added in funding |
| REST endpoint | `GET /v2/book/{symbol}/{precision}?len=N` |

---

## Sources

- [WebSocket Books Channel](https://docs.bitfinex.com/reference/ws-public-books)
- [WebSocket Raw Books Channel](https://docs.bitfinex.com/reference/ws-public-raw-books)
- [WebSocket Checksum](https://docs.bitfinex.com/docs/ws-websocket-checksum)
- [WebSocket General / Flags](https://docs.bitfinex.com/docs/ws-general)
- [REST Public Book](https://docs.bitfinex.com/reference/rest-public-book)
