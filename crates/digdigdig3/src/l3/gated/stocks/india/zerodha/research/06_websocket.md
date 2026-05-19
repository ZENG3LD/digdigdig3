# Zerodha Kite Connect - WebSocket Documentation

## Availability: Yes

WebSocket streaming is the most efficient way to receive real-time quotes for instruments across all exchanges during live market hours.

---

## Connection

### URLs

- **WebSocket URL**: `wss://ws.kite.trade`
- **Protocol**: WSS (secure WebSocket)
- **Single URL**: No separate public/private streams

### Connection Process

**1. Connect to URL with authentication**
```
wss://ws.kite.trade?api_key={api_key}&access_token={access_token}
```

**2. Authentication**
- Authentication happens during WebSocket handshake via query parameters
- **No post-connection authentication message required**
- If authentication fails, connection will be rejected immediately

**3. Connection established**
- Connection opens immediately after successful handshake
- Ready to receive subscription commands

**4. Subscription**
- Send JSON subscription messages to subscribe to instrument tokens
- Start receiving binary market data packets

**Example Connection** (JavaScript):
```javascript
const apiKey = "your_api_key";
const accessToken = "your_access_token";
const ws = new WebSocket(`wss://ws.kite.trade?api_key=${apiKey}&access_token=${accessToken}`);

ws.onopen = () => {
    console.log("Connected to Kite WebSocket");
    // Subscribe to instruments
    ws.send(JSON.stringify({
        "a": "subscribe",
        "v": [408065, 738561]  // INFY, RELIANCE tokens
    }));
};

ws.onmessage = (event) => {
    // Handle binary/text messages
};
```

---

## Subscription Format

### Request Structure

All subscription requests are **JSON messages** with two parameters:

| Parameter | Type | Description |
|-----------|------|-------------|
| a | string | Action ("subscribe", "unsubscribe", "mode") |
| v | array/object | Value (instrument tokens or mode configuration) |

### Actions

#### 1. Subscribe

**Purpose**: Subscribe to instrument(s) for real-time updates

**Format**:
```json
{
  "a": "subscribe",
  "v": [408065, 738561, 15199234]
}
```

**Parameters**:
- `a`: "subscribe"
- `v`: Array of instrument tokens (integers)

**Example** (Python):
```python
subscribe_msg = {
    "a": "subscribe",
    "v": [408065, 738561]  # INFY, RELIANCE
}
ws.send(json.dumps(subscribe_msg))
```

---

#### 2. Unsubscribe

**Purpose**: Unsubscribe from instrument(s)

**Format**:
```json
{
  "a": "unsubscribe",
  "v": [408065]
}
```

**Parameters**:
- `a`: "unsubscribe"
- `v`: Array of instrument tokens to unsubscribe

**Example**:
```python
unsubscribe_msg = {
    "a": "unsubscribe",
    "v": [408065]  # Stop receiving INFY quotes
}
ws.send(json.dumps(unsubscribe_msg))
```

---

#### 3. Set Mode

**Purpose**: Change streaming mode for subscribed instruments

**Format**:
```json
{
  "a": "mode",
  "v": ["full", [408065, 738561]]
}
```

**Parameters**:
- `a`: "mode"
- `v`: [mode_string, [instrument_tokens]]

**Available Modes**:
- `"ltp"` - Last Traded Price only (8 bytes per packet)
- `"quote"` - OHLC and other fields, no market depth (44 bytes)
- `"full"` - All fields including 5-level market depth (184 bytes)

**Example**:
```python
# Set to full mode for detailed data
mode_msg = {
    "a": "mode",
    "v": ["full", [408065, 738561]]
}
ws.send(json.dumps(mode_msg))

# Set to LTP mode for lightweight updates
mode_msg = {
    "a": "mode",
    "v": ["ltp", [408065]]
}
ws.send(json.dumps(mode_msg))
```

---

## ALL Available Channels/Topics

WebSocket provides real-time streaming for:

| Channel Type | Description | Auth Required | Free Tier | Update Frequency | Mode |
|--------------|-------------|---------------|-----------|------------------|------|
| Market Data | Price, OHLC, volume, depth | Yes | No (Paid only) | Real-time | ltp/quote/full |
| Order Postbacks | Order status updates | Yes | Yes | Real-time | Text messages |
| Error Messages | Error notifications | Yes | Yes | Event-based | Text messages |
| System Messages | Broker alerts, messages | Yes | Yes | Event-based | Text messages |

**Important**: WebSocket streaming is **NOT available in the free Personal API tier**. Requires paid Connect API subscription (₹500/month).

---

## Streaming Modes

### Mode Comparison

| Mode | Fields Included | Packet Size | Use Case |
|------|----------------|-------------|----------|
| **ltp** | Last price only | 8 bytes | Price tracking only |
| **quote** | OHLC, volume, bid/ask quantities, OI | 44 bytes | Standard quotes without depth |
| **full** | All quote fields + 5-level market depth | 184 bytes | Full orderbook analysis |

### Mode: LTP (Lightweight)

**Packet Size**: 8 bytes

**Fields**:
- Instrument token (4 bytes)
- Last traded price (4 bytes, int32, in paise)

**Use Case**: High-frequency price monitoring with minimal bandwidth

**Example Data**:
```
Instrument: 408065 (INFY)
Last Price: 145050 (paise) = 1450.50 INR
```

---

### Mode: QUOTE (Standard)

**Packet Size**: 44 bytes

**Fields** (all int32 in paise):
- Instrument token
- Last traded price
- Last traded quantity
- Average traded price
- Volume traded
- Total buy quantity
- Total sell quantity
- Open
- High
- Low
- Close
- Net change
- Timestamp (seconds since epoch)
- Open Interest (for derivatives)

**Use Case**: Comprehensive quotes without orderbook depth

---

### Mode: FULL (Complete)

**Packet Size**: 184 bytes

**Fields**: All quote fields + market depth

**Market Depth Structure**:
- **5 levels of buy orders** (bid side)
- **5 levels of sell orders** (ask side)

**Each depth level** (10 levels total):
- Quantity (int32)
- Price (int32, paise)
- Orders count (int32)

**Use Case**: Full orderbook analysis, algorithmic trading with depth

---

## Message Formats

### Binary Messages (Market Data)

**Type**: Binary (ArrayBuffer/Blob)

**Encoding**: Custom binary format

**Structure**:
```
[2-byte packet count]
[2-byte packet 1 length][packet 1 data]
[2-byte packet 2 length][packet 2 data]
...
```

**Important**:
- Market data is **ALWAYS binary**
- Must parse as bytes, not text
- Data is in int32 format (4 bytes per field)
- **Prices are in PAISE** (divide by 100 for INR)
- Use official SDKs for parsing (complex binary structure)

**Binary Structure Detail**:

1. **Packet Count** (2 bytes, uint16): Number of packets in message
2. For each packet:
   - **Packet Length** (2 bytes, uint16): Length of packet data
   - **Packet Data** (N bytes):
     - Instrument token (4 bytes, int32)
     - Mode-specific fields (4 bytes each, int32)
     - For full mode: 10 depth levels × 3 fields × 4 bytes

**Example Binary Parsing** (JavaScript):
```javascript
ws.onmessage = (event) => {
    if (event.data instanceof ArrayBuffer) {
        // Binary market data
        const buffer = new DataView(event.data);
        const packetCount = buffer.getUint16(0, false);  // Big-endian

        let offset = 2;
        for (let i = 0; i < packetCount; i++) {
            const packetLength = buffer.getUint16(offset, false);
            offset += 2;

            const instrumentToken = buffer.getInt32(offset, false);
            offset += 4;

            // Parse mode-specific fields...
            // (Use official SDK for complete parsing)
        }
    } else {
        // Text message (order postback, error, etc.)
        const message = JSON.parse(event.data);
        console.log("Text message:", message);
    }
};
```

**Recommendation**: **Use official SDKs** for binary parsing. The binary structure is complex and error-prone to parse manually.

---

### Text Messages (JSON)

**Type**: Text (JSON-encoded)

**Message Types**:
- `order` - Order postback
- `error` - Error messages
- `message` - System messages

#### Order Postback

**Purpose**: Real-time order status updates

**Format**:
```json
{
  "type": "order",
  "data": {
    "order_id": "240126000012345",
    "exchange_order_id": "1200000012345678",
    "parent_order_id": null,
    "status": "COMPLETE",
    "tradingsymbol": "INFY",
    "exchange": "NSE",
    "instrument_token": 408065,
    "transaction_type": "BUY",
    "order_type": "LIMIT",
    "product": "CNC",
    "quantity": 10,
    "filled_quantity": 10,
    "pending_quantity": 0,
    "price": 1450.00,
    "trigger_price": 0,
    "average_price": 1449.75,
    "status_message": null,
    "guid": "abc123xyz",
    "placed_by": "XX0000",
    "variety": "regular",
    "order_timestamp": "2026-01-26 09:15:32",
    "exchange_timestamp": "2026-01-26 09:15:33",
    "exchange_update_timestamp": "2026-01-26 09:15:34",
    "tag": "myorder123"
  }
}
```

**Fields**: Same as REST API order object

**Use Case**: Real-time order tracking without polling

**Example Handler**:
```javascript
if (message.type === "order") {
    const order = message.data;
    console.log(`Order ${order.order_id} status: ${order.status}`);

    if (order.status === "COMPLETE") {
        console.log(`Filled ${order.filled_quantity} @ ${order.average_price}`);
    }
}
```

---

#### Error Message

**Purpose**: Error notifications

**Format**:
```json
{
  "type": "error",
  "data": "Error message description"
}
```

**Common Errors**:
- Invalid subscription
- Rate limit exceeded
- Connection issues
- Authentication failures

**Example Handler**:
```javascript
if (message.type === "error") {
    console.error("WebSocket error:", message.data);
}
```

---

#### System Message

**Purpose**: Broker alerts, system notifications

**Format**:
```json
{
  "type": "message",
  "data": "System message from broker"
}
```

**Example Messages**:
- Market opening/closing alerts
- Corporate action notifications
- System maintenance alerts

---

## Heartbeat / Ping-Pong

### Who initiates?

- **Server → Client ping**: No (server doesn't send explicit pings)
- **Client → Server ping**: Not required
- **Automatic keepalive**: Yes (1-byte heartbeat from server)

### Keepalive Mechanism

**Server sends 1-byte heartbeat** every few seconds on idle connections to maintain the link.

**Format**: Binary 1-byte message

**Client Action**: No response required (automatic handling)

**Purpose**: Prevent connection timeout, detect dead connections

### Timing

- **Heartbeat interval**: Every few seconds (exact interval not documented)
- **Timeout**: Connection closed if no activity for extended period
- **Client must NOT send ping**: Not necessary, server handles keepalive

### Example

```
Server → Client: [0x01] (1-byte binary heartbeat)
Client: (no action required, connection kept alive)
```

**Important**:
- Do NOT implement manual ping/pong
- Server handles connection keepalive automatically
- Client receives 1-byte messages during idle periods (ignore them)

---

## Connection Limits

### Subscription Limits

| Limit Type | Value |
|------------|-------|
| **Max instruments per connection** | 3,000 |
| **Max connections per API key** | 3 |
| **Max subscriptions total** | 3 connections × 3,000 = 9,000 instruments |

### Message Rate Limits

- **Outgoing messages**: No documented limit (reasonable usage expected)
- **Incoming messages**: Unlimited (server pushes as needed)
- **Subscription changes**: No specific limit (avoid rapid subscribe/unsubscribe)

### Connection Duration

- **Max lifetime**: Unlimited (until market close or manual disconnect)
- **Auto-disconnect after market close**: Yes (typically)
- **Idle timeout**: No (server sends heartbeats)
- **Reconnection required**: After token expiry (6 AM next day)

---

## Authentication

### Method

**URL parameters** during WebSocket handshake:

```
wss://ws.kite.trade?api_key={api_key}&access_token={access_token}
```

**No message-based authentication** - authentication happens during connection setup.

### Authentication Failure

If authentication fails:
- Connection will be **rejected immediately**
- No WebSocket connection established
- Check api_key and access_token validity

### Token Expiry

- access_token expires daily at 6 AM IST
- WebSocket connection will be disconnected
- Must reconnect with new access_token after re-authentication

---

## Subscription Confirmation

**No explicit confirmation messages** for subscriptions.

**Subscription success** indicated by:
- Immediate binary data packets for subscribed instruments
- No error message received

**Subscription failure** indicated by:
- Error message in text format
- No data packets received

---

## Error Handling

### Common Errors

| Error | Cause | Resolution |
|-------|-------|------------|
| Connection rejected | Invalid api_key or access_token | Re-authenticate and reconnect |
| Invalid subscription | Invalid instrument token | Check instrument token from /instruments |
| Rate limit | Too many connections | Close unused connections |
| Token expired | access_token expired (6 AM) | Re-authenticate and reconnect |
| Network error | Connection lost | Implement automatic reconnection |

### Reconnection Strategy

**Best Practices**:

1. **Detect disconnection**:
   ```javascript
   ws.onclose = (event) => {
       console.log("WebSocket closed:", event.code, event.reason);
       // Implement reconnection logic
   };

   ws.onerror = (error) => {
       console.error("WebSocket error:", error);
   };
   ```

2. **Implement exponential backoff**:
   ```javascript
   let reconnectDelay = 1000;  // Start with 1 second
   const maxDelay = 60000;     // Max 60 seconds

   function reconnect() {
       setTimeout(() => {
           console.log(`Reconnecting in ${reconnectDelay}ms...`);
           connectWebSocket();
           reconnectDelay = Math.min(reconnectDelay * 2, maxDelay);
       }, reconnectDelay);
   }
   ```

3. **Re-subscribe after reconnection**:
   ```javascript
   ws.onopen = () => {
       console.log("Reconnected successfully");
       reconnectDelay = 1000;  // Reset delay

       // Re-subscribe to previous instruments
       ws.send(JSON.stringify({
           "a": "subscribe",
           "v": subscribedInstruments
       }));
   };
   ```

4. **Handle token expiry**:
   ```javascript
   ws.onclose = (event) => {
       if (event.code === 1006 || event.reason.includes("token")) {
           console.log("Token expired, re-authenticating...");
           // Trigger re-authentication flow
           reAuthenticate();
       } else {
           reconnect();
       }
   };
   ```

---

## Complete WebSocket Example

### Python Example (using official SDK)

```python
from kiteconnect import KiteTicker
import logging

logging.basicConfig(level=logging.DEBUG)

api_key = "your_api_key"
access_token = "your_access_token"

kws = KiteTicker(api_key, access_token)

def on_ticks(ws, ticks):
    """Callback for tick reception"""
    for tick in ticks:
        print(f"Token: {tick['instrument_token']}, LTP: {tick['last_price']}")

def on_connect(ws, response):
    """Callback on successful connection"""
    print("Connected successfully")

    # Subscribe to instruments
    instruments = [408065, 738561]  # INFY, RELIANCE
    ws.subscribe(instruments)

    # Set mode to full
    ws.set_mode(ws.MODE_FULL, instruments)

def on_close(ws, code, reason):
    """Callback on connection close"""
    print(f"Connection closed: {code} - {reason}")

def on_error(ws, code, reason):
    """Callback on error"""
    print(f"Error: {code} - {reason}")

def on_reconnect(ws, attempts_count):
    """Callback on reconnection attempt"""
    print(f"Reconnecting... Attempt #{attempts_count}")

def on_noreconnect(ws):
    """Callback when max reconnection attempts reached"""
    print("Max reconnection attempts reached")

def on_order_update(ws, data):
    """Callback for order postbacks"""
    print(f"Order update: {data}")

# Assign callbacks
kws.on_ticks = on_ticks
kws.on_connect = on_connect
kws.on_close = on_close
kws.on_error = on_error
kws.on_reconnect = on_reconnect
kws.on_noreconnect = on_noreconnect
kws.on_order_update = on_order_update

# Start WebSocket connection (blocking)
kws.connect(threaded=False)
```

### JavaScript Example

```javascript
const apiKey = "your_api_key";
const accessToken = "your_access_token";

let ws;
let subscribedInstruments = [408065, 738561];  // INFY, RELIANCE

function connectWebSocket() {
    ws = new WebSocket(`wss://ws.kite.trade?api_key=${apiKey}&access_token=${accessToken}`);

    ws.onopen = () => {
        console.log("Connected to Kite WebSocket");

        // Subscribe to instruments
        ws.send(JSON.stringify({
            "a": "subscribe",
            "v": subscribedInstruments
        }));

        // Set to full mode
        ws.send(JSON.stringify({
            "a": "mode",
            "v": ["full", subscribedInstruments]
        }));
    };

    ws.onmessage = (event) => {
        if (event.data instanceof ArrayBuffer || event.data instanceof Blob) {
            // Binary market data
            console.log("Binary data received:", event.data);
            // Parse using library or custom parser
        } else {
            // Text message
            const message = JSON.parse(event.data);

            if (message.type === "order") {
                console.log("Order update:", message.data);
            } else if (message.type === "error") {
                console.error("Error:", message.data);
            } else if (message.type === "message") {
                console.log("System message:", message.data);
            }
        }
    };

    ws.onerror = (error) => {
        console.error("WebSocket error:", error);
    };

    ws.onclose = (event) => {
        console.log("WebSocket closed:", event.code, event.reason);

        // Reconnect after 5 seconds
        setTimeout(connectWebSocket, 5000);
    };
}

connectWebSocket();
```

---

## Best Practices

1. **Use official SDKs**: Binary parsing is complex, use pre-built libraries

2. **Handle reconnections**: Implement automatic reconnection with exponential backoff

3. **Manage subscriptions**:
   - Don't exceed 3,000 instruments per connection
   - Unsubscribe from unused instruments
   - Use appropriate mode (ltp/quote/full) for bandwidth

4. **Mode selection**:
   - Use `ltp` for price-only monitoring (lowest bandwidth)
   - Use `quote` for standard quotes without depth
   - Use `full` only when market depth is needed

5. **Error handling**:
   - Handle token expiry (6 AM IST)
   - Detect and reconnect on network issues
   - Log all errors for debugging

6. **Order postbacks**:
   - Use WebSocket for real-time order updates
   - Reduce REST API polling
   - Handle all order statuses (OPEN, COMPLETE, REJECTED, etc.)

7. **Connection management**:
   - Close connections when not needed
   - Don't exceed 3 connections per API key
   - Reuse connections for multiple instruments

8. **Bandwidth optimization**:
   - Use ltp mode when possible
   - Limit subscriptions to actively monitored instruments
   - Unsubscribe from instruments no longer needed

9. **Thread safety** (for multi-threaded apps):
   - Handle WebSocket callbacks in separate thread
   - Use thread-safe data structures
   - Synchronize access to shared state

10. **Monitoring**:
    - Log connection events
    - Track subscription count
    - Monitor data reception rates
    - Alert on extended disconnections
