# Jupiter API Rate Limits

## Overview

Jupiter uses a comprehensive rate limiting system with different tiers, specialized buckets, and both fixed and dynamic rate limiting mechanisms.

---

## Rate Limiting Systems

Jupiter offers two distinct rate limiting approaches:

### 1. Fixed Rate Limit System
Used by Free and Pro tiers with predetermined request limits per time window.

### 2. Dynamic Rate Limit System
Used by Ultra tier (BETA) where limits scale based on executed swap volume.

---

## Fixed Rate Limit Tiers

### Free Tier

**Rate Limit:**
```
60 requests per 60 seconds (1 request/second average)
```

**Base URL:**
```
https://api.jup.ag/
```

**Window:** 60-second sliding window

**Cost:** Free

**Features:**
- All API endpoints accessible
- Same data freshness as paid tiers
- Requires API key
- Good for development and testing

**Use Cases:**
- Development
- Testing
- Low-volume applications
- Personal projects

---

### Pro Tier Plans

**Base URL:**
```
https://api.jup.ag/
```

**Window:** 10-second sliding window

**Available Plans:**

| Tier | Requests/Window | Avg RPS | Monthly Cost | Use Case |
|------|----------------|---------|--------------|----------|
| Pro I | 100 | 10 | $ | Small applications |
| Pro II | 500 | 50 | $$ | Medium applications |
| Pro III | 1,000 | 100 | $$$ | Large applications |
| Pro IV | 5,000 | 500 | $$$$ | High-volume applications |

**Calculation:**
- 100 requests / 10 seconds = 10 requests per second average
- 500 requests / 10 seconds = 50 requests per second average
- etc.

**Payment Options:**
1. **Helio**: USDC on Solana (manual renewal)
2. **Coinflow**: Credit card (automatic subscription)

**Features:**
- Higher rate limits than Free
- Same data and endpoints as Free
- Priority support
- No functional differences (just higher limits)

---

## Dynamic Rate Limit (Ultra Tier)

### Overview

**Status:** BETA

**Base URL:**
```
https://api.jup.ag/ultra/
```

**Mechanism:** Rate limit scales with your 24-hour executed swap volume

**Update Frequency:** Every 10 minutes

**Window:** Based on rolling 24-hour period

---

### Volume-Based Scaling

Rate limits adjust automatically based on swap volume:

| 24h Swap Volume (USD) | Rate Limit (requests/10s) | Avg RPS |
|----------------------|---------------------------|---------|
| $0 | 50 (base) | 5 |
| $10,000 | 51 | 5.1 |
| $100,000 | 61 | 6.1 |
| $1,000,000 | 165 | 16.5 |
| $10,000,000+ | Higher (scales further) | 50+ |

**Formula:** Appears to be logarithmic scaling
```
rate_limit = base_limit + log_scale(swap_volume)
```

**Example:**
- Start with base limit: 50 requests/10s
- Execute $500,000 in swaps over 24h
- Limit increases to ~110 requests/10s
- Limit updates every 10 minutes

---

### Ultra Tier Benefits

1. **No RPC Required**: Jupiter handles transaction submission
2. **End-to-End Execution**: API submits transactions on your behalf
3. **Auto-Scaling Limits**: Heavy users get higher limits automatically
4. **Transaction Optimization**: Jupiter optimizes priority fees and execution
5. **Simplified Integration**: No need to manage Solana RPC connections

---

## Rate Limit Buckets

### Independent Buckets

Requests are distributed across **three independent buckets**, each with its own limits:

#### 1. Price API Bucket

**Endpoints:**
```
/price/v3/*
```

**Limits:** Separate from main bucket (exact limits vary by tier)

**Purpose:** Price queries don't affect swap API rate limits

---

#### 2. Studio API Bucket

**Endpoints:**
```
/studio/*
```

**Limits:**

| Tier | Rate Limit | Window |
|------|-----------|--------|
| Pro | 10 requests | 10 seconds |
| Free | 100 requests | 5 minutes |

**Purpose:** Jupiter Studio token launch and tracking data

---

#### 3. Default Bucket

**Endpoints:** All other APIs
```
/swap/v1/*
/tokens/v2/*
/quote/*
```

**Limits:** As per tier (60/min for Free, 100-5000/10s for Pro, dynamic for Ultra)

**Purpose:** Main API operations (swaps, quotes, token data)

---

## Sliding Window Enforcement

### How It Works

Jupiter uses a **sliding window** rate limit algorithm:

**Not Token Bucket:**
- NOT a refilling bucket
- Does NOT average over time
- Does NOT allow bursts beyond the limit

**Sliding Window:**
- Tracks requests in last N seconds
- Any request beyond limit is rejected
- Window slides continuously

**Example (100 requests / 10 seconds):**
```
Time: 00:00:00 - Make 100 requests ✓
Time: 00:00:01 - Make 1 request ✗ (still 100 in last 10s)
Time: 00:00:02 - Make 1 request ✗ (still 100 in last 10s)
...
Time: 00:00:10 - Make 100 requests ✓ (previous 100 expired)
```

**Key Point:** You cannot burst more than your limit, even if you were idle before.

---

## Rate Limit Headers

### Response Headers

Every API response includes rate limit information:

```http
HTTP/1.1 200 OK
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 95
X-RateLimit-Reset: 1640000010
Content-Type: application/json
```

**Header Descriptions:**

| Header | Type | Description |
|--------|------|-------------|
| `X-RateLimit-Limit` | integer | Total requests allowed in current window |
| `X-RateLimit-Remaining` | integer | Requests remaining in current window |
| `X-RateLimit-Reset` | integer | Unix timestamp when window resets |

**Calculating Time Until Reset:**
```javascript
const now = Math.floor(Date.now() / 1000);
const resetTime = parseInt(response.headers['X-RateLimit-Reset']);
const secondsUntilReset = resetTime - now;
```

---

## 429 Rate Limited Response

### Error Response

When rate limit is exceeded:

**Status Code:** `429 Too Many Requests`

**Response Body:**
```json
{
  "error": "Rate limit exceeded. Please slow down requests or upgrade your plan."
}
```

**Headers:**
```http
HTTP/1.1 429 Too Many Requests
X-RateLimit-Limit: 100
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1640000020
Retry-After: 10
Content-Type: application/json
```

**Retry-After:** Seconds to wait before retrying

---

## Handling Rate Limits

### Best Practices

#### 1. Exponential Backoff

Implement exponential backoff when receiving 429:

```rust
use std::time::Duration;
use tokio::time::sleep;

pub async fn request_with_retry<F, T>(
    mut request_fn: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: FnMut() -> futures::future::BoxFuture<'static, Result<T, Error>>,
{
    let mut retry_count = 0;
    let base_delay = Duration::from_millis(100);

    loop {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(e) if is_rate_limit_error(&e) && retry_count < max_retries => {
                let delay = base_delay * 2u32.pow(retry_count);
                retry_count += 1;

                eprintln!(
                    "Rate limited. Retry {}/{} after {}ms",
                    retry_count,
                    max_retries,
                    delay.as_millis()
                );

                sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_rate_limit_error(error: &Error) -> bool {
    matches!(error, Error::RateLimited)
}
```

#### 2. Request Queue

Implement a request queue with rate limiting:

```rust
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

pub struct RateLimiter {
    semaphore: Semaphore,
    rate_limit: u32,
    window: Duration,
    last_reset: std::sync::Mutex<Instant>,
}

impl RateLimiter {
    pub fn new(rate_limit: u32, window: Duration) -> Self {
        Self {
            semaphore: Semaphore::new(rate_limit as usize),
            rate_limit,
            window,
            last_reset: std::sync::Mutex::new(Instant::now()),
        }
    }

    pub async fn acquire(&self) {
        // Reset if window expired
        {
            let mut last_reset = self.last_reset.lock().unwrap();
            if last_reset.elapsed() >= self.window {
                *last_reset = Instant::now();
                self.semaphore.add_permits(self.rate_limit as usize);
            }
        }

        // Acquire permit
        let _permit = self.semaphore.acquire().await.unwrap();
    }
}

// Usage
let limiter = RateLimiter::new(100, Duration::from_secs(10));

for _ in 0..200 {
    limiter.acquire().await;
    make_api_request().await?;
}
```

#### 3. Monitor Headers

Always check rate limit headers:

```rust
pub fn check_rate_limit(headers: &HeaderMap) -> RateLimitInfo {
    let limit = headers
        .get("X-RateLimit-Limit")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let remaining = headers
        .get("X-RateLimit-Remaining")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(0);

    let reset = headers
        .get("X-RateLimit-Reset")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    RateLimitInfo {
        limit,
        remaining,
        reset_at: reset,
    }
}

pub struct RateLimitInfo {
    pub limit: u32,
    pub remaining: u32,
    pub reset_at: i64,
}

impl RateLimitInfo {
    pub fn seconds_until_reset(&self) -> i64 {
        self.reset_at - chrono::Utc::now().timestamp()
    }

    pub fn usage_percent(&self) -> f64 {
        if self.limit == 0 {
            return 0.0;
        }
        ((self.limit - self.remaining) as f64 / self.limit as f64) * 100.0
    }
}
```

#### 4. Respect Retry-After

When 429 occurs, respect the `Retry-After` header:

```rust
pub async fn handle_rate_limit(response: &Response) -> Result<(), Error> {
    if response.status() == StatusCode::TOO_MANY_REQUESTS {
        if let Some(retry_after) = response.headers().get("Retry-After") {
            if let Ok(seconds) = retry_after.to_str()?.parse::<u64>() {
                tokio::time::sleep(Duration::from_secs(seconds)).await;
                return Ok(());
            }
        }

        // Fallback to default wait
        tokio::time::sleep(Duration::from_secs(10)).await;
    }
    Ok(())
}
```

---

## Rate Limit Strategies

### 1. Request Batching

Batch multiple operations into single requests where possible:

**Instead of:**
```rust
for mint in mints {
    get_price(mint).await?;  // 100 requests
}
```

**Do:**
```rust
// Single request for up to 50 tokens
get_prices(&mints[0..50]).await?;
get_prices(&mints[50..100]).await?;
```

### 2. Caching

Cache responses to reduce API calls:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub struct PriceCache {
    cache: Arc<RwLock<HashMap<String, CachedPrice>>>,
    ttl: Duration,
}

struct CachedPrice {
    price: f64,
    cached_at: Instant,
}

impl PriceCache {
    pub async fn get_or_fetch(
        &self,
        mint: &str,
        fetch_fn: impl Future<Output = Result<f64, Error>>,
    ) -> Result<f64, Error> {
        // Check cache
        {
            let cache = self.cache.read().await;
            if let Some(cached) = cache.get(mint) {
                if cached.cached_at.elapsed() < self.ttl {
                    return Ok(cached.price);
                }
            }
        }

        // Fetch and cache
        let price = fetch_fn.await?;
        {
            let mut cache = self.cache.write().await;
            cache.insert(
                mint.to_string(),
                CachedPrice {
                    price,
                    cached_at: Instant::now(),
                },
            );
        }

        Ok(price)
    }
}
```

### 3. Request Prioritization

Prioritize critical requests:

```rust
pub enum RequestPriority {
    Critical,   // User-initiated swaps
    High,       // Price updates
    Medium,     // Token discovery
    Low,        // Background analytics
}

pub struct PriorityQueue {
    critical: VecDeque<Request>,
    high: VecDeque<Request>,
    medium: VecDeque<Request>,
    low: VecDeque<Request>,
}

impl PriorityQueue {
    pub fn next(&mut self) -> Option<Request> {
        self.critical.pop_front()
            .or_else(|| self.high.pop_front())
            .or_else(|| self.medium.pop_front())
            .or_else(|| self.low.pop_front())
    }
}
```

---

## Per-Account Limits

### Important: Not Per-Key

**Rate limits apply to your ACCOUNT, not individual API keys.**

**Implication:**
- Generating multiple API keys does NOT increase rate limit
- All keys under one account share the same limit pool
- Requests from different keys count toward the same limit

**Example:**
```
Account: user@example.com
- API Key 1: abc123...
- API Key 2: def456...
- API Key 3: ghi789...

Rate Limit: 100 requests / 10 seconds (SHARED across all keys)

Key 1 makes 60 requests ✓
Key 2 makes 30 requests ✓
Key 3 makes 11 requests ✗ (exceeds account limit)
```

---

## Upgrading Tiers

### When to Upgrade

Consider upgrading when:

1. **Frequent 429 Errors**: Regularly hitting rate limits
2. **Growing Usage**: Application traffic increasing
3. **Real-Time Needs**: Need faster update frequencies
4. **Multiple Users**: Serving many concurrent users
5. **High Volume Trading**: Executing many swaps

### Tier Selection Guide

| Use Case | Recommended Tier | Requests/10s |
|----------|------------------|--------------|
| Development/Testing | Free | 10 |
| Small dApp (<100 users) | Pro I | 100 |
| Medium dApp (100-1000 users) | Pro II | 500 |
| Large dApp (1000-10000 users) | Pro III | 1,000 |
| High-Volume Exchange | Pro IV | 5,000 |
| Trading Bot (high volume) | Ultra | Dynamic (50-500+) |

### Ultra vs Pro IV

**Choose Ultra if:**
- High swap volume ($1M+ daily)
- Need transaction execution
- Don't want to manage RPC
- Want auto-scaling limits

**Choose Pro IV if:**
- Need maximum fixed limit
- Want to control transaction submission
- Using custom RPC setup
- Predictable rate limit preferred

---

## Monitoring Usage

### Portal Dashboard

Access https://portal.jup.ag to monitor:
- Current rate limit tier
- Request usage (hourly, daily, monthly)
- Rate limit violations
- API key management
- Upgrade options

### Programmatic Monitoring

```rust
pub struct UsageTracker {
    requests_made: AtomicU32,
    window_start: Mutex<Instant>,
}

impl UsageTracker {
    pub fn record_request(&self, limit_info: &RateLimitInfo) {
        self.requests_made.fetch_add(1, Ordering::Relaxed);

        let usage = limit_info.usage_percent();
        if usage > 80.0 {
            warn!("Rate limit usage: {:.1}%", usage);
        }

        if limit_info.remaining < 10 {
            warn!("Only {} requests remaining", limit_info.remaining);
        }
    }
}
```

---

## Common Rate Limit Issues

### Issue 1: Burst Traffic

**Problem:** Application makes many requests at once

**Solution:**
- Implement request queue
- Spread requests over time
- Use batch endpoints

### Issue 2: Polling Too Frequently

**Problem:** Polling for price updates every second

**Solution:**
- Reduce polling frequency
- Use caching with TTL
- Consider WebSocket alternatives (via third parties)

### Issue 3: No Backoff on Errors

**Problem:** Retry immediately on 429

**Solution:**
- Implement exponential backoff
- Respect Retry-After header
- Add jitter to retries

---

## Notes

1. **Sliding Window**: Rate limits use sliding windows, not token buckets
2. **Per-Account**: Limits apply per account, not per API key
3. **Independent Buckets**: Price, Studio, and default APIs have separate limits
4. **Dynamic Scaling**: Ultra tier scales with swap volume
5. **Same Data**: All tiers access same data; only limits differ
6. **Free Tier Available**: No cost to start
7. **Upgrade Anytime**: Can upgrade/downgrade tiers as needed
8. **No Overage Charges**: Requests are blocked, not charged
9. **Monitor Headers**: Always check rate limit headers
10. **Plan Ahead**: Choose tier based on expected peak usage

---

## Support

For rate limit issues:
- Documentation: https://dev.jup.ag/docs/api-rate-limit
- Portal: https://portal.jup.ag
- Discord: https://discord.gg/jup
