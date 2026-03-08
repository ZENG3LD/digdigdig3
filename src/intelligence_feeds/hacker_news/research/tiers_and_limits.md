# Hacker News Firebase API - Tiers and Rate Limits

## Service Tiers

**Tier**: Single free public tier
**Cost**: Free
**Access Level**: Full read access to all public data

There are no premium tiers, paid plans, or service level differences. All users have identical access.

## Rate Limits

### Official Policy

According to the official documentation:

> "There is currently no rate limit."

**Source**: https://github.com/HackerNews/API

This means there are no documented hard limits on:
- Requests per second
- Requests per minute
- Requests per hour
- Requests per day
- Concurrent connections

### Practical Limits

While no official rate limit exists, Firebase infrastructure imposes practical constraints:

#### 1. Connection Limits
Firebase may throttle or block excessive connections from a single IP:
- **Suspected Threshold**: ~50-100 concurrent connections per IP
- **Enforcement**: Soft throttling (delays) rather than hard blocks
- **Detection**: Based on connection count, not request rate

#### 2. Fan-Out Prevention
Making thousands of concurrent requests in a short burst may trigger Firebase's DDoS protection:
- **Risk**: Fetching 500 items simultaneously without throttling
- **Mitigation**: Limit concurrent requests to ~10-20

#### 3. Infrastructure Capacity
Firebase/Google infrastructure can handle high load, but excessive abuse could lead to:
- Temporary IP throttling
- Manual investigation by Firebase/YC team
- Future policy changes

## Recommended Best Practices

### Concurrent Request Limiting

**Recommended**: Max 10 concurrent requests
**Acceptable**: Up to 20 concurrent requests
**Risky**: 50+ concurrent requests

**Rust Implementation** (using tokio semaphore):
```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

pub struct HackerNewsClient {
    semaphore: Arc<Semaphore>,
    // ...
}

impl HackerNewsClient {
    pub fn new() -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(10)), // Max 10 concurrent
            // ...
        }
    }

    pub async fn get_item(&self, id: u64) -> Result<Item, Error> {
        let _permit = self.semaphore.acquire().await?;
        // Make request...
    }
}
```

### Polling Intervals

For periodic polling, use reasonable intervals based on data freshness:

| Resource | Update Frequency | Recommended Poll Interval |
|----------|------------------|---------------------------|
| `/topstories.json` | 1-5 minutes | Every 5 minutes |
| `/newstories.json` | 5-30 seconds | Every 30 seconds |
| `/beststories.json` | 5-10 minutes | Every 10 minutes |
| `/askstories.json` | 5-10 minutes | Every 10 minutes |
| `/showstories.json` | 5-10 minutes | Every 10 minutes |
| `/jobstories.json` | 30-60 minutes | Every 30 minutes |
| `/maxitem.json` | 1-10 seconds | Every 10 seconds or stream |
| `/item/{id}.json` | Varies | Cache immutables, poll 60s for scores |
| `/user/{id}.json` | Rarely | Every 10 minutes or on-demand |
| `/updates.json` | 1-5 minutes | Every 5 minutes |

### Caching Strategy

Many HN data fields are immutable and should be cached indefinitely:

**Immutable Fields** (cache forever):
- `id`
- `type`
- `by` (author)
- `time` (creation timestamp)
- `text` (comment/story text)
- `url` (story URL)
- `title`
- `parent`
- `poll`
- `parts`

**Mutable Fields** (cache with TTL or poll):
- `score` (changes as users vote)
- `descendants` (total comments)
- `kids` (child comments)
- `deleted`
- `dead`

**Caching Example**:
```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

pub struct ItemCache {
    immutable: RwLock<HashMap<u64, ImmutableItem>>,
    mutable: RwLock<HashMap<u64, (MutableItem, Instant)>>,
}

impl ItemCache {
    pub async fn get_or_fetch(&self, id: u64, client: &HackerNewsClient) -> Result<Item, Error> {
        // Check cache first
        if let Some(immutable) = self.immutable.read().await.get(&id) {
            if let Some((mutable, cached_at)) = self.mutable.read().await.get(&id) {
                if cached_at.elapsed() < Duration::from_secs(60) {
                    return Ok(merge(immutable, mutable));
                }
            }
        }

        // Fetch from API
        let item = client.get_item(id).await?;
        // Store in cache...
        Ok(item)
    }
}
```

### Burst Handling

When fetching multiple items (e.g., top 30 stories):

**Pattern 1: Sequential with Delay**
```rust
for id in story_ids.iter().take(30) {
    let item = client.get_item(*id).await?;
    tokio::time::sleep(Duration::from_millis(100)).await; // 100ms delay
}
```

**Pattern 2: Concurrent with Semaphore** (Recommended)
```rust
let tasks: Vec<_> = story_ids.iter().take(30)
    .map(|id| client.get_item(*id))
    .collect();

let items = futures::future::join_all(tasks).await; // Semaphore limits concurrency
```

**Pattern 3: Chunked Concurrent**
```rust
for chunk in story_ids.chunks(10) {
    let tasks: Vec<_> = chunk.iter()
        .map(|id| client.get_item(*id))
        .collect();

    let chunk_items = futures::future::join_all(tasks).await;
    tokio::time::sleep(Duration::from_millis(500)).await; // Delay between chunks
}
```

## Timeouts

### Request Timeouts

Recommended timeout settings:

| Request Type | Timeout | Rationale |
|--------------|---------|-----------|
| Story list fetch | 10 seconds | Small JSON arrays, fast response |
| Single item fetch | 5 seconds | Small objects, usually <1s |
| User profile fetch | 5 seconds | Small objects |
| SSE connection | 60 seconds initial | Long-lived stream |

**Rust Configuration**:
```rust
use reqwest;
use std::time::Duration;

let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(5))
    .connect_timeout(Duration::from_secs(3))
    .build()?;
```

### Connection Timeout

- **Connect Timeout**: 3 seconds
- **Read Timeout**: 5 seconds
- **Total Timeout**: 10 seconds (for slow responses)

### SSE Stream Timeout

For SSE streams, use longer timeouts but implement keep-alive detection:

```rust
use tokio::time::{timeout, Duration};

let stream = client.stream();

while let Ok(Some(event)) = timeout(Duration::from_secs(60), stream.next()).await {
    match event {
        SSE::Event(ev) if ev.event_type == "keep-alive" => {
            // Reset timeout, connection alive
        }
        SSE::Event(ev) => {
            // Handle data event
        }
    }
}
// Reconnect if timeout
```

## Error Handling

### HTTP Status Codes

| Status Code | Meaning | Handling |
|-------------|---------|----------|
| 200 OK | Success (even if data is `null`) | Parse response |
| 307 Temporary Redirect | Firebase load balancing | Follow redirect |
| 429 Too Many Requests | Rate limited (hypothetical) | Exponential backoff |
| 500 Internal Server Error | Firebase issue | Retry with backoff |
| 503 Service Unavailable | Firebase down | Retry with long backoff |
| Timeout | Network/slow response | Retry with backoff |

### Retry Strategy

**Exponential Backoff**:
```rust
use tokio::time::{sleep, Duration};

async fn fetch_with_retry<T, F, Fut>(f: F, max_retries: usize) -> Result<T, Error>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    let mut attempt = 0;

    loop {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt >= max_retries => return Err(e),
            Err(_) => {
                attempt += 1;
                let delay = Duration::from_secs(2u64.pow(attempt as u32));
                sleep(delay).await;
            }
        }
    }
}
```

**Backoff Schedule**:
- Attempt 1: Immediate
- Attempt 2: 2s delay
- Attempt 3: 4s delay
- Attempt 4: 8s delay
- Attempt 5: 16s delay
- Max: 30s delay

## Monitoring & Telemetry

### Request Metrics

Track these metrics to avoid hitting undocumented limits:

- **Requests per minute**: Keep below 1000/min for safety
- **Concurrent connections**: Keep below 20
- **Error rate**: Increase backoff if >5% errors
- **Response times**: Alert if p95 > 2 seconds

### Rust Implementation (with tracing):
```rust
use tracing::{info, warn};
use std::sync::atomic::{AtomicU64, Ordering};

pub struct Metrics {
    requests: AtomicU64,
    errors: AtomicU64,
}

impl Metrics {
    pub fn record_request(&self) {
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_error(&self) {
        let errors = self.errors.fetch_add(1, Ordering::Relaxed) + 1;
        let requests = self.requests.load(Ordering::Relaxed);

        if requests > 0 && (errors * 100 / requests) > 5 {
            warn!("Error rate exceeds 5%, consider throttling");
        }
    }
}
```

## Comparison with Other APIs

| API | Rate Limit | Cost | Authentication |
|-----|------------|------|----------------|
| Hacker News | None (officially) | Free | None |
| Reddit | 60 req/min | Free | OAuth |
| Twitter | 300 req/15min | Free tier | API key |
| GitHub | 5000 req/hour | Free | Token |
| Algolia HN Search | 10,000 req/hour | Free | API key |

Hacker News is more permissive than most APIs, but still requires responsible usage.

## Future Considerations

### Potential Future Limits

If abuse becomes widespread, Firebase/YC might introduce:

1. **API Keys**: Required registration with per-key limits (e.g., 10,000 req/hour)
2. **IP Rate Limits**: Hard limits like 100 req/min per IP
3. **Tiered Access**: Free tier (limited) + paid tiers (higher limits)
4. **CAPTCHA**: For browser-based clients

### Stay Informed

Monitor these resources for policy changes:
- **API Docs**: https://github.com/HackerNews/API
- **HN Blog**: https://blog.ycombinator.com
- **Firebase Blog**: https://firebase.blog
- **Support Email**: api@ycombinator.com

## Summary

- **Official Rate Limit**: None currently
- **Practical Limit**: ~10-20 concurrent requests recommended
- **Polling Intervals**: 10s-5min depending on resource freshness
- **Caching**: Cache immutable fields forever, mutable fields for 60s
- **Timeouts**: 5s for item fetches, 10s for lists
- **Retry Strategy**: Exponential backoff (2s, 4s, 8s, 16s, 30s)
- **Monitoring**: Track requests/min, concurrent connections, error rate
- **Cost**: Free, no tiers
- **Future**: Potential for API keys or IP limits if abuse increases

Use the API responsibly, and it will remain freely accessible for all developers.
