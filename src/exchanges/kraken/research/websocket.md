# Kraken WebSocket API

Kraken provides WebSocket APIs for real-time market data and private account updates. There are separate WebSocket implementations for Spot (v1 and v2) and Futures trading.

---

## Spot WebSocket API

### Connection URLs

**WebSocket v2** (Recommended):
```
wss://ws.kraken.com/v2
```

**WebSocket v1** (Legacy):
```
wss://ws.kraken.com
```

**Connection Requirements**:
- Transport Layer Security (TLS) with Server Name Indication (SNI) required
- Standard WebSocket protocol (RFC 6455)
- JSON message format

---

## WebSocket v2 (Spot)

### Connection and Authentication

#### Public Channels (No Authentication)

Connect directly and subscribe:

```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD", "ETH/USD"],
    "snapshot": true
  }
}
```

#### Private Channels (Authentication Required)

**Step 1**: Get authentication token via REST API

```bash
POST /0/private/GetWebSocketsToken
```

**Step 2**: Use token in WebSocket subscription

```json
{
  "method": "subscribe",
  "params": {
    "channel": "executions",
    "token": "1Dwc4lzSwNWOAwkMdqhssNNFhs1ed606d1WcF3XfEbg",
    "snapshot": true
  }
}
```

**Token Characteristics**:
- Valid for **15 minutes** from creation
- Must establish connection and subscribe within 15 minutes
- Once connected, token remains valid for duration of connection
- One token can be used for multiple private subscriptions
- Requires API key permission: "WebSocket interface - On"

---

### Public Channels (v2)

#### 1. Ticker Channel

**Channel Name**: `ticker`

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD", "ETH/USD"],
    "event_trigger": "trades",
    "snapshot": true
  }
}
```

**Parameters**:
- `symbol`: Array of trading pair symbols (e.g., `["BTC/USD"]`)
- `event_trigger`: `"trades"` (default) or `"bbo"` (best bid/offer changes)
- `snapshot`: `true` (default) to receive initial snapshot

**Subscription Acknowledgment**:
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "snapshot": true,
    "symbol": "BTC/USD"
  },
  "success": true,
  "time_in": "2024-01-20T12:00:00.000000Z",
  "time_out": "2024-01-20T12:00:00.100000Z"
}
```

**Snapshot Message**:
```json
{
  "channel": "ticker",
  "type": "snapshot",
  "data": [
    {
      "symbol": "BTC/USD",
      "bid": 43210.0,
      "bid_qty": 5.123,
      "ask": 43210.5,
      "ask_qty": 3.456,
      "last": 43210.1,
      "volume": 1234.567,
      "vwap": 43150.5,
      "low": 43000.0,
      "high": 43500.0,
      "change": 210.0,
      "change_pct": 0.49
    }
  ]
}
```

**Update Message**:
```json
{
  "channel": "ticker",
  "type": "update",
  "data": [
    {
      "symbol": "BTC/USD",
      "bid": 43211.0,
      "ask": 43211.5,
      "last": 43211.2,
      "volume": 1235.0,
      "change": 211.2,
      "change_pct": 0.49
    }
  ]
}
```

---

#### 2. Book Channel (Order Book)

**Channel Name**: `book`

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "book",
    "symbol": ["BTC/USD"],
    "depth": 10,
    "snapshot": true
  }
}
```

**Parameters**:
- `symbol`: Array of symbols
- `depth`: Number of price levels (10, 25, 100, 500, 1000), default: 10
- `snapshot`: Boolean, default: true

**Snapshot Message**:
```json
{
  "channel": "book",
  "type": "snapshot",
  "data": [
    {
      "symbol": "BTC/USD",
      "bids": [
        {"price": 43210.0, "qty": 5.123},
        {"price": 43209.5, "qty": 2.456}
      ],
      "asks": [
        {"price": 43210.5, "qty": 3.789},
        {"price": 43211.0, "qty": 1.234}
      ],
      "checksum": 123456789,
      "timestamp": "2024-01-20T12:00:00.000000Z"
    }
  ]
}
```

**Update Message**:
```json
{
  "channel": "book",
  "type": "update",
  "data": [
    {
      "symbol": "BTC/USD",
      "bids": [
        {"price": 43210.0, "qty": 5.500}
      ],
      "asks": [],
      "checksum": 123456790,
      "timestamp": "2024-01-20T12:00:01.000000Z"
    }
  ]
}
```

**Checksum Validation**:
- CRC32 checksum of top 10 bids and asks
- Use to verify order book integrity
- Recalculate and compare on each update

---

#### 3. Trade Channel

**Channel Name**: `trade`

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "trade",
    "symbol": ["BTC/USD"],
    "snapshot": true
  }
}
```

**Trade Message**:
```json
{
  "channel": "trade",
  "type": "update",
  "data": [
    {
      "symbol": "BTC/USD",
      "side": "buy",
      "price": 43210.5,
      "qty": 0.5,
      "ord_type": "market",
      "trade_id": 12345678,
      "timestamp": "2024-01-20T12:00:00.123456Z"
    }
  ]
}
```

**Fields**:
- `side`: `"buy"` or `"sell"`
- `price`: Trade price (float)
- `qty`: Trade quantity (float)
- `ord_type`: `"market"` or `"limit"`
- `trade_id`: Unique trade identifier (integer)
- `timestamp`: RFC3339 timestamp

---

#### 4. OHLC/Candles Channel

**Channel Name**: `ohlc`

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "ohlc",
    "symbol": ["BTC/USD"],
    "interval": 60,
    "snapshot": true
  }
}
```

**Parameters**:
- `interval`: Candle interval in minutes (1, 5, 15, 30, 60, 240, 1440)

**OHLC Message**:
```json
{
  "channel": "ohlc",
  "type": "update",
  "data": [
    {
      "symbol": "BTC/USD",
      "timestamp": "2024-01-20T12:00:00.000000Z",
      "open": 43100.0,
      "high": 43250.0,
      "low": 43090.0,
      "close": 43200.0,
      "volume": 125.5,
      "trades": 1500,
      "interval": 60
    }
  ]
}
```

---

### Private Channels (v2)

#### 1. Executions Channel

**Channel Name**: `executions`

**Purpose**: Receive trade execution notifications

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "executions",
    "token": "YOUR_TOKEN_HERE",
    "snapshot": true
  }
}
```

**Execution Message**:
```json
{
  "channel": "executions",
  "type": "update",
  "data": [
    {
      "exec_id": "ABCD-1234-EFGH",
      "exec_type": "filled",
      "order_id": "ORDER-123",
      "client_order_id": "my-order-1",
      "symbol": "BTC/USD",
      "side": "buy",
      "order_qty": 1.0,
      "filled_qty": 1.0,
      "cum_qty": 1.0,
      "avg_price": 43210.0,
      "last_qty": 1.0,
      "last_price": 43210.0,
      "liquidity_ind": "taker",
      "timestamp": "2024-01-20T12:00:00.123456Z"
    }
  ]
}
```

---

#### 2. Balances Channel

**Channel Name**: `balances`

**Purpose**: Real-time balance updates

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "balances",
    "token": "YOUR_TOKEN_HERE",
    "snapshot": true
  }
}
```

**Balance Message**:
```json
{
  "channel": "balances",
  "type": "snapshot",
  "data": [
    {
      "asset": "USD",
      "balance": 10000.50
    },
    {
      "asset": "BTC",
      "balance": 0.12345678
    }
  ]
}
```

---

### WebSocket v2 Features

#### Ping/Pong

**Client → Server (Ping)**:
```json
{
  "method": "ping",
  "req_id": 12345
}
```

**Server → Client (Pong)**:
```json
{
  "method": "pong",
  "req_id": 12345,
  "time_in": "2024-01-20T12:00:00.000000Z",
  "time_out": "2024-01-20T12:00:00.001000Z"
}
```

**Purpose**: Application-level keepalive (distinct from WebSocket protocol ping/pong)

---

#### Unsubscribe

```json
{
  "method": "unsubscribe",
  "params": {
    "channel": "ticker",
    "symbol": ["BTC/USD"]
  }
}
```

---

## WebSocket v1 (Spot - Legacy)

### Connection URL

```
wss://ws.kraken.com
```

### Key Differences from v2

- Uses numeric channel IDs instead of named channels
- Symbol format: `XBT/USD` (not `BTC/USD`)
- Different message structure
- Less streamlined API

### Example Subscription (v1)

```json
{
  "event": "subscribe",
  "pair": ["XBT/USD"],
  "subscription": {
    "name": "ticker"
  }
}
```

**Recommendation**: Use v2 for new implementations

---

## Futures WebSocket API

### Connection URL

**Production**:
```
wss://futures.kraken.com/ws/v1
```

**Demo**:
```
wss://demo-futures.kraken.com/ws/v1
```

### Connection Maintenance

**Critical**: Must send **ping every 60 seconds** to keep connection alive

**Ping Message**:
```json
{
  "event": "ping"
}
```

**Pong Response**:
```json
{
  "event": "pong"
}
```

---

### Authentication (Futures)

Futures WebSocket uses **challenge-response authentication**.

**Step 1: Request Challenge**

Send request for challenge string (returns UUID).

**Step 2: Sign Challenge**

```rust
use sha2::{Digest, Sha256, Sha512};
use hmac::{Hmac, Mac};
use base64;

type HmacSha512 = Hmac<Sha512>;

fn sign_challenge(challenge: &str, api_secret: &str) -> Result<String, Error> {
    // 1. SHA-256 hash of challenge
    let mut hasher = Sha256::new();
    hasher.update(challenge.as_bytes());
    let challenge_hash = hasher.finalize();

    // 2. Base64 decode API secret
    let secret_decoded = base64::decode(api_secret)?;

    // 3. HMAC-SHA512
    let mut mac = HmacSha512::new_from_slice(&secret_decoded)?;
    mac.update(&challenge_hash);
    let signature = mac.finalize().into_bytes();

    // 4. Base64 encode
    Ok(base64::encode(&signature))
}
```

**Step 3: Subscribe with Signed Challenge**

```json
{
  "event": "subscribe",
  "feed": "fills",
  "api_key": "your_api_key",
  "original_challenge": "challenge_uuid",
  "signed_challenge": "base64_signature"
}
```

---

### Public Channels (Futures)

#### 1. Ticker Feed

**Feed Name**: `ticker`

**Subscription**:
```json
{
  "event": "subscribe",
  "feed": "ticker",
  "product_ids": ["PI_XBTUSD", "PI_ETHUSD"]
}
```

**Subscription Confirmation**:
```json
{
  "event": "subscribed",
  "feed": "ticker",
  "product_ids": ["PI_XBTUSD"]
}
```

**Ticker Message**:
```json
{
  "feed": "ticker",
  "product_id": "PI_XBTUSD",
  "bid": 43210.0,
  "ask": 43210.5,
  "bid_size": 5000,
  "ask_size": 3000,
  "last": 43210.0,
  "volume": 1234567,
  "change": 210.0,
  "open": 43000.0,
  "high": 43500.0,
  "low": 42900.0,
  "funding_rate": 0.0001,
  "funding_rate_prediction": 0.00012,
  "relative_funding_rate": 0.01,
  "next_funding_rate_time": 1705752000000,
  "markPrice": 43210.5,
  "index": 43209.8,
  "time": 1705752000000,
  "seq": 12345
}
```

**Throttling**: Updates published every ~1 second

---

#### 2. Book Feed (Futures)

**Feed Name**: `book`

**Subscription**:
```json
{
  "event": "subscribe",
  "feed": "book",
  "product_ids": ["PI_XBTUSD"]
}
```

**Book Snapshot**:
```json
{
  "feed": "book_snapshot",
  "product_id": "PI_XBTUSD",
  "bids": [
    {"price": 43210.0, "qty": 5000},
    {"price": 43209.5, "qty": 3000}
  ],
  "asks": [
    {"price": 43210.5, "qty": 4000},
    {"price": 43211.0, "qty": 2000}
  ],
  "timestamp": 1705752000000,
  "seq": 12345
}
```

**Book Update**:
```json
{
  "feed": "book",
  "product_id": "PI_XBTUSD",
  "side": "buy",
  "price": 43210.0,
  "qty": 5500,
  "timestamp": 1705752001000,
  "seq": 12346
}
```

**Maintaining Order Book**:
1. Start with snapshot
2. Apply updates sequentially using `seq` number
3. If `qty` is 0, remove that price level
4. Otherwise, update or add the price level

---

#### 3. Trade Feed (Futures)

**Feed Name**: `trade`

**Subscription**:
```json
{
  "event": "subscribe",
  "feed": "trade",
  "product_ids": ["PI_XBTUSD"]
}
```

**Trade Message**:
```json
{
  "feed": "trade",
  "product_id": "PI_XBTUSD",
  "uid": "abc123",
  "side": "buy",
  "type": "fill",
  "price": 43210.5,
  "qty": 1000,
  "time": 1705752000000,
  "seq": 12347
}
```

---

### Private Channels (Futures)

#### Fills Feed

**Feed Name**: `fills`

**Purpose**: Receive notifications when your orders are filled

**Subscription**:
```json
{
  "event": "subscribe",
  "feed": "fills",
  "api_key": "your_api_key",
  "original_challenge": "challenge_uuid",
  "signed_challenge": "signed_challenge"
}
```

**Fill Message**:
```json
{
  "feed": "fills",
  "username": "user@example.com",
  "fills": [
    {
      "instrument": "PI_XBTUSD",
      "time": 1705752000000,
      "price": 43210.0,
      "seq": 12348,
      "buy": true,
      "qty": 1000,
      "order_id": "e35d61dd-8a30-4d5f-a574-b5593ef0c050",
      "cli_ord_id": "my-order-123",
      "fill_id": "fill-uuid",
      "fill_type": "maker"
    }
  ]
}
```

---

## Rust WebSocket Implementation

### Basic WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

async fn connect_kraken_ws() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://ws.kraken.com/v2";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to ticker
    let subscribe_msg = json!({
        "method": "subscribe",
        "params": {
            "channel": "ticker",
            "symbol": ["BTC/USD"],
            "snapshot": true
        }
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Listen for messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                println!("Received: {}", text);
                let data: serde_json::Value = serde_json::from_str(&text)?;
                // Process message
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

---

### With Ping/Pong for Futures

```rust
use tokio::time::{interval, Duration};

async fn connect_futures_ws() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://futures.kraken.com/ws/v1";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Ping task
    let write_clone = write.clone();
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            let ping = json!({"event": "ping"});
            let _ = write_clone.send(Message::Text(ping.to_string())).await;
        }
    });

    // Subscribe
    let subscribe = json!({
        "event": "subscribe",
        "feed": "ticker",
        "product_ids": ["PI_XBTUSD"]
    });
    write.send(Message::Text(subscribe.to_string())).await?;

    // Listen
    while let Some(msg) = read.next().await {
        // Process messages
    }

    Ok(())
}
```

---

## Best Practices

### 1. Connection Management

- **Reuse connections**: Don't reconnect for each subscription
- **Handle disconnections**: Implement automatic reconnection with exponential backoff
- **Monitor heartbeat**: Detect stale connections

```rust
async fn reconnect_with_backoff(
    url: &str,
    max_retries: u32,
) -> Result<WebSocketStream, Error> {
    let mut retry = 0;
    loop {
        match connect_async(url).await {
            Ok((stream, _)) => return Ok(stream),
            Err(e) if retry < max_retries => {
                let delay = 2u64.pow(retry) * 1000;
                tokio::time::sleep(Duration::from_millis(delay)).await;
                retry += 1;
            }
            Err(e) => return Err(e.into()),
        }
    }
}
```

---

### 2. Message Handling

- **Parse incrementally**: Don't buffer entire messages
- **Handle errors gracefully**: One bad message shouldn't crash connection
- **Validate checksums**: For order book data integrity

---

### 3. Subscription Management

- **Batch subscriptions**: Subscribe to multiple symbols in one message
- **Track subscriptions**: Know what you're subscribed to
- **Unsubscribe when done**: Free server resources

---

### 4. Order Book Maintenance

```rust
use std::collections::BTreeMap;

struct OrderBook {
    bids: BTreeMap<i64, f64>, // price_scaled -> quantity
    asks: BTreeMap<i64, f64>,
    last_seq: u64,
}

impl OrderBook {
    fn apply_snapshot(&mut self, snapshot: BookSnapshot) {
        self.bids.clear();
        self.asks.clear();

        for bid in snapshot.bids {
            let price_scaled = (bid.price * 10.0) as i64;
            self.bids.insert(price_scaled, bid.qty);
        }

        for ask in snapshot.asks {
            let price_scaled = (ask.price * 10.0) as i64;
            self.asks.insert(price_scaled, ask.qty);
        }

        self.last_seq = snapshot.seq;
    }

    fn apply_update(&mut self, update: BookUpdate) {
        // Verify sequence
        if update.seq != self.last_seq + 1 {
            // Request new snapshot
            return;
        }

        let price_scaled = (update.price * 10.0) as i64;

        let book = match update.side.as_str() {
            "buy" => &mut self.bids,
            "sell" => &mut self.asks,
            _ => return,
        };

        if update.qty == 0.0 {
            book.remove(&price_scaled);
        } else {
            book.insert(price_scaled, update.qty);
        }

        self.last_seq = update.seq;
    }
}
```

---

## Summary

| Feature | Spot v2 | Spot v1 | Futures |
|---------|---------|---------|---------|
| **URL** | wss://ws.kraken.com/v2 | wss://ws.kraken.com | wss://futures.kraken.com/ws/v1 |
| **Symbol Format** | BTC/USD | XBT/USD | PI_XBTUSD |
| **Authentication** | Token (via REST) | Token (via REST) | Challenge-response |
| **Ping Required** | Optional | Optional | Every 60 seconds |
| **Public Channels** | ticker, book, trade, ohlc | ticker, spread, book, trade, ohlc | ticker, book, trade |
| **Private Channels** | executions, balances | ownTrades, openOrders | fills, account_balances_and_margins |

**Recommendations**:
- Use **WebSocket v2** for new Spot implementations
- Implement **reconnection logic** with exponential backoff
- **Ping Futures** every 30-45 seconds (well before 60-second timeout)
- **Validate checksums** for order book integrity
- **Track sequence numbers** to detect gaps
