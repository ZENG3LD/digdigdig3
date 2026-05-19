# Gate.io APIv4 — Trading API Reference

**Base URL (Live):** `https://api.gateio.ws/api/v4`
**Base URL (Futures alt):** `https://fx-api.gateio.ws/api/v4`
**Testnet (Futures only):** `https://fx-api-testnet.gateio.ws/api/v4`
**Testnet (all):** `https://api-testnet.gateapi.io/api/v4`

> **NOT SUPPORTED for spot:** No spot testnet. Futures testnet via `fx-api-testnet.gateio.ws`.

---

## 1. Order Types

### Spot (POST /spot/orders) — `type` field

| Value | Meaning |
|-------|---------|
| `limit` | Limit order (default). Requires `price`. |
| `market` | Market order. `price` field ignored. |

### Time-in-Force — `time_in_force` field

| Value | Spot | Futures | Meaning |
|-------|------|---------|---------|
| `gtc` | Yes | Yes | Good Till Cancelled (default) |
| `ioc` | Yes | Yes | Immediate Or Cancel (taker only; unfilled portion cancelled) |
| `poc` | Yes | Yes | Pending Or Cancelled (Post-Only; cancelled if would match) |
| `fok` | Yes | Yes | Fill Or Kill (all or nothing) |

> For market orders: only `ioc` and `fok` supported.
> `poc` is Gate.io's Post-Only equivalent.

---

## 2. Spot Order Management

### 2.1 Create Order

```
POST /spot/orders
```

**Request Body (JSON):**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `currency_pair` | string | Yes | e.g. `"BTC_USDT"` |
| `side` | string | Yes | `"buy"` or `"sell"` |
| `amount` | string | Yes | Order quantity |
| `price` | string | Cond. | Required for `limit` orders |
| `type` | string | No | `"limit"` (default) or `"market"` |
| `time_in_force` | string | No | `"gtc"` (default), `"ioc"`, `"poc"`, `"fok"` |
| `account` | string | No | `"spot"` (default), `"margin"`, `"unified"` |
| `iceberg` | string | No | Visible amount for iceberg orders; `"0"` = normal |
| `auto_borrow` | bool | No | Auto-borrow for margin if balance insufficient |
| `auto_repay` | bool | No | Auto-repay margin loan after fill |
| `text` | string | No | Custom order label; must start with `"t-"` |
| `stp_act` | string | No | Self-trade prevention: `"cn"` (cancel newest), `"co"` (cancel oldest), `"cb"` (cancel both) |
| `action_mode` | string | No | `"ACK"` (async, returns id+status only), `"RESULT"` (no clearing info), `"FULL"` (default, complete) |

**Example Request:**
```json
{
  "currency_pair": "BTC_USDT",
  "side": "buy",
  "type": "limit",
  "amount": "0.001",
  "price": "40000",
  "time_in_force": "gtc",
  "account": "spot",
  "text": "t-my-order-001"
}
```

**Response (full Order object):**

| Field | Type | Description |
|-------|------|-------------|
| `id` | string | Order ID |
| `text` | string | Custom label |
| `amend_text` | string | Custom label set during amendment |
| `create_time` | string | Unix timestamp (seconds) |
| `update_time` | string | Unix timestamp (seconds) |
| `create_time_ms` | int64 | Unix timestamp (ms) |
| `update_time_ms` | int64 | Unix timestamp (ms) |
| `status` | string | `"open"`, `"closed"`, `"cancelled"` |
| `currency_pair` | string | Trading pair |
| `type` | string | `"limit"` or `"market"` |
| `account` | string | Account type |
| `side` | string | `"buy"` or `"sell"` |
| `amount` | string | Original order quantity |
| `price` | string | Order price |
| `time_in_force` | string | TIF strategy |
| `iceberg` | string | Iceberg visible amount |
| `auto_borrow` | bool | Auto-borrow flag |
| `auto_repay` | bool | Auto-repay flag |
| `left` | string | Remaining unfilled quantity |
| `filled_amount` | string | Filled quantity |
| `fill_price` | string | Total fill cost (cumulative) |
| `filled_total` | string | Total fill value in quote currency |
| `avg_deal_price` | string | Average fill price |
| `fee` | string | Fee charged |
| `fee_currency` | string | Fee currency |
| `point_fee` | string | POINT token fee used |
| `gt_fee` | string | GT token fee used |
| `gt_maker_fee` | string | GT maker fee rate |
| `gt_taker_fee` | string | GT taker fee rate |
| `gt_discount` | bool | GT fee discount applied |
| `rebated_fee` | string | Rebate fee amount |
| `rebated_fee_currency` | string | Rebate currency |
| `stp_id` | int32 | STP group ID |
| `stp_act` | string | STP action taken |
| `finish_as` | string | How the order ended: `"open"`, `"filled"`, `"cancelled"`, `"ioc"`, `"auto_deleveraged"`, `"liquidated"`, `"reduce_only"`, `"poc"` |
| `action_mode` | string | Action mode used |

---

### 2.2 Amend Single Order (Spot)

```
PATCH /spot/orders/{order_id}
```

> **NOTE:** Spot uses PATCH for amendment. Futures uses PATCH as well (see Section 4.2).

**Query Params:**
- `currency_pair` (required): e.g. `"BTC_USDT"`
- `account` (optional): account type

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `amount` | string | New amount (only reducing quantity preserves matching priority) |
| `price` | string | New price (changes priority to end of price level) |
| `amend_text` | string | Custom text for this amendment |
| `action_mode` | string | `"ACK"`, `"RESULT"`, `"FULL"` |

> Modifying price or increasing quantity moves order to end of queue at that price level.

---

### 2.3 Cancel Single Order (Spot)

```
DELETE /spot/orders/{order_id}
```

**Query Params:** `currency_pair` (required), `account` (optional)

---

### 2.4 Cancel All Open Orders (Spot)

```
DELETE /spot/orders
```

**Query Params:**
- `currency_pair` (required): Trading pair to cancel
- `side` (optional): `"buy"` or `"sell"`
- `account` (optional)

---

### 2.5 Get Single Order (Spot)

```
GET /spot/orders/{order_id}
```

**Query Params:** `currency_pair` (required), `account` (optional)

---

### 2.6 List Open Orders (Spot)

```
GET /spot/orders
```

**Query Params:**
- `currency_pair` (required for non-unified)
- `status` (required): `"open"` or `"finished"`
- `page` (optional): page number (default 1)
- `limit` (optional): max 1000 (default 100)
- `account` (optional)
- `from` / `to` (optional): Unix timestamp range (for finished orders)

---

### 2.7 Fills / Trade History (Spot)

```
GET /spot/my_trades
```

**Query Params:**
- `currency_pair` (optional)
- `limit` (optional, default 100, max 1000)
- `page` (optional)
- `order_id` (optional): filter by order
- `account` (optional)
- `from` / `to` (optional): Unix timestamp range

---

### 2.8 Batch Create Orders (Spot)

```
POST /spot/batch_orders
```

Request body: JSON array of order objects (same fields as POST /spot/orders).
Returns array of order results.

> Maximum 10 orders per batch request.

---

### 2.9 Batch Cancel Orders (Spot)

```
POST /spot/cancel_batch_orders
```

Request body: JSON array of objects with `id` and `currency_pair`.

```json
[
  { "id": "123456789", "currency_pair": "BTC_USDT" },
  { "id": "987654321", "currency_pair": "ETH_USDT" }
]
```

---

### 2.10 Batch Amend Orders (Spot)

```
POST /spot/amend_batch_orders
```

Request body: array of amendment objects. Each must include `currency_pair` and `id`.
Modifies orders in spot, unified, and isolated margin accounts.

---

## 3. Spot Price-Triggered Orders (TP/SL for Spot)

```
POST /spot/price_orders
GET  /spot/price_orders
DELETE /spot/price_orders
GET  /spot/price_orders/{order_id}
DELETE /spot/price_orders/{order_id}
```

### SpotPriceTriggeredOrder Structure

```json
{
  "trigger": {
    "price": "45000",
    "rule": ">=",
    "expiration": 86400
  },
  "put": {
    "type": "limit",
    "side": "sell",
    "price": "45000",
    "amount": "0.001",
    "account": "normal",
    "time_in_force": "gtc",
    "auto_borrow": false,
    "auto_repay": false,
    "text": "t-api"
  },
  "market": "BTC_USDT"
}
```

**Trigger fields:**

| Field | Type | Description |
|-------|------|-------------|
| `price` | string | Trigger price |
| `rule` | string | `">="` (trigger when market price >= trigger price) or `"<="` |
| `expiration` | int32 | Max wait time in seconds; order cancelled if expired |

**Put (order to execute) fields:**

| Field | Type | Description |
|-------|------|-------------|
| `type` | string | `"limit"` (default) or `"market"` |
| `side` | string | `"buy"` or `"sell"` |
| `price` | string | Order price |
| `amount` | string | Order quantity |
| `account` | string | `"normal"`, `"margin"`, or `"unified"` |
| `time_in_force` | string | `"gtc"` or `"ioc"` |
| `auto_borrow` | bool | Auto-borrow for margin |
| `auto_repay` | bool | Auto-repay flag |
| `text` | string | Order source: `"web"`, `"api"`, or `"app"` |

**Response status values:** `"open"`, `"cancelled"`, `"finish"`, `"failed"`, `"expired"`

---

## 4. Futures Order Management

**Path pattern:** `/futures/{settle}/...`

**`{settle}` values:**
- `usdt` — USDT-margined perpetual swaps
- `btc` — BTC-margined (coin-margined) perpetual swaps

### 4.1 Create Futures Order

```
POST /futures/{settle}/orders
```

**Request Body:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `contract` | string | Yes | e.g. `"BTC_USDT"` |
| `size` | int64 | Yes | Contract quantity (positive = long, negative = short, 0 = close all) |
| `price` | string | Yes | Order price; `"0"` for market order |
| `tif` | string | No | `"gtc"` (default), `"ioc"`, `"poc"`, `"fok"` |
| `text` | string | No | Source tag: `"web"`, `"api"`, `"app"` |
| `iceberg` | int64 | No | Iceberg visible size; 0 = normal |
| `close` | bool | No | `true` to close all positions in single mode |
| `reduce_only` | bool | No | `true` = reduce-only; auto-reduce position |
| `auto_size` | string | No | For dual-mode full close: `"close_long"` or `"close_short"` |
| `stp_id` | int32 | No | STP group ID |
| `stp_act` | string | No | `"cn"`, `"co"`, `"cb"` |
| `amend_text` | string | No | Custom text |

**Example Request:**
```json
{
  "contract": "BTC_USDT",
  "size": 10,
  "price": "40000",
  "tif": "gtc"
}
```

**Response (FuturesOrder object):**

| Field | Type | Description |
|-------|------|-------------|
| `id` | int64 | Order ID |
| `user` | int32 | User ID |
| `create_time` | float64 | Creation timestamp (Unix seconds) |
| `update_time` | float64 | Last update timestamp |
| `finish_time` | float64 | Finish timestamp (absent if still open) |
| `finish_as` | string | Completion reason (see below) |
| `status` | string | `"open"`, `"finished"` |
| `contract` | string | Contract name |
| `size` | int64 | Order contract size |
| `iceberg` | int64 | Iceberg visible size |
| `price` | string | Order price |
| `close` | bool | Position close flag |
| `is_close` | bool | Whether order is to close position |
| `reduce_only` | bool | Reduce-only flag (set in request) |
| `is_reduce_only` | bool | Server-determined reduce-only |
| `is_liq` | bool | Liquidation order |
| `tif` | string | Time in force |
| `left` | int64 | Remaining unfilled contracts |
| `fill_price` | string | Cumulative fill price |
| `text` | string | Order source tag |
| `tkfr` | string | Taker fee rate |
| `mkfr` | string | Maker fee rate |
| `refu` | int32 | Referral user ID |
| `auto_size` | string | Auto-size used |
| `stp_id` | int32 | STP group ID |
| `stp_act` | string | STP action |
| `amend_text` | string | Amendment text |

**`finish_as` values:** `"filled"`, `"cancelled"`, `"liquidated"`, `"ioc"`, `"auto_deleveraged"`, `"reduce_only"`, `"poc"`, `"stp"`

---

### 4.2 Amend Futures Order

```
PATCH /futures/{settle}/orders/{order_id}
```

> **NOTE:** Futures amendment also uses PATCH (same as spot). Gate.io docs historically showed PUT for futures but current APIv4 uses PATCH for both.

**Request Body:**

| Field | Type | Description |
|-------|------|-------------|
| `size` | int64 | New size |
| `price` | string | New price |
| `amend_text` | string | Amendment annotation |

---

### 4.3 Cancel Single Futures Order

```
DELETE /futures/{settle}/orders/{order_id}
```

---

### 4.4 Cancel All Open Futures Orders

```
DELETE /futures/{settle}/orders
```

**Query Params:**
- `contract` (required): Contract name
- `side` (optional): `"ask"` or `"bid"`

---

### 4.5 Get Single Futures Order

```
GET /futures/{settle}/orders/{order_id}
```

---

### 4.6 List Futures Orders

```
GET /futures/{settle}/orders
```

**Query Params:**
- `contract` (required for status=open)
- `status` (required): `"open"` or `"finished"`
- `limit` (optional, default 100, max 1000)
- `offset` (optional): offset for pagination
- `last_id` (optional): cursor pagination
- `from` / `to` (optional): Unix timestamp range

---

### 4.7 Batch Create Futures Orders

```
POST /futures/{settle}/batch_orders
```

Request body: JSON array of futures order objects.

---

### 4.8 Batch Cancel Futures Orders

```
POST /futures/{settle}/batch_cancel_orders
```

Request body: array of order IDs with contract name.

---

### 4.9 Futures Fills / Trade History

```
GET /futures/{settle}/my_trades
```

**Query Params:**
- `contract` (optional): filter by contract
- `order` (optional): filter by order ID
- `limit` (optional, default 100, max 1000)
- `offset` (optional)
- `last_id` (optional)
- `from` / `to` (optional)

---

## 5. Futures Price-Triggered Orders (TP/SL for Futures)

```
POST   /futures/{settle}/price_orders
GET    /futures/{settle}/price_orders
DELETE /futures/{settle}/price_orders
GET    /futures/{settle}/price_orders/{order_id}
DELETE /futures/{settle}/price_orders/{order_id}
PUT    /futures/{settle}/price_orders/amend/{order_id}
```

### FuturesPriceTriggeredOrder Structure

```json
{
  "initial": {
    "contract": "BTC_USDT",
    "size": 10,
    "price": "45000",
    "tif": "gtc",
    "close": false,
    "reduce_only": true,
    "text": "api",
    "auto_size": ""
  },
  "trigger": {
    "strategy_type": 0,
    "price_type": 0,
    "price": "45000",
    "rule": 1,
    "expiration": 86400
  },
  "order_type": "plan"
}
```

**Trigger fields:**

| Field | Type | Description |
|-------|------|-------------|
| `strategy_type` | int32 | `0` = price trigger; `1` = price spread trigger |
| `price_type` | int32 | `0` = last trade price; `1` = mark price; `2` = index price |
| `price` | string | Trigger price or spread value |
| `rule` | int32 | `1` = trigger when price >= trigger price; `2` = trigger when price <= trigger price |
| `expiration` | int32 | Max wait seconds before cancellation |

**Initial (order to execute) fields:**

| Field | Type | Description |
|-------|------|-------------|
| `contract` | string | Contract name |
| `size` | int64 | Order size (0 = close all) |
| `price` | string | Order price; `"0"` for market |
| `close` | bool | Close all in single mode |
| `tif` | string | Time in force (`"gtc"` or `"ioc"`) |
| `text` | string | Source tag |
| `reduce_only` | bool | Reduce-only flag |
| `auto_size` | string | `"close_long"` or `"close_short"` for dual mode |

**Response extra fields:**

| Field | Type | Description |
|-------|------|-------------|
| `id` | int64 | Triggered order ID |
| `user` | int32 | User ID |
| `create_time` | float64 | Creation timestamp |
| `finish_time` | float64 | Completion timestamp |
| `trade_id` | int64 | Resulting trade ID |
| `status` | string | `"open"`, `"cancelled"`, `"finish"`, `"failed"`, `"expired"` |
| `finish_as` | string | Completion reason |
| `reason` | string | Cancellation reason |
| `order_type` | string | `"plan"`, `"profit"`, `"loss"`, `"manual"` |
| `me_order_id` | int64 | Matching engine order ID |

---

## 6. Order History Endpoints Summary

| Endpoint | Description |
|----------|-------------|
| `GET /spot/orders?status=finished` | Spot order history |
| `GET /spot/my_trades` | Spot fills history |
| `GET /futures/{settle}/orders?status=finished` | Futures order history |
| `GET /futures/{settle}/my_trades` | Futures fills history |

---

## 7. Key Behavioral Notes

- **Spot vs Futures amendment:** Both use `PATCH`. Rate limit announcement mentioned `PUT` for futures historically, but current implementation uses `PATCH`.
- **Futures size type:** `int64` (contract count), not decimal string. `size > 0` = long, `size < 0` = short.
- **Spot amount type:** `string` decimal (e.g. `"0.001"` BTC).
- **Market orders:** Use `price: "0"` (futures) or omit price (spot with `type: "market"`).
- **Iceberg orders:** Deprecated/adjusted per Gate.io announcement 2022. Set iceberg amount <= total amount.
- **action_mode (spot only):** `"ACK"` returns minimal data for high-throughput strategies; `"FULL"` is default.
- **STP:** Self-trade prevention requires creating an STP group via `POST /account/stp_groups` first.

---

## Sources

- [Gate API v4 Official Docs](https://www.gate.com/docs/developers/apiv4/en/)
- [gateapi-go model_order.go](https://github.com/gateio/gateapi-go/blob/master/model_order.go)
- [gateapi-go model_futures_order.go](https://github.com/gateio/gateapi-go/blob/master/model_futures_order.go)
- [gateapi-go model_spot_price_triggered_order.go](https://github.com/gateio/gateapi-go/blob/master/model_spot_price_triggered_order.go)
- [gateapi-go model_futures_price_triggered_order.go](https://github.com/gateio/gateapi-go/blob/master/model_futures_price_triggered_order.go)
- [Gate.io Rate Limit Announcement Jan 2024](https://www.gate.com/announcements/article/33995)
