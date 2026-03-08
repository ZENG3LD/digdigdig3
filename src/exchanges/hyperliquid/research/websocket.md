# HyperLiquid WebSocket API

Complete guide to HyperLiquid's WebSocket API for real-time market data and account updates.

---

## Connection Details

### WebSocket URLs

- **Mainnet**: `wss://api.hyperliquid.xyz/ws`
- **Testnet**: `wss://api.hyperliquid-testnet.xyz/ws`

### Connection Example
```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};

let (ws_stream, _) = connect_async("wss://api.hyperliquid.xyz/ws").await?;
```

---

## Message Format

### Subscribe to Channel
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "trades",
    "coin": "BTC"
  }
}
```

### Unsubscribe from Channel
```json
{
  "method": "unsubscribe",
  "subscription": {
    "type": "trades",
    "coin": "BTC"
  }
}
```

**Note**: Unsubscribe subscription object must match subscribe exactly.

---

## Subscription Types

### 1. All Mids (All Symbol Prices)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "allMids",
    "dex": ""
  }
}
```

**Channel**: `"allMids"`

**Message Format**:
```json
{
  "channel": "allMids",
  "data": {
    "mids": {
      "BTC": "50123.45",
      "ETH": "2500.67",
      "SOL": "100.23",
      "PURR/USDC": "0.000123"
    }
  }
}
```

**Use Case**: Real-time ticker prices for all assets
**Update Frequency**: On every price change

---

### 2. Trades (Recent Trades Stream)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "trades",
    "coin": "BTC"
  }
}
```

**Channel**: `"trades"`

**Message Format**:
```json
{
  "channel": "trades",
  "data": [
    {
      "coin": "BTC",
      "side": "B",
      "px": "50123.45",
      "sz": "0.5",
      "hash": "0x...",
      "time": 1704067200123,
      "tid": 123456789,
      "fee": "0.25"
    }
  ]
}
```

**Field Descriptions**:
- `side`: "B" = buy, "A" = sell
- `px`: Trade price
- `sz`: Trade size
- `hash`: Transaction hash
- `time`: Timestamp (ms)
- `tid`: Trade ID
- `fee`: Fee paid

**Use Case**: Real-time trade feed, volume tracking
**Update Frequency**: On every trade

---

### 3. L2 Order Book

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "l2Book",
    "coin": "BTC",
    "nSigFigs": null,
    "mantissa": null
  }
}
```

**Optional Aggregation**:
- `nSigFigs`: 2-5 for significant figures
- `mantissa`: 1, 2, or 5 for rounding

**Channel**: `"l2Book"`

**Message Format**:
```json
{
  "channel": "l2Book",
  "data": {
    "coin": "BTC",
    "time": 1704067200000,
    "levels": [
      [
        {"px": "50123.5", "sz": "1.234", "n": 3},
        {"px": "50123.0", "sz": "2.567", "n": 5}
      ],
      [
        {"px": "50124.0", "sz": "0.567", "n": 1},
        {"px": "50124.5", "sz": "3.456", "n": 7}
      ]
    ]
  }
}
```

**Structure**:
- `levels[0]`: Bids (best to worst)
- `levels[1]`: Asks (best to worst)
- Up to 20 levels per side

**Use Case**: Order book updates, liquidity tracking
**Update Frequency**: On every order book change

---

### 4. BBO (Best Bid/Offer)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "bbo",
    "coin": "BTC"
  }
}
```

**Channel**: `"bbo"`

**Message Format**:
```json
{
  "channel": "bbo",
  "data": {
    "coin": "BTC",
    "bid": "50123.0",
    "ask": "50124.0",
    "time": 1704067200000
  }
}
```

**Use Case**: Top-of-book tracking, spread monitoring
**Update Frequency**: On block changes (not every tick)

**Note**: Updates less frequently than l2Book but lower bandwidth.

---

### 5. Candles (OHLCV)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "candle",
    "coin": "BTC",
    "interval": "15m"
  }
}
```

**Supported Intervals**: `"1m"`, `"3m"`, `"5m"`, `"15m"`, `"30m"`, `"1h"`, `"2h"`, `"4h"`, `"8h"`, `"12h"`, `"1d"`, `"3d"`, `"1w"`, `"1M"`

**Channel**: `"candle"`

**Message Format**:
```json
{
  "channel": "candle",
  "data": [
    {
      "t": 1704067200000,
      "T": 1704067259999,
      "s": "BTC",
      "i": "15m",
      "o": "50100.0",
      "c": "50200.0",
      "h": "50250.0",
      "l": "50050.0",
      "v": "123.456",
      "n": 1234
    }
  ]
}
```

**Use Case**: Chart updates, technical analysis
**Update Frequency**: On candle close and in-progress updates

---

### 6. Notification (User Notifications)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "notification",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"notification"`

**Message Format**:
```json
{
  "channel": "notification",
  "data": {
    "notification": "Your order was filled at 50100.0"
  }
}
```

**Use Case**: User-facing notifications
**Update Frequency**: Event-driven

---

### 7. Open Orders

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "openOrders",
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "dex": ""
  }
}
```

**Channel**: `"openOrders"`

**Message Format**:
```json
{
  "channel": "openOrders",
  "data": [
    {
      "coin": "BTC",
      "limitPx": "50000.0",
      "oid": 123456789,
      "side": "B",
      "sz": "0.1",
      "timestamp": 1704067200000,
      "origSz": "0.1",
      "cloid": null
    }
  ]
}
```

**Use Case**: Real-time order status updates
**Update Frequency**: On order state changes

---

### 8. Order Updates

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "orderUpdates",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"orderUpdates"`

**Message Format**:
```json
{
  "channel": "orderUpdates",
  "data": [
    {
      "order": {
        "coin": "BTC",
        "side": "B",
        "limitPx": "50000.0",
        "sz": "0.05",
        "oid": 123456789,
        "timestamp": 1704067200000,
        "origSz": "0.1"
      },
      "status": "filled",
      "statusTimestamp": 1704067201000
    }
  ]
}
```

**Status Values**: `"open"`, `"filled"`, `"canceled"`, `"triggered"`, `"rejected"`, `"marginCanceled"`

**Use Case**: Track order lifecycle
**Update Frequency**: On every order status change

---

### 9. User Fills (Trade Executions)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "userFills",
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "aggregateByTime": false
  }
}
```

**Channel**: `"userFills"`

**Message Format**:
```json
{
  "channel": "userFills",
  "data": {
    "isSnapshot": false,
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "fills": [
      {
        "coin": "BTC",
        "px": "50100.0",
        "sz": "0.1",
        "side": "B",
        "time": 1704067200123,
        "startPosition": "0.0",
        "dir": "Open Long",
        "closedPnl": "0.0",
        "hash": "0x...",
        "oid": 123456789,
        "crossed": true,
        "fee": "2.505",
        "feeToken": "USDC",
        "tid": 987654321,
        "builderFee": "0.0"
      }
    ]
  }
}
```

**Important**: First message has `isSnapshot: true`, subsequent have `isSnapshot: false`

**Use Case**: Real-time fill notifications, PnL tracking
**Update Frequency**: On every fill

---

### 10. User Events (Comprehensive Account Events)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "userEvents",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"userEvents"`

**Message Format**:
```json
{
  "channel": "userEvents",
  "data": {
    "fills": [...],
    "funding": {...},
    "liquidation": {...},
    "nonUserCancel": [...]
  }
}
```

**Event Types**:
- **fills**: Trade executions
- **funding**: Funding payments
- **liquidation**: Liquidation events
- **nonUserCancel**: System-triggered cancels

**Use Case**: Complete account event stream
**Update Frequency**: Event-driven

---

### 11. User Fundings (Funding Payments)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "userFundings",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"userFundings"`

**Message Format**:
```json
{
  "channel": "userFundings",
  "data": {
    "isSnapshot": false,
    "user": "0x...",
    "fundings": [
      {
        "time": 1704067200000,
        "coin": "BTC",
        "fundingRate": "0.00001234",
        "szi": "1.5",
        "fundingPayment": "-0.925"
      }
    ]
  }
}
```

**Use Case**: Track funding payments over time
**Update Frequency**: Every hour (funding payment)

---

### 12. User Non-Funding Ledger Updates

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "userNonFundingLedgerUpdates",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"userNonFundingLedgerUpdates"`

**Event Types**:
- Deposits
- Withdrawals
- Internal transfers
- Liquidations

**Use Case**: Track account balance changes
**Update Frequency**: Event-driven

---

### 13. Clearinghouse State (Account Summary)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "clearinghouseState",
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "dex": ""
  }
}
```

**Channel**: `"clearinghouseState"`

**Message Format**: Same as REST API clearinghouseState response

**Use Case**: Real-time account summary updates
**Update Frequency**: On account state changes

---

### 14. WebData3 (Aggregate Frontend Data)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "webData3",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"webData3"`

**Message Format**: Aggregate data used by HyperLiquid frontend

**Use Case**: Complete user dashboard data
**Update Frequency**: On any relevant update

**Note**: Combines multiple data sources into single stream

---

### 15. Active Asset Context

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "activeAssetCtx",
    "coin": "BTC"
  }
}
```

**Channel**: `"activeAssetCtx"`

**Message Format**:
```json
{
  "channel": "activeAssetCtx",
  "data": {
    "coin": "BTC",
    "dayNtlVlm": "1234567890.5",
    "funding": "0.000012345",
    "openInterest": "987654.321",
    "markPx": "50123.45",
    "midPx": "50123.5",
    "impactPxs": ["50120.0", "50127.0"],
    "premium": "0.5",
    "oraclePx": "50122.95"
  }
}
```

**Use Case**: Real-time asset statistics
**Update Frequency**: On changes to asset context

---

### 16. Active Asset Data (User-Specific Asset Data)

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "activeAssetData",
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "coin": "BTC"
  }
}
```

**Channel**: `"activeAssetData"`

**Note**: Perps only (not supported for spot)

**Use Case**: User-specific asset limits and leverage
**Update Frequency**: On relevant changes

---

### 17. TWAP States

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "twapStates",
    "user": "0x1234567890abcdef1234567890abcdef12345678",
    "dex": ""
  }
}
```

**Channel**: `"twapStates"`

**Use Case**: Monitor TWAP order execution
**Update Frequency**: On TWAP state changes

---

### 18. User TWAP Slice Fills

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "userTwapSliceFills",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"userTwapSliceFills"`

**Use Case**: Track individual TWAP slice executions
**Update Frequency**: On each TWAP slice fill

---

### 19. User TWAP History

**Subscribe**:
```json
{
  "method": "subscribe",
  "subscription": {
    "type": "userTwapHistory",
    "user": "0x1234567890abcdef1234567890abcdef12345678"
  }
}
```

**Channel**: `"userTwapHistory"`

**Use Case**: Historical TWAP orders
**Update Frequency**: On TWAP completion

---

## WebSocket POST Requests

### Send HTTP-Style Requests via WebSocket

**Format**:
```json
{
  "method": "post",
  "id": 12345,
  "request": {
    "type": "info" | "action",
    "payload": {
      // Same as REST API payload
    }
  }
}
```

**Response**:
```json
{
  "channel": "post",
  "data": {
    "id": 12345,
    "response": {
      "type": "info" | "action" | "error",
      "payload": {
        // Response data
      }
    }
  }
}
```

### Example: Query Order Book via WS
```json
{
  "method": "post",
  "id": 1,
  "request": {
    "type": "info",
    "payload": {
      "type": "l2Book",
      "coin": "BTC"
    }
  }
}
```

**Note**: Cannot send `explorer` requests via WebSocket

---

## Rate Limits

### WebSocket Connection Limits

| Limit Type | Value | Scope |
|------------|-------|-------|
| Max connections | 100 | Per IP |
| Max subscriptions | 1000 | Total across all connections |
| Max unique users | 10 | For user-specific subscriptions |
| Message rate | 2000/min | Across all connections |
| Inflight POST messages | 100 | Simultaneous |

### Managing Limits

**Connection Strategy**:
```
Connection 1: Market data (trades, l2Book, candles)
Connection 2: User data (userFills, orderUpdates, clearinghouseState)
Connection 3: Additional symbols or users
```

**Subscription Counting**:
```javascript
// Each subscription counts separately
{ type: "trades", coin: "BTC" }      // 1
{ type: "trades", coin: "ETH" }      // 2
{ type: "l2Book", coin: "BTC" }      // 3
{ type: "userFills", user: "0x..." } // 4
```

**User Limit**:
```javascript
// Max 10 unique user addresses
{ type: "userFills", user: "0xUser1..." }  // User 1
{ type: "openOrders", user: "0xUser1..." } // Still user 1
{ type: "userFills", user: "0xUser2..." }  // User 2
// ... up to 10 unique users
```

---

## Reliability and Reconnection

### Disconnect Handling

**Critical**: "All automated users should handle disconnects from the server side and gracefully reconnect. Disconnection from API servers may happen periodically and without announcement."

### Reconnection Strategy

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};

async fn connect_with_retry(url: &str, max_retries: u32) -> Result<WebSocketStream> {
    let mut retries = 0;

    loop {
        match connect_async(url).await {
            Ok((ws_stream, _)) => return Ok(ws_stream),
            Err(e) if retries < max_retries => {
                retries += 1;
                let delay = Duration::from_secs(2_u64.pow(retries.min(5)));
                warn!("WebSocket connection failed, retrying in {:?}", delay);
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e.into()),
        }
    }
}

async fn maintain_connection() -> Result<()> {
    loop {
        let mut ws = connect_with_retry("wss://api.hyperliquid.xyz/ws", 10).await?;

        // Resubscribe to all channels
        for subscription in get_active_subscriptions() {
            ws.send(Message::Text(subscription)).await?;
        }

        // Handle messages
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(text)) => handle_message(text).await?,
                Ok(Message::Close(_)) => {
                    warn!("WebSocket closed, reconnecting...");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    ws.send(Message::Pong(data)).await?;
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    break;
                }
                _ => {}
            }
        }

        // Connection lost, retry
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}
```

---

## Heartbeat and Ping/Pong

### WebSocket Ping/Pong
Handle ping frames automatically:
```rust
Ok(Message::Ping(data)) => {
    ws.send(Message::Pong(data)).await?;
}
```

### Application-Level Heartbeat
Implement timeout detection:
```rust
use tokio::time::{timeout, Duration};

let heartbeat_interval = Duration::from_secs(30);
let mut last_message = Instant::now();

loop {
    match timeout(heartbeat_interval, ws.next()).await {
        Ok(Some(Ok(msg))) => {
            last_message = Instant::now();
            handle_message(msg).await?;
        }
        Ok(Some(Err(e))) => {
            error!("WebSocket error: {}", e);
            break; // Reconnect
        }
        Ok(None) => {
            warn!("WebSocket stream ended");
            break; // Reconnect
        }
        Err(_) => {
            if last_message.elapsed() > Duration::from_secs(60) {
                warn!("No messages for 60s, reconnecting...");
                break;
            }
        }
    }
}
```

---

## Snapshot and Incremental Updates

### Snapshot Messages
Some subscriptions send initial snapshot:
```json
{
  "channel": "userFills",
  "data": {
    "isSnapshot": true,
    "user": "0x...",
    "fills": [...]  // All recent fills
  }
}
```

### Incremental Updates
Subsequent messages are incremental:
```json
{
  "channel": "userFills",
  "data": {
    "isSnapshot": false,
    "user": "0x...",
    "fills": [...]  // New fills only
  }
}
```

### Handling Pattern
```rust
match data.is_snapshot {
    true => {
        // Replace entire state
        state.fills = data.fills;
    }
    false => {
        // Append to state
        state.fills.extend(data.fills);
    }
}
```

---

## Subscription Response

### Acknowledgment
```json
{
  "channel": "subscriptionResponse",
  "data": {
    "method": "subscribe",
    "subscription": {
      "type": "trades",
      "coin": "BTC"
    }
  }
}
```

**Purpose**: Confirms subscription successful

### Error Response
```json
{
  "channel": "error",
  "data": {
    "error": "Invalid subscription type"
  }
}
```

---

## Example Implementation

### Complete WebSocket Client

```rust
use tokio_tungstenite::{connect_async, tungstenite::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

pub struct HyperLiquidWs {
    url: String,
    subscriptions: Vec<serde_json::Value>,
}

impl HyperLiquidWs {
    pub fn new(mainnet: bool) -> Self {
        let url = if mainnet {
            "wss://api.hyperliquid.xyz/ws"
        } else {
            "wss://api.hyperliquid-testnet.xyz/ws"
        };

        Self {
            url: url.to_string(),
            subscriptions: Vec::new(),
        }
    }

    pub async fn subscribe_trades(&mut self, coin: &str) -> Result<()> {
        let sub = json!({
            "method": "subscribe",
            "subscription": {
                "type": "trades",
                "coin": coin
            }
        });

        self.subscriptions.push(sub);
        Ok(())
    }

    pub async fn subscribe_l2_book(&mut self, coin: &str) -> Result<()> {
        let sub = json!({
            "method": "subscribe",
            "subscription": {
                "type": "l2Book",
                "coin": coin
            }
        });

        self.subscriptions.push(sub);
        Ok(())
    }

    pub async fn subscribe_user_fills(&mut self, user: &str) -> Result<()> {
        let sub = json!({
            "method": "subscribe",
            "subscription": {
                "type": "userFills",
                "user": user,
                "aggregateByTime": false
            }
        });

        self.subscriptions.push(sub);
        Ok(())
    }

    pub async fn connect<F>(&self, mut handler: F) -> Result<()>
    where
        F: FnMut(String) -> Result<()>,
    {
        loop {
            match self.connect_once(&mut handler).await {
                Ok(_) => {
                    warn!("WebSocket connection ended, reconnecting...");
                }
                Err(e) => {
                    error!("WebSocket error: {}, reconnecting...", e);
                }
            }

            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    async fn connect_once<F>(&self, handler: &mut F) -> Result<()>
    where
        F: FnMut(String) -> Result<()>,
    {
        let (mut ws, _) = connect_async(&self.url).await?;

        // Send all subscriptions
        for sub in &self.subscriptions {
            let msg = serde_json::to_string(sub)?;
            ws.send(Message::Text(msg)).await?;
        }

        // Handle messages
        while let Some(msg) = ws.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    handler(text)?;
                }
                Ok(Message::Close(frame)) => {
                    warn!("WebSocket closed: {:?}", frame);
                    break;
                }
                Ok(Message::Ping(data)) => {
                    ws.send(Message::Pong(data)).await?;
                }
                Err(e) => {
                    return Err(e.into());
                }
                _ => {}
            }
        }

        Ok(())
    }
}

// Usage
#[tokio::main]
async fn main() -> Result<()> {
    let mut ws = HyperLiquidWs::new(true);

    ws.subscribe_trades("BTC").await?;
    ws.subscribe_l2_book("BTC").await?;

    ws.connect(|msg| {
        let value: serde_json::Value = serde_json::from_str(&msg)?;

        match value["channel"].as_str() {
            Some("trades") => {
                println!("Trade: {:?}", value["data"]);
            }
            Some("l2Book") => {
                println!("Order book update");
            }
            Some("subscriptionResponse") => {
                println!("Subscription confirmed");
            }
            _ => {}
        }

        Ok(())
    }).await
}
```

---

## Best Practices

### 1. Reconnection Logic
- Implement exponential backoff
- Resubscribe on reconnect
- Track last message time
- Detect stale connections

### 2. Subscription Management
```rust
// Track active subscriptions
let mut subscriptions = HashSet::new();

// Before subscribing
if !subscriptions.contains(&sub_key) {
    send_subscribe(sub).await?;
    subscriptions.insert(sub_key);
}

// On reconnect
for sub in &subscriptions {
    send_subscribe(sub).await?;
}
```

### 3. Message Handling
- Parse channel field first
- Route to appropriate handler
- Handle errors gracefully
- Log unexpected messages

### 4. Resource Management
- Close unused subscriptions
- Limit concurrent connections
- Monitor subscription count
- Use connection pooling for multiple symbols

### 5. Data Integrity
- Handle snapshots correctly
- Track sequence numbers if available
- Detect and recover from data gaps
- Verify against REST API periodically

---

## Troubleshooting

### Connection Issues
- Check URL (mainnet vs testnet)
- Verify network connectivity
- Check WebSocket library version
- Review firewall/proxy settings

### Subscription Failures
- Verify subscription format
- Check symbol validity
- Ensure user address exists (for user subscriptions)
- Monitor subscription limit (1000 max)

### Missing Updates
- Check if subscription confirmed
- Verify connection still alive
- Look for error messages
- Test with simple subscription first

### Performance Issues
- Reduce subscription count
- Use BBO instead of L2 book
- Filter messages client-side
- Increase buffer sizes

---

## Summary

### Key Points
1. **Two types of data**: Market data (public) and user data (private)
2. **Automatic reconnection**: Required for production
3. **Snapshot + incremental**: Handle both message types
4. **Rate limits**: 100 connections, 1000 subscriptions, 2000 msg/min
5. **Reliable**: Handle disconnects, implement heartbeat

### Subscription Recommendations

| Use Case | Subscriptions |
|----------|---------------|
| **Trading Bot** | orderUpdates, userFills, l2Book, trades |
| **Market Maker** | l2Book, bbo, userFills, openOrders |
| **Price Monitor** | allMids or trades |
| **Portfolio Tracker** | clearinghouseState, userFundings, webData3 |
| **Chart Application** | candles, trades |

### Implementation Checklist
- [ ] WebSocket connection with auto-reconnect
- [ ] Exponential backoff on errors
- [ ] Subscription management (add/remove/resubscribe)
- [ ] Ping/pong handling
- [ ] Heartbeat timeout detection
- [ ] Snapshot vs incremental handling
- [ ] Channel-based message routing
- [ ] Error logging and monitoring
- [ ] Subscription limit tracking
- [ ] Graceful shutdown
