# MEXC Trading API — Order Management

## Overview

MEXC provides two entirely separate trading APIs:
- **Spot V3** — Binance-compatible REST API at `https://api.mexc.com`
- **Futures (Contract V1)** — Separate API at `https://contract.mexc.com`

These share no endpoints and use different authentication schemes.

---

## SPOT API — Order Endpoints

### Base URL
```
https://api.mexc.com
```

---

### POST /api/v3/order — Place New Order

**Permission:** `SPOT_DEAL_WRITE`
**Weight:** 1 (IP), 1 (UID)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | YES | Trading pair, e.g. `BTCUSDT` |
| `side` | ENUM | YES | `BUY` or `SELL` |
| `type` | ENUM | YES | Order type (see below) |
| `quantity` | DECIMAL | NO | Base asset quantity |
| `quoteOrderQty` | DECIMAL | NO | Quote asset quantity (MARKET buy only) |
| `price` | DECIMAL | NO | Limit price |
| `newClientOrderId` | STRING | NO | Custom order ID (client-side reference) |
| `recvWindow` | LONG | NO | Max `60000` ms; default `5000` |
| `timestamp` | LONG | YES | Current Unix time in milliseconds |

#### Order Types

| Type | Required Fields | Description |
|------|----------------|-------------|
| `LIMIT` | `quantity`, `price` | Standard limit order, GTC by default |
| `MARKET` | `quantity` OR `quoteOrderQty` | Execute at best available price |
| `LIMIT_MAKER` | `quantity`, `price` | Post-only; rejected if it would take liquidity |
| `IOC` | `quantity`, `price` | Immediate-or-Cancel: fill what you can, cancel the rest |
| `FOK` | `quantity`, `price` | Fill-or-Kill: fill entirely or cancel entirely |

**NOTE: `timeInForce` parameter is NOT separately supported.** IOC and FOK are specified directly via `type`. There is no `GTC`/`GTD` enum field.

#### Response JSON

```json
{
  "symbol": "MXUSDT",
  "orderId": "06a480e69e604477bfb48dddd5f0b750",
  "orderListId": -1,
  "price": "0.1",
  "origQty": "50",
  "type": "LIMIT",
  "side": "BUY",
  "transactTime": 1666676533741
}
```

**Fields:**
- `orderId` — STRING (UUID-style hex, not integer like Binance)
- `orderListId` — always `-1` (OCO not supported)
- `transactTime` — Unix milliseconds

**NOT returned in new-order response** (unlike Binance):
- `status` — not in create response
- `executedQty` — not in create response
- `cummulativeQuoteQty` — not in create response

---

### POST /api/v3/order/test — Test New Order

**Permission:** `SPOT_DEAL_WRITE`
**Weight:** 1 (IP)

Same parameters as `/api/v3/order`. Validates parameters without placing a real order. Returns empty `{}` on success.

**NOTE: This is a validation endpoint only — no simulated fill data is returned. There is no real testnet.**

---

### DELETE /api/v3/order — Cancel Order

**Permission:** `SPOT_DEAL_WRITE`
**Weight:** 1 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | YES | Trading pair |
| `orderId` | STRING | NO* | Order ID to cancel |
| `origClientOrderId` | STRING | NO* | Client order ID to cancel |
| `newClientOrderId` | STRING | NO | New client ID for the cancellation record |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

*Either `orderId` or `origClientOrderId` must be provided.

#### Response JSON

```json
{
  "symbol": "LTCBTC",
  "origClientOrderId": "myOrder1",
  "orderId": 4,
  "clientOrderId": "cancelMyOrder1",
  "price": "2.00000000",
  "origQty": "1.00000000",
  "executedQty": "0.00000000",
  "cummulativeQuoteQty": "0.00000000",
  "status": "CANCELED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY"
}
```

---

### DELETE /api/v3/openOrders — Cancel All Open Orders

**Permission:** `SPOT_DEAL_WRITE`
**Weight:** 1 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | NO | Cancel for up to 5 symbols (comma-separated) |
| `timestamp` | LONG | YES | |

#### Response JSON

Array of cancelled orders:
```json
[
  {
    "symbol": "BTCUSDT",
    "origClientOrderId": "E6APeyTJvkMvLMYMqu1KQ4",
    "orderId": 11,
    "orderListId": -1,
    "clientOrderId": "pXLV6Hz6mprAcVYpVMTGgx",
    "price": "0.089853",
    "origQty": "0.178622",
    "executedQty": "0.000000",
    "cummulativeQuoteQty": "0.000000",
    "status": "CANCELED",
    "timeInForce": "GTC",
    "type": "LIMIT",
    "side": "BUY"
  }
]
```

---

### GET /api/v3/order — Query Order Status

**Permission:** `SPOT_DEAL_READ`
**Weight:** 2 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | YES | |
| `orderId` | STRING | NO* | |
| `origClientOrderId` | STRING | NO* | |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

*Either `orderId` or `origClientOrderId` required.

#### Response JSON

```json
{
  "symbol": "LTCBTC",
  "orderId": 1,
  "orderListId": -1,
  "clientOrderId": "myOrder1",
  "price": "0.1",
  "origQty": "1.0",
  "executedQty": "0.0",
  "cummulativeQuoteQty": "0.0",
  "status": "NEW",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY",
  "stopPrice": "0.0",
  "time": 1499827319559,
  "updateTime": 1499827319559,
  "isWorking": true,
  "origQuoteOrderQty": "0.000000"
}
```

**Order Status Values:**
- `NEW` — Order placed, not yet filled
- `PARTIALLY_FILLED` — Partially executed
- `FILLED` — Fully executed
- `CANCELED` — Cancelled by user or system
- `PENDING_CANCEL` — Cancel in progress (rare)

**NOTE:** `stopPrice` field exists in response but **stop orders are NOT natively supported** on Spot. This field is typically `"0.0"`.

---

### GET /api/v3/openOrders — Current Open Orders

**Permission:** `SPOT_DEAL_READ`
**Weight:** 3 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | NO | Up to 5 symbols comma-separated |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

Returns array of order objects in same format as `GET /api/v3/order`.

---

### GET /api/v3/allOrders — All Orders (History)

**Permission:** `SPOT_DEAL_READ`
**Weight:** 10 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | YES | Single symbol only |
| `orderId` | STRING | NO | Return orders >= this orderId |
| `startTime` | LONG | NO | |
| `endTime` | LONG | NO | |
| `limit` | INT | NO | Default 500; max 1000 |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

**IMPORTANT LIMITATIONS:**
- Default query period: last 24 hours
- Maximum query range: 7 days
- Maximum records returned: 1000

#### Response JSON

Array of order objects:
```json
[
  {
    "symbol": "LTCBTC",
    "orderId": 1,
    "orderListId": -1,
    "clientOrderId": "myOrder1",
    "price": "0.1",
    "origQty": "1.0",
    "executedQty": "0.0",
    "cummulativeQuoteQty": "0.0",
    "status": "NEW",
    "timeInForce": "GTC",
    "type": "LIMIT",
    "side": "BUY",
    "stopPrice": "0.0",
    "icebergQty": "0.0",
    "time": 1499827319559,
    "updateTime": 1499827319559,
    "isWorking": true,
    "origQuoteOrderQty": "0.000000"
  }
]
```

---

### GET /api/v3/myTrades — Account Trade List

**Permission:** `SPOT_ACCOUNT_READ`
**Weight:** 10 (IP)

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | STRING | YES | |
| `orderId` | STRING | NO | Filter by order |
| `startTime` | LONG | NO | |
| `endTime` | LONG | NO | |
| `limit` | INT | NO | Default 500; max 100 |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

#### Response JSON

```json
[
  {
    "symbol": "BNBBTC",
    "id": "fad2af9e942049b6adbda1a271f990c6",
    "orderId": "bb41e5663e124046bd9497a3f5692f39",
    "orderListId": -1,
    "price": "4.00000100",
    "qty": "12.00000000",
    "quoteQty": "48.000012",
    "commission": "10.10000000",
    "commissionAsset": "BNB",
    "time": 1499865549590,
    "isBuyer": true,
    "isMaker": false,
    "isBestMatch": true,
    "isSelfTrade": true,
    "clientOrderId": null
  }
]
```

**MEXC-specific field:**
- `isSelfTrade` — BOOL: indicates self-trade (not present in Binance)

---

### POST /api/v3/batchOrders — Batch Order Placement

**Permission:** `SPOT_DEAL_WRITE`
**Weight:** 1 (IP), 1 (UID)
**Rate Limit:** 2 times/second (separate limit)

**Constraints:**
- Maximum 20 orders per request
- All orders must be for the **same symbol**

#### Request JSON

```json
{
  "batchOrders": [
    {
      "symbol": "BTCUSDT",
      "side": "BUY",
      "type": "LIMIT",
      "quantity": "0.0002",
      "price": "40000"
    },
    {
      "symbol": "BTCUSDT",
      "side": "SELL",
      "type": "LIMIT",
      "quantity": "0.0002",
      "price": "50000"
    }
  ]
}
```

Response is an array of individual order responses.

---

## NOT SUPPORTED — Spot API

- **Modify/amend order** — No `PUT /api/v3/order` endpoint; cancel and resubmit required
- **Stop-loss / Take-profit orders** — Not supported natively on Spot
- **Conditional orders / Trigger orders** — Not supported on Spot
- **OCO (One-Cancels-Other)** — Not supported (`orderListId` always -1)
- **Trailing stop** — Not supported
- **Iceberg orders** — `icebergQty` field exists in response but creation not documented
- **Testnet** — No testnet environment; `/api/v3/order/test` only validates params

---

## FUTURES (Contract) API — Order Endpoints

### Base URL
```
https://contract.mexc.com
```

**CRITICAL DIFFERENCE:** Symbol format uses underscore: `BTC_USDT` (not `BTCUSDT`).

---

### POST /api/v1/private/order/submit — Place Futures Order

**Rate Limit:** 20 times/2 seconds

#### Request JSON

```json
{
  "symbol": "BTC_USDT",
  "price": 8800,
  "vol": 100,
  "leverage": 20,
  "side": 1,
  "type": 1,
  "openType": 1,
  "externalOid": "order1",
  "stopLossPrice": 0,
  "takeProfitPrice": 0,
  "positionMode": 2,
  "reduceOnly": false
}
```

#### Request Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | e.g. `BTC_USDT` |
| `price` | decimal | YES | Order price |
| `vol` | decimal | YES | Order volume (contracts) |
| `leverage` | int | NO | Required for isolated margin |
| `side` | int | YES | `1`=open long, `2`=close short, `3`=open short, `4`=close long |
| `type` | int | YES | Order type (see below) |
| `openType` | int | YES | `1`=isolated, `2`=cross |
| `positionId` | long | NO | Recommended when closing |
| `externalOid` | string | NO | Client order ID |
| `stopLossPrice` | decimal | NO | Stop-loss trigger price |
| `takeProfitPrice` | decimal | NO | Take-profit trigger price |
| `positionMode` | int | NO | `1`=hedge, `2`=one-way |
| `reduceOnly` | boolean | NO | Default `false`; one-way positions only |

#### Order Types (Futures `type` field)

| Value | Description | Equivalent |
|-------|-------------|-----------|
| `1` | Limit order | LIMIT |
| `2` | Post Only Maker | LIMIT_MAKER |
| `3` | Transact or cancel immediately | IOC |
| `4` | Transact completely or cancel completely | FOK |
| `5` | Market order | MARKET |
| `6` | Convert market price to current price | Market-to-limit |

#### Response JSON

```json
{
  "success": true,
  "code": 0,
  "data": 102057569836905984
}
```

`data` is the numeric order ID (long integer, unlike Spot's UUID strings).

---

### POST /api/v1/private/order/submit_batch — Batch Futures Orders

**Max Orders:** 50 per request
**Rate Limit:** 20 times/2 seconds

Request is an array of the same order objects as single order submission.

---

### POST /api/v1/private/order/cancel — Cancel Futures Order(s)

#### Request

Array of order IDs (maximum 50):
```json
[101716841474621953, 101716841474621954]
```

#### Response JSON

```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "orderId": 101716841474621953,
      "errorCode": 0,
      "errorMsg": "success"
    }
  ]
}
```

---

### GET /api/v1/private/order/list/open_orders/{symbol} — Futures Open Orders

#### Response JSON

```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "orderId": 102015012431820288,
      "symbol": "ETH_USDT",
      "price": 1209.05,
      "vol": 1,
      "state": 3,
      "dealVol": 1
    }
  ]
}
```

---

### GET /api/v1/private/order/list/history_orders — Futures Order History

#### Full Order Object Fields

```json
{
  "orderId": "long",
  "symbol": "string",
  "positionId": "long",
  "price": "decimal",
  "vol": "decimal",
  "leverage": "long",
  "side": "int",
  "category": "int",
  "orderType": "int",
  "dealAvgPrice": "decimal",
  "dealVol": "decimal",
  "orderMargin": "decimal",
  "takerFee": "decimal",
  "makerFee": "decimal",
  "profit": "decimal",
  "feeCurrency": "string",
  "openType": "int",
  "state": "int",
  "externalOid": "string",
  "errorCode": "int",
  "usedMargin": "decimal",
  "createTime": "date",
  "updateTime": "date",
  "stopLossPrice": "decimal",
  "takeProfitPrice": "decimal"
}
```

**Futures Order `state` values:**
- `1` — Uninformed (pending)
- `2` — Uncompleted (partially filled / open)
- `3` — Completed (fully filled)
- `4` — Cancelled
- `5` — Invalid

---

## Differences: MEXC Spot vs Binance Spot

| Feature | MEXC Spot | Binance Spot |
|---------|-----------|--------------|
| Order ID type | UUID string (`"06a480e..."`) | Long integer (`1234567`) |
| `timeInForce` param | Not used; use order `type` | Separate enum (GTC/IOC/FOK) |
| OCO orders | Not supported | Supported |
| Stop-limit orders | Not supported | Supported (`STOP_LOSS_LIMIT`) |
| Modify order | Not supported | Not supported either |
| Batch orders | Yes, up to 20 same-symbol | Not supported on Spot |
| `isSelfTrade` field | Yes (MEXC addition) | No |
| `orderListId` | Always -1 | Used for OCO |
| Testnet | No (only `/order/test`) | Yes (testnet.binance.vision) |
| allOrders max range | 7 days | 24h default, no hard limit |
| myTrades max limit | 100 | 1000 |

---

## Sources

- MEXC Spot V3 API: https://mexcdevelop.github.io/apidocs/spot_v3_en/
- MEXC Contract V1 API: https://mexcdevelop.github.io/apidocs/contract_v1_en/
