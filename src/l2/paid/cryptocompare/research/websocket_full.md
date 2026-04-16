# CryptoCompare - WebSocket Documentation

## Availability: Yes

## Connection

### URLs
- Public streams: wss://streamer.cryptocompare.com/v2?api_key={YOUR_API_KEY}
- Private streams: Same URL (no separate private stream)
- Regional: None (global endpoint)
- Version: v2 (current)

### Connection Process
1. Connect to WebSocket URL with API key in query parameter
2. Connection established (WebSocket handshake)
3. No explicit welcome message
4. Send subscription messages to start receiving data
5. Optional: Authenticate for enhanced features (if using private data)

### Connection URL Format
```
wss://streamer.cryptocompare.com/v2?api_key=YOUR_API_KEY_HERE
```

## ALL Available Channels/Topics

| Channel/Topic | Type | Description | Auth? | Free? | Update Frequency | Subscription ID Format |
|---------------|------|-------------|-------|-------|------------------|----------------------|
| 0 (TRADE) | Public | Individual trades | No | Yes | Real-time | 0~{EXCHANGE}~{FROM}~{TO} |
| 2 (CURRENT) | Public | Ticker updates | No | Yes | Real-time | 2~{EXCHANGE}~{FROM}~{TO} |
| 5 (CURRENTAGG) | Public | Aggregate ticker | No | Yes | Real-time | 5~{FROM}~{TO} |
| 8 (LOADCOMPLATE) | System | Initial load complete | No | Yes | Once | 8~LOAD_COMPLETE |
| 11 (COINPAIRS) | System | Coin pairs update | No | Yes | On change | 11~COIN~PAIRS |
| 16 (ORDERBOOK) | Public | L2 orderbook | Paid | No | Real-time | 16~{EXCHANGE}~{FROM}~{TO} |
| 17 (OHLC) | Public | Candlestick bars | No | Yes | Per interval | 17~{EXCHANGE}~{FROM}~{TO}~{INTERVAL} |
| 24 (VOLUME) | Public | Volume updates | No | Yes | Real-time | 24~{EXCHANGE}~{FROM}~{TO} |

### Channel Details

#### Channel 0 - TRADE
- Individual trade executions
- Real-time trade stream
- Available for all exchanges
- Format: `0~{EXCHANGE}~{FROM_SYMBOL}~{TO_SYMBOL}`
- Example: `0~Binance~BTC~USDT`

#### Channel 2 - CURRENT (Ticker)
- Current price and 24h stats for specific exchange
- Exchange-specific ticker
- Format: `2~{EXCHANGE}~{FROM_SYMBOL}~{TO_SYMBOL}`
- Example: `2~Coinbase~BTC~USD`

#### Channel 5 - CURRENTAGG (Aggregate Ticker)
- CCCAGG aggregate index ticker
- Volume-weighted across all exchanges
- Format: `5~{FROM_SYMBOL}~{TO_SYMBOL}`
- Example: `5~BTC~USD`
- No exchange parameter (aggregate)

#### Channel 16 - ORDERBOOK (Level 2)
- Orderbook snapshot and updates
- Bids and asks depth
- Paid tier only
- Format: `16~{EXCHANGE}~{FROM_SYMBOL}~{TO_SYMBOL}`
- Example: `16~Kraken~ETH~USD`

#### Channel 17 - OHLC (Candlesticks)
- OHLC bars in real-time
- Multiple intervals supported
- Format: `17~{EXCHANGE}~{FROM_SYMBOL}~{TO_SYMBOL}~{INTERVAL}`
- Intervals: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 12h, 1d
- Example: `17~Binance~BTC~USDT~1h`

#### Channel 24 - VOLUME
- Volume updates
- Total and by market
- Format: `24~{EXCHANGE}~{FROM_SYMBOL}~{TO_SYMBOL}`
- Example: `24~Binance~BTC~USDT`

## Subscription Format

### Subscribe Message
```json
{
  "action": "SubAdd",
  "subs": [
    "0~Binance~BTC~USDT",
    "2~Coinbase~ETH~USD",
    "5~BTC~USD"
  ]
}
```

### Unsubscribe Message
```json
{
  "action": "SubRemove",
  "subs": [
    "0~Binance~BTC~USDT"
  ]
}
```

### Multiple Subscriptions
You can subscribe to multiple channels in a single message by adding them to the `subs` array.

### Subscription Confirmation
No explicit confirmation message. Data starts flowing immediately if subscription is valid.

### Error Response
If subscription fails (invalid format, unauthorized), you may receive an error in the stream, but format is not well documented.

## Message Formats (for EVERY channel)

### Channel 0 - Trade Update
```json
{
  "TYPE": "0",
  "M": "0",
  "FSYM": "BTC",
  "TSYM": "USDT",
  "F": "0x1",
  "ID": "1234567890",
  "TS": 1706280000,
  "Q": 0.5,
  "P": 45000.50,
  "TOTAL": 22500.25,
  "RTS": 1706280001
}
```

**Fields:**
- `TYPE`: Message type ("0" for trade)
- `M`: Market (exchange) or "0" for CCCAGG
- `FSYM`: From symbol (BTC)
- `TSYM`: To symbol (USDT)
- `F`: Flags (binary flags for trade properties)
- `ID`: Trade ID
- `TS`: Trade timestamp (Unix seconds)
- `Q`: Quantity (amount traded)
- `P`: Price
- `TOTAL`: Total value (Q * P)
- `RTS`: Received timestamp

### Channel 2 - Ticker Update (CURRENT)
```json
{
  "TYPE": "2",
  "M": "Coinbase",
  "FSYM": "BTC",
  "TSYM": "USD",
  "FLAGS": "1",
  "PRICE": 45000.50,
  "BID": 45000.00,
  "OFFER": 45001.00,
  "LASTUPDATE": 1706280000,
  "LASTVOLUME": 0.5,
  "LASTVOLUMETO": 22500.25,
  "LASTTRADEID": "1234567890",
  "VOLUMEDAY": 1250.75,
  "VOLUMEDAYTO": 56281250.00,
  "VOLUME24HOUR": 1500.50,
  "VOLUME24HOURTO": 67500000.00,
  "OPENDAY": 44500.00,
  "HIGHDAY": 45500.00,
  "LOWDAY": 44000.00,
  "OPEN24HOUR": 44200.00,
  "HIGH24HOUR": 45500.00,
  "LOW24HOUR": 43800.00,
  "LASTMARKET": "Coinbase"
}
```

**Fields:**
- `TYPE`: Message type ("2")
- `M`: Market/exchange name
- `FSYM`: From symbol
- `TSYM`: To symbol
- `FLAGS`: Flags
- `PRICE`: Last price
- `BID`: Best bid price
- `OFFER`: Best offer/ask price
- `LASTUPDATE`: Last update timestamp
- `LASTVOLUME`: Last trade volume
- `LASTVOLUMETO`: Last trade value
- `LASTTRADEID`: Last trade ID
- `VOLUMEDAY`: Volume today (from 00:00 GMT)
- `VOLUMEDAYTO`: Volume today in quote currency
- `VOLUME24HOUR`: 24h volume
- `VOLUME24HOURTO`: 24h volume in quote currency
- `OPENDAY`: Open price today
- `HIGHDAY`: High price today
- `LOWDAY`: Low price today
- `OPEN24HOUR`: Open price 24h ago
- `HIGH24HOUR`: 24h high
- `LOW24HOUR`: 24h low
- `LASTMARKET`: Last market where trade occurred

### Channel 5 - Aggregate Ticker Update (CURRENTAGG)
```json
{
  "TYPE": "5",
  "MARKET": "CCCAGG",
  "FROMSYMBOL": "BTC",
  "TOSYMBOL": "USD",
  "FLAGS": "1",
  "PRICE": 45000.50,
  "LASTUPDATE": 1706280000,
  "LASTVOLUME": 0.5,
  "LASTVOLUMETO": 22500.25,
  "LASTTRADEID": "1234567890",
  "VOLUMEHOUR": 125.75,
  "VOLUMEHOURTO": 5628125.00,
  "VOLUMEDAY": 1250.75,
  "VOLUMEDAYTO": 56281250.00,
  "VOLUME24HOUR": 1500.50,
  "VOLUME24HOURTO": 67500000.00,
  "OPENHOUR": 44800.00,
  "HIGHHOUR": 45100.00,
  "LOWHOUR": 44700.00,
  "OPENDAY": 44500.00,
  "HIGHDAY": 45500.00,
  "LOWDAY": 44000.00,
  "OPEN24HOUR": 44200.00,
  "HIGH24HOUR": 45500.00,
  "LOW24HOUR": 43800.00,
  "LASTMARKET": "Binance",
  "TOPTIERVOLUME24HOUR": 1200.00,
  "TOPTIERVOLUME24HOURTO": 54000000.00
}
```

**Fields:** Similar to Channel 2, but aggregated across all exchanges
- Additional fields for top-tier volume (major exchanges only)

### Channel 16 - Orderbook Snapshot
```json
{
  "TYPE": "16",
  "M": "Kraken",
  "FSYM": "BTC",
  "TSYM": "USD",
  "BIDS": [
    {"P": 45000.00, "Q": 1.5},
    {"P": 44999.50, "Q": 2.0},
    {"P": 44999.00, "Q": 0.75}
  ],
  "ASKS": [
    {"P": 45001.00, "Q": 1.2},
    {"P": 45001.50, "Q": 1.8},
    {"P": 45002.00, "Q": 0.5}
  ],
  "TS": 1706280000
}
```

**Fields:**
- `TYPE`: Message type ("16")
- `M`: Market/exchange
- `FSYM`: From symbol
- `TSYM`: To symbol
- `BIDS`: Array of bid levels (price and quantity)
- `ASKS`: Array of ask levels (price and quantity)
- `TS`: Timestamp

### Channel 16 - Orderbook Delta/Update
```json
{
  "TYPE": "16~UPDATE",
  "M": "Kraken",
  "FSYM": "BTC",
  "TSYM": "USD",
  "BID_CHANGES": [
    {"P": 45000.00, "Q": 2.0}
  ],
  "ASK_CHANGES": [
    {"P": 45001.00, "Q": 0}
  ],
  "TS": 1706280001
}
```

**Fields:**
- Similar to snapshot, but only changed levels
- `Q: 0` means level removed

### Channel 17 - OHLC Update
```json
{
  "TYPE": "17",
  "M": "Binance",
  "FSYM": "BTC",
  "TSYM": "USDT",
  "TS": 1706280000,
  "OPEN": 45000.00,
  "HIGH": 45100.00,
  "LOW": 44950.00,
  "CLOSE": 45050.00,
  "VOLUME": 125.5,
  "VOLUMETO": 5650000.00,
  "INTERVAL": "1h"
}
```

**Fields:**
- `TYPE`: Message type ("17")
- `M`: Market/exchange
- `FSYM`: From symbol
- `TSYM`: To symbol
- `TS`: Candle start timestamp
- `OPEN`: Open price
- `HIGH`: High price
- `LOW`: Low price
- `CLOSE`: Close price
- `VOLUME`: Volume in base currency
- `VOLUMETO`: Volume in quote currency
- `INTERVAL`: Candle interval (1m, 1h, 1d, etc.)

### Channel 24 - Volume Update
```json
{
  "TYPE": "24",
  "M": "Binance",
  "FSYM": "BTC",
  "TSYM": "USDT",
  "VOLUME24HOUR": 1500.50,
  "VOLUME24HOURTO": 67500000.00,
  "TS": 1706280000
}
```

**Fields:**
- `TYPE`: Message type ("24")
- `M`: Market/exchange
- `FSYM`: From symbol
- `TSYM`: To symbol
- `VOLUME24HOUR`: 24h volume in base currency
- `VOLUME24HOURTO`: 24h volume in quote currency
- `TS`: Timestamp

## Heartbeat / Ping-Pong

### Who initiates?
- Server → Client ping: No (standard WebSocket ping frames may be used)
- Client → Server ping: Not required (but recommended for keeping connection alive)

### Message Format
- Binary ping/pong frames: Yes (standard WebSocket protocol)
- Text messages: No specific text-based ping/pong
- JSON messages: No heartbeat messages documented

### Timing
- Ping interval: Not specified (use WebSocket-level ping)
- Timeout: Connection may close after ~60 seconds of inactivity
- Client should send ping: Every 30-45 seconds recommended

### Example
Standard WebSocket ping/pong frames (binary, handled by WebSocket library):
```
Client → Server: PING frame (opcode 0x9)
Server → Client: PONG frame (opcode 0xA)
```

**Note:** Most WebSocket libraries handle this automatically. CryptoCompare doesn't document a custom heartbeat mechanism, so rely on standard WebSocket protocol.

## Connection Limits

- Max connections per IP: Not publicly documented (likely 5-10)
- Max connections per API key: Not publicly documented (varies by tier)
- Max subscriptions per connection: 300 (recommended limit)
- Message rate limit: No specific limit, but avoid spamming
- Auto-disconnect after: 24 hours (recommended to reconnect periodically)

**Note:** Paid tiers have higher limits. Contact CryptoCompare for enterprise limits.

## Authentication (for private channels)

### Method
- URL params: `wss://streamer.cryptocompare.com/v2?api_key=YOUR_API_KEY`
- Message after connect: Not required (authentication via URL)
- Other: API key in URL query parameter

### Auth Message Format
No separate authentication message. API key is validated on connection.

### Auth Success/Failure
- **Success:** Connection established, ready to receive subscriptions
- **Failure:** Connection refused or closed immediately

No explicit success/failure message. If connected, authentication succeeded.

### Private Channels
Currently, all documented channels are public. Private channels (if any) would use the same API key authentication.

### Enhanced Features with API Key
- Higher rate limits
- Access to paid features (orderbook, extended history)
- Better connection stability

## Error Handling

### Invalid Subscription
If you subscribe to invalid pair or exchange:
```json
{
  "TYPE": "500",
  "MESSAGE": "INVALID_SUB",
  "PARAMETER": "0~InvalidExchange~BTC~USD",
  "INFO": "Exchange not found or pair not available"
}
```

**Note:** Error format may vary. CryptoCompare doesn't document errors comprehensively.

### Connection Errors
- **401 Unauthorized:** Invalid API key (connection refused)
- **429 Rate Limit:** Too many connections from IP/API key
- **500 Server Error:** CryptoCompare server issue

## Reconnection Strategy

**Recommended:**
1. On disconnect, wait 1-5 seconds
2. Reconnect with exponential backoff (max 60 seconds)
3. Re-subscribe to all channels
4. Keep track of last message timestamp to detect stale connections

## Data Reliability

### Message Ordering
- Messages are generally in order, but not guaranteed
- Use timestamp fields for ordering
- Trades may arrive out of order during high volatility

### Missing Messages
- Network issues may cause missed messages
- Use REST API to backfill gaps
- Orderbook should be refreshed periodically

### Duplicates
- Rare, but possible
- Use message ID or timestamp to deduplicate

## WebSocket vs REST

**When to use WebSocket:**
- Real-time price updates
- Trade stream
- Live orderbook
- High-frequency updates

**When to use REST:**
- Historical data
- One-time queries
- Backfilling gaps
- Lower frequency updates

## Examples

### JavaScript Connection
```javascript
const WebSocket = require('ws');

const apiKey = 'YOUR_API_KEY';
const ws = new WebSocket(`wss://streamer.cryptocompare.com/v2?api_key=${apiKey}`);

ws.on('open', () => {
  console.log('Connected');

  // Subscribe to BTC/USD aggregate ticker
  const subMessage = {
    action: 'SubAdd',
    subs: ['5~BTC~USD', '0~Binance~BTC~USDT']
  };

  ws.send(JSON.stringify(subMessage));
});

ws.on('message', (data) => {
  const message = JSON.parse(data);
  console.log('Received:', message);
});

ws.on('error', (error) => {
  console.error('WebSocket error:', error);
});

ws.on('close', () => {
  console.log('Disconnected');
});
```

### Python Connection
```python
import websocket
import json

API_KEY = 'YOUR_API_KEY'
WS_URL = f'wss://streamer.cryptocompare.com/v2?api_key={API_KEY}'

def on_message(ws, message):
    data = json.loads(message)
    print('Received:', data)

def on_open(ws):
    print('Connected')
    sub_message = {
        'action': 'SubAdd',
        'subs': ['5~BTC~USD', '2~Coinbase~ETH~USD']
    }
    ws.send(json.dumps(sub_message))

def on_error(ws, error):
    print('Error:', error)

def on_close(ws, close_status_code, close_msg):
    print('Disconnected')

ws = websocket.WebSocketApp(
    WS_URL,
    on_open=on_open,
    on_message=on_message,
    on_error=on_error,
    on_close=on_close
)

ws.run_forever()
```

## Notes

- WebSocket URL has changed over time (older docs may show different URLs)
- Current URL: `wss://streamer.cryptocompare.com/v2`
- API key is required (even for public channels, to avoid severe rate limits)
- CryptoCompare WebSocket documentation is sparse; much knowledge comes from community
- CCCAGG (aggregate) data is CryptoCompare's proprietary index
- Orderbook (Channel 16) requires paid tier
- Not all exchanges support all channels (check availability)
