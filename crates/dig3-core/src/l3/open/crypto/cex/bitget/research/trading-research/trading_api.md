# Bitget V2 Trading API — Order Management

Base URL: `https://api.bitget.com`

---

## Overview

Bitget V2 splits trading into two distinct namespaces:
- **Spot V2**: `/api/v2/spot/trade/`
- **Futures (Mix) V2**: `/api/v2/mix/order/`

Plan/TP/SL orders for futures are under `/api/v2/mix/order/` (plan endpoints).

All order endpoints require authentication headers (see `auth_levels.md`).

---

## Spot V2 — Order Types

| `orderType` | `force` options | Notes |
|-------------|-----------------|-------|
| `limit`     | `gtc`, `post_only`, `fok`, `ioc` | Price required |
| `market`    | `gtc` | No price; `size` is quote amount for buy, base amount for sell |

`side`: `buy` or `sell`

---

## Spot V2 — Endpoints

### 1. Place Order

**POST** `/api/v2/spot/trade/place-order`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":    "BTCUSDT",
  "side":      "buy",
  "orderType": "limit",
  "force":     "gtc",
  "price":     "30000.00",
  "size":      "0.001",
  "clientOid": "my_order_001",
  "tpslType":  "normal",
  "presetTakeProfitPrice": "35000.00",
  "presetStopLossPrice":   "28000.00"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair, e.g. `BTCUSDT` |
| `side` | string | Yes | `buy` or `sell` |
| `orderType` | string | Yes | `limit` or `market` |
| `force` | string | Yes | `gtc`, `post_only`, `fok`, `ioc` |
| `price` | string | Cond. | Required for `limit` orders |
| `size` | string | Yes | Base qty for limit/sell-market; quote qty for buy-market |
| `clientOid` | string | No | Client-defined order ID (max 40 chars) |
| `tpslType` | string | No | `normal` (default) or `tpsl` |
| `presetTakeProfitPrice` | string | No | Preset TP price (triggers market close) |
| `presetStopLossPrice` | string | No | Preset SL price (triggers market close) |

**Response:**

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695808690167,
  "data": {
    "orderId":   "1088499571612344321",
    "clientOid": "my_order_001"
  }
}
```

---

### 2. Cancel Order

**POST** `/api/v2/spot/trade/cancel-order`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":    "BTCUSDT",
  "orderId":   "1088499571612344321",
  "clientOid": "my_order_001"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `orderId` | string | Cond. | Either `orderId` or `clientOid` required |
| `clientOid` | string | Cond. | Either `orderId` or `clientOid` required |

**Response:**

```json
{
  "code": "00000",
  "msg": "success",
  "data": {
    "orderId":   "1088499571612344321",
    "clientOid": "my_order_001"
  }
}
```

---

### 3. Batch Place Orders

**POST** `/api/v2/spot/trade/batch-orders`

Rate limit: 5 req/sec/UID
Max batch size: **50 orders**

**Request Body:**

```json
{
  "symbol": "BTCUSDT",
  "orderList": [
    {
      "side":      "buy",
      "orderType": "limit",
      "force":     "gtc",
      "price":     "29000.00",
      "size":      "0.001",
      "clientOid": "batch_001"
    },
    {
      "side":      "buy",
      "orderType": "limit",
      "force":     "gtc",
      "price":     "28000.00",
      "size":      "0.001",
      "clientOid": "batch_002"
    }
  ]
}
```

**Response:**

```json
{
  "code": "00000",
  "msg": "success",
  "data": {
    "successList": [
      { "orderId": "111", "clientOid": "batch_001" }
    ],
    "failureList": [
      { "clientOid": "batch_002", "errorCode": "43012", "errorMsg": "Insufficient balance" }
    ]
  }
}
```

---

### 4. Batch Cancel Orders

**POST** `/api/v2/spot/trade/batch-cancel-order`

Rate limit: 5 req/sec/UID
Max batch size: **50 orders**

**Request Body:**

```json
{
  "symbol": "BTCUSDT",
  "orderList": [
    { "orderId": "1088499571612344321" },
    { "clientOid": "batch_002" }
  ]
}
```

---

### 5. Cancel All Orders by Symbol

**POST** `/api/v2/spot/trade/cancel-symbol-orders`

**Request Body:**

```json
{
  "symbol": "BTCUSDT"
}
```

---

### 6. Get Open Orders (Unfilled)

**GET** `/api/v2/spot/trade/unfilled-orders`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | No | Filter by trading pair |
| `orderId` | string | No | Start order ID for pagination |
| `startTime` | string | No | Start timestamp (ms) |
| `endTime` | string | No | End timestamp (ms) |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination cursor |

**Response (single order object):**

```json
{
  "userId":       "123456",
  "symbol":       "BTCUSDT",
  "orderId":      "1088499571612344321",
  "clientOid":    "my_order_001",
  "price":        "30000.00",
  "size":         "0.001",
  "orderType":    "limit",
  "side":         "buy",
  "status":       "live",
  "priceAvg":     "0",
  "baseVolume":   "0",
  "quoteVolume":  "0",
  "enterPointSource": "API",
  "feeDetail": {
    "feeCoin":   "BTC",
    "totalFee":  "0"
  },
  "cTime":        "1695808690167",
  "uTime":        "1695808690167"
}
```

---

### 7. Get Order History

**GET** `/api/v2/spot/trade/history-orders`

Rate limit: 10 req/sec/UID
Supports data within 90 days.

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | No | Trading pair |
| `orderId` | string | No | Specific order ID |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination cursor |

**Response fields** (same as open orders) plus:
- `status`: `filled`, `cancelled`, `partial_fill`

---

### 8. Get Fills (Trade Executions)

**GET** `/api/v2/spot/trade/fills`

Rate limit: 10 req/sec/UID
Supports data within 90 days.

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | No | Now optional (changed from required in V2) |
| `orderId` | string | No | Filter by order |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination cursor |

**Response (fill object):**

```json
{
  "userId":    "123456",
  "symbol":    "BTCUSDT",
  "orderId":   "1088499571612344321",
  "tradeId":   "1088500000000000001",
  "orderType": "limit",
  "side":      "buy",
  "priceAvg":  "30000.00",
  "size":      "0.001",
  "amount":    "30.00",
  "feeDetail": {
    "feeCoin":  "BTC",
    "totalFee": "-0.000001"
  },
  "tradeScope": "taker",
  "cTime":     "1695808690167",
  "uTime":     "1695808690167"
}
```

---

## Futures (Mix) V2 — Order Types

**`productType`** values:

| Value | Description |
|-------|-------------|
| `USDT-FUTURES` | USDT-margined perpetual (live) |
| `USDC-FUTURES` | USDC-margined perpetual (live) |
| `COIN-FUTURES` | Coin-margined perpetual/delivery (live) |
| `SUSDT-FUTURES` | USDT-margined demo/simulation |
| `SUSDC-FUTURES` | USDC-margined demo/simulation |
| `SCOIN-FUTURES` | Coin-margined demo/simulation |

**`side`** + **`tradeSide`** (hedge mode):

| `side` | `tradeSide` | Effect |
|--------|-------------|--------|
| `buy`  | `open`      | Open long |
| `sell` | `open`      | Open short |
| `buy`  | `close`     | Close short |
| `sell` | `close`     | Close long |

In one-way mode, `tradeSide` is optional; set `side` to `buy`/`sell` directly.

**`orderType`**: `limit` or `market`
**`force`**: `gtc`, `post_only`, `fok`, `ioc`

---

## Futures (Mix) V2 — Endpoints

### 1. Place Order

**POST** `/api/v2/mix/order/place-order`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginMode":  "isolated",
  "marginCoin":  "USDT",
  "size":        "0.01",
  "price":       "30000.00",
  "side":        "buy",
  "tradeSide":   "open",
  "orderType":   "limit",
  "force":       "gtc",
  "clientOid":   "futures_001",
  "reduceOnly":  "NO",
  "presetStopSurplusPrice": "35000.00",
  "presetStopLossPrice":    "28000.00"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | e.g. `BTCUSDT` |
| `productType` | string | Yes | See table above |
| `marginMode` | string | Yes | `isolated` or `crossed` |
| `marginCoin` | string | Yes | e.g. `USDT`, `BTC` |
| `size` | string | Yes | Contract quantity |
| `price` | string | Cond. | Required for `limit` |
| `side` | string | Yes | `buy` or `sell` |
| `tradeSide` | string | Cond. | `open` or `close` (hedge mode) |
| `orderType` | string | Yes | `limit` or `market` |
| `force` | string | Yes | `gtc`, `post_only`, `fok`, `ioc` |
| `clientOid` | string | No | Client order ID |
| `reduceOnly` | string | No | `YES` or `NO` (one-way mode) |
| `presetStopSurplusPrice` | string | No | Preset TP trigger price |
| `presetStopLossPrice` | string | No | Preset SL trigger price |
| `presetStopSurplusTriggerType` | string | No | `fill_price` or `mark_price` |
| `presetStopLossTriggerType` | string | No | `fill_price` or `mark_price` |

**Response:**

```json
{
  "code": "00000",
  "msg": "success",
  "data": {
    "orderId":   "1088499571612344321",
    "clientOid": "futures_001"
  }
}
```

---

### 2. Cancel Order

**POST** `/api/v2/mix/order/cancel-order`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "orderId":     "1088499571612344321",
  "clientOid":   "futures_001"
}
```

---

### 3. Modify Order

**POST** `/api/v2/mix/order/modify-order`

Rate limit: 10 req/sec/UID
Only works on unfilled/partially-filled orders.

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "orderId":     "1088499571612344321",
  "newClientOid": "futures_modified_001",
  "newSize":     "0.02",
  "newPrice":    "29500.00",
  "presetStopSurplusPrice": "35500.00",
  "presetStopLossPrice":    "27500.00"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `productType` | string | Yes | Product type |
| `marginCoin` | string | Yes | Margin coin |
| `orderId` | string | Cond. | Either `orderId` or `clientOid` required |
| `clientOid` | string | Cond. | Client order ID |
| `newClientOid` | string | No | New client order ID |
| `newSize` | string | No | New quantity |
| `newPrice` | string | No | New price (limit orders only) |
| `presetStopSurplusPrice` | string | No | Modified TP price |
| `presetStopLossPrice` | string | No | Modified SL price |

Note: When modifying only TP/SL, do not pass `newPrice` or `newSize`.

---

### 4. Batch Place Orders

**POST** `/api/v2/mix/order/batch-place-order`

Rate limit: 5 req/sec/UID
Max batch size: **50 orders**

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "orderList": [
    {
      "marginMode": "isolated",
      "size":       "0.01",
      "price":      "30000.00",
      "side":       "buy",
      "tradeSide":  "open",
      "orderType":  "limit",
      "force":      "gtc",
      "clientOid":  "b_001"
    }
  ]
}
```

---

### 5. Batch Cancel Orders

**POST** `/api/v2/mix/order/batch-cancel-orders`

Rate limit: 5 req/sec/UID
Max batch size: **50 orders**

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "orderIdList": [
    { "orderId": "1088499571612344321" },
    { "clientOid": "futures_002" }
  ]
}
```

---

### 6. Flash Close Position (Market Close All)

**POST** `/api/v2/mix/order/close-positions`

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "holdSide":    "long"
}
```

---

### 7. Get Open Orders (Pending)

**GET** `/api/v2/mix/order/orders-pending`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `symbol` | string | No | Filter by symbol |
| `orderId` | string | No | Specific order |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination |

**Response (order object):**

```json
{
  "symbol":       "BTCUSDT",
  "size":         "0.01",
  "orderId":      "1088499571612344321",
  "clientOid":    "futures_001",
  "baseVolume":   "0",
  "fee":          "0",
  "price":        "30000.00",
  "priceAvg":     "0",
  "status":       "live",
  "side":         "buy",
  "force":        "gtc",
  "totalProfits": "0",
  "posSide":      "long",
  "marginCoin":   "USDT",
  "presetStopSurplusPrice": "35000.00",
  "presetStopLossPrice":    "28000.00",
  "quoteVolume":  "0",
  "orderType":    "limit",
  "leverage":     "10",
  "marginMode":   "isolated",
  "reduceOnly":   "NO",
  "enterPointSource": "API",
  "tradeSide":    "open",
  "posMode":      "hedge_mode",
  "orderSource":  "normal",
  "cTime":        "1695808690167",
  "uTime":        "1695808690167"
}
```

---

### 8. Get Order History

**GET** `/api/v2/mix/order/orders-history`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `symbol` | string | No | Filter by symbol |
| `orderId` | string | No | Specific order |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination |

Response structure same as pending orders with additional `status` values: `filled`, `cancelled`, `partial_fill`.

---

### 9. Get Order Fill Details

**GET** `/api/v2/mix/order/fills`

Rate limit: 10 req/sec/UID

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `symbol` | string | No | Filter by symbol |
| `orderId` | string | No | Filter by order |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |
| `idLessThan` | string | No | Pagination |

**Response (fill object):**

```json
{
  "tradeId":      "1088500000000000001",
  "symbol":       "BTCUSDT",
  "orderId":      "1088499571612344321",
  "price":        "30000.00",
  "baseVolume":   "0.01",
  "feeDetail": [
    {
      "deduction":  "no",
      "feeCoin":    "USDT",
      "totalDeductionFee": "0",
      "totalFee":   "-0.018"
    }
  ],
  "side":         "buy",
  "fillAmount":   "300.00",
  "profit":       "0",
  "enterPointSource": "API",
  "tradeSide":    "open",
  "holdMode":     "hedge_mode",
  "posSide":      "long",
  "marginCoin":   "USDT",
  "tradeScope":   "taker",
  "cTime":        "1695808690167",
  "uTime":        "1695808690167"
}
```

---

## Futures (Mix) V2 — Plan / TP/SL Orders

Plan orders are separate from regular orders on Bitget. There are three types:

| Plan Type | Endpoint | Use Case |
|-----------|----------|----------|
| Trigger order | `/api/v2/mix/order/place-plan-order` | Entry on price trigger |
| TP/SL on position | `/api/v2/mix/order/place-tpsl-order` | TP or SL on existing position |
| Simultaneous TP+SL | `/api/v2/mix/order/place-pos-tpsl-order` | Set both TP and SL at once |

---

### 1. Place Plan/Trigger Order

**POST** `/api/v2/mix/order/place-plan-order`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":        "BTCUSDT",
  "productType":   "USDT-FUTURES",
  "marginMode":    "isolated",
  "marginCoin":    "USDT",
  "size":          "0.01",
  "price":         "30000.00",
  "triggerPrice":  "31000.00",
  "triggerType":   "mark_price",
  "side":          "buy",
  "tradeSide":     "open",
  "orderType":     "limit",
  "planType":      "normal_plan",
  "clientOid":     "plan_001",
  "presetStopSurplusPrice": "35000.00",
  "presetStopLossPrice":    "28000.00",
  "stopSurplusTriggerPrice": "35000.00",
  "stopLossTriggerPrice":    "28000.00",
  "stopSurplusTriggerType":  "mark_price",
  "stopLossTriggerType":     "mark_price"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `productType` | string | Yes | Product type |
| `marginMode` | string | Yes | `isolated` or `crossed` |
| `marginCoin` | string | Yes | Margin coin |
| `size` | string | Yes | Order size |
| `price` | string | Cond. | Execution price (limit plan) |
| `triggerPrice` | string | Yes | Trigger price |
| `triggerType` | string | Yes | `fill_price` or `mark_price` |
| `side` | string | Yes | `buy` or `sell` |
| `tradeSide` | string | Cond. | `open` or `close` (hedge) |
| `orderType` | string | Yes | `limit` or `market` |
| `planType` | string | Yes | `normal_plan`, `track_plan` (trailing stop) |
| `clientOid` | string | No | Client ID |
| `callbackRatio` | string | Cond. | Required for `track_plan` (trailing %) |
| `reduceOnly` | string | No | `YES` or `NO` |
| `stopSurplusTriggerPrice` | string | No | TP trigger attached to this plan order |
| `stopLossTriggerPrice` | string | No | SL trigger attached to this plan order |
| `stopSurplusTriggerType` | string | No | `fill_price` or `mark_price` |
| `stopLossTriggerType` | string | No | `fill_price` or `mark_price` |

**Response:**

```json
{
  "code": "00000",
  "data": {
    "orderId":   "1088600000000000001",
    "clientOid": "plan_001"
  }
}
```

---

### 2. Place TP/SL Order (Position-Level)

**POST** `/api/v2/mix/order/place-tpsl-order`

Rate limit: 10 req/sec/UID

**Request Body:**

```json
{
  "symbol":       "BTCUSDT",
  "productType":  "USDT-FUTURES",
  "marginCoin":   "USDT",
  "planType":     "profit_loss",
  "triggerPrice": "35000.00",
  "triggerType":  "mark_price",
  "executePrice": "0",
  "holdSide":     "long",
  "size":         "0.01",
  "clientOid":    "tpsl_001"
}
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `productType` | string | Yes | Product type |
| `marginCoin` | string | Yes | Margin coin |
| `planType` | string | Yes | `profit_loss`, `loss` (SL only), `profit` (TP only) |
| `triggerPrice` | string | Yes | Trigger price |
| `triggerType` | string | Yes | `fill_price` or `mark_price` |
| `executePrice` | string | No | Execution price (`0` = market) |
| `holdSide` | string | Yes | `long` or `short` |
| `size` | string | No | Partial close size (omit for full position) |
| `clientOid` | string | No | Client ID |
| `rangeRate` | string | No | Trailing stop range rate |

---

### 3. Place Simultaneous TP + SL (Position-Level)

**POST** `/api/v2/mix/order/place-pos-tpsl-order`

Sets both take-profit and stop-loss on an existing position atomically.

**Request Body:**

```json
{
  "symbol":              "BTCUSDT",
  "productType":         "USDT-FUTURES",
  "marginCoin":          "USDT",
  "holdSide":            "long",
  "stopSurplusTriggerPrice": "35000.00",
  "stopSurplusTriggerType":  "mark_price",
  "stopSurplusExecutePrice": "0",
  "stopLossTriggerPrice":    "28000.00",
  "stopLossTriggerType":     "mark_price",
  "stopLossExecutePrice":    "0",
  "clientOid":           "pos_tpsl_001"
}
```

---

### 4. Modify Plan Order

**POST** `/api/v2/mix/order/modify-plan-order`

**Request Body:**

```json
{
  "orderId":       "1088600000000000001",
  "clientOid":     "plan_001",
  "symbol":        "BTCUSDT",
  "productType":   "USDT-FUTURES",
  "marginCoin":    "USDT",
  "triggerPrice":  "31500.00",
  "triggerType":   "mark_price",
  "price":         "31400.00",
  "size":          "0.01"
}
```

---

### 5. Modify TP/SL Order

**POST** `/api/v2/mix/order/modify-tpsl-order`

**Request Body:**

```json
{
  "orderId":      "1088600000000000002",
  "clientOid":    "tpsl_001",
  "symbol":       "BTCUSDT",
  "productType":  "USDT-FUTURES",
  "marginCoin":   "USDT",
  "triggerPrice": "36000.00",
  "triggerType":  "mark_price",
  "executePrice": "0",
  "size":         "0.01"
}
```

---

### 6. Cancel Plan Order

**POST** `/api/v2/mix/order/cancel-plan-order`

**Request Body:**

```json
{
  "symbol":      "BTCUSDT",
  "productType": "USDT-FUTURES",
  "marginCoin":  "USDT",
  "orderId":     "1088600000000000001",
  "planType":    "normal_plan"
}
```

`planType`: `normal_plan`, `profit_loss`, `loss`, `profit`

---

### 7. Get Current Plan Orders

**GET** `/api/v2/mix/order/orders-plan-pending`

**Query Parameters:**

| Param | Type | Required | Description |
|-------|------|----------|-------------|
| `productType` | string | Yes | Product type |
| `symbol` | string | No | Filter by symbol |
| `planType` | string | No | Filter by plan type |
| `startTime` | string | No | ms timestamp |
| `endTime` | string | No | ms timestamp |
| `limit` | string | No | Default 20, max 100 |

---

### 8. Get Plan Order History

**GET** `/api/v2/mix/order/orders-plan-history`

Same parameters as current plan orders.

---

## Order Status Values

### Spot

| Status | Description |
|--------|-------------|
| `live` | Open, not filled |
| `partially_filled` | Partially executed |
| `filled` | Fully executed |
| `cancelled` | Cancelled |
| `initial` | Pending submission |

### Futures

| Status | Description |
|--------|-------------|
| `live` | Open |
| `partially_filled` | Partially executed |
| `filled` | Fully executed |
| `cancelled` | Cancelled |

### Plan Orders

| Status | Description |
|--------|-------------|
| `live` | Waiting for trigger |
| `executed` | Triggered and placed |
| `cancelled` | Cancelled |
| `failed` | Triggered but order placement failed |

---

## Batch Operation Limits Summary

| Operation | Max Batch | Rate Limit |
|-----------|-----------|------------|
| Spot batch place | 50 | 5 req/sec/UID |
| Spot batch cancel | 50 | 5 req/sec/UID |
| Futures batch place | 50 | 5 req/sec/UID |
| Futures batch cancel | 50 | 5 req/sec/UID |

---

## Sources

- [Place Spot Order](https://www.bitget.com/api-doc/spot/trade/Place-Order)
- [Batch Place Spot Orders](https://www.bitget.com/api-doc/spot/trade/Batch-Place-Orders)
- [Batch Cancel Spot Orders](https://www.bitget.com/api-doc/spot/trade/Batch-Cancel-Orders)
- [Place Futures Order](https://www.bitget.com/api-doc/contract/trade/Place-Order)
- [Modify Futures Order](https://www.bitget.com/api-doc/contract/trade/Modify-Order)
- [Cancel Futures Order](https://www.bitget.com/api-doc/contract/trade/Cancel-Order)
- [Place Plan/Trigger Order](https://www.bitget.com/api-doc/contract/plan/Place-Plan-Order)
- [Place TPSL Order](https://www.bitget.com/api-doc/contract/plan/Place-Tpsl-Order)
- [Place Position TPSL Order](https://www.bitget.com/api-doc/contract/plan/Place-Pos-Tpsl-Order)
- [Get Futures Order Fills](https://www.bitget.com/api-doc/contract/trade/Get-Order-Fills)
- [Get Spot Fills](https://www.bitget.com/api-doc/spot/trade/Get-Fills)
- [V2 API Update Guide](https://www.bitget.com/api-doc/common/release-note)
