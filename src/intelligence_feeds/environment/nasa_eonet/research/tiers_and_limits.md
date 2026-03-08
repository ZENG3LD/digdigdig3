# NASA EONET Rate Limits and Tiers

## API Tiers

EONET does **not have formal pricing tiers**. The API is completely free with optional API key for higher limits.

### Tier Comparison

| Tier | Cost | Authentication | Rate Limit (estimated) | Use Case |
|------|------|----------------|------------------------|----------|
| **Anonymous** | Free | None | ~1000 req/hour (shared pool) | Testing, light usage |
| **API Key** | Free | API key in query params | ~1000 req/hour (individual) | Production apps, higher reliability |

**Note**: Specific rate limits for EONET are not documented. Values above are based on general NASA API guidelines.

## Rate Limit Details

### Enforcement Mechanism

- **Header-based tracking**: Every response includes rate limit headers
- **Automatic reset**: Limits reset every hour (rolling or fixed window not specified)
- **Temporary block**: Exceeding limit results in 1-hour block
- **No permanent bans**: Rate limit violations don't result in permanent API access loss

### Rate Limit Headers

Every API response includes:

```http
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 987
```

**Interpretation**:
- `X-RateLimit-Limit`: Total requests allowed per hour
- `X-RateLimit-Remaining`: Requests left in current window

### Rate Limit Exceeded Response

When limit is exceeded:

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

**Recovery**:
1. Wait for rate limit window to reset (up to 1 hour)
2. Reduce polling frequency
3. Implement exponential backoff

## Polling Recommendations

Given natural event update cadence, these polling intervals stay well within rate limits:

| Event Category | Recommended Polling | Requests/Day | % of Hourly Limit |
|----------------|---------------------|--------------|-------------------|
| Wildfires | 15 minutes | 96 | ~10% |
| Severe Storms | 10 minutes | 144 | ~15% |
| Volcanoes | 30 minutes | 48 | ~5% |
| Floods | 30 minutes | 48 | ~5% |
| All Events | 30 minutes | 48 | ~5% |

**Multi-category monitoring** (polling 3 categories every 15 min): ~288 req/day = ~12 req/hour (~1% of limit)

## Request Optimization

### Efficient Query Patterns

1. **Use `days` parameter** to limit response size:
   ```
   GET /api/v3/events?status=open&days=1
   ```
   Fewer events = smaller response = faster processing.

2. **Filter by category** instead of fetching all events:
   ```
   GET /api/v3/events?category=wildfires,severeStorms&status=open
   ```

3. **Use `limit` parameter** for dashboard views:
   ```
   GET /api/v3/events?status=open&limit=50
   ```

4. **Conditional requests** (if EONET supports ETags):
   ```http
   If-None-Match: "abc123"
   ```
   Returns 304 Not Modified if data hasn't changed.

### Anti-Patterns (Avoid)

- Polling every 1-5 minutes (unnecessary for slow-moving events)
- Fetching all historical events (`start=2000-01-01`) repeatedly
- Not using category filters when monitoring specific event types
- Making parallel requests for same data
- Not caching static data (categories, sources)

## Data Limits

### Response Size Limits

- **No documented hard limit** on response size
- Events endpoint can return hundreds of events
- GeoJSON responses can be large (geometry arrays)

**Recommendation**: Use `limit` parameter to cap response size:
```
GET /api/v3/events?status=open&days=7&limit=100
```

### Query Parameter Limits

- **Bounding box**: No documented size limit
- **Date ranges**: Can query years of historical data
- **Multiple sources**: Comma-separated list (no documented limit)
- **Multiple categories**: Comma-separated list (13 max categories exist)

### Time Range Limits

EONET stores events from **2000 onwards**:
- Oldest events: ~2000-2005 (varies by category)
- No limit on historical queries
- Closed events remain in database indefinitely

## Connection Limits

### Concurrent Connections

- **No documented limit** on concurrent connections
- Standard HTTP/1.1 best practice: 2-6 connections per host
- HTTP/2 supported (multiplexing on single connection)

### Request Timeout

- **No documented timeout** from API side
- Client-side timeout recommendation: 30 seconds for events endpoint

## Bandwidth Considerations

### Typical Response Sizes

| Endpoint | Avg Size | Notes |
|----------|----------|-------|
| `/events?days=1` | 10-50 KB | 10-50 events |
| `/events?days=30` | 50-200 KB | 100-500 events |
| `/events/geojson?days=30` | 75-300 KB | Larger due to GeoJSON structure |
| `/categories` | <5 KB | 13 categories (static) |
| `/sources` | <10 KB | 33 sources (static) |

**Bandwidth estimate** (polling every 15 min): ~2-5 MB/day

## API Key Management

### Obtaining API Key

1. Visit: https://api.nasa.gov/
2. Fill form: email, first name, last name, app description
3. Receive key instantly
4. Key format: ~40-character alphanumeric string

### Key Usage

Add to any request:
```
https://eonet.gsfc.nasa.gov/api/v3/events?status=open&api_key=YOUR_KEY
```

**Security**:
- Don't commit API keys to version control
- Use environment variables: `EONET_API_KEY`
- Rotate keys if exposed

### Key Limits

- **No limit** on number of API keys per user
- Keys don't expire
- Can be revoked/regenerated at api.nasa.gov

## Best Practices

### For Rust Connector

1. **Track rate limits**:
   ```rust
   struct RateLimiter {
       remaining: u32,
       limit: u32,
       reset_time: Option<Instant>,
   }
   ```

2. **Implement backoff** on 429 errors:
   ```rust
   match status {
       429 => {
           tokio::time::sleep(Duration::from_secs(3600)).await;
           retry_request()
       }
   }
   ```

3. **Cache static data**:
   - Categories list (changes rarely)
   - Sources list (changes rarely)
   - Fetch once on startup, refresh daily

4. **Warn before limits**:
   ```rust
   if remaining < 50 {
       warn!("Approaching rate limit: {} requests remaining", remaining);
   }
   ```

5. **Use exponential backoff** for retries:
   ```rust
   let delay = min(2^retry_count * 1000, 60000); // Cap at 60s
   ```

## Summary Table

| Limit Type | Value | Enforcement |
|------------|-------|-------------|
| Hourly rate limit (est.) | 1000 requests | 429 response + 1hr block |
| Response size | Unlimited (practical: 1MB max) | None |
| Concurrent connections | Unlimited (recommended: 6) | None |
| Historical data | From ~2000 onwards | Data availability |
| Query complexity | No documented limits | None |
| API key cost | Free | None |
| Bandwidth | Unlimited | None |

**Conclusion**: EONET is extremely permissive. Reasonable polling (every 15-30 minutes) will never approach limits.
