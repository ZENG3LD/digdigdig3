# Dhan - WebSocket Documentation

## Availability: Yes

## Connection

### URLs
- **Live Market Feed (Real-time Quotes)**: wss://api-feed.dhan.co
- **20-Level Market Depth**: wss://depth-api-feed.dhan.co/twentydepth?token={JWT}&clientId={clientId}&authType=2
- **200-Level Market Depth**: wss://full-depth-api.dhan.co/twohundreddepth?token={JWT}&clientId={clientId}&authType=2

### Connection Process

#### Live Market Feed
1. Connect to `wss://api-feed.dhan.co`
2. No handshake message - connection established immediately
3. Authentication via query parameters OR initial message
4. Send subscription message in JSON format
5. Receive data in binary format (Little Endian)

#### Market Depth WebSockets
1. Connect to depth endpoint with authentication in URL
2. Query parameters:
   - `token`: Access token (JWT)
   - `clientId`: Dhan client ID
   - `authType`: 2 (for API-based auth)
3. Send subscription in JSON format
4. Receive binary packets

### Authentication
- **Method 1**: Query parameters in WebSocket URL
  ```
  wss://api-feed.dhan.co?token=eyJhbGc...&version=2
  ```
- **Method 2**: Via subscription message (for main feed)
- Access token must be valid (24-hour validity)

## ALL Available Channels/Topics

### Live Market Feed Channels

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Example Subscription |
|---------------|------|-------------|-------|-------|------------------|---------------------|
| Ticker | Public | LTP updates only | Yes | Conditional | Real-time | {"RequestCode":15,"InstrumentCount":1,"InstrumentList":[{"ExchangeSegment":1,"SecurityId":"1333"}]} |
| Quote | Public | LTP + OHLC + Volume + OI | Yes | Conditional | Real-time | {"RequestCode":17,"InstrumentCount":1,"InstrumentList":[{"ExchangeSegment":1,"SecurityId":"1333"}]} |
| Market Depth | Public | 5-level orderbook | Yes | Conditional | Real-time | {"RequestCode":21,"InstrumentCount":1,"InstrumentList":[{"ExchangeSegment":1,"SecurityId":"1333"}]} |
| Full Packet | Public | All data (LTP+Quote+OI+Depth) | Yes | Conditional | Real-time | {"RequestCode":19,"InstrumentCount":1,"InstrumentList":[{"ExchangeSegment":1,"SecurityId":"1333"}]} |
| OI | Public | Open Interest | Yes | Conditional | Real-time | Included in Quote/Full packet |
| 20 Depth | Public | 20-level orderbook | Yes | Conditional | Real-time | Separate WebSocket endpoint |
| 200 Depth | Public | 200-level orderbook | Yes | Conditional | Real-time | Separate WebSocket endpoint |
| Order Updates | Private | Live order status | Yes | Yes | Real-time | {"RequestCode":5,"ClientId":"1000000123"} |

**Note**: "Conditional" free means free if 25+ trades in previous 30 days, otherwise Rs. 499/month

### Request Codes

| Code | Channel | Description |
|------|---------|-------------|
| 4 | Subscribe | Subscribe to market feed |
| 5 | Order Feed | Subscribe to live order updates |
| 6 | Unsubscribe | Unsubscribe from instruments |
| 15 | Ticker | LTP only |
| 17 | Quote | LTP + Quote + OI |
| 19 | Full Packet | Complete data (Ticker + Quote + OI + Depth) |
| 21 | Market Depth | 5-level bid/ask |

## Subscription Format

### Subscribe Message (Ticker - LTP Only)
```json
{
  "RequestCode": 15,
  "InstrumentCount": 2,
  "InstrumentList": [
    {
      "ExchangeSegment": 1,
      "SecurityId": "1333"
    },
    {
      "ExchangeSegment": 2,
      "SecurityId": "52175"
    }
  ]
}
```

### Subscribe Message (Quote - Full Quote Data)
```json
{
  "RequestCode": 17,
  "InstrumentCount": 1,
  "InstrumentList": [
    {
      "ExchangeSegment": 1,
      "SecurityId": "11536"
    }
  ]
}
```

### Subscribe Message (Full Packet - Everything)
```json
{
  "RequestCode": 19,
  "InstrumentCount": 1,
  "InstrumentList": [
    {
      "ExchangeSegment": 1,
      "SecurityId": "1333"
    }
  ]
}
```

### Subscribe Message (Market Depth - 5 Levels)
```json
{
  "RequestCode": 21,
  "InstrumentCount": 1,
  "InstrumentList": [
    {
      "ExchangeSegment": 1,
      "SecurityId": "1333"
    }
  ]
}
```

### Unsubscribe Message
```json
{
  "RequestCode": 6,
  "InstrumentCount": 1,
  "InstrumentList": [
    {
      "ExchangeSegment": 1,
      "SecurityId": "1333"
    }
  ]
}
```

### Order Updates Subscription
```json
{
  "RequestCode": 5,
  "ClientId": "1000000123"
}
```

### Exchange Segment Codes
| Code | Segment |
|------|---------|
| 1 | NSE_EQ (NSE Cash) |
| 2 | NSE_FNO (NSE F&O) |
| 3 | BSE_EQ (BSE Cash) |
| 4 | MCX_COMM (MCX Commodities) |

### Subscription Confirmation
**No explicit confirmation message**. Server starts sending binary data immediately after subscription.

## Message Formats (Binary - CRITICAL)

**IMPORTANT**: All request messages are JSON, but ALL response messages are BINARY (Little Endian).

### Binary Packet Structure

Each packet type has a fixed structure. Data must be unpacked using Little Endian byte order.

### Ticker Packet (52 bytes)
Binary structure for LTP data:
```
Byte 0: Packet Type (uint8)
Bytes 1-4: Exchange Segment (uint32, LE)
Bytes 5-12: Security ID (uint64, LE)
Bytes 13-20: LTP (double, LE)
Bytes 21-28: LTT (Last Traded Time, uint64, LE)
Bytes 29-36: LTQ (Last Traded Quantity, uint64, LE)
Bytes 37-44: Volume (uint64, LE)
Bytes 45-52: Best Bid Price (double, LE)
... (additional fields)
```

### Quote Packet (180 bytes approx)
Includes all fields from Ticker plus:
- Open, High, Low, Close prices
- Previous Close
- Open Interest (for derivatives)
- Total Buy/Sell Quantities
- Average Trade Price

### Market Depth Packet (Variable size)
Contains:
- 5 levels of bids (price, quantity, orders)
- 5 levels of asks (price, quantity, orders)

Each level is 16 bytes:
- Bytes 0-7: Price (double, LE)
- Bytes 8-11: Quantity (uint32, LE)
- Bytes 12-15: Order count (uint32, LE)

### Full Packet
Combination of Ticker + Quote + Market Depth data

### Order Update Packet
Real-time order status updates (binary format, structure not publicly documented)

### 20-Level Depth Packet (Separate WebSocket)
Each bid/ask packet is 16 bytes:
```
struct Level {
  price: f64,      // 8 bytes, LE
  quantity: u32,   // 4 bytes, LE
  orders: u32      // 4 bytes, LE
}
```

Bid and Ask packets sent separately, each containing multiple 16-byte levels.

### 200-Level Depth Packet (Separate WebSocket)
Similar structure to 20-level, but 200 levels:
- Bid packets: Up to 200 levels × 16 bytes
- Ask packets: Up to 200 levels × 16 bytes
- Sent as separate packets for bids and asks

## Heartbeat / Ping-Pong

### Who initiates?
- **Server → Client ping**: No
- **Client → Server ping**: No (connection is persistent, no explicit ping required)

### Connection Management
- **No explicit ping/pong**: WebSocket connection remains open
- **Timeout Detection**: Monitor incoming data
- **Reconnection**: Client responsibility to detect disconnection and reconnect
- **Idle Handling**: Connection stays alive as long as subscriptions exist

### Recommended Client-Side Heartbeat
While not required by server, clients should:
- Monitor last received packet timestamp
- Reconnect if no data received for >30 seconds
- Implement exponential backoff for reconnections

## Connection Limits

### Live Market Feed
- **Max connections per user**: 5
- **Max subscriptions per connection**: 5,000 instruments
- **Total instruments**: Up to 25,000 (5 connections × 5,000 instruments)
- **Message rate limit**: No explicit limit (server controls push rate)

### 20-Level Market Depth
- **Max connections**: 5 per user
- **Max subscriptions per connection**: 50 instruments
- **Supported segments**: NSE Equity and NSE Derivatives only

### 200-Level Market Depth
- **Max connections**: Multiple allowed
- **Max subscriptions per connection**: 1 instrument only
- **Supported segments**: NSE Equity and NSE Derivatives only

### General Limits
- **Auto-disconnect after**: No automatic disconnect
- **Idle timeout**: None
- **Rate limiting**: Controlled by server (client cannot exceed server push rate)

## Authentication (All Channels)

### Method 1: Query Parameters (Depth WebSockets)
```
wss://depth-api-feed.dhan.co/twentydepth?token=eyJhbGc...&clientId=1000000123&authType=2
```

### Method 2: JSON Message After Connect (Main Feed)
```json
{
  "RequestCode": 4,
  "ClientId": "1000000123",
  "Token": "eyJhbGc..."
}
```

### Auth Parameters
- **token**: Access token (JWT) - 24-hour validity
- **clientId**: Dhan client ID (string)
- **authType**: 2 (for API-based authentication)
- **version**: 2 (optional, for v2 feed)

### Auth Success/Failure
**No explicit auth response message**.
- **Success**: Server starts accepting subscriptions and sending data
- **Failure**: Connection closes immediately

### Token Expiry Handling
- Access tokens expire after 24 hours
- Client must generate new token and reconnect
- No in-session token refresh mechanism

## Binary Data Parsing

### Endianness: Little Endian
- All multi-byte values are Little Endian
- If your system is Big Endian, you must convert

### Data Types Used
- `uint8` - 1 byte unsigned integer
- `uint32` - 4 bytes unsigned integer (LE)
- `uint64` - 8 bytes unsigned integer (LE)
- `double` - 8 bytes IEEE 754 floating point (LE)

### Example Python Parsing (Ticker Packet)
```python
import struct

def parse_ticker_packet(data):
    # Unpack binary data (Little Endian format)
    packet_type = struct.unpack('<B', data[0:1])[0]
    exchange_segment = struct.unpack('<I', data[1:5])[0]
    security_id = struct.unpack('<Q', data[5:13])[0]
    ltp = struct.unpack('<d', data[13:21])[0]
    ltt = struct.unpack('<Q', data[21:29])[0]
    ltq = struct.unpack('<Q', data[29:37])[0]
    volume = struct.unpack('<Q', data[37:45])[0]

    return {
        'packet_type': packet_type,
        'exchange_segment': exchange_segment,
        'security_id': security_id,
        'ltp': ltp,
        'last_traded_time': ltt,
        'last_traded_qty': ltq,
        'volume': volume
    }
```

### Example Rust Parsing (Ticker Packet)
```rust
use byteorder::{LittleEndian, ReadBytesExt};
use std::io::Cursor;

struct TickerPacket {
    packet_type: u8,
    exchange_segment: u32,
    security_id: u64,
    ltp: f64,
    last_traded_time: u64,
    last_traded_qty: u64,
    volume: u64,
}

fn parse_ticker(data: &[u8]) -> Result<TickerPacket, std::io::Error> {
    let mut cursor = Cursor::new(data);

    Ok(TickerPacket {
        packet_type: cursor.read_u8()?,
        exchange_segment: cursor.read_u32::<LittleEndian>()?,
        security_id: cursor.read_u64::<LittleEndian>()?,
        ltp: cursor.read_f64::<LittleEndian>()?,
        last_traded_time: cursor.read_u64::<LittleEndian>()?,
        last_traded_qty: cursor.read_u64::<LittleEndian>()?,
        volume: cursor.read_u64::<LittleEndian>()?,
    })
}
```

## Error Handling

### Connection Errors
- **Invalid token**: Connection closes immediately
- **Expired token**: Connection closes
- **Invalid subscription**: No error returned, simply no data received

### Best Practices
1. Always check for connection close events
2. Implement automatic reconnection with exponential backoff
3. Re-subscribe to all instruments after reconnection
4. Validate binary packet sizes before parsing
5. Handle partial packets (buffer incomplete data)
6. Monitor for stale data (timestamp checking)

## WebSocket Libraries Recommended
- **Python**: `websocket-client`, `websockets` (async)
- **JavaScript/Node.js**: Native `WebSocket`, `ws` library
- **Rust**: `tokio-tungstenite`, `async-tungstenite`

Binary parsing libraries:
- **Python**: `struct` (built-in)
- **Rust**: `byteorder` crate
- **JavaScript**: `DataView`, `Buffer`

## Data Subscription Strategy

### Efficient Subscription
- Group instruments by exchange segment
- Use Full Packet (RequestCode 19) for comprehensive data
- Use Ticker (RequestCode 15) for LTP-only needs (lighter bandwidth)

### Connection Management
- Distribute instruments across multiple connections
- Keep per-connection instrument count < 5000
- Monitor connection health on all connections
- Implement connection pooling

### Bandwidth Optimization
- Subscribe to Ticker instead of Full Packet when only LTP needed
- Unsubscribe from instruments not actively monitored
- Use separate connections for different data priorities

## Official SDK Support

### Python SDK (DhanHQ-py)
```python
from dhanhq import marketfeed

# Initialize
instruments = [(1, "1333"), (2, "52175")]  # (segment, security_id)
feed = marketfeed.DhanFeed("client_id", "access_token", instruments)

# Set callbacks
@feed.on_connect
def connected():
    print("Connected")

@feed.on_message
def message_received(data):
    print(f"LTP: {data['LTP']}")

# Subscribe to ticker
feed.subscribe_ticker()

# Run
feed.run_forever()
```

The SDK handles binary parsing automatically.

## Rate Limit Considerations for WebSocket

While WebSocket itself has connection limits (5 connections, 5000 instruments each), there are no explicit rate limits on:
- Subscription messages
- Unsubscription messages
- Data receive rate (server-controlled)

However, REST API rate limits still apply:
- Generating access tokens: 20/sec
- Any REST calls while maintaining WebSocket: Subject to REST limits
