# BingX WebSocket API

Complete documentation of BingX WebSocket streams for real-time market data and user data.

---

## Overview

BingX provides WebSocket API for real-time streaming of market data and account updates. All data is GZIP-compressed and requires decompression before parsing.

---

## WebSocket Endpoints

### Market Data (Public)

**Base URL:** `wss://open-api-ws.bingx.com/market`

**Protocols:**
- Market depth streams
- Trade streams
- Kline/candlestick streams
- Ticker streams

### User Data (Authenticated)

**Base URL:** `wss://open-api-ws.bingx.com/market?listenKey=<listen_key>`

**Protocols:**
- Order updates
- Position updates
- Balance updates

---

## Connection Management

### Establishing Connection

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{SinkExt, StreamExt};

async fn connect_market_stream() -> Result<WebSocketStream, Error> {
    let url = "wss://open-api-ws.bingx.com/market";
    let (ws_stream, _) = connect_async(url).await?;
    Ok(ws_stream)
}
```

### Connection Lifecycle

1. **Connect** to WebSocket endpoint
2. **Subscribe** to desired data streams
3. **Receive** and decompress messages
4. **Handle** heartbeat (ping/pong)
5. **Reconnect** if connection drops

---

## Message Format

### Request Format

All subscription/unsubscription requests use this format:

```json
{
  "id": "unique-request-id",
  "reqType": "sub",
  "dataType": "BTC-USDT@depth"
}
```

**Fields:**
- `id` (string) - Unique request identifier (UUID recommended)
- `reqType` (string) - Request type: `sub` (subscribe) or `unsub` (unsubscribe)
- `dataType` (string) - Stream identifier (format: `SYMBOL@stream_type`)

### Response Format

All data messages follow this structure:

```json
{
  "dataType": "BTC-USDT@depth",
  "data": { ... }
}
```

**Fields:**
- `dataType` (string) - Stream identifier
- `data` (object) - Stream-specific data payload

---

## Data Decompression

All messages from the server are **GZIP-compressed** and must be decompressed:

```rust
use flate2::read::GzDecoder;
use std::io::Read;

fn decompress_message(compressed: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(compressed);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}

async fn handle_websocket_message(msg: Message) -> Result<String, Error> {
    match msg {
        Message::Binary(data) => {
            let decompressed = decompress_message(&data)?;
            Ok(decompressed)
        }
        Message::Text(text) => Ok(text),
        _ => Err(Error::UnexpectedMessageType),
    }
}
```

---

## Heartbeat Mechanism

### Server Ping

Server sends ping every **5 seconds**:

```json
{
  "ping": 1649404670162
}
```

### Client Pong

Client **must** respond with pong:

```json
{
  "pong": 1649404670162
}
```

### Implementation

```rust
async fn handle_heartbeat(
    ws_stream: &mut WebSocketStream,
    msg: &str,
) -> Result<(), Error> {
    if let Ok(ping) = serde_json::from_str::<PingMessage>(msg) {
        let pong = PongMessage { pong: ping.ping };
        let pong_text = serde_json::to_string(&pong)?;
        ws_stream.send(Message::Text(pong_text)).await?;
    }
    Ok(())
}

#[derive(Deserialize)]
struct PingMessage {
    ping: i64,
}

#[derive(Serialize)]
struct PongMessage {
    pong: i64,
}
```

**Note:** Failure to respond to pings will result in disconnection.

---

## Market Data Streams

### Depth Stream (Order Book)

**Stream Type:** `SYMBOL@depth` or `SYMBOL@depth20`

**Subscribe:**
```json
{
  "id": "24dd0e35-56a4-4f7a-af8a-394c7060909c",
  "reqType": "sub",
  "dataType": "BTC-USDT@depth"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@depth",
  "data": {
    "bids": [
      ["43302.00", "0.521000"],
      ["43301.50", "0.234000"],
      ["43301.00", "1.002000"]
    ],
    "asks": [
      ["43303.00", "0.321000"],
      ["43303.50", "0.892000"],
      ["43304.00", "0.456000"]
    ]
  }
}
```

**Data Fields:**
- `bids` (array) - Buy orders as [price, quantity]
- `asks` (array) - Sell orders as [price, quantity]

**Update Frequency:** Pushed every second with partial book depth (level 20)

**Rust Model:**
```rust
#[derive(Debug, Deserialize)]
struct DepthUpdate {
    #[serde(rename = "dataType")]
    data_type: String,
    data: DepthData,
}

#[derive(Debug, Deserialize)]
struct DepthData {
    bids: Vec<[String; 2]>,
    asks: Vec<[String; 2]>,
}
```

### Trade Stream

**Stream Type:** `SYMBOL@trade`

**Subscribe:**
```json
{
  "id": "24dd0e35-56a4-4f7a-af8a-394c7060909c",
  "reqType": "sub",
  "dataType": "BTC-USDT@trade"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@trade",
  "data": {
    "e": "trade",
    "s": "BTC-USDT",
    "t": 1649404670162,
    "p": "43302.50",
    "q": "0.125000",
    "m": false
  }
}
```

**Data Fields:**
- `e` (string) - Event type ("trade")
- `s` (string) - Symbol
- `t` (long) - Trade timestamp (milliseconds)
- `p` (string) - Trade price
- `q` (string) - Trade quantity
- `m` (boolean) - Is buyer maker (true if buyer is maker)

**Update Frequency:** Real-time, pushed when trades occur

**Rust Model:**
```rust
#[derive(Debug, Deserialize)]
struct TradeUpdate {
    #[serde(rename = "dataType")]
    data_type: String,
    data: TradeData,
}

#[derive(Debug, Deserialize)]
struct TradeData {
    e: String,
    s: String,
    t: i64,
    p: String,
    q: String,
    m: bool,
}
```

### Kline Stream

**Stream Type:** `SYMBOL@kline_INTERVAL`

**Intervals:** `1min`, `5min`, `15min`, `30min`, `60min`, `1day`

**Subscribe:**
```json
{
  "id": "24dd0e35-56a4-4f7a-af8a-394c7060909c",
  "reqType": "sub",
  "dataType": "BTC-USDT@kline_1min"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@kline_1min",
  "data": {
    "e": "kline",
    "s": "BTC-USDT",
    "k": {
      "t": 1649404800000,
      "T": 1649404859999,
      "s": "BTC-USDT",
      "i": "1min",
      "o": "43250.00",
      "c": "43302.50",
      "h": "43350.00",
      "l": "43200.00",
      "v": "125.450000"
    }
  }
}
```

**Data Fields:**
- `e` (string) - Event type ("kline")
- `s` (string) - Symbol
- `k` (object) - Kline data:
  - `t` (long) - Kline start time
  - `T` (long) - Kline close time
  - `s` (string) - Symbol
  - `i` (string) - Interval
  - `o` (string) - Open price
  - `c` (string) - Close price
  - `h` (string) - High price
  - `l` (string) - Low price
  - `v` (string) - Volume

**Update Frequency:** Pushed every second for current kline

**Rust Model:**
```rust
#[derive(Debug, Deserialize)]
struct KlineUpdate {
    #[serde(rename = "dataType")]
    data_type: String,
    data: KlineEventData,
}

#[derive(Debug, Deserialize)]
struct KlineEventData {
    e: String,
    s: String,
    k: KlineData,
}

#[derive(Debug, Deserialize)]
struct KlineData {
    t: i64,
    #[serde(rename = "T")]
    close_time: i64,
    s: String,
    i: String,
    o: String,
    c: String,
    h: String,
    l: String,
    v: String,
}
```

### Ticker Stream

**Stream Type:** `SYMBOL@ticker` or `SYMBOL@lastPrice`

**Subscribe:**
```json
{
  "id": "24dd0e35-56a4-4f7a-af8a-394c7060909c",
  "reqType": "sub",
  "dataType": "BTC-USDT@ticker"
}
```

**Response:**
```json
{
  "dataType": "BTC-USDT@ticker",
  "data": {
    "e": "24hrTicker",
    "s": "BTC-USDT",
    "p": "1250.50",
    "P": "2.98",
    "c": "43302.50",
    "h": "43500.00",
    "l": "41800.00",
    "v": "12458.250000",
    "q": "536428950.25",
    "O": 1649318270162,
    "C": 1649404670162
  }
}
```

**Data Fields:**
- `e` (string) - Event type
- `s` (string) - Symbol
- `p` (string) - Price change
- `P` (string) - Price change percent
- `c` (string) - Last price
- `h` (string) - High price
- `l` (string) - Low price
- `v` (string) - Total traded volume
- `q` (string) - Total traded quote volume
- `O` (long) - Statistics open time
- `C` (long) - Statistics close time

**Rust Model:**
```rust
#[derive(Debug, Deserialize)]
struct TickerUpdate {
    #[serde(rename = "dataType")]
    data_type: String,
    data: TickerData,
}

#[derive(Debug, Deserialize)]
struct TickerData {
    e: String,
    s: String,
    p: String,
    #[serde(rename = "P")]
    price_change_percent: String,
    c: String,
    h: String,
    l: String,
    v: String,
    q: String,
    #[serde(rename = "O")]
    open_time: i64,
    #[serde(rename = "C")]
    close_time: i64,
}
```

---

## User Data Streams

### Authentication

User data streams require a **listen key** obtained via REST API.

#### 1. Generate Listen Key

**Spot:**
```
POST /openApi/spot/v1/user/listen-key
Headers: X-BX-APIKEY: <api_key>
```

**Swap:**
```
POST /openApi/swap/v2/user/listen-key
Headers: X-BX-APIKEY: <api_key>
```

**Response:**
```json
{
  "code": 0,
  "msg": "",
  "data": {
    "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
  }
}
```

#### 2. Connect with Listen Key

```
wss://open-api-ws.bingx.com/market?listenKey=pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1
```

**Rust Example:**
```rust
async fn connect_user_stream(listen_key: &str) -> Result<WebSocketStream, Error> {
    let url = format!("wss://open-api-ws.bingx.com/market?listenKey={}", listen_key);
    let (ws_stream, _) = connect_async(&url).await?;
    Ok(ws_stream)
}
```

#### 3. Maintain Listen Key

Listen keys are valid for **1 hour**. Extend every **30 minutes**:

```rust
async fn extend_listen_key_periodically(api_key: &str) {
    let mut interval = tokio::time::interval(Duration::from_secs(30 * 60));

    loop {
        interval.tick().await;

        let url = "https://open-api.bingx.com/openApi/spot/v1/user/listen-key";
        let client = reqwest::Client::new();
        let response = client
            .put(url)
            .header("X-BX-APIKEY", api_key)
            .send()
            .await;

        match response {
            Ok(_) => println!("Listen key extended"),
            Err(e) => eprintln!("Failed to extend listen key: {}", e),
        }
    }
}
```

### Order Update Stream

**Subscribe:**
```json
{
  "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
  "dataType": "spot.executionReport"
}
```

**Response:**
```json
{
  "dataType": "spot.executionReport",
  "data": {
    "e": "executionReport",
    "s": "BTC-USDT",
    "c": "client_order_id_123",
    "S": "BUY",
    "o": "LIMIT",
    "f": "GTC",
    "q": "0.125000",
    "p": "43300.00",
    "x": "TRADE",
    "X": "FILLED",
    "i": 1234567890,
    "l": "0.125000",
    "z": "0.125000",
    "L": "43302.50",
    "n": "5.4125",
    "N": "USDT",
    "T": 1649404670162,
    "t": 28457
  }
}
```

**Data Fields:**
- `e` (string) - Event type
- `s` (string) - Symbol
- `c` (string) - Client order ID
- `S` (string) - Side (BUY/SELL)
- `o` (string) - Order type
- `f` (string) - Time in force
- `q` (string) - Order quantity
- `p` (string) - Order price
- `x` (string) - Execution type
- `X` (string) - Order status
- `i` (long) - Order ID
- `l` (string) - Last executed quantity
- `z` (string) - Cumulative filled quantity
- `L` (string) - Last executed price
- `n` (string) - Commission amount
- `N` (string) - Commission asset
- `T` (long) - Transaction time
- `t` (long) - Trade ID

**Execution Types:**
- `NEW` - New order
- `CANCELED` - Order canceled
- `REPLACED` - Order replaced
- `REJECTED` - Order rejected
- `TRADE` - Trade execution
- `EXPIRED` - Order expired

**Order Status:**
- `NEW` - Order accepted
- `PARTIALLY_FILLED` - Partially filled
- `FILLED` - Fully filled
- `CANCELED` - Canceled
- `REJECTED` - Rejected
- `EXPIRED` - Expired

**Rust Model:**
```rust
#[derive(Debug, Deserialize)]
struct OrderUpdate {
    #[serde(rename = "dataType")]
    data_type: String,
    data: OrderData,
}

#[derive(Debug, Deserialize)]
struct OrderData {
    e: String,
    s: String,
    c: String,
    #[serde(rename = "S")]
    side: String,
    o: String,
    f: String,
    q: String,
    p: String,
    x: String,
    #[serde(rename = "X")]
    status: String,
    i: i64,
    l: String,
    z: String,
    #[serde(rename = "L")]
    last_price: String,
    n: String,
    #[serde(rename = "N")]
    commission_asset: String,
    #[serde(rename = "T")]
    transaction_time: i64,
    t: i64,
}
```

### Balance Update Stream

**Subscribe:**
```json
{
  "id": "e745cd6d-d0f6-4a70-8d5a-043e4c741b40",
  "dataType": "spot.account"
}
```

**Response:**
```json
{
  "dataType": "spot.account",
  "data": {
    "e": "outboundAccountInfo",
    "E": 1649404670162,
    "B": [
      {
        "a": "USDT",
        "f": "9500.00000000",
        "l": "500.00000000"
      },
      {
        "a": "BTC",
        "f": "0.62500000",
        "l": "0.00000000"
      }
    ]
  }
}
```

**Data Fields:**
- `e` (string) - Event type
- `E` (long) - Event time
- `B` (array) - Balances:
  - `a` (string) - Asset
  - `f` (string) - Free amount
  - `l` (string) - Locked amount

**Rust Model:**
```rust
#[derive(Debug, Deserialize)]
struct BalanceUpdate {
    #[serde(rename = "dataType")]
    data_type: String,
    data: BalanceData,
}

#[derive(Debug, Deserialize)]
struct BalanceData {
    e: String,
    #[serde(rename = "E")]
    event_time: i64,
    #[serde(rename = "B")]
    balances: Vec<Balance>,
}

#[derive(Debug, Deserialize)]
struct Balance {
    a: String,
    f: String,
    l: String,
}
```

---

## Subscription Management

### Multiple Subscriptions

Subscribe to multiple streams:

```rust
async fn subscribe_multiple(
    ws_stream: &mut WebSocketStream,
    streams: Vec<&str>,
) -> Result<(), Error> {
    for stream in streams {
        let sub_msg = SubscribeMessage {
            id: uuid::Uuid::new_v4().to_string(),
            req_type: "sub".to_string(),
            data_type: stream.to_string(),
        };

        let json = serde_json::to_string(&sub_msg)?;
        ws_stream.send(Message::Text(json)).await?;

        // Small delay between subscriptions
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    Ok(())
}

// Usage
let streams = vec![
    "BTC-USDT@depth",
    "BTC-USDT@trade",
    "ETH-USDT@depth",
    "ETH-USDT@kline_1min",
];

subscribe_multiple(&mut ws_stream, streams).await?;
```

### Unsubscribe

```json
{
  "id": "24dd0e35-56a4-4f7a-af8a-394c7060909c",
  "reqType": "unsub",
  "dataType": "BTC-USDT@depth"
}
```

```rust
async fn unsubscribe(
    ws_stream: &mut WebSocketStream,
    stream: &str,
) -> Result<(), Error> {
    let unsub_msg = SubscribeMessage {
        id: uuid::Uuid::new_v4().to_string(),
        req_type: "unsub".to_string(),
        data_type: stream.to_string(),
    };

    let json = serde_json::to_string(&unsub_msg)?;
    ws_stream.send(Message::Text(json)).await?;

    Ok(())
}
```

### Subscription Limits

**Spot Trading:** Maximum **200 subscriptions** per WebSocket connection

**Recommendation:** Use 1-3 WebSocket connections, distribute subscriptions evenly

---

## Error Handling

### Connection Errors

```rust
async fn maintain_connection(
    url: &str,
    on_message: impl Fn(String) -> Result<(), Error>,
) -> Result<(), Error> {
    loop {
        match connect_and_run(url, &on_message).await {
            Ok(_) => {
                println!("WebSocket connection closed normally");
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                println!("Reconnecting in 5 seconds...");
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}

async fn connect_and_run(
    url: &str,
    on_message: &impl Fn(String) -> Result<(), Error>,
) -> Result<(), Error> {
    let (mut ws_stream, _) = connect_async(url).await?;

    // Resubscribe to streams
    resubscribe_all(&mut ws_stream).await?;

    while let Some(msg) = ws_stream.next().await {
        let msg = msg?;
        let text = handle_websocket_message(msg).await?;

        // Handle heartbeat
        handle_heartbeat(&mut ws_stream, &text).await?;

        // Process message
        on_message(text)?;
    }

    Ok(())
}
```

### Message Parse Errors

```rust
async fn process_message(json: &str) -> Result<(), Error> {
    // Try to parse as different message types
    if let Ok(depth) = serde_json::from_str::<DepthUpdate>(json) {
        handle_depth_update(depth)?;
    } else if let Ok(trade) = serde_json::from_str::<TradeUpdate>(json) {
        handle_trade_update(trade)?;
    } else if let Ok(kline) = serde_json::from_str::<KlineUpdate>(json) {
        handle_kline_update(kline)?;
    } else {
        // Unknown or unsupported message type
        eprintln!("Unknown message: {}", json);
    }

    Ok(())
}
```

---

## Complete WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message, WebSocketStream};
use futures_util::{SinkExt, StreamExt};
use flate2::read::GzDecoder;
use std::io::Read;
use serde::{Deserialize, Serialize};

pub struct BingXWebSocket {
    ws_stream: WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>,
    subscriptions: Vec<String>,
}

impl BingXWebSocket {
    pub async fn connect_market() -> Result<Self, Error> {
        let url = "wss://open-api-ws.bingx.com/market";
        let (ws_stream, _) = connect_async(url).await?;

        Ok(Self {
            ws_stream,
            subscriptions: Vec::new(),
        })
    }

    pub async fn connect_user(listen_key: &str) -> Result<Self, Error> {
        let url = format!("wss://open-api-ws.bingx.com/market?listenKey={}", listen_key);
        let (ws_stream, _) = connect_async(&url).await?;

        Ok(Self {
            ws_stream,
            subscriptions: Vec::new(),
        })
    }

    pub async fn subscribe(&mut self, data_type: &str) -> Result<(), Error> {
        let sub_msg = SubscribeMessage {
            id: uuid::Uuid::new_v4().to_string(),
            req_type: "sub".to_string(),
            data_type: data_type.to_string(),
        };

        let json = serde_json::to_string(&sub_msg)?;
        self.ws_stream.send(Message::Text(json)).await?;
        self.subscriptions.push(data_type.to_string());

        Ok(())
    }

    pub async fn unsubscribe(&mut self, data_type: &str) -> Result<(), Error> {
        let unsub_msg = SubscribeMessage {
            id: uuid::Uuid::new_v4().to_string(),
            req_type: "unsub".to_string(),
            data_type: data_type.to_string(),
        };

        let json = serde_json::to_string(&unsub_msg)?;
        self.ws_stream.send(Message::Text(json)).await?;

        self.subscriptions.retain(|s| s != data_type);

        Ok(())
    }

    pub async fn next_message(&mut self) -> Result<Option<String>, Error> {
        while let Some(msg) = self.ws_stream.next().await {
            let msg = msg?;

            let text = match msg {
                Message::Binary(data) => {
                    let mut decoder = GzDecoder::new(&data[..]);
                    let mut decompressed = String::new();
                    decoder.read_to_string(&mut decompressed)?;
                    decompressed
                }
                Message::Text(text) => text,
                Message::Close(_) => return Ok(None),
                _ => continue,
            };

            // Handle heartbeat
            if let Ok(ping) = serde_json::from_str::<PingMessage>(&text) {
                let pong = PongMessage { pong: ping.ping };
                let pong_text = serde_json::to_string(&pong)?;
                self.ws_stream.send(Message::Text(pong_text)).await?;
                continue;
            }

            return Ok(Some(text));
        }

        Ok(None)
    }

    pub async fn close(&mut self) -> Result<(), Error> {
        self.ws_stream.close(None).await?;
        Ok(())
    }
}

#[derive(Serialize)]
struct SubscribeMessage {
    id: String,
    #[serde(rename = "reqType")]
    req_type: String,
    #[serde(rename = "dataType")]
    data_type: String,
}

#[derive(Deserialize)]
struct PingMessage {
    ping: i64,
}

#[derive(Serialize)]
struct PongMessage {
    pong: i64,
}
```

**Usage:**
```rust
#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut ws = BingXWebSocket::connect_market().await?;

    // Subscribe to streams
    ws.subscribe("BTC-USDT@depth").await?;
    ws.subscribe("BTC-USDT@trade").await?;

    // Process messages
    while let Some(msg) = ws.next_message().await? {
        println!("Received: {}", msg);

        // Parse and handle specific message types
        // ...
    }

    ws.close().await?;
    Ok(())
}
```

---

## Best Practices

1. **Handle Reconnection:** Always implement automatic reconnection with exponential backoff
2. **Maintain Listen Keys:** Extend user stream listen keys every 30 minutes
3. **Decompress All Messages:** All server messages are GZIP-compressed
4. **Respond to Pings:** Always respond to heartbeat pings within 5 seconds
5. **Limit Subscriptions:** Don't exceed 200 subscriptions per connection
6. **Use Multiple Connections:** Distribute subscriptions across 2-3 connections for high-volume
7. **Parse Robustly:** Handle unknown message types gracefully
8. **Monitor Connection Health:** Track ping/pong latency and reconnect if degraded

---

## Sources

- [BingX API Docs](https://bingx-api.github.io/docs/)
- [BingX WebSocket Subscription Limits](https://bingx.com/en/support/articles/36544879951641)
- [BingX User Data Streams](https://hexdocs.pm/bingex/Bingex.User.html)
- [BingX WebSocket Demo Discussion](https://github.com/BingX-API/BingX-swap-api-doc/issues/6)
- [BingX WebSocket Domain Update](https://bingx.com/en/support/articles/13745185632527)
