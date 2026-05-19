# OKX API v5 WebSocket

## Overview

OKX provides WebSocket APIs for real-time market data and trading operations. WebSocket is a full-duplex protocol that enables bidirectional communication between client and server.

---

## WebSocket URLs

### Production Trading

| Type | URL |
|------|-----|
| Public | `wss://ws.okx.com:8443/ws/v5/public` |
| Private | `wss://ws.okx.com:8443/ws/v5/private` |
| Business | `wss://ws.okx.com:8443/ws/v5/business` |

### Demo Trading

| Type | URL |
|------|-----|
| Public | `wss://wspap.okx.com:8443/ws/v5/public` |
| Private | `wss://wspap.okx.com:8443/ws/v5/private` |
| Business | `wss://wspap.okx.com:8443/ws/v5/business` |

---

## Connection Types

### Public WebSocket
- **URL:** `wss://ws.okx.com:8443/ws/v5/public`
- **Authentication:** Not required
- **Channels:** Market data (tickers, order books, trades, candles, funding rates)
- **Rate Limit:** 3 subscriptions per second

### Private WebSocket
- **URL:** `wss://ws.okx.com:8443/ws/v5/private`
- **Authentication:** Required (login operation)
- **Channels:** Account, positions, orders, balance
- **Rate Limit:** 480 login/subscribe/unsubscribe operations per hour

### Business WebSocket
- **URL:** `wss://ws.okx.com:8443/ws/v5/business`
- **Authentication:** Required
- **Channels:** Deposit, withdrawal, and other business operations
- **Use Case:** Specialized business operations

---

## Message Format

### Standard Request

All WebSocket requests follow this JSON structure:

```json
{
  "op": "operation_type",
  "args": [
    {
      "channel": "channel_name",
      "instId": "instrument_id"
    }
  ]
}
```

**Fields:**
- `op`: Operation type (`subscribe`, `unsubscribe`, `login`)
- `args`: Array of argument objects
  - `channel`: Channel name
  - `instId`: Instrument ID (optional for some channels)
  - `instType`: Instrument type (optional, for filtering)

### Standard Response

```json
{
  "event": "event_type",
  "arg": {
    "channel": "channel_name",
    "instId": "instrument_id"
  },
  "connId": "connection_id"
}
```

**Fields:**
- `event`: Event type (`subscribe`, `unsubscribe`, `error`, `login`)
- `arg`: Echo of subscription arguments
- `connId`: Connection ID (for tracking)

### Data Push

```json
{
  "arg": {
    "channel": "channel_name",
    "instId": "instrument_id"
  },
  "data": [
    {
      // Channel-specific data
    }
  ]
}
```

---

## Authentication (Private Channels)

### Login Message

To access private channels, send a login message after connecting:

```json
{
  "op": "login",
  "args": [
    {
      "apiKey": "37c541a1-XXXX-XXXX-XXXX-10840aXXXXX",
      "passphrase": "MyPassphrase123",
      "timestamp": "2020-12-08T09:08:57.715Z",
      "sign": "VMrVeqsGTDI2vqAzIPEW0aQ...=="
    }
  ]
}
```

### Signature Generation

**Pre-hash String:**
```
timestamp + "GET" + "/users/self/verify"
```

**Example:**
```
2020-12-08T09:08:57.715ZGET/users/self/verify
```

**Sign with HMAC SHA256:**
```
signature = Base64(HMAC-SHA256(prehash, SecretKey))
```

**Rust Example:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::Engine;

type HmacSha256 = Hmac<Sha256>;

fn generate_ws_signature(timestamp: &str, secret: &str) -> String {
    let prehash = format!("{}GET/users/self/verify", timestamp);

    let mut mac = HmacSha256::new_from_slice(secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(prehash.as_bytes());

    let result = mac.finalize();
    base64::engine::general_purpose::STANDARD.encode(result.into_bytes())
}
```

### Login Response

**Success:**
```json
{
  "event": "login",
  "code": "0",
  "msg": "",
  "connId": "a4d3ae55"
}
```

**Failure:**
```json
{
  "event": "error",
  "code": "60009",
  "msg": "Login failed. Invalid signature"
}
```

---

## Public Channels

### Tickers Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "tickers",
      "instId": "BTC-USDT"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "tickers",
    "instId": "BTC-USDT"
  },
  "data": [
    {
      "instType": "SPOT",
      "instId": "BTC-USDT",
      "last": "43250.5",
      "lastSz": "0.15",
      "askPx": "43251.0",
      "askSz": "2.5",
      "bidPx": "43250.0",
      "bidSz": "3.2",
      "open24h": "42800.0",
      "high24h": "43500.0",
      "low24h": "42500.0",
      "vol24h": "1850.25",
      "volCcy24h": "79852341.25",
      "ts": "1672841403093"
    }
  ]
}
```

### Order Book Channels

**Available Depths:**
- `books` - 400 levels
- `books5` - 5 levels (fast updates)
- `books-l2-tbt` - 400 levels tick-by-tick
- `books50-l2-tbt` - 50 levels tick-by-tick

**Subscribe (books5):**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "books5",
      "instId": "BTC-USDT"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "books5",
    "instId": "BTC-USDT"
  },
  "action": "snapshot",
  "data": [
    {
      "asks": [
        ["43251.5", "1.2", "0", "3"],
        ["43252.0", "2.5", "0", "4"]
      ],
      "bids": [
        ["43250.0", "1.8", "0", "2"],
        ["43249.5", "3.1", "0", "5"]
      ],
      "ts": "1672841403093",
      "checksum": -123456789
    }
  ]
}
```

**Action Types:**
- `snapshot` - Full order book snapshot
- `update` - Incremental update

**Checksum:** CRC32 checksum for data integrity verification

### Trades Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "trades",
      "instId": "BTC-USDT"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "trades",
    "instId": "BTC-USDT"
  },
  "data": [
    {
      "instId": "BTC-USDT",
      "tradeId": "130639474",
      "px": "43250.5",
      "sz": "0.15",
      "side": "buy",
      "ts": "1672841403093"
    }
  ]
}
```

### Candlesticks Channel

**Bar Sizes:**
- `1m`, `3m`, `5m`, `15m`, `30m`
- `1H`, `2H`, `4H`, `6H`, `12H`
- `1D`, `1W`, `1M`, `3M`, `6M`, `1Y`
- UTC variants: `6Hutc`, `12Hutc`, `1Dutc`, `1Wutc`, `1Mutc`, `3Mutc`, `6Mutc`, `1Yutc`

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "candle1m",
      "instId": "BTC-USDT"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "candle1m",
    "instId": "BTC-USDT"
  },
  "data": [
    [
      "1672840800000",
      "43200.0",
      "43350.0",
      "43150.0",
      "43250.5",
      "125.8",
      "5432108.9",
      "5432108.9",
      "1"
    ]
  ]
}
```

**Array Format:** `[timestamp, open, high, low, close, vol, volCcy, volCcyQuote, confirm]`

### Funding Rate Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "funding-rate",
      "instId": "BTC-USDT-SWAP"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "funding-rate",
    "instId": "BTC-USDT-SWAP"
  },
  "data": [
    {
      "instId": "BTC-USDT-SWAP",
      "instType": "SWAP",
      "fundingRate": "0.0001",
      "nextFundingRate": "0.00015",
      "fundingTime": "1672848000000",
      "nextFundingTime": "1672876800000"
    }
  ]
}
```

---

## Private Channels

### Account Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "account"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "account"
  },
  "data": [
    {
      "uTime": "1672841403093",
      "totalEq": "41624.32",
      "isoEq": "0",
      "adjEq": "41624.32",
      "details": [
        {
          "ccy": "USDT",
          "eq": "1000.5",
          "cashBal": "1000.5",
          "availBal": "950.25",
          "frozenBal": "50.25",
          "ordFrozen": "50.25",
          "upl": "0"
        }
      ]
    }
  ]
}
```

### Positions Channel

**Subscribe (All Positions):**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "positions",
      "instType": "ANY"
    }
  ]
}
```

**Subscribe (Specific Instrument):**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "positions",
      "instType": "SWAP",
      "instId": "BTC-USDT-SWAP"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "positions",
    "instType": "SWAP"
  },
  "data": [
    {
      "instId": "BTC-USDT-SWAP",
      "instType": "SWAP",
      "mgnMode": "isolated",
      "posId": "312269865356374016",
      "posSide": "long",
      "pos": "10",
      "availPos": "10",
      "avgPx": "43000.0",
      "upl": "250.5",
      "uplRatio": "0.0058",
      "lever": "10",
      "liqPx": "39500.0",
      "markPx": "43025.05",
      "margin": "4300.0",
      "mgnRatio": "0.092",
      "uTime": "1672841403093"
    }
  ]
}
```

### Orders Channel

**Subscribe (All Order Types):**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "orders",
      "instType": "ANY"
    }
  ]
}
```

**Subscribe (Specific Instrument Type):**
```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "orders",
      "instType": "SPOT"
    }
  ]
}
```

**Data Push:**
```json
{
  "arg": {
    "channel": "orders",
    "instType": "SPOT"
  },
  "data": [
    {
      "instType": "SPOT",
      "instId": "BTC-USDT",
      "ordId": "312269865356374016",
      "clOrdId": "b15",
      "px": "43200.0",
      "sz": "0.5",
      "ordType": "limit",
      "side": "buy",
      "state": "filled",
      "avgPx": "43200.0",
      "accFillSz": "0.5",
      "fillPx": "43200.0",
      "fillSz": "0.5",
      "fillTime": "1672841403093",
      "cTime": "1672841400000",
      "uTime": "1672841403093"
    }
  ]
}
```

**Order States:**
- `live` - Order active
- `partially_filled` - Partially filled
- `filled` - Completely filled
- `canceled` - Canceled
- `mmp_canceled` - Market maker protection canceled

---

## Trading via WebSocket

### Place Order

**Request:**
```json
{
  "id": "1512",
  "op": "order",
  "args": [
    {
      "instId": "BTC-USDT",
      "tdMode": "cash",
      "side": "buy",
      "ordType": "limit",
      "px": "43200",
      "sz": "0.5",
      "clOrdId": "ws_order_001"
    }
  ]
}
```

**Response:**
```json
{
  "id": "1512",
  "op": "order",
  "data": [
    {
      "ordId": "312269865356374016",
      "clOrdId": "ws_order_001",
      "tag": "",
      "sCode": "0",
      "sMsg": ""
    }
  ],
  "code": "0",
  "msg": ""
}
```

### Batch Place Orders

**Request:**
```json
{
  "id": "1513",
  "op": "batch-orders",
  "args": [
    {
      "instId": "BTC-USDT",
      "tdMode": "cash",
      "side": "buy",
      "ordType": "limit",
      "px": "43200",
      "sz": "0.5"
    },
    {
      "instId": "ETH-USDT",
      "tdMode": "cash",
      "side": "buy",
      "ordType": "limit",
      "px": "2300",
      "sz": "1"
    }
  ]
}
```

### Cancel Order

**Request:**
```json
{
  "id": "1514",
  "op": "cancel-order",
  "args": [
    {
      "instId": "BTC-USDT",
      "ordId": "312269865356374016"
    }
  ]
}
```

**Response:**
```json
{
  "id": "1514",
  "op": "cancel-order",
  "data": [
    {
      "ordId": "312269865356374016",
      "clOrdId": "ws_order_001",
      "sCode": "0",
      "sMsg": ""
    }
  ],
  "code": "0",
  "msg": ""
}
```

### Amend Order

**Request:**
```json
{
  "id": "1515",
  "op": "amend-order",
  "args": [
    {
      "instId": "BTC-USDT",
      "ordId": "312269865356374016",
      "newPx": "43300",
      "newSz": "0.8"
    }
  ]
}
```

---

## Connection Management

### Ping/Pong

**Client Ping:**
```
ping
```

**Server Pong:**
```
pong
```

**Frequency:** Send ping if no data received within 30 seconds

### Heartbeat

- Connection breaks automatically after 30 seconds of inactivity
- No data push or successful subscription within 30 seconds triggers disconnect
- Implement periodic ping to maintain connection

**Rust Example:**
```rust
use tokio::time::{interval, Duration};

async fn heartbeat_task(ws_sender: WebSocketSender) {
    let mut interval = interval(Duration::from_secs(20));

    loop {
        interval.tick().await;
        if let Err(e) = ws_sender.send("ping".into()).await {
            eprintln!("Heartbeat failed: {}", e);
            break;
        }
    }
}
```

### Reconnection Strategy

**Exponential Backoff:**
```rust
async fn connect_with_retry(url: &str, max_retries: u32) -> Result<WebSocket, Error> {
    let mut retry_count = 0;

    loop {
        match connect_async(url).await {
            Ok((ws, _)) => return Ok(ws),
            Err(e) if retry_count < max_retries => {
                let backoff = Duration::from_millis(100 * 2u64.pow(retry_count));
                tokio::time::sleep(backoff).await;
                retry_count += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Subscription Management

### Subscribe to Multiple Channels

```json
{
  "op": "subscribe",
  "args": [
    {
      "channel": "tickers",
      "instId": "BTC-USDT"
    },
    {
      "channel": "books5",
      "instId": "BTC-USDT"
    },
    {
      "channel": "trades",
      "instId": "BTC-USDT"
    }
  ]
}
```

### Unsubscribe

```json
{
  "op": "unsubscribe",
  "args": [
    {
      "channel": "tickers",
      "instId": "BTC-USDT"
    }
  ]
}
```

### Subscription Limits

- **Maximum:** 480 subscribe/unsubscribe/login operations per hour per connection
- **New Markets:** 3 subscriptions per second
- **Data Size:** Total channel length cannot exceed 64 KB
- **Channel Connections:** Maximum 30 connections per specific private channel per sub-account

---

## Error Handling

### Common Errors

| Code | Message | Description |
|------|---------|-------------|
| 60009 | Login failed | Invalid credentials or signature |
| 60012 | Invalid request | Malformed JSON or invalid parameters |
| 60013 | Channel doesn't exist | Invalid channel name |
| 60014 | Too Many Requests | Exceeded subscription rate limit (3/s) |
| `channel-conn-count-error` | Channel connection count exceeded | Exceeded 30 connection limit for private channel |

### Error Response Format

```json
{
  "event": "error",
  "code": "60012",
  "msg": "Invalid request: malformed JSON"
}
```

---

## Best Practices

### 1. Connection Management
- Implement automatic reconnection with exponential backoff
- Send periodic pings (every 20 seconds recommended)
- Handle disconnect gracefully and resubscribe

### 2. Message Handling
- Parse JSON asynchronously to avoid blocking
- Use message queues for high-frequency data
- Implement checksum validation for order books

### 3. Subscription Strategy
- Subscribe to only needed channels
- Use `instType: ANY` for account/positions if monitoring all instruments
- Unsubscribe from unused channels to free resources

### 4. Rate Limiting
- Track subscription operations (480/hour limit)
- Batch subscriptions when possible
- Respect 3 subscriptions/second limit for new markets

### 5. Order Management
- Use `clOrdId` for tracking orders
- Listen to `orders` channel for order updates
- Implement idempotency for order placement

### 6. Error Recovery
- Log all error messages with codes
- Implement retry logic for transient errors
- Alert on persistent authentication failures

---

## Implementation Example (Rust)

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

async fn okx_websocket_example() -> Result<(), Box<dyn std::error::Error>> {
    let url = "wss://ws.okx.com:8443/ws/v5/public";

    // Connect
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to ticker
    let subscribe_msg = json!({
        "op": "subscribe",
        "args": [{
            "channel": "tickers",
            "instId": "BTC-USDT"
        }]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Handle messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                if text == "pong" {
                    continue;
                }
                let data: serde_json::Value = serde_json::from_str(&text)?;
                println!("Received: {}", data);
            }
            Message::Ping(_) => {
                write.send(Message::Pong(vec![])).await?;
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

## Notes

1. **WebSocket vs REST:** Trading rate limits are shared between WebSocket and REST
2. **Data Integrity:** Use checksums for order book validation
3. **Authentication:** Private channels require login before subscription
4. **Subscription Limits:** 480 operations/hour includes login, subscribe, and unsubscribe
5. **Connection Limits:** Max 30 connections per private channel per sub-account
6. **Demo Trading:** Use `wspap.okx.com` URLs for demo environment
7. **Timestamps:** All timestamps in milliseconds (UTC)
