# Phemex API Rate Limits

Complete rate limiting specification for V5 connector implementation.

## Rate Limit Architecture

Phemex implements **two-tier rate limiting**:

1. **IP-based Rate Limiting**: Hard limit per IP address
2. **User-based API Grouping**: Capacity pools shared by endpoint groups

## IP-Based Rate Limiting

### Global IP Limit

| Limit Type | Capacity | Time Window | Penalty |
|------------|----------|-------------|---------|
| IP Requests | 5,000 requests | 5 minutes | 5-minute block |

**Behavior:**
- Applies to ALL requests from a single IP address
- Shared across all users/accounts on that IP
- Exceeding limit results in 5-minute IP block
- Affects both REST API and WebSocket connections

**Response when exceeded:**
- HTTP Status: `429 Too Many Requests`
- All subsequent requests blocked for 5 minutes

**Important:** This is a hard limit separate from user API limits.

## User-Based API Groups

All API endpoints are divided into **three capacity groups**:

| Group | Capacity | Time Window | Endpoints |
|-------|----------|-------------|-----------|
| **Contract** | 500 requests | 1 minute | Contract trading operations |
| **SpotOrder** | 500 requests | 1 minute | Spot trading operations |
| **Others** | 100 requests | 1 minute | General queries, market data |

### How Grouping Works

- Each API call consumes capacity from its group
- Different endpoints have different **weights** (cost in capacity units)
- Groups are independent (Contract usage doesn't affect SpotOrder capacity)
- Capacity resets every minute (rolling window)

## Endpoint Weights

### Contract Group (500/minute capacity)

| Endpoint | Method | Weight | Notes |
|----------|--------|--------|-------|
| `/orders` | POST/PUT | 1 | Place order |
| `/orders/replace` | PUT | 1 | Amend order |
| `/orders/cancel` | DELETE | 1 | Cancel single order |
| `/orders` | DELETE | 1 per order | Bulk cancel |
| `/orders/all` | DELETE | 3 | Cancel all by symbol |
| `/g-orders/*` | Any | 1 | Hedged contract orders |
| `/accounts/accountPositions` | GET | 1 | Query positions |
| `/accounts/positions` | GET | 25 | Query with real-time PnL |
| `/orders/activeList` | GET | 1 | Query open orders |
| `/exchange/order/list` | GET | 1 | Query closed orders |
| `/exchange/order/trade` | GET | 1 | Query trades |
| `/positions/leverage` | PUT | 1 | Set leverage |
| `/positions/riskLimit` | PUT | 1 | Set risk limit |
| `/positions/assign` | POST | 1 | Assign position balance |

### SpotOrder Group (500/minute capacity)

| Endpoint | Method | Weight | Notes |
|----------|--------|--------|-------|
| `/spot/orders` | POST/PUT | 1 | Place/amend order |
| `/spot/orders` | DELETE | 2 | Cancel order |
| `/spot/orders/all` | DELETE | 2 | Cancel all by symbol |
| `/spot/orders/active` | GET | 1 | Query open orders |
| `/spot/orders` | GET | 1 | Query all orders |

### Others Group (100/minute capacity)

| Endpoint | Method | Weight | Notes |
|----------|--------|--------|-------|
| `/public/products` | GET | 1 | Product information |
| `/public/time` | GET | 1 | Server time |
| `/spot/wallets` | GET | 1 | Query spot balances |
| `/assets/transfer` | POST | 1 | Transfer funds |
| `/assets/transfer` | GET | 1 | Transfer history |
| `/md/orderbook` | GET | 1 | Order book |
| `/md/fullbook` | GET | 1 | Full order book |
| `/md/trade` | GET | 1 | Recent trades |
| `/md/ticker/24hr` | GET | 1 | 24h ticker |
| `/exchange/public/md/v2/kline` | GET | 10 | Kline/candlestick data |

**Note:** Kline queries have higher weight (10) due to data volume.

## Symbol-Level Rate Limiting (VIP/VAPI)

For VIP users accessing `https://vapi.phemex.com`:

### Enhanced Limits

| Group | Standard Capacity | Symbol-Specific |
|-------|-------------------|-----------------|
| Contract | 500/minute (global) | 500/minute (per symbol) |
| SpotOrder | 500/minute (global) | Not specified |

### How Symbol Limits Work

Each contract symbol gets its **own 500/minute capacity pool**:

**Example:**
- Place 500 orders on BTCUSD: Uses BTCUSD symbol pool
- Place 500 orders on ETHUSD: Uses ETHUSD symbol pool (independent)
- Both operations can occur simultaneously

**Additional Group:**
- `CONTACT_ALL_SYM`: For multi-symbol operations (e.g., cancel all orders across symbols)

## Rate Limit Response Headers

Every API response includes rate limit headers for tracking:

### Header Format

```
X-RateLimit-Remaining-<GROUP>: <remaining_capacity>
X-RateLimit-Capacity-<GROUP>: <total_capacity>
X-RateLimit-Retry-After-<GROUP>: <reset_seconds>
```

### Example Headers

**Contract Group:**
```
X-RateLimit-Remaining-CONTRACT: 498
X-RateLimit-Capacity-CONTRACT: 500
X-RateLimit-Retry-After-CONTRACT: 15
```

**SpotOrder Group:**
```
X-RateLimit-Remaining-SPOTORDER: 495
X-RateLimit-Capacity-SPOTORDER: 500
X-RateLimit-Retry-After-SPOTORDER: 8
```

**Others Group:**
```
X-RateLimit-Remaining-OTHERS: 97
X-RateLimit-Capacity-OTHERS: 100
X-RateLimit-Retry-After-OTHERS: 22
```

### Header Field Meanings

| Header | Description |
|--------|-------------|
| `Remaining` | Capacity left in current time window |
| `Capacity` | Total capacity for this group |
| `Retry-After` | Seconds until capacity resets (on 429 error) |

## Rate Limit Exceeded Response

When rate limit is exceeded:

**HTTP Status Code:**
```
429 Too Many Requests
```

**Response Headers:**
```
X-RateLimit-Retry-After-<GROUP>: 42
```

**Response Body:**
```json
{
  "code": 429,
  "msg": "Too many requests",
  "data": null
}
```

**Action Required:**
- Wait for `Retry-After` seconds before retrying
- Check which group was exceeded
- Reduce request frequency for that group

## WebSocket Rate Limits

### Connection Limits

| Limit Type | Capacity | Description |
|------------|----------|-------------|
| Concurrent Connections | 5 per user | Maximum simultaneous WebSocket connections |
| Subscriptions per Connection | 20 | Maximum channels per WebSocket |
| Request Throttle | 20 req/s | Maximum request rate per connection |

### WebSocket IP Limit

```
wss://ws.phemex.com: 200 requests / 5 minutes per IP
```

### Heartbeat Requirements

| Parameter | Value | Description |
|-----------|-------|-------------|
| Heartbeat Interval | < 30 seconds | Maximum time between pings |
| Recommended Interval | 5 seconds | Suggested ping frequency |
| Timeout Action | Disconnect | Server drops connection if no heartbeat |

**Heartbeat Message:**
```json
{
  "id": 1234,
  "method": "server.ping",
  "params": []
}
```

**Expected Response:**
```json
{
  "id": 1234,
  "result": "pong"
}
```

## Testnet Rate Limits

**Testnet API:** `https://testnet-api.phemex.com`

| Limit | Capacity |
|-------|----------|
| Total Requests | 500 / 5 minutes |

**Note:** All requests share a single 500/5min pool on testnet (no group separation).

## VIP/Institutional Rate Limits

Higher rate limits available for:
- Institutional users
- Market makers
- VIP account holders

**Contact:** VIP@phemex.com to request higher limits

**Typical enhancements:**
- Increased capacity per group
- Symbol-level rate limiting (vapi.phemex.com)
- Dedicated infrastructure
- Priority API access

## Rate Limit Best Practices

### 1. Track Remaining Capacity

```rust
use std::collections::HashMap;

#[derive(Debug)]
pub struct RateLimiter {
    remaining: HashMap<String, u32>,
    capacity: HashMap<String, u32>,
}

impl RateLimiter {
    pub fn update_from_headers(&mut self, headers: &HashMap<String, String>) {
        for (key, value) in headers {
            if key.starts_with("x-ratelimit-remaining-") {
                let group = key.strip_prefix("x-ratelimit-remaining-")
                    .unwrap()
                    .to_uppercase();
                if let Ok(remaining) = value.parse::<u32>() {
                    self.remaining.insert(group, remaining);
                }
            }
            if key.starts_with("x-ratelimit-capacity-") {
                let group = key.strip_prefix("x-ratelimit-capacity-")
                    .unwrap()
                    .to_uppercase();
                if let Ok(capacity) = value.parse::<u32>() {
                    self.capacity.insert(group, capacity);
                }
            }
        }
    }

    pub fn should_throttle(&self, group: &str, threshold: f32) -> bool {
        if let (Some(&remaining), Some(&capacity)) =
            (self.remaining.get(group), self.capacity.get(group)) {
            let usage_ratio = 1.0 - (remaining as f32 / capacity as f32);
            usage_ratio > threshold
        } else {
            false
        }
    }
}
```

### 2. Implement Adaptive Throttling

```rust
pub struct AdaptiveThrottle {
    rate_limiter: RateLimiter,
    delay_ms: HashMap<String, u64>,
}

impl AdaptiveThrottle {
    pub fn calculate_delay(&self, group: &str) -> u64 {
        if self.rate_limiter.should_throttle(group, 0.9) {
            // 90% capacity used - slow down significantly
            1000
        } else if self.rate_limiter.should_throttle(group, 0.75) {
            // 75% capacity used - moderate slowdown
            500
        } else if self.rate_limiter.should_throttle(group, 0.5) {
            // 50% capacity used - slight slowdown
            100
        } else {
            // Plenty of capacity
            0
        }
    }

    pub async fn wait_if_needed(&self, group: &str) {
        let delay = self.calculate_delay(group);
        if delay > 0 {
            tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;
        }
    }
}
```

### 3. Handle 429 Responses

```rust
pub async fn execute_with_retry<F, T>(
    func: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: Fn() -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, Error>>>>,
{
    let mut retries = 0;

    loop {
        match func().await {
            Ok(result) => return Ok(result),
            Err(Error::RateLimitExceeded { retry_after }) => {
                if retries >= max_retries {
                    return Err(Error::MaxRetriesExceeded);
                }

                let wait_time = retry_after.unwrap_or(60);
                tokio::time::sleep(
                    tokio::time::Duration::from_secs(wait_time)
                ).await;

                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 4. Batch Operations When Possible

Instead of:
```rust
// BAD: 100 individual requests
for order_id in order_ids {
    cancel_order(order_id).await?;
}
```

Use:
```rust
// GOOD: 1 bulk request
cancel_all_orders_by_symbol(symbol).await?;
```

### 5. Prioritize WebSocket for Real-time Data

**Don't poll via REST API:**
```rust
// BAD: Consumes rate limit
loop {
    let orderbook = get_orderbook(symbol).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

**Use WebSocket subscription:**
```rust
// GOOD: No rate limit impact
ws.subscribe("orderbook", symbol).await?;
// Receive updates via WebSocket stream
```

### 6. Cache Static Data

```rust
use std::time::{Duration, Instant};

pub struct ProductCache {
    products: Option<ProductsResponse>,
    last_fetch: Option<Instant>,
    ttl: Duration,
}

impl ProductCache {
    pub async fn get_products(&mut self, api: &ApiClient) -> Result<&ProductsResponse, Error> {
        let should_refresh = self.last_fetch
            .map(|t| t.elapsed() > self.ttl)
            .unwrap_or(true);

        if should_refresh {
            self.products = Some(api.get_products().await?);
            self.last_fetch = Some(Instant::now());
        }

        Ok(self.products.as_ref().unwrap())
    }
}
```

## Rate Limit Calculation Examples

### Example 1: High-Frequency Trading

**Scenario:** Place 10 orders per second on BTCUSD

**Calculation:**
- 10 orders/second × 60 seconds = 600 orders/minute
- Contract group capacity: 500/minute
- **Result:** WILL EXCEED LIMIT

**Solution:**
- Reduce to 8 orders/second (480/minute)
- Use VIP access with symbol-level limits
- Batch orders when possible

### Example 2: Market Data Polling

**Scenario:** Poll orderbook + ticker for 10 symbols every second

**Calculation:**
- (Orderbook + Ticker) × 10 symbols = 20 requests/second
- 20 × 60 = 1,200 requests/minute
- Others group capacity: 100/minute
- **Result:** WILL EXCEED LIMIT

**Solution:**
- Use WebSocket subscriptions instead
- If REST required, poll every 10 seconds (120/minute still too high)
- Fetch only changed symbols, not all 10

### Example 3: Mixed Operations

**Scenario:**
- 5 order placements/second (Contract)
- 2 balance queries/second (Others)
- 1 kline request/second (Others, weight=10)

**Calculation:**

Contract group:
- 5 × 60 = 300/minute (within 500 limit) ✓

Others group:
- Balance: 2 × 60 = 120/minute
- Klines: 1 × 60 × 10 weight = 600/minute
- Total: 720/minute (exceeds 100 limit) ✗

**Solution:**
- Reduce kline requests to 1 every 10 seconds (6/minute × 10 = 60)
- New total: 120 + 60 = 180/minute (still exceeds)
- Further reduce balance queries to 1 every 2 seconds (30/minute)
- Final total: 30 + 60 = 90/minute ✓

## Monitoring Rate Limits

### Log Rate Limit Usage

```rust
pub fn log_rate_limits(headers: &HashMap<String, String>) {
    for group in ["CONTRACT", "SPOTORDER", "OTHERS"] {
        if let Some(remaining) = headers.get(&format!("x-ratelimit-remaining-{}", group.to_lowercase())) {
            if let Some(capacity) = headers.get(&format!("x-ratelimit-capacity-{}", group.to_lowercase())) {
                let usage_pct = 100.0 * (1.0 - remaining.parse::<f64>().unwrap() / capacity.parse::<f64>().unwrap());
                log::info!("{} rate limit: {:.1}% used", group, usage_pct);
            }
        }
    }
}
```

### Alert on High Usage

```rust
pub fn check_rate_limit_health(rate_limiter: &RateLimiter) {
    for group in ["CONTRACT", "SPOTORDER", "OTHERS"] {
        if rate_limiter.should_throttle(group, 0.8) {
            log::warn!("{} group at 80% capacity - consider throttling", group);
        }
        if rate_limiter.should_throttle(group, 0.95) {
            log::error!("{} group at 95% capacity - throttling required!", group);
        }
    }
}
```

## Summary Table

| Limit Type | Capacity | Window | Scope | Penalty |
|------------|----------|--------|-------|---------|
| IP Global | 5,000 | 5 min | Per IP | 5-min block |
| Contract Group | 500 | 1 min | Per user | 429 error |
| SpotOrder Group | 500 | 1 min | Per user | 429 error |
| Others Group | 100 | 1 min | Per user | 429 error |
| Symbol (VIP) | 500 | 1 min | Per symbol | 429 error |
| WebSocket Connections | 5 | - | Per user | Connection refused |
| WebSocket Subscriptions | 20 | - | Per connection | Subscribe failed |
| WebSocket Requests | 20/s | 1 sec | Per connection | Throttle |
| Testnet Total | 500 | 5 min | All requests | 429 error |
