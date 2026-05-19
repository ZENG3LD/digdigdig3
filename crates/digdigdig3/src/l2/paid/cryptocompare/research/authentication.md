# CryptoCompare - Authentication

## Public Endpoints

- Public endpoints exist: Yes
- Require authentication: No (but recommended)
- Rate limits without auth: Very low (10-20 req/min, not officially documented)
- Rate limits with free API key: 50/sec, 1000/min, 150,000/hr

## API Key

### Required For
- All endpoints: No (but strongly recommended)
- Paid tier only: No (free tier provides API keys)
- Rate limit increase: Yes (major benefit)
- Specific endpoints:
  - News API (`/data/v2/news/`) - Required
  - Social stats (`/data/social/coin/*`) - Required
  - Rate limit stats (`/stats/rate/*`) - Required
  - WebSocket streaming - Required (for stable connections)
  - Orderbook data - Required (paid tier)

### How to Obtain
- Sign up: https://www.cryptocompare.com/
- API key management: https://www.cryptocompare.com/cryptopian/api-keys
- Free tier includes key: Yes
- Process:
  1. Create account at CryptoCompare.com
  2. Navigate to API Keys section (https://www.cryptocompare.com/cryptopian/api-keys)
  3. Click "Create API Key"
  4. Set permissions: "Read All Price Streaming and Polling Endpoints" (minimum)
  5. Copy API key (shown once)

### API Key Format
- Header: `Authorization: Apikey {YOUR_API_KEY}`
- OR Query param: `?api_key={YOUR_API_KEY}` (most common)
- Bearer token: Not supported
- Custom header: `Apikey: {YOUR_API_KEY}` (alternative)

**Preferred method:** Query parameter
```
https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD&api_key=YOUR_API_KEY
```

**Alternative method:** Authorization header
```bash
curl -H "Authorization: Apikey YOUR_API_KEY" \
  https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD
```

### Multiple Keys
- Multiple keys allowed: Yes
- Rate limits per key: Yes (each key has independent limits)
- Use cases for multiple keys:
  - Separate development/production keys
  - Different applications
  - Isolate rate limits per service
  - Team members with different access levels

### API Key Permissions
When creating an API key, you can set permissions:
- **Read All Price Streaming and Polling Endpoints** (default, required)
- **Write to Forum** (optional, for community features)
- Additional permissions may be available for paid tiers

## OAuth (if applicable)

### OAuth 2.0
- Supported: No
- CryptoCompare uses simple API key authentication
- OAuth not implemented for data API

## Signature/HMAC (if applicable - rare for data providers)

### Not Required
CryptoCompare does NOT use HMAC signatures for authentication. Simple API key is sufficient.

**Reason:** Data provider (read-only), not an exchange. No account management or trading operations require additional security.

## Authentication Examples

### REST with API Key (Query Parameter)
```bash
# Price endpoint
curl "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD&api_key=YOUR_API_KEY"

# Historical data
curl "https://min-api.cryptocompare.com/data/histoday?fsym=BTC&tsym=USD&limit=30&api_key=YOUR_API_KEY"

# News (requires API key)
curl "https://min-api.cryptocompare.com/data/v2/news/?api_key=YOUR_API_KEY"
```

### REST with API Key (Header)
```bash
# Using Authorization header
curl -H "Authorization: Apikey YOUR_API_KEY" \
  "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"

# Alternative: Custom Apikey header
curl -H "Apikey: YOUR_API_KEY" \
  "https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD"
```

### JavaScript Example
```javascript
const apiKey = 'YOUR_API_KEY';

// Using query parameter
fetch(`https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD&api_key=${apiKey}`)
  .then(response => response.json())
  .then(data => console.log(data));

// Using header
fetch('https://min-api.cryptocompare.com/data/price?fsym=BTC&tsyms=USD', {
  headers: {
    'Authorization': `Apikey ${apiKey}`
  }
})
  .then(response => response.json())
  .then(data => console.log(data));
```

### Python Example
```python
import requests

API_KEY = 'YOUR_API_KEY'

# Using query parameter
url = 'https://min-api.cryptocompare.com/data/price'
params = {
    'fsym': 'BTC',
    'tsyms': 'USD',
    'api_key': API_KEY
}
response = requests.get(url, params=params)
print(response.json())

# Using header
headers = {
    'Authorization': f'Apikey {API_KEY}'
}
response = requests.get(url, params={'fsym': 'BTC', 'tsyms': 'USD'}, headers=headers)
print(response.json())
```

### WebSocket with API Key
```javascript
const WebSocket = require('ws');

const apiKey = 'YOUR_API_KEY';
const ws = new WebSocket(`wss://streamer.cryptocompare.com/v2?api_key=${apiKey}`);

ws.on('open', () => {
  console.log('Connected');

  const subMessage = {
    action: 'SubAdd',
    subs: ['5~BTC~USD']
  };

  ws.send(JSON.stringify(subMessage));
});

ws.on('message', (data) => {
  console.log('Received:', JSON.parse(data));
});
```

### Python WebSocket
```python
import websocket
import json

API_KEY = 'YOUR_API_KEY'

def on_open(ws):
    print('Connected')
    sub_message = {
        'action': 'SubAdd',
        'subs': ['5~BTC~USD']
    }
    ws.send(json.dumps(sub_message))

def on_message(ws, message):
    print('Received:', json.loads(message))

ws = websocket.WebSocketApp(
    f'wss://streamer.cryptocompare.com/v2?api_key={API_KEY}',
    on_open=on_open,
    on_message=on_message
)

ws.run_forever()
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Unauthorized - Invalid API key | Check API key is correct and active |
| 403 | Forbidden - Insufficient permissions | Upgrade tier or check API key permissions |
| 429 | Rate limit exceeded | Wait for rate limit reset or upgrade tier |
| 500 | Internal server error | Retry with exponential backoff |
| 1001 | Invalid parameter | Check request parameters |
| 2001 | Subscription error (WebSocket) | Check subscription format |

### Error Response Format (REST)
```json
{
  "Response": "Error",
  "Message": "You are over your rate limit please upgrade your account!",
  "HasWarning": false,
  "Type": 99,
  "RateLimit": {
    "calls_made": {
      "second": 51,
      "minute": 1005,
      "hour": 150100
    },
    "calls_left": {
      "second": 0,
      "minute": 0,
      "hour": 0
    }
  },
  "Data": {}
}
```

**Error Types:**
- `Type: 1` - General error
- `Type: 2` - Invalid parameter
- `Type: 99` - Rate limit exceeded
- `Type: 500` - Internal server error

### Error Response Format (WebSocket)
WebSocket errors are not well documented. Connection may close on authentication failure.

## Rate Limit Headers

CryptoCompare does NOT return standard rate limit headers (`X-RateLimit-*`).

To check rate limits:
- Use `/stats/rate/limit` endpoint (requires API key)
- Monitor error responses (Type 99 indicates rate limit)

### Rate Limit Check Endpoint
```bash
curl "https://min-api.cryptocompare.com/stats/rate/limit?api_key=YOUR_API_KEY"
```

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "Data": {
    "calls_made": {
      "second": 10,
      "minute": 150,
      "hour": 5000
    },
    "calls_left": {
      "second": 40,
      "minute": 850,
      "hour": 145000
    }
  }
}
```

## Best Practices

### API Key Security
1. **Never expose API key in client-side code** (browsers)
2. **Use environment variables** for API key storage
3. **Rotate keys periodically** (especially if exposed)
4. **Create separate keys** for different environments (dev, prod)
5. **Set minimal permissions** needed for each key

### Rate Limit Management
1. **Always use API key** (even for public endpoints)
2. **Cache responses** when appropriate (price data has 10s cache)
3. **Implement exponential backoff** on errors
4. **Monitor usage** via `/stats/rate/limit` endpoint
5. **Upgrade tier** if consistently hitting limits

### Error Handling
```javascript
async function fetchPrice(fsym, tsym) {
  const apiKey = process.env.CRYPTOCOMPARE_API_KEY;
  const url = `https://min-api.cryptocompare.com/data/price?fsym=${fsym}&tsyms=${tsym}&api_key=${apiKey}`;

  try {
    const response = await fetch(url);
    const data = await response.json();

    if (data.Response === 'Error') {
      if (data.Type === 99) {
        console.error('Rate limit exceeded:', data.RateLimit);
        // Wait and retry
        await sleep(60000); // Wait 1 minute
        return fetchPrice(fsym, tsym); // Retry
      } else {
        throw new Error(data.Message);
      }
    }

    return data;
  } catch (error) {
    console.error('Fetch error:', error);
    throw error;
  }
}
```

## Migration from Old Authentication

### Historical Note
CryptoCompare was acquired by CoinDesk. Some endpoints may redirect to `developers.coindesk.com`.

### API Key Compatibility
- Old CryptoCompare API keys continue to work
- New keys created through CoinDesk/CCData portal
- Both authentication methods supported

### URL Changes
- Old: `min-api.cryptocompare.com` (still works)
- New: May redirect to CoinDesk endpoints for some features
- WebSocket: `wss://streamer.cryptocompare.com/v2` (unchanged)

## Attribution Requirements (Free Tier)

Free tier users MUST provide attribution:

### Required Attribution
- Display "Powered by CryptoCompare" or similar
- Link to CryptoCompare website
- Follow branding guidelines: https://www.cryptocompare.com/branding/

### Example Attribution
```html
<div>
  Data provided by <a href="https://www.cryptocompare.com/">CryptoCompare</a>
</div>
```

### Consequences of Non-Compliance
- API key may be suspended
- Legal action for commercial use without license
- Upgrade to paid tier removes attribution requirement

## Summary

| Authentication Method | REST | WebSocket |
|-----------------------|------|-----------|
| No auth | Limited | Not recommended |
| Query parameter (`?api_key=`) | Yes | Yes (in URL) |
| Authorization header | Yes | No |
| OAuth | No | No |
| HMAC/Signature | No | No |

**Recommended:** Use API key via query parameter for both REST and WebSocket.
