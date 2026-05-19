# Coinbase Advanced Trade API Rate Limits

Comprehensive documentation of Coinbase Advanced Trade API rate limiting system for implementing robust connector with proper rate limit handling.

## Overview

Coinbase Advanced Trade API uses a **requests-per-second** rate limiting system with separate limits for public and private endpoints. Rate limits are tracked per IP (public) or per user (private).

### Core Concepts

- **Per-User Limits**: Private endpoints throttled by user account
- **Per-IP Limits**: Public endpoints throttled by IP address
- **Fixed Rate**: Requests per second (not quota pools like KuCoin)
- **Simple System**: No weight system or VIP tiers
- **Response Headers**: Rate limit info included in headers

---

## 1. Rate Limit Types

### 1.1 Private Endpoints (User-Based)

**Rate Limit**: **30 requests per second** per user

**Scope**: All authenticated endpoints requiring JWT token:
- Account endpoints (`/accounts`)
- Order endpoints (`/orders`)
- Fill endpoints (`/orders/historical/fills`)
- Transaction summary (`/transaction_summary`)
- Product endpoints (private versions)

**Tracking**: Per user account (identified by API key in JWT)

### 1.2 Public Endpoints (IP-Based)

**Rate Limit**: **10 requests per second** per IP address

**Scope**: Public market data endpoints (no authentication):
- Public products (`GET /market/products`)
- Public product book (`GET /market/product_book`)
- Public candles (`GET /market/products/{product_id}/candles`)
- Public ticker (`GET /market/products/{product_id}/ticker`)
- Server time (`GET /api/v3/brokerage/time`)

**Tracking**: Per IP address

---

## 2. WebSocket Rate Limits

### 2.1 Connection Limits

**Maximum Connections**: **750 per second** per IP address

**Applies to**: New WebSocket connection attempts

**Notes**:
- Sustained high connection rate may trigger throttling
- Use persistent connections instead of frequent reconnects
- Failover between `advanced-trade-ws` and `advanced-trade-ws-user`

### 2.2 Message Rate Limits

**Unauthenticated Messages**: **8 per second** per IP address

**Authenticated Messages**: No documented per-message limit

**Subscription Limit**: No documented maximum number of subscriptions per connection

**Notes**:
- Unauthenticated = messages sent before subscribe (e.g., ping during handshake)
- After successful subscribe, authenticated messages not rate-limited
- WebSocket more efficient than REST polling for real-time data

### 2.3 Mandatory Subscription

**Requirement**: Must send subscribe message **within 5 seconds** of connection

**Consequence**: Connection closed if no subscribe received

**Implementation**:
```json
{
  "type": "subscribe",
  "product_ids": ["BTC-USD"],
  "channel": "level2",
  "jwt": "eyJhbGci..."
}
```

---

## 3. Response Headers

### 3.1 Rate Limit Headers

Coinbase includes rate limit information in response headers:

| Header Name | Description | Example Value |
|-------------|-------------|---------------|
| `CB-RATELIMIT-REMAINING` | Remaining requests in current window | `25` |
| `CB-RATELIMIT-RESET` | Time when limit resets (Unix timestamp) | `1698315990` |
| `CB-RATELIMIT-LIMIT` | Total rate limit | `30` |

**Note**: Not all endpoints return these headers. Presence may vary.

### 3.2 Using Headers for Rate Limit Tracking

**Implementation Strategy:**

```rust
fn parse_rate_limit_headers(headers: &HeaderMap) -> Option<RateLimitInfo> {
    let remaining = headers.get("CB-RATELIMIT-REMAINING")?.to_str().ok()?.parse().ok()?;
    let reset = headers.get("CB-RATELIMIT-RESET")?.to_str().ok()?.parse().ok()?;
    let limit = headers.get("CB-RATELIMIT-LIMIT")?.to_str().ok()?.parse().ok()?;

    Some(RateLimitInfo {
        remaining,
        reset_at: Instant::from_unix(reset),
        limit,
    })
}
```

**Usage**:
1. Parse headers after each response
2. Track remaining requests
3. Implement adaptive throttling based on remaining count
4. Pause requests when limit approaching

---

## 4. Error Responses for Rate Limits

### 4.1 Rate Limit Exceeded Error

**HTTP Status Code**: `429 Too Many Requests`

**Error Response**:
```json
{
  "error": "rate_limit_exceeded",
  "message": "Too many requests. Please try again later.",
  "error_details": "Rate limit exceeded for this API key"
}
```

**Error Type**: `rate_limit_exceeded`

### 4.2 Retry Strategy

**When receiving 429**:

1. **Check reset time**: Parse `CB-RATELIMIT-RESET` header if available
2. **Wait until reset**: Calculate wait time from reset timestamp
3. **Exponential backoff**: If no reset header, use exponential backoff starting at 1 second

**Recommended Implementation**:

```rust
async fn handle_rate_limit_error(headers: &HeaderMap, attempt: u32) -> Result<Duration> {
    if let Some(reset_str) = headers.get("CB-RATELIMIT-RESET") {
        let reset_timestamp = reset_str.to_str()?.parse::<i64>()?;
        let current_timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs() as i64;
        let wait_seconds = (reset_timestamp - current_timestamp).max(1);
        Ok(Duration::from_secs(wait_seconds as u64))
    } else {
        // Exponential backoff: 1s, 2s, 4s, 8s, ...
        let backoff_secs = 2_u64.pow(attempt).min(60);
        Ok(Duration::from_secs(backoff_secs))
    }
}
```

---

## 5. Best Practices

### 5.1 Request Throttling

**For Private Endpoints (30 req/s)**:

```rust
struct RateLimiter {
    interval: Duration,
    last_request: Instant,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        Self {
            interval: Duration::from_millis(1000 / requests_per_second as u64),
            last_request: Instant::now(),
        }
    }

    async fn wait(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < self.interval {
            sleep(self.interval - elapsed).await;
        }
        self.last_request = Instant::now();
    }
}

// Usage
let mut limiter = RateLimiter::new(30);  // 30 req/s
limiter.wait().await;
make_api_request().await;
```

**Conservative approach**: Use **25 req/s** instead of 30 to leave safety margin

### 5.2 Adaptive Rate Limiting

Monitor `CB-RATELIMIT-REMAINING` and slow down as limit approaches:

```rust
fn calculate_delay(remaining: u32, limit: u32) -> Duration {
    let usage_percent = (limit - remaining) as f64 / limit as f64;

    if usage_percent > 0.9 {
        // 90%+ used - be very conservative
        Duration::from_millis(500)
    } else if usage_percent > 0.75 {
        // 75%+ used - slow down
        Duration::from_millis(100)
    } else {
        // Normal rate
        Duration::from_millis(33)  // ~30 req/s
    }
}
```

### 5.3 Burst Protection

Implement token bucket for handling bursts:

```rust
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,  // tokens per second
    last_refill: Instant,
}

impl TokenBucket {
    async fn acquire(&mut self) -> bool {
        self.refill();
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = Instant::now();
    }
}
```

### 5.4 WebSocket vs REST

**Prefer WebSocket for**:
- Real-time price updates
- Order book updates
- Order status monitoring
- High-frequency data

**Reasons**:
- No REST rate limits apply
- More efficient (push vs poll)
- Lower latency
- Better for sustained connections

**Use REST for**:
- Initial data retrieval
- Historical data (candles)
- One-time queries
- Order placement (no WebSocket alternative)

### 5.5 IP Rotation (Public Endpoints Only)

For public endpoints hitting 10 req/s limit:

- Use multiple IP addresses if available
- Rotate requests across IPs
- Each IP gets independent 10 req/s quota

**Note**: Not applicable for private endpoints (tracked per user, not IP)

---

## 6. Comparison with KuCoin

### Rate Limit Comparison

| Feature | KuCoin | Coinbase |
|---------|--------|----------|
| **System** | Quota pools (30s window) | Requests per second |
| **Private Limit** | 16,000/30s (VIP 5) ≈ 533 req/s | 30 req/s |
| **Public Limit** | 2,000/30s ≈ 67 req/s | 10 req/s |
| **Weight System** | Yes (different endpoints, different weights) | No (all endpoints count as 1) |
| **VIP Tiers** | Yes (higher limits for higher VIP) | No (same for all users) |
| **Tracking** | Per UID (private) / IP (public) | Per user (private) / IP (public) |
| **Response Headers** | `gw-ratelimit-*` | `CB-RATELIMIT-*` |
| **Error Code** | `429000` | `429` |
| **Reset Time** | Countdown (milliseconds) | Absolute timestamp (seconds) |

### Key Differences

1. **Simpler System**: Coinbase has no weight system or tiers
2. **Lower Limits**: 30 req/s vs KuCoin's 533 req/s (VIP 5)
3. **Fixed Limits**: No VIP upgrades or professional tier applications
4. **Per-Second**: Easier to implement (no 30-second quota tracking)

---

## 7. Implementation Checklist

### 7.1 Essential Features

- [ ] Track requests per second (30 for private, 10 for public)
- [ ] Implement simple rate limiter with fixed delay
- [ ] Parse `CB-RATELIMIT-*` headers when available
- [ ] Handle 429 errors with exponential backoff
- [ ] Calculate retry delay from `CB-RATELIMIT-RESET` header
- [ ] Log rate limit metrics for monitoring

### 7.2 Advanced Features

- [ ] Adaptive rate limiting based on remaining count
- [ ] Token bucket for burst handling
- [ ] Separate limiters for public vs private endpoints
- [ ] WebSocket failover for high-frequency data
- [ ] Rate limit dashboard/monitoring
- [ ] Automatic slowdown at 80%+ usage

### 7.3 Testing Considerations

- [ ] Test behavior when limit exceeded
- [ ] Verify retry logic for 429 errors
- [ ] Test with sustained high request rate
- [ ] Validate header parsing
- [ ] Ensure WebSocket connections stay alive
- [ ] Test failover scenarios

---

## 8. Summary Table

### Quick Reference: Rate Limits

| Category | Limit | Tracking | Notes |
|----------|-------|----------|-------|
| **Private Endpoints** | 30 req/s | Per user | All authenticated endpoints |
| **Public Endpoints** | 10 req/s | Per IP | Market data (no auth) |
| **WebSocket Connections** | 750/s | Per IP | New connection attempts |
| **WebSocket Messages (unauth)** | 8/s | Per IP | Before subscription |
| **Subscription Deadline** | 5 seconds | Per connection | Must subscribe or disconnect |

### Error Response Quick Reference

| Status | Error Type | Meaning | Action |
|--------|----------|---------|--------|
| 429 | `rate_limit_exceeded` | Too many requests | Wait for reset or exponential backoff |

### Header Reference

| Header | Example | Description |
|--------|---------|-------------|
| `CB-RATELIMIT-LIMIT` | `30` | Total rate limit (req/s) |
| `CB-RATELIMIT-REMAINING` | `25` | Remaining requests |
| `CB-RATELIMIT-RESET` | `1698315990` | Unix timestamp when resets |

---

## 9. Rate Limit Calculator

### Calculate Safe Request Rate

For 30 req/s private limit with 20% safety margin:

```
Safe rate = 30 * 0.8 = 24 req/s
Delay between requests = 1000ms / 24 = 41.67ms
```

For 10 req/s public limit with 20% safety margin:

```
Safe rate = 10 * 0.8 = 8 req/s
Delay between requests = 1000ms / 8 = 125ms
```

### Burst Capacity

Token bucket with max 10 tokens, refill 30/s:

```
Max burst = 10 requests instantly
Sustained rate = 30 req/s
Recovery time = 10 tokens / 30 tokens/s = 0.33s
```

---

## Sources

Research compiled from official Coinbase API documentation:

- [Advanced Trade WebSocket Rate Limits](https://docs.cdp.coinbase.com/coinbase-app/advanced-trade-apis/websocket/websocket-rate-limits)
- [Advanced Trade API Rate Limits](https://docs-stage.cloud.coinbase.com/advanced-trade/docs/rest-api-rate-limits)
- [Coinbase API Essentials - Rollout](https://rollout.com/integration-guides/coinbase/api-essentials)
- [Coinbase API Cheat Sheet - Vezgo](https://vezgo.com/blog/coinbase-api-cheat-sheet-for-developers/)
- [Coinbase Advanced Python SDK](https://github.com/coinbase/coinbase-advanced-py)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Research Compiled By:** Claude Code Research Agent
**Key Difference from KuCoin:** Coinbase uses simple requests-per-second limits (30 private, 10 public) vs KuCoin's weight-based quota pools with VIP tiers.
