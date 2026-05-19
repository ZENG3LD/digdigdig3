# Binance API Response Formats

All Binance API responses are in JSON format by default. This document details the exact response structures for all endpoints required by the V5 connector traits.

---

## General Response Format

### Success Response

All successful requests return JSON objects or arrays with the requested data.

### Error Response

```json
{
  "code": -1121,
  "msg": "Invalid symbol."
}
```

Common error codes:
- `-1000`: Unknown error
- `-1001`: Disconnected
- `-1002`: Unauthorized
- `-1003`: Too many requests
- `-1021`: Timestamp outside recvWindow
- `-1022`: Invalid signature
- `-2010`: New order rejected
- `-2011`: Cancel order rejected

---

## MarketData Trait Responses

### 1. Ping

**Endpoint**: `GET /api/v3/ping`

**Response**:
```json
{}
```

Empty JSON object indicates successful connection.

---

### 2. Server Time

**Endpoint**: `GET /api/v3/time`

**Response**:
```json
{
  "serverTime": 1499827319559
}
```

**Fields**:
- `serverTime` (LONG): Current server time in milliseconds

---

### 3. Get Price

**Endpoint**: `GET /api/v3/ticker/price`

**Response (single symbol)**:
```json
{
  "symbol": "LTCBTC",
  "price": "4.00000200"
}
```

**Response (multiple symbols)**:
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

**Fields**:
- `symbol` (STRING): Trading pair symbol
- `price` (STRING): Current price as decimal string

---

### 4. Get Orderbook

**Endpoint**: `GET /api/v3/depth`

**Response**:
```json
{
  "lastUpdateId": 1027024,
  "bids": [
    [
      "4.00000000",
      "431.00000000"
    ]
  ],
  "asks": [
    [
      "4.00000200",
      "12.00000000"
    ]
  ]
}
```

**Fields**:
- `lastUpdateId` (LONG): Order book update ID
- `bids` (ARRAY): Array of bid [price, quantity] pairs
  - Index 0 (STRING): Price level
  - Index 1 (STRING): Quantity at price level
- `asks` (ARRAY): Array of ask [price, quantity] pairs
  - Index 0 (STRING): Price level
  - Index 1 (STRING): Quantity at price level

**Note**: Bids and asks are sorted from best to worst price.

---

### 5. Get Klines

**Endpoint**: `GET /api/v3/klines`

**Response**:
```json
[
  [
    1499040000000,
    "0.01634790",
    "0.80000000",
    "0.01575800",
    "0.01577100",
    "148976.11427815",
    1499644799999,
    "2434.19055334",
    308,
    "1756.87402397",
    "28.46694368",
    "0"
  ]
]
```

**Array Fields** (in order):
- Index 0 (LONG): Kline open time (milliseconds)
- Index 1 (STRING): Open price
- Index 2 (STRING): High price
- Index 3 (STRING): Low price
- Index 4 (STRING): Close price
- Index 5 (STRING): Volume
- Index 6 (LONG): Kline close time (milliseconds)
- Index 7 (STRING): Quote asset volume
- Index 8 (INT): Number of trades
- Index 9 (STRING): Taker buy base asset volume
- Index 10 (STRING): Taker buy quote asset volume
- Index 11 (STRING): Unused field, ignore

---

### 6. Get Ticker (24hr Statistics)

**Endpoint**: `GET /api/v3/ticker/24hr`

**Response (FULL type)**:
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

**Response (MINI type)**:
```json
{
  "symbol": "BNBBTC",
  "openPrice": "99.00000000",
  "highPrice": "100.00000000",
  "lowPrice": "0.10000000",
  "lastPrice": "4.00000200",
  "volume": "8913.30000000",
  "quoteVolume": "15.30000000",
  "openTime": 1499783499040,
  "closeTime": 1499869899040,
  "firstId": 28385,
  "lastId": 28460,
  "count": 76
}
```

**Fields (FULL)**:
- `symbol` (STRING): Trading pair
- `priceChange` (STRING): Absolute price change
- `priceChangePercent` (STRING): Relative price change in percentage
- `weightedAvgPrice` (STRING): Weighted average price
- `prevClosePrice` (STRING): Previous close price
- `lastPrice` (STRING): Last traded price
- `lastQty` (STRING): Last traded quantity
- `bidPrice` (STRING): Best bid price
- `bidQty` (STRING): Best bid quantity
- `askPrice` (STRING): Best ask price
- `askQty` (STRING): Best ask quantity
- `openPrice` (STRING): Open price
- `highPrice` (STRING): High price
- `lowPrice` (STRING): Low price
- `volume` (STRING): Total traded base asset volume
- `quoteVolume` (STRING): Total traded quote asset volume
- `openTime` (LONG): Statistics open time (milliseconds)
- `closeTime` (LONG): Statistics close time (milliseconds)
- `firstId` (LONG): First trade ID
- `lastId` (LONG): Last trade ID
- `count` (INT): Total number of trades

---

## Trading Trait Responses

### 1. Market Order

**Endpoint**: `POST /api/v3/order`

**Response (ACK)**:
```json
{
  "symbol": "BTCUSDT",
  "orderId": 28,
  "orderListId": -1,
  "clientOrderId": "6gCrw2kRUAF9CvJDGP16IP",
  "transactTime": 1507725176595
}
```

**Response (RESULT)**:
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

**Response (FULL)**:
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
    },
    {
      "price": "3999.00000000",
      "qty": "5.00000000",
      "commission": "19.99500000",
      "commissionAsset": "USDT",
      "tradeId": 57
    }
  ]
}
```

**Fields**:
- `symbol` (STRING): Trading pair
- `orderId` (LONG): Order ID
- `orderListId` (LONG): OCO list ID (-1 if not part of OCO)
- `clientOrderId` (STRING): Client-assigned order ID
- `transactTime` (LONG): Transaction time (milliseconds)
- `price` (STRING): Order price
- `origQty` (STRING): Original order quantity
- `executedQty` (STRING): Executed quantity
- `cummulativeQuoteQty` (STRING): Cumulative quote asset quantity
- `status` (STRING): Order status (NEW, PARTIALLY_FILLED, FILLED, etc.)
- `timeInForce` (STRING): Time in force (GTC, IOC, FOK)
- `type` (STRING): Order type (MARKET, LIMIT, etc.)
- `side` (STRING): Order side (BUY, SELL)
- `workingTime` (LONG): Time when order started working (milliseconds)
- `selfTradePreventionMode` (STRING): STP mode
- `fills` (ARRAY): Array of fill objects (FULL response only)
  - `price` (STRING): Fill price
  - `qty` (STRING): Fill quantity
  - `commission` (STRING): Commission amount
  - `commissionAsset` (STRING): Commission asset
  - `tradeId` (LONG): Trade ID

**Note**: Use `newOrderRespType` parameter to control response format (ACK, RESULT, or FULL).

---

### 2. Limit Order

**Endpoint**: `POST /api/v3/order`

**Response** (same structure as Market Order):
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

Difference from market order: `type` is "LIMIT" and `price` is the limit price.

---

### 3. Cancel Order

**Endpoint**: `DELETE /api/v3/order`

**Response**:
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

**Fields**:
- `symbol` (STRING): Trading pair
- `origClientOrderId` (STRING): Original client order ID
- `orderId` (LONG): Order ID
- `orderListId` (LONG): OCO list ID
- `clientOrderId` (STRING): New client order ID for cancel request
- `transactTime` (LONG): Cancellation time (milliseconds)
- `price` (STRING): Order price
- `origQty` (STRING): Original quantity
- `executedQty` (STRING): Executed quantity before cancel
- `cummulativeQuoteQty` (STRING): Cumulative quote quantity
- `status` (STRING): Order status (should be "CANCELED")
- `timeInForce` (STRING): Time in force
- `type` (STRING): Order type
- `side` (STRING): Order side
- `selfTradePreventionMode` (STRING): STP mode

---

### 4. Get Order (Query Order)

**Endpoint**: `GET /api/v3/order`

**Response**:
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

**Fields**:
- `symbol` (STRING): Trading pair
- `orderId` (LONG): Order ID
- `orderListId` (LONG): OCO list ID
- `clientOrderId` (STRING): Client order ID
- `price` (STRING): Order price
- `origQty` (STRING): Original quantity
- `executedQty` (STRING): Executed quantity
- `cummulativeQuoteQty` (STRING): Cumulative quote quantity
- `status` (STRING): Order status
- `timeInForce` (STRING): Time in force
- `type` (STRING): Order type
- `side` (STRING): Order side
- `stopPrice` (STRING): Stop price (0 if not stop order)
- `icebergQty` (STRING): Iceberg quantity
- `time` (LONG): Order creation time (milliseconds)
- `updateTime` (LONG): Last update time (milliseconds)
- `isWorking` (BOOLEAN): Whether order is currently active
- `workingTime` (LONG): Time when order started working
- `origQuoteOrderQty` (STRING): Original quote order quantity
- `selfTradePreventionMode` (STRING): STP mode

**Possible Status Values**:
- `NEW`: Order accepted but not yet executed
- `PARTIALLY_FILLED`: Partially executed
- `FILLED`: Fully executed
- `CANCELED`: Canceled by user
- `PENDING_CANCEL`: Cancellation in progress
- `REJECTED`: Rejected by exchange
- `EXPIRED`: Expired (e.g., IOC/FOK orders)
- `EXPIRED_IN_MATCH`: Expired due to STP

---

### 5. Get Open Orders

**Endpoint**: `GET /api/v3/openOrders`

**Response** (array of orders):
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

Same structure as Get Order, but returns an array of all open orders.

---

## Account Trait Responses

### 1. Get Balance & Get Account Info

**Endpoint**: `GET /api/v3/account`

**Response**:
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

**Fields**:
- `makerCommission` (INT): Maker commission rate (deprecated, use commissionRates)
- `takerCommission` (INT): Taker commission rate (deprecated, use commissionRates)
- `buyerCommission` (INT): Buyer commission rate (deprecated)
- `sellerCommission` (INT): Seller commission rate (deprecated)
- `commissionRates` (OBJECT): Commission rates as decimals
  - `maker` (STRING): Maker commission rate
  - `taker` (STRING): Taker commission rate
  - `buyer` (STRING): Buyer commission rate
  - `seller` (STRING): Seller commission rate
- `canTrade` (BOOLEAN): Account can trade
- `canWithdraw` (BOOLEAN): Account can withdraw
- `canDeposit` (BOOLEAN): Account can deposit
- `brokered` (BOOLEAN): Account is brokered
- `requireSelfTradePrevention` (BOOLEAN): STP required
- `preventSor` (BOOLEAN): Prevent SOR
- `updateTime` (LONG): Last update time (milliseconds)
- `accountType` (STRING): Account type (SPOT, MARGIN, etc.)
- `balances` (ARRAY): Array of balance objects
  - `asset` (STRING): Asset symbol
  - `free` (STRING): Available balance
  - `locked` (STRING): Locked balance (in orders)
- `permissions` (ARRAY): Account permissions
- `uid` (LONG): User ID

**Note**: Use `omitZeroBalances=true` to exclude assets with zero balance.

---

## Positions Trait Responses (Futures Only)

### 1. Get Positions

**Endpoint**: `GET /fapi/v2/positionRisk`

**Response (One-way mode)**:
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

**Response (Hedge mode)**:
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

**Fields**:
- `symbol` (STRING): Trading pair
- `positionAmt` (STRING): Position quantity (negative for short)
- `entryPrice` (STRING): Average entry price
- `breakEvenPrice` (STRING): Break-even price
- `markPrice` (STRING): Current mark price
- `unRealizedProfit` (STRING): Unrealized PnL
- `liquidationPrice` (STRING): Liquidation price
- `leverage` (STRING): Current leverage
- `maxNotionalValue` (STRING): Maximum notional value
- `marginType` (STRING): Margin type (isolated, cross)
- `isolatedMargin` (STRING): Isolated margin amount
- `isAutoAddMargin` (STRING): Auto-add margin enabled
- `positionSide` (STRING): Position side (BOTH, LONG, SHORT)
- `notional` (STRING): Position notional value
- `isolatedWallet` (STRING): Isolated wallet balance
- `updateTime` (LONG): Last update time (milliseconds)

---

### 2. Get Funding Rate

**Endpoint**: `GET /fapi/v1/fundingRate`

**Response**:
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

**Fields**:
- `symbol` (STRING): Trading pair
- `fundingRate` (STRING): Funding rate (as decimal)
- `fundingTime` (LONG): Funding timestamp (milliseconds)
- `markPrice` (STRING): Mark price at funding time

---

### 3. Set Leverage

**Endpoint**: `POST /fapi/v1/leverage`

**Response**:
```json
{
  "leverage": 21,
  "maxNotionalValue": "1000000",
  "symbol": "BTCUSDT"
}
```

**Fields**:
- `leverage` (INT): New leverage setting
- `maxNotionalValue` (STRING): Maximum notional value at this leverage
- `symbol` (STRING): Trading pair

---

## User Data Stream Responses

### 1. Create/Keepalive Listen Key

**Endpoint**: `POST /api/v3/userDataStream` or `PUT /api/v3/userDataStream`

**Response**:
```json
{
  "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
}
```

**Fields**:
- `listenKey` (STRING): WebSocket listen key (valid for 60 minutes)

---

### 2. Close Listen Key

**Endpoint**: `DELETE /api/v3/userDataStream`

**Response**:
```json
{}
```

Empty object indicates successful deletion.

---

## Time Fields

All timestamps in Binance API are in **milliseconds** by default:
- `timestamp`: Request timestamp
- `serverTime`: Server time
- `transactTime`: Transaction time
- `time`: Order creation time
- `updateTime`: Last update time
- `openTime`: Kline/statistics open time
- `closeTime`: Kline/statistics close time
- `fundingTime`: Funding time

To use **microseconds** instead, add header:
```
X-MBX-TIME-UNIT: MICROSECOND
```

---

## Numeric Fields

All numeric values are returned as **STRING** type to preserve precision:
- Prices: `"0.00012345"`
- Quantities: `"10.50000000"`
- Volumes: `"1234567.89012345"`

**Important**: Always parse as decimal/float in your application to avoid precision loss.

---

## Data Sources

Responses include data from different sources:
- **Memory**: Fastest, cached data
- **Database**: Persistent storage, may be slightly delayed

Most market data comes from **Memory**, while account data comes from **Database**.
