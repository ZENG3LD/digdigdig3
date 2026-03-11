# Raydium API Rate Limits

**Research Date**: 2026-01-20

Comprehensive documentation of Raydium API rate limiting for implementing robust connector with proper rate limit handling.

---

## Table of Contents

1. [Overview](#overview)
2. [Official Documentation](#official-documentation)
3. [Rate Limit Detection](#rate-limit-detection)
4. [Best Practices](#best-practices)
5. [Comparison with CEX Rate Limits](#comparison-with-cex-rate-limits)
6. [Alternative Solutions](#alternative-solutions)

---

## Overview

### Critical Information

**Raydium does NOT publicly document specific rate limits.**

From official documentation:
> "Services within Raydium API have a quota so that services are not compromised."

However, **no specific numbers are provided** for:
- Requests per second
- Requests per minute
- Weight system
- Per-endpoint limits
- IP-based vs user-based limits

---

## Official Documentation

### What We Know

**From Raydium Docs**:

1. **Quota System Exists**:
   - "Services within Raydium API have a quota"
   - Implies rate limiting is enforced
   - Purpose: Prevent service disruption

2. **Recommended Practices**:
   - "Limiting API calls and employing some form of caching can help avoid frequency and volume increases"
   - Suggests polling should be minimized
   - Caching is explicitly recommended

3. **API Purpose**:
   - "APIs are for data access and monitoring — not real-time tracking"
   - Not designed for high-frequency polling
   - Real-time data should use gRPC instead

### What We Don't Know

**Undocumented**:
- ❌ Requests per second limit
- ❌ Requests per minute limit
- ❌ Daily request quota
- ❌ Per-endpoint weight system
- ❌ IP-based vs API-key-based tracking (no API keys exist)
- ❌ Burst limits
- ❌ Rate limit reset window
- ❌ Specific error codes for rate limiting
- ❌ Rate limit headers in responses

---

## Rate Limit Detection

### HTTP Status Code

**Expected**: `429 Too Many Requests`

**Standard HTTP behavior**:
```http
HTTP/1.1 429 Too Many Requests
Content-Type: application/json

{
  "id": "req-12345",
  "success": false,
  "error": {
    "code": "TOO_MANY_REQUESTS",
    "message": "Rate limit exceeded"
  }
}
```

**Note**: Exact error format not documented. Above is inferred from standard REST practices.

### No Rate Limit Headers (Assumed)

Unlike centralized exchanges (KuCoin, Binance), Raydium likely does NOT provide rate limit headers:

**Not Expected** (but check responses):
```http
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1737379200
```

**Why**: No authentication system means no user-specific rate tracking. Likely uses simple IP-based throttling.

---

## Best Practices

### 1. Implement Client-Side Caching

**Cache API Responses**:
```rust
use std::time::{Duration, Instant};
use std::collections::HashMap;

struct CachedResponse<T> {
    data: T,
    timestamp: Instant,
    ttl: Duration,
}

struct ApiCache {
    pool_list: Option<CachedResponse<Vec<Pool>>>,
    token_list: Option<CachedResponse<Vec<Token>>>,
    // ... other cached endpoints
}

impl ApiCache {
    fn get_or_fetch<T, F>(
        &mut self,
        cached: &Option<CachedResponse<T>>,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
        T: Clone,
    {
        if let Some(cached) = cached {
            if cached.timestamp.elapsed() < cached.ttl {
                return Ok(cached.data.clone());
            }
        }

        // Cache expired or missing, fetch new data
        fetch_fn()
    }
}
```

**Recommended Cache TTLs**:
- Token list: 1 hour (rarely changes)
- Pool list: 5-10 minutes (relatively stable)
- Pool-specific data: 1-2 minutes
- Token prices: 30-60 seconds
- Swap quotes: No caching (always fetch fresh)

### 2. Avoid Rapid Polling

**Bad** (High Request Rate):
```rust
// DON'T DO THIS
loop {
    let pool = api.get_pool(id).await?;
    println!("TVL: {}", pool.tvl);
    tokio::time::sleep(Duration::from_secs(1)).await; // Every second!
}
```

**Good** (Reasonable Polling):
```rust
// Reasonable polling with caching
loop {
    let pool = api.get_pool_cached(id).await?; // Cache for 1 minute
    println!("TVL: {}", pool.tvl);
    tokio::time::sleep(Duration::from_secs(60)).await; // Every minute
}
```

**Better** (Use WebSocket/gRPC):
```rust
// For real-time updates, use gRPC account subscription
let mut stream = geyser_client.subscribe_account_updates(pool_id).await?;
while let Some(update) = stream.next().await {
    println!("Pool updated: {:?}", update);
}
```

### 3. Use Batch Endpoints

**Instead of multiple requests**:
```rust
// BAD: 3 separate requests
let pool1 = api.get_pool("id1").await?;
let pool2 = api.get_pool("id2").await?;
let pool3 = api.get_pool("id3").await?;
```

**Use batch endpoint**:
```rust
// GOOD: 1 request for multiple pools
let pools = api.get_pools_by_ids("id1,id2,id3").await?;
```

**Batch Endpoints in Raydium**:
- `GET /pools/info/ids?ids=id1,id2,id3` (comma-separated)
- `GET /mint/ids?mints=mint1,mint2,mint3`
- `GET /farms/info/ids?ids=farm1,farm2`

### 4. Implement Exponential Backoff

**On 429 Errors**:
```rust
async fn fetch_with_retry<T, F>(
    mut fetch_fn: F,
    max_retries: u32,
) -> Result<T>
where
    F: FnMut() -> BoxFuture<'static, Result<T>>,
{
    let mut attempt = 0;

    loop {
        match fetch_fn().await {
            Ok(data) => return Ok(data),
            Err(e) if is_rate_limit_error(&e) && attempt < max_retries => {
                let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
                tokio::time::sleep(backoff).await;
                attempt += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_rate_limit_error(error: &Error) -> bool {
    // Check if error is 429 Too Many Requests
    matches!(error, Error::Http { status: 429, .. })
}
```

**Backoff Schedule**:
- Attempt 1: Wait 100ms
- Attempt 2: Wait 200ms
- Attempt 3: Wait 400ms
- Attempt 4: Wait 800ms
- Attempt 5: Wait 1600ms
- Max: Give up or wait longer

### 5. Rate Limiter Implementation

**Client-Side Rate Limiting**:
```rust
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

struct RateLimiter {
    semaphore: Semaphore,
    rate_per_second: u32,
    window: Duration,
    last_reset: Instant,
}

impl RateLimiter {
    fn new(rate_per_second: u32) -> Self {
        Self {
            semaphore: Semaphore::new(rate_per_second as usize),
            rate_per_second,
            window: Duration::from_secs(1),
            last_reset: Instant::now(),
        }
    }

    async fn acquire(&mut self) {
        // Reset counter every second
        if self.last_reset.elapsed() >= self.window {
            self.semaphore = Semaphore::new(self.rate_per_second as usize);
            self.last_reset = Instant::now();
        }

        let _ = self.semaphore.acquire().await;
    }
}

// Usage
let mut limiter = RateLimiter::new(10); // Max 10 req/sec

loop {
    limiter.acquire().await;
    let response = api.get_pool(id).await?;
    // ...
}
```

**Conservative Default**: 10 requests per second (arbitrary, adjust based on observation)

---

## Comparison with CEX Rate Limits

### KuCoin Rate Limits (For Reference)

**KuCoin provides detailed rate limit info**:
- VIP 0: 2,000 requests per 30 seconds (Spot pool)
- VIP 5: 16,000 requests per 30 seconds (Spot pool)
- Weight system: Each endpoint has specific weight
- Headers: `gw-ratelimit-limit`, `gw-ratelimit-remaining`, `gw-ratelimit-reset`

**KuCoin Example**:
```http
HTTP/1.1 200 OK
gw-ratelimit-limit: 16000
gw-ratelimit-remaining: 15998
gw-ratelimit-reset: 25000

{ "code": "200000", "data": { ... } }
```

### Raydium Rate Limits (Unknown)

**Raydium provides NO rate limit info**:
- No documented limits
- No rate limit headers
- No VIP tiers (no authentication)
- No weight system
- No per-endpoint specifications

**Raydium Example** (expected):
```http
HTTP/1.1 200 OK
Content-Type: application/json

{ "id": "req-123", "success": true, "data": { ... } }
```

**Key Difference**: Complete absence of rate limit transparency.

---

## Alternative Solutions

### 1. Use Third-Party RPC Providers

**Providers with Higher Limits**:

**Chainstack**:
- "No limits on request rate" (with subscription)
- Dedicated Solana RPC nodes
- Geyser plugin access
- Pricing: Paid plans

**QuickNode**:
- Higher rate limits on paid tiers
- Yellowstone Geyser marketplace add-on
- Dedicated endpoints

**Triton One**:
- Program-specific real-time streams
- Geyser-fed gRPC
- Higher throughput

**Helius**:
- Enhanced RPC with higher limits
- WebSocket subscriptions
- gRPC support

**Note**: These are Solana RPC providers, not Raydium-specific. They provide blockchain access, not Raydium's REST API.

### 2. Use gRPC for Real-Time Data

**Instead of polling Raydium REST API**:

**Raydium Recommendation**:
> "For real-time pool creation, refer to gRPC example in SDK demo"

**gRPC Advantages**:
- Push-based (no polling)
- Sub-second latency
- Direct blockchain monitoring
- No REST API rate limits
- Filter by program ID

**Example Flow**:
```
1. Subscribe to Raydium program account updates (gRPC)
2. Receive real-time pool creation events
3. Fetch pool details from REST API (one-time)
4. Cache pool data locally
5. Update via gRPC subscription (no polling)
```

**Resources**:
- [Raydium SDK V2 Demo](https://github.com/raydium-io/raydium-sdk-V2-demo)
- [Chainstack Geyser Guide](https://chainstack.com/solana-geyser-raydium-bonk/)
- [Shyft gRPC Network](https://blogs.shyft.to/how-to-stream-and-parse-raydium-transactions-with-shyfts-grpc-network-b16d5b3af249)

### 3. Run Local Solana Node

**Ultimate Solution** (for advanced users):

**Benefits**:
- No external rate limits
- Full blockchain access
- Complete control
- Highest reliability

**Challenges**:
- High hardware requirements (2TB+ disk, 128GB+ RAM)
- Expensive to maintain
- Technical complexity
- Requires Solana expertise

**Use Case**: High-frequency trading bots, market makers

---

## Monitoring and Alerting

### Track Request Metrics

```rust
struct ApiMetrics {
    total_requests: u64,
    successful_requests: u64,
    failed_requests: u64,
    rate_limit_errors: u64,
    last_rate_limit: Option<Instant>,
}

impl ApiMetrics {
    fn record_request(&mut self, result: &Result<Response>) {
        self.total_requests += 1;
        match result {
            Ok(_) => self.successful_requests += 1,
            Err(e) if is_rate_limit_error(e) => {
                self.rate_limit_errors += 1;
                self.last_rate_limit = Some(Instant::now());
            }
            Err(_) => self.failed_requests += 1,
        }
    }

    fn get_error_rate(&self) -> f64 {
        self.failed_requests as f64 / self.total_requests as f64
    }

    fn get_rate_limit_frequency(&self) -> Option<Duration> {
        self.last_rate_limit.map(|t| t.elapsed())
    }
}
```

### Alert Thresholds

**Warning Conditions**:
- Error rate > 5%
- Rate limit hit in last 5 minutes
- Request success rate < 95%

**Action**: Increase caching, reduce polling frequency, switch to gRPC

---

## Implementation Checklist

### Essential Features

- [ ] Client-side caching with configurable TTL
- [ ] Exponential backoff on 429 errors
- [ ] Batch endpoint usage when available
- [ ] Request metrics tracking
- [ ] Configurable request rate limiter (conservative default)
- [ ] Log rate limit errors for monitoring

### Advanced Features

- [ ] Adaptive rate limiting (adjust based on 429 responses)
- [ ] Circuit breaker pattern (stop requests after repeated failures)
- [ ] gRPC fallback for real-time data
- [ ] Multiple endpoint instances (round-robin)
- [ ] Request queue with priority
- [ ] Health check monitoring

### Testing Considerations

- [ ] Test behavior under rate limiting (manually trigger 429s)
- [ ] Verify backoff logic increases delays correctly
- [ ] Confirm cache prevents unnecessary requests
- [ ] Measure actual request rate with metrics
- [ ] Test recovery after rate limit period

---

## Recommendations

### Conservative Approach

**Since rate limits are undocumented**:

1. **Start Conservative**: Assume low limits (e.g., 10 req/sec)
2. **Monitor Closely**: Track 429 errors and adjust
3. **Cache Aggressively**: Most data doesn't change rapidly
4. **Use gRPC**: For any real-time requirements
5. **Respect 429s**: Always backoff, never ignore

### Adaptive Strategy

```rust
struct AdaptiveRateLimiter {
    current_rate: u32,
    min_rate: u32,
    max_rate: u32,
    recent_429s: VecDeque<Instant>,
}

impl AdaptiveRateLimiter {
    fn adjust_rate(&mut self) {
        // Count 429s in last minute
        self.recent_429s.retain(|t| t.elapsed() < Duration::from_secs(60));

        if self.recent_429s.len() > 3 {
            // Too many rate limits, decrease rate
            self.current_rate = (self.current_rate / 2).max(self.min_rate);
        } else if self.recent_429s.is_empty() {
            // No recent rate limits, can increase
            self.current_rate = (self.current_rate + 1).min(self.max_rate);
        }
    }
}
```

**Benefits**:
- Automatically finds optimal rate
- Adapts to API changes
- Minimizes 429 errors
- Maximizes throughput

---

## Summary

### Key Points

1. **No Official Documentation**: Raydium doesn't publish specific rate limits
2. **Quota System Exists**: Rate limiting is enforced, but specifics unknown
3. **No Rate Limit Headers**: Unlike CEX, no headers indicate remaining quota
4. **Best Practice**: Implement conservative client-side rate limiting
5. **Cache Extensively**: API data doesn't update rapidly
6. **Use gRPC**: For real-time data needs instead of polling
7. **Handle 429 Gracefully**: Exponential backoff on rate limit errors

### Comparison with CEX

| Feature | CEX (KuCoin) | DEX (Raydium) |
|---------|--------------|---------------|
| **Documentation** | Detailed limits published | No limits published |
| **Headers** | `gw-ratelimit-*` headers | No headers |
| **Weight System** | Per-endpoint weights | No weight system |
| **VIP Tiers** | Different limits by tier | No tiers (no auth) |
| **Tracking** | User-based (API key) | IP-based (assumed) |
| **Transparency** | Full transparency | No transparency |

### Final Recommendation

**For Raydium Connector**:
- Implement conservative 10 req/sec default
- Add aggressive caching (1-5 min TTLs)
- Use batch endpoints when possible
- Implement exponential backoff
- Monitor 429 errors and adjust
- Document that limits are unknown
- Recommend gRPC for real-time use cases

---

## Sources

Research compiled from the following sources:

- [Raydium API Documentation](https://docs.raydium.io/raydium/for-developers/api)
- [Best Practices to Increase Raydium API Performance](https://vocal.media/trader/best-practices-to-increase-raydium-api-performance)
- [Chainstack Solana Geyser Guide](https://chainstack.com/solana-geyser-raydium-bonk/)
- [Shyft gRPC Network Guide](https://blogs.shyft.to/how-to-stream-and-parse-raydium-transactions-with-shyfts-grpc-network-b16d5b3af249)
- [KuCoin Rate Limits Documentation](https://www.kucoin.com/docs-new/rate-limit) (for comparison)

---

**Document Version**: 1.0
**Last Updated**: 2026-01-20
**Research Completed By**: Claude Code Research Agent
