# Alpaca - WebSocket Documentation

## Availability: Yes

Alpaca provides TWO separate WebSocket systems:
1. **Market Data Streams** - Real-time market data (stocks, options, crypto, news)
2. **Trading Updates Stream** - Trade/order/account updates

---

## MARKET DATA WEBSOCKET

### URLs

**Production:**
```
wss://stream.data.alpaca.markets/v2/{feed}
```

Where `{feed}` is:
- `iex` - IEX exchange only (FREE tier)
- `sip` - All US exchanges (PAID tier)

**Sandbox:**
```
wss://stream.data.sandbox.alpaca.markets/v2/{feed}
```

**Test Stream (24/7 available):**
```
wss://stream.data.alpaca.markets/v2/test
```
Use symbol "FAKEPACA" for testing

**Crypto Stream:**
```
wss://stream.data.alpaca.markets/v1beta3/crypto/us
```

### Connection Process

1. **Connect** to WebSocket URL
2. **Server sends welcome** (no explicit welcome message documented)
3. **Authenticate** within 10 seconds (see Authentication section)
4. **Server sends auth response** (authorized/unauthorized)
5. **Subscribe** to channels
6. **Server sends subscription confirmation**
7. **Receive data streams**

## ALL Available Channels/Topics

### Stock Market Data Channels

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Example Subscription |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| trades | Public | Individual trade executions | Yes | Yes (IEX) | Real-time | {"action":"subscribe","trades":["AAPL"]} |
| quotes | Public | Bid/ask updates | Yes | Yes (IEX) | Real-time | {"action":"subscribe","quotes":["AAPL"]} |
| bars | Public | 1-minute OHLCV bars | Yes | Yes (IEX) | Every minute | {"action":"subscribe","bars":["AAPL"]} |
| dailyBars | Public | Daily OHLCV aggregates | Yes | Yes (IEX) | Intraday updates | {"action":"subscribe","dailyBars":["AAPL"]} |
| updatedBars | Public | Corrected bars from late trades | Yes | Yes (IEX) | Real-time | {"action":"subscribe","updatedBars":["AAPL"]} |
| statuses | Public | Trading halts/resumptions | Yes | Yes (IEX) | Real-time | {"action":"subscribe","statuses":["AAPL"]} |
| lulds | Public | Limit Up-Limit Down bands | Yes | Yes (IEX) | Real-time | {"action":"subscribe","lulds":["AAPL"]} |
| corrections | Auto | Trade corrections | Yes | Yes (IEX) | Real-time | Auto-subscribed with trades |
| cancelErrors | Auto | Trade cancellations | Yes | Yes (IEX) | Real-time | Auto-subscribed with trades |
| imbalances | Public | Order imbalances | Yes | Yes (IEX) | Real-time | {"action":"subscribe","imbalances":["AAPL"]} |

**Note:** `corrections` and `cancelErrors` are automatically subscribed when subscribing to `trades`

### Options Data Channels

Available via v1beta1 WebSocket (not extensively documented in provided sources)

### Crypto Data Channels

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Example Subscription |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| trades | Public | Crypto trade executions | Yes | Yes | Real-time | subscribe_crypto_trades |
| quotes | Public | Crypto bid/ask | Yes | Yes | Real-time | subscribe_crypto_quotes |
| bars | Public | Crypto minute bars | Yes | Yes | Every minute | subscribe_crypto_bars |
| dailyBars | Public | Crypto daily bars | Yes | Yes | Daily | subscribe_crypto_dailyBars |
| orderbooks | Public | Order book depth | Yes | Yes | Real-time | subscribe_crypto_orderbooks |

### News Channels

Real-time news streams available (specific channel documentation not provided in sources)

## Subscription Format

### Subscribe Message (Stock Data)

**Subscribe to specific symbols:**
```json
{
  "action": "subscribe",
  "trades": ["AAPL", "TSLA"],
  "quotes": ["AMD", "NVDA"],
  "bars": ["MSFT"]
}
```

**Subscribe to all symbols (wildcard):**
```json
{
  "action": "subscribe",
  "trades": ["*"],
  "quotes": ["*"]
}
```

**Multiple channels at once:**
```json
{
  "action": "subscribe",
  "trades": ["AAPL"],
  "quotes": ["AAPL"],
  "bars": ["AAPL"],
  "dailyBars": ["AAPL"],
  "statuses": ["*"]
}
```

### Unsubscribe Message

**Unsubscribe from specific symbols:**
```json
{
  "action": "unsubscribe",
  "trades": ["AAPL"],
  "quotes": ["AMD"]
}
```

**Unsubscribe from all:**
```json
{
  "action": "unsubscribe",
  "trades": ["*"],
  "quotes": ["*"],
  "bars": ["*"]
}
```

### Subscription Confirmation

**Success response:**
```json
[{
  "T": "subscription",
  "trades": ["AAPL", "TSLA"],
  "quotes": ["AMD", "NVDA"],
  "bars": ["MSFT"],
  "dailyBars": [],
  "updatedBars": [],
  "statuses": [],
  "lulds": [],
  "corrections": ["AAPL", "TSLA"],
  "cancelErrors": ["AAPL", "TSLA"]
}]
```

**Note:** `corrections` and `cancelErrors` are auto-added when subscribing to `trades`

## Message Formats (for EVERY channel)

All messages arrive as **JSON arrays**: `[{...}, {...}, ...]`

Control messages (subscription, error, success) are single-item arrays.
Data messages may be batched in larger arrays for efficiency.

### Trade Update

```json
[{
  "T": "t",
  "S": "AAPL",
  "i": 52983525029461,
  "x": "D",
  "p": 150.25,
  "s": 100,
  "t": "2024-01-18T15:30:45.123456789Z",
  "c": ["@", "I"],
  "z": "C"
}]
```

**Fields:**
- `T`: Message type ("t" = trade)
- `S`: Symbol
- `i`: Trade ID (unique)
- `x`: Exchange code (D=EDGX, V=IEX, etc.)
- `p`: Price
- `s`: Size (shares)
- `t`: Timestamp (RFC-3339, nanosecond precision)
- `c`: Condition codes (array of strings)
- `z`: Tape (A/B/C for CTA/UTP)

### Quote Update

```json
[{
  "T": "q",
  "S": "AAPL",
  "bx": "U",
  "bp": 150.24,
  "bs": 100,
  "ax": "Q",
  "ap": 150.26,
  "as": 200,
  "t": "2024-01-18T15:30:45.123456789Z",
  "c": ["R"],
  "z": "C"
}]
```

**Fields:**
- `T`: Message type ("q" = quote)
- `S`: Symbol
- `bx`: Bid exchange
- `bp`: Bid price
- `bs`: Bid size
- `ax`: Ask exchange
- `ap`: Ask price
- `as`: Ask size
- `t`: Timestamp (RFC-3339, nanosecond precision)
- `c`: Condition codes
- `z`: Tape

### Bar Update (Minute)

```json
[{
  "T": "b",
  "S": "AAPL",
  "o": 150.00,
  "h": 150.50,
  "l": 149.80,
  "c": 150.25,
  "v": 125000,
  "vw": 150.12,
  "n": 1500,
  "t": "2024-01-18T15:30:00Z"
}]
```

**Fields:**
- `T`: Message type ("b" = bar)
- `S`: Symbol
- `o`: Open price
- `h`: High price
- `l`: Low price
- `c`: Close price
- `v`: Volume
- `vw`: Volume-weighted average price
- `n`: Number of trades
- `t`: Bar timestamp (start of minute)

### Daily Bar Update

```json
[{
  "T": "d",
  "S": "AAPL",
  "o": 148.50,
  "h": 151.00,
  "l": 147.80,
  "c": 150.25,
  "v": 50000000,
  "vw": 149.85,
  "n": 250000,
  "t": "2024-01-18"
}]
```

**Fields:** Same as bar, but `T`: "d" for daily, `t` is date only

### Updated Bar

```json
[{
  "T": "u",
  "S": "AAPL",
  "o": 150.00,
  "h": 150.55,
  "l": 149.80,
  "c": 150.30,
  "v": 126000,
  "vw": 150.13,
  "n": 1505,
  "t": "2024-01-18T15:30:00Z"
}]
```

**Fields:** Same as bar, but `T`: "u" for updated (corrected due to late-arriving trades)

### Trading Status

```json
[{
  "T": "s",
  "S": "AAPL",
  "sc": "H",
  "sm": "Halted",
  "rc": "T1",
  "rm": "Trading halt",
  "t": "2024-01-18T15:30:00Z",
  "z": "C"
}]
```

**Fields:**
- `T`: Message type ("s" = status)
- `S`: Symbol
- `sc`: Status code (H=Halted, R=Resumed, etc.)
- `sm`: Status message (human-readable)
- `rc`: Reason code
- `rm`: Reason message
- `t`: Timestamp
- `z`: Tape

### LULD (Limit Up-Limit Down)

```json
[{
  "T": "l",
  "S": "AAPL",
  "lu": 155.00,
  "ld": 145.00,
  "i": "B",
  "t": "2024-01-18T15:30:00Z",
  "z": "C"
}]
```

**Fields:**
- `T`: Message type ("l" = LULD)
- `S`: Symbol
- `lu`: Limit up price
- `ld`: Limit down price
- `i`: Indicator (B=Straddle, etc.)
- `t`: Timestamp
- `z`: Tape

### Trade Correction

```json
[{
  "T": "c",
  "S": "AAPL",
  "x": "V",
  "oi": 52983525029461,
  "op": 150.25,
  "os": 100,
  "oc": ["@"],
  "ci": 52983525029462,
  "cp": 150.30,
  "cs": 100,
  "cc": ["@"],
  "t": "2024-01-18T15:30:45Z",
  "z": "C"
}]
```

**Fields:**
- `T`: Message type ("c" = correction)
- `S`: Symbol
- `x`: Exchange
- `oi`: Original trade ID
- `op`: Original price
- `os`: Original size
- `oc`: Original conditions
- `ci`: Corrected trade ID
- `cp`: Corrected price
- `cs`: Corrected size
- `cc`: Corrected conditions
- `t`: Timestamp
- `z`: Tape

### Trade Cancel/Error

```json
[{
  "T": "x",
  "S": "AAPL",
  "i": 52983525029461,
  "x": "V",
  "p": 150.25,
  "s": 100,
  "a": "D",
  "t": "2024-01-18T15:30:45Z",
  "z": "C"
}]
```

**Fields:**
- `T`: Message type ("x" = cancel)
- `S`: Symbol
- `i`: Trade ID (being canceled)
- `x`: Exchange
- `p`: Price
- `s`: Size
- `a`: Action (D=Delete)
- `t`: Timestamp
- `z`: Tape

### Order Imbalance

```json
[{
  "T": "imbalance",
  "S": "AAPL",
  "side": "buy",
  "quantity": 50000,
  "price": 150.25,
  "timestamp": "2024-01-18T15:30:00Z"
}]
```

**Fields:**
- `T`: Message type ("imbalance")
- `S`: Symbol
- `side`: Buy or sell imbalance
- `quantity`: Imbalance size
- `price`: Reference price
- `timestamp`: Event time

### Crypto Orderbook

```json
[{
  "T": "o",
  "S": "BTC/USD",
  "b": [
    [45000.00, 1.5],
    [44995.00, 2.0]
  ],
  "a": [
    [45005.00, 1.2],
    [45010.00, 1.8]
  ],
  "t": "2024-01-18T15:30:45Z"
}]
```

**Fields:**
- `T`: Message type ("o" = orderbook)
- `S`: Symbol
- `b`: Bids array [[price, size], ...]
- `a`: Asks array [[price, size], ...]
- `t`: Timestamp

### Error Message

```json
[{
  "T": "error",
  "code": 401,
  "msg": "not authenticated"
}]
```

**Error codes:**
- 401: Not authenticated
- 402: Authentication failed
- 405: Symbol limit exceeded
- 406: Connection limit exceeded
- 409: Insufficient subscription

### Success Message

```json
[{
  "T": "success",
  "msg": "authenticated"
}]
```

## Heartbeat / Ping-Pong

**CRITICAL:** Ping/pong mechanism NOT explicitly documented in provided sources.

### Who initiates?
- Server → Client ping: **Not documented**
- Client → Server ping: **Recommended but not required**

### Message Format
- Binary ping/pong frames: **Likely (standard WebSocket frames per RFC6455)**
- Text messages: **Not documented**
- JSON messages: **Not documented**

### Timing
- Ping interval: **Not specified**
- Timeout: **Not specified**
- Client must send ping: **Not required, but recommended for connection health**

### Example
**Not provided in documentation** - likely uses standard WebSocket binary ping/pong frames

**Note:** Alpaca SDKs handle this automatically. Manual implementations should send periodic pings to keep connection alive.

## Connection Limits

### Per Subscription Tier

**Free Tier (IEX feed):**
- Max connections per IP: **Likely 1** (most subscriptions allow only 1)
- Max connections per API key: **1**
- Max subscriptions per connection: **30 symbols**
- Message rate limit: **Not specified**
- Auto-disconnect after: **Not specified**

**Algo Trader Plus (SIP feed):**
- Max connections per IP: **Likely 1**
- Max connections per API key: **1** (error 406 if exceeded)
- Max subscriptions per connection: **Unlimited symbols**
- Message rate limit: **Not specified**
- Auto-disconnect after: **Not specified**

### Error When Limit Exceeded
**Error code 406:** Connection limit exceeded (trying to open multiple connections)
**Error code 405:** Symbol limit exceeded (free tier > 30 symbols)

## Authentication (for all channels)

Market data streams require authentication even for public channels.

### Method 1: HTTP Headers (Trading API keys)

Connect with headers:
```
APCA-API-KEY-ID: your_key_id
APCA-API-SECRET-KEY: your_secret_key
```

### Method 2: Basic Authentication (Broker API)

Authorization header with base64-encoded credentials:
```
Authorization: Basic base64encode(key:secret)
```

### Method 3: Message-based Auth (Most Common)

**After connecting, send within 10 seconds:**
```json
{
  "action": "auth",
  "key": "your_key_id",
  "secret": "your_secret_key"
}
```

### Method 4: OAuth Token

```json
{
  "action": "auth",
  "key": "oauth",
  "secret": "your_oauth_token"
}
```

### Auth Success

```json
[{
  "T": "success",
  "msg": "authenticated"
}]
```

### Auth Failure

```json
[{
  "T": "error",
  "code": 402,
  "msg": "authentication failed"
}]
```

**Timeout:** Must authenticate within **10 seconds** after connecting or connection will be closed.

---

## TRADING UPDATES WEBSOCKET

Separate WebSocket system for real-time trade/order/account updates.

### URLs

**Live Trading:**
```
wss://api.alpaca.markets/stream
```

**Paper Trading:**
```
wss://paper-api.alpaca.markets/stream
```

### Connection Process

1. **Connect** to WebSocket URL
2. **Authenticate** with credentials
3. **Subscribe** to streams (trade_updates, etc.)
4. **Server confirms** subscription
5. **Receive updates**

### Available Event Streams

| Stream | Type | Description | Auth? | Free? | Update Frequency | Example Subscription |
|--------|------|-------------|-------|-------|------------------|---------------------|
| trade_updates | Private | Order/trade events | Yes | Yes | Real-time | {"action":"listen","data":{"streams":["trade_updates"]}} |

**Trade updates include:**
- Order placements and routing
- Order fills (complete and partial)
- Order cancellations and rejections
- Order expirations and replacements
- Account and position changes

### Authentication

**Send after connecting:**
```json
{
  "action": "auth",
  "key": "your_api_key_id",
  "secret": "your_api_secret_key"
}
```

**Auth success:**
```json
{
  "stream": "authorization",
  "data": {
    "status": "authorized",
    "action": "authenticate"
  }
}
```

**Auth failure:**
```json
{
  "stream": "authorization",
  "data": {
    "status": "unauthorized",
    "action": "authenticate"
  }
}
```

### Subscribe to Trade Updates

```json
{
  "action": "listen",
  "data": {
    "streams": ["trade_updates"]
  }
}
```

**Subscription confirmation:**
```json
{
  "stream": "listening",
  "data": {
    "streams": ["trade_updates"]
  }
}
```

**Unsubscribe (empty streams array):**
```json
{
  "action": "listen",
  "data": {
    "streams": []
  }
}
```

### Trade Update Event Types

**Common events:**
- `new` - Order accepted
- `fill` - Order completely filled
- `partial_fill` - Order partially filled
- `canceled` - Order canceled
- `expired` - Order expired
- `done_for_day` - Order done for day
- `replaced` - Order replaced/modified

**Uncommon events:**
- `accepted` - Order accepted by broker
- `rejected` - Order rejected
- `pending_new` - Order pending acceptance
- `stopped` - Order stopped
- `pending_cancel` - Cancel pending
- `pending_replace` - Replace pending
- `calculated` - Order calculated
- `suspended` - Order suspended
- `order_replace_rejected` - Replace rejected
- `order_cancel_rejected` - Cancel rejected

### Trade Update Message Format

```json
{
  "stream": "trade_updates",
  "data": {
    "event": "fill",
    "order": {
      "id": "61e69015-8549-4bfd-b9c3-01e75843f47d",
      "client_order_id": "my_order_123",
      "created_at": "2024-01-18T15:30:00Z",
      "updated_at": "2024-01-18T15:30:05Z",
      "submitted_at": "2024-01-18T15:30:00Z",
      "filled_at": "2024-01-18T15:30:05Z",
      "expired_at": null,
      "canceled_at": null,
      "failed_at": null,
      "replaced_at": null,
      "asset_id": "904837e3-3b76-47ec-b432-046db621571b",
      "symbol": "AAPL",
      "asset_class": "us_equity",
      "qty": "100",
      "filled_qty": "100",
      "type": "market",
      "side": "buy",
      "time_in_force": "day",
      "limit_price": null,
      "stop_price": null,
      "filled_avg_price": "150.25",
      "status": "filled",
      "extended_hours": false,
      "legs": null,
      "trail_price": null,
      "trail_percent": null,
      "hwm": null
    },
    "timestamp": "2024-01-18T15:30:05.123456Z",
    "position_qty": "100",
    "price": "150.25",
    "qty": "100",
    "execution_id": "74e21155-d465-4e67-9b5e-b5a5c2c4b5c6"
  }
}
```

**Key fields:**
- `event`: Event type (fill, partial_fill, canceled, etc.)
- `order`: Complete order object (matches REST API format)
- `timestamp`: Event timestamp
- `position_qty`: New position quantity after event
- `price`: Execution price (for fills)
- `qty`: Quantity filled (for fills)
- `execution_id`: Unique execution ID

## Content Types

Both WebSocket systems support:
- **Content-Type: application/json** (default)
- **Content-Type: application/msgpack** (binary format for efficiency)

**Compression:** Messages use compression per RFC-7692. Official SDKs handle this automatically.

## Connection Best Practices

1. **Authenticate quickly** - Within 10 seconds for market data streams
2. **Handle reconnections** - Implement exponential backoff
3. **Subscribe in batches** - Don't exceed symbol limits
4. **Monitor for errors** - Check for error messages
5. **Use wildcard carefully** - `"*"` subscribes to ALL symbols (can be overwhelming)
6. **Process arrays** - Messages arrive as arrays, iterate to process all items
7. **SDK recommended** - Official SDKs handle auth, compression, reconnection automatically
