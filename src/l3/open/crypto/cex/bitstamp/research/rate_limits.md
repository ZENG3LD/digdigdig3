# Bitstamp API Rate Limits

This document details Bitstamp's rate limiting policies and best practices for staying within limits.

---

## Rate Limit Overview

Bitstamp enforces rate limits to protect their infrastructure and ensure fair access for all users.

### Standard Rate Limits

- **Requests per second**: 400 requests/second
- **Requests per 10 minutes**: 10,000 requests/10 minutes (default threshold)

### Custom Rate Limits

Rate limits can be increased beyond the standard limits:
- Contact Bitstamp support to request higher limits
- Requires entering a bespoke agreement with Bitstamp
- Typically for institutional or high-frequency trading clients

---

## Rate Limit Enforcement

### How It Works

Bitstamp uses a sliding window rate limiter:
1. Tracks requests per second (burst limit)
2. Tracks requests per 10-minute window (sustained limit)
3. Both limits must be respected

### Rate Limit Windows

- **Per-second window**: Rolling 1-second window
- **Per-10-minute window**: Rolling 600-second window

---

## Rate Limit Errors

### Error Response

When you exceed rate limits, you'll receive:

**HTTP Status Code**: `429 Too Many Requests`

**Response Body**:
```json
{
  "status": "error",
  "reason": "Request rejected due to exceeded rate limit",
  "code": "400.002"
}
```

or

**HTTP Status Code**: `400 Bad Request`

**Response Body**:
```json
{
  "status": "error",
  "reason": "Request rejected due to exceeded rate limit",
  "code": "400.002"
}
```

### Temporary Blocks

- Too many requests in a short time will result in temporary blocking
- Block duration varies based on violation severity
- Repeated violations may lead to longer blocks or API key suspension

---

## Rate Limit Headers

Bitstamp does not currently provide rate limit information in response headers (unlike some other exchanges). You must track your own request rates.

---

## Endpoint-Specific Limits

### Cached Endpoints

Some endpoints have caching:

**Open Orders Endpoints**:
- `POST /api/v2/open_orders/all/`
- `POST /api/v2/open_orders/{pair}/`
- **Cache Duration**: 10 seconds

These endpoints return cached data for 10 seconds, so repeated calls within that window return the same data.

### Public vs Private Endpoints

- **Public endpoints** (market data): Contribute to the same rate limit
- **Private endpoints** (trading, account): Contribute to the same rate limit
- Both public and private endpoints share the same rate limit pool (no separate limits)

---

## Best Practices

### 1. Use WebSocket API for Real-Time Data

Instead of polling REST endpoints repeatedly, use the WebSocket API for:
- Live trades (`live_trades_{pair}`)
- Order book updates (`order_book_{pair}`, `diff_order_book_{pair}`)
- Ticker updates

**Benefits**:
- No REST rate limit consumption
- Real-time push updates
- More efficient and lower latency

### 2. Batch Non-Urgent Requests

Group multiple non-urgent operations:
- Fetch all balances with one call (`/api/v2/account_balances/`)
- Fetch all open orders with one call (`/api/v2/open_orders/all/`)
- Use multi-pair endpoints when available

### 3. Implement Request Spacing

Space out API calls to avoid bursts:

```rust
// Example: Rate limiter that allows 400 req/s
let mut limiter = RateLimiter::new(400, Duration::from_secs(1));

async fn make_request(&mut self) {
    limiter.wait().await;
    // Make API request
}
```

### 4. Cache Data Locally

Cache data that doesn't change frequently:
- Trading pair information (`/api/v2/markets/`)
- Currency information (`/api/v2/currencies/`)
- Fee schedules (`/api/v2/fees/trading/`)

**Recommended Cache Duration**:
- Markets/currencies: 1 hour
- Fees: 1 hour
- Ticker: 1-5 seconds (or use WebSocket)
- Order book: Use WebSocket instead

### 5. Handle 429 Errors Gracefully

Implement exponential backoff:

```rust
async fn request_with_retry(&self, max_retries: u32) -> Result<Response> {
    let mut retries = 0;
    let mut delay = Duration::from_secs(1);

    loop {
        match self.make_request().await {
            Ok(response) => return Ok(response),
            Err(e) if e.is_rate_limit() && retries < max_retries => {
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 6. Monitor Your Request Rate

Track your own request rates:

```rust
struct RateMonitor {
    requests_last_second: AtomicU32,
    requests_last_10min: AtomicU32,
    last_reset_second: Mutex<Instant>,
    last_reset_10min: Mutex<Instant>,
}

impl RateMonitor {
    fn check_limits(&self) -> Result<(), Error> {
        let req_per_sec = self.requests_last_second.load(Ordering::Relaxed);
        let req_per_10min = self.requests_last_10min.load(Ordering::Relaxed);

        if req_per_sec >= 400 {
            return Err(Error::RateLimitPerSecond);
        }
        if req_per_10min >= 10_000 {
            return Err(Error::RateLimit10Min);
        }

        Ok(())
    }
}
```

### 7. Prioritize Critical Requests

Use separate rate limiters for different priority levels:
- **High priority**: Order placement, cancellation
- **Medium priority**: Balance checks, order status
- **Low priority**: Historical data, market info

Reserve rate limit capacity for critical trading operations.

---

## Rate Limit Strategies

### Strategy 1: Conservative Approach

Stay well below the limits:
- **Target**: 300 requests/second (75% of limit)
- **Target**: 7,500 requests/10 minutes (75% of limit)

### Strategy 2: Burst with Monitoring

Allow short bursts but monitor closely:
- Track requests in real-time
- Slow down when approaching limits
- Use adaptive rate limiting

### Strategy 3: WebSocket-First

Minimize REST API usage:
- Use WebSocket for all real-time data
- Use REST only for:
  - Order placement/cancellation
  - Initial data fetching
  - Account operations

**Expected REST usage**:
- ~10-50 requests per minute for active trading
- Well within rate limits

---

## Rate Limit Calculation

### Per-Second Limit

```
Max requests per second: 400
Minimum time between requests: 2.5ms
```

### Per-10-Minute Limit

```
Max requests per 10 minutes: 10,000
Average sustainable rate: ~16.67 requests/second
```

**Note**: The per-second limit (400/sec) is much higher than the sustainable rate (16.67/sec). This allows for short bursts while maintaining an average rate.

---

## Endpoint Request Costs

All endpoints currently have the same cost (1 request = 1 toward limit).

Unlike some exchanges, Bitstamp does not have weighted rate limits where different endpoints cost different amounts.

---

## Rate Limit Testing

### How to Test Your Limits

**Not Recommended**: Intentionally hitting rate limits can lead to temporary blocks.

**Better Approach**:
1. Implement rate tracking in your connector
2. Log request rates during development
3. Monitor in staging/testnet environment first
4. Use conservative limits in production

### Test Sandbox

Bitstamp provides a sandbox environment:
- **URL**: `https://sandbox.bitstamp.net`
- Same rate limits as production
- Test your rate limiting logic here

---

## Rate Limit Per API Key

Rate limits are enforced **per API key**:
- Each API key has independent rate limits
- Using multiple API keys can increase total throughput
- Ensure each key is properly managed

**Warning**: Using multiple API keys to circumvent rate limits may violate Bitstamp's terms of service. Consult with Bitstamp before implementing multi-key strategies.

---

## Order-Specific Considerations

### Order Placement Rate Limits

While there's no separate limit for order endpoints, consider:
- **Risk management**: Avoid accidental order spam
- **Market impact**: Too many orders can impact your fills
- **Exchange load**: Be a good citizen of the exchange

**Recommended**:
- Limit order placement to 10-20 orders/second max
- Use batch order operations when available
- Implement circuit breakers for abnormal conditions

### Order Cancellation

Cancellation endpoints share the same rate limit:
- `POST /api/v2/cancel_order/` - Cancel single order (1 request)
- `POST /api/v2/cancel_all_orders/` - Cancel all orders (1 request)

**Tip**: Use `cancel_all_orders` when canceling multiple orders to save rate limit quota.

---

## WebSocket Rate Limits

WebSocket connections have different limits:

### Connection Limits

- **Maximum concurrent connections**: Not publicly documented
- **Recommended**: 1-2 connections per client

### Subscription Limits

- **Maximum subscriptions per connection**: Not publicly documented
- **Recommended**: Subscribe to only the channels you need

### Message Rate Limits

WebSocket messages (subscriptions, unsubscriptions) may have limits, but these are not publicly documented.

**Best Practice**: Batch subscriptions in a single connection rather than creating multiple connections.

---

## Monitoring and Alerting

### What to Monitor

1. **Request rate per second**
2. **Request rate per 10 minutes**
3. **429 error frequency**
4. **Average response time** (may increase when approaching limits)

### Alert Thresholds

- **Warning**: 80% of rate limit
- **Critical**: 90% of rate limit
- **Emergency**: 429 errors received

### Logging

Log all rate limit related events:
```
[WARN] Request rate: 320/400 per second (80%)
[ERROR] Rate limit exceeded: 429 response received
[INFO] Backing off for 5 seconds
```

---

## Rate Limit Recovery

### After Hitting Limits

1. **Immediate**: Stop making requests
2. **Wait**: Implement backoff strategy (1s, 2s, 4s, 8s...)
3. **Resume**: Gradually resume at lower rate
4. **Monitor**: Watch for additional 429 errors

### Backoff Strategy

```rust
async fn handle_rate_limit_error(&mut self) {
    let mut backoff = Duration::from_secs(1);
    let max_backoff = Duration::from_secs(60);

    while backoff <= max_backoff {
        log::warn!("Rate limited, waiting {:?}", backoff);
        tokio::time::sleep(backoff).await;

        if self.test_request().await.is_ok() {
            log::info!("Rate limit recovered");
            return;
        }

        backoff *= 2;
    }
}
```

---

## Third-Party Rate Limit Libraries

Consider using existing rate limiting libraries:

**Rust**:
- `governor` crate: Token bucket rate limiter
- `ratelimit` crate: Simple rate limiter
- `leaky-bucket` crate: Leaky bucket algorithm

**Example with `governor`**:
```rust
use governor::{Quota, RateLimiter, Jitter};
use std::num::NonZeroU32;

let quota = Quota::per_second(NonZeroU32::new(400).unwrap());
let limiter = RateLimiter::direct(quota);

// Before each request
limiter.until_ready().await;
make_api_request().await;
```

---

## Summary

### Key Points

1. **Limits**: 400 req/s, 10,000 req/10min
2. **Shared**: All endpoints share the same limit pool
3. **WebSocket**: Use for real-time data to avoid REST limits
4. **Caching**: Cache static data (markets, fees)
5. **Backoff**: Implement exponential backoff for 429 errors
6. **Monitor**: Track your own request rates

### Recommended Implementation

```rust
pub struct BitstampRateLimiter {
    per_second: RateLimiter,
    per_10min: RateLimiter,
}

impl BitstampRateLimiter {
    pub fn new() -> Self {
        Self {
            per_second: RateLimiter::new(400, Duration::from_secs(1)),
            per_10min: RateLimiter::new(10_000, Duration::from_secs(600)),
        }
    }

    pub async fn wait(&self) {
        // Wait for both limiters
        self.per_second.wait().await;
        self.per_10min.wait().await;
    }
}
```

---

## Reference

- **Official Documentation**: https://www.bitstamp.net/api/
- **Standard Limits**: 400 req/s, 10,000 req/10min
- **Error Code**: `400.002` or HTTP 429
- **Contact**: support@bitstamp.net for custom limits
