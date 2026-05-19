# HTX WebSocket API

Complete WebSocket documentation for HTX (formerly Huobi) exchange.

## Overview

HTX provides three WebSocket endpoints for real-time data:

| Endpoint | Purpose | Authentication |
|----------|---------|----------------|
| `wss://api.huobi.pro/ws` | Market data | None |
| `wss://api.huobi.pro/feed` | MBP incremental updates | None |
| `wss://api.huobi.pro/ws/v2` | Account & order updates | Required |

Alternative AWS-optimized endpoints:
- `wss://api-aws.huobi.pro/ws`
- `wss://api-aws.huobi.pro/feed`
- `wss://api-aws.huobi.pro/ws/v2`

## Connection Limits

| Type | Limit | Per |
|------|-------|-----|
| Market data connections | 100 | IP address |
| Account/order connections (v2) | 10 | API key |
| MBP feed connections | 50 | IP address |

## Message Format

### Compression

All WebSocket messages are **GZIP compressed**. You must decompress before parsing.

```rust
use flate2::read::GzDecoder;
use std::io::Read;

fn decompress_message(data: &[u8]) -> Result<String, std::io::Error> {
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = String::new();
    decoder.read_to_string(&mut decompressed)?;
    Ok(decompressed)
}
```

### Encoding

Messages are UTF-8 encoded JSON after decompression.

## Heartbeat Mechanism

### Market Data WebSocket (v1)

Server sends ping every **5 seconds**:

```json
{
  "ping": 1629384000000
}
```

Client must respond with pong within **2 consecutive pings**:

```json
{
  "pong": 1629384000000
}
```

**Important:** If client fails to respond to 2 consecutive pings, server disconnects.

### Account/Order WebSocket (v2)

Server sends ping every **20 seconds**:

```json
{
  "action": "ping",
  "data": {
    "ts": 1629384000000
  }
}
```

Client must respond:

```json
{
  "action": "pong",
  "data": {
    "ts": 1629384000000
  }
}
```

## Market Data WebSocket

### Connection

```
wss://api.huobi.pro/ws
```

No authentication required for market data.

### Subscribe to Channel

```json
{
  "sub": "market.btcusdt.kline.1min",
  "id": "id1"
}
```

**Response:**
```json
{
  "id": "id1",
  "status": "ok",
  "subbed": "market.btcusdt.kline.1min",
  "ts": 1629384000000
}
```

### Unsubscribe

```json
{
  "unsub": "market.btcusdt.kline.1min",
  "id": "id2"
}
```

**Response:**
```json
{
  "id": "id2",
  "status": "ok",
  "unsubbed": "market.btcusdt.kline.1min",
  "ts": 1629384000000
}
```

### Request Data (One-time)

```json
{
  "req": "market.btcusdt.kline.1min",
  "id": "id3",
  "from": 1629380000,
  "to": 1629384000
}
```

**Response:**
```json
{
  "id": "id3",
  "rep": "market.btcusdt.kline.1min",
  "status": "ok",
  "data": [...]
}
```

**Rate limit:** Max 50 `req` requests per connection.

## Available Market Channels

### Kline/Candlestick

**Topic:** `market.$symbol.kline.$period`

**Periods:** 1min, 5min, 15min, 30min, 60min, 4hour, 1day, 1mon, 1week, 1year

**Subscribe:**
```json
{
  "sub": "market.btcusdt.kline.1min"
}
```

**Update message:**
```json
{
  "ch": "market.btcusdt.kline.1min",
  "ts": 1629384000000,
  "tick": {
    "id": 1629384000,
    "open": 50000.00,
    "close": 50100.00,
    "low": 49900.00,
    "high": 50200.00,
    "amount": 123.45,
    "vol": 6172500.00,
    "count": 1234
  }
}
```

**Fields:**
- `id`: Kline start time (Unix seconds)
- `open`: Opening price
- `close`: Closing price
- `low`: Lowest price
- `high`: Highest price
- `amount`: Volume in base currency
- `vol`: Volume in quote currency
- `count`: Number of trades

### Market Depth

**Topic:** `market.$symbol.depth.$type`

**Types:** step0, step1, step2, step3, step4, step5
- step0: No aggregation (best precision)
- step1-5: Increasing price aggregation

**Subscribe:**
```json
{
  "sub": "market.btcusdt.depth.step0"
}
```

**Update message:**
```json
{
  "ch": "market.btcusdt.depth.step0",
  "ts": 1629384000000,
  "tick": {
    "bids": [
      [50000.00, 1.5],
      [49999.00, 2.3]
    ],
    "asks": [
      [50001.00, 2.1],
      [50002.00, 1.8]
    ],
    "version": 100001234567,
    "ts": 1629384000000
  }
}
```

**Fields:**
- `bids`: [[price, amount]] - Buy orders, descending price
- `asks`: [[price, amount]] - Sell orders, ascending price
- `version`: Order book version number
- `ts`: Timestamp

### Best Bid/Offer (BBO)

**Topic:** `market.$symbol.bbo`

**Subscribe:**
```json
{
  "sub": "market.btcusdt.bbo"
}
```

**Update message:**
```json
{
  "ch": "market.btcusdt.bbo",
  "ts": 1629384000000,
  "tick": {
    "seqId": 100001234567,
    "ask": 50001.00,
    "askSize": 2.1,
    "bid": 50000.00,
    "bidSize": 1.5,
    "quoteTime": 1629384000000,
    "symbol": "btcusdt"
  }
}
```

### Trade Detail

**Topic:** `market.$symbol.trade.detail`

**Subscribe:**
```json
{
  "sub": "market.btcusdt.trade.detail"
}
```

**Update message:**
```json
{
  "ch": "market.btcusdt.trade.detail",
  "ts": 1629384000000,
  "tick": {
    "id": 100001234567,
    "ts": 1629384000000,
    "data": [
      {
        "id": 100001234567123,
        "ts": 1629384000000,
        "tradeId": 100001234567,
        "amount": 0.1,
        "price": 50000.00,
        "direction": "buy"
      }
    ]
  }
}
```

**Fields:**
- `id`: Trade ID
- `ts`: Trade timestamp
- `tradeId`: Unique trade ID
- `amount`: Trade size
- `price`: Trade price
- `direction`: "buy" or "sell"

### Market Ticker

**Topic:** `market.$symbol.detail`

**Subscribe:**
```json
{
  "sub": "market.btcusdt.detail"
}
```

**Update message:**
```json
{
  "ch": "market.btcusdt.detail",
  "ts": 1629384000000,
  "tick": {
    "id": 100001234567,
    "open": 48000.00,
    "close": 50000.00,
    "high": 51000.00,
    "low": 47500.00,
    "amount": 12345.67,
    "vol": 617283500.00,
    "count": 89472
  }
}
```

### All Market Tickers

**Topic:** `market.tickers`

**Subscribe:**
```json
{
  "sub": "market.tickers"
}
```

**Update message:**
```json
{
  "ch": "market.tickers",
  "ts": 1629384000000,
  "data": [
    {
      "symbol": "btcusdt",
      "open": 48000.00,
      "high": 51000.00,
      "low": 47500.00,
      "close": 50000.00,
      "amount": 12345.67,
      "vol": 617283500.00,
      "count": 89472,
      "bid": 49999.00,
      "bidSize": 1.5,
      "ask": 50001.00,
      "askSize": 2.1
    }
  ]
}
```

## MBP (Market By Price) Feed

### Connection

```
wss://api.huobi.pro/feed
```

Provides incremental order book updates for high-frequency trading.

### MBP Incremental Updates

**Topic:** `market.$symbol.mbp.$levels`

**Levels:** 5, 10, 20, 150, 400

**Subscribe:**
```json
{
  "sub": "market.btcusdt.mbp.150"
}
```

**Update message:**
```json
{
  "ch": "market.btcusdt.mbp.150",
  "ts": 1629384000000,
  "tick": {
    "seqNum": 100001234567,
    "prevSeqNum": 100001234566,
    "bids": [
      [50000.00, 1.5]
    ],
    "asks": [
      [50001.00, 2.1]
    ]
  }
}
```

**Fields:**
- `seqNum`: Current sequence number
- `prevSeqNum`: Previous sequence number
- `bids`: Updated bid levels
- `asks`: Updated ask levels

**Important:** Track `seqNum` to detect missing updates. Request refresh if gap detected.

### MBP Refresh

**Topic:** `market.$symbol.mbp.refresh.$levels`

**Request:**
```json
{
  "req": "market.btcusdt.mbp.refresh.150"
}
```

**Response:**
```json
{
  "rep": "market.btcusdt.mbp.refresh.150",
  "status": "ok",
  "id": "id1",
  "data": {
    "seqNum": 100001234567,
    "bids": [
      [50000.00, 1.5],
      [49999.00, 2.3]
    ],
    "asks": [
      [50001.00, 2.1],
      [50002.00, 1.8]
    ]
  }
}
```

Use refresh when:
- Detecting sequence number gap
- Initial connection
- Recovering from disconnection

## Account & Order WebSocket (v2)

### Connection

```
wss://api.huobi.pro/ws/v2
```

Requires authentication.

### Authentication

After connecting, send authentication message:

```json
{
  "action": "req",
  "ch": "auth",
  "params": {
    "authType": "api",
    "accessKey": "your-access-key",
    "signatureMethod": "HmacSHA256",
    "signatureVersion": "2.1",
    "timestamp": "2023-01-20T12:34:56",
    "signature": "computed-signature"
  }
}
```

**Signature Computation:**

Pre-sign string:
```
GET\n
api.huobi.pro\n
/ws/v2\n
accessKey=xxx&signatureMethod=HmacSHA256&signatureVersion=2.1&timestamp=2023-01-20T12:34:56
```

Compute HMAC SHA256, Base64 encode, URL encode.

**Success response:**
```json
{
  "action": "req",
  "code": 200,
  "ch": "auth",
  "data": {}
}
```

**Error response:**
```json
{
  "action": "req",
  "code": 2002,
  "ch": "auth",
  "message": "invalid signature"
}
```

**Error codes:**
- `2002`: Invalid signature
- `2003`: Invalid timestamp
- `2004`: Invalid API key

### Subscribe to Order Updates

**Topic:** `orders#${symbol}`

**Subscribe:**
```json
{
  "action": "sub",
  "ch": "orders#btcusdt"
}
```

**Success response:**
```json
{
  "action": "sub",
  "code": 200,
  "ch": "orders#btcusdt",
  "data": {}
}
```

**Order update message:**
```json
{
  "action": "push",
  "ch": "orders#btcusdt",
  "data": {
    "orderSide": "buy",
    "lastActTime": 1629384000000,
    "clientOrderId": "my-order-1",
    "orderStatus": "filled",
    "symbol": "btcusdt",
    "eventType": "trade",
    "orderId": 100001234567,
    "type": "buy-limit",
    "orderPrice": "50000.00",
    "orderSize": "0.1",
    "orderValue": "5000.00",
    "tradePrice": "50000.00",
    "tradeVolume": "0.1",
    "tradeId": 200001234567,
    "tradeTime": 1629384000000,
    "aggressor": true,
    "remainAmt": "0.0"
  }
}
```

**Fields:**
- `orderSide`: "buy" or "sell"
- `lastActTime`: Last action timestamp
- `clientOrderId`: Client order ID
- `orderStatus`: "submitted", "partial-filled", "filled", "canceled"
- `symbol`: Trading symbol
- `eventType`: "creation", "trade", "cancellation"
- `orderId`: Order ID
- `type`: Order type
- `orderPrice`: Order price
- `orderSize`: Order size
- `orderValue`: Order value
- `tradePrice`: Fill price (if trade event)
- `tradeVolume`: Fill amount (if trade event)
- `tradeId`: Trade ID (if trade event)
- `tradeTime`: Trade timestamp (if trade event)
- `aggressor`: true if taker, false if maker
- `remainAmt`: Remaining amount

**Event types:**
- `creation`: Order created
- `trade`: Order filled (partial or full)
- `cancellation`: Order canceled

### Subscribe to All Orders

**Topic:** `orders#*`

```json
{
  "action": "sub",
  "ch": "orders#*"
}
```

Receives updates for all trading pairs.

### Subscribe to Trade Clearing

**Topic:** `trade.clearing#${symbol}#${mode}`

**Modes:**
- `0`: Limit orders only
- `1`: All orders (limit + market)

**Subscribe:**
```json
{
  "action": "sub",
  "ch": "trade.clearing#btcusdt#1"
}
```

**Update message:**
```json
{
  "action": "push",
  "ch": "trade.clearing#btcusdt#1",
  "data": {
    "eventType": "trade",
    "symbol": "btcusdt",
    "orderId": 100001234567,
    "tradePrice": "50000.00",
    "tradeVolume": "0.1",
    "orderSide": "buy",
    "aggressor": true,
    "tradeId": 200001234567,
    "tradeTime": 1629384000000,
    "transactFee": "0.0002",
    "feeDeduct": "0",
    "feeDeductType": "",
    "feeCurrency": "btc",
    "accountId": 123456,
    "source": "api",
    "orderPrice": "50000.00",
    "orderSize": "0.1",
    "clientOrderId": "my-order-1",
    "orderCreateTime": 1629383990000,
    "orderStatus": "filled"
  }
}
```

**Additional fields:**
- `transactFee`: Trading fee paid
- `feeDeduct`: Fee deduction (point card, etc.)
- `feeDeductType`: Deduction type
- `feeCurrency`: Fee currency
- `accountId`: Account ID
- `source`: Order source

### Subscribe to Account Updates

**Topic:** `accounts.update#${mode}`

**Modes:**
- `0`: Balance updates only
- `1`: All account changes

**Subscribe:**
```json
{
  "action": "sub",
  "ch": "accounts.update#1"
}
```

**Update message:**
```json
{
  "action": "push",
  "ch": "accounts.update#1",
  "data": {
    "currency": "btc",
    "accountId": 123456,
    "balance": "0.5",
    "available": "0.5",
    "changeType": "order.place",
    "accountType": "trade",
    "seqNum": 100001234567,
    "changeTime": 1629384000000
  }
}
```

**Fields:**
- `currency`: Currency code
- `accountId`: Account ID
- `balance`: Total balance
- `available`: Available balance
- `changeType`: Reason for change (order.place, order.match, etc.)
- `accountType`: Account type (trade, frozen)
- `seqNum`: Sequence number
- `changeTime`: Change timestamp

**Change types:**
- `order.place`: Order placed
- `order.match`: Order matched
- `order.refund`: Order refunded
- `order.cancel`: Order canceled
- `deposit`: Deposit
- `withdraw`: Withdrawal
- `transfer`: Transfer

## WebSocket Best Practices

### 1. Handle Disconnections

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

async fn reconnect_on_disconnect() {
    loop {
        match connect_and_subscribe().await {
            Ok(_) => println!("Disconnected, reconnecting..."),
            Err(e) => {
                eprintln!("Connection error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
```

### 2. Implement Heartbeat

```rust
use tokio::time::{interval, Duration};

async fn heartbeat_loop(ws_write: &mut SplitSink) {
    let mut ticker = interval(Duration::from_secs(5));

    loop {
        ticker.tick().await;
        // Server sends ping, we respond with pong
        // Already handled by message processing
    }
}
```

### 3. Decompress Messages

```rust
use flate2::read::GzDecoder;
use std::io::Read;

fn handle_message(data: &[u8]) -> Result<String, std::io::Error> {
    let mut gz = GzDecoder::new(data);
    let mut s = String::new();
    gz.read_to_string(&mut s)?;
    Ok(s)
}
```

### 4. Track Sequence Numbers (MBP)

```rust
struct OrderBookTracker {
    last_seq: u64,
}

impl OrderBookTracker {
    fn process_update(&mut self, update: MBPUpdate) -> Result<(), String> {
        if update.seqNum != self.last_seq + 1 {
            return Err("Sequence gap detected".to_string());
        }
        self.last_seq = update.seqNum;
        Ok(())
    }
}
```

### 5. Separate Read/Write Tasks

```rust
use tokio::sync::mpsc;

async fn websocket_handler() {
    let (ws_stream, _) = connect_async("wss://api.huobi.pro/ws").await.unwrap();
    let (mut write, mut read) = ws_stream.split();

    let (tx, mut rx) = mpsc::channel(100);

    // Read task
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            // Process incoming messages
        }
    });

    // Write task
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            write.send(msg).await.unwrap();
        }
    });

    // Main task sends messages via tx
}
```

## Error Handling

### Connection Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Connection refused | Invalid URL | Check endpoint URL |
| Authentication failed | Invalid signature | Verify signature computation |
| Connection timeout | Network issue | Retry with backoff |
| Disconnected | Server/network | Implement reconnection |

### Subscription Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Invalid topic | Wrong channel name | Verify topic format |
| Unauthorized | Missing auth | Authenticate before subscribing |
| Rate limit | Too many subscriptions | Reduce subscription count |

## Rate Limits

- **Max connections:** 10 per API key (v2), 100 per IP (market data)
- **Max req requests:** 50 per connection
- **No limit on sub:** Unlimited subscriptions

## Summary

### Market Data WebSocket
- URL: `wss://api.huobi.pro/ws`
- No authentication
- GZIP compressed
- Ping/pong every 5 seconds
- Topics: kline, depth, trade, ticker, bbo

### MBP Feed
- URL: `wss://api.huobi.pro/feed`
- No authentication
- Incremental order book updates
- Track sequence numbers
- Request refresh on gaps

### Account/Order WebSocket
- URL: `wss://api.huobi.pro/ws/v2`
- Authentication required
- Topics: orders, trade.clearing, accounts.update
- Ping/pong every 20 seconds
- Max 10 connections per API key

### Key Points
1. All messages are GZIP compressed
2. Implement heartbeat (ping/pong)
3. Handle reconnections
4. Track sequence numbers (MBP)
5. Separate read/write tasks
6. Monitor for gaps and refresh when needed
