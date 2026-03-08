# FRED - Authentication

## Public Endpoints

- Public endpoints exist: No - all endpoints require authentication
- Require authentication: Yes - API key required for all requests
- Rate limits without auth: N/A - requests without API key return 400 Bad Request

## API Key

### Required For

- All endpoints: Yes - every API request must include api_key parameter
- Paid tier only: No - API is completely free
- Rate limit increase: No - single rate limit (120 req/min) for all users
- Specific endpoints: All 30 endpoints require API key

### How to Obtain

1. **Sign up**: https://fredaccount.stlouisfed.org/
   - Create a free FRED account
   - No credit card required
   - Completely free registration

2. **API key management**: https://fredaccount.stlouisfed.org/apikeys
   - Request API key after account creation
   - Can view existing keys
   - Can request multiple keys for different applications

3. **Free tier includes key**: Yes - 100% free, no paid tiers exist

### API Key Format

**Query Parameter (Standard Method):**
```
https://api.stlouisfed.org/fred/series/observations?series_id=GNPCA&api_key=YOUR_32_CHAR_KEY
```

- API key is passed as URL query parameter
- Parameter name: `api_key`
- Format: 32-character lowercase alphanumeric string
- Example: `abcdef0123456789abcdef0123456789`

**NOT supported:**
- Header-based authentication (no X-API-Key header)
- Bearer token (no Authorization header)
- Basic authentication
- OAuth

### Multiple Keys

- Multiple keys allowed: Yes
- Rate limits per key: Yes - each API key has independent 120 req/min limit
- Use cases for multiple keys:
  - Different applications
  - Development vs Production
  - Team members with separate quotas
  - Distribute load across multiple keys if needed

**Recommendation**: Request distinct API key for each application you build (per FRED terms of use).

## OAuth (if applicable)

### OAuth 2.0

- Supported: No
- FRED uses simple API key authentication only
- No OAuth flows required or available

## Signature/HMAC (if applicable - rare for data providers)

### Algorithm

- HMAC-SHA256: Not required
- HMAC-SHA512: Not required
- Other: Not required

**FRED does NOT use signature-based authentication.**

Authentication is extremely simple: just append `api_key=YOUR_KEY` to query string.

## Authentication Examples

### REST with API Key (GET request)

**Basic request:**
```bash
curl "https://api.stlouisfed.org/fred/series/observations?series_id=GNPCA&api_key=your_api_key_here&file_type=json"
```

**With all common parameters:**
```bash
curl "https://api.stlouisfed.org/fred/series/observations?\
series_id=UNRATE&\
api_key=abcdef0123456789abcdef0123456789&\
file_type=json&\
observation_start=2020-01-01&\
observation_end=2024-12-31&\
sort_order=desc"
```

**Python example:**
```python
import requests

API_KEY = "your_32_char_api_key"
BASE_URL = "https://api.stlouisfed.org/fred"

params = {
    'series_id': 'GNPCA',
    'api_key': API_KEY,
    'file_type': 'json'
}

response = requests.get(f"{BASE_URL}/series/observations", params=params)
data = response.json()
```

**JavaScript example:**
```javascript
const API_KEY = 'your_32_char_api_key';
const BASE_URL = 'https://api.stlouisfed.org/fred';

const params = new URLSearchParams({
    series_id: 'UNRATE',
    api_key: API_KEY,
    file_type: 'json'
});

fetch(`${BASE_URL}/series/observations?${params}`)
    .then(response => response.json())
    .then(data => console.log(data));
```

**Rust example:**
```rust
use reqwest;
use serde_json::Value;

const API_KEY: &str = "your_32_char_api_key";
const BASE_URL: &str = "https://api.stlouisfed.org/fred";

async fn fetch_series_data(series_id: &str) -> Result<Value, reqwest::Error> {
    let url = format!(
        "{}/series/observations?series_id={}&api_key={}&file_type=json",
        BASE_URL, series_id, API_KEY
    );

    let response = reqwest::get(&url).await?;
    let data = response.json::<Value>().await?;
    Ok(data)
}
```

### WebSocket with API Key

- Not applicable - FRED does not support WebSocket

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 400 | Bad Request - API key not set or invalid | Check api_key parameter is included and correct |
| 400 | Bad Request - Invalid series_id | Verify series_id exists in FRED database |
| 400 | Bad Request - Invalid parameter value | Check parameter format (dates, limits, etc.) |
| 401 | Unauthorized (rare) | API key not recognized - regenerate key |
| 404 | Not Found - Series or resource doesn't exist | Verify the series_id or resource_id |
| 429 | Too Many Requests - Rate limit exceeded | Wait before retrying, or use multiple API keys |
| 500 | Internal Server Error | FRED server issue - retry later |

### Error Response Format

**JSON format:**
```json
{
  "error_code": 400,
  "error_message": "Bad Request. The value for variable api_key is not registered."
}
```

**XML format:**
```xml
<?xml version="1.0" encoding="utf-8" ?>
<error code="400" message="Bad Request. The value for variable api_key is not registered."/>
```

### Common Error Messages

1. **"Bad Request. Variable api_key is not set."**
   - Cause: Missing api_key parameter
   - Fix: Add `api_key=YOUR_KEY` to request

2. **"Bad Request. The value for variable api_key is not registered."**
   - Cause: Invalid or incorrect API key
   - Fix: Verify API key is correct 32-character string

3. **"Bad Request. The value for variable series_id is not valid."**
   - Cause: Series doesn't exist or wrong format
   - Fix: Search for series via /fred/series/search first

4. **Rate limit exceeded (HTTP 429)**
   - Cause: More than 120 requests per minute
   - Fix: Implement rate limiting in your code, or use multiple API keys

## Rate Limit Headers

FRED does not provide rate limit headers in responses (no X-RateLimit-* headers).

You must track rate limits client-side:
- Maximum: 120 requests per minute
- Window: 60 seconds (rolling or fixed - not documented)
- Recommended: Implement token bucket or sliding window rate limiter

**Rate Limiting Strategy:**
```python
import time
from collections import deque

class FREDRateLimiter:
    def __init__(self, max_requests=120, window_seconds=60):
        self.max_requests = max_requests
        self.window_seconds = window_seconds
        self.requests = deque()

    def wait_if_needed(self):
        now = time.time()
        # Remove requests outside window
        while self.requests and self.requests[0] < now - self.window_seconds:
            self.requests.popleft()

        # If at limit, wait
        if len(self.requests) >= self.max_requests:
            sleep_time = self.window_seconds - (now - self.requests[0])
            if sleep_time > 0:
                time.sleep(sleep_time)

        self.requests.append(time.time())
```

## Best Practices

1. **Store API key securely**: Use environment variables, never commit to source control
2. **One key per application**: Follow FRED's recommendation
3. **Handle errors gracefully**: Check for 400/404/429 and retry appropriately
4. **Implement rate limiting**: Don't rely on FRED to throttle you
5. **Cache responses**: Economic data doesn't change frequently
6. **Use appropriate file_type**: JSON is typically easier to parse than XML
7. **Monitor usage**: No dashboard exists, so log your requests

## Security Notes

- API keys are transmitted in URL (query parameter), not headers
- Always use HTTPS (api.stlouisfed.org enforces HTTPS)
- URLs with API keys may appear in logs - be cautious
- Rotate API keys periodically if concerned about exposure
- FRED API keys are low-risk (free service, public data) but still protect them
