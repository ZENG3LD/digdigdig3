# Upbit Trading API — Order Endpoints

Source: https://global-docs.upbit.com/reference
Exchange type: **Spot only** (no futures, no margin, no leverage — confirmed by multiple sources)
Base URL pattern: `https://{region}-api.upbit.com/v1`
Regions: `sg` (Singapore), `id` (Indonesia), `th` (Thailand)

---

## Order Types Supported

Upbit supports four `ord_type` values:

| `ord_type` | Description |
|------------|-------------|
| `limit` | Limit buy or sell at a fixed price |
| `price` | Market **buy** order (specify total amount in quote currency) |
| `market` | Market **sell** order (specify quantity of base asset) |
| `best` | Best available price order (requires `time_in_force`) |

Note: `price` and `market` are asymmetric — `price` is used for market buys, `market` is used for market sells. This is a naming quirk of the Upbit API.

### Time-in-Force (TIF) Options

| `time_in_force` | Description |
|-----------------|-------------|
| `ioc` | Immediate or Cancel — fill available quantity, cancel remainder |
| `fok` | Fill or Kill — fill entire quantity or cancel entirely |
| `post_only` | Place only as maker; cancel if would execute as taker |

TIF applicability:
- `best` orders: **require** `fok` or `ioc` (not `post_only`)
- `limit` orders: support `ioc`, `fok`, `post_only` optionally
- `price` / `market` orders: TIF is optional

### Bid (Buy) Types — composite view

| Type | Description |
|------|-------------|
| `price` | Market buy by total spend amount |
| `limit` | Limit buy |
| `limit_ioc` | Limit buy, Immediate or Cancel |
| `limit_fok` | Limit buy, Fill or Kill |
| `best_ioc` | Best price buy, Immediate or Cancel |
| `best_fok` | Best price buy, Fill or Kill |

### Ask (Sell) Types — composite view

| Type | Description |
|------|-------------|
| `market` | Market sell by quantity |
| `limit` | Limit sell |
| `limit_ioc` | Limit sell, Immediate or Cancel |
| `limit_fok` | Limit sell, Fill or Kill |
| `best_ioc` | Best price sell, Immediate or Cancel |
| `best_fok` | Best price sell, Fill or Kill |

### What is NOT supported

- Stop-Limit orders: NOT AVAILABLE
- Stop-Market orders: NOT AVAILABLE
- Trailing Stop orders: NOT AVAILABLE
- GTD (Good Till Date): NOT AVAILABLE
- GTC (Good Till Cancelled): NOT DOCUMENTED as explicit TIF — open orders remain until cancelled implicitly
- OCO (One-Cancels-Other): NOT AVAILABLE
- Bracket orders: NOT AVAILABLE

---

## Order Placement

### Create Order

**Method:** `POST`
**Path:** `/orders`
**Full URL:** `https://{region}-api.upbit.com/v1/orders`
**Auth:** Bearer JWT (requires `Make Orders` permission)
**Rate limit group:** `order` — 8 req/sec

#### Parameters (request body)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market` | string | Yes | Trading pair code (e.g., `SGD-BTC`) |
| `side` | string | Yes | `bid` (buy) or `ask` (sell) |
| `ord_type` | string | Yes | `limit`, `price`, `market`, or `best` |
| `volume` | string | Conditional | Quantity of base asset. Required for: limit buy/sell, market sell, best-price sell |
| `price` | string | Conditional | Unit price (limit/best) or total spend amount (market buy). Required for: limit buy/sell, market buy |
| `identifier` | string | No | Client-defined order ID for deduplication/tracking |
| `time_in_force` | string | Conditional | `fok`, `ioc`, or `post_only`. Required for `best` orders |
| `smp_type` | string | No | Self-Match Prevention mode: `cancel_maker`, `cancel_taker`, or `reduce` |

#### Response (HTTP 201)

```json
{
  "market": "SGD-BTC",
  "uuid": "9ca023a5-851b-4fec-9f0a-48cd83c2eaae",
  "side": "bid",
  "ord_type": "limit",
  "price": "30000.0",
  "state": "wait",
  "created_at": "2024-01-01T00:00:00+00:00",
  "volume": "0.001",
  "remaining_volume": "0.001",
  "executed_volume": "0.0",
  "reserved_fee": "0.03",
  "remaining_fee": "0.03",
  "paid_fee": "0.0",
  "locked": "30.03",
  "prevented_volume": null,
  "prevented_locked": null,
  "trades_count": 0
}
```

### Test Order (Dry Run)

**Method:** `POST`
**Path:** `/orders/test`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/test`
**Auth:** Bearer JWT (requires `Make Orders` permission)
**Rate limit group:** `order-test` — 8 req/sec

Validates the order request and returns what the order would look like without actually submitting it to the exchange. Accepts the same parameters as `POST /orders`.

### Batch Order Placement

NOT AVAILABLE — Upbit does not support bulk/batch order creation in a single request.

### Conditional Orders (TP/SL, OCO, Bracket)

NOT AVAILABLE — Upbit does not support conditional, OCO, or bracket orders.

### Algo Orders (TWAP, Iceberg, Grid, Copy Trading)

NOT AVAILABLE — no algorithmic order types documented.

---

## Order Management

### Get Single Order

**Method:** `GET`
**Path:** `/order`
**Full URL:** `https://{region}-api.upbit.com/v1/order`
**Auth:** Bearer JWT (requires `View Orders` permission)
**Rate limit group:** `default` — 30 req/sec

At least one parameter is required:

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid` | string | System-assigned order UUID |
| `identifier` | string | Client-defined order identifier |

When both are provided, `uuid` takes precedence.

#### Response fields

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Trading pair |
| `uuid` | string | System order ID |
| `side` | string | `ask` or `bid` |
| `ord_type` | string | `limit`, `price`, `market`, `best` |
| `price` | string | Unit price or total amount |
| `state` | string | `wait`, `watch`, `done`, `cancel` |
| `created_at` | string | UTC timestamp |
| `volume` | string | Requested quantity |
| `remaining_volume` | string | Unfilled quantity |
| `executed_volume` | string | Filled quantity |
| `reserved_fee` | string | Fee reserved at order creation |
| `remaining_fee` | string | Fee not yet consumed |
| `paid_fee` | string | Fee already paid |
| `locked` | string | Locked balance for this order |
| `prevented_volume` | string | Qty cancelled by SMP |
| `prevented_locked` | string | Balance released via SMP |
| `trades_count` | integer | Number of partial fills |
| `trades` | array | Array of individual fill objects |

#### Trade object fields

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | Trading pair |
| `uuid` | string | Trade ID |
| `price` | string | Fill price |
| `volume` | string | Fill quantity |
| `funds` | string | Fill value in quote currency |
| `trend` | string | Price trend (`up` / `down`) |
| `created_at` | string | Fill timestamp |
| `side` | string | `bid` or `ask` |

---

### List Orders by Multiple IDs

**Method:** `GET`
**Path:** `/orders/uuids`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/uuids`
**Auth:** Bearer JWT (requires `View Orders` permission)
**Rate limit group:** `default` — 30 req/sec

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid[]` | string array | System UUIDs |
| `identifier[]` | string array | Client identifiers |

Maximum batch size: NOT DOCUMENTED in official reference.

---

### List Open (Pending) Orders

**Method:** `GET`
**Path:** `/orders/open`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/open`
**Auth:** Bearer JWT (requires `View Orders` permission)
**Rate limit group:** `default` — 30 req/sec

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `market` | string | — | Filter by trading pair (e.g., `SGD-BTC`) |
| `state` | string | `wait` | Single state filter: `wait` or `watch` |
| `states[]` | array | `["wait"]` | Multiple state filter |
| `page` | integer | `1` | Page number |
| `limit` | integer | `100` | Results per page (max: 100) |
| `order_by` | string | `desc` | Sort direction: `asc` or `desc` |

Order states:
- `wait` — pending, not yet matched
- `watch` — conditional order monitoring state
- `done` — fully filled
- `cancel` — cancelled

---

### List Closed Orders

**Method:** `GET`
**Path:** `/orders/closed`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/closed`
**Auth:** Bearer JWT (requires `View Orders` permission)
**Rate limit group:** `default` — 30 req/sec

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `market` | string | — | Filter by trading pair |
| `state` | string | — | Filter by state: `done` or `cancel` |
| `page` | integer | `1` | Page number |
| `limit` | integer | — | Results per page |
| `order_by` | string | `desc` | Sort direction: `asc` or `desc` |

Note: Complete parameter documentation for this endpoint is partially incomplete in official docs at time of research.

---

### Cancel Single Order

**Method:** `DELETE`
**Path:** `/order`
**Full URL:** `https://{region}-api.upbit.com/v1/order`
**Auth:** Bearer JWT (requires `Make Orders` permission)
**Rate limit group:** `default` — 30 req/sec

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid` | string | System order UUID |
| `identifier` | string | Client order identifier |

At least one required. When both provided, `uuid` takes precedence. Returns cancelled order object.

---

### Cancel Multiple Orders by IDs

**Method:** `DELETE`
**Path:** `/orders/uuids`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/uuids`
**Auth:** Bearer JWT (requires `Make Orders` permission)

| Parameter | Type | Description |
|-----------|------|-------------|
| `uuid[]` | string array | System UUIDs to cancel |
| `identifier[]` | string array | Client identifiers to cancel |

---

### Batch Cancel All Orders

**Method:** `DELETE`
**Path:** `/orders/open`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/open`
**Auth:** Bearer JWT (requires `Make Orders` permission)
**Rate limit group:** `order-cancel-all` — **1 req per 2 seconds**

Can cancel up to **300 orders** in a single request. Accepts filter parameters to narrow which open orders are cancelled (e.g., by `market`).

---

### Cancel-and-New Order (Atomic Replace)

**Method:** `POST`
**Path:** `/orders/cancel_and_new`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/cancel_and_new`
**Auth:** Bearer JWT (requires `Make Orders` permission)
**Rate limit group:** `order` — 8 req/sec

This is the closest Upbit has to order amendment — it atomically cancels an existing order and creates a new one. This is **not** a true modify/amend (no PATCH endpoint exists).

Parameters:

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market` | string | Yes | Trading pair |
| `side` | string | Yes | `bid` or `ask` |
| `ord_type` | string | Yes | `limit`, `price`, `market`, `best` |
| `volume` | string | Conditional | Quantity (same rules as create order) |
| `price` | string | Conditional | Price (same rules as create order) |
| `identifier` | string | No | New client order identifier |
| `time_in_force` | string | Conditional | `fok`, `ioc`, `post_only` |
| `smp_type` | string | No | SMP mode |

Note: The cancellation target (which order to cancel) is NOT explicitly documented in the available reference pages. Likely identified by `identifier` field or the original order UUID.

---

## Get Available Order Info (Per Market)

**Method:** `GET`
**Path:** `/orders/info`
**Full URL:** `https://{region}-api.upbit.com/v1/orders/info`
**Auth:** Bearer JWT
**Rate limit group:** `default` — 30 req/sec

Returns per-market order constraints and current account balances for that market.

| Response Field | Type | Description |
|----------------|------|-------------|
| `market` | object | Market code and name |
| `bid_types` | array | Supported buy order types for this market |
| `ask_types` | array | Supported sell order types for this market |
| `order_types` | array | All supported order types (deprecated field) |
| `bid.currency` | string | Quote currency |
| `bid.min_total` | string | Minimum buy order value in quote currency |
| `bid.max_total` | string | Maximum available buy amount |
| `bid.state` | string | Market buy status (`active`) |
| `ask.currency` | string | Base currency |
| `ask.min_total` | string | Minimum sell quantity |
| `ask.state` | string | Market sell status |
| `bid_account` | object | Current account balance for quote currency |
| `ask_account` | object | Current account balance for base currency |

---

## Position Management

NOT APPLICABLE — Upbit is a **spot-only** exchange. There are no futures, margin, perpetuals, or leverage products. The following features do not exist:

- Get positions: NOT AVAILABLE
- Close position: NOT AVAILABLE
- Set leverage: NOT AVAILABLE
- Cross/isolated margin mode: NOT AVAILABLE
- Add/remove margin: NOT AVAILABLE
- Funding rate: NOT AVAILABLE
- Liquidation price: NOT AVAILABLE

---

## Advanced Trading Features

| Feature | Status |
|---------|--------|
| TWAP / Iceberg orders | NOT AVAILABLE |
| Bracket orders | NOT AVAILABLE |
| Copy trading API | NOT AVAILABLE |
| Grid trading API | NOT AVAILABLE |
| Conditional orders (TP/SL) | NOT AVAILABLE |
| OCO orders | NOT AVAILABLE |

Self-Match Prevention (SMP) is available via the `smp_type` parameter:
- `cancel_maker` — cancel the resting (maker) order
- `cancel_taker` — cancel the incoming (taker) order
- `reduce` — partially cancel to avoid self-match

---

## Sources

- [Upbit Global Developer Center — Reference](https://global-docs.upbit.com/reference)
- [Create Order](https://global-docs.upbit.com/reference/new-order)
- [Available Order Information](https://global-docs.upbit.com/reference/available-order-information)
- [List Open Orders (versioned)](https://global-docs.upbit.com/v1.2.2/reference/open-order)
- [Get Order](https://global-docs.upbit.com/reference/get-order)
- [Test Order](https://global-docs.upbit.com/reference/order-test)
