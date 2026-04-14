# AlphaVantage - Authentication

## Public Endpoints

- **Public endpoints exist**: No - all endpoints require authentication
- **Require authentication**: Yes - API key required for all requests
- **Rate limits without auth**: Not applicable (auth always required)

## API Key

### Required For
- **All endpoints**: Yes - every API call requires an API key
- **Paid tier only**: No - free tier also requires API key
- **Rate limit increase**: Yes - premium tiers unlock higher rate limits
- **Specific endpoints**: All endpoints require authentication

### How to Obtain

1. **Sign up**: https://www.alphavantage.co/support/#api-key
2. **Registration**:
   - Requires legitimate email address
   - Free tier available immediately upon sign-up
   - No credit card required for free tier
3. **API key management**:
   - Login to account dashboard
   - View API key
   - Monitor usage
4. **Free tier includes key**: Yes - instant API key upon registration

### API Key Format

**Method**: Query parameter (most common)

```bash
https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=IBM&apikey=YOUR_API_KEY
```

**NOT supported**:
- ❌ Header: `X-API-Key: your_api_key_here`
- ❌ Bearer token: `Authorization: Bearer xxx`
- ✅ **Only query param works**: `?apikey=YOUR_API_KEY`

### Demo API Key

For testing and examples:
```bash
https://www.alphavantage.co/query?function=TIME_SERIES_DAILY&symbol=IBM&apikey=demo
```

**Limitations**:
- Works with **IBM stock only**
- Free tier rate limits apply (5 req/min)
- Cannot be used for other symbols
- For demo/documentation purposes only

### Multiple Keys

- **Multiple keys allowed**: Likely not for free tier, check with support
- **Rate limits per key**: Yes - each API key has independent rate limits
- **Use cases for multiple keys**:
  - Multiple applications
  - Separate development/production environments
  - Rate limit distribution (requires multiple accounts)

## OAuth

### OAuth 2.0
- **Supported**: No
- **Grant types**: Not applicable
- **Scopes**: Not applicable
- **Token endpoint**: Not applicable
- **Authorization endpoint**: Not applicable

AlphaVantage uses simple API key authentication only.

## Signature/HMAC

### Algorithm
**Not required** - AlphaVantage does NOT use HMAC signatures.

Unlike trading APIs (e.g., Binance, Kraken), AlphaVantage is a **read-only data provider** with no order execution. Simple API key in query string is sufficient for security.

### Why No HMAC?
- **Read-only API**: No ability to execute trades or modify account state
- **Lower security risk**: Data access only, no financial transactions
- **Simpler integration**: Easy for developers to implement
- **Query parameter approach**: Standard for data APIs

## Authentication Examples

### REST with API Key (Query Parameter)

```bash
# Get current quote
curl "https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol=IBM&apikey=YOUR_API_KEY"

# Get forex daily data
curl "https://www.alphavantage.co/query?function=FX_DAILY&from_symbol=EUR&to_symbol=USD&apikey=YOUR_API_KEY"

# Get company overview
curl "https://www.alphavantage.co/query?function=COMPANY_OVERVIEW&symbol=AAPL&apikey=YOUR_API_KEY"

# Get technical indicator
curl "https://www.alphavantage.co/query?function=SMA&symbol=IBM&interval=daily&time_period=20&series_type=close&apikey=YOUR_API_KEY"
```

### Python Example

```python
import requests

API_KEY = "YOUR_API_KEY"
BASE_URL = "https://www.alphavantage.co/query"

# Get daily time series
params = {
    'function': 'TIME_SERIES_DAILY',
    'symbol': 'IBM',
    'outputsize': 'compact',
    'apikey': API_KEY
}

response = requests.get(BASE_URL, params=params)
data = response.json()

# Access data
meta_data = data.get('Meta Data', {})
time_series = data.get('Time Series (Daily)', {})
```

### JavaScript Example

```javascript
const API_KEY = "YOUR_API_KEY";
const BASE_URL = "https://www.alphavantage.co/query";

// Get forex exchange rate
const params = new URLSearchParams({
    function: 'CURRENCY_EXCHANGE_RATE',
    from_currency: 'EUR',
    to_currency: 'USD',
    apikey: API_KEY
});

fetch(`${BASE_URL}?${params}`)
    .then(response => response.json())
    .then(data => console.log(data))
    .catch(error => console.error('Error:', error));
```

### R Example

```r
library(httr)
library(jsonlite)

API_KEY <- "YOUR_API_KEY"
BASE_URL <- "https://www.alphavantage.co/query"

# Get daily data
response <- GET(BASE_URL, query = list(
    `function` = "TIME_SERIES_DAILY",
    symbol = "IBM",
    apikey = API_KEY
))

data <- content(response, "parsed")
```

## WebSocket with API Key

**Not applicable** - AlphaVantage does not support WebSocket.

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| N/A | Invalid API key | Check key is correct, verify from dashboard |
| N/A | Missing API key parameter | Add `apikey=YOUR_KEY` to query string |
| HTTP 429 | Rate limit exceeded | Wait until next minute or upgrade to premium |
| N/A | Daily limit reached (free tier) | Wait until next day or upgrade to premium |
| N/A | Invalid function parameter | Check function name spelling |
| N/A | Invalid symbol | Verify ticker symbol exists |
| N/A | Premium feature | Upgrade to premium tier |

### Invalid API Key Response

```json
{
  "Error Message": "Invalid API call. Please retry or visit the documentation for API_KEY"
}
```

**Resolution**:
1. Check API key spelling
2. Verify API key from dashboard
3. Ensure `apikey` parameter is included
4. Register for new key if lost

### Rate Limit Exceeded Response

```json
{
  "Note": "Thank you for using Alpha Vantage! Our standard API call frequency is 5 calls per minute. Please visit https://www.alphavantage.co/premium/ if you would like to target a higher API call frequency."
}
```

**Resolution**:
1. Wait 60 seconds before next request
2. Implement exponential backoff
3. Upgrade to premium tier for higher limits
4. Use REALTIME_BULK_QUOTES for multiple symbols

### Daily Limit Reached (Free Tier)

```json
{
  "Note": "Thank you for using Alpha Vantage! You have reached the daily limit of 25 API requests. Please try again tomorrow or visit https://www.alphavantage.co/premium/ to upgrade."
}
```

**Resolution**:
1. Wait until next day (resets at midnight UTC)
2. Upgrade to premium (no daily limits)
3. Optimize API usage (cache responses, bulk queries)

### Invalid Parameters

```json
{
  "Error Message": "Invalid parameter 'interval'. Please retry."
}
```

**Resolution**:
1. Check documentation for valid parameter values
2. Verify parameter spelling
3. Ensure required parameters are included

### Premium Feature Response

```json
{
  "Note": "This API endpoint is not available on your current plan. Please visit https://www.alphavantage.co/premium/ to upgrade."
}
```

**Resolution**:
1. Upgrade to premium tier
2. Use free tier alternative endpoints
3. Check endpoint documentation for tier requirements

## Rate Limit Headers

AlphaVantage **does not return rate limit headers** in HTTP response.

**No headers like**:
- ❌ `X-RateLimit-Limit`
- ❌ `X-RateLimit-Remaining`
- ❌ `X-RateLimit-Reset`
- ❌ `Retry-After`

**Client-side tracking required**:
- Track requests per minute manually
- Implement request queue with rate limiting
- Monitor for rate limit error messages in response body

## Security Best Practices

### API Key Protection

1. **Never commit API keys to version control**
   ```bash
   # Use environment variables
   export ALPHAVANTAGE_API_KEY="your_key_here"
   ```

2. **Use environment variables**
   ```python
   import os
   API_KEY = os.environ.get('ALPHAVANTAGE_API_KEY')
   ```

3. **Rotate keys periodically** (if supported)

4. **Separate keys for dev/prod** (if multiple keys allowed)

5. **Monitor usage** via dashboard for unauthorized access

### HTTPS

All requests are over HTTPS - no HTTP support:
```
✅ https://www.alphavantage.co/query
❌ http://www.alphavantage.co/query
```

### API Key in Logs

**Warning**: API key is in URL query string, which may appear in:
- Server logs
- Browser history
- Proxy logs
- Network monitoring tools

For sensitive applications, consider caching responses and minimizing API calls.

## Summary

| Aspect | Details |
|--------|---------|
| **Auth method** | API key in query parameter |
| **Key location** | `?apikey=YOUR_KEY` |
| **Required for** | All endpoints (no public access) |
| **How to get** | Sign up at alphavantage.co/support/#api-key |
| **Free tier** | Yes, with rate limits (5/min, 25/day) |
| **OAuth** | No |
| **HMAC/Signature** | No |
| **Headers** | Not supported for auth |
| **Bearer token** | Not supported |
| **Rate limit headers** | Not provided |
| **Demo key** | `apikey=demo` (IBM stock only) |
| **Multiple keys** | Possible with multiple accounts |
| **Security** | HTTPS only, simple key-based |
