# HTX (Huobi) Trading API — Spot & Futures

## Critical Design Note: Compound Order Types

HTX bakes **side into the order type name**. There is no separate `side` field for spot orders.

```
buy-limit       sell-limit
buy-market      sell-market
buy-ioc         sell-ioc
buy-limit-maker sell-limit-maker
buy-stop-limit  sell-stop-limit
buy-limit-fok   sell-limit-fok
buy-stop-limit-fok  sell-stop-limit-fok
```

This is fundamentally different from other exchanges (Binance, OKX) where `side=BUY` + `type=LIMIT` are separate fields.

---

## Base URLs

| Market         | REST Base URL                    |
|----------------|----------------------------------|
| Spot           | `https://api.huobi.pro`          |
| Spot (AWS)     | `https://api-aws.huobi.pro`      |
| Coin-M Futures | `https://api.hbdm.com`           |
| USDT-M Swap    | `https://api.hbdm.com`           |

---

## SPOT TRADING API

### POST /v1/order/orders/place — Place Order

**Auth required**: Yes (Trade permission)

**Request Parameters:**

| Field           | Type    | Required | Description |
|-----------------|---------|----------|-------------|
| account-id      | long    | YES      | From GET /v1/account/accounts |
| symbol          | string  | YES      | e.g. `btcusdt` (always lowercase) |
| type            | string  | YES      | Compound order type (see below) |
| amount          | decimal | YES      | Quantity of base currency (limit) or quote currency (buy-market) |
| price           | decimal | COND     | Required for all limit orders |
| stop-price      | decimal | COND     | Trigger price for stop-limit orders |
| operator        | string  | COND     | `gte` or `lte` — comparison for stop trigger |
| client-order-id | string  | NO       | User-defined, max 64 chars, for idempotency |
| source          | string  | NO       | Default `"api"` |
| self-match-prevent | string | NO    | Self-match prevention mode |

**All Order Types (`type` field):**

| Type                  | Side | Behavior |
|-----------------------|------|----------|
| `buy-limit`           | Buy  | Limit order, GTC |
| `sell-limit`          | Sell | Limit order, GTC |
| `buy-market`          | Buy  | Market order — `amount` = quote currency to spend |
| `sell-market`         | Sell | Market order — `amount` = base currency to sell |
| `buy-ioc`             | Buy  | Limit IOC — unfilled portion cancelled immediately |
| `sell-ioc`            | Sell | Limit IOC |
| `buy-limit-maker`     | Buy  | Post-only — rejected if would take liquidity |
| `sell-limit-maker`    | Sell | Post-only |
| `buy-stop-limit`      | Buy  | Triggers limit buy when price reaches `stop-price` |
| `sell-stop-limit`     | Sell | Triggers limit sell when price reaches `stop-price` |
| `buy-limit-fok`       | Buy  | Fill-or-Kill limit |
| `sell-limit-fok`      | Sell | Fill-or-Kill limit |
| `buy-stop-limit-fok`  | Buy  | Stop-limit with FOK execution |
| `sell-stop-limit-fok` | Sell | Stop-limit with FOK execution |

**Stop-Limit `operator` values:**

| operator | Meaning | Typical use |
|----------|---------|-------------|
| `gte`    | Trigger when market price >= stop-price | Stop-loss sell / breakout buy |
| `lte`    | Trigger when market price <= stop-price | Take-profit sell / support buy |

**Request JSON (limit):**
```json
{
  "account-id": "100009",
  "symbol": "btcusdt",
  "type": "buy-limit",
  "amount": "0.01",
  "price": "38000",
  "client-order-id": "my-order-001"
}
```

**Request JSON (market buy — spending quote):**
```json
{
  "account-id": "100009",
  "symbol": "btcusdt",
  "type": "buy-market",
  "amount": "100"
}
```

**Request JSON (stop-limit):**
```json
{
  "account-id": "100009",
  "symbol": "btcusdt",
  "type": "buy-stop-limit",
  "amount": "0.01",
  "price": "38000",
  "stop-price": "37500",
  "operator": "gte"
}
```

**Response JSON:**
```json
{
  "status": "ok",
  "data": "1234567890"
}
```

`data` is the order ID (string representation of long).

**Rate limit:** 50 times/2s per UID (25 req/sec)

---

### POST /v1/order/batch-orders — Batch Place Orders

**Auth required**: Yes (Trade permission)
**Max orders per request**: 10

**Request JSON:**
```json
[
  {
    "account-id": "100009",
    "symbol": "btcusdt",
    "type": "buy-limit",
    "amount": "0.01",
    "price": "38000",
    "client-order-id": "batch-001"
  },
  {
    "account-id": "100009",
    "symbol": "ethusdt",
    "type": "sell-limit",
    "amount": "0.1",
    "price": "2000",
    "client-order-id": "batch-002"
  }
]
```

**Response JSON:**
```json
{
  "status": "ok",
  "data": {
    "success": [
      {
        "index": 0,
        "order-id": 1234567890,
        "client-order-id": "batch-001"
      }
    ],
    "failed": [
      {
        "index": 1,
        "err-code": "invalid-amount",
        "err-msg": "...",
        "client-order-id": "batch-002"
      }
    ]
  }
}
```

**Rate limit:** 25 times/2s per UID (12.5 req/sec)

---

### POST /v1/order/orders/{order-id}/submitcancel — Cancel Order

**Auth required**: Yes (Trade permission)

URL parameter: `order-id` (long, the exchange-assigned order ID)

**Response JSON:**
```json
{
  "status": "ok",
  "data": "1234567890"
}
```

`data` is the cancelled order ID. **Rate limit:** No explicit limit (cancelled per announcement in 2023).

---

### POST /v1/order/orders/submitCancelClientOrder — Cancel by Client Order ID

**Request JSON:**
```json
{
  "client-order-id": "my-order-001"
}
```

Note: `client-order-id` is valid for **24 hours** after order completion.

---

### POST /v1/order/orders/batchCancelOpenOrders — Batch Cancel Open Orders

**Request JSON:**
```json
{
  "account-id": "100009",
  "symbol": "btcusdt",
  "side": "buy",
  "size": 100
}
```

| Field      | Type   | Required | Description |
|------------|--------|----------|-------------|
| account-id | string | YES      | Account identifier |
| symbol     | string | NO       | Filter by symbol |
| side       | string | NO       | `buy` or `sell` |
| size       | int    | NO       | Max orders to cancel, default 100, max 100 |

**Response JSON:**
```json
{
  "status": "ok",
  "data": {
    "success-count": 4,
    "failed-count": 0,
    "next-id": -1
  }
}
```

---

### GET /v1/order/orders/{order-id} — Get Order Detail

**Auth required**: Yes (Read permission)

**Response JSON:**
```json
{
  "status": "ok",
  "data": {
    "id": 1234567890,
    "symbol": "btcusdt",
    "account-id": 100009,
    "user-id": 10001,
    "client-order-id": "my-order-001",
    "amount": "0.01000000",
    "price": "38000.000000000000",
    "created-at": 1630000000000,
    "type": "buy-limit",
    "filled-amount": "0.01000000",
    "filled-cash-amount": "380.00",
    "filled-fees": "-0.000010",
    "finished-at": 1630000010000,
    "state": "filled",
    "canceled-at": 0,
    "source": "api",
    "stop-price": "",
    "operator": ""
  }
}
```

**Order State Values:**

| State            | Description |
|------------------|-------------|
| `submitted`      | Order accepted, not yet matched |
| `partial-filled` | Partially executed |
| `filled`         | Fully executed |
| `canceled`       | Fully cancelled (including partial-canceled) |
| `partial-canceled` | Partially filled then cancelled |

---

### GET /v1/order/openOrders — Get Open Orders

**Auth required**: Yes (Read permission)

**Query Parameters:**

| Field      | Type   | Required | Description |
|------------|--------|----------|-------------|
| account-id | string | YES      | Account identifier |
| symbol     | string | YES      | e.g. `btcusdt` |
| side       | string | NO       | `buy` or `sell` |
| size       | int    | NO       | Max results, default 10, max 500 |
| from       | string | NO       | Starting order ID for pagination |
| direct     | string | NO       | `prev` or `next` |

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 1234567890,
      "symbol": "btcusdt",
      "account-id": 100009,
      "amount": "0.01000000",
      "price": "38000.000000000000",
      "created-at": 1630000000000,
      "type": "buy-limit",
      "filled-amount": "0.00000000",
      "filled-cash-amount": "0.000000000000",
      "filled-fees": "0.000000000000",
      "state": "submitted",
      "source": "api",
      "client-order-id": ""
    }
  ]
}
```

---

### GET /v1/order/orders — Search Orders

**Auth required**: Yes (Read permission)
**Max time window**: 48 hours

**Query Parameters:**

| Field      | Type   | Required | Description |
|------------|--------|----------|-------------|
| symbol     | string | YES      | e.g. `btcusdt` |
| start-time | long   | NO       | UTC timestamp in ms |
| end-time   | long   | NO       | UTC timestamp in ms |
| states     | string | NO       | Comma-separated: `submitted,partial-filled,filled,canceled` |
| types      | string | NO       | Comma-separated order types |
| from       | string | NO       | Order ID for pagination |
| direct     | string | NO       | `prev` or `next` |
| size       | int    | NO       | Default 10, max 100 |

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 1234567890,
      "symbol": "btcusdt",
      "account-id": 100009,
      "user-id": 10001,
      "client-order-id": "",
      "amount": "0.01000000",
      "price": "38000.000000000000",
      "created-at": 1630000000000,
      "type": "buy-limit",
      "filled-amount": "0.01000000",
      "filled-cash-amount": "380.00",
      "filled-fees": "-0.000010",
      "finished-at": 1630000010000,
      "canceled-at": 0,
      "state": "filled",
      "source": "api"
    }
  ]
}
```

---

### GET /v1/order/history — Order History (Recommended)

Preferred over `/v1/order/orders` for recent 48 hours. Better service level, same parameters.

---

### GET /v1/order/matchresults — Order Match/Trade Results

**Auth required**: Yes (Read permission)

**Query Parameters:** Same as `/v1/order/orders` (symbol, start-time, end-time, from, direct, size)

**Response JSON:**
```json
{
  "status": "ok",
  "data": [
    {
      "id": 9876543210,
      "order-id": 1234567890,
      "trade-id": 555555,
      "symbol": "btcusdt",
      "created-at": 1630000010000,
      "type": "buy-limit",
      "price": "38000.000000000000",
      "filled-amount": "0.01000000",
      "filled-fees": "-0.000010",
      "fee-currency": "btc",
      "fee-deduct-state": "done",
      "role": "taker",
      "source": "api"
    }
  ]
}
```

**Key fields:**
- `trade-id`: Unique trade execution ID (added 2019-11)
- `role`: `"taker"` or `"maker"` (added 2020-07)
- `fee-deduct-state`: Fee deduction status (added 2020-12)
- `fee-currency`: Currency in which fee was charged

---

## ALGO / CONDITIONAL ORDERS (Spot TP/SL)

HTX does not support TP/SL inline with spot order placement. Instead, use the separate algo-orders API.

### POST /v2/algo-orders — Place Conditional Order

**Auth required**: Yes (Trade permission)
**Rate limit**: 20 times/2s

**Request Parameters:**

| Field           | Type    | Required | Description |
|-----------------|---------|----------|-------------|
| accountId       | long    | YES      | Account ID |
| symbol          | string  | YES      | e.g. `btcusdt` |
| orderPrice      | decimal | COND     | Limit price (required for limit type) |
| orderSide       | string  | YES      | `buy` or `sell` |
| orderType       | string  | YES      | `limit` or `market` |
| orderSize       | decimal | COND     | Order size (for non-trailing) |
| stopPrice       | decimal | YES      | Trigger price |
| trailingRate    | decimal | COND     | Trailing rate in % (for trailing-stop-limit) |
| timeInForce     | string  | NO       | `gtc`, `boc` (post-only), `ioc`, `fok` |
| clientOrderId   | string  | NO       | User-defined, unique within 24h |

**timeInForce values:**
- `gtc` — Good Till Cancel (default)
- `boc` — Book Or Cancel (Post-Only / Maker-only)
- `ioc` — Immediate Or Cancel
- `fok` — Fill Or Kill

**Response JSON:**
```json
{
  "code": 200,
  "data": {
    "clientOrderId": "my-algo-001"
  }
}
```

Note: v2 endpoints return `code` (int) instead of `status` (string).

### POST /v2/algo-orders/cancel — Cancel Conditional Orders

### GET /v2/algo-orders/opening — Query Open Conditional Orders

### GET /v2/algo-orders/history — Query Conditional Order History

### GET /v2/algo-orders/specific — Query Specific Conditional Order

### POST /v2/algo-orders/cancel-all-after — Dead Man's Switch

**Request JSON:**
```json
{
  "timeout": 60
}
```

`timeout` in seconds. Set to `0` to cancel the timer.

---

## MODIFY ORDER

HTX spot does **not** support modifying an existing order. To change an order, cancel it and re-place.

---

## FUTURES TRADING API — Coin-Margined (DM)

**Base URL**: `https://api.hbdm.com`

### Key Differences from Spot

| Aspect | Spot | Futures |
|--------|------|---------|
| Side field | Baked into `type` | Separate `direction` field |
| Offset | N/A | `open` or `close` |
| Volume | Decimal amount | Integer contracts |
| Symbol | `btcusdt` | `BTC` (just coin) or `BTC-USD` (contract code) |
| TP/SL | Separate algo-orders API | Inline with order placement |

### POST /api/v1/contract_order — Place Futures Order

**Request Parameters:**

| Field              | Type    | Required | Description |
|--------------------|---------|----------|-------------|
| symbol             | string  | COND     | e.g. `BTC` (use with contract_type) |
| contract_code      | string  | COND     | e.g. `BTC-USD` (use instead of symbol+contract_type) |
| contract_type      | string  | COND     | `this_week`, `next_week`, `quarter`, `next_quarter` |
| client_order_id    | long    | NO       | Client order ID (numeric, unlike spot) |
| price              | decimal | COND     | Required for limit orders |
| volume             | int     | YES      | Number of contracts |
| direction          | string  | YES      | `buy` or `sell` |
| offset             | string  | YES      | `open`, `close`, or `both` (reduce-only mode) |
| lever_rate         | int     | YES      | Leverage multiplier |
| order_price_type   | string  | YES      | Order type (see below) |
| tp_trigger_price   | decimal | NO       | Take-profit trigger price |
| tp_order_price     | decimal | NO       | Take-profit order price |
| tp_order_price_type | string | NO      | TP order type: `limit` or `optimal_5/10/20` |
| sl_trigger_price   | decimal | NO       | Stop-loss trigger price |
| sl_order_price     | decimal | NO       | Stop-loss order price |
| sl_order_price_type | string | NO      | SL order type: `limit` or `optimal_5/10/20` |

**Futures `order_price_type` values:**

| Value              | Description |
|--------------------|-------------|
| `limit`            | Limit order |
| `opponent`         | Market order (best opposite price) |
| `post_only`        | Maker-only |
| `ioc`              | Immediate or Cancel |
| `fok`              | Fill or Kill |
| `optimal_5`        | Best 5 BBO prices |
| `optimal_10`       | Best 10 BBO prices |
| `optimal_20`       | Best 20 BBO prices |
| `opponent_ioc`     | IOC at opponent price |
| `optimal_5_ioc`    | IOC at optimal-5 |
| `optimal_10_ioc`   | IOC at optimal-10 |
| `optimal_20_ioc`   | IOC at optimal-20 |
| `opponent_fok`     | FOK at opponent price |
| `optimal_5_fok`    | FOK at optimal-5 |
| `optimal_10_fok`   | FOK at optimal-10 |
| `optimal_20_fok`   | FOK at optimal-20 |
| `lightning`        | Flash close |
| `lightning_ioc`    | Flash close IOC |
| `lightning_fok`    | Flash close FOK |

**Request JSON:**
```json
{
  "contract_code": "BTC-USD",
  "direction": "buy",
  "offset": "open",
  "lever_rate": 10,
  "volume": 1,
  "price": "38000",
  "order_price_type": "limit",
  "tp_trigger_price": "40000",
  "tp_order_price": "40000",
  "sl_trigger_price": "36000",
  "sl_order_price": "36000"
}
```

**Response JSON:**
```json
{
  "status": "ok",
  "data": {
    "order_id": 987654321,
    "order_id_str": "987654321",
    "client_order_id": null
  },
  "ts": 1630000000000
}
```

Note: Both `order_id` (long) and `order_id_str` (string) are returned to handle JavaScript 64-bit integer precision issues.

---

### POST /api/v1/contract_batchorder — Batch Futures Orders

Max 10 orders per request.

---

### POST /api/v1/contract_cancel — Cancel Futures Order

```json
{
  "symbol": "BTC",
  "order_id": "987654321",
  "client_order_id": null
}
```

Cancel multiple: comma-separated order_ids (max 10).

---

### POST /api/v1/contract_cancelall — Cancel All Futures Orders

```json
{
  "symbol": "BTC",
  "contract_code": "BTC-USD",
  "direction": "buy",
  "offset": "open"
}
```

---

### POST /api/v1/contract_order_info — Get Futures Order Info

```json
{
  "symbol": "BTC",
  "order_id": "987654321"
}
```

---

### POST /api/v1/contract_openorders — Query Open Futures Orders

**Request JSON:**
```json
{
  "symbol": "BTC",
  "contract_code": "BTC-USD",
  "sort_by": "created_at",
  "trade_type": 0,
  "page_index": 1,
  "page_size": 20
}
```

`trade_type`: 0=all, 1=buy open, 2=sell open, 3=buy close, 4=sell close

---

### POST /api/v1/contract_hisorders — Futures Order History

**Order status values:** 1=submitted, 2=accepted, 5=partial-filled, 6=filled, 7=cancelled

---

## FUTURES TRADING API — USDT-Margined Swaps (Linear)

**Base URL**: `https://api.hbdm.com`
**API path prefix**: `/linear-swap-api/v1/`

### Isolated Margin Endpoints

| Action | Endpoint |
|--------|----------|
| Place order | `POST /linear-swap-api/v1/swap_order` |
| Batch orders | `POST /linear-swap-api/v1/swap_batchorder` |
| Cancel order | `POST /linear-swap-api/v1/swap_cancel` |
| Cancel all | `POST /linear-swap-api/v1/swap_cancelall` |
| Open orders | `POST /linear-swap-api/v1/swap_openorders` |
| Order info | `POST /linear-swap-api/v1/swap_order_info` |
| Order history | `POST /linear-swap-api/v1/swap_hisorders` |

### Cross Margin Endpoints (suffix `_cross_`)

| Action | Endpoint |
|--------|----------|
| Place order | `POST /linear-swap-api/v1/swap_cross_order` |
| Batch orders | `POST /linear-swap-api/v1/swap_cross_batchorder` |
| Cancel order | `POST /linear-swap-api/v1/swap_cross_cancel` |
| Cancel all | `POST /linear-swap-api/v1/swap_cross_cancelall` |
| Open orders | `POST /linear-swap-api/v1/swap_cross_openorders` |

### USDT-M Order Parameters (same as Coin-M plus)

- `reduce_only`: `0` or `1` — for one-way position mode
- `contract_code`: e.g. `BTC-USDT` (perpetual) or `BTC-USDT-211231` (futures)

### Trigger Orders (Futures TP/SL on existing positions)

| Action | Endpoint |
|--------|----------|
| Place trigger order | `POST /linear-swap-api/v1/swap_trigger_order` |
| Cancel trigger order | `POST /linear-swap-api/v1/swap_trigger_cancel` |
| Query trigger orders | `POST /linear-swap-api/v1/swap_trigger_openorders` |

---

## Futures Rate Limits

| Interface Type | Limit |
|----------------|-------|
| Private REST (per UID) | 72 times/3s (~24 req/sec) |
| Public REST | 60 times/3s (~20 req/sec) |
| Master-Sub transfers | 10 per minute |

---

## Sources

- [HTX Spot API Reference](https://huobiapi.github.io/docs/spot/v1/en/)
- [HTX Coin-M Futures API Reference](https://huobiapi.github.io/docs/dm/v1/en/)
- [HTX USDT-M Swap API Reference](https://huobiapi.github.io/docs/usdt_swap/v1/en/)
- [HTX Order Rate Limit Adjustment Announcement](https://www.htx.com/support/24873931166922)
