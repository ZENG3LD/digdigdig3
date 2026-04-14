# Crypto.com Exchange API v1 — Trading API Research

Source: https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html
Research Date: 2026-03-11

---

## REST Base URL

```
Production: https://api.crypto.com/exchange/v1/{method}
UAT Sandbox: https://uat-api.3ona.co/exchange/v1/{method}
```

All private methods use HTTP POST with JSON body. All numeric values MUST be strings (wrapped in double quotes) in JSON requests.

---

## Order Types Supported

### Standard Orders (private/create-order)

As of 2026-02-20 (post-migration), only two types remain in the standard endpoint:

| Type | Description |
|------|-------------|
| `LIMIT` | Limit order at specified price |
| `MARKET` | Market order at best available price |

**REMOVED from standard endpoint (migrated to Advanced Order Management API as of 2026-01-28):**
- `STOP_LOSS` — was: market order triggered at ref_price
- `STOP_LIMIT` — was: limit order triggered at ref_price
- `TAKE_PROFIT` — was: market TP triggered at ref_price
- `TAKE_PROFIT_LIMIT` — was: limit TP triggered at ref_price

### Time-in-Force (TIF) Options

| Value | Description |
|-------|-------------|
| `GOOD_TILL_CANCEL` | GTC — remains open until filled or cancelled (default) |
| `IMMEDIATE_OR_CANCEL` | IOC — fill as much as possible immediately, cancel remainder |
| `FILL_OR_KILL` | FOK — fill entire order immediately or cancel entirely |

**NOT DOCUMENTED:** GTD (Good Till Date) — not available in v1.

### Execution Instructions (exec_inst — array field)

| Value | Description |
|-------|-------------|
| `POST_ONLY` | Reject order if it would execute as taker |
| `SMART_POST_ONLY` | Post-only with smart queue handling |
| `ISOLATED_MARGIN` | Execute as isolated margin order |

---

## Order Placement

### Single Order: `private/create-order`

**Method:** POST
**Path:** `https://api.crypto.com/exchange/v1/private/create-order`
**Rate Limit:** 15 requests per 100ms
**Response:** Asynchronous confirmation only (no immediate fill info)

#### Request Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `instrument_name` | string | YES | e.g. `BTC_USDT`, `BTCUSD-PERP` |
| `side` | string | YES | `BUY` or `SELL` |
| `type` | string | YES | `LIMIT` or `MARKET` |
| `price` | string | Depends | Required for LIMIT orders |
| `quantity` | string | Depends | Order quantity (required except MARKET BUY with notional) |
| `notional` | number | Depends | For MARKET BUY only — amount in quote currency to spend |
| `client_oid` | string | NO | Client-assigned order ID, max 36 characters |
| `exec_inst` | array of string | NO | `POST_ONLY`, `SMART_POST_ONLY`, `ISOLATED_MARGIN` |
| `time_in_force` | string | NO | `GOOD_TILL_CANCEL`, `IMMEDIATE_OR_CANCEL`, `FILL_OR_KILL` |
| `spot_margin` | string | NO | `SPOT` (non-margin) or `MARGIN` (margin order) |
| `stp_scope` | string | NO | Self-trade prevention scope: `M` (master+sub), `S` (sub only) |
| `stp_inst` | string | NO | STP instruction: `M` (cancel maker), `T` (cancel taker), `B` (cancel both) |
| `stp_id` | string of number | NO | STP group ID: 0–32767 |
| `fee_instrument_name` | string | NO | Preferred fee token (USD/USDT/EUR or base/quote currency) |
| `isolation_id` | string | NO | For isolated margin position orders |
| `leverage` | string | NO | Max leverage for isolated position |
| `isolated_margin_amount` | string | NO | Amount to transfer to isolated position on open |

#### Example Request

```json
{
  "id": 1,
  "nonce": 1610905028000,
  "method": "private/create-order",
  "api_key": "YOUR_API_KEY",
  "params": {
    "instrument_name": "BTCUSD-PERP",
    "side": "SELL",
    "type": "LIMIT",
    "price": "50000.5",
    "quantity": "1",
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02",
    "exec_inst": ["POST_ONLY"],
    "time_in_force": "FILL_OR_KILL"
  },
  "sig": "COMPUTED_HMAC_SHA256"
}
```

#### Example Response

```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 0,
  "result": {
    "client_oid": "c5f682ed-7108-4f1c-b755-972fcdca0f02",
    "order_id": "18342311"
  }
}
```

**Note:** Response is asynchronous — only returns `order_id` and `client_oid`. Use `private/get-order-detail` to check actual fill status.

**Limits:** Max 200 open orders per trading pair; max 1000 open orders per account.

---

### Batch/Bulk Orders: `private/create-order-list`

**Method:** POST
**Path:** `https://api.crypto.com/exchange/v1/private/create-order-list`
**Rate Limit:** 15 requests per 100ms
**Max Batch Size:** 10 orders per request

#### Contingency Types

| `contingency_type` | Behavior |
|--------------------|----------|
| `LIST` | Independent orders — none linked, each executes or fails independently |
| `OCO` | One-Cancels-Other — two orders; when one executes, the other is cancelled |

**Note:** OCO in `create-order-list` is the legacy mechanism. New OCO orders should use `private/advanced/create-oco`.

#### Request Parameters

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `contingency_type` | string | YES | `LIST` or `OCO` |
| `order_list` | array | YES | 1–10 order objects (same fields as create-order params) |

Each object in `order_list` accepts the same fields as `private/create-order`.

---

## Conditional / Advanced Orders (Advanced Order Management API)

**Migration Note:** As of 2026-01-28, stop-loss and take-profit orders were fully migrated to the Advanced Order Management API. Trigger orders migrated as of 2025-12-17.

### `private/advanced/create-order` (Conditional Order)

Replaces the removed STOP_LOSS, STOP_LIMIT, TAKE_PROFIT, TAKE_PROFIT_LIMIT types.

- Used for: stop-loss, stop-limit, take-profit, take-profit-limit, trigger orders
- Full parameter specification: NOT FULLY DOCUMENTED in public-facing content retrieved

### One-Cancels-Other: `private/advanced/create-oco`

Creates a two-legged OCO order where executing one leg automatically cancels the other.

### One-Triggers-Other: `private/advanced/create-oto`

Creates a primary order which, when filled, triggers a secondary order.

### One-Triggers-One-Cancels-Other: `private/advanced/create-otoco`

Bracket order: primary order triggers an OCO pair (TP + SL around a position).

### Complete Advanced Order Endpoint List

| Endpoint | Description |
|----------|-------------|
| `private/advanced/create-order` | Conditional (trigger) order |
| `private/advanced/create-oco` | One-Cancels-Other |
| `private/advanced/cancel-oco` | Cancel an OCO order |
| `private/advanced/create-oto` | One-Triggers-Other |
| `private/advanced/cancel-oto` | Cancel an OTO order |
| `private/advanced/create-otoco` | One-Triggers-One-Cancels-Other (bracket) |
| `private/advanced/cancel-otoco` | Cancel an OTOCO order |
| `private/advanced/cancel-order` | Cancel single advanced order |
| `private/advanced/cancel-all-orders` | Cancel all advanced orders |
| `private/advanced/get-open-orders` | List open advanced orders |
| `private/advanced/get-order-detail` | Get single advanced order detail |
| `private/advanced/get-order-history` | Advanced order history |

---

## Order Management

### Cancel Single Order: `private/cancel-order`

**Rate Limit:** 15 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | string | Depends | Order ID to cancel |
| `client_oid` | string | Depends | Client order ID (alternative to order_id) |
| `instrument_name` | string | YES | Instrument to cancel on |

### Cancel Multiple Orders: `private/cancel-order-list`

Batch cancellation. Accepts an array of order IDs.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `contingency_type` | string | YES | Must be `LIST` |
| `order_list` | array | YES | Array of objects with `order_id` or `client_oid` + `instrument_name` |

### Cancel All Orders: `private/cancel-all-orders`

**Rate Limit:** 15 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | NO | If specified, cancel only for this instrument |
| `type` | string | NO | Filter by order type |

**No result block returned** — response code 0 means request was queued successfully.

### Amend/Modify Order: `private/amend-order`

Added 2025-06-10. Supports modifying price and/or quantity without cancel+replace.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | string | Depends | Order to amend (or use orig_client_oid) |
| `orig_client_oid` | string | Depends | Alternative to order_id |
| `new_price` | string | Depends | New price (at least one of new_price/new_quantity required) |
| `new_quantity` | string | Depends | New quantity |

**Queue Priority:** Amending either price OR increasing quantity causes the order to LOSE queue priority. Exception: amending ONLY to reduce quantity does NOT lose queue priority.

### Get Single Order: `private/get-order-detail`

**Rate Limit:** 30 requests per 100ms

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `order_id` | string | Depends | Order ID |
| `client_oid` | string | Depends | Alternative to order_id |

### Get Open Orders: `private/get-open-orders`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | NO | Filter by instrument; omit for all |
| `page_size` | int | NO | Max orders returned, default 20, max 200 |
| `page` | int | NO | Page number, 0-based |

Response includes fields: `order_id`, `client_oid`, `instrument_name`, `side`, `type`, `price`, `quantity`, `exec_inst`, `status`, `time_in_force`, `isolation_id`, `isolation_type`, timestamps.

### Get Order History: `private/get-order-history`

**Rate Limit:** 1 request per second

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | NO | Filter by instrument; omit for all |
| `start_ts` | long | NO | Start timestamp (ms since Unix epoch), default 24 hours ago |
| `end_ts` | long | NO | End timestamp (ms since Unix epoch), default now |
| `page_size` | int | NO | Max records, default 20, max 200 |
| `page` | int | NO | Page number, 0-based |

**History retention:** 6 months. Older records require contacting support.

---

## Position Management (Derivatives / Margin)

### Get Positions: `private/get-positions`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | NO | Filter by instrument |

Response fields include: `quantity`, `cost`, `open_pos_cost`, `open_position_pnl`, `session_pnl`, `isolation_id`, `isolation_type`, timestamps.

### Close Position: `private/close-position`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | YES | Instrument to close |
| `type` | string | YES | `LIMIT` or `MARKET` |
| `price` | string | Depends | Required for LIMIT |
| `isolation_id` | string | NO | For isolated margin positions |

Response includes: `order_id`, `client_oid`.

### Set Account Leverage: `private/change-account-leverage`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `account_id` | string | YES | Account UUID |
| `leverage` | string | YES | Desired max leverage |

**Note:** Each instrument has its own max leverage ceiling. Whichever is lower (account-level or instrument-level) is applied.

### Set Isolated Position Leverage: `private/change-isolated-margin-leverage`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `isolation_id` | string | YES | Isolated position ID |
| `leverage` | string | YES | New leverage |

### Add/Remove Margin: `private/create-isolated-margin-transfer`

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `isolation_id` | string | YES | Isolated position ID |
| `amount` | string | YES | Amount (positive = add, negative = remove from isolated position) |

**Note:** Must be a positive number when transferring TO isolated position.

### Funding Rate History: `public/get-valuations`

Public endpoint for funding rate history.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `instrument_name` | string | YES | Perpetual instrument, e.g. `BTCUSD-PERP` |
| `valuation_type` | string | YES | `funding_hist` for hourly settled funding rates |
| `count` | int | NO | Number of records |
| `start_ts` | long | NO | Start timestamp |
| `end_ts` | long | NO | End timestamp |

### Risk Parameters / Liquidation Info: `public/get-risk-parameters`

Returns default max leverage, maintenance margin requirements, and other risk settings per instrument. Liquidation price is NOT returned as a separate endpoint — it is derived from position data and risk parameters.

### Change Margin Mode: NOT AVAILABLE

The API does not expose a direct "change margin mode (cross/isolated)" endpoint. Isolated positions are created by specifying `isolation_id` and `leverage` at order creation time. Margin mode switching as a standalone operation is NOT DOCUMENTED.

---

## Advanced Features

### Algo Orders (TWAP, Iceberg): NOT AVAILABLE

TWAP and iceberg order types are NOT DOCUMENTED in the Exchange API v1. NOT AVAILABLE through this API.

### Bracket Orders: AVAILABLE (via OTOCO)

Full bracket orders (entry + TP + SL) are available through `private/advanced/create-otoco`.

### Copy Trading API: NOT DOCUMENTED

NOT AVAILABLE through the Exchange API v1.

### Grid Trading API: NOT DOCUMENTED

NOT AVAILABLE through the Exchange API v1.

### Self-Trade Prevention (STP)

Available on standard orders via:
- `stp_scope`: `M` (master+sub accounts) or `S` (sub-account only)
- `stp_inst`: `M` (cancel maker), `T` (cancel taker), `B` (cancel both)
- `stp_id`: Group ID 0–32767 for grouping accounts in STP rules

### Fee Token Selection

Available on standard orders via `fee_instrument_name`. Users can specify preferred fee currency (USD, USDT, EUR, or base/quote of the instrument).

---

## WebSocket Trading (User API)

**WebSocket URL:** `wss://stream.crypto.com/exchange/v1/user`

All private REST trading methods are also available over WebSocket with identical request/response formats. WebSocket auth is done once per session via `public/auth`.

**Rate Limit:** 150 requests per second for User API WebSocket.

**Important:** Add 1-second sleep after establishing WebSocket connection before sending requests to avoid rate-limit errors (limits are pro-rated per calendar second).

---

## Sources

- [Crypto.com Exchange API v1 Official Docs](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index.html)
- [Crypto.com Exchange Institutional API v1](https://exchange-docs.crypto.com/exchange/v1/rest-ws/index-insto-8556ea5c-4dbb-44d4-beb0-20a4d31f63a7.html)
- [Order Amendment Announcement](https://crypto.com/us/product-news/exchange-order-fast-amendments-spot-derivatives)
