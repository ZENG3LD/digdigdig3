# Gemini Exchange WebSocket API

Complete WebSocket specification for implementing V5 connector WebSocket module.

---

## Overview

Gemini provides two primary WebSocket APIs:

1. **Market Data WebSocket (v2)**: Public, real-time market data
2. **Order Events WebSocket**: Private, real-time order updates

Gemini is migrating to **Fast API**, a unified low-latency solution, but legacy WebSocket APIs remain fully supported.

---

## Market Data WebSocket (v2)

### Connection Details

**URL**: `wss://api.gemini.com/v2/marketdata`
**Sandbox URL**: `wss://api.sandbox.gemini.com/v2/marketdata`

**Authentication**: None (public)
**Protocol**: WebSocket (WSS)

### Connection Lifecycle

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn connect_market_data() -> Result<WebSocketStream, Error> {
    let url = "wss://api.gemini.com/v2/marketdata";
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}
```

---

### Subscription Mechanism

#### Subscribe Message Format

```json
{
  "type": "subscribe",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["BTCUSD", "ETHUSD"]
    },
    {
      "name": "candles_1m",
      "symbols": ["BTCUSD"]
    }
  ]
}
```

**Fields**:
- `type` (string): "subscribe"
- `subscriptions` (array): List of subscription objects
  - `name` (string): Feed type (see below)
  - `symbols` (array): Trading pair symbols (uppercase)

#### Available Feed Types

| Feed Name | Description | Update Frequency |
|-----------|-------------|------------------|
| `l2` | Order book Level 2 data | Real-time |
| `candles_1m` | 1-minute candles | Every minute |
| `candles_5m` | 5-minute candles | Every 5 minutes |
| `candles_15m` | 15-minute candles | Every 15 minutes |
| `candles_30m` | 30-minute candles | Every 30 minutes |
| `candles_1h` | 1-hour candles | Every hour |
| `candles_6h` | 6-hour candles | Every 6 hours |
| `candles_1d` | 1-day candles | Every day |

#### Subscription Acknowledgment

```json
{
  "type": "subscribed",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["BTCUSD", "ETHUSD"]
    }
  ]
}
```

#### Multiple Subscriptions

You can subscribe to multiple feeds in a single connection:

```json
{
  "type": "subscribe",
  "subscriptions": [
    {"name": "l2", "symbols": ["BTCUSD", "ETHUSD", "ETHBTC"]},
    {"name": "candles_1m", "symbols": ["BTCUSD"]},
    {"name": "candles_1h", "symbols": ["ETHUSD"]}
  ]
}
```

---

### Message Types

#### 1. L2 Order Book Updates

```json
{
  "type": "l2_updates",
  "symbol": "BTCUSD",
  "changes": [
    ["buy", "50000.00", "1.5"],
    ["sell", "50001.00", "0.8"],
    ["buy", "49999.00", "0.0"]
  ],
  "trades": [
    {
      "type": "trade",
      "symbol": "BTCUSD",
      "event_id": 123456789,
      "timestamp": 1640000000000,
      "price": "50000.50",
      "quantity": "0.5",
      "side": "buy"
    }
  ],
  "auction_events": []
}
```

**Changes Array**: `[side, price, quantity]`
- `side` (string): "buy" or "sell"
- `price` (string): Price level
- `quantity` (string): New quantity at this level
  - `"0.0"` or `"0"`: Level removed
  - Non-zero: Level added/updated

**Initial Snapshot**:
- First message includes existing order book state
- Contains last 50 trades
- Subsequent messages are incremental updates

**Processing Logic**:
```rust
pub struct OrderBook {
    bids: BTreeMap<Decimal, Decimal>, // price -> quantity
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderBook {
    pub fn apply_changes(&mut self, changes: Vec<(String, String, String)>) {
        for (side, price_str, qty_str) in changes {
            let price: Decimal = price_str.parse().unwrap();
            let qty: Decimal = qty_str.parse().unwrap();

            match side.as_str() {
                "buy" => {
                    if qty.is_zero() {
                        self.bids.remove(&price);
                    } else {
                        self.bids.insert(price, qty);
                    }
                }
                "sell" => {
                    if qty.is_zero() {
                        self.asks.remove(&price);
                    } else {
                        self.asks.insert(price, qty);
                    }
                }
                _ => {}
            }
        }
    }
}
```

#### 2. Trade Updates

Trades are included in `l2_updates` messages in the `trades` array:

```json
{
  "type": "trade",
  "symbol": "BTCUSD",
  "event_id": 123456789,
  "timestamp": 1640000000000,
  "price": "50000.00",
  "quantity": "0.5",
  "side": "buy"
}
```

**Fields**:
- `type`: "trade"
- `symbol`: Trading pair
- `event_id`: Unique trade ID
- `timestamp`: Unix timestamp (milliseconds)
- `price`: Trade price (string)
- `quantity`: Trade quantity (string)
- `side`: "buy" (taker bought) or "sell" (taker sold)

#### 3. Candle Updates

```json
{
  "type": "candles_1m_updates",
  "symbol": "BTCUSD",
  "changes": [
    [1640000000000, 49500, 50100, 49400, 50000, 123.456789]
  ]
}
```

**Changes Array**: `[timestamp, open, high, low, close, volume]`
- `timestamp` (number): Candle start time (milliseconds)
- `open` (number): Opening price
- `high` (number): High price
- `low` (number): Low price
- `close` (number): Closing price
- `volume` (number): Volume in base currency

**Update Behavior**:
- Current candle updates continuously until period ends
- Completed candles sent once
- Check timestamp to identify which candle updated

---

### Unsubscribe

```json
{
  "type": "unsubscribe",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["ETHUSD"]
    }
  ]
}
```

**Response**:
```json
{
  "type": "unsubscribed",
  "subscriptions": [
    {
      "name": "l2",
      "symbols": ["ETHUSD"]
    }
  ]
}
```

---

### Heartbeat

Market Data v2 does **not** explicitly send heartbeat messages. Use WebSocket ping/pong for connection health:

```rust
// Send ping every 30 seconds
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = ws_write.send(Message::Ping(vec![])).await {
            eprintln!("Ping failed: {}", e);
            break;
        }
    }
});
```

---

### Error Handling

#### Connection Errors

- **Network disconnection**: Reconnect with exponential backoff
- **Invalid subscription**: Check symbol format and feed name
- **Rate limit**: Max 1 subscription change per symbol per minute

#### Reconnection Strategy

```rust
pub async fn connect_with_retry(
    max_retries: u32,
) -> Result<WebSocketStream, Error> {
    let mut delay = Duration::from_secs(1);

    for attempt in 0..max_retries {
        match connect_async("wss://api.gemini.com/v2/marketdata").await {
            Ok((ws, _)) => return Ok(ws),
            Err(e) if attempt < max_retries - 1 => {
                eprintln!("Connection failed (attempt {}): {}", attempt + 1, e);
                tokio::time::sleep(delay).await;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
            }
            Err(e) => return Err(e.into()),
        }
    }

    unreachable!()
}
```

---

## Order Events WebSocket

### Connection Details

**URL**: `wss://api.gemini.com/v1/order/events`
**Sandbox URL**: `wss://api.sandbox.gemini.com/v1/order/events`

**Authentication**: Required (HMAC-SHA384)
**Protocol**: WebSocket (WSS)

---

### Authentication

Order Events requires authentication during connection establishment.

#### Authentication Headers

Include these headers in the WebSocket upgrade request:

```rust
use tokio_tungstenite::tungstenite::handshake::client::Request;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use hmac::{Hmac, Mac};
use sha2::Sha384;

type HmacSha384 = Hmac<Sha384>;

async fn connect_order_events(
    api_key: &str,
    api_secret: &str,
) -> Result<WebSocketStream, Error> {
    let endpoint = "/v1/order/events";
    let nonce = get_nonce();

    // Create payload
    let payload = serde_json::json!({
        "request": endpoint,
        "nonce": nonce,
    });
    let payload_str = payload.to_string();

    // Base64 encode payload
    let b64_payload = BASE64.encode(payload_str.as_bytes());

    // Generate signature
    let mut mac = HmacSha384::new_from_slice(api_secret.as_bytes())?;
    mac.update(b64_payload.as_bytes());
    let signature = hex::encode(mac.finalize().into_bytes());

    // Build WebSocket request with auth headers
    let url = "wss://api.gemini.com/v1/order/events";
    let request = Request::builder()
        .uri(url)
        .header("X-GEMINI-APIKEY", api_key)
        .header("X-GEMINI-PAYLOAD", &b64_payload)
        .header("X-GEMINI-SIGNATURE", &signature)
        .header("Content-Type", "text/plain")
        .header("Content-Length", "0")
        .body(())?;

    let (ws_stream, _) = connect_async(request).await?;

    Ok(ws_stream)
}
```

---

### Event Types

#### 1. Subscription Acknowledgment

Received immediately after connection:

```json
{
  "type": "subscription_ack",
  "accountId": 123456,
  "subscriptionId": "abc-def-ghi-jkl",
  "symbolFilter": [],
  "apiSessionFilter": [],
  "eventTypeFilter": []
}
```

**Fields**:
- `type`: "subscription_ack"
- `accountId`: Your account ID
- `subscriptionId`: Unique subscription ID
- `symbolFilter`: Filtered symbols (empty = all)
- `apiSessionFilter`: Filtered API sessions (empty = all)
- `eventTypeFilter`: Filtered event types (empty = all)

#### 2. Heartbeat

Sent every **5 seconds**:

```json
{
  "type": "heartbeat",
  "timestampms": 1640000000000,
  "sequence": 12345,
  "socket_sequence": 67890,
  "trace_id": "xyz123abc456"
}
```

**Fields**:
- `type`: "heartbeat"
- `timestampms`: Server timestamp (milliseconds)
- `sequence`: Global event sequence number
- `socket_sequence`: Socket-specific sequence
- `trace_id`: Trace ID for debugging

**Purpose**: Connection health check; no response required

#### 3. Initial Orders

Sent after subscription_ack, lists all active orders:

```json
{
  "type": "initial",
  "order_id": "987654321",
  "account_name": "primary",
  "api_session": "session-abc-123",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestamp": "1640000000",
  "timestampms": 1640000000000,
  "is_live": true,
  "is_cancelled": false,
  "is_hidden": false,
  "executed_amount": "0.2",
  "remaining_amount": "0.3",
  "original_amount": "0.5",
  "price": "50000.00",
  "client_order_id": "my-order-123",
  "socket_sequence": 1
}
```

**Note**: One `initial` message per existing active order

#### 4. Accepted

Order accepted by exchange:

```json
{
  "type": "accepted",
  "order_id": "987654321",
  "account_name": "primary",
  "api_session": "session-abc-123",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000000000,
  "is_live": true,
  "is_cancelled": false,
  "is_hidden": false,
  "executed_amount": "0",
  "remaining_amount": "0.5",
  "original_amount": "0.5",
  "price": "50000.00",
  "socket_sequence": 2
}
```

**Trigger**: Immediately after order submission

#### 5. Rejected

Order rejected by exchange:

```json
{
  "type": "rejected",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000000000,
  "is_live": false,
  "is_cancelled": false,
  "original_amount": "0.5",
  "price": "50000.00",
  "reason": "InsufficientFunds",
  "socket_sequence": 3
}
```

**Fields**:
- `reason`: Error reason (e.g., "InsufficientFunds", "InvalidPrice")

**Common Rejection Reasons**:
- `InsufficientFunds`
- `InvalidPrice`
- `InvalidQuantity`
- `OrderNotFound`
- `InvalidSymbol`

#### 6. Booked

Order placed on the order book:

```json
{
  "type": "booked",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000000100,
  "is_live": true,
  "is_cancelled": false,
  "executed_amount": "0",
  "remaining_amount": "0.5",
  "original_amount": "0.5",
  "price": "50000.00",
  "socket_sequence": 4
}
```

**Trigger**: After accepted, when order visible on book

#### 7. Fill

Order execution (partial or complete):

```json
{
  "type": "fill",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000001000,
  "is_live": false,
  "is_cancelled": false,
  "executed_amount": "0.5",
  "remaining_amount": "0",
  "original_amount": "0.5",
  "price": "50000.00",
  "fill": {
    "trade_id": "123456789",
    "liquidity": "Maker",
    "price": "50000.00",
    "amount": "0.5",
    "fee": "25.00",
    "fee_currency": "USD"
  },
  "socket_sequence": 5
}
```

**Fill Object**:
- `trade_id` (string): Unique trade ID
- `liquidity` (string): "Maker" or "Taker"
- `price` (string): Fill price
- `amount` (string): Fill quantity
- `fee` (string): Fee charged
- `fee_currency` (string): Fee currency

**Partial vs Complete**:
- Partial fill: `remaining_amount > 0`, `is_live = true`
- Complete fill: `remaining_amount = 0`, `is_live = false`

#### 8. Cancelled

Order cancelled:

```json
{
  "type": "cancelled",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000002000,
  "is_live": false,
  "is_cancelled": true,
  "executed_amount": "0.3",
  "remaining_amount": "0",
  "original_amount": "0.5",
  "price": "50000.00",
  "reason": "Requested",
  "socket_sequence": 6
}
```

**Fields**:
- `reason`: Cancellation reason
  - `"Requested"`: User-requested cancel
  - `"IOC"`: Immediate-or-cancel order
  - `"MakerOrCancelWouldTake"`: Maker-or-cancel would be taker
  - `"PostOnlyWouldCross"`: Post-only would cross spread

#### 9. Cancel Rejected

Cancellation request rejected:

```json
{
  "type": "cancel_rejected",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "timestampms": 1640000003000,
  "reason": "OrderNotFound",
  "socket_sequence": 7
}
```

**Common Reasons**:
- `OrderNotFound`: Order doesn't exist or already cancelled
- `OrderAlreadyFilled`: Order filled before cancel processed

#### 10. Closed

Final event, order removed from book:

```json
{
  "type": "closed",
  "order_id": "987654321",
  "account_name": "primary",
  "symbol": "btcusd",
  "side": "buy",
  "order_type": "exchange limit",
  "timestampms": 1640000004000,
  "is_live": false,
  "is_cancelled": false,
  "executed_amount": "0.5",
  "remaining_amount": "0",
  "original_amount": "0.5",
  "price": "50000.00",
  "socket_sequence": 8
}
```

**Trigger**: After fill or cancel, final state

---

### Event Filtering

Gemini supports filtering events (though filters are empty by default).

#### Symbol Filter

To receive events only for specific symbols, include in authentication payload:

```json
{
  "request": "/v1/order/events",
  "nonce": 1640000000000,
  "symbolFilter": ["btcusd", "ethusd"]
}
```

#### Event Type Filter

Filter by event type:

```json
{
  "request": "/v1/order/events",
  "nonce": 1640000000000,
  "eventTypeFilter": ["fill", "cancelled"]
}
```

**Filterable Event Types**:
- `heartbeat`
- `initial`
- `accepted`
- `rejected`
- `booked`
- `fill`
- `cancelled`
- `cancel_rejected`
- `closed`

**Note**: `subscription_ack` is always sent

---

### Sequence Numbers

Every event includes sequence numbers for ordering:

```json
{
  "sequence": 12345,
  "socket_sequence": 67890
}
```

**sequence**: Global sequence across all events
**socket_sequence**: Per-connection sequence

**Use Case**: Detect missing events, ensure correct ordering

```rust
pub struct SequenceTracker {
    last_sequence: Option<u64>,
    last_socket_sequence: Option<u64>,
}

impl SequenceTracker {
    pub fn check_sequence(&mut self, seq: u64, socket_seq: u64) -> bool {
        let mut missing = false;

        if let Some(last) = self.last_sequence {
            if seq != last + 1 {
                eprintln!("Missing sequence: expected {}, got {}", last + 1, seq);
                missing = true;
            }
        }

        if let Some(last_sock) = self.last_socket_sequence {
            if socket_seq != last_sock + 1 {
                eprintln!("Missing socket_sequence: expected {}, got {}", last_sock + 1, socket_seq);
                missing = true;
            }
        }

        self.last_sequence = Some(seq);
        self.last_socket_sequence = Some(socket_seq);

        !missing
    }
}
```

---

### Reconnection

Order Events WebSocket may disconnect. Implement automatic reconnection:

```rust
pub async fn maintain_order_events_connection(
    api_key: String,
    api_secret: String,
) {
    loop {
        match connect_order_events(&api_key, &api_secret).await {
            Ok(mut ws) => {
                while let Some(msg) = ws.next().await {
                    match msg {
                        Ok(Message::Text(text)) => {
                            // Process event
                            handle_order_event(&text);
                        }
                        Ok(Message::Close(_)) => {
                            eprintln!("WebSocket closed by server");
                            break;
                        }
                        Err(e) => {
                            eprintln!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }

        eprintln!("Reconnecting in 5 seconds...");
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
```

---

## Implementation Examples

### Market Data Subscription

```rust
use serde_json::json;

pub async fn subscribe_orderbook(
    ws: &mut WebSocketStream,
    symbols: Vec<&str>,
) -> Result<(), Error> {
    let subscribe_msg = json!({
        "type": "subscribe",
        "subscriptions": [
            {
                "name": "l2",
                "symbols": symbols,
            }
        ]
    });

    ws.send(Message::Text(subscribe_msg.to_string())).await?;

    Ok(())
}

pub async fn process_market_data(ws: &mut WebSocketStream) {
    while let Some(msg) = ws.next().await {
        if let Ok(Message::Text(text)) = msg {
            let event: serde_json::Value = serde_json::from_str(&text).unwrap();

            match event["type"].as_str() {
                Some("subscribed") => {
                    println!("Subscription confirmed");
                }
                Some("l2_updates") => {
                    // Process order book updates
                    let symbol = event["symbol"].as_str().unwrap();
                    let changes = event["changes"].as_array().unwrap();
                    println!("L2 update for {}: {} changes", symbol, changes.len());
                }
                Some("trade") => {
                    // Process trade
                    let price = event["price"].as_str().unwrap();
                    let qty = event["quantity"].as_str().unwrap();
                    println!("Trade: {} @ {}", qty, price);
                }
                _ => {}
            }
        }
    }
}
```

### Order Events Processing

```rust
pub async fn process_order_events(ws: &mut WebSocketStream) {
    let mut sequence_tracker = SequenceTracker::new();

    while let Some(msg) = ws.next().await {
        if let Ok(Message::Text(text)) = msg {
            let event: serde_json::Value = serde_json::from_str(&text).unwrap();

            let event_type = event["type"].as_str().unwrap();

            match event_type {
                "subscription_ack" => {
                    println!("Order events subscription active");
                }
                "heartbeat" => {
                    // Connection alive
                    let ts = event["timestampms"].as_u64().unwrap();
                    println!("Heartbeat at {}", ts);
                }
                "initial" => {
                    // Existing order
                    let order_id = event["order_id"].as_str().unwrap();
                    println!("Initial order: {}", order_id);
                }
                "accepted" => {
                    let order_id = event["order_id"].as_str().unwrap();
                    println!("Order accepted: {}", order_id);
                }
                "fill" => {
                    let order_id = event["order_id"].as_str().unwrap();
                    let fill = &event["fill"];
                    let amount = fill["amount"].as_str().unwrap();
                    let price = fill["price"].as_str().unwrap();
                    println!("Order {} filled: {} @ {}", order_id, amount, price);
                }
                "cancelled" => {
                    let order_id = event["order_id"].as_str().unwrap();
                    let reason = event["reason"].as_str().unwrap();
                    println!("Order {} cancelled: {}", order_id, reason);
                }
                _ => {
                    println!("Event: {}", event_type);
                }
            }

            // Check sequence
            if let Some(seq) = event["sequence"].as_u64() {
                if let Some(sock_seq) = event["socket_sequence"].as_u64() {
                    sequence_tracker.check_sequence(seq, sock_seq);
                }
            }
        }
    }
}
```

---

## Best Practices

### 1. Separate WebSocket Connections

- **Market Data**: One connection per symbol or group of symbols
- **Order Events**: One connection per account

**Don't**: Mix market data and order events

### 2. Handle Connection Drops

```rust
// Automatic reconnection with backoff
let mut retry_delay = Duration::from_secs(1);
const MAX_DELAY: Duration = Duration::from_secs(60);

loop {
    match connect().await {
        Ok(ws) => {
            retry_delay = Duration::from_secs(1); // Reset on success
            process_messages(ws).await;
        }
        Err(e) => {
            eprintln!("Connection failed: {}, retrying in {:?}", e, retry_delay);
            tokio::time::sleep(retry_delay).await;
            retry_delay = std::cmp::min(retry_delay * 2, MAX_DELAY);
        }
    }
}
```

### 3. Validate Sequence Numbers

Always track and validate sequence numbers to detect data loss.

### 4. Use Ping/Pong for Health

```rust
// Send ping every 30 seconds
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        ws_write.send(Message::Ping(vec![])).await.ok();
    }
});
```

### 5. Buffer Events During Reconnection

```rust
use tokio::sync::mpsc;

let (tx, mut rx) = mpsc::channel(1000);

// Event processor
tokio::spawn(async move {
    while let Some(event) = rx.recv().await {
        process_event(event);
    }
});

// WebSocket receiver (with reconnection)
loop {
    let mut ws = connect().await.unwrap();
    while let Some(msg) = ws.next().await {
        if let Ok(Message::Text(text)) = msg {
            tx.send(text).await.ok();
        }
    }
    // Reconnect
}
```

### 6. Snapshot + Updates Pattern

```rust
pub struct OrderBookManager {
    snapshot_received: bool,
    order_book: OrderBook,
}

impl OrderBookManager {
    pub fn apply_update(&mut self, changes: Vec<(String, String, String)>) {
        if !self.snapshot_received {
            // First update is snapshot
            self.order_book.reset();
            self.snapshot_received = true;
        }

        self.order_book.apply_changes(changes);
    }
}
```

---

## Summary Table

| Aspect | Market Data v2 | Order Events |
|--------|----------------|--------------|
| **URL** | `wss://api.gemini.com/v2/marketdata` | `wss://api.gemini.com/v1/order/events` |
| **Auth** | None | HMAC-SHA384 |
| **Subscription** | Subscribe message | Automatic on connect |
| **Heartbeat** | None (use ping) | Every 5 seconds |
| **Feeds** | l2, candles_* | Order events |
| **Message Format** | JSON | JSON |
| **Reconnect** | Required | Required |
| **Rate Limit** | 1 req/symbol/min | N/A |
| **Use Case** | Market data | Order tracking |

---

## References

- Market Data v2: https://docs.gemini.com/websocket/market-data/v2/about
- Order Events: https://docs.gemini.com/websocket/order-events/event-types
- WebSocket Overview: https://docs.gemini.com/websocket/overview/introduction
- Private API: https://docs.gemini.com/websocket/overview/requests/private-api
