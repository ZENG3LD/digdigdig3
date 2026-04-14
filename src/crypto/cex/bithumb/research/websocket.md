# Bithumb WebSocket API

## Overview

Bithumb provides WebSocket APIs for real-time market data and private account updates:
- **Bithumb Korea**: Limited documentation (reverse-engineered from implementations)
- **Bithumb Pro**: Well-documented WebSocket API

---

## Bithumb Pro WebSocket API

### Connection Details

**Base URL**: `wss://global-api.bithumb.pro/message/realtime`

**Connection Methods**:

1. **Simple Connection** (subscribe after connecting):
   ```
   wss://global-api.bithumb.pro/message/realtime
   ```

2. **Subscribe on Connect** (recommended):
   ```
   wss://global-api.bithumb.pro/message/realtime?subscribe=TICKER:BTC-USDT,ORDERBOOK:BTC-USDT
   ```

### Authentication

**For Private Topics** (orders, positions, balances):

**Method 1: Headers on Connect**
```json
{
  "apiKey": "YOUR_API_KEY",
  "apiTimestamp": "1551848831000",
  "apiSignature": "generated_signature"
}
```

**Signature Generation**:
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn generate_ws_signature(api_key: &str, secret_key: &str, timestamp: i64) -> String {
    let path = "/message/realtime";
    let message = format!("{}{}{}", path, timestamp, api_key);

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).unwrap();
    mac.update(message.as_bytes());
    format!("{:x}", mac.finalize().into_bytes())
}
```

**Method 2: Auth Command After Connect**
```json
{
  "cmd": "authKey",
  "args": ["apiKey", "timestamp_ms", "signature"]
}
```

**Success Response**:
```json
{
  "code": "00000",
  "msg": "Authentication successful",
  "timestamp": 1712230310689
}
```

---

## Command Structure

### Base Format
```json
{
  "cmd": "command_name",
  "args": ["arg1", "arg2", ...]
}
```

### Available Commands

| Command | Description | Args |
|---------|-------------|------|
| `subscribe` | Subscribe to topics | `["TOPIC:SYMBOL", ...]` |
| `unSubscribe` | Unsubscribe from topics | `["TOPIC:SYMBOL", ...]` |
| `authKey` | Authenticate for private topics | `["apiKey", "timestamp", "signature"]` |
| `ping` | Heartbeat to keep connection alive | `[]` |

---

## Public Topics

### TICKER - Current Price Data

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["TICKER:BTC-USDT"]
}
```

**Response Format**:
```json
{
  "code": "00007",
  "data": {
    "c": "51000.00",      // current/close price
    "h": "52000.00",      // 24h high
    "l": "49500.00",      // 24h low
    "p": "2.00",          // 24h change percent
    "v": "12345.678",     // 24h volume
    "s": "BTC-USDT",      // symbol
    "ver": "123456789"    // version
  },
  "timestamp": 1712230310689,
  "topic": "TICKER"
}
```

**Field Details**:
- `c`: Current/close price
- `h`: 24-hour high
- `l`: 24-hour low
- `p`: 24-hour price change percentage
- `v`: 24-hour trading volume
- `s`: Symbol
- `ver`: Version number (for detecting updates)

### ORDERBOOK - Order Book Updates

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["ORDERBOOK:BTC-USDT"]
}
```

**Initial Snapshot** (code `00006`):
```json
{
  "code": "00006",
  "data": {
    "b": [                    // bids (buy orders)
      ["50000.00", "0.123"],
      ["49990.00", "0.234"],
      ["49980.00", "0.345"]
    ],
    "s": [                    // asks (sell orders)
      ["50010.00", "0.456"],
      ["50020.00", "0.567"],
      ["50030.00", "0.678"]
    ],
    "ver": "123456789",
    "symbol": "BTC-USDT"
  },
  "timestamp": 1712230310689,
  "topic": "ORDERBOOK"
}
```

**Incremental Update** (code `00007`):
```json
{
  "code": "00007",
  "data": {
    "b": [
      ["50000.00", "0.150"],    // updated quantity
      ["49995.00", "0.000"]     // quantity = 0 means remove this level
    ],
    "s": [
      ["50010.00", "0.500"]
    ],
    "ver": "123456790",
    "symbol": "BTC-USDT"
  },
  "timestamp": 1712230311689,
  "topic": "ORDERBOOK"
}
```

**Important Notes**:
- Code `00006` = complete snapshot
- Code `00007` = incremental update
- When quantity = `"0.000"`, remove that price level
- Use `ver` to detect missed updates (if version jumps, request new snapshot)

### TRADE - Recent Trades

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["TRADE:BTC-USDT"]
}
```

**Response Format**:
```json
{
  "code": "00007",
  "data": [
    {
      "p": "50000.00",      // price
      "s": "buy",           // side (buy/sell)
      "v": "0.123",         // volume/quantity
      "t": 1712230310689    // timestamp (ms)
    },
    {
      "p": "49995.00",
      "s": "sell",
      "v": "0.234",
      "t": 1712230310690
    }
  ],
  "timestamp": 1712230310691,
  "topic": "TRADE"
}
```

### CONTRACT_TICKER - Futures Ticker

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["CONTRACT_TICKER:BTC-PERP"]
}
```

**Similar format to TICKER** with additional futures-specific fields

### CONTRACT_ORDERBOOK / CONTRACT_ORDERBOOK10

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["CONTRACT_ORDERBOOK10:BTC-PERP"]
}
```

**CONTRACT_ORDERBOOK**: Full order book
**CONTRACT_ORDERBOOK10**: Top 10 levels only (recommended for bandwidth)

---

## Private Topics

**Requires Authentication** (see Authentication section above)

### ORDER - Spot Order Updates

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["ORDER"]
}
```

**Response Format**:
```json
{
  "code": "00007",
  "data": {
    "oId": "1234567890123456789",     // order ID
    "price": "50000.00",               // order price
    "quantity": "1.00000000",          // original quantity
    "dealQuantity": "0.50000000",      // filled quantity
    "side": "buy",                     // buy/sell
    "symbol": "BTC-USDT",              // trading pair
    "type": "limit",                   // order type
    "status": "trading",               // order status
    "dealPrice": "50000.00",           // average fill price
    "fee": "0.00025000",               // trading fee
    "timestamp": 1712230310689         // update timestamp
  },
  "timestamp": 1712230310689,
  "topic": "ORDER"
}
```

**Order Status Values**:
- `"pending"`: Order placed, not filled
- `"trading"`: Partially filled
- `"traded"`: Fully filled
- `"cancelled"`: Cancelled

### CONTRACT_ORDER - Futures Order Updates

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["CONTRACT_ORDER"]
}
```

**Response Format**:
```json
{
  "code": "00007",
  "data": {
    "orderId": "1234567890123456789",
    "price": "50000.00",
    "side": "buy",
    "status": "trading",
    "symbol": "BTC-PERP",
    "quantity": "1.0",
    "amountFill": "0.5",
    "leverage": "10",
    "timestamp": 1712230310689
  },
  "timestamp": 1712230310689,
  "topic": "CONTRACT_ORDER"
}
```

### CONTRACT_ASSET - Account Balance Updates

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["CONTRACT_ASSET"]
}
```

**Response Format**:
```json
{
  "code": "00007",
  "data": {
    "availableAmount": "10000.00",        // available balance
    "totalAmount": "12000.00",            // total balance
    "coin": "USDT",                       // currency
    "openOrderMarginTotal": "2000.00",    // margin used in orders
    "timestamp": 1712230310689
  },
  "timestamp": 1712230310689,
  "topic": "CONTRACT_ASSET"
}
```

### CONTRACT_POSITION - Position Updates

**Subscribe**:
```json
{
  "cmd": "subscribe",
  "args": ["CONTRACT_POSITION"]
}
```

**Response Format**:
```json
{
  "code": "00007",
  "data": {
    "symbol": "BTC-PERP",
    "amount": "1.5",                  // position size
    "entryPrice": "49500.00",         // average entry price
    "margin": "500.00",               // margin used
    "status": "1",                    // position status
    "leverage": "10",                 // leverage used
    "unrealizedPL": "750.00",         // unrealized profit/loss
    "timestamp": 1712230310689
  },
  "timestamp": 1712230310689,
  "topic": "CONTRACT_POSITION"
}
```

---

## Response Codes

| Code | Type | Description |
|------|------|-------------|
| `0` | Pong | Response to ping |
| `00000` | Success | Authentication/command success |
| `00001` | Success | Subscribe success |
| `00002` | Success | Connection success |
| `00006` | Data | Initial message (snapshot) |
| `00007` | Data | Normal message (update) |
| `10000+` | Error | Error codes |

**Common Error Codes**:
| Code | Description |
|------|-------------|
| `10001` | System error |
| `10002` | Invalid parameter |
| `10005` | Invalid API key |
| `10006` | Invalid signature |

---

## Heartbeat / Ping-Pong

**Requirement**: Send ping every **30 seconds** to keep connection alive

**Ping Command**:
```json
{
  "cmd": "ping",
  "args": []
}
```

**Pong Response**:
```json
{
  "code": "0",
  "data": "pong",
  "timestamp": 1712230310689
}
```

**Implementation**:
```rust
use tokio::time::{interval, Duration};

async fn heartbeat_loop(ws_sender: &mut WsSender) {
    let mut ticker = interval(Duration::from_secs(30));

    loop {
        ticker.tick().await;

        let ping = json!({
            "cmd": "ping",
            "args": []
        });

        if let Err(e) = ws_sender.send(ping.to_string()).await {
            eprintln!("Failed to send ping: {}", e);
            break;
        }
    }
}
```

---

## Complete Rust Implementation

### WebSocket Client Structure

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

pub struct BithumbProWebSocket {
    ws_stream: WebSocketStream,
    api_key: Option<String>,
    secret_key: Option<String>,
}

impl BithumbProWebSocket {
    pub async fn connect(
        api_key: Option<String>,
        secret_key: Option<String>,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let url = "wss://global-api.bithumb.pro/message/realtime";
        let (ws_stream, _) = connect_async(url).await?;

        let mut client = Self {
            ws_stream,
            api_key,
            secret_key,
        };

        // Authenticate if credentials provided
        if api_key.is_some() && secret_key.is_some() {
            client.authenticate().await?;
        }

        Ok(client)
    }

    async fn authenticate(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let api_key = self.api_key.as_ref().unwrap();
        let secret_key = self.secret_key.as_ref().unwrap();
        let timestamp = chrono::Utc::now().timestamp_millis();

        let signature = generate_ws_signature(api_key, secret_key, timestamp);

        let auth_cmd = json!({
            "cmd": "authKey",
            "args": [api_key, timestamp.to_string(), signature]
        });

        self.send(auth_cmd.to_string()).await?;

        // Wait for auth response
        if let Some(msg) = self.ws_stream.next().await {
            let msg = msg?;
            println!("Auth response: {}", msg);
        }

        Ok(())
    }

    pub async fn subscribe(&mut self, topics: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let subscribe_cmd = json!({
            "cmd": "subscribe",
            "args": topics
        });

        self.send(subscribe_cmd.to_string()).await?;
        Ok(())
    }

    pub async fn unsubscribe(&mut self, topics: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
        let unsubscribe_cmd = json!({
            "cmd": "unSubscribe",
            "args": topics
        });

        self.send(unsubscribe_cmd.to_string()).await?;
        Ok(())
    }

    async fn send(&mut self, msg: String) -> Result<(), Box<dyn std::error::Error>> {
        self.ws_stream.send(Message::Text(msg)).await?;
        Ok(())
    }

    pub async fn next_message(&mut self) -> Option<Result<String, Box<dyn std::error::Error>>> {
        match self.ws_stream.next().await {
            Some(Ok(Message::Text(text))) => Some(Ok(text)),
            Some(Ok(Message::Close(_))) => None,
            Some(Err(e)) => Some(Err(e.into())),
            _ => None,
        }
    }
}
```

### Usage Example

```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect without authentication (public topics only)
    let mut ws = BithumbProWebSocket::connect(None, None).await?;

    // Subscribe to ticker and orderbook
    ws.subscribe(vec![
        "TICKER:BTC-USDT",
        "ORDERBOOK:BTC-USDT",
        "TRADE:BTC-USDT"
    ]).await?;

    // Start heartbeat task
    let ws_clone = ws.clone(); // Need to implement Clone or use Arc<Mutex<>>
    tokio::spawn(async move {
        heartbeat_loop(ws_clone).await;
    });

    // Process messages
    while let Some(msg_result) = ws.next_message().await {
        let msg = msg_result?;
        println!("Received: {}", msg);

        // Parse and handle different message types
        if let Ok(parsed) = serde_json::from_str::<WebSocketMessage>(&msg) {
            handle_message(parsed).await;
        }
    }

    Ok(())
}

#[derive(Debug, Deserialize)]
struct WebSocketMessage {
    code: String,
    data: serde_json::Value,
    timestamp: i64,
    topic: Option<String>,
}

async fn handle_message(msg: WebSocketMessage) {
    match msg.topic.as_deref() {
        Some("TICKER") => {
            // Handle ticker update
            println!("Ticker update: {:?}", msg.data);
        }
        Some("ORDERBOOK") => {
            // Handle orderbook update
            if msg.code == "00006" {
                println!("Orderbook snapshot: {:?}", msg.data);
            } else if msg.code == "00007" {
                println!("Orderbook update: {:?}", msg.data);
            }
        }
        Some("TRADE") => {
            // Handle trade update
            println!("Trade: {:?}", msg.data);
        }
        Some("ORDER") => {
            // Handle order update
            println!("Order update: {:?}", msg.data);
        }
        _ => {
            println!("Other message: {:?}", msg);
        }
    }
}
```

---

## Bithumb Korea WebSocket

**Note**: Limited official documentation. Information based on reverse engineering.

### Possible Endpoint (Unconfirmed)
```
wss://pubwss.bithumb.com/pub/ws
```

### Subscription Format (Estimated)
```json
{
  "type": "ticker",
  "symbols": ["BTC_KRW"]
}
```

**Recommendation**: Use REST API for Bithumb Korea or focus on Bithumb Pro WebSocket which is well-documented.

---

## Order Book Management

### Maintain Local Order Book

```rust
use std::collections::BTreeMap;

pub struct OrderBook {
    bids: BTreeMap<String, String>,  // price -> quantity
    asks: BTreeMap<String, String>,
    version: String,
}

impl OrderBook {
    pub fn new() -> Self {
        Self {
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
            version: String::new(),
        }
    }

    pub fn apply_snapshot(&mut self, data: OrderBookData) {
        self.bids.clear();
        self.asks.clear();

        for bid in data.b {
            self.bids.insert(bid[0].clone(), bid[1].clone());
        }

        for ask in data.s {
            self.asks.insert(ask[0].clone(), ask[1].clone());
        }

        self.version = data.ver;
    }

    pub fn apply_update(&mut self, data: OrderBookData) {
        // Check version gap
        if !self.version.is_empty() {
            let current_ver: u64 = self.version.parse().unwrap_or(0);
            let new_ver: u64 = data.ver.parse().unwrap_or(0);

            if new_ver != current_ver + 1 {
                eprintln!("Version gap detected! Need to re-subscribe for snapshot.");
                return;
            }
        }

        // Update bids
        for bid in data.b {
            let price = bid[0].clone();
            let quantity = bid[1].clone();

            if quantity == "0" || quantity == "0.000" {
                self.bids.remove(&price);
            } else {
                self.bids.insert(price, quantity);
            }
        }

        // Update asks
        for ask in data.s {
            let price = ask[0].clone();
            let quantity = ask[1].clone();

            if quantity == "0" || quantity == "0.000" {
                self.asks.remove(&price);
            } else {
                self.asks.insert(price, quantity);
            }
        }

        self.version = data.ver;
    }

    pub fn get_best_bid(&self) -> Option<(&String, &String)> {
        self.bids.iter().next_back()  // Highest bid
    }

    pub fn get_best_ask(&self) -> Option<(&String, &String)> {
        self.asks.iter().next()  // Lowest ask
    }
}

#[derive(Debug, Deserialize)]
struct OrderBookData {
    b: Vec<Vec<String>>,
    s: Vec<Vec<String>>,
    ver: String,
}
```

---

## Connection Management

### Reconnection Strategy

```rust
pub async fn connect_with_retry(
    max_retries: u32,
) -> Result<BithumbProWebSocket, Box<dyn std::error::Error>> {
    let mut attempts = 0;

    loop {
        match BithumbProWebSocket::connect(None, None).await {
            Ok(ws) => return Ok(ws),
            Err(e) if attempts < max_retries => {
                attempts += 1;
                let backoff = std::cmp::min(2u64.pow(attempts), 60);
                eprintln!("Connection failed (attempt {}/{}): {}. Retrying in {}s...",
                    attempts, max_retries, e, backoff);
                tokio::time::sleep(Duration::from_secs(backoff)).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Summary

### Key Points

1. **Bithumb Pro WebSocket**: Well-documented, recommended
2. **Base URL**: `wss://global-api.bithumb.pro/message/realtime`
3. **Public Topics**: TICKER, ORDERBOOK, TRADE
4. **Private Topics**: ORDER, CONTRACT_ORDER, CONTRACT_ASSET, CONTRACT_POSITION
5. **Authentication**: Required for private topics
6. **Heartbeat**: Send ping every 30 seconds
7. **Order Book**: Track version numbers, handle snapshots/updates
8. **Reconnection**: Implement exponential backoff

### Implementation Checklist

- [ ] WebSocket connection with TLS
- [ ] Authentication for private topics
- [ ] Subscribe/unsubscribe commands
- [ ] Heartbeat/ping every 30 seconds
- [ ] Message parsing and routing
- [ ] Order book maintenance (snapshot + updates)
- [ ] Reconnection logic with backoff
- [ ] Error handling for all response codes
- [ ] Topic management (track subscriptions)
- [ ] Graceful disconnect
