# Lighter Exchange WebSocket API

## Overview

Lighter provides WebSocket connections for real-time market data and account updates. WebSocket is the preferred method for receiving live data as it reduces REST API usage and provides lower latency.

---

## Connection Details

### Endpoints

- **Mainnet**: `wss://mainnet.zklighter.elliot.ai/stream`
- **Testnet**: `wss://testnet.zklighter.elliot.ai/stream`

### Protocol

- **Protocol**: WSS (WebSocket Secure)
- **Encoding**: JSON
- **Compression**: Not specified (assume uncompressed)

---

## Connection Limits

See `rate_limits.md` for detailed limits.

**Per IP Limits**:
- Max Connections: 100
- Max Subscriptions per Connection: 100
- Max Total Subscriptions: 1,000
- Max Connections Per Minute: 60
- Max Messages Per Minute: 200 (excludes sendTx/sendBatchTx)
- Max Inflight Messages: 50
- Max Unique Accounts: 10

---

## Connection Lifecycle

### 1. Establish Connection

```javascript
// Example with wscat
wscat -c wss://mainnet.zklighter.elliot.ai/stream

// Example with JavaScript
const ws = new WebSocket('wss://mainnet.zklighter.elliot.ai/stream');

ws.onopen = () => {
    console.log('Connected to Lighter WebSocket');
};
```

---

### 2. Subscribe to Channels

**Subscription Message Format**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "channel_name"
  }
}
```

**For Authenticated Channels**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "channel_name",
    "auth": "auth_token"
  }
}
```

---

### 3. Receive Updates

Server sends updates as JSON messages with channel-specific data.

---

### 4. Unsubscribe (Optional)

```json
{
  "method": "unsubscribe",
  "params": {
    "channel": "channel_name"
  }
}
```

---

### 5. Close Connection

```javascript
ws.close();
```

---

## Public Channels (No Authentication)

### 1. Order Book Channel

**Channel**: `order_book/{MARKET_INDEX}`

**Description**: Real-time order book updates sent in batches every 50ms

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "order_book/0"
  }
}
```

**Update Message**:
```json
{
  "channel": "order_book/0",
  "type": "update/orderbook",
  "timestamp": 1640995200,
  "asks": [
    ["3024.66", "1.5"],
    ["3025.00", "2.0"]
  ],
  "bids": [
    ["3024.00", "1.0"],
    ["3023.50", "0.5"]
  ],
  "offset": 12345,
  "nonce": 67890
}
```

**Fields**:
- `channel` (string): Channel identifier
- `type` (string): Update type
- `timestamp` (integer): Unix timestamp
- `asks` (array): Ask orders as [price, size] pairs
- `bids` (array): Bid orders as [price, size] pairs
- `offset` (integer): Update offset/sequence number
- `nonce` (integer): Orderbook nonce

**Update Frequency**: 50ms batches

**Example** (Market ID 0 = ETH):
```json
{
  "method": "subscribe",
  "params": {
    "channel": "order_book/0"
  }
}
```

---

### 2. Market Statistics Channel

**Channel**: `market_stats/{MARKET_INDEX}`

**Description**: Market statistics and funding rates

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "market_stats/0"
  }
}
```

**Update Message**:
```json
{
  "channel": "market_stats/0",
  "type": "update/market_stats",
  "timestamp": 1640995200,
  "last_price": "3024.66",
  "daily_volume": "1000000.0",
  "daily_high": "3100.00",
  "daily_low": "2950.00",
  "daily_change": "50.00",
  "funding_rate": "0.0001",
  "open_interest": "5000.0"
}
```

**Update Frequency**: Periodic (when statistics change)

---

### 3. Trade Channel

**Channel**: `trade/{MARKET_INDEX}`

**Description**: Real-time trade execution data

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "trade/0"
  }
}
```

**Update Message**:
```json
{
  "channel": "trade/0",
  "type": "update/trade",
  "timestamp": 1640995200,
  "trade_id": 12345,
  "price": "3024.66",
  "size": "1.5",
  "side": "buy",
  "is_maker_ask": true
}
```

**Fields**:
- `trade_id` (integer): Unique trade identifier
- `price` (string): Trade price
- `size` (string): Trade size
- `side` (string): "buy" or "sell" (from taker perspective)
- `is_maker_ask` (boolean): True if ask was maker side

**Update Frequency**: Real-time (as trades execute)

---

### 4. Account All Channel

**Channel**: `account_all/{ACCOUNT_ID}`

**Description**: Comprehensive account data across all markets

**Authentication**: No (but limited to public account data)

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "account_all/1"
  }
}
```

**Update Message**:
```json
{
  "channel": "account_all/1",
  "type": "update/account",
  "timestamp": 1640995200,
  "account_index": 1,
  "collateral": "50000.0",
  "available_balance": "25000.0",
  "positions": [
    {
      "market_id": 0,
      "position": "3.5",
      "avg_entry_price": "3000.00",
      "unrealized_pnl": "86.31"
    }
  ]
}
```

---

### 5. User Statistics Channel

**Channel**: `user_stats/{ACCOUNT_ID}`

**Description**: Account statistics including leverage and margins

**Authentication**: No

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "user_stats/1"
  }
}
```

**Update Message**:
```json
{
  "channel": "user_stats/1",
  "type": "update/user_stats",
  "timestamp": 1640995200,
  "account_index": 1,
  "collateral": "50000.0",
  "portfolio_value": "51000.0",
  "leverage": "2.5",
  "available_balance": "25000.0",
  "margin_usage": "0.25",
  "buying_power": "75000.0"
}
```

**Fields**:
- `collateral` (string): Total collateral
- `portfolio_value` (string): Total portfolio value including unrealized PnL
- `leverage` (string): Current leverage ratio
- `available_balance` (string): Available for trading
- `margin_usage` (string): Margin usage ratio (0-1)
- `buying_power` (string): Maximum order value

---

### 6. Height Channel

**Channel**: `height`

**Description**: Blockchain height updates

**Authentication**: No

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "height"
  }
}
```

**Update Message**:
```json
{
  "channel": "height",
  "type": "update/height",
  "timestamp": 1640995200,
  "height": 123456
}
```

---

## Authenticated Channels

These channels require an auth token in the subscription message.

### Authentication Token

**Generation**: See `authentication.md` for details

**Token Structure**:
```
{expiry_unix}:{account_index}:{api_key_index}:{random_hex}
```

**Example**:
```
1640999999:1:3:a1b2c3d4e5f6
```

---

### 7. Account Market Channel

**Channel**: `account_market/{MARKET_ID}/{ACCOUNT_ID}`

**Description**: Market-specific account data

**Authentication**: **Required**

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "account_market/0/1",
    "auth": "1640999999:1:3:a1b2c3d4e5f6"
  }
}
```

**Update Message**:
```json
{
  "channel": "account_market/0/1",
  "type": "update/account_market",
  "timestamp": 1640995200,
  "market_id": 0,
  "account_index": 1,
  "position": "3.5",
  "avg_entry_price": "3000.00",
  "unrealized_pnl": "86.31",
  "realized_pnl": "100.00",
  "open_orders": [
    {
      "order_id": 12345,
      "side": "buy",
      "price": "2950.00",
      "size": "1.0",
      "filled": "0.0"
    }
  ]
}
```

---

### 8. Account Transactions Channel

**Channel**: `account_tx/{ACCOUNT_ID}`

**Description**: Account transaction history

**Authentication**: **Required**

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "account_tx/1",
    "auth": "1640999999:1:3:a1b2c3d4e5f6"
  }
}
```

**Update Message**:
```json
{
  "channel": "account_tx/1",
  "type": "update/transaction",
  "timestamp": 1640995200,
  "tx_hash": "0xabc123...",
  "tx_type": 14,
  "status": "executed",
  "details": {
    "market_id": 0,
    "side": "buy",
    "size": "1.5",
    "price": "3024.66"
  }
}
```

---

### 9. Notification Channel

**Channel**: `notification/{ACCOUNT_ID}`

**Description**: Liquidation, deleverage, and announcement notifications

**Authentication**: **Required**

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "notification/1",
    "auth": "1640999999:1:3:a1b2c3d4e5f6"
  }
}
```

**Update Message**:
```json
{
  "channel": "notification/1",
  "type": "update/notification",
  "timestamp": 1640995200,
  "notification_type": "liquidation",
  "severity": "critical",
  "message": "Position liquidated in market ETH",
  "details": {
    "market_id": 0,
    "position_size": "5.0",
    "liquidation_price": "2800.00"
  }
}
```

**Notification Types**:
- `liquidation` - Position liquidated
- `deleverage` - Position deleveraged
- `announcement` - System announcements
- `warning` - Margin warnings

---

### 10. Pool Data Channel

**Channel**: `pool_data/{ACCOUNT_ID}`

**Description**: Liquidity pool activity tracking

**Authentication**: **Required**

**Subscription**:
```json
{
  "method": "subscribe",
  "params": {
    "channel": "pool_data/1",
    "auth": "1640999999:1:3:a1b2c3d4e5f6"
  }
}
```

**Update Message**:
```json
{
  "channel": "pool_data/1",
  "type": "update/pool_data",
  "timestamp": 1640995200,
  "pool_id": 100,
  "shares": "1000.0",
  "value": "10500.0",
  "pnl": "500.0"
}
```

---

## Transaction Submission via WebSocket

### Send Transaction

**Message Type**: `jsonapi/sendtx`

**Structure**:
```json
{
  "type": "jsonapi/sendtx",
  "data": {
    "tx_type": 14,
    "tx_info": {
      "account_index": 1,
      "api_key_index": 3,
      "market_id": 0,
      "base_amount": "1000000",
      "price": "30246600",
      "side": "buy",
      "order_type": "limit",
      "client_order_index": 12345,
      "nonce": 1,
      "signature": "0x..."
    }
  }
}
```

**Transaction Types**:
- `14` - L2CreateOrder
- `15` - L2CancelOrder
- `17` - L2ModifyOrder

**Response**:
```json
{
  "type": "response/sendtx",
  "timestamp": 1640995200,
  "success": true,
  "tx_hash": "0xabc123...",
  "message": "Transaction accepted"
}
```

---

### Send Transaction Batch

**Message Type**: `jsonapi/sendtxbatch`

**Structure**:
```json
{
  "type": "jsonapi/sendtxbatch",
  "data": {
    "tx_types": [14, 15],
    "tx_infos": [
      {
        "account_index": 1,
        "api_key_index": 3,
        "market_id": 0,
        "base_amount": "1000000",
        "price": "30246600",
        "side": "buy",
        "order_type": "limit",
        "client_order_index": 12345,
        "nonce": 1,
        "signature": "0x..."
      },
      {
        "account_index": 1,
        "api_key_index": 3,
        "order_index": 12344,
        "nonce": 2,
        "signature": "0x..."
      }
    ]
  }
}
```

**Batch Limit**: Up to 50 transactions

**Response**:
```json
{
  "type": "response/sendtxbatch",
  "timestamp": 1640995200,
  "success": true,
  "tx_hashes": ["0xabc123...", "0xdef456..."],
  "message": "Batch accepted"
}
```

**Note**: Transaction submissions via WebSocket count toward REST API rate limits (6 weight per transaction).

---

## Message Format

### General Structure

All WebSocket messages follow this pattern:

```json
{
  "channel": "channel_name",
  "type": "update/[type]",
  "timestamp": 1640995200,
  "[data_fields]": {}
}
```

**Common Fields**:
- `channel` (string): Channel identifier
- `type` (string): Message type (e.g., "update/orderbook", "update/trade")
- `timestamp` (integer): Unix timestamp of update

---

### Error Messages

```json
{
  "type": "error",
  "timestamp": 1640995200,
  "error_code": "INVALID_AUTH",
  "message": "Authentication token expired",
  "channel": "account_market/0/1"
}
```

**Common Error Codes**:
- `INVALID_AUTH` - Invalid or expired auth token
- `INVALID_CHANNEL` - Channel doesn't exist
- `RATE_LIMIT` - Too many messages
- `SUBSCRIPTION_LIMIT` - Too many subscriptions

---

## Heartbeat / Ping-Pong

### Server-Initiated Ping

Server may send ping messages to keep connection alive.

**Ping Message**:
```json
{
  "type": "ping",
  "timestamp": 1640995200
}
```

**Required Response**:
```json
{
  "type": "pong",
  "timestamp": 1640995200
}
```

**Timeout**: Respond within 30 seconds (verify actual timeout)

---

### Client-Initiated Ping

Clients can send ping to check connection.

**Client Ping**:
```json
{
  "type": "ping"
}
```

**Server Response**:
```json
{
  "type": "pong",
  "timestamp": 1640995200
}
```

---

## Connection Management

### Reconnection Strategy

**Implement Exponential Backoff**:
1. Initial delay: 1 second
2. On failure: delay * 2
3. Max delay: 60 seconds
4. Reset on successful connection

**Example (JavaScript)**:
```javascript
let reconnectDelay = 1000;
const maxDelay = 60000;

function connect() {
    const ws = new WebSocket('wss://mainnet.zklighter.elliot.ai/stream');

    ws.onopen = () => {
        console.log('Connected');
        reconnectDelay = 1000;  // Reset on success
        // Resubscribe to channels
    };

    ws.onclose = () => {
        console.log(`Reconnecting in ${reconnectDelay}ms`);
        setTimeout(connect, reconnectDelay);
        reconnectDelay = Math.min(reconnectDelay * 2, maxDelay);
    };

    ws.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
}

connect();
```

---

### Subscription Recovery

After reconnection, resubscribe to all channels:

```javascript
const subscriptions = [];

function subscribe(channel, auth = null) {
    const msg = {
        method: 'subscribe',
        params: { channel }
    };

    if (auth) {
        msg.params.auth = auth;
    }

    ws.send(JSON.stringify(msg));
    subscriptions.push({ channel, auth });
}

ws.onopen = () => {
    // Resubscribe to all channels
    subscriptions.forEach(({ channel, auth }) => {
        subscribe(channel, auth);
    });
};
```

---

### Connection Pooling

**Recommendation**: Use single connection with multiple subscriptions

**Why**:
- Avoid connection limits (max 100 per IP)
- Reduce overhead
- Simplify management

**Pattern**:
```javascript
// Single connection
const ws = new WebSocket('wss://mainnet.zklighter.elliot.ai/stream');

// Multiple subscriptions on same connection
subscribe('order_book/0');
subscribe('order_book/1');
subscribe('trade/0');
subscribe('account_all/1');
```

**When to Use Multiple Connections**:
- Isolate critical vs non-critical channels
- Separate accounts (due to 10 account limit per IP)
- Geographic distribution (different servers if available)

---

## Data Handling

### Orderbook Management

**Approach**: Snapshot + Incremental Updates

**Initial Subscription**:
1. Subscribe to `order_book/{market_id}`
2. Receive initial snapshot
3. Apply incremental updates using offset/nonce

**Update Logic**:
```javascript
let orderbook = { asks: [], bids: [], nonce: 0 };

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);

    if (msg.channel.startsWith('order_book/')) {
        if (msg.nonce > orderbook.nonce) {
            // Apply update
            orderbook.asks = msg.asks;
            orderbook.bids = msg.bids;
            orderbook.nonce = msg.nonce;
        } else {
            // Out-of-order update, ignore or resubscribe
            console.warn('Out-of-order update');
        }
    }
};
```

**Handling Gaps**:
- Track nonce sequence
- If gap detected, resubscribe to get fresh snapshot

---

### Trade Aggregation

**For Chart Data**: Aggregate trades into candles

```javascript
const trades = [];
let currentCandle = null;

ws.onmessage = (event) => {
    const msg = JSON.parse(event.data);

    if (msg.type === 'update/trade') {
        trades.push({
            timestamp: msg.timestamp,
            price: parseFloat(msg.price),
            size: parseFloat(msg.size),
            side: msg.side
        });

        updateCandle(msg);
    }
};

function updateCandle(trade) {
    const candleTime = Math.floor(trade.timestamp / 60) * 60;  // 1-min candles

    if (!currentCandle || currentCandle.timestamp !== candleTime) {
        currentCandle = {
            timestamp: candleTime,
            open: parseFloat(trade.price),
            high: parseFloat(trade.price),
            low: parseFloat(trade.price),
            close: parseFloat(trade.price),
            volume: parseFloat(trade.size)
        };
    } else {
        currentCandle.high = Math.max(currentCandle.high, parseFloat(trade.price));
        currentCandle.low = Math.min(currentCandle.low, parseFloat(trade.price));
        currentCandle.close = parseFloat(trade.price);
        currentCandle.volume += parseFloat(trade.size);
    }
}
```

---

## Best Practices

### 1. Minimize Connections

Use one connection with multiple subscriptions instead of multiple connections.

---

### 2. Handle Reconnections Gracefully

- Implement exponential backoff
- Resubscribe to all channels on reconnect
- Validate data continuity after reconnection

---

### 3. Validate Auth Tokens

- Refresh tokens before expiry (max 8 hours)
- Handle auth errors gracefully
- Resubscribe with new token when expired

---

### 4. Rate Limit Awareness

- Track messages sent (max 200/minute)
- Avoid subscription churn (rapid subscribe/unsubscribe)
- Use batch transactions when possible

---

### 5. Data Validation

- Validate nonce/offset sequences
- Handle out-of-order updates
- Detect and recover from data gaps

---

### 6. Error Handling

- Log all error messages
- Implement fallback to REST API if WebSocket fails
- Alert on repeated connection failures

---

### 7. Resource Management

- Unsubscribe from unused channels
- Close connections cleanly on shutdown
- Monitor memory usage for orderbook storage

---

## Implementation Example (Rust)

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

#[tokio::main]
async fn main() {
    let url = "wss://mainnet.zklighter.elliot.ai/stream";

    let (ws_stream, _) = connect_async(url)
        .await
        .expect("Failed to connect");

    println!("Connected to Lighter WebSocket");

    let (mut write, mut read) = ws_stream.split();

    // Subscribe to orderbook
    let subscribe_msg = json!({
        "method": "subscribe",
        "params": {
            "channel": "order_book/0"
        }
    });

    write.send(Message::Text(subscribe_msg.to_string()))
        .await
        .expect("Failed to subscribe");

    // Read messages
    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received: {}", text);
                // Parse and handle message
            },
            Ok(Message::Close(_)) => {
                println!("Connection closed");
                break;
            },
            Err(e) => {
                eprintln!("Error: {}", e);
                break;
            },
            _ => {}
        }
    }
}
```

---

## Channel Summary Table

| Channel | Auth Required | Description | Update Frequency |
|---------|---------------|-------------|------------------|
| `order_book/{market_id}` | No | Order book updates | 50ms batches |
| `market_stats/{market_id}` | No | Market statistics | On change |
| `trade/{market_id}` | No | Trade executions | Real-time |
| `account_all/{account_id}` | No | Account overview | On change |
| `user_stats/{account_id}` | No | Account statistics | On change |
| `height` | No | Blockchain height | On new block |
| `account_market/{market}/{account}` | Yes | Market-specific account data | On change |
| `account_tx/{account_id}` | Yes | Transaction history | Real-time |
| `notification/{account_id}` | Yes | Liquidation/warnings | Real-time |
| `pool_data/{account_id}` | Yes | Pool activity | On change |

---

## Testing WebSocket Connection

### Using wscat

```bash
# Install wscat
npm install -g wscat

# Connect
wscat -c wss://mainnet.zklighter.elliot.ai/stream

# Subscribe (paste after connection)
{"method":"subscribe","params":{"channel":"order_book/0"}}

# Unsubscribe
{"method":"unsubscribe","params":{"channel":"order_book/0"}}
```

---

### Using curl (HTTP Upgrade)

WebSocket connections cannot be established with curl, use dedicated tools.

---

## Troubleshooting

### Connection Refused

**Possible Causes**:
- Invalid URL
- Network firewall blocking WSS
- Server maintenance

**Solutions**:
- Verify URL is correct
- Check firewall settings
- Try testnet endpoint
- Wait and retry with backoff

---

### Subscription Not Working

**Possible Causes**:
- Invalid channel name
- Missing auth token for authenticated channel
- Subscription limit reached (100 per connection)

**Solutions**:
- Verify channel name format
- Include auth token for authenticated channels
- Reduce subscriptions or use multiple connections

---

### No Updates Received

**Possible Causes**:
- Market inactive
- No trades happening
- Connection issue

**Solutions**:
- Subscribe to active market (check market status)
- Monitor connection health (ping/pong)
- Resubscribe to channel

---

### Auth Token Expired

**Error**: `INVALID_AUTH`

**Solutions**:
- Generate new auth token
- Resubscribe with new token
- Implement token refresh before expiry

---

## Implementation Checklist

- [ ] Establish WebSocket connection
- [ ] Implement subscription mechanism
- [ ] Handle public channels (orderbook, trades, market stats)
- [ ] Handle authenticated channels (account updates)
- [ ] Implement auth token generation and refresh
- [ ] Implement reconnection with exponential backoff
- [ ] Implement ping/pong heartbeat
- [ ] Validate message sequences (nonce/offset)
- [ ] Handle errors gracefully
- [ ] Implement orderbook management
- [ ] Implement trade aggregation
- [ ] Monitor connection health
- [ ] Respect rate limits (200 msgs/min)
- [ ] Test with multiple markets
- [ ] Test auth token expiry handling

---

## References

- [Lighter WebSocket Reference](https://apidocs.lighter.xyz/docs/websocket-reference)
- [Lighter API Documentation](https://apidocs.lighter.xyz)
- [Get Started for Programmers](https://apidocs.lighter.xyz/docs/get-started-for-programmers-1)
