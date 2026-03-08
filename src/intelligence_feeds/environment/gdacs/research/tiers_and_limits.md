# GDACS Rate Limits and Service Tiers

## Service Tiers

**GDACS operates a single public tier with no tiered access levels.**

- **Tier**: Public / Free
- **Cost**: Free
- **Registration**: Not required
- **API Key**: Not required
- **Access Level**: Full access to all data

### No Premium Tiers

Unlike commercial APIs, GDACS does not offer:
- Premium subscriptions
- Enhanced rate limits for paid users
- Priority support
- Historical data paywalls
- White-label services

## Rate Limits

### Official Documentation

**No explicit rate limits are documented** in GDACS API documentation as of February 2026.

### Observed Characteristics

Based on system design and RSS feed update frequency:

**Data Update Frequency**:
- RSS feeds: Updated every **6 minutes**
- API endpoints: Assumed similar (**~6 minutes**)
- Event modifications: Variable (seconds to hours after initial detection)

**Recommended Request Limits**:
- **Poll interval**: 5-6 minutes minimum
- **Requests per hour**: 10-12 (once per 5 minutes)
- **Requests per day**: 240-288 (once per 5-6 minutes)
- **Concurrent connections**: 1-2

### Why These Limits?

1. **Data Freshness**: RSS feeds update every 6 minutes, so polling more frequently provides no benefit
2. **Server Load**: Public API without authentication should be used responsibly
3. **Data Characteristics**: Disasters develop over hours/days, not seconds
4. **Community Resource**: Shared infrastructure for humanitarian purposes

## Request Quotas

### No Hard Quotas

GDACS does not implement:
- Daily request caps
- Monthly quotas
- Request count tracking per IP
- API key-based quotas (no API keys exist)

### Soft Limits (Expected Behavior)

While not enforced technically, these limits represent responsible use:

| Use Case | Requests/Hour | Requests/Day | Notes |
|----------|---------------|--------------|-------|
| Real-time monitoring | 10-12 | 240-288 | Poll every 5-6 min |
| Periodic checks | 4-6 | 96-144 | Poll every 10-15 min |
| Historical data fetch | 20-30 | 100-200 | Use pagination, cache results |
| Development/testing | 10-20 | 50-100 | Use mock data when possible |

## Response Size Limits

### Pagination

**Maximum records per request**: 100 events

**Pagination Parameters**:
- `pagesize`: Max 100 (default: 100)
- `pagenumber`: Sequential page number (1, 2, 3...)

**Example**:
```
# First page (events 1-100)
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?pagesize=100&pagenumber=1

# Second page (events 101-200)
https://www.gdacs.org/gdacsapi/api/events/geteventlist/SEARCH?pagesize=100&pagenumber=2
```

### Response Payload Size

**Typical Response Sizes**:
- Single event: ~2-5 KB
- 10 events: ~20-50 KB
- 100 events (max): ~200-500 KB
- Empty response: ~150 bytes

**Gzip Compression**:
- Supported via `Accept-Encoding: gzip` header
- Reduces payload by 60-80%
- Recommended for all requests

## Timeout Limits

### No Documented Timeouts

GDACS API does not specify server-side timeouts.

### Recommended Client Timeouts

```rust
use std::time::Duration;

pub struct TimeoutConfig {
    pub connect_timeout: Duration,    // 10 seconds
    pub request_timeout: Duration,    // 30 seconds
    pub idle_timeout: Duration,       // 60 seconds
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            connect_timeout: Duration::from_secs(10),
            request_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(60),
        }
    }
}
```

**Rationale**:
- **Connect timeout**: GDACS servers usually respond quickly (<5s)
- **Request timeout**: Large payloads (100 events) may take 10-20s
- **Idle timeout**: Keep-alive connections for efficient polling

## Error Responses

### HTTP Status Codes

| Status Code | Meaning | Action |
|-------------|---------|--------|
| 200 OK | Success | Process response |
| 400 Bad Request | Invalid parameters | Fix query params |
| 404 Not Found | Invalid endpoint | Check URL |
| 500 Internal Server Error | Server error | Retry with backoff |
| 503 Service Unavailable | Maintenance/overload | Retry after delay |

### No Rate Limit Errors

**Notably absent**:
- `429 Too Many Requests`: Not implemented
- `403 Forbidden`: Not used for rate limiting
- `Retry-After` header: Not provided

**Implication**: API does not actively enforce rate limits, relying on user responsibility.

## Retry Strategy

### Exponential Backoff

```rust
use std::time::Duration;

pub struct RetryConfig {
    pub max_retries: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
        }
    }
}

pub async fn fetch_with_retry(
    client: &GdacsClient,
    config: &RetryConfig,
) -> Result<EventList, GdacsError> {
    let mut delay = config.initial_delay;

    for attempt in 0..=config.max_retries {
        match client.get_events().await {
            Ok(events) => return Ok(events),
            Err(e) if attempt < config.max_retries && e.is_retryable() => {
                tracing::warn!(
                    attempt = attempt + 1,
                    max_retries = config.max_retries,
                    retry_after = ?delay,
                    error = ?e,
                    "Request failed, retrying"
                );

                tokio::time::sleep(delay).await;
                delay = (delay.mul_f64(config.multiplier)).min(config.max_delay);
            }
            Err(e) => return Err(e),
        }
    }

    Err(GdacsError::MaxRetriesExceeded)
}
```

### Retryable vs. Non-Retryable Errors

**Retry**:
- `500 Internal Server Error`
- `503 Service Unavailable`
- Network timeouts
- Connection errors
- DNS resolution failures

**Don't Retry**:
- `400 Bad Request` (fix parameters)
- `404 Not Found` (wrong endpoint)
- JSON parsing errors (API change)
- Invalid response structure

## Caching Strategy

### Why Cache?

1. **Reduce API load**: Same data requested repeatedly
2. **Improve response time**: Local data instant
3. **Offline capability**: Continue working during network issues
4. **Cost efficiency**: Public resource, be considerate

### Cache TTL (Time-To-Live)

**Recommended TTL by Data Type**:

| Data Type | TTL | Rationale |
|-----------|-----|-----------|
| Current events (last 24h) | 5 minutes | Match RSS update frequency |
| Recent events (7 days) | 15 minutes | Data stabilizes over time |
| Historical events (>7 days) | 24 hours | Rarely changes |
| Event details | 15 minutes | May be updated as situation evolves |
| Event geometry | 1 hour | Forecast tracks may change (TC) |

### Cache Implementation

```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct EventCache {
    cache: RwLock<HashMap<CacheKey, CacheEntry>>,
    ttl: Duration,
}

#[derive(Hash, Eq, PartialEq, Clone)]
pub struct CacheKey {
    endpoint: String,
    params: String, // Serialized query params
}

pub struct CacheEntry {
    data: EventList,
    timestamp: Instant,
}

impl EventCache {
    pub fn new(ttl: Duration) -> Self {
        Self {
            cache: RwLock::new(HashMap::new()),
            ttl,
        }
    }

    pub async fn get_or_fetch<F, Fut>(
        &self,
        key: CacheKey,
        fetch_fn: F,
    ) -> Result<EventList, GdacsError>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<EventList, GdacsError>>,
    {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&key) {
                if entry.timestamp.elapsed() < self.ttl {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Fetch fresh data
        let data = fetch_fn().await?;

        // Update cache
        {
            let mut cache = self.cache.write().await;
            cache.insert(key, CacheEntry {
                data: data.clone(),
                timestamp: Instant::now(),
            });
        }

        Ok(data)
    }

    pub async fn invalidate(&self, key: &CacheKey) {
        let mut cache = self.cache.write().await;
        cache.remove(key);
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }
}
```

## Bandwidth Considerations

### Data Transfer

**Per Request**:
- Typical: 20-50 KB (10 events)
- Maximum: 200-500 KB (100 events)
- With gzip: 40-150 KB (100 events)

**Per Day** (5-minute polling):
- Requests: 288
- Uncompressed: 5.76-14.4 MB
- Compressed: 2.3-8.6 MB

**Per Month**:
- Requests: 8,640
- Uncompressed: 172-432 MB
- Compressed: 69-259 MB

### Optimization

1. **Use gzip compression**: `Accept-Encoding: gzip`
2. **Filter by alert level**: `alertlevel=orange;red` (exclude Green)
3. **Filter by disaster type**: `eventlist=EQ,TC` (only what you need)
4. **Use smaller page sizes**: `pagesize=50` if you don't need 100
5. **Implement caching**: Reduce duplicate requests

## Monitoring and Compliance

### Metrics to Track

```rust
pub struct ApiMetrics {
    pub requests_total: u64,
    pub requests_successful: u64,
    pub requests_failed: u64,
    pub bytes_transferred: u64,
    pub average_response_time: Duration,
    pub last_request_timestamp: Instant,
}

impl ApiMetrics {
    pub fn requests_per_hour(&self) -> f64 {
        // Calculate based on time window
    }

    pub fn is_within_limits(&self, max_requests_per_hour: u64) -> bool {
        self.requests_per_hour() <= max_requests_per_hour as f64
    }
}
```

### Logging

```rust
tracing::info!(
    response_time_ms = ?response_time.as_millis(),
    status_code = response.status().as_u16(),
    payload_size = response_body.len(),
    cached = false,
    "GDACS API request completed"
);

tracing::warn!(
    requests_per_hour = metrics.requests_per_hour(),
    recommended_limit = 12,
    "Request rate exceeding recommended limits"
);
```

## Best Practices

### Do's
- ✅ Poll every 5-6 minutes (match data update frequency)
- ✅ Implement caching (5-15 minute TTL)
- ✅ Use gzip compression
- ✅ Filter by alert level and disaster type
- ✅ Handle errors gracefully with exponential backoff
- ✅ Use pagination for large historical queries
- ✅ Set reasonable client timeouts
- ✅ Log request metrics

### Don'ts
- ❌ Poll more frequently than 5 minutes
- ❌ Make hundreds of requests per hour
- ❌ Fetch all historical data without pagination
- ❌ Ignore retry strategies
- ❌ Skip caching for repeated queries
- ❌ Use the API for high-frequency trading signals
- ❌ Abuse the public resource

## Future Changes

### If GDACS Implements Rate Limiting

**Possible Changes**:
1. **HTTP 429 responses**: With `Retry-After` header
2. **API key requirement**: For tracking and quotas
3. **Tiered access**: Free vs. paid with different limits
4. **Documented quotas**: Explicit requests per hour/day

**Preparation**:
```rust
pub fn handle_rate_limit_response(response: &Response) -> Result<(), GdacsError> {
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        let retry_after = response
            .headers()
            .get("Retry-After")
            .and_then(|v| v.to_str().ok())
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(60); // Default to 60 seconds

        return Err(GdacsError::RateLimitExceeded { retry_after });
    }
    Ok(())
}
```

## Summary

- **No authentication** = No tiered access or quotas
- **No documented rate limits** = Use responsibly (5-6 min polling)
- **100 events max per request** = Use pagination for historical data
- **6-minute data updates** = No benefit to faster polling
- **Caching essential** = Reduce load, improve performance
- **Monitor your usage** = Track requests per hour, implement self-imposed limits
- **Public resource** = Be considerate of humanitarian mission
