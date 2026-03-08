# NASA EONET Authentication

## Summary

**No authentication required** for accessing EONET API v3.

## Public API

EONET is a fully public, open-access API. All endpoints can be accessed without:
- API keys
- OAuth tokens
- Basic authentication
- Bearer tokens
- Request signing

## Optional NASA API Key

While not required for EONET, NASA recommends obtaining an API key for **intensive use**:

### When to Get an API Key

- High-frequency polling (< 5 minute intervals)
- Large-scale data collection
- Production applications with many users
- Avoiding shared rate limit pool

### How to Get an API Key

1. Register at: https://api.nasa.gov/
2. Provide email and application description
3. Receive API key immediately (free)

### Using the API Key

Add as query parameter:

```
GET https://eonet.gsfc.nasa.gov/api/v3/events?status=open&api_key=YOUR_API_KEY
```

**Note**: As of testing (February 2026), EONET endpoints work without `api_key` parameter. The key may provide higher rate limits if NASA enforces them.

## No Headers Required

Standard requests require no special headers:

```http
GET /api/v3/events?status=open&days=30 HTTP/1.1
Host: eonet.gsfc.nasa.gov
Accept: application/json
```

Optional headers:
```http
User-Agent: YourApp/1.0
Accept-Encoding: gzip, deflate
```

## Rate Limiting

### Without API Key
- Shared rate limit pool for all unauthenticated requests
- Specific limits not documented in EONET docs
- General NASA API: ~1000 requests/hour for demo key

### With API Key
- Individual rate limit (higher than shared pool)
- Tracked via `X-RateLimit-Limit` and `X-RateLimit-Remaining` headers
- Automatic 1-hour block when exceeded

### Rate Limit Headers

Response includes:
```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
```

Check `X-RateLimit-Remaining` to avoid hitting limits:

```rust
if remaining < 10 {
    // Slow down or pause polling
}
```

## Error Responses

### Rate Limit Exceeded (429)

```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json

{
  "error": {
    "code": "OVER_RATE_LIMIT",
    "message": "API rate limit exceeded"
  }
}
```

**Recovery**: Wait 1 hour, or reduce request frequency.

### Invalid API Key (403)

If you provide an invalid `api_key` parameter:

```http
HTTP/1.1 403 Forbidden
Content-Type: application/json

{
  "error": {
    "code": "API_KEY_INVALID",
    "message": "Invalid API key"
  }
}
```

## Implementation Notes

For the Rust connector:

1. **Make API key optional** in connector config:
   ```rust
   pub struct EonetConfig {
       pub api_key: Option<String>,
   }
   ```

2. **Add API key to requests if provided**:
   ```rust
   let mut params = vec![("status", "open")];
   if let Some(key) = &config.api_key {
       params.push(("api_key", key));
   }
   ```

3. **Track rate limits**:
   ```rust
   if let Some(remaining) = response.headers().get("x-ratelimit-remaining") {
       if remaining.parse::<u32>()? < 10 {
           warn!("Rate limit approaching");
       }
   }
   ```

4. **Handle 429 errors gracefully**:
   ```rust
   match status {
       429 => Err(ExchangeError::RateLimitExceeded),
       _ => // handle other errors
   }
   ```

## Data Access Policy

From NASA Earthdata:

> NASA data are openly available without restriction. However, an Earthdata Login is required to download data and use some tools with full functionality.

For EONET API specifically:
- **Read access**: No login required
- **Data download**: No restrictions
- **Commercial use**: Allowed (public domain data)
- **Attribution**: Not required but encouraged

## CORS

EONET supports Cross-Origin Resource Sharing (CORS):
- Can be accessed from browser JavaScript
- No preflight request restrictions
- Accessible from any origin

## Summary Table

| Feature | Status |
|---------|--------|
| Authentication required | No |
| API key required | No (optional) |
| Rate limits | Yes (1000/hour typical) |
| CORS enabled | Yes |
| OAuth support | No |
| API key cost | Free |
| Commercial use | Allowed |
| Data restrictions | None |
