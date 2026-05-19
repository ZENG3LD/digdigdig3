# Twelvedata - WebSocket Documentation

## Availability: Yes

WebSocket streaming is available for Pro plan and above subscribers.

## Connection

### URLs
- Public streams: `wss://ws.twelvedata.com/v1/quotes/price?apikey=YOUR_API_KEY`
- Private streams: N/A (no separate private stream, authentication via API key)
- Regional: None (single global endpoint)

### Connection Process
1. Connect to WebSocket URL with API key in query parameter
2. Connection established immediately upon successful handshake
3. No explicit welcome message documented
4. Authentication is validated via API key in URL
5. Send subscription message to start receiving data

### Authentication
Authentication is performed via API key in the connection URL:
```
wss://ws.twelvedata.com/v1/quotes/price?apikey=your_api_key
```

Alternative: WebSocket playground handles authentication automatically for testing.

## ALL Available Channels/Topics

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Plan Required |
|---------------|------|-------------|-------|-------|------------------|---------------|
| price | Public | Real-time tick prices | Yes | No | Real-time (~170ms latency) | Pro+ |

**Note**: The documentation only explicitly describes a **price** event channel. Unlike some providers, Twelvedata WebSocket appears to focus on real-time price streaming rather than multiple channel types (orderbook, trades, etc.).

## Subscription Format

### Subscribe Message
```json
{
  "action": "subscribe",
  "params": {
    "symbols": "AAPL,TRP,QQQ,EUR/USD,USD/JPY,BTC/USD,ETH/BTC"
  }
}
```

**Key Points:**
- Multiple symbols can be subscribed in a single message (comma-separated)
- Symbol tickers work identically to REST API format
- Can subscribe to different markets/exchanges/asset types simultaneously
- Maximum 120 symbols per subscription (inferred from REST API batch limit)

### Unsubscribe Message
**Format not explicitly documented.** Based on event types mentioned:
```json
{
  "action": "unsubscribe",
  "params": {
    "symbols": "AAPL,BTC/USD"
  }
}
```

### Reset Message
**Format not explicitly documented.** Mentioned as clearing subscriptions:
```json
{
  "action": "reset"
}
```

### Subscription Confirmation
**Exact format not documented in search results.**

Two return event types mentioned:
1. **Subscribe-status events**: Return subscription details and symbol success/failure information
2. **Price events**: Actual market data

Expected format (not confirmed):
```json
{
  "event": "subscribe-status",
  "success": true,
  "symbols": ["AAPL", "BTC/USD"],
  "failed": []
}
```

## Message Formats

### Price Event (Real-time Data)
```json
{
  "event": "price",
  "symbol": "BTC/USD",
  "currency_base": "Bitcoin",
  "currency_quote": "US Dollar",
  "exchange": "Binance",
  "type": "Digital Currency",
  "timestamp": 1600595462,
  "price": 10964.8,
  "day_volume": 38279
}
```

**Field Descriptions:**
| Field | Type | Description |
|-------|------|-------------|
| event | string | Event type identifier ("price") |
| symbol | string | Ticker symbol as subscribed |
| currency_base | string | Base currency full name (for forex/crypto) |
| currency_quote | string | Quote currency full name (for forex/crypto) |
| exchange | string | Exchange name where data originates |
| type | string | Instrument type (e.g., "Digital Currency", "Common Stock") |
| timestamp | integer | Unix timestamp (seconds, not milliseconds) |
| price | float | Current/latest price |
| day_volume | integer | Daily trading volume (may be null if unavailable) |

**Notes:**
- `day_volume` field may be absent for some instruments where volume data is unavailable
- Stock messages may have different `currency_base`/`currency_quote` fields or null values
- Timestamp is UNIX format in **seconds** (not milliseconds)

### Subscribe-Status Event
**Exact format not documented.** Expected structure:
```json
{
  "event": "subscribe-status",
  "symbols": ["AAPL", "TSLA"],
  "status": "success",
  "message": "Successfully subscribed to 2 symbols"
}
```

### Error Event
**Format not explicitly documented.** Expected structure:
```json
{
  "event": "error",
  "code": 401,
  "message": "Invalid API key"
}
```

## Heartbeat / Ping-Pong

**CRITICAL:** Heartbeat mechanism is required to maintain connection.

### Who initiates?
- Server → Client ping: No (not mentioned)
- Client → Server ping: **Yes** (client must send heartbeat)

### Message Format
**Exact format not documented in search results.**

Expected structure based on "heartbeat events" mention:
```json
{
  "action": "heartbeat"
}
```

**Alternative possible formats:**
- Text message: `"heartbeat"`
- Binary ping/pong frames: Possibly supported (standard WebSocket ping/pong)

### Timing
- **Ping interval**: Recommended **every 10 seconds**
- Timeout: Not explicitly documented
- **Client must send heartbeat**: Every 10 seconds to ensure connection stays alive
- Consequence of missing heartbeat: Connection may be terminated (implied)

### Example
```
Client → Server (every 10 seconds): {"action": "heartbeat"}
Server → Client: (no explicit response documented, connection remains alive)
```

**Note**: Documentation states heartbeat events "are advised to be sent every 10 seconds to make sure that your connection stays alive" but exact response format is not provided.

## Connection Limits

### Connections
- **Max connections per API key**: 3 concurrent connections
- Max connections per IP: Not explicitly documented (likely same as per API key)
- Connection lifetime: Not documented (likely unlimited with proper heartbeat)

### Subscriptions
- Max subscriptions per connection: Not explicitly documented
- Max symbols total: Likely 120 (based on REST API batch limit)
- Symbol format: Same as REST API (stocks, forex, crypto all supported)

### Message Rate Limits
- Messages per second: Not explicitly documented
- Server throttling: Not documented
- Auto-disconnect on violation: Not documented

### Credits System
- **WebSocket credits**: Separate from REST API credits
- Credit consumption: Per connection/subscription (exact formula not documented)
- Free tier: **No WebSocket access** (Pro plan minimum required)
- Pro plan: 8 WebSocket credits included
- Ultra plan: 2,500+ WebSocket credits

### Connection Duration
- Max lifetime: Not documented (appears unlimited with heartbeat)
- Auto-reconnect needed: Not mentioned (likely not required if heartbeat maintained)
- Idle timeout: Not documented
- **Explicit disconnect**: Users should close connection via "Close connection" button (in playground) or WebSocket close method in code to avoid wasting credits

## Authentication (for WebSocket)

### Method
**URL parameter** authentication:
```
wss://ws.twelvedata.com/v1/quotes/price?apikey=YOUR_API_KEY
```

**Alternative methods not documented:**
- No message-based authentication after connection
- No signature/HMAC required
- No OAuth

### Auth Success/Failure
- **Success**: Connection established, subscription messages accepted
- **Failure**: Connection refused or error event (exact format not documented)

Expected error response:
```json
{
  "event": "error",
  "code": 401,
  "message": "Unauthorized: Invalid API key"
}
```

## Data Coverage

### Supported Asset Types
- **Stocks**: All supported exchanges (US + 90+ international)
- **Forex**: All 200+ currency pairs
- **Cryptocurrencies**: All 180+ exchanges
- **ETFs**: Supported (not explicitly confirmed for WebSocket)
- **Indices**: Supported (not explicitly confirmed for WebSocket)
- **Commodities**: Supported (not explicitly confirmed for WebSocket)

### Real-time Latency
- **Average latency**: ~170ms for all instruments
- **Comparison**: Significantly faster than REST API (which updates minutely)
- **Update frequency**: Real-time tick-by-tick

## WebSocket Playground

Twelvedata provides an official WebSocket playground for testing:
- Interactive testing environment
- Automatic authentication handling
- Visual subscription management
- "Close connection" button to explicitly disconnect

Location: Available in documentation at https://twelvedata.com/docs (WebSocket section)

## Best Practices

1. **Always send heartbeat every 10 seconds** to maintain connection
2. **Explicitly close idle connections** to avoid wasting WebSocket credits
3. **Subscribe to multiple symbols in one message** when possible
4. **Use same symbol format as REST API** for consistency
5. **Handle null values** in `day_volume` field defensively
6. **Limit to 3 concurrent connections** per API key
7. **Pro plan minimum** required for WebSocket access
8. **Monitor WebSocket credit consumption** via dashboard

## Limitations & Unknowns

### Not Documented (inferred or unknown):
1. **Exact unsubscribe message format** (inferred from event type mention)
2. **Reset message format** (inferred from event type mention)
3. **Heartbeat response format** (if any)
4. **Subscribe-status event structure** (concept mentioned, format not shown)
5. **Error event structure** (expected but not documented)
6. **Max symbols per subscription** (likely 120, not confirmed)
7. **Reconnection policy** (not mentioned)
8. **Order book depth** (not available via WebSocket per search results)
9. **Trade stream** (not available as separate channel per search results)
10. **Kline/candle updates** (not available as separate channel per search results)

### Unique Characteristics

Unlike crypto exchange WebSockets, Twelvedata WebSocket:
- **Single channel type** (price events only, no orderbook/trades/klines)
- **Credit-based consumption** (not just connection-based)
- **Multi-asset support** (stocks, forex, crypto in same connection)
- **No separate public/private streams** (authentication required for all)
- **Simplified message format** (focus on price streaming)

## Comparison: WebSocket vs REST

| Feature | WebSocket | REST API |
|---------|-----------|----------|
| Latency | ~170ms | Minutely updates |
| Real-time | Yes | No (polling required) |
| Update type | Push | Pull |
| Plan required | Pro+ | Basic (free tier exists) |
| Credits | WebSocket credits | API credits |
| Max connections | 3 concurrent | Rate limited |
| Use case | Real-time monitoring | Historical/batch queries |
| Complexity | Higher (connection management) | Lower (stateless) |

## Implementation Considerations

### Connection Management
```rust
// Pseudo-code for Rust implementation
// 1. Connect with API key
let ws_url = format!("wss://ws.twelvedata.com/v1/quotes/price?apikey={}", api_key);
let ws = connect(ws_url).await?;

// 2. Subscribe to symbols
ws.send(json!({
    "action": "subscribe",
    "params": {
        "symbols": "AAPL,BTC/USD,EUR/USD"
    }
})).await?;

// 3. Start heartbeat task
tokio::spawn(async move {
    loop {
        sleep(Duration::from_secs(10)).await;
        ws.send(json!({"action": "heartbeat"})).await?;
    }
});

// 4. Handle incoming messages
while let Some(msg) = ws.next().await {
    match msg {
        Message::Text(text) => {
            let event: Event = serde_json::from_str(&text)?;
            match event.event.as_str() {
                "price" => handle_price(event),
                "subscribe-status" => handle_subscription(event),
                "error" => handle_error(event),
                _ => log_unknown(event),
            }
        }
    }
}

// 5. Explicit cleanup
ws.close().await?;
```

## Missing Features (vs Crypto Exchanges)

Twelvedata WebSocket does **NOT** provide (based on documentation):
- [ ] Order book snapshots/deltas
- [ ] Individual trade streams
- [ ] Candlestick/kline real-time updates
- [ ] Liquidation events
- [ ] Funding rate updates
- [ ] Open interest updates
- [ ] Depth-20/50/100 order book
- [ ] Private account streams (balances, orders)

**Focus**: Price streaming only, suitable for real-time quote monitoring rather than trading execution or depth analysis.
