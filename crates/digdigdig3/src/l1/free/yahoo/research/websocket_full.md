# yahoo - WebSocket Documentation

## Availability: Yes

## Connection

### URLs
- Public streams: wss://streamer.finance.yahoo.com/
- Private streams: N/A (no private/authenticated streams)
- Regional: None (single global endpoint)

### Connection Process
1. Connect to `wss://streamer.finance.yahoo.com/?version=2`
2. Connection established (no explicit handshake message)
3. No welcome message (connection ready immediately)
4. No auth required (public data only)

## ALL Available Channels/Topics

**CRITICAL:** Yahoo Finance WebSocket does NOT use traditional "channels" - it uses a subscription model with ticker symbols.

| Data Type | Type | Description | Auth? | Free? | Update Frequency | Notes |
|-----------|------|-------------|-------|-------|------------------|-------|
| Price updates | Public | Real-time price, bid, ask, volume | No | Yes | Real-time | Protobuf-encoded |
| Market hours indicator | Public | Pre/regular/post market status | No | Yes | Real-time | Included in price |
| Change data | Public | Price change, % change | No | Yes | Real-time | Included in price |
| Day stats | Public | Day high, low, open | No | Yes | Real-time | Included in price |
| Volume data | Public | Current volume | No | Yes | Real-time | Included in price |

**Note:** All data is delivered in a single message type (PricingData) containing all fields. There are no separate channels for trades, orderbook, etc.

## Subscription Format

### Subscribe Message
```json
{
  "subscribe": ["AAPL", "MSFT", "GOOGL"]
}
```

**As JSON string:**
```javascript
ws.send('{"subscribe":["AAPL","MSFT","GOOGL"]}')
```

### Unsubscribe Message
```json
{
  "unsubscribe": ["TSLA"]
}
```

**As JSON string:**
```javascript
ws.send('{"unsubscribe":["TSLA"]}')
```

### Subscription Confirmation
No explicit confirmation message. Data starts flowing immediately upon subscription.

## Message Formats

### Response Protocol
**CRITICAL:** Yahoo Finance WebSocket responses are **Protocol Buffer (Protobuf) messages**, NOT JSON!

Messages are:
1. Protobuf-encoded binary data
2. Base64-encoded (in some implementations)
3. Must be decoded using the PricingData.proto schema

### PricingData Protobuf Schema

```protobuf
syntax = "proto3";

message PricingData {
  string id = 1;                    // Ticker symbol
  float price = 2;                  // Current price
  sint64 time = 3;                  // Unix timestamp (milliseconds)
  string currency = 4;              // Currency code (e.g., "USD")
  string exchange = 5;              // Exchange name
  int32 quote_type = 6;             // Quote type enum
  int32 market_hours = 7;           // Market hours status
  float change_percent = 8;         // Percent change
  float day_high = 9;               // Day high
  float day_low = 10;               // Day low
  float day_open = 11;              // Day open
  float previous_close = 12;        // Previous close
  float bid = 13;                   // Bid price
  float ask = 14;                   // Ask price
  int64 bid_size = 15;              // Bid size
  int64 ask_size = 16;              // Ask size
  int64 volume = 17;                // Volume
  float change = 18;                // Price change
  string short_name = 19;           // Short name
  string exchange_name = 20;        // Full exchange name
  int32 source_interval = 21;       // Update interval
  int32 exchange_data_delayed = 22; // Delayed data flag
  string tradeable = 23;            // Tradeable status
  float change_percent_real_time = 24;
  float change_real_time = 25;
  float price_real_time = 26;
  sint64 exchange_timezone = 27;    // Exchange timezone
  string exchange_timezone_name = 28;
  float gmt_offset = 29;            // GMT offset
  string market_state = 30;         // PRE, REGULAR, POST, CLOSED
  int64 premarketchange = 31;
  float premarketchangepercent = 32;
  float premarketprice = 33;
  sint64 premarkettime = 34;
  int64 postmarketchange = 35;
  float postmarketchangepercent = 36;
  float postmarketprice = 37;
  sint64 postmarkettime = 38;
}
```

### Example Decoded PricingData (as JSON representation)
```json
{
  "id": "AAPL",
  "price": 150.25,
  "time": 1640995200000,
  "currency": "USD",
  "exchange": "NMS",
  "quote_type": 1,
  "market_hours": 1,
  "change_percent": 1.25,
  "day_high": 151.50,
  "day_low": 149.00,
  "day_open": 149.50,
  "previous_close": 148.50,
  "bid": 150.24,
  "ask": 150.26,
  "bid_size": 100,
  "ask_size": 200,
  "volume": 25000000,
  "change": 1.75,
  "short_name": "Apple Inc.",
  "exchange_name": "NasdaqGS",
  "source_interval": 15,
  "exchange_data_delayed": 0,
  "tradeable": "true",
  "market_state": "REGULAR"
}
```

## Heartbeat / Ping-Pong

### Who initiates?
- Server → Client ping: Yes
- Client → Server ping: No (client should respond to server pings)

### Message Format
- Binary ping/pong frames: Yes (WebSocket protocol level)
- Text messages: No
- JSON messages: No

### Timing
- Ping interval: ~30 seconds (from server)
- Timeout: ~60 seconds (connection closed if no response)
- Client must send ping: Not required (only respond to server pings)

### Implementation
Standard WebSocket ping/pong frames are used. Most WebSocket libraries handle this automatically.

## Connection Limits

- Max connections per IP: Unknown (no official documentation)
- Max connections per API key: N/A (no API keys)
- Max subscriptions per connection: Unknown (~100-200 symbols estimated safe)
- Message rate limit: No explicit limit (server may throttle)
- Auto-disconnect after: None (connections can remain open indefinitely with proper ping/pong)

## Authentication (for private channels)

**Not Applicable** - Yahoo Finance WebSocket only provides public market data. No authentication is supported or required.

## Error Handling

### Connection Errors
WebSocket will close with standard close codes:
- 1000: Normal closure
- 1001: Going away
- 1002: Protocol error
- 1006: Abnormal closure (no close frame)

### Invalid Symbols
If you subscribe to an invalid symbol, no error is returned. You simply won't receive data for that symbol.

### Rate Limiting
No explicit rate limiting on WebSocket. However, excessive connections from the same IP may result in blocks.

## Implementation Notes

### Required Libraries
To use Yahoo Finance WebSocket, you need:
1. WebSocket client library
2. Protocol Buffers decoder/library
3. PricingData.proto schema file

### Example Implementation Flow (Python)

```python
import websocket
import json
from google.protobuf import json_format
from pricing_data_pb2 import PricingData  # Generated from .proto

def on_message(ws, message):
    # Decode protobuf message
    pricing_data = PricingData()
    pricing_data.ParseFromString(message)

    # Convert to dict for easier handling
    data_dict = json_format.MessageToDict(pricing_data)
    print(f"Price update: {data_dict}")

def on_open(ws):
    # Subscribe to symbols
    subscribe_msg = json.dumps({"subscribe": ["AAPL", "MSFT"]})
    ws.send(subscribe_msg)

ws = websocket.WebSocketApp(
    "wss://streamer.finance.yahoo.com/?version=2",
    on_message=on_message,
    on_open=on_open
)
ws.run_forever()
```

### Example Implementation Flow (JavaScript)

```javascript
const WebSocket = require('ws');
const protobuf = require('protobufjs');

// Load proto schema
const root = protobuf.loadSync('PricingData.proto');
const PricingData = root.lookupType('PricingData');

const ws = new WebSocket('wss://streamer.finance.yahoo.com/?version=2');

ws.on('open', () => {
  // Subscribe to symbols
  const subscribeMsg = JSON.stringify({ subscribe: ['AAPL', 'MSFT'] });
  ws.send(subscribeMsg);
});

ws.on('message', (data) => {
  // Decode protobuf message
  const message = PricingData.decode(data);
  const object = PricingData.toObject(message);
  console.log('Price update:', object);
});
```

## Community Libraries with WebSocket Support

### Python
1. **yfinance** (https://github.com/ranaroussi/yfinance)
   - Classes: `WebSocket`, `AsyncWebSocket`
   - Handles protobuf decoding automatically

2. **yliveticker** (https://github.com/yahoofinancelive/yliveticker)
   - Dedicated WebSocket client
   - Built-in protobuf support

3. **yflive** (https://pypi.org/project/yflive/)
   - Simple WebSocket wrapper
   - Real-time quotes

### JavaScript/TypeScript
1. **yahoo-node-streamer** (https://github.com/markosole/yahoo-node-streamer)
   - Node.js streaming client
   - Protobuf decoding included

### Go
1. **go-yfinance** (https://github.com/wnjoon/go-yfinance)
   - Package: `pkg/live`
   - Full protobuf support

## Important Warnings

1. **No Official Documentation**: WebSocket endpoint is reverse-engineered
2. **Subject to Change**: Yahoo can modify or disable endpoint at any time
3. **Rate Limiting**: Excessive connections may result in IP blocks
4. **Protobuf Requirement**: Must handle binary protobuf decoding
5. **Personal Use Only**: Yahoo's terms prohibit commercial use
6. **No Historical Data**: Only real-time streaming data
7. **No Reconnection Guarantee**: Implement reconnection logic in your client

## Data Quality

- **Latency**: Generally <100ms for real-time quotes
- **Reliability**: Good during market hours, occasional disconnects
- **Coverage**: All symbols available via REST API also available via WebSocket
- **Delayed Data**: Some exchanges provide 15-minute delayed data
- **Pre/Post Market**: Available for supported exchanges

## Comparison to REST API

| Feature | WebSocket | REST |
|---------|-----------|------|
| Real-time | Yes (push) | No (poll) |
| Historical | No | Yes |
| Fundamentals | No | Yes |
| Rate limiting | Less strict | Strict (~2000/hr) |
| Complexity | Higher (protobuf) | Lower (JSON) |
| Use case | Live prices | Historical analysis |
