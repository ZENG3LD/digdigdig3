# Binance WebSocket Streams

## Overview

Binance provides WebSocket streams for real-time market data and user account updates. WebSocket streams are the recommended way to receive live updates and avoid REST API rate limits.

---

## Base URLs

### Spot Trading

**Market Data Streams**:
- `wss://stream.binance.com:9443/ws/<streamName>`
- `wss://stream.binance.com:443/ws/<streamName>`
- `wss://data-stream.binance.vision/ws/<streamName>` (market data only)

**Combined Streams**:
- `wss://stream.binance.com:9443/stream?streams=<stream1>/<stream2>`

**User Data Streams**:
- `wss://stream.binance.com:9443/ws/<listenKey>`

### Futures USDT-M

**Market Data Streams**:
- `wss://fstream.binance.com/ws/<streamName>`

**Combined Streams**:
- `wss://fstream.binance.com/stream?streams=<stream1>/<stream2>`

**User Data Streams**:
- `wss://fstream.binance.com/ws/<listenKey>`

### Futures COIN-M

**Market Data Streams**:
- `wss://dstream.binance.com/ws/<streamName>`

**Combined Streams**:
- `wss://dstream.binance.com/stream?streams=<stream1>/<stream2>`

---

## Connection Requirements

### Limits

- **Maximum 1,024 streams** per connection
- **5 incoming messages per second** limit per connection
- **300 connections per 5 minutes** per IP
- **24-hour connection validity** (auto-disconnect after 24 hours)

### Ping/Pong

- Server sends **ping frame every 20 seconds**
- Client must respond with **pong within 1 minute**
- Connection closes if no pong received

### Keep-Alive

Connections remain open as long as:
1. Ping/pong is maintained
2. Connection hasn't exceeded 24 hours
3. No errors occur

---

## Stream Naming Convention

### Format

All stream names must be **lowercase**:

```
<symbol>@<streamType>
```

### Examples

```
btcusdt@ticker
ethusdt@depth
bnbusdt@trade
btcusdt@kline_1m
```

**Important**: Symbols in stream names are lowercase (different from REST API).

---

## Market Data Streams

### 1. Ticker Stream

**Stream Name**: `<symbol>@ticker`

**Update Speed**: 1000ms (1 second)

**Example**: `btcusdt@ticker`

**Message Format**:
```json
{
  "e": "24hrTicker",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "p": "0.0015",
  "P": "250.00",
  "w": "0.0018",
  "x": "0.0009",
  "c": "0.0025",
  "Q": "10",
  "b": "0.0024",
  "B": "10",
  "a": "0.0026",
  "A": "100",
  "o": "0.0010",
  "h": "0.0025",
  "l": "0.0010",
  "v": "10000",
  "q": "18",
  "O": 1672442580000,
  "C": 1672515780000,
  "F": 0,
  "L": 18150,
  "n": 18151
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `s`: Symbol
- `p`: Price change
- `P`: Price change percent
- `w`: Weighted average price
- `x`: First trade(F)-1 price
- `c`: Last price
- `Q`: Last quantity
- `b`: Best bid price
- `B`: Best bid quantity
- `a`: Best ask price
- `A`: Best ask quantity
- `o`: Open price
- `h`: High price
- `l`: Low price
- `v`: Total traded base asset volume
- `q`: Total traded quote asset volume
- `O`: Statistics open time
- `C`: Statistics close time
- `F`: First trade ID
- `L`: Last trade ID
- `n`: Total number of trades

---

### 2. Mini Ticker Stream

**Stream Name**: `<symbol>@miniTicker`

**Update Speed**: 1000ms

**Example**: `btcusdt@miniTicker`

**Message Format**:
```json
{
  "e": "24hrMiniTicker",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "c": "0.0025",
  "o": "0.0010",
  "h": "0.0025",
  "l": "0.0010",
  "v": "10000",
  "q": "18"
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `s`: Symbol
- `c`: Close price (last)
- `o`: Open price
- `h`: High price
- `l`: Low price
- `v`: Total traded base asset volume
- `q`: Total traded quote asset volume

---

### 3. All Market Tickers

**Stream Name**: `!ticker@arr`

**Update Speed**: 1000ms

**Message Format**: Array of ticker objects for all symbols

```json
[
  {
    "e": "24hrTicker",
    "E": 1672515782136,
    "s": "BTCUSDT",
    ...
  },
  {
    "e": "24hrTicker",
    "E": 1672515782136,
    "s": "ETHUSDT",
    ...
  }
]
```

---

### 4. Order Book Depth Stream

**Stream Name**: `<symbol>@depth<levels>` or `<symbol>@depth<levels>@100ms`

**Levels**: 5, 10, 20

**Update Speed**: 1000ms or 100ms

**Examples**:
- `btcusdt@depth5`
- `btcusdt@depth10@100ms`
- `ethusdt@depth20`

**Message Format**:
```json
{
  "lastUpdateId": 160,
  "bids": [
    ["0.0024", "10"]
  ],
  "asks": [
    ["0.0026", "100"]
  ]
}
```

**Fields**:
- `lastUpdateId`: Last update ID
- `bids`: Array of [price, quantity]
- `asks`: Array of [price, quantity]

**Note**: This is a snapshot, not incremental updates.

---

### 5. Diff. Depth Stream (Full Order Book)

**Stream Name**: `<symbol>@depth` or `<symbol>@depth@100ms`

**Update Speed**: 1000ms or 100ms

**Example**: `btcusdt@depth@100ms`

**Message Format**:
```json
{
  "e": "depthUpdate",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "U": 157,
  "u": 160,
  "b": [
    ["0.0024", "10"]
  ],
  "a": [
    ["0.0026", "100"]
  ]
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `s`: Symbol
- `U`: First update ID in event
- `u`: Final update ID in event
- `b`: Bids to be updated (price, quantity)
- `a`: Asks to be updated (price, quantity)

**Note**: This is incremental. Quantity "0" means remove price level.

---

### 6. Trade Stream

**Stream Name**: `<symbol>@trade`

**Update Speed**: Real-time

**Example**: `btcusdt@trade`

**Message Format**:
```json
{
  "e": "trade",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "t": 12345,
  "p": "0.001",
  "q": "100",
  "b": 88,
  "a": 50,
  "T": 1672515782136,
  "m": true,
  "M": true
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `s`: Symbol
- `t`: Trade ID
- `p`: Price
- `q`: Quantity
- `b`: Buyer order ID
- `a`: Seller order ID
- `T`: Trade time
- `m`: Is buyer market maker
- `M`: Ignore

---

### 7. Aggregate Trade Stream

**Stream Name**: `<symbol>@aggTrade`

**Update Speed**: Real-time

**Example**: `btcusdt@aggTrade`

**Message Format**:
```json
{
  "e": "aggTrade",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "a": 12345,
  "p": "0.001",
  "q": "100",
  "f": 100,
  "l": 105,
  "T": 1672515782136,
  "m": true,
  "M": true
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `s`: Symbol
- `a`: Aggregate trade ID
- `p`: Price
- `q`: Quantity
- `f`: First trade ID
- `l`: Last trade ID
- `T`: Trade time
- `m`: Is buyer market maker
- `M`: Ignore

---

### 8. Kline/Candlestick Stream

**Stream Name**: `<symbol>@kline_<interval>`

**Intervals**: 1s, 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M

**Update Speed**: Real-time (updates on every trade)

**Example**: `btcusdt@kline_1m`

**Message Format**:
```json
{
  "e": "kline",
  "E": 1672515782136,
  "s": "BTCUSDT",
  "k": {
    "t": 1672515780000,
    "T": 1672515839999,
    "s": "BTCUSDT",
    "i": "1m",
    "f": 100,
    "L": 200,
    "o": "0.0010",
    "c": "0.0020",
    "h": "0.0025",
    "l": "0.0010",
    "v": "1000",
    "n": 100,
    "x": false,
    "q": "1.0000",
    "V": "500",
    "Q": "0.500",
    "B": "123456"
  }
}
```

**Kline Object Fields**:
- `t`: Kline start time
- `T`: Kline close time
- `s`: Symbol
- `i`: Interval
- `f`: First trade ID
- `L`: Last trade ID
- `o`: Open price
- `c`: Close price
- `h`: High price
- `l`: Low price
- `v`: Base asset volume
- `n`: Number of trades
- `x`: Is this kline closed?
- `q`: Quote asset volume
- `V`: Taker buy base asset volume
- `Q`: Taker buy quote asset volume
- `B`: Ignore

**Important**: Only use kline data when `x` is `true` (kline closed).

---

### 9. Book Ticker Stream

**Stream Name**: `<symbol>@bookTicker`

**Update Speed**: Real-time (fastest)

**Example**: `btcusdt@bookTicker`

**Message Format**:
```json
{
  "u": 400900217,
  "s": "BTCUSDT",
  "b": "25.35190000",
  "B": "31.21000000",
  "a": "25.36520000",
  "A": "40.66000000"
}
```

**Fields**:
- `u`: Order book update ID
- `s`: Symbol
- `b`: Best bid price
- `B`: Best bid quantity
- `a`: Best ask price
- `A`: Best ask quantity

---

## User Data Stream

### Setup Process

1. **Create Listen Key** (REST API):
   ```
   POST /api/v3/userDataStream
   ```

   Response:
   ```json
   {
     "listenKey": "pqia91ma19a5s61cv6a81va65sdf19v8a65a1a5s61cv6a81va65sdf19v8a65a1"
   }
   ```

2. **Connect to WebSocket**:
   ```
   wss://stream.binance.com:9443/ws/<listenKey>
   ```

3. **Keepalive** (every 30-60 minutes):
   ```
   PUT /api/v3/userDataStream?listenKey=<listenKey>
   ```

4. **Close** (when done):
   ```
   DELETE /api/v3/userDataStream?listenKey=<listenKey>
   ```

### User Data Event Types

#### 1. Account Update (Balance Change)

**Event**: `outboundAccountPosition`

**Message Format**:
```json
{
  "e": "outboundAccountPosition",
  "E": 1564034571105,
  "u": 1564034571073,
  "B": [
    {
      "a": "ETH",
      "f": "10000.000000",
      "l": "0.000000"
    }
  ]
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `u`: Time of last account update
- `B`: Balances array
  - `a`: Asset
  - `f`: Free amount
  - `l`: Locked amount

---

#### 2. Balance Update

**Event**: `balanceUpdate`

**Triggers**: Deposit, withdrawal, transfer

**Message Format**:
```json
{
  "e": "balanceUpdate",
  "E": 1573200697110,
  "a": "BTC",
  "d": "100.00000000",
  "T": 1573200697068
}
```

**Fields**:
- `e`: Event type
- `E`: Event time
- `a`: Asset
- `d`: Balance delta
- `T`: Clear time

---

#### 3. Order Update

**Event**: `executionReport`

**Triggers**: Order placed, filled, canceled, expired

**Message Format**:
```json
{
  "e": "executionReport",
  "E": 1499405658658,
  "s": "ETHBTC",
  "c": "mUvoqJxFIILMdfAW5iGSOW",
  "S": "BUY",
  "o": "LIMIT",
  "f": "GTC",
  "q": "1.00000000",
  "p": "0.10264410",
  "P": "0.00000000",
  "F": "0.00000000",
  "g": -1,
  "C": "",
  "x": "NEW",
  "X": "NEW",
  "r": "NONE",
  "i": 4293153,
  "l": "0.00000000",
  "z": "0.00000000",
  "L": "0.00000000",
  "n": "0",
  "N": null,
  "T": 1499405658657,
  "t": -1,
  "I": 8641984,
  "w": true,
  "m": false,
  "M": false,
  "O": 1499405658657,
  "Z": "0.00000000",
  "Y": "0.00000000",
  "Q": "0.00000000"
}
```

**Key Fields**:
- `e`: Event type
- `E`: Event time
- `s`: Symbol
- `c`: Client order ID
- `S`: Side (BUY/SELL)
- `o`: Order type
- `f`: Time in force
- `q`: Order quantity
- `p`: Order price
- `x`: Current execution type (NEW, CANCELED, REPLACED, REJECTED, TRADE, EXPIRED)
- `X`: Current order status
- `i`: Order ID
- `l`: Last executed quantity
- `z`: Cumulative filled quantity
- `L`: Last executed price
- `n`: Commission amount
- `N`: Commission asset
- `T`: Transaction time
- `t`: Trade ID
- `w`: Is order on book?
- `m`: Is this trade maker side?
- `O`: Order creation time
- `Z`: Cumulative quote asset transacted quantity
- `Y`: Last quote asset transacted quantity

**Execution Types**:
- `NEW`: New order placed
- `CANCELED`: Order canceled
- `REPLACED`: Order replaced (cancel-replace)
- `REJECTED`: Order rejected
- `TRADE`: Order executed (partial or full)
- `EXPIRED`: Order expired

**Order Status**:
- `NEW`: Order accepted
- `PARTIALLY_FILLED`: Partially filled
- `FILLED`: Fully filled
- `CANCELED`: Canceled
- `PENDING_CANCEL`: Pending cancel
- `REJECTED`: Rejected
- `EXPIRED`: Expired

---

## Combined Streams

### Format

```
wss://stream.binance.com:9443/stream?streams=<stream1>/<stream2>/<stream3>
```

### Example

```
wss://stream.binance.com:9443/stream?streams=btcusdt@ticker/ethusdt@ticker/bnbusdt@depth
```

### Message Format

```json
{
  "stream": "btcusdt@ticker",
  "data": {
    "e": "24hrTicker",
    "E": 1672515782136,
    ...
  }
}
```

**Note**: Combined stream messages are wrapped with `stream` and `data` fields.

---

## WebSocket API Methods

### Subscribe

**Method**: SUBSCRIBE

**Request**:
```json
{
  "method": "SUBSCRIBE",
  "params": [
    "btcusdt@aggTrade",
    "btcusdt@depth"
  ],
  "id": 1
}
```

**Response**:
```json
{
  "result": null,
  "id": 1
}
```

---

### Unsubscribe

**Method**: UNSUBSCRIBE

**Request**:
```json
{
  "method": "UNSUBSCRIBE",
  "params": [
    "btcusdt@depth"
  ],
  "id": 312
}
```

**Response**:
```json
{
  "result": null,
  "id": 312
}
```

---

### List Subscriptions

**Method**: LIST_SUBSCRIPTIONS

**Request**:
```json
{
  "method": "LIST_SUBSCRIPTIONS",
  "id": 3
}
```

**Response**:
```json
{
  "result": [
    "btcusdt@aggTrade"
  ],
  "id": 3
}
```

---

## Rust Implementation Example

### Basic WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::Value;

async fn connect_websocket(streams: Vec<&str>) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "wss://stream.binance.com:9443/stream?streams={}",
        streams.join("/")
    );

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let data: Value = serde_json::from_str(&text)?;
                println!("Received: {}", data);
            }
            Message::Ping(ping) => {
                write.send(Message::Pong(ping)).await?;
            }
            Message::Close(_) => {
                println!("WebSocket closed");
                break;
            }
            _ => {}
        }
    }

    Ok(())
}

// Usage
#[tokio::main]
async fn main() {
    let streams = vec!["btcusdt@ticker", "ethusdt@depth5"];
    connect_websocket(streams).await.unwrap();
}
```

### User Data Stream

```rust
async fn create_listen_key(api_key: &str) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let response = client
        .post("https://api.binance.com/api/v3/userDataStream")
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;

    let data: Value = response.json().await?;
    Ok(data["listenKey"].as_str().unwrap().to_string())
}

async fn connect_user_data_stream(api_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let listen_key = create_listen_key(api_key).await?;
    let url = format!("wss://stream.binance.com:9443/ws/{}", listen_key);

    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Keepalive task
    let api_key_clone = api_key.to_string();
    let listen_key_clone = listen_key.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30 * 60)).await;
            keepalive_listen_key(&api_key_clone, &listen_key_clone).await.ok();
        }
    });

    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let data: Value = serde_json::from_str(&text)?;
                handle_user_data_event(data);
            }
            Message::Ping(ping) => {
                write.send(Message::Pong(ping)).await?;
            }
            _ => {}
        }
    }

    Ok(())
}

async fn keepalive_listen_key(api_key: &str, listen_key: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    client
        .put(format!("https://api.binance.com/api/v3/userDataStream?listenKey={}", listen_key))
        .header("X-MBX-APIKEY", api_key)
        .send()
        .await?;
    Ok(())
}
```

---

## Best Practices

1. **Use WebSocket for Real-Time Data**:
   - Don't poll REST endpoints
   - Subscribe to relevant streams

2. **Handle Reconnections**:
   - Implement automatic reconnect on disconnect
   - Exponential backoff for reconnection attempts

3. **Ping/Pong**:
   - Always respond to ping frames
   - Connection closes without pong

4. **Combined Streams**:
   - Use combined streams to reduce connections
   - Maximum 1,024 streams per connection

5. **User Data Stream Keepalive**:
   - Keepalive every 30-60 minutes
   - Listen key expires after 60 minutes without keepalive

6. **Buffer Management**:
   - Handle high-frequency updates (depth@100ms)
   - Use buffering for burst traffic

7. **Error Handling**:
   - Gracefully handle disconnections
   - Resubscribe to streams on reconnect

---

## References

- [Binance WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- [Binance User Data Streams](https://developers.binance.com/docs/binance-spot-api-docs/user-data-stream)
- [Binance Futures WebSocket](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-market-streams)
