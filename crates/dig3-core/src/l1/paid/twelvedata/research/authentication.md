# Twelvedata - Authentication

## Public Endpoints

- Public endpoints exist: **Yes**
- Require authentication: **Recommended but not always required**
- Rate limits without auth: Very limited (demo key available for testing)
- Demo key: `apikey=demo` available for basic testing

## API Key

### Required For
- All endpoints: **Recommended** (some work without but with severe limits)
- Paid tier only: **No** (free tier Basic plan includes API key)
- Rate limit increase: **Yes** (each tier increases limits)
- Specific endpoints:
  - **Always required**: Fundamentals (profile, financials), Market Movers, Cross Listings, Exchange Schedule
  - **Recommended for**: All other endpoints (better rate limits, guaranteed access)
  - **Demo available**: Time series, quote, price (use `apikey=demo`)

### How to Obtain
- Sign up: https://twelvedata.com/pricing
- API key management: https://twelvedata.com/account (dashboard after signup)
- Free tier includes key: **Yes** (Basic plan - 8 API calls/min, 800/day)
- Trial: 8 API credits + 8 trial WebSocket API credits

### API Key Format

**Recommended method** (HTTP Header):
```
Authorization: apikey YOUR_API_KEY_HERE
```

**Alternative method 1** (Query Parameter):
```
?apikey=YOUR_API_KEY_HERE
```

**Alternative method 2** (Custom Header):
```
X-API-Key: YOUR_API_KEY_HERE
```

**Note**: Documentation indicates header method is recommended over query parameter for security.

### API Key Characteristics
- Format: Alphanumeric string (exact length not specified)
- Single key per account on free tier
- Multiple keys possible on higher tiers (not explicitly documented)
- Key rotation: Manage via dashboard

### Multiple Keys
- Multiple keys allowed: **Likely on higher tiers** (not explicitly documented)
- Rate limits per key: **Yes** (each key has independent rate limit)
- Use cases for multiple keys:
  - Separate production/development environments
  - Different applications
  - Team member separation
  - Service isolation

### Storage & Security
- **Server-side only**: Keys must be stored server-side, **never** expose in client-side code
- Environment variables recommended
- Never commit to version control
- Rotate periodically for security

## OAuth

### OAuth 2.0
- Supported: **No**
- Grant types: N/A
- Scopes: N/A
- Token endpoint: N/A
- Authorization endpoint: N/A

**Twelvedata uses simple API key authentication only.**

## Signature/HMAC

### Algorithm
- **Not required**: Twelvedata does NOT use HMAC/signature authentication
- **Simple API key only**: Unlike trading APIs, Twelvedata uses straightforward API key authentication

This is typical for **data providers** (vs trading exchanges which often require signatures).

## Authentication Examples

### REST with API Key (Header - Recommended)
```bash
curl -H "Authorization: apikey YOUR_API_KEY" \
  "https://api.twelvedata.com/time_series?symbol=AAPL&interval=1min&outputsize=10"
```

### REST with API Key (Query Parameter)
```bash
curl "https://api.twelvedata.com/quote?symbol=AAPL&apikey=YOUR_API_KEY"
```

### REST with Demo Key
```bash
curl "https://api.twelvedata.com/price?symbol=AAPL&apikey=demo"
```

### WebSocket with API Key
```javascript
const ws = new WebSocket('wss://ws.twelvedata.com/v1/quotes/price?apikey=YOUR_API_KEY');

ws.onopen = () => {
  ws.send(JSON.stringify({
    action: 'subscribe',
    params: {
      symbols: 'AAPL,TSLA,BTC/USD'
    }
  }));
};
```

### Python with Official SDK
```python
from twelvedata import TDClient

# Initialize client with API key
td = TDClient(apikey="YOUR_API_KEY")

# Fetch time series
ts = td.time_series(
    symbol="AAPL",
    interval="1min",
    outputsize=10
)

# Get as JSON
data = ts.as_json()
```

### Rust Implementation Example
```rust
use reqwest::Client;
use serde_json::Value;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    let api_key = std::env::var("TWELVEDATA_API_KEY")?;

    // Method 1: Header authentication (recommended)
    let response = client
        .get("https://api.twelvedata.com/quote")
        .header("Authorization", format!("apikey {}", api_key))
        .query(&[("symbol", "AAPL")])
        .send()
        .await?;

    let data: Value = response.json().await?;
    println!("{:#?}", data);

    Ok(())
}
```

### Multiple Symbols (Batch)
```bash
curl -H "Authorization: apikey YOUR_API_KEY" \
  "https://api.twelvedata.com/time_series?symbol=AAPL,TSLA,MSFT&interval=1day&outputsize=5"
```

## Error Codes

| Code | Description | Resolution |
|------|-------------|------------|
| 400 | Bad Request | Invalid parameters - check query params, ensure symbol exists |
| 401 | Unauthorized / Invalid API key | Verify API key is correct, check it hasn't been revoked |
| 403 | Forbidden | Endpoint requires higher tier plan - upgrade subscription |
| 404 | Not Found | Symbol or data not available - verify symbol spelling |
| 414 | URI Too Long | Parameter array exceeds limits - reduce number of symbols |
| 429 | Rate Limit Exceeded | Too many requests - wait for rate limit reset or upgrade plan |
| 500 | Internal Server Error | Backend issue - retry with exponential backoff or contact support |

### Error Response Format
```json
{
  "code": 401,
  "message": "Invalid API key. Please check your API key at https://twelvedata.com/account",
  "status": "error"
}
```

### Detailed Error Examples

#### 401 - Invalid API Key
```json
{
  "code": 401,
  "message": "Invalid API key",
  "status": "error"
}
```

**Solutions:**
1. Verify API key copied correctly (no extra spaces)
2. Check key status in dashboard (not revoked/expired)
3. Ensure using correct authentication method (header vs query param)
4. Regenerate key if necessary

#### 403 - Forbidden (Tier Restriction)
```json
{
  "code": 403,
  "message": "This endpoint requires a Grow plan or higher",
  "status": "error"
}
```

**Solutions:**
1. Upgrade to required plan tier
2. Use alternative endpoint if available
3. Check plan requirements in documentation

#### 429 - Rate Limit Exceeded
```json
{
  "code": 429,
  "message": "Rate limit exceeded. Your plan allows 60 API calls per minute",
  "status": "error",
  "limit": 60,
  "remaining": 0,
  "reset": 1234567890
}
```

**Solutions:**
1. Implement exponential backoff retry logic
2. Check `Retry-After` header for wait time
3. Reduce request frequency
4. Upgrade to higher tier for increased limits
5. Use batch requests where possible (1 credit per 100 symbols)

### Rate Limit Response Headers

On all responses:
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1234567890
```

On 429 errors:
```
Retry-After: 30
```

**Header explanations:**
- `X-RateLimit-Limit`: Total requests allowed in current window
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets
- `Retry-After`: Seconds to wait before retrying (on 429 only)

## Best Practices

### Security
1. **Never expose API keys** in client-side JavaScript, mobile apps, or public repositories
2. **Use environment variables** for key storage
3. **Implement key rotation** policy (e.g., quarterly)
4. **Monitor key usage** via dashboard for suspicious activity
5. **Use separate keys** for dev/staging/production environments
6. **Revoke compromised keys** immediately via dashboard

### Rate Limiting
1. **Implement exponential backoff** for 429 errors
2. **Cache responses** when appropriate (e.g., reference data, daily bars)
3. **Use batch requests** for multiple symbols (1 credit per 100 symbols vs 1 per symbol)
4. **Monitor rate limit headers** proactively
5. **Spread requests** across time windows when possible
6. **Upgrade tier** if consistently hitting limits

### Error Handling
```rust
// Rust error handling example
async fn fetch_quote(symbol: &str) -> Result<Quote, TwelvedataError> {
    let response = client
        .get("https://api.twelvedata.com/quote")
        .header("Authorization", format!("apikey {}", api_key))
        .query(&[("symbol", symbol)])
        .send()
        .await?;

    match response.status() {
        StatusCode::OK => {
            let quote: Quote = response.json().await?;
            Ok(quote)
        }
        StatusCode::UNAUTHORIZED => {
            Err(TwelvedataError::InvalidApiKey)
        }
        StatusCode::FORBIDDEN => {
            Err(TwelvedataError::InsufficientTier)
        }
        StatusCode::TOO_MANY_REQUESTS => {
            let retry_after = response.headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(60);
            Err(TwelvedataError::RateLimitExceeded { retry_after })
        }
        StatusCode::NOT_FOUND => {
            Err(TwelvedataError::SymbolNotFound(symbol.to_string()))
        }
        _ => {
            let error: ErrorResponse = response.json().await?;
            Err(TwelvedataError::ApiError(error.message))
        }
    }
}
```

## API Key Management Dashboard

### Available at: https://twelvedata.com/account

**Features:**
- View current API key(s)
- Regenerate keys
- Monitor usage statistics
- Check quota consumption
- View rate limit status
- Upgrade/downgrade plan
- Billing information

## Tier Comparison & Authentication

| Tier | Rate Limit | API Key Required | WebSocket | Credits/Day | Price |
|------|------------|------------------|-----------|-------------|-------|
| Basic (Free) | 8/min, 800/day | Yes | No | 800 | $0 |
| Grow | 55-377/min | Yes | Yes | ~40K-270K | $29-79/mo |
| Pro | 610-1597/min | Yes | Yes | ~440K-1.15M | $99-249/mo |
| Ultra | 2584+ /min | Yes | Yes | 1.87M+ | $329-1999/mo |
| Enterprise | Custom | Yes | Yes | Custom | Contact |

**All tiers require API key for full functionality.**

## Demo Key Limitations

Using `apikey=demo`:
- **Very limited rate limits** (exact limits not documented, likely <10/min)
- **Basic endpoints only**: time_series, quote, price
- **No fundamentals** or premium data
- **No WebSocket** access
- **Not for production**: Testing only
- **No SLA** or support
- **May have outdated/sample data**

## Authentication Flow Summary

1. **Sign up** at https://twelvedata.com/pricing
2. **Obtain API key** from dashboard (automatically generated on signup)
3. **Store securely** in environment variable or secrets manager
4. **Use in requests** via `Authorization: apikey YOUR_KEY` header (recommended)
5. **Monitor usage** via dashboard and rate limit headers
6. **Handle errors** gracefully (especially 401, 403, 429)
7. **Upgrade tier** when hitting limits or needing premium features
8. **Rotate keys** periodically for security

## No Authentication Needed (Very Limited)

Certain endpoints may work without authentication using demo key:
```bash
curl "https://api.twelvedata.com/price?symbol=AAPL&apikey=demo"
```

However, **this is NOT recommended** for production use due to:
- Severe rate limiting
- Potential data delays or sample data
- No SLA or reliability guarantees
- Limited endpoint access

**Always use proper API key for production applications.**
