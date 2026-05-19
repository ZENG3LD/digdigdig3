# Bitget WebSocket API

## Overview

Bitget provides WebSocket API for real-time streaming data including market data, order updates, and account changes. WebSocket is strongly recommended over polling REST endpoints for live data.

## WebSocket URLs

### Production

**Public Channel:**
```
wss://ws.bitget.com/v2/ws/public
```

**Private Channel:**
```
wss://ws.bitget.com/v2/ws/private
```

### Demo Trading (Testnet)

**Public Channel:**
```
wss://wspap.bitget.com/v2/ws/public
```

**Private Channel:**
```
wss://wspap.bitget.com/v2/ws/private
```

### Legacy V1 URLs

Some documentation references older URLs. Use V2 URLs for new implementations:
- V1: `wss://ws.bitget.com/mix/v1/stream` (deprecated)
- V2: `wss://ws.bitget.com/v2/ws/public` (current)

## Connection Limits

- **Subscription requests:** 240 per hour per connection
- **Maximum channels:** 1,000 subscriptions per connection
- **Recommended:** Less than 50 channels per connection
- **Message rate:** 10 messages per second (subscribe/unsubscribe/other)

Exceeding message rate will cause disconnection.

## Connection Flow

### Public Channel

1. **Connect** to `wss://ws.bitget.com/v2/ws/public`
2. **Subscribe** to desired channels
3. **Send ping** every 30 seconds
4. **Receive pong** confirmation
5. **Process** data messages

### Private Channel

1. **Connect** to `wss://ws.bitget.com/v2/ws/private`
2. **Authenticate** via login operation
3. **Wait** for login success response
4. **Subscribe** to desired channels
5. **Send ping** every 30 seconds
6. **Process** data messages

## Heartbeat Mechanism

**Critical:** Maintain heartbeat to keep connection alive.

### Client → Server: ping

Send string `"ping"` every **30 seconds**:

```json
"ping"
```

### Server → Client: pong

Server responds with string `"pong"`:

```json
"pong"
```

### Timeout Rules

- If **no pong** received: reconnect
- If server receives **no ping for 2 minutes**: server disconnects

### Implementation Example

```rust
use tokio::time::{interval, Duration};
use futures_util::{SinkExt, StreamExt};

async fn maintain_heartbeat(
    ws_write: &mut SplitSink<WebSocketStream, Message>,
) {
    let mut heartbeat = interval(Duration::from_secs(30));

    loop {
        heartbeat.tick().await;
        if let Err(e) = ws_write.send(Message::Text("ping".to_string())).await {
            eprintln!("Heartbeat failed: {}", e);
            break;
        }
    }
}
```

## Authentication (Private Channels)

Private channels require authentication before subscribing.

### Authentication Message

```json
{
  "op": "login",
  "args": [
    {
      "apiKey": "<your_api_key>",
      "passphrase": "<your_passphrase>",
      "timestamp": "<timestamp_milliseconds>",
      "sign": "<signature>"
    }
  ]
}
```

### Signature Generation

The signature for WebSocket login differs from REST API:

```
prehash = timestamp + "GET" + "/user/verify"
signature = Base64(HMAC-SHA256(prehash, secretKey))
```

**Python Example:**
```python
import hmac
import base64
import time
from hashlib import sha256

def generate_ws_signature(secret_key, timestamp):
    prehash = str(timestamp) + "GET" + "/user/verify"
    signature = hmac.new(
        secret_key.encode('utf-8'),
        prehash.encode('utf-8'),
        sha256
    ).digest()
    return base64.b64encode(signature).decode('utf-8')

# Usage
timestamp = int(time.time() * 1000)
signature = generate_ws_signature("your_secret_key", timestamp)

login_msg = {
    "op": "login",
    "args": [{
        "apiKey": "your_api_key",
        "passphrase": "your_passphrase",
        "timestamp": str(timestamp),
        "sign": signature
    }]
}
```

**Rust Example:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use base64::{Engine as _, engine::general_purpose};

type HmacSha256 = Hmac<Sha256>;

fn generate_ws_signature(secret_key: &str, timestamp: i64) -> String {
    let prehash = format!("{}GET/user/verify", timestamp);

    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(prehash.as_bytes());
    let result = mac.finalize();

    general_purpose::STANDARD.encode(result.into_bytes())
}

// Usage
let timestamp = chrono::Utc::now().timestamp_millis();
let signature = generate_ws_signature("your_secret_key", timestamp);

let login_msg = serde_json::json!({
    "op": "login",
    "args": [{
        "apiKey": "your_api_key",
        "passphrase": "your_passphrase",
        "timestamp": timestamp.to_string(),
        "sign": signature
    }]
});
```

### Login Response

**Success:**
```json
{
  "event": "login",
  "code": "0",
  "msg": "success"
}
```

**Failure:**
```json
{
  "event": "error",
  "code": "40018",
  "msg": "Invalid ACCESS-SIGN"
}
```

## Subscription Format

### Subscribe Request

```json
{
  "op": "subscribe",
  "args": [
    {
      "instType": "SPOT",
      "channel": "ticker",
      "instId": "BTCUSDT"
    },
    {
      "instType": "SPOT",
      "channel": "candle5m",
      "instId": "BTCUSDT"
    }
  ]
}
```

**Fields:**
- `op`: Operation ("subscribe" or "unsubscribe")
- `args`: Array of channel subscriptions
  - `instType`: Market type (SPOT, USDT-FUTURES, etc.)
  - `channel`: Channel name
  - `instId`: Symbol (without suffix, e.g., "BTCUSDT" not "BTCUSDT_SPBL")

### Subscription Response

**Success:**
```json
{
  "event": "subscribe",
  "arg": {
    "instType": "SPOT",
    "channel": "ticker",
    "instId": "BTCUSDT"
  }
}
```

**Error:**
```json
{
  "event": "error",
  "code": "60012",
  "msg": "Invalid channel",
  "arg": {
    "instType": "SPOT",
    "channel": "invalid_channel",
    "instId": "BTCUSDT"
  }
}
```

### Unsubscribe Request

```json
{
  "op": "unsubscribe",
  "args": [
    {
      "instType": "SPOT",
      "channel": "ticker",
      "instId": "BTCUSDT"
    }
  ]
}
```

## Market Types (instType)

| instType | Description | Markets |
|----------|-------------|---------|
| `SPOT` | Spot trading | BTC/USDT, ETH/USDT, etc. |
| `USDT-FUTURES` | USDT-margined perpetual futures | BTCUSDT perp, etc. |
| `COIN-FUTURES` | Coin-margined perpetual futures | BTCUSD perp, etc. |
| `USDC-FUTURES` | USDC-margined perpetual futures | BTCUSDC perp, etc. |

## Public Channels

### Ticker Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "ticker",
    "instId": "BTCUSDT"
  }]
}
```

**Data Message:**
```json
{
  "action": "snapshot",
  "arg": {
    "instType": "SPOT",
    "channel": "ticker",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "last": "50500.00",
      "open24h": "50000.00",
      "high24h": "52000.00",
      "low24h": "49000.00",
      "bestBid": "50499.50",
      "bestAsk": "50500.50",
      "baseVolume": "3000.5500",
      "quoteVolume": "150000000.50",
      "ts": "1695806875837",
      "labeId": 0,
      "openUtc": "50100.00",
      "changeUtc24h": "0.008",
      "change24h": "0.01"
    }
  ]
}
```

**Update frequency:** Real-time on price change

### All Tickers Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "tickers"
  }]
}
```

Note: No `instId` field - subscribes to all symbols.

### Candle/Kline Channel

**Available Timeframes:**
- `candle1m`, `candle3m`, `candle5m`, `candle15m`, `candle30m`
- `candle1H`, `candle2H`, `candle4H`, `candle6H`, `candle12H`
- `candle1D`, `candle3D`, `candle1W`, `candle1M`

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "candle5m",
    "instId": "BTCUSDT"
  }]
}
```

**Data Message:**
```json
{
  "action": "update",
  "arg": {
    "instType": "SPOT",
    "channel": "candle5m",
    "instId": "BTCUSDT"
  },
  "data": [
    [
      "1695806400000",
      "50000.00",
      "50500.00",
      "49800.00",
      "50200.00",
      "100.5000",
      "5025000.00",
      "5025000.00"
    ]
  ]
}
```

**Array format:** `[timestamp, open, high, low, close, baseVolume, quoteVolume, usdtVolume]`

### Order Book (Depth) Channel

**Depth Levels:**
- `books`: Full order book
- `books5`: Top 5 levels
- `books15`: Top 15 levels

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "books",
    "instId": "BTCUSDT"
  }]
}
```

**Data Message:**
```json
{
  "action": "snapshot",
  "arg": {
    "instType": "SPOT",
    "channel": "books",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "asks": [
        ["50500.50", "0.1000"],
        ["50501.00", "0.2000"]
      ],
      "bids": [
        ["50499.50", "0.1500"],
        ["50499.00", "0.2500"]
      ],
      "ts": "1695806875837"
    }
  ]
}
```

**Update types:**
- `snapshot`: Full order book (initial)
- `update`: Incremental updates

### Trade Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "trade",
    "instId": "BTCUSDT"
  }]
}
```

**Data Message:**
```json
{
  "action": "update",
  "arg": {
    "instType": "SPOT",
    "channel": "trade",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "tradeId": "1234567890",
      "px": "50500.00",
      "sz": "0.1000",
      "side": "buy",
      "ts": "1695806875837"
    }
  ]
}
```

### Funding Rate Channel (Futures)

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "USDT-FUTURES",
    "channel": "funding-rate",
    "instId": "BTCUSDT"
  }]
}
```

**Data Message:**
```json
{
  "action": "snapshot",
  "arg": {
    "instType": "USDT-FUTURES",
    "channel": "funding-rate",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "fundingRate": "0.0001",
      "nextFundingTime": "1695808000000"
    }
  ]
}
```

## Private Channels

### Orders Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "orders",
    "instId": "default"
  }]
}
```

**Note:** Use `"instId": "default"` to subscribe to all symbols, or specify symbol.

**Data Message:**
```json
{
  "action": "update",
  "arg": {
    "instType": "SPOT",
    "channel": "orders",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "ordId": "1098394857234",
      "clOrdId": "custom_id_123",
      "px": "50000.00",
      "sz": "0.1000",
      "notionalUsd": "5000.00",
      "ordType": "limit",
      "side": "buy",
      "fillPx": "50000.00",
      "fillSz": "0.1000",
      "state": "filled",
      "accFillSz": "0.1000",
      "fillTime": "1695806876000",
      "uTime": "1695806876000",
      "cTime": "1695806875000"
    }
  ]
}
```

**Order States:**
- `live`: Active
- `partially_filled`: Partial fill
- `filled`: Fully filled
- `canceled`: Cancelled

### Fills/Trades Channel

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "SPOT",
    "channel": "fill",
    "instId": "default"
  }]
}
```

**Data Message:**
```json
{
  "action": "update",
  "arg": {
    "instType": "SPOT",
    "channel": "fill",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "tradeId": "5678901234",
      "ordId": "1098394857234",
      "clOrdId": "custom_id_123",
      "fillPx": "50000.00",
      "fillSz": "0.1000",
      "side": "buy",
      "fillTime": "1695806876000",
      "feeCcy": "USDT",
      "fee": "10.00"
    }
  ]
}
```

### Account Channel (Futures)

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "USDT-FUTURES",
    "channel": "account",
    "coin": "default"
  }]
}
```

**Data Message:**
```json
{
  "action": "snapshot",
  "arg": {
    "instType": "USDT-FUTURES",
    "channel": "account",
    "coin": "USDT"
  },
  "data": [
    {
      "marginCoin": "USDT",
      "locked": "500.00",
      "available": "9500.50",
      "equity": "10500.00",
      "unrealizedPL": "500.00",
      "crossRiskRate": "0.0500",
      "updateTime": "1695806875837"
    }
  ]
}
```

### Positions Channel (Futures)

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "USDT-FUTURES",
    "channel": "positions",
    "instId": "default"
  }]
}
```

**Data Message:**
```json
{
  "action": "snapshot",
  "arg": {
    "instType": "USDT-FUTURES",
    "channel": "positions",
    "instId": "BTCUSDT"
  },
  "data": [
    {
      "instId": "BTCUSDT",
      "marginCoin": "USDT",
      "posId": "1234567890",
      "posSide": "long",
      "pos": "0.1000",
      "availPos": "0.1000",
      "avgPx": "50000.00",
      "upl": "100.00",
      "uplRatio": "0.01",
      "lever": "10",
      "liqPx": "45000.00",
      "markPx": "51000.00",
      "marginMode": "cross",
      "uTime": "1695806875837"
    }
  ]
}
```

### Plan Orders Channel (Stop Loss/Take Profit)

**Subscribe:**
```json
{
  "op": "subscribe",
  "args": [{
    "instType": "USDT-FUTURES",
    "channel": "orders-algo",
    "instId": "default"
  }]
}
```

## Message Actions

WebSocket messages include an `action` field indicating message type:

| Action | Description |
|--------|-------------|
| `snapshot` | Initial full data (first message after subscribe) |
| `update` | Incremental update |

## Error Codes

Common WebSocket error codes:

| Code | Message | Cause |
|------|---------|-------|
| 60012 | Invalid channel | Channel name doesn't exist |
| 60013 | Invalid instType | Market type invalid |
| 60014 | Invalid instId | Symbol doesn't exist |
| 40018 | Invalid signature | Authentication failed |
| 40019 | Request expired | Timestamp too old |

## Reconnection Strategy

Implement automatic reconnection with exponential backoff:

```rust
use tokio::time::{sleep, Duration};

async fn connect_with_retry(
    url: &str,
    max_retries: u32,
) -> Result<WebSocketStream> {
    let mut retries = 0;

    loop {
        match connect_async(url).await {
            Ok((ws_stream, _)) => return Ok(ws_stream),
            Err(e) if retries < max_retries => {
                retries += 1;
                let backoff = Duration::from_secs(2u64.pow(retries));
                eprintln!("Connection failed, retrying in {:?}: {}", backoff, e);
                sleep(backoff).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

async fn run_websocket() {
    loop {
        let ws_stream = connect_with_retry(
            "wss://ws.bitget.com/v2/ws/public",
            5
        ).await.unwrap();

        match handle_websocket(ws_stream).await {
            Ok(_) => break,
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                sleep(Duration::from_secs(5)).await;
                // Reconnect
            }
        }
    }
}
```

## Complete Implementation Example

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

async fn bitget_websocket_example() -> Result<()> {
    // Connect
    let (ws_stream, _) = connect_async("wss://ws.bitget.com/v2/ws/public")
        .await?;

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to ticker
    let subscribe_msg = json!({
        "op": "subscribe",
        "args": [{
            "instType": "SPOT",
            "channel": "ticker",
            "instId": "BTCUSDT"
        }]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Spawn heartbeat task
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(30));
        loop {
            interval.tick().await;
            if write.send(Message::Text("ping".to_string())).await.is_err() {
                break;
            }
        }
    });

    // Process messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                if text == "pong" {
                    println!("Heartbeat OK");
                    continue;
                }

                let data: serde_json::Value = serde_json::from_str(&text)?;
                println!("Received: {}", serde_json::to_string_pretty(&data)?);
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

## Best Practices

1. **Always implement heartbeat** - Connection will timeout without ping
2. **Handle reconnections** - Network issues are common
3. **Use snapshot action** - Rebuild state from snapshots
4. **Limit subscriptions** - Stay under 50 channels per connection
5. **Process messages quickly** - Don't block the receive loop
6. **Buffer messages** - Use channels for async processing
7. **Validate subscriptions** - Check for error responses
8. **Use private channels wisely** - Minimize sensitive data exposure
9. **Monitor connection health** - Track pong responses
10. **Clean up on disconnect** - Properly close connections

## WebSocket vs REST API

| Feature | WebSocket | REST API |
|---------|-----------|----------|
| Latency | Lower | Higher |
| Real-time updates | Yes | No (polling required) |
| Rate limits | Per connection | Per endpoint |
| Connection overhead | Once | Every request |
| Best for | Streaming data | One-time queries |

## Sources

- [Bitget WebSocket API](https://www.bitget.com/api-doc/common/websocket-intro)
- [Bitget WebSocket Demo Trading](https://www.bitget.com/api-doc/common/demotrading/websocket)
- [Bitget Spot WebSocket Channels](https://bitgetlimited.github.io/apidoc/en/spot/)
- [Bitget Futures WebSocket Channels](https://bitgetlimited.github.io/apidoc/en/mix/)
