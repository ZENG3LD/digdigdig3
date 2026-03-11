# Paradex DEX - Trading API Specification

Source: https://docs.paradex.trade/
Researched: 2026-03-11

## Base URLs

| Environment | REST Base URL |
|-------------|---------------|
| Mainnet (prod) | `https://api.prod.paradex.trade/v1` |
| Testnet (Sepolia) | `https://api.testnet.paradex.trade/v1` |

Swagger UI: `https://api.prod.paradex.trade/swagger/index.html`

---

## Order Types Supported

### Standard Order Types (REST `type` field)

| Type | Description |
|------|-------------|
| `MARKET` | Execute immediately at current market price; use `price: "0"` |
| `LIMIT` | Execute at specified price or better |
| `STOP_LIMIT` | Trigger-activated limit order |
| `STOP_MARKET` | Trigger-activated market order |
| `TAKE_PROFIT_LIMIT` | TP trigger → limit execution |
| `TAKE_PROFIT_MARKET` | TP trigger → market execution |
| `STOP_LOSS_LIMIT` | SL trigger → limit execution |
| `STOP_LOSS_MARKET` | SL trigger → market execution |

### Algorithmic Order Types (separate `/v1/algo/orders` endpoint)

| Type | Description |
|------|-------------|
| `TWAP` | Time-Weighted Average Price; sub-orders every 30 seconds, duration 30–86400 sec in 30s increments; sub-order type is MARKET only |

### Advanced (UI) Order Types

| Type | Description |
|------|-------------|
| Scaled Order | Distributes order quantity across multiple price levels |
| TP/SL combined | Take Profit + Stop Loss pair |

---

## Time-in-Force (TIF) — `instruction` field

| Value | Description |
|-------|-------------|
| `GTC` | Good Till Canceled (default if omitted) |
| `IOC` | Immediate or Cancel; unfilled portion is canceled |
| `POST_ONLY` | Maker only; order is canceled if it would take liquidity |
| `RPI` | Retail Price Improvement — special liquidity designation |

---

## Order Flags

| Flag | Description |
|------|-------------|
| `REDUCE_ONLY` | Only reduces existing position; minimum order value exemption applies |
| `STOP_CONDITION_BELOW_TRIGGER` | Stop fires when price goes below trigger |
| `STOP_CONDITION_ABOVE_TRIGGER` | Stop fires when price goes above trigger |
| `INTERACTIVE` | UI-placed order designation |
| `TARGET_STRATEGY_VWAP` | VWAP-targeted execution strategy |

---

## Self-Trade Prevention (STP) — `stp` field

| Value | Description |
|-------|-------------|
| `EXPIRE_MAKER` | Cancel the resting (maker) order |
| `EXPIRE_TAKER` | Cancel the incoming (taker) order |
| `EXPIRE_BOTH` | Cancel both orders |

---

## Order Placement

### Create Single Order

```
POST https://api.prod.paradex.trade/v1/orders
Authorization: Bearer {JWT}
Content-Type: application/json
```

**Required request body fields:**

| Field | Type | Description |
|-------|------|-------------|
| `market` | string | e.g. `"BTC-USD-PERP"` |
| `side` | string | `"BUY"` or `"SELL"` |
| `type` | string | Order type (see table above) |
| `price` | string | Price as decimal string; `"0"` for market orders |
| `size` | string | Order quantity as decimal string |
| `instruction` | string | TIF instruction (GTC/IOC/POST_ONLY/RPI); defaults to GTC |
| `signature` | string | STARK signature `"[r,s]"` signed by Paradex private key |
| `signature_timestamp` | integer | Unix timestamp milliseconds at signing time |

**Optional request body fields:**

| Field | Type | Description |
|-------|------|-------------|
| `client_id` | string | Unique client-assigned order ID (idempotency) |
| `flags` | array[string] | Order flags e.g. `["REDUCE_ONLY"]` |
| `stp` | string | Self-trade prevention mode |
| `trigger_price` | string | Required for stop/TP/SL orders |
| `vwap_price` | string | VWAP price protection for market orders |
| `recv_window` | integer | Max milliseconds for API to receive order (min 10ms) |
| `on_behalf_of_account` | string | Isolated margin account address |

**Response: HTTP 201 — OrderResp object:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Paradex-assigned order ID |
| `account` | string | StarkNet account address |
| `market` | string | Market symbol |
| `side` | string | BUY or SELL |
| `type` | string | Order type |
| `size` | string | Total order size |
| `price` | string | Limit price |
| `status` | string | `NEW`, `UNTRIGGERED`, `OPEN`, `CLOSED` |
| `instruction` | string | TIF instruction |
| `flags` | array | Applied flags |
| `stp` | string | STP mode |
| `client_id` | string | Client-assigned ID |
| `avg_fill_price` | string | Average fill price |
| `remaining_size` | string | Unfilled quantity |
| `cancel_reason` | string | Reason if canceled |
| `trigger_price` | string | Trigger price for conditional orders |
| `created_at` | integer | Creation timestamp (ms) |
| `last_updated_at` | integer | Last update timestamp (ms) |
| `received_at` | integer | API receipt timestamp (ms) |
| `published_at` | integer | Publication timestamp (ms) |
| `seq_no` | integer | Sequence number for deduplication |
| `timestamp` | integer | Signature timestamp (ms) |
| `request_info` | object | `{id, status, request_type, message}` |

**Order status flow:**
`NEW` (validation/risk checks) → `OPEN` (resting on book) or `CLOSED` (filled/rejected/canceled)
`UNTRIGGERED` → stop/TP/SL orders waiting for trigger price

---

### Create Batch of Orders

```
POST https://api.prod.paradex.trade/v1/orders/batch
Authorization: Bearer {JWT}
Content-Type: application/json
```

- **Batch size**: 1–10 orders per request
- **Rate limit efficiency**: 1 batch request = 1 rate limit unit regardless of order count
- If signature validation fails for any order → entire batch rejected
- Individual orders undergo independent risk checking; one failure does not block others

**Request body:** Array of order objects; each requires same fields as single order creation.

**Response: HTTP 201 — BatchResponse:**

```json
{
  "orders": [/* array of OrderResp objects for successful orders */],
  "errors": [/* array of {error, message} for failed orders */]
}
```

---

### Create Algo (TWAP) Order

```
POST https://api.prod.paradex.trade/v1/algo/orders
Authorization: Bearer {JWT}
Content-Type: application/json
```

**Required fields:**

| Field | Type | Description |
|-------|------|-------------|
| `algo_type` | string | `"TWAP"` (only supported type) |
| `market` | string | Market symbol |
| `side` | string | BUY or SELL |
| `type` | string | Must be `"MARKET"` |
| `size` | string | Total quantity to execute |
| `duration_seconds` | integer | Run duration 30–86400s in 30s increments |
| `signature` | string | STARK signature |
| `signature_timestamp` | integer | Signing timestamp (ms) |

**Optional fields:** `on_behalf_of_account`, `recv_window`

**Response: HTTP 201 — AlgoOrderResp:** algo ID, status, market, side, size, avg_fill_price, timestamps, remaining_size.

---

## Order Management

### Get Single Order (by Paradex ID)

```
GET https://api.prod.paradex.trade/v1/orders/{order_id}
Authorization: Bearer {JWT}
```

- Returns `OrderResp` object
- **Only returns orders in `OPEN` or `NEW` status**
- Path param: `order_id` (string, required)

---

### Get Single Order (by Client ID)

```
GET https://api.prod.paradex.trade/v1/orders/by_client_id/{client_id}
Authorization: Bearer {JWT}
```

- Returns `OrderResp` object
- **Only returns orders in `OPEN` status**
- Path param: `client_id` (string, required)

---

### List Orders (History + Open)

```
GET https://api.prod.paradex.trade/v1/orders-history
Authorization: Bearer {JWT}
```

**Query parameters:**

| Parameter | Type | Description |
|-----------|------|-------------|
| `market` | string | Filter by market (e.g. `BTC-USD-PERP`) |
| `side` | string | `BUY` or `SELL` |
| `status` | string | `OPEN`, `CLOSED`, `NEW` |
| `type` | string | Filter by order type |
| `client_id` | string | Filter by client order ID |
| `start_at` | integer | Start time (unix ms) |
| `end_at` | integer | End time (unix ms) |
| `page_size` | integer | Results per page (default: 100) |
| `cursor` | string | Pagination cursor |

**Response:**
```json
{
  "results": [/* OrderResp objects */],
  "next": "cursor_string_or_null",
  "prev": "cursor_string_or_null"
}
```

Note: To get **only open orders**, filter with `status=OPEN`. To get all history use `status=CLOSED`.

---

### Modify/Update Order

The Python SDK exposes `modify_order(order_id, order)` as a private method, indicating the endpoint exists. Based on SDK patterns and API structure, the endpoint is:

```
PUT https://api.prod.paradex.trade/v1/orders/{order_id}
Authorization: Bearer {JWT}
```

**Note**: The specific PUT modify endpoint URL was not directly confirmed in the official docs pages during this research. The Python SDK confirms this functionality exists. Requires re-signing of the modified order (same signing requirements as order creation).

---

### Cancel Single Order (by Paradex ID)

```
DELETE https://api.prod.paradex.trade/v1/orders/{order_id}
Authorization: Bearer {JWT}
```

- Path param: `order_id` (string, required)
- Response: **HTTP 204 No Content** (empty body) = queued for cancellation
- Confirmation arrives via WebSocket or by polling `GET /v1/orders/{id}`
- Orders in `CLOSED` state cannot be canceled
- Error response (400): `{error: "ORDER_ID_NOT_FOUND" | "ORDER_IS_CLOSED", message: "...", data: {}}`

---

### Cancel Batch of Orders

```
DELETE https://api.prod.paradex.trade/v1/orders/batch
Authorization: Bearer {JWT}
Content-Type: application/json
```

**Request body** (at least one field required):

| Field | Type | Description |
|-------|------|-------------|
| `order_ids` | array[string] | Paradex-assigned order IDs |
| `client_order_ids` | array[string] | Client-assigned order IDs |

**Response: HTTP 200:**
```json
{
  "results": [
    {
      "id": "string",
      "client_id": "string",
      "account": "string",
      "market": "string",
      "status": "QUEUED_FOR_CANCELLATION | ALREADY_CLOSED | NOT_FOUND"
    }
  ]
}
```

---

### Cancel All Orders

```
DELETE https://api.prod.paradex.trade/v1/orders
Authorization: Bearer {JWT}
```

**Query parameter:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market` | string | No | If specified, cancels only orders for this market; faster when market is provided |

**Response: HTTP 200** — empty JSON object `{}`

---

### Get Algo Orders History

```
GET https://api.prod.paradex.trade/v1/algo/orders-history
Authorization: Bearer {JWT}
```

Query params: `market`, `side`, `status` (OPEN/CLOSED/NEW), `type`, `client_id`, `start_at`, `end_at`, `page_size`, `cursor`.

---

### Cancel Algo Order

```
DELETE https://api.prod.paradex.trade/v1/algo/orders/{algo_order_id}
Authorization: Bearer {JWT}
```

---

## Position Management

### List Open Positions

```
GET https://api.prod.paradex.trade/v1/positions
Authorization: Bearer {JWT}
```

No query parameters required.

**Response: HTTP 200:**
```json
{
  "results": [
    {
      "id": "string",
      "account": "string",
      "market": "string",
      "side": "LONG | SHORT",
      "size": "string",
      "status": "OPEN | CLOSED",
      "average_entry_price": "string",
      "average_entry_price_usd": "string",
      "average_exit_price": "string",
      "liquidation_price": "string",
      "leverage": "string",
      "unrealized_pnl": "string",
      "unrealized_funding_pnl": "string",
      "realized_positional_pnl": "string",
      "realized_positional_funding_pnl": "string",
      "cost": "string",
      "cost_usd": "string",
      "cached_funding_index": "string",
      "last_fill_id": "string",
      "seq_no": "integer",
      "created_at": "integer",
      "last_updated_at": "integer",
      "closed_at": "integer"
    }
  ]
}
```

**Key fields:**

| Field | Description |
|-------|-------------|
| `side` | `LONG` or `SHORT` |
| `liquidation_price` | Price at which position is liquidated |
| `leverage` | Current effective leverage |
| `unrealized_pnl` | Unrealized P&L in quote asset |
| `unrealized_funding_pnl` | Unrealized funding P&L |

Note: There is **no explicit "close position" REST endpoint** — close by placing an opposing MARKET or LIMIT order with `REDUCE_ONLY` flag and matching size.

---

### Get Margin Configuration (Leverage)

```
GET https://api.prod.paradex.trade/v1/account/margin?market={market}
Authorization: Bearer {JWT}
```

**Required query param:** `market` (string)

**Response: HTTP 200 — GetAccountMarginConfigsResp:**

```json
{
  "account": "string",
  "margin_methodology": "cross_margin | portfolio_margin",
  "configs": [
    {
      "market": "string",
      "margin_type": "CROSS | ISOLATED",
      "leverage": "integer",
      "isolated_margin_leverage": "integer"
    }
  ]
}
```

Note: Leverage defaults to market maximum and can be set to a lower value by the user. There is no confirmed PUT endpoint for changing leverage in the public docs at time of research; SDK `modify_order` covers order-level leverage.

---

## Funding Rate

### List Funding Payments (Account History)

```
GET https://api.prod.paradex.trade/v1/funding/payments
Authorization: Bearer {JWT}
```

**Query parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `market` | string | Yes | Market symbol |
| `start_at` | integer | No | Start time (unix ms) |
| `end_at` | integer | No | End time (unix ms) |
| `cursor` | string | No | Pagination cursor |
| `page_size` | integer | No | Results per page (default: 100) |

**Response: HTTP 200:**
```json
{
  "results": [
    {
      "id": "string",
      "account": "string",
      "market": "string",
      "payment": "string",
      "index": "string",
      "created_at": "integer",
      "fill_id": "string"
    }
  ],
  "next": "cursor_or_null",
  "prev": "cursor_or_null"
}
```

---

## Advanced / Unique Paradex Features

### Privacy via Zero-Knowledge Proofs
All position and account data is private to the account holder via ZK proofs on Starknet. Unlike most DEXes, Paradex does not expose user position data on-chain in plaintext.

### STARK Cryptographic Order Signing
Every order must be cryptographically signed using the STARK elliptic curve private key before submission. The exchange verifies the signature server-side — no plain API keys are used for order authentication.

### Isolated Margin Accounts
The `on_behalf_of_account` parameter on order creation allows placing orders on behalf of isolated margin sub-accounts. Isolated margin is a distinct account model from the default cross-margin.

### Subkey Authorization
Trading can be done via subkeys (derived keypairs with scoped permissions — no withdrawal/transfer rights). The main Ethereum account's L2 Paradex private key can derive subkeys for safer API access.

### Market Health Endpoint
Poll `GET /v1/system/state` for exchange operational status:
- `ok` — fully operational
- `maintenance` — trading unavailable
- `cancel_only` — only order cancellations accepted

### VWAP Price Protection
Market orders can include `vwap_price` parameter to set a maximum slippage bound (VWAP-based price protection). Only for MARKET type orders.

### RPI (Retail Price Improvement)
`instruction: "RPI"` is a special TIF designation for retail price improvement fills. GET /fills and WebSocket fills channel include flags to indicate RPI fills.

### Block Trades

```
POST https://api.prod.paradex.trade/v1/block-trades
DELETE https://api.prod.paradex.trade/v1/block-trades/{block_trade_id}
```

Block trade functionality for bilateral off-book executions.

### TWAP Algorithmic Orders
Native TWAP execution via `/v1/algo/orders` — sub-orders placed every 30 seconds over a configurable duration. No manual order slicing needed.

---

## Market Symbol Format

Perpetual futures follow the pattern: `{BASE}-{QUOTE}-PERP`
Examples: `BTC-USD-PERP`, `ETH-USD-PERP`, `SOL-USD-PERP`

The `GET /v1/markets` endpoint returns all available markets and their static parameters (tick size, min order size, max order size, max open orders, etc.).

---

## Sources

- [Create order](https://docs.paradex.trade/api/prod/orders/new)
- [Create batch of orders](https://docs.paradex.trade/api/prod/orders/batch)
- [Cancel order](https://docs.paradex.trade/api/prod/orders/cancel)
- [Cancel batch of orders](https://docs.paradex.trade/api/prod/orders/cancel-batch)
- [Cancel all orders](https://docs.paradex.trade/api/prod/orders/cancel-all)
- [Get order](https://docs.paradex.trade/api/prod/orders/get)
- [Get order by client id](https://docs.paradex.trade/api/prod/orders/get-by-client-id)
- [Get orders (history)](https://docs.paradex.trade/api/prod/orders/get-orders)
- [List open positions](https://docs.paradex.trade/api/prod/account/get-positions)
- [Get account margin configuration](https://docs.paradex.trade/api/prod/account/get-account-margin)
- [Funding payments history](https://docs.paradex.trade/api/prod/account/get-funding)
- [Create algo order](https://docs.paradex.trade/api/prod/algos/create-order)
- [Get algo orders history](https://docs.paradex.trade/api/prod/algos/get-orders-history)
- [Cancel algo order](https://docs.paradex.trade/api/prod/algos/cancel-order)
- [Placing orders (overview)](https://docs.paradex.trade/trading/placing-orders)
- [Advanced API Best Practices](https://docs.paradex.trade/trading/api-best-practices)
- [Paradex Python SDK](https://tradeparadex.github.io/paradex-py/)
