# Bitget API Response Formats

## Standard Response Structure

All Bitget API responses follow a consistent JSON structure:

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": { ... }
}
```

### Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `code` | string | Response code ("00000" = success) |
| `msg` | string | Response message ("success" or error description) |
| `requestTime` | number | Server timestamp in milliseconds |
| `data` | object/array | Actual response data (structure varies by endpoint) |

## Success Response

### Code: "00000"

All successful responses return code `"00000"` with `msg: "success"`.

**Example:**
```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "orderId": "1234567890",
    "clientOrderId": "custom_id_123"
  }
}
```

## Error Response

### Common Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| 00000 | 200 | Success |
| 40001 | 400 | Invalid parameter |
| 40002 | 400 | Parameter error |
| 40006 | 400 | Invalid request |
| 40015 | 401 | Invalid timestamp |
| 40016 | 401 | Invalid API key |
| 40017 | 401 | Invalid passphrase |
| 40018 | 401 | Invalid signature |
| 40019 | 401 | Request expired |
| 40020 | 403 | Permission denied |
| 40029 | 400 | Invalid order status |
| 40725 | 503 | System under maintenance |
| 40808 | 400 | Parameter verification exception |
| 43019 | 400 | Order does not exist |
| 43115 | 400 | Insufficient balance |
| 45001 | 503 | Backend maintenance |
| 50001 | 500 | Internal server error |
| 50004 | 500 | Service unavailable |

### Error Response Format

```json
{
  "code": "40018",
  "msg": "Invalid ACCESS-SIGN",
  "requestTime": 1695806875837,
  "data": null
}
```

## HTTP Status Codes

| Status | Meaning |
|--------|---------|
| 200 | Success |
| 400 | Bad Request (invalid parameters) |
| 401 | Unauthorized (authentication failure) |
| 403 | Forbidden (permission denied) |
| 404 | Not Found |
| 429 | Too Many Requests (rate limit exceeded) |
| 500 | Internal Server Error |
| 503 | Service Unavailable (maintenance) |

**Note:** HTTP 200 doesn't guarantee success - always check the `code` field.

## Market Data Response Formats

### Get Server Time

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "serverTime": "1695806875837"
  }
}
```

### Get Trading Symbols (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "symbol": "BTCUSDT_SPBL",
      "baseCoin": "BTC",
      "quoteCoin": "USDT",
      "minTradeAmount": "0.0001",
      "maxTradeAmount": "10000000",
      "takerFeeRate": "0.002",
      "makerFeeRate": "0.002",
      "pricePrecision": "2",
      "quantityPrecision": "4",
      "quotePrecision": "8",
      "status": "online",
      "minTradeUSDT": "5",
      "buyLimitPriceRatio": "0.05",
      "sellLimitPriceRatio": "0.05"
    }
  ]
}
```

### Get Ticker (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "symbol": "BTCUSDT_SPBL",
    "high24h": "52000.00",
    "low24h": "49000.00",
    "close": "50500.00",
    "quoteVol": "150000000.50",
    "baseVol": "3000.5500",
    "usdtVol": "150000000.50",
    "ts": "1695806875837",
    "bidPr": "50499.50",
    "askPr": "50500.50",
    "bidSz": "0.5000",
    "askSz": "0.3000",
    "openUtc": "50000.00",
    "changeUtc24h": "0.01",
    "change24h": "0.0102"
  }
}
```

### Get Order Book (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "asks": [
      ["50500.50", "0.1000"],
      ["50501.00", "0.2000"],
      ["50502.00", "0.3000"]
    ],
    "bids": [
      ["50499.50", "0.1500"],
      ["50499.00", "0.2500"],
      ["50498.00", "0.3500"]
    ],
    "ts": "1695806875837"
  }
}
```

**Note:** Each entry is `[price, quantity]` array.

### Get Candles (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    [
      "1695806400000",
      "50000.00",
      "50500.00",
      "49800.00",
      "50200.00",
      "100.5000",
      "5025000.00"
    ],
    [
      "1695806340000",
      "49900.00",
      "50100.00",
      "49850.00",
      "50000.00",
      "95.3000",
      "4756850.00"
    ]
  ]
}
```

**Array format:** `[timestamp, open, high, low, close, baseVolume, quoteVolume]`

### Get Recent Trades (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "symbol": "BTCUSDT_SPBL",
      "tradeId": "1234567890",
      "side": "buy",
      "fillPrice": "50500.00",
      "fillQuantity": "0.1000",
      "fillTime": "1695806875000"
    }
  ]
}
```

## Futures Market Data Response Formats

### Get Futures Symbols

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "symbol": "BTCUSDT_UMCBL",
      "baseCoin": "BTC",
      "quoteCoin": "USDT",
      "buyLimitPriceRatio": "0.01",
      "sellLimitPriceRatio": "0.01",
      "feeRateUpRatio": "0.005",
      "makerFeeRate": "0.0002",
      "takerFeeRate": "0.0006",
      "openCostUpRatio": "0.01",
      "supportMarginCoins": ["USDT"],
      "minTradeNum": "0.001",
      "priceEndStep": "0.5",
      "volumePlace": "3",
      "sizeMultiplier": "0.001"
    }
  ]
}
```

### Get Futures Ticker

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "symbol": "BTCUSDT_UMCBL",
    "last": "50500.00",
    "bestAsk": "50500.50",
    "bestBid": "50499.50",
    "bidSz": "100",
    "askSz": "150",
    "high24h": "52000.00",
    "low24h": "49000.00",
    "timestamp": "1695806875837",
    "priceChangePercent": "0.0102",
    "baseVolume": "50000.000",
    "quoteVolume": "2525000000.00",
    "usdtVolume": "2525000000.00",
    "openUtc": "50000.00",
    "chgUtc": "0.01",
    "indexPrice": "50505.50",
    "fundingRate": "0.0001",
    "holdingAmount": "100000.000"
  }
}
```

### Get Funding Rate

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "symbol": "BTCUSDT_UMCBL",
    "fundingRate": "0.0001",
    "fundingTime": "1695808000000"
  }
}
```

## Trading Response Formats

### Place Order (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "orderId": "1098394857234",
    "clientOrderId": "custom_id_123"
  }
}
```

### Place Order (Futures)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "orderId": "1098394857234",
    "clientOid": "custom_id_123"
  }
}
```

### Cancel Order

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "orderId": "1098394857234",
    "clientOrderId": "custom_id_123"
  }
}
```

### Batch Place Orders

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "successList": [
      {
        "orderId": "1098394857234",
        "clientOrderId": "id1"
      },
      {
        "orderId": "1098394857235",
        "clientOrderId": "id2"
      }
    ],
    "failureList": [
      {
        "clientOrderId": "id3",
        "errorMsg": "Insufficient balance",
        "errorCode": "43115"
      }
    ]
  }
}
```

### Get Order Details (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "orderId": "1098394857234",
    "clientOrderId": "custom_id_123",
    "symbol": "BTCUSDT_SPBL",
    "side": "buy",
    "orderType": "limit",
    "price": "50000.00",
    "quantity": "0.1000",
    "fillPrice": "50000.00",
    "fillQuantity": "0.1000",
    "fillTotalAmount": "5000.00",
    "enterPointSource": "API",
    "status": "full_fill",
    "cTime": "1695806875000",
    "uTime": "1695806876000"
  }
}
```

**Order Status Values:**
- `init`: Initialized
- `new`: New (unfilled)
- `partial_fill`: Partially filled
- `full_fill`: Fully filled
- `canceled`: Cancelled
- `failed`: Failed

### Get Open Orders (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "orderId": "1098394857234",
      "clientOrderId": "custom_id_123",
      "symbol": "BTCUSDT_SPBL",
      "side": "buy",
      "orderType": "limit",
      "price": "49000.00",
      "quantity": "0.2000",
      "fillPrice": "0.00",
      "fillQuantity": "0.0000",
      "status": "new",
      "cTime": "1695806875000"
    }
  ]
}
```

### Get Fills/Trades (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "orderId": "1098394857234",
      "tradeId": "5678901234",
      "symbol": "BTCUSDT_SPBL",
      "side": "buy",
      "orderType": "limit",
      "fillPrice": "50000.00",
      "fillQuantity": "0.1000",
      "fillTotalAmount": "5000.00",
      "feeCoin": "USDT",
      "fees": "10.00",
      "cTime": "1695806876000"
    }
  ]
}
```

## Account Response Formats

### Get Account Info

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "userId": "123456789",
    "inviterId": "0",
    "ips": "",
    "authorities": ["trader", "spotTrader", "marginTrader"],
    "parentId": "0",
    "trader": true
  }
}
```

### Get Account Assets (Spot)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "coin": "USDT",
      "available": "10000.50",
      "frozen": "500.00",
      "locked": "0.00",
      "uTime": "1695806875000"
    },
    {
      "coin": "BTC",
      "available": "0.5000",
      "frozen": "0.0000",
      "locked": "0.0000",
      "uTime": "1695806875000"
    }
  ]
}
```

### Get Futures Account

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "marginCoin": "USDT",
    "locked": "500.00",
    "available": "9500.50",
    "crossMaxAvailable": "9500.50",
    "fixedMaxAvailable": "9500.50",
    "maxTransferOut": "9000.00",
    "equity": "10500.00",
    "usdtEquity": "10500.00",
    "btcEquity": "0.2079",
    "crossRiskRate": "0.0500",
    "unrealizedPL": "500.00",
    "bonus": "0.00"
  }
}
```

### Get User Fee Rate

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "symbol": "BTCUSDT",
    "makerFeeRate": "0.002",
    "takerFeeRate": "0.006"
  }
}
```

## Position Response Formats

### Get Single Position (Futures)

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "marginCoin": "USDT",
      "symbol": "BTCUSDT_UMCBL",
      "holdSide": "long",
      "openDelegateCount": "2",
      "margin": "1000.00",
      "available": "0.1000",
      "locked": "0.0000",
      "total": "0.1000",
      "leverage": "10",
      "achievedProfits": "50.00",
      "averageOpenPrice": "50000.00",
      "marginMode": "crossed",
      "holdMode": "single_hold",
      "unrealizedPL": "100.00",
      "liquidationPrice": "45000.00",
      "keepMarginRate": "0.004",
      "marketPrice": "51000.00",
      "cTime": "1695800000000",
      "uTime": "1695806875000"
    }
  ]
}
```

**Hold Sides:**
- `long`: Long position
- `short`: Short position
- `net`: Net position (one-way mode)

**Margin Modes:**
- `crossed`: Cross margin
- `fixed`: Isolated margin

**Hold Modes:**
- `single_hold`: One-way position mode
- `double_hold`: Hedge mode

### Get All Positions

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": [
    {
      "marginCoin": "USDT",
      "symbol": "BTCUSDT_UMCBL",
      "holdSide": "long",
      "total": "0.1000",
      "available": "0.1000",
      "averageOpenPrice": "50000.00",
      "unrealizedPL": "100.00",
      "leverage": "10"
    },
    {
      "marginCoin": "USDT",
      "symbol": "ETHUSDT_UMCBL",
      "holdSide": "short",
      "total": "1.0000",
      "available": "1.0000",
      "averageOpenPrice": "3000.00",
      "unrealizedPL": "-50.00",
      "leverage": "5"
    }
  ]
}
```

## WebSocket Response Formats

### Subscription Success

```json
{
  "event": "subscribe",
  "arg": {
    "instType": "SPOT",
    "channel": "ticker",
    "instId": "BTCUSDT"
  }
}
```

### Subscription Error

```json
{
  "event": "error",
  "code": "60012",
  "msg": "Invalid channel"
}
```

### Ticker Update (WebSocket)

```json
{
  "action": "snapshot",
  "arg": {
    "instType": "SPOT",
    "channel": "ticker",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "last": "50500.00",
      "open24h": "50000.00",
      "high24h": "52000.00",
      "low24h": "49000.00",
      "bestBid": "50499.50",
      "bestAsk": "50500.50",
      "baseVolume": "3000.5500",
      "quoteVolume": "150000000.50",
      "ts": "1695806875837"
    }
  ]
}
```

### Order Update (WebSocket)

```json
{
  "action": "update",
  "arg": {
    "instType": "SPOT",
    "channel": "orders",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "ordId": "1098394857234",
      "clOrdId": "custom_id_123",
      "px": "50000.00",
      "sz": "0.1000",
      "notionalUsd": "5000.00",
      "ordType": "limit",
      "side": "buy",
      "fillPx": "50000.00",
      "fillSz": "0.1000",
      "state": "filled",
      "accFillSz": "0.1000",
      "uTime": "1695806876000",
      "cTime": "1695806875000"
    }
  ]
}
```

## Pagination Response Format

For endpoints supporting pagination (V2 APIs):

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": {
    "resultList": [
      { /* item 1 */ },
      { /* item 2 */ }
    ],
    "endId": "1098394857234",
    "hasMore": true
  }
}
```

**Pagination Fields:**
- `resultList`: Array of results
- `endId`: Last item ID (use as `idLessThan` for next page)
- `hasMore`: `true` if more data available

## Empty Response

When no data is available:

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": []
}
```

or

```json
{
  "code": "00000",
  "msg": "success",
  "requestTime": 1695806875837,
  "data": null
}
```

## Sources

- [Bitget API Introduction](https://www.bitget.com/api-doc/common/intro)
- [Bitget Request Interaction](https://www.bitget.com/api-doc/common/signature-samaple/interaction)
- [Bitget Spot API Docs](https://bitgetlimited.github.io/apidoc/en/spot/)
- [Bitget Futures API Docs](https://bitgetlimited.github.io/apidoc/en/mix/)
- [Bitget REST API Error Codes](https://www.bitget.com/api-doc/spot/error-code/restapi)
