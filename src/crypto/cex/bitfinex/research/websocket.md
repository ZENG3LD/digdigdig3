# Bitfinex WebSocket API v2

## Connection URLs

- **Public Channels**: `wss://api-pub.bitfinex.com/ws/2`
- **Authenticated Channels**: `wss://api.bitfinex.com/ws/2`

## Connection Limits

### Public Connections
- **Limit**: 20 connections per minute
- **Penalty**: 60-second rate limit if exceeded

### Authenticated Connections
- **Limit**: 5 connections per 15 seconds
- **Penalty**: 15-second rate limit if exceeded

### Channel Subscriptions
- **Limit**: 30 subscriptions per connection
  - 25 public market data channels
  - 1 channel reserved for account info (on authenticated connections)

## General Message Format

All WebSocket messages are JSON objects or arrays.

### Event Messages (Objects)
```json
{
  "event": "event_name",
  "key": "value",
  ...
}
```

### Data Messages (Arrays)
```json
[CHANNEL_ID, DATA]
```

## Connection Lifecycle

### 1. Connect
```javascript
const ws = new WebSocket('wss://api-pub.bitfinex.com/ws/2');
```

### 2. Connection Confirmation
**Server sends**:
```json
{
  "event": "info",
  "version": 2,
  "serverId": "server-id",
  "platform": {
    "status": 1
  }
}
```

### 3. Configure (Optional)
**Client sends**:
```json
{
  "event": "conf",
  "flags": 65536
}
```

**Available Flags**:
- `65536` (DEC_S): Enable all decimal as strings (recommended)
- `32768` (TIME_S): Enable all times as date strings
- `131072` (TIMESTAMP): Timestamp in milliseconds
- `536870912` (SEQ_ALL): Enable sequencing
- `32` (CHECKSUM): Enable checksums for books

### 4. Subscribe to Channels
See channel-specific sections below.

### 5. Receive Data
See channel-specific sections below.

### 6. Unsubscribe
```json
{
  "event": "unsubscribe",
  "chanId": CHANNEL_ID
}
```

### 7. Disconnect
Close WebSocket connection.

## Heartbeat & Ping/Pong

### Heartbeat
**Server sends every 15 seconds**:
```json
[CHANNEL_ID, "hb"]
```

**Purpose**: Indicates connection is alive, no new data.

### Ping/Pong
**Client sends**:
```json
{
  "event": "ping",
  "cid": 1234
}
```

**Server responds**:
```json
{
  "event": "pong",
  "cid": 1234,
  "ts": 1234567890000
}
```

## Public Channels

### Ticker Channel

**Subscribe**:
```json
{
  "event": "subscribe",
  "channel": "ticker",
  "symbol": "tBTCUSD"
}
```

**Subscription Confirmation**:
```json
{
  "event": "subscribed",
  "channel": "ticker",
  "chanId": 1,
  "symbol": "tBTCUSD",
  "pair": "BTCUSD"
}
```

**Data Message** (Trading Pair):
```json
[
  1,                    // CHANNEL_ID
  [
    10645,              // BID
    73.93854271,        // BID_SIZE
    10647,              // ASK
    75.22266119,        // ASK_SIZE
    731.60645389,       // DAILY_CHANGE
    0.0738,             // DAILY_CHANGE_RELATIVE
    10644.00645389,     // LAST_PRICE
    14480.89849423,     // VOLUME
    10766,              // HIGH
    9889.1449809        // LOW
  ]
]
```

### Trades Channel

**Subscribe**:
```json
{
  "event": "subscribe",
  "channel": "trades",
  "symbol": "tBTCUSD"
}
```

**Subscription Confirmation**:
```json
{
  "event": "subscribed",
  "channel": "trades",
  "chanId": 2,
  "symbol": "tBTCUSD",
  "pair": "BTCUSD"
}
```

**Snapshot** (initial data):
```json
[
  2,                    // CHANNEL_ID
  [
    [388063448, 1567526214876, 1.918524, 10682],
    [388063447, 1567526214000, -0.5, 10683],
    ...
  ]
]
```

**Trade Update**:
```json
[
  2,                    // CHANNEL_ID
  "te",                 // TYPE: te=trade executed, tu=trade updated
  [
    388063448,          // ID
    1567526214876,      // MTS
    1.918524,           // AMOUNT
    10682               // PRICE
  ]
]
```

**Update Types**:
- `te` - Trade executed
- `tu` - Trade updated
- `fte` - Funding trade executed
- `ftu` - Funding trade updated

### Books Channel

**Subscribe**:
```json
{
  "event": "subscribe",
  "channel": "book",
  "symbol": "tBTCUSD",
  "prec": "P0",
  "freq": "F0",
  "len": "25",
  "subId": 123
}
```

**Parameters**:
- `prec`: P0, P1, P2, P3, P4 (precision level), or R0 (raw book)
- `freq`: F0 (realtime), F1 (2 seconds)
- `len`: 25 (default), 100
- `subId`: Optional subscription identifier

**Precision Levels**:
| Level | Significant Figures |
|-------|-------------------|
| P0 | 5 |
| P1 | 4 |
| P2 | 3 |
| P3 | 2 |
| P4 | 1 |
| R0 | Raw (no aggregation) |

**Subscription Confirmation**:
```json
{
  "event": "subscribed",
  "channel": "book",
  "chanId": 3,
  "symbol": "tBTCUSD",
  "prec": "P0",
  "freq": "F0",
  "len": "25",
  "pair": "BTCUSD"
}
```

**Snapshot** (initial orderbook):
```json
[
  3,                    // CHANNEL_ID
  [
    [8744.9, 2, 0.45603413],
    [8744.8, 1, 0.25],
    [8744.7, 3, -0.75],
    ...
  ]
]
```

**Update** (single level change):
```json
[
  3,                    // CHANNEL_ID
  [
    8744.9,             // PRICE
    2,                  // COUNT (0 = remove this level)
    0.45603413          // AMOUNT (positive=bid, negative=ask)
  ]
]
```

**Book Entry Interpretation**:
- `COUNT > 0`: Update or add this price level
- `COUNT = 0`: Remove this price level
- `AMOUNT > 0`: Bid side
- `AMOUNT < 0`: Ask side

### Candles Channel

**Subscribe**:
```json
{
  "event": "subscribe",
  "channel": "candles",
  "key": "trade:1m:tBTCUSD"
}
```

**Key Format**: `trade:{TIMEFRAME}:{SYMBOL}`

**Timeframes**:
- 1m, 5m, 15m, 30m
- 1h, 3h, 6h, 12h
- 1D, 1W, 14D, 1M

**Subscription Confirmation**:
```json
{
  "event": "subscribed",
  "channel": "candles",
  "chanId": 4,
  "key": "trade:1m:tBTCUSD"
}
```

**Snapshot** (initial candles):
```json
[
  4,                    // CHANNEL_ID
  [
    [1678465320000, 20097, 20114, 20125, 20094, 1.43504645],
    [1678465260000, 20100, 20097, 20105, 20090, 0.95234123],
    ...
  ]
]
```

**Update** (single candle):
```json
[
  4,                    // CHANNEL_ID
  [
    1678465320000,      // MTS
    20097,              // OPEN
    20094,              // CLOSE
    20097,              // HIGH
    20094,              // LOW
    0.07870586          // VOLUME
  ]
]
```

### Status Channel

**Subscribe**:
```json
{
  "event": "subscribe",
  "channel": "status",
  "key": "deriv:tBTCF0:USTF0"
}
```

Used for derivative status information.

## Authenticated Channels

### Authentication

**Send auth message after connection**:
```json
{
  "event": "auth",
  "apiKey": "YOUR_API_KEY",
  "authSig": "SIGNATURE",
  "authNonce": "NONCE",
  "authPayload": "AUTH_PAYLOAD",
  "dms": 4,
  "filter": ["trading"]
}
```

**Parameters**:
- `apiKey`: Your API key
- `authNonce`: Microsecond timestamp (must be increasing)
- `authPayload`: `AUTH{nonce}`
- `authSig`: HMAC-SHA384(authPayload, apiSecret) as hex
- `dms`: Dead-Man-Switch (4 = cancel all orders on disconnect)
- `filter`: Optional array to filter messages (["trading", "wallet", "balance"])

**Signature Generation**:
```python
import hmac
import hashlib
import time

nonce = str(int(time.time() * 1000000))  # microseconds
auth_payload = f"AUTH{nonce}"
signature = hmac.new(
    api_secret.encode('utf8'),
    auth_payload.encode('utf8'),
    hashlib.sha384
).hexdigest()
```

**Authentication Success**:
```json
{
  "event": "auth",
  "status": "OK",
  "chanId": 0,
  "userId": 123456,
  "auth_id": "auth-id",
  "caps": {
    "orders": { "read": 1, "write": 1 },
    "account": { "read": 1, "write": 0 },
    "funding": { "read": 1, "write": 1 },
    "history": { "read": 1, "write": 0 },
    "wallets": { "read": 1, "write": 0 },
    "withdraw": { "read": 0, "write": 0 },
    "positions": { "read": 1, "write": 1 }
  }
}
```

**Authentication Failure**:
```json
{
  "event": "auth",
  "status": "FAILED",
  "chanId": 0,
  "code": 10100,
  "msg": "apikey: invalid"
}
```

### Account Info Channel

Once authenticated, channel 0 is reserved for account updates.

**Message Types**:
- `os` - Order snapshot
- `on` - Order new
- `ou` - Order update
- `oc` - Order cancel
- `ps` - Position snapshot
- `pn` - Position new
- `pu` - Position update
- `pc` - Position close
- `ws` - Wallet snapshot
- `wu` - Wallet update
- `te` - Trade executed
- `tu` - Trade update
- `fos` - Funding offer snapshot
- `fon` - Funding offer new
- `fou` - Funding offer update
- `foc` - Funding offer cancel

**Order Snapshot**:
```json
[
  0,                    // CHANNEL_ID (always 0 for account)
  "os",                 // TYPE
  [
    [ORDER_DATA_1],     // 32 fields
    [ORDER_DATA_2],
    ...
  ]
]
```

**Order New**:
```json
[
  0,
  "on",
  [ORDER_DATA]          // 32 fields
]
```

**Order Update**:
```json
[
  0,
  "ou",
  [ORDER_DATA]
]
```

**Order Cancel**:
```json
[
  0,
  "oc",
  [ORDER_DATA]
]
```

**Trade Executed**:
```json
[
  0,
  "te",
  [
    ID,                 // Trade ID
    SYMBOL,
    MTS,
    ORDER_ID,
    EXEC_AMOUNT,
    EXEC_PRICE,
    ORDER_TYPE,
    ORDER_PRICE,
    MAKER,
    FEE,
    FEE_CURRENCY,
    CID
  ],
  "e"                   // Execution flag
]
```

**Wallet Snapshot**:
```json
[
  0,
  "ws",
  [
    [TYPE, CURRENCY, BALANCE, UNSETTLED_INTEREST, AVAILABLE_BALANCE],
    ...
  ]
]
```

**Wallet Update**:
```json
[
  0,
  "wu",
  [TYPE, CURRENCY, BALANCE, UNSETTLED_INTEREST, AVAILABLE_BALANCE]
]
```

**Position Snapshot**:
```json
[
  0,
  "ps",
  [
    [POSITION_DATA_1],  // 18 fields
    [POSITION_DATA_2],
    ...
  ]
]
```

## WebSocket Input Commands

### Submit Order
```json
[
  0,
  "on",
  null,
  {
    "type": "EXCHANGE LIMIT",
    "symbol": "tBTCUSD",
    "amount": "0.5",
    "price": "10000",
    "cid": 12345
  }
]
```

### Update Order
```json
[
  0,
  "ou",
  null,
  {
    "id": 123456789,
    "price": "10500"
  }
]
```

### Cancel Order
```json
[
  0,
  "oc",
  null,
  {
    "id": 123456789
  }
]
```

### Cancel Multiple Orders
```json
[
  0,
  "oc_multi",
  null,
  {
    "id": [123456789, 987654321]
  }
]
```

### Request Calculation
```json
[
  0,
  "calc",
  null,
  [
    ["margin_sym_tBTCUSD"],
    ["funding_sym_fUSD"]
  ]
]
```

## Error Handling

### Error Message Format
```json
[
  CHANNEL_ID,
  "error",
  ERROR_CODE,
  "error message"
]
```

### Notification Messages

Notifications about order operations:

```json
[
  0,
  "n",
  [
    MTS,
    TYPE,               // "on-req", "ou-req", "oc-req"
    MESSAGE_ID,
    null,
    NOTIFY_INFO,        // Order or other data
    CODE,
    STATUS,             // "SUCCESS", "ERROR", "FAILURE"
    TEXT
  ]
]
```

**Notification Types**:
- `on-req` - Order new request
- `ou-req` - Order update request
- `oc-req` - Order cancel request
- `oc_multi-req` - Multiple order cancel request
- `ERROR` - General error
- `SUCCESS` - Operation successful

## Maintenance Events

### Maintenance Start
```json
{
  "event": "info",
  "code": 20060,
  "msg": "Entering maintenance mode. Please pause trading and cancel all orders."
}
```

### Maintenance End
```json
{
  "event": "info",
  "code": 20061,
  "msg": "Maintenance ended. You can resume trading."
}
```

## Connection Management

### Reconnection Strategy

```rust
struct WebSocketClient {
    url: String,
    max_reconnect_delay: Duration,
}

impl WebSocketClient {
    async fn connect_with_retry(&self) -> Result<WebSocket> {
        let mut delay = Duration::from_secs(1);

        loop {
            match self.connect().await {
                Ok(ws) => return Ok(ws),
                Err(e) => {
                    warn!("Connection failed: {}, retrying in {:?}", e, delay);
                    tokio::time::sleep(delay).await;
                    delay = std::cmp::min(delay * 2, self.max_reconnect_delay);
                }
            }
        }
    }
}
```

### Heartbeat Monitoring

```rust
struct HeartbeatMonitor {
    last_heartbeat: Instant,
    timeout: Duration,
}

impl HeartbeatMonitor {
    fn on_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    fn is_alive(&self) -> bool {
        Instant::now().duration_since(self.last_heartbeat) < self.timeout
    }
}
```

## Best Practices

1. **Handle Heartbeats**: Respond to `hb` messages to detect connection issues
2. **Implement Reconnection**: Automatic reconnection with exponential backoff
3. **Subscribe Gradually**: Don't subscribe to all channels at once
4. **Use Sequence Numbers**: Enable `SEQ_ALL` flag for message ordering
5. **Enable Checksums**: For order books, enable checksums to verify data integrity
6. **Monitor Notifications**: Check notification messages for operation status
7. **Handle Maintenance**: Detect maintenance events and pause operations
8. **Separate Connections**: Use different connections for different purposes
9. **Dead-Man-Switch**: Use `dms=4` to auto-cancel orders on disconnect
10. **Filter Messages**: Use `filter` to reduce unnecessary data

## Example: Complete WebSocket Flow

```python
import asyncio
import websockets
import json
import hmac
import hashlib
import time

async def bitfinex_ws():
    url = "wss://api-pub.bitfinex.com/ws/2"

    async with websockets.connect(url) as ws:
        # 1. Receive info message
        msg = await ws.recv()
        print(f"Connected: {msg}")

        # 2. Subscribe to ticker
        subscribe_msg = {
            "event": "subscribe",
            "channel": "ticker",
            "symbol": "tBTCUSD"
        }
        await ws.send(json.dumps(subscribe_msg))

        # 3. Receive subscription confirmation
        msg = await ws.recv()
        print(f"Subscribed: {msg}")

        # 4. Receive data
        while True:
            msg = await ws.recv()
            data = json.loads(msg)

            if isinstance(data, list):
                channel_id = data[0]

                if len(data) > 1 and data[1] == "hb":
                    print(f"Heartbeat on channel {channel_id}")
                else:
                    print(f"Data: {data}")
            else:
                print(f"Event: {data}")

asyncio.run(bitfinex_ws())
```

## WebSocket vs REST Comparison

| Feature | WebSocket | REST |
|---------|-----------|------|
| Connection | Persistent | Per request |
| Latency | Low | Higher |
| Updates | Push (real-time) | Pull (on request) |
| Rate Limit | Connection-based | Request-based |
| Use Case | Live data streams | One-time queries |
| Complexity | Higher | Lower |
| Bandwidth | Lower (after initial) | Higher |
