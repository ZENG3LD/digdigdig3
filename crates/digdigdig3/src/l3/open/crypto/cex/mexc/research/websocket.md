# MEXC WebSocket API

## Overview

MEXC provides WebSocket APIs for real-time market data and user account updates. WebSocket is recommended over REST for high-frequency data to avoid rate limits.

---

## Connection URLs

### Spot WebSocket
```
wss://wbs.mexc.com/ws
```

### Futures WebSocket
```
wss://contract.mexc.com/edge
```

### User Data Streams (Spot)
```
wss://wbs.mexc.com/ws?listenKey={listenKey}
```

---

## Connection Limits

### General Limits

**Spot WebSocket:**
- Maximum **30 subscriptions** per WebSocket connection
- Connection valid for **24 hours** maximum
- Server disconnects after **30 seconds** if no valid subscription
- Server disconnects after **1 minute** if subscription has no data flow

**Futures WebSocket:**
- Similar subscription limits
- Connection requires regular ping messages

### Multiple Connections

If you need more than 30 streams:
- Create multiple WebSocket connections
- Each connection can have up to 30 subscriptions
- No limit on number of connections per IP (within reason)

---

## Heartbeat / Keep-Alive

### Spot WebSocket Ping/Pong

**Client-initiated ping:**
```json
{
  "method": "PING"
}
```

**Server response:**
```json
{
  "id": null,
  "code": 0,
  "msg": "pong"
}
```

**Timing:**
- If no ping received within **1 minute**, server closes connection
- Send ping every **10-20 seconds** recommended
- Server may also send PING, client should respond with PONG

### Futures WebSocket Ping/Pong

**Client sends:**
```json
{
  "method": "ping"
}
```

**Server responds:**
```json
{
  "channel": "pong",
  "data": 1587453241453
}
```

**Critical Timing:**
- If no ping received within **1 minute**, connection will be closed
- **Send ping every 10-20 seconds** to maintain connection
- This is strictly enforced

---

## Message Format

### Subscription Request

**Spot format:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.deals.v3.api@BTCUSDT",
    "spot@public.kline.v3.api.pb@ETHUSDT@Min15"
  ]
}
```

**Success response:**
```json
{
  "id": null,
  "code": 0,
  "msg": "success"
}
```

**Error response:**
```json
{
  "id": null,
  "code": 1,
  "msg": "Invalid symbol"
}
```

### Unsubscription Request

**Spot format:**
```json
{
  "method": "UNSUBSCRIPTION",
  "params": [
    "spot@public.deals.v3.api@BTCUSDT"
  ]
}
```

---

## Market Data Streams (Spot)

### Trade Streams

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.deals.v3.api@BTCUSDT"
  ]
}
```

**Stream name format:**
```
spot@public.deals.v3.api@{symbol}
```

**Response:**
```json
{
  "c": "spot@public.deals.v3.api",
  "d": {
    "deals": [
      {
        "p": "93220.00",
        "v": "0.04438243",
        "S": 2,
        "t": 1736409765051
      }
    ]
  },
  "s": "BTCUSDT",
  "t": 1736409765051
}
```

**Fields:**
- `p`: Price
- `v`: Volume/Quantity
- `S`: Trade type (1 = buy, 2 = sell)
- `t`: Timestamp

### Aggregate Trade Streams

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.aggre.deals.v3.api.pb@100ms@BTCUSDT"
  ]
}
```

**Stream name format:**
```
spot@public.aggre.deals.v3.api.pb@{interval}@{symbol}
```

**Intervals:**
- `100ms`: 100 milliseconds
- `10ms`: 10 milliseconds (high frequency)

### Depth/Order Book Streams

#### Limited Depth (5/10/20 levels)

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.limit.depth.v3.api.pb@BTCUSDT@5"
  ]
}
```

**Stream name format:**
```
spot@public.limit.depth.v3.api.pb@{symbol}@{levels}
```

**Levels:** `5`, `10`, or `20`

**Response:**
```json
{
  "c": "spot@public.limit.depth.v3.api",
  "d": {
    "asks": [
      {
        "p": "93230.00",
        "v": "0.8"
      },
      {
        "p": "93240.00",
        "v": "2.1"
      }
    ],
    "bids": [
      {
        "p": "93220.00",
        "v": "0.5"
      },
      {
        "p": "93210.00",
        "v": "1.2"
      }
    ]
  },
  "s": "BTCUSDT",
  "t": 1736409765051
}
```

#### Incremental Depth Updates

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.aggre.depth.v3.api.pb@100ms@BTCUSDT"
  ]
}
```

**Stream name format:**
```
spot@public.aggre.depth.v3.api.pb@{interval}@{symbol}
```

**Intervals:**
- `100ms`: 100 milliseconds
- `10ms`: 10 milliseconds

**Note:** This provides incremental updates. You need to maintain local order book.

### Kline/Candlestick Streams

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.kline.v3.api.pb@BTCUSDT@Min15"
  ]
}
```

**Stream name format:**
```
spot@public.kline.v3.api.pb@{symbol}@{interval}
```

**Intervals:**
- `Min1`: 1 minute
- `Min5`: 5 minutes
- `Min15`: 15 minutes
- `Min30`: 30 minutes
- `Min60`: 60 minutes
- `Hour4`: 4 hours
- `Hour8`: 8 hours
- `Day1`: 1 day
- `Week1`: 1 week
- `Month1`: 1 month

**Response:**
```json
{
  "c": "spot@public.kline.v3.api",
  "d": {
    "k": {
      "i": "Min15",
      "t": 1736410500,
      "o": "92925",
      "c": "93158.47",
      "h": "93158.47",
      "l": "92800",
      "v": "36.83803224",
      "a": "3424811.05",
      "T": 1736411400
    }
  },
  "s": "BTCUSDT",
  "t": 1736410500000
}
```

**Fields:**
- `i`: Interval
- `t`: Kline start time (seconds)
- `T`: Kline end time (seconds)
- `o`: Open price
- `c`: Close price
- `h`: High price
- `l`: Low price
- `v`: Volume
- `a`: Quote asset volume (amount)

### Ticker Streams

#### Mini Ticker

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@public.miniTicker.v3.api.pb@BTCUSDT@UTC+8"
  ]
}
```

**Stream name format:**
```
spot@public.miniTicker.v3.api.pb@{symbol}@{timezone}
```

**Timezones:** `UTC+0`, `UTC+8`, etc.

**Response:**
```json
{
  "c": "spot@public.miniTicker.v3.api",
  "d": {
    "e": "24hrMiniTicker",
    "s": "BTCUSDT",
    "c": "93200.50",
    "o": "92000.00",
    "h": "93500.00",
    "l": "91800.00",
    "v": "12345.67",
    "t": 1640167200000
  },
  "s": "BTCUSDT"
}
```

**Fields:**
- `e`: Event type
- `s`: Symbol
- `c`: Close/current price
- `o`: Open price
- `h`: High price
- `l`: Low price
- `v`: Volume
- `t`: Timestamp

#### Full Ticker

Contains additional information including bid/ask prices, price changes, etc.

---

## User Data Streams (Spot)

User data streams require a **listen key** for authentication.

### Creating a Listen Key

**REST Endpoint:**
```http
POST /api/v3/userDataStream
X-MEXC-APIKEY: your_api_key
```

**Response:**
```json
{
  "listenKey": "pqia91ma19a5s61cv6a81va65sd099v8a65a1a5s61cv6a81va65sdf19v8a65a1"
}
```

### Connecting with Listen Key

```
wss://wbs.mexc.com/ws?listenKey=pqia91ma19a5s61cv6a81va65sd099v8a65a1a5s61cv6a81va65sdf19v8a65a1
```

### Keep Listen Key Alive

**REST Endpoint:**
```http
PUT /api/v3/userDataStream?listenKey={listenKey}
X-MEXC-APIKEY: your_api_key
```

**Timing:**
- Listen key valid for **60 minutes** after creation
- **Send keepalive every 30 minutes** to extend validity
- Server will close connection if key expires

### Closing Listen Key

**REST Endpoint:**
```http
DELETE /api/v3/userDataStream?listenKey={listenKey}
X-MEXC-APIKEY: your_api_key
```

### Listen Key Limits

- Maximum **60 listen keys** per UID
- Each listen key supports maximum **5 WebSocket connections**
- Single connection valid for **24 hours**

### Account Update Event

**Subscription after connection:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@private.account.v3.api.pb"
  ]
}
```

**Event:**
```json
{
  "c": "spot@private.account.v3.api",
  "d": {
    "e": "spot@private.account.v3.api",
    "B": [
      {
        "a": "USDT",
        "f": "10000.5",
        "l": "500.0"
      },
      {
        "a": "BTC",
        "f": "0.5",
        "l": "0.1"
      }
    ]
  },
  "t": 1640080800000
}
```

**Fields:**
- `e`: Event type
- `B`: Balances array
- `a`: Asset
- `f`: Free balance
- `l`: Locked balance

### Order Update Event

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@private.orders.v3.api.pb"
  ]
}
```

**Event:**
```json
{
  "c": "spot@private.orders.v3.api",
  "d": {
    "e": "spot@private.orders.v3.api",
    "s": "BTCUSDT",
    "S": "BUY",
    "o": "LIMIT",
    "i": "91d9a3c4a3ab40c7ba76c98598dcf85a",
    "c": "myOrder1",
    "p": "90000",
    "q": "0.1",
    "z": "0.05",
    "n": "0.00000005",
    "N": "BTC",
    "u": true,
    "w": true,
    "m": false,
    "O": 1640080800000,
    "E": 1640084400000,
    "x": "PARTIALLY_FILLED",
    "X": "PARTIALLY_FILLED",
    "Z": "4500",
    "A": "90000"
  },
  "s": "BTCUSDT",
  "t": 1640084400000
}
```

**Fields:**
- `e`: Event type
- `s`: Symbol
- `S`: Side (BUY/SELL)
- `o`: Order type
- `i`: Order ID
- `c`: Client order ID
- `p`: Price
- `q`: Quantity
- `z`: Executed quantity
- `n`: Commission amount
- `N`: Commission asset
- `u`: Is trade normal user
- `w`: Is working
- `m`: Is maker
- `O`: Order creation time
- `E`: Event time
- `x`: Execution type
- `X`: Order status
- `Z`: Cumulative quote qty
- `A`: Average price

### Deal/Trade Update Event

**Subscription:**
```json
{
  "method": "SUBSCRIPTION",
  "params": [
    "spot@private.deals.v3.api.pb"
  ]
}
```

**Event:**
```json
{
  "c": "spot@private.deals.v3.api",
  "d": {
    "p": "93220.00",
    "v": "0.04438243",
    "a": "4137.85",
    "S": 1,
    "T": 1736409765051,
    "i": "trade_id_123",
    "o": "order_id_456",
    "f": "0.0000443824",
    "N": "BTC",
    "m": false
  },
  "s": "BTCUSDT",
  "t": 1736409765051
}
```

**Fields:**
- `p`: Price
- `v`: Volume/Quantity
- `a`: Quote amount
- `S`: Side (1 = buy, 2 = sell)
- `T`: Trade time
- `i`: Trade ID
- `o`: Order ID
- `f`: Fee
- `N`: Fee asset
- `m`: Is maker

---

## Futures WebSocket Streams

### Connection

```
wss://contract.mexc.com/edge
```

### Subscription Format

**Example:**
```json
{
  "method": "sub.deal",
  "param": {
    "symbol": "BTC_USDT"
  }
}
```

### Available Channels

#### Deal/Trade Channel

**Subscribe:**
```json
{
  "method": "sub.deal",
  "param": {
    "symbol": "BTC_USDT"
  }
}
```

#### Depth Channel

**Subscribe:**
```json
{
  "method": "sub.depth",
  "param": {
    "symbol": "BTC_USDT"
  }
}
```

#### Kline Channel

**Subscribe:**
```json
{
  "method": "sub.kline",
  "param": {
    "symbol": "BTC_USDT",
    "interval": "Min1"
  }
}
```

#### Personal Position Channel

**Subscribe:**
```json
{
  "method": "sub.personal.position",
  "param": {}
}
```

**Channel name:** `push.personal.position`

#### Personal Order Channel

**Subscribe:**
```json
{
  "method": "sub.personal.order",
  "param": {}
}
```

**Channel name:** `push.personal.order`

---

## Implementation Guide

### Basic Connection Example

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

async fn connect_websocket() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://wbs.mexc.com/ws";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to trades
    let subscribe = serde_json::json!({
        "method": "SUBSCRIPTION",
        "params": ["spot@public.deals.v3.api@BTCUSDT"]
    });
    write.send(Message::Text(subscribe.to_string())).await?;

    // Start ping task
    let write_clone = write.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(15));
        loop {
            interval.tick().await;
            let ping = serde_json::json!({"method": "PING"});
            if write_clone.send(Message::Text(ping.to_string())).await.is_err() {
                break;
            }
        }
    });

    // Read messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Received: {}", text);
            }
            Message::Close(_) => {
                println!("Connection closed");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}
```

### Listen Key Management

```rust
struct ListenKeyManager {
    api_key: String,
    listen_key: Option<String>,
    last_keepalive: u64,
}

impl ListenKeyManager {
    async fn get_or_create(&mut self) -> Result<String, Error> {
        if let Some(key) = &self.listen_key {
            // Check if we need keepalive (every 30 min)
            if current_timestamp_ms() - self.last_keepalive > 30 * 60 * 1000 {
                self.keepalive().await?;
            }
            return Ok(key.clone());
        }

        // Create new listen key
        let response = self.create_listen_key().await?;
        self.listen_key = Some(response.listen_key.clone());
        self.last_keepalive = current_timestamp_ms();
        Ok(response.listen_key)
    }

    async fn create_listen_key(&self) -> Result<ListenKeyResponse, Error> {
        // POST /api/v3/userDataStream
        // with X-MEXC-APIKEY header
        unimplemented!()
    }

    async fn keepalive(&mut self) -> Result<(), Error> {
        if let Some(key) = &self.listen_key {
            // PUT /api/v3/userDataStream?listenKey={key}
            // Update last_keepalive on success
            self.last_keepalive = current_timestamp_ms();
        }
        Ok(())
    }
}
```

### Reconnection Logic

```rust
async fn websocket_with_reconnect() {
    let mut reconnect_delay = 1000;
    let max_delay = 60000;

    loop {
        match connect_and_run().await {
            Ok(_) => {
                println!("Connection closed normally");
                reconnect_delay = 1000; // Reset delay
            }
            Err(e) => {
                eprintln!("Connection error: {}", e);
                tokio::time::sleep(Duration::from_millis(reconnect_delay)).await;
                reconnect_delay = (reconnect_delay * 2).min(max_delay);
            }
        }
    }
}
```

---

## Best Practices

### 1. Always Send Pings

```rust
// Send ping every 15 seconds (well within 60s limit)
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(15));
    loop {
        interval.tick().await;
        send_ping().await;
    }
});
```

### 2. Handle Reconnections

- Implement exponential backoff
- Resubscribe to channels after reconnection
- Maintain state between connections

### 3. Manage Listen Keys

- Create listen key before WebSocket connection
- Send keepalive every 30 minutes
- Handle expired keys gracefully
- Close unused keys to avoid hitting 60 key limit

### 4. Subscription Management

- Track active subscriptions
- Don't exceed 30 subscriptions per connection
- Use multiple connections if needed
- Unsubscribe from unused channels

### 5. Message Parsing

- Handle all message types (data, ping, pong, close)
- Validate JSON structure
- Handle missing fields gracefully
- Log parsing errors for debugging

### 6. Connection Health

- Monitor message frequency
- Detect stale connections
- Reconnect if no messages for expected duration
- Track connection uptime

---

## Troubleshooting

### Connection Drops

**Symptoms:**
- Connection closes unexpectedly
- No error message

**Solutions:**
- Ensure ping sent every 10-20 seconds
- Check subscription limit (max 30)
- Verify listen key is valid (for user streams)
- Check network connectivity

### Subscription Failures

**Symptoms:**
- No data received after subscription
- Error response to subscription

**Solutions:**
- Verify symbol format (uppercase, correct separator)
- Check symbol exists and is tradable
- Ensure subscription format is correct
- Verify channel name spelling

### Listen Key Expired

**Symptoms:**
- Connection closes after 60 minutes
- User data stops flowing

**Solutions:**
- Implement keepalive (every 30 minutes)
- Monitor last keepalive time
- Create new key if expired
- Reconnect with new key

---

## Summary

### Key Points

1. **Connection URLs**:
   - Spot: `wss://wbs.mexc.com/ws`
   - Futures: `wss://contract.mexc.com/edge`

2. **Limits**:
   - Max 30 subscriptions per connection
   - Connection valid 24 hours
   - 60 listen keys per UID
   - 5 connections per listen key

3. **Heartbeat**:
   - Send ping every 10-20 seconds
   - Connection closes after 1 minute without ping

4. **Listen Keys**:
   - Valid for 60 minutes
   - Keepalive every 30 minutes
   - Required for user data streams

5. **Symbol Format**:
   - Spot: `BTCUSDT`
   - Futures: `BTC_USDT`
   - Always uppercase

6. **Best Practices**:
   - Implement reconnection logic
   - Monitor connection health
   - Handle all message types
   - Manage subscriptions efficiently
   - Keep listen keys alive

---

## Sources

- [MEXC WebSocket Market Streams](https://www.mexc.com/api-docs/spot-v3/websocket-market-streams)
- [MEXC WebSocket User Data Streams](https://www.mexc.com/api-docs/spot-v3/websocket-user-data-streams)
- [MEXC Futures WebSocket API](https://www.mexc.com/api-docs/futures/websocket-api)
