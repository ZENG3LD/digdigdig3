# MEXC API Rate Limits

## Overview

MEXC implements rate limiting to ensure fair access and system stability. Rate limits are enforced per endpoint based on either IP address or UID (User ID).

---

## Rate Limit Types

### 1. IP-Based Limits

For endpoints that **do not require authentication** (public market data), rate limits are applied per IP address.

**Default IP Limit**: 500 requests per 10 seconds per endpoint

### 2. UID-Based Limits

For endpoints that **require authentication** (private trading/account data), rate limits are applied per user account (UID).

**Default UID Limit**: 500 requests per 10 seconds per endpoint

---

## Request Weight System

Each API endpoint has an assigned **weight** that counts toward the rate limit.

### Weight Examples

| Endpoint | Method | Weight | Limit Type |
|----------|--------|--------|------------|
| `/api/v3/ping` | GET | 1 | IP |
| `/api/v3/time` | GET | 1 | IP |
| `/api/v3/exchangeInfo` | GET | 10 | IP |
| `/api/v3/depth` | GET | 1-50* | IP |
| `/api/v3/trades` | GET | 1 | IP |
| `/api/v3/klines` | GET | 1 | IP |
| `/api/v3/ticker/24hr` | GET | 1-40** | IP |
| `/api/v3/ticker/price` | GET | 1-2** | IP |
| `/api/v3/ticker/bookTicker` | GET | 1-2** | IP |
| `/api/v3/order` | POST | 1 | UID |
| `/api/v3/order` | DELETE | 1 | UID |
| `/api/v3/order` | GET | 2 | UID |
| `/api/v3/batchOrders` | POST | 5 | UID |
| `/api/v3/openOrders` | GET | 3-40*** | UID |
| `/api/v3/openOrders` | DELETE | 1 | UID |
| `/api/v3/allOrders` | GET | 10 | UID |
| `/api/v3/account` | GET | 10 | UID |
| `/api/v3/myTrades` | GET | 10 | UID |
| `/api/v3/capital/config/getall` | GET | 10 | UID |

**Notes:**
- `*` Depth weight varies by limit parameter (see Depth Endpoint Weights below)
- `**` Weight depends on whether symbol parameter is provided
- `***` Weight depends on whether symbol parameter is provided

### Depth Endpoint Weights

The `/api/v3/depth` endpoint weight varies based on the `limit` parameter:

| Limit Range | Weight |
|-------------|--------|
| 1-100 | 1 |
| 101-500 | 5 |
| 501-1000 | 10 |
| 1001-5000 | 50 |

### Ticker Endpoint Weights

Ticker endpoints have different weights when querying all symbols:

| Endpoint | Single Symbol | All Symbols |
|----------|---------------|-------------|
| `/api/v3/ticker/24hr` | 1 | 40 |
| `/api/v3/ticker/price` | 1 | 2 |
| `/api/v3/ticker/bookTicker` | 1 | 2 |

### Open Orders Weight

`/api/v3/openOrders` endpoint weight varies:

| Condition | Weight |
|-----------|--------|
| With symbol parameter | 3 |
| Without symbol parameter | 40 |

---

## Rate Limit Rules

### Per Endpoint Limits

Each endpoint has an **independent** limit:
- Each endpoint with IP limits: 500 requests per 10 seconds
- Each endpoint with UID limits: 500 requests per 10 seconds

**Example:**
```
/api/v3/depth: 500 req/10s
/api/v3/trades: 500 req/10s
/api/v3/ticker/price: 500 req/10s
(These are independent - total can be 1500 req/10s across different endpoints)
```

### Recent Update (March 2025)

MEXC adjusted API spot order rate limits effective March 25, 2025. Check official announcements for current limits.

---

## HTTP Response Headers

Rate limit information is included in response headers:

```http
X-RATELIMIT-LIMIT: 500
X-RATELIMIT-REMAINING: 485
X-RATELIMIT-RESET: 1640080810000
```

| Header | Description |
|--------|-------------|
| `X-RATELIMIT-LIMIT` | Maximum requests allowed in window |
| `X-RATELIMIT-REMAINING` | Remaining requests in current window |
| `X-RATELIMIT-RESET` | Timestamp when limit resets (milliseconds) |

---

## Rate Limit Violations

### HTTP 429 Response

When rate limit is exceeded, server returns HTTP 429 (Too Many Requests):

```json
{
  "code": 429,
  "msg": "Too Many Requests"
}
```

### Retry-After Header

The response may include a `Retry-After` header indicating seconds to wait:

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 10
```

---

## Ban System

### Automated IP Bans

Repeatedly violating rate limits and/or failing to back off after receiving 429 responses will result in an **automated IP ban**.

### Ban Duration

Bans are tracked and **scaled in duration** for repeat offenders:

| Violation Level | Ban Duration |
|----------------|--------------|
| First offense | 2 minutes |
| Repeated violations | Up to 3 days |
| Severe violations | Longer periods |

### Ban Detection

When IP is banned:
- HTTP 418 status code may be returned
- All requests from IP are rejected
- Ban applies to all endpoints

---

## WebSocket Rate Limits

### Connection Limits

**Spot WebSocket:**
- Maximum 30 subscriptions per WebSocket connection
- Connection valid for 24 hours maximum
- Server disconnects after 30 seconds if no valid subscription
- Server disconnects after 1 minute if subscription has no data flow

**Futures WebSocket:**
- Similar limits to spot
- Ping required every 10-20 seconds to prevent disconnection

### Listen Key Limits

For user data streams:
- Maximum 60 listen keys per UID
- Each listen key supports maximum 5 WebSocket connections
- Listen keys expire after 60 minutes without keepalive

---

## Best Practices

### 1. Track Request Weight

Maintain a counter for request weight in your application:

```rust
struct RateLimiter {
    weight: u32,
    window_start: u64,
    limit: u32,
}

impl RateLimiter {
    fn can_request(&mut self, weight: u32) -> bool {
        let now = current_timestamp_ms();

        // Reset if 10 seconds passed
        if now - self.window_start >= 10_000 {
            self.weight = 0;
            self.window_start = now;
        }

        // Check if request would exceed limit
        if self.weight + weight <= self.limit {
            self.weight += weight;
            true
        } else {
            false
        }
    }
}
```

### 2. Implement Exponential Backoff

When receiving 429 errors, use exponential backoff:

```rust
async fn request_with_backoff<T>(
    request_fn: impl Fn() -> Future<Output = Result<T, Error>>,
) -> Result<T, Error> {
    let mut delay = 1000; // Start with 1 second
    let max_delay = 60000; // Max 60 seconds
    let mut attempts = 0;
    let max_attempts = 5;

    loop {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(Error::RateLimit) if attempts < max_attempts => {
                tokio::time::sleep(Duration::from_millis(delay)).await;
                delay = (delay * 2).min(max_delay);
                attempts += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 3. Respect Retry-After Header

When 429 response includes `Retry-After`, wait at least that duration:

```rust
fn parse_retry_after(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.parse().ok())
}

async fn handle_rate_limit(response: Response) -> Result<(), Error> {
    if response.status() == 429 {
        if let Some(seconds) = parse_retry_after(response.headers()) {
            tokio::time::sleep(Duration::from_secs(seconds)).await;
        } else {
            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
    Ok(())
}
```

### 4. Use WebSocket for Real-Time Data

For high-frequency price updates, use WebSocket instead of REST polling:

```rust
// Bad: Polling every second
loop {
    let price = client.get_ticker("BTCUSDT").await?; // 1 weight/sec = 3600/hour
    tokio::time::sleep(Duration::from_secs(1)).await;
}

// Good: WebSocket subscription
client.subscribe_ticker("BTCUSDT", |ticker| {
    println!("Price: {}", ticker.price);
}); // No polling weight
```

### 5. Batch Requests When Possible

Use batch endpoints to reduce request count:

```rust
// Bad: Individual order placements
for order in orders {
    client.place_order(order).await?; // 5 requests = 5 weight
}

// Good: Batch order placement
client.place_batch_orders(orders).await?; // 1 request = 5 weight
```

### 6. Cache Static Data

Cache data that doesn't change frequently:

```rust
struct SymbolCache {
    data: HashMap<String, SymbolInfo>,
    last_update: u64,
}

impl SymbolCache {
    async fn get_or_fetch(&mut self, client: &Client) -> Result<&HashMap<String, SymbolInfo>, Error> {
        let now = current_timestamp_ms();

        // Refresh every 24 hours
        if now - self.last_update > 86_400_000 {
            let info = client.get_exchange_info(None).await?;
            self.data = info.symbols.into_iter()
                .map(|s| (s.symbol.clone(), s))
                .collect();
            self.last_update = now;
        }

        Ok(&self.data)
    }
}
```

### 7. Monitor Rate Limit Headers

Track remaining requests to prevent hitting limits:

```rust
fn check_rate_limit(headers: &HeaderMap) -> Option<RateLimitInfo> {
    let limit = headers.get("X-RATELIMIT-LIMIT")?.to_str().ok()?.parse().ok()?;
    let remaining = headers.get("X-RATELIMIT-REMAINING")?.to_str().ok()?.parse().ok()?;
    let reset = headers.get("X-RATELIMIT-RESET")?.to_str().ok()?.parse().ok()?;

    Some(RateLimitInfo { limit, remaining, reset })
}

// Pause if close to limit
if let Some(info) = check_rate_limit(&response.headers()) {
    if info.remaining < 10 {
        let wait = info.reset - current_timestamp_ms();
        tokio::time::sleep(Duration::from_millis(wait)).await;
    }
}
```

---

## Order Rate Limits

### Order Placement Limits

Additional limits may apply to order placement beyond general API limits:

- Check official announcements for current limits
- Limits may vary by account type (retail vs institutional)
- Special limits may apply during high volatility

### Order Cancellation Limits

Canceling orders also counts toward rate limits:
- Single order cancel: 1 weight
- Cancel all orders: 1 weight

**Note**: Use batch cancellation when possible to reduce API calls.

---

## Special Considerations

### IP Whitelisting

When using IP whitelisting:
- Maximum 10 IP addresses per API key
- Rate limits apply per IP
- If multiple IPs share API key, they share UID-based limits

### Sub-Accounts

Sub-accounts have separate UID limits:
- Each sub-account has independent rate limits
- Master account can create up to 60 sub-accounts
- Useful for distributing request load

### Institutional Accounts

Institutional users may have different rate limits:
- Contact institution@mexc.com for details
- Higher limits may be available
- Custom arrangements possible

---

## Rate Limit Monitoring

### Implementation Example

```rust
use std::sync::Arc;
use tokio::sync::Mutex;

struct EndpointLimiter {
    limits: Arc<Mutex<HashMap<String, RateLimiter>>>,
}

impl EndpointLimiter {
    async fn check_and_wait(&self, endpoint: &str, weight: u32) {
        let mut limits = self.limits.lock().await;
        let limiter = limits.entry(endpoint.to_string())
            .or_insert_with(|| RateLimiter::new(500, 10_000));

        while !limiter.can_request(weight) {
            let wait = limiter.time_until_reset();
            drop(limits); // Release lock while waiting
            tokio::time::sleep(Duration::from_millis(wait)).await;
            limits = self.limits.lock().await;
        }
    }
}

// Usage
limiter.check_and_wait("/api/v3/order", 1).await;
client.place_order(order).await?;
```

---

## Testing Rate Limits

### Sandbox Testing

When testing rate limit handling:

1. Use a separate API key for testing
2. Implement mock rate limiter for unit tests
3. Test exponential backoff logic
4. Verify ban detection and recovery

### Example Test

```rust
#[tokio::test]
async fn test_rate_limit_backoff() {
    let mut attempt = 0;
    let result = retry_with_backoff(|| async {
        attempt += 1;
        if attempt < 3 {
            Err(Error::RateLimit)
        } else {
            Ok("Success")
        }
    }).await;

    assert!(result.is_ok());
    assert_eq!(attempt, 3);
}
```

---

## Summary

### Key Points

1. **500 requests per 10 seconds** per endpoint (default)
2. **IP-based** for public endpoints, **UID-based** for private
3. **Request weight** varies by endpoint (1-50)
4. **HTTP 429** indicates rate limit exceeded
5. **Exponential backoff** required for retries
6. **Automated bans** for repeated violations (2 minutes to 3 days)
7. **WebSocket preferred** for high-frequency data
8. **Cache static data** to reduce API calls
9. **Monitor headers** to track remaining quota
10. **Batch requests** when possible

### Recommended Limits

To stay within limits safely:
- Keep average weight under 400/10s (80% of limit)
- Implement request queuing
- Use WebSocket for real-time data
- Cache exchange info and symbol data
- Monitor rate limit headers
- Implement proper error handling and backoff

---

## References

- MEXC API Documentation: https://www.mexc.com/api-docs/spot-v3/general-info
- Rate limit adjustments announced at: https://www.mexc.com/announcements/api-updates
