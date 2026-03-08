# FAA NASSTATUS API - Tiers and Rate Limits

## Pricing Tiers
**Not applicable** - The FAA NASSTATUS API is completely **free** with no paid tiers.

---

## Rate Limits

### Official Documentation
**None publicly available.** The FAA does not publish explicit rate limits for the NASSTATUS API.

### Observed Behavior
Based on community usage and best practices:
- No documented requests-per-second limit
- No documented requests-per-day limit
- No API key or quota system
- Likely IP-based throttling for excessive use

### Recommended Limits (Self-Imposed)

| Metric | Recommended Value | Reasoning |
|--------|-------------------|-----------|
| Polling interval | 60 seconds | Balances freshness with server load |
| Minimum interval | 30 seconds | Avoid excessive polling |
| Burst requests | 1-2 per interval | No need for multiple simultaneous requests |
| Concurrent connections | 1-2 | Single endpoint, no parallelization needed |
| Retry backoff | Exponential (1s → 60s) | Graceful degradation on errors |
| Timeout | 30 seconds | Reasonable for government API |

---

## Connection Limits

### Concurrent Connections
- **Recommended**: 1 connection per client instance
- **Maximum**: Unknown (likely standard HTTP limits: 2-6 per domain)

**Why limit to 1?**
- Single endpoint returns all data
- No need for parallel requests
- Reduces load on FAA servers
- Avoids potential throttling

### WebSocket
**Not applicable** - No WebSocket support.

---

## Data Volume Limits

### Response Size
- **Typical**: 2-20 KB per response (XML)
- **Maximum observed**: ~50 KB (during major weather events)
- **Average**: ~10 KB

### Bandwidth Considerations

**60-second polling for 1 month**:
```
Requests per day: 1,440
Requests per month: 43,200
Average response: 10 KB
Monthly bandwidth: ~432 MB
Annual bandwidth: ~5.2 GB
```

**This is minimal bandwidth usage.**

### No Historical Data Limits
Historical data is not available, so there are no limits on:
- Time range queries
- Bulk downloads
- Archive access

---

## Throttling Indicators

### How to Detect Throttling

Since there are no documented rate limits, watch for:

1. **HTTP 429 (Too Many Requests)**
   - Standard rate limit response
   - Not confirmed for FAA API, but possible

2. **HTTP 503 (Service Unavailable)**
   - May indicate temporary throttling
   - Could also indicate genuine outage

3. **Increased latency**
   - Response times > 5 seconds
   - May precede hard throttling

4. **Connection timeouts**
   - Requests hanging without response
   - Could indicate IP-level blocking

### Recommended Response

```rust
match response.status() {
    StatusCode::OK => {
        // Process normally
    },
    StatusCode::TOO_MANY_REQUESTS => {
        // Back off for 5 minutes
        tokio::time::sleep(Duration::from_secs(300)).await;
    },
    StatusCode::SERVICE_UNAVAILABLE => {
        // Exponential backoff (start with 1 minute)
        tokio::time::sleep(Duration::from_secs(60)).await;
    },
    _ => {
        // Log error and retry with backoff
    }
}
```

---

## Caching Requirements

### Client-Side Caching
**Strongly recommended** to:
- Reduce load on FAA servers
- Improve response times
- Avoid potential throttling

**Recommended cache strategy**:
```rust
Cache-Control: max-age=60, stale-while-revalidate=30
```

**Rust implementation**:
```rust
use cached::proc_macro::cached;
use std::time::Duration;

#[cached(time = 60, result = true)]
async fn fetch_cached_airport_status() -> Result<AirportStatus, Error> {
    fetch_from_faa().await
}
```

### Stale-While-Revalidate
Serve stale data while fetching fresh data in background:
```
Age 0-60s: Serve from cache, don't fetch
Age 60-90s: Serve from cache, fetch new data in background
Age 90s+: Force fetch new data
```

**Benefits**:
- Reduces perceived latency
- Graceful degradation during outages
- Fewer requests to FAA servers

---

## Scaling Considerations

### Single Client
**No issues** - Standard polling every 60 seconds is well within expected usage.

### Multiple Clients (Same IP)
If running multiple instances from the same IP (e.g., load-balanced servers):
- **Risk**: Potential IP-based throttling
- **Solution**: Implement shared cache layer

```
┌─────────┐    ┌─────────┐    ┌─────────┐
│ Client 1│    │ Client 2│    │ Client 3│
└────┬────┘    └────┬────┘    └────┬────┘
     │              │              │
     └──────────────┼──────────────┘
                    ▼
            ┌───────────────┐
            │  Redis Cache  │
            └───────┬───────┘
                    │ 60s poll
                    ▼
            ┌───────────────┐
            │   FAA API     │
            └───────────────┘
```

### Enterprise Deployment
For high-volume or mission-critical usage:
- **Contact FAA** to discuss usage patterns
- **Consider SWIM** (System Wide Information Management) for enterprise needs
- **Implement proxy service** to centralize polling

---

## Geographic Restrictions

### Access from Outside US
**Unknown** - The FAA may restrict access to US-based IPs, but this is not documented.

**Observed behavior**:
- API is accessible internationally (anecdotal reports)
- No CORS headers (may require server-side proxy for browser clients)

**Recommendation**: Test from your deployment region. If blocked, use US-based proxy.

---

## Fair Use Policy

### Implicit Guidelines
While not documented, the FAA expects:
- **Reasonable polling intervals** (not sub-second)
- **Descriptive User-Agent** (identify your application)
- **Error handling** (don't hammer on failures)
- **Caching** (reduce redundant requests)

### Prohibited Behavior (Assumed)
- Sub-second polling
- Distributed denial of service (intentional or accidental)
- Scraping for non-aviation purposes
- Reselling raw API access

---

## Comparison with Other Aviation APIs

| API | Rate Limit | Authentication | Cost |
|-----|------------|----------------|------|
| FAA NASSTATUS | Undocumented | None | Free |
| FlightAware | 500 req/day (free tier) | API key | Paid tiers available |
| AviationStack | 100 req/month (free tier) | API key | Paid tiers available |
| OpenSky Network | Anonymous: 400 req/day | Optional account | Free |
| Aviation Weather (aviationweather.gov) | Undocumented | None | Free |

**FAA NASSTATUS is one of the most permissive free aviation APIs.**

---

## Monitoring Your Usage

### Recommended Metrics

Track these metrics to ensure compliance:
```rust
struct ApiMetrics {
    requests_per_minute: u32,
    requests_per_hour: u32,
    requests_per_day: u32,
    average_response_time_ms: u64,
    error_rate: f64,
    cache_hit_rate: f64,
}
```

### Warning Thresholds

| Metric | Warning Level | Action |
|--------|---------------|--------|
| Requests/minute | > 2 | Review polling logic |
| Requests/hour | > 120 | Increase polling interval |
| Error rate | > 5% | Check FAA status, implement backoff |
| Response time | > 5s | Check network, consider failover |
| Cache hit rate | < 80% | Review cache TTL settings |

---

## Summary

| Feature | Status |
|---------|--------|
| Official rate limit | None documented |
| Recommended polling | 60 seconds |
| Minimum polling | 30 seconds |
| Authentication | None |
| Cost | Free |
| Concurrent connections | 1-2 recommended |
| Caching | Strongly recommended (60s TTL) |
| Geographic restrictions | Unknown (likely unrestricted) |
| Fair use policy | Implicit (reasonable use) |

**Best practice**: Implement 60-second polling with client-side caching and exponential backoff on errors. This ensures good citizenship while maintaining data freshness.
