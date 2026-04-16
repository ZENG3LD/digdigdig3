# dYdX v4 WebSocket API

## Overview

The dYdX v4 Indexer provides real-time data feeds through WebSocket connections, offering lower latency than REST API polling.

**WebSocket Endpoints**:
- **Mainnet**: `wss://indexer.dydx.trade/v4/ws`
- **Testnet**: `wss://indexer.v4testnet.dydx.exchange/v4/ws`

## Connection

### Basic Connection

```javascript
const ws = new WebSocket('wss://indexer.dydx.trade/v4/ws');

ws.on('open', () => {
  console.log('Connected to dYdX v4 WebSocket');
});

ws.on('message', (data) => {
  const message = JSON.parse(data);
  console.log('Received:', message);
});

ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});

ws.on('close', () => {
  console.log('WebSocket connection closed');
});
```

### Rust Example

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

async fn connect_websocket() -> Result<(), ExchangeError> {
    let url = "wss://indexer.dydx.trade/v4/ws";
    let (ws_stream, _) = connect_async(url).await?;

    let (mut write, mut read) = ws_stream.split();

    // Send subscription message
    let subscribe_msg = serde_json::json!({
        "type": "subscribe",
        "channel": "v4_orderbook",
        "id": "BTC-USD"
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Read messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let data: serde_json::Value = serde_json::from_str(&text)?;
                println!("Received: {}", data);
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

## Available Channels

dYdX v4 provides **6 primary WebSocket channels**:

1. **v4_orderbook** - Order book updates (bids/asks)
2. **v4_trades** - Trade executions
3. **v4_markets** - Market data and oracle prices
4. **v4_candles** - OHLC candle data
5. **v4_subaccounts** - Account updates (positions, orders, fills)
6. **v4_parent_subaccounts** - Parent account updates (aggregated)
7. **v4_block_height** - Block height and timestamp updates

## Message Format

### General Structure

All messages follow this format:

```json
{
  "type": "message_type",
  "connection_id": "unique-connection-id",
  "channel": "channel_name",
  "id": "channel_identifier",
  "message_id": 123,
  "contents": { /* channel-specific data */ },
  "version": "1.0"
}
```

**Fields**:
- `type`: Message type ("subscribed", "unsubscribed", "channel_data", "error")
- `connection_id`: Unique connection identifier (UUID)
- `channel`: Channel name (e.g., "v4_orderbook")
- `id`: Channel-specific identifier (market ticker, subaccount ID, etc.)
- `message_id`: Incremental message counter (starts at 0)
- `contents`: Channel-specific payload
- `version`: Protocol version

## Subscription Format

### Subscribe Message

```json
{
  "type": "subscribe",
  "channel": "channel_name",
  "id": "identifier",
  "batched": false
}
```

**Parameters**:
- `type`: Always "subscribe"
- `channel`: One of the available channels
- `id`: Channel-specific identifier (market ticker, subaccount ID, etc.)
- `batched`: Optional boolean to reduce message frequency (default: false)

### Unsubscribe Message

```json
{
  "type": "unsubscribe",
  "channel": "channel_name",
  "id": "identifier"
}
```

### Subscription Acknowledgement

```json
{
  "type": "subscribed",
  "connection_id": "conn-uuid-123",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "message_id": 0
}
```

### Unsubscribe Acknowledgement

```json
{
  "type": "unsubscribed",
  "connection_id": "conn-uuid-123",
  "channel": "v4_orderbook",
  "id": "BTC-USD"
}
```

## Channel Details

### 1. v4_orderbook - Order Book Updates

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "batched": false
}
```

**Update Message**:
```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_orderbook",
  "id": "BTC-USD",
  "message_id": 1,
  "contents": {
    "bids": [
      ["50000.0", "1.5"],
      ["49999.0", "2.3"]
    ],
    "asks": [
      ["50001.0", "0.8"],
      ["50002.0", "1.2"]
    ]
  },
  "version": "1.0"
}
```

**Contents Structure**:
- `bids`: Array of [price, size] tuples (sorted descending)
- `asks`: Array of [price, size] tuples (sorted ascending)

**Update Type**: Snapshot + incremental updates
- First message is full snapshot
- Subsequent messages are incremental changes
- Price level with size "0" means remove that level

**Batched Mode**:
- Reduces update frequency
- Aggregates multiple updates into one message
- Lower bandwidth, slightly higher latency

### 2. v4_trades - Trade Executions

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_trades",
  "id": "BTC-USD",
  "batched": false
}
```

**Update Message**:
```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_trades",
  "id": "BTC-USD",
  "message_id": 2,
  "contents": {
    "trades": [
      {
        "id": "trade-uuid-123",
        "side": "BUY",
        "size": "0.5",
        "price": "50000.0",
        "createdAt": "2026-01-20T12:34:56.789Z",
        "type": "LIMIT"
      }
    ]
  },
  "version": "1.0"
}
```

**Trade Fields**:
- `id`: Unique trade ID
- `side`: "BUY" or "SELL"
- `size`: Trade size (string decimal)
- `price`: Execution price (string decimal)
- `createdAt`: ISO 8601 timestamp
- `type`: Order type that generated trade

**Update Frequency**: Real-time (as trades occur)

### 3. v4_markets - Market Data Updates

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_markets"
}
```

**Note**: No `id` field - subscribes to all markets

**Update Message**:
```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_markets",
  "message_id": 3,
  "contents": {
    "trading": {
      "BTC-USD": {
        "oraclePrice": "50000.5",
        "priceChange24H": "1250.75",
        "nextFundingRate": "0.00001"
      },
      "ETH-USD": {
        "oraclePrice": "3000.2",
        "priceChange24H": "-50.30",
        "nextFundingRate": "0.00002"
      }
    }
  },
  "version": "1.0"
}
```

**Contents Structure**:
- `trading`: Map of market tickers to market data
- Each market includes:
  - `oraclePrice`: Oracle price (string decimal)
  - `priceChange24H`: 24h price change (string decimal)
  - `nextFundingRate`: Next funding rate (string decimal)

**Update Frequency**: Periodic (when oracle price or funding rate updates)

### 4. v4_candles - OHLC Candle Data

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_candles",
  "id": "BTC-USD/1MIN"
}
```

**ID Format**: `{ticker}/{resolution}`

**Resolutions**: "1MIN", "5MINS", "15MINS", "30MINS", "1HOUR", "4HOURS", "1DAY"

**Update Message**:
```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_candles",
  "id": "BTC-USD/1MIN",
  "message_id": 4,
  "contents": {
    "candles": [
      {
        "startedAt": "2026-01-20T12:00:00.000Z",
        "ticker": "BTC-USD",
        "resolution": "1MIN",
        "low": "49950.0",
        "high": "50100.0",
        "open": "50000.0",
        "close": "50050.0",
        "baseTokenVolume": "125.5",
        "usdVolume": "6277500.0",
        "trades": 543,
        "startingOpenInterest": "10000.0"
      }
    ]
  },
  "version": "1.0"
}
```

**Candle Fields**: Same as REST API candles endpoint

**Update Frequency**: At the end of each candle period + real-time updates during current candle

### 5. v4_subaccounts - Account Updates

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_subaccounts",
  "id": "dydx1abc123.../0"
}
```

**ID Format**: `{address}/{subaccountNumber}`

**Update Message**:
```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_subaccounts",
  "id": "dydx1abc123.../0",
  "message_id": 5,
  "contents": {
    "orders": [
      {
        "id": "order-uuid-123",
        "side": "BUY",
        "size": "1.0",
        "price": "50000.0",
        "status": "OPEN",
        "type": "LIMIT",
        "clobPairId": "0"
      }
    ],
    "fills": [
      {
        "id": "fill-uuid-456",
        "side": "SELL",
        "size": "0.5",
        "price": "51000.0",
        "market": "BTC-USD",
        "liquidity": "TAKER",
        "fee": "5.0",
        "createdAt": "2026-01-20T12:00:00.000Z"
      }
    ],
    "positions": {
      "BTC-USD": {
        "market": "BTC-USD",
        "side": "LONG",
        "size": "2.5",
        "entryPrice": "48000.0",
        "unrealizedPnl": "5000.0"
      }
    },
    "transfers": [
      {
        "id": "transfer-uuid-789",
        "type": "DEPOSIT",
        "size": "1000.0",
        "symbol": "USDC",
        "createdAt": "2026-01-20T10:00:00.000Z"
      }
    ]
  },
  "version": "1.0"
}
```

**Contents Structure**:
- `orders`: Array of order updates (new, filled, canceled)
- `fills`: Array of new trade fills
- `positions`: Map of market → position updates
- `transfers`: Array of transfers (deposits, withdrawals)

**Update Frequency**: Real-time (as account events occur)

**Authentication**: None required (but subaccount must exist and have activity)

### 6. v4_parent_subaccounts - Parent Account Updates

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_parent_subaccounts",
  "id": "dydx1abc123.../0"
}
```

**ID Format**: `{address}/{parentSubaccountNumber}`

**Update Message**: Similar to v4_subaccounts but includes data from child subaccounts (128+)

**Use Case**: Monitor isolated positions across child subaccounts

### 7. v4_block_height - Block Updates

**Subscribe**:
```json
{
  "type": "subscribe",
  "channel": "v4_block_height"
}
```

**Note**: No `id` field needed

**Update Message**:
```json
{
  "type": "channel_data",
  "connection_id": "conn-uuid-123",
  "channel": "v4_block_height",
  "message_id": 6,
  "contents": {
    "blockHeight": "12345678",
    "time": "2026-01-20T12:34:56.789Z"
  },
  "version": "1.0"
}
```

**Contents**:
- `blockHeight`: Current block height (string)
- `time`: Block timestamp (ISO 8601)

**Update Frequency**: Every new block (~1-2 seconds)

**Use Case**: Track blockchain state, calculate order expiry

## Error Messages

### Error Response

```json
{
  "type": "error",
  "connection_id": "conn-uuid-123",
  "message": "Invalid channel or subscription ID",
  "code": "INVALID_SUBSCRIPTION"
}
```

**Common Error Codes**:
- `INVALID_SUBSCRIPTION`: Invalid channel or ID
- `RATE_LIMIT_EXCEEDED`: Too many subscriptions or messages
- `INTERNAL_ERROR`: Server-side error

## Connection Management

### Heartbeat / Ping-Pong

**Client → Server**:
```json
{
  "type": "ping"
}
```

**Server → Client**:
```json
{
  "type": "pong"
}
```

**Recommendation**: Send ping every 30 seconds to keep connection alive

### Reconnection Strategy

```rust
async fn maintain_websocket_connection(url: &str) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(60);

    loop {
        match connect_and_subscribe(url).await {
            Ok(_) => {
                // Connection closed normally, reset backoff
                backoff = Duration::from_secs(1);
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                tokio::time::sleep(backoff).await;
                backoff = std::cmp::min(backoff * 2, max_backoff);
            }
        }
    }
}
```

### Graceful Disconnection

**Client → Server**:
```json
{
  "type": "unsubscribe",
  "channel": "v4_orderbook",
  "id": "BTC-USD"
}
```

Then close WebSocket connection.

## Subscription Limits

**Not explicitly documented**, but best practices:

1. **Limit concurrent subscriptions** per connection
   - Recommend: 10-20 subscriptions per connection
   - Open multiple connections if needed

2. **Use batched mode** for high-frequency channels
   - Reduces message rate
   - Lower bandwidth usage

3. **Monitor connection health**
   - Track message latency
   - Detect stale connections
   - Reconnect if no messages received for 30+ seconds

## Data Freshness

### WebSocket vs REST API

**WebSocket Advantages**:
- Lower latency (push vs poll)
- Real-time updates
- No polling overhead

**Typical Latency**:
- WebSocket: <100ms from event to client
- REST API: 0-2 seconds behind (read replica lag)
- During high load: REST lag can increase

**Recommendation**: Use WebSocket for:
- Order book tracking
- Trade monitoring
- Position updates
- Real-time price feeds

Use REST API for:
- Initial data load (historical candles, etc.)
- Periodic snapshots
- Data validation

## Example: Full Implementation

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct SubscribeMessage {
    r#type: String,
    channel: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    batched: Option<bool>,
}

#[derive(Deserialize)]
struct WebSocketMessage {
    r#type: String,
    #[serde(default)]
    connection_id: String,
    #[serde(default)]
    channel: String,
    #[serde(default)]
    id: String,
    #[serde(default)]
    message_id: u64,
    #[serde(default)]
    contents: serde_json::Value,
}

async fn subscribe_orderbook(ticker: &str) -> Result<(), ExchangeError> {
    let url = "wss://indexer.dydx.trade/v4/ws";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to orderbook
    let subscribe = SubscribeMessage {
        r#type: "subscribe".to_string(),
        channel: "v4_orderbook".to_string(),
        id: ticker.to_string(),
        batched: Some(false),
    };

    write.send(Message::Text(serde_json::to_string(&subscribe)?)).await?;

    // Read messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let message: WebSocketMessage = serde_json::from_str(&text)?;

                match message.r#type.as_str() {
                    "subscribed" => {
                        println!("Subscribed to {} {}", message.channel, message.id);
                    }
                    "channel_data" => {
                        if message.channel == "v4_orderbook" {
                            handle_orderbook_update(message.contents)?;
                        }
                    }
                    "error" => {
                        eprintln!("WebSocket error: {:?}", message);
                    }
                    _ => {}
                }
            }
            Message::Ping(data) => {
                write.send(Message::Pong(data)).await?;
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

fn handle_orderbook_update(contents: serde_json::Value) -> Result<(), ExchangeError> {
    #[derive(Deserialize)]
    struct OrderbookUpdate {
        bids: Vec<(String, String)>,
        asks: Vec<(String, String)>,
    }

    let update: OrderbookUpdate = serde_json::from_value(contents)?;

    println!("Bids: {:?}", update.bids);
    println!("Asks: {:?}", update.asks);

    Ok(())
}
```

## Multiple Subscriptions

```rust
async fn subscribe_multiple(subscriptions: Vec<(&str, &str)>) -> Result<(), ExchangeError> {
    let url = "wss://indexer.dydx.trade/v4/ws";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to multiple channels
    for (channel, id) in subscriptions {
        let subscribe = SubscribeMessage {
            r#type: "subscribe".to_string(),
            channel: channel.to_string(),
            id: id.to_string(),
            batched: Some(false),
        };

        write.send(Message::Text(serde_json::to_string(&subscribe)?)).await?;
    }

    // Handle all messages
    while let Some(msg) = read.next().await {
        // Process messages...
    }

    Ok(())
}

// Usage
let subscriptions = vec![
    ("v4_orderbook", "BTC-USD"),
    ("v4_orderbook", "ETH-USD"),
    ("v4_trades", "BTC-USD"),
    ("v4_markets", ""),
];

subscribe_multiple(subscriptions).await?;
```

## Best Practices

1. **Handle Connection Loss**:
   - Implement automatic reconnection
   - Use exponential backoff
   - Resubscribe to all channels after reconnect

2. **Message Ordering**:
   - Use `message_id` to detect missing messages
   - If gap detected, fetch snapshot from REST API

3. **Batched Mode**:
   - Use for high-frequency channels (orderbook)
   - Reduces bandwidth and message processing load

4. **Ping/Pong**:
   - Send ping every 30 seconds
   - Reconnect if pong not received within 10 seconds

5. **Error Handling**:
   - Log all errors
   - Don't crash on single message error
   - Validate message structure before processing

6. **Rate Limiting**:
   - Limit subscription messages
   - Don't subscribe/unsubscribe rapidly
   - Reuse connections

7. **Resource Management**:
   - Close connections properly
   - Unsubscribe before closing
   - Clean up resources in error paths

## Monitoring

### Metrics to Track
- Messages received per second (by channel)
- Message latency (server timestamp vs receive time)
- Connection uptime
- Reconnection count
- Message processing errors

### Health Checks
```rust
struct WebSocketHealth {
    last_message_time: Instant,
    message_count: u64,
    reconnect_count: u32,
}

impl WebSocketHealth {
    fn is_stale(&self) -> bool {
        self.last_message_time.elapsed() > Duration::from_secs(30)
    }

    fn update(&mut self) {
        self.last_message_time = Instant::now();
        self.message_count += 1;
    }
}
```

## Summary

- **6+ WebSocket channels** for real-time data
- **No authentication** required (public data feeds)
- **Lower latency** than REST API
- **Subscription-based** model
- **Automatic reconnection** recommended
- **Message IDs** for ordering and gap detection
- **Batched mode** available for high-frequency channels

For Rust implementation:
- Use `tokio-tungstenite` for WebSocket client
- Implement automatic reconnection with backoff
- Handle message types: subscribed, unsubscribed, channel_data, error
- Parse JSON messages with `serde_json`
- Maintain message ID sequence per channel
- Implement health monitoring and alerting
