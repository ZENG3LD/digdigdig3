# Bybit V5 API Rate Limits

Comprehensive documentation of Bybit V5 API rate limiting system for implementing robust connector with proper rate limit handling.

**Research Date:** 2026-01-20

## Overview

Bybit V5 uses a **rolling per-second window per user ID** basis for authenticated endpoints and **per-IP** basis for public endpoints. Rate limits vary by endpoint and operation type.

### Core Concepts

- **User ID Based**: Private endpoints tracked per user ID
- **IP Based**: Public endpoints tracked per IP address
- **Rolling Window**: Per-second rolling window (not fixed intervals)
- **Endpoint Specific**: Each endpoint has its own rate limit
- **Response Headers**: Three headers inform you of current limit status

---

## 1. Rate Limit Types

### 1.1 Dimension-Based Classification

| Dimension | Scope | Description |
|-----------|-------|-------------|
| **User ID** | Private endpoints | Rate limits tracked per user ID (authenticated requests) |
| **IP** | Public endpoints | Rate limits tracked per IP address (unauthenticated requests) |

**Key Points:**
- Private endpoints (trading, account) use UID-based limiting
- Public endpoints (market data) use IP-based limiting
- Multiple API keys under same account share rate limits
- Using multiple IPs can help avoid public endpoint limits

### 1.2 Default IP Limits

**HTTP REST API:**
- **600 requests within a 5-second window** per IP
- This equals approximately **120 requests per second** per IP
- Applies to all domains (main, testnet, bytick)

**Violation Response:**
- HTTP Status: `403 Forbidden`
- Error message: "access too frequent"
- **Automatic ban**: 10 minutes

**Recovery:**
- Terminate all HTTP sessions immediately
- Wait at least 10 minutes before resuming
- Consider implementing exponential backoff

### 1.3 WebSocket Limits

**Connection Limits:**
- Maximum **500 connections** per 5-minute window per IP
- Maximum **1,000 total connections** per IP for market data
- Counted separately by market type (spot, linear, inverse, option)

---

## 2. REST API Rate Limits

### 2.1 Response Headers

Every API response includes three rate limit headers:

| Header Name | Description | Format |
|-------------|-------------|--------|
| `X-Bapi-Limit` | Current endpoint rate limit | Integer (e.g., 50) |
| `X-Bapi-Limit-Status` | Remaining requests in window | Integer (e.g., 48) |
| `X-Bapi-Limit-Reset-Timestamp` | Time when limit resets (seconds) | Unix timestamp (seconds) |

### 2.2 Example Header Values

```
X-Bapi-Limit: 50
X-Bapi-Limit-Status: 45
X-Bapi-Limit-Reset-Timestamp: 1702617480
```

**Interpretation:**
- Limit: 50 requests per second for this endpoint
- Remaining: 45 requests available
- Reset at: Unix timestamp 1702617480 (in seconds)

### 2.3 Using Headers for Rate Limit Tracking

**Implementation Strategy:**
1. Parse response headers after each request
2. Track remaining quota in real-time
3. Calculate safe request rate based on remaining quota
4. Implement adaptive throttling before hitting limit

**Calculation Example:**
```rust
let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
let time_until_reset = reset_timestamp - current_time;
let safe_requests_per_sec = remaining / time_until_reset.max(1);
```

---

## 3. Endpoint-Specific Limits

### 3.1 Market Data Endpoints (Public)

| Endpoint | Rate Limit | Notes |
|----------|------------|-------|
| GET /v5/market/tickers | Shared IP limit | 600 per 5s |
| GET /v5/market/orderbook | Shared IP limit | 600 per 5s |
| GET /v5/market/kline | Shared IP limit | 600 per 5s |
| GET /v5/market/recent-trade | Shared IP limit | 600 per 5s |
| GET /v5/market/instruments-info | Shared IP limit | 600 per 5s |

**Notes:**
- All public endpoints share the 600 per 5-second IP limit
- Consider using WebSocket for high-frequency market data
- Rotating IPs can help distribute load

### 3.2 Trading Endpoints (Private)

| Endpoint | Rate Limit | Category |
|----------|------------|----------|
| POST /v5/order/create | 10 req/s (spot), 10 req/s (linear), 10 req/s (inverse) | Per category per user |
| POST /v5/order/amend | 10 req/s | Per category per user |
| POST /v5/order/cancel | 10 req/s | Per category per user |
| POST /v5/order/cancel-all | 10 req/s | Per category per user |
| POST /v5/order/create-batch | 10 req/s | Per category per user |
| POST /v5/order/amend-batch | 10 req/s | Per category per user |
| POST /v5/order/cancel-batch | 10 req/s | Per category per user |

**Notes:**
- Trading endpoints have separate limits per category (spot, linear, inverse)
- Batch operations count as single request but process multiple orders
- Each category maintains independent rate limit counter

### 3.3 Query Endpoints (Private)

| Endpoint | Rate Limit | Notes |
|----------|------------|-------|
| GET /v5/order/realtime | 50 req/s | Active orders |
| GET /v5/order/history | 10 req/s | Historical orders |
| GET /v5/execution/list | 10 req/s | Trade history |
| GET /v5/position/list | 50 req/s | Position info |
| GET /v5/account/wallet-balance | 50 req/s | Balance queries |
| GET /v5/account/fee-rate | 10 req/s | Fee rates |

### 3.4 Asset Management (Private)

| Endpoint | Rate Limit | Notes |
|----------|------------|-------|
| POST /v5/asset/transfer/inter-transfer | 5 req/s | Internal transfers |
| GET /v5/asset/transfer/query-inter-transfer-list | 60 req/min | Transfer history |
| GET /v5/asset/deposit/query-record | 300 req/min | Deposit records |
| GET /v5/asset/withdraw/query-record | 300 req/min | Withdrawal records |

---

## 4. Error Responses for Rate Limits

### 4.1 Rate Limit Exceeded Error

**HTTP Status Code:** `403` (for IP limits) or depends on endpoint

**Error Response:**
```json
{
  "retCode": 10006,
  "retMsg": "Too many visits!",
  "result": {},
  "retExtInfo": {},
  "time": 1702617474601
}
```

**Error Code:** `10006`

### 4.2 IP Ban Response

When IP is banned for 10 minutes:

**HTTP Status Code:** `403`

**Error Message:** "access too frequent"

**Response Action:**
- Terminate all active HTTP sessions
- Wait 10 minutes minimum
- Implement exponential backoff
- Consider rotating IPs

### 4.3 Retry Strategy

**Recommended Implementation:**
```rust
match error_code {
    10006 => {
        // Rate limit exceeded
        if let Some(reset_ts) = headers.get("X-Bapi-Limit-Reset-Timestamp") {
            let wait_time = reset_ts - current_time();
            sleep(Duration::from_secs(wait_time));
        } else {
            // No reset timestamp, use exponential backoff
            sleep(Duration::from_millis(backoff_delay));
        }
    }
    _ => {
        // Other errors
    }
}
```

---

## 5. Enhanced Rate Limits

### 5.1 VIP Level Enhancements

Bybit does not publicly document VIP-specific rate limits, but higher VIP levels generally receive:
- Increased rate limits for trading endpoints
- Better execution priority
- Lower fees

**Application Process:**
- Contact Bybit support for institutional accounts
- Provide trading volume and API usage requirements

### 5.2 Institutional API Rate Limits

**Starting August 13, 2025**, Bybit rolled out enhanced rate limits for institutional traders:

**Benefits:**
- Higher rate limits for high-frequency trading clients
- Customized limits based on trading needs
- Priority support

**Eligibility:**
- Professional traders
- Market makers
- High-volume API users

**Application:**
- Contact Bybit institutional support
- Email: Not specified in public docs (check official channels)
- Provide: Trading strategy, volume, and API requirements

### 5.3 SDK-Enhanced Limits

Some official SDKs report enhanced limits:

**JavaScript SDK Users:**
- Spot & perpetuals: **400 requests per second**
- Significantly higher than standard limits
- Automatically applied when using official SDK

---

## 6. Best Practices

### 6.1 Rate Limit Tracking

**Track Remaining Quota:**
```rust
struct RateLimitTracker {
    limit: u32,
    remaining: u32,
    reset_at: SystemTime,
    endpoint: String,
}

impl RateLimitTracker {
    fn can_request(&self) -> bool {
        if SystemTime::now() >= self.reset_at {
            return true; // Quota has reset
        }
        self.remaining > 0
    }

    fn update_from_headers(&mut self, headers: &HeaderMap) {
        self.limit = parse_header(headers, "X-Bapi-Limit");
        self.remaining = parse_header(headers, "X-Bapi-Limit-Status");
        let reset_ts = parse_header(headers, "X-Bapi-Limit-Reset-Timestamp");
        self.reset_at = UNIX_EPOCH + Duration::from_secs(reset_ts);
    }
}
```

### 6.2 Request Throttling

**Adaptive Rate Limiting:**
1. Monitor `X-Bapi-Limit-Status` header
2. If remaining < 20% of limit: Reduce request rate
3. If remaining < 5% of limit: Pause until reset
4. Calculate safe request rate dynamically

**Example Implementation:**
```rust
fn calculate_safe_delay(&self) -> Duration {
    let time_until_reset = self.reset_at.duration_since(SystemTime::now())
        .unwrap_or(Duration::from_secs(1));
    let time_until_reset_secs = time_until_reset.as_secs_f64();

    if time_until_reset_secs <= 0.0 {
        return Duration::from_millis(0);
    }

    // Distribute remaining quota over remaining time
    let safe_requests_per_sec = self.remaining as f64 / time_until_reset_secs;
    if safe_requests_per_sec <= 0.0 {
        return time_until_reset; // Wait for reset
    }

    let delay_secs = 1.0 / safe_requests_per_sec;
    Duration::from_secs_f64(delay_secs.max(0.01)) // Min 10ms delay
}
```

### 6.3 Batch Operations

**Use batch endpoints when possible:**
- `POST /v5/order/create-batch` - Create up to 10 orders in single request
- `POST /v5/order/amend-batch` - Amend up to 10 orders
- `POST /v5/order/cancel-batch` - Cancel up to 10 orders

**Benefits:**
- Single rate limit consumption
- Atomic operations
- Reduced network overhead

**Trade-off:**
- Partial failures possible (some orders succeed, others fail)
- More complex error handling

### 6.4 WebSocket for Market Data

**Prefer WebSocket over REST for:**
- Real-time price updates
- Orderbook streaming
- Trade execution streams
- Position updates

**Advantages:**
- No REST rate limit consumption
- Lower latency
- More efficient bandwidth usage

**WebSocket Rate Limits:**
- 500 new connections per 5 minutes
- 1,000 total concurrent connections per IP

### 6.5 IP Rotation

**For public endpoint optimization:**
- Bind multiple IPv4/IPv6 addresses to same server
- Rotate requests across different IPs
- Each IP gets independent 600 per 5-second quota

**Example:**
- Single IP: 120 req/s
- 5 IPs: 600 req/s total capacity

### 6.6 Monitoring and Alerting

**Track Metrics:**
- Requests per second per endpoint
- Remaining quota percentage
- Number of 10006 errors per hour
- Time until quota reset
- IP ban occurrences

**Alert Thresholds:**
- Warning: 80% quota consumed
- Critical: 95% quota consumed
- Error: 10006 errors increasing
- Emergency: IP ban occurred

---

## 7. Implementation Checklist

### 7.1 Essential Features

- [ ] Parse rate limit headers from every response
- [ ] Track remaining quota per endpoint
- [ ] Calculate quota reset time
- [ ] Implement request throttling based on remaining quota
- [ ] Handle error code 10006 with appropriate retry logic
- [ ] Handle 403 IP ban with 10-minute cooldown
- [ ] Log rate limit metrics for monitoring

### 7.2 Advanced Features

- [ ] Adaptive request rate based on quota utilization
- [ ] Predictive throttling to avoid hitting limits
- [ ] IP rotation for public endpoint optimization
- [ ] WebSocket fallback for high-frequency data
- [ ] Batch operations for order management
- [ ] Real-time rate limit dashboard
- [ ] Automatic quota reset detection

### 7.3 Testing Considerations

- [ ] Test behavior when quota exhausted
- [ ] Verify retry logic for 10006 errors
- [ ] Confirm quota reset after 1 second
- [ ] Test concurrent requests across multiple endpoints
- [ ] Validate header parsing for all endpoints
- [ ] Simulate IP ban scenario (10-minute cooldown)

---

## 8. Summary Table

### Quick Reference: Common Endpoint Limits

| Category | Endpoint | Method | Rate Limit (req/s) | Scope |
|----------|----------|--------|-------------------|-------|
| **Market Data** |
| | Get Tickers | GET | 120 (IP) | Per IP |
| | Get Orderbook | GET | 120 (IP) | Per IP |
| | Get Kline | GET | 120 (IP) | Per IP |
| **Trading** |
| | Create Order | POST | 10 | Per UID per category |
| | Amend Order | POST | 10 | Per UID per category |
| | Cancel Order | POST | 10 | Per UID per category |
| | Cancel All | POST | 10 | Per UID per category |
| | Create Batch | POST | 10 | Per UID per category |
| **Query** |
| | Get Open Orders | GET | 50 | Per UID |
| | Get Order History | GET | 10 | Per UID |
| | Get Positions | GET | 50 | Per UID |
| | Get Balance | GET | 50 | Per UID |
| **Asset** |
| | Internal Transfer | POST | 5 | Per UID |
| | Transfer History | GET | 1 (60/min) | Per UID |

### Error Codes Quick Reference

| Code | HTTP | Meaning | Action |
|------|------|---------|--------|
| 10006 | Varies | Too many visits | Wait for reset, use exponential backoff |
| - | 403 | IP ban (access too frequent) | Terminate sessions, wait 10 minutes |

### Header Reference

| Header | Example | Description |
|--------|---------|-------------|
| `X-Bapi-Limit` | 50 | Endpoint rate limit per second |
| `X-Bapi-Limit-Status` | 45 | Remaining requests in current window |
| `X-Bapi-Limit-Reset-Timestamp` | 1702617480 | Unix timestamp when limit resets (seconds) |

---

## 9. Differences from KuCoin

| Feature | Bybit V5 | KuCoin |
|---------|----------|---------|
| Rate limit window | Per-second rolling | Per-30-second window |
| Response headers | X-Bapi-* | gw-ratelimit-* |
| Reset time format | Unix timestamp (seconds) | Milliseconds until reset |
| Error code | 10006 | 429000 |
| IP ban duration | 10 minutes | Not specified |
| Weight system | Per-endpoint limits | Weight-based quota pools |
| VIP levels | Not publicly documented | Detailed quota tables |
| Batch operations | Up to 10 orders | No batch endpoints |

---

## Sources

Research compiled from official Bybit V5 API documentation:

- [Rate Limit Rules | Bybit API Documentation](https://bybit-exchange.github.io/docs/v5/rate-limit)
- [Bybit V5 Changelog](https://bybit-exchange.github.io/docs/changelog/v5)
- [Updates on API Rate Limit for Perpetual and Futures Contracts](https://announcements.bybit.com/article/updates-on-api-rate-limit-for-perpetual-and-futures-contracts-bltb570f5552f51de97/)
- [Update: Bybit enhances API rate limits for Institutional Traders](https://announcements.bybit.global/article/update-bybit-enhances-api-rate-limits-for-institutional-traders-bltbbbf60de757d074e/)
- [Exclusive: Higher rate limits for bybit-api JavaScript SDK](https://github.com/tiagosiebler/bybit-api/issues/458)
- [Bybit API Cheat Sheet](https://vezgo.com/blog/bybit-api-cheat-sheet-for-developers/)

---

**Document Version:** 1.0
**Last Updated:** 2026-01-20
**Key Finding:** Per-second rolling window with endpoint-specific limits, simpler than KuCoin's weight system
