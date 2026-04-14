# Tiingo - Authentication

## Public Endpoints

- **Public endpoints exist**: No (all endpoints require API key)
- **Require authentication**: Yes (all endpoints require API token)
- **Rate limits without auth**: Not applicable (no access without API key)

---

## API Key

### Required For
- **All endpoints**: Yes (every REST and WebSocket endpoint requires API token)
- **Paid tier only**: No (free tier provides API token)
- **Rate limit increase**: Yes (paid tiers have higher rate limits)
- **Specific endpoints**: All endpoints require authentication

### How to Obtain
- **Sign up**: https://www.tiingo.com/ (free account registration)
- **API key management**: https://api.tiingo.com/account/api/token
  - Also: https://www.tiingo.com/account/api/token
- **Free tier includes key**: Yes (immediately upon registration)
- **Credit card required**: No (for free tier)

### API Key Format

Tiingo supports two authentication methods:

#### 1. Authorization Header (Recommended)

```
Authorization: Token YOUR_API_KEY_HERE
```

**Format:**
- Header name: `Authorization`
- Header value: `Token {your_api_key}`
- Note the space between "Token" and the key
- Case-sensitive: "Token" with capital T

**Example:**
```bash
curl -H "Authorization: Token abc123xyz456" \
  https://api.tiingo.com/tiingo/daily/AAPL
```

#### 2. Query Parameter (Alternative)

```
?token=YOUR_API_KEY_HERE
```

**Format:**
- Parameter name: `token` (lowercase)
- Parameter value: Your API key
- Append to URL query string

**Example:**
```bash
curl "https://api.tiingo.com/tiingo/daily/AAPL?token=abc123xyz456"
```

#### 3. WebSocket Authentication

For WebSocket connections, authentication is included in the subscribe message:

```json
{
  "eventName": "subscribe",
  "authorization": "YOUR_API_KEY_HERE",
  "eventData": {
    "thresholdLevel": 5
  }
}
```

**Format:**
- Field name: `authorization` (lowercase)
- Field value: Your API key (no "Token" prefix for WebSocket)

---

### Multiple Keys

- **Multiple keys allowed**: Not explicitly documented (likely account-dependent)
- **Rate limits per key**: Yes (each API key has its own rate limits)
- **Use cases for multiple keys**:
  - Separate production/development environments
  - Different applications with isolated rate limits
  - Team members with individual tracking
  - Upgrade specific keys to different tiers

---

## OAuth (if applicable)

### OAuth 2.0
- **Supported**: No
- Tiingo uses simple API token authentication only
- No OAuth 2.0, no grant types, no authorization flows

---

## Signature/HMAC (if applicable)

**NOT REQUIRED** - Tiingo uses simple API token authentication.

- **HMAC signature**: Not used
- **Timestamp**: Not required
- **Request signing**: Not required
- **Secret key**: Not used (only API token)

Tiingo's authentication is straightforward: just include your API token via header or query parameter. No cryptographic signatures or HMAC computation needed.

---

## Authentication Examples

### REST with Authorization Header (Recommended)

```bash
curl -H "Authorization: Token YOUR_API_KEY" \
  https://api.tiingo.com/tiingo/daily/AAPL/prices?startDate=2020-01-01&endDate=2020-12-31
```

### REST with Query Parameter

```bash
curl "https://api.tiingo.com/tiingo/daily/AAPL/prices?startDate=2020-01-01&endDate=2020-12-31&token=YOUR_API_KEY"
```

### REST with Content-Type Header (Python-style)

```python
import requests

url = "https://api.tiingo.com/tiingo/daily/AAPL/prices"
headers = {
    "Authorization": "Token YOUR_API_KEY",
    "Content-Type": "application/json"
}
params = {
    "startDate": "2020-01-01",
    "endDate": "2020-12-31"
}

response = requests.get(url, headers=headers, params=params)
data = response.json()
```

### REST with User-Agent (SDK Pattern)

Tiingo's official Python SDK adds a User-Agent header:

```python
headers = {
    "Authorization": "Token YOUR_API_KEY",
    "Content-Type": "application/json",
    "User-Agent": "tiingo-python-client/VERSION"
}
```

This is optional but recommended for identifying your client application.

### WebSocket with API Key

```python
import websocket
import json

def on_open(ws):
    subscribe_msg = {
        "eventName": "subscribe",
        "authorization": "YOUR_API_KEY",
        "eventData": {
            "thresholdLevel": 5
        }
    }
    ws.send(json.dumps(subscribe_msg))

def on_message(ws, message):
    data = json.loads(message)
    print(data)

ws = websocket.WebSocketApp(
    "wss://api.tiingo.com/iex",
    on_message=on_message,
    on_open=on_open
)
ws.run_forever()
```

### JavaScript/Node.js Example

```javascript
const axios = require('axios');

const apiKey = 'YOUR_API_KEY';
const url = 'https://api.tiingo.com/tiingo/daily/AAPL';

axios.get(url, {
  headers: {
    'Authorization': `Token ${apiKey}`,
    'Content-Type': 'application/json'
  }
})
.then(response => {
  console.log(response.data);
})
.catch(error => {
  console.error(error);
});
```

### R Example

```r
library(httr)

api_key <- "YOUR_API_KEY"
url <- "https://api.tiingo.com/tiingo/daily/AAPL"

response <- GET(
  url,
  add_headers(Authorization = paste("Token", api_key))
)

data <- content(response)
```

---

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 401 | Unauthorized / Invalid API key | Check API key is correct and active |
| 403 | Forbidden / Insufficient permissions | Upgrade tier or check endpoint access |
| 404 | Not Found / Invalid endpoint or ticker | Verify endpoint URL and ticker symbol |
| 429 | Too Many Requests / Rate limit exceeded | Wait for rate limit reset or upgrade tier |
| 500 | Internal Server Error | Retry request, contact support if persists |
| 502 | Bad Gateway | Temporary server issue, retry with backoff |
| 503 | Service Unavailable | Server maintenance or overload, retry later |

### 401 Unauthorized

**Common causes:**
- Missing API key
- Invalid API key
- Expired API key
- Incorrect authentication format

**Example error response:**
```json
{
  "detail": "Authentication credentials were not provided."
}
```

**Resolution:**
1. Verify API key at https://api.tiingo.com/account/api/token
2. Check authentication header format: `Authorization: Token YOUR_KEY`
3. Ensure no extra spaces or typos in API key
4. Generate new API key if needed

### 403 Forbidden

**Common causes:**
- Endpoint requires paid tier
- Historical data depth exceeds tier limit
- Feature not available in current tier

**Example error response:**
```json
{
  "detail": "You do not have permission to access this resource."
}
```

**Resolution:**
1. Check tier requirements on https://www.tiingo.com/about/pricing
2. Upgrade to appropriate tier if needed
3. Verify endpoint is available in your tier
4. Check if data range exceeds tier limits (e.g., >5 years fundamentals on free tier)

### 429 Rate Limit Exceeded

**Common causes:**
- Exceeded requests per minute (5/min on free tier)
- Exceeded daily request limit (500/day on free tier)
- Exceeded symbols per hour (50/hour on free tier)

**Example error response:**
```json
{
  "detail": "Request was throttled. Expected available in X seconds."
}
```

**Response headers:**
```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1234567890
Retry-After: 30
```

**Resolution:**
1. Implement rate limiting in client code
2. Use exponential backoff for retries
3. Cache responses to reduce API calls
4. Upgrade tier for higher rate limits (up to 1200/min)
5. Wait for `Retry-After` seconds before next request

---

## API Key Security Best Practices

### Do:
- Store API key in environment variables (`TIINGO_API_KEY`)
- Use secrets management systems (AWS Secrets Manager, HashiCorp Vault)
- Restrict API key access to authorized services/users
- Regenerate API key if compromised
- Use different API keys for development and production
- Monitor API usage for unusual activity

### Don't:
- Commit API keys to version control (add to .gitignore)
- Share API keys in public forums or documentation
- Embed API keys in client-side JavaScript (use backend proxy)
- Log API keys in application logs
- Send API keys over unencrypted connections (always use HTTPS)

### Environment Variable Setup

**Bash/Linux/Mac:**
```bash
export TIINGO_API_KEY="YOUR_API_KEY"
```

**Windows (CMD):**
```cmd
set TIINGO_API_KEY=YOUR_API_KEY
```

**Windows (PowerShell):**
```powershell
$env:TIINGO_API_KEY="YOUR_API_KEY"
```

**Python (.env file with python-dotenv):**
```python
# .env file
TIINGO_API_KEY=YOUR_API_KEY

# Python code
from dotenv import load_dotenv
import os

load_dotenv()
api_key = os.getenv("TIINGO_API_KEY")
```

---

## Rate Limit Management

### Check Current Limits

Rate limit information is returned in response headers:

```python
import requests

response = requests.get(
    "https://api.tiingo.com/tiingo/daily/AAPL",
    headers={"Authorization": "Token YOUR_API_KEY"}
)

print(f"Limit: {response.headers.get('X-RateLimit-Limit')}")
print(f"Remaining: {response.headers.get('X-RateLimit-Remaining')}")
print(f"Reset: {response.headers.get('X-RateLimit-Reset')}")
```

### Implement Rate Limiting

```python
import time
import requests

class TiingoClient:
    def __init__(self, api_key, requests_per_minute=5):
        self.api_key = api_key
        self.min_interval = 60.0 / requests_per_minute
        self.last_request = 0

    def get(self, url, **kwargs):
        # Ensure minimum interval between requests
        elapsed = time.time() - self.last_request
        if elapsed < self.min_interval:
            time.sleep(self.min_interval - elapsed)

        headers = kwargs.get('headers', {})
        headers['Authorization'] = f'Token {self.api_key}'
        kwargs['headers'] = headers

        response = requests.get(url, **kwargs)
        self.last_request = time.time()

        # Check for rate limit errors
        if response.status_code == 429:
            retry_after = int(response.headers.get('Retry-After', 60))
            time.sleep(retry_after)
            return self.get(url, **kwargs)  # Retry

        return response
```

---

## Session Reuse for Performance

Tiingo's Python SDK recommends session reuse:

```python
from tiingo import TiingoClient

# Enable session reuse
config = {
    'api_key': 'YOUR_API_KEY',
    'session': True  # Reuse HTTP session
}

client = TiingoClient(config)
```

This improves performance by:
- Reusing TCP connections
- Reducing handshake overhead
- Maintaining connection pooling

**Manual session reuse with requests:**

```python
import requests

session = requests.Session()
session.headers.update({
    'Authorization': 'Token YOUR_API_KEY',
    'Content-Type': 'application/json'
})

# Reuse session for multiple requests
resp1 = session.get('https://api.tiingo.com/tiingo/daily/AAPL')
resp2 = session.get('https://api.tiingo.com/tiingo/daily/GOOGL')
resp3 = session.get('https://api.tiingo.com/tiingo/daily/MSFT')
```

---

## Summary

- **Simple token authentication**: No OAuth, no HMAC, just API token
- **Two methods**: Authorization header (recommended) or query parameter
- **WebSocket auth**: Include `authorization` field in subscribe message
- **Free tier**: API key provided immediately upon registration
- **Rate limits**: Enforced per API key (5/min, 500/day on free tier)
- **Error handling**: Standard HTTP status codes (401, 403, 429, 500)
- **Security**: Store in environment variables, never commit to version control
- **Performance**: Use session reuse for multiple requests
