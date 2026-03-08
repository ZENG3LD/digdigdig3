# C2IntelFeeds Tiers and Rate Limits

## Pricing Tiers

**Free Tier**: Only tier (public repository)
**Cost**: $0

## Rate Limits

Rate limits are imposed by GitHub's infrastructure, not by the C2IntelFeeds project itself.

### GitHub Raw File Rate Limits (raw.githubusercontent.com)

#### Unauthenticated Access

**Historical limits** (pre-2024):
- **60 requests/hour** per IP address
- IP-based enforcement
- Shared across all raw.githubusercontent.com requests from that IP

**Recent changes (2024-2025)**:
- GitHub rolled out more aggressive rate limiting between December 2024 and early 2025
- Some users report single requests triggering HTTP 429 errors
- Exact new limits not officially documented
- IP-based enforcement, not account-based
- More aggressive than github.com (main site) limits

#### Authenticated Access (GitHub Token)

Using a GitHub Personal Access Token (PAT) in requests:
- **~5000 requests/hour** per token
- Account-based enforcement
- Significantly higher than unauthenticated limits

### Rate Limit Headers

GitHub raw file responses may include rate limit headers:

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 59
X-RateLimit-Reset: 1708094400
```

**Note**: Headers may not always be present on raw.githubusercontent.com responses.

### HTTP 429 Response

When rate limited:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 3600
Content-Type: text/plain

rate limit exceeded
```

## Recommended Request Patterns

### For Unauthenticated Access

**Polling intervals** (assuming 60 req/hour historical limit):

| Feeds Monitored | Max Poll Frequency |
|-----------------|-------------------|
| 1 feed | Every 1 minute |
| 2 feeds | Every 2 minutes |
| 4 feeds | Every 4 minutes |
| 6 feeds | Every 6 minutes |
| 12 feeds | Every 12 minutes |
| 30 feeds | Every 30 minutes |

**Best practice**: Poll every 15-30 minutes for most use cases to stay well below limits.

### For Authenticated Access (5000 req/hour)

With a GitHub token, you can poll much more aggressively:

| Feeds Monitored | Max Poll Frequency |
|-----------------|-------------------|
| 1 feed | Every ~1 second (not recommended) |
| 10 feeds | Every ~7 seconds |
| 30 feeds | Every ~20 seconds |
| All 26 feeds | Every ~30 seconds |

**Recommended**: Still use 5-15 minute intervals to be respectful and avoid edge cases.

## Data Transfer Limits

GitHub does not publish explicit bandwidth limits for raw file access, but:

- Large repositories (>1GB) may have restrictions
- C2IntelFeeds files are small (typically <1MB each)
- Bandwidth unlikely to be a limiting factor

### Feed File Sizes (Approximate)

| Feed Type | Typical Size |
|-----------|--------------|
| IPC2s-30day.csv | 15-50 KB |
| domainC2s-30day.csv | 20-80 KB |
| domainC2swithURLwithIP-30day.csv | 50-150 KB |
| Full historical feeds | 100-500 KB |

**Total bandwidth** for polling all 26 feeds: ~1-5 MB per poll cycle

## Concurrent Requests

GitHub raw file CDN can handle concurrent requests, but:

- Rate limits are cumulative across all concurrent requests
- No documented concurrency limits
- Recommended: Sequential requests or max 2-3 concurrent connections

## Caching Headers

GitHub raw files support HTTP caching:

### Cache-Control

```http
Cache-Control: max-age=300
```

Feeds may be cached for up to 5 minutes by GitHub's CDN.

### Last-Modified

```http
Last-Modified: Sat, 15 Feb 2026 14:30:00 GMT
```

Use with `If-Modified-Since` to avoid unnecessary downloads:

```http
GET /drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv HTTP/1.1
Host: raw.githubusercontent.com
If-Modified-Since: Sat, 15 Feb 2026 14:30:00 GMT
```

Response: `304 Not Modified` (doesn't count against bandwidth, still counts against request limit).

### ETag

```http
ETag: "abc123def456..."
```

Use with `If-None-Match`:

```http
GET /drb-ra/C2IntelFeeds/master/feeds/IPC2s-30day.csv HTTP/1.1
Host: raw.githubusercontent.com
If-None-Match: "abc123def456..."
```

## GitHub API Alternative

Instead of raw file access, use GitHub Contents API:

**Endpoint**: `https://api.github.com/repos/drb-ra/C2IntelFeeds/contents/feeds/IPC2s-30day.csv`

**Rate limits**:
- Unauthenticated: 60 requests/hour (same as raw files)
- Authenticated: 5000 requests/hour
- Better rate limit observability (headers always present)

## Quota Preservation Strategies

### 1. Conditional Requests

Always use `If-Modified-Since` or `If-None-Match`:
- Avoids downloading unchanged files
- Still counts against request quota
- Saves bandwidth

### 2. Local Caching

Implement client-side caching:
- Store Last-Modified or ETag values
- Skip requests if within cache TTL
- Reduces request count

### 3. Single Comprehensive Feed

Instead of polling multiple feeds, choose the most comprehensive feed for your use case:
- `domainC2swithURLwithIP-30day.csv` provides most complete context
- Reduces number of requests from 26 to 1-3

### 4. Batch Processing

Poll at longer intervals (30-60 minutes) and process all changes in batch.

### 5. GitHub Webhooks

For advanced users: Monitor repository commits via GitHub webhooks instead of polling.

## Error Handling

### Rate Limit Exceeded (429)

```rust
if response.status() == 429 {
    let retry_after = response.headers()
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(3600);

    // Wait retry_after seconds before retrying
    sleep(Duration::from_secs(retry_after)).await;
}
```

### Exponential Backoff

Recommended for resilient polling:

```
Initial retry: 5 seconds
Second retry: 10 seconds
Third retry: 20 seconds
Fourth retry: 40 seconds
Max backoff: 300 seconds (5 minutes)
```

## Summary

| Limit Type | Unauthenticated | Authenticated |
|------------|-----------------|---------------|
| Requests/hour | ~60 (subject to change) | ~5000 |
| Enforcement | IP-based | Token-based |
| File size limit | None (files are small) | None |
| Bandwidth limit | Not documented | Not documented |
| Concurrent requests | Not limited (respect rate limits) | Not limited |
| Cost | Free | Free (with GitHub account) |

**Recommended approach**: Use authenticated access (GitHub PAT) for production systems to avoid rate limiting issues.
