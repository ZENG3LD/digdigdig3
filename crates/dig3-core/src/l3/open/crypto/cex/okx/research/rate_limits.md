# OKX API v5 Rate Limits

## Overview

OKX enforces rate limits to protect against malicious usage and ensure fair API access. Rate limits are applied differently based on:
- Endpoint type (public vs private)
- Request method (REST vs WebSocket)
- User tier (regular vs VIP)

---

## REST API Rate Limits

### Public Endpoints

**Limit Basis:** IP Address

**Default Limit:** 20 requests per 2 seconds

**Applies to:**
- Market data endpoints (`/api/v5/market/*`)
- Public data endpoints (`/api/v5/public/*`)
- Unauthenticated requests

**Global Cap:** 250 requests per second (matches 500 requests / 2 seconds IP allowance)

### Private Endpoints

**Limit Basis:** User ID (each sub-account has individual User ID)

**Varies by Endpoint Category:**

| Endpoint Category | Rate Limit | Unit |
|-------------------|------------|------|
| Trading - Place Order | 60 per 2s | Instrument ID level |
| Trading - Cancel Order | Independent | Instrument ID level |
| Trading - Amend Order | Independent | Instrument ID level |
| Account - Balance | 10 per 2s | User ID |
| Account - Positions | 10 per 2s | User ID |
| Account - Config | 20 per 2s | User ID |
| Account - Leverage | 20 per 2s | User ID |
| Order Details | 20 per 2s | User ID |
| Pending Orders | 20 per 2s | User ID |

**Important Notes:**
1. **Independent Limits:** Rate limits for placing, amending, and canceling orders are **independent** from each other
2. **Instrument-Level Limits:** Trading endpoints (except Options) have limits per instrument ID
3. **Options Exception:** Rate limits for Options are based on **instrument family** level instead of individual instrument

---

## Sub-Account Level Limits

### Order Management Limits

**Maximum:** 1,000 requests per 2 seconds per sub-account

**Applies to:**
- Place order (`POST /api/v5/trade/order`)
- Place batch orders (`POST /api/v5/trade/batch-orders`)
- Amend order (`POST /api/v5/trade/amend-order`)
- Cancel order (`POST /api/v5/trade/cancel-order`)
- Cancel batch orders (`POST /api/v5/trade/cancel-batch-orders`)

**Error Code:** 50061 - "Order rate limit reached"

**Batch Counting:** Each order in a batch request counts individually toward the limit

**Example:**
- Placing 10 orders in a single batch request = 10 requests toward the 1,000/2s limit

---

## VIP Rate Limit Incentive

### Enhanced Limits for VIP5+

**Higher Limits:** Up to **10,000 requests per 2 seconds**

**Eligibility:** Based on fill ratio calculations from past 7 days of trading data

**Calculation:** Fill ratio considers:
- Order placement volume
- Successfully filled order volume
- Cancellation rates

**Benefits:**
- 10x higher sub-account level order limits
- Potentially higher per-endpoint limits
- Priority API queue placement

---

## WebSocket Rate Limits

### Connection-Based Limits

| Action | Rate Limit | Basis |
|--------|------------|-------|
| Login | 480 per hour | Per connection |
| Subscribe | 480 per hour | Per connection |
| Unsubscribe | 480 per hour | Per connection |
| New market subscriptions | 3 per second | Per connection |

**Combined Limit:** Total of login/subscribe/unsubscribe operations cannot exceed **480 per hour per connection**

**Subscription Data Limit:** Total length of multiple channels cannot exceed **64 KB**

### Channel-Specific Connection Limits

**Maximum:** 30 WebSocket connections per specific channel per sub-account

**Affected Channels:**
- `orders` - Order updates
- `account` - Account updates
- `positions` - Position updates
- `balance_and_position` - Combined balance and position updates
- `position-risk-warning` - Position risk warnings
- `account-greeks` - Account Greeks (options)

**Error Code:** `channel-conn-count-error` - "Channel connection count exceeded"

**Behavior:** System rejects the latest subscription when limit exceeded

### WebSocket Order Management

**Limit Basis:** User ID (same as REST)

**Rate Limits:**
- Same limits as REST API trading endpoints
- Place/amend/cancel operations share rate limits across REST and WebSocket

**Example:**
- 500 orders placed via REST + 500 orders via WebSocket = 1,000 total (hits sub-account limit)

### Connection Stability

**Keep-Alive Requirement:**
- Must send "ping" if no data arrives within 30 seconds
- Connection breaks automatically after 30 seconds of inactivity
- Applies to both subscription establishment and data push

---

## Rate Limit Headers (REST)

OKX does not currently provide rate limit headers in REST responses (unlike some exchanges).

**Best Practice:** Implement client-side rate limiting based on documented limits.

---

## Error Codes

### Rate Limit Errors

| Code | Message | Description |
|------|---------|-------------|
| 50011 | Rate limit reached. Please refer to API documentation | General rate limit exceeded |
| 50061 | Order rate limit reached | Sub-account order limit (1,000/2s) exceeded |
| 60012 | Invalid request | WebSocket subscription malformed |
| `channel-conn-count-error` | Channel connection count exceeded | WebSocket channel connection limit (30) exceeded |

---

## Rate Limit Strategies

### 1. Client-Side Rate Limiting

Implement token bucket or sliding window algorithms to stay within limits.

**Rust Example (Token Bucket):**
```rust
use std::time::{Duration, Instant};

pub struct RateLimiter {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window_seconds: u32) -> Self {
        let max_tokens = max_requests as f64;
        let refill_rate = max_tokens / window_seconds as f64;

        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    pub fn acquire(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
    }
}

// Usage
let mut limiter = RateLimiter::new(20, 2); // 20 requests per 2 seconds

if limiter.acquire() {
    // Make API request
} else {
    // Wait or queue request
}
```

### 2. Request Queuing

Queue requests and process them at a controlled rate.

**Example:**
```rust
use tokio::time::{sleep, Duration};
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct RequestQueue {
    interval: Duration,
    last_request: Arc<Mutex<Instant>>,
}

impl RequestQueue {
    pub fn new(requests_per_second: u32) -> Self {
        let interval = Duration::from_secs_f64(1.0 / requests_per_second as f64);

        Self {
            interval,
            last_request: Arc::new(Mutex::new(Instant::now() - interval)),
        }
    }

    pub async fn wait_for_slot(&self) {
        let mut last = self.last_request.lock().await;
        let now = Instant::now();
        let elapsed = now.duration_since(*last);

        if elapsed < self.interval {
            sleep(self.interval - elapsed).await;
        }

        *last = Instant::now();
    }
}
```

### 3. Batch Operations

Use batch endpoints to reduce request count.

**Available Batch Endpoints:**
- `POST /api/v5/trade/batch-orders` - Place up to 20 orders in one request
- `POST /api/v5/trade/cancel-batch-orders` - Cancel multiple orders in one request
- `POST /api/v5/trade/amend-batch-orders` - Amend multiple orders in one request

**Note:** Each order in batch still counts toward sub-account limit (1,000/2s)

### 4. WebSocket over REST

For real-time data, prefer WebSocket over polling REST endpoints.

**Benefits:**
- No rate limit for data reception (only subscription requests)
- Real-time updates without polling
- Reduced load on both client and server

### 5. Exponential Backoff

When rate limit is hit, implement exponential backoff before retrying.

**Example:**
```rust
use tokio::time::{sleep, Duration};

pub async fn request_with_backoff<F, T>(mut request_fn: F, max_retries: u32) -> Result<T, String>
where
    F: FnMut() -> Result<T, String>,
{
    let mut retry_count = 0;

    loop {
        match request_fn() {
            Ok(result) => return Ok(result),
            Err(err) if err.contains("50011") || err.contains("50061") => {
                if retry_count >= max_retries {
                    return Err(format!("Max retries exceeded: {}", err));
                }

                let backoff = Duration::from_millis(100 * 2u64.pow(retry_count));
                sleep(backoff).await;
                retry_count += 1;
            }
            Err(err) => return Err(err),
        }
    }
}
```

---

## Per-Endpoint Rate Limits

### Market Data Endpoints

| Endpoint | Rate Limit | Basis |
|----------|------------|-------|
| `GET /api/v5/market/ticker` | 20 per 2s | IP |
| `GET /api/v5/market/tickers` | 20 per 2s | IP |
| `GET /api/v5/market/books` | 20 per 2s | IP |
| `GET /api/v5/market/books-full` | 20 per 2s | IP |
| `GET /api/v5/market/candles` | 20 per 2s | IP |
| `GET /api/v5/market/history-candles` | 20 per 2s | IP |
| `GET /api/v5/market/trades` | 20 per 2s | IP |
| `GET /api/v5/public/time` | 20 per 2s | IP |
| `GET /api/v5/public/instruments` | 20 per 2s | IP |
| `GET /api/v5/public/funding-rate` | 20 per 2s | IP |
| `GET /api/v5/public/funding-rate-history` | 20 per 2s | IP |

### Trading Endpoints

| Endpoint | Rate Limit | Basis |
|----------|------------|-------|
| `POST /api/v5/trade/order` | 60 per 2s | Instrument ID |
| `POST /api/v5/trade/batch-orders` | 300 per 2s | User ID |
| `POST /api/v5/trade/cancel-order` | Independent | Instrument ID |
| `POST /api/v5/trade/cancel-batch-orders` | Independent | User ID |
| `POST /api/v5/trade/amend-order` | Independent | Instrument ID |
| `POST /api/v5/trade/amend-batch-orders` | Independent | User ID |
| `GET /api/v5/trade/order` | 20 per 2s | User ID |
| `GET /api/v5/trade/orders-pending` | 20 per 2s | User ID |
| `GET /api/v5/trade/orders-history` | 20 per 2s | User ID |
| `GET /api/v5/trade/orders-history-archive` | 20 per 2s | User ID |

### Account Endpoints

| Endpoint | Rate Limit | Basis |
|----------|------------|-------|
| `GET /api/v5/account/balance` | 10 per 2s | User ID |
| `GET /api/v5/account/positions` | 10 per 2s | User ID |
| `GET /api/v5/account/config` | 5 per 2s | User ID |
| `GET /api/v5/account/instruments` | 20 per 2s | User ID |
| `GET /api/v5/account/leverage-info` | 20 per 2s | User ID |
| `POST /api/v5/account/set-leverage` | 20 per 2s | User ID |
| `GET /api/v5/account/max-size` | 20 per 2s | User ID |
| `POST /api/v5/account/position/margin-balance` | 20 per 2s | User ID |
| `POST /api/v5/account/set-position-mode` | 5 per 2s | User ID |

---

## WebSocket Channels Rate Limits

### Public Channels (No Authentication)

| Channel | Subscription Limit | Update Frequency |
|---------|-------------------|------------------|
| `tickers` | 3 per second | Real-time |
| `books` | 3 per second | Real-time (400 levels) |
| `books5` | 3 per second | Real-time (5 levels) |
| `books-l2-tbt` | 3 per second | Tick-by-tick (400 levels) |
| `books50-l2-tbt` | 3 per second | Tick-by-tick (50 levels) |
| `trades` | 3 per second | Real-time |
| `candle{bar}` | 3 per second | Per bar close |
| `funding-rate` | 3 per second | Every 8 hours |

### Private Channels (Authentication Required)

| Channel | Connection Limit | Update Frequency |
|---------|-----------------|------------------|
| `account` | 30 per sub-account | Real-time |
| `positions` | 30 per sub-account | Real-time |
| `orders` | 30 per sub-account | Real-time |
| `balance_and_position` | 30 per sub-account | Real-time |
| `position-risk-warning` | 30 per sub-account | Real-time |
| `account-greeks` | 30 per sub-account | Real-time |

---

## Best Practices Summary

1. **Implement Client-Side Limiting:** Don't rely on server errors; limit requests before sending
2. **Use Batch Endpoints:** Reduce request count by batching operations
3. **Prefer WebSocket:** Use WebSocket for real-time data instead of polling
4. **Handle 50011/50061 Gracefully:** Implement exponential backoff on rate limit errors
5. **Monitor VIP Status:** Track eligibility for enhanced limits (VIP5+)
6. **Separate Limits:** Remember place/amend/cancel have independent limits
7. **Sub-Account Awareness:** Each sub-account has separate User ID-based limits
8. **Instrument-Level Tracking:** Track limits per instrument for trading endpoints
9. **WebSocket Hygiene:** Don't exceed 480 operations/hour or 30 connections per channel
10. **Queue Management:** Implement request queues for high-frequency operations

---

## Testing Rate Limits

### Test Plan

1. **Single Endpoint:** Send requests at increasing rates to find actual limit
2. **Concurrent Instruments:** Test instrument-level limits across different symbols
3. **Batch Operations:** Verify batch requests count correctly (each order = 1 request)
4. **WebSocket Limits:** Test 480/hour limit and 30 connection limit
5. **Mixed REST/WebSocket:** Confirm shared limits for trading operations
6. **Error Recovery:** Ensure proper handling of 50011/50061 errors

### Monitoring

Track these metrics in production:
- Requests per second per endpoint
- Rate limit errors (50011/50061) frequency
- Average backoff delay
- Queue depth
- WebSocket subscription count per channel

---

## Notes

1. **No Rate Limit Headers:** OKX doesn't provide `X-RateLimit-*` headers; track limits client-side
2. **Shared Limits:** REST and WebSocket trading operations share the same rate limits
3. **Sub-Account Isolation:** Each sub-account has independent limits
4. **Instrument vs User ID:** Trading uses instrument-level limits; most others use User ID
5. **VIP Benefits:** Significant improvement at VIP5+ (10x limit increase)
6. **Options Exception:** Options use instrument family instead of individual instrument for rate limiting
