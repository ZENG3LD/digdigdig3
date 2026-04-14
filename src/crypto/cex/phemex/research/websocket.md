# Phemex WebSocket API

Complete WebSocket specification for V5 connector implementation.

## WebSocket Endpoints

| Environment | URL | Rate Limit |
|-------------|-----|------------|
| Production | `wss://ws.phemex.com/ws` | Standard |
| High Rate Limit (VIP) | `wss://vapi.phemex.com/ws` | Enhanced |
| Testnet | `wss://testnet.phemex.com/ws` | Limited |

## Connection Limits

| Limit Type | Capacity | Scope |
|------------|----------|-------|
| Concurrent Connections | 5 | Per user |
| Subscriptions per Connection | 20 | Per WebSocket connection |
| Request Throttle | 20 requests/second | Per connection |
| IP Rate Limit | 200 requests / 5 minutes | Per IP (ws.phemex.com) |

## Message Format

All WebSocket messages use JSON format:

### Request Message Structure

```json
{
  "id": 1234,          // Integer request ID (client-generated)
  "method": "method.name",
  "params": [...]      // Array of parameters
}
```

### Response Message Structure

```json
{
  "id": 1234,          // Matches request ID
  "error": null,       // null on success, object on error
  "result": {...}      // Response data
}
```

### Server Push Message Structure

```json
{
  "type": "snapshot" | "incremental",
  "symbol": "BTCUSD",
  "data": {...}
}
```

## Heartbeat (Ping/Pong)

### Requirements

| Parameter | Value | Description |
|-----------|-------|-------------|
| Maximum Interval | 30 seconds | Server drops connection if exceeded |
| Recommended Interval | 5 seconds | Best practice for stability |
| Timeout Action | Disconnect | No warning, immediate drop |

### Heartbeat Message

**Client sends:**
```json
{
  "id": 1234,
  "method": "server.ping",
  "params": []
}
```

**Server responds:**
```json
{
  "id": 1234,
  "error": null,
  "result": "pong"
}
```

**Implementation:**
```rust
pub async fn heartbeat_loop(ws: &mut WebSocket) {
    let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        let ping = serde_json::json!({
            "id": generate_id(),
            "method": "server.ping",
            "params": []
        });

        if let Err(e) = ws.send(ping.to_string()).await {
            log::error!("Heartbeat failed: {}", e);
            break;
        }
    }
}
```

## Authentication

Required for private channels (AOP - Account Order Position).

### Authentication Message

```json
{
  "method": "user.auth",
  "params": [
    "API",
    "<api_key>",
    "<signature>",
    <expiry>
  ],
  "id": 1234
}
```

### Parameter Details

| Parameter | Type | Description |
|-----------|------|-------------|
| `params[0]` | String | Always `"API"` |
| `params[1]` | String | Your API Key ID |
| `params[2]` | String | HMAC SHA256 signature |
| `params[3]` | Integer | Unix timestamp (seconds) |

### Signature Generation

**Message to sign:**
```
<api_key><expiry>
```

**Example:**
```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

fn generate_ws_signature(api_key: &str, secret: &str, expiry: u64) -> String {
    let message = format!("{}{}", api_key, expiry);

    let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
    mac.update(message.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}
```

**Complete example:**
```json
{
  "method": "user.auth",
  "params": [
    "API",
    "806066b0-f02b-4d3e-b444-76ec718e1023",
    "8c939f7a6e6716ab7c4240384e07c81840dacd371cdcf5051bb6b7084897470e",
    1570091232
  ],
  "id": 1234
}
```

### Authentication Response

**Success:**
```json
{
  "error": null,
  "id": 1234,
  "result": {
    "status": "success"
  }
}
```

**Failure:**
```json
{
  "error": {
    "code": 401,
    "message": "Invalid signature"
  },
  "id": 1234,
  "result": null
}
```

## Public Channels

Public channels don't require authentication.

### Order Book

**Subscribe:**
```json
{
  "id": 1234,
  "method": "orderbook.subscribe",
  "params": ["sBTCUSDT"]
}
```

**Unsubscribe:**
```json
{
  "id": 1234,
  "method": "orderbook.unsubscribe",
  "params": []
}
```

**Snapshot Message:**
```json
{
  "book": {
    "asks": [
      [priceEp, size],
      [87705000, 1000000]
    ],
    "bids": [
      [priceEp, size],
      [87700000, 2000000]
    ]
  },
  "depth": 30,
  "sequence": 123456789,
  "timestamp": 1234567890000000000,
  "symbol": "BTCUSD",
  "type": "snapshot"
}
```

**Incremental Update:**
```json
{
  "book": {
    "asks": [
      [87710000, 500000]  // New or updated level
    ],
    "bids": [
      [87695000, 0]       // Size 0 = remove level
    ]
  },
  "sequence": 123456790,
  "timestamp": 1234567890100000000,
  "symbol": "BTCUSD",
  "type": "incremental"
}
```

**Update Frequency:** ~20ms for incremental updates

### Full Order Book

**Subscribe:**
```json
{
  "id": 1234,
  "method": "orderbook_p.subscribe",
  "params": ["BTCUSD"]
}
```

**Note:** Full order book includes ALL levels (not just top 30).

### Trades

**Subscribe:**
```json
{
  "id": 1234,
  "method": "trade.subscribe",
  "params": ["BTCUSD"]
}
```

**Unsubscribe:**
```json
{
  "id": 1234,
  "method": "trade.unsubscribe",
  "params": []
}
```

**Snapshot Message:**
```json
{
  "trades": [
    [timestamp, side, priceEp, size],
    [1234567890000000000, "Buy", 87705000, 1000],
    [1234567891000000000, "Sell", 87700000, 500]
  ],
  "sequence": 123456789,
  "symbol": "BTCUSD",
  "type": "snapshot"
}
```

**Incremental Update:**
```json
{
  "trades": [
    [1234567892000000000, "Buy", 87710000, 2000]
  ],
  "sequence": 123456790,
  "symbol": "BTCUSD",
  "type": "incremental"
}
```

**Trade Fields:**
- `[0]`: Timestamp (nanoseconds)
- `[1]`: Side (`"Buy"` or `"Sell"`)
- `[2]`: Price (scaled, Ep)
- `[3]`: Size

### Klines (Candlesticks)

**Subscribe:**
```json
{
  "id": 1234,
  "method": "kline.subscribe",
  "params": ["BTCUSD", 60]
}
```

**Parameters:**
- `params[0]`: Symbol
- `params[1]`: Interval in seconds (60, 300, 900, 1800, 3600, 14400, 86400, etc.)

**Unsubscribe:**
```json
{
  "id": 1234,
  "method": "kline.unsubscribe",
  "params": []
}
```

**Snapshot Message:**
```json
{
  "kline": [
    [timestamp, interval, lastEp, highEp, lowEp, openEp, volume, turnoverEv],
    [1590019200, 60, 87700000, 87750000, 87650000, 87680000, 123456, 1234567890]
  ],
  "sequence": 123456789,
  "symbol": "BTCUSD",
  "type": "snapshot"
}
```

**Incremental Update:**
```json
{
  "kline": [
    [1590019260, 60, 87720000, 87750000, 87700000, 87700000, 234567, 2345678901]
  ],
  "sequence": 123456790,
  "symbol": "BTCUSD",
  "type": "incremental"
}
```

**Kline Fields (Array):**
- `[0]`: Timestamp (seconds)
- `[1]`: Interval (seconds)
- `[2]`: Close price (Ep)
- `[3]`: High price (Ep)
- `[4]`: Low price (Ep)
- `[5]`: Open price (Ep)
- `[6]`: Volume
- `[7]`: Turnover (Ev)

**Update Frequency:** Every interval period (e.g., every 60 seconds for 1-minute klines)

### 24-Hour Ticker

**Subscribe:**
```json
{
  "id": 1234,
  "method": "market24h.subscribe",
  "params": []
}
```

**Snapshot Message:**
```json
{
  "market24h": {
    "openEp": 87000000,
    "highEp": 88000000,
    "lowEp": 86500000,
    "lastEp": 87700000,
    "bidEp": 87695000,
    "askEp": 87705000,
    "indexEp": 87702000,
    "markEp": 87700000,
    "openInterest": 123456789,
    "fundingRateEr": 10000,
    "predFundingRateEr": 10000,
    "timestamp": 1234567890000000000,
    "turnoverEv": 12345678900000,
    "volume": 12345678
  },
  "symbol": "BTCUSD",
  "type": "snapshot"
}
```

**Update Frequency:** Real-time on changes

### Symbol Price

**Subscribe:**
```json
{
  "id": 1234,
  "method": "tick.subscribe",
  "params": ["BTCUSD"]
}
```

**Message:**
```json
{
  "tick": {
    "last": 87700000,
    "timestamp": 1234567890000000000
  },
  "symbol": "BTCUSD",
  "type": "incremental"
}
```

## Private Channels

Requires authentication before subscribing.

### AOP (Account-Order-Position)

**Subscribe:**
```json
{
  "id": 1234,
  "method": "aop.subscribe",
  "params": []
}
```

**Unsubscribe:**
```json
{
  "id": 1234,
  "method": "aop.unsubscribe",
  "params": []
}
```

**Initial Snapshot:**

Upon subscription, server sends 0 or more account snapshot messages containing:
- Trading account information
- Holding positions
- Open orders (all)
- Closed orders (max 100 most recent)
- Filled orders (max 100 most recent)

**Snapshot Message Structure:**
```json
{
  "accounts": [
    {
      "accountId": 123456,
      "currency": "BTC",
      "accountBalanceEv": 100000000,
      "totalUsedBalanceEv": 20000000
    }
  ],
  "orders": [
    {
      "orderID": "uuid",
      "clOrdID": "client-id",
      "symbol": "BTCUSD",
      "side": "Buy",
      "ordType": "Limit",
      "priceEp": 87700000,
      "orderQty": 1000,
      "ordStatus": "New",
      "createTimeNs": 1234567890000000000
    }
  ],
  "positions": [
    {
      "accountID": 123456,
      "symbol": "BTCUSD",
      "side": "Buy",
      "size": 1000,
      "avgEntryPriceEp": 87700000,
      "unrealisedPnlEv": 0,
      "leverageEr": 1000000
    }
  ],
  "sequence": 123456789,
  "timestamp": 1234567890000000000,
  "type": "snapshot"
}
```

**Incremental Updates:**

Real-time updates for:
- Account balance changes
- Order status changes (new, filled, canceled)
- Position updates

```json
{
  "orders": [
    {
      "orderID": "uuid",
      "symbol": "BTCUSD",
      "ordStatus": "Filled",
      "cumQty": 1000,
      "avgPriceEp": 87700000
    }
  ],
  "sequence": 123456790,
  "timestamp": 1234567890100000000,
  "type": "incremental"
}
```

**Update Frequency:** Real-time on any change

## Subscription Management

### Subscribe Response

**Success:**
```json
{
  "error": null,
  "id": 1234,
  "result": {
    "status": "success"
  }
}
```

**Failure:**
```json
{
  "error": {
    "code": 2003,
    "message": "invalid symbol"
  },
  "id": 1234,
  "result": null
}
```

### Multiple Subscriptions

**Single connection can handle up to 20 subscriptions:**

```rust
// Good: Stay within limit
ws.subscribe("orderbook", "BTCUSD").await?;
ws.subscribe("trade", "BTCUSD").await?;
ws.subscribe("kline", "BTCUSD", 60).await?;
// ... up to 17 more

// Bad: Exceeds 20 subscription limit
for symbol in symbols {  // 50 symbols
    ws.subscribe("orderbook", symbol).await?;  // Will fail after 20
}
```

**Solution for many symbols:**
```rust
// Use multiple connections (max 5)
let mut connections = Vec::new();
for i in 0..5 {
    let ws = connect_websocket().await?;
    connections.push(ws);
}

// Distribute subscriptions across connections
for (idx, symbol) in symbols.iter().enumerate() {
    let conn_idx = idx % 5;
    connections[conn_idx].subscribe("orderbook", symbol).await?;
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 2001 | invalid request | Malformed JSON or missing fields |
| 2002 | invalid argument | Invalid parameter value |
| 2003 | invalid symbol | Symbol doesn't exist |
| 2004 | not subscribed | Attempting to unsubscribe without subscription |
| 2005 | subscription limit reached | Exceeded 20 subscriptions per connection |

## Reconnection Handling

### Exponential Backoff

```rust
pub async fn connect_with_retry(url: &str, max_retries: u32) -> Result<WebSocket, Error> {
    let mut retries = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match tokio_tungstenite::connect_async(url).await {
            Ok((ws, _)) => return Ok(ws),
            Err(e) => {
                if retries >= max_retries {
                    return Err(Error::MaxRetriesExceeded);
                }

                log::warn!("Connection failed: {}, retrying in {:?}", e, delay);
                tokio::time::sleep(delay).await;

                retries += 1;
                delay = std::cmp::min(delay * 2, Duration::from_secs(60));
            }
        }
    }
}
```

### State Recovery

After reconnection:

1. **Re-authenticate** (for private channels)
2. **Re-subscribe** to all previous channels
3. **Fetch snapshots** to sync state
4. **Resume processing** incremental updates

```rust
pub async fn recover_subscriptions(
    ws: &mut WebSocket,
    subscriptions: &[Subscription],
) -> Result<(), Error> {
    // Re-authenticate if needed
    if subscriptions.iter().any(|s| s.requires_auth) {
        authenticate(ws).await?;
    }

    // Re-subscribe to all channels
    for sub in subscriptions {
        ws.subscribe(&sub.channel, &sub.symbol).await?;
    }

    Ok(())
}
```

## Sequence Number Handling

All messages include a `sequence` number for ordering:

```rust
pub struct SequenceTracker {
    last_sequence: HashMap<String, u64>,
}

impl SequenceTracker {
    pub fn check_sequence(&mut self, symbol: &str, sequence: u64) -> bool {
        if let Some(&last) = self.last_sequence.get(symbol) {
            if sequence <= last {
                log::warn!("Out-of-order message: {} (expected > {})", sequence, last);
                return false;
            }
            if sequence != last + 1 {
                log::warn!("Sequence gap: {} (expected {})", sequence, last + 1);
                // May need to re-subscribe to get snapshot
            }
        }

        self.last_sequence.insert(symbol.to_string(), sequence);
        true
    }
}
```

## Complete Implementation Example

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

pub struct PhemexWebSocket {
    ws: WebSocket,
    subscriptions: Vec<Subscription>,
    sequence_tracker: SequenceTracker,
}

impl PhemexWebSocket {
    pub async fn connect(url: &str) -> Result<Self, Error> {
        let (ws, _) = connect_async(url).await?;

        Ok(Self {
            ws,
            subscriptions: Vec::new(),
            sequence_tracker: SequenceTracker::new(),
        })
    }

    pub async fn authenticate(&mut self, api_key: &str, secret: &str) -> Result<(), Error> {
        let expiry = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs() + 60;

        let signature = generate_ws_signature(api_key, secret, expiry);

        let auth_msg = json!({
            "method": "user.auth",
            "params": ["API", api_key, signature, expiry],
            "id": generate_id()
        });

        self.ws.send(Message::Text(auth_msg.to_string())).await?;

        // Wait for auth response
        if let Some(msg) = self.ws.next().await {
            let response: AuthResponse = serde_json::from_str(&msg?.to_string())?;
            if response.error.is_some() {
                return Err(Error::AuthenticationFailed);
            }
        }

        Ok(())
    }

    pub async fn subscribe_orderbook(&mut self, symbol: &str) -> Result<(), Error> {
        let sub_msg = json!({
            "id": generate_id(),
            "method": "orderbook.subscribe",
            "params": [symbol]
        });

        self.ws.send(Message::Text(sub_msg.to_string())).await?;
        self.subscriptions.push(Subscription {
            channel: "orderbook".to_string(),
            symbol: symbol.to_string(),
            requires_auth: false,
        });

        Ok(())
    }

    pub async fn subscribe_aop(&mut self) -> Result<(), Error> {
        let sub_msg = json!({
            "id": generate_id(),
            "method": "aop.subscribe",
            "params": []
        });

        self.ws.send(Message::Text(sub_msg.to_string())).await?;
        self.subscriptions.push(Subscription {
            channel: "aop".to_string(),
            symbol: String::new(),
            requires_auth: true,
        });

        Ok(())
    }

    pub async fn run(mut self, handler: impl Fn(PhemexMessage)) {
        let mut heartbeat_interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                Some(msg) = self.ws.next() => {
                    match msg {
                        Ok(Message::Text(text)) => {
                            if let Ok(parsed) = serde_json::from_str::<PhemexMessage>(&text) {
                                handler(parsed);
                            }
                        }
                        Err(e) => {
                            log::error!("WebSocket error: {}", e);
                            break;
                        }
                        _ => {}
                    }
                }
                _ = heartbeat_interval.tick() => {
                    let ping = json!({
                        "id": generate_id(),
                        "method": "server.ping",
                        "params": []
                    });

                    if let Err(e) = self.ws.send(Message::Text(ping.to_string())).await {
                        log::error!("Heartbeat failed: {}", e);
                        break;
                    }
                }
            }
        }
    }
}
```

## Best Practices

1. **Always send heartbeats** every 5 seconds
2. **Track sequence numbers** to detect gaps
3. **Handle reconnections** with exponential backoff
4. **Re-subscribe after reconnection**
5. **Use multiple connections** for >20 subscriptions
6. **Authenticate once per connection** (before private subscriptions)
7. **Process incremental updates** against local state
8. **Validate message structure** before parsing
9. **Log all errors** for debugging
10. **Monitor connection health** and reconnect proactively

## Summary Table

| Feature | Public | Private | Limit |
|---------|--------|---------|-------|
| Order Book | ✓ | | Top 30 levels |
| Full Order Book | ✓ | | All levels |
| Trades | ✓ | | Real-time |
| Klines | ✓ | | By interval |
| 24h Ticker | ✓ | | Real-time |
| Symbol Price | ✓ | | Real-time |
| AOP (Account/Orders/Positions) | | ✓ | Real-time |
| Connections per User | | | 5 max |
| Subscriptions per Connection | | | 20 max |
| Request Rate | | | 20/second |
| Heartbeat Interval | | | 5 seconds recommended |
