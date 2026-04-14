# Binance API Endpoints

## Base URLs

### Spot Trading

**Production:**
- `https://api.binance.com`
- `https://api-gcp.binance.com`
- `https://api1.binance.com` through `https://api4.binance.com`

**Market Data Only:**
- `https://data-api.binance.vision`

**Testnet (Demo Trading):**
- REST: `https://demo-api.binance.com`
- WS Trade: `wss://demo-ws-api.binance.com`
- WS Market: `wss://demo-stream.binance.com`

### Futures USDT-M

**Production:**
- `https://fapi.binance.com`

**Testnet:**
- REST: `https://demo-fapi.binance.com`
- WebSocket: `wss://fstream.binancefuture.com`

### Futures COIN-M (Delivery)

**Production:**
- `https://dapi.binance.com`

**Testnet:**
- `https://testnet.binancefuture.com`

---

## MarketData Trait Endpoints

### 1. Ping - Test Connectivity

**Spot:**
```
GET /api/v3/ping
```

**Futures USDT-M:**
```
GET /fapi/v1/ping
```

**Parameters:** NONE

**Weight:** 1

**Response:**
```json
{}
```

---

### 2. Server Time

**Spot:**
```
GET /api/v3/time
```

**Parameters:** NONE

**Weight:** 1

**Response:**
```json
{
  "serverTime": 1499827319559
}
```

---

### 3. Get Price - Symbol Price Ticker

**Spot:**
```
GET /api/v3/ticker/price
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | No | Trading pair (e.g., BTCUSDT) |
| symbols | STRING | No | Array of symbols (cannot combine with symbol) |
| symbolStatus | ENUM | No | TRADING, HALT, or BREAK |

**Weight:** 2 (single symbol) or 4 (multiple/all)

**Response (single symbol):**
```json
{
  "symbol": "LTCBTC",
  "price": "4.00000200"
}
```

**Response (multiple symbols):**
```json
[
  {
    "symbol": "LTCBTC",
    "price": "4.00000200"
  },
  {
    "symbol": "ETHBTC",
    "price": "0.07946600"
  }
]
```

---

### 4. Get Orderbook - Depth

**Spot:**
```
GET /api/v3/depth
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| limit | INT | No | Default 100, Max 5000 |
| symbolStatus | ENUM | No | TRADING, HALT, or BREAK |

**Weight:**
| Limit | Weight |
|-------|--------|
| 1-100 | 5 |
| 101-500 | 25 |
| 501-1000 | 50 |
| 1001-5000 | 250 |

**Response:**
```json
{
  "lastUpdateId": 1027024,
  "bids": [
    ["4.00000000", "431.00000000"]
  ],
  "asks": [
    ["4.00000200", "12.00000000"]
  ]
}
```

---

### 5. Get Klines - Candlestick Data

**Spot:**
```
GET /api/v3/klines
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| interval | ENUM | Yes | 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M |
| startTime | LONG | No | Start time in milliseconds |
| endTime | LONG | No | End time in milliseconds |
| timeZone | STRING | No | Default UTC, range -12:00 to +14:00 |
| limit | INT | No | Default 500, Max 1000 |

**Weight:** 2

**Response:**
```json
[
  [
    1499040000000,      // Open time
    "0.01634790",       // Open
    "0.80000000",       // High
    "0.01575800",       // Low
    "0.01577100",       // Close
    "148976.11427815",  // Volume
    1499644799999,      // Close time
    "2434.19055334",    // Quote asset volume
    308,                // Number of trades
    "1756.87402397",    // Taker buy base asset volume
    "28.46694368",      // Taker buy quote asset volume
    "0"                 // Unused field
  ]
]
```

---

### 6. Get Ticker - 24hr Statistics

**Spot:**
```
GET /api/v3/ticker/24hr
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | No | Trading pair |
| symbols | STRING | No | Array of symbols |
| type | ENUM | No | FULL or MINI |
| symbolStatus | ENUM | No | TRADING, HALT, or BREAK |

**Weight:**
| Scenario | Weight |
|----------|--------|
| Single symbol | 2 |
| 1-20 symbols | 2 |
| 21-100 symbols | 40 |
| 101+ symbols | 80 |
| No parameter | 80 |

**Response (FULL):**
```json
{
  "symbol": "BNBBTC",
  "priceChange": "-94.99999800",
  "priceChangePercent": "-95.960",
  "weightedAvgPrice": "0.29628482",
  "prevClosePrice": "0.10002000",
  "lastPrice": "4.00000200",
  "lastQty": "200.00000000",
  "bidPrice": "4.00000000",
  "bidQty": "100.00000000",
  "askPrice": "4.00000200",
  "askQty": "100.00000000",
  "openPrice": "99.00000000",
  "highPrice": "100.00000000",
  "lowPrice": "0.10000000",
  "volume": "8913.30000000",
  "quoteVolume": "15.30000000",
  "openTime": 1499783499040,
  "closeTime": 1499869899040,
  "firstId": 28385,
  "lastId": 28460,
  "count": 76
}
```

---

## Trading Trait Endpoints

### 1. Market Order

**Spot:**
```
POST /api/v3/order
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| side | ENUM | Yes | BUY or SELL |
| type | ENUM | Yes | MARKET |
| quantity | DECIMAL | No | Order quantity (or use quoteOrderQty) |
| quoteOrderQty | DECIMAL | No | Quote asset quantity |
| newClientOrderId | STRING | No | Unique order ID |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

**Weight:** 1

**Response:**
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

---

### 2. Limit Order

**Spot:**
```
POST /api/v3/order
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| side | ENUM | Yes | BUY or SELL |
| type | ENUM | Yes | LIMIT |
| quantity | DECIMAL | Yes | Order quantity |
| price | DECIMAL | Yes | Limit price |
| timeInForce | ENUM | Yes | GTC, IOC, FOK |
| newClientOrderId | STRING | No | Unique order ID |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

**Weight:** 1

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "orderId": 28,
  "orderListId": -1,
  "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
  "transactTime": 1507725176595,
  "price": "1.00000000",
  "origQty": "10.00000000",
  "executedQty": "10.00000000",
  "cummulativeQuoteQty": "10.00000000",
  "status": "FILLED",
  "timeInForce": "GTC",
  "type": "LIMIT",
  "side": "SELL",
  "workingTime": 1507725176595,
  "selfTradePreventionMode": "NONE"
}
```

---

### 3. Cancel Order

**Spot:**
```
DELETE /api/v3/order
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| orderId | LONG | No* | Order ID |
| origClientOrderId | STRING | No* | Client order ID |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

*Either orderId or origClientOrderId must be sent

**Weight:** 1

**Response:**
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

### 4. Get Order - Query Order Status

**Spot:**
```
GET /api/v3/order
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| orderId | LONG | No* | Order ID |
| origClientOrderId | STRING | No* | Client order ID |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

*Either orderId or origClientOrderId must be sent

**Weight:** 4

**Response:**
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
  "icebergQty": "0.0",
  "time": 1499827319559,
  "updateTime": 1499827319559,
  "isWorking": true,
  "workingTime": 1499827319559,
  "origQuoteOrderQty": "0.000000",
  "selfTradePreventionMode": "NONE"
}
```

**Status Values:**
- NEW
- PARTIALLY_FILLED
- FILLED
- CANCELED
- PENDING_CANCEL
- REJECTED
- EXPIRED
- EXPIRED_IN_MATCH

---

### 5. Get Open Orders - Query All Open Orders

**Spot:**
```
GET /api/v3/openOrders
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | No | Trading pair (if omitted, returns all symbols) |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

**Weight:** 6 (with symbol) or 80 (without symbol)

**Response:**
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
    "workingTime": 1499827319559,
    "origQuoteOrderQty": "0.000000",
    "selfTradePreventionMode": "NONE"
  }
]
```

---

## Account Trait Endpoints

### 1. Get Balance & Get Account Info

**Spot:**
```
GET /api/v3/account
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| omitZeroBalances | BOOLEAN | No | Excludes zero-balance assets (default: false) |
| recvWindow | LONG | No | Max 60000 |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

**Weight:** 20

**Response:**
```json
{
  "makerCommission": 15,
  "takerCommission": 15,
  "buyerCommission": 0,
  "sellerCommission": 0,
  "commissionRates": {
    "maker": "0.00150000",
    "taker": "0.00150000",
    "buyer": "0.00000000",
    "seller": "0.00000000"
  },
  "canTrade": true,
  "canWithdraw": true,
  "canDeposit": true,
  "brokered": false,
  "requireSelfTradePrevention": false,
  "preventSor": false,
  "updateTime": 123456789,
  "accountType": "SPOT",
  "balances": [
    {
      "asset": "BTC",
      "free": "4723846.89208129",
      "locked": "0.00000000"
    },
    {
      "asset": "LTC",
      "free": "4763368.68006011",
      "locked": "0.00000000"
    }
  ],
  "permissions": ["SPOT"],
  "uid": 354937868
}
```

**Balance Fields:**
- `asset`: Asset symbol (e.g., BTC, ETH)
- `free`: Available balance
- `locked`: Locked balance (in orders)

---

## Positions Trait Endpoints (Futures Only)

### 1. Get Positions - Position Risk

**Futures USDT-M:**
```
GET /fapi/v2/positionRisk
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | No | Filter by trading pair |
| recvWindow | LONG | No | Request validity window |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

**Weight:** 5

**Response (One-way mode):**
```json
[
  {
    "entryPrice": "0.00000",
    "breakEvenPrice": "0.0",
    "marginType": "isolated",
    "isAutoAddMargin": "false",
    "isolatedMargin": "0.00000000",
    "leverage": "10",
    "liquidationPrice": "0",
    "markPrice": "6679.50671178",
    "maxNotionalValue": "20000000",
    "positionAmt": "0.000",
    "notional": "0",
    "isolatedWallet": "0",
    "symbol": "BTCUSDT",
    "unRealizedProfit": "0.00000000",
    "positionSide": "BOTH",
    "updateTime": 0
  }
]
```

**Response (Hedge mode):**
```json
[
  {
    "symbol": "BTCUSDT",
    "positionAmt": "0.001",
    "entryPrice": "22185.2",
    "breakEvenPrice": "0.0",
    "markPrice": "21123.05052574",
    "unRealizedProfit": "-1.06214947",
    "liquidationPrice": "19731.45529116",
    "leverage": "4",
    "maxNotionalValue": "100000000",
    "marginType": "cross",
    "isolatedMargin": "0.00000000",
    "isAutoAddMargin": "false",
    "positionSide": "LONG",
    "notional": "21.12305052",
    "isolatedWallet": "0",
    "updateTime": 1655217461579
  }
]
```

---

### 2. Get Funding Rate

**Futures USDT-M:**
```
GET /fapi/v1/fundingRate
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | No | Trading pair |
| startTime | LONG | No | Start time in milliseconds (INCLUSIVE) |
| endTime | LONG | No | End time in milliseconds (INCLUSIVE) |
| limit | INT | No | Default 100, Max 1000 |

**Weight:** Shares 500/5min/IP rate limit with GET /fapi/v1/fundingInfo

**Response:**
```json
[
  {
    "symbol": "BTCUSDT",
    "fundingRate": "-0.03750000",
    "fundingTime": 1570608000000,
    "markPrice": "34287.54619963"
  }
]
```

**Notes:**
- If startTime and endTime are not sent, returns most recent 200 records
- Results ordered in ascending sequence

---

### 3. Set Leverage

**Futures USDT-M:**
```
POST /fapi/v1/leverage
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| symbol | STRING | Yes | Trading pair |
| leverage | INT | Yes | Target leverage (1 to 125) |
| recvWindow | LONG | No | Optional response window |
| timestamp | LONG | Yes | Current timestamp |
| signature | STRING | Yes | HMAC SHA256 signature |

**Weight:** 1

**Response:**
```json
{
  "leverage": 21,
  "maxNotionalValue": "1000000",
  "symbol": "BTCUSDT"
}
```

---

## User Data Stream Endpoints (Spot)

### 1. Create Listen Key

```
POST /api/v3/userDataStream
```

**Parameters:** NONE (requires X-MBX-APIKEY header)

**Weight:** 2

**Response:**
```json
{
  "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
}
```

---

### 2. Keepalive Listen Key

```
PUT /api/v3/userDataStream
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| listenKey | STRING | Yes | Listen key to extend |

**Weight:** 2

**Response:**
```json
{}
```

---

### 3. Close Listen Key

```
DELETE /api/v3/userDataStream
```

**Parameters:**
| Name | Type | Required | Description |
|------|------|----------|-------------|
| listenKey | STRING | Yes | Listen key to close |

**Weight:** 2

**Response:**
```json
{}
```

---

## User Data Stream Endpoints (Futures USDT-M)

### 1. Create Listen Key

```
POST /fapi/v1/listenKey
```

### 2. Keepalive Listen Key

```
PUT /fapi/v1/listenKey
```

### 3. Close Listen Key

```
DELETE /fapi/v1/listenKey
```

**Note:** All futures user data stream endpoints have the same structure as spot endpoints.
