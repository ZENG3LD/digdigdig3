# Bitfinex Trading API — Order Management

Source: https://docs.bitfinex.com/reference

---

## 1. ORDER TYPES

Bitfinex has one of the richest order type sets of any exchange. Types split into two categories:
- **Margin orders** — no prefix, execute via margin/funding accounts
- **Exchange orders** — `EXCHANGE` prefix, execute via spot (exchange) wallet

### Full Order Type List

| Type String | Category | Description |
|---|---|---|
| `LIMIT` | Margin | Standard limit order |
| `MARKET` | Margin | Market order |
| `STOP` | Margin | Stop order (market stop) |
| `STOP LIMIT` | Margin | Stop triggers a limit order |
| `TRAILING STOP` | Margin | Trailing stop, price_trailing param |
| `FOK` | Margin | Fill-or-Kill |
| `IOC` | Margin | Immediate-or-Cancel |
| `EXCHANGE LIMIT` | Spot | Exchange limit order |
| `EXCHANGE MARKET` | Spot | Exchange market order |
| `EXCHANGE STOP` | Spot | Exchange stop |
| `EXCHANGE STOP LIMIT` | Spot | Exchange stop limit |
| `EXCHANGE TRAILING STOP` | Spot | Exchange trailing stop |
| `EXCHANGE FOK` | Spot | Exchange FOK |
| `EXCHANGE IOC` | Spot | Exchange IOC |

**Critical distinction**: `LIMIT` routes to margin account, `EXCHANGE LIMIT` routes to exchange (spot) wallet. For standard spot trading you must use the `EXCHANGE` prefix variants.

### Order Flags (Bitmask — sum values to combine)

| Flag Name | Value | Description |
|---|---|---|
| `HIDDEN` | 64 | Order does not appear in order book |
| `CLOSE` | 512 | Close existing position if one is open |
| `REDUCE_ONLY` | 1024 | Prevents order from flipping an open position |
| `POST_ONLY` | 4096 | Ensures order is added to book (not matched immediately) |
| `OCO` | 16384 | One-Cancels-Other — pairs with `price_oco_stop` |
| `NO_VAR_RATES` | 524288 | Excludes variable rate funding on margin orders |

**Example**: Hidden + Post-Only = 64 + 4096 = **4160**

Default flags value: **0**

---

## 2. ORDER MANAGEMENT ENDPOINTS

### Submit Order

```
POST https://api.bitfinex.com/v2/auth/w/order/submit
```

**Required parameters:**

| Field | Type | Description |
|---|---|---|
| `type` | string | Order type (e.g. `EXCHANGE LIMIT`, `MARKET`) |
| `symbol` | string | Trading pair (e.g. `tBTCUSD`, `tETHUSD`) |
| `amount` | string | Quantity; positive = buy, negative = sell |
| `price` | string | Order price (required for limit orders) |

**Optional parameters:**

| Field | Type | Description |
|---|---|---|
| `lev` | int | Leverage 1–100 (derivatives only, default 10) |
| `price_trailing` | string | Trailing stop distance |
| `price_aux_limit` | string | Limit price for STOP LIMIT orders |
| `price_oco_stop` | string | OCO stop price (requires flag 16384) |
| `gid` | int | Group Order ID (for grouping related orders) |
| `cid` | int | Client Order ID (unique per UTC day) |
| `flags` | int | Sum of flag values (see flags table) |
| `tif` | string | Time-In-Force datetime `"2025-01-15 10:45:23"` UTC |
| `meta` | object | `{aff_code, make_visible, protect_selfmatch}` |

**Rate limit**: 90 req/min

---

### Update Order

```
POST https://api.bitfinex.com/v2/auth/w/order/update
```

Bitfinex supports in-place order updates (not cancel+replace). Works for margin, exchange, and derivative orders.

**Required parameters:**

| Field | Type | Description |
|---|---|---|
| `id` | int64 | Order ID to update |

**Optional parameters:**

| Field | Type | Description |
|---|---|---|
| `amount` | string | New quantity |
| `price` | string | New price |
| `delta` | string | Amount delta to apply (relative change) |
| `price_aux_limit` | string | New aux limit price |
| `price_trailing` | string | New trailing distance |
| `lev` | int | New leverage (derivatives, 1–100) |
| `cid` | int | Client order ID |
| `cid_date` | string | `"YYYY-MM-DD"` format |
| `gid` | int | Group ID |
| `flags` | int | New flags bitmask |
| `tif` | string | New Time-In-Force |
| `meta` | object | Metadata |

**Rate limit**: 90 req/min

---

### Cancel Order

```
POST https://api.bitfinex.com/v2/auth/w/order/cancel
```

| Field | Type | Description |
|---|---|---|
| `id` | int | Order ID to cancel |
| `cid` | int | Client Order ID (use with cid_date) |
| `cid_date` | string | `"YYYY-MM-DD"` date for CID lookup |

---

### Cancel Multiple Orders

```
POST https://api.bitfinex.com/v2/auth/w/order/cancel/multi
```

| Field | Type | Description |
|---|---|---|
| `id` | array[int] | Array of order IDs |
| `gids` | array[int] | Array of Group Order IDs |
| `cids` | array | Array of `[cid, date]` pairs |
| `all` | int | Set to `1` to cancel ALL open orders (trading + derivatives) |

**Rate limit**: 90 req/min

---

### Retrieve Active Orders

```
POST https://api.bitfinex.com/v2/auth/r/orders
POST https://api.bitfinex.com/v2/auth/r/orders/{symbol}
```

Returns array of order arrays (no body parameters needed). Optionally filter by symbol in path.

---

### Order History

```
POST https://api.bitfinex.com/v2/auth/r/orders/hist
POST https://api.bitfinex.com/v2/auth/r/orders/{symbol}/hist
```

Returns closed/cancelled orders up to **2 weeks** in the past.

| Parameter | Type | Description |
|---|---|---|
| `start` | int | Start timestamp (ms) |
| `end` | int | End timestamp (ms) |
| `limit` | int | Max records, cap: **2500** |
| `id` | array | Filter by specific order IDs |

---

### Trades History

```
POST https://api.bitfinex.com/v2/auth/r/trades/hist
POST https://api.bitfinex.com/v2/auth/r/trades/{symbol}/hist
```

| Parameter | Type | Description |
|---|---|---|
| `start` | int | Start timestamp (ms) |
| `end` | int | End timestamp (ms) |
| `limit` | int | Max records |
| `sort` | int | `1` = oldest first, `-1` = newest first |

---

## 3. TP/SL AND CONDITIONAL ORDERS

### OCO (One-Cancels-Other)
Set flag `16384` on a LIMIT order and provide `price_oco_stop`. This creates a linked pair:
- Primary order: LIMIT at `price`
- OCO stop: STOP at `price_oco_stop`
- If either fills/triggers, the other is automatically cancelled

```json
{
  "type": "EXCHANGE LIMIT",
  "symbol": "tBTCUSD",
  "amount": "0.1",
  "price": "50000",
  "flags": 16384,
  "price_oco_stop": "45000"
}
```

### Stop-Limit Orders
Use type `STOP LIMIT` or `EXCHANGE STOP LIMIT` with:
- `price` = stop trigger price
- `price_aux_limit` = limit price after trigger

### Close Flag
Set flag `512` to close an existing open position. The exchange will match the order against the open position before entering the book.

### Reduce-Only Flag
Set flag `1024` — prevents the order from opening a new position if it fills beyond the current position size (derivative trading primarily).

---

## 4. BATCH OPERATIONS — Order Multi-OP

```
POST https://api.bitfinex.com/v2/auth/w/order/multi
```

Allows mixing create, update, and cancel in **a single API call**. Maximum **75 operations** per request.

**Request format:**

```json
{
  "ops": [
    ["on", {"type": "EXCHANGE LIMIT", "symbol": "tBTCUSD", "amount": "0.1", "price": "50000"}],
    ["ou", {"id": 123456789, "price": "51000"}],
    ["oc", {"id": 987654321}],
    ["oc_multi", {"id": [111, 222, 333]}]
  ]
}
```

**Operation codes:**

| Code | Operation | Description |
|---|---|---|
| `"on"` | Order New | Submit a new order |
| `"ou"` | Order Update | Update an existing order |
| `"oc"` | Order Cancel | Cancel a single order |
| `"oc_multi"` | Order Cancel Multi | Cancel multiple orders |

**Response**: Nested array with per-operation status and confirmation. Operations are processed sequentially.

---

## 5. ORDER RESPONSE FORMAT (ARRAY STRUCTURE)

**CRITICAL**: Bitfinex uses array responses, NOT JSON objects. Field positions are fixed.

### Order Notification Response (from submit/update)

```
[MTS, TYPE, MESSAGE_ID, null, DATA, null, STATUS, TEXT]
```

| Index | Field | Type | Description |
|---|---|---|---|
| [0] | MTS | int | Notification timestamp (ms) |
| [1] | TYPE | string | `"on-req"` (submit), `"ou-req"` (update) |
| [2] | MESSAGE_ID | int | Unique notification ID |
| [3] | — | null | Placeholder |
| [4] | DATA | array | Order object array (see below) |
| [5] | — | null | Placeholder |
| [6] | STATUS | string | `"SUCCESS"`, `"ERROR"`, `"FAILURE"` |
| [7] | TEXT | string | Human-readable status description |

### Order Object Array (32 fields)

| Index | Field | Type | Description |
|---|---|---|---|
| [0] | ID | int | Order identifier |
| [1] | GID | int | Group Order ID |
| [2] | CID | int | Client Order ID |
| [3] | SYMBOL | string | Trading pair (e.g. `tBTCUSD`) |
| [4] | MTS_CREATE | int | Creation timestamp (ms) |
| [5] | MTS_UPDATE | int | Last update timestamp (ms) |
| [6] | AMOUNT | float | Current amount (positive=buy, negative=sell) |
| [7] | AMOUNT_ORIG | float | Original amount before modifications |
| [8] | ORDER_TYPE | string | Order type string |
| [9] | TYPE_PREV | string | Previous type before last update |
| [10] | MTS_TIF | int | Time-In-Force expiry timestamp (ms) |
| [11] | — | null | Placeholder |
| [12] | FLAGS | int | Active order flags bitmask |
| [13] | STATUS | string | Order status (see below) |
| [14] | — | null | Placeholder |
| [15] | — | null | Placeholder |
| [16] | PRICE | float | Order price |
| [17] | PRICE_AVG | float | Average execution price |
| [18] | PRICE_TRAILING | float | Trailing stop price |
| [19] | PRICE_AUX_LIMIT | float | Auxiliary limit price (STOP LIMIT) |
| [20] | — | null | Placeholder |
| [21] | — | null | Placeholder |
| [22] | — | null | Placeholder |
| [23] | NOTIFY | int | Notification flag (1=on, 0=off) |
| [24] | HIDDEN | int | Hidden order flag (1=hidden, 0=visible) |
| [25] | PLACED_ID | int | ID of order that triggered this (OCO) |
| [26] | — | null | Placeholder |
| [27] | — | null | Placeholder |
| [28] | ROUTING | string | Origin: `"BFX"` or `"API>BFX"` |
| [29] | — | null | Placeholder |
| [30] | — | null | Placeholder |
| [31] | META | JSON | Metadata (leverage, aff_code, etc.) |

### Order Status Values

- `ACTIVE` — order is in book
- `EXECUTED @ PRICE(AMOUNT)` — fully filled
- `PARTIALLY FILLED @ PRICE(AMOUNT)` — partially filled
- `CANCELED` — cancelled by user or system
- `RSN_DUST` — rejected (too small)
- `RSN_PAUSE` — rejected (trading paused)

### Trades History Response Array (per trade)

| Index | Field | Type | Description |
|---|---|---|---|
| [0] | ID | int | Trade ID |
| [1] | PAIR | string | Symbol |
| [2] | MTS_CREATE | int | Execution timestamp (ms) |
| [3] | ORDER_ID | int | Parent order ID |
| [4] | EXEC_AMOUNT | float | Executed amount (positive=buy, negative=sell) |
| [5] | EXEC_PRICE | float | Execution price |
| [6] | ORDER_TYPE | string | Type of parent order |
| [7] | ORDER_PRICE | float | Original order price |
| [8] | MAKER | int | 1=maker, 0=taker |
| [9] | FEE | float | Fee charged |
| [10] | FEE_CURRENCY | string | Fee currency (e.g. `USD`, `BTC`) |

---

## Sources

- https://docs.bitfinex.com/reference/rest-auth-submit-order
- https://docs.bitfinex.com/reference/rest-auth-update-order
- https://docs.bitfinex.com/reference/rest-auth-cancel-order
- https://docs.bitfinex.com/reference/rest-auth-retrieve-orders
- https://docs.bitfinex.com/reference/rest-auth-orders-history
- https://docs.bitfinex.com/reference/rest-auth-order-multi
- https://docs.bitfinex.com/docs/flag-values
