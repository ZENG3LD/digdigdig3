# Binance Trading API — Complete Reference

Sources: Binance Spot REST API docs, Binance USDM Futures REST API docs (developers.binance.com)

---

## 1. ORDER TYPES

### Spot Order Types (`POST /api/v3/order`, param `type=`)

| API Value | Description | Required Additional Params |
|-----------|-------------|---------------------------|
| `LIMIT` | Limit order | `timeInForce`, `quantity`, `price` |
| `MARKET` | Market order | `quantity` OR `quoteOrderQty` |
| `STOP_LOSS` | Stop-loss market (triggers market order at stopPrice) | `quantity`, `stopPrice` OR `trailingDelta` |
| `STOP_LOSS_LIMIT` | Stop-limit order | `timeInForce`, `quantity`, `price`, `stopPrice` OR `trailingDelta` |
| `TAKE_PROFIT` | Take-profit market (triggers market order at stopPrice) | `quantity`, `stopPrice` OR `trailingDelta` |
| `TAKE_PROFIT_LIMIT` | Take-profit limit order | `timeInForce`, `quantity`, `price`, `stopPrice` OR `trailingDelta` |
| `LIMIT_MAKER` | Limit order that is rejected if it would be a taker (PostOnly) | `quantity`, `price` |

**Trailing stop (Spot):** Use `trailingDelta` instead of `stopPrice`. Value is in BIPS (1/10000 of a percent). This applies to STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT, TAKE_PROFIT_LIMIT types.

**Iceberg orders (Spot):** Use `icebergQty` param on LIMIT or LIMIT_MAKER orders. Must have `timeInForce=GTC`.

**LIMIT_MAKER = PostOnly:** An order that will be rejected if it would immediately match and trade as a taker. This is Binance Spot's PostOnly equivalent.

**Pegged orders (Spot, newer):** Use `pegPriceType` (`PRIMARY_PEG` or `MARKET_PEG`) and `pegOffsetValue` (INT, max 100) and `pegOffsetType` (`PRICE_LEVEL`). These are advanced order types.

---

### Futures (USDM) Order Types (`POST /fapi/v1/order`, param `type=`)

| API Value | Description | Required Additional Params |
|-----------|-------------|---------------------------|
| `LIMIT` | Limit order | `timeInForce`, `quantity`, `price` |
| `MARKET` | Market order | `quantity` |
| `STOP` | Stop-limit order (trigger → limit) | `quantity`, `price`, `stopPrice` |
| `STOP_MARKET` | Stop-market order (trigger → market) | `quantity`, `stopPrice` |
| `TAKE_PROFIT` | Take-profit limit order | `quantity`, `price`, `stopPrice` |
| `TAKE_PROFIT_MARKET` | Take-profit market order | `quantity`, `stopPrice` |
| `TRAILING_STOP_MARKET` | Trailing stop market | `quantity`, `callbackRate` (0.1–10) OR `activationPrice` + `callbackRate` |
| `LIQUIDATION` | Forced liquidation order (read-only, exchange-generated) | N/A |

**Futures has no native ICO/trailing-stop-limit combo — TRAILING_STOP_MARKET is the trailing type.**

**Futures `priceMatch` param:** LIMIT and STOP orders support `priceMatch` (OPPONENT, OPPONENT_5, OPPONENT_10, OPPONENT_20, QUEUE, QUEUE_5, QUEUE_10, QUEUE_20) as an alternative to setting explicit price.

**Futures `closePosition`:** STRING param `"true"` on STOP_MARKET or TAKE_PROFIT_MARKET closes the entire position automatically. Incompatible with `quantity`.

**Futures `reduceOnly`:** STRING `"true"` — order only reduces existing position, never opens new one.

---

### TimeInForce Options

| Value | Name | Spot | Futures | Description |
|-------|------|------|---------|-------------|
| `GTC` | Good Till Cancel | Yes | Yes | Active until executed or manually canceled |
| `IOC` | Immediate Or Cancel | Yes | Yes | Execute any amount immediately, cancel remainder |
| `FOK` | Fill Or Kill | Yes | Yes | Execute full amount immediately or cancel entirely |
| `GTX` | Good Till Crossing (PostOnly) | No | Yes | Futures only: passive order, cancels if it would match immediately (maker-only) |
| `GTD` | Good Till Date | No | Yes | Futures only: canceled at `goodTillDate` timestamp. Requires `goodTillDate` param |

**Note:** `GTX` is Binance Futures PostOnly equivalent. Spot PostOnly is via `type=LIMIT_MAKER`.

---

## 2. ORDER MANAGEMENT ENDPOINTS

### Full Endpoint Table

| Action | Method | Spot Endpoint | Futures Endpoint | Auth | Spot Weight | Futures Weight |
|--------|--------|---------------|-----------------|------|-------------|----------------|
| Create order | POST | `/api/v3/order` | `/fapi/v1/order` | TRADE | 1 | 0 IP / 1 order-count |
| Test order (no execution) | POST | `/api/v3/order/test` | N/A | TRADE | 1 or 20 | N/A |
| Amend order (keep priority) | PUT | `/api/v3/order/amend/keepPriority` | N/A | TRADE | 4 | N/A |
| Modify order | PUT | N/A | `/fapi/v1/order` | TRADE | N/A | 0 IP / 1 order-count |
| Cancel order | DELETE | `/api/v3/order` | `/fapi/v1/order` | TRADE | 1 | 1 |
| Cancel all orders on symbol | DELETE | `/api/v3/openOrders` | `/fapi/v1/allOpenOrders` | TRADE | 1 | 1 |
| Cancel + replace order | POST | `/api/v3/order/cancelReplace` | N/A | TRADE | 1 | N/A |
| Batch create orders | POST | N/A | `/fapi/v1/batchOrders` | TRADE | N/A | 5 IP / 5 order-count |
| Batch modify orders | PUT | N/A | `/fapi/v1/batchOrders` | TRADE | N/A | 5 IP / 5 order-count |
| Batch cancel orders | DELETE | N/A | `/fapi/v1/batchOrders` | TRADE | N/A | 1 |
| Get single order | GET | `/api/v3/order` | `/fapi/v1/order` | USER_DATA | 4 | 1 |
| Get open orders | GET | `/api/v3/openOrders` | `/fapi/v1/openOrders` | USER_DATA | 6 (1 symbol) / 80 (all) | 1 (1 symbol) / 40 (all) |
| Get all orders (history) | GET | `/api/v3/allOrders` | `/fapi/v1/allOrders` | USER_DATA | 20 | 5 |
| Get my trades / fills | GET | `/api/v3/myTrades` | `/fapi/v1/userTrades` | USER_DATA | 20 (no orderId) / 5 (with orderId) | 5 |
| Auto-cancel countdown | POST | N/A | `/fapi/v1/countdownCancelAll` | TRADE | N/A | 10 |

---

### Create Order — Spot (`POST /api/v3/order`)

**All Parameters:**

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | e.g. `BTCUSDT` |
| `side` | ENUM | YES | `BUY` or `SELL` |
| `type` | ENUM | YES | See order types above |
| `timeInForce` | ENUM | Conditional | Required for LIMIT, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT |
| `quantity` | DECIMAL | Conditional | Required for most types; MARKET can use `quoteOrderQty` instead |
| `quoteOrderQty` | DECIMAL | Conditional | MARKET only: buy/sell `quoteOrderQty` worth of `symbol` |
| `price` | DECIMAL | Conditional | Required for LIMIT, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT |
| `newClientOrderId` | STRING | NO | Custom order ID (max 36 chars); auto-generated if omitted |
| `strategyId` | LONG | NO | Arbitrary strategy ID |
| `strategyType` | INT | NO | Arbitrary strategy type (min value: 1000000) |
| `stopPrice` | DECIMAL | Conditional | Required for STOP_LOSS*, TAKE_PROFIT* (unless using trailingDelta) |
| `trailingDelta` | LONG | Conditional | In BIPS; alternative to stopPrice for trailing |
| `icebergQty` | DECIMAL | NO | Split order into visible + hidden quantity; requires `timeInForce=GTC` |
| `newOrderRespType` | ENUM | NO | `ACK`, `RESULT`, or `FULL` (default LIMIT=FULL, MARKET=FULL, others=ACK) |
| `selfTradePreventionMode` | ENUM | NO | `NONE`, `EXPIRE_MAKER`, `EXPIRE_TAKER`, `EXPIRE_BOTH` |
| `pegPriceType` | ENUM | NO | `PRIMARY_PEG` or `MARKET_PEG` |
| `pegOffsetValue` | INT | NO | Max 100 |
| `pegOffsetType` | ENUM | NO | `PRICE_LEVEL` |
| `recvWindow` | LONG | NO | Max 60000 ms |
| `timestamp` | LONG | YES | Current time in ms |

---

### Create Order — Futures (`POST /fapi/v1/order`)

**All Parameters:**

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | e.g. `BTCUSDT` |
| `side` | ENUM | YES | `BUY` or `SELL` |
| `positionSide` | ENUM | NO | `BOTH` (One-way mode, default), `LONG`, `SHORT` (Hedge mode only) |
| `type` | ENUM | YES | See Futures order types above |
| `timeInForce` | ENUM | Conditional | Required for LIMIT, STOP, TAKE_PROFIT |
| `quantity` | DECIMAL | Conditional | Required unless `closePosition=true` |
| `reduceOnly` | STRING | NO | `"true"` or `"false"` (default `"false"`); N/A in Hedge mode |
| `price` | DECIMAL | Conditional | Required for LIMIT, STOP, TAKE_PROFIT |
| `newClientOrderId` | STRING | NO | Max 36 chars |
| `stopPrice` | DECIMAL | Conditional | Required for STOP*, TAKE_PROFIT*, TRAILING_STOP_MARKET |
| `closePosition` | STRING | NO | `"true"` for STOP_MARKET/TAKE_PROFIT_MARKET to close full position |
| `activationPrice` | DECIMAL | Conditional | For TRAILING_STOP_MARKET; defaults to current price if omitted |
| `callbackRate` | DECIMAL | Conditional | For TRAILING_STOP_MARKET; range 0.1–10 (percent) |
| `workingType` | ENUM | NO | `MARK_PRICE` (default for most) or `CONTRACT_PRICE` — what price triggers stopPrice |
| `priceProtect` | STRING | NO | `"TRUE"` or `"FALSE"` — trigger price protection for stop orders |
| `newOrderRespType` | ENUM | NO | `ACK` (default) or `RESULT` |
| `priceMatch` | ENUM | NO | Alternative to explicit price; one of OPPONENT, OPPONENT_5..20, QUEUE, QUEUE_5..20 |
| `selfTradePreventionMode` | ENUM | NO | `EXPIRE_TAKER`, `EXPIRE_MAKER`, `EXPIRE_BOTH` (default EXPIRE_MAKER) |
| `goodTillDate` | LONG | Conditional | Required when `timeInForce=GTD`; cancellation timestamp in ms |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

---

### Modify Order — Futures Only (`PUT /fapi/v1/order`)

Spot does NOT support modifying a live order by changing price/quantity (only `amend/keepPriority` which can only reduce quantity while keeping queue priority).

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `side` | ENUM | YES | |
| `orderId` | LONG | Conditional | Either this or `origClientOrderId` |
| `origClientOrderId` | STRING | Conditional | |
| `quantity` | DECIMAL | YES | New quantity |
| `price` | DECIMAL | YES | New price |
| `priceMatch` | ENUM | NO | Alternative to price |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

**Constraints:**
- Both `quantity` and `price` must be sent (or use `priceMatch` instead of price).
- One order can be modified less than 10,000 times total.
- If new quantity <= executedQty (partially filled), order is CANCELED.
- If order is GTX and new price causes immediate execution, order is CANCELED.
- Weight: 0 IP / 1 order-count-10s / 1 order-count-1m.

---

### Spot Amend (Keep Queue Priority) (`PUT /api/v3/order/amend/keepPriority`)

Can only **reduce** quantity. Does NOT allow price changes. Keeps queue position.

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `newQty` | DECIMAL | YES | Must be > 0 and < current quantity |
| `orderId` | LONG | Conditional | Either this or `origClientOrderId` |
| `origClientOrderId` | STRING | Conditional | |
| `newClientOrderId` | STRING | NO | |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

Response:
```json
{
  "transactTime": 1741926410255,
  "executionId": 75,
  "amendedOrder": {
    "symbol": "BTCUSDT",
    "orderId": 33,
    "price": "6.00000000",
    "qty": "5.00000000",
    "status": "NEW"
  }
}
```

---

### Cancel Order — Spot (`DELETE /api/v3/order`)

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `orderId` | LONG | Conditional | Either this or `origClientOrderId` |
| `origClientOrderId` | STRING | Conditional | |
| `newClientOrderId` | STRING | NO | ID for the cancellation |
| `cancelRestrictions` | ENUM | NO | `ONLY_NEW` or `ONLY_PARTIALLY_FILLED` |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

Response (weight: 1):
```json
{
  "symbol": "LTCBTC",
  "origClientOrderId": "myOrder1",
  "orderId": 4,
  "orderListId": -1,
  "clientOrderId": "cancelMyOrder1",
  "transactTime": 1684804350068,
  "price": "2.00000000",
  "origQty": "1.00000000",
  "executedQty": "0.00000000",
  "cummulativeQuoteQty": "0.00000000",
  "status": "CANCELED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY",
  "selfTradePreventionMode": "NONE"
}
```

---

### Cancel and Replace Order — Spot Only (`POST /api/v3/order/cancelReplace`)

Atomically cancel one order and place a new one. **Futures has no equivalent — must cancel then create separately.**

| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `side` | ENUM | YES | |
| `type` | ENUM | YES | |
| `cancelReplaceMode` | ENUM | YES | `STOP_ON_FAILURE` (if cancel fails, do not place new) or `ALLOW_FAILURE` (place new regardless) |
| `cancelOrderId` | LONG | Conditional | Cancel target: either this or `cancelOrigClientOrderId` |
| `cancelOrigClientOrderId` | STRING | Conditional | |
| `cancelNewClientOrderId` | STRING | NO | |
| `cancelRestrictions` | ENUM | NO | `ONLY_NEW` or `ONLY_PARTIALLY_FILLED` |
| `orderRateLimitExceededMode` | ENUM | NO | `DO_NOTHING` or `CANCEL_ONLY` |
| + all new order params | | | Same as `POST /api/v3/order` |
| `timestamp` | LONG | YES | |

Response (weight: 1, order-count: 1):
```json
{
  "cancelResult": "SUCCESS",
  "newOrderResult": "SUCCESS",
  "cancelResponse": { "...order fields..." },
  "newOrderResponse": { "...order fields..." }
}
```

Possible `cancelResult`/`newOrderResult` values: `SUCCESS`, `FAILURE`, `NOT_ATTEMPTED`.

---

## 3. TP/SL AND CONDITIONAL ORDERS

### Spot TP/SL Implementation

Spot has **no dedicated TP/SL attached to an existing order**. TP/SL are placed as **separate independent orders**:
- `STOP_LOSS` / `STOP_LOSS_LIMIT` — stops out loss position.
- `TAKE_PROFIT` / `TAKE_PROFIT_LIMIT` — locks in profit.
- **OCO** (`POST /api/v3/orderList/oco`) — places a LIMIT_MAKER + STOP_LOSS or STOP_LOSS_LIMIT pair; when one fills, the other is canceled.
- TP/SL orders are independent once placed; they cannot be "attached" to another order.
- After placing TP/SL, they must be managed and canceled manually if the entry order is canceled.

**Spot Stop Price Rules:**
- STOP_LOSS BUY: `stopPrice` must be ABOVE current market price.
- STOP_LOSS SELL: `stopPrice` must be BELOW current market price.
- TAKE_PROFIT BUY: `stopPrice` must be BELOW current market price.
- TAKE_PROFIT SELL: `stopPrice` must be ABOVE current market price.

---

### Futures TP/SL Implementation

Futures also has **no single-call TP+SL on entry**. Each is a separate order:
- Place entry LIMIT or MARKET order.
- Place `STOP_MARKET` (with `reduceOnly=true`) for stop-loss.
- Place `TAKE_PROFIT_MARKET` (with `reduceOnly=true`) for take-profit.
- Both use `stopPrice` as trigger.
- `workingType` controls what price triggers the stop: `MARK_PRICE` (default, anti-manipulation) or `CONTRACT_PRICE` (last traded price).

**Futures `closePosition=true`:** A single `STOP_MARKET` or `TAKE_PROFIT_MARKET` order with `closePosition=true` will automatically close the full position at trigger. No need to specify `quantity`.

**Modifying TP/SL:** Cancel existing order (`DELETE /fapi/v1/order`) and place a new one. The `PUT /fapi/v1/order` endpoint can also modify the price/qty of an existing stop/take-profit order.

**Note:** Binance Futures does NOT support bracket orders (entry + TP + SL in one call). Each leg is a separate API call.

---

### OCO (One-Cancels-Other) — Spot

The new unified OCO endpoint is `POST /api/v3/orderList/oco` (the old `POST /api/v3/order/oco` is deprecated).

**Parameters:**
| Parameter | Type | Required | Notes |
|-----------|------|----------|-------|
| `symbol` | STRING | YES | |
| `side` | ENUM | YES | `BUY` or `SELL` |
| `quantity` | DECIMAL | YES | Same quantity for both legs |
| `aboveType` | ENUM | YES | Order type for the above-price leg: `STOP_LOSS_LIMIT`, `LIMIT_MAKER`, `STOP_LOSS`, `TAKE_PROFIT`, `TAKE_PROFIT_LIMIT` |
| `belowType` | ENUM | YES | Order type for the below-price leg |
| `abovePrice` | DECIMAL | Conditional | Limit price for above leg (if limit type) |
| `aboveStopPrice` | DECIMAL | Conditional | Stop trigger for above leg |
| `aboveTimeInForce` | ENUM | Conditional | Required if above type is STOP_LOSS_LIMIT/TAKE_PROFIT_LIMIT |
| `aboveClientOrderId` | STRING | NO | |
| `aboveTrailingDelta` | LONG | NO | |
| `aboveIcebergQty` | DECIMAL | NO | |
| `belowPrice` | DECIMAL | Conditional | |
| `belowStopPrice` | DECIMAL | Conditional | |
| `belowTimeInForce` | ENUM | Conditional | |
| `belowClientOrderId` | STRING | NO | |
| `belowTrailingDelta` | LONG | NO | |
| `belowIcebergQty` | DECIMAL | NO | |
| `listClientOrderId` | STRING | NO | ID for the order list |
| `newOrderRespType` | ENUM | NO | |
| `selfTradePreventionMode` | ENUM | NO | |
| `recvWindow` | LONG | NO | |
| `timestamp` | LONG | YES | |

Weight: 1, order-count: 2.

**OCO Note:** Both legs share the same `quantity`. When one order is triggered/filled, the other is automatically canceled.

**Futures has NO OCO.** Binance Futures does not natively support OCO orders.

---

### OTO (One-Triggers-Other) — Spot Only

`POST /api/v3/orderList/oto` — when the working order fills, the pending order is automatically placed.

Weight: 1, order-count: 2.

### OTOCO (One-Triggers-OCO) — Spot Only

`POST /api/v3/orderList/otoco` — working order fills → triggers an OCO pair.

Weight: 1, order-count: 3.

### OPO / OPOCO — Spot Only

`POST /api/v3/orderList/opo`, `POST /api/v3/orderList/opoco` — pending order variants.

Weight: 1, order-count: 2/3.

**Futures has NONE of these order list types.**

---

## 4. BATCH OPERATIONS

### Futures Batch Create (`POST /fapi/v1/batchOrders`)

- Max **5 orders** per batch.
- Pass as `batchOrders` parameter — a JSON-encoded list of order objects.
- Each order object has same params as single `POST /fapi/v1/order`.
- Orders processed **concurrently** (matching order not guaranteed).
- Response: array of individual order responses (or error per failed order).
- Weight: 5 IP / 5 order-count-10s / 1 order-count-1m.

### Futures Batch Modify (`PUT /fapi/v1/batchOrders`)

- Max **5 orders** per batch.
- Same rules as single `PUT /fapi/v1/order`.
- Weight: 5 IP / 5 order-count-10s / 1 order-count-1m.

### Futures Batch Cancel (`DELETE /fapi/v1/batchOrders`)

- Pass `orderIdList` (JSON array of order IDs) or `origClientOrderIdList`.
- Max **10 orders** per batch.
- Weight: 1.

### Spot Batch Operations

- **NO native batch create endpoint for Spot.**
- **NO native batch modify for Spot.**
- **Cancel all on symbol:** `DELETE /api/v3/openOrders` (symbol required) — weight 1.
- Individual cancel only: `DELETE /api/v3/order`.

---

## 5. ALGO ORDERS (NATIVE)

### Native on Binance Exchange

- **TWAP (Spot/Futures):** Binance Broker ALGO API at `https://api.binance.com/sapi/v1/algo/spot/newOrderTwap` and `/sapi/v1/algo/futures/newOrderTwap`. These are not part of the regular REST API and require broker permissions. **NOT available to regular API users.**
- **Grid Trading:** Available in Binance app/web UI but **no public API endpoint** for algorithmic grid orders.
- **DCA (Dollar Cost Averaging):** **No native API endpoint.**
- **VP (Volume Participation):** Binance Broker ALGO API. Not publicly available.

### Iceberg Orders (Standard API)

Available via regular spot/futures API using `icebergQty` parameter:
- **Spot:** Add `icebergQty` to any LIMIT or LIMIT_MAKER order. Must use `timeInForce=GTC`. The visible quantity in the order book will be `icebergQty`; the rest is hidden. Total size is `quantity`.
- **Futures:** No native iceberg orders. `icebergQty` is NOT supported on `POST /fapi/v1/order`.

### Trailing Stop (Native)

- **Spot:** Use `trailingDelta` (BIPS) on STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT, TAKE_PROFIT_LIMIT. Not a separate order type; it's a parameter variant.
- **Futures:** `TRAILING_STOP_MARKET` is a dedicated order type. Use `callbackRate` (percent, 0.1–10%) and optionally `activationPrice`.

---

## 6. ORDER RESPONSE FORMATS

### Spot — Create Order (`POST /api/v3/order`)

**ACK Response** (minimal, weight 1):
```json
{
  "symbol": "BTCUSDT",
  "orderId": 28,
  "orderListId": -1,
  "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
  "transactTime": 1507725176595
}
```

**RESULT Response:**
```json
{
  "symbol": "BTCUSDT",
  "orderId": 28,
  "orderListId": -1,
  "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
  "transactTime": 1507725176595,
  "price": "0.00000000",
  "origQty": "10.00000000",
  "executedQty": "10.00000000",
  "cummulativeQuoteQty": "10.00000000",
  "status": "FILLED",
  "timeInForce": "GTC",
  "type": "MARKET",
  "side": "SELL",
  "workingTime": 1507725176595,
  "selfTradePreventionMode": "NONE"
}
```

**FULL Response** (includes fills array):
```json
{
  "symbol": "BTCUSDT",
  "orderId": 28,
  "orderListId": -1,
  "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
  "transactTime": 1507725176595,
  "price": "0.00000000",
  "origQty": "10.00000000",
  "executedQty": "10.00000000",
  "cummulativeQuoteQty": "10.00000000",
  "status": "FILLED",
  "timeInForce": "GTC",
  "type": "MARKET",
  "side": "SELL",
  "workingTime": 1507725176595,
  "selfTradePreventionMode": "NONE",
  "fills": [
    {
      "price": "4000.00000000",
      "qty": "1.00000000",
      "commission": "4.00000000",
      "commissionAsset": "USDT",
      "tradeId": 56
    }
  ]
}
```

**Conditional response fields** (present depending on order type):
- `stopPrice` — for STOP_LOSS*, TAKE_PROFIT* orders
- `icebergQty` — for iceberg orders
- `trailingDelta`, `trailingTime` — for trailing stop orders
- `strategyId`, `strategyType` — if set on creation
- `preventedMatchId`, `preventedQuantity` — if STP triggered
- `usedSor`, `workingFloor` — for SOR orders
- `pegPriceType`, `pegOffsetType`, `pegOffsetValue`, `peggedPrice` — for pegged orders
- `expiryReason` — why order expired

---

### Futures — Create Order (`POST /fapi/v1/order`)

**ACK Response** (default):
```json
{
  "clientOrderId": "testOrder",
  "cumQty": "0",
  "cumQuote": "0",
  "executedQty": "0",
  "orderId": 22542179,
  "avgPrice": "0.00000",
  "origQty": "10",
  "price": "0",
  "reduceOnly": false,
  "side": "BUY",
  "positionSide": "SHORT",
  "status": "NEW",
  "stopPrice": "9300",
  "closePosition": false,
  "symbol": "BTCUSDT",
  "timeInForce": "GTD",
  "type": "TRAILING_STOP_MARKET",
  "origType": "TRAILING_STOP_MARKET",
  "activatePrice": "9020",
  "priceRate": "0.3",
  "updateTime": 1566818724722,
  "workingType": "CONTRACT_PRICE",
  "priceProtect": false,
  "priceMatch": "NONE",
  "selfTradePreventionMode": "NONE",
  "goodTillDate": 1693207680000
}
```

**Futures-specific fields vs Spot:**
- `positionSide` — `BOTH`, `LONG`, or `SHORT`
- `avgPrice` — average fill price
- `cumQty` — cumulative filled quantity
- `cumQuote` — cumulative quote volume
- `reduceOnly` — boolean
- `closePosition` — boolean
- `activatePrice` — activation price for trailing stops
- `priceRate` — callback rate for trailing stops
- `workingType` — `MARK_PRICE` or `CONTRACT_PRICE`
- `priceProtect` — boolean
- `origType` — original order type (same as type unless modified)
- `goodTillDate` — for GTD orders

**No `fills` array in Futures responses.** Individual fills accessible via `/fapi/v1/userTrades`.

---

### Order Status Values

| Status | Spot | Futures | Description |
|--------|------|---------|-------------|
| `NEW` | Yes | Yes | Order placed, not yet filled |
| `PARTIALLY_FILLED` | Yes | Yes | Some quantity has been filled |
| `FILLED` | Yes | Yes | Entire quantity has been filled |
| `CANCELED` | Yes | Yes | Canceled by user or system |
| `PENDING_NEW` | Yes | No | Order being processed (brief state) |
| `REJECTED` | Yes | Yes | Order rejected (invalid params etc.) |
| `EXPIRED` | Yes | Yes | IOC/FOK not fully filled; GTD expired; GTX would cross |
| `EXPIRED_IN_MATCH` | Yes | No | Expired due to STP self-trade prevention |
| `NEW_INSURANCE` | No | Yes | Liquidation order pending (Futures) |
| `NEW_ADL` | No | Yes | Auto-deleveraging order (Futures) |

---

### Field Mapping: Binance → V5 Unified

| Binance Spot Field | Binance Futures Field | V5 Unified Field | Notes |
|--------------------|-----------------------|------------------|-------|
| `orderId` | `orderId` | `exchange_order_id` | Exchange-assigned numeric ID |
| `clientOrderId` | `clientOrderId` | `client_order_id` | User-assigned string ID |
| `symbol` | `symbol` | `symbol` | |
| `side` | `side` | `side` | `BUY`/`SELL` |
| `type` | `type` | `order_type` | Different enums; needs mapping |
| `origQty` | `origQty` | `quantity` | Original submitted quantity |
| `executedQty` | `executedQty` | `filled_qty` | Filled so far |
| `cummulativeQuoteQty` | `cumQuote` | `filled_quote_qty` | Quote asset filled |
| `price` | `price` | `price` | Limit price (`"0"` for market) |
| `stopPrice` | `stopPrice` | `stop_price` | Trigger price |
| `status` | `status` | `status` | Needs enum mapping |
| `timeInForce` | `timeInForce` | `time_in_force` | |
| `transactTime` | `updateTime` | `updated_at` | ms timestamp |
| `workingTime` | N/A | `created_at` | Spot only |
| N/A | `avgPrice` | `avg_fill_price` | Futures only |
| N/A | `positionSide` | `position_side` | Futures only |
| N/A | `reduceOnly` | `reduce_only` | Futures only |
| `fills[].price` | N/A (use userTrades) | `fills[].price` | Spot FULL response |
| `fills[].commission` | N/A (use userTrades) | `fills[].fee` | Spot FULL response |

---

### Spot Trade History Response (`GET /api/v3/myTrades`)

```json
[
  {
    "symbol": "BNBBTC",
    "id": 28457,
    "orderId": 100234,
    "orderListId": -1,
    "price": "4.00000100",
    "qty": "12.00000000",
    "quoteQty": "48.000012",
    "commission": "10.10000000",
    "commissionAsset": "BNB",
    "time": 1499865549590,
    "isBuyer": true,
    "isMaker": false,
    "isBestMatch": true
  }
]
```

Parameters: `symbol` (required), `orderId`, `startTime`, `endTime`, `fromId`, `limit` (max 1000).
Weight: 20 (without orderId), 5 (with orderId).

---

### Futures Trade History Response (`GET /fapi/v1/userTrades`)

Fields: `buyer`, `commission`, `commissionAsset`, `id`, `maker`, `orderId`, `price`, `qty`, `quoteQty`, `realizedPnl`, `side`, `positionSide`, `symbol`, `time`.

Parameters: `symbol` (required), `orderId`, `startTime`, `endTime`, `fromId`, `limit` (max 1000). Time range max 7 days. Weight: 5.

---

## Sources

- [Binance Spot Trading Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Binance Spot Account Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints)
- [Binance USDM Futures New Order](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api)
- [Binance USDM Futures Modify Order](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Modify-Order)
- [Binance USDM Futures Place Multiple Orders](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Place-Multiple-Orders)
- [Binance USDM Futures Cancel All Open Orders](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Cancel-All-Open-Orders)
- [Binance USDM Futures All Orders](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/All-Orders)
- [Binance USDM Futures Position Information V2](https://developers.binance.com/docs/derivatives/usds-margined-futures/trade/rest-api/Position-Information-V2)
- [Binance Spot Testnet](https://developers.binance.com/docs/binance-spot-api-docs/testnet)
- [Binance Futures Order Types FAQ](https://www.binance.com/en/support/faq/types-of-order-on-binance-futures-360033779452)
- [Binance Spot GitHub API Docs](https://github.com/binance/binance-spot-api-docs/blob/master/rest-api.md)
