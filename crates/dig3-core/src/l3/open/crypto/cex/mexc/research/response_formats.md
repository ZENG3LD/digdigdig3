# MEXC API Response Formats

## Response Structure

MEXC API uses different response formats for Spot and Futures endpoints.

---

## Spot API Responses

### Success Response Format

Most Spot API endpoints return data directly without a wrapper:

```json
{
  "symbol": "BTCUSDT",
  "price": "93200.50"
}
```

Or for array responses:
```json
[
  {
    "symbol": "BTCUSDT",
    "price": "93200.50"
  },
  {
    "symbol": "ETHUSDT",
    "price": "2100.30"
  }
]
```

### Error Response Format

```json
{
  "code": 10001,
  "msg": "Missing required parameter"
}
```

---

## Futures API Responses

### Success Response Format

Futures endpoints use a wrapper structure:

```json
{
  "success": true,
  "code": 0,
  "data": {
    // Actual response data
  }
}
```

### Error Response Format

```json
{
  "success": false,
  "code": 1002,
  "msg": "Contract not allow place order!"
}
```

---

## Market Data Responses

### Ping
```json
{}
```

### Server Time
```json
{
  "serverTime": 1640080800000
}
```

### Exchange Information
```json
{
  "timezone": "UTC",
  "serverTime": 1640080800000,
  "rateLimits": [
    {
      "rateLimitType": "REQUEST_WEIGHT",
      "interval": "MINUTE",
      "intervalNum": 1,
      "limit": 1200
    },
    {
      "rateLimitType": "ORDERS",
      "interval": "SECOND",
      "intervalNum": 10,
      "limit": 100
    }
  ],
  "exchangeFilters": [],
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "status": "ENABLED",
      "baseAsset": "BTC",
      "baseAssetPrecision": 8,
      "quoteAsset": "USDT",
      "quotePrecision": 8,
      "quoteAssetPrecision": 8,
      "baseCommissionPrecision": 8,
      "quoteCommissionPrecision": 8,
      "orderTypes": [
        "LIMIT",
        "MARKET",
        "LIMIT_MAKER"
      ],
      "quoteOrderQtyMarketAllowed": true,
      "isSpotTradingAllowed": true,
      "isMarginTradingAllowed": false,
      "quoteAmountPrecision": "8",
      "baseSizePrecision": "0.00001",
      "permissions": [
        "SPOT"
      ],
      "filters": [
        {
          "filterType": "PERCENT_PRICE_BY_SIDE",
          "bidMultiplierUp": "5",
          "bidMultiplierDown": "0.2",
          "askMultiplierUp": "5",
          "askMultiplierDown": "0.2"
        },
        {
          "filterType": "LOT_SIZE",
          "minQty": "0.00001",
          "maxQty": "9000000",
          "stepSize": "0.00001"
        },
        {
          "filterType": "MIN_NOTIONAL",
          "minNotional": "5"
        }
      ],
      "maxQuoteAmount": "5000000",
      "makerCommission": "0.002",
      "takerCommission": "0.002"
    }
  ]
}
```

### Order Book Depth
```json
{
  "lastUpdateId": 123456789,
  "bids": [
    [
      "93220.00",    // Price
      "0.5"          // Quantity
    ],
    [
      "93210.00",
      "1.2"
    ]
  ],
  "asks": [
    [
      "93230.00",
      "0.8"
    ],
    [
      "93240.00",
      "2.1"
    ]
  ]
}
```

### Recent Trades
```json
[
  {
    "id": 28457,
    "price": "93220.00",
    "qty": "0.04438243",
    "quoteQty": "4137.85",
    "time": 1736409765051,
    "isBuyerMaker": false,
    "isBestMatch": true
  }
]
```

### Aggregate Trades
```json
[
  {
    "a": 26129,         // Aggregate trade ID
    "p": "93220.00",    // Price
    "q": "1.5",         // Quantity
    "f": 100,           // First trade ID
    "l": 105,           // Last trade ID
    "T": 1498793709153, // Timestamp
    "m": true,          // Was the buyer the maker?
    "M": true           // Was the trade the best price match?
  }
]
```

### Klines/Candlesticks
```json
[
  [
    1640804880000,     // Open time
    "47482.36",        // Open price
    "47482.36",        // High price
    "47416.57",        // Low price
    "47436.1",         // Close price
    "3.550717",        // Volume
    1640804940000,     // Close time
    "168387.3"         // Quote asset volume
  ]
]
```

### Average Price
```json
{
  "mins": 5,
  "price": "93150.25"
}
```

### 24hr Ticker
```json
{
  "symbol": "BTCUSDT",
  "priceChange": "1200.50",
  "priceChangePercent": "1.3",
  "weightedAvgPrice": "93000.00",
  "prevClosePrice": "92000.00",
  "lastPrice": "93200.50",
  "lastQty": "0.5",
  "bidPrice": "93200.00",
  "bidQty": "1.2",
  "askPrice": "93210.00",
  "askQty": "0.8",
  "openPrice": "92000.00",
  "highPrice": "93500.00",
  "lowPrice": "91800.00",
  "volume": "12345.67",
  "quoteVolume": "1147893256.23",
  "openTime": 1640080800000,
  "closeTime": 1640167200000,
  "firstId": 100,
  "lastId": 200,
  "count": 100
}
```

### Price Ticker
```json
{
  "symbol": "BTCUSDT",
  "price": "93200.50"
}
```

Or array for all symbols:
```json
[
  {
    "symbol": "BTCUSDT",
    "price": "93200.50"
  },
  {
    "symbol": "ETHUSDT",
    "price": "2100.30"
  }
]
```

### Book Ticker
```json
{
  "symbol": "BTCUSDT",
  "bidPrice": "93200.00",
  "bidQty": "1.5",
  "askPrice": "93210.00",
  "askQty": "2.3"
}
```

---

## Trading Responses

### New Order
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

### New Order (Futures)
```json
{
  "success": true,
  "code": 0,
  "data": {
    "orderId": "739113577038255616",
    "ts": 1761888808839
  }
}
```

### Cancel Order
```json
{
  "symbol": "BTCUSDT",
  "origClientOrderId": "myOrder1",
  "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
  "orderListId": -1,
  "clientOrderId": "cancelMyOrder1",
  "price": "90000",
  "origQty": "0.1",
  "executedQty": "0",
  "cummulativeQuoteQty": "0",
  "status": "CANCELED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY"
}
```

### Query Order
```json
{
  "symbol": "BTCUSDT",
  "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
  "orderListId": -1,
  "clientOrderId": "myOrder1",
  "price": "90000",
  "origQty": "0.1",
  "executedQty": "0.05",
  "cummulativeQuoteQty": "4500",
  "status": "PARTIALLY_FILLED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "BUY",
  "stopPrice": "0",
  "icebergQty": "0",
  "time": 1640080800000,
  "updateTime": 1640084400000,
  "isWorking": true,
  "origQuoteOrderQty": "0"
}
```

### Open Orders
```json
[
  {
    "symbol": "BTCUSDT",
    "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
    "price": "90000",
    "origQty": "0.1",
    "executedQty": "0",
    "status": "NEW",
    "type": "LIMIT",
    "side": "BUY",
    "time": 1640080800000
  }
]
```

---

## Account Responses

### Account Information
```json
{
  "makerCommission": 20,
  "takerCommission": 20,
  "buyerCommission": 0,
  "sellerCommission": 0,
  "canTrade": true,
  "canWithdraw": true,
  "canDeposit": true,
  "updateTime": 1640080800000,
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "0.5",
      "locked": "0.1"
    },
    {
      "asset": "USDT",
      "free": "10000.0",
      "locked": "500.0"
    },
    {
      "asset": "MX",
      "free": "3",
      "locked": "0"
    }
  ],
  "permissions": [
    "SPOT"
  ]
}
```

### Account Trades
```json
[
  {
    "symbol": "BTCUSDT",
    "id": 28457,
    "orderId": "91d9a3c4a3ab40c7ba76c98598dcf85a",
    "orderListId": -1,
    "price": "93220.00",
    "qty": "0.04438243",
    "quoteQty": "4137.85",
    "commission": "0.0000443824",
    "commissionAsset": "BTC",
    "time": 1736409765051,
    "isBuyer": true,
    "isMaker": false,
    "isBestMatch": true
  }
]
```

### Trade Fee
```json
[
  {
    "symbol": "BTCUSDT",
    "makerCommission": "0.002",
    "takerCommission": "0.002"
  }
]
```

---

## Futures Account Responses

### Position Information
```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "positionId": 123456,
      "symbol": "BTC_USDT",
      "positionType": 1,
      "holdVol": "1.5",
      "holdAvgPrice": "92000.00",
      "leverage": 10,
      "liquidatePrice": "83000.00",
      "oim": "13800.00",
      "im": "13800.00",
      "holdFee": "5.2",
      "realised": "0",
      "autoAddIm": false
    }
  ]
}
```

### Futures Account Balance
```json
{
  "success": true,
  "code": 0,
  "data": [
    {
      "currency": "USDT",
      "positionMargin": 13800.00,
      "availableBalance": 32809.85,
      "cashBalance": 32809.85,
      "frozenBalance": 0,
      "equity": 32809.85,
      "unrealized": 0,
      "bonus": 0,
      "availableCash": 32809.85,
      "availableOpen": 32809.85,
      "debtAmount": 0,
      "contributeMarginAmount": 0,
      "vcoinId": "128f589271cb4951b03e71e6323eb7be"
    }
  ]
}
```

---

## Wallet Responses

### All Coins Information
```json
[
  {
    "coin": "BTC",
    "name": "Bitcoin",
    "networkList": [
      {
        "network": "BTC",
        "coin": "BTC",
        "withdrawIntegerMultiple": "0.00000001",
        "isDefault": true,
        "depositEnable": true,
        "withdrawEnable": true,
        "withdrawFee": "0.0005",
        "withdrawMin": "0.001",
        "withdrawMax": "9000",
        "minConfirm": 1,
        "unLockConfirm": 2
      }
    ]
  }
]
```

### Deposit History
```json
[
  {
    "id": "abc123",
    "amount": "0.5",
    "coin": "BTC",
    "network": "BTC",
    "status": 1,
    "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
    "addressTag": "",
    "txId": "0x123...abc",
    "insertTime": 1640080800000,
    "confirmTimes": "2/2"
  }
]
```

### Withdrawal History
```json
[
  {
    "id": "def456",
    "amount": "0.3",
    "coin": "BTC",
    "network": "BTC",
    "status": 6,
    "address": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
    "addressTag": "",
    "txId": "0x456...def",
    "applyTime": 1640080800000,
    "withdrawOrderId": "client_order_123",
    "info": "",
    "confirmNo": 2,
    "transferType": 0
  }
]
```

---

## Field Descriptions

### Common Fields

| Field | Type | Description |
|-------|------|-------------|
| `symbol` | string | Trading pair (e.g., "BTCUSDT" for spot, "BTC_USDT" for futures) |
| `orderId` | string | Order ID assigned by MEXC |
| `clientOrderId` | string | Client-assigned order ID |
| `price` | string | Order/trade price (numeric string) |
| `origQty` | string | Original order quantity |
| `executedQty` | string | Quantity that has been filled |
| `status` | string | Order status (NEW, PARTIALLY_FILLED, FILLED, CANCELED, etc.) |
| `type` | string | Order type (LIMIT, MARKET, LIMIT_MAKER) |
| `side` | string | Order side (BUY, SELL) |
| `time` | long | Timestamp in milliseconds |
| `updateTime` | long | Last update timestamp in milliseconds |

### Order Status Values

- `NEW`: Order accepted by engine
- `PARTIALLY_FILLED`: Order partially filled
- `FILLED`: Order fully filled
- `CANCELED`: Order canceled by user
- `REJECTED`: Order rejected by engine
- `EXPIRED`: Order expired (e.g., IOC order not filled)

### Deposit/Withdrawal Status Values

**Deposit Status:**
- `0`: Pending
- `1`: Success
- `2`: Failed

**Withdrawal Status:**
- `0`: Email sent
- `1`: Cancelled
- `2`: Awaiting approval
- `3`: Rejected
- `4`: Processing
- `5`: Failure
- `6`: Completed

### Futures Position Type

- `1`: Long position
- `2`: Short position

---

## Error Response Codes

### Common Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 429 | Too Many Requests | Rate limit exceeded |
| 10001 | Missing required parameter | Required parameter not provided |
| 10002 | Invalid parameter | Parameter value is invalid |
| 10003 | Invalid API key | API key is incorrect or expired |
| 10004 | Invalid signature | Signature verification failed |
| 10073 | Invalid Request-Time | Timestamp is invalid |
| 10074 | Timestamp outside recvWindow | Request is too old |
| 1002 | Contract not allow place order | Trading not allowed for this contract |

---

## Data Types

### Numeric Precision

All numeric values are returned as **strings** to preserve precision:

```json
{
  "price": "93220.00",
  "quantity": "0.04438243"
}
```

**Important**: Always parse as decimal/float, never as integer.

### Timestamps

All timestamps are in **milliseconds** since UNIX epoch:

```json
{
  "serverTime": 1640080800000,
  "time": 1736409765051
}
```

### Booleans

Standard JSON boolean values:

```json
{
  "canTrade": true,
  "canWithdraw": false
}
```

---

## Response Size Limits

- Maximum response size varies by endpoint
- Large responses (e.g., all symbols, all orders) may be paginated
- Use `limit` parameter to control response size
- Some endpoints have maximum limit values (typically 1000)

---

## Notes

1. **Spot vs Futures**: Different response wrapper formats
2. **Numeric Strings**: All prices/quantities are strings for precision
3. **Timestamps**: Always in milliseconds, not seconds
4. **Empty Arrays**: Valid responses can be empty arrays `[]`
5. **Null Values**: Some fields may be `null` or omitted
6. **HTTP Status**: Successful requests return 200, errors return 4xx/5xx
7. **Error Format**: Errors include `code` and `msg` fields
