# Paradex API Rate Limits

## Overview

Paradex implements **two-tier rate limiting**:
1. **Per-account limits** for private endpoints
2. **Per-IP limits** for both public and private endpoints

---

## Public Endpoints

### Default Public Limit
- **Limit**: 1,500 requests per minute per IP address
- **Applies to**: All public endpoints except those listed below

### Special Public Endpoints

| Endpoint | Limit | Scope |
|----------|-------|-------|
| `POST /onboarding` | 600 req/min | Per IP address |
| `POST /auth` | 600 req/min | Per IP address |

---

## Private Endpoints

### Order Management Endpoints

**Endpoints**:
- `POST /orders`
- `POST /orders/batch`
- `PUT /orders/:id` (modify)
- `DELETE /orders/:id`
- `DELETE /orders/batch`
- `DELETE /orders` (cancel all)

**Rate Limits**:
- **800 requests per second** OR
- **17,250 requests per minute**
- **Scope**: Per account

**Important Notes**:
- POST and DELETE operations **share the same rate limit pool**
- Batch operations (batch create, batch cancel) count as **1 unit** regardless of order count (1-50 orders)
- This provides **50x efficiency improvement** over individual operations

### Read Endpoints (GET)

**Endpoints**:
- `GET /account`
- `GET /account/*`
- `GET /positions`
- `GET /orders`
- `GET /orders/*`
- `GET /fills`
- `GET /balances`
- All other GET endpoints for account data

**Rate Limits**:
- **120 requests per second** OR
- **600 requests per minute**
- **Scope**: Per account

---

## IP-Based Constraint (Global)

### Additional IP Limit for Private Endpoints

**Critical Constraint**: Private endpoint rate limits are applied per account, but are also subject to an **additional IP-based rate limit of 1,500 req/m** across all accounts from the same IP address.

**Example Scenario**:
```
Account A: 500 req/min to private endpoints
Account B: 600 req/min to private endpoints
Account C: 400 req/min to private endpoints
---
Total from IP: 1,500 req/min (at limit)
```

Even if individual accounts are under their per-account limits, the combined traffic from the IP cannot exceed 1,500 req/min.

**Impact**:
- Multiple accounts from same IP share bandwidth
- Important for market makers with multiple sub-accounts
- Consider using multiple IPs for high-frequency trading

---

## Rate Limit Summary Table

| Endpoint Type | Specific Endpoint | Limit | Scope |
|---------------|-------------------|-------|-------|
| **Public** | Default | 1,500 req/min | Per IP |
| **Public** | `POST /onboarding` | 600 req/min | Per IP |
| **Public** | `POST /auth` | 600 req/min | Per IP |
| **Private** | `POST/DELETE/PUT /orders*` | 800 req/s OR 17,250 req/min | Per account |
| **Private** | `GET /*` | 120 req/s OR 600 req/min | Per account |
| **Private** | All private endpoints | 1,500 req/min | Per IP (additional) |

---

## Rate Limit Headers

**Note**: The provided documentation does not specify rate limit response headers. Typical implementations might include:

- `X-RateLimit-Limit`: Maximum requests allowed
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Timestamp when limit resets

**Recommendation**: Monitor HTTP 429 (Too Many Requests) responses and implement exponential backoff.

---

## Best Practices

### 1. Batch Operations

**Use batch endpoints whenever possible**:

```rust
// BAD: Individual cancellations (50 units)
for order_id in order_ids {
    cancel_order(order_id).await?; // 1 unit each
}

// GOOD: Batch cancellation (1 unit)
cancel_orders_batch(order_ids).await?; // 1 unit total
```

**Efficiency Gain**: 50x reduction in rate limit consumption

### 2. Request Prioritization

Prioritize requests by importance:

**High Priority** (critical for trading):
- Order submissions
- Order cancellations
- Position updates
- Fill notifications

**Low Priority** (can be batched/delayed):
- Historical data queries
- Account history
- Funding payment history

### 3. Token Refresh Strategy

**JWT tokens expire every 5 minutes**, but auth endpoint has 600 req/min limit:

```rust
const JWT_LIFETIME_MS: u64 = 5 * 60 * 1000; // 5 minutes
const REFRESH_MARGIN_MS: u64 = 3 * 60 * 1000; // Refresh at 3 minutes

// Refresh before expiration to avoid rate limit spikes
if time_since_issue >= REFRESH_MARGIN_MS {
    jwt = refresh_jwt().await?;
}
```

**Why 3 minutes?**
- Leaves 2-minute buffer for retries
- Avoids mass refresh when token expires
- Spreads auth requests over time

### 4. Multi-Account Management

For operations across multiple accounts from the same IP:

```rust
// Track combined IP usage
struct IpRateLimiter {
    ip_limit: u32,           // 1500 req/min
    window_start: Instant,
    request_count: u32,
}

impl IpRateLimiter {
    fn can_send(&mut self) -> bool {
        self.reset_if_needed();
        self.request_count < self.ip_limit
    }

    fn record_request(&mut self) {
        self.request_count += 1;
    }

    fn reset_if_needed(&mut self) {
        if self.window_start.elapsed() >= Duration::from_secs(60) {
            self.window_start = Instant::now();
            self.request_count = 0;
        }
    }
}
```

### 5. WebSocket for Real-Time Data

**Avoid polling** for real-time updates. Use WebSocket channels instead:

```rust
// BAD: Polling (consumes rate limits)
loop {
    let positions = get_positions().await?; // 120 req/s limit
    sleep(Duration::from_millis(100)).await;
}

// GOOD: WebSocket subscription (no rate limit)
ws.subscribe("positions", |data| {
    handle_position_update(data);
}).await?;
```

**WebSocket Benefits**:
- No rate limit consumption for subscriptions
- Real-time updates (no polling delay)
- Reduced server load

### 6. Error Handling

Implement exponential backoff for rate limit errors:

```rust
async fn execute_with_backoff<T>(
    operation: impl Fn() -> Future<Output = Result<T>>,
    max_retries: u32,
) -> Result<T> {
    let mut delay_ms = 100;

    for attempt in 0..max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if e.status_code() == 429 => {
                // Rate limit exceeded
                sleep(Duration::from_millis(delay_ms)).await;
                delay_ms *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }

    Err("Max retries exceeded")
}
```

### 7. Request Queuing

Implement request queue with rate limit awareness:

```rust
struct RateLimitedQueue {
    queue: VecDeque<Request>,
    tokens: u32,
    max_tokens: u32,
    refill_rate: u32, // tokens per second
    last_refill: Instant,
}

impl RateLimitedQueue {
    fn new(max_tokens: u32, refill_rate: u32) -> Self {
        Self {
            queue: VecDeque::new(),
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_refill.elapsed().as_secs_f64();
        let new_tokens = (elapsed * self.refill_rate as f64) as u32;

        self.tokens = (self.tokens + new_tokens).min(self.max_tokens);
        self.last_refill = Instant::now();
    }

    async fn execute(&mut self, request: Request) -> Result<Response> {
        self.refill();

        while self.tokens == 0 {
            sleep(Duration::from_millis(100)).await;
            self.refill();
        }

        self.tokens -= 1;
        request.send().await
    }
}
```

---

## Rate Limit Scenarios

### Scenario 1: High-Frequency Trading

**Requirements**:
- 500 order submissions per second
- 300 order cancellations per second

**Analysis**:
- Combined: 800 req/s (at limit)
- Per minute: 48,000 req/min (exceeds 17,250 limit)

**Solution**:
- Use batch operations where possible
- Stagger non-critical cancellations
- Consider order modifications instead of cancel/replace

### Scenario 2: Market Maker with Multiple Accounts

**Setup**:
- 5 accounts from same IP
- Each needs 200 req/min for order management

**Analysis**:
- Per account: 200 req/min (under 17,250 limit)
- Combined IP usage: 1,000 req/min (under 1,500 IP limit)

**Status**: ✅ Within limits

### Scenario 3: Data Collection

**Requirements**:
- Fetch account data every second
- 10 different endpoints

**Analysis**:
- 10 GET requests per second = 600 req/min
- Exactly at the limit for GET endpoints

**Solution**:
- Use WebSocket for real-time data (no rate limit)
- Only poll non-streaming data
- Increase polling interval where acceptable

---

## Monitoring Rate Limits

### Implementation Checklist

1. **Track request counts** per endpoint type
2. **Monitor HTTP 429 responses** (rate limit exceeded)
3. **Log rate limit violations** for analysis
4. **Implement metrics** for rate limit usage percentage
5. **Alert on approaching limits** (e.g., >80% usage)
6. **Use circuit breakers** to prevent cascading failures

### Example Metrics

```rust
struct RateLimitMetrics {
    orders_per_second: Gauge,
    orders_per_minute: Gauge,
    gets_per_second: Gauge,
    gets_per_minute: Gauge,
    ip_requests_per_minute: Gauge,
    rate_limit_errors: Counter,
}

impl RateLimitMetrics {
    fn record_request(&self, endpoint_type: EndpointType) {
        match endpoint_type {
            EndpointType::OrderMutation => {
                self.orders_per_second.inc();
                self.orders_per_minute.inc();
            }
            EndpointType::Get => {
                self.gets_per_second.inc();
                self.gets_per_minute.inc();
            }
        }
        self.ip_requests_per_minute.inc();
    }

    fn check_alerts(&self) {
        if self.orders_per_second.get() > 640 { // 80% of 800
            warn!("Approaching order rate limit");
        }

        if self.ip_requests_per_minute.get() > 1200 { // 80% of 1500
            warn!("Approaching IP rate limit");
        }
    }
}
```

---

## Comparison with Other Exchanges

| Exchange | Order Limit | Read Limit | Notes |
|----------|-------------|------------|-------|
| **Paradex** | 800 req/s | 120 req/s | Additional 1,500 req/min per IP |
| Binance | 100 req/10s | 1,200 req/min | Weight-based system |
| Bybit | 100 req/5s | 120 req/min | Different limits by endpoint |
| OKX | 60 req/2s | 20 req/2s | More restrictive |

**Paradex Advantages**:
- High order throughput (800 req/s)
- Batch operations count as 1 unit
- Separate pools for reads and writes

---

## WebSocket Rate Limits

### Connection Limits

**Note**: Specific WebSocket connection limits not documented. Typical constraints:

- **Max connections per IP**: Usually 5-10
- **Subscription limits**: Usually 100-200 channels per connection
- **Message rate**: Usually no hard limit for subscriptions

### Heartbeat Requirements

**Ping/Pong Mechanism**:
- Server sends ping every 55 seconds
- Client must respond with pong within 5 seconds
- Connection terminated if no pong response

**No rate limit** for heartbeat messages.

---

## Summary

1. **Public endpoints**: 1,500 req/min per IP (600 for auth/onboarding)
2. **Private order endpoints**: 800 req/s OR 17,250 req/min per account
3. **Private read endpoints**: 120 req/s OR 600 req/min per account
4. **IP constraint**: 1,500 req/min across all accounts from same IP
5. **Batch operations**: Count as 1 unit (50x efficiency)
6. **JWT refresh**: 600 req/min (plan refresh at 3-minute intervals)
7. **WebSocket**: No rate limits for subscriptions (preferred for real-time data)

---

## Additional Resources

- **Official Rate Limits**: https://docs.paradex.trade/api/general-information/rate-limits/api
- **Best Practices**: https://docs.paradex.trade/trading/api-best-practices
- **Python SDK**: https://github.com/tradeparadex/paradex-py (handles rate limiting internally)
