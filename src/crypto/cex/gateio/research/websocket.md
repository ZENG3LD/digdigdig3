# Gate.io WebSocket API v4

**Research Date**: 2026-01-21
**Documentation**: https://www.gate.com/docs/developers/apiv4/ws/en/

---

## Table of Contents

- [WebSocket URLs](#websocket-urls)
- [Authentication](#authentication)
- [Connection and Ping/Pong](#connection-and-pingpong)
- [Subscription Format](#subscription-format)
- [Public Channels](#public-channels)
- [Private Channels](#private-channels)
- [Message Formats](#message-formats)

---

## WebSocket URLs

### Spot Trading

**Production**:
```
wss://api.gateio.ws/ws/v4/
```

**TestNet**:
```
wss://ws-testnet.gate.com/v4/ws/spot
```

### Futures Trading (USDT-margined)

**Production**:
```
wss://fx-ws.gateio.ws/v4/ws/usdt
```

**TestNet**:
```
wss://ws-testnet.gate.com/v4/ws/futures/usdt
```

### Futures Trading (BTC-margined)

**Production**:
```
wss://fx-ws.gateio.ws/v4/ws/btc
```

**TestNet**:
```
wss://fx-ws-testnet.gateio.ws/v4/ws/btc
```

---

## Authentication

### Public Channels

**No authentication required** for public market data channels:
- Tickers
- Orderbook
- Trades
- Candlesticks

**Connection**: Simply connect to WebSocket URL and subscribe to public channels.

### Private Channels

**Authentication required** for private user data channels:
- Orders
- Balances
- Positions (futures)
- User trades

**Authentication Method**: HMAC-SHA512 signature

**Authentication Flow**:

1. **Generate signature** (same as REST API):
   ```
   signature_string = "channel={channel}&event={event}&time={timestamp}"
   signature = HMAC_SHA512(api_secret, signature_string).hexdigest()
   ```

2. **Send authentication in subscription request**:
   ```json
   {
     "time": 1729100692,
     "channel": "spot.orders",
     "event": "subscribe",
     "auth": {
       "method": "api_key",
       "KEY": "your_api_key",
       "SIGN": "generated_signature"
     }
   }
   ```

**Note**: The signature string format is different from REST API. For WebSocket, it's a query-string-like format.

---

## Connection and Ping/Pong

### Connection

1. **Establish WebSocket connection** to appropriate URL
2. **Connection confirmation**: Server sends initial message
3. **Subscribe to channels** (send subscribe messages)
4. **Maintain connection** with ping/pong

### Ping/Pong Mechanism

**Purpose**: Keep connection alive and detect disconnections

**Interval**: Send ping every **10-30 seconds**

**Ping Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.ping"
}
```

**Pong Response**:
```json
{
  "time": 1729100692,
  "channel": "spot.pong",
  "event": "",
  "result": null
}
```

**Note**: Different channels for spot and futures:
- Spot: `spot.ping` / `spot.pong`
- Futures: `futures.ping` / `futures.pong`

**Timeout**: If no pong received within 10 seconds, connection is likely dead. Reconnect.

### Connection Limits

**Maximum connections per IP**: Not explicitly documented, but reasonable limits apply

**Recommended**: Use single connection with multiple subscriptions

---

## Subscription Format

### Subscribe Request

**General format**:
```json
{
  "time": 1729100692,
  "channel": "spot.tickers",
  "event": "subscribe",
  "payload": ["BTC_USDT", "ETH_USDT"]
}
```

**Fields**:
- `time` (required): Unix timestamp in **seconds** (not milliseconds)
- `channel` (required): Channel name (e.g., "spot.tickers")
- `event` (required): "subscribe" or "unsubscribe"
- `payload` (optional): Array of symbols or parameters
- `auth` (required for private channels): Authentication object

### Subscribe Response

**Success**:
```json
{
  "time": 1729100692,
  "channel": "spot.tickers",
  "event": "subscribe",
  "result": {
    "status": "success"
  }
}
```

**Error**:
```json
{
  "time": 1729100692,
  "channel": "spot.tickers",
  "event": "subscribe",
  "error": {
    "code": 1,
    "message": "invalid channel"
  }
}
```

### Unsubscribe Request

```json
{
  "time": 1729100692,
  "channel": "spot.tickers",
  "event": "unsubscribe",
  "payload": ["BTC_USDT"]
}
```

---

## Public Channels

### Spot Tickers (spot.tickers)

**Description**: Real-time ticker updates for spot trading pairs

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.tickers",
  "event": "subscribe",
  "payload": ["BTC_USDT", "ETH_USDT"]
}
```

**Update Message**:
```json
{
  "time": 1729100692,
  "time_ms": 1729100692123,
  "channel": "spot.tickers",
  "event": "update",
  "result": {
    "currency_pair": "BTC_USDT",
    "last": "48600.5",
    "lowest_ask": "48601.0",
    "highest_bid": "48600.0",
    "change_percentage": "2.5",
    "base_volume": "1234.567",
    "quote_volume": "60000000.00",
    "high_24h": "49000.0",
    "low_24h": "47500.0"
  }
}
```

**Update Frequency**: Real-time (every trade affects ticker)

---

### Spot Trades (spot.trades)

**Description**: Real-time public trades

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.trades",
  "event": "subscribe",
  "payload": ["BTC_USDT"]
}
```

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.trades",
  "event": "update",
  "result": {
    "id": 123456789,
    "create_time": 1729100692,
    "create_time_ms": "1729100692123",
    "side": "sell",
    "currency_pair": "BTC_USDT",
    "amount": "0.01",
    "price": "48600.5"
  }
}
```

**Update Frequency**: Real-time (every trade)

---

### Spot Orderbook (spot.order_book_update)

**Description**: Real-time incremental orderbook updates

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.order_book_update",
  "event": "subscribe",
  "payload": ["BTC_USDT", "1000ms"]
}
```

**Payload Parameters**:
- `[0]`: Symbol (e.g., "BTC_USDT")
- `[1]`: Update interval ("100ms", "1000ms") - optional

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.order_book_update",
  "event": "update",
  "result": {
    "t": 1729100692123,
    "e": "depthUpdate",
    "s": "BTC_USDT",
    "U": 123456,
    "u": 123457,
    "b": [
      ["48600.0", "0.5"],
      ["48595.0", "0"]
    ],
    "a": [
      ["48610.0", "1.2"]
    ]
  }
}
```

**Fields**:
- `t`: Timestamp (milliseconds)
- `s`: Symbol
- `U`: First update ID in this event
- `u`: Last update ID in this event
- `b`: Bid updates [[price, quantity], ...]
- `a`: Ask updates [[price, quantity], ...]

**Note**: If quantity is "0", remove that price level from orderbook.

**Recommended**: First fetch snapshot with REST API, then apply incremental updates.

---

### Spot Orderbook Snapshot (spot.order_book)

**Description**: Full orderbook snapshots (not incremental)

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.order_book",
  "event": "subscribe",
  "payload": ["BTC_USDT", "20", "1000ms"]
}
```

**Payload Parameters**:
- `[0]`: Symbol (e.g., "BTC_USDT")
- `[1]`: Depth limit ("5", "10", "20", "50", "100")
- `[2]`: Update interval ("100ms", "1000ms") - optional

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.order_book",
  "event": "update",
  "result": {
    "t": 1729100692123,
    "lastUpdateId": 123456,
    "s": "BTC_USDT",
    "bids": [
      ["48600.0", "0.5"],
      ["48595.0", "2.1"]
    ],
    "asks": [
      ["48610.0", "1.2"],
      ["48615.0", "0.8"]
    ]
  }
}
```

**Use case**: Simpler to implement than incremental updates, but higher bandwidth.

---

### Spot Candlesticks (spot.candlesticks)

**Description**: Real-time candlestick updates

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.candlesticks",
  "event": "subscribe",
  "payload": ["1m", "BTC_USDT"]
}
```

**Payload Parameters**:
- `[0]`: Interval ("10s", "1m", "5m", "15m", "30m", "1h", "4h", "8h", "1d")
- `[1]`: Symbol (e.g., "BTC_USDT")

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.candlesticks",
  "event": "update",
  "result": {
    "t": "1729100640",
    "v": "8533.02",
    "c": "8553.74",
    "h": "8550.24",
    "l": "8527.17",
    "o": "8553.74",
    "n": "BTC_USDT",
    "a": "123.456"
  }
}
```

**Fields** (note: different from REST API):
- `t`: Candle start time (seconds, string)
- `v`: Volume (base currency, string)
- `c`: Close price (string)
- `h`: High price (string)
- `l`: Low price (string)
- `o`: Open price (string)
- `n`: Symbol name (string)
- `a`: Quote volume (string)

**Note**: Updates sent every time candle changes. Last candle is not closed until interval ends.

---

### Futures Tickers (futures.tickers)

**Channel**: `futures.tickers`

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "futures.tickers",
  "event": "subscribe",
  "payload": ["BTC_USDT"]
}
```

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "futures.tickers",
  "event": "update",
  "result": {
    "contract": "BTC_USDT",
    "last": "48600.5",
    "mark_price": "48601.2",
    "index_price": "48599.8",
    "funding_rate": "0.0001",
    "funding_rate_indicative": "0.00012",
    ...
  }
}
```

---

### Futures Orderbook (futures.order_book_update)

Same structure as spot, but for futures contracts.

**Channel**: `futures.order_book_update`

---

### Futures Trades (futures.trades)

**Channel**: `futures.trades`

Similar to spot trades, but for futures contracts.

---

## Private Channels

### Spot Orders (spot.orders)

**Description**: Real-time updates for user's spot orders

**Authentication**: Required

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.orders",
  "event": "subscribe",
  "payload": ["BTC_USDT"],
  "auth": {
    "method": "api_key",
    "KEY": "your_api_key",
    "SIGN": "generated_signature"
  }
}
```

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.orders",
  "event": "update",
  "result": [
    {
      "id": "123456789",
      "text": "my-order",
      "create_time": "1729100692",
      "update_time": "1729100692",
      "currency_pair": "BTC_USDT",
      "status": "closed",
      "type": "limit",
      "side": "buy",
      "amount": "0.01",
      "price": "48000",
      "left": "0",
      "filled_total": "480.0",
      "fee": "0.096",
      "fee_currency": "USDT",
      "event": "finish"
    }
  ]
}
```

**Event Types**:
- `put`: New order placed
- `update`: Order partially filled
- `finish`: Order fully filled or cancelled

---

### Spot Balances (spot.balances)

**Description**: Real-time balance updates

**Authentication**: Required

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "spot.balances",
  "event": "subscribe",
  "auth": {
    "method": "api_key",
    "KEY": "your_api_key",
    "SIGN": "generated_signature"
  }
}
```

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "spot.balances",
  "event": "update",
  "result": [
    {
      "timestamp": "1729100692",
      "timestamp_ms": "1729100692123",
      "user": "123456",
      "currency": "USDT",
      "change": "-480.0",
      "total": "9520.0",
      "available": "9520.0"
    }
  ]
}
```

**Fields**:
- `change`: Balance change (negative for decrease)
- `total`: Total balance after change
- `available`: Available balance after change

---

### Futures Orders (futures.orders)

**Channel**: `futures.orders`

**Authentication**: Required

Similar to spot orders, but for futures contracts.

---

### Futures Positions (futures.positions)

**Description**: Real-time position updates

**Authentication**: Required

**Channel**: `futures.positions`

**Subscribe**:
```json
{
  "time": 1729100692,
  "channel": "futures.positions",
  "event": "subscribe",
  "payload": ["BTC_USDT"],
  "auth": {
    "method": "api_key",
    "KEY": "your_api_key",
    "SIGN": "generated_signature"
  }
}
```

**Update Message**:
```json
{
  "time": 1729100692,
  "channel": "futures.positions",
  "event": "update",
  "result": [
    {
      "user": 123456,
      "contract": "BTC_USDT",
      "size": 10,
      "leverage": "10",
      "entry_price": "48600.5",
      "liq_price": "43740.45",
      "mark_price": "48605.2",
      "unrealised_pnl": "0.47",
      "realised_pnl": "-1.20",
      "update_time": 1729100692
    }
  ]
}
```

---

### Futures Balances (futures.balances)

**Channel**: `futures.balances`

**Authentication**: Required

Similar to spot balances, but for futures account.

---

## Message Formats

### Request Message

```json
{
  "time": 1729100692,
  "channel": "channel_name",
  "event": "subscribe" | "unsubscribe",
  "payload": ["param1", "param2"],
  "auth": {
    "method": "api_key",
    "KEY": "api_key",
    "SIGN": "signature"
  }
}
```

### Response Message

**Success**:
```json
{
  "time": 1729100692,
  "channel": "channel_name",
  "event": "subscribe" | "update",
  "result": { ... }
}
```

**Error**:
```json
{
  "time": 1729100692,
  "channel": "channel_name",
  "event": "subscribe",
  "error": {
    "code": 1,
    "message": "error description"
  }
}
```

---

## Implementation Example (Rust)

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

async fn connect_spot_websocket() -> Result<()> {
    let url = "wss://api.gateio.ws/ws/v4/";
    let (ws_stream, _) = connect_async(url).await?;
    let (mut write, mut read) = ws_stream.split();

    // Subscribe to tickers
    let subscribe_msg = json!({
        "time": std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs(),
        "channel": "spot.tickers",
        "event": "subscribe",
        "payload": ["BTC_USDT", "ETH_USDT"]
    });

    write.send(Message::Text(subscribe_msg.to_string())).await?;

    // Start ping task
    let mut write_clone = write.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(20));
        loop {
            interval.tick().await;
            let ping = json!({
                "time": std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                "channel": "spot.ping"
            });
            if write_clone.send(Message::Text(ping.to_string())).await.is_err() {
                break;
            }
        }
    });

    // Handle messages
    while let Some(msg) = read.next().await {
        match msg? {
            Message::Text(text) => {
                let data: serde_json::Value = serde_json::from_str(&text)?;
                if data["channel"] == "spot.tickers" {
                    println!("Ticker update: {}", data["result"]);
                }
            }
            Message::Close(_) => break,
            _ => {}
        }
    }

    Ok(())
}
```

---

## Summary

### Key Points

1. **URLs**:
   - Spot: `wss://api.gateio.ws/ws/v4/`
   - Futures USDT: `wss://fx-ws.gateio.ws/v4/ws/usdt`
   - Futures BTC: `wss://fx-ws.gateio.ws/v4/ws/btc`

2. **Authentication**:
   - Not required for public channels
   - Required for private channels (orders, balances, positions)
   - Signature format: `"channel={channel}&event={event}&time={timestamp}"`

3. **Ping/Pong**:
   - Send ping every 10-30 seconds
   - Channel: `spot.ping` or `futures.ping`
   - Response: `spot.pong` or `futures.pong`

4. **Timestamps**:
   - All timestamps in **seconds** (not milliseconds) in requests
   - Responses include both `time` (seconds) and `time_ms` (milliseconds)

5. **Channel Names**:
   - Format: `{market}.{channel_name}`
   - Spot: `spot.tickers`, `spot.trades`, `spot.order_book`, etc.
   - Futures: `futures.tickers`, `futures.trades`, etc.

6. **Payload Format**:
   - Array of parameters: `["BTC_USDT", "ETH_USDT"]`
   - Order matters for some channels (e.g., candlesticks: `["1m", "BTC_USDT"]`)

---

## Sources

- [Gate.io Spot WebSocket v4](https://www.gate.com/docs/developers/apiv4/ws/en/)
- [Gate.io Futures WebSocket v4](https://www.gate.com/docs/developers/futures/ws/en/)
- [Gate.io WebSocket API Reference](https://www.gate.com/docs/developers/websocket/)

---

**Research completed**: 2026-01-21
**Implementation note**: WebSocket is essential for real-time data. Implement before production use to avoid REST API rate limits.
