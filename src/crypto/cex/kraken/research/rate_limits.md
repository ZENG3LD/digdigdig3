# Kraken API Rate Limits

Kraken implements comprehensive rate limiting across REST and WebSocket APIs to ensure system stability and fair usage.

---

## Spot REST API Rate Limits

### Call Counter Mechanism

Every user has a **call counter** that:
- Starts at **0**
- Increases with each API call
- Decreases over time based on verification tier
- Is maintained separately per API key

**When counter exceeds maximum**: All subsequent calls are rate limited until counter decreases.

---

## Rate Limit Tiers

Rate limits vary by Kraken account verification level:

| Verification Tier | Max Counter | Decay Rate | Time to Full Recovery |
|-------------------|-------------|------------|----------------------|
| **Starter** | 15 | -0.33/sec | ~45 seconds |
| **Intermediate** | 20 | -0.5/sec | 40 seconds |
| **Pro** | 20 | -1/sec | 20 seconds |

**Decay Rate**: Counter decreases by this amount every second
**Time to Full Recovery**: Time for counter to go from max to 0

---

## Endpoint-Specific Costs

Different endpoints increment the counter by different amounts:

| Endpoint Type | Counter Cost |
|---------------|--------------|
| **Most Public Endpoints** | +0 (no cost) |
| **Most Private Endpoints** | +1 |
| **Ledger Queries** | +2 |
| **Trade History** | +2 |

### Free Public Endpoints (Cost: 0)

These endpoints don't affect your rate limit counter:
- `GET /0/public/Time`
- `GET /0/public/SystemStatus`
- `GET /0/public/Assets`
- `GET /0/public/AssetPairs`

### Public Endpoints with IP-Based Limits

Some public endpoints have separate rate limiting:

**Ticker and OHLC Data**:
- Rate limited by **IP address** and **currency pair**
- Recommended: 1 request per second or less

**Order Book (Depth)**:
- Rate limited by **IP address** only
- Recommended: 1 request per second or less

**Trades Endpoint**:
- Rate limited by **IP address** and **currency pair**

---

## Trading Engine Rate Limits

Beyond REST API limits, the **matching engine** enforces additional limits on order placement and cancellation.

### Order Rate Limits

Each account has a **points system** for trading actions:

**Actions and Point Costs**:
- **Place Order**: Consumes points based on order size
- **Cancel Order**: Consumes points
- **Order Execution**: Consumes points

**Points Regeneration**:
- Points replenish every second
- Maximum points available per second varies by pair and account tier

**Rate Limit Structure**:
- Separate limits per currency pair
- Limits apply to all order flow (REST, WebSocket, FIX)
- When exhausted, new orders/cancels are rejected temporarily

---

## AddOrder and CancelOrder Limiters

AddOrder and CancelOrder operate on **separate rate limiters** from general REST endpoints.

**Characteristics**:
- Independent of the general call counter
- Based on trading engine capacity
- May have stricter limits than general API calls

---

## Shared Limits (Master/Sub-Accounts)

**Important**: Master accounts and subaccounts share the same default trading rate limits.

- All subaccounts contribute to the master account's rate limit
- Counter is **shared** across master and all subaccounts
- Each API key still has its own independent counter for REST calls

---

## Error Messages

### REST API Rate Limit Exceeded

```json
{
  "error": ["EAPI:Rate limit exceeded"]
}
```

**Cause**: Call counter exceeded maximum for your tier

**Solution**:
- Wait for counter to decay
- Reduce request frequency
- Upgrade verification tier for higher limits

---

### Throttling Error

```json
{
  "error": ["EService: Throttled: 1705752000"]
}
```

**Cause**: Too many concurrent requests

**Solution**: Retry after the UNIX timestamp indicated

---

### Trading Rate Limit

```json
{
  "error": ["EOrder:Rate limit exceeded"]
}
```

**Cause**: Trading engine rate limit reached

**Solution**:
- Wait for points to regenerate (typically 1-2 seconds)
- Reduce order placement frequency
- Batch operations where possible

---

## Futures REST API Rate Limits

### /derivatives Endpoints

**Rate Limit**: 500 points per 10 seconds

**Point System**:
- Each endpoint has a point cost (typically 1-5 points)
- Counter resets every 10 seconds
- Exceeding 500 points results in rate limit error

**Example**:
```
GET /accounts        - 1 point
POST /sendorder      - 5 points
GET /openpositions   - 2 points
```

If you make 100 sendorder requests in 10 seconds (100 × 5 = 500 points), you'll hit the limit.

---

## WebSocket Rate Limits

### Connection Limits

**Spot WebSocket**:
- Connection rate limit: Not explicitly documented
- Recommend: Reuse connections, don't reconnect frequently

**Futures WebSocket**:
- Must send **ping every 60 seconds** to keep connection alive
- No explicit connection limit, but avoid excessive reconnections

### Subscription Limits

**Spot WebSocket v2**:
- Multiple subscriptions per connection allowed
- Recommended: Batch subscriptions in single message when possible

**Level 3 Order Book**:
- Maximum **200 symbols** per WebSocket connection

### Message Rate Limits

WebSocket feeds are throttled at the server side:
- Ticker updates: Throttled to ~1 second intervals (Futures)
- Book updates: Real-time, but may be batched during high activity

---

## Best Practices

### 1. Implement Exponential Backoff

```rust
async fn retry_with_backoff<F, T>(
    mut f: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: FnMut() -> Result<T, Error>,
{
    let mut retry = 0;
    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if is_rate_limit_error(&e) && retry < max_retries => {
                let delay = 2u64.pow(retry) * 1000; // Exponential backoff
                tokio::time::sleep(Duration::from_millis(delay)).await;
                retry += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_rate_limit_error(error: &Error) -> bool {
    // Check if error is rate limit related
    error.to_string().contains("Rate limit exceeded") ||
    error.to_string().contains("Throttled")
}
```

---

### 2. Track Call Counter Locally

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

struct RateLimiter {
    counter: Arc<Mutex<f64>>,
    max_counter: f64,
    decay_rate: f64,
    last_update: Arc<Mutex<Instant>>,
}

impl RateLimiter {
    fn new(tier: VerificationTier) -> Self {
        let (max_counter, decay_rate) = match tier {
            VerificationTier::Starter => (15.0, 0.33),
            VerificationTier::Intermediate => (20.0, 0.5),
            VerificationTier::Pro => (20.0, 1.0),
        };

        RateLimiter {
            counter: Arc::new(Mutex::new(0.0)),
            max_counter,
            decay_rate,
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    async fn update_counter(&self) {
        let mut counter = self.counter.lock().await;
        let mut last_update = self.last_update.lock().await;

        let elapsed = last_update.elapsed().as_secs_f64();
        *counter = (*counter - (elapsed * self.decay_rate)).max(0.0);
        *last_update = Instant::now();
    }

    async fn can_make_request(&self, cost: f64) -> bool {
        self.update_counter().await;
        let counter = self.counter.lock().await;
        *counter + cost <= self.max_counter
    }

    async fn record_request(&self, cost: f64) {
        self.update_counter().await;
        let mut counter = self.counter.lock().await;
        *counter += cost;
    }

    async fn wait_if_needed(&self, cost: f64) {
        while !self.can_make_request(cost).await {
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }
}

enum VerificationTier {
    Starter,
    Intermediate,
    Pro,
}
```

**Usage**:
```rust
let limiter = RateLimiter::new(VerificationTier::Pro);

// Before making a request
limiter.wait_if_needed(1.0).await;
let response = make_api_call().await?;
limiter.record_request(1.0).await;
```

---

### 3. Batch Requests When Possible

Instead of:
```rust
// BAD: 100 separate requests
for pair in pairs {
    get_ticker(pair).await?;
}
```

Use:
```rust
// GOOD: 1 request for all pairs
let pairs_str = pairs.join(",");
get_ticker(&pairs_str).await?;
```

---

### 4. Use WebSocket for Real-Time Data

**Instead of polling**:
```rust
// BAD: Polling every second
loop {
    let ticker = get_ticker("XBTUSD").await?;
    process(ticker);
    sleep(Duration::from_secs(1)).await;
}
```

**Use WebSocket**:
```rust
// GOOD: WebSocket subscription
let mut ws = connect_websocket().await?;
subscribe_ticker(&mut ws, "BTC/USD").await?;

while let Some(msg) = ws.next().await {
    let ticker = parse_ticker(msg)?;
    process(ticker);
}
```

---

### 5. Minimize Ledger and History Queries

Ledger and trade history queries cost **2 points** each:

```rust
// Use pagination wisely
async fn get_all_trades(
    start: i64,
    end: i64,
) -> Result<Vec<Trade>, Error> {
    let mut all_trades = Vec::new();
    let mut offset = 0;

    loop {
        // This costs 2 points per call
        let response = query_trades_history(start, end, offset).await?;

        all_trades.extend(response.trades);

        if response.count < 50 { // Adjust based on response
            break;
        }

        offset += response.count;

        // Add delay to avoid rate limits
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    Ok(all_trades)
}
```

---

### 6. Monitor Rate Limit Headers (if available)

While Kraken doesn't currently expose rate limit info in response headers, you should:
- Track your own counter locally
- Log rate limit errors
- Adjust request frequency dynamically

---

### 7. Use Independent API Keys

For different services/processes:
```rust
// Service A: Market data polling
let api_key_market_data = "key_1";

// Service B: Order management
let api_key_trading = "key_2";

// Service C: Account monitoring
let api_key_account = "key_3";
```

Each API key maintains its own call counter, allowing parallel operations.

---

## Rate Limiting Strategy

### Recommended Approach

```rust
use std::sync::Arc;
use tokio::sync::Semaphore;

struct KrakenClient {
    rate_limiter: RateLimiter,
    // Allow max 5 concurrent requests
    semaphore: Arc<Semaphore>,
}

impl KrakenClient {
    async fn request<T>(
        &self,
        endpoint: &str,
        cost: f64,
    ) -> Result<T, Error> {
        // Wait for rate limit
        self.rate_limiter.wait_if_needed(cost).await;

        // Acquire semaphore (limit concurrency)
        let _permit = self.semaphore.acquire().await?;

        // Make request
        let response = self.http_client
            .get(endpoint)
            .send()
            .await?;

        // Record request
        self.rate_limiter.record_request(cost).await;

        // Parse response
        let result = response.json::<T>().await?;
        Ok(result)
    }
}
```

---

## Testing Rate Limits

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limit_recovery() {
        let limiter = RateLimiter::new(VerificationTier::Pro);

        // Make 20 requests (hit limit)
        for _ in 0..20 {
            limiter.record_request(1.0).await;
        }

        // Should be at max
        assert!(!limiter.can_make_request(1.0).await);

        // Wait for decay (Pro tier: 1 point/sec)
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Should have recovered ~5 points
        assert!(limiter.can_make_request(1.0).await);
    }

    #[tokio::test]
    async fn test_ledger_query_cost() {
        let limiter = RateLimiter::new(VerificationTier::Pro);

        // Ledger query costs 2 points
        limiter.wait_if_needed(2.0).await;
        limiter.record_request(2.0).await;

        let counter = limiter.counter.lock().await;
        assert_eq!(*counter, 2.0);
    }
}
```

---

## Summary Table

| Limit Type | Scope | Limit | Recovery |
|------------|-------|-------|----------|
| **REST Call Counter** | Per API key | 15-20 calls | 0.33-1.0/sec |
| **Public Ticker/OHLC** | Per IP + Pair | ~1/sec | N/A |
| **Public Depth** | Per IP | ~1/sec | N/A |
| **Trading Engine** | Per account + pair | Points/sec | 1 second |
| **Futures /derivatives** | Per account | 500 points/10sec | Every 10 sec |
| **WebSocket Ping** | Per connection | 1/60sec minimum | N/A |
| **Level 3 WebSocket** | Per connection | 200 symbols max | N/A |

---

## Key Takeaways

1. **Track your counter locally** - Don't rely on error messages
2. **Upgrade verification tier** - Pro tier has best limits
3. **Use WebSocket for real-time data** - Avoids REST polling
4. **Separate API keys** - Independent counters for different services
5. **Implement backoff** - Handle rate limits gracefully
6. **Batch requests** - Single request for multiple pairs when possible
7. **Ledger queries are expensive** - Cost 2 points each
8. **Trading has separate limits** - Independent from REST counter
9. **Futures has 10-second window** - 500 points per 10 seconds
10. **Monitor and log** - Track your usage patterns

---

## Additional Resources

- Official Rate Limits Guide: https://docs.kraken.com/api/docs/guides/spot-rest-ratelimits/
- Trading Engine Limits: https://docs.kraken.com/api/docs/guides/spot-ratelimits/
- Futures Rate Limits: https://docs.kraken.com/api/docs/guides/futures-rate-limits/
