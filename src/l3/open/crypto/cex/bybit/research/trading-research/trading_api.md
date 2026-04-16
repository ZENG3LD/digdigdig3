# Bybit V5 Trading API — Order Management

Source: https://bybit-exchange.github.io/docs/v5/order/create-order

---

## 1. ORDER TYPES

Bybit V5 uses a **unified API** — all order endpoints take a `category` parameter that selects the product type. There is no separate "futures" vs "spot" URL.

### `category` Values

| Value | Description |
|-------|-------------|
| `spot` | Spot trading |
| `linear` | USDT/USDC perpetuals and futures |
| `inverse` | Inverse (coin-margined) perpetuals and futures |
| `option` | USDC options |

### `orderType` Values

| Value | Description | Notes |
|-------|-------------|-------|
| `Market` | Execute at best available price | Always uses IOC time-in-force internally |
| `Limit` | Execute at specified price or better | Requires `price` param |

NOTE: There is no native `StopOrder`, `StopLimit`, `TakeProfit`, or `TrailingStop` orderType at the top level. These are achieved via `triggerPrice` (conditional) or `stopOrderType` semantics expressed through separate position TP/SL endpoints. The `stopOrderType` field in responses indicates the sub-type of a conditional order.

### `stopOrderType` Values (in order responses — not a request param for create)

| Value | Description |
|-------|-------------|
| `TakeProfit` | Take profit order |
| `StopLoss` | Stop loss order |
| `TrailingStop` | Trailing stop order |
| `Stop` | Conditional stop order |
| `PartialTakeProfit` | Partial TP |
| `PartialStopLoss` | Partial SL |
| `tpslOrder` | Spot TP/SL order |
| `OcoOrder` | OCO order (spot) |
| `BidirectionalTpslOrder` | Bidirectional TP/SL |
| `MmRateClose` | Maintenance margin close |

### `timeInForce` Values

| Value | Description | Notes |
|-------|-------------|-------|
| `GTC` | Good Till Cancel | Default if not specified |
| `IOC` | Immediate Or Cancel | Partial fills allowed; remainder cancelled |
| `FOK` | Fill Or Kill | Must fill entirely or cancel |
| `PostOnly` | Post Only | Cancelled if would fill immediately |
| `RPI` | Retail Price Improvement | Special market maker type |

Market orders always use `IOC` internally regardless of the `timeInForce` param.

---

### Order Parameters — `POST /v5/order/create`

#### Core Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `category` | string | YES | `spot`, `linear`, `inverse`, `option` |
| `symbol` | string | YES | Trading pair (e.g. `BTCUSDT`) |
| `side` | string | YES | `Buy` or `Sell` |
| `orderType` | string | YES | `Market` or `Limit` |
| `qty` | string | YES | Order quantity |
| `price` | string | NO | Limit price. Required for Limit orders |
| `timeInForce` | string | NO | Default: `GTC` |
| `orderLinkId` | string | NO | Client order ID (user-defined, unique per order) |
| `orderFilter` | string | NO | Spot only: `Order` (default), `tpslOrder`, `StopOrder` |

#### Conditional Order Parameters

| Parameter | Type | Required | Description | Categories |
|-----------|------|----------|-------------|------------|
| `triggerPrice` | string | NO | Setting this makes the order conditional | all |
| `triggerBy` | string | NO | `LastPrice`, `IndexPrice`, `MarkPrice` | `linear`, `inverse` |
| `triggerDirection` | integer | NO | `1` = triggered when price rises to triggerPrice; `2` = triggered when falls | `linear`, `inverse` |

#### Position Management Parameters

| Parameter | Type | Required | Description | Categories |
|-----------|------|----------|-------------|------------|
| `reduceOnly` | boolean | NO | If true, can only reduce position size. MUST be `true` when closing/reducing | all |
| `closeOnTrigger` | boolean | NO | Closing/reduce-only for triggered orders | `linear`, `inverse` |
| `isLeverage` | integer | NO | `0` = spot trading (default), `1` = spot margin trading | `spot` |
| `positionIdx` | integer | NO | `0` = one-way mode, `1` = buy hedge side, `2` = sell hedge side | `linear`, `inverse` |

#### TP/SL at Order Creation

| Parameter | Type | Description | Categories |
|-----------|------|-------------|------------|
| `takeProfit` | string | TP price. Set to `"0"` to cancel existing TP | `linear`, `inverse`, `spot` (limit only) |
| `stopLoss` | string | SL price. Set to `"0"` to cancel existing SL | `linear`, `inverse`, `spot` (limit only) |
| `tpTriggerBy` | string | `MarkPrice`, `IndexPrice`, `LastPrice`. Default: `LastPrice` | `linear`, `inverse` |
| `slTriggerBy` | string | `MarkPrice`, `IndexPrice`, `LastPrice`. Default: `LastPrice` | `linear`, `inverse` |
| `tpslMode` | string | `Full` (whole position) or `Partial` (partial position TP/SL, supports limit orders) | `linear`, `inverse` |
| `tpOrderType` | string | `Market` (default) or `Limit`. Full mode: must be Market | `linear`, `inverse`, `spot` |
| `slOrderType` | string | `Market` (default) or `Limit`. Full mode: must be Market | `linear`, `inverse`, `spot` |
| `tpLimitPrice` | string | Limit price when TP triggers. Required for spot when `tpOrderType=Limit` | `linear`, `inverse`, `spot` |
| `slLimitPrice` | string | Limit price when SL triggers. Required for spot when `slOrderType=Limit` | `linear`, `inverse`, `spot` |

---

## 2. ORDER MANAGEMENT

### Endpoints Summary

| Action | Method | Endpoint | Auth | Rate Limit (linear) |
|--------|--------|----------|------|---------------------|
| Place order | POST | `/v5/order/create` | Required | 20/s |
| Amend order | POST | `/v5/order/amend` | Required | 10/s |
| Cancel order | POST | `/v5/order/cancel` | Required | 20/s |
| Cancel all orders | POST | `/v5/order/cancel-all` | Required | 20/s |
| Open orders | GET | `/v5/order/realtime` | Required | — |
| Order history | GET | `/v5/order/history` | Required | 50/s |
| Trade executions | GET | `/v5/execution/list` | Required | — |

---

### POST /v5/order/create

**Request example:**
```json
{
  "category": "spot",
  "symbol": "BTCUSDT",
  "side": "Buy",
  "orderType": "Limit",
  "qty": "0.1",
  "price": "15600",
  "timeInForce": "PostOnly",
  "orderLinkId": "my-order-001",
  "isLeverage": 0
}
```

**Response:**
```json
{
  "retCode": 0,
  "retMsg": "OK",
  "result": {
    "orderId": "1321003749386327552",
    "orderLinkId": "my-order-001"
  },
  "retExtInfo": {},
  "time": 1672211918471
}
```

NOTE: The create response contains only `orderId` and `orderLinkId`. Full order details must be fetched via `/v5/order/realtime` or received via WebSocket.

---

### POST /v5/order/amend

Bybit V5 natively supports order amendment — price, qty, and TP/SL can be modified without cancelling and re-creating.

**Required:** `category`, `symbol`, plus at least one of `orderId` or `orderLinkId`

**Optional amendable fields:**
- `qty` — new order quantity
- `price` — new limit price
- `triggerPrice` — new conditional trigger price
- `orderIv` — implied volatility (options only)
- `takeProfit` — set to `"0"` to cancel TP
- `stopLoss` — set to `"0"` to cancel SL
- `tpTriggerBy`, `slTriggerBy`
- `tpslMode`, `tpLimitPrice`, `slLimitPrice`
- `triggerBy`

**Constraint:** Only `unfilled` or `partially filled` orders can be amended. Conditional orders are NOT supported in batch amend.

**Response:**
```json
{
  "retCode": 0,
  "result": {
    "orderId": "c6f055d9-7f21-4079-913d-e6523a9cfffa",
    "orderLinkId": "linear-004"
  }
}
```

---

### POST /v5/order/cancel

**Required:** `category`, `symbol`, plus at least one of `orderId` or `orderLinkId`

**Optional:** `orderFilter` (spot only: `Order`, `tpslOrder`, `StopOrder`)

**Constraint:** `orderId` takes priority if both identifiers are provided. Only unfilled or partially filled orders can be cancelled.

**Response:** Same structure as amend — returns `orderId` and `orderLinkId`.

---

### POST /v5/order/cancel-all

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | Product type |
| `symbol` | Conditional | Required for linear/inverse unless baseCoin or settleCoin provided |
| `baseCoin` | Conditional | Cancel all orders for this coin |
| `settleCoin` | Conditional | Cancel all orders settled in this coin |
| `orderFilter` | NO | Spot: `Order`, `tpslOrder`, `StopOrder`, `OcoOrder`, `BidirectionalTpslOrder` |
| `stopOrderType` | NO | `Stop` — for linear/inverse conditional stop orders |

**Cancellation limits:**
- Spot: no limit
- Futures: max 500 orders per call (random selection if exceeded)
- Options: no limit

---

### GET /v5/order/realtime

Returns currently open orders (unfilled and partially filled).

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | Product type |
| `symbol` | NO | Filter by symbol |
| `orderId` | NO | Specific order |
| `orderLinkId` | NO | Specific client order ID |
| `openOnly` | NO | `0` = active orders (default); `1` = active + last 500 closed orders |
| `limit` | NO | [1-50], default 20 |
| `cursor` | NO | Pagination cursor |

NOTE: Closed order cache limited to last 500 records. Data clears on service restart. Use `/v5/order/history` for persistent historical data.

---

### GET /v5/order/history

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | Product type |
| `symbol` | NO | Filter by symbol |
| `orderId` | NO | Specific order |
| `orderStatus` | NO | Filter by status |
| `startTime` / `endTime` | NO | Timestamp ms, max 7-day span |
| `limit` | NO | [1-50], default 20 |
| `cursor` | NO | Pagination cursor |

**Query rules:**
- Last 7 days: all closed statuses except Cancelled/Rejected/Deactivated
- Last 24 hours: can query Cancelled/Rejected/Deactivated
- Beyond 7 days: only orders with fills

---

### GET /v5/execution/list — Trade History / Fills

| Parameter | Required | Description |
|-----------|----------|-------------|
| `category` | YES | `linear`, `inverse`, `spot`, `option` |
| `symbol` | NO | Filter by symbol |
| `orderId` | NO | Filter by order |
| `orderLinkId` | NO | Filter by client order ID |
| `execType` | NO | Execution type filter |
| `startTime` / `endTime` | NO | Timestamp ms, defaults to last 7 days |
| `limit` | NO | [1-100], default 50 |
| `cursor` | NO | Pagination cursor |

**Response fields:**

| Field | Description |
|-------|-------------|
| `execId` | Unique execution ID |
| `orderId` | Associated order ID |
| `orderLinkId` | Client order ID |
| `symbol` | Trading pair |
| `side` | `Buy` or `Sell` |
| `execPrice` | Execution price |
| `execQty` | Executed quantity |
| `execFee` | Fee charged |
| `execType` | Execution type |
| `execTime` | Execution timestamp (ms) |
| `leavesQty` | Remaining unfilled quantity |
| `markPrice` | Mark price at execution |
| `isMaker` | Whether order was maker |
| `feeRate` | Applied fee rate |
| `closedSize` | Closed position size (if applicable) |
| `seq` | Cross-sequence number |

---

## 3. TP/SL & CONDITIONAL ORDERS

### TP/SL at Order Creation

Set `takeProfit` and/or `stopLoss` params directly in `/v5/order/create`. See Section 1 table above.

### TP/SL on Existing Position — `POST /v5/position/trading-stop`

This is the Bybit-specific endpoint for setting/modifying TP/SL/Trailing Stop on an **existing open position** without placing a new order.

**Supported categories:** `linear`, `inverse` ONLY (not spot)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `category` | string | YES | `linear` or `inverse` |
| `symbol` | string | YES | Trading pair |
| `positionIdx` | integer | YES | `0` = one-way, `1` = buy hedge, `2` = sell hedge |
| `tpslMode` | string | NO | `Full` or `Partial` |
| `takeProfit` | string | NO | TP price. `"0"` cancels TP |
| `stopLoss` | string | NO | SL price. `"0"` cancels SL |
| `trailingStop` | string | NO | Trailing stop distance. `"0"` cancels trailing stop |
| `activePrice` | string | NO | Trailing stop activation threshold price |
| `tpSize` | string | NO | Partial mode only — size for TP (must equal slSize if both set) |
| `slSize` | string | NO | Partial mode only — size for SL |
| `tpLimitPrice` | string | NO | Limit order price when TP triggers (Partial + Limit only) |
| `slLimitPrice` | string | NO | Limit order price when SL triggers (Partial + Limit only) |
| `tpOrderType` | string | NO | `Market` or `Limit`. Full mode: Market only |
| `slOrderType` | string | NO | `Market` or `Limit`. Full mode: Market only |
| `tpTriggerBy` | string | NO | `MarkPrice`, `IndexPrice`, `LastPrice` |
| `slTriggerBy` | string | NO | `MarkPrice`, `IndexPrice`, `LastPrice` |

**Response:** Empty result object `{}` on success.

### Conditional Orders

A conditional (stop) order is created via `/v5/order/create` by providing `triggerPrice`. The order remains `Untriggered` until the trigger condition is met, at which point it transitions to `Triggered` → `New` state.

- `triggerBy`: which price type activates the trigger (`LastPrice`, `MarkPrice`, `IndexPrice`)
- `triggerDirection`: `1` = trigger when price rises to `triggerPrice`; `2` = trigger when falls

---

## 4. BATCH OPERATIONS

### POST /v5/order/create-batch

| Category | Max Orders Per Batch |
|----------|---------------------|
| `linear` | 20 |
| `inverse` | 20 |
| `option` | 20 |
| `spot` | 10 |

**Request format:**
```json
{
  "category": "linear",
  "request": [
    {
      "symbol": "BTCUSDT",
      "side": "Buy",
      "orderType": "Limit",
      "qty": "0.1",
      "price": "30000",
      "timeInForce": "GTC",
      "orderLinkId": "batch-001"
    },
    {
      "symbol": "ETHUSDT",
      "side": "Sell",
      "orderType": "Market",
      "qty": "1"
    }
  ]
}
```

**Response format** (two parallel lists):
```json
{
  "result": {
    "list": [
      {
        "category": "linear",
        "symbol": "BTCUSDT",
        "orderId": "...",
        "orderLinkId": "batch-001",
        "createAt": "1672211918471"
      }
    ]
  },
  "retExtInfo": {
    "list": [
      { "code": 0, "msg": "OK" }
    ]
  }
}
```

Partial failures are possible: individual orders in `retExtInfo.list` will have non-zero `code` on failure while others succeed.

---

### POST /v5/order/amend-batch

- Path: `POST /v5/order/amend-batch`
- Max batch: same as create-batch (20/10 per category)
- Each item requires `symbol` + (`orderId` or `orderLinkId`)
- Optional fields: `qty`, `price`, `takeProfit`, `stopLoss`, trigger params
- **NOTE:** Conditional orders are NOT supported in batch amend

---

### POST /v5/order/cancel-batch

- Path: `POST /v5/order/cancel-batch`
- Max batch: same as create-batch (20/10 per category)
- Each item requires `symbol` + (`orderId` or `orderLinkId`)
- `orderId` takes priority if both provided

---

## 5. ALGO ORDERS

Bybit V5 does **NOT** expose native TWAP, VWAP, Grid, or DCA order types via the public trading API. These are only available through the Bybit web/app UI.

The V5 API provides all primitives needed to implement these strategies programmatically:
- Limit and Market orders with full control over timing and sizing
- Conditional orders via `triggerPrice` for entry/exit at specific prices
- Batch create/amend/cancel for multi-leg strategies
- TP/SL at order creation and via `trading-stop` for position management

Summary: **No native algo order API endpoints exist in Bybit V5.**

---

## 6. ORDER RESPONSE FORMAT & STATUS VALUES

### Order Status (`orderStatus`)

**Open statuses:**

| Value | Description |
|-------|-------------|
| `New` | Order placed successfully, not yet filled |
| `PartiallyFilled` | Partially executed |
| `Untriggered` | Conditional order waiting for trigger |

**Closed statuses:**

| Value | Description |
|-------|-------------|
| `Filled` | Completely filled |
| `Cancelled` | Cancelled. In derivatives, may still have partial execution |
| `PartiallyFilledCanceled` | Spot only: partially filled then cancelled |
| `Triggered` | Transient state: conditional order just triggered, transitioning to `New` |
| `Deactivated` | UTA: Spot TP/SL, conditional, or OCO order was cancelled |
| `Rejected` | Rejected by exchange |

### `orderId` vs `orderLinkId`

| Field | Type | Description |
|-------|------|-------------|
| `orderId` | string | System-generated unique order ID. Always present. |
| `orderLinkId` | string | Client-defined order ID. Optional on create; must be unique per category per account. Useful for idempotency. |

If both are provided on cancel/amend and they don't match, `orderId` takes priority.

### Full Open Order Response Fields (from `/v5/order/realtime`)

| Field | Description |
|-------|-------------|
| `orderId` | Unique order ID |
| `orderLinkId` | Client order ID |
| `symbol` | Trading pair |
| `side` | `Buy` or `Sell` |
| `orderType` | `Market` or `Limit` |
| `price` | Order price |
| `qty` | Order quantity |
| `orderStatus` | Current status (see table above) |
| `avgPrice` | Average fill price |
| `cumExecQty` | Total executed quantity |
| `cumExecValue` | Total executed value |
| `leavesQty` | Remaining quantity |
| `leavesValue` | Remaining value |
| `stopOrderType` | Conditional order sub-type |
| `triggerPrice` | Conditional trigger price |
| `triggerBy` | Trigger price type |
| `triggerDirection` | Trigger direction |
| `takeProfit` | TP price |
| `stopLoss` | SL price |
| `tpTriggerBy` | TP trigger type |
| `slTriggerBy` | SL trigger type |
| `timeInForce` | Time in force |
| `positionIdx` | Position index |
| `reduceOnly` | Whether reduce-only |
| `closeOnTrigger` | Whether close-on-trigger |
| `isLeverage` | Spot margin flag |
| `createdTime` | Creation timestamp (ms) |
| `updatedTime` | Last update timestamp (ms) |
| `cancelType` | Cancellation reason |
| `rejectReason` | Rejection reason |
| `blockTradeId` | Block trade ID if applicable |
| `cumFeeDetail` | Fee breakdown object |
