# Dukascopy - WebSocket Documentation

## Availability: Third-Party Only (Unofficial)

**Official Status**: Dukascopy does NOT provide an official WebSocket API.

**Alternatives**:
1. **JForex SDK**: Real-time data via Java listeners (not WebSocket)
2. **FIX API**: Real-time market data via FIX protocol (not WebSocket)
3. **Third-Party Wrapper**: Community-built WebSocket interface (see below)

---

## Third-Party WebSocket Implementation

**Source**: https://github.com/ismailfer/dukascopy-api-websocket
**Status**: Unofficial, community-maintained
**Technology**: Spring Boot wrapper around JForex SDK
**License**: MIT (check repository)

---

## Connection

### URLs
- Public streams: ws://localhost:7081/ticker
- Private streams: N/A (authentication via config file, single connection)
- Regional: N/A (local deployment only)

### Connection Process
1. Configure Dukascopy credentials in application.properties
2. Start Spring Boot application
3. Connect WebSocket client to ws://localhost:7081/ticker
4. Subscribe by sending URL query parameters or using default feed
5. Receive real-time market data updates

### Query Parameters

| Parameter | Type | Required | Values | Description |
|-----------|------|----------|--------|-------------|
| topOfBook | boolean | No | true, false | true=2-level quotes, false=10-level order book |
| instIDs | string | No | Comma-separated | EURUSD,GBPUSD,USDJPY (custom instruments) |

**Default Behavior** (no params):
- Top-of-book quotes (2-level)
- Instruments: EURUSD, EURJPY, USDJPY

**Examples**:
```
ws://localhost:7081/ticker
ws://localhost:7081/ticker?topOfBook=true
ws://localhost:7081/ticker?topOfBook=false&instIDs=EURUSD,GBPUSD
```

---

## ALL Available Channels/Topics

**Note**: This is a simplified wrapper, not a multi-channel system.

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Query Parameter |
|---------------|------|-------------|-------|-------|------------------|----------------|
| ticker (top-of-book) | Public | Best bid/ask quotes | Config | Yes | Real-time (~100-500ms) | topOfBook=true |
| ticker (order book) | Public | 10-level market depth | Config | Yes | Real-time (~100-500ms) | topOfBook=false |

**No subscription messages** - configured via URL parameters at connection time.

---

## Message Formats

### Top-of-Book Update (topOfBook=true)

```json
{
  "symbol": "EURUSD",
  "bidQty": 1500000.0,
  "bid": 1.12345,
  "ask": 1.12347,
  "askQty": 1200000.0,
  "last": 1.12346,
  "spread": 0.00002,
  "spreadBps": 0.2,
  "updateTime": 1234567890123,
  "updateNumber": 12345,
  "depthLevels": 2,
  "live": true
}
```

**Field Descriptions**:

| Field | Type | Description |
|-------|------|-------------|
| symbol | string | Instrument identifier (e.g., EURUSD) |
| bidQty | number | Size available at best bid (base currency) |
| bid | number | Best bid price |
| ask | number | Best ask price |
| askQty | number | Size available at best ask (base currency) |
| last | number | Last trade price (mid-point estimate) |
| spread | number | Spread (ask - bid) |
| spreadBps | number | Spread in basis points |
| updateTime | number | Unix timestamp in milliseconds |
| updateNumber | number | Sequential update counter |
| depthLevels | number | Number of price levels (2 for top-of-book) |
| live | boolean | True if live data, false if historical/delayed |

### Order Book Update (topOfBook=false)

```json
{
  "symbol": "EURUSD",
  "bidQty": 1500000.0,
  "bid": 1.12345,
  "ask": 1.12347,
  "askQty": 1200000.0,
  "last": 1.12346,
  "spread": 0.00002,
  "spreadBps": 0.2,
  "updateTime": 1234567890123,
  "updateNumber": 12345,
  "depthLevels": 10,
  "live": true,
  "bids": [
    {
      "quantity": 1500000.0,
      "price": 1.12345
    },
    {
      "quantity": 800000.0,
      "price": 1.12344
    },
    {
      "quantity": 1200000.0,
      "price": 1.12343
    }
    // ... up to 10 levels
  ],
  "asks": [
    {
      "quantity": 1200000.0,
      "price": 1.12347
    },
    {
      "quantity": 900000.0,
      "price": 1.12348
    },
    {
      "quantity": 1100000.0,
      "price": 1.12349
    }
    // ... up to 10 levels
  ]
}
```

**Additional Fields**:

| Field | Type | Description |
|-------|------|-------------|
| bids | array | Array of bid levels (up to 10), ordered by price descending |
| asks | array | Array of ask levels (up to 10), ordered by price ascending |

**Price Level Object**:

| Field | Type | Description |
|-------|------|-------------|
| quantity | number | Available volume at this price level |
| price | number | Price level |

---

## Subscription Format

### No Explicit Subscription Messages

This WebSocket implementation uses **URL-based subscription** rather than message-based subscription.

**Configuration happens at connection time** via query parameters.

**No subscribe/unsubscribe messages** supported.

**To change subscription**:
1. Disconnect current WebSocket
2. Reconnect with new URL parameters

---

## Heartbeat / Ping-Pong

### Standard WebSocket Ping/Pong

**Who initiates?**
- Server → Client ping: Yes (standard WebSocket frames)
- Client → Server ping: Yes (optional, recommended)

**Message Format**
- Binary ping/pong frames: Yes (standard WebSocket protocol)
- Text messages: No
- JSON messages: No

**Timing**
- Ping interval: Depends on WebSocket client/server implementation
- Timeout: Typically 60-120 seconds (WebSocket default)
- Client should respond to ping: Automatically handled by WebSocket libraries

**Note**: Uses standard WebSocket ping/pong frames, not custom application-level heartbeat.

---

## Connection Limits

**Third-Party Wrapper Limits**:
- Max connections per IP: Not specified (single-user deployment)
- Max connections per API key: N/A
- Max subscriptions per connection: Limited by URL parameters
- Message rate limit: Driven by JForex SDK tick rate
- Auto-disconnect after: N/A (persistent connection)

**Underlying JForex SDK Limits**:
- Real-time data feed: Based on account type
- Instruments: Based on subscription

---

## Authentication (Third-Party Wrapper)

### Method
- Configuration file: application.properties (server-side)
- No per-connection authentication
- WebSocket connections inherit server's JForex session

### Configuration Format
```properties
dukascopy.username=YOUR_DEMO_USERNAME
dukascopy.password=YOUR_PASSWORD
dukascopy.demo=true
```

### Auth Success/Failure
- Authentication happens when Spring Boot app starts
- WebSocket connections fail if JForex session not established
- No explicit auth messages on WebSocket

---

## Data Flow

### Connection Lifecycle

1. **Server Startup**
   - Spring Boot application starts
   - Connects to Dukascopy via JForex SDK
   - Establishes market data session

2. **WebSocket Connection**
   - Client connects to ws://localhost:7081/ticker
   - Server parses URL parameters
   - Server subscribes to requested instruments via JForex SDK

3. **Data Streaming**
   - JForex SDK receives tick updates
   - Server transforms to JSON format
   - Server broadcasts to connected WebSocket clients

4. **Disconnection**
   - Client disconnects
   - Server unsubscribes from instruments (if no other clients)

---

## Error Handling

### Connection Errors

**JForex Session Failed**:
```
WebSocket connection refused (server not connected to Dukascopy)
```

**Invalid Instrument**:
```json
{
  "error": "Unknown instrument: INVALID"
}
```
(Actual error format depends on implementation)

### Data Errors

**No Data Available**:
- Field values may be 0 or null if no market data received

**Market Closed**:
- live: false
- Updates may stop or slow significantly

---

## Performance Characteristics

### Update Frequency
- **Active market hours**: 100-500ms per update (varies by liquidity)
- **Low liquidity**: Updates only when price changes
- **Market closed**: No updates

### Latency
- **JForex SDK → WebSocket**: ~10-50ms (local network)
- **Total latency**: Depends on Dukascopy → JForex SDK latency

### Data Completeness
- **Tick aggregation**: May aggregate rapid ticks (not pure tick-by-tick)
- **Order book**: Top 10 levels (not full depth)

---

## Comparison with Official Methods

| Feature | Third-Party WS | JForex SDK | FIX API |
|---------|---------------|------------|---------|
| Protocol | WebSocket | Java Listeners | FIX 4.4 |
| Real-time data | Yes | Yes | Yes |
| Order book depth | 10 levels | 10 levels | Configurable |
| Tick-by-tick | Aggregated | Full | Full |
| Authentication | Config file | SDK login | FIX logon |
| Latency | Higher | Low | Lowest |
| Ease of use | High | Medium | Low |
| Official support | No | Yes | Yes |
| Language | Any (WebSocket) | Java | Any (FIX) |

---

## Limitations & Caveats

1. **Unofficial Implementation**: Not supported by Dukascopy
2. **Local Deployment Only**: Runs on localhost, requires local Java server
3. **Single Session**: Limited by JForex SDK session limits
4. **No Historical Replay**: Real-time data only
5. **Dependency on JForex SDK**: Updates may break wrapper
6. **No Multi-Channel**: Single data stream per connection
7. **No Snapshots**: No initial snapshot, only updates
8. **Order Book Snapshots**: Full snapshot sent on each update (not deltas)

---

## Alternative: Direct JForex SDK Subscription

For official real-time data access, use JForex SDK directly:

```java
// IFeedListener interface
public void onFeedData(IFeedDescriptor feedDescriptor, ITimedData feedData) {
    ITick tick = (ITick) feedData;
    System.out.println("Tick: " + tick.getBid() + " / " + tick.getAsk());
}

// Subscribe
context.subscribeToFeed(new FeedDescriptor(Instrument.EURUSD), feedListener);
```

**Advantages**:
- Official support
- Lower latency
- More granular control
- Full tick data

**Disadvantages**:
- Requires Java
- More complex setup
- Not WebSocket-based

---

## Summary

- **Official WebSocket**: None
- **Third-Party WebSocket**: Available but unofficial
- **Best for**: Quick prototyping, language-agnostic integration
- **Not recommended for**: Production, high-frequency trading, official support
- **Better alternatives**:
  - JForex SDK (official, Java)
  - FIX API (official, professional)
  - Binary downloads (historical, free)
