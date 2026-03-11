# Coinbase Advanced Trade API WebSocket Documentation

Comprehensive research on Coinbase Advanced Trade WebSocket API for Spot trading.

---

## 1. Connection Setup

### 1.1 WebSocket Endpoint URLs

**Market Data (Public):**
```
wss://advanced-trade-ws.coinbase.com
```

**User Order Data (Private):**
```
wss://advanced-trade-ws-user.coinbase.com
```

**Failover Strategy**: If `advanced-trade-ws-user` is your primary connection, use `advanced-trade-ws` as failover.

### 1.2 Connection Flow

1. **Open WebSocket Connection** to endpoint URL
2. **Receive Welcome Message** (no explicit welcome in docs, but connection established)
3. **Send Subscribe Message** within **5 seconds** or connection will be closed
4. **Receive Data** on subscribed channels
5. **Maintain Connection** (no explicit ping/pong documented)

**Critical**: You **MUST** send a subscribe message within 5 seconds of connection, or the connection will be automatically closed.

### 1.3 No Token Endpoint Required

Unlike KuCoin which requires calling `/api/v1/bullet-public` or `/api/v1/bullet-private` to get a token first, Coinbase WebSocket:

- **Direct connection** to WebSocket URL
- **No pre-connection REST call** needed
- **JWT included in subscribe message** for private channels

---

## 2. Message Format

### 2.1 Subscribe Message

**Without Authentication (Public Channels):**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD", "ETH-USD"],
  "channel": "level2"
}
```

**With Authentication (Private Channels):**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD", "ETH-USD"],
  "channel": "user",
  "jwt": "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Im9yZ2FuaXphdGlvbnMve29yZ19pZH0vYXBpS2V5cy97a2V5X2lkfSIsIm5vbmNlIjoiYzU3ZTM5YjE4MTU5NDRkYjk5ODk4ZTUzMDg1YjhkYTIifQ..."
}
```

**Fields:**
- `type` (string): `"subscribe"` (REQUIRED)
- `product_ids` (array): List of product IDs to subscribe to (REQUIRED)
- `channel` (string): Channel name (REQUIRED)
- `jwt` (string): JWT token for authentication (REQUIRED for private channels)

### 2.2 Unsubscribe Message

```json
{
  "type": "unsubscribe",
  "product_ids": ["BTC-USD"],
  "channel": "level2"
}
```

**Fields**: Same as subscribe message, with `type: "unsubscribe"`

### 2.3 Server Response Message Format

**General Format:**
```json
{
  "channel": "level2",
  "client_id": "",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "product_id": "BTC-USD",
      // ... channel-specific data
    }
  ]
}
```

**Common Fields:**
- `channel` (string): Channel name
- `client_id` (string): Client identifier (optional)
- `timestamp` (string): Message timestamp (RFC3339)
- `sequence_num` (integer): Sequence number for ordering
- `events` (array): Array of event objects

### 2.4 Sequence Numbers

**Purpose**: Ensure message ordering and detect dropped messages

**Usage**:
1. Each message contains `sequence_num`
2. Sequence increments by 1 for each message
3. If `sequence_num` > previous + 1, a message was dropped
4. Re-subscribe or fetch snapshot to recover

**Quote from docs**: "Sequence numbers that are greater than one integer value from the previous number indicate that a message has been dropped."

---

## 3. Available Channels

### 3.1 Public Channels

#### Level2 (Order Book)

**Channel Name**: `"level2"`

**Subscribe:**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "level2"
}
```

**Response:**
```json
{
  "channel": "level2",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "product_id": "BTC-USD",
      "updates": [
        {
          "side": "bid",
          "event_time": "2023-10-26T10:05:30.123456Z",
          "price_level": "50000.00",
          "new_quantity": "1.5"
        },
        {
          "side": "ask",
          "event_time": "2023-10-26T10:05:30.123456Z",
          "price_level": "50001.00",
          "new_quantity": "0"
        }
      ]
    }
  ]
}
```

**Update Fields:**
- `side` (string): "bid" or "ask"
- `event_time` (string): Update time (RFC3339)
- `price_level` (string): Price level
- `new_quantity` (string): New quantity at price (0 = remove level)

**Initial Snapshot**: First message after subscribe contains full orderbook snapshot

**Incremental Updates**: Subsequent messages are incremental updates

---

#### Ticker (Price Updates)

**Channel Name**: `"ticker"`

**Subscribe:**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "ticker"
}
```

**Response:**
```json
{
  "channel": "ticker",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "product_id": "BTC-USD",
      "price": "50000.00",
      "volume_24_h": "1234.56",
      "low_24_h": "49000.00",
      "high_24_h": "51000.00",
      "price_percent_chg_24_h": "0.02"
    }
  ]
}
```

**Event Fields:**
- `price` (string): Current price
- `volume_24_h` (string): 24h volume
- `low_24_h` (string): 24h low
- `high_24_h` (string): 24h high
- `price_percent_chg_24_h` (string): 24h price change percentage (decimal)

---

#### Ticker Batch (Multiple Products)

**Channel Name**: `"ticker_batch"`

**Subscribe:**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD", "ETH-USD"],
  "channel": "ticker_batch"
}
```

**Response**: Same as ticker, but events array contains updates for multiple products

---

#### Candles (OHLC Data)

**Channel Name**: `"candles"`

**Subscribe (with granularity):**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "candles",
  "granularity": "ONE_MINUTE"
}
```

**Supported Granularities**:
- `ONE_MINUTE`
- `FIVE_MINUTE`
- `FIFTEEN_MINUTE`
- `THIRTY_MINUTE`
- `ONE_HOUR`
- `TWO_HOUR`
- `SIX_HOUR`
- `ONE_DAY`

**Response:**
```json
{
  "channel": "candles",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "product_id": "BTC-USD",
      "candles": [
        {
          "start": "1698315900",
          "high": "50100.00",
          "low": "50000.00",
          "open": "50050.00",
          "close": "50080.00",
          "volume": "123.45"
        }
      ]
    }
  ]
}
```

**Candle Fields:**
- `start` (string): Candle start time (Unix seconds)
- `high` (string): High price
- `low` (string): Low price
- `open` (string): Open price
- `close` (string): Close price
- `volume` (string): Volume

---

#### Market Trades (Executions)

**Channel Name**: `"market_trades"`

**Subscribe:**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "market_trades"
}
```

**Response:**
```json
{
  "channel": "market_trades",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "product_id": "BTC-USD",
      "trades": [
        {
          "trade_id": "12345678",
          "side": "BUY",
          "size": "0.01",
          "price": "50000.00",
          "time": "2023-10-26T10:05:30.123456Z"
        }
      ]
    }
  ]
}
```

**Trade Fields:**
- `trade_id` (string): Trade ID
- `side` (string): "BUY" or "SELL" (taker side)
- `size` (string): Trade size
- `price` (string): Trade price
- `time` (string): Trade time (RFC3339)

---

#### Status (System Status)

**Channel Name**: `"status"`

**Subscribe:**
```json
{
  "type": "subscribe",
  "product_ids": [],
  "channel": "status"
}
```

**Response:**
```json
{
  "channel": "status",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "products": [
        {
          "product_id": "BTC-USD",
          "status": "online",
          "status_message": ""
        }
      ]
    }
  ]
}
```

---

### 3.2 Private Channels

#### User (Order Updates)

**Channel Name**: `"user"`

**Subscribe (requires JWT):**
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "user",
  "jwt": "eyJhbGci..."
}
```

**Response:**
```json
{
  "channel": "user",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "type": "update",
      "orders": [
        {
          "order_id": "11111-00000-000000",
          "client_order_id": "0000-00000-000000",
          "product_id": "BTC-USD",
          "side": "BUY",
          "order_type": "LIMIT",
          "status": "OPEN",
          "creation_time": "2023-10-26T10:05:00Z",
          "filled_size": "0.005",
          "average_filled_price": "49950.00",
          "limit_price": "50000.00",
          "total_fees": "0.50"
        }
      ]
    }
  ]
}
```

**Order Event Types**:
- Order opened
- Order matched (partial fill)
- Order filled (complete)
- Order cancelled
- Order updated

**Order Fields**: Same as REST API order response

---

#### Heartbeats (User Endpoint Only)

**Channel Name**: `"heartbeats"`

**Available on**: `wss://advanced-trade-ws-user.coinbase.com` only

**Subscribe:**
```json
{
  "type": "subscribe",
  "product_ids": [],
  "channel": "heartbeats",
  "jwt": "eyJhbGci..."
}
```

**Response:**
```json
{
  "channel": "heartbeats",
  "timestamp": "2023-10-26T10:05:30.123456Z",
  "sequence_num": 12345,
  "events": [
    {
      "current_time": "2023-10-26T10:05:30.123456Z",
      "heartbeat_counter": 42
    }
  ]
}
```

**Purpose**: Keep-alive mechanism to detect connection issues

---

## 4. Authentication

### 4.1 JWT Generation for WebSocket

**Important**: Must generate a **different JWT for each WebSocket message** sent, since JWTs expire after 2 minutes.

**JWT Payload for WebSocket:**
```json
{
  "sub": "organizations/{org_id}/apiKeys/{key_id}",
  "iss": "cdp",
  "nbf": 1706986630,
  "exp": 1706986750,
  "uri": "GET advanced-trade-ws.coinbase.com"
}
```

**Key Difference from REST JWT**:
- **URI field**: `"GET advanced-trade-ws.coinbase.com"` (not REST path)
- For user endpoint: `"GET advanced-trade-ws-user.coinbase.com"`

**Header**: Same as REST (ES256, kid, nonce)

### 4.2 Subscribe with JWT

```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "user",
  "jwt": "eyJhbGciOiJFUzI1NiIsInR5cCI6IkpXVCIsImtpZCI6Im9yZ2FuaXphdGlvbnMve29yZ19pZH0vYXBpS2V5cy97a2V5X2lkfSIsIm5vbmNlIjoiYzU3ZTM5YjE4MTU5NDRkYjk5ODk4ZTUzMDg1YjhkYTIifQ.eyJzdWIiOiJvcmdhbml6YXRpb25zL3tvcmdfaWR9L2FwaUtleXMve2tleV9pZH0iLCJpc3MiOiJjZHAiLCJuYmYiOjE3MDY5ODY2MzAsImV4cCI6MTcwNjk4Njc1MCwidXJpIjoiR0VUIGFkdmFuY2VkLXRyYWRlLXdzLmNvaW5iYXNlLmNvbSJ9.signature"
}
```

### 4.3 JWT Expiration

- **Validity**: 2 minutes
- **Recommendation**: Generate new JWT for each subscribe/unsubscribe message
- **No continuous re-auth**: Once subscribed, connection stays authenticated until closed

---

## 5. Connection Management

### 5.1 Subscription Deadline

**Requirement**: Send subscribe message within **5 seconds** of connection

**Consequence**: Connection automatically closed if no subscribe received

**Implementation**:
```rust
async fn connect_and_subscribe() -> Result<()> {
    let ws = connect("wss://advanced-trade-ws.coinbase.com").await?;

    // MUST subscribe within 5 seconds
    let subscribe_msg = json!({
        "type": "subscribe",
        "product_ids": ["BTC-USD"],
        "channel": "level2"
    });

    ws.send(subscribe_msg.to_string()).await?;

    // Now listen for messages
    while let Some(msg) = ws.next().await {
        // Process message
    }

    Ok(())
}
```

### 5.2 Message Handling

**Quote from docs**: "New message types can be added at any time. Clients are expected to ignore messages they do not support."

**Implementation**: Use permissive parsing that ignores unknown fields

### 5.3 Reconnection Strategy

**When to reconnect**:
- Connection lost
- Missed sequence numbers (gap > 1)
- No messages received for extended period

**Reconnection flow**:
1. Close existing connection
2. Open new connection
3. Subscribe to channels within 5 seconds
4. Receive snapshot
5. Resume normal operation

**Exponential backoff**: Use exponential backoff for reconnection attempts

---

## 6. Orderbook Synchronization

### 6.1 Initial Snapshot

**First message** after subscribing to `level2` contains full orderbook snapshot with all price levels.

**Snapshot Structure**:
```json
{
  "channel": "level2",
  "events": [
    {
      "type": "snapshot",
      "product_id": "BTC-USD",
      "updates": [
        {"side": "bid", "price_level": "50000.00", "new_quantity": "1.5"},
        {"side": "bid", "price_level": "49999.00", "new_quantity": "2.0"},
        {"side": "ask", "price_level": "50001.00", "new_quantity": "1.0"}
      ]
    }
  ]
}
```

### 6.2 Incremental Updates

**Subsequent messages** are incremental updates:

```json
{
  "channel": "level2",
  "sequence_num": 12346,
  "events": [
    {
      "type": "update",
      "product_id": "BTC-USD",
      "updates": [
        {"side": "bid", "price_level": "50000.00", "new_quantity": "2.0"}
      ]
    }
  ]
}
```

**Update Semantics**:
- `new_quantity > 0`: Set price level to new quantity
- `new_quantity = 0`: Remove price level

### 6.3 Sequence Number Validation

**Algorithm**:
1. Store last `sequence_num`
2. For each new message:
   - If `sequence_num == last + 1`: Apply update
   - If `sequence_num > last + 1`: Message(s) dropped, re-subscribe
   - If `sequence_num <= last`: Ignore (duplicate/old)
3. Update `last = sequence_num`

**Recovery**: If gap detected, re-subscribe to get fresh snapshot

---

## 7. Rate Limits

### 7.1 Connection Rate

**Limit**: 750 connections per second per IP

**Notes**:
- Applies to new connection attempts
- Use persistent connections
- Implement reconnection delay (exponential backoff)

### 7.2 Message Rate

**Unauthenticated**: 8 messages per second per IP

**Authenticated**: No documented per-message limit after subscription

**Subscribe Deadline**: 5 seconds to send subscribe message

---

## 8. Differences from KuCoin WebSocket

| Feature | KuCoin | Coinbase |
|---------|--------|----------|
| **Token Endpoint** | `/api/v1/bullet-public` or `bullet-private` | Not needed |
| **Connection URL** | From token response | Fixed (`advanced-trade-ws.coinbase.com`) |
| **Welcome Message** | Explicit welcome with `sessionId` | No documented welcome |
| **Ping/Pong** | Required (client sends ping) | Not documented (heartbeats channel available) |
| **Subscription Format** | Topic strings (`/market/ticker:BTC-USDT`) | Channel + product_ids |
| **Authentication** | Token in URL query param | JWT in subscribe message |
| **Sequence Numbers** | Per-message `sequenceStart`/`sequenceEnd` | Per-message `sequence_num` |
| **Initial Snapshot** | Manual REST call + calibration | Automatic on subscribe |
| **Timestamp Format** | Nanoseconds | RFC3339 (microseconds) |

---

## 9. Implementation Checklist

### 9.1 Required Components

- [ ] WebSocket client library
- [ ] JWT generation for WebSocket (uri: "GET advanced-trade-ws.coinbase.com")
- [ ] Subscribe within 5 seconds of connection
- [ ] Handle snapshot messages (type: "snapshot")
- [ ] Handle update messages (type: "update")
- [ ] Track sequence numbers for gap detection
- [ ] Reconnection with exponential backoff
- [ ] Parse RFC3339 timestamps

### 9.2 Optional Features

- [ ] Heartbeats channel subscription (user endpoint)
- [ ] Multi-channel subscriptions
- [ ] Orderbook synchronization with sequence validation
- [ ] Automatic re-subscribe on gap detection
- [ ] Failover between ws and ws-user endpoints

---

## 10. Summary

### Key Takeaways

1. **No Token Endpoint**: Direct WebSocket connection (unlike KuCoin)
2. **5-Second Deadline**: Must subscribe within 5 seconds
3. **JWT in Subscribe**: Authentication via JWT in subscribe message
4. **Automatic Snapshot**: First message is full snapshot
5. **Sequence Numbers**: Single `sequence_num` per message (not start/end)
6. **RFC3339 Timestamps**: Not milliseconds or nanoseconds
7. **Two Endpoints**: Market data vs user data (with failover)

### Official Documentation

- [WebSocket Overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-overview)
- [WebSocket Authentication](https://docs.cdp.coinbase.com/advanced-trade/docs/ws-auth)
- [WebSocket Rate Limits](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-rate-limits)

---

## Sources

- [Advanced Trade WebSocket Overview](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-overview)
- [Advanced Trade WebSocket Authentication](https://docs.cdp.coinbase.com/advanced-trade/docs/ws-auth)
- [Advanced Trade WebSocket Rate Limits](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-rate-limits)
- [Coinbase Advanced Python SDK](https://github.com/coinbase/coinbase-advanced-py)
- [Coinbase API Cheat Sheet](https://vezgo.com/blog/coinbase-api-cheat-sheet-for-developers/)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Research Completed By:** Claude Code (Sonnet 4.5)
