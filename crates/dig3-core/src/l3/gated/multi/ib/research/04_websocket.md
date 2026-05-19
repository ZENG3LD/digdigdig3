# Interactive Brokers Client Portal Web API - WebSocket Streaming

## Overview

Interactive Brokers Client Portal Web API provides WebSocket support for **real-time asynchronous streaming** of market data, order updates, account information, and notifications. This enables event-driven applications without the need for constant polling.

## WebSocket URLs

**Gateway (Local):** `wss://localhost:5000/v1/api/ws`
**Production (OAuth):** `wss://api.ibkr.com/v1/api/ws`

## Connection Requirements

### Prerequisites
- Active brokerage session (authenticated via Gateway or OAuth)
- Valid session established through REST API first
- Market data subscriptions (for market data streaming)
- WebSocket client library supporting WSS protocol

### Authentication
WebSocket connections **inherit authentication from the established brokerage session**. No additional authentication headers are required in the WebSocket handshake. The session cookies/tokens are used automatically.

## Connection Establishment

### Python Example (websocket-client library)

```python
import websocket
import ssl
import json

def on_open(ws):
    print("WebSocket connection opened")
    # Subscribe to market data after connection
    subscribe_msg = 'smd+265598+{"fields":["31","84","86"]}'
    ws.send(subscribe_msg)

def on_message(ws, message):
    print(f"Received: {message}")
    data = json.loads(message)
    # Handle incoming data

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("WebSocket connection closed")

# Disable SSL verification for localhost Gateway (development only)
ssl_options = {"cert_reqs": ssl.CERT_NONE}

# Create WebSocket connection
ws = websocket.WebSocketApp(
    "wss://localhost:5000/v1/api/ws",
    on_open=on_open,
    on_message=on_message,
    on_error=on_error,
    on_close=on_close
)

# Run forever
ws.run_forever(sslopt=ssl_options)
```

### JavaScript Example (browser/Node.js)

```javascript
const WebSocket = require('ws');

// For localhost Gateway, ignore self-signed certificate
const ws = new WebSocket('wss://localhost:5000/v1/api/ws', {
  rejectUnauthorized: false  // Only for development with Gateway
});

ws.on('open', function open() {
  console.log('WebSocket connected');

  // Subscribe to market data
  const subscribe = 'smd+265598+{"fields":["31","84","86"]}';
  ws.send(subscribe);
});

ws.on('message', function incoming(data) {
  console.log('Received:', data);
  const message = JSON.parse(data);
  // Handle message
});

ws.on('error', function error(err) {
  console.error('WebSocket error:', err);
});

ws.on('close', function close() {
  console.log('WebSocket disconnected');
});
```

### Connection Handshake

The WebSocket connection uses standard WebSocket handshake with HTTP Upgrade:

```http
GET /v1/api/ws HTTP/1.1
Host: localhost:5000
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
```

**Server Response:**
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

## WebSocket Message Format

All WebSocket messages are **text-based strings** (not JSON objects). The format varies by topic:

### General Format
```
TOPIC+PARAMETERS
```

## Subscription Topics

### 1. Market Data Streaming (smd)

**Format:**
```
smd+{conid}+{"fields":["field1","field2",...]}
```

**Subscribe to Single Contract:**
```
smd+265598+{"fields":["31","84","86","88","85","87"]}
```

**Subscribe to Multiple Contracts:**
Send multiple subscription messages:
```
smd+265598+{"fields":["31","84","86"]}
smd+8314+{"fields":["31","84","86"]}
smd+756733+{"fields":["31","84","86"]}
```

**Field IDs (Same as REST API):**
- `31` - Last price
- `55` - Symbol
- `70` - High (session)
- `71` - Low (session)
- `82` - Open price
- `83` - Close price
- `84` - Bid price
- `85` - Ask size
- `86` - Ask price
- `87` - Volume
- `88` - Bid size
- `7059` - Last size
- `7051` - Last exchange
- `7057` - Ask exchange
- `7058` - Bid exchange

**Market Data Response Format:**
```json
{
  "topic": "smd",
  "conid": 265598,
  "conidEx": "265598",
  "31": 185.50,
  "84": 185.48,
  "86": 185.52,
  "88": 500,
  "85": 300,
  "87": 55234000,
  "_updated": 1706282450123,
  "server_id": "m1"
}
```

**Fields:**
- `topic` - Subscription topic ("smd")
- `conid` - Contract ID
- `conidEx` - Contract ID with exchange
- Field IDs as keys with corresponding values
- `_updated` - Unix timestamp (milliseconds) of last update
- `server_id` - Server identifier

**Update Frequency:**
Market data updates arrive whenever there's a change in the subscribed fields (event-driven). High-frequency updates possible during active trading.

**Subscription Limits:**
- Concurrent subscriptions consume market data lines from your allocation
- Standard accounts: Typically 100 simultaneous market data lines
- Each WebSocket subscription counts toward this limit

**Unsubscribe:**
Use REST API endpoint:
```http
DELETE /iserver/marketdata/{conid}/unsubscribe
```

### 2. Live Order Updates (sor)

**Format:**
```
sor+{}
```

**Subscribe to Order Updates:**
```
sor+{}
```

**Order Update Response Format:**
```json
{
  "topic": "sor",
  "acct": "DU12345",
  "conid": 265598,
  "orderId": 987654321,
  "orderStatus": "Filled",
  "side": "B",
  "totalSize": 100.0,
  "filledQuantity": 100.0,
  "remainingQuantity": 0.0,
  "avgPrice": 185.52,
  "price": 185.50,
  "ticker": "AAPL",
  "secType": "STK",
  "timeInForce": "DAY",
  "lastExecutionTime": "240126 10:30:45",
  "lastExecutionTime_r": 1706268645000,
  "orderType": "LMT",
  "bgColor": "#FFFFFF",
  "fgColor": "#000000"
}
```

**Order Status Values:**
- `PendingSubmit` - Pending submission
- `Submitted` - Submitted to exchange
- `PreSubmitted` - Pre-submitted
- `Filled` - Completely filled
- `Cancelled` - Cancelled
- `Inactive` - Inactive
- `ApiCancelled` - Cancelled via API
- `PendingCancel` - Cancellation pending

**Update Triggers:**
- Order submission
- Order acceptance by exchange
- Partial fills
- Complete fills
- Order modifications
- Order cancellations
- Order rejections

**No Unsubscribe:**
Order updates are account-level and cannot be unsubscribed individually. They remain active for the session duration.

### 3. Account Summary Updates (acc)

**Format:**
```
acc+{}
```

**Subscribe to Account Updates:**
```
acc+{}
```

**Account Update Response Format:**
```json
{
  "topic": "acc",
  "acctcode": "DU12345",
  "netliquidation": 125000.50,
  "totalcashvalue": 100000.00,
  "buyingpower": 250001.00,
  "equity": 125000.50,
  "unrealizedpnl": 550.00,
  "realizedpnl": 0.00,
  "timestamp": 1706282450123
}
```

**Fields (subset of account summary):**
- `topic` - Subscription topic ("acc")
- `acctcode` - Account code
- `netliquidation` - Net liquidation value
- `totalcashvalue` - Total cash value
- `buyingpower` - Buying power
- `equity` - Equity with loan value
- `unrealizedpnl` - Unrealized P&L
- `realizedpnl` - Realized P&L
- `timestamp` - Update timestamp (Unix milliseconds)

**Update Frequency:**
Account summary updates arrive when account values change due to:
- Market price movements (affects positions)
- Order executions
- Deposits/withdrawals
- Margin changes

Typically updates every few seconds during active trading.

### 4. Profit & Loss Updates (pnl)

**Format:**
```
pnl+{}
```

**Subscribe to P&L Updates:**
```
pnl+{}
```

**P&L Update Response Format:**
```json
{
  "topic": "pnl",
  "acct": "DU12345",
  "dpl": 550.00,
  "nl": 125000.50,
  "upl": 550.00,
  "el": 125000.50,
  "mv": 25000.50,
  "timestamp": 1706282450123
}
```

**Fields:**
- `topic` - Subscription topic ("pnl")
- `acct` - Account code
- `dpl` - Daily P&L
- `nl` - Net liquidation
- `upl` - Unrealized P&L
- `el` - Equity with loan value
- `mv` - Market value of positions
- `timestamp` - Update timestamp

**Update Frequency:**
Real-time updates as P&L changes due to market movements or executions.

### 5. Trade Execution Data (trades)

**Format:**
```
trades+{}
```

**Subscribe to Trade Executions:**
```
trades+{}
```

**Trade Execution Response Format:**
```json
{
  "topic": "trades",
  "execution_id": "0000e0d5.63d4e3e2.01.01",
  "symbol": "AAPL",
  "side": "B",
  "order_description": "Bought 100 Limit 185.50",
  "trade_time": "240126 10:30:45",
  "trade_time_r": 1706268645000,
  "size": 100.0,
  "price": "185.52",
  "order_ref": "ClientRef123",
  "exchange": "NASDAQ",
  "commission": "1.00",
  "net_amount": 18553.00,
  "account": "DU12345",
  "conid": 265598,
  "sec_type": "STK"
}
```

**Fields:**
- `topic` - Subscription topic ("trades")
- `execution_id` - Unique execution identifier
- `symbol` - Symbol
- `side` - Side: "B" (Buy) or "S" (Sell)
- `size` - Executed quantity
- `price` - Execution price
- `trade_time_r` - Execution timestamp (Unix milliseconds)
- `commission` - Commission charged
- `net_amount` - Net amount (price * size + commission)
- `exchange` - Execution exchange
- `conid` - Contract ID
- `sec_type` - Security type

**Update Triggers:**
Real-time notifications when orders are executed (fully or partially).

### 6. Notifications (ntf)

**Format:**
```
ntf+{}
```

**Subscribe to Notifications:**
```
ntf+{}
```

**Notification Response Format:**
```json
{
  "topic": "ntf",
  "id": "notif123",
  "timestamp": 1706282450123,
  "category": "Trade",
  "title": "Order Filled",
  "message": "Your order for AAPL has been filled",
  "details": "Order #987654321 - Bought 100 AAPL @ 185.52"
}
```

**Notification Categories:**
- `Trade` - Trade execution notifications
- `Order` - Order status notifications
- `Account` - Account-related notifications
- `Margin` - Margin notifications
- `System` - System messages

## Heartbeat Messages

The WebSocket connection sends periodic **heartbeat messages** to maintain the connection and confirm it's active.

**Heartbeat Format:**
```json
{
  "topic": "system",
  "heartbeat": true,
  "timestamp": 1706282450123
}
```

**Frequency:** Typically every 30-60 seconds

**Client Response:** No action required; heartbeats are informational.

## Connection Management

### Connection Lifecycle

1. **Establish REST Session:** Authenticate via REST API first
2. **Open WebSocket:** Connect to WSS endpoint
3. **Receive Welcome Message:** Server sends initial connection confirmation
4. **Subscribe to Topics:** Send subscription messages
5. **Receive Updates:** Handle incoming messages
6. **Maintain Connection:** Monitor heartbeats
7. **Close Gracefully:** Send close frame when done

### Welcome Message

Upon connection, the server sends an initial message containing authentication status and account information:

```json
{
  "topic": "system",
  "authenticated": true,
  "competing": false,
  "connected": true,
  "accounts": ["DU12345"],
  "serverTime": 1706282450123
}
```

**Fields:**
- `authenticated` - Brokerage session authenticated
- `competing` - Competing session detected
- `connected` - Connected to backend
- `accounts` - Accessible account list
- `serverTime` - Server timestamp

### Session Timeout

WebSocket connections share the same session timeout as REST API:
- **Idle Timeout:** ~6 minutes without activity
- **Maximum Duration:** 24 hours
- **Session Reset:** Midnight in account's timezone

**Maintaining Session:**
Continue using REST API `/tickle` endpoint periodically (every 30-60 seconds) to keep the underlying brokerage session alive. The WebSocket connection depends on the REST session.

### Reconnection Logic

**Best Practices:**

```python
import time
import websocket
import ssl

class IBWebSocket:
    def __init__(self, url):
        self.url = url
        self.ws = None
        self.should_reconnect = True
        self.reconnect_delay = 5  # seconds

    def connect(self):
        ssl_options = {"cert_reqs": ssl.CERT_NONE}
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self.on_open,
            on_message=self.on_message,
            on_error=self.on_error,
            on_close=self.on_close
        )
        self.ws.run_forever(sslopt=ssl_options)

    def on_open(self, ws):
        print("Connected to IB WebSocket")
        # Re-subscribe to topics after reconnection
        self.resubscribe()

    def on_message(self, ws, message):
        # Handle message
        print(f"Received: {message}")

    def on_error(self, ws, error):
        print(f"WebSocket error: {error}")

    def on_close(self, ws, close_status_code, close_msg):
        print("WebSocket closed")
        if self.should_reconnect:
            print(f"Reconnecting in {self.reconnect_delay} seconds...")
            time.sleep(self.reconnect_delay)
            self.connect()

    def resubscribe(self):
        # Re-send subscriptions
        self.ws.send('smd+265598+{"fields":["31","84","86"]}')
        self.ws.send('sor+{}')
        self.ws.send('acc+{}')

    def close(self):
        self.should_reconnect = False
        if self.ws:
            self.ws.close()

# Usage
ws_client = IBWebSocket("wss://localhost:5000/v1/api/ws")
ws_client.connect()
```

**Reconnection Strategy:**
1. Detect disconnection via `on_close` callback
2. Wait for backoff period (e.g., 5 seconds)
3. Attempt reconnection
4. Re-subscribe to all topics upon successful connection
5. Implement exponential backoff for repeated failures

### Graceful Shutdown

```python
import signal
import sys

def signal_handler(sig, frame):
    print('Shutting down gracefully...')
    ws_client.close()
    sys.exit(0)

signal.signal(signal.SIGINT, signal_handler)
```

## Message Parsing

### Handling Different Message Types

```python
import json

def on_message(ws, message):
    try:
        data = json.loads(message)
        topic = data.get('topic')

        if topic == 'smd':
            handle_market_data(data)
        elif topic == 'sor':
            handle_order_update(data)
        elif topic == 'acc':
            handle_account_update(data)
        elif topic == 'pnl':
            handle_pnl_update(data)
        elif topic == 'trades':
            handle_trade_execution(data)
        elif topic == 'ntf':
            handle_notification(data)
        elif topic == 'system':
            handle_system_message(data)
        else:
            print(f"Unknown topic: {topic}")

    except json.JSONDecodeError:
        print(f"Invalid JSON: {message}")
    except Exception as e:
        print(f"Error handling message: {e}")

def handle_market_data(data):
    conid = data.get('conid')
    last_price = data.get('31')
    bid = data.get('84')
    ask = data.get('86')
    timestamp = data.get('_updated')
    print(f"Market Data [{conid}]: Last={last_price}, Bid={bid}, Ask={ask}, Time={timestamp}")

def handle_order_update(data):
    order_id = data.get('orderId')
    status = data.get('orderStatus')
    filled_qty = data.get('filledQuantity')
    avg_price = data.get('avgPrice')
    print(f"Order Update [{order_id}]: Status={status}, Filled={filled_qty} @ {avg_price}")

def handle_account_update(data):
    account = data.get('acctcode')
    net_liq = data.get('netliquidation')
    buying_power = data.get('buyingpower')
    print(f"Account Update [{account}]: NetLiq={net_liq}, BuyingPower={buying_power}")

def handle_pnl_update(data):
    account = data.get('acct')
    daily_pnl = data.get('dpl')
    unrealized_pnl = data.get('upl')
    print(f"P&L Update [{account}]: Daily={daily_pnl}, Unrealized={unrealized_pnl}")

def handle_trade_execution(data):
    symbol = data.get('symbol')
    side = data.get('side')
    size = data.get('size')
    price = data.get('price')
    print(f"Trade Execution: {side} {size} {symbol} @ {price}")

def handle_notification(data):
    title = data.get('title')
    message = data.get('message')
    print(f"Notification: {title} - {message}")

def handle_system_message(data):
    if data.get('heartbeat'):
        print("Heartbeat received")
    else:
        print(f"System message: {data}")
```

## Rate Limits

WebSocket subscriptions are **not rate-limited** in the same way as REST API. However:

- **Market Data Lines:** Limited by account's market data subscription (typically 100 concurrent)
- **Connection Limits:** Single WebSocket connection per session recommended
- **Message Frequency:** Server controls update frequency based on data changes

**Best Practice:** Use a single WebSocket connection and multiplex all subscriptions through it.

## SSL Certificate Handling

### Localhost Gateway (Development)

For Client Portal Gateway on localhost, SSL verification must be disabled:

**Python:**
```python
ssl_options = {"cert_reqs": ssl.CERT_NONE}
ws.run_forever(sslopt=ssl_options)
```

**JavaScript/Node.js:**
```javascript
const ws = new WebSocket('wss://localhost:5000/v1/api/ws', {
  rejectUnauthorized: false
});
```

### Production (OAuth)

For production OAuth endpoints with valid SSL certificates:

**Python:**
```python
# SSL verification enabled by default
ws.run_forever()
```

**JavaScript/Node.js:**
```javascript
// SSL verification enabled by default
const ws = new WebSocket('wss://api.ibkr.com/v1/api/ws');
```

## Error Handling

### Common WebSocket Errors

**Connection Refused:**
```
Error: Connection refused
Cause: Gateway not running or wrong port
Solution: Start Gateway and verify port
```

**Authentication Failed:**
```
Error: 401 Unauthorized
Cause: Brokerage session not authenticated
Solution: Authenticate via REST API first
```

**SSL Certificate Error:**
```
Error: CERTIFICATE_VERIFY_FAILED
Cause: Self-signed certificate (localhost Gateway)
Solution: Disable SSL verification for development
```

**Connection Closed Unexpectedly:**
```
Error: Connection closed by server
Cause: Session timeout or competing session
Solution: Re-authenticate and reconnect
```

### Error Response Format

```json
{
  "topic": "error",
  "code": "AUTH_ERROR",
  "message": "Brokerage session not authenticated",
  "timestamp": 1706282450123
}
```

## Performance Considerations

### Bandwidth Usage

**Typical Data Rates:**
- **Market Data:** 1-10 messages/second per subscribed contract (during active trading)
- **Order Updates:** Event-driven, typically <1 message/second
- **Account Updates:** 1 message every few seconds
- **P&L Updates:** 1 message every few seconds

**Bandwidth Estimate:**
- Low activity: ~1 KB/s
- Medium activity (10 subscriptions): ~10 KB/s
- High activity (100 subscriptions): ~100 KB/s

### Latency

**Expected Latency:**
- **Gateway (Localhost):** <10ms
- **Production (OAuth):** 50-200ms (depends on geographic location)

**Factors Affecting Latency:**
- Network connection quality
- Distance to IBKR servers
- Server load
- Market data provider delays

### Message Buffering

For high-frequency market data updates, implement buffering:

```python
from collections import deque
from threading import Lock
import time

class MessageBuffer:
    def __init__(self, max_size=1000):
        self.buffer = deque(maxlen=max_size)
        self.lock = Lock()

    def add(self, message):
        with self.lock:
            self.buffer.append({
                'data': message,
                'timestamp': time.time()
            })

    def get_batch(self, max_age=1.0):
        """Get messages newer than max_age seconds"""
        with self.lock:
            current_time = time.time()
            return [
                msg['data'] for msg in self.buffer
                if current_time - msg['timestamp'] < max_age
            ]

# Usage
buffer = MessageBuffer()

def on_message(ws, message):
    buffer.add(message)

# Process buffered messages periodically
def process_messages():
    while True:
        messages = buffer.get_batch(max_age=1.0)
        for msg in messages:
            # Process message
            pass
        time.sleep(0.1)
```

## Multi-Threading Considerations

WebSocket callbacks run in a separate thread. Use thread-safe data structures:

```python
from threading import Thread, Lock
from queue import Queue
import json

class IBWebSocketClient:
    def __init__(self):
        self.message_queue = Queue()
        self.lock = Lock()
        self.subscriptions = {}

    def on_message(self, ws, message):
        # Put message in queue for processing in main thread
        self.message_queue.put(message)

    def process_messages(self):
        """Run in main thread"""
        while True:
            if not self.message_queue.empty():
                message = self.message_queue.get()
                try:
                    data = json.loads(message)
                    self.handle_message(data)
                except Exception as e:
                    print(f"Error processing message: {e}")

    def handle_message(self, data):
        topic = data.get('topic')
        with self.lock:
            # Thread-safe message handling
            if topic in self.subscriptions:
                self.subscriptions[topic](data)

# Usage
client = IBWebSocketClient()

# Start message processing in main thread
processing_thread = Thread(target=client.process_messages, daemon=True)
processing_thread.start()
```

## Complete Example: Market Data Streaming

```python
import websocket
import ssl
import json
import time
from threading import Thread

class IBMarketDataStreamer:
    def __init__(self, url, contracts):
        self.url = url
        self.contracts = contracts  # List of conids
        self.ws = None
        self.prices = {}  # Store latest prices

    def connect(self):
        ssl_options = {"cert_reqs": ssl.CERT_NONE}
        self.ws = websocket.WebSocketApp(
            self.url,
            on_open=self.on_open,
            on_message=self.on_message,
            on_error=self.on_error,
            on_close=self.on_close
        )

        # Run in separate thread
        ws_thread = Thread(target=lambda: self.ws.run_forever(sslopt=ssl_options))
        ws_thread.daemon = True
        ws_thread.start()

    def on_open(self, ws):
        print("WebSocket connected")
        # Subscribe to market data for all contracts
        for conid in self.contracts:
            subscription = f'smd+{conid}+{{"fields":["31","84","86","87","88","85"]}}'
            ws.send(subscription)
            print(f"Subscribed to market data for conid {conid}")

    def on_message(self, ws, message):
        try:
            data = json.loads(message)

            if data.get('topic') == 'smd':
                conid = data.get('conid')
                self.prices[conid] = {
                    'last': data.get('31'),
                    'bid': data.get('84'),
                    'ask': data.get('86'),
                    'volume': data.get('87'),
                    'bid_size': data.get('88'),
                    'ask_size': data.get('85'),
                    'timestamp': data.get('_updated')
                }
                self.display_quote(conid)

            elif data.get('topic') == 'system':
                if data.get('heartbeat'):
                    print("Heartbeat received")

        except json.JSONDecodeError:
            print(f"Invalid JSON: {message}")
        except Exception as e:
            print(f"Error: {e}")

    def on_error(self, ws, error):
        print(f"WebSocket error: {error}")

    def on_close(self, ws, close_status_code, close_msg):
        print("WebSocket closed")

    def display_quote(self, conid):
        quote = self.prices.get(conid, {})
        print(f"[{conid}] Last: {quote.get('last')}, "
              f"Bid: {quote.get('bid')} x {quote.get('bid_size')}, "
              f"Ask: {quote.get('ask')} x {quote.get('ask_size')}, "
              f"Volume: {quote.get('volume')}")

    def get_latest_price(self, conid):
        """Get latest price for a contract"""
        return self.prices.get(conid, {}).get('last')

    def close(self):
        if self.ws:
            self.ws.close()

# Usage
if __name__ == "__main__":
    # Contract IDs: AAPL, SPY, TSLA
    contracts = [265598, 8314, 76792991]

    streamer = IBMarketDataStreamer("wss://localhost:5000/v1/api/ws", contracts)
    streamer.connect()

    # Keep main thread alive
    try:
        while True:
            time.sleep(1)
            # Access latest prices
            aapl_price = streamer.get_latest_price(265598)
            if aapl_price:
                print(f"Current AAPL price: {aapl_price}")

    except KeyboardInterrupt:
        print("Shutting down...")
        streamer.close()
```

## Best Practices

### 1. Session Management
- Maintain REST session with periodic `/tickle` calls
- Monitor WebSocket connection health
- Implement automatic reconnection with exponential backoff

### 2. Subscription Management
- Subscribe only to needed data (respects market data limits)
- Unsubscribe from unused contracts via REST API
- Track active subscriptions to avoid duplicates

### 3. Error Handling
- Handle all exceptions in message callbacks
- Log errors for debugging
- Implement graceful degradation on errors

### 4. Performance
- Use single WebSocket connection for all subscriptions
- Process messages asynchronously (avoid blocking callbacks)
- Buffer high-frequency updates if needed

### 5. Security
- Only disable SSL verification for localhost Gateway
- Never expose Gateway to external networks
- Use OAuth 2.0 for production deployments

### 6. Testing
- Test reconnection logic thoroughly
- Simulate network failures
- Test session timeout scenarios
- Verify all subscription topics

## Troubleshooting

### Issue: WebSocket connects but no data received

**Cause:** Not subscribed to any topics
**Solution:** Send subscription messages after connection opens

### Issue: Market data fields return null

**Cause:** No market data subscription for exchange
**Solution:** Verify market data subscriptions in account settings

### Issue: Connection drops after 6 minutes

**Cause:** Session timeout due to inactivity
**Solution:** Implement tickle mechanism in REST API

### Issue: "Market data lines exceeded" error

**Cause:** Too many concurrent subscriptions
**Solution:** Unsubscribe from unused contracts, upgrade data plan

### Issue: High latency in updates

**Cause:** Network latency or server load
**Solution:** Check network connection, consider co-location options

---

**Research Date:** 2026-01-26
**API Version:** v1.0
**WebSocket Protocol:** WSS (WebSocket Secure)
**Documentation:** https://www.interactivebrokers.com/campus/trading-lessons/websockets/
