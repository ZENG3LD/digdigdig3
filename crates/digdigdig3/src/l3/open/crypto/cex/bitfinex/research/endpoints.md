# Bitfinex API v2 Endpoints

## Base URLs

- **Public Endpoints**: `https://api-pub.bitfinex.com/v2`
- **Authenticated Endpoints**: `https://api.bitfinex.com/v2`

## MarketData Trait Endpoints

### Platform Status
```
GET /platform/status
```

**Response**: `[1]` (1 = operative, 0 = maintenance)

### Ticker
```
GET /ticker/{symbol}
```

**Parameters**:
- `symbol` (path): Trading pair (tBTCUSD) or funding currency (fUSD)

**Response** (Trading Pair):
```json
[
  BID,                    // float
  BID_SIZE,              // float
  ASK,                   // float
  ASK_SIZE,              // float
  DAILY_CHANGE,          // float
  DAILY_CHANGE_RELATIVE, // float
  LAST_PRICE,            // float
  VOLUME,                // float
  HIGH,                  // float
  LOW                    // float
]
```

### Tickers (Multiple)
```
GET /tickers?symbols={symbols}
```

**Parameters**:
- `symbols` (query): Comma-separated list (e.g., `fUSD,tBTCUSD`)

### Order Book
```
GET /book/{symbol}/{precision}?len={len}
```

**Parameters**:
- `symbol` (path): Trading pair or funding currency
- `precision` (path): P0, P1, P2, P3, P4 (price aggregation) or R0 (raw)
- `len` (query): Number of price points: 1, 25, or 100 (default: 25)

**Response** (Precision P0-P4, Trading Pair):
```json
[
  [
    PRICE,    // float
    COUNT,    // int - number of orders
    AMOUNT    // float - total amount (positive=bid, negative=ask)
  ]
]
```

**Response** (Precision R0 - Raw Book, Trading Pair):
```json
[
  [
    ORDER_ID,  // int
    PRICE,     // float
    AMOUNT     // float
  ]
]
```

### Trades
```
GET /trades/{symbol}/hist?limit={limit}&start={start}&end={end}&sort={sort}
```

**Parameters**:
- `symbol` (path): Trading pair or funding currency
- `limit` (query): Max 10000 records
- `start` (query): Millisecond timestamp filter (MTS >= start)
- `end` (query): Millisecond timestamp filter (MTS <= end)
- `sort` (query): +1 ascending, -1 descending (by MTS)

**Response** (Trading Pair):
```json
[
  [
    ID,       // int
    MTS,      // int - millisecond timestamp
    AMOUNT,   // float
    PRICE     // float
  ]
]
```

### Candles
```
GET /candles/{candle}/{section}?limit={limit}&start={start}&end={end}&sort={sort}
```

**Parameters**:
- `candle` (path): Format `trade:{timeframe}:{symbol}` (e.g., `trade:1m:tBTCUSD`)
  - Timeframes: 1m, 5m, 15m, 30m, 1h, 3h, 6h, 12h, 1D, 1W, 14D, 1M
- `section` (path): `last` (single candle) or `hist` (historical)
- `limit` (query): Max 10000 records, default 100
- `start` (query): Millisecond timestamp
- `end` (query): Millisecond timestamp
- `sort` (query): +1 ascending, -1 descending

**Response** (Single Candle):
```json
[
  MTS,     // int - millisecond timestamp
  OPEN,    // float
  CLOSE,   // float
  HIGH,    // float
  LOW,     // float
  VOLUME   // float
]
```

**Response** (Historical):
```json
[
  [MTS, OPEN, CLOSE, HIGH, LOW, VOLUME],
  ...
]
```

### Configuration
```
GET /conf/pub:{action}:{object}:{detail}
```

**Examples**:
- `/conf/pub:list:pair:exchange` - List all trading pairs
- `/conf/pub:list:currency` - List all currencies
- `/conf/pub:map:currency:label` - Currency labels mapping

## Trading Trait Endpoints

### Submit Order
```
POST /auth/w/order/submit
```

**Request Body**:
```json
{
  "type": "EXCHANGE LIMIT",     // required: order type
  "symbol": "tBTCUSD",          // required: trading pair
  "amount": "0.5",              // required: positive=buy, negative=sell
  "price": "10000",             // required: order price
  "lev": 10,                    // optional: leverage (1-100)
  "price_trailing": "0",        // optional: trailing price
  "price_aux_limit": "0",       // optional: STOP LIMIT aux price
  "price_oco_stop": "0",        // optional: OCO stop price
  "gid": 0,                     // optional: group order ID
  "cid": 0,                     // optional: client order ID
  "flags": 0,                   // optional: sum of order flags
  "tif": "",                    // optional: time-in-force (UTC datetime)
  "meta": {}                    // optional: metadata
}
```

**Order Types**:
- EXCHANGE LIMIT
- EXCHANGE MARKET
- EXCHANGE STOP
- EXCHANGE STOP LIMIT
- EXCHANGE TRAILING STOP
- EXCHANGE FOK (Fill or Kill)
- EXCHANGE IOC (Immediate or Cancel)

**Response**:
```json
[
  [
    MTS,           // int
    TYPE,          // string - notification type
    MESSAGE_ID,    // int
    null,
    [ORDER_DATA],  // array - 32 fields
    CODE,          // int
    STATUS,        // string
    TEXT           // string
  ]
]
```

### Cancel Order
```
POST /auth/w/order/cancel
```

**Request Body** (choose one):
```json
{
  "id": 123456789  // Order ID
}
```
OR
```json
{
  "cid": 12345,
  "cid_date": "2024-01-15"
}
```

### Cancel Multiple Orders
```
POST /auth/w/order/cancel/multi
```

**Request Body**:
```json
{
  "id": [123, 456, 789],                        // array of order IDs
  "gid": [1, 2],                                // array of group IDs
  "cid": [[12345, "2024-01-15"], ...],          // array of [cid, date] tuples
  "all": 1                                      // 1 = cancel all orders
}
```

### Update Order
```
POST /auth/w/order/update
```

**Request Body**:
```json
{
  "id": 123456789,
  "price": "10500",
  "amount": "0.6",
  "flags": 0
}
```

### Retrieve Active Orders
```
POST /auth/r/orders
```

**Request Body**:
```json
{
  "id": 123456789,          // optional: specific order ID
  "gid": 1,                 // optional: group ID
  "cid": 12345,             // optional: client order ID
  "cid_date": "2024-01-15"  // required if cid provided
}
```

**Response**: Array of order objects (32 fields each)

### Retrieve Active Orders by Symbol
```
POST /auth/r/orders/{symbol}
```

**Parameters**:
- `symbol` (path): Trading pair symbol

### Order History
```
POST /auth/r/orders/hist
```

**Request Body**:
```json
{
  "start": 1609459200000,   // optional: MTS >= start
  "end": 1612137600000,     // optional: MTS <= end
  "limit": 2500,            // optional: max 2500
  "id": [123, 456]          // optional: array of order IDs
}
```

Returns orders from last 2 weeks.

### Order Trades
```
POST /auth/r/order/{symbol}:{id}/trades
```

**Parameters**:
- `symbol` (path): Trading pair
- `id` (path): Order ID

## Account Trait Endpoints

### Wallets
```
POST /auth/r/wallets
```

**Request Body**: `{}`

**Response**:
```json
[
  [
    TYPE,                  // string - "exchange", "margin", "funding"
    CURRENCY,              // string - "BTC", "USD", etc.
    BALANCE,               // float
    UNSETTLED_INTEREST,    // float
    AVAILABLE_BALANCE,     // float
    LAST_CHANGE,           // string - description
    LAST_CHANGE_METADATA   // object - optional
  ]
]
```

### User Info
```
POST /auth/r/info/user
```

### Account Summary
```
POST /auth/r/summary
```

### Trade History
```
POST /auth/r/trades/hist
```

**Request Body**:
```json
{
  "start": 1609459200000,  // optional: MTS >= start
  "end": 1612137600000,    // optional: MTS <= end
  "limit": 2500,           // optional: max 2500
  "sort": -1               // optional: +1 asc, -1 desc
}
```

**Response**:
```json
[
  [
    ID,              // int
    SYMBOL,          // string
    MTS,             // int - millisecond timestamp
    ORDER_ID,        // int
    EXEC_AMOUNT,     // float - positive=buy, negative=sell
    EXEC_PRICE,      // float
    ORDER_TYPE,      // string
    ORDER_PRICE,     // float
    MAKER,           // int - 1=maker, -1=taker
    FEE,             // float
    FEE_CURRENCY,    // string
    CID              // int - client order ID
  ]
]
```

### Trades by Symbol
```
POST /auth/r/trades/{symbol}/hist
```

**Parameters**:
- `symbol` (path): Trading pair

## Positions Trait Endpoints

### Retrieve Positions
```
POST /auth/r/positions
```

**Request Body**: `{}`

**Response**:
```json
[
  [
    SYMBOL,           // string - trading pair
    STATUS,           // string - "ACTIVE", "CLOSED"
    AMOUNT,           // float - position size
    BASE_PRICE,       // float - entry price
    FUNDING,          // float
    FUNDING_TYPE,     // int
    PL,               // float - profit/loss
    PL_PERC,          // float - P/L percentage
    PRICE_LIQ,        // float - liquidation price
    LEVERAGE,         // float
    PLACEHOLDER,      // null
    POSITION_ID,      // int
    MTS_CREATE,       // int - creation timestamp
    MTS_UPDATE,       // int - update timestamp
    TYPE,             // int
    COLLATERAL,       // float
    COLLATERAL_MIN,   // float
    META              // object - metadata
  ]
]
```

### Position History
```
POST /auth/r/positions/hist
```

**Request Body**:
```json
{
  "start": 1609459200000,
  "end": 1612137600000,
  "limit": 2500
}
```

### Position Snapshot
```
POST /auth/r/positions/snap
```

## Rate Limits

All authenticated endpoints: **90 requests per minute**

Public endpoints: **10-90 requests per minute** (varies by endpoint)

## Error Response Format

```json
["error", ERROR_CODE, "error message"]
```

Common error codes:
- 10020: Invalid symbol
- 10050: Invalid order
- ERR_RATE_LIMIT: Rate limit exceeded
