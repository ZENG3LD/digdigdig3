# Upbit WebSocket API Documentation

Comprehensive research on Upbit WebSocket API for real-time market data and account updates.

---

## 1. Connection Setup

### 1.1 WebSocket Endpoints by Region

#### Public Endpoints (Market Data)

| Region | WebSocket URL |
|--------|---------------|
| **Singapore** | `wss://sg-api.upbit.com/websocket/v1` |
| **Indonesia** | `wss://id-api.upbit.com/websocket/v1` |
| **Thailand** | `wss://th-api.upbit.com/websocket/v1` |

#### Private Endpoints (Account Data)

| Region | WebSocket URL |
|--------|---------------|
| **Singapore** | `wss://sg-api.upbit.com/websocket/v1/private` |
| **Indonesia** | `wss://id-api.upbit.com/websocket/v1/private` |
| **Thailand** | `wss://th-api.upbit.com/websocket/v1/private` |

**Note**: Replace `sg`, `id`, or `th` with your target region.

---

### 1.2 Authentication

#### Public Endpoints (No Auth)

Public WebSocket connections require **no authentication**.

**Connection**:
```javascript
const ws = new WebSocket('wss://sg-api.upbit.com/websocket/v1');
```

---

#### Private Endpoints (JWT Required)

Private WebSocket connections require **JWT authentication** via custom header.

**Header Format**:
```
Authorization: Bearer {JWT_TOKEN}
```

**Example** (Python with websocket-client):
```python
import websocket
import jwt
import uuid

# Generate JWT token
payload = {
    "access_key": access_key,
    "nonce": str(uuid.uuid4())
}
token = jwt.encode(payload, secret_key, algorithm="HS512")

# Connect with authentication header
ws = websocket.WebSocketApp(
    "wss://sg-api.upbit.com/websocket/v1/private",
    header={
        "Authorization": f"Bearer {token}"
    }
)
ws.run_forever()
```

**Important**: Some WebSocket clients do not support custom headers. Use a library that supports headers (e.g., `websocket-client` in Python, `ws` in Node.js).

---

## 2. Message Format

### 2.1 Subscription Message Structure

WebSocket messages are **JSON arrays** containing three components:

1. **Ticket Object** - Unique identifier for this connection
2. **Type Object(s)** - Data types to subscribe to
3. **Format Object** - Output format specification (optional)

**Basic Structure**:
```json
[
  {"ticket": "unique-connection-id"},
  {"type": "ticker", "codes": ["SGD-BTC", "SGD-ETH"]},
  {"format": "DEFAULT"}
]
```

---

### 2.2 Ticket Object

**Purpose**: Unique identifier for tracking this WebSocket connection

**Format**:
```json
{"ticket": "unique-connection-id"}
```

**Field**:
- `ticket` (string): Any unique string (UUID, timestamp, custom ID)

**Example**:
```json
{"ticket": "test-connection-12345"}
{"ticket": "a1b2c3d4-e5f6-7890"}
{"ticket": "user-123-session-456"}
```

---

### 2.3 Type Object

**Purpose**: Specify data type and markets to subscribe to

**Format**:
```json
{"type": "data_type", "codes": ["MARKET-1", "MARKET-2"]}
```

**Fields**:
- `type` (string): Data type to subscribe to
- `codes` (array): Array of market identifiers

**Supported Types**:
- `"ticker"` - Real-time price updates
- `"orderbook"` - Order book depth updates
- `"trade"` - Trade executions
- `"myAsset"` - Personal asset balance (private only)
- `"myOrder"` - Personal order updates (private only)

**Examples**:
```json
// Single market ticker
{"type": "ticker", "codes": ["SGD-BTC"]}

// Multiple markets orderbook
{"type": "orderbook", "codes": ["SGD-BTC", "SGD-ETH", "SGD-XRP"]}

// All personal orders (private)
{"type": "myOrder"}
```

---

### 2.4 Format Object

**Purpose**: Specify output format for received messages

**Format**:
```json
{"format": "FORMAT_TYPE"}
```

**Supported Formats**:
- `"DEFAULT"` - Full field names (e.g., `market`, `trade_price`)
- `"SIMPLE"` - Abbreviated field names (e.g., `mk`, `tp`)
- `"JSON_LIST"` - Array format with full names
- `"SIMPLE_LIST"` - Array format with abbreviated names

**Example**:
```json
[
  {"ticket": "test"},
  {"type": "ticker", "codes": ["SGD-BTC"]},
  {"format": "SIMPLE"}
]
```

**Field Name Mapping** (DEFAULT vs SIMPLE):
| DEFAULT | SIMPLE | Description |
|---------|--------|-------------|
| `market` | `mk` | Market identifier |
| `trade_price` | `tp` | Trade price |
| `trade_volume` | `tv` | Trade volume |
| `acc_trade_volume` | `atv` | Accumulated volume |
| `high_price` | `hp` | High price |
| `low_price` | `lp` | Low price |

**Note**: `SIMPLE` format reduces bandwidth for high-frequency subscriptions.

---

## 3. Public Channels

### 3.1 Ticker Channel

**Type**: `"ticker"`

**Data**: Real-time price updates

**Subscription**:
```json
[
  {"ticket": "test"},
  {"type": "ticker", "codes": ["SGD-BTC", "SGD-ETH"]}
]
```

**Response Format**:
```json
{
  "type": "ticker",
  "code": "SGD-BTC",
  "opening_price": 66000.0,
  "high_price": 68000.0,
  "low_price": 65500.0,
  "trade_price": 67300.0,
  "prev_closing_price": 66000.0,
  "change": "RISE",
  "change_price": 1300.0,
  "change_rate": 0.0197,
  "signed_change_price": 1300.0,
  "signed_change_rate": 0.0197,
  "trade_volume": 0.15,
  "acc_trade_price": 45678901.23,
  "acc_trade_price_24h": 45678901.23,
  "acc_trade_volume": 678.45,
  "acc_trade_volume_24h": 678.45,
  "highest_52_week_price": 85000.0,
  "highest_52_week_date": "2023-11-15",
  "lowest_52_week_price": 25000.0,
  "lowest_52_week_date": "2023-07-01",
  "trade_date": "20240619",
  "trade_time": "083143",
  "trade_date_kst": "20240619",
  "trade_time_kst": "173143",
  "trade_timestamp": 1718788303000,
  "timestamp": 1718788303000,
  "stream_type": "REALTIME"
}
```

**Key Fields**:
- `type`: Always `"ticker"`
- `code`: Market identifier (e.g., "SGD-BTC")
- `trade_price`: Current price
- `change`: Price direction ("RISE", "EVEN", "FALL")
- `timestamp`: Message timestamp (milliseconds)
- `stream_type`: "SNAPSHOT" (initial) or "REALTIME" (updates)

**Update Frequency**: Real-time (on every trade)

---

### 3.2 Orderbook Channel

**Type**: `"orderbook"`

**Data**: Order book depth updates

**Subscription**:
```json
[
  {"ticket": "test"},
  {"type": "orderbook", "codes": ["SGD-BTC"]},
  {"format": "DEFAULT"}
]
```

**Additional Parameter** (since v1.1.0):
```json
{
  "type": "orderbook",
  "codes": ["SGD-BTC"],
  "level": 15
}
```
- `level` (integer): Number of price levels (1-30, default: 15)

**Response Format**:
```json
{
  "type": "orderbook",
  "code": "SGD-BTC",
  "timestamp": 1718788303000,
  "total_ask_size": 123.45,
  "total_bid_size": 234.56,
  "orderbook_units": [
    {
      "ask_price": 67500.0,
      "bid_price": 67300.0,
      "ask_size": 5.23,
      "bid_size": 6.78
    },
    {
      "ask_price": 67600.0,
      "bid_price": 67200.0,
      "ask_size": 3.45,
      "bid_size": 4.12
    }
  ],
  "stream_type": "SNAPSHOT"
}
```

**Key Fields**:
- `type`: Always `"orderbook"`
- `code`: Market identifier
- `timestamp`: Snapshot timestamp (milliseconds)
- `orderbook_units`: Array of price levels (up to `level` entries)
- `total_ask_size`: Total ask volume across all levels
- `total_bid_size`: Total bid volume across all levels
- `stream_type`: "SNAPSHOT" (full orderbook) or "REALTIME" (updates)

**Update Frequency**: Real-time (on orderbook changes)

**Note**: From v1.1.0, you can adjust the number of orderbook levels (1-30) using the `level` parameter.

---

### 3.3 Trade Channel

**Type**: `"trade"`

**Data**: Trade executions

**Subscription**:
```json
[
  {"ticket": "test"},
  {"type": "trade", "codes": ["SGD-BTC", "SGD-ETH"]}
]
```

**Response Format**:
```json
{
  "type": "trade",
  "code": "SGD-BTC",
  "timestamp": 1718788303000,
  "trade_date": "2024-06-19",
  "trade_time": "08:31:43",
  "trade_timestamp": 1718788303000,
  "trade_price": 67300.0,
  "trade_volume": 0.15,
  "ask_bid": "BID",
  "prev_closing_price": 66000.0,
  "change_price": 1300.0,
  "sequential_id": 1234567890123,
  "stream_type": "REALTIME"
}
```

**Key Fields**:
- `type`: Always `"trade"`
- `code`: Market identifier
- `trade_price`: Execution price
- `trade_volume`: Execution volume
- `ask_bid`: Taker side ("ASK" = taker sold, "BID" = taker bought)
- `sequential_id`: Sequential trade ID
- `stream_type`: Always "REALTIME" for trades

**Update Frequency**: Real-time (on every trade execution)

---

## 4. Private Channels

**Requirement**: Private channels require authentication via private WebSocket endpoint with JWT token.

### 4.1 My Asset Channel

**Type**: `"myAsset"`

**Data**: Personal asset balance updates

**Subscription**:
```json
[
  {"ticket": "test"},
  {"type": "myAsset"}
]
```

**Response Format**:
```json
{
  "type": "myAsset",
  "balances": [
    {
      "currency": "SGD",
      "balance": "1000000.0",
      "locked": "0.0",
      "avg_buy_price": "0",
      "avg_buy_price_modified": false,
      "unit_currency": "SGD"
    },
    {
      "currency": "BTC",
      "balance": "2.0",
      "locked": "0.1",
      "avg_buy_price": "67000",
      "avg_buy_price_modified": false,
      "unit_currency": "SGD"
    }
  ],
  "timestamp": 1718788303000
}
```

**Key Fields**:
- `type`: Always `"myAsset"`
- `balances`: Array of balance objects (same format as REST `/v1/balances`)
- `timestamp`: Update timestamp

**Update Frequency**: Real-time (on balance changes)

---

### 4.2 My Order Channel

**Type**: `"myOrder"`

**Data**: Personal order status updates

**Subscription**:
```json
[
  {"ticket": "test"},
  {"type": "myOrder"}
]
```

**Response Format**:
```json
{
  "type": "myOrder",
  "uuid": "a1b2c3d4-e5f6-7890-abcd-ef1234567890",
  "market": "SGD-BTC",
  "side": "bid",
  "ord_type": "limit",
  "price": "67000.0",
  "state": "wait",
  "volume": "0.1",
  "remaining_volume": "0.05",
  "executed_volume": "0.05",
  "reserved_fee": "0.5",
  "remaining_fee": "0.25",
  "paid_fee": "0.25",
  "locked": "3350.25",
  "trades_count": 2,
  "created_at": "2024-06-19T08:31:43+00:00",
  "timestamp": 1718788303000
}
```

**Key Fields**:
- `type`: Always `"myOrder"`
- `uuid`: Order UUID
- `state`: Order state ("wait", "done", "cancel")
- `remaining_volume`: Unfilled volume
- `executed_volume`: Filled volume
- `timestamp`: Update timestamp

**Update Frequency**: Real-time (on order state changes: created, partially filled, fully filled, canceled)

---

## 5. Connection Maintenance

### 5.1 Ping/Pong Mechanism

**Purpose**: Keep connection alive and detect disconnections

**Client Responsibility**: Send periodic messages to prevent timeout

**Timeout**: **120 seconds** of inactivity will cause automatic disconnection

**Ping Message Options**:

1. **WebSocket PING Frame** (preferred):
   ```javascript
   // Most WebSocket libraries support ping frames
   ws.ping();
   ```

2. **Text "PING" Message**:
   ```json
   "PING"
   ```

**Server Response**:

For "PING" text message, server responds:
```json
{
  "status": "UP"
}
```

**Server Status Messages**:

Server sends status every 10 seconds while active:
```json
{
  "status": "UP"
}
```

**Recommendation**: Send PING every 30-60 seconds to maintain connection

**Example** (Python):
```python
import websocket
import time
import threading

def ping_thread(ws):
    while True:
        time.sleep(30)
        ws.send("PING")

ws = websocket.WebSocketApp("wss://sg-api.upbit.com/websocket/v1")
threading.Thread(target=ping_thread, args=(ws,), daemon=True).start()
ws.run_forever()
```

---

### 5.2 Reconnection Strategy

**Events Requiring Reconnection**:
- Connection timeout (no activity for 120 seconds)
- Network interruption
- Server maintenance
- Error messages

**Recommended Strategy**:

1. **Immediate Reconnect**: After network errors
2. **Exponential Backoff**: After repeated failures
3. **Re-subscribe**: Send subscription messages again after reconnection

**Example** (Python):
```python
import time

def on_error(ws, error):
    print(f"WebSocket error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("WebSocket closed, reconnecting...")
    time.sleep(5)
    reconnect()

def reconnect():
    ws = websocket.WebSocketApp(
        "wss://sg-api.upbit.com/websocket/v1",
        on_open=on_open,
        on_message=on_message,
        on_error=on_error,
        on_close=on_close
    )
    ws.run_forever()

def on_open(ws):
    # Re-subscribe after reconnection
    subscription = [
        {"ticket": "test"},
        {"type": "ticker", "codes": ["SGD-BTC"]}
    ]
    ws.send(json.dumps(subscription))
```

---

## 6. Compression Support

### 6.1 WebSocket Compression

**Support**: Upbit WebSocket server supports compression

**Benefit**: Reduced bandwidth usage for high-frequency data

**Enabling**: Compression is negotiated during WebSocket handshake

**Example** (Python with websocket-client):
```python
ws = websocket.WebSocketApp(
    "wss://sg-api.upbit.com/websocket/v1",
    compression="permessage-deflate"
)
```

**Example** (JavaScript):
```javascript
// Most browsers automatically negotiate compression
const ws = new WebSocket('wss://sg-api.upbit.com/websocket/v1');
```

**Note**: Compression support depends on WebSocket library. Check library documentation.

---

## 7. Rate Limits

### 7.1 Connection Limits

| Limit Type | Value | Description |
|------------|-------|-------------|
| **Connection Rate** | 5 connections/sec | Maximum new connections per second |
| **Subscription Rate** | 5 messages/sec | Maximum subscription messages per second |
| **Subscription Burst** | 100 messages/min | Maximum subscription messages per minute |

**Measurement**: Per IP address (public) or per account (private)

---

### 7.2 Best Practices

1. **Reuse Connections**: Don't create new connections for each subscription
2. **Batch Subscriptions**: Subscribe to multiple markets in one message
3. **Respect Limits**: Wait between subscription messages if subscribing to many markets

**Example** (Subscribe to 20 markets):
```python
import json
import time

# Subscribe in batches of 5 (to respect 5 msg/sec limit)
markets = ["SGD-BTC", "SGD-ETH", "SGD-XRP", ...] # 20 markets

for i in range(0, len(markets), 5):
    batch = markets[i:i+5]
    subscription = [
        {"ticket": f"batch-{i}"},
        {"type": "ticker", "codes": batch}
    ]
    ws.send(json.dumps(subscription))
    time.sleep(1)  # Wait 1 second between batches
```

---

## 8. Important Details

### 8.1 Multiple Subscriptions

**Single Connection**: You can subscribe to multiple data types and markets on one connection

**Example**:
```json
[
  {"ticket": "test"},
  {"type": "ticker", "codes": ["SGD-BTC", "SGD-ETH"]},
  {"type": "orderbook", "codes": ["SGD-BTC"]},
  {"type": "trade", "codes": ["SGD-BTC", "SGD-ETH", "SGD-XRP"]}
]
```

---

### 8.2 Unsubscribe

**Not Supported**: Upbit WebSocket does not support unsubscribing from specific channels

**Workaround**: Close connection and create new one with desired subscriptions

---

### 8.3 Historical Data

**Not Available**: WebSocket only provides real-time data, not historical data

**For Historical**: Use REST API endpoints (`/v1/candles/*`, `/v1/trades/recent`)

---

### 8.4 Message Order

**Not Guaranteed**: Messages may arrive out of order during high-frequency trading

**Recommendation**: Use `timestamp` or `sequential_id` fields to order messages client-side

---

## 9. Example Implementations

### 9.1 Python (websocket-client)

```python
import websocket
import json

def on_message(ws, message):
    data = json.loads(message)
    print(f"Received: {data.get('type')} for {data.get('code')}")

def on_error(ws, error):
    print(f"Error: {error}")

def on_close(ws, close_status_code, close_msg):
    print("Connection closed")

def on_open(ws):
    subscription = [
        {"ticket": "test"},
        {"type": "ticker", "codes": ["SGD-BTC", "SGD-ETH"]},
        {"type": "orderbook", "codes": ["SGD-BTC"], "level": 15},
        {"type": "trade", "codes": ["SGD-BTC"]}
    ]
    ws.send(json.dumps(subscription))

ws = websocket.WebSocketApp(
    "wss://sg-api.upbit.com/websocket/v1",
    on_open=on_open,
    on_message=on_message,
    on_error=on_error,
    on_close=on_close
)

ws.run_forever()
```

---

### 9.2 JavaScript (Browser)

```javascript
const ws = new WebSocket('wss://sg-api.upbit.com/websocket/v1');

ws.onopen = () => {
    const subscription = [
        {ticket: 'test'},
        {type: 'ticker', codes: ['SGD-BTC', 'SGD-ETH']},
        {type: 'trade', codes: ['SGD-BTC']}
    ];
    ws.send(JSON.stringify(subscription));
};

ws.onmessage = (event) => {
    const data = JSON.parse(event.data);
    console.log('Received:', data.type, 'for', data.code);
};

ws.onerror = (error) => {
    console.error('WebSocket error:', error);
};

ws.onclose = () => {
    console.log('Connection closed');
};
```

---

### 9.3 Rust (tokio-tungstenite)

```rust
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

#[tokio::main]
async fn main() {
    let (ws_stream, _) = connect_async("wss://sg-api.upbit.com/websocket/v1")
        .await
        .expect("Failed to connect");

    let (mut write, mut read) = ws_stream.split();

    // Send subscription
    let subscription = json!([
        {"ticket": "test"},
        {"type": "ticker", "codes": ["SGD-BTC", "SGD-ETH"]},
        {"type": "trade", "codes": ["SGD-BTC"]}
    ]);
    write.send(Message::Text(subscription.to_string()))
        .await
        .expect("Failed to send subscription");

    // Read messages
    while let Some(message) = read.next().await {
        match message {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
            }
            Ok(Message::Binary(bin)) => {
                let text = String::from_utf8_lossy(&bin);
                println!("Received binary: {}", text);
            }
            Err(e) => eprintln!("Error: {}", e),
            _ => {}
        }
    }
}
```

---

## 10. Summary

### Key Takeaways

1. **Regional Endpoints**: Use region-specific WebSocket URLs (sg, id, th)
2. **Authentication**: Private channels require JWT in Authorization header
3. **Message Format**: JSON array with ticket, type, and format objects
4. **Subscription Types**: ticker, orderbook, trade (public); myAsset, myOrder (private)
5. **Connection Timeout**: 120 seconds inactivity triggers disconnect
6. **Ping Mechanism**: Send PING every 30-60 seconds to keep connection alive
7. **Rate Limits**: 5 connections/sec, 5 messages/sec, 100 messages/min
8. **Compression**: Supported for bandwidth optimization
9. **No Unsubscribe**: Close and reconnect to change subscriptions
10. **Real-Time Only**: Historical data not available via WebSocket

### WebSocket vs REST

| Feature | WebSocket | REST |
|---------|-----------|------|
| **Real-time updates** | Yes | No (polling) |
| **Historical data** | No | Yes |
| **Rate limits** | 5 msg/sec | 10 req/sec (public), 30 req/sec (private) |
| **Connection overhead** | Low (persistent) | High (per request) |
| **Use case** | Live monitoring | Data retrieval, trading |

---

## Sources

- [Upbit Open API - WebSocket Guide](https://global-docs.upbit.com/reference/websocket-guide)
- [Upbit Open API - WebSocket Orderbook](https://global-docs.upbit.com/v1.2.2/reference/websocket-orderbook)
- [Upbit Open API - Rate Limits](https://global-docs.upbit.com/reference/rate-limits)
- [Tardis.dev - Upbit WebSocket Documentation](https://docs.tardis.dev/historical-data-details/upbit)
- [GitHub - Upbit Client Examples](https://github.com/upbit-exchange/client)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
