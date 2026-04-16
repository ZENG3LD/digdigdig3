# Bitfinex API v2 Response Formats

## General Conventions

- All responses are in JSON format
- Timestamps are in milliseconds (UTC)
- All symbols must be uppercase
- Array-based responses (not objects)
- Positive amounts = buy/bid, negative amounts = sell/ask

## Public Endpoints

### Platform Status

**Endpoint**: `GET /v2/platform/status`

**Response**:
```json
[1]
```

- `[1]` = Operative (platform running normally)
- `[0]` = Maintenance

### Ticker (Trading Pair)

**Endpoint**: `GET /v2/ticker/{symbol}`

**Response** (10 fields):
```json
[
  10645,              // [0] BID - float
  73.93854271,        // [1] BID_SIZE - float
  10647,              // [2] ASK - float
  75.22266119,        // [3] ASK_SIZE - float
  731.60645389,       // [4] DAILY_CHANGE - float
  0.0738,             // [5] DAILY_CHANGE_RELATIVE - float (percentage)
  10644.00645389,     // [6] LAST_PRICE - float
  14480.89849423,     // [7] VOLUME - float
  10766,              // [8] HIGH - float
  9889.1449809        // [9] LOW - float
]
```

### Ticker (Funding Currency)

**Response** (16 fields):
```json
[
  0.0003447041095890411,  // [0] FRR - Flash Return Rate
  0.000316,               // [1] BID - float
  30,                     // [2] BID_PERIOD - int (days)
  1681669.14021675,       // [3] BID_SIZE - float
  0.00033268,             // [4] ASK - float
  2,                      // [5] ASK_PERIOD - int (days)
  146425121.0115426,      // [6] ASK_SIZE - float
  -0.00001062,            // [7] DAILY_CHANGE - float
  -0.0307,                // [8] DAILY_CHANGE_PERC - float
  0.00033534,             // [9] LAST_PRICE - float
  157503514.7976808,      // [10] VOLUME - float
  0.000347,               // [11] HIGH - float
  4e-7,                   // [12] LOW - float
  null,                   // [13] (reserved)
  null,                   // [14] (reserved)
  146212916.17047712      // [15] FRR_AMOUNT_AVAILABLE - float
]
```

### Tickers (Multiple)

**Endpoint**: `GET /v2/tickers?symbols=tBTCUSD,tETHUSD`

**Response**: Array of ticker arrays
```json
[
  ["tBTCUSD", BID, BID_SIZE, ASK, ASK_SIZE, DAILY_CHANGE, DAILY_CHANGE_RELATIVE, LAST_PRICE, VOLUME, HIGH, LOW],
  ["tETHUSD", BID, BID_SIZE, ASK, ASK_SIZE, DAILY_CHANGE, DAILY_CHANGE_RELATIVE, LAST_PRICE, VOLUME, HIGH, LOW]
]
```

### Order Book (Precision P0-P4)

**Endpoint**: `GET /v2/book/{symbol}/{precision}`

**Response** (Trading Pair):
```json
[
  [
    8744.9,        // [0] PRICE - float
    2,             // [1] COUNT - int (number of orders at this price)
    0.45603413     // [2] AMOUNT - float (positive=bid, negative=ask)
  ],
  [8744.8, 1, -0.25],
  ...
]
```

**Response** (Funding Currency):
```json
[
  [
    0.0003301,     // [0] RATE - float
    30,            // [1] PERIOD - int (days)
    1,             // [2] COUNT - int
    -3862.874      // [3] AMOUNT - float
  ]
]
```

### Order Book (Raw - R0)

**Response** (Trading Pair):
```json
[
  [
    34006738527,   // [0] ORDER_ID - int
    8744.9,        // [1] PRICE - float
    0.25603413     // [2] AMOUNT - float
  ]
]
```

**Response** (Funding Currency):
```json
[
  [
    645902785,     // [0] OFFER_ID - int
    30,            // [1] PERIOD - int (days)
    0.0003301,     // [2] RATE - float
    -3862.874      // [3] AMOUNT - float
  ]
]
```

### Trades

**Endpoint**: `GET /v2/trades/{symbol}/hist`

**Response** (Trading Pair):
```json
[
  [
    388063448,        // [0] ID - int
    1567526214876,    // [1] MTS - int (millisecond timestamp)
    1.918524,         // [2] AMOUNT - float
    10682             // [3] PRICE - float
  ],
  [388063447, 1567526214000, -0.5, 10683],
  ...
]
```

**Response** (Funding Currency):
```json
[
  [
    124486873,           // [0] ID - int
    1567526287066,       // [1] MTS - int
    -210.69675707,       // [2] AMOUNT - float
    0.00034369,          // [3] RATE - float
    2                    // [4] PERIOD - int (days)
  ]
]
```

### Candles

**Endpoint**: `GET /v2/candles/trade:1m:tBTCUSD/last`

**Response** (Single Candle):
```json
[
  1678465320000,  // [0] MTS - int (millisecond timestamp)
  20097,          // [1] OPEN - float
  20094,          // [2] CLOSE - float
  20097,          // [3] HIGH - float
  20094,          // [4] LOW - float
  0.07870586      // [5] VOLUME - float
]
```

**Response** (Historical):
```json
[
  [1678465320000, 20097, 20114, 20125, 20094, 1.43504645],
  [1678465260000, 20100, 20097, 20105, 20090, 0.95234123],
  ...
]
```

## Authenticated Endpoints

### Wallets

**Endpoint**: `POST /v2/auth/r/wallets`

**Response**:
```json
[
  [
    "exchange",              // [0] TYPE - string (exchange/margin/funding)
    "BTC",                   // [1] CURRENCY - string
    1.5,                     // [2] BALANCE - float
    0,                       // [3] UNSETTLED_INTEREST - float
    1.5,                     // [4] AVAILABLE_BALANCE - float
    "Trade: tBTCUSD",        // [5] LAST_CHANGE - string (description)
    {"trade_id": 123456}     // [6] LAST_CHANGE_METADATA - object (optional)
  ],
  [
    "margin",
    "USD",
    10000.25,
    0.15,
    9500.10,
    "Funding payment",
    null
  ],
  ...
]
```

### Order Object (32 fields)

Used in: Submit Order, Cancel Order, Retrieve Orders, Order History

**Format**:
```json
[
  123456789,              // [0] ID - int
  123,                    // [1] GID - int (group ID)
  456789,                 // [2] CID - int (client order ID)
  "tBTCUSD",              // [3] SYMBOL - string
  1609459200000,          // [4] MTS_CREATE - int (millisecond timestamp)
  1609459205000,          // [5] MTS_UPDATE - int
  0.5,                    // [6] AMOUNT - float (positive=buy, negative=sell)
  0.5,                    // [7] AMOUNT_ORIG - float (original amount)
  "EXCHANGE LIMIT",       // [8] TYPE - string
  null,                   // [9] TYPE_PREV - string
  null,                   // [10] MTS_TIF - int (time-in-force)
  null,                   // [11] (reserved)
  0,                      // [12] FLAGS - int
  "ACTIVE",               // [13] ORDER_STATUS - string
  null,                   // [14] (reserved)
  null,                   // [15] (reserved)
  10000,                  // [16] PRICE - float
  0,                      // [17] PRICE_AVG - float (average execution price)
  0,                      // [18] PRICE_TRAILING - float
  0,                      // [19] PRICE_AUX_LIMIT - float
  null,                   // [20] (reserved)
  null,                   // [21] (reserved)
  null,                   // [22] (reserved)
  0,                      // [23] NOTIFY - int
  0,                      // [24] HIDDEN - int (1=hidden, 0=visible)
  null,                   // [25] PLACED_ID - int
  null,                   // [26] (reserved)
  null,                   // [27] (reserved)
  null,                   // [28] ROUTING - string
  null,                   // [29] (reserved)
  null,                   // [30] (reserved)
  {}                      // [31] META - object
]
```

**Order Status Values**:
- ACTIVE
- PARTIALLY FILLED
- EXECUTED
- CANCELED
- INSUFFICIENT BALANCE
- INSUFFICIENT MARGIN

### Submit Order Response

**Endpoint**: `POST /v2/auth/w/order/submit`

**Response**:
```json
[
  [
    1609459200000,       // [0] MTS - int
    "on-req",            // [1] TYPE - string (notification type)
    123456,              // [2] MESSAGE_ID - int
    null,                // [3] (reserved)
    [ORDER_DATA],        // [4] ORDER - array (32 fields)
    null,                // [5] CODE - int
    "SUCCESS",           // [6] STATUS - string
    "Order submitted"    // [7] TEXT - string
  ]
]
```

**Notification Types**:
- `on-req` - Order new request
- `ou-req` - Order update request
- `oc-req` - Order cancel request
- `oc_multi-req` - Multiple order cancel request

### Cancel Order Response

**Response**:
```json
[
  [
    1609459200000,
    "oc-req",
    123457,
    null,
    [ORDER_DATA],
    null,
    "SUCCESS",
    "Order cancelled"
  ]
]
```

### Retrieve Orders

**Endpoint**: `POST /v2/auth/r/orders`

**Response**: Array of order objects
```json
[
  [ID, GID, CID, SYMBOL, ..., META],  // Order 1
  [ID, GID, CID, SYMBOL, ..., META],  // Order 2
  ...
]
```

### Trade Object (12 fields)

**Endpoint**: `POST /v2/auth/r/trades/hist`

**Response**:
```json
[
  [
    388063448,        // [0] ID - int
    "tBTCUSD",        // [1] SYMBOL - string
    1567526214876,    // [2] MTS - int (execution timestamp)
    987654321,        // [3] ORDER_ID - int
    0.5,              // [4] EXEC_AMOUNT - float (positive=buy, negative=sell)
    10682,            // [5] EXEC_PRICE - float
    "LIMIT",          // [6] ORDER_TYPE - string
    10680,            // [7] ORDER_PRICE - float
    1,                // [8] MAKER - int (1=maker, -1=taker, 0=unknown)
    -0.02,            // [9] FEE - float (negative value)
    "USD",            // [10] FEE_CURRENCY - string
    123456            // [11] CID - int (client order ID)
  ],
  ...
]
```

### Position Object (18 fields)

**Endpoint**: `POST /v2/auth/r/positions`

**Response**:
```json
[
  [
    "tBTCUSD",         // [0] SYMBOL - string
    "ACTIVE",          // [1] STATUS - string (ACTIVE/CLOSED)
    0.5,               // [2] AMOUNT - float (position size)
    10000,             // [3] BASE_PRICE - float (entry price)
    -0.15,             // [4] FUNDING - float
    0,                 // [5] FUNDING_TYPE - int
    50.25,             // [6] PL - float (profit/loss)
    0.5025,            // [7] PL_PERC - float (P/L percentage)
    9500,              // [8] PRICE_LIQ - float (liquidation price)
    10,                // [9] LEVERAGE - float
    null,              // [10] PLACEHOLDER
    12345678,          // [11] POSITION_ID - int
    1609459200000,     // [12] MTS_CREATE - int
    1609459800000,     // [13] MTS_UPDATE - int
    0,                 // [14] TYPE - int
    500,               // [15] COLLATERAL - float
    450,               // [16] COLLATERAL_MIN - float
    {                  // [17] META - object
      "reason": "TRADE",
      "order_id": 987654321,
      "trade_price": 10100,
      "trade_amount": 0.5
    }
  ],
  ...
]
```

## Error Responses

### Error Format

All errors return as arrays:
```json
["error", ERROR_CODE, "error message"]
```

### Common Error Codes

```json
["error", 10000, "unknown error"]
["error", 10020, "symbol: invalid"]
["error", 10050, "order: invalid"]
["error", 10100, "minimum size for order is ..."]
["error", 11000, "amount: invalid"]
["error", 11010, "insufficient balance"]
["error", ERR_RATE_LIMIT, "ratelimit"]
```

### Maintenance Error

```json
["error", 20060, "maintenance"]
```

### Authentication Errors

```json
["error", 10100, "apikey: invalid"]
["error", 10111, "apikey: invalid signature"]
["error", 10112, "apikey: nonce too small"]
["error", 10113, "apikey: invalid permission"]
```

## WebSocket Response Formats

See `websocket.md` for detailed WebSocket message formats.

## Data Type Conventions

| Type | Description | Example |
|------|-------------|---------|
| int | Integer | 123456 |
| float | Floating point number | 10000.5 |
| string | Text | "tBTCUSD" |
| array | Ordered list | [1, 2, 3] |
| object | Key-value pairs | {"key": "value"} |
| null | Null value | null |

## Timestamp Format

All timestamps are **milliseconds since Unix epoch (UTC)**:
- Creation: `1609459200000`
- JavaScript: `Date.now()`
- Python: `int(time.time() * 1000)`
- Rust: `SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis()`

## Amount Sign Convention

**Trading Pairs**:
- Positive amount = Buy / Bid
- Negative amount = Sell / Ask

**Order Book**:
- Positive amount = Bid side
- Negative amount = Ask side

**Positions**:
- Positive amount = Long position
- Negative amount = Short position

## Precision Notes

- Prices and amounts use standard floating-point precision
- Some fields may use scientific notation (e.g., `4e-7`)
- Fees are typically negative values
- Always use string type for precise decimal values in requests (e.g., amount, price)
