# Phemex API Response Formats

Complete response structure documentation for all API endpoints.

## Standard Response Format

All REST API endpoints (except market data `/md/*`) follow this format:

```json
{
  "code": 0,           // Integer status code (0 = success)
  "msg": "OK",         // String message
  "data": { ... }      // Response data (structure varies by endpoint)
}
```

### Response Code Values

| Code | Meaning |
|------|---------|
| `0` | Success (normal processing) |
| Non-zero | Error occurred (see error codes below) |

### BizError Codes

| Code | Message | Description |
|------|---------|-------------|
| 19999 | REQUEST_IS_DUPLICATED | Duplicated request ID |
| 10001 | OM_DUPLICATE_ORDERID | Duplicated order ID |
| 10002 | OM_ORDER_NOT_FOUND | Cannot find order ID |
| 10003 | OM_ORDER_PENDING_CANCEL | Cannot cancel during pending cancellation |
| 10004 | OM_ORDER_PENDING_REPLACE | Cannot cancel during pending replacement |
| 10005 | OM_ORDER_PENDING | Cannot cancel while pending |
| 11001 | TE_NO_ENOUGH_AVAILABLE_BALANCE | Insufficient available balance |
| 11002 | TE_INVALID_RISK_LIMIT | Invalid risk limit value |
| 11003 | TE_NO_ENOUGH_BALANCE_FOR_NEW_RISK_LIMIT | Insufficient funds for risk adjustment |
| 11004 | TE_INVALID_LEVERAGE | Invalid leverage or exceeds maximum |
| 11027 | TE_SYMBOL_INVALID | Invalid symbol ID or name |
| 11028 | TE_CURRENCY_INVALID | Invalid currency ID or name |
| 11031 | TE_SO_NUM_EXCEEDS | Number of conditional orders exceeds limit |
| 11032 | TE_AO_NUM_EXCEEDS | Number of active orders exceeds limit |
| 11034 | TE_SIDE_INVALID | Trade direction is invalid |
| 11056 | TE_QTY_TOO_LARGE | Order quantity is too large |
| 11058 | TE_QTY_TOO_SMALL | Order quantity is too small |
| 11114 | TE_ORDER_VALUE_TOO_LARGE | Order value is too large |
| 11115 | TE_ORDER_VALUE_TOO_SMALL | Order value is too small |

### CxlRejReason Codes

| Code | Reason | Description |
|------|--------|-------------|
| 100 | CE_NO_ENOUGH_QTY | Quantity is not enough |
| 101 | CE_WILLCROSS | Passive order rejected due to price may cross |
| 116 | CE_NO_ENOUGH_BASE_QTY | Spot trading lacks base quantity |
| 117 | CE_NO_ENOUGH_QUOTE_QTY | Spot trading lacks quote quantity |

### HTTP Status Codes

| Status | Meaning |
|--------|---------|
| 200 | Success |
| 401 | Unauthorized (invalid credentials) |
| 403 | Forbidden (lack of privilege) |
| 429 | Too Many Requests (rate limit exceeded) |
| 5xx | Server error (execution status unknown) |

## Market Data Responses

### Server Time

**Endpoint:** `GET /public/time`

```json
{
  "error": null,
  "id": 0,
  "result": 1234567890000000  // Nanoseconds
}
```

### Products Information

**Endpoint:** `GET /public/products`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "products": [
      {
        "symbol": "BTCUSD",
        "displaySymbol": "BTC/USD",
        "indexSymbol": ".BTC",
        "markSymbol": ".MBTC",
        "fundingRateSymbol": ".BTCFR",
        "fundingRate8hSymbol": ".BTCFR8H",
        "contractUnderlyingAssets": "USD",
        "settleCurrency": "BTC",
        "quoteCurrency": "USD",
        "contractSize": 1.0,
        "lotSize": 1,
        "tickSize": 0.5,
        "priceScale": 4,
        "ratioScale": 8,
        "valueScale": 8,
        "pricePrecision": 1,
        "minPriceEp": 5000000,
        "maxPriceEp": 10000000000,
        "maxOrderQty": 1000000,
        "type": "PerpetualV2",
        "status": "Listed",
        "tipOrderQty": 1000000,
        "defaultLeverage": 0,
        "maxLeverage": 100,
        "initMarginEr": 1000000,
        "maintMarginEr": 500000,
        "defaultRiskLimitEv": 10000000000,
        "deleverage": true,
        "makerFeeRateEr": -25000,
        "takerFeeRateEr": 75000,
        "fundingInterval": 8,
        "description": "BTC/USD perpetual contracts"
      }
    ],
    "currencies": [
      {
        "currency": "BTC",
        "valueScale": 8,
        "minValueEv": 1,
        "maxValueEv": 5000000000000000,
        "name": "Bitcoin"
      }
    ],
    "perpProductsV2": [...],  // Hedged mode products
    "riskLimits": [...]
  }
}
```

**Key Fields:**
- `priceScale`: Scaling factor for `Ep` fields (10^priceScale)
- `ratioScale`: Scaling factor for `Er` fields (10^ratioScale)
- `valueScale`: Scaling factor for `Ev` fields (10^valueScale)

### Order Book

**Endpoint:** `GET /md/orderbook`

```json
{
  "error": null,
  "id": 0,
  "result": {
    "book": {
      "asks": [
        [priceEp, size],
        [87705000, 1000000]
      ],
      "bids": [
        [priceEp, size],
        [87700000, 2000000]
      ]
    },
    "depth": 30,
    "sequence": 123456789,
    "timestamp": 1234567890000000000,
    "symbol": "BTCUSD",
    "type": "snapshot"
  }
}
```

### Recent Trades

**Endpoint:** `GET /md/trade`

```json
{
  "error": null,
  "id": 0,
  "result": {
    "type": "snapshot",
    "sequence": 123456789,
    "symbol": "BTCUSD",
    "trades": [
      [timestamp, side, priceEp, size],
      [1234567890000000000, "Buy", 87705000, 1000]
    ]
  }
}
```

**Trade Fields:**
- `timestamp`: Nanoseconds
- `side`: `"Buy"` or `"Sell"`
- `priceEp`: Scaled price
- `size`: Trade quantity

### 24-Hour Ticker

**Endpoint:** `GET /md/ticker/24hr`

```json
{
  "error": null,
  "id": 0,
  "result": {
    "symbol": "BTCUSD",
    "openEp": 87000000,
    "highEp": 88000000,
    "lowEp": 86500000,
    "lastEp": 87700000,
    "bidEp": 87695000,
    "askEp": 87705000,
    "indexEp": 87702000,
    "markEp": 87700000,
    "openInterest": 123456789,
    "fundingRateEr": 10000,
    "predFundingRateEr": 10000,
    "timestamp": 1234567890000000000,
    "turnoverEv": 12345678900000,
    "volume": 12345678
  }
}
```

### Klines/Candlesticks

**Endpoint:** `GET /exchange/public/md/v2/kline`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "total": -1,
    "rows": [
      [timestamp, interval, lastEp, highEp, lowEp, openEp, volume, turnoverEv],
      [1590019200, 300, 87700000, 88000000, 87500000, 87600000, 123456, 1234567890]
    ]
  }
}
```

**Kline Fields (Array):**
- `[0]`: Timestamp (seconds)
- `[1]`: Interval (seconds)
- `[2]`: Close price (Ep)
- `[3]`: High price (Ep)
- `[4]`: Low price (Ep)
- `[5]`: Open price (Ep)
- `[6]`: Volume
- `[7]`: Turnover (Ev)

## Trading Responses

### Place Order (Spot)

**Endpoint:** `POST /spot/orders`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "orderID": "12345678-1234-1234-1234-123456789012",
    "clOrdID": "client-order-id-123",
    "symbol": "sBTCUSDT",
    "side": "Buy",
    "priceEp": 8770000000,
    "baseQtyEv": 100000000,
    "quoteQtyEv": 877000000000,
    "ordType": "Limit",
    "timeInForce": "GoodTillCancel",
    "ordStatus": "New",
    "createTimeNs": 1234567890000000000,
    "qtyType": "ByBase"
  }
}
```

### Place Order (Contract)

**Endpoint:** `POST /orders`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "bizError": 0,
    "orderID": "12345678-1234-1234-1234-123456789012",
    "clOrdID": "client-order-id-123",
    "symbol": "BTCUSD",
    "side": "Buy",
    "actionTimeNs": 1234567890000000000,
    "transactTimeNs": 1234567890000000000,
    "orderType": "Limit",
    "priceEp": 87700000,
    "price": 8770.0,
    "orderQty": 1000,
    "displayQty": 1000,
    "timeInForce": "GoodTillCancel",
    "reduceOnly": false,
    "closeOnTrigger": false,
    "takeProfitEp": 0,
    "stopLossEp": 0,
    "triggerType": "UNSPECIFIED",
    "pegOffsetValueEp": 0,
    "pegPriceType": "UNSPECIFIED",
    "ordStatus": "New"
  }
}
```

### Amend Order

**Endpoint:** `PUT /orders/replace`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "bizError": 0,
    "orderID": "12345678-1234-1234-1234-123456789012",
    "clOrdID": "client-order-id-123",
    "symbol": "BTCUSD",
    "side": "Buy",
    "priceEp": 88000000,
    "orderQty": 1500,
    "ordStatus": "New",
    "actionTimeNs": 1234567890000000000
  }
}
```

### Cancel Order

**Endpoint:** `DELETE /orders`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "bizError": 0,
    "orderID": "12345678-1234-1234-1234-123456789012",
    "clOrdID": "client-order-id-123",
    "symbol": "BTCUSD",
    "side": "Buy",
    "ordStatus": "Canceled",
    "actionTimeNs": 1234567890000000000
  }
}
```

### Query Open Orders

**Endpoint:** `GET /orders/activeList`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "rows": [
      {
        "bizError": 0,
        "orderID": "12345678-1234-1234-1234-123456789012",
        "clOrdID": "client-order-id-123",
        "symbol": "BTCUSD",
        "side": "Buy",
        "ordType": "Limit",
        "priceEp": 87700000,
        "price": 8770.0,
        "orderQty": 1000,
        "displayQty": 1000,
        "timeInForce": "GoodTillCancel",
        "reduceOnly": false,
        "closeOnTrigger": false,
        "takeProfitEp": 0,
        "stopLossEp": 0,
        "triggerType": "UNSPECIFIED",
        "stopPxEp": 0,
        "ordStatus": "New",
        "cumQty": 0,
        "leavesQty": 1000,
        "cumValueEv": 0,
        "leavesValueEv": 11404,
        "avgPriceEp": 0,
        "createTimeNs": 1234567890000000000,
        "actionTimeNs": 1234567890000000000
      }
    ]
  }
}
```

### Query Closed Orders

**Endpoint:** `GET /exchange/order/list`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "total": 50,
    "rows": [
      {
        "orderID": "12345678-1234-1234-1234-123456789012",
        "clOrdID": "client-order-id-123",
        "symbol": "BTCUSD",
        "side": "Buy",
        "ordType": "Limit",
        "priceEp": 87700000,
        "price": 8770.0,
        "orderQty": 1000,
        "ordStatus": "Filled",
        "cumQty": 1000,
        "leavesQty": 0,
        "cumValueEv": 11404,
        "avgPriceEp": 87700000,
        "createTimeNs": 1234567890000000000,
        "actionTimeNs": 1234567891000000000
      }
    ]
  }
}
```

### Query Trades

**Endpoint:** `GET /exchange/order/trade`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "total": 100,
    "rows": [
      {
        "transactTimeNs": 1234567890000000000,
        "symbol": "BTCUSD",
        "currency": "BTC",
        "action": "New",
        "side": "Buy",
        "tradeType": "Trade",
        "execQty": 1000,
        "execPriceEp": 87700000,
        "execValueEv": 11404,
        "feeRateEr": 75000,
        "execFeeEv": 9,
        "orderID": "12345678-1234-1234-1234-123456789012",
        "clOrdID": "client-order-id-123",
        "execID": "98765432-1234-1234-1234-123456789012"
      }
    ]
  }
}
```

## Account Responses

### Query Spot Wallets

**Endpoint:** `GET /spot/wallets`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "balances": [
      {
        "currency": "BTC",
        "balanceEv": 100000000,
        "lockedTradingBalanceEv": 10000000,
        "lockedWithdrawEv": 0,
        "lastUpdateTimeNs": 1234567890000000000,
        "walletVid": 0
      },
      {
        "currency": "USDT",
        "balanceEv": 1000000000000,
        "lockedTradingBalanceEv": 100000000000,
        "lockedWithdrawEv": 0,
        "lastUpdateTimeNs": 1234567890000000000,
        "walletVid": 0
      }
    ]
  }
}
```

**Balance Fields:**
- `balanceEv`: Total balance (scaled)
- `lockedTradingBalanceEv`: Locked in orders (scaled)
- `lockedWithdrawEv`: Locked for withdrawal (scaled)
- Available balance = `balanceEv - lockedTradingBalanceEv - lockedWithdrawEv`

### Query Contract Account & Positions

**Endpoint:** `GET /accounts/accountPositions`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "account": {
      "accountId": 123456,
      "currency": "BTC",
      "accountBalanceEv": 100000000,
      "totalUsedBalanceEv": 20000000,
      "bonusBalanceEv": 0
    },
    "positions": [
      {
        "accountID": 123456,
        "symbol": "BTCUSD",
        "currency": "BTC",
        "side": "Buy",
        "positionStatus": "Normal",
        "crossMargin": false,
        "leverageEr": 1000000,
        "leverage": 10.0,
        "initMarginReqEr": 1000000,
        "maintMarginReqEr": 500000,
        "riskLimitEv": 10000000000,
        "size": 1000,
        "value": 11404,
        "valueEv": 11404,
        "avgEntryPriceEp": 87700000,
        "avgEntryPrice": 8770.0,
        "posCostEv": 1140,
        "assignedPosBalanceEv": 1140,
        "bankruptCommEv": 0,
        "bankruptPriceEp": 79730000,
        "positionMarginEv": 1140,
        "liquidationPriceEp": 80230000,
        "deleveragePercentileEr": 0,
        "buyValueToCostEr": 10114,
        "sellValueToCostEr": 10126,
        "markPriceEp": 87700000,
        "markPrice": 8770.0,
        "estLiquidationPriceEp": 80230000,
        "unrealisedPnlEv": 0,
        "unrealisedPnl": 0.0,
        "realisedPnlEv": 0,
        "realisedPnl": 0.0,
        "cumRealisedPnlEv": 0
      }
    ]
  }
}
```

**Key Position Fields:**
- `side`: `"Buy"` (long) or `"Sell"` (short)
- `size`: Position size
- `avgEntryPriceEp`: Average entry price (scaled)
- `liquidationPriceEp`: Liquidation price (scaled)
- `unrealisedPnlEv`: Unrealized profit/loss (scaled)
- `leverageEr`: Leverage (scaled, sign indicates margin mode)
  - Positive = Isolated margin
  - Zero/Negative = Cross margin

### Transfer Between Accounts

**Endpoint:** `POST /assets/transfer`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "linkKey": "transfer-12345678-1234-1234-1234-123456789012",
    "userId": 123456,
    "currency": "BTC",
    "amountEv": 10000000,
    "side": 2,
    "status": 10
  }
}
```

**Fields:**
- `linkKey`: Transfer ID
- `amountEv`: Transfer amount (scaled)
- `side`: Direction (1 = futures to spot, 2 = spot to futures)
- `status`: 10 = success

## Position Management Responses

### Set Leverage

**Endpoint:** `PUT /positions/leverage`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "symbol": "BTCUSD",
    "leverageEr": 2000000,
    "leverage": 20.0,
    "riskLimitEv": 10000000000
  }
}
```

### Set Risk Limit

**Endpoint:** `PUT /positions/riskLimit`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "symbol": "BTCUSD",
    "riskLimitEv": 20000000000,
    "leverageEr": 2000000
  }
}
```

### Assign Position Balance

**Endpoint:** `POST /positions/assign`

```json
{
  "code": 0,
  "msg": "OK",
  "data": {
    "symbol": "BTCUSD",
    "posBalanceEv": 5000000,
    "leverage": 20.0
  }
}
```

## Field Naming Conventions

Phemex uses suffixes to indicate scaled values:

| Suffix | Meaning | Scaling |
|--------|---------|---------|
| `Ep` | Scaled price | Price × 10^priceScale |
| `Er` | Scaled ratio | Ratio × 10^ratioScale |
| `Ev` | Scaled value | Value × 10^valueScale |

**Example Conversions:**

For `BTCUSD` with `priceScale=4`:
- `priceEp=87700000` → actual price = 87700000 / 10^4 = 8770.0 USD

For `BTC` with `valueScale=8`:
- `balanceEv=100000000` → actual balance = 100000000 / 10^8 = 1.0 BTC

For leverage with `ratioScale=8`:
- `leverageEr=1000000` → actual leverage = 1000000 / 10^8 = 0.01x (cross margin)
- `leverageEr=2000000` → actual leverage = 2000000 / 10^8 = 0.02x (or 20x isolated)

## Timestamp Formats

Phemex uses nanoseconds for most timestamps:

| Field | Format | Example |
|-------|--------|---------|
| `timestamp` | Nanoseconds | 1234567890000000000 |
| `createTimeNs` | Nanoseconds | 1234567890000000000 |
| `actionTimeNs` | Nanoseconds | 1234567890000000000 |
| Kline timestamp | Seconds | 1590019200 |

**Conversion to seconds:**
```rust
let seconds = timestamp_ns / 1_000_000_000;
```
