# KuCoin WebSocket API Documentation

Comprehensive research on KuCoin WebSocket API for Spot and Futures trading.

---

## 1. Connection Setup

### 1.1 Obtaining WebSocket Token

#### Public Token (No Authentication)

**Spot/Margin Endpoint:**
```
POST https://api.kucoin.com/api/v1/bullet-public
```

**Futures Endpoint:**
```
POST https://api-futures.kucoin.com/api/v1/bullet-public
```

**Request:**
No parameters required for public token.

**Response Format:**
```json
{
  "code": "200000",
  "data": {
    "token": "string",
    "instanceServers": [
      {
        "endpoint": "wss://ws-api-spot.kucoin.com/",
        "encrypt": true,
        "protocol": "websocket",
        "pingInterval": 18000,
        "pingTimeout": 10000
      }
    ]
  }
}
```

#### Private Token (Authentication Required)

**Spot/Margin Endpoint:**
```
GET /api/v1/bullet-private
```

**Futures Endpoint:**
```
GET /api/v1/bullet-private
```

**Pro API (Unified Account):**
```
GET /api/v2/bullet-private
```

**Request Headers (for private endpoints):**
```
KC-API-KEY: your-api-key
KC-API-SIGN: signature
KC-API-TIMESTAMP: timestamp
KC-API-KEY-VERSION: 1
```

**Response Format:**
```json
{
  "code": "200000",
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "instanceServers": [
      {
        "endpoint": "wss://wsapi-push.kucoin.com",
        "protocol": "websocket",
        "encrypt": true,
        "pingInterval": 50000,
        "pingTimeout": 10000,
        "features": ["binary"]
      }
    ],
    "userId": "user-id",
    "acceptUserMessage": true
  },
  "success": true
}
```

**Response Fields:**
- `token` (string): Authentication token for WebSocket connection, valid for 24 hours
- `instanceServers` (array): Array of available server instances
  - `endpoint` (string): WebSocket server URL
  - `protocol` (string): Connection protocol ("websocket")
  - `encrypt` (boolean): Encryption enabled (true)
  - `pingInterval` (number): Time between pings in milliseconds (18000-50000)
  - `pingTimeout` (number): Timeout for ping response in milliseconds (10000)

### 1.2 WebSocket Endpoint URLs

**Classic Account - Spot:**
```
wss://ws-api-spot.kucoin.com
```

**Classic Account - Futures:**
```
wss://ws-api-futures.kucoin.com
```

**Unified Account - Spot & Margin Public:**
```
wss://x-push-spot.kucoin.com
```

**Unified Account - Futures Public:**
```
wss://x-push-futures.kucoin.com
```

**Private Channels:**
```
wss://wsapi-push.kucoin.com
```

**Order Operations:**
```
wss://wsapi.kucoin.com
```

### 1.3 Connection URL Format

**Format:**
```
wss://<endpoint>/?token=<token>&[connectId=<connectId>]
```

**Example:**
```javascript
var socket = new WebSocket('wss://ws-api-spot.kucoin.com/?token=xxx&connectId=xxxxx');
```

**Parameters:**
- `token` (required): Token obtained from bullet-public or bullet-private endpoint
- `connectId` (optional): Unique connection ID from client side for troubleshooting

### 1.4 Welcome Message

After successful connection, the server sends a welcome message:

```json
{
  "sessionId": "7245afa1-a57c-4f90-b4bd-90126387214b",
  "message": "welcome",
  "pingInterval": 30000
}
```

**Or (alternative format):**
```json
{
  "sessionId": "92f2aec4-d87e-47cc-917d-4e7c93911bdc",
  "data": "welcome",
  "pingInterval": 18000,
  "pingTimeout": 10000
}
```

**Fields:**
- `sessionId` / `connectId` (string): Unique connection identifier for troubleshooting
- `message` / `data` (string): "welcome" indicates successful connection
- `pingInterval` (number): Client ping interval in milliseconds (18000-30000)
- `pingTimeout` (number): Ping timeout in milliseconds (10000)

### 1.5 Ping/Pong Mechanism

**Client Ping Message:**
```json
{
  "id": "1545910590801",
  "type": "ping"
}
```

**Server Pong Response:**
```json
{
  "id": "1545910590801",
  "type": "pong"
}
```

**Behavior:**
- Client must send ping every `pingInterval` milliseconds to keep connection alive
- If server receives no message from client for extended period, connection will be disconnected
- Spot trading: pingInterval typically 30000ms (30 seconds)
- Futures trading: pingInterval typically 18000ms (18 seconds)

### 1.6 Token Expiration and Reconnection

- **Token Validity:** 24 hours
- **Connection Duration:** Single connection expected to disconnect after 24 hours
- **Reconnection Strategy:**
  1. Obtain new token before 24-hour expiration
  2. Establish new WebSocket connection with new token
  3. Re-subscribe to all required topics

---

## 2. Message Format

### 2.1 Subscribe Message

```json
{
  "id": "1545910660739",
  "type": "subscribe",
  "topic": "/market/ticker:BTC-USDT,ETH-USDT",
  "privateChannel": false,
  "response": true
}
```

**Fields:**
- `id` (string/number): Unique value to identify request (client-generated, e.g., timestamp)
- `type` (string): "subscribe"
- `topic` (string): Topic to subscribe to, can include multiple symbols separated by commas
- `privateChannel` (boolean): Set to `true` for private channels, `false` for public (default: false)
- `response` (boolean): Whether server should return ack message (default: false)

### 2.2 Unsubscribe Message

```json
{
  "id": "1545910840805",
  "type": "unsubscribe",
  "topic": "/market/ticker:BTC-USDT,ETH-USDT",
  "privateChannel": false,
  "response": true
}
```

**Same fields as subscribe message, with `type` set to "unsubscribe"**

### 2.3 Acknowledgment (Ack) Message

When `response: true` is set in subscribe/unsubscribe request:

```json
{
  "id": "1545910840805",
  "type": "ack"
}
```

**Fields:**
- `id` (string/number): Same as request id
- `type` (string): "ack"

### 2.4 Server Response Message Format

```json
{
  "type": "message",
  "topic": "/market/ticker:BTC-USDT",
  "subject": "trade.ticker",
  "data": {
    // ... channel-specific data
  }
}
```

**Fields:**
- `type` (string): "message"
- `topic` (string): Topic that triggered this message
- `subject` (string): Message subject identifying event type
- `data` (object): Channel-specific data payload

### 2.5 Error Message Format

```json
{
  "id": "1545910660739",
  "type": "error",
  "code": "error_code",
  "data": "error description"
}
```

**Fields:**
- `id` (string/number): Same as request id (or connectId for connection errors)
- `type` (string): "error"
- `code` (string): Error code
- `data` (string): Error description

### 2.6 Generating Unique IDs

The `id` field should be a unique value for each request. Common approaches:
- **Timestamp:** `Date.now()` or epoch milliseconds (e.g., `1545910660739`)
- **UUID:** Generate unique identifier
- **Sequential:** Incremental counter

The same `id` will be returned in ack and error messages for request matching.

---

## 3. Public Channels - Spot

### 3.1 Ticker - `/market/ticker:{symbol}`

**Topic:** `/market/ticker:BTC-USDT`

**Subscribe Message:**
```json
{
  "id": "1545910660739",
  "type": "subscribe",
  "topic": "/market/ticker:BTC-USDT",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/market/ticker:BTC-USDT",
  "subject": "trade.ticker",
  "data": {
    "sequence": "1545896668986",
    "price": "67523",
    "size": "0.003",
    "bestAsk": "67524",
    "bestAskSize": "1.234",
    "bestBid": "67523",
    "bestBidSize": "2.456",
    "time": 1729843222921000000
  }
}
```

**Data Fields:**
- `sequence` (string): Sequence number
- `price` (string): Last traded price
- `size` (string): Last traded amount
- `bestAsk` (string): Best ask price
- `bestAskSize` (string): Best ask size
- `bestBid` (string): Best bid price
- `bestBidSize` (string): Best bid size
- `time` (number): Matching time of latest transaction (nanoseconds)

**Update Frequency:** Real-time (on each trade)

### 3.2 All Tickers - `/market/ticker:all`

**Topic:** `/market/ticker:all`

**Subscribe Message:**
```json
{
  "id": "1545910660740",
  "type": "subscribe",
  "topic": "/market/ticker:all",
  "response": true
}
```

**Response:** Similar to single ticker but includes all trading pairs

**Update Frequency:** Real-time

### 3.3 Match Execution - `/market/match:{symbol}`

**Topic:** `/market/match:BTC-USDT`

**Subscribe Message:**
```json
{
  "id": "1545910660741",
  "type": "subscribe",
  "topic": "/market/match:BTC-USDT",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/market/match:BTC-USDT",
  "subject": "trade.l3match",
  "data": {
    "sequence": "11067996711960577",
    "symbol": "BTC-USDT",
    "side": "buy",
    "size": "0.003",
    "price": "67523",
    "takerOrderId": "671b50161777ff00074c168d",
    "makerOrderId": "671b5007389355000701b1d3",
    "tradeId": "11067996711960577",
    "time": "1729843222921000000",
    "type": "match"
  }
}
```

**Data Fields:**
- `sequence` (string): Sequence number
- `symbol` (string): Trading pair
- `side` (string): Taker side ("buy" or "sell")
- `size` (string): Filled amount
- `price` (string): Filled price
- `takerOrderId` (string): Taker order ID
- `makerOrderId` (string): Maker order ID
- `tradeId` (string): Trade ID
- `time` (string): Trade execution timestamp (nanoseconds)
- `type` (string): Event type ("match")

**Update Frequency:** Real-time (on each trade)

**Note:** Supports up to 100 symbols per subscription

### 3.4 Level2 Orderbook - `/market/level2:{symbol}`

**Topic:** `/market/level2:BTC-USDT`

**Subscribe Message:**
```json
{
  "id": "1545910660742",
  "type": "subscribe",
  "topic": "/market/level2:BTC-USDT",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/market/level2:BTC-USDT",
  "subject": "trade.l2update",
  "data": {
    "sequenceStart": 1545896669105,
    "sequenceEnd": 1545896669106,
    "symbol": "BTC-USDT",
    "changes": {
      "asks": [
        ["67524", "100", "1545896669105"],
        ["67525", "200", "1545896669106"]
      ],
      "bids": [
        ["67523", "50", "1545896669105"]
      ]
    }
  }
}
```

**Data Fields:**
- `sequenceStart` (number): Starting sequence number of this update
- `sequenceEnd` (number): Ending sequence number of this update
- `symbol` (string): Trading pair
- `changes` (object): Orderbook changes
  - `asks` (array): Ask updates `[price, size, sequence]`
  - `bids` (array): Bid updates `[price, size, sequence]`

**Changes Array Format:** `[price, size, sequence]`
- `price` (string): Price level
- `size` (string): New size at this price (0 means remove level)
- `sequence` (string): Sequence of last modification at this price

**Update Frequency:** Real-time (incremental)

**Sequence Handling:**

To build and maintain a local orderbook:

1. Subscribe to `/market/level2:{symbol}`
2. Cache incoming WebSocket messages
3. Request REST snapshot via `GET /api/v1/market/orderbook/level2_{depth}?symbol={symbol}`
4. Discard all cached messages with `sequenceEnd <= snapshot.sequence`
5. Apply remaining cached messages where `sequenceStart <= snapshot.sequence + 1`
6. Apply new incoming messages ensuring:
   - `sequenceStart(new) <= sequenceEnd(old) + 1`
   - `sequenceEnd(new) > sequenceEnd(old)`

**Important:** The sequence number on each record in `changes` represents the last modification of that price level, not message continuity. Use `sequenceStart` and `sequenceEnd` for message continuity validation.

### 3.5 Klines/Candles - `/market/candles:{symbol}_{interval}`

**Topic:** `/market/candles:BTC-USDT_1hour`

**Subscribe Message:**
```json
{
  "id": "1545910660743",
  "type": "subscribe",
  "topic": "/market/candles:BTC-USDT_1hour",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/market/candles:BTC-USDT_1hour",
  "subject": "trade.candles.update",
  "data": {
    "symbol": "BTC-USDT",
    "candles": [
      "1545904980",  // Start time of candle cycle (second)
      "67000",       // Open price
      "67500",       // Close price
      "67600",       // High price
      "66900",       // Low price
      "123.456",     // Transaction volume
      "8234567.89"   // Transaction amount
    ],
    "time": 1545904980000000000
  }
}
```

**Candles Array Format:**
```
[0] Start time (seconds)
[1] Open price
[2] Close price
[3] High price
[4] Low price
[5] Transaction volume
[6] Transaction amount
```

**Supported Intervals:**
- `1min`, `3min`, `5min`, `15min`, `30min`
- `1hour`, `2hour`, `4hour`, `6hour`, `8hour`, `12hour`
- `1day`, `1week`, `1month`

**Update Frequency:** Real-time

### 3.6 Market Snapshot - `/market/snapshot:{market}`

**Topic:** `/market/snapshot:BTC` (for all BTC-based pairs)

**Subscribe Message:**
```json
{
  "id": "1545910660744",
  "type": "subscribe",
  "topic": "/market/snapshot:BTC",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/market/snapshot:BTC",
  "subject": "trade.snapshot",
  "data": {
    "sequence": "1545896669100",
    "data": [
      {
        "symbol": "BTC-USDT",
        "baseCurrency": "BTC",
        "quoteCurrency": "USDT",
        "buy": "67523",
        "sell": "67524",
        "lastTradedPrice": "67523",
        "high": "68000",
        "low": "66000",
        "changePrice": "1523",
        "changeRate": "0.0231",
        "vol": "1234.567",
        "volValue": "83456789.12",
        "board": 0,
        "mark": 0,
        "trading": true,
        "datetime": 1545896669000
      }
    ]
  }
}
```

**Data Fields:**
- `symbol` (string): Trading pair
- `baseCurrency` (string): Base currency
- `quoteCurrency` (string): Quote currency
- `buy` (string): Best bid price
- `sell` (string): Best ask price
- `lastTradedPrice` (string): Last traded price
- `high` (string): 24h high
- `low` (string): 24h low
- `changePrice` (string): 24h price change
- `changeRate` (string): 24h change rate
- `vol` (string): 24h volume
- `volValue` (string): 24h volume value
- `board` (number): Board identifier
- `mark` (number): Mark
- `trading` (boolean): Trading status
- `datetime` (number): Timestamp (milliseconds)

**Update Frequency:** Real-time

**Alternative:** `/market/snapshot:{symbol}` for specific symbol (e.g., `/market/snapshot:KCS-BTC`)

---

## 4. Public Channels - Futures

### 4.1 Ticker V2 - `/contractMarket/tickerV2:{symbol}`

**Topic:** `/contractMarket/tickerV2:XBTUSDTM`

**Subscribe Message:**
```json
{
  "id": "1545910660745",
  "type": "subscribe",
  "topic": "/contractMarket/tickerV2:XBTUSDTM",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/contractMarket/tickerV2:XBTUSDTM",
  "subject": "tickerV2",
  "data": {
    "symbol": "XBTUSDTM",
    "sequence": 45,
    "bestBidSize": 795,
    "bestBidPrice": "3200.0",
    "bestAskPrice": "3600.0",
    "bestAskSize": 284,
    "ts": 1553846081210004941
  }
}
```

**Data Fields:**
- `symbol` (string): Contract symbol
- `sequence` (number): Sequence number
- `bestBidPrice` (string): Best bid price
- `bestBidSize` (number): Best bid size (contracts)
- `bestAskPrice` (string): Best ask price
- `bestAskSize` (number): Best ask size (contracts)
- `ts` (number): Timestamp (nanoseconds)

**Update Frequency:** Real-time

**Note:** This is the recommended ticker endpoint for futures. The V1 endpoint `/contractMarket/ticker:{symbol}` is deprecated.

### 4.2 Ticker V1 (Deprecated) - `/contractMarket/ticker:{symbol}`

**Topic:** `/contractMarket/ticker:XBTUSDTM`

**Response Message:**
```json
{
  "type": "message",
  "topic": "/contractMarket/ticker:XBTUSDTM",
  "subject": "ticker",
  "data": {
    "symbol": "XBTUSDTM",
    "sequence": 45,
    "side": "sell",
    "price": "3600.0",
    "size": 16,
    "tradeId": "5c9dcf4170744d6f5a3d32fb",
    "bestBidSize": 795,
    "bestBidPrice": "3200.0",
    "bestAskPrice": "3600.0",
    "bestAskSize": 284,
    "ts": 1553846081210004941
  }
}
```

**Additional Fields (compared to V2):**
- `side` (string): Side of last trade ("buy" or "sell")
- `price` (string): Last trade price
- `size` (number): Last trade size
- `tradeId` (string): Last trade ID

**Update Frequency:** Real-time (only on match events; if multiple matches occur simultaneously, only last is pushed)

**Deprecation Note:** Not recommended. Use `/contractMarket/tickerV2:{symbol}` instead.

### 4.3 Match Execution - `/contractMarket/execution:{symbol}`

**Topic:** `/contractMarket/execution:XBTUSDTM`

**Subscribe Message:**
```json
{
  "id": "1545910660746",
  "type": "subscribe",
  "topic": "/contractMarket/execution:XBTUSDTM",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/contractMarket/execution:XBTUSDTM",
  "subject": "match",
  "data": {
    "symbol": "XBTUSDTM",
    "sequence": 36,
    "side": "buy",
    "size": 100,
    "price": "3600.0",
    "takerOrderId": "5c9dd00870744d71c43f5e25",
    "makerOrderId": "5c9dcf4170744d6f5a3d32fb",
    "tradeId": "5c9dd00970744d6f5a3d32fc",
    "ts": 1553846281210000000
  }
}
```

**Data Fields:**
- `symbol` (string): Contract symbol
- `sequence` (number): Sequence number
- `side` (string): Liquidity taker side ("buy" or "sell")
- `size` (number): Filled quantity (contracts)
- `price` (string): Filled price
- `takerOrderId` (string): Taker order ID
- `makerOrderId` (string): Maker order ID
- `tradeId` (string): Trade ID
- `ts` (number): Trade timestamp (nanoseconds)

**Update Frequency:** Real-time (on each trade)

### 4.4 Level2 Orderbook - `/contractMarket/level2:{symbol}`

**Topic:** `/contractMarket/level2:XBTUSDTM`

**Subscribe Message:**
```json
{
  "id": "1545910660747",
  "type": "subscribe",
  "topic": "/contractMarket/level2:XBTUSDTM",
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/contractMarket/level2:XBTUSDTM",
  "subject": "level2",
  "data": {
    "sequence": 18,
    "change": "5000.0,sell,83",
    "timestamp": 1551770400000
  }
}
```

**Data Fields:**
- `sequence` (number): Sequence number
- `change` (string): Change data in format `"{price},{side},{quantity}"`
- `timestamp` (number): Timestamp (milliseconds)

**Change String Format:** `"{price},{side},{quantity}"`
- `price` (string): Price level
- `side` (string): "buy" or "sell"
- `quantity` (number): New quantity at price (0 means remove level)

**Update Frequency:** Real-time (incremental)

**Sequence Handling (same as Spot):**
1. Subscribe to WebSocket level2 feed and cache messages
2. Request REST snapshot via `GET /api/v1/level2/snapshot?symbol={symbol}`
3. Discard cached messages with `sequence <= snapshot.sequence`
4. Apply remaining cached messages
5. Apply new messages ensuring sequence continuity

### 4.5 Instrument Data - `/contract/instrument:{symbol}`

**Topic:** `/contract/instrument:XBTUSDTM`

**Subscribe Message:**
```json
{
  "id": "1545910660748",
  "type": "subscribe",
  "topic": "/contract/instrument:XBTUSDTM",
  "response": true
}
```

**Response Message (Mark & Index Price):**
```json
{
  "type": "message",
  "topic": "/contract/instrument:XBTUSDTM",
  "subject": "mark.index.price",
  "data": {
    "symbol": "XBTUSDTM",
    "granularity": 1000,
    "indexPrice": 4000.23,
    "markPrice": 4010.52,
    "timestamp": 1551770400000
  }
}
```

**Data Fields:**
- `symbol` (string): Contract symbol
- `granularity` (number): Granularity (milliseconds)
- `indexPrice` (number): Index price
- `markPrice` (number): Mark price
- `timestamp` (number): Timestamp (milliseconds)

**Update Frequency:** Every 1 second (granularity: 1000ms)

### 4.6 Funding Rate - `/contract/instrument:{symbol}` (subject: funding.rate)

**Response Message:**
```json
{
  "type": "message",
  "topic": "/contract/instrument:XBTUSDTM",
  "subject": "funding.rate",
  "data": {
    "symbol": "XBTUSDTM",
    "fundingRate": 0.0001,
    "timestamp": 1551770400000
  }
}
```

**Data Fields:**
- `symbol` (string): Contract symbol
- `fundingRate` (number): Current funding rate
- `timestamp` (number): Timestamp (milliseconds)

**Update Frequency:** Every funding period (typically 8 hours)

---

## 5. Private Channels - Spot

All private channels require `"privateChannel": true` and a private token from `/api/v1/bullet-private` or `/api/v2/bullet-private`.

### 5.1 Order Updates V2 - `/spotMarket/tradeOrdersV2`

**Topic:** `/spotMarket/tradeOrdersV2`

**Subscribe Message:**
```json
{
  "id": "1545910660749",
  "type": "subscribe",
  "topic": "/spotMarket/tradeOrdersV2",
  "privateChannel": true,
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/spotMarket/tradeOrdersV2",
  "subject": "orderChange",
  "channelType": "private",
  "data": {
    "symbol": "BTC-USDT",
    "orderType": "limit",
    "side": "buy",
    "orderId": "5c35c02703aa673ceec2a168",
    "type": "open",
    "orderTime": 1547026471000,
    "size": "1.0",
    "filledSize": "0",
    "price": "67000",
    "clientOid": "my-order-id-001",
    "remainSize": "1.0",
    "status": "open",
    "ts": 1547026471000000000
  }
}
```

**Data Fields:**
- `symbol` (string): Trading pair
- `orderType` (string): Order type ("limit", "market", "stop_limit", etc.)
- `side` (string): Order side ("buy" or "sell")
- `orderId` (string): Order ID
- `type` (string): Event type ("open", "match", "filled", "canceled", "update")
- `orderTime` (number): Order creation time (milliseconds)
- `size` (string): Order size
- `filledSize` (string): Filled size
- `price` (string): Order price
- `clientOid` (string): Client order ID (optional)
- `remainSize` (string): Remaining size
- `status` (string): Order status ("new", "open", "done")
- `ts` (number): Event timestamp (nanoseconds)

**Order Statuses:**
- `new`: Order enters matching system (V2 only)
- `open`: Order enters order book as maker
- `match`: Taker order executes
- `done`: Order fully executed or canceled

**Update Frequency:** Real-time (on each order event)

**V1 vs V2 Difference:** V2 adds "new" status when order enters matching system.

### 5.2 Order Updates V1 (Deprecated) - `/spotMarket/tradeOrders`

**Topic:** `/spotMarket/tradeOrders`

Same as V2 but without "new" status. Use V2 instead.

### 5.3 Balance Updates - `/account/balance`

**Topic:** `/account/balance`

**Subscribe Message:**
```json
{
  "id": "1545910660750",
  "type": "subscribe",
  "topic": "/account/balance",
  "privateChannel": true,
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/account/balance",
  "subject": "account.balance",
  "channelType": "private",
  "data": {
    "accountId": "548674591753",
    "currency": "USDT",
    "total": "21.133773386762",
    "available": "20.132773386762",
    "hold": "1.001",
    "availableChange": "-0.5005",
    "holdChange": "0.5005",
    "relationContext": {
      "symbol": "BTC-USDT",
      "orderId": "6721d0632db25b0007071fdc",
      "tradeId": "11116472408358913"
    },
    "relationEvent": "trade.hold",
    "relationEventId": "354689988084000",
    "time": "1730269283892"
  }
}
```

**Data Fields:**
- `accountId` (string): Account ID
- `currency` (string): Currency code (e.g., "USDT", "BTC")
- `total` (string): Total balance
- `available` (string): Available balance
- `hold` (string): Held/frozen balance
- `availableChange` (string): Change in available balance
- `holdChange` (string): Change in held balance
- `relationContext` (object): Context of balance change
  - `symbol` (string): Related trading pair
  - `orderId` (string): Related order ID
  - `tradeId` (string): Related trade ID
- `relationEvent` (string): Event type causing balance change
- `relationEventId` (string): Event ID
- `time` (string): Timestamp (milliseconds)

**Update Frequency:** Real-time (on each balance change)

### 5.4 Trade Execution (Spot)

**Note:** Spot trade executions are included in order updates (`/spotMarket/tradeOrdersV2`) with `type: "match"`.

For dedicated execution channel, refer to Pro API private channels.

---

## 6. Private Channels - Futures

All private channels require `"privateChannel": true` and a private token.

### 6.1 Order Updates - `/contractMarket/tradeOrders`

**Topic:** `/contractMarket/tradeOrders` or `/contractMarket/tradeOrders:{symbol}`

**Subscribe Message (All Symbols):**
```json
{
  "id": "1545910660751",
  "type": "subscribe",
  "topic": "/contractMarket/tradeOrders",
  "privateChannel": true,
  "response": true
}
```

**Subscribe Message (Specific Symbol):**
```json
{
  "id": "1545910660752",
  "type": "subscribe",
  "topic": "/contractMarket/tradeOrders:XBTUSDTM",
  "privateChannel": true,
  "response": true
}
```

**Response Message:**
```json
{
  "type": "message",
  "topic": "/contractMarket/tradeOrders",
  "subject": "orderChange",
  "channelType": "private",
  "data": {
    "orderId": "5cdfc138b21023a909e5ad55",
    "symbol": "XBTUSDTM",
    "type": "match",
    "status": "open",
    "matchSize": 10,
    "matchPrice": "3600.0",
    "orderType": "limit",
    "side": "buy",
    "price": "3600.0",
    "size": 100,
    "remainSize": 90,
    "filledSize": 10,
    "canceledSize": 0,
    "tradeId": "5ce24c16b210233c36eexxxx",
    "clientOid": "my-futures-order-001",
    "orderTime": 1558167638000,
    "oldSize": 0,
    "liquidity": "taker",
    "ts": 1558167638000000000
  }
}
```

**Data Fields:**
- `orderId` (string): Order ID
- `symbol` (string): Contract symbol
- `type` (string): Event type ("open", "match", "filled", "canceled", "update")
- `status` (string): Order status ("open", "done")
- `matchSize` (number): Matched size in this event (contracts)
- `matchPrice` (string): Matched price in this event
- `orderType` (string): Order type ("limit", "market", "stop", etc.)
- `side` (string): Order side ("buy" or "sell")
- `price` (string): Order price
- `size` (number): Total order size (contracts)
- `remainSize` (number): Remaining unfilled size
- `filledSize` (number): Total filled size
- `canceledSize` (number): Canceled size
- `tradeId` (string): Trade ID (on match events)
- `clientOid` (string): Client order ID (optional)
- `orderTime` (number): Order creation time (milliseconds)
- `oldSize` (number): Previous order size (for update events)
- `liquidity` (string): "maker" or "taker"
- `ts` (number): Event timestamp (nanoseconds)

**Order Statuses:**
- `open`: Order active in order book
- `done`: Order fully filled or canceled

**Update Frequency:** Real-time (on each order event)

### 6.2 Balance Updates - `/contractAccount/wallet`

**Topic:** `/contractAccount/wallet`

**Subscribe Message:**
```json
{
  "id": "1545910660753",
  "type": "subscribe",
  "topic": "/contractAccount/wallet",
  "privateChannel": true,
  "response": true
}
```

**Response Message (Wallet Balance Event):**
```json
{
  "type": "message",
  "topic": "/contractAccount/wallet",
  "subject": "walletBalance.change",
  "channelType": "private",
  "data": {
    "currency": "USDT",
    "walletBalance": 10000.0,
    "availableBalance": 8500.0,
    "holdBalance": 1500.0,
    "isolatedOrderMargin": 200.0,
    "isolatedPosMargin": 300.0,
    "crossOrderMargin": 400.0,
    "crossPosMargin": 600.0,
    "equity": 9500.0,
    "timestamp": 1558167638000
  }
}
```

**Alternative Subjects (Legacy):**

**Order Margin Event:**
```json
{
  "subject": "orderMargin.change",
  "data": {
    "currency": "USDT",
    "orderMargin": 400.0,
    "timestamp": 1558167638000
  }
}
```

**Available Balance Event:**
```json
{
  "subject": "availableBalance.change",
  "data": {
    "currency": "USDT",
    "availableBalance": 8500.0,
    "holdBalance": 1500.0,
    "timestamp": 1558167638000
  }
}
```

**Withdrawal Hold Event:**
```json
{
  "subject": "withdrawHold.change",
  "data": {
    "currency": "USDT",
    "withdrawHold": 100.0,
    "timestamp": 1558167638000
  }
}
```

**Data Fields (Wallet Balance):**
- `currency` (string): Settlement currency (e.g., "USDT")
- `walletBalance` (number): Total wallet balance
- `availableBalance` (number): Available balance for trading
- `holdBalance` (number): Total frozen balance (positionMargin + orderMargin + frozenFunds)
- `isolatedOrderMargin` (number): Margin for isolated mode orders
- `isolatedPosMargin` (number): Margin for isolated mode positions
- `crossOrderMargin` (number): Margin for cross mode orders
- `crossPosMargin` (number): Margin for cross mode positions
- `equity` (number): Account equity
- `timestamp` (number): Timestamp (milliseconds)

**Note:** After first switch from isolated to cross margin mode, legacy subjects (orderMargin, availableBalance, withdrawHold) stop pushing and are replaced by `walletBalance.change`.

**Update Frequency:** Real-time (on each balance change)

### 6.3 Position Updates - `/contract/position:{symbol}`

**Topic:** `/contract/position:XBTUSDTM` or `/contract/positionAll` (all symbols)

**Subscribe Message (Specific Symbol):**
```json
{
  "id": "1545910660754",
  "type": "subscribe",
  "topic": "/contract/position:XBTUSDTM",
  "privateChannel": true,
  "response": true
}
```

**Subscribe Message (All Symbols):**
```json
{
  "id": "1545910660755",
  "type": "subscribe",
  "topic": "/contract/positionAll",
  "privateChannel": true,
  "response": true
}
```

**Response Message (Position Change by Operations):**
```json
{
  "type": "message",
  "topic": "/contract/position:XBTUSDTM",
  "subject": "position.change",
  "channelType": "private",
  "data": {
    "realisedGrossPnl": 0,
    "symbol": "XBTUSDTM",
    "crossMode": false,
    "liquidationPrice": 1000.0,
    "posLoss": 0,
    "avgEntryPrice": 7508.22,
    "unrealisedPnl": -0.00014735,
    "markPrice": 7947.83,
    "posMargin": 0.00266779,
    "autoDeposit": false,
    "riskLimit": 100000,
    "unrealisedCost": 0.00266375,
    "posComm": 0.00000392,
    "posMaint": 0.00001724,
    "posCost": 0.00266375,
    "maintMarginReq": 0.005,
    "bankruptPrice": 1000.0,
    "realisedCost": 0.00000271,
    "markValue": 0.00251640,
    "posInit": 0.00266375,
    "realisedPnl": -0.00000253,
    "maintMargin": 0.00252044,
    "realLeverage": 1.06,
    "changeReason": "positionChange",
    "currentCost": 0.00266375,
    "openingTimestamp": 1558433191000,
    "currentQty": 20,
    "delevPercentage": 0.52,
    "currentComm": 0.00000271,
    "realisedGrossCost": 0,
    "isOpen": true,
    "posCross": 1.2e-7,
    "currentTimestamp": 1558506060394,
    "unrealisedRoePcnt": -0.0553,
    "unrealisedPnlPcnt": -0.0553,
    "settleCurrency": "USDT",
    "leverage": 20
  }
}
```

**Response Message (Position Change by Mark Price):**
```json
{
  "subject": "position.change",
  "channelType": "private",
  "data": {
    // Similar fields as above, with updated mark price-related values
    "changeReason": "markPriceChange"
  }
}
```

**Response Message (Funding Settlement):**
```json
{
  "type": "message",
  "topic": "/contract/position:XBTUSDTM",
  "subject": "position.settlement",
  "channelType": "private",
  "data": {
    "fundingTime": 1558435200000,
    "qty": 100,
    "fundingRate": 0.0001,
    "fundingFee": -0.000125,
    "ts": 1558435200000000000,
    "settleCurrency": "USDT"
  }
}
```

**Data Fields (Position Change):**
- `symbol` (string): Contract symbol
- `crossMode` (boolean): Cross margin mode enabled
- `currentQty` (number): Current position size (contracts, negative for short)
- `avgEntryPrice` (number): Average entry price
- `markPrice` (number): Current mark price
- `unrealisedPnl` (number): Unrealized PnL
- `realisedPnl` (number): Realized PnL
- `leverage` (number): Leverage
- `realLeverage` (number): Real leverage
- `posMargin` (number): Position margin
- `maintMargin` (number): Maintenance margin
- `liquidationPrice` (number): Liquidation price
- `bankruptPrice` (number): Bankruptcy price
- `settleCurrency` (string): Settlement currency
- `isOpen` (boolean): Position is open
- `changeReason` (string): Reason for update ("positionChange", "markPriceChange")
- `currentTimestamp` (number): Timestamp (milliseconds)

**Data Fields (Funding Settlement):**
- `fundingTime` (number): Funding settlement time (milliseconds)
- `qty` (number): Position quantity at settlement
- `fundingRate` (number): Funding rate applied
- `fundingFee` (number): Funding fee (negative = paid, positive = received)
- `ts` (number): Timestamp (nanoseconds)
- `settleCurrency` (string): Settlement currency

**Update Frequency:**
- Position change: Real-time (on position changes or mark price updates)
- Funding settlement: Every funding period (typically 8 hours)

---

## 7. Important Details

### 7.1 Symbol Format

**Spot:**
- Format: `{BASE}-{QUOTE}`
- Example: `BTC-USDT`, `ETH-USDT`, `KCS-BTC`
- Separator: Hyphen (`-`)

**Futures:**
- Perpetual contracts: `{BASE}{QUOTE}M` or `{BASE}{QUOTE}TM`
- Example: `XBTUSDTM` (BTC-USDT perpetual mini contract)
- `M` suffix: Mini contract (0.001 BTC per contract)
- No expiry date for perpetual contracts

**Index Symbols:**
- Format: `.K{BASE}{QUOTE}`
- Example: `.KXBTUSDT` (BTC-USDT spot index)

### 7.2 Sequence Numbers and Orderbook Handling

**Purpose:** Sequence numbers ensure message ordering and prevent race conditions when building local orderbooks.

**Key Points:**
- `sequenceStart` and `sequenceEnd` in messages indicate message boundaries
- Individual `sequence` in price level changes represents last modification time of that level
- **Do not** use individual price-level sequences for message continuity validation
- **Do** use `sequenceStart` and `sequenceEnd` for continuity

**Calibration Procedure:**

1. **Subscribe** to WebSocket orderbook topic
2. **Cache** all incoming messages temporarily
3. **Request REST snapshot** via `GET /api/v1/market/orderbook/level2_100?symbol={symbol}`
4. **Discard** cached messages where `sequenceEnd <= snapshot.sequence`
5. **Apply** cached messages where `sequenceStart <= snapshot.sequence + 1`
6. **Validate** new messages: `sequenceStart(new) <= sequenceEnd(old) + 1` and `sequenceEnd(new) > sequenceEnd(old)`
7. **Drop** messages that don't meet validation criteria

**Example Scenario:**

```
Snapshot sequence: 100

Cached messages:
- Message A: sequenceStart=99, sequenceEnd=101  -> Apply (99 <= 100+1)
- Message B: sequenceStart=102, sequenceEnd=104 -> Apply (102 <= 101+1)
- Message C: sequenceStart=50, sequenceEnd=60   -> Discard (60 <= 100)

New incoming message:
- Message D: sequenceStart=105, sequenceEnd=107
  Validation: 105 <= 104+1 ✓ and 107 > 104 ✓ -> Apply
```

### 7.3 Rate Limits

**Subscription Limits:**
- **Batch subscriptions:** Maximum 100 topics per subscribe message
- **Per connection:** Maximum 500 topics subscribed
- **Uplink messages:** Maximum 100 messages per 10 seconds sent to server

**Connection Limits:**
- **Token validity:** 24 hours
- **Connection duration:** Single connection expected to disconnect after 24 hours
- **Max connections per account:** Not explicitly documented (no hard limit mentioned)

**Message Size:**
- No explicit message size limit documented
- Use reasonable batch sizes (<=100 topics per subscribe)

### 7.4 Connection Management

**Best Practices:**

1. **Token Renewal:**
   - Obtain new token before 24-hour expiration
   - Establish new connection with new token
   - Gracefully close old connection after new one is ready

2. **Ping/Pong:**
   - Send ping every `pingInterval` milliseconds (from welcome message)
   - Expect pong response within `pingTimeout` milliseconds
   - Reconnect if pong not received

3. **Reconnection Strategy:**
   - Implement exponential backoff for reconnection attempts
   - Maintain subscription list to re-subscribe after reconnection
   - Use orderbook calibration procedure after reconnection

4. **Error Handling:**
   - Monitor for error messages with `type: "error"`
   - Log `connectId`/`sessionId` for troubleshooting
   - Handle subscription errors gracefully (invalid topic, permission denied, etc.)

### 7.5 Private Channel Authentication

**Steps:**

1. **Generate API credentials:**
   - API Key
   - API Secret
   - API Passphrase

2. **Request private token:**
   - Endpoint: `GET /api/v1/bullet-private` (Classic) or `GET /api/v2/bullet-private` (Pro)
   - Sign request with API credentials (KC-API-KEY, KC-API-SIGN, KC-API-TIMESTAMP, KC-API-KEY-VERSION headers)

3. **Connect WebSocket:**
   - Use `endpoint` from token response
   - Append `token` as query parameter: `wss://wsapi-push.kucoin.com/?token={token}`

4. **Subscribe to private topics:**
   - Set `"privateChannel": true` in subscribe message
   - Use topics like `/spotMarket/tradeOrdersV2`, `/account/balance`, `/contractMarket/tradeOrders`, etc.

**Security Notes:**
- Never share API credentials or tokens
- Use IP whitelist for API keys when possible
- Regenerate tokens before 24-hour expiration
- Tokens are tied to specific API keys; revoking key invalidates tokens

### 7.6 Multi-Symbol Subscriptions

**Comma-Separated:**
Topics support comma-separated symbol lists:

```json
{
  "id": "123",
  "type": "subscribe",
  "topic": "/market/ticker:BTC-USDT,ETH-USDT,LTC-USDT",
  "response": true
}
```

**Limits:**
- Maximum 100 symbols per subscribe message
- Maximum 500 total subscriptions per connection

**All Symbols:**
Some topics support wildcard or "all" subscriptions:
- `/market/ticker:all` - All spot tickers
- `/contract/positionAll` - All futures positions (private)

### 7.7 Timestamp Precision

**Milliseconds:**
- Most REST API timestamps
- Welcome message `pingInterval` and `pingTimeout`
- Orderbook `timestamp` fields
- Balance update `time` fields

**Nanoseconds:**
- Trade execution `time` fields
- Ticker `time` fields (spot)
- Match execution `ts` fields (futures)
- Order update `ts` fields

**Convert nanoseconds to milliseconds:** `nanoseconds / 1_000_000`

### 7.8 WebSocket vs REST

**When to use WebSocket:**
- Real-time price updates (tickers, orderbook)
- Order and position monitoring
- Balance updates
- High-frequency data consumption

**When to use REST:**
- Initial data retrieval (historical klines, order history)
- Orderbook snapshots for calibration
- One-time queries
- Account management operations

**Hybrid Approach:**
- Use REST for initial snapshot
- Use WebSocket for incremental updates
- Periodically re-sync with REST snapshots to prevent drift

---

## Sources

- [KuCoin WebSocket Introduction](https://www.kucoin.com/docs/websocket/introduction)
- [KuCoin WebSocket Basic Info](https://www.kucoin.com/docs/websocket/basic-info/apply-connect-token/introduction)
- [Get Public Token - Spot/Margin](https://www.kucoin.com/docs-new/websocket-api/base-info/get-public-token-spot-margin)
- [Get Public Token - Futures](https://www.kucoin.com/docs-new/websocket-api/base-info/get-public-token-futures)
- [WebSocket Subscribe](https://www.kucoin.com/docs/websocket/basic-info/subscribe/introduction)
- [WebSocket Ping/Pong](https://www.kucoin.com/docs/websocket/basic-info/ping)
- [Spot Ticker Channel](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/ticker)
- [Spot Match Execution](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/match-execution-data)
- [Spot Level2 Orderbook](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/level2-market-data)
- [Spot Klines Channel](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/klines)
- [Spot Market Snapshot](https://www.kucoin.com/docs/websocket/spot-trading/public-channels/market-snapshot)
- [Spot Private Order Updates V2](https://www.kucoin.com/docs/websocket/spot-trading/private-channels/private-order-change-v2)
- [Spot Balance Updates](https://www.kucoin.com/docs/websocket/spot-trading/private-channels/account-balance-change)
- [Futures Ticker V2](https://www.kucoin.com/docs/websocket/futures-trading/public-channels/get-ticker-v2)
- [Futures Match Execution](https://www.kucoin.com/docs/websocket/futures-trading/public-channels/match-execution-data)
- [Futures Level2 Orderbook](https://www.kucoin.com/docs/websocket/futures-trading/public-channels/level2-market-data)
- [Futures Instrument Data](https://www.kucoin.com/docs/websocket/futures-trading/public-channels/contract-market-data)
- [Futures Order Updates](https://www.kucoin.com/docs/websocket/futures-trading/private-channels/trade-orders)
- [Futures Balance Updates](https://www.kucoin.com/docs/websocket/futures-trading/private-channels/account-balance-events)
- [Futures Position Updates](https://www.kucoin.com/docs/websocket/futures-trading/private-channels/position-change-events)
- [WebSocket Rate Limits](https://www.kucoin.com/docs/basic-info/request-rate-limit/websocket)
- [KuCoin API GitHub - Level2 Calibration Issue](https://github.com/Kucoin/kucoin-api-docs/issues/305)
- [KuCoin Node.js SDK - Level2 Demo](https://github.com/Kucoin/kucoin-node-sdk/blob/master/demo/level2_demo.js)
- [Python KuCoin - WebSocket Documentation](https://python-kucoin.readthedocs.io/en/latest/websockets.html)
- [XBTUSDTM Contract Specifications](https://www.kucoin.com/futures/contract/detail/XBTUSDTM)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Research Completed By:** Claude Code (Sonnet 4.5)
