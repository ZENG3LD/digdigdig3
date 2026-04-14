# KuCoin Trading API Specification

## CRITICAL ARCHITECTURE NOTE
KuCoin has **two completely separate trading systems** with different base URLs:
- **Spot**: `https://api.kucoin.com`
- **Futures**: `https://api-futures.kucoin.com`

KuCoin also has a **dual endpoint system for Spot**:
- **Classic** endpoints: `/api/v1/orders` (older, deprecated but still functional)
- **HF (High-Frequency)** endpoints: `/api/v1/hf/orders` (current recommended, lower latency)

The current `docs-new` documentation routes ALL classic spot order traffic through the HF endpoint (`POST /api/v1/hf/orders`). The old `/api/v1/orders` is listed under "Abandoned Endpoints" in the new docs, though it remains functional.

---

## 1. ORDER TYPES

### Spot (Classic `/api/v1/orders` and HF `/api/v1/hf/orders`)
| Type | Description |
|------|-------------|
| `limit` | Limit order — requires `price` and `size` |
| `market` | Market order — requires `size` OR `funds` (not both) |

Stop orders are placed on a **separate endpoint**: `POST /api/v1/stop-order`

### Futures (`POST https://api-futures.kucoin.com/api/v1/orders`)
| Type | Description |
|------|-------------|
| `limit` | Limit order — requires `price` and `size` |
| `market` | Market order — requires `size` (contracts) |

Stop orders embedded directly in the futures order via `stop`, `stopPrice`, `stopPriceType` fields.

### TimeInForce Values (Both Spot and Futures)
| Value | Meaning |
|-------|---------|
| `GTC` | Good-Till-Cancelled (default) |
| `GTT` | Good-Till-Time — use with `cancelAfter` (seconds) |
| `IOC` | Immediate-Or-Cancel |
| `FOK` | Fill-Or-Kill |

**Note**: `postOnly` is incompatible with `IOC` and `FOK`.

### Special Order Flags
| Flag | Type | Applies To | Description |
|------|------|-----------|-------------|
| `postOnly` | boolean | Spot + Futures | Maker-only; disabled with IOC/FOK |
| `hidden` | boolean | Spot + Futures | Hide order from order book |
| `iceberg` | boolean | Spot + Futures | Show only `visibleSize` in book |
| `visibleSize` | string | Spot + Futures | Max visible qty for iceberg orders |

### Self-Trade Prevention (STP)
| Value | Meaning |
|-------|---------|
| `CN` | Cancel Newest |
| `CO` | Cancel Oldest |
| `CB` | Cancel Both |
| `DC` | Decrease and Cancel |

---

## 2. ORDER MANAGEMENT

### 2a. Spot HF Orders (CURRENT RECOMMENDED SYSTEM)
Base URL: `https://api.kucoin.com`

#### Place HF Order
```
POST /api/v1/hf/orders
```
**Required Permission**: Spot Trading

**Request Body Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `clientOid` | string | No | Unique client order ID (UUID recommended, max 40 chars, alphanumeric + `_` + `-`) |
| `symbol` | string | Yes | Trading pair, e.g. `BTC-USDT` |
| `type` | string | Yes | `limit` or `market` |
| `side` | string | Yes | `buy` or `sell` |
| `stp` | string | No | Self-trade prevention: `CN`, `CO`, `CB`, `DC` |
| `tags` | string | No | Order tag (max 20 ASCII chars) |
| `remark` | string | No | Order remarks (max 20 ASCII chars) |

**Limit Order Additional Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `price` | string | Yes | Order price (must be multiple of tickSize) |
| `size` | string | Yes | Order quantity (must be multiple of baseIncrement) |
| `timeInForce` | string | No | `GTC` (default), `GTT`, `IOC`, `FOK` |
| `cancelAfter` | long | No | Cancel after N seconds (requires `GTT`) |
| `postOnly` | boolean | No | Passive order (maker-only) |
| `hidden` | boolean | No | Hide from order book |
| `iceberg` | boolean | No | Iceberg order |
| `visibleSize` | string | No | Visible quantity for iceberg |

**Market Order Additional Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `size` | string | Cond. | Quantity in base currency (use size OR funds) |
| `funds` | string | Cond. | Amount in quote currency (use size OR funds) |

**Response**:
```json
{
  "code": "200000",
  "data": {
    "orderId": "670fd33bf9406e0007ab3945",
    "clientOid": "5c52e11203aa677f33e493fb"
  }
}
```

#### Modify (Alter) HF Order
```
POST /api/v1/hf/orders/alter
```
**Note**: Internally cancels old order and creates new one at same trading pair. Returns `newOrderId`.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | Yes | Trading pair |
| `orderId` | string | Cond. | Order ID (use orderId OR clientOid) |
| `clientOid` | string | Cond. | Client order ID (use orderId OR clientOid) |
| `newPrice` | string | No | New order price |
| `newSize` | string | No | New order quantity |

**Response**:
```json
{
  "code": "200000",
  "data": {
    "newOrderId": "670fd33bf9406e0007ab3946",
    "clientOid": "5c52e11203aa677f33e493fb"
  }
}
```

#### Cancel HF Order by orderId
```
DELETE /api/v1/hf/orders/{orderId}?symbol={symbol}
```
- `symbol` query param is **required** for HF cancel

#### Cancel HF Order by clientOid
```
DELETE /api/v1/hf/orders/client-order/{clientOid}?symbol={symbol}
```

#### Cancel All HF Orders
```
DELETE /api/v1/hf/orders?symbol={symbol}
```

#### Batch Place HF Orders
```
POST /api/v1/hf/orders/multi
```
Array of order objects, max **5** per batch.

**Response**: Array of results per order:
```json
{
  "code": "200000",
  "data": [
    {
      "orderId": "670fd33bf9406e0007ab3945",
      "clientOid": "abc123",
      "success": true
    }
  ]
}
```

#### Batch Cancel HF Orders by orderIds
```
DELETE /api/v1/hf/orders/cancel?orderIds={id1},{id2}&symbol={symbol}
```

#### Get HF Order by orderId
```
GET /api/v1/hf/orders/{orderId}?symbol={symbol}
```

#### Get HF Active Orders (open)
```
GET /api/v1/hf/orders/active?symbol={symbol}
```

#### Get HF Order List (with pagination)
```
GET /api/v1/hf/orders?symbol={symbol}&side=buy&type=limit&startAt=&endAt=&lastId=&limit=100
```

#### Get HF Fills (Trade History)
```
GET /api/v1/hf/fills
```

---

### 2b. Spot Classic Orders (LEGACY — use HF instead)
Base URL: `https://api.kucoin.com`

**Note**: KuCoin's new `docs-new` documentation lists these under "Abandoned Endpoints". They remain functional but new integrations should use HF.

```
POST   /api/v1/orders              # Place order
DELETE /api/v1/orders/{orderId}    # Cancel by orderId
DELETE /api/v1/orders              # Cancel all (query: symbol, tradeType)
GET    /api/v1/orders/{orderId}    # Get order
GET    /api/v1/orders              # List orders
GET    /api/v1/fills               # Trade history
```

**Classic batch place** (max 5 orders):
```
POST /api/v1/orders/multi
```

Parameters are identical to HF endpoint except `remark` allows 50 ASCII chars in classic vs 20 in HF.

---

### 2c. Spot Stop Orders (Separate System)
Base URL: `https://api.kucoin.com`

Stop orders are **NOT** embedded in the regular order — they have their own endpoint and lifecycle.

#### Place Stop Order
```
POST /api/v1/stop-order
```
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `clientOid` | string | No | Unique client ID |
| `symbol` | string | Yes | Trading pair |
| `type` | string | Yes | `limit` or `market` |
| `side` | string | Yes | `buy` or `sell` |
| `price` | string | Cond. | Required for limit type |
| `stopPrice` | string | Yes | Trigger price |
| `stop` | string | No | Direction: `entry` (above) or `loss` (below) |
| `size` | string | Cond. | Required for limit; size OR funds for market |
| `funds` | string | Cond. | For market orders (alternative to size) |
| `timeInForce` | string | No | GTC, GTT, IOC, FOK |
| `postOnly` | boolean | No | Maker-only |
| `hidden` | boolean | No | Hide from book |
| `iceberg` | boolean | No | Iceberg |
| `visibleSize` | string | No | Visible size for iceberg |
| `remark` | string | No | Remarks (max 50 ASCII) |

**Constraints**: Max 20 untriggered stop orders per trading pair per account.

**Response**:
```json
{
  "code": "200000",
  "data": {
    "orderId": "670fd33bf9406e0007ab3945"
  }
}
```

Other stop order endpoints:
```
DELETE /api/v1/stop-order/{orderId}         # Cancel stop order
DELETE /api/v1/stop-order                   # Cancel all stop orders (query: symbol)
GET    /api/v1/stop-order/{orderId}         # Get by orderId
GET    /api/v1/stop-order/queryOrderByClientOid?clientOid=xxx  # Get by clientOid
GET    /api/v1/stop-order                   # List stop orders
```

---

### 2d. Futures Orders
Base URL: `https://api-futures.kucoin.com`

#### Place Futures Order
```
POST /api/v1/orders
```
**Required Permission**: Futures Trading

**Request Body Parameters**:
| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `clientOid` | string | No | Unique client order ID (max 40 chars) |
| `side` | string | Yes | `buy` or `sell` |
| `symbol` | string | Yes | Contract symbol, e.g. `XBTUSDTM` |
| `type` | string | Yes | `limit` or `market` |
| `leverage` | integer | No | Leverage multiplier (only effective for ISOLATED margin mode) |
| `price` | string | Cond. | Required for `limit` orders; must be multiple of `tickSize` |
| `size` | integer | Yes | Number of contracts (must be ≥1 and multiple of lot size) |
| `timeInForce` | string | No | `GTC` (default), `IOC`, `FOK` |
| `postOnly` | boolean | No | Maker-only; disabled with IOC/FOK |
| `hidden` | boolean | No | Hide from order book |
| `iceberg` | boolean | No | Iceberg order |
| `visibleSize` | integer | No | Max visible contracts for iceberg |
| `stop` | string | No | Stop direction: `up` (triggers when price ≥ stopPrice) or `down` (triggers when price ≤ stopPrice) |
| `stopPrice` | string | Cond. | Required if `stop` is set |
| `stopPriceType` | string | Cond. | Required if `stop` is set: `TP` (last trade price), `MP` (mark price), `IP` (index price) |
| `reduceOnly` | boolean | No | If true, only reduces position (no funds frozen) |
| `closeOrder` | boolean | No | If true, closes position at market (no funds frozen) |
| `forceHold` | boolean | No | Force freeze funds even for reduce-only scenarios |
| `marginMode` | string | No | `ISOLATED` (default) or `CROSS` |
| `remark` | string | No | Order remarks (max 100 chars) |

**Response**:
```json
{
  "code": "200000",
  "data": {
    "orderId": "234125150956625920",
    "clientOid": "5c52e11203aa677f33e493fb"
  }
}
```

**Rate Limit Weight**: 2

#### Futures Cancel Order by orderId
```
DELETE /api/v1/orders/{orderId}
```

#### Futures Cancel Order by clientOid
```
DELETE /api/v1/orders/client-order/{clientOid}
```

#### Futures Cancel All Orders
```
DELETE /api/v1/orders?symbol=XBTUSDTM
```

#### Futures Batch Cancel Orders
```
DELETE /api/v1/orders/multi?orderIds={id1},{id2}
```

#### Futures Get Order by orderId/clientOid
```
GET /api/v1/orders/{orderId-or-clientOid}
```

#### Futures Get Active Orders
```
GET /api/v1/openOrders?symbol=XBTUSDTM
```

#### Futures Get Order List
```
GET /api/v1/orders?status=active&symbol=XBTUSDTM&side=buy&type=limit&startAt=&endAt=&currentPage=1&pageSize=50
```

#### Futures Get Fills
```
GET /api/v1/fills
```

#### Futures Get Untriggered Stop Orders
```
GET /api/v1/stopOrders?symbol=XBTUSDTM
```

---

## 3. TP/SL & CONDITIONAL ORDERS

### Spot Stop Orders
See Section 2c above — use `POST /api/v1/stop-order` with `stop: "entry"` (take profit) or `stop: "loss"` (stop loss).

### Futures Embedded Stop Orders
Attach `stop`, `stopPrice`, `stopPriceType` directly to a regular futures order (Section 2d above).

`stopPriceType` values:
- `TP` — Last trade price
- `MP` — Mark price
- `IP` — Index price

### Futures Dedicated TP/SL Order (Combined)
```
POST https://api-futures.kucoin.com/api/v1/st-orders
```
Allows placing a single order that sets both a take-profit AND stop-loss trigger simultaneously.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `clientOid` | string | No | Unique client ID |
| `side` | string | Yes | `buy` or `sell` |
| `symbol` | string | Yes | Contract symbol |
| `leverage` | integer | No | Leverage |
| `type` | string | Yes | `limit` or `market` |
| `price` | string | Cond. | For limit orders |
| `size` | integer | Yes | Number of contracts |
| `timeInForce` | string | No | `GTC`, etc. |
| `triggerStopUpPrice` | string | No | Take profit trigger price |
| `triggerStopDownPrice` | string | No | Stop loss trigger price |
| `stopPriceType` | string | Cond. | `TP`, `MP`, or `IP` (required if trigger prices set) |
| `reduceOnly` | boolean | No | Reduce-only |
| `marginMode` | string | No | `ISOLATED` or `CROSS` |
| `positionSide` | string | No | `BOTH`, `LONG`, or `SHORT` |
| `remark` | string | No | Remarks |

**Response**:
```json
{
  "code": "200000",
  "data": {
    "orderId": "234125150956625920",
    "clientOid": "5c52e11203aa677f33e493fb"
  }
}
```
**Rate Limit Weight**: 2

---

## 4. BATCH OPERATIONS

### Spot HF Batch Place
```
POST /api/v1/hf/orders/multi
```
- Max **5** orders per request
- Same parameters as single HF order
- Each order in `orderList` array

### Spot Classic Batch Place (Deprecated)
```
POST /api/v1/orders/multi
```
- Max **5** orders per request

### Futures Batch Place
```
POST https://api-futures.kucoin.com/api/v1/orders/multi
```
- Max **limit orders per contract**: 100 per account
- Max **stop orders per contract**: 50 per account
- Array of order objects with same parameters as single futures order

**Futures Batch Cancel**:
```
DELETE https://api-futures.kucoin.com/api/v1/orders/multi?orderIds={id1},{id2}
```

---

## 5. ALGO ORDERS

KuCoin does **not** have a native algo order system comparable to Bybit's TWAP or OKX's Algo endpoints. The available conditional order mechanisms are:

1. **Spot stop orders** via `POST /api/v1/stop-order` (triggered by last price only)
2. **Futures embedded stops** via `stop`, `stopPrice`, `stopPriceType` on regular order (TP/MP/IP trigger)
3. **Futures dedicated TP/SL** via `POST /api/v1/st-orders` (combined take-profit + stop-loss)
4. **GTT orders** (Good-Till-Time) — cancel after N seconds

---

## 6. ORDER RESPONSE FORMAT

### Spot (Both Classic and HF) — Minimal Response on Create
```json
{
  "code": "200000",
  "data": {
    "orderId": "670fd33bf9406e0007ab3945",
    "clientOid": "5c52e11203aa677f33e493fb"
  }
}
```
- `orderId`: String, KuCoin-generated unique ID (24-character hex string)
- `clientOid`: String, echoed from request (or empty if not provided)

### Futures — Minimal Response on Create
```json
{
  "code": "200000",
  "data": {
    "orderId": "234125150956625920",
    "clientOid": "5c52e11203aa677f33e493fb"
  }
}
```
- `orderId`: String, numeric ID for futures (vs hex for spot)

### Full Order Detail Response (GET /api/v1/hf/orders/{orderId})
```json
{
  "code": "200000",
  "data": {
    "id": "670fd33bf9406e0007ab3945",
    "clientOid": "5c52e11203aa677f33e493fb",
    "symbol": "BTC-USDT",
    "type": "limit",
    "side": "buy",
    "price": "50000",
    "size": "0.001",
    "funds": "0",
    "dealFunds": "0",
    "dealSize": "0",
    "fee": "0",
    "feeCurrency": "USDT",
    "stp": "",
    "stop": "",
    "stopTriggered": false,
    "stopPrice": "0",
    "timeInForce": "GTC",
    "postOnly": false,
    "hidden": false,
    "iceberg": false,
    "visibleSize": "0",
    "cancelAfter": 0,
    "channel": "API",
    "remark": "",
    "tags": "",
    "isActive": true,
    "cancelExist": false,
    "createdAt": 1705123456789,
    "tradeType": "TRADE"
  }
}
```

### Full Futures Order Detail Response
```json
{
  "code": "200000",
  "data": {
    "id": "234125150956625920",
    "symbol": "XBTUSDTM",
    "type": "limit",
    "side": "buy",
    "price": "50000",
    "size": 1,
    "value": "0.002",
    "dealValue": "0",
    "dealSize": 0,
    "stp": "",
    "stop": "",
    "stopPriceType": "",
    "stopTriggered": false,
    "stopPrice": null,
    "timeInForce": "GTC",
    "postOnly": false,
    "hidden": false,
    "iceberg": false,
    "leverage": "10",
    "forceHold": false,
    "closeOrder": false,
    "visibleSize": null,
    "clientOid": "5c52e11203aa677f33e493fb",
    "remark": null,
    "tags": null,
    "isActive": true,
    "cancelExist": false,
    "createdAt": 1705123456789,
    "updatedAt": 1705123456789,
    "orderTime": 1705123456789000,
    "settleCurrency": "USDT",
    "marginMode": "ISOLATED",
    "filledSize": 0,
    "filledValue": "0",
    "status": "open",
    "reduceOnly": false
  }
}
```

---

## 7. COMPLETE ENDPOINT REFERENCE TABLE

### Spot HF (Current System)
| Method | Path | Description | Permission |
|--------|------|-------------|-----------|
| POST | `/api/v1/hf/orders` | Place order | Spot |
| POST | `/api/v1/hf/orders/alter` | Modify order | Spot |
| POST | `/api/v1/hf/orders/multi` | Batch place (max 5) | Spot |
| DELETE | `/api/v1/hf/orders/{orderId}` | Cancel by orderId | Spot |
| DELETE | `/api/v1/hf/orders/client-order/{clientOid}` | Cancel by clientOid | Spot |
| DELETE | `/api/v1/hf/orders` | Cancel all | Spot |
| DELETE | `/api/v1/hf/orders/cancel` | Batch cancel by IDs | Spot |
| GET | `/api/v1/hf/orders/{orderId}` | Get by orderId | General |
| GET | `/api/v1/hf/orders/active` | Get active orders | General |
| GET | `/api/v1/hf/orders` | List orders | General |
| GET | `/api/v1/hf/fills` | Trade fills | General |

### Spot Stop Orders
| Method | Path | Description | Permission |
|--------|------|-------------|-----------|
| POST | `/api/v1/stop-order` | Place stop order | Spot |
| DELETE | `/api/v1/stop-order/{orderId}` | Cancel stop | Spot |
| DELETE | `/api/v1/stop-order` | Cancel all stops | Spot |
| GET | `/api/v1/stop-order/{orderId}` | Get stop order | General |
| GET | `/api/v1/stop-order` | List stop orders | General |

### Spot Classic (Legacy)
| Method | Path | Description | Permission |
|--------|------|-------------|-----------|
| POST | `/api/v1/orders` | Place order | Spot |
| POST | `/api/v1/orders/multi` | Batch place (max 5) | Spot |
| DELETE | `/api/v1/orders/{orderId}` | Cancel | Spot |
| DELETE | `/api/v1/orders` | Cancel all | Spot |
| GET | `/api/v1/orders/{orderId}` | Get order | General |
| GET | `/api/v1/orders` | List orders | General |
| GET | `/api/v1/fills` | Trade fills | General |

### Futures (Base: `https://api-futures.kucoin.com`)
| Method | Path | Description | Permission |
|--------|------|-------------|-----------|
| POST | `/api/v1/orders` | Place order | Futures |
| POST | `/api/v1/orders/multi` | Batch place | Futures |
| POST | `/api/v1/st-orders` | Place TP+SL order | Futures |
| DELETE | `/api/v1/orders/{orderId}` | Cancel by orderId | Futures |
| DELETE | `/api/v1/orders/client-order/{clientOid}` | Cancel by clientOid | Futures |
| DELETE | `/api/v1/orders` | Cancel all | Futures |
| DELETE | `/api/v1/orders/multi` | Batch cancel | Futures |
| GET | `/api/v1/orders/{orderId}` | Get order | General |
| GET | `/api/v1/openOrders` | Active orders | General |
| GET | `/api/v1/orders` | Order history | General |
| GET | `/api/v1/fills` | Trade fills | General |
| GET | `/api/v1/stopOrders` | Untriggered stops | General |

---

## Sources
- [KuCoin Add Order (Spot HF) - docs-new](https://www.kucoin.com/docs-new/rest/spot-trading/orders/add-order)
- [KuCoin Add Order (Futures) - docs-new](https://www.kucoin.com/docs-new/rest/futures-trading/orders/add-order)
- [KuCoin Add Stop Order (Spot)](https://www.kucoin.com/docs-new/rest/spot-trading/orders/add-stop-order)
- [KuCoin Add TP/SL Order (Futures)](https://www.kucoin.com/docs-new/rest/futures-trading/orders/add-take-profit-and-stop-loss-order)
- [KuCoin Modify Order (Spot HF)](https://www.kucoin.com/docs-new/rest/spot-trading/orders/modify-order)
- [KuCoin Batch Add Orders (Futures)](https://www.kucoin.com/docs-new/rest/futures-trading/orders/batch-add-orders)
- [KuCoin Place Order docs (legacy)](https://www.kucoin.com/docs/rest/spot-trading/orders/place-order)
- [KuCoin Futures Place Order docs (legacy)](https://www.kucoin.com/docs/rest/futures-trading/orders/place-order)
- [KuCoin Place Multiple Orders](https://www.kucoin.com/docs/rest/spot-trading/orders/place-multiple-orders)
- [KuCoin Place HF Order (legacy docs)](https://www.kucoin.com/docs/rest/spot-trading/spot-hf-trade-pro-account/place-hf-order)
