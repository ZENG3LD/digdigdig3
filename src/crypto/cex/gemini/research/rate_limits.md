# Gemini Exchange API Rate Limits

Complete rate limit specification for implementing V5 connector with proper throttling and error handling.

---

## Overview

Gemini enforces rate limits to protect system performance and ensure fair usage. Different limits apply to public vs private endpoints and REST vs WebSocket APIs.

---

## REST API Rate Limits

### Public Endpoints

**Limit**: **120 requests per minute**
**Recommendation**: Do not exceed **1 request per second**

**Affected Endpoints**:
- `GET /v1/symbols`
- `GET /v1/symbols/details/{symbol}`
- `GET /v1/pubticker/{symbol}`
- `GET /v2/ticker/{symbol}`
- `GET /v1/book/{symbol}`
- `GET /v1/trades/{symbol}`
- `GET /v1/pricefeed`
- `GET /v2/candles/{symbol}/{time_frame}`
- `GET /v2/derivatives/candles/{symbol}/{time_frame}`
- `GET /v1/fundingamount/{symbol}`
- `GET /v1/feepromos`
- `GET /v1/riskstats/{symbol}`

**Calculation**:
- 120 requests / 60 seconds = 2 requests per second maximum
- Recommended: 1 request per second for safety margin

### Private Endpoints

**Limit**: **600 requests per minute**
**Recommendation**: Do not exceed **5 requests per second**

**Affected Endpoints**: All authenticated endpoints
- Order management (`/v1/order/*`)
- Account info (`/v1/balances`, `/v1/account`)
- Positions (`/v1/positions`, `/v1/margin`)
- Fund management (`/v1/withdraw/*`, `/v1/transfers`)
- Trading history (`/v1/mytrades`, `/v1/tradevolume`)

**Calculation**:
- 600 requests / 60 seconds = 10 requests per second maximum
- Recommended: 5 requests per second for safety margin

---

## Rate Limit Enforcement

### Sliding Window

Gemini uses a **sliding window** approach, not a fixed-time bucket.

- Limits apply to requests in the **previous 60 seconds**
- Each request moves the window forward
- Not reset at specific intervals (e.g., top of the minute)

**Example**:
```
Time  12:00:00 - Request 1
Time  12:00:01 - Request 2
...
Time  12:00:59 - Request 120
Time  12:01:00 - Request 121 is allowed (Request 1 fell out of window)
```

### Burst Allowance

Gemini provides a **burst allowance of 5 additional requests**:

1. **First 10 requests** (for 600/min limit): Processed immediately
2. **Next 5 requests**: **Queued** and processed when rate drops
3. **Requests 16+**: Receive **429 Too Many Requests** error

**Example Scenario** (600 req/min limit):
```
Send 20 rapid requests:
- Requests 1-10: Processed immediately
- Requests 11-15: Queued (delayed processing)
- Requests 16-20: Rejected with 429 error
```

**Key Points**:
- Burst allows brief spikes above the limit
- Queued requests will process when capacity available
- Beyond burst capacity: immediate 429 rejection

---

## Rate Limit Errors

### HTTP Status Code

**429 Too Many Requests**

### Response Format

```json
{
  "result": "error",
  "reason": "RateLimitExceeded",
  "message": "Too many requests. Please try again later."
}
```

### Error Fields

- `result`: "error"
- `reason`: "RateLimitExceeded"
- `message`: Human-readable description

### When You'll See 429

1. **Exceeding base limit**: More than 120 (public) or 600 (private) per minute
2. **Exceeding burst capacity**: More than 5 queued requests
3. **Sustained high rate**: Consistently at or above recommended rate

---

## WebSocket Rate Limits

### Public Market Data WebSocket

**Connection Limit**: Not explicitly documented

**Subscription Rate**: **1 request per symbol per minute** (recommended)

**Affected Streams**:
- `wss://api.gemini.com/v2/marketdata`
- Subscriptions: `l2`, `candles_*`, `trades`

**Notes**:
- Initial subscription can include multiple symbols
- Subsequent subscription changes limited to 1 per symbol per minute
- Heartbeats and data messages do not count toward limit

### Private Order Events WebSocket

**Connection Limit**: Not explicitly documented

**Message Rate**: Not explicitly limited (events are pushed, not pulled)

**Stream**:
- `wss://api.gemini.com/v1/order/events`

**Notes**:
- Receive-only (no subscription management after connection)
- No request rate limit (you receive events as they occur)
- Heartbeat every 5 seconds (doesn't count as rate-limited request)

---

## Implementation Strategy

### Rate Limiter Design

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct RateLimiter {
    requests_per_minute: u32,
    max_requests_per_second: f64,
    last_request: Option<Instant>,
    request_history: Vec<Instant>,
}

impl RateLimiter {
    pub fn new_public() -> Self {
        Self {
            requests_per_minute: 120,
            max_requests_per_second: 1.0,
            last_request: None,
            request_history: Vec::new(),
        }
    }

    pub fn new_private() -> Self {
        Self {
            requests_per_minute: 600,
            max_requests_per_second: 5.0,
            last_request: None,
            request_history: Vec::new(),
        }
    }

    /// Wait if necessary to respect rate limits
    pub async fn throttle(&mut self) {
        let now = Instant::now();

        // Remove requests older than 1 minute
        self.request_history.retain(|&time| {
            now.duration_since(time) < Duration::from_secs(60)
        });

        // Check per-minute limit
        if self.request_history.len() >= self.requests_per_minute as usize {
            // Wait until oldest request falls out of window
            let oldest = self.request_history[0];
            let wait_time = Duration::from_secs(60)
                .saturating_sub(now.duration_since(oldest));

            if !wait_time.is_zero() {
                sleep(wait_time).await;
            }

            // Clean up again after waiting
            let now = Instant::now();
            self.request_history.retain(|&time| {
                now.duration_since(time) < Duration::from_secs(60)
            });
        }

        // Check per-second limit
        if let Some(last) = self.last_request {
            let min_interval = Duration::from_secs_f64(1.0 / self.max_requests_per_second);
            let elapsed = now.duration_since(last);

            if elapsed < min_interval {
                sleep(min_interval - elapsed).await;
            }
        }

        // Record this request
        let now = Instant::now();
        self.last_request = Some(now);
        self.request_history.push(now);
    }

    /// Check if request would exceed limits (without waiting)
    pub fn would_exceed_limit(&self) -> bool {
        let now = Instant::now();

        // Count requests in last minute
        let recent_count = self.request_history.iter()
            .filter(|&&time| now.duration_since(time) < Duration::from_secs(60))
            .count();

        recent_count >= self.requests_per_minute as usize
    }
}
```

### Usage Example

```rust
pub struct GeminiConnector {
    client: Client,
    public_limiter: RateLimiter,
    private_limiter: RateLimiter,
}

impl GeminiConnector {
    pub async fn get_ticker(&mut self, symbol: &str) -> Result<Ticker, Error> {
        // Wait if necessary
        self.public_limiter.throttle().await;

        // Make request
        let url = format!("{}/v1/pubticker/{}", self.base_url, symbol);
        let response = self.client.get(&url).send().await?;

        // Handle 429 with retry
        if response.status() == 429 {
            return self.handle_rate_limit_error().await;
        }

        response.json().await
    }

    pub async fn create_order(&mut self, /* params */) -> Result<Order, Error> {
        // Wait if necessary
        self.private_limiter.throttle().await;

        // Make authenticated request
        // ...
    }
}
```

---

## Retry Strategy

### Exponential Backoff

When receiving a 429 error, implement exponential backoff:

```rust
use tokio::time::{sleep, Duration};

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
            max_delay: Duration::from_secs(30),
            multiplier: 2.0,
        }
    }
}

pub async fn retry_with_backoff<F, T, E>(
    config: &RetryConfig,
    mut f: F,
) -> Result<T, E>
where
    F: FnMut() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, E>>>>,
    E: std::fmt::Debug,
{
    let mut delay = config.initial_delay;

    for attempt in 0..=config.max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < config.max_retries => {
                eprintln!("Attempt {} failed: {:?}, retrying in {:?}", attempt + 1, e, delay);
                sleep(delay).await;
                delay = std::cmp::min(
                    Duration::from_secs_f64(delay.as_secs_f64() * config.multiplier),
                    config.max_delay,
                );
            }
            Err(e) => return Err(e),
        }
    }

    unreachable!()
}
```

### Recommended Backoff Schedule

| Attempt | Delay |
|---------|-------|
| 1st retry | 1 second |
| 2nd retry | 2 seconds |
| 3rd retry | 4 seconds |
| 4th retry | 8 seconds |
| 5th retry | 16 seconds |
| 6+ retry | 30 seconds (max) |

---

## Best Practices

### 1. Separate Limiters for Public/Private

```rust
pub struct GeminiConnector {
    public_limiter: RateLimiter,
    private_limiter: RateLimiter,
}
```

**Reason**: Public and private have different limits; don't let one affect the other.

### 2. Pre-check Before Critical Operations

```rust
if self.private_limiter.would_exceed_limit() {
    return Err(Error::RateLimitWouldExceed);
}

self.private_limiter.throttle().await;
// Critical order operation
```

### 3. Batch Requests When Possible

Instead of:
```rust
for symbol in symbols {
    get_ticker(symbol).await?;
}
```

Use multi-symbol endpoints when available:
```rust
get_all_tickers().await?; // Single request
```

### 4. Cache Frequently-Accessed Data

```rust
use std::time::{Duration, Instant};

pub struct CachedData<T> {
    data: Option<T>,
    last_updated: Option<Instant>,
    ttl: Duration,
}

impl<T: Clone> CachedData<T> {
    pub fn new(ttl: Duration) -> Self {
        Self {
            data: None,
            last_updated: None,
            ttl,
        }
    }

    pub fn get(&self) -> Option<T> {
        if let (Some(data), Some(last)) = (&self.data, self.last_updated) {
            if Instant::now().duration_since(last) < self.ttl {
                return Some(data.clone());
            }
        }
        None
    }

    pub fn set(&mut self, data: T) {
        self.data = Some(data);
        self.last_updated = Some(Instant::now());
    }
}
```

**Example**:
```rust
// Cache symbol list for 1 hour
let mut symbol_cache = CachedData::new(Duration::from_secs(3600));

pub async fn get_symbols(&mut self) -> Result<Vec<String>, Error> {
    if let Some(cached) = symbol_cache.get() {
        return Ok(cached);
    }

    self.public_limiter.throttle().await;
    let symbols = self.fetch_symbols().await?;
    symbol_cache.set(symbols.clone());

    Ok(symbols)
}
```

### 5. Use WebSocket for Real-Time Data

Instead of polling REST endpoints:
```rust
// DON'T: Poll every second (60 req/min per symbol)
loop {
    let trades = get_trades(symbol).await?;
    sleep(Duration::from_secs(1)).await;
}

// DO: Use WebSocket
let ws = connect_market_data().await?;
ws.subscribe_trades(symbol).await?;
loop {
    let trade = ws.recv_trade().await?;
    // Process trade
}
```

### 6. Implement Circuit Breaker

```rust
pub struct CircuitBreaker {
    failure_count: u32,
    failure_threshold: u32,
    state: CircuitState,
    last_failure: Option<Instant>,
    timeout: Duration,
}

pub enum CircuitState {
    Closed,  // Normal operation
    Open,    // Block requests
    HalfOpen, // Test if service recovered
}

impl CircuitBreaker {
    pub fn new(failure_threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_count: 0,
            failure_threshold,
            state: CircuitState::Closed,
            last_failure: None,
            timeout,
        }
    }

    pub fn record_success(&mut self) {
        self.failure_count = 0;
        self.state = CircuitState::Closed;
    }

    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(Instant::now());

        if self.failure_count >= self.failure_threshold {
            self.state = CircuitState::Open;
        }
    }

    pub fn can_request(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                if let Some(last) = self.last_failure {
                    if Instant::now().duration_since(last) > self.timeout {
                        self.state = CircuitState::HalfOpen;
                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            CircuitState::HalfOpen => true,
        }
    }
}
```

---

## Historical Data Limitations

### Public Data Availability

**Limit**: Public API endpoints return data for **maximum 7 calendar days**

**Affected Endpoints**:
- `GET /v1/trades/{symbol}` - Last 7 days of trades
- Historical candles (7 day limit)

**Note**: For data older than 7 days, you must use authenticated endpoints or have downloaded data previously.

### Workaround

1. **Regularly download data**: Fetch and store historical data daily
2. **Use authenticated endpoints**: Some endpoints may offer longer history
3. **Third-party data providers**: Consider historical data services

---

## Rate Limit Headers

### Response Headers

Gemini **does not** currently provide rate limit headers in responses (unlike some exchanges).

**Missing headers** (not available):
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`

**Implication**: Must track limits client-side; cannot rely on server headers.

---

## Testing Rate Limits

### Sandbox Limits

Sandbox environment (`api.sandbox.gemini.com`) has the **same rate limits** as production.

**Do not** use sandbox to "test" rate limit behavior without consequences.

### Local Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_throttle() {
        let mut limiter = RateLimiter::new_public();
        let start = Instant::now();

        // Make 3 requests
        for _ in 0..3 {
            limiter.throttle().await;
        }

        let elapsed = start.elapsed();

        // Should take ~2 seconds (1 req/sec recommended rate)
        assert!(elapsed >= Duration::from_secs(2));
        assert!(elapsed < Duration::from_secs(3));
    }

    #[tokio::test]
    async fn test_burst_allowance() {
        let mut limiter = RateLimiter::new_private();

        // Should handle burst of 15 requests
        for _ in 0..15 {
            assert!(!limiter.would_exceed_limit());
            limiter.throttle().await;
        }
    }
}
```

---

## Monitoring and Metrics

### Track Rate Limit Usage

```rust
pub struct RateLimitMetrics {
    pub requests_last_minute: u32,
    pub requests_last_second: f64,
    pub rejected_429_count: u32,
    pub retry_count: u32,
}

impl GeminiConnector {
    pub fn get_metrics(&self) -> RateLimitMetrics {
        RateLimitMetrics {
            requests_last_minute: self.private_limiter.request_history.len() as u32,
            requests_last_second: self.calculate_recent_rate(),
            rejected_429_count: self.error_count_429,
            retry_count: self.retry_count,
        }
    }
}
```

### Alerts

Set up alerts when:
- 429 errors exceed threshold
- Request rate consistently near limit
- Retry count increasing

---

## Summary Table

| Aspect | Public Endpoints | Private Endpoints |
|--------|------------------|-------------------|
| **Rate Limit** | 120 req/min | 600 req/min |
| **Recommended Max** | 1 req/sec | 5 req/sec |
| **Burst Allowance** | 5 requests | 5 requests |
| **Window Type** | Sliding 60s | Sliding 60s |
| **Error Code** | 429 | 429 |
| **Retry Strategy** | Exponential backoff | Exponential backoff |
| **WebSocket Limit** | 1 req/symbol/min | N/A (push-based) |
| **Historical Data** | 7 days max | Varies by endpoint |

---

## Implementation Checklist

- [ ] Separate rate limiters for public and private endpoints
- [ ] Sliding window tracking (60 seconds)
- [ ] Per-second throttling (1/sec public, 5/sec private)
- [ ] Burst allowance handling (5 request queue)
- [ ] 429 error detection and handling
- [ ] Exponential backoff retry strategy
- [ ] Circuit breaker for repeated failures
- [ ] Caching for frequently-accessed data
- [ ] WebSocket usage for real-time data
- [ ] Rate limit metrics and monitoring
- [ ] Pre-flight limit checks for critical operations
- [ ] Unit tests for rate limiter behavior

---

## References

- Rate Limits: https://docs.gemini.com/rate-limit
- WebSocket Rate Limits: https://docs.gemini.com/websocket/overview/rate-limits
- API Agreement: https://gemini.com/api-agreement/
