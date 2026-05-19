# Fyers - WebSocket Documentation

## Availability: Yes

Fyers API V3 provides three types of WebSocket connections for real-time data streaming.

---

## Connection URLs

### 1. Data WebSocket (Market Data)
- URL: `wss://api-t1.fyers.in/socket/v3/dataSock`
- Purpose: Real-time market data (quotes, depth, trades)
- Authentication: Required (access token)
- Public/Private: Private

### 2. Order WebSocket (Order Updates)
- URL: `wss://api-t1.fyers.in/socket/v3/orderSock`
- Purpose: Real-time order, trade, and position updates
- Authentication: Required (access token)
- Public/Private: Private

### 3. TBT WebSocket (Tick-by-Tick)
- URL: `wss://rtsocket-api.fyers.in/versova`
- Purpose: Tick-by-tick market data with depth
- Authentication: Required (access token)
- Public/Private: Private
- Protocol: Binary (Protobuf)

---

## Connection Process

### General Flow
1. Obtain access token via OAuth flow (REST API)
2. Connect to WebSocket URL
3. Authenticate with access token
4. Subscribe to channels/symbols
5. Receive real-time updates

### Authentication
Authentication is performed during connection establishment by including the access token in the connection request or initial message.

**Access Token Format:**
```
APPID:ACCESS_TOKEN
```

---

## Data WebSocket - Available Channels

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Subscription Format |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| Symbol Update | Private | Real-time price updates | Yes | Yes | Real-time | Subscribe with symbol list |
| Depth Update | Private | Market depth (L2) changes | Yes | Yes | Real-time | Subscribe with symbol list |
| Lite Mode | Private | LTP-only updates | Yes | Yes | Real-time | Subscribe with lite flag |

### Symbol Update (Full Mode)
Provides comprehensive market data updates for subscribed symbols.

**Fields in Update:**
- `symbol` - Trading symbol
- `timestamp` - Update timestamp
- `fytoken` - Fyers token
- `ltp` - Last traded price
- `open_price`, `high_price`, `low_price`, `close_price`
- `prev_close_price` - Previous close
- `volume` - Total volume
- `chp` - Change in points
- `ch` - Change percentage
- `bid_price`, `ask_price` - Best bid/ask
- `bid_size`, `ask_size` - Bid/ask quantities
- `last_traded_qty` - Last trade quantity
- `last_traded_time` - Last trade time
- `avg_trade_price` - VWAP
- `tot_buy_qty`, `tot_sell_qty` - Total quantities

### Depth Update
Provides market depth (order book) updates.

**Fields in Update:**
- `symbol` - Trading symbol
- `timestamp` - Update timestamp
- `bids` - Array of [price, volume, orders] (top 5)
- `asks` - Array of [price, volume, orders] (top 5)
- `totalbuyqty` - Total buy quantity
- `totalsellqty` - Total sell quantity

### Lite Mode
Provides only LTP (Last Traded Price) updates for minimal bandwidth.

**Fields in Update:**
- `symbol` - Trading symbol
- `ltp` - Last traded price
- `timestamp` - Update timestamp

---

## Order WebSocket - Available Channels

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Subscription Format |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| Order Updates | Private | Order status changes | Yes | Yes | Real-time | Auto-subscribed on connect |
| Trade Updates | Private | Trade execution events | Yes | Yes | Real-time | Auto-subscribed on connect |
| Position Updates | Private | Position changes | Yes | Yes | Real-time | Auto-subscribed on connect |
| General Updates | Private | Account events | Yes | Yes | Real-time | Auto-subscribed on connect |

### Order Updates
Real-time notifications when order status changes.

**Fields in Update:**
- `id` - Order ID
- `symbol` - Trading symbol
- `type` - Order type (LIMIT, MARKET, etc.)
- `side` - BUY or SELL
- `status` - Order status
- `qty` - Total quantity
- `filledQty` - Filled quantity
- `remainingQty` - Remaining quantity
- `limitPrice`, `stopPrice` - Prices
- `productType` - INTRADAY, CNC, etc.
- `orderDateTime` - Order placement time
- `orderUpdateTime` - Last update time
- `message` - Status message

**Status Values:**
- `1` - Cancelled
- `2` - Traded/Filled
- `4` - Transit
- `5` - Rejected
- `6` - Pending
- `7` - Expired

### Trade Updates
Real-time notifications when orders are executed.

**Fields in Update:**
- `tradeId` - Trade ID
- `orderId` - Parent order ID
- `symbol` - Trading symbol
- `side` - BUY or SELL
- `qty` - Traded quantity
- `tradePrice` - Execution price
- `tradeTime` - Execution timestamp
- `productType` - Product type
- `exchange` - Exchange name
- `segment` - Market segment

### Position Updates
Real-time notifications when positions change.

**Fields in Update:**
- `symbol` - Trading symbol
- `side` - Position side (LONG/SHORT)
- `netQty` - Net quantity
- `avgPrice` - Average price
- `buyQty`, `sellQty` - Buy/sell quantities
- `buyAvg`, `sellAvg` - Average buy/sell prices
- `realizedProfit` - Realized P&L
- `unrealizedProfit` - Unrealized P&L
- `productType` - Product type

### General Updates
Account-level events and notifications.

**Fields vary by event type.**

---

## TBT WebSocket (Tick-by-Tick)

### Connection
- URL: `wss://rtsocket-api.fyers.in/versova`
- Protocol: Binary (Protobuf)
- Encoding: Protobuf message format

### Subscription Modes

| Mode | Description | Data Provided |
|------|-------------|---------------|
| Depth | Market depth with tick data | Full order book + tick-by-tick trades |

### Data Structure
TBT WebSocket uses Protobuf encoding for efficient binary transmission.

**Message Types:**
- Snapshot - Initial full data
- Diff/Update - Incremental changes

**SocketMessage Structure:**
- `message_type` - Type of message (snapshot/diff)
- `symbol_data` - Map of symbols to market data
- `is_snapshot` - Boolean flag
- `is_diff` - Boolean flag

When you first subscribe, you receive a snapshot (complete view). After that, only differential updates (diffs) are sent for efficiency.

---

## Subscription Format

### Data WebSocket - Subscribe Message

**Full Mode (Symbol Update + Depth):**
```json
{
  "action": 1,
  "data": {
    "symbols": ["NSE:SBIN-EQ", "NSE:RELIANCE-EQ"],
    "mode": "full"
  }
}
```

**Lite Mode (LTP Only):**
```json
{
  "action": 1,
  "data": {
    "symbols": ["NSE:SBIN-EQ", "NSE:RELIANCE-EQ"],
    "mode": "lite"
  }
}
```

**Depth Only:**
```json
{
  "action": 1,
  "data": {
    "symbols": ["NSE:SBIN-EQ"],
    "mode": "depth"
  }
}
```

### Data WebSocket - Unsubscribe Message

```json
{
  "action": 0,
  "data": {
    "symbols": ["NSE:SBIN-EQ"]
  }
}
```

### Order WebSocket - Subscribe

Order WebSocket auto-subscribes on connection. No explicit subscription needed.

```json
{
  "action": "subscribe"
}
```

### Order WebSocket - Unsubscribe

```json
{
  "action": "unsubscribe"
}
```

---

## Subscription Confirmation

### Success Response
```json
{
  "s": "ok",
  "code": 200,
  "message": "Subscribed successfully"
}
```

### Error Response
```json
{
  "s": "error",
  "code": -351,
  "message": "You have provided symbols greater than 50"
}
```

---

## Message Formats

### Data WebSocket - Symbol Update (Full Mode)

```json
{
  "type": "sf",
  "symbol": "NSE:SBIN-EQ",
  "timestamp": 1640000000000,
  "fytoken": "1234",
  "ltp": 550.50,
  "open_price": 548.00,
  "high_price": 552.00,
  "low_price": 547.50,
  "close_price": 549.00,
  "prev_close_price": 549.00,
  "volume": 1234567,
  "chp": 1.50,
  "ch": 0.27,
  "bid_price": 550.45,
  "ask_price": 550.55,
  "bid_size": 100,
  "ask_size": 150,
  "last_traded_qty": 50,
  "last_traded_time": 1640000000,
  "avg_trade_price": 550.25,
  "tot_buy_qty": 50000,
  "tot_sell_qty": 48000
}
```

### Data WebSocket - Lite Mode Update

```json
{
  "type": "ltp",
  "symbol": "NSE:SBIN-EQ",
  "ltp": 550.50,
  "timestamp": 1640000000000
}
```

### Data WebSocket - Depth Update

```json
{
  "type": "dp",
  "symbol": "NSE:SBIN-EQ",
  "timestamp": 1640000000000,
  "bids": [
    [550.45, 100, 5],
    [550.40, 200, 8],
    [550.35, 150, 6],
    [550.30, 300, 12],
    [550.25, 250, 10]
  ],
  "asks": [
    [550.55, 150, 7],
    [550.60, 180, 9],
    [550.65, 220, 11],
    [550.70, 200, 8],
    [550.75, 300, 15]
  ],
  "totalbuyqty": 45000,
  "totalsellqty": 43000
}
```

### Order WebSocket - Order Update

```json
{
  "type": "order",
  "id": "ORD123456789",
  "symbol": "NSE:SBIN-EQ",
  "type": 1,
  "side": 1,
  "status": 6,
  "qty": 100,
  "filledQty": 0,
  "remainingQty": 100,
  "limitPrice": 550.00,
  "stopPrice": 0,
  "productType": "INTRADAY",
  "orderDateTime": "2026-01-26 09:15:30",
  "orderUpdateTime": "2026-01-26 09:15:30",
  "message": "Order placed successfully"
}
```

### Order WebSocket - Trade Update

```json
{
  "type": "trade",
  "tradeId": "TRD987654321",
  "orderId": "ORD123456789",
  "symbol": "NSE:SBIN-EQ",
  "side": 1,
  "qty": 100,
  "tradePrice": 550.50,
  "tradeTime": 1640000000000,
  "productType": "INTRADAY",
  "exchange": "NSE",
  "segment": "CM"
}
```

### Order WebSocket - Position Update

```json
{
  "type": "position",
  "symbol": "NSE:SBIN-EQ",
  "side": "LONG",
  "netQty": 100,
  "avgPrice": 550.50,
  "buyQty": 100,
  "sellQty": 0,
  "buyAvg": 550.50,
  "sellAvg": 0,
  "realizedProfit": 0,
  "unrealizedProfit": 50.00,
  "productType": "INTRADAY"
}
```

---

## Heartbeat / Ping-Pong

### Who Initiates?
- **Server → Client ping:** Yes (periodic)
- **Client → Server ping:** Recommended (optional)

### Message Format
- **Binary ping/pong frames:** Yes (standard WebSocket ping/pong)
- **Text messages:** Not specified
- **JSON messages:** Not specified

### Timing
- **Ping interval:** Not officially documented
- **Timeout:** Connection closed if no response
- **Client must send ping:** Optional (for keep-alive)

### Recommended Practice
- Use standard WebSocket ping/pong frames
- Client should respond to server pings
- Client can send periodic pings to detect connection issues
- SDKs handle ping/pong automatically

### Example (Binary Frames)
```
Server → Client: PING frame
Client → Server: PONG frame
```

**Note:** Fyers WebSocket uses standard WebSocket ping/pong control frames. Most WebSocket libraries handle this automatically.

---

## Connection Limits

### Maximum Connections
- **Max connections per IP:** Not officially documented
- **Max connections per API key:** Not officially documented
- **Max connections total:** Not officially documented
- **Recommended:** 1-2 connections per account (Data + Order)

### Subscription Limits

**Data WebSocket:**
- **V3 (Latest):** Up to 5,000 symbols per connection (with latest SDK)
- **Practical Limit:** 200 symbols per connection (widely reported)
- **Legacy Limit:** 50 symbols per connection (older API versions)
- **Conflicting Information:** Documentation shows varying limits

**Note:** The 5,000 symbol limit was announced in V3 updates, but practical testing shows errors above 50 symbols in some implementations. Recommend testing with your specific SDK version.

**Order WebSocket:**
- No symbol subscription (auto-receives all account updates)

**TBT WebSocket:**
- Channel-based subscriptions
- Limits not officially documented

### Message Rate Limits
- **Messages per second:** Not officially documented
- **Server may throttle:** Yes (if excessive)
- **Auto-disconnect on violation:** Possible

### Connection Duration
- **Max lifetime:** Unlimited (persistent connection)
- **Auto-reconnect needed:** Yes (on disconnect)
- **Idle timeout:** None (as long as ping/pong active)

---

## Authentication

### Method
- **Access Token:** Required in connection/initial message
- **Token Format:** `APPID:ACCESS_TOKEN`

### Authentication Flow

**1. Data WebSocket:**
```javascript
const ws = new WebSocket('wss://api-t1.fyers.in/socket/v3/dataSock');

// Send access token after connection
ws.on('open', () => {
  ws.send(JSON.stringify({
    "T": "SUB_L2",
    "SLIST": ["NSE:SBIN-EQ"],
    "SUB_T": 1
  }));
});
```

**2. Order WebSocket:**
Similar authentication with access token on connect.

**3. TBT WebSocket:**
Access token passed during connection establishment.

### Authentication Success
Connection stays open and ready for subscriptions.

### Authentication Failure
Connection closed with error message.

---

## Error Handling

### Common Errors

| Code | Message | Cause | Resolution |
|------|---------|-------|------------|
| -351 | Symbol limit exceeded | >50 symbols in subscription | Reduce symbol count or use multiple connections |
| -100 | Invalid symbol | Incorrect symbol format | Check symbol format |
| 401 | Unauthorized | Invalid/expired token | Re-authenticate |
| - | Connection closed | Network/server issue | Implement auto-reconnect |

### Auto-Reconnection

**Recommended Strategy:**
1. Detect connection close
2. Wait exponential backoff (1s, 2s, 4s, 8s, max 60s)
3. Re-establish connection
4. Re-authenticate
5. Re-subscribe to symbols

**SDK Support:**
Most official SDKs support auto-reconnection:
```python
# Python SDK
data_ws = FyersDataSocket(
    access_token=access_token,
    log_path="",
    write_to_file=False,
    reconnect=True  # Auto-reconnect enabled
)
```

---

## Usage Examples

### Python SDK - Data WebSocket

```python
from fyers_apiv3 import fyersModel
from fyers_apiv3.FyersWebsocket import data_ws

access_token = "APPID:ACCESS_TOKEN"
client_id = "YOUR_APP_ID"

def on_message(message):
    print("Message:", message)

def on_error(error):
    print("Error:", error)

def on_close():
    print("Connection closed")

def on_open():
    print("Connection opened")
    # Subscribe to symbols
    data_ws.subscribe(["NSE:SBIN-EQ", "NSE:RELIANCE-EQ"], mode="full")

# Create WebSocket connection
data_socket = data_ws.FyersDataSocket(
    access_token=access_token,
    log_path="",
    write_to_file=False,
    reconnect=True,
    on_message=on_message,
    on_error=on_error,
    on_close=on_close,
    on_open=on_open
)

# Connect
data_socket.connect()
```

### Python SDK - Order WebSocket

```python
from fyers_apiv3.FyersWebsocket import order_ws

def on_orders(orders):
    print("Orders:", orders)

def on_trades(trades):
    print("Trades:", trades)

def on_positions(positions):
    print("Positions:", positions)

# Create Order WebSocket
order_socket = order_ws.FyersOrderSocket(
    access_token=access_token,
    log_path="",
    write_to_file=False,
    on_orders=on_orders,
    on_trades=on_trades,
    on_positions=on_positions
)

# Connect (auto-subscribes to all order/trade/position updates)
order_socket.connect()
```

### JavaScript SDK - Market Data

```javascript
const { connectMarketData, subscribeMarketQuote } = require('extra-fyers');

const accessToken = 'APPID:ACCESS_TOKEN';

// Connect to market data WebSocket
connectMarketData(accessToken, (data) => {
  console.log('Market data:', data);
});

// Subscribe to symbols
subscribeMarketQuote(accessToken, ['NSE:SBIN-EQ', 'NSE:RELIANCE-EQ']);
```

---

## Best Practices

1. **Use Lite Mode** when only LTP is needed to save bandwidth
2. **Implement Auto-Reconnect** with exponential backoff
3. **Handle Errors Gracefully** and log for debugging
4. **Batch Subscriptions** to minimize messages
5. **Monitor Connection Health** with ping/pong
6. **Test Symbol Limits** with your SDK version
7. **Use Order WebSocket** for real-time order tracking
8. **Avoid Excessive Subscriptions** to prevent throttling
9. **Keep Connections Persistent** (don't reconnect frequently)
10. **Use Official SDKs** for automatic handling of protocol details

---

## Notes

1. WebSocket connections require valid access token
2. Access tokens expire after certain period (re-authenticate as needed)
3. Symbol subscription limits vary by SDK version (test before production)
4. TBT WebSocket uses binary Protobuf encoding (use SDK)
5. Order WebSocket automatically subscribes to all account updates
6. Market data updates are real-time (sub-second latency)
7. Depth data provides top 5 bid/ask levels
8. Official SDKs handle connection management, ping/pong, and reconnection
9. Multiple WebSocket types can be used simultaneously
10. WebSocket is more efficient than polling REST API for real-time data
