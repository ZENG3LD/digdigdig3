# BingX API Response Formats

Complete documentation of BingX API response structures for all endpoints.

---

## General Response Structure

### Success Response

All BingX API responses follow this general JSON structure:

```json
{
  "code": 0,
  "msg": "",
  "data": { ... }
}
```

**Fields:**
- `code` (integer) - Status code (0 = success)
- `msg` (string) - Error message (empty on success)
- `data` (object/array) - Response payload

### HTTP Status Codes

- **200 OK** - Successful response
- **401 Unauthorized** - Authentication failed
- **403 Forbidden** - Insufficient permissions
- **429 Too Many Requests** - Rate limit exceeded
- **500 Internal Server Error** - Server error

---

## Status Codes

### Standard Status Codes

| Code | Message | Description |
|------|---------|-------------|
| 0 | SUCCESS | Request successful |
| 100204 | SEARCH_NO_CONTENT | No data found |
| 100205 | REPEAT_REQUEST | Duplicate request |
| 100400 | ILLEGAL_ARGUMENT | Invalid parameters |
| 100401 | AUTHENTICATION_FAIL | Authentication failed |
| 100403 | AUTHORIZATION_FAIL | Insufficient permissions |
| 100410 | FREQUENCY_LIMIT | Rate limit exceeded |
| 100500 | INTERNAL_SERVER_ERROR | Server error |

### Error Response Example

```json
{
  "code": 100401,
  "msg": "AUTHENTICATION_FAIL",
  "data": null
}
```

---

## Market Data Responses

### Get Trading Symbols (Spot)

**Endpoint:** `GET /openApi/spot/v1/common/symbols`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbols": [
      {
        "symbol": "BTC-USDT",
        "minQty": 0.0001,
        "maxQty": 100,
        "minNotional": 5,
        "maxNotional": 1000000,
        "status": "TRADING"
      },
      {
        "symbol": "ETH-USDT",
        "minQty": 0.001,
        "maxQty": 1000,
        "minNotional": 5,
        "maxNotional": 1000000,
        "status": "TRADING"
      }
    ]
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair symbol
- `minQty` (decimal) - Minimum order quantity
- `maxQty` (decimal) - Maximum order quantity
- `minNotional` (decimal) - Minimum order value
- `maxNotional` (decimal) - Maximum order value
- `status` (string) - Trading status: `TRADING`, `HALT`, `BREAK`

### Get Order Book

**Endpoint:** `GET /openApi/spot/v1/market/depth`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "bids": [
      ["43302.00", "0.521000"],
      ["43301.50", "0.234000"],
      ["43301.00", "1.002000"]
    ],
    "asks": [
      ["43303.00", "0.321000"],
      ["43303.50", "0.892000"],
      ["43304.00", "0.456000"]
    ]
  }
}
```

**Fields:**
- `bids` (array) - Buy orders [price, quantity]
- `asks` (array) - Sell orders [price, quantity]

### Get Recent Trades

**Endpoint:** `GET /openApi/spot/v1/market/trades`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": [
    {
      "id": 28457,
      "price": "43302.50",
      "qty": "0.125000",
      "time": 1649404670162,
      "isBuyerMaker": false
    },
    {
      "id": 28458,
      "price": "43303.00",
      "qty": "0.050000",
      "time": 1649404670200,
      "isBuyerMaker": true
    }
  ]
}
```

**Fields:**
- `id` (long) - Trade ID
- `price` (string) - Trade price
- `qty` (string) - Trade quantity
- `time` (long) - Trade timestamp in milliseconds
- `isBuyerMaker` (boolean) - True if buyer is maker

### Get Kline Data

**Endpoint:** `GET /openApi/spot/v1/market/kline`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": [
    {
      "time": 1649404800000,
      "open": "43250.00",
      "high": "43350.00",
      "low": "43200.00",
      "close": "43302.50",
      "volume": "125.450000"
    },
    {
      "time": 1649404860000,
      "open": "43302.50",
      "high": "43400.00",
      "low": "43280.00",
      "close": "43380.00",
      "volume": "98.230000"
    }
  ]
}
```

**Fields:**
- `time` (long) - Kline open time in milliseconds
- `open` (string) - Opening price
- `high` (string) - Highest price
- `low` (string) - Lowest price
- `close` (string) - Closing price
- `volume` (string) - Trading volume

### Get 24hr Ticker

**Endpoint:** `GET /openApi/spot/v1/ticker/24hr`

**Response (single symbol):**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "priceChange": "1250.50",
    "priceChangePercent": "2.98",
    "lastPrice": "43302.50",
    "lastQty": "0.125000",
    "highPrice": "43500.00",
    "lowPrice": "41800.00",
    "volume": "12458.250000",
    "quoteVolume": "536428950.25",
    "openPrice": "42052.00",
    "openTime": 1649318270162,
    "closeTime": 1649404670162
  }
}
```

**Response (all symbols):**
```json
{
  "code": 0,
  "msg": "",
  "data": [
    {
      "symbol": "BTC-USDT",
      "priceChange": "1250.50",
      "priceChangePercent": "2.98",
      "lastPrice": "43302.50",
      ...
    },
    {
      "symbol": "ETH-USDT",
      "priceChange": "-45.20",
      "priceChangePercent": "-1.52",
      "lastPrice": "2925.30",
      ...
    }
  ]
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `priceChange` (string) - Price change in 24h
- `priceChangePercent` (string) - Price change percentage
- `lastPrice` (string) - Latest price
- `lastQty` (string) - Last trade quantity
- `highPrice` (string) - Highest price in 24h
- `lowPrice` (string) - Lowest price in 24h
- `volume` (string) - Trading volume in 24h
- `quoteVolume` (string) - Quote asset volume
- `openPrice` (string) - Opening price
- `openTime` (long) - Open time in milliseconds
- `closeTime` (long) - Close time in milliseconds

### Get Symbol Price

**Endpoint:** `GET /openApi/spot/v1/ticker/price`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "price": "43302.50"
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `price` (string) - Current price

### Get Book Ticker

**Endpoint:** `GET /openApi/spot/v1/ticker/bookTicker`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "bidPrice": "43302.00",
    "bidQty": "0.521000",
    "askPrice": "43303.00",
    "askQty": "0.321000"
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `bidPrice` (string) - Best bid price
- `bidQty` (string) - Best bid quantity
- `askPrice` (string) - Best ask price
- `askQty` (string) - Best ask quantity

---

## Trading Responses

### Place Order (Spot)

**Endpoint:** `POST /openApi/spot/v1/trade/order`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "orderId": 1234567890,
    "transactTime": 1649404670162,
    "price": "43300.00",
    "origQty": "0.125000",
    "executedQty": "0.000000",
    "cummulativeQuoteQty": "0.00",
    "status": "NEW",
    "type": "LIMIT",
    "side": "BUY"
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `orderId` (long) - Order ID
- `transactTime` (long) - Transaction timestamp
- `price` (string) - Order price
- `origQty` (string) - Original quantity
- `executedQty` (string) - Executed quantity
- `cummulativeQuoteQty` (string) - Cumulative quote quantity
- `status` (string) - Order status: `NEW`, `PARTIALLY_FILLED`, `FILLED`, `CANCELED`, `REJECTED`
- `type` (string) - Order type: `MARKET`, `LIMIT`
- `side` (string) - Order side: `BUY`, `SELL`

### Place Order (Swap)

**Endpoint:** `POST /openApi/swap/v2/trade/order`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "order": {
      "symbol": "BTC-USDT",
      "orderId": 9876543210,
      "side": "BUY",
      "positionSide": "LONG",
      "type": "LIMIT",
      "origQty": "0.100",
      "price": "43300.00",
      "executedQty": "0.000",
      "avgPrice": "0.00",
      "cumQuote": "0.00",
      "stopPrice": "",
      "profit": "0.00",
      "commission": "0.00",
      "status": "NEW",
      "time": 1649404670162,
      "updateTime": 1649404670162
    }
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `orderId` (long) - Order ID
- `side` (string) - Order side: `BUY`, `SELL`
- `positionSide` (string) - Position side: `LONG`, `SHORT`
- `type` (string) - Order type
- `origQty` (string) - Original quantity
- `price` (string) - Order price
- `executedQty` (string) - Executed quantity
- `avgPrice` (string) - Average execution price
- `cumQuote` (string) - Cumulative quote quantity
- `stopPrice` (string) - Stop price (for stop orders)
- `profit` (string) - Realized profit
- `commission` (string) - Commission paid
- `status` (string) - Order status
- `time` (long) - Order creation time
- `updateTime` (long) - Last update time

### Query Order

**Endpoint:** `GET /openApi/spot/v1/trade/order`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "orderId": 1234567890,
    "price": "43300.00",
    "origQty": "0.125000",
    "executedQty": "0.125000",
    "cummulativeQuoteQty": "5412.50",
    "status": "FILLED",
    "type": "LIMIT",
    "side": "BUY",
    "time": 1649404670162,
    "updateTime": 1649404672000
  }
}
```

### Cancel Order

**Endpoint:** `DELETE /openApi/spot/v1/trade/order`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "orderId": 1234567890,
    "status": "CANCELED"
  }
}
```

### Get Open Orders

**Endpoint:** `GET /openApi/spot/v1/trade/openOrders`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "orders": [
      {
        "symbol": "BTC-USDT",
        "orderId": 1234567890,
        "price": "43300.00",
        "origQty": "0.125000",
        "executedQty": "0.000000",
        "status": "NEW",
        "type": "LIMIT",
        "side": "BUY",
        "time": 1649404670162
      },
      {
        "symbol": "ETH-USDT",
        "orderId": 1234567891,
        "price": "2900.00",
        "origQty": "1.000000",
        "executedQty": "0.500000",
        "status": "PARTIALLY_FILLED",
        "type": "LIMIT",
        "side": "BUY",
        "time": 1649404680000
      }
    ]
  }
}
```

### Get Order History

**Endpoint:** `GET /openApi/spot/v1/trade/historyOrders`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "orders": [
      {
        "symbol": "BTC-USDT",
        "orderId": 1234567888,
        "price": "42000.00",
        "origQty": "0.100000",
        "executedQty": "0.100000",
        "cummulativeQuoteQty": "4200.00",
        "status": "FILLED",
        "type": "LIMIT",
        "side": "BUY",
        "time": 1649300000000,
        "updateTime": 1649300005000
      }
    ]
  }
}
```

---

## Account Responses

### Get Spot Balance

**Endpoint:** `GET /openApi/spot/v1/account/balance`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "balances": [
      {
        "asset": "USDT",
        "free": "10000.00000000",
        "locked": "500.00000000"
      },
      {
        "asset": "BTC",
        "free": "0.50000000",
        "locked": "0.00000000"
      },
      {
        "asset": "ETH",
        "free": "5.25000000",
        "locked": "1.00000000"
      }
    ]
  }
}
```

**Fields:**
- `asset` (string) - Asset name
- `free` (string) - Available balance
- `locked` (string) - Locked balance (in orders)

### Get Swap Balance

**Endpoint:** `GET /openApi/swap/v2/user/balance`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "balance": {
      "userId": "123456789",
      "asset": "USDT",
      "balance": "15000.00000000",
      "equity": "15250.50000000",
      "unrealizedProfit": "250.50000000",
      "realisedProfit": "1200.00000000",
      "availableMargin": "12000.00000000",
      "usedMargin": "3000.00000000",
      "freezedMargin": "250.50000000"
    }
  }
}
```

**Fields:**
- `userId` (string) - User ID
- `asset` (string) - Asset (usually USDT)
- `balance` (string) - Total balance
- `equity` (string) - Account equity
- `unrealizedProfit` (string) - Unrealized PnL
- `realisedProfit` (string) - Realized PnL
- `availableMargin` (string) - Available margin
- `usedMargin` (string) - Used margin
- `freezedMargin` (string) - Frozen margin

### Get Commission Rate

**Endpoint:** `GET /openApi/spot/v1/account/commissionRate`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "makerCommission": "0.001",
    "takerCommission": "0.001"
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `makerCommission` (string) - Maker fee rate (0.001 = 0.1%)
- `takerCommission` (string) - Taker fee rate

---

## Position Responses

### Get Positions (Standard Contract)

**Endpoint:** `GET /openApi/contract/v1/allPosition`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": [
    {
      "symbol": "BTC-USDT",
      "positionId": "123456789",
      "positionSide": "LONG",
      "isolated": false,
      "positionAmt": "0.500",
      "availableAmt": "0.500",
      "unrealizedProfit": "125.50",
      "realisedProfit": "50.00",
      "initialMargin": "2150.00",
      "avgPrice": "43000.00",
      "leverage": 10
    }
  ]
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `positionId` (string) - Position ID
- `positionSide` (string) - Position side: `LONG`, `SHORT`
- `isolated` (boolean) - Isolated margin mode
- `positionAmt` (string) - Position amount
- `availableAmt` (string) - Available amount
- `unrealizedProfit` (string) - Unrealized PnL
- `realisedProfit` (string) - Realized PnL
- `initialMargin` (string) - Initial margin
- `avgPrice` (string) - Average entry price
- `leverage` (integer) - Leverage multiplier

### Get Swap Positions

**Endpoint:** `GET /openApi/swap/v2/user/positions`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": [
    {
      "symbol": "BTC-USDT",
      "positionId": "987654321",
      "positionSide": "LONG",
      "positionAmt": "1.000",
      "availableAmt": "0.500",
      "unrealizedProfit": "250.00",
      "realisedProfit": "100.00",
      "initialMargin": "4300.00",
      "avgPrice": "43000.00",
      "leverage": 10,
      "isolatedMargin": "4300.00",
      "positionValue": "43000.00",
      "markPrice": "43250.00",
      "liquidationPrice": "39100.00"
    }
  ]
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `positionId` (string) - Position ID
- `positionSide` (string) - Position side
- `positionAmt` (string) - Total position amount
- `availableAmt` (string) - Available to close
- `unrealizedProfit` (string) - Unrealized PnL
- `realisedProfit` (string) - Realized PnL
- `initialMargin` (string) - Initial margin
- `avgPrice` (string) - Average entry price
- `leverage` (integer) - Leverage
- `isolatedMargin` (string) - Isolated margin (if applicable)
- `positionValue` (string) - Position value
- `markPrice` (string) - Current mark price
- `liquidationPrice` (string) - Liquidation price

### Set Leverage

**Endpoint:** `POST /openApi/swap/v2/trade/leverage`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "leverage": 20,
    "side": "LONG"
  }
}
```

### Get Leverage

**Endpoint:** `GET /openApi/swap/v2/trade/leverage`

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "symbol": "BTC-USDT",
    "longLeverage": 20,
    "shortLeverage": 15
  }
}
```

**Fields:**
- `symbol` (string) - Trading pair
- `longLeverage` (integer) - Long position leverage
- `shortLeverage` (integer) - Short position leverage

---

## WebSocket Response Formats

### Market Data WebSocket

**Connection:** `wss://open-api-ws.bingx.com/market`

### Depth Stream

**Subscribe:**
```json
{
  "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
  "reqType": "sub",
  "dataType": "BTC-USDT@depth"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@depth",
  "data": {
    "bids": [
      ["43302.00", "0.521000"],
      ["43301.50", "0.234000"]
    ],
    "asks": [
      ["43303.00", "0.321000"],
      ["43303.50", "0.892000"]
    ]
  }
}
```

### Trade Stream

**Subscribe:**
```json
{
  "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
  "reqType": "sub",
  "dataType": "BTC-USDT@trade"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@trade",
  "data": {
    "e": "trade",
    "s": "BTC-USDT",
    "t": 1649404670162,
    "p": "43302.50",
    "q": "0.125000",
    "m": false
  }
}
```

**Fields:**
- `e` (string) - Event type
- `s` (string) - Symbol
- `t` (long) - Trade time
- `p` (string) - Price
- `q` (string) - Quantity
- `m` (boolean) - Is buyer maker

### Kline Stream

**Subscribe:**
```json
{
  "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
  "reqType": "sub",
  "dataType": "BTC-USDT@kline_1min"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@kline_1min",
  "data": {
    "e": "kline",
    "s": "BTC-USDT",
    "k": {
      "t": 1649404800000,
      "T": 1649404859999,
      "s": "BTC-USDT",
      "i": "1min",
      "o": "43250.00",
      "c": "43302.50",
      "h": "43350.00",
      "l": "43200.00",
      "v": "125.450000"
    }
  }
}
```

**Fields:**
- `t` (long) - Kline start time
- `T` (long) - Kline close time
- `s` (string) - Symbol
- `i` (string) - Interval
- `o` (string) - Open price
- `c` (string) - Close price
- `h` (string) - High price
- `l` (string) - Low price
- `v` (string) - Volume

### User Data Stream

**Connection:** `wss://open-api-ws.bingx.com/market?listenKey=<your_listen_key>`

**Order Update:**
```json
{
  "dataType": "spot.executionReport",
  "data": {
    "e": "executionReport",
    "s": "BTC-USDT",
    "c": "client_order_id",
    "S": "BUY",
    "o": "LIMIT",
    "f": "GTC",
    "q": "0.125000",
    "p": "43300.00",
    "x": "TRADE",
    "X": "FILLED",
    "i": 1234567890,
    "l": "0.125000",
    "z": "0.125000",
    "L": "43302.50",
    "n": "5.4125",
    "N": "USDT",
    "T": 1649404670162,
    "t": 28457
  }
}
```

**Fields:**
- `e` (string) - Event type
- `s` (string) - Symbol
- `c` (string) - Client order ID
- `S` (string) - Side
- `o` (string) - Order type
- `f` (string) - Time in force
- `q` (string) - Order quantity
- `p` (string) - Order price
- `x` (string) - Execution type
- `X` (string) - Order status
- `i` (long) - Order ID
- `l` (string) - Last executed quantity
- `z` (string) - Cumulative filled quantity
- `L` (string) - Last executed price
- `n` (string) - Commission amount
- `N` (string) - Commission asset
- `T` (long) - Transaction time
- `t` (long) - Trade ID

**Balance Update:**
```json
{
  "dataType": "spot.account",
  "data": {
    "e": "outboundAccountInfo",
    "E": 1649404670162,
    "B": [
      {
        "a": "USDT",
        "f": "9500.00000000",
        "l": "500.00000000"
      },
      {
        "a": "BTC",
        "f": "0.62500000",
        "l": "0.00000000"
      }
    ]
  }
}
```

**Fields:**
- `e` (string) - Event type
- `E` (long) - Event time
- `B` (array) - Balances
  - `a` (string) - Asset
  - `f` (string) - Free amount
  - `l` (string) - Locked amount

---

## Compressed Data

All WebSocket responses are **GZIP compressed**. Clients must decompress before parsing.

**Rust Example:**
```rust
use flate2::read::GzDecoder;
use std::io::Read;

fn decompress_message(compressed: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}
```

---

## Heartbeat (WebSocket)

**Server sends every 5 seconds:**
```json
{
  "ping": 1649404670162
}
```

**Client must respond:**
```json
{
  "pong": 1649404670162
}
```

Failure to respond to pings will result in disconnection.

---

## Parsing Notes

### Decimal Values

Most numeric values are returned as **strings** to preserve precision:
```json
{
  "price": "43302.50",
  "quantity": "0.125000"
}
```

Parse using decimal/fixed-point libraries, not floats.

### Timestamps

All timestamps are in **milliseconds** since Unix epoch (not seconds).

### Empty Values

Empty strings indicate null/not applicable:
```json
{
  "stopPrice": ""  // No stop price set
}
```

### Order Status Values

- `NEW` - Order accepted
- `PARTIALLY_FILLED` - Partially executed
- `FILLED` - Fully executed
- `CANCELED` - Canceled by user
- `REJECTED` - Rejected by exchange
- `EXPIRED` - Time-in-force expired

---

## Sources

- [BingX API Docs](https://bingx-api.github.io/docs/)
- [BingX Standard Contract API](https://github.com/BingX-API/BingX-Standard-Contract-doc/blob/main/REST%20API.md)
- [BingX Response Status Codes](https://docs.ccxt.com/exchanges/bingx)
- [BingX WebSocket Streams](https://hexdocs.pm/bingex/Bingex.Swap.html)
