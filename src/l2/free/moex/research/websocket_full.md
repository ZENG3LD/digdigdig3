# MOEX - WebSocket Documentation

## Availability: Yes

WebSocket support available via STOMP protocol for real-time market data.

## Connection

### URLs
- Public/Private streams: wss://iss.moex.com/infocx/v3/websocket
- Regional: None (single endpoint)
- Protocol: STOMP (Simple Text Oriented Messaging Protocol)

### Connection Process
1. Connect to WebSocket URL: `wss://iss.moex.com/infocx/v3/websocket`
2. Send STOMP CONNECT frame with authentication headers
3. Receive CONNECTED frame with session info and available data
4. Subscribe to desired topics
5. Receive real-time updates

### Authentication for Private Data
- **Free users**: No authentication required for delayed data
- **Paid subscribers**: Use MOEX Passport account credentials for real-time data
- **Authentication method**: STOMP CONNECT frame with credentials

### STOMP CONNECT Frame Example
```
CONNECT
accept-version:1.2
host:iss.moex.com
login:your_username
passcode:your_password

^@
```

### CONNECTED Response
Upon successful connection, server responds with CONNECTED frame containing:
- Session ID
- Server version
- Available data subscriptions
- Heart-beat configuration

```
CONNECTED
version:1.2
session:session-id-here
heart-beat:10000,10000

^@
```

## Available Channels/Topics

**Note**: Exact channel list and subscription format not publicly documented in detail. Based on ISS REST API capabilities, WebSocket likely provides:

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Notes |
|---------------|------|-------------|-------|-------|------------------|-------|
| Market data streams | Public | Price updates | No | Delayed | Real-time (15min delay for free) | Ticker data |
| Trade streams | Public | Trade updates | No | Delayed | Real-time (15min delay) | Individual trades |
| Quote streams | Public | Best bid/ask | No | Delayed | Real-time (15min delay) | Top of book |
| Orderbook streams | Private | L2 orderbook | Yes | No | Real-time | Requires subscription |
| Index streams | Public | Index values | No | Delayed | 1-second intervals | IMOEX, RTSI, etc. |
| Statistics streams | Public | Market stats | No | Delayed | Per-session updates | Turnover, volumes |

**Real-time data** (requires authentication and paid subscription):
- Live market data (no delay)
- Full orderbook depth (10x10 for equities/bonds/FX, 5x5 for derivatives)
- Streaming trades
- Real-time quotes

**Delayed data** (free, no authentication):
- 15-minute delayed market data
- Delayed trades
- Delayed quotes
- End-of-day bulletins

## Subscription Format

### Subscribe via STOMP
```
SUBSCRIBE
id:sub-1
destination:/topic/market.stock.shares.SBER
ack:auto

^@
```

### Unsubscribe
```
UNSUBSCRIBE
id:sub-1

^@
```

### Subscription Confirmation
Server sends MESSAGE frame to confirm subscription:
```
MESSAGE
destination:/topic/market.stock.shares.SBER
subscription:sub-1
message-id:msg-001

{json payload}
^@
```

## Message Formats

**Note**: Exact message formats are not publicly documented. Based on REST API responses, expected structures:

### Market Data Update (Ticker)
```json
{
  "type": "marketdata",
  "engine": "stock",
  "market": "shares",
  "board": "TQBR",
  "secid": "SBER",
  "last": 306.75,
  "bid": 306.74,
  "ask": 306.76,
  "volume": 4800000,
  "value": 1472000000,
  "change": -0.13,
  "changepercent": -0.04,
  "high": 307.35,
  "low": 305.12,
  "numtrades": 43199,
  "timestamp": "2026-01-26T19:00:01",
  "systime": "2026-01-26T19:00:01.123"
}
```

### Trade Update
```json
{
  "type": "trade",
  "tradeno": 1234567890,
  "tradetime": "19:00:01",
  "boardid": "TQBR",
  "secid": "SBER",
  "price": 306.75,
  "quantity": 100,
  "value": 30675.00,
  "buysell": "B",
  "period": "N",
  "tradingsession": "N",
  "systime": "2026-01-26T19:00:01.123",
  "tradedate": "2026-01-26"
}
```

### Quote/Best Bid-Ask Update
```json
{
  "type": "quote",
  "secid": "SBER",
  "boardid": "TQBR",
  "bid": 306.74,
  "ask": 306.76,
  "biddepth": 1000,
  "askdepth": 1500,
  "spread": 0.02,
  "timestamp": "2026-01-26T19:00:01"
}
```

### Orderbook Snapshot (Subscriber-only)
```json
{
  "type": "orderbook",
  "secid": "SBER",
  "boardid": "TQBR",
  "timestamp": "2026-01-26T19:00:01",
  "bids": [
    {"price": 306.74, "quantity": 1000},
    {"price": 306.73, "quantity": 1500},
    {"price": 306.72, "quantity": 2000},
    {"price": 306.71, "quantity": 2500},
    {"price": 306.70, "quantity": 3000},
    {"price": 306.69, "quantity": 1000},
    {"price": 306.68, "quantity": 1200},
    {"price": 306.67, "quantity": 1400},
    {"price": 306.66, "quantity": 1600},
    {"price": 306.65, "quantity": 1800}
  ],
  "asks": [
    {"price": 306.76, "quantity": 1500},
    {"price": 306.77, "quantity": 2000},
    {"price": 306.78, "quantity": 2500},
    {"price": 306.79, "quantity": 3000},
    {"price": 306.80, "quantity": 3500},
    {"price": 306.81, "quantity": 1200},
    {"price": 306.82, "quantity": 1400},
    {"price": 306.83, "quantity": 1600},
    {"price": 306.84, "quantity": 1800},
    {"price": 306.85, "quantity": 2000}
  ]
}
```

### Index Update
```json
{
  "type": "index",
  "secid": "IMOEX",
  "currentvalue": 2850.45,
  "lastchange": 12.30,
  "lastchangeprc": 0.43,
  "timestamp": "2026-01-26T19:00:01"
}
```

## Heartbeat / Ping-Pong

**CRITICAL**: STOMP protocol heart-beat mechanism

### Who initiates?
- **Server → Client**: Yes (sends heart-beat)
- **Client → Server**: Yes (must respond)
- **Bidirectional**: Both parties send heart-beats

### Message Format
- **STOMP heart-beat**: Single newline character (`\n`) or EOL byte
- **Not JSON messages**: Uses STOMP protocol frames
- **Binary frames**: Not used (text-based STOMP protocol)

### Timing
Configured during CONNECT handshake via `heart-beat` header:

**Format**: `heart-beat:<client-send-ms>,<client-receive-ms>`

Example: `heart-beat:10000,10000`
- Client will send heart-beat every 10 seconds
- Client expects server heart-beat every 10 seconds
- If no heart-beat received within timeout, connection is dead

**Server response** in CONNECTED frame: `heart-beat:10000,10000`
- Server agrees to heart-beat intervals
- Negotiated to minimum of client/server values

### Example Heart-beat
```
Client → Server: \n  (every 10 seconds)
Server → Client: \n  (every 10 seconds)
```

**Timeout**: If expected heart-beat not received within configured interval + grace period, connection should be closed and reconnected.

### Implementation Notes
- Heart-beats are independent of normal STOMP frames
- Can be sent alongside regular messages
- Prevents idle connection timeout
- Critical for maintaining long-lived WebSocket connections

## Connection Limits

**Note**: Specific limits not publicly documented. Estimated based on typical exchange WebSocket practices:

- **Max connections per IP**: Unknown (likely 5-10 for free tier)
- **Max connections per API key**: Unknown (likely higher for paid subscribers)
- **Max subscriptions per connection**: Unknown (likely 100-500)
- **Message rate limit**: Unknown (server may throttle high-frequency subscriptions)
- **Auto-disconnect after**: Unknown (likely no time limit with active heart-beats)

**Free tier likely has**:
- Fewer simultaneous connections
- Limited subscriptions
- 15-minute data delay
- Potential rate limiting

**Paid tier likely has**:
- More connections
- More subscriptions
- Real-time data
- Higher message throughput

## Authentication (for Real-time/Private Channels)

### Method
Authentication via STOMP CONNECT frame credentials

### CONNECT Frame with Auth
```
CONNECT
accept-version:1.2
host:iss.moex.com
login:your_moex_username
passcode:your_moex_password
heart-beat:10000,10000

^@
```

**For OAuth/API key users** (WebAPI integration):
```
CONNECT
accept-version:1.2
host:iss.moex.com
Authorization:Bearer your_access_token
heart-beat:10000,10000

^@
```

### Auth Success
```
CONNECTED
version:1.2
session:abc123xyz456
user-name:your_username
heart-beat:10000,10000
server:MOEX-ISS/v3

^@
```

### Auth Failure
```
ERROR
message:Authentication failed
content-type:text/plain

Invalid credentials
^@
```

Connection will be closed by server after ERROR frame.

### Access Levels
- **No authentication**: Delayed data only (15-minute delay)
- **Authenticated free user**: Delayed data with user tracking
- **Authenticated paid subscriber**: Real-time data + full orderbook + private data

## Error Handling

### ERROR Frame
```
ERROR
message:Subscription limit exceeded
content-type:text/plain

Maximum 100 subscriptions per connection
^@
```

Common error scenarios:
- Authentication failure
- Invalid topic/destination
- Subscription limit exceeded
- Rate limit exceeded
- Connection timeout

## Data Delay

### Free Tier (No Auth or Free Account)
- **Market data**: 15-minute delay
- **Trades**: 15-minute delay
- **Quotes**: 15-minute delay
- **Orderbook**: Not available
- **End-of-day data**: No delay (available after market close)

### Paid Tier (Subscription Required)
- **Market data**: Real-time
- **Trades**: Real-time
- **Quotes**: Real-time
- **Orderbook**: Real-time (10x10 for equities/bonds/FX, 5x5 for derivatives)
- **Historical data**: Full access

## Alternative: Full Order Book Product

For institutional users requiring complete order book reconstruction:

**Product**: Full Order Book
- **Format**: Zipped files with all MOEX Market Data messages
- **Content**: Complete message stream for order book reconstruction
- **Use case**: Replay market events, backtesting, high-frequency analysis
- **Access**: Requires separate subscription and contract

## WebSocket vs REST API

**Use WebSocket for**:
- Real-time price monitoring
- Live trade feeds
- Continuous quote updates
- Order book depth monitoring
- High-frequency data needs

**Use REST API for**:
- Historical data queries
- Bulk data downloads
- One-time data requests
- Candle/OHLC data
- Reference data (securities, boards, etc.)
- Corporate actions, financials, ratings

## Developer Resources

- **STOMP Protocol Spec**: https://stomp.github.io/
- **MOEX ISS Reference**: https://iss.moex.com/iss/reference/
- **Support Email**: help@moex.com
- **Support Phone**: +7 (495) 733-9507

## Implementation Notes

**IMPORTANT**: WebSocket documentation is limited in public sources. Key points:

1. **Protocol**: STOMP over WebSocket (not raw WebSocket with JSON)
2. **Libraries needed**: STOMP client library (not just WebSocket client)
3. **Authentication**: Required for real-time data
4. **Data delay**: 15 minutes for free tier
5. **Heart-beats**: Mandatory for connection stability
6. **Subscription format**: Topic-based (destination paths)
7. **Message format**: Likely similar to REST API JSON responses
8. **Error handling**: STOMP ERROR frames

**Recommended STOMP libraries**:
- Rust: `stomp-rs` or implement STOMP protocol manually
- JavaScript: `@stomp/stompjs`
- Python: `stomp.py`

## Missing Public Documentation

The following information is **not publicly available** and requires:
- Paid subscription to access
- Contact with MOEX support
- Trial access request

**Unknown details**:
- Exact topic/destination naming conventions
- Complete message format specifications
- Precise connection limits
- Detailed error codes
- Subscription management best practices
- Reconnection policies
- Message sequencing and gap handling

**To obtain full WebSocket documentation**:
1. Contact MOEX support: help@moex.com
2. Request ISS WebSocket developer documentation
3. Apply for paid subscription trial
4. Request access to Full Order Book product specifications

## Subscription Process

To access real-time WebSocket data:

1. **Create MOEX Passport account**: https://passport.moex.com/
2. **Apply for data subscription**: Contact client support manager
3. **Receive credentials**: Username and password for authentication
4. **Connect via WebSocket**: Use credentials in STOMP CONNECT frame
5. **Subscribe to topics**: Real-time data streams become available

## Summary

MOEX WebSocket support via STOMP protocol provides real-time market data for paid subscribers and delayed data for free users. Limited public documentation requires contacting MOEX support for detailed integration specifications. REST API remains primary interface for historical and reference data.
