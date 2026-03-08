# NWS Alerts API - Tiers, Rate Limits, and Usage Policies

## Pricing Tiers

**Tier**: FREE (Public Service)

**Cost**: $0.00 - Completely free

**Access Levels**: NONE (no tiered access)

The NWS API is a public service of the United States Government. There are no premium tiers, paid plans, or subscription levels.

---

## Rate Limits

### Official Rate Limit

**Limit**: 1 request per 30 seconds

**Scope**: Per IP address (not per User-Agent)

**Source**: Official NWS documentation
- "Requests should not exceed one per 30 seconds"
- Documented at https://www.weather.gov/documentation/services-web-alerts

### Enforcement

**Method**: Rate-limiting firewalls

**Purpose**: "Prevent abuse and ensure the service is accessible to all partners"

**Response When Limited**:
- HTTP Status: `429 Too Many Requests`
- Retry After: Typically 5 seconds
- Error Body: Problem detail JSON

**Example Error**:
```json
{
  "correlationId": "xyz789",
  "title": "Too Many Requests",
  "type": "https://api.weather.gov/problems/RateLimited",
  "status": 429,
  "detail": "Rate limit exceeded. Please retry after 5 seconds."
}
```

### Rate Limit Variability

**Direct Clients**: Less likely to hit limits
- Individual users making API calls from unique IPs
- Mobile apps with distributed user base

**Proxy/Aggregator Requests**: More likely to hit limits
- Corporate networks (shared IP)
- Server-side aggregators serving many users
- Cloud functions (shared egress IPs)

**Implication**: If building a service, distribute requests across multiple source IPs or implement aggressive caching

---

## Request Quotas

**Daily Quota**: NONE

**Monthly Quota**: NONE

**Total Requests**: UNLIMITED (subject to rate limits)

Unlike commercial APIs, NWS does not impose:
- Daily request caps
- Monthly usage limits
- Per-user quotas
- Request counting or tracking

**Practical Limit**:
With 1 request per 30 seconds:
- Per Hour: ~120 requests
- Per Day: ~2,880 requests
- Per Month: ~86,400 requests

This is a soft guidance, not a hard quota.

---

## Usage Policies

### Acceptable Use

**Permitted**:
- Commercial applications (free or paid)
- Personal projects
- Research and analysis
- Emergency management systems
- Mobile applications
- Web services aggregating NWS data
- Any lawful purpose

**No Restrictions On**:
- Redistribution of data
- Commercial use
- Modification of data
- Combining with other datasets

### Data Attribution

**Required**: No formal attribution required (public domain)

**Recommended**: Credit NWS as data source
- "Data provided by the National Weather Service"
- Link to https://www.weather.gov

**Legal Status**: US government works are not copyrighted

### Prohibited Use

**Do Not**:
- Exceed rate limits (aggressive polling)
- Impersonate NWS or government authority
- Claim data as your own creation
- Use for malicious purposes
- Deliberately overload the service

**Abuse Definition**:
- Excessive request rates (>1 per 30 seconds sustained)
- Distributed denial of service attacks
- Automated scraping beyond reasonable limits
- Requests designed to circumvent rate limits

---

## Request Limits by Endpoint

All endpoints share the same rate limit (1 req/30 sec), but practical limits differ:

### /alerts/active
- **Limit**: 1 req/30 sec
- **Response Size**: ~100-500 KB (varies by # of active alerts)
- **Recommended Poll Interval**: 60 seconds
- **Use Case**: Real-time monitoring

### /alerts/active/area/{state}
- **Limit**: 1 req/30 sec
- **Response Size**: ~10-100 KB (smaller, filtered)
- **Recommended Poll Interval**: 30-60 seconds
- **Use Case**: State-level monitoring

### /alerts/active/zone/{zoneId}
- **Limit**: 1 req/30 sec
- **Response Size**: ~1-10 KB (smallest)
- **Recommended Poll Interval**: 30 seconds
- **Use Case**: Hyper-local alerts

### /alerts/{id}
- **Limit**: 1 req/30 sec
- **Response Size**: ~1-5 KB
- **Use Case**: On-demand lookups (not polling)

### /alerts/types
- **Limit**: 1 req/30 sec
- **Response Size**: ~5-10 KB
- **Caching**: Cache indefinitely (rarely changes)
- **Use Case**: One-time fetch for UI/validation

---

## Bandwidth Limits

**No Explicit Bandwidth Limit**: Not documented

**Practical Considerations**:
- Typical alert payload: 1-5 KB per alert
- Active alerts nationwide: 50-200 alerts typical
- Response size: 100-500 KB for `/alerts/active`
- With 60-second polling: ~0.5-1 MB per hour

**Compression**:
Responses support gzip compression. Use `Accept-Encoding: gzip` to reduce bandwidth:

```http
GET /alerts/active HTTP/1.1
Host: api.weather.gov
User-Agent: (MyApp, contact@example.com)
Accept-Encoding: gzip
```

---

## Concurrency Limits

**No Documented Limit**: NWS documentation doesn't specify concurrent connection limits

**Recommended Practice**:
- Single persistent HTTP client
- Sequential requests (not parallel)
- Reuse connections (HTTP keep-alive)

**Avoid**:
- Opening multiple simultaneous connections
- Parallel requests to different endpoints (counts toward rate limit)
- Connection pooling beyond 2-3 connections

---

## Caching Policies

### Server-Side Caching

**Cache Headers**: NWS responses include cache control headers

**Example Response Headers**:
```
Expires: Sun, 16 Feb 2026 14:00:00 GMT
Cache-Control: public, max-age=300
```

**Typical TTL**: 5-15 minutes (based on alert update frequency)

### Client-Side Caching

**Recommended**:
- Cache responses locally for 30-60 seconds
- Use `If-Modified-Since` for conditional requests (if supported)
- Store alert IDs to detect changes without re-parsing full payload

**Benefits**:
- Reduces redundant data processing
- Lowers request volume
- Improves app responsiveness

**Example**:
```rust
// Cache structure
struct AlertCache {
    data: Vec<Alert>,
    fetched_at: Instant,
    ttl: Duration,
}

impl AlertCache {
    fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() > self.ttl
    }
}
```

---

## CDN & Geographic Distribution

**CDN**: NWS likely uses CDN for distribution (not explicitly documented)

**Implications**:
- Responses may be cached at edge nodes
- Geographic proximity improves latency
- Cache hit rates reduce load on origin servers

**Redirect Behavior**:
Popular queries may return HTTP 301 redirects to cached versions:
```
GET /alerts/active?area=TX
< 301 Moved Permanently
< Location: https://cached-endpoint.weather.gov/...
```

**Action**: Follow redirects automatically (standard HTTP client behavior)

---

## Burst Limits

**No Documented Burst Allowance**:
Unlike APIs with "burst" limits (e.g., 100 requests in 1 minute, then throttle), NWS appears to enforce strict 1 req/30 sec.

**Testing Observations** (unofficial):
- Small bursts (2-3 requests) may be tolerated
- Sustained bursts quickly trigger 429 responses
- Rate limiter has ~5-second memory

**Safe Practice**: Never burst; maintain steady 30-60 sec interval

---

## IP-Based vs User-Based Limits

**Limiting Factor**: Source IP address

**No Per-User Tracking**:
Since there's no authentication, NWS cannot track per-user usage. All limits are IP-based.

**Shared IP Scenarios**:
1. **Corporate Network**: All employees share one egress IP
   - If 10 employees use weather apps, they share the rate limit
   - Potential for 429 errors if uncoordinated

2. **Cloud Servers**: Multiple services on same cloud IP
   - Kubernetes pods sharing NAT gateway
   - Lambda functions in same region

3. **Mobile Carriers**: CGNAT (Carrier-Grade NAT)
   - Many mobile users share same public IP
   - Less likely to hit limits (distributed timing)

**Mitigation**:
- Implement single aggregator service behind your IP
- Cache responses to serve multiple users
- Use multiple egress IPs if possible (multi-region deployment)

---

## Rate Limit Recovery

### When Rate Limited

**Step 1**: Detect 429 response
```rust
if response.status() == 429 {
    // Rate limited
}
```

**Step 2**: Extract retry-after header (if present)
```rust
let retry_after = response.headers()
    .get("Retry-After")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.parse::<u64>().ok())
    .unwrap_or(5);  // Default 5 seconds
```

**Step 3**: Wait and retry
```rust
tokio::time::sleep(Duration::from_secs(retry_after)).await;
let retry_response = client.fetch_alerts().await?;
```

### Exponential Backoff

For resilient clients:

```rust
let mut backoff = 5;
let max_backoff = 120;

for attempt in 0..5 {
    match client.fetch_alerts().await {
        Ok(data) => return Ok(data),
        Err(e) if e.is_rate_limited() => {
            sleep(Duration::from_secs(backoff)).await;
            backoff = (backoff * 2).min(max_backoff);
        }
        Err(e) => return Err(e),
    }
}
```

---

## Service Level Agreement (SLA)

**SLA**: NONE

As a free public service, NWS provides **no formal SLA**.

**Uptime**: Not guaranteed (but historically high)

**Support**: Best-effort; operational issues reported to nco.ops@noaa.gov

**Downtime**:
- Maintenance windows not publicly announced
- Unexpected outages possible
- No compensation for service interruptions

**Reliability**:
Despite no SLA, NWS API is generally very reliable due to:
- Critical infrastructure status
- Redundant government systems
- Emergency management dependencies

**Monitoring**:
Implement your own monitoring and failover strategies; don't rely on uptime guarantees.

---

## Regional Availability

**Availability**: Global (accessible worldwide)

**Geographic Restrictions**: NONE
- No IP geofencing
- Accessible from any country
- No VPN requirements

**Data Coverage**: US-only
While the API is globally accessible, alert data only covers:
- 50 US states
- US territories (Puerto Rico, Guam, US Virgin Islands, etc.)
- US marine regions

**International Use**:
Developers worldwide can access the API, but data relevance limited to US geography.

---

## Future Tier Plans

**No Paid Tiers Planned**:
As a US government service funded by taxpayers, NWS is unlikely to introduce paid tiers.

**Potential Changes**:
- Stricter rate limits if abuse increases
- API key requirement for tracking (but still free)
- Tiered limits based on use case (emergency vs commercial)

**Monitoring Changes**:
Subscribe to updates via:
- GitHub: https://github.com/weather-gov/api
- NWS announcements: https://www.weather.gov/documentation

---

## Optimization Strategies

To maximize usage within rate limits:

### 1. Geographic Filtering
```
# Instead of nationwide polling
GET /alerts/active  (500 KB, all US)

# Use state filtering
GET /alerts/active/area/TX  (50 KB, one state)
```

**Impact**: 10x smaller payloads, 10x more effective polling within same rate limit

### 2. Intelligent Polling
```rust
// Variable interval based on time of day
let interval = if is_severe_weather_season() && is_afternoon() {
    30  // Poll more frequently during high-risk periods
} else {
    120  // Slower polling during low-risk periods
};
```

**Impact**: Allocates rate limit budget to high-value time windows

### 3. Multi-Region Architecture
Deploy alert fetchers in multiple geographic regions:
- US East (different egress IP)
- US West (different egress IP)
- Europe (different egress IP)

**Impact**: 3x effective rate limit (3 requests per 30 sec across all IPs)

### 4. Shared Caching Layer
If serving multiple users:
```
[100 users] → [Your API Gateway with cache] → [NWS API (1 req/30s)]
```

**Impact**: Serve 100 users with single NWS request stream

---

## Recommended Configuration

For production systems:

```rust
pub struct NwsRateLimiter {
    min_interval: Duration,      // 30 seconds
    recommended_interval: Duration,  // 60 seconds (safe margin)
    last_request: Instant,
}

impl NwsRateLimiter {
    pub fn new() -> Self {
        Self {
            min_interval: Duration::from_secs(30),
            recommended_interval: Duration::from_secs(60),
            last_request: Instant::now(),
        }
    }

    pub async fn wait_if_needed(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < self.recommended_interval {
            let wait = self.recommended_interval - elapsed;
            tokio::time::sleep(wait).await;
        }
        self.last_request = Instant::now();
    }
}
```

**Usage**:
```rust
let mut limiter = NwsRateLimiter::new();
loop {
    limiter.wait_if_needed().await;
    let alerts = fetch_alerts().await?;
    process(alerts);
}
```

This ensures compliance with rate limits while maintaining near-real-time updates.
