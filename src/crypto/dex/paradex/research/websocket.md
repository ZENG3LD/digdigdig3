# Paradex WebSocket API

## Overview

Paradex provides a **WebSocket API** for real-time market data and account updates using **JSON-RPC 2.0** protocol.

**Key Features**:
- No rate limits for subscriptions
- Real-time updates (no polling required)
- Public and private channels
- Persistent connections
- Automatic heartbeat mechanism

---

## WebSocket URLs

### Production (Mainnet)
```
wss://ws.api.prod.paradex.trade/v1
```

### Testnet (Sepolia)
```
wss://ws.api.testnet.paradex.trade/v1
```

---

## Connection

### Basic Connection

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn connect() -> Result<WebSocket, Error> {
    let (ws_stream, _) = connect_async("wss://ws.api.prod.paradex.trade/v1").await?;
    Ok(ws_stream)
}
```

### Connection Requirements

**Minimum Python Version**: 3.10+ (for Python clients)

**Libraries**:
- Python: `websocket-client` or `websockets`
- Rust: `tokio-tungstenite`
- JavaScript/TypeScript: `ws` or native WebSocket
- Go: `gorilla/websocket`

---

## Heartbeat Mechanism

### Ping/Pong

**Server Behavior**:
- Server sends **ping every 55 seconds**
- Client must respond with **pong within 5 seconds**
- Connection terminated if no pong response

**Implementation**:

```rust
use tokio_tungstenite::tungstenite::Message;

async fn handle_message(msg: Message) -> Result<Option<Message>, Error> {
    match msg {
        Message::Ping(payload) => {
            // Respond with pong
            Ok(Some(Message::Pong(payload)))
        }
        Message::Pong(_) => {
            // Pong received from server
            Ok(None)
        }
        Message::Text(text) => {
            // Handle JSON-RPC message
            handle_json_rpc(&text).await?;
            Ok(None)
        }
        _ => Ok(None),
    }
}
```

**Automatic Handling**: Most WebSocket libraries handle ping/pong automatically.

---

## JSON-RPC 2.0 Protocol

### Message Format

All messages follow JSON-RPC 2.0 specification:

**Client Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "METHOD_NAME",
  "params": { ... },
  "id": 1
}
```

**Server Response**:
```json
{
  "jsonrpc": "2.0",
  "result": { ... },
  "id": 1
}
```

**Server Notification** (no id):
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "CHANNEL_NAME",
    "data": { ... }
  }
}
```

---

## Authentication

### Public Channels

**No authentication required** for public channels:
- `markets_summary`
- `order_book`
- `trades`
- `bbo`
- `funding_data`

### Private Channels

**Authentication required** for private channels:
- `account`
- `balance_events`
- `positions`
- `orders`
- `fills`
- `funding_payments`
- `transactions`
- `transfers`
- `tradebusts`
- `block_trades`

### Authentication Flow

**Step 1**: Obtain JWT token (see authentication.md)

**Step 2**: Send authentication message:

```json
{
  "jsonrpc": "2.0",
  "method": "authenticate",
  "params": {
    "jwt_token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."
  },
  "id": 1
}
```

**Step 3**: Receive confirmation:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "authenticated": true
  },
  "id": 1
}
```

**Important**: After initial authentication, **no re-authentication required** for the connection's duration (even though JWT expires every 5 minutes).

---

## Subscription Management

### Subscribe to Channel

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "CHANNEL_NAME"
  },
  "id": 1
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "channel": "CHANNEL_NAME",
    "subscribed": true
  },
  "id": 1
}
```

**Error** (duplicate subscription):
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Already subscribed to this channel"
  },
  "id": 1
}
```

### Unsubscribe from Channel

**Request**:
```json
{
  "jsonrpc": "2.0",
  "method": "unsubscribe",
  "params": {
    "channel": "CHANNEL_NAME"
  },
  "id": 2
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "result": {
    "channel": "CHANNEL_NAME",
    "unsubscribed": true
  },
  "id": 2
}
```

**Error** (not subscribed):
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32000,
    "message": "Not subscribed to this channel"
  },
  "id": 2
}
```

---

## Public Channels

### markets_summary

**Description**: Market ticker data (price, volume, funding, etc.)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "markets_summary"
  },
  "id": 1
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "markets_summary",
    "data": {
      "market": "BTC-USD-PERP",
      "best_bid": "65432.1",
      "best_ask": "65432.5",
      "last_traded_price": "65432.3",
      "mark_price": "65432.2",
      "volume_24h": "123456789.50",
      "price_change_rate_24h": "0.0234",
      "funding_rate": "0.0001",
      "open_interest": "45678.5"
    }
  }
}
```

**Fields**:
- `market`: Market symbol
- `best_bid`: Highest bid price
- `best_ask`: Lowest ask price
- `last_traded_price`: Last trade price
- `mark_price`: Fair price for margin
- `volume_24h`: 24-hour volume (USD)
- `price_change_rate_24h`: 24h price change percentage
- `funding_rate`: Current funding rate
- `open_interest`: Total open positions

### order_book

**Description**: Orderbook snapshot updates (depth 15, throttled to 50ms or 100ms)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "refresh_rate": "50ms"
  },
  "id": 1
}
```

**Parameters**:
- `refresh_rate` (optional): "50ms" or "100ms"
- `price_tick` (optional): Price grouping

**Snapshot Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "data": {
      "type": "snapshot",
      "market": "BTC-USD-PERP",
      "asks": [
        ["65432.5", "1.234"],
        ["65432.6", "2.456"],
        ["65432.7", "3.789"]
      ],
      "bids": [
        ["65432.4", "1.111"],
        ["65432.3", "2.222"],
        ["65432.2", "3.333"]
      ],
      "seq_no": 12345678,
      "timestamp": 1681759756789
    }
  }
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "order_book.BTC-USD-PERP",
    "data": {
      "type": "update",
      "market": "BTC-USD-PERP",
      "asks": [
        ["65432.8", "1.5"]  // New ask
      ],
      "bids": [
        ["65432.3", "0"]    // Removed (size = 0)
      ],
      "seq_no": 12345679,
      "timestamp": 1681759756839
    }
  }
}
```

**Important**:
- Use `seq_no` to order updates correctly
- Size "0" means level removed
- Updates are delta-based (not full snapshots)

### bbo

**Description**: Best bid/offer with no artificial throttling (event-driven)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "bbo.BTC-USD-PERP"
  },
  "id": 1
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "bbo.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "bid": "65432.4",
      "bid_size": "1.111",
      "ask": "65432.5",
      "ask_size": "1.234",
      "seq_no": 12345678,
      "timestamp": 1681759756789
    }
  }
}
```

**Features**:
- **Event-driven**: Updates only when price/size changes
- **No throttling**: Immediate updates
- **Optimal for**: Price-only strategies, execution algorithms

### trades

**Description**: Public trades for specific market or all markets

**Subscription (Single Market)**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "trades.BTC-USD-PERP"
  },
  "id": 1
}
```

**Subscription (All Markets)**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "trades.ALL"
  },
  "id": 1
}
```

**Trade Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "trades.BTC-USD-PERP",
    "data": {
      "id": "trade_123456",
      "market": "BTC-USD-PERP",
      "side": "BUY",
      "price": "65432.3",
      "size": "0.5",
      "timestamp": 1681759756789
    }
  }
}
```

### funding_data

**Description**: Historical funding data

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "funding_data.BTC-USD-PERP"
  },
  "id": 1
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "funding_data.BTC-USD-PERP",
    "data": {
      "market": "BTC-USD-PERP",
      "funding_rate": "0.0001",
      "funding_time": 1681759756789,
      "predicted_rate": "0.00012"
    }
  }
}
```

---

## Private Channels

### account

**Description**: Account summary updates (margin, collateral, PnL)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "account"
  },
  "id": 1
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "account",
    "data": {
      "account": "0x...",
      "account_value": "125432.50",
      "free_collateral": "85432.50",
      "initial_margin_requirement": "35000.00",
      "maintenance_margin_requirement": "25000.00",
      "margin_cushion": "100432.50",
      "total_collateral": "120000.00",
      "status": "ACTIVE",
      "seq_no": 12345,
      "updated_at": 1681759756789
    }
  }
}
```

### positions

**Description**: Position updates (opens, closes, changes)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "positions"
  },
  "id": 1
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "positions",
    "data": {
      "id": "pos_123456",
      "account": "0x...",
      "market": "BTC-USD-PERP",
      "side": "LONG",
      "status": "OPEN",
      "size": "1.5",
      "average_entry_price": "65000.00",
      "liquidation_price": "59500.00",
      "unrealized_pnl": "648.00",
      "seq_no": 12345,
      "last_updated_at": 1681759756789
    }
  }
}
```

### orders

**Description**: Order updates (NEW, OPEN, FILLED, CANCELLED)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "orders"
  },
  "id": 1
}
```

**Update Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "orders",
    "data": {
      "id": "order_123456789",
      "status": "OPEN",
      "account": "0x...",
      "market": "BTC-USD-PERP",
      "side": "BUY",
      "type": "LIMIT",
      "size": "0.5",
      "price": "65000.00",
      "remaining_size": "0.5",
      "avg_fill_price": "0",
      "created_at": 1681759756789,
      "last_updated_at": 1681759756789,
      "seq_no": 12345
    }
  }
}
```

**Status Values**:
- `NEW`: Queued for risk checks
- `OPEN`: Active in order book
- `PARTIALLY_FILLED`: Partial execution
- `FILLED`: Fully executed
- `CANCELLED`: Cancelled by user
- `REJECTED`: Rejected by system

### fills

**Description**: Fill notifications (trade executions)

**Subscription (Single Market)**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "fills.BTC-USD-PERP"
  },
  "id": 1
}
```

**Subscription (All Markets)**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "fills.ALL"
  },
  "id": 1
}
```

**Fill Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "fills.BTC-USD-PERP",
    "data": {
      "id": "fill_123456",
      "order_id": "order_789",
      "account": "0x...",
      "market": "BTC-USD-PERP",
      "side": "BUY",
      "size": "0.5",
      "price": "65000.00",
      "fee": "16.25",
      "fee_currency": "USDC",
      "liquidity": "TAKER",
      "is_rpi": false,
      "is_liquidation": false,
      "created_at": 1681759756789,
      "seq_no": 12345
    }
  }
}
```

**Flags**:
- `is_rpi`: Retail Price Improvement fill
- `is_liquidation`: Fill from liquidation

### funding_payments

**Description**: Funding payment notifications

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "funding_payments"
  },
  "id": 1
}
```

**Payment Message**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "funding_payments",
    "data": {
      "id": "funding_123456",
      "account": "0x...",
      "market": "BTC-USD-PERP",
      "position_size": "1.5",
      "funding_rate": "0.0001",
      "payment": "-9.75",
      "timestamp": 1681759756789
    }
  }
}
```

**Note**: Negative payment means paid, positive means received.

### balance_events

**Description**: Balance change notifications

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "balance_events"
  },
  "id": 1
}
```

### transactions

**Description**: Transaction notifications

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "transactions"
  },
  "id": 1
}
```

### transfers

**Description**: Transfer notifications (deposits/withdrawals)

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "transfers"
  },
  "id": 1
}
```

### tradebusts

**Description**: Tradebust notifications

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "tradebusts"
  },
  "id": 1
}
```

### block_trades

**Description**: Block trade notifications

**Subscription**:
```json
{
  "jsonrpc": "2.0",
  "method": "subscribe",
  "params": {
    "channel": "block_trades"
  },
  "id": 1
}
```

---

## Channel Naming Convention

### Public Channels

**Global** (no market specified):
- `markets_summary`
- `funding_data` (may require market)

**Market-Specific**:
- `order_book.{MARKET}`
- `bbo.{MARKET}`
- `trades.{MARKET}`

**All Markets**:
- `trades.ALL`

### Private Channels

**Account-Level**:
- `account`
- `balance_events`
- `positions`
- `orders`
- `funding_payments`
- `transactions`
- `transfers`
- `tradebusts`
- `block_trades`

**Market-Specific** (with ALL option):
- `fills.{MARKET}`
- `fills.ALL`

---

## Error Handling

### Connection Errors

**Error Types**:
- Connection refused
- Connection timeout
- Connection closed unexpectedly

**Handling**:
```rust
async fn connect_with_retry(max_retries: u32) -> Result<WebSocket, Error> {
    let mut delay_ms = 1000;

    for attempt in 0..max_retries {
        match connect_async("wss://ws.api.prod.paradex.trade/v1").await {
            Ok((ws, _)) => return Ok(ws),
            Err(e) if attempt < max_retries - 1 => {
                eprintln!("Connection failed (attempt {}): {}", attempt + 1, e);
                tokio::time::sleep(Duration::from_millis(delay_ms)).await;
                delay_ms *= 2; // Exponential backoff
            }
            Err(e) => return Err(e.into()),
        }
    }

    Err("Max retries exceeded".into())
}
```

### Subscription Errors

**Common Errors**:

| Code | Message | Cause |
|------|---------|-------|
| -32000 | Already subscribed | Duplicate subscription |
| -32000 | Not subscribed | Unsubscribe from unsubscribed channel |
| -32600 | Invalid Request | Malformed JSON-RPC |
| -32601 | Method not found | Invalid method name |
| -32602 | Invalid params | Missing or invalid parameters |

**Example Error**:
```json
{
  "jsonrpc": "2.0",
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": {
      "field": "channel",
      "reason": "Channel does not exist"
    }
  },
  "id": 1
}
```

### Disconnection Handling

**Graceful Disconnection**:
```json
{
  "jsonrpc": "2.0",
  "method": "close",
  "params": {},
  "id": 999
}
```

**Unexpected Disconnection**:
- Detect via WebSocket close event
- Implement automatic reconnection
- Re-subscribe to all channels after reconnection

**Reconnection Strategy**:
```rust
async fn maintain_connection() {
    loop {
        match connect_and_subscribe().await {
            Ok(mut ws) => {
                while let Some(msg) = ws.next().await {
                    match msg {
                        Ok(msg) => handle_message(msg).await,
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break; // Reconnect
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }

        // Wait before reconnecting
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

---

## Best Practices

### 1. Subscribe to Relevant Channels Only

**Bad** (unnecessary subscriptions):
```rust
subscribe("trades.ALL").await?;
subscribe("order_book.BTC-USD-PERP").await?;
subscribe("order_book.ETH-USD-PERP").await?;
subscribe("order_book.SOL-USD-PERP").await?;
// ... 50 more markets
```

**Good** (targeted subscriptions):
```rust
// Only subscribe to markets you're actively trading
for market in active_markets {
    subscribe(&format!("bbo.{}", market)).await?;
}
```

### 2. Use BBO for Price Updates

**For price-only strategies**, use BBO instead of full orderbook:

```rust
// BBO: Event-driven, no throttling, minimal bandwidth
subscribe("bbo.BTC-USD-PERP").await?;

// vs

// Order book: 50ms throttled, larger payload
subscribe("order_book.BTC-USD-PERP").await?;
```

### 3. Handle Sequence Numbers

**Maintain order integrity**:

```rust
struct OrderBookManager {
    last_seq_no: u64,
    orderbook: OrderBook,
}

impl OrderBookManager {
    fn apply_update(&mut self, update: OrderBookUpdate) -> Result<(), Error> {
        // Check sequence
        if update.seq_no <= self.last_seq_no {
            return Err("Out-of-order update");
        }

        if update.seq_no != self.last_seq_no + 1 {
            // Gap detected, request snapshot
            return Err("Sequence gap");
        }

        self.orderbook.apply(update);
        self.last_seq_no = update.seq_no;
        Ok(())
    }
}
```

### 4. Authenticate Once per Connection

```rust
async fn connect_authenticated(jwt: &str) -> Result<WebSocket, Error> {
    let mut ws = connect().await?;

    // Authenticate once
    let auth_msg = json!({
        "jsonrpc": "2.0",
        "method": "authenticate",
        "params": {
            "jwt_token": jwt
        },
        "id": 1
    });

    ws.send(Message::Text(auth_msg.to_string())).await?;

    // Wait for confirmation
    wait_for_auth_response(&mut ws).await?;

    Ok(ws)
}
```

### 5. Separate Read/Write Tasks

```rust
async fn run_websocket(ws: WebSocket) {
    let (write, read) = ws.split();

    let read_task = tokio::spawn(async move {
        read.for_each(|msg| async {
            handle_message(msg).await;
        }).await;
    });

    let write_task = tokio::spawn(async move {
        // Send subscriptions, handle outgoing messages
        send_subscriptions(write).await;
    });

    tokio::try_join!(read_task, write_task).unwrap();
}
```

### 6. Monitor Connection Health

```rust
struct ConnectionMonitor {
    last_message_time: Arc<RwLock<Instant>>,
}

impl ConnectionMonitor {
    fn mark_message_received(&self) {
        *self.last_message_time.write().unwrap() = Instant::now();
    }

    fn is_stale(&self, threshold: Duration) -> bool {
        self.last_message_time.read().unwrap().elapsed() > threshold
    }
}

// In message loop
async fn monitor_loop(monitor: ConnectionMonitor) {
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;

        if monitor.is_stale(Duration::from_secs(120)) {
            eprintln!("Connection stale, reconnecting...");
            // Trigger reconnection
        }
    }
}
```

---

## Rate Limits

**WebSocket Subscriptions**: No explicit rate limits documented

**Connection Limits**: Not specified (typically 5-10 connections per IP)

**Best Practice**: Use single connection with multiple subscriptions rather than multiple connections.

---

## Complete Example

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::json;

async fn run_paradex_websocket(jwt: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Connect
    let (mut ws, _) = connect_async("wss://ws.api.prod.paradex.trade/v1").await?;

    // Authenticate
    let auth_msg = json!({
        "jsonrpc": "2.0",
        "method": "authenticate",
        "params": { "jwt_token": jwt },
        "id": 1
    });
    ws.send(Message::Text(auth_msg.to_string())).await?;

    // Subscribe to channels
    let subscriptions = vec![
        "account",
        "positions",
        "orders",
        "fills.ALL",
        "bbo.BTC-USD-PERP",
    ];

    for (i, channel) in subscriptions.iter().enumerate() {
        let sub_msg = json!({
            "jsonrpc": "2.0",
            "method": "subscribe",
            "params": { "channel": channel },
            "id": i + 2
        });
        ws.send(Message::Text(sub_msg.to_string())).await?;
    }

    // Message loop
    while let Some(msg) = ws.next().await {
        match msg? {
            Message::Text(text) => {
                let parsed: serde_json::Value = serde_json::from_str(&text)?;

                if let Some(method) = parsed.get("method") {
                    if method == "subscription" {
                        // Handle subscription data
                        handle_subscription(&parsed).await?;
                    }
                } else if parsed.get("result").is_some() {
                    // Handle response to our request
                    println!("Response: {}", text);
                }
            }
            Message::Ping(payload) => {
                ws.send(Message::Pong(payload)).await?;
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

async fn handle_subscription(msg: &serde_json::Value) -> Result<(), Box<dyn std::error::Error>> {
    let channel = msg["params"]["channel"].as_str().unwrap();
    let data = &msg["params"]["data"];

    match channel {
        "account" => println!("Account update: {}", data),
        "positions" => println!("Position update: {}", data),
        "orders" => println!("Order update: {}", data),
        ch if ch.starts_with("fills") => println!("Fill: {}", data),
        ch if ch.starts_with("bbo") => println!("BBO: {}", data),
        _ => println!("Unknown channel: {}", channel),
    }

    Ok(())
}
```

---

## Summary

1. **Protocol**: JSON-RPC 2.0 over WebSocket
2. **URLs**: `wss://ws.api.prod.paradex.trade/v1` (prod), `wss://ws.api.testnet.paradex.trade/v1` (testnet)
3. **Heartbeat**: Server pings every 55s, client must pong within 5s
4. **Authentication**: JWT token for private channels (once per connection)
5. **Public Channels**: markets_summary, order_book, bbo, trades, funding_data
6. **Private Channels**: account, positions, orders, fills, funding_payments, etc.
7. **Channel Format**: `{channel}.{MARKET}` or `{channel}.ALL`
8. **Sequence Numbers**: Use `seq_no` to order updates
9. **No Rate Limits**: For subscriptions (unlike REST API)
10. **Best Practice**: Single connection, multiple subscriptions, BBO for prices

---

## Additional Resources

- **WebSocket Documentation**: https://docs.paradex.trade/ws/general-information/introduction
- **Subscription Channels**: https://docs.paradex.trade/ws/general-information/subscription-channels
- **Code Examples**: https://docs.paradex.trade/ws/general-information/code-examples
- **Python SDK**: https://github.com/tradeparadex/paradex-py (WebSocket client implementation)
- **Best Practices**: https://docs.paradex.trade/trading/api-best-practices
