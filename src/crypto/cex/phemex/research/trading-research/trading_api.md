# Phemex Trading API — Order & Position Endpoints

Source: https://phemex-docs.github.io/

---

## Order Types Supported

Applies across COIN-M Perpetual, USDM Perpetual, and Spot APIs.

| Order Type        | Description                                                              |
|-------------------|--------------------------------------------------------------------------|
| `Limit`           | Standard limit order at specified price                                   |
| `Market`          | Immediate execution at current market price                               |
| `Stop`            | Stop-market: triggers a market order at stop price                        |
| `StopLimit`       | Stop-limit: triggers a limit order when stop price is hit                 |
| `MarketIfTouched` | Market order triggered when price touches specified level                  |
| `LimitIfTouched`  | Limit order triggered when price touches specified level                   |

### Time-in-Force Options

| Value           | Description                                                              |
|-----------------|--------------------------------------------------------------------------|
| `GoodTillCancel`  | Order remains active until manually cancelled (GTC)                    |
| `ImmediateOrCancel` | Fill immediately (partial OK), cancel unfilled remainder (IOC)       |
| `FillOrKill`      | Fill entire order immediately or cancel completely (FOK)               |
| `PostOnly`        | Maker-only; rejected if it would immediately match (taker)             |
| `RPIPostOnly`     | Retail Price Improvement; only valid for Approved Market Makers        |

### Conditional Orders (TP/SL)

- Attach take-profit and stop-loss to primary orders via `takeProfitEp` / `stopLossEp` parameters.
- Trigger source selectable: `ByMarkPrice` or `ByLastPrice` (via `tpTrigger` / `slTrigger` fields).

### Trailing Stop Orders

- Use `pegPriceType: TrailingStopPeg` with `pegOffsetValueEp` (negative for longs, positive for shorts).
- Supports activation with `TrailingTakeProfitPeg`.

### Bracket Orders

- Up to 5 related orders can share the same side.
- Automatic TP/SL management fires on main order fill.

### Execution Flags

| Flag              | Behavior                                                                |
|-------------------|-------------------------------------------------------------------------|
| `reduceOnly`      | Order can only reduce existing position size, never increase it         |
| `closeOnTrigger`  | Closes the position and cancels related directional orders on trigger   |

---

## COIN-M Perpetual — Order Placement

Base URL: `https://api.phemex.com`

### Place Order (preferred)

```
PUT /orders/create
```

Parameters (query string or JSON body):

| Field            | Type    | Required | Description                                          |
|------------------|---------|----------|------------------------------------------------------|
| `symbol`         | string  | Yes      | Trading pair (e.g. `BTCUSD`)                         |
| `clOrdID`        | string  | No       | Client order ID (max 40 chars)                       |
| `side`           | string  | Yes      | `Buy` or `Sell`                                      |
| `orderQty`       | integer | Yes      | Order quantity (in contracts)                        |
| `ordType`        | string  | Yes      | Order type (see above)                               |
| `priceEp`        | integer | Cond.    | Scaled price (required for Limit orders)             |
| `stopPxEp`       | integer | Cond.    | Scaled stop/trigger price                            |
| `timeInForce`    | string  | No       | GoodTillCancel / ImmediateOrCancel / FillOrKill / PostOnly |
| `reduceOnly`     | bool    | No       | Reduce-only flag                                     |
| `closeOnTrigger` | bool    | No       | Close-on-trigger flag                                |
| `takeProfitEp`   | integer | No       | Scaled take-profit price                             |
| `stopLossEp`     | integer | No       | Scaled stop-loss price                               |
| `triggerType`    | string  | No       | `ByMarkPrice` or `ByLastPrice`                       |
| `pegOffsetValueEp` | integer | No    | Trailing stop offset (scaled)                        |
| `pegPriceType`   | string  | No       | `TrailingStopPeg` or `TrailingTakeProfitPeg`         |

### Place Order (alternative — JSON body)

```
POST /orders
```

Same parameters as above, passed in JSON body.

### Amend/Modify Order

```
PUT /orders/replace
```

| Field              | Type    | Required | Description                         |
|--------------------|---------|----------|-------------------------------------|
| `symbol`           | string  | Yes      | Trading pair                        |
| `orderID`          | string  | Either   | Exchange-assigned order ID          |
| `origClOrdID`      | string  | Either   | Original client order ID            |
| `priceEp`          | integer | No       | New price (scaled)                  |
| `orderQty`         | integer | No       | New quantity                        |
| `stopPxEp`         | integer | No       | New stop price (scaled)             |
| `takeProfitEp`     | integer | No       | New take-profit price               |
| `stopLossEp`       | integer | No       | New stop-loss price                 |
| `pegOffsetValueEp` | integer | No       | New trailing offset                 |
| `pegPriceType`     | string  | No       | New peg price type                  |

### Cancel Single Order

```
DELETE /orders/cancel
```

| Field     | Type   | Required | Description          |
|-----------|--------|----------|----------------------|
| `symbol`  | string | Yes      | Trading pair         |
| `orderID` | string | Either   | Exchange order ID    |
| `clOrdID` | string | Either   | Client order ID      |

### Cancel Multiple Orders (Bulk)

```
DELETE /orders
```

| Field     | Type   | Required | Description                                |
|-----------|--------|----------|--------------------------------------------|
| `symbol`  | string | Yes      | Trading pair                               |
| `orderID` | string | Yes      | Comma-separated order IDs                  |

### Cancel All Orders for Symbol

```
DELETE /orders/all
```

| Field         | Type   | Required | Description                          |
|---------------|--------|----------|--------------------------------------|
| `symbol`      | string | Yes      | Trading pair                         |
| `untriggered` | bool   | No       | Cancel only untriggered conditional orders |
| `text`        | string | No       | Optional note                        |

### Get Open Orders

```
GET /orders/activeList
```

| Field    | Type   | Required | Description  |
|----------|--------|----------|--------------|
| `symbol` | string | Yes      | Trading pair |

### Get Single Open Order

```
GET /orders/active
```

| Field     | Type   | Required | Description       |
|-----------|--------|----------|-------------------|
| `symbol`  | string | Yes      | Trading pair      |
| `orderID` | string | Yes      | Exchange order ID |

### Get Order History (Closed Orders)

```
GET /exchange/order/list
```

| Field       | Type    | Required | Description                                      |
|-------------|---------|----------|--------------------------------------------------|
| `symbol`    | string  | Yes      | Trading pair                                     |
| `start`     | integer | No       | Start time (milliseconds)                        |
| `end`       | integer | No       | End time (milliseconds)                          |
| `offset`    | integer | No       | Pagination offset                                |
| `limit`     | integer | No       | Records per page                                 |
| `ordStatus` | string  | No       | Filter by status (e.g. `Filled`, `Cancelled`)    |

### Get Order by ID

```
GET /exchange/order
```

| Field     | Type   | Required | Description                           |
|-----------|--------|----------|---------------------------------------|
| `symbol`  | string | Yes      | Trading pair                          |
| `orderID` | string | Either   | Comma-separated order IDs             |
| `clOrdID` | string | Either   | Comma-separated client order IDs      |

### Get Trade/Execution History

```
GET /exchange/order/trade
```

| Field    | Type    | Required | Description          |
|----------|---------|----------|----------------------|
| `symbol` | string  | Yes      | Trading pair         |
| `start`  | integer | No       | Start time (ms)      |
| `end`    | integer | No       | End time (ms)        |
| `limit`  | integer | No       | Records per page     |
| `offset` | integer | No       | Pagination offset    |

---

## USDM Perpetual — Order Placement

Prefix: `/g-` (all endpoints mirror COIN-M but use this prefix)

### Place Order

```
PUT /g-orders/create
POST /g-orders
```

Same parameters as COIN-M. Key differences:
- No `pegOffsetValueEp` / `pegPriceType` documented in this section
- Position mode switching available via `PUT /g-positions/switch-pos-mode-sync`

### Amend Order

```
PUT /g-orders/replace
```

| Field          | Type    | Required | Description              |
|----------------|---------|----------|--------------------------|
| `symbol`       | string  | Yes      | Trading pair             |
| `orderID`      | string  | Either   | Exchange order ID        |
| `origClOrdID`  | string  | Either   | Client order ID          |
| `priceEp`      | integer | No       | New price (scaled)       |
| `orderQty`     | integer | No       | New quantity             |
| `stopPxEp`     | integer | No       | New stop price           |
| `takeProfitEp` | integer | No       | New take-profit          |
| `stopLossEp`   | integer | No       | New stop-loss            |

### Cancel Single Order

```
DELETE /g-orders/cancel
```

### Cancel All Orders

```
DELETE /g-orders/all
```

| Field    | Type   | Required | Description  |
|----------|--------|----------|--------------|
| `symbol` | string | Yes      | Trading pair |

### Bulk Cancel

```
DELETE /g-orders
```

| Field     | Type   | Required | Description               |
|-----------|--------|----------|---------------------------|
| `symbol`  | string | Yes      | Trading pair              |
| `orderID` | string | Yes      | Comma-separated order IDs |

### Get Open Orders

```
GET /g-orders/activeList
GET /g-orders/active
```

### Get Order History

```
GET /g-orders
```

| Field    | Type    | Required | Description          |
|----------|---------|----------|----------------------|
| `symbol` | string  | Yes      | Trading pair         |
| `start`  | integer | No       | Start time (ms)      |
| `end`    | integer | No       | End time (ms)        |
| `offset` | integer | No       | Pagination offset    |
| `limit`  | integer | No       | Records per page     |

### Get Trade History

```
GET /g-orders/trade
```

---

## Spot — Order Placement

### Place Order

```
PUT /spot/orders    (preferred)
POST /spot/orders
```

| Field         | Type    | Required | Description                                    |
|---------------|---------|----------|------------------------------------------------|
| `symbol`      | string  | Yes      | Spot pair (e.g. `sBTCUSDT`)                    |
| `clOrdID`     | string  | No       | Client order ID (max 40 chars)                 |
| `side`        | string  | Yes      | `Buy` or `Sell`                                |
| `orderQty`    | string  | Yes      | Quantity                                       |
| `priceEp`     | integer | Cond.    | Scaled price (Limit orders)                    |
| `ordType`     | string  | Yes      | `Market`, `Limit`, `Stop`, `StopLimit`, etc.   |
| `timeInForce` | string  | No       | GoodTillCancel / ImmediateOrCancel / FillOrKill / PostOnly |
| `displayQty`  | integer | No       | Iceberg visible quantity                       |
| `text`        | string  | No       | Optional notes                                 |

Note: `displayQty` enables iceberg order behavior on spot.

### Amend Spot Order

```
PUT /spot/orders
```

| Field     | Type    | Required | Description                  |
|-----------|---------|----------|------------------------------|
| `symbol`  | string  | Yes      | Trading pair                 |
| `orderID` | string  | Either   | Exchange order ID            |
| `clOrdID` | string  | Either   | Client order ID              |
| `priceEp` | integer | No       | New price (scaled)           |
| `orderQty`| string  | No       | New quantity                 |

### Cancel Spot Order

```
DELETE /spot/orders
```

| Field     | Type   | Required | Description          |
|-----------|--------|----------|----------------------|
| `symbol`  | string | Yes      | Trading pair         |
| `orderID` | string | Either   | Exchange order ID    |
| `clOrdID` | string | Either   | Client order ID      |

### Cancel All Spot Orders

```
DELETE /spot/orders/all
```

| Field    | Type   | Required | Description  |
|----------|--------|----------|--------------|
| `symbol` | string | Yes      | Trading pair |

### Get Open Spot Orders

```
GET /spot/orders/active
GET /spot/orders
```

| Field     | Type   | Required | Description          |
|-----------|--------|----------|----------------------|
| `symbol`  | string | Yes      | Trading pair         |
| `orderID` | string | No       | Specific order ID    |
| `clOrdID` | string | No       | Client order ID      |

### Get Spot Order History

```
GET /spot/orders
```

| Field       | Type    | Required | Description                   |
|-------------|---------|----------|-------------------------------|
| `symbol`    | string  | Yes      | Trading pair                  |
| `start`     | integer | No       | Start time (ms)               |
| `end`       | integer | No       | End time (ms)                 |
| `offset`    | integer | No       | Pagination offset             |
| `limit`     | integer | No       | Records per page              |
| `ordStatus` | string  | No       | Filter: `Filled`, `Cancelled` |

---

## Position Management (Futures)

### Get Account + Positions

```
GET /accounts/accountPositions      (COIN-M)
GET /g-accounts/accountPositions    (USDM)
```

| Field      | Type   | Required | Description          |
|------------|--------|----------|----------------------|
| `currency` | string | Cond.    | Settlement currency (COIN-M only) |

### Get Positions with Unrealized PnL

```
GET /accounts/positions     (COIN-M)
GET /g-accounts/positions   (USDM)
```

### Set Leverage

```
PUT /positions/leverage     (COIN-M)
PUT /g-positions/leverage   (USDM)
```

| Field        | Type    | Required | Description                                              |
|--------------|---------|----------|----------------------------------------------------------|
| `symbol`     | string  | Yes      | Trading pair                                             |
| `leverage`   | integer | Either   | Leverage value; ≤0 = cross margin, >0 = isolated (COIN-M) |
| `leverageEr` | integer | Either   | Scaled leverage (COIN-M alt / USDM)                      |

Note: Leverage sign determines margin mode. Setting `leverage=0` enables cross margin; positive values set isolated margin with that leverage.

### Set Risk Limit

```
PUT /positions/riskLimit    (COIN-M)
PUT /g-positions/riskLimit  (USDM)
```

| Field          | Type    | Required | Description                    |
|----------------|---------|----------|--------------------------------|
| `symbol`       | string  | Yes      | Trading pair                   |
| `riskLimit`    | integer | Either   | Risk limit value (COIN-M)      |
| `riskLimitEv`  | integer | Either   | Scaled risk limit              |

### Assign Margin to Isolated Position

```
POST /positions/assign      (COIN-M)
POST /g-positions/assign    (USDM)
```

| Field          | Type    | Required | Description                         |
|----------------|---------|----------|-------------------------------------|
| `symbol`       | string  | Yes      | Trading pair                        |
| `posBalance`   | integer | Either   | Balance to assign (COIN-M)          |
| `posBalanceEv` | integer | Either   | Scaled balance (COIN-M alt / USDM)  |

### Switch Position Mode (USDM only)

```
PUT /g-positions/switch-pos-mode-sync
```

### Get Funding Rate History

```
GET /api-data/public/data/funding-rate-history?symbol=<symbol>
```

Public endpoint (no auth required).

### Get Liquidation Price

Returned as the `liquidationPriceEp` field in position query responses (no dedicated endpoint).

---

## Advanced Features

### Iceberg Orders
Supported on Spot via `displayQty` parameter in `/spot/orders`. Sets visible quantity for the order.

### Bracket Orders
Supported on Futures. Up to 5 related orders with same side; TP/SL auto-managed on fill.

### Trailing Stop Orders
Supported on Futures via `pegPriceType: TrailingStopPeg` and `pegOffsetValueEp`.

### Bulk Order Placement
No dedicated bulk placement endpoint documented. Bulk cancellation is supported (comma-separated `orderID`).

### Algo Orders (TWAP, Grid, Copy Trade)
- **Copy Trade**: Read-only query endpoint `GET /copy-trade/traders` for trader performance metrics. No autonomous order placement API.
- **TWAP / Grid Trading**: Not documented in the REST API reference.

### Self-Trade Prevention (STP)
Available via `stpGroupId` parameter — contact Phemex support to enable for your account.

---

## Fee Rate Query

```
GET /api-data/futures/fee-rate?settleCurrency=<settleCurrency>
```

Returns `takerFeeRateEr` and `makerFeeRateEr` per symbol. Public endpoint.

---

## Margin Trading — Order Endpoints

Margin orders use the same Spot endpoints:

```
PUT  /spot/orders    — Place margin order
PUT  /spot/orders    — Amend margin order
DELETE /spot/orders  — Cancel margin order
GET  /spot/orders    — Query open margin orders
GET  /margin/orders/details  — Margin order details
GET  /margin/orders/trades   — Margin order trade details
```

---

## Key Notes on Scaled Prices

Phemex uses integer-scaled prices to avoid floating point. The scale factor varies by symbol (e.g., for BTCUSD: priceEp = price × 10000). The scale factor for each symbol is found in product info endpoints.

---

Sources:
- https://phemex-docs.github.io/
