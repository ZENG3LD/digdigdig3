# Crypto.com Exchange API v1 - WebSocket Documentation

## Overview

Crypto.com Exchange provides two separate WebSocket servers:
- **User API WebSocket:** Authenticated user-specific data
- **Market Data WebSocket:** Public market information

Both use JSON message format and require proper connection management.

---

## WebSocket URLs

### Production

```
User API:        wss://stream.crypto.com/exchange/v1/user
Market Data:     wss://stream.crypto.com/exchange/v1/market
```

### UAT Sandbox

```
User API:        wss://uat-stream.3ona.co/exchange/v1/user
Market Data:     wss://uat-stream.3ona.co/exchange/v1/market
```

---

## Connection Setup

### Critical Connection Rule

**ALWAYS add a 1-second sleep after establishing the WebSocket connection before sending any requests.**

This is required because rate limits are pro-rated based on the calendar-second that the WebSocket connection was opened.

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

async fn connect_to_websocket(url: &str) -> Result<WebSocket, Error> {
    let (ws_stream, _response) = connect_async(url).await?;

    // CRITICAL: Wait 1 second before sending requests
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(ws_stream)
}
```

---

## Authentication (User API Only)

### Authentication Flow

1. Connect to User API WebSocket
2. Wait 1 second
3. Send `public/auth` request with signature
4. Receive authentication confirmation
5. Subscribe to user-specific channels

### Authentication Message

```json
{
  "id": 1,
  "method": "public/auth",
  "api_key": "your_api_key_here",
  "sig": "generated_signature",
  "nonce": 1587523073344
}
```

### Signature Generation

**Payload:** `method + id + api_key + nonce`

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn generate_ws_auth_signature(
    id: i64,
    api_key: &str,
    nonce: i64,
    api_secret: &str,
) -> String {
    let payload = format!("public/auth{}{}{}", id, api_key, nonce);

    let mut mac = HmacSha256::new_from_slice(api_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());

    hex::encode(mac.finalize().into_bytes())
}
```

### Authentication Response

**Success:**
```json
{
  "id": 1,
  "method": "public/auth",
  "code": 0
}
```

**Failure:**
```json
{
  "id": 1,
  "method": "public/auth",
  "code": 10003,
  "message": "INVALID_SIGNATURE"
}
```

### Authentication Example

```rust
pub async fn authenticate(&mut self) -> Result<(), Error> {
    let nonce = generate_nonce();
    let signature = generate_ws_auth_signature(
        1,
        &self.api_key,
        nonce,
        &self.api_secret,
    );

    let auth_msg = json!({
        "id": 1,
        "method": "public/auth",
        "api_key": self.api_key,
        "sig": signature,
        "nonce": nonce
    });

    self.ws.send(Message::Text(auth_msg.to_string())).await?;

    // Wait for auth response
    let response = self.receive_message().await?;

    if response["code"] == 0 {
        Ok(())
    } else {
        Err(Error::AuthenticationFailed(response["message"].to_string()))
    }
}
```

---

## Subscription Management

### Subscribe Message Format

```json
{
  "id": 1,
  "method": "subscribe",
  "params": {
    "channels": ["ticker.BTCUSD-PERP", "book.BTCUSD-PERP.10"]
  },
  "nonce": 1587523073344
}
```

### Unsubscribe Message Format

```json
{
  "id": 2,
  "method": "unsubscribe",
  "params": {
    "channels": ["ticker.BTCUSD-PERP"]
  },
  "nonce": 1587523073345
}
```

### Subscription Response

**Success:**
```json
{
  "id": 1,
  "method": "subscribe",
  "code": 0,
  "result": {
    "subscription": "ticker.BTCUSD-PERP",
    "channel": "ticker"
  }
}
```

**Failure:**
```json
{
  "id": 1,
  "method": "subscribe",
  "code": 30002,
  "message": "INSTRUMENT_NOT_FOUND"
}
```

---

## Public Market Data Channels

### Ticker Channel

**Subscription:** `ticker.{instrument_name}`

**Example:** `ticker.BTCUSD-PERP`

**Update Message:**
```json
{
  "method": "subscribe",
  "result": {
    "channel": "ticker",
    "subscription": "ticker.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "data": [
      {
        "i": "BTCUSD-PERP",
        "b": "51170.000000",
        "k": "51180.000000",
        "a": "51174.500000",
        "c": "0.03955106",
        "h": "51790.00",
        "l": "47895.50",
        "v": "879.5024",
        "vv": "26370000.12",
        "oi": "12345.12",
        "t": 1613580710768
      }
    ]
  }
}
```

**Fields:**
- `i` - Instrument name
- `b` - Best bid
- `k` - Best ask
- `a` - Last price
- `c` - 24h change
- `h` - 24h high
- `l` - 24h low
- `v` - 24h volume (base)
- `vv` - 24h volume (quote)
- `oi` - Open interest
- `t` - Timestamp (ms)

---

### Order Book Channel

**Subscription:** `book.{instrument_name}.{depth}`

**Examples:**
- `book.BTCUSD-PERP.10`
- `book.BTC_USDT.50`

**Depth Options:** 10, 50 (default: 50)

**Update Types:**
- **Delta Updates:** Every 100ms
- **Snapshot:** Every 500ms

**Snapshot Message:**
```json
{
  "method": "subscribe",
  "result": {
    "channel": "book",
    "subscription": "book.BTCUSD-PERP.10",
    "instrument_name": "BTCUSD-PERP",
    "depth": 10,
    "data": [
      {
        "bids": [
          ["50113.500000", "0.400000", "0"],
          ["50113.000000", "0.051800", "0"]
        ],
        "asks": [
          ["50126.000000", "0.400000", "0"],
          ["50130.000000", "1.279000", "0"]
        ],
        "t": 1613580710768,
        "tt": "SNAPSHOT"
      }
    ]
  }
}
```

**Delta Message:**
```json
{
  "result": {
    "channel": "book",
    "subscription": "book.BTCUSD-PERP.10",
    "instrument_name": "BTCUSD-PERP",
    "depth": 10,
    "data": [
      {
        "bids": [
          ["50115.000000", "0.500000", "0"]
        ],
        "asks": [],
        "t": 1613580710868,
        "tt": "DELTA"
      }
    ]
  }
}
```

**Update Type Field:**
- `tt: "SNAPSHOT"` - Full order book
- `tt: "DELTA"` - Incremental update

**Entry Format:** `[price, quantity, order_count]`
- Quantity `0.000000` means remove price level

---

### Trade Channel

**Subscription:** `trade.{instrument_name}`

**Example:** `trade.BTCUSD-PERP`

**Update Message:**
```json
{
  "result": {
    "channel": "trade",
    "subscription": "trade.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "data": [
      {
        "dataTime": 1613580710768,
        "d": "18342311001",
        "s": "BUY",
        "p": "51100.00",
        "q": "0.5000",
        "t": 1613580710768,
        "i": "BTCUSD-PERP"
      }
    ]
  }
}
```

**Fields:**
- `dataTime` - Trade timestamp
- `d` - Trade ID
- `s` - Side (BUY/SELL)
- `p` - Price
- `q` - Quantity
- `t` - Trade time
- `i` - Instrument name

---

### Candlestick Channel

**Subscription:** `candlestick.{timeframe}.{instrument_name}`

**Examples:**
- `candlestick.1h.BTCUSD-PERP`
- `candlestick.5m.BTC_USDT`

**Timeframes:** `1m`, `5m`, `15m`, `30m`, `1h`, `2h`, `4h`, `12h`, `1D`, `7D`, `14D`, `1M`

**Update Message:**
```json
{
  "result": {
    "channel": "candlestick",
    "subscription": "candlestick.1h.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "interval": "1h",
    "data": [
      {
        "t": 1613577600000,
        "o": "50100.00",
        "h": "51500.00",
        "l": "49800.00",
        "c": "51100.00",
        "v": "123.4567"
      }
    ]
  }
}
```

**Fields:**
- `t` - Candle open time (ms)
- `o` - Open price
- `h` - High price
- `l` - Low price
- `c` - Close price
- `v` - Volume

---

### Additional Public Channels

**Index Price:**
- `index.{instrument_name}`
- Example: `index.BTCUSD-PERP`

**Mark Price:**
- `mark.{instrument_name}`
- Example: `mark.BTCUSD-PERP`

**Settlement Price:**
- `settlement.{instrument_name}`
- Example: `settlement.BTCUSD-PERP`

**Funding Rate:**
- `funding.{instrument_name}`
- Example: `funding.BTCUSD-PERP`

**Estimated Funding:**
- `estimatedfunding.{instrument_name}`
- Example: `estimatedfunding.BTCUSD-PERP`

---

## Private User Channels

### User Order Channel

**Subscription:** `user.order.{instrument_name}`

**Examples:**
- `user.order.BTCUSD-PERP`
- `user.order.BTC_USDT`

**Update Message:**
```json
{
  "result": {
    "channel": "user.order",
    "subscription": "user.order.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "data": [
      {
        "order_id": "18342311",
        "client_oid": "client_order_123",
        "instrument_name": "BTCUSD-PERP",
        "side": "BUY",
        "type": "LIMIT",
        "price": "50000.00",
        "quantity": "0.5000",
        "cumulative_quantity": "0.2500",
        "avg_price": "49950.00",
        "status": "ACTIVE",
        "time_in_force": "GOOD_TILL_CANCEL",
        "create_time": 1587523073344,
        "update_time": 1613580710768
      }
    ]
  }
}
```

**Status Values:**
- `ACTIVE` - Order is open
- `FILLED` - Fully filled
- `CANCELED` - Canceled
- `REJECTED` - Rejected
- `EXPIRED` - Expired

---

### User Advanced Order Channel

**Subscription:** `user.advanced.order.{instrument_name}`

**Purpose:** Updates for advanced order types (OTO, OTOCO)

---

### User Trade Channel

**Subscription:** `user.trade.{instrument_name}`

**Examples:**
- `user.trade.BTCUSD-PERP`
- `user.trade.BTC_USDT`

**Update Message:**
```json
{
  "result": {
    "channel": "user.trade",
    "subscription": "user.trade.BTCUSD-PERP",
    "instrument_name": "BTCUSD-PERP",
    "data": [
      {
        "trade_id": "183423110001",
        "order_id": "18342311",
        "client_oid": "client_order_123",
        "instrument_name": "BTCUSD-PERP",
        "side": "BUY",
        "fee": "0.25",
        "fee_currency": "USD",
        "create_time": 1613580710768,
        "traded_price": "49950.00",
        "traded_quantity": "0.2500",
        "liquidity_indicator": "MAKER"
      }
    ]
  }
}
```

**Fields:**
- `liquidity_indicator` - MAKER or TAKER
- `fee` - Trading fee
- `fee_currency` - Fee denomination

---

### User Balance Channel

**Subscription:** `user.balance`

**Update Message:**
```json
{
  "result": {
    "channel": "user.balance",
    "subscription": "user.balance",
    "data": [
      {
        "currency": "USDT",
        "balance": "10000.00",
        "available": "9500.00",
        "order": "500.00",
        "stake": "0.00"
      }
    ]
  }
}
```

**Trigger:** Updates on any balance change (trade, deposit, withdrawal)

---

### User Positions Channel

**Subscription:** `user.positions`

**Update Message:**
```json
{
  "result": {
    "channel": "user.positions",
    "subscription": "user.positions",
    "data": [
      {
        "account_id": "account_123",
        "instrument_name": "BTCUSD-PERP",
        "quantity": "1.5000",
        "cost": "75000.00",
        "open_position_pnl": "1500.00",
        "entry_price": "50000.00",
        "mark_price": "51000.00",
        "leverage": "10",
        "type": "CROSS"
      }
    ]
  }
}
```

**Trigger:** Updates on position changes (fills, liquidations)

---

### User Account Risk Channel

**Subscription:** `user.account_risk`

**Purpose:** Real-time margin and risk metrics

---

### User Position Balance Channel

**Subscription:** `user.position_balance`

**Purpose:** Position-specific balance updates for isolated margin

---

## Heartbeat Mechanism

### Heartbeat Flow

The WebSocket connection requires periodic heartbeats to maintain the connection.

**Server Ping:**
```json
{
  "method": "public/heartbeat"
}
```

**Client Pong:**
```json
{
  "id": 123,
  "method": "public/respond-heartbeat"
}
```

### Implementation

```rust
async fn handle_heartbeat(&mut self, msg: Value) -> Result<(), Error> {
    if msg["method"] == "public/heartbeat" {
        let pong = json!({
            "id": self.next_id(),
            "method": "public/respond-heartbeat"
        });

        self.ws.send(Message::Text(pong.to_string())).await?;
    }

    Ok(())
}
```

---

## Connection Termination

### Termination Codes

- **1000:** Normal closure
- **1006:** Abnormal closure (connection lost)
- **1013:** Server restart (reconnect required)

### Reconnection Strategy

```rust
pub async fn maintain_connection(&mut self) {
    loop {
        match self.ws.next().await {
            Some(Ok(msg)) => {
                self.handle_message(msg).await;
            }
            Some(Err(e)) => {
                log::error!("WebSocket error: {:?}", e);
                self.reconnect().await;
            }
            None => {
                log::warn!("WebSocket connection closed");
                self.reconnect().await;
            }
        }
    }
}

async fn reconnect(&mut self) {
    let mut retries = 0;
    let max_retries = 10;

    while retries < max_retries {
        match self.connect().await {
            Ok(_) => {
                log::info!("Reconnected successfully");
                self.resubscribe().await;
                return;
            }
            Err(e) => {
                retries += 1;
                let backoff = Duration::from_secs(2u64.pow(retries.min(6)));
                log::warn!("Reconnection attempt {} failed: {:?}", retries, e);
                tokio::time::sleep(backoff).await;
            }
        }
    }

    log::error!("Max reconnection attempts reached");
}
```

---

## Rate Limits

### WebSocket Rate Limits

| WebSocket Type | Rate Limit |
|----------------|------------|
| User API | 150 requests/second |
| Market Data | 100 requests/second |

**Note:** Rate limits are pro-rated based on connection timestamp.

### Rate Limiting Implementation

```rust
pub struct WebSocketClient {
    rate_limiter: RateLimiter,
}

impl WebSocketClient {
    pub async fn send_request(&mut self, request: Value) -> Result<(), Error> {
        self.rate_limiter.acquire().await;
        self.ws.send(Message::Text(request.to_string())).await?;
        Ok(())
    }
}
```

---

## Complete Example

### Market Data Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use serde_json::{json, Value};

pub struct MarketDataClient {
    ws: WebSocket,
    subscriptions: Vec<String>,
}

impl MarketDataClient {
    pub async fn new() -> Result<Self, Error> {
        let url = "wss://stream.crypto.com/exchange/v1/market";
        let (ws, _) = connect_async(url).await?;

        // CRITICAL: Wait 1 second
        tokio::time::sleep(Duration::from_secs(1)).await;

        Ok(Self {
            ws,
            subscriptions: Vec::new(),
        })
    }

    pub async fn subscribe_ticker(&mut self, symbol: &str) -> Result<(), Error> {
        let channel = format!("ticker.{}", symbol);
        self.subscribe(vec![channel]).await
    }

    pub async fn subscribe_orderbook(&mut self, symbol: &str, depth: u32) -> Result<(), Error> {
        let channel = format!("book.{}.{}", symbol, depth);
        self.subscribe(vec![channel]).await
    }

    async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Error> {
        let msg = json!({
            "id": self.next_id(),
            "method": "subscribe",
            "params": {
                "channels": channels
            },
            "nonce": generate_nonce()
        });

        self.ws.send(Message::Text(msg.to_string())).await?;
        self.subscriptions.extend(channels);

        Ok(())
    }

    pub async fn receive(&mut self) -> Result<Value, Error> {
        loop {
            match self.ws.next().await {
                Some(Ok(Message::Text(text))) => {
                    let msg: Value = serde_json::from_str(&text)?;

                    // Handle heartbeat
                    if msg["method"] == "public/heartbeat" {
                        self.handle_heartbeat().await?;
                        continue;
                    }

                    return Ok(msg);
                }
                Some(Ok(Message::Close(_))) => {
                    return Err(Error::ConnectionClosed);
                }
                Some(Err(e)) => {
                    return Err(Error::WebSocketError(e));
                }
                None => {
                    return Err(Error::ConnectionClosed);
                }
                _ => continue,
            }
        }
    }

    async fn handle_heartbeat(&mut self) -> Result<(), Error> {
        let pong = json!({
            "id": self.next_id(),
            "method": "public/respond-heartbeat"
        });

        self.ws.send(Message::Text(pong.to_string())).await?;
        Ok(())
    }

    fn next_id(&self) -> i64 {
        // Implementation for generating unique IDs
        1
    }
}
```

---

### User Data Client

```rust
pub struct UserDataClient {
    ws: WebSocket,
    api_key: String,
    api_secret: String,
    authenticated: bool,
}

impl UserDataClient {
    pub async fn new(api_key: String, api_secret: String) -> Result<Self, Error> {
        let url = "wss://stream.crypto.com/exchange/v1/user";
        let (ws, _) = connect_async(url).await?;

        // CRITICAL: Wait 1 second
        tokio::time::sleep(Duration::from_secs(1)).await;

        let mut client = Self {
            ws,
            api_key,
            api_secret,
            authenticated: false,
        };

        // Authenticate immediately
        client.authenticate().await?;

        Ok(client)
    }

    async fn authenticate(&mut self) -> Result<(), Error> {
        let nonce = generate_nonce();
        let signature = generate_ws_auth_signature(
            1,
            &self.api_key,
            nonce,
            &self.api_secret,
        );

        let auth_msg = json!({
            "id": 1,
            "method": "public/auth",
            "api_key": self.api_key,
            "sig": signature,
            "nonce": nonce
        });

        self.ws.send(Message::Text(auth_msg.to_string())).await?;

        // Wait for auth response
        let response = self.receive_raw().await?;

        if response["code"] == 0 {
            self.authenticated = true;
            Ok(())
        } else {
            Err(Error::AuthenticationFailed)
        }
    }

    pub async fn subscribe_orders(&mut self, symbol: &str) -> Result<(), Error> {
        if !self.authenticated {
            return Err(Error::NotAuthenticated);
        }

        let channel = format!("user.order.{}", symbol);
        self.subscribe(vec![channel]).await
    }

    pub async fn subscribe_trades(&mut self, symbol: &str) -> Result<(), Error> {
        if !self.authenticated {
            return Err(Error::NotAuthenticated);
        }

        let channel = format!("user.trade.{}", symbol);
        self.subscribe(vec![channel]).await
    }

    pub async fn subscribe_balance(&mut self) -> Result<(), Error> {
        if !self.authenticated {
            return Err(Error::NotAuthenticated);
        }

        self.subscribe(vec!["user.balance".to_string()]).await
    }

    async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Error> {
        let msg = json!({
            "id": self.next_id(),
            "method": "subscribe",
            "params": {
                "channels": channels
            },
            "nonce": generate_nonce()
        });

        self.ws.send(Message::Text(msg.to_string())).await?;
        Ok(())
    }

    async fn receive_raw(&mut self) -> Result<Value, Error> {
        // Similar to MarketDataClient::receive
        todo!()
    }

    fn next_id(&self) -> i64 {
        1
    }
}
```

---

## Best Practices

### 1. Always Wait After Connection
```rust
connect_async(url).await?;
tokio::time::sleep(Duration::from_secs(1)).await; // REQUIRED
```

### 2. Handle Heartbeats
```rust
if msg["method"] == "public/heartbeat" {
    respond_heartbeat().await?;
}
```

### 3. Implement Reconnection Logic
```rust
loop {
    match ws.next().await {
        None | Some(Err(_)) => reconnect().await,
        Some(Ok(msg)) => handle(msg).await,
    }
}
```

### 4. Resubscribe After Reconnection
```rust
async fn reconnect(&mut self) {
    self.connect().await;
    self.authenticate().await; // User API only
    self.resubscribe_all().await;
}
```

### 5. Use Separate Connections for Market Data and User Data
```rust
let market_ws = MarketDataClient::new().await?;
let user_ws = UserDataClient::new(api_key, api_secret).await?;
```

---

## Troubleshooting

### Connection Immediately Closes

**Cause:** Not waiting 1 second after connection

**Solution:**
```rust
tokio::time::sleep(Duration::from_secs(1)).await;
```

---

### Authentication Fails

**Cause:** Incorrect signature generation

**Solution:**
- Verify payload format: `public/auth{id}{api_key}{nonce}`
- Check API key/secret
- Ensure nonce is unique and recent

---

### Missing Updates

**Cause:** Not responding to heartbeats

**Solution:**
```rust
if msg["method"] == "public/heartbeat" {
    send_pong().await;
}
```

---

## Summary

- Two WebSocket servers: User API and Market Data
- **Always** wait 1 second after connection
- User API requires authentication via `public/auth`
- Public channels: ticker, book, trade, candlestick
- Private channels: user.order, user.trade, user.balance, user.positions
- Respond to heartbeats to maintain connection
- Implement reconnection with exponential backoff
- Rate limits: 150 req/s (User), 100 req/s (Market)
