# Deribit WebSocket API

Complete specification for WebSocket connectivity, subscriptions, and real-time data streaming.

## Overview

Deribit's WebSocket API provides:
- **Real-time market data** (orderbook, trades, ticker)
- **Real-time account updates** (orders, positions, trades)
- **JSON-RPC 2.0 over WebSocket**
- **Bidirectional communication** (request/response + push notifications)
- **Subscriptions** for continuous data streams

**Recommendation**: WebSocket is the preferred transport mechanism because:
- Faster than HTTP (persistent connection)
- Supports subscriptions (push-based updates)
- Supports cancel-on-disconnect (orders cancelled if connection drops)
- Lower latency for trading operations

---

## WebSocket Endpoints

### Production
```
wss://www.deribit.com/ws/api/v2
```

### Test Environment
```
wss://test.deribit.com/ws/api/v2
```

**Note**: Test and production require separate accounts and API keys.

---

## Connection Limits

- **Max 32 connections per IP address**
- **Max 16 sessions per API key**

**Best Practice**: Reuse connections for multiple subscriptions (up to 500 channels per connection).

---

## Connection Lifecycle

### 1. Connect

Establish WebSocket connection:

```javascript
const ws = new WebSocket('wss://test.deribit.com/ws/api/v2');

ws.onopen = () => {
    console.log('WebSocket connected');
};
```

```rust
use tokio_tungstenite::connect_async;

let (ws_stream, _) = connect_async("wss://test.deribit.com/ws/api/v2").await?;
```

---

### 2. Authenticate (for private methods)

Send `public/auth` request over the WebSocket:

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "public/auth",
  "params": {
    "grant_type": "client_credentials",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET"
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "access_token": "eyJ0eXAiOiJKV1QiLC...",
    "token_type": "bearer",
    "refresh_token": "eyJ0eXAiOiJKV1QiLC...",
    "expires_in": 900,
    "scope": "trade:read trade:write"
  }
}
```

After successful authentication, **all subsequent messages on this connection are authenticated**.

---

### 3. Subscribe to Channels

Send subscription request:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "public/subscribe",
  "params": {
    "channels": [
      "ticker.BTC-PERPETUAL.100ms",
      "book.BTC-PERPETUAL.100ms",
      "trades.BTC-PERPETUAL.100ms"
    ]
  }
}
```

**Response**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": [
    "ticker.BTC-PERPETUAL.100ms",
    "book.BTC-PERPETUAL.100ms",
    "trades.BTC-PERPETUAL.100ms"
  ]
}
```

---

### 4. Receive Notifications

Server pushes updates as notifications (no `id` field):

```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": {
    "channel": "ticker.BTC-PERPETUAL.100ms",
    "data": {
      "timestamp": 1590484645991,
      "last_price": 9050.5,
      "best_bid_price": 9050.0,
      "best_ask_price": 9051.0
    }
  }
}
```

**Format**:
- `method`: Always `"subscription"`
- `params.channel`: Which subscription channel
- `params.data`: Channel-specific data

---

### 5. Heartbeat (Keep-Alive)

**Server-side heartbeat**: Deribit may send `test` requests:

```json
{
  "jsonrpc": "2.0",
  "id": 8212,
  "method": "public/test"
}
```

**You must respond**:
```json
{
  "jsonrpc": "2.0",
  "id": 8212,
  "result": {
    "version": "1.0.0"
  }
}
```

**Client-side heartbeat**: Send `public/test` periodically to keep connection alive:

```json
{
  "jsonrpc": "2.0",
  "id": 1234,
  "method": "public/test"
}
```

**Recommendation**: Send heartbeat every 30 seconds.

---

### 6. Disconnect

Close WebSocket connection gracefully:

```javascript
ws.close();
```

**Cancel-on-Disconnect**: If you have open orders and connection drops, orders may be cancelled (depending on order settings).

---

## Subscription Methods

### public/subscribe

Subscribe to **public channels** (market data).

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "public/subscribe",
  "params": {
    "channels": ["book.BTC-PERPETUAL.100ms", "ticker.ETH-PERPETUAL.100ms"]
  }
}
```

**Channels**: Array of channel names (max 500 per request)

---

### private/subscribe

Subscribe to **private channels** (account-specific data).

**Requires**: Authenticated connection

**Request**:
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "private/subscribe",
  "params": {
    "channels": ["user.orders.BTC-PERPETUAL.raw", "user.trades.BTC-PERPETUAL.raw"]
  }
}
```

---

### public/unsubscribe

Unsubscribe from channels:

```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "public/unsubscribe",
  "params": {
    "channels": ["book.BTC-PERPETUAL.100ms"]
  }
}
```

---

## Public Channels

### Book Channel: `book.{instrument}.{interval}`

Real-time orderbook updates.

**Format**: `book.{instrument_name}.{interval}`

**Intervals**:
- `raw` - Every update (requires authentication)
- `100ms` - Aggregated every 100ms (recommended)
- `agg2` - Aggregated (lower frequency)

**Examples**:
- `book.BTC-PERPETUAL.100ms`
- `book.ETH-29MAR24.100ms`
- `book.BTC-27DEC24-50000-C.raw`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "book.BTC-PERPETUAL.100ms",
    "data": {
      "type": "change",
      "timestamp": 1590484645991,
      "instrument_name": "BTC-PERPETUAL",
      "change_id": 37580800,
      "bids": [
        ["change", 9050.5, 12340],
        ["delete", 9050.0, 0]
      ],
      "asks": [
        ["new", 9051.0, 5670]
      ]
    }
  }
}
```

**Entry Format**: `[action, price, amount]`
- `action`: `"new"`, `"change"`, `"delete"`
- `price`: Price level
- `amount`: Total quantity at this price (0 for delete)

**Types**:
- `snapshot` - Full orderbook (initial message)
- `change` - Delta update (incremental)

**Maintaining Orderbook**:
1. Receive `snapshot` (full book)
2. Apply `change` updates incrementally
3. Use `change_id` to detect missed updates (reconnect if gap detected)

---

### Ticker Channel: `ticker.{instrument}.{interval}`

Real-time ticker updates (best bid/ask, mark price, volume, etc.).

**Format**: `ticker.{instrument_name}.{interval}`

**Intervals**: `100ms`, `agg2`, `raw`

**Examples**:
- `ticker.BTC-PERPETUAL.100ms`
- `ticker.ETH-PERPETUAL.agg2`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "ticker.BTC-PERPETUAL.100ms",
    "data": {
      "timestamp": 1590484645991,
      "stats": {
        "volume": 15894.89,
        "price_change": -1.48,
        "low": 8744.5,
        "high": 9061.5
      },
      "state": "open",
      "settlement_price": 9003.23,
      "open_interest": 88234.5,
      "min_price": 8676.82,
      "max_price": 9332.44,
      "mark_price": 9004.63,
      "last_price": 9004.5,
      "instrument_name": "BTC-PERPETUAL",
      "index_price": 9002.76,
      "funding_8h": 0.00003852,
      "current_funding": 0.00001284,
      "best_bid_price": 9004.5,
      "best_bid_amount": 51960,
      "best_ask_price": 9005.0,
      "best_ask_amount": 68750
    }
  }
}
```

**Key Fields**:
- `last_price`: Last traded price
- `best_bid_price` / `best_ask_price`: Top of book
- `mark_price`: Fair value (for liquidations)
- `index_price`: Spot index
- `funding_8h`: 8-hour funding rate (perpetuals)
- `open_interest`: Total open contracts
- `volume`: 24h volume

---

### Trades Channel: `trades.{instrument}.{interval}`

Real-time trade feed.

**Format**: `trades.{instrument_name}.{interval}`

**Intervals**: `raw`, `100ms`, `agg2`

**Examples**:
- `trades.BTC-PERPETUAL.raw`
- `trades.ETH-PERPETUAL.100ms`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "trades.BTC-PERPETUAL.raw",
    "data": [
      {
        "trade_seq": 35684502,
        "trade_id": "48079338",
        "timestamp": 1590484645991,
        "tick_direction": 1,
        "price": 9052.5,
        "instrument_name": "BTC-PERPETUAL",
        "index_price": 9048.33,
        "direction": "buy",
        "amount": 40
      }
    ]
  }
}
```

**Fields**:
- `trade_id`: Unique trade identifier
- `trade_seq`: Sequence number (incremental)
- `direction`: `"buy"` or `"sell"` (taker side)
- `price`: Trade price
- `amount`: Trade quantity
- `tick_direction`: `0` (zero plus tick), `1` (plus tick), `2` (minus tick), `3` (zero minus tick)

---

### Deribit Price Index: `deribit_price_index.{index_name}`

Spot price index updates.

**Examples**:
- `deribit_price_index.btc_usd`
- `deribit_price_index.eth_usd`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "deribit_price_index.btc_usd",
    "data": {
      "timestamp": 1590484645991,
      "price": 9050.25,
      "index_name": "btc_usd"
    }
  }
}
```

---

### Deribit Price Ranking: `deribit_price_ranking.{index_name}`

Constituent exchange prices for index calculation.

**Examples**:
- `deribit_price_ranking.btc_usd`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "deribit_price_ranking.btc_usd",
    "data": {
      "timestamp": 1590484645991,
      "price": 9050.25,
      "identifier": "bitstamp",
      "weight": 10,
      "original_price": 9051.00,
      "enabled": true
    }
  }
}
```

---

## Private Channels

### User Orders: `user.orders.{kind}.{currency}.{interval}`

Real-time order updates (placement, fills, cancellations).

**Format**: `user.orders.{kind}.{currency}.{interval}`

**Kind**: `future`, `option`, `spot`, `future_combo`, `option_combo`, or wildcard `*`
**Currency**: `BTC`, `ETH`, `SOL`, etc., or wildcard `any`
**Interval**: `raw`, `100ms`

**Examples**:
- `user.orders.BTC-PERPETUAL.raw` - Orders for specific instrument
- `user.orders.future.BTC.raw` - All BTC futures orders
- `user.orders.any.any.raw` - All orders (all currencies, all kinds)

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "user.orders.BTC-PERPETUAL.raw",
    "data": {
      "order_id": "4008314325",
      "order_state": "open",
      "max_show": 40,
      "api": true,
      "amount": 40,
      "web": false,
      "time_in_force": "good_til_cancelled",
      "replaced": false,
      "reduce_only": false,
      "profit_loss": 0.0,
      "price": 9050.0,
      "post_only": false,
      "order_type": "limit",
      "last_update_timestamp": 1590484645991,
      "label": "",
      "is_liquidation": false,
      "instrument_name": "BTC-PERPETUAL",
      "filled_amount": 0,
      "direction": "buy",
      "creation_timestamp": 1590484645991,
      "commission": 0.0,
      "average_price": 0.0
    }
  }
}
```

**Order States**:
- `open` - Active order
- `filled` - Fully executed
- `rejected` - Rejected by exchange
- `cancelled` - Cancelled by user or system
- `untriggered` - Stop order not yet triggered

---

### User Trades: `user.trades.{kind}.{currency}.{interval}`

Real-time trade notifications (fills).

**Format**: `user.trades.{kind}.{currency}.{interval}`

**Examples**:
- `user.trades.BTC-PERPETUAL.raw`
- `user.trades.future.BTC.raw`
- `user.trades.any.any.100ms`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "user.trades.BTC-PERPETUAL.raw",
    "data": [
      {
        "trade_seq": 35684502,
        "trade_id": "48079338",
        "timestamp": 1590484645991,
        "tick_direction": 1,
        "state": "filled",
        "reduce_only": false,
        "price": 9052.5,
        "post_only": false,
        "order_type": "market",
        "order_id": "4008314325",
        "matching_id": null,
        "mark_price": 9048.79,
        "liquidity": "T",
        "instrument_name": "BTC-PERPETUAL",
        "index_price": 9048.33,
        "fee_currency": "BTC",
        "fee": 0.00002244,
        "direction": "buy",
        "amount": 40
      }
    ]
  }
}
```

**Fields**:
- `trade_id`: Unique trade identifier
- `order_id`: Originating order ID
- `liquidity`: `"M"` (maker) or `"T"` (taker)
- `fee`: Trading fee
- `price`: Execution price

---

### User Portfolio: `user.portfolio.{currency}`

Portfolio-level updates (margin, balance, P&L).

**Format**: `user.portfolio.{currency}`

**Examples**:
- `user.portfolio.BTC`
- `user.portfolio.ETH`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "user.portfolio.BTC",
    "data": {
      "total_pl": 0.0234,
      "session_upl": 0.0012,
      "session_rpl": 0.0,
      "projected_maintenance_margin": 0.1234,
      "projected_initial_margin": 0.2345,
      "portfolio_margining_enabled": false,
      "margin_balance": 1.2345,
      "maintenance_margin": 0.1234,
      "initial_margin": 0.2345,
      "futures_session_upl": 0.0012,
      "futures_session_rpl": 0.0,
      "futures_pl": 0.0234,
      "fee_balance": 0.0,
      "equity": 1.2579,
      "delta_total": 1.5,
      "currency": "BTC",
      "balance": 1.2345,
      "available_withdrawal_funds": 1.0,
      "available_funds": 1.0111
    }
  }
}
```

---

### User Changes: `user.changes.{kind}.{currency}.{interval}`

Combined feed (orders + trades + positions).

**Format**: `user.changes.{kind}.{currency}.{interval}`

**Examples**:
- `user.changes.BTC-PERPETUAL.raw`
- `user.changes.future.BTC.100ms`

**Notification Data**:
```json
{
  "method": "subscription",
  "params": {
    "channel": "user.changes.BTC-PERPETUAL.raw",
    "data": {
      "trades": [ /* array of trades */ ],
      "orders": [ /* array of orders */ ],
      "positions": [ /* array of positions */ ]
    }
  }
}
```

**Use Case**: Single subscription for all user updates (convenient but higher data volume).

---

## Channel Wildcards

Use wildcards for broad subscriptions:

**Currency Wildcard**:
- `user.orders.future.any.raw` - All futures, all currencies

**Kind Wildcard**:
- `user.orders.*.BTC.raw` - All instrument kinds for BTC

**Instrument Wildcard**:
- `ticker.*.100ms` - NOT SUPPORTED (must specify instrument)

**Note**: Public channels (book, ticker, trades) do NOT support wildcards; you must specify the exact instrument.

---

## Batch Subscriptions

Subscribe to multiple channels in one request (up to **500 channels**):

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "public/subscribe",
  "params": {
    "channels": [
      "ticker.BTC-PERPETUAL.100ms",
      "ticker.ETH-PERPETUAL.100ms",
      "book.BTC-PERPETUAL.100ms",
      "book.ETH-PERPETUAL.100ms",
      "trades.BTC-PERPETUAL.100ms",
      "trades.ETH-PERPETUAL.100ms"
    ]
  }
}
```

**Benefit**: One request, one credit cost, multiple streams.

---

## Subscription Best Practices

### 1. Batch Subscriptions on Connect

Prepare all channels upfront and subscribe once:

```rust
let channels = vec![
    "ticker.BTC-PERPETUAL.100ms",
    "book.BTC-PERPETUAL.100ms",
    "trades.BTC-PERPETUAL.100ms",
    "user.orders.any.any.raw",
    "user.trades.any.any.raw",
];

subscribe(&channels).await?;
```

---

### 2. Choose Intervals Wisely

| Interval | Use Case | Update Frequency |
|----------|----------|------------------|
| `raw` | High-frequency trading | Every update (highest load) |
| `100ms` | Regular trading | Every 100ms (good balance) |
| `agg2` | Monitoring/charting | Aggregated (lowest load) |

**Recommendation**: Use `100ms` unless you need tick-by-tick data.

---

### 3. Subscribe to `raw` Feeds Only When Needed

**Raw feeds require authentication** (anti-abuse measure).

- `book.BTC-PERPETUAL.raw` - Requires authenticated connection
- `book.BTC-PERPETUAL.100ms` - Can use public connection

**Why**: Prevents unauthenticated abuse of high-frequency streams.

---

### 4. Monitor Subscription Confirmations

Check subscription response for failures:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": [
    "ticker.BTC-PERPETUAL.100ms",
    "book.BTC-PERPETUAL.100ms"
  ]
}
```

If a channel is missing from the result, subscription failed (check channel name).

---

### 5. Unsubscribe from Unused Channels

Free up resources:

```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "public/unsubscribe",
  "params": {
    "channels": ["ticker.ETH-PERPETUAL.100ms"]
  }
}
```

---

## Reconnection Strategy

### Detecting Disconnection

```rust
ws_stream.next().await {
    Some(Ok(msg)) => {
        // Process message
    },
    Some(Err(e)) => {
        // Connection error
        warn!("WebSocket error: {}", e);
        reconnect().await?;
    },
    None => {
        // Connection closed
        warn!("WebSocket closed");
        reconnect().await?;
    }
}
```

---

### Reconnection Steps

1. **Wait with Backoff**:
   ```rust
   let mut backoff = Duration::from_secs(1);
   loop {
       tokio::time::sleep(backoff).await;
       match connect().await {
           Ok(_) => break,
           Err(_) => {
               backoff = min(backoff * 2, Duration::from_secs(60));
           }
       }
   }
   ```

2. **Re-authenticate**:
   ```rust
   let auth_response = authenticate(client_id, client_secret).await?;
   ```

3. **Re-subscribe**:
   ```rust
   subscribe(&channels).await?;
   ```

4. **Resynchronize State**:
   - Fetch current orderbook snapshot
   - Fetch open orders
   - Fetch positions

---

### Heartbeat Implementation

**Send periodic test messages**:

```rust
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        if let Err(e) = send_test().await {
            warn!("Heartbeat failed: {}", e);
            break;
        }
    }
});
```

**Handle incoming test requests**:

```rust
if msg.method == "public/test" {
    send_response(msg.id, json!({"version": "1.0.0"})).await?;
}
```

---

## Cancel-on-Disconnect

**Feature**: Orders can be automatically cancelled if WebSocket connection drops.

**How to Enable**: Use `cancel_on_disconnect` scope when authenticating:

```json
{
  "method": "public/auth",
  "params": {
    "grant_type": "client_credentials",
    "client_id": "YOUR_CLIENT_ID",
    "client_secret": "YOUR_CLIENT_SECRET",
    "scope": "session:my_session trade:read trade:write cancel_on_disconnect"
  }
}
```

**Behavior**:
- If connection drops, all orders placed during this session are cancelled
- Protects against orphan orders due to connectivity issues
- Useful for market-making bots

**Disable for Manual Trading**: If you want orders to persist after disconnect, don't include `cancel_on_disconnect` scope.

---

## Message Routing

### Handling Responses vs Notifications

**Responses** (have `id` field):
```json
{
  "jsonrpc": "2.0",
  "id": 123,
  "result": { /* result data */ }
}
```
Match `id` to pending requests.

**Notifications** (no `id` field):
```json
{
  "jsonrpc": "2.0",
  "method": "subscription",
  "params": { /* notification data */ }
}
```
Route by `params.channel`.

**Implementation**:
```rust
match msg {
    Message::Response { id, result } => {
        // Match to pending request
        pending_requests.get(&id)?.resolve(result);
    },
    Message::Notification { method, params } => {
        if method == "subscription" {
            let channel = params.channel;
            route_to_handler(channel, params.data);
        } else if method == "public/test" {
            send_response(msg.id, json!({"version": "1.0.0"}));
        }
    }
}
```

---

## Error Handling

### Subscription Errors

If subscription fails:

```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": 10030,
    "message": "subscription_not_found"
  }
}
```

**Common Errors**:
- **10030**: `subscription_not_found` - Invalid channel name
- **13001**: `authorization_required` - Authentication needed for private/raw channels
- **11042**: `must_be_websocket_request` - Method only available via WebSocket

---

### Connection Errors

| Error | Cause | Solution |
|-------|-------|----------|
| Connection timeout | Network issue | Retry with backoff |
| TLS handshake failure | Certificate issue | Verify endpoint URL |
| Authentication failure | Wrong credentials | Check API key and secret |
| Rate limit exceeded | Too many connections | Close unused connections |

---

## Performance Optimization

### 1. Connection Pooling

Reuse connections for multiple subscriptions:

```rust
// GOOD: One connection, many subscriptions
let ws = connect().await?;
subscribe(&ws, &["ticker.BTC-PERPETUAL.100ms", "book.BTC-PERPETUAL.100ms"]).await?;

// BAD: Multiple connections for each subscription
let ws1 = connect().await?;
subscribe(&ws1, &["ticker.BTC-PERPETUAL.100ms"]).await?;
let ws2 = connect().await?;
subscribe(&ws2, &["book.BTC-PERPETUAL.100ms"]).await?;
```

---

### 2. Efficient Message Parsing

Use streaming JSON parsers:

```rust
use serde_json::from_str;

async fn handle_message(msg: String) {
    let notification: Notification = from_str(&msg)?;
    route_notification(notification).await;
}
```

---

### 3. Backpressure Handling

Prevent message queue overflow:

```rust
let (tx, mut rx) = mpsc::channel(1000);

// Producer (WebSocket reader)
tokio::spawn(async move {
    while let Some(msg) = ws_stream.next().await {
        if tx.send(msg).await.is_err() {
            warn!("Channel full, dropping message");
        }
    }
});

// Consumer (message handler)
tokio::spawn(async move {
    while let Some(msg) = rx.recv().await {
        handle_message(msg).await;
    }
});
```

---

## Implementation Checklist

For V5 connector:

- [ ] Connect to WebSocket endpoint (test and production)
- [ ] Implement authentication over WebSocket (`public/auth`)
- [ ] Implement subscription methods (`public/subscribe`, `private/subscribe`)
- [ ] Handle public channels (book, ticker, trades)
- [ ] Handle private channels (user.orders, user.trades, user.portfolio)
- [ ] Implement unsubscribe functionality
- [ ] Batch subscriptions (up to 500 channels)
- [ ] Handle subscription notifications (route by channel)
- [ ] Distinguish responses (with `id`) from notifications (without `id`)
- [ ] Implement heartbeat (send `public/test` every 30s)
- [ ] Handle incoming `public/test` requests (respond with version)
- [ ] Implement reconnection with exponential backoff
- [ ] Re-authenticate and re-subscribe after reconnect
- [ ] Support cancel-on-disconnect (optional)
- [ ] Limit concurrent connections (32 per IP, 16 per API key)
- [ ] Handle subscription errors (10030, 13001, 11042)
- [ ] Implement backpressure handling
- [ ] Log WebSocket events (connect, disconnect, errors)
- [ ] Monitor WebSocket latency (use `usDiff` from responses)

---

## Example: Full WebSocket Client Flow

```rust
// 1. Connect
let (ws_stream, _) = connect_async("wss://test.deribit.com/ws/api/v2").await?;
let (mut write, mut read) = ws_stream.split();

// 2. Authenticate
let auth_msg = json!({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "public/auth",
    "params": {
        "grant_type": "client_credentials",
        "client_id": env::var("DERIBIT_CLIENT_ID")?,
        "client_secret": env::var("DERIBIT_CLIENT_SECRET")?
    }
});
write.send(Message::Text(auth_msg.to_string())).await?;

// Wait for auth response
let auth_response = read.next().await.unwrap()?;
// Parse and store access token

// 3. Subscribe
let subscribe_msg = json!({
    "jsonrpc": "2.0",
    "id": 2,
    "method": "public/subscribe",
    "params": {
        "channels": [
            "ticker.BTC-PERPETUAL.100ms",
            "book.BTC-PERPETUAL.100ms",
            "user.orders.any.any.raw"
        ]
    }
});
write.send(Message::Text(subscribe_msg.to_string())).await?;

// 4. Start heartbeat
tokio::spawn(async move {
    let mut interval = tokio::time::interval(Duration::from_secs(30));
    loop {
        interval.tick().await;
        let test_msg = json!({"jsonrpc": "2.0", "id": 9999, "method": "public/test"});
        write.send(Message::Text(test_msg.to_string())).await?;
    }
});

// 5. Handle messages
while let Some(msg) = read.next().await {
    match msg? {
        Message::Text(text) => {
            let parsed: Value = serde_json::from_str(&text)?;

            if parsed.get("method") == Some(&json!("subscription")) {
                // Notification
                let channel = parsed["params"]["channel"].as_str().unwrap();
                let data = &parsed["params"]["data"];
                handle_notification(channel, data).await;
            } else if parsed.get("id").is_some() {
                // Response to request
                handle_response(parsed).await;
            }
        },
        Message::Close(_) => {
            warn!("WebSocket closed");
            reconnect().await?;
            break;
        },
        _ => {}
    }
}
```

---

## References

- Deribit API Documentation: https://docs.deribit.com/
- WebSocket API Guide: https://docs.deribit.com/articles/deribit-quickstart
- How to Maintain WebSocket Connection: https://insights.deribit.com/dev-hub/how-to-maintain-and-authenticate-a-websocket-connection-to-deribit-python/
- Market Data Best Practices: https://support.deribit.com/hc/en-us/articles/29592500256669-Market-Data-Collection-Best-Practices
