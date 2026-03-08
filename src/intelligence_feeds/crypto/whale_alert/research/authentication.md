# Whale Alert - Authentication

## Public Endpoints

- Public endpoints exist: No
- Require authentication: Yes (all endpoints require API key)
- Rate limits without auth: N/A (authentication always required)

## API Key

### Required For
- All endpoints: Yes
- Paid tier only: No (Developer API v1 has free tier, but still requires API key)
- Rate limit increase: Yes (different tiers have different limits)
- Specific endpoints: All endpoints

### How to Obtain
- Sign up: https://developer.whale-alert.io/
- API key management: https://developer.whale-alert.io/ (account dashboard)
- Free tier includes key: Yes (Developer API v1 - deprecated but functional)
- Paid tier signup: Required for Custom Alerts, Priority Alerts, Quantitative, and Historical APIs

### API Key Format
- Header: Not used
- Query param: `?api_key=YOUR_API_KEY` (primary method)
- Bearer token: Not used
- Other: For WebSocket, API key is in connection URL

**REST API Example:**
```bash
# Query parameter method (standard)
https://api.whale-alert.io/v1/status?api_key=YOUR_API_KEY

# For endpoints with other parameters
https://leviathan.whale-alert.io/bitcoin/transaction/abc123?api_key=YOUR_API_KEY

# Multiple parameters
https://api.whale-alert.io/v1/transactions?start=1234567890&min_value=10000&api_key=YOUR_API_KEY
```

**WebSocket Example:**
```javascript
// API key in connection URL
const ws = new WebSocket('wss://leviathan.whale-alert.io/ws?api_key=YOUR_API_KEY');
```

### Multiple Keys
- Multiple keys allowed: Not explicitly documented
- Rate limits per key: Yes (each API key has its own rate limit quota)
- Use cases for multiple keys: Separate projects, different rate limit pools

---

## OAuth (if applicable)

### OAuth 2.0
- Supported: No
- Grant types: N/A
- Scopes: N/A
- Token endpoint: N/A
- Authorization endpoint: N/A

**Whale Alert uses simple API key authentication only.**

---

## Signature/HMAC (if applicable - rare for data providers)

### Algorithm
- HMAC-SHA256: No
- HMAC-SHA512: No
- Other: No

**Whale Alert does NOT require signature/HMAC authentication.**

Simple API key in query parameter is sufficient for all endpoints.

### Components
Not applicable - no signature required

### Signature Construction
Not applicable - no signature required

### Headers
Not applicable - API key is passed as query parameter, not header

---

## Authentication Examples

### REST with API Key

#### Developer API v1 (Deprecated)

**Get Status:**
```bash
curl "https://api.whale-alert.io/v1/status?api_key=YOUR_API_KEY"
```

**Get Single Transaction:**
```bash
curl "https://api.whale-alert.io/v1/transaction/bitcoin/abc123def456?api_key=YOUR_API_KEY"
```

**Get Multiple Transactions:**
```bash
curl "https://api.whale-alert.io/v1/transactions?start=1640000000&min_value=1000000&api_key=YOUR_API_KEY"
```

#### Enterprise API v2

**Get Supported Blockchains:**
```bash
curl "https://leviathan.whale-alert.io/status?api_key=YOUR_API_KEY"
```

**Get Blockchain Status:**
```bash
curl "https://leviathan.whale-alert.io/ethereum/status?api_key=YOUR_API_KEY"
```

**Get Transaction:**
```bash
curl "https://leviathan.whale-alert.io/ethereum/transaction/0x1234567890abcdef?api_key=YOUR_API_KEY"
```

**Stream Transactions:**
```bash
curl "https://leviathan.whale-alert.io/ethereum/transactions?start_height=17000000&symbol=USDC&limit=100&api_key=YOUR_API_KEY"
```

**Get Address Transactions:**
```bash
curl "https://leviathan.whale-alert.io/ethereum/address/0xabc123def456/transactions?api_key=YOUR_API_KEY"
```

**Get Address Attribution:**
```bash
curl "https://leviathan.whale-alert.io/ethereum/address/0xabc123def456/owner_attributions?api_key=YOUR_API_KEY"
```

### WebSocket with API Key

#### JavaScript/Node.js
```javascript
const WebSocket = require('ws');

// API key in connection URL
const API_KEY = 'YOUR_API_KEY';
const ws = new WebSocket(`wss://leviathan.whale-alert.io/ws?api_key=${API_KEY}`);

ws.on('open', () => {
  console.log('Connected - authentication successful');

  // Subscribe to alerts
  ws.send(JSON.stringify({
    type: 'subscribe_alerts',
    min_value_usd: 1000000
  }));
});

ws.on('error', (error) => {
  console.error('Connection error (possibly invalid API key):', error);
});
```

#### Python
```python
import websocket
import json

API_KEY = 'YOUR_API_KEY'
ws_url = f'wss://leviathan.whale-alert.io/ws?api_key={API_KEY}'

def on_open(ws):
    print('Connected - authentication successful')
    subscription = {
        'type': 'subscribe_alerts',
        'blockchains': ['ethereum', 'bitcoin'],
        'min_value_usd': 1000000
    }
    ws.send(json.dumps(subscription))

def on_error(ws, error):
    print(f'Connection error (possibly invalid API key): {error}')

def on_message(ws, message):
    data = json.loads(message)
    print(f'Received: {data["type"]}')

ws = websocket.WebSocketApp(ws_url,
                            on_open=on_open,
                            on_message=on_message,
                            on_error=on_error)
ws.run_forever()
```

#### Go
```go
package main

import (
    "context"
    "fmt"
    "nhooyr.io/websocket"
)

func main() {
    apiKey := "YOUR_API_KEY"
    url := fmt.Sprintf("wss://leviathan.whale-alert.io/ws?api_key=%s", apiKey)

    ctx := context.Background()
    conn, _, err := websocket.Dial(ctx, url, nil)
    if err != nil {
        panic(fmt.Sprintf("Connection failed (possibly invalid API key): %v", err))
    }
    defer conn.Close(websocket.StatusNormalClosure, "")

    fmt.Println("Connected - authentication successful")

    // Subscribe
    subscription := map[string]interface{}{
        "type": "subscribe_alerts",
        "min_value_usd": 1000000,
    }
    // ... send subscription and handle messages
}
```

---

## Error Codes

### REST API Errors

| Code | Description | Resolution |
|------|-------------|------------|
| 200 | Success | N/A - request successful |
| 400 | Bad Request | Check request parameters and format |
| 401 | Unauthorized | Invalid API key - verify key is correct |
| 403 | Forbidden | API key valid but lacks permissions for this endpoint/tier |
| 404 | Not Found | Resource not found (invalid blockchain, hash, etc.) |
| 429 | Rate Limit Exceeded | Too many requests - wait and retry or upgrade tier |
| 500 | Internal Server Error | Server-side error - contact support if persists |
| 503 | Service Unavailable | Service temporarily down - retry with backoff |

### Error Response Format

```json
{
  "error_code": 401,
  "error_message": "Invalid API key",
  "details": "The provided API key is not valid or has been revoked"
}
```

OR

```json
{
  "error": "Rate limit exceeded",
  "limit": 60,
  "remaining": 0,
  "reset": 1640000000,
  "retry_after": 30
}
```

### WebSocket Errors

| Error | Description | Resolution |
|-------|-------------|------------|
| Connection refused | Invalid API key or network issue | Verify API key, check network connectivity |
| Connection closed (4xx) | Authentication or authorization failure | Check API key validity and tier permissions |
| No subscription confirmation | Invalid subscription parameters | Verify subscription message format and parameters |
| Rate limit | Too many connections or messages | Reduce connection count or upgrade tier |

### WebSocket Error Format (inferred)

```json
{
  "type": "error",
  "error_code": "INVALID_SUBSCRIPTION",
  "message": "min_value_usd must be at least 100000"
}
```

---

## Rate Limiting by Tier

### Developer API v1 (Deprecated)

**Free Tier:**
- Rate limit: 10 requests per minute
- Authentication: Required (API key)
- HTTP 429 returned when exceeded

**Personal Tier:**
- Rate limit: 60 requests per minute
- Authentication: Required (API key)
- HTTP 429 returned when exceeded

### Enterprise API v2 (Quantitative)

**Quantitative Tier ($699/month):**
- Rate limit: 1,000 requests per minute
- Authentication: Required (API key)
- Recommended for algorithmic trading and ML models

### WebSocket APIs

**Custom Alerts ($29.95/month):**
- Max connections: 2 per API key
- Max alerts: 100 per hour
- Authentication: Required (API key in URL)

**Priority Alerts ($1,299/month):**
- Max connections: 2 per API key
- Max alerts: 10,000 per hour (technically unlimited)
- Authentication: Required (API key in URL)
- Latency: Up to 1 minute faster than Custom Alerts

---

## Security Best Practices

1. **Never expose API key in client-side code:** API keys should be stored server-side
2. **Use environment variables:** Store API keys in `.env` files, never hardcode
3. **Rotate keys periodically:** Generate new API keys and revoke old ones
4. **Monitor usage:** Track API usage to detect unauthorized access
5. **Use HTTPS/WSS only:** All connections are encrypted (enforced by Whale Alert)
6. **Separate keys per environment:** Use different API keys for development, staging, production

### Environment Variable Example

```bash
# .env file
WHALE_ALERT_API_KEY=your_api_key_here
```

```javascript
// Node.js
require('dotenv').config();
const apiKey = process.env.WHALE_ALERT_API_KEY;
const url = `https://api.whale-alert.io/v1/status?api_key=${apiKey}`;
```

```python
# Python
import os
from dotenv import load_dotenv

load_dotenv()
api_key = os.getenv('WHALE_ALERT_API_KEY')
url = f'https://api.whale-alert.io/v1/status?api_key={api_key}'
```

```rust
// Rust
use std::env;

let api_key = env::var("WHALE_ALERT_API_KEY").expect("API key not set");
let url = format!("https://api.whale-alert.io/v1/status?api_key={}", api_key);
```

---

## Notes

1. **Simple Authentication:** Whale Alert uses straightforward API key authentication - no complex OAuth or HMAC signing
2. **Query Parameter Only:** API key is always passed as query parameter, never in headers
3. **WebSocket Auth:** Authentication happens at connection time via URL parameter
4. **No Session Management:** Each request is independently authenticated
5. **Tier-Based Access:** Different API keys have different rate limits and endpoint access based on subscription tier
6. **No Public Endpoints:** Unlike many APIs, Whale Alert requires authentication for ALL endpoints, even status checks
