# Bybit V5 WebSocket API Documentation

Comprehensive research on Bybit V5 WebSocket API for Spot and Futures (Linear) trading.

**Research Date:** 2026-01-20

---

## 1. Connection Setup

### 1.1 WebSocket Endpoint URLs

**Public Streams (Mainnet):**
- Spot: `wss://stream.bybit.com/v5/public/spot`
- Linear (USDT Perpetual): `wss://stream.bybit.com/v5/public/linear`
- Inverse Contracts: `wss://stream.bybit.com/v5/public/inverse`
- Options: `wss://stream.bybit.com/v5/public/option`

**Private Streams (Mainnet):**
- Unified: `wss://stream.bybit.com/v5/private`

**Testnet:**
- Public Spot: `wss://stream-testnet.bybit.com/v5/public/spot`
- Public Linear: `wss://stream-testnet.bybit.com/v5/public/linear`
- Private: `wss://stream-testnet.bybit.com/v5/private`

**Note**: No token endpoint required for Bybit (unlike KuCoin's bullet-public/private)

### 1.2 Connection Process

1. **Establish WebSocket Connection**
   - Connect directly to the appropriate endpoint URL
   - No pre-authentication token fetch required

2. **Authentication (Private Channels Only)**
   - Send authentication message after connection
   - Use HMAC-SHA256 signature with API credentials

3. **Subscribe to Channels**
   - Send subscription message with desired topics
   - Can subscribe to multiple topics in single message

### 1.3 Authentication (Private Channels)

**Authentication Message Format:**
```json
{
  "req_id": "unique_id",
  "op": "auth",
  "args": ["<api_key>", "<expires>", "<signature>"]
}
```

**Signature Generation:**
```
String to sign: GET/realtime<expires>
Signature: HMAC_SHA256(api_secret, string_to_sign)
Output: Hexadecimal string
```

**Example:**
```python
import hmac
import hashlib
import time

api_key = "XXXXXXXXXX"
api_secret = "XXXXXXXXXX"
expires = int((time.time() + 10) * 1000)  # 10 seconds from now, in milliseconds

# String to sign
str_to_sign = f"GET/realtime{expires}"

# Generate signature
signature = hmac.new(
    api_secret.encode('utf-8'),
    str_to_sign.encode('utf-8'),
    hashlib.sha256
).hexdigest()

# Auth message
auth_message = {
    "req_id": "auth001",
    "op": "auth",
    "args": [api_key, str(expires), signature]
}
```

**Response:**
```json
{
  "success": true,
  "ret_msg": "",
  "op": "auth",
  "conn_id": "xxx-xxx-xxx-xxx"
}
```

### 1.4 Heartbeat (Ping/Pong)

**Client Ping:**
Send ping every **20 seconds** to maintain connection:

```json
{
  "req_id": "ping001",
  "op": "ping"
}
```

**Server Pong:**
```json
{
  "success": true,
  "ret_msg": "pong",
  "op": "ping",
  "conn_id": "xxx-xxx-xxx-xxx"
}
```

**Important:**
- Send ping every 20 seconds
- Connection will be dropped if no message sent for extended period
- Receiving data messages also keeps connection alive

---

## 2. Message Format

### 2.1 Subscribe Message

```json
{
  "req_id": "sub001",
  "op": "subscribe",
  "args": [
    "orderbook.1.BTCUSDT",
    "publicTrade.BTCUSDT"
  ]
}
```

**Fields:**
- `req_id` (string): Unique request identifier (client-generated)
- `op` (string): "subscribe"
- `args` (array): Array of topic strings to subscribe to

**Multiple Topics:**
```json
{
  "req_id": "sub002",
  "op": "subscribe",
  "args": [
    "orderbook.1.BTCUSDT",
    "orderbook.1.ETHUSDT",
    "publicTrade.BTCUSDT",
    "kline.60.BTCUSDT"
  ]
}
```

**Limits:**
- Spot: Up to **10 args** per subscription request
- Linear/Inverse: No explicit limit documented
- Maximum **21,000 characters** for args array

### 2.2 Unsubscribe Message

```json
{
  "req_id": "unsub001",
  "op": "unsubscribe",
  "args": [
    "orderbook.1.BTCUSDT"
  ]
}
```

**Same format as subscribe, with `op` set to "unsubscribe"**

### 2.3 Acknowledgment Message

**Subscribe Success:**
```json
{
  "success": true,
  "ret_msg": "",
  "op": "subscribe",
  "conn_id": "xxx-xxx-xxx-xxx"
}
```

**Subscribe Failure:**
```json
{
  "success": false,
  "ret_msg": "error message",
  "op": "subscribe",
  "conn_id": "xxx-xxx-xxx-xxx"
}
```

### 2.4 Server Data Message Format

```json
{
  "topic": "orderbook.1.BTCUSDT",
  "type": "snapshot",
  "ts": 1702617474601,
  "data": {
    // channel-specific data
  }
}
```

**Fields:**
- `topic` (string): Topic that triggered this message
- `type` (string): Message type ("snapshot", "delta")
- `ts` (number): Timestamp in milliseconds
- `data` (object): Channel-specific data payload

---

## 3. Public Channels - Spot

### 3.1 Orderbook - `orderbook.{depth}.{symbol}`

**Topic:** `orderbook.1.BTCUSDT` or `orderbook.50.BTCUSDT`

**Available Depths:**
- `1` - Best bid/ask only
- `50` - 50 levels each side
- `200` - 200 levels each side (spot only)
- `500` - 500 levels each side (linear/inverse)

**Subscribe:**
```json
{
  "req_id": "ob001",
  "op": "subscribe",
  "args": ["orderbook.50.BTCUSDT"]
}
```

**Snapshot Message:**
```json
{
  "topic": "orderbook.50.BTCUSDT",
  "type": "snapshot",
  "ts": 1702617474601,
  "data": {
    "s": "BTCUSDT",
    "b": [
      ["39999.00", "1.5"],
      ["39998.00", "2.3"]
    ],
    "a": [
      ["40001.00", "1.8"],
      ["40002.00", "2.1"]
    ],
    "u": 123456,
    "seq": 7890123
  }
}
```

**Delta Message:**
```json
{
  "topic": "orderbook.50.BTCUSDT",
  "type": "delta",
  "ts": 1702617474602,
  "data": {
    "s": "BTCUSDT",
    "b": [
      ["39999.00", "2.0"]  // Updated bid
    ],
    "a": [
      ["40001.00", "0"]    // Removed ask (size = 0)
    ],
    "u": 123457,
    "seq": 7890124
  }
}
```

**Data Fields:**
- `s`: Symbol name
- `b`: Bids array `[price, size]`
- `a`: Asks array `[price, size]`
- `u`: Update ID
- `seq`: Sequence number

**Update Frequency:**
- Level 1000: Every **100ms**
- Level 200: Every **20ms** (linear), Every **100ms** (spot)
- Level 50: Every **20ms**

**Orderbook Maintenance:**
1. Receive snapshot message first
2. Apply delta messages incrementally
3. If size = "0", remove that price level
4. Use `u` (update ID) for sequencing

### 3.2 Public Trade - `publicTrade.{symbol}`

**Topic:** `publicTrade.BTCUSDT`

**Subscribe:**
```json
{
  "req_id": "trade001",
  "op": "subscribe",
  "args": ["publicTrade.BTCUSDT"]
}
```

**Message:**
```json
{
  "topic": "publicTrade.BTCUSDT",
  "type": "snapshot",
  "ts": 1702617474601,
  "data": [
    {
      "T": 1702617474601,
      "s": "BTCUSDT",
      "S": "Buy",
      "v": "0.01",
      "p": "40000.00",
      "L": "PlusTick",
      "i": "trade-id-123",
      "BT": false
    }
  ]
}
```

**Data Fields:**
- `T`: Trade timestamp (milliseconds)
- `s`: Symbol
- `S`: Side ("Buy" or "Sell")
- `v`: Volume (trade size)
- `p`: Price
- `L`: Tick direction ("PlusTick", "ZeroPlusTick", "MinusTick", "ZeroMinusTick")
- `i`: Trade ID
- `BT`: Whether block trade

**Update Frequency:** Real-time (on each trade)

### 3.3 Kline/Candlestick - `kline.{interval}.{symbol}`

**Topic:** `kline.60.BTCUSDT`

**Available Intervals:**
- Minutes: `1`, `3`, `5`, `15`, `30`, `60`, `120`, `240`, `360`, `720`
- Day/Week/Month: `D`, `W`, `M`

**Subscribe:**
```json
{
  "req_id": "kline001",
  "op": "subscribe",
  "args": ["kline.60.BTCUSDT"]
}
```

**Message:**
```json
{
  "topic": "kline.60.BTCUSDT",
  "type": "snapshot",
  "ts": 1702617474601,
  "data": [
    {
      "start": 1702617000000,
      "end": 1702617060000,
      "interval": "60",
      "open": "40000.00",
      "close": "40100.00",
      "high": "40150.00",
      "low": "39950.00",
      "volume": "123.456",
      "turnover": "4950000.00",
      "confirm": false,
      "timestamp": 1702617474601
    }
  ]
}
```

**Data Fields:**
- `start`: Candle start time (milliseconds)
- `end`: Candle end time (milliseconds)
- `interval`: Interval string
- `open`, `close`, `high`, `low`: OHLC prices
- `volume`: Trading volume
- `turnover`: Trading turnover
- `confirm`: Whether candle is finalized (true when interval completes)
- `timestamp`: Update timestamp

**Update Frequency:** Real-time during candle formation, final update when `confirm: true`

### 3.4 Ticker - `tickers.{symbol}`

**Topic:** `tickers.BTCUSDT`

**Subscribe:**
```json
{
  "req_id": "ticker001",
  "op": "subscribe",
  "args": ["tickers.BTCUSDT"]
}
```

**Message:**
```json
{
  "topic": "tickers.BTCUSDT",
  "type": "snapshot",
  "ts": 1702617474601,
  "data": {
    "symbol": "BTCUSDT",
    "lastPrice": "40000.00",
    "highPrice24h": "41000.00",
    "lowPrice24h": "39000.00",
    "volume24h": "1234.567",
    "turnover24h": "49500000.00",
    "price24hPcnt": "0.0125"
  }
}
```

**Data Fields:** Similar to REST ticker response

**Update Frequency:** Real-time (100ms push frequency)

---

## 4. Public Channels - Futures (Linear)

### 4.1 Orderbook - `orderbook.{depth}.{symbol}`

Same as Spot, but available depths:
- `1`, `50`, `200`, `500`

**Topic:** `orderbook.50.BTCUSDT` (on `wss://stream.bybit.com/v5/public/linear`)

### 4.2 Public Trade - `publicTrade.{symbol}`

Same format as Spot

### 4.3 Kline - `kline.{interval}.{symbol}`

Same format as Spot

### 4.4 Ticker - `tickers.{symbol}`

Similar to Spot, with additional futures-specific fields:

```json
{
  "topic": "tickers.BTCUSDT",
  "type": "snapshot",
  "ts": 1702617474601,
  "data": {
    "symbol": "BTCUSDT",
    "lastPrice": "40000.00",
    "markPrice": "40005.00",
    "indexPrice": "40003.00",
    "fundingRate": "0.0001",
    "nextFundingTime": "1702620000000",
    "openInterest": "50000.00",
    "openInterestValue": "2000000000.00",
    // ... other ticker fields
  }
}
```

**Additional Futures Fields:**
- `markPrice`: Mark price
- `indexPrice`: Index price
- `fundingRate`: Current funding rate
- `nextFundingTime`: Next funding timestamp
- `openInterest`: Open interest
- `openInterestValue`: Open interest value in USD

---

## 5. Private Channels

### 5.1 Order Updates - `order`

**Topic:** `order`

**Subscribe (after authentication):**
```json
{
  "req_id": "order001",
  "op": "subscribe",
  "args": ["order"]
}
```

**Message:**
```json
{
  "id": "xxx-xxx-xxx",
  "topic": "order",
  "creationTime": 1702617474601,
  "data": [
    {
      "orderId": "order-id-123",
      "orderLinkId": "custom-123",
      "symbol": "BTCUSDT",
      "side": "Buy",
      "orderType": "Limit",
      "price": "40000.00",
      "qty": "0.01",
      "leavesQty": "0.005",
      "cumExecQty": "0.005",
      "cumExecValue": "200.00",
      "avgPrice": "40000.00",
      "orderStatus": "PartiallyFilled",
      "timeInForce": "GTC",
      "createdTime": "1702617400000",
      "updatedTime": "1702617474601",
      "category": "spot"
    }
  ]
}
```

**Order Statuses:**
- `Created`: Order created
- `New`: Order entered order book
- `PartiallyFilled`: Partially filled
- `Filled`: Fully filled
- `Cancelled`: Cancelled
- `Rejected`: Rejected

**Update Frequency:** Real-time on order events

### 5.2 Execution (Trade) - `execution`

**Topic:** `execution`

**Subscribe:**
```json
{
  "req_id": "exec001",
  "op": "subscribe",
  "args": ["execution"]
}
```

**Message:**
```json
{
  "id": "xxx-xxx-xxx",
  "topic": "execution",
  "creationTime": 1702617474601,
  "data": [
    {
      "orderId": "order-id-123",
      "orderLinkId": "custom-123",
      "symbol": "BTCUSDT",
      "side": "Buy",
      "execId": "exec-id-456",
      "execPrice": "40000.00",
      "execQty": "0.005",
      "execValue": "200.00",
      "execFee": "0.08",
      "feeRate": "0.0004",
      "execTime": "1702617474601",
      "isMaker": false,
      "category": "spot"
    }
  ]
}
```

**Data Fields:**
- `execId`: Execution ID
- `execPrice`: Execution price
- `execQty`: Executed quantity
- `execValue`: Execution value
- `execFee`: Fee charged
- `feeRate`: Fee rate applied
- `execTime`: Execution timestamp
- `isMaker`: Whether maker order

**Update Frequency:** Real-time on each execution

### 5.3 Wallet - `wallet`

**Topic:** `wallet`

**Subscribe:**
```json
{
  "req_id": "wallet001",
  "op": "subscribe",
  "args": ["wallet"]
}
```

**Message:**
```json
{
  "id": "xxx-xxx-xxx",
  "topic": "wallet",
  "creationTime": 1702617474601,
  "data": [
    {
      "accountType": "UNIFIED",
      "accountIMRate": "0.0250",
      "accountMMRate": "0.0100",
      "totalEquity": "50000.00",
      "totalWalletBalance": "50000.00",
      "totalMarginBalance": "50000.00",
      "totalAvailableBalance": "48750.00",
      "totalPerpUPL": "0.00",
      "totalInitialMargin": "1250.00",
      "totalMaintenanceMargin": "500.00",
      "coin": [
        {
          "coin": "USDT",
          "equity": "10000.00",
          "usdValue": "10000.00",
          "walletBalance": "10000.00",
          "availableToWithdraw": "9800.00",
          "locked": "200.00",
          "unrealisedPnl": "0.00"
        }
      ]
    }
  ]
}
```

**Update Frequency:** Real-time on balance changes

### 5.4 Position - `position`

**Topic:** `position`

**Subscribe:**
```json
{
  "req_id": "position001",
  "op": "subscribe",
  "args": ["position"]
}
```

**Message:**
```json
{
  "id": "xxx-xxx-xxx",
  "topic": "position",
  "creationTime": 1702617474601,
  "data": [
    {
      "positionIdx": 0,
      "symbol": "BTCUSDT",
      "side": "Buy",
      "size": "0.5",
      "positionValue": "20000.00",
      "avgPrice": "40000.00",
      "markPrice": "40100.00",
      "liqPrice": "35000.00",
      "leverage": "50",
      "unrealisedPnl": "50.00",
      "cumRealisedPnl": "120.00",
      "positionStatus": "Normal",
      "updatedTime": "1702617474601",
      "category": "linear"
    }
  ]
}
```

**Update Frequency:** Real-time on position changes

---

## 6. Important Details

### 6.1 Connection Management

**Reconnection Strategy:**
1. Implement exponential backoff for reconnection attempts
2. Re-authenticate after reconnection (private channels)
3. Re-subscribe to all previous topics
4. Request orderbook snapshot if maintaining local orderbook

**Connection Limits:**
- Maximum 500 new connections per 5 minutes per IP
- Maximum 1,000 total concurrent connections per IP

### 6.2 Message Sequencing

**Orderbook Sequencing:**
- Use `u` (update ID) field for message ordering
- `seq` field for cross-sequence comparison
- Always process messages in order of `u`

**Trade Sequencing:**
- Use `i` (trade ID) field
- Timestamps may not be strictly ordered

### 6.3 Subscription Limits

**Per Connection:**
- Spot: Maximum 10 topics per subscription request
- Maximum 21,000 characters for args array
- Total subscriptions per connection: Not explicitly limited

**Best Practice:**
- Keep subscriptions under 100 per connection
- Use multiple connections for high topic counts

### 6.4 Error Handling

**Common Errors:**
- Invalid topic format
- Subscription limit exceeded
- Authentication failed (private channels)
- Connection closed by server

**Error Message Format:**
```json
{
  "success": false,
  "ret_msg": "error description",
  "op": "subscribe",
  "conn_id": "xxx-xxx-xxx-xxx"
}
```

---

## 7. Comparison with KuCoin WebSocket

| Feature | Bybit V5 | KuCoin |
|---------|----------|---------|
| Token fetch | ❌ Not required | ✅ Required (bullet-public/private) |
| Authentication | In-band (auth message) | Via token in URL |
| Ping interval | 20 seconds | 18-30 seconds |
| Subscription format | `{"op":"subscribe","args":[...]}` | `{"type":"subscribe","topic":"..."}` |
| Topic format | `orderbook.50.BTCUSDT` | `/market/level2:BTC-USDT` |
| Message type field | `type` ("snapshot"/"delta") | `subject` |
| Orderbook updates | Delta with snapshot | Delta with calibration |
| Timestamp format | Always milliseconds | Mixed (nanoseconds for some) |
| Private channels | Single connection, multiple topics | Separate topics per channel |

**Key Advantages of Bybit:**
- No token fetch step (simpler connection)
- Consistent timestamp format
- Clearer topic naming
- Unified connection for all private channels

**Key Advantages of KuCoin:**
- Token-based auth (stateless)
- More granular channel types
- Explicit sequence handling documentation

---

## Sources

Research compiled from official Bybit V5 WebSocket documentation:

- [Connect | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/ws/connect)
- [Orderbook | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/websocket/public/orderbook)
- [Ticker | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/websocket/public/ticker)
- [Trade | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/websocket/public/trade)
- [Bybit V5 Changelog](https://bybit-exchange.github.io/docs/changelog/v5)
- [Step-by-step guide: collecting tick data from Bybit](https://medium.com/@eeiaao/step-by-step-guide-collecting-tick-data-from-bybit-trades-order-book-b33a206baf08)
- [pybit WebSocket Examples](https://github.com/bybit-exchange/pybit/blob/master/examples/websocket_example_explanatory.py)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Key Finding:** Simpler connection than KuCoin (no token fetch), consistent millisecond timestamps
