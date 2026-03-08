# Phase 1: Research Agent Prompt

## Agent Type
`research-agent`

## Variables
- `{EXCHANGE}` - Exchange name in lowercase (e.g., "bybit")
- `{DOCS_URL}` - Official documentation URL

---

## Prompt

```
Research {EXCHANGE} API for V5 connector implementation.

Documentation: {DOCS_URL}

Create folder: src/exchanges/{EXCHANGE}/research/

═══════════════════════════════════════════════════════════════════════════════
FILE 1: endpoints.md
═══════════════════════════════════════════════════════════════════════════════

Document ALL REST endpoints:

## Base URLs
- Spot production:
- Spot testnet:
- Futures production:
- Futures testnet:

## Market Data Endpoints (Public)
| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/v1/ticker | 24h ticker |
| ... | ... | ... |

## Account Endpoints (Private)
...

## Trading Endpoints (Private)
...

═══════════════════════════════════════════════════════════════════════════════
FILE 2: authentication.md
═══════════════════════════════════════════════════════════════════════════════

Document signature algorithm step-by-step:

1. Required headers:
   - Header name: value description

2. Signature string construction:
   - What components? (timestamp + method + path + body)
   - What order?
   - Any encoding of body?

3. HMAC algorithm:
   - SHA256 / SHA512 / other?
   - Key: API Secret

4. Signature encoding:
   - Base64 / Hex / other?

5. Timestamp format:
   - Milliseconds / Seconds?
   - Max clock skew allowed?

6. Example:
   - Request: GET /api/v1/account
   - Timestamp: 1234567890000
   - Signature string: "1234567890000GET/api/v1/account"
   - HMAC result: "xxx"
   - Final signature: "yyy"

═══════════════════════════════════════════════════════════════════════════════
FILE 3: response_formats.md
═══════════════════════════════════════════════════════════════════════════════

EXACT JSON examples from docs (not invented):

## Ticker Response
```json
{
  "field1": "value",  // description
  "field2": 123       // description
}
```

## Orderbook Response
...

## Klines Response (Candlesticks)
...

## Balance Response
...

## Order Response
...

CRITICAL: Copy exact field names. Note differences between Spot and Futures.

═══════════════════════════════════════════════════════════════════════════════
FILE 4: symbols.md
═══════════════════════════════════════════════════════════════════════════════

## Symbol Format

| Type | Format | Example |
|------|--------|---------|
| Spot | ??? | BTC-USDT / BTCUSDT / BTC_USDT |
| Futures | ??? | BTCUSDT / BTCUSDTM / BTC-USDT-SWAP |

## Conversion Rules
- Our internal: Symbol { base: "BTC", quote: "USDT" }
- To Spot: format_symbol_spot(symbol) -> "???"
- To Futures: format_symbol_futures(symbol) -> "???"

═══════════════════════════════════════════════════════════════════════════════
FILE 5: rate_limits.md
═══════════════════════════════════════════════════════════════════════════════

## REST API Rate Limits

### General Limits
- Requests per second/minute: ???
- Weight system: yes/no
- Per-IP or per-API-key: ???

### Public Endpoints
| Endpoint Type | Limit | Window |
|---------------|-------|--------|
| Market data | ??? | ??? |

### Private Endpoints
| Endpoint Type | Limit | Window |
|---------------|-------|--------|
| Account | ??? | ??? |
| Orders | ??? | ??? |

### Rate Limit Headers
Does API return headers? Which ones?
- X-RateLimit-Remaining: ???
- X-RateLimit-Reset: ???

### Rate Limit Error
- HTTP status code: 429 / other?
- Error code in response: ???
- Retry-After header: yes/no

## WebSocket Rate Limits

### Connection Limits
- Max connections per IP: ???
- Max subscriptions per connection: ???

### Message Limits
- Messages per second: ???

═══════════════════════════════════════════════════════════════════════════════
FILE 6: websocket.md
═══════════════════════════════════════════════════════════════════════════════

## Connection

### URLs
- Spot public: wss://...
- Spot private: wss://...
- Futures public: wss://...
- Futures private: wss://...

### Connection Process
1. Connect to URL
2. Any initial handshake required?
3. Any welcome message received?

## Authentication (Private Channels)

How to authenticate on WebSocket?
- Sign in URL params?
- Send auth message after connect?
- Auth message format?

## Subscription

### Subscribe Message Format
```json
{
  "op": "subscribe",
  "args": ["topic"]
}
```

### Unsubscribe Message Format
...

### Topics/Channels
| Topic | Format | Example |
|-------|--------|---------|
| Ticker | ??? | ??? |
| Orderbook | ??? | ??? |
| Trades | ??? | ??? |
| Klines | ??? | ??? |
| User Orders | ??? | ??? |
| User Balance | ??? | ??? |

## Message Formats

### Ticker Update
```json
{ ... }
```

### Orderbook Update
```json
{ ... }
```

### Trade Update
```json
{ ... }
```

## Heartbeat / Ping-Pong

CRITICAL: Document exactly!

### Who initiates?
- Server sends ping, client responds pong?
- Client sends ping, server responds pong?
- Both?

### Message format
- Binary ping/pong frames?
- Text messages ("ping"/"pong", "Ping"/"Pong")?
- JSON messages?

### Timing
- Ping interval: ??? seconds
- Timeout if no response: ??? seconds
- Different for Spot vs Futures?

### Compression
- Messages gzip compressed? (BingX does this!)
- Need to decompress before checking for ping?

### Example
Server: "ping" or {"op":"ping","ts":123}
Client: "pong" or {"op":"pong","ts":123}
```

---

## Exit Criteria
- All 6 research files created
- Each file has EXACT examples from official docs
- No guessed or invented data
