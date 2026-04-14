# Bitstamp WebSocket API

This document describes the Bitstamp WebSocket API v2 for real-time market data streaming.

---

## Overview

Bitstamp provides a WebSocket API for receiving real-time updates without polling. WebSocket connections provide:
- **Real-time updates**: Instant push of market data
- **Lower latency**: No polling overhead
- **Rate limit friendly**: WebSocket usage doesn't count toward REST rate limits

**Official Documentation**: https://www.bitstamp.net/websocket/v2/

---

## WebSocket Endpoint

**URL**: `wss://ws.bitstamp.net`

**Protocol**: WebSocket (WSS - secure WebSocket)

---

## Connection

### Establishing Connection

```javascript
// Example connection
const ws = new WebSocket('wss://ws.bitstamp.net');
```

```rust
// Rust example with tokio-tungstenite
use tokio_tungstenite::{connect_async, tungstenite::Message};

let (ws_stream, _) = connect_async("wss://ws.bitstamp.net").await?;
```

### Connection Lifecycle

1. **Connect**: Establish WebSocket connection to `wss://ws.bitstamp.net`
2. **Subscribe**: Send subscription messages for desired channels
3. **Receive**: Process incoming messages
4. **Unsubscribe** (optional): Unsubscribe from channels
5. **Disconnect**: Close connection gracefully

---

## Message Format

All messages are JSON strings.

### Client to Server Messages

**Subscription**:
```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "channel_name"
  }
}
```

**Unsubscription**:
```json
{
  "event": "bts:unsubscribe",
  "data": {
    "channel": "channel_name"
  }
}
```

### Server to Client Messages

**Subscription Confirmation**:
```json
{
  "event": "bts:subscription_succeeded",
  "channel": "channel_name",
  "data": {}
}
```

**Data Update**:
```json
{
  "event": "data",
  "channel": "channel_name",
  "data": { ... }
}
```

**Error**:
```json
{
  "event": "bts:error",
  "channel": "channel_name",
  "data": {
    "message": "error description"
  }
}
```

---

## Available Channels

### Public Channels

All public channels follow the pattern: `{channel_type}_{pair}`

| Channel Type | Description | Example |
|--------------|-------------|---------|
| `live_trades` | Real-time trades | `live_trades_btcusd` |
| `order_book` | Full order book snapshots | `order_book_btcusd` |
| `diff_order_book` | Differential order book updates | `diff_order_book_btcusd` |
| `live_orders` | Order book updates (legacy) | `live_orders_btcusd` |

**Note**: `live_orders` and `diff_order_book` are similar channels for order book updates.

---

## Channel Details

### 1. Live Trades

**Channel**: `live_trades_{pair}`

**Description**: Streams all executed trades in real-time.

**Subscription**:
```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "live_trades_btcusd"
  }
}
```

**Message Format**:
```json
{
  "data": {
    "amount": 0.01513062,
    "buy_order_id": 297260696,
    "sell_order_id": 297260910,
    "amount_str": "0.01513062",
    "price_str": "212.80",
    "timestamp": "1505558814",
    "price": 212.8,
    "type": 1,
    "id": 21565524,
    "cost": 3.219795936
  },
  "channel": "live_trades_btcusd",
  "event": "trade"
}
```

**Field Descriptions**:
- `id`: Trade ID (integer)
- `timestamp`: Unix timestamp (string, seconds)
- `price`: Trade price (number)
- `price_str`: Trade price (string, for precision)
- `amount`: Trade amount (number)
- `amount_str`: Trade amount (string, for precision)
- `type`: Trade type (0 = buy, 1 = sell)
- `buy_order_id`: Buy order ID (integer)
- `sell_order_id`: Sell order ID (integer)
- `cost`: Total cost (amount * price)

**Event Type**: `"trade"`

---

### 2. Order Book (Full Snapshot)

**Channel**: `order_book_{pair}`

**Description**: Provides full order book snapshots and updates.

**Subscription**:
```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "order_book_btcusd"
  }
}
```

**Initial Snapshot**:
```json
{
  "data": {
    "timestamp": "1643643584",
    "microtimestamp": "1643643584684047",
    "bids": [
      ["3284.06000000", "0.16927410"],
      ["3284.05000000", "1.00000000"],
      ["3284.02000000", "0.72755647"]
    ],
    "asks": [
      ["3289.00000000", "3.16123001"],
      ["3291.99000000", "0.22000000"],
      ["3292.00000000", "49.94312963"]
    ]
  },
  "channel": "order_book_btcusd",
  "event": "data"
}
```

**Field Descriptions**:
- `timestamp`: Unix timestamp in seconds (string)
- `microtimestamp`: Unix timestamp in microseconds (string)
- `bids`: Array of [price, amount] bid orders
- `asks`: Array of [price, amount] ask orders

**Update Messages**: Full order book is sent on every update (not incremental).

**Event Type**: `"data"`

---

### 3. Differential Order Book

**Channel**: `diff_order_book_{pair}`

**Description**: Provides incremental order book updates (more efficient than full snapshots).

**Subscription**:
```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "diff_order_book_btcusd"
  }
}
```

**Update Message**:
```json
{
  "data": {
    "timestamp": "1643643584",
    "microtimestamp": "1643643584684047",
    "bids": [
      ["3284.06000000", "0.16927410"]
    ],
    "asks": [
      ["3289.00000000", "0.00000000"]
    ]
  },
  "channel": "diff_order_book_btcusd",
  "event": "data"
}
```

**Important**:
- Amount of `"0.00000000"` means the price level was removed
- Non-zero amount means the price level was added or updated
- **No initial snapshot**: You must fetch the initial order book from REST API (`GET /api/v2/order_book/{pair}/`)

**Building Order Book**:
1. Fetch initial snapshot from REST API
2. Subscribe to `diff_order_book_{pair}`
3. Apply incremental updates to local order book
4. Remove price levels with amount = 0
5. Add/update price levels with amount > 0

**Event Type**: `"data"`

---

### 4. Live Orders (Legacy)

**Channel**: `live_orders_{pair}`

**Description**: Order book updates (older channel type, similar to `diff_order_book`).

**Note**: `diff_order_book` is preferred for new implementations.

**Subscription**:
```json
{
  "event": "bts:subscribe",
  "data": {
    "channel": "live_orders_btcusd"
  }
}
```

---

## Multi-Pair Subscriptions

You can subscribe to multiple pairs on the same connection:

```json
// Subscribe to BTC/USD trades
{
  "event": "bts:subscribe",
  "data": {
    "channel": "live_trades_btcusd"
  }
}

// Subscribe to ETH/USD trades
{
  "event": "bts:subscribe",
  "data": {
    "channel": "live_trades_ethusd"
  }
}

// Subscribe to BTC/USD order book
{
  "event": "bts:subscribe",
  "data": {
    "channel": "order_book_btcusd"
  }
}
```

**Recommended**: Use a single WebSocket connection and subscribe to multiple channels.

---

## Connection Management

### Heartbeat / Ping-Pong

Bitstamp WebSocket supports standard WebSocket ping/pong frames for keepalive.

**Client**: Send WebSocket ping frames periodically
**Server**: Responds with pong frames

**Recommended interval**: 30-60 seconds

### Reconnection

Implement automatic reconnection logic:

```rust
async fn maintain_connection(&mut self) {
    loop {
        match self.connect().await {
            Ok(ws) => {
                if let Err(e) = self.handle_messages(ws).await {
                    log::error!("WebSocket error: {}", e);
                }
            }
            Err(e) => {
                log::error!("Connection failed: {}", e);
            }
        }

        // Exponential backoff
        let delay = std::cmp::min(self.backoff, 60);
        tokio::time::sleep(Duration::from_secs(delay)).await;
        self.backoff = std::cmp::min(self.backoff * 2, 60);
    }
}
```

### Disconnect Handling

**Graceful Disconnect**:
1. Unsubscribe from all channels
2. Close WebSocket connection

**Unexpected Disconnect**:
1. Detect connection loss
2. Attempt reconnection with backoff
3. Resubscribe to previous channels

---

## Error Handling

### Subscription Errors

```json
{
  "event": "bts:error",
  "channel": "invalid_channel",
  "data": {
    "message": "Invalid channel"
  }
}
```

### Common Errors

- **Invalid channel**: Channel name doesn't exist
- **Invalid symbol**: Trading pair doesn't exist
- **Connection limit**: Too many concurrent connections

### Error Recovery

```rust
match message.event.as_str() {
    "bts:error" => {
        log::error!("Subscription error: {:?}", message.data);
        // Handle error (retry, alert, etc.)
    }
    "bts:subscription_succeeded" => {
        log::info!("Subscribed to {}", message.channel);
    }
    "trade" | "data" => {
        // Process market data
    }
    _ => {
        log::warn!("Unknown event: {}", message.event);
    }
}
```

---

## Order Book Management

### Building Order Book from Differential Updates

```rust
pub struct OrderBook {
    bids: BTreeMap<Decimal, Decimal>, // price -> amount
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderBook {
    // 1. Fetch initial snapshot from REST API
    pub async fn initialize(pair: &str) -> Result<Self> {
        let snapshot = fetch_rest_orderbook(pair).await?;
        let mut bids = BTreeMap::new();
        let mut asks = BTreeMap::new();

        for [price, amount] in snapshot.bids {
            bids.insert(price.parse()?, amount.parse()?);
        }
        for [price, amount] in snapshot.asks {
            asks.insert(price.parse()?, amount.parse()?);
        }

        Ok(Self { bids, asks })
    }

    // 2. Apply differential updates
    pub fn apply_update(&mut self, update: DiffOrderBookUpdate) {
        for [price, amount] in update.bids {
            let price: Decimal = price.parse().unwrap();
            let amount: Decimal = amount.parse().unwrap();

            if amount.is_zero() {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, amount);
            }
        }

        for [price, amount] in update.asks {
            let price: Decimal = price.parse().unwrap();
            let amount: Decimal = amount.parse().unwrap();

            if amount.is_zero() {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, amount);
            }
        }
    }

    pub fn best_bid(&self) -> Option<(Decimal, Decimal)> {
        self.bids.iter().next_back().map(|(p, a)| (*p, *a))
    }

    pub fn best_ask(&self) -> Option<(Decimal, Decimal)> {
        self.asks.iter().next().map(|(p, a)| (*p, *a))
    }
}
```

---

## Implementation Example

### Complete WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures::{StreamExt, SinkExt};
use serde_json::json;

pub struct BitstampWebSocket {
    url: String,
}

impl BitstampWebSocket {
    pub fn new() -> Self {
        Self {
            url: "wss://ws.bitstamp.net".to_string(),
        }
    }

    pub async fn connect(&self) -> Result<WebSocketStream, Error> {
        let (ws_stream, _) = connect_async(&self.url).await?;
        Ok(ws_stream)
    }

    pub async fn subscribe(&mut self, ws: &mut WebSocketStream, channel: &str) -> Result<()> {
        let subscribe_msg = json!({
            "event": "bts:subscribe",
            "data": {
                "channel": channel
            }
        });

        ws.send(Message::Text(subscribe_msg.to_string())).await?;
        Ok(())
    }

    pub async fn handle_messages(&mut self, mut ws: WebSocketStream) -> Result<()> {
        while let Some(msg) = ws.next().await {
            match msg? {
                Message::Text(text) => {
                    self.process_message(&text)?;
                }
                Message::Ping(data) => {
                    ws.send(Message::Pong(data)).await?;
                }
                Message::Close(_) => {
                    log::info!("WebSocket closed");
                    break;
                }
                _ => {}
            }
        }
        Ok(())
    }

    fn process_message(&mut self, text: &str) -> Result<()> {
        let msg: serde_json::Value = serde_json::from_str(text)?;

        match msg["event"].as_str() {
            Some("bts:subscription_succeeded") => {
                log::info!("Subscribed: {}", msg["channel"]);
            }
            Some("trade") => {
                self.handle_trade(msg)?;
            }
            Some("data") => {
                self.handle_data(msg)?;
            }
            Some("bts:error") => {
                log::error!("Error: {:?}", msg["data"]);
            }
            _ => {
                log::debug!("Unknown message: {}", text);
            }
        }

        Ok(())
    }

    fn handle_trade(&mut self, msg: serde_json::Value) -> Result<()> {
        let trade = msg["data"].clone();
        log::info!("Trade: {} @ {}", trade["amount"], trade["price"]);
        // Process trade...
        Ok(())
    }

    fn handle_data(&mut self, msg: serde_json::Value) -> Result<()> {
        let data = msg["data"].clone();
        let channel = msg["channel"].as_str().unwrap_or("");

        if channel.starts_with("order_book_") {
            // Handle full order book
            log::debug!("Order book update");
        } else if channel.starts_with("diff_order_book_") {
            // Handle differential update
            log::debug!("Differential order book update");
        }

        Ok(())
    }
}
```

---

## Performance Considerations

### Message Frequency

- **Trades**: Variable (depends on market activity)
- **Order Book**: High frequency during active trading
- **Diff Order Book**: Lower bandwidth than full order book

### Recommended Channels

For different use cases:

| Use Case | Recommended Channel |
|----------|---------------------|
| Trade history | `live_trades_{pair}` |
| Full order book | `order_book_{pair}` (low frequency updates) |
| Order book tracking | `diff_order_book_{pair}` (high frequency) |
| Market monitoring | `live_trades_{pair}` |

### Bandwidth

- **Full Order Book**: Higher bandwidth (entire book on each update)
- **Differential Order Book**: Lower bandwidth (only changes)

**For production**: Use `diff_order_book_{pair}` for order book tracking.

---

## Connection Limits

**Not publicly documented**, but recommendations:
- **1-2 connections per client** (typical)
- **Multiple subscriptions per connection** (preferred)

Avoid creating many concurrent connections.

---

## Testing

### Test Connection

```bash
# Using websocat
websocat wss://ws.bitstamp.net
```

Then send:
```json
{"event":"bts:subscribe","data":{"channel":"live_trades_btcusd"}}
```

### Test Subscription

Monitor for:
1. `bts:subscription_succeeded` confirmation
2. Incoming `trade` events
3. No `bts:error` messages

---

## Best Practices

### 1. Single Connection, Multiple Subscriptions

```rust
// Good
let ws = connect().await?;
subscribe(&ws, "live_trades_btcusd").await?;
subscribe(&ws, "live_trades_ethusd").await?;
subscribe(&ws, "order_book_btcusd").await?;

// Avoid
let ws1 = connect().await?; // Connection 1
subscribe(&ws1, "live_trades_btcusd").await?;

let ws2 = connect().await?; // Connection 2
subscribe(&ws2, "live_trades_ethusd").await?;
```

### 2. Implement Reconnection Logic

Always implement automatic reconnection with exponential backoff.

### 3. Validate Messages

Validate all incoming messages before processing:
- Check for required fields
- Validate data types
- Handle malformed messages gracefully

### 4. Use Differential Order Book

For order book tracking, use `diff_order_book_{pair}` to save bandwidth.

### 5. Synchronize with REST

Periodically synchronize WebSocket data with REST API to ensure consistency:
- Fetch REST order book snapshot every N minutes
- Compare with WebSocket-maintained order book
- Reset if significant drift detected

---

## Troubleshooting

### Connection Issues

**Problem**: Cannot connect to WebSocket
- Check internet connectivity
- Verify URL: `wss://ws.bitstamp.net`
- Check firewall/proxy settings

**Problem**: Connection drops frequently
- Implement ping/pong keepalive
- Check network stability
- Use reconnection logic with backoff

### Subscription Issues

**Problem**: No data after subscription
- Verify channel name format: `{type}_{pair}`
- Check trading pair exists (use `/api/v2/markets/`)
- Look for `bts:error` messages

### Data Issues

**Problem**: Order book becomes inconsistent
- Re-fetch snapshot from REST API
- Rebuild from scratch
- Verify update application logic

---

## Summary

### Key Points

- **Endpoint**: `wss://ws.bitstamp.net`
- **Channels**: `live_trades_{pair}`, `order_book_{pair}`, `diff_order_book_{pair}`
- **Message Format**: JSON
- **Subscription**: `{"event":"bts:subscribe","data":{"channel":"..."}}`
- **Reconnection**: Required for production use
- **Order Book**: Fetch initial snapshot from REST, apply diffs from WebSocket

### Recommended Implementation

1. Establish single WebSocket connection
2. Subscribe to required channels
3. Handle `trade` and `data` events
4. Implement reconnection with backoff
5. Use `diff_order_book` for order book updates
6. Periodically sync with REST API

---

## Reference

- **Official WebSocket Docs**: https://www.bitstamp.net/websocket/v2/
- **REST API**: https://www.bitstamp.net/api/
- **Trading Pairs**: `GET /api/v2/markets/`
