# KuCoin API Rate Limits

Comprehensive documentation of KuCoin API rate limiting system for implementing robust connector with proper rate limit handling.

## Overview

KuCoin uses **REST Rate Limit 2.0** system based on resource pools with quota allocation determined by VIP level. Rate limits are tracked per UID (authenticated) or per IP (public endpoints), with quotas resetting every 30 seconds.

### Core Concepts

- **Resource Pools**: Separate quota pools for different API categories
- **Weight System**: Each endpoint consumes specific weight units from quota
- **30-Second Window**: Quotas reset every 30s from first request
- **VIP-Based Quotas**: Higher VIP levels receive larger quotas
- **Independent Accounting**: Sub-accounts and master accounts have separate rate limits

---

## 1. Rate Limit Types

### 1.1 Dimension-Based Classification

| Dimension | Scope | Description |
|-----------|-------|-------------|
| **UID-based** | Private endpoints | Rate limits tracked per user ID (authenticated requests) |
| **IP-based** | Public endpoints | Rate limits tracked per IP address (unauthenticated requests) |

**Key Points:**
- Spot, Futures, Management, Earn, and CopyTrading pools are UID-based
- Public endpoints use IP-based limiting
- Sub-account and master account limits are independent
- Multiple IP addresses can be used to avoid public endpoint limits

### 1.2 Resource Pool Classification

KuCoin organizes rate limits into **7 resource pools**:

1. **Unified Account** - Unified trading account operations
2. **Spot** - Spot trading (including Margin)
3. **Futures** - Futures trading operations
4. **Management** - Account management operations
5. **Earn** - Earn product operations
6. **CopyTrading** - Copy trading operations
7. **Public** - Public market data (IP-based)

Each pool has independent quota allocation based on VIP level.

---

## 2. Spot Rate Limits

### 2.1 Public Endpoints (IP-based)

Public market data endpoints are rate-limited per IP address.

**Common Public Endpoint Weights:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Get Ticker (Level 1) | `GET /api/v1/market/orderbook/level1` | 2 | Public |
| Get Part Order Book | `GET /api/v1/market/orderbook/level2_20` | - | Public |
| Get Full Order Book | `GET /api/v1/market/orderbook/level2` | - | Public |

**Best Practices for Public Endpoints:**
- Prefer WebSocket for high-frequency market data
- Use multiple IP addresses if needed
- Request partial order books (level2_20, level2_100) for faster response

### 2.2 Private Endpoints (UID-based)

Private spot trading endpoints consume from the Spot resource pool.

**Order Management Weights:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Place Order | `POST /api/v1/orders` | 2 | Spot |
| Cancel Order by OrderId | `DELETE /api/v1/hf/orders/{orderId}` | 1 | Spot |
| Modify Order | `POST /api/v1/orders/alter` | 1 | Spot |
| Place Stop Order | `POST /api/v1/stop-order` | 2 | Spot |
| Get Order List | `GET /api/v1/orders` | 2 | Spot |
| Get Stop Orders List | `GET /api/v1/stop-order` | 8 | Spot |

**Account Information Weights:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Get Account Detail | `GET /api/v1/accounts/{accountId}` | 5 | Spot |
| Get Sub-Account Balance | `GET /api/v2/sub/user` | 15 | Spot |
| Get All Sub-Accounts Balance (V2) | `GET /api/v2/sub-accounts` | 20 | Spot |

**Convert Endpoints:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Cancel Convert Limit Order | `DELETE /api/v3/convert/order` | 5 | Spot |

### 2.3 Order Rate Limits

**Active Order Limits:**
- Maximum **2,000 active orders** per account
- Maximum **200 active orders** per trading pair
- Maximum **20 untriggered stop orders** per trading pair

### 2.4 Example: VIP 5 Spot Quota

When user VIP level is 5:
- **Total Spot quota**: 16,000 per 30s
- After placing first order (weight 2): Remaining = 15,998
- After placing second order (weight 2): Remaining = 15,996
- After getting order list (weight 2): Remaining = 15,994

---

## 3. Futures Rate Limits

### 3.1 Public Endpoints

Futures public endpoints follow similar IP-based limiting as Spot.

**Common Futures Public Endpoint Weights:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Get All Symbols | `GET /api/v1/contracts/active` | - | Public |
| Get Full OrderBook | `GET /api/v1/level2/snapshot` | - | Public |
| Get Part OrderBook | `GET /api/v1/level2/depth{depth}` | - | Public |

### 3.2 Private Endpoints (UID-based)

Futures trading endpoints consume from the Futures resource pool.

**Order Management Weights:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Place Order (Add Order) | `POST /api/v1/orders` | 2 | Futures |
| Add Order Test | `POST /api/v1/orders/test` | 2 | Futures |
| Add Take Profit/Stop Loss | `POST /api/v1/orders/tpsl` | 2 | Futures |
| Cancel All Orders | `DELETE /api/v3/orders` | 10 | Futures |
| Cancel All Orders V1 (deprecated) | `DELETE /api/v1/orders` | 800 | Futures |

**Query Endpoint Limits:**

| Endpoint | Rate Limit | Pool |
|----------|------------|------|
| Get Order List | 30 times/3s | Futures |
| Get Fills List | 9 times/3s | Futures |

**Account Information Weights:**

| Endpoint | Method | Weight | Pool |
|----------|--------|--------|------|
| Get All Sub-Accounts Balance (Futures) | `GET /api/v1/account-overview-all` | 6 | Futures |

### 3.3 Historical Rate Limit Notes

**Legacy System (Pre-2021):**
- Place Order (POST /api/v1/orders): 30 per 3s per UserID
- This was replaced by the current weight-based system

---

## 4. Headers for Rate Limit Info

### 4.1 Response Headers

Every API response includes three rate limit headers:

| Header Name | Description | Format |
|-------------|-------------|--------|
| `gw-ratelimit-limit` | Total resource pool quota | Integer (e.g., 16000) |
| `gw-ratelimit-remaining` | Resource pool remaining quota | Integer (e.g., 15998) |
| `gw-ratelimit-reset` | Quota reset countdown (milliseconds) | Integer (e.g., 25000) |

### 4.2 Example Header Values

```http
gw-ratelimit-limit: 16000
gw-ratelimit-remaining: 15998
gw-ratelimit-reset: 25000
```

**Interpretation:**
- Total quota: 16,000 per 30s
- Remaining quota: 15,998
- Reset in: 25,000ms (25 seconds)

### 4.3 Code Examples for Header Parsing

**Rust Example:**
```rust
// Parse headers from response
let limit = response.headers()
    .get("gw-ratelimit-limit")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(0);

let remaining = response.headers()
    .get("gw-ratelimit-remaining")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.parse::<u32>().ok())
    .unwrap_or(0);

let reset_ms = response.headers()
    .get("gw-ratelimit-reset")
    .and_then(|v| v.to_str().ok())
    .and_then(|s| s.parse::<u64>().ok())
    .unwrap_or(0);
```

**Go Example (from official SDK):**
```go
limit := resp.Header.Get("gw-ratelimit-limit")
remaining := resp.Header.Get("gw-ratelimit-remaining")
reset := resp.Header.Get("gw-ratelimit-reset")
```

### 4.4 Using Headers for Rate Limit Tracking

**Implementation Strategy:**
1. Parse response headers after each request
2. Track remaining quota in real-time
3. Calculate safe request rate based on remaining quota and reset time
4. Implement adaptive throttling before hitting limit

**Calculation Example:**
```rust
let safe_requests_per_sec = remaining as f64 / (reset_ms as f64 / 1000.0);
```

---

## 5. Error Responses for Rate Limits

### 5.1 Rate Limit Exceeded Error

When quota is exhausted within 30-second window:

**HTTP Status Code:** `429`

**REST API Error Response:**
```json
{
  "code": "429000",
  "msg": "Too Many Requests"
}
```

**WebSocket Error Response (with details):**
```json
{
  "code": "429000",
  "id": "order-1741590647179",
  "op": "futures.order",
  "msg": "Too many requests in a short period of time, please retry later.",
  "inTime": 1741589852255,
  "outTime": 1741589852355,
  "rateLimit": {
    "limit": 1600,
    "reset": 15244,
    "remaining": 1528
  }
}
```

**Error Code:** `429000`

### 5.2 Two Types of 429000 Errors

#### Type 1: Personal Rate Limit Exceeded
- Triggered when user's resource pool quota is depleted
- **Response headers INCLUDE** rate limit information
- Headers show `gw-ratelimit-remaining: 0`
- Headers show reset countdown in `gw-ratelimit-reset`

#### Type 2: Server Overload
- Triggered by KuCoin server capacity limits (stand-alone capacity limit)
- **Response headers DO NOT INCLUDE** personal rate limit information
- Does **not** count toward your rate limit history
- Using high-frequency VIP accounts can reduce these errors

### 5.3 Distinguishing Error Types

**Check for Headers:**
```rust
if status == 429 && error_code == "429000" {
    if headers.contains_key("gw-ratelimit-reset") {
        // Type 1: User quota exhausted
        let wait_ms = headers.get("gw-ratelimit-reset").unwrap();
        // Wait for exact reset time
    } else {
        // Type 2: Server overload
        // Use exponential backoff or retry immediately
    }
}
```

### 5.4 Retry-After Behavior

**Retry Strategy:**
- When receiving 429000, check response headers
- If headers present: Wait for `gw-ratelimit-reset` milliseconds
- If headers absent (server overload): Retry immediately or use exponential backoff
- According to KuCoin support: 429000 should be handled by retrying immediately (for server overload)

**Recommended Implementation:**
```rust
match error_code {
    "429000" => {
        if let Some(reset_ms) = headers.get("gw-ratelimit-reset") {
            // Personal rate limit - wait for reset
            sleep(Duration::from_millis(reset_ms.parse()?));
        } else {
            // Server overload - exponential backoff
            let backoff = min(100 * 2_u64.pow(attempt), 30000);
            sleep(Duration::from_millis(backoff));
        }
    }
}
```

---

## 6. VIP Level Quotas

### 6.1 Resource Pool Quotas by VIP Level

Quota allocation per 30-second window varies by VIP tier:

| VIP Level | Unified Account | Spot (incl. Margin) | Futures | Management | Earn | CopyTrading | Public (IP) |
|-----------|----------------|---------------------|---------|------------|------|-------------|-------------|
| **VIP 0** | 2,000 | 4,000 | 2,000 | 2,000 | 2,000 | 2,000 | 2,000 |
| **VIP 5** | 7,000 | 16,000 | - | - | - | - | - |
| **VIP 12** | 20,000 | 40,000 | - | - | - | - | - |

**Notes:**
- VIP 0 is the base tier (non-VIP users)
- VIP 12 is the highest tier
- Spot quota for VIP 0 is 4,000 (confirmed from multiple sources)
- Exact quotas for VIP 1-4, 6-11 not publicly documented
- Complete quota tables may be available in KuCoin account dashboard
- Professional traders can request custom higher limits

### 6.2 Example: VIP 5 User

**Spot Trading Quota:**
- Total quota: 16,000 per 30s
- Placing 1,000 orders (weight 2 each): Consumes 2,000, Remaining = 14,000
- Getting 100 order lists (weight 2 each): Consumes 200, Remaining = 13,800

### 6.3 Broker Users

**Special Quotas:**
- Each Broker user has a resource pool quota of **1,000** per pool
- Applies to Broker-specific endpoints

### 6.4 Applying for Higher Limits

**Eligibility:**
- Professional traders
- Market makers
- High-volume traders

**Application Process:**
1. Email: `api@kucoin.com`
2. Include:
   - KuCoin account ID
   - Reason for higher limits
   - Approximate trading volume
   - Trading strategy description (if applicable)

---

## 7. WebSocket Rate Limits

### 7.1 Connection Limits

**Classic Mode:**
- **Private interfaces**: ≤ 800 concurrent connections per UID
- **Public interfaces**: ≤ 800 concurrent connections per IP (for public endpoints)
- **Connection rate**: Maximum 30 new connections per minute

**Unified Account Mode:**
- **Connections per IP**: ≤ 256 concurrent connections
- **Connections per UID**: ≤ 800 (same as Classic)

**Historical Note:**
- Connection limit increased from 500 → 800 per UID in recent updates

### 7.2 Subscription Limits

**Topics per Connection:**
- **Spot/Margin**: Maximum 400 subscribed topics per connection
- **Futures**: No limit on topics
- **Batch subscription**: Maximum 100 topics per subscription request

**Subscription Rate:**
- Maximum 100 messages per 10 seconds per connection
- If violated, additional subscriptions are ignored

### 7.3 Message Rate Limits

**Sending Messages:**
- Maximum 100 messages per 10 seconds per connection
- Recommended write interval: 300ms between messages

### 7.4 Token Expiration

**WebSocket Token Lifetime:**
- Tokens expire after **24 hours**
- Stream is stopped immediately upon expiration
- Must obtain new token and reconnect

### 7.5 WebSocket Rate Limit Information

WebSocket API responses include rate limit information in JSON body:

```json
{
  "rateLimit": {
    "limit": 1600,
    "reset": 15244,
    "remaining": 1528
  }
}
```

**Fields:**
- `limit`: Total quota
- `reset`: Reset countdown (milliseconds)
- `remaining`: Remaining quota

### 7.6 Best Practices for WebSocket

**Multiple Connections:**
- For > 400 topics on Spot: Create multiple connections
- Conservative limit: 200-300 topics per connection to avoid threshold
- Distribute topics across connections for redundancy

**Connection Management:**
- Monitor token expiration (24h lifetime)
- Implement auto-reconnection with new token
- Respect 300ms write interval between messages
- Track message rate (100 per 10s)

**Master/Sub-Account Strategy:**
- Master and sub-accounts are independent (different UIDs)
- Each has separate 800 connection limit
- Can create multiple sub-accounts to multiply connection capacity

---

## 8. Best Practices

### 8.1 Weight System Understanding

**Weight Assignment Logic:**
- Simple queries (get by ID): Weight 1-2
- List queries: Weight 2-8 (depends on result size)
- Complex operations (batch cancel): Weight 10+
- Deprecated endpoints: Very high weight (e.g., 800) to discourage use

**Strategic Endpoint Selection:**
- Prefer low-weight endpoints when possible
- Use WebSocket instead of REST for frequent updates
- Request partial data (e.g., level2_20) instead of full datasets

### 8.2 Quota Management Strategies

**Track Remaining Quota:**
```rust
struct RateLimitTracker {
    limit: u32,
    remaining: u32,
    reset_at: Instant,
}

impl RateLimitTracker {
    fn can_request(&self, weight: u32) -> bool {
        if Instant::now() >= self.reset_at {
            return true; // Quota has reset
        }
        self.remaining >= weight
    }

    fn update_from_headers(&mut self, headers: &HeaderMap) {
        self.limit = parse_header(headers, "gw-ratelimit-limit");
        self.remaining = parse_header(headers, "gw-ratelimit-remaining");
        let reset_ms = parse_header(headers, "gw-ratelimit-reset");
        self.reset_at = Instant::now() + Duration::from_millis(reset_ms);
    }
}
```

### 8.3 Request Throttling

**Adaptive Rate Limiting:**
1. Monitor `gw-ratelimit-remaining` header
2. If remaining < 20% of limit: Reduce request rate
3. If remaining < 5% of limit: Pause until reset
4. Calculate safe request rate dynamically

**Example Implementation:**
```rust
fn calculate_safe_delay(&self) -> Duration {
    let time_until_reset = self.reset_at - Instant::now();
    let time_until_reset_secs = time_until_reset.as_secs_f64();

    if time_until_reset_secs <= 0.0 {
        return Duration::from_millis(0);
    }

    // Distribute remaining quota over remaining time
    let safe_requests_per_sec = self.remaining as f64 / time_until_reset_secs;
    let delay_secs = 1.0 / safe_requests_per_sec;

    Duration::from_secs_f64(delay_secs)
}
```

### 8.4 Error Handling Strategy

**429000 Error Handling:**
```rust
fn handle_rate_limit_error(
    &self,
    headers: &HeaderMap,
    attempt: u32,
) -> Result<Duration, Error> {
    match headers.get("gw-ratelimit-reset") {
        Some(reset_ms) => {
            // Personal rate limit exceeded
            let wait_time = Duration::from_millis(reset_ms.parse()?);
            Ok(wait_time)
        }
        None => {
            // Server overload - exponential backoff
            let backoff = Duration::from_millis(100 * 2_u64.pow(attempt));
            Ok(backoff.min(Duration::from_secs(30)))
        }
    }
}
```

### 8.5 Sub-Account Strategy

**Leveraging Independent Limits:**
- Master account and sub-accounts have independent rate limits
- For high-volume trading:
  - Create multiple sub-accounts
  - Distribute API requests across accounts
  - Effectively multiply available quota

**Example:**
- Master account: 16,000 quota (VIP 5)
- 5 sub-accounts × 16,000 each = 80,000 total quota
- Total available: 96,000 per 30s across all accounts

### 8.6 Public Endpoint Optimization

**IP-Based Limit Workarounds:**
- Bind multiple IPv4/IPv6 addresses to same server
- Rotate requests across different IPs
- Each IP gets independent quota for public endpoints

**Prefer WebSocket for Market Data:**
- WebSocket has separate connection limits
- More efficient for real-time data
- Reduces REST API quota consumption

### 8.7 Monitoring and Alerting

**Track Metrics:**
- Quota utilization percentage
- Number of 429 errors per hour
- Average remaining quota
- Time until quota reset

**Alert Thresholds:**
- Warning: 80% quota consumed
- Critical: 95% quota consumed
- Error: 429000 errors increasing

### 8.8 Documentation References

Always check official documentation for:
- Latest endpoint weights (may change)
- VIP level quota updates
- New resource pools
- Rate limit policy changes

**Primary Documentation:**
- Main Rate Limit Page: https://www.kucoin.com/docs-new/rate-limit
- REST API Rate Limits: https://www.kucoin.com/docs/basic-info/request-rate-limit/rest-api
- WebSocket Rate Limits: https://www.kucoin.com/docs/basic-info/request-rate-limit/websocket

---

## 9. Implementation Checklist

### 9.1 Essential Features

- [ ] Parse rate limit headers from every response
- [ ] Track remaining quota per resource pool
- [ ] Calculate quota reset time
- [ ] Implement request throttling based on remaining quota
- [ ] Handle 429000 errors with appropriate retry logic
- [ ] Distinguish between personal limit and server overload errors
- [ ] Log rate limit metrics for monitoring

### 9.2 Advanced Features

- [ ] Adaptive request rate based on quota utilization
- [ ] Predictive throttling to avoid hitting limits
- [ ] Multiple sub-account support for quota multiplication
- [ ] IP rotation for public endpoint optimization
- [ ] WebSocket fallback for high-frequency data
- [ ] Real-time rate limit dashboard
- [ ] Automatic quota reset detection

### 9.3 Testing Considerations

- [ ] Test behavior when quota exhausted
- [ ] Verify retry logic for 429000 errors
- [ ] Confirm quota reset after 30 seconds
- [ ] Test concurrent requests across multiple pools
- [ ] Validate header parsing for all endpoints
- [ ] Simulate server overload scenarios

---

## 10. Summary Table

### Quick Reference: Common Endpoint Weights

| Category | Endpoint | Method | Weight | Pool |
|----------|----------|--------|--------|------|
| **Spot Trading** |
| | Place Order | POST | 2 | Spot |
| | Cancel Order | DELETE | 1 | Spot |
| | Modify Order | POST | 1 | Spot |
| | Get Order List | GET | 2 | Spot |
| | Place Stop Order | POST | 2 | Spot |
| | Get Stop Orders | GET | 8 | Spot |
| **Futures Trading** |
| | Place Order | POST | 2 | Futures |
| | Add TP/SL Order | POST | 2 | Futures |
| | Cancel All Orders | DELETE | 10 | Futures |
| **Market Data** |
| | Get Ticker | GET | 2 | Public |
| **Account Info** |
| | Get Account Detail | GET | 5 | Spot |
| | Get Sub-Account Balance | GET | 15 | Spot |
| | Get All Sub-Accounts (V2) | GET | 20 | Spot |
| | Get Futures Sub-Accounts | GET | 6 | Futures |

### Error Codes Quick Reference

| Code | Status | Meaning | Action |
|------|--------|---------|--------|
| 429000 | 429 | Too Many Requests | Check headers: if present wait for reset, if absent use exponential backoff |

### Header Reference

| Header | Example | Description |
|--------|---------|-------------|
| `gw-ratelimit-limit` | 16000 | Total quota per 30s |
| `gw-ratelimit-remaining` | 15998 | Remaining quota |
| `gw-ratelimit-reset` | 25000 | Milliseconds until reset |

### WebSocket Limits Summary

| Limit Type | Value |
|------------|-------|
| Connections per UID (Classic) | ≤ 800 |
| Connections per IP (Unified) | ≤ 256 |
| New connections per minute | 30 |
| Topics per connection (Spot) | ≤ 400 |
| Topics per subscription request | ≤ 100 |
| Messages per 10 seconds | ≤ 100 |
| Token lifetime | 24 hours |

---

## Sources

Research compiled from official KuCoin API documentation:

- [Basic Info - Request Rate Limit (REST API)](https://www.kucoin.com/docs/basic-info/request-rate-limit/rest-api)
- [Rate Limit - Main Documentation](https://www.kucoin.com/docs-new/rate-limit)
- [WebSocket Rate Limits](https://www.kucoin.com/docs/basic-info/request-rate-limit/websocket)
- [WebSocket Introduction - Unified Account](https://www.kucoin.com/docs-new/websocket-api/base-info/introduction-uta)
- [Error Response Documentation](https://www.kucoin.com/docs/basic-info/connection-method/request/error-response)
- [Add Order - Spot Trading](https://www.kucoin.com/docs-new/rest/spot-trading/orders/add-order)
- [Add Order - Futures Trading](https://www.kucoin.com/docs-new/rest/futures-trading/orders/add-order)
- [Place Order - Spot Stop Orders](https://www.kucoin.com/docs/rest/spot-trading/stop-order/place-order)
- [Get Ticker - Spot Market Data](https://www.kucoin.com/docs-new/rest/spot-trading/market-data/get-ticker)
- [Cancel Order by OrderId](https://www.kucoin.com/docs-new/rest/spot-trading/orders/cancel-order-by-orderld)
- [Cancel All Orders - Futures](https://www.kucoin.com/docs-new/rest/futures-trading/orders/cancel-all-orders)
- [Apply for Higher Request Rate Limit](https://www.kucoin.com/docs/basic-info/request-rate-limit/apply-for-higher-request-rate-limit)
- [KuCoin Universal SDK - Rate Limit Headers](https://github.com/Kucoin/kucoin-universal-sdk/blob/main/sdk/golang/internal/infra/default_transport.go)
- [Adjustment of Spot and Futures API Request Limit (News)](https://www.kucoin.com/news/en-adjustment-of-the-spot-and-futures-api-request-limit)
- [KuCoin API Request Rate Limit Upgrade (News)](https://www.kucoin.com/news/en-kucoin-api-request-rate-limit-upgrade)
- [KuCoin Request Limit for API Upgrade (Announcement)](https://www.kucoin.com/announcement/en-kucoin-api-request-rate-limit-upgrade)

---

**Document Version:** 2.0
**Last Updated:** 2026-01-20
**Research Compiled By:** Claude Code Research Agent
