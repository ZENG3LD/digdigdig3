# BingX Trading API — Complete Reference

**Base URL (Production):** `https://open-api.bingx.com`
**Base URL (Sandbox/VST):** `https://open-api-vst.bingx.com`

**API prefix for all endpoints:** `/openApi/`

BingX offers three trading market types:
- **Spot** — `/openApi/spot/v1/` and `/openApi/spot/v2/`
- **Perpetual Futures (USDT-M)** — `/openApi/swap/v2/` (primary production path)
- **Coin-M Perpetual (Inverse)** — `/openApi/cswap/v1/`

---

## Order Types Supported

### Spot Order Types

| Type | Description |
|------|-------------|
| `MARKET` | Executes immediately at best available price |
| `LIMIT` | Executes at specified price or better |

Source: Standard Contract REST API doc; CCXT bingx.py implementation confirms spot supports market and limit.

### Perpetual Futures (Swap V2) Order Types

| Type | Description |
|------|-------------|
| `MARKET` | Market order, fills immediately |
| `LIMIT` | Limit order at specified price |
| `STOP` | Stop-limit order — triggers at stopPrice, then places a limit |
| `STOP_MARKET` | Stop-market order — triggers at stopPrice, then fills as market |
| `TAKE_PROFIT` | Take-profit limit order |
| `TAKE_PROFIT_MARKET` | Take-profit market order |
| `TRAILING_STOP_MARKET` | Trailing stop — activates at activationPrice, trails by priceRate |
| `TRAILING_TP_SL` | Trailing TP/SL conditional order for existing positions |

### Time-in-Force Options (Futures)

| TIF | Name | Behavior |
|-----|------|----------|
| `GTC` | Good-Till-Cancelled | Active until fully filled or manually cancelled |
| `IOC` | Immediate-Or-Cancel | Fill as much as possible immediately, cancel remainder |
| `FOK` | Fill-Or-Kill | Fill entire order immediately or cancel entirely |
| `PostOnly` | Post-Only (Maker-Only) | Cancelled if it would execute as a taker |

Note: `timeInForce` applies to LIMIT and STOP order types. GTC, IOC, FOK confirmed in BingX official support article "Perpetual Futures | What Are GTC, IOC and FOK Orders".

### Time-in-Force Options (Spot)

Spot API exposes `timeInForce` parameter on limit orders. GTC is default. IOC and FOK confirmed supported via CCXT implementation. PostOnly: NOT DOCUMENTED for spot.

---

## Order Placement — Spot

### Place Single Order

```
POST /openApi/spot/v1/trade/order
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair, e.g. `BTC-USDT` |
| `side` | enum | YES | `BUY` or `SELL` |
| `type` | enum | YES | `MARKET` or `LIMIT` |
| `quantity` | decimal | Conditional | Order quantity in base asset. Required for LIMIT and for MARKET sell |
| `quoteOrderQty` | decimal | Conditional | Quote asset quantity for MARKET buy |
| `price` | decimal | Conditional | Required for LIMIT orders |
| `timeInForce` | enum | NO | `GTC` / `IOC` / `FOK` — defaults to GTC for LIMIT |
| `clientOrderId` | string | NO | Custom client-assigned order ID |
| `recvWindow` | long | NO | Request validity window in milliseconds (default 5000) |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

**Authentication header:** `X-BX-APIKEY: <apiKey>`

**Note:** V1 spot interface is being deprecated in favor of V2. Use V2 for new integrations.

### Batch/Bulk Order Placement (Spot)

```
POST /openApi/spot/v1/trade/batchOrders
```

Batch order placement is confirmed to exist in CCXT implementation and bingx-py SDK. Maximum batch size: NOT DOCUMENTED in publicly accessible sources.

---

## Order Placement — Perpetual Futures (Swap V2)

### Place Single Order

```
POST /openApi/swap/v2/trade/order
```

**Rate limit:** 10 requests/second (upgraded from 5 req/sec as of 2025-10-16)

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair, e.g. `BTC-USDT` |
| `side` | enum | YES | `BUY` or `SELL` |
| `positionSide` | enum | Conditional | `LONG`, `SHORT`, or `BOTH` — required in hedge mode |
| `type` | enum | YES | See order types table above |
| `quantity` | decimal | Conditional | Order quantity in contracts. Required for non-closePosition orders |
| `price` | decimal | Conditional | Required for LIMIT, STOP, TAKE_PROFIT types |
| `stopPrice` | decimal | Conditional | Trigger price for STOP, STOP_MARKET, TAKE_PROFIT, TAKE_PROFIT_MARKET |
| `timeInForce` | enum | NO | `GTC` / `IOC` / `FOK` — for LIMIT and STOP orders |
| `reduceOnly` | boolean | NO | If true, order will only reduce an existing position |
| `closePosition` | boolean | NO | If true, close the entire open position for this symbol |
| `activationPrice` | decimal | Conditional | Required for `TRAILING_STOP_MARKET` — price that activates the trailing stop |
| `priceRate` | decimal | Conditional | Required for `TRAILING_STOP_MARKET` — trailing distance as percentage |
| `workingType` | enum | NO | Price type for stopPrice: `MARK_PRICE` or `CONTRACT_PRICE` |
| `newOrderRespType` | enum | NO | Response format: `ACK` or `RESULT` |
| `takeProfit` | object | NO | Embedded TP object — use type `TAKE_PROFIT` or `TAKE_PROFIT_MARKET` |
| `stopLoss` | object | NO | Embedded SL object — use type `STOP` or `STOP_MARKET` |
| `clientOrderId` | string | NO | Custom client-assigned order ID |
| `recvWindow` | long | NO | Request validity window in milliseconds |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

**TP/SL note:** `takeProfit` and `stopLoss` fields support specific types only:
- Stop loss must use type `STOP_MARKET` or `STOP`
- Take profit must use type `TAKE_PROFIT_MARKET` or `TAKE_PROFIT`

### Place Test Order (Futures)

```
POST /openApi/swap/v2/trade/testOrder
```

Same parameters as place order but does NOT execute. Used for parameter validation.

### Batch Order Placement (Futures)

```
POST /openApi/swap/v2/trade/batchOrders
```

Rate limit weight: 2. Maximum batch size: NOT DOCUMENTED in publicly accessible sources. Confirmed to exist in CCXT bingx.py and bingx-php SDK (54-method trade service with batch support).

### TWAP Order Placement (Futures — Algo)

```
POST /openApi/swap/v1/twap/order
```

TWAP (Time-Weighted Average Price) algorithmic orders are confirmed supported. Endpoints:

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/openApi/swap/v1/twap/order` | Place TWAP algo order |
| `POST` | `/openApi/swap/v1/twap/cancelOrder` | Cancel TWAP order |
| `GET` | `/openApi/swap/v1/twap/openOrders` | Get open TWAP orders |
| `GET` | `/openApi/swap/v1/twap/historyOrders` | Get TWAP order history |
| `GET` | `/openApi/swap/v1/twap/orderDetail` | Get TWAP order detail |

Source: CCXT bingx.py endpoint dictionary, bingx-php SDK TWAP service (7 methods).

### Iceberg Orders

NOT DOCUMENTED in publicly accessible BingX API sources.

### Copy Trading API

Copy trading API is confirmed to exist (bingx-php SDK has 13-method copy trading service), but specific endpoints are NOT DOCUMENTED in publicly accessible sources.

---

## Order Management — Spot

### Cancel Single Order

```
POST /openApi/spot/v1/trade/cancel
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair |
| `orderId` | long | Conditional | Order ID. One of orderId or clientOrderId required |
| `clientOrderId` | string | Conditional | Client-assigned order ID |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

### Get Single Order (Spot)

```
GET /openApi/spot/v1/trade/query
```

Parameters: `symbol`, `orderId` or `clientOrderId`, `timestamp`, `signature`.

### Get Open Orders (Spot)

```
GET /openApi/spot/v1/trade/openOrders
```

Parameters: `symbol` (optional filter), `timestamp`, `signature`.

### Get Order History (Spot)

```
GET /openApi/spot/v1/trade/historyOrders
```

Pagination: NOT DOCUMENTED (specific parameters not in accessible sources).

### Get My Trades (Spot)

```
GET /openApi/spot/v1/trade/myTrades
```

Rate limit weight: 2.

### Amend/Modify Order (Spot)

NOT AVAILABLE — Spot does not have an amend endpoint. Use cancel + replace.

---

## Order Management — Perpetual Futures (Swap V2)

### Cancel Single Order

```
DELETE /openApi/swap/v2/trade/order
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair |
| `orderId` | long | Conditional | Exchange order ID |
| `clientOrderId` | string | Conditional | Client-assigned order ID |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

### Cancel All Open Orders (Futures)

```
DELETE /openApi/swap/v2/trade/allOpenOrders
```

Parameters: `symbol`, `timestamp`, `signature`.

### Cancel Batch Orders (Futures)

```
DELETE /openApi/swap/v2/trade/batchOrders
```

Cancel multiple orders by providing a list of order IDs.

### Cancel-Replace / Amend Order (Futures — Swap V1)

```
POST /openApi/swap/v1/trade/cancelReplace
```

Modify an existing order (cancel and replace in one request). This is the primary order amendment method.

```
POST /openApi/swap/v1/trade/amend
```

Direct order amendment — modifies price/quantity of an existing pending order without cancel+replace. Confirmed in CCXT bingx.py private endpoint dictionary.

Note: Amend/cancelReplace exist under swap v1 paths, not v2.

### Batch Cancel-Replace (Futures)

```
GET /openApi/swap/v1/trade/batchCancelReplace
POST /openApi/swap/v1/trade/batchCancelReplace
```

Confirmed in CCXT bingx.py.

### Get Single Order (Futures)

```
GET /openApi/swap/v2/trade/order
```

Parameters: `symbol`, `orderId` or `clientOrderId`, `timestamp`, `signature`.

### Get Open Orders (Futures)

```
GET /openApi/swap/v2/trade/openOrders
```

Parameters: `symbol` (optional), `timestamp`, `signature`.

### Get All Orders / Order History (Futures)

```
GET /openApi/swap/v2/trade/allOrders
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair |
| `orderId` | long | NO | Starting order ID for pagination |
| `startTime` | long | NO | Start time in milliseconds |
| `endTime` | long | NO | End time in milliseconds |
| `limit` | int | NO | Number of records to return |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

---

## Position Management — Perpetual Futures

### Get Open Positions

```
GET /openApi/swap/v2/user/positions
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | NO | Filter by trading pair |
| `recvWindow` | long | NO | Request validity window |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

**Response fields:**
```json
{
  "symbol": "BTC/USDT",
  "initialMargin": 2,
  "leverage": 1,
  "unrealizedProfit": -0.7239062,
  "isolated": true,
  "entryPrice": 30006.65,
  "positionSide": "LONG",
  "positionAmt": 0.00006666,
  "currentPrice": 19145.65,
  "time": 1654782192000
}
```

### Get Position History

```
GET /openApi/swap/v2/user/positionsHistory
```

Parameters: `symbol`, `startTime`, `endTime`, confirmed in Bingex Elixir client.

### Close All Positions

```
POST /openApi/swap/v2/trade/closeAllPositions
```

Parameters: `symbol`, `timestamp`, `signature`.

### Close Position (Single — Swap V1)

```
POST /openApi/swap/v1/trade/closePosition
```

Confirmed in CCXT bingx.py swap v1 private endpoint dictionary.

### Set Leverage

```
POST /openApi/swap/v2/trade/leverage
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair |
| `side` | enum | YES | Position side: `LONG` or `SHORT` — or `:crossed`/`:isolated` in some clients |
| `leverage` | int | YES | Leverage value (1 to 125; some pairs up to 150) |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Rate limit weight: 5.

### Get Leverage

```
GET /openApi/swap/v2/trade/leverage
```

Same symbol parameter. Returns current leverage and available positions.

### Change Margin Mode

```
POST /openApi/swap/v2/trade/marginType
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair |
| `marginType` | enum | YES | `ISOLATED` or `CROSSED` |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Rate limit weight: 5.

### Add/Reduce Margin

```
POST /openApi/swap/v2/trade/positionMargin
```

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `symbol` | string | YES | Trading pair |
| `amount` | decimal | YES | Margin amount to add (positive) or reduce (negative) |
| `type` | int | YES | `1` = Add margin, `2` = Reduce margin |
| `positionSide` | enum | NO | `LONG` or `SHORT` |
| `timestamp` | long | YES | Millisecond timestamp |
| `signature` | string | YES | HMAC-SHA256 signature |

Rate limit weight: 5.

### Get Funding Rate

```
GET /openApi/swap/v2/quote/fundingRate
```

Public endpoint. Parameters: `symbol`. Returns current funding rate.

**Funding rate settlement:** 3 times per day at 00:00, 08:00, and 16:00 (UTC+8).

### Get Liquidation Price

NOT AVAILABLE as a dedicated API endpoint. Liquidation price is calculated client-side based on position margin and maintenance margin requirement. BingX uses a Dual-Price Mechanism (both last price AND mark price must reach liquidation price before liquidation is triggered).

### Set Dual/Hedge Position Side

```
POST /openApi/swap/v1/positionSide/dual
```

Switches between one-way mode and hedge mode (which allows simultaneous LONG/SHORT positions). Rate limit weight: 5.

---

## Coin-M (Inverse) Perpetual Futures

Endpoint base: `/openApi/cswap/v1/`

Endpoints confirmed (CCXT bingx.py):

| Method | Path | Description |
|--------|------|-------------|
| `POST` | `/openApi/cswap/v1/trade/order` | Place order |
| `POST` | `/openApi/cswap/v1/trade/cancelOrder` | Cancel order |
| `DELETE` | `/openApi/cswap/v1/trade/allOpenOrders` | Cancel all open orders |
| `GET` | `/openApi/cswap/v1/trade/openOrders` | Get open orders |
| `POST` | `/openApi/cswap/v1/trade/leverage` | Set leverage |
| `POST` | `/openApi/cswap/v1/trade/marginType` | Set margin mode |
| `POST` | `/openApi/cswap/v1/trade/positionMargin` | Adjust margin |
| `GET` | `/openApi/cswap/v1/user/balance` | Get balance |
| `GET` | `/openApi/cswap/v1/user/positions` | Get positions |

---

## Advanced Order Features

### Trailing Stop Orders (Futures)

Use `type = TRAILING_STOP_MARKET` on the standard order endpoint.

Key parameters:
- `activationPrice` — price that triggers the trailing mechanism
- `priceRate` — trailing percentage distance from market price
- `price` — initial price reference

### Bracket Orders / OCO

NOT AVAILABLE as a dedicated OCO endpoint. However, TP and SL can be attached directly to the order placement request via the `takeProfit` and `stopLoss` embedded objects, achieving bracket-like behavior in a single call.

### Grid Trading API

Confirmed to exist conceptually (BingX offers grid trading on the platform). Specific grid API endpoints are NOT DOCUMENTED in publicly accessible sources.

### Copy Trading API

Confirmed to exist (13-method copy trading service in bingx-php SDK). Specific endpoints NOT DOCUMENTED in publicly accessible sources.

---

## WebSocket — Trading Data

| Stream | URL |
|--------|-----|
| Market data | `wss://open-api-ws.bingx.com/market` |
| Account/private | `wss://open-api-ws.bingx.com/market?listenKey=<listenKey>` |

**Listen Key management:**
```
POST   /openApi/user/auth/userDataStream   — Generate listen key
PUT    /openApi/user/auth/userDataStream   — Extend listen key
DELETE /openApi/user/auth/userDataStream   — Delete listen key
```

**Note:** All WebSocket messages are GZIP compressed and must be decompressed.

---

## Sources

- [BingX Official API Docs](https://bingx-api.github.io/docs/)
- [BingX Standard Contract REST API (GitHub)](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [CCXT BingX Implementation (bingx.py)](https://raw.githubusercontent.com/ccxt/ccxt/master/python/ccxt/bingx.py)
- [bingx-php SDK — 220+ Methods](https://github.com/tigusigalpa/bingx-php)
- [bingx_py Python Client](https://bingx-py.readthedocs.io/en/latest/)
- [Bingex Elixir Client](https://hexdocs.pm/bingex/Bingex.Swap.html)
- [py-bingx Unofficial Client](https://github.com/amirinsight/py-bingx/)
- [BingX Swap API Doc Issue — Trailing Stop](https://github.com/BingX-API/BingX-swap-api-doc/issues/28)
- [BingX Perpetual Futures Order Types](https://bingx.com/en/support/articles/17917912060569-perpetualfuturesordertypes)
- [BingX Rate Limit Upgrade 2025-10-16](https://bingx.com/en/support/articles/31103871611289)
