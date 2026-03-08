# Vertex Protocol Rate Limits

Vertex Protocol enforces rate limits on all API endpoints to ensure system stability and fair usage.

## Rate Limit System

### Limit Types

1. **Per-IP Limits**: Applied to specific IP addresses
2. **Aggregate Limits**: Combined across all endpoints
3. **Weighted Limits**: Some endpoints count more toward limits

### Enforcement

- Limits enforced by gateway server
- Exceeding limits returns `RATE_LIMIT_EXCEEDED` error
- No automatic retry-after headers (manually track)

## Global Aggregate Limit

**All Endpoints Combined**: 600 requests per 10 seconds

This is the total allowance across ALL query and execute endpoints.

## Query Endpoint Limits

### Market Data Queries

| Endpoint | Limit | Window | Weight | Per IP |
|----------|-------|--------|--------|--------|
| **status** | 60 | 1s | 1 | No |
| **contracts** | 60 | 1s | 1 | No |
| **order** | 60 | 1s | 1 | No |
| **market_price** | 60 | 1s | 1 | No |
| **market_liquidity** | 60 | 1s | 1 | Yes (40/s) |
| **all_products** | 12 | 1s | 5 | No |

### Account Queries

| Endpoint | Limit | Window | Weight | Per IP |
|----------|-------|--------|--------|--------|
| **subaccount_info** | 40 | 10s | 10 | Yes |
| **fee_rates** | 30 | 1s | 2 | No |
| **subaccount_orders** | 30 | 1s | 2 | No |
| **max_withdrawable** | 12 | 1s | 5 | No |
| **max_order_size** | 40 | 1s | 1 | No |

### Indexer Queries

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| **indexer (all)** | 60 | 1s | Archive endpoint |
| **candlesticks** | 60 | 1s | Via indexer |
| **product_snapshots** | 60 | 1s | Via indexer |

## Execute Endpoint Limits

### Trading Operations

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| **place_order (leveraged)** | 10 | 1s | Perps or margin spot |
| **place_order (no leverage)** | 5 | 10s | Spot without leverage |
| **cancel_orders** | 600 | 1s | High limit for mass cancels |
| **cancel_product_orders** | 2 | 1s | Cancel all for product |
| **cancel_and_place** | 10 | 1s | Same as place_order |

### Account Operations

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| **withdraw_collateral** | 10 | 10s | Estimated (not documented) |
| **liquidate_subaccount** | 10 | 1s | Liquidation bots |
| **mint_lp / burn_lp** | 10 | 10s | Liquidity operations |

## Weight System

Endpoints with weight > 1 count multiple times toward aggregate limit.

### Weight Values

- **Weight 1**: Most queries (status, order, market_price, etc.)
- **Weight 2**: Account queries (fee_rates, subaccount_orders)
- **Weight 5**: Heavy queries (all_products, max_withdrawable)
- **Weight 10**: Subaccount info (most expensive)

### Effective Limits with Weights

Example: `all_products` (weight 5)
- Base limit: 12 requests/second
- Counts as: 60 requests toward 600/10s aggregate
- Effective: Can call all_products 12x/s, but blocks other calls

## Per-IP Limits

Some endpoints have additional per-IP restrictions:

| Endpoint | Per-IP Limit | Window |
|----------|-------------|--------|
| **market_liquidity** | 40 | 1s |
| **subaccount_info** | 40 | 10s |

**Note**: These are LOWER than global limits, creating bottleneck for single IP.

## Rate Limit Tracking

### Client-Side Tracking

Implement token bucket or sliding window:

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

struct RateLimiter {
    max_requests: usize,
    window: Duration,
    requests: VecDeque<Instant>,
}

impl RateLimiter {
    fn new(max_requests: usize, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window: Duration::from_secs(window_seconds),
            requests: VecDeque::new(),
        }
    }

    fn check_and_add(&mut self) -> bool {
        let now = Instant::now();

        // Remove old requests outside window
        while let Some(&oldest) = self.requests.front() {
            if now.duration_since(oldest) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we can add
        if self.requests.len() < self.max_requests {
            self.requests.push_back(now);
            true
        } else {
            false
        }
    }

    fn wait_time(&self) -> Option<Duration> {
        if self.requests.len() < self.max_requests {
            return None;
        }

        let oldest = self.requests.front()?;
        let elapsed = Instant::now().duration_since(*oldest);

        if elapsed >= self.window {
            None
        } else {
            Some(self.window - elapsed)
        }
    }
}
```

### Multi-Endpoint Tracking

```rust
use std::collections::HashMap;

struct MultiRateLimiter {
    limiters: HashMap<String, RateLimiter>,
    aggregate: RateLimiter,
}

impl MultiRateLimiter {
    fn new() -> Self {
        let mut limiters = HashMap::new();

        // Market data queries
        limiters.insert("status".to_string(), RateLimiter::new(60, 1));
        limiters.insert("market_liquidity".to_string(), RateLimiter::new(60, 1));
        limiters.insert("all_products".to_string(), RateLimiter::new(12, 1));

        // Account queries
        limiters.insert("subaccount_info".to_string(), RateLimiter::new(40, 10));
        limiters.insert("fee_rates".to_string(), RateLimiter::new(30, 1));

        // Trading
        limiters.insert("place_order_lev".to_string(), RateLimiter::new(10, 1));
        limiters.insert("place_order_spot".to_string(), RateLimiter::new(5, 10));
        limiters.insert("cancel_orders".to_string(), RateLimiter::new(600, 1));

        Self {
            limiters,
            aggregate: RateLimiter::new(600, 10),
        }
    }

    async fn acquire(&mut self, endpoint: &str) -> Result<(), Duration> {
        // Check endpoint-specific limit
        if let Some(limiter) = self.limiters.get_mut(endpoint) {
            if !limiter.check_and_add() {
                if let Some(wait) = limiter.wait_time() {
                    return Err(wait);
                }
            }
        }

        // Check aggregate limit
        if !self.aggregate.check_and_add() {
            if let Some(wait) = self.aggregate.wait_time() {
                return Err(wait);
            }
        }

        Ok(())
    }
}
```

### Usage Example

```rust
async fn make_request(
    limiter: &mut MultiRateLimiter,
    endpoint: &str,
) -> Result<Response, Error> {
    loop {
        match limiter.acquire(endpoint).await {
            Ok(()) => break,
            Err(wait_duration) => {
                log::warn!(
                    "Rate limit hit for {}. Waiting {:?}",
                    endpoint,
                    wait_duration
                );
                tokio::time::sleep(wait_duration).await;
            }
        }
    }

    // Make actual request
    reqwest::get("...").await
}
```

## Weight-Based Tracking

Track weighted requests for aggregate limit:

```rust
fn get_endpoint_weight(endpoint: &str) -> usize {
    match endpoint {
        "subaccount_info" => 10,
        "all_products" | "max_withdrawable" => 5,
        "fee_rates" | "subaccount_orders" => 2,
        _ => 1,
    }
}

struct WeightedRateLimiter {
    max_weight: usize,
    window: Duration,
    requests: VecDeque<(Instant, usize)>, // (time, weight)
}

impl WeightedRateLimiter {
    fn current_weight(&self) -> usize {
        let now = Instant::now();
        self.requests
            .iter()
            .filter(|(time, _)| now.duration_since(*time) <= self.window)
            .map(|(_, weight)| weight)
            .sum()
    }

    fn check_and_add(&mut self, weight: usize) -> bool {
        let now = Instant::now();

        // Clean old requests
        while let Some((time, _)) = self.requests.front() {
            if now.duration_since(*time) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Check if we can add
        if self.current_weight() + weight <= self.max_weight {
            self.requests.push_back((now, weight));
            true
        } else {
            false
        }
    }
}
```

## Retry Strategy

### Exponential Backoff

```rust
async fn request_with_retry<F, T>(
    mut request_fn: F,
    max_retries: u32,
) -> Result<T, Error>
where
    F: FnMut() -> Pin<Box<dyn Future<Output = Result<T, Error>>>>,
{
    let mut retries = 0;
    let mut delay = Duration::from_millis(100);

    loop {
        match request_fn().await {
            Ok(response) => return Ok(response),
            Err(e) if is_rate_limit_error(&e) && retries < max_retries => {
                log::warn!("Rate limit hit. Retry {}/{}. Waiting {:?}",
                    retries + 1, max_retries, delay);

                tokio::time::sleep(delay).await;

                retries += 1;
                delay = std::cmp::min(delay * 2, Duration::from_secs(10));
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_rate_limit_error(error: &Error) -> bool {
    // Check if error contains "RATE_LIMIT_EXCEEDED"
    error.to_string().contains("RATE_LIMIT_EXCEEDED")
}
```

## WebSocket Rate Limits

### Connection Limits

- **Max Connections per Wallet**: 5 WebSocket connections
- **Heartbeat**: Ping every 30 seconds required
- **Subscription Limits**: No documented limit on subscriptions per connection

### WebSocket Best Practices

1. Use single connection for multiple subscriptions
2. Send ping frames every 25 seconds (5s buffer)
3. Limit to 3-4 connections per wallet (leave room)
4. Reconnect with exponential backoff

```rust
async fn websocket_heartbeat(ws: &mut WebSocketStream) {
    let mut interval = tokio::time::interval(Duration::from_secs(25));

    loop {
        interval.tick().await;
        if ws.send(Message::Ping(vec![])).await.is_err() {
            log::error!("Failed to send ping");
            break;
        }
    }
}
```

## Production Recommendations

### Request Prioritization

1. **Critical**: User orders, cancellations (execute immediately)
2. **High**: Account balances, open orders (cache 1-2s)
3. **Medium**: Market data, orderbook (cache 500ms)
4. **Low**: Product info, symbols (cache 1 hour)

### Optimization Strategies

1. **Cache aggressively**: Store frequently accessed data
2. **Batch queries**: Use all_products instead of individual queries
3. **Use WebSocket**: Subscribe to live updates instead of polling
4. **Request coalescing**: Combine multiple user requests
5. **Throttle UI updates**: Don't query on every render

### Example: Caching Layer

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

struct CachedData<T> {
    data: T,
    updated_at: Instant,
}

struct DataCache {
    symbols: Arc<RwLock<Option<CachedData<HashMap<String, u32>>>>>,
    orderbooks: Arc<RwLock<HashMap<u32, CachedData<Orderbook>>>>,
    balances: Arc<RwLock<HashMap<String, CachedData<Balance>>>>,
}

impl DataCache {
    async fn get_or_fetch<T, F>(
        &self,
        cache: &Arc<RwLock<Option<CachedData<T>>>>,
        ttl: Duration,
        fetch_fn: F,
    ) -> Result<T, Error>
    where
        T: Clone,
        F: FnOnce() -> Pin<Box<dyn Future<Output = Result<T, Error>>>>,
    {
        // Check cache
        {
            let cached = cache.read().await;
            if let Some(ref data) = *cached {
                if data.updated_at.elapsed() < ttl {
                    return Ok(data.data.clone());
                }
            }
        }

        // Fetch fresh data
        let fresh_data = fetch_fn().await?;

        // Update cache
        {
            let mut cached = cache.write().await;
            *cached = Some(CachedData {
                data: fresh_data.clone(),
                updated_at: Instant::now(),
            });
        }

        Ok(fresh_data)
    }
}
```

## Error Response Handling

When rate limit is hit:

```json
{
  "status": "error",
  "error": {
    "code": "RATE_LIMIT_EXCEEDED",
    "message": "Rate limit exceeded for place_order. Limit: 10 req/s, Current: 15 req/s. Retry after 1000ms"
  }
}
```

Parse the message to extract retry delay:

```rust
fn parse_retry_delay(error_message: &str) -> Option<Duration> {
    // Parse "Retry after 1000ms" from message
    let re = regex::Regex::new(r"Retry after (\d+)ms").ok()?;
    let caps = re.captures(error_message)?;
    let ms: u64 = caps.get(1)?.as_str().parse().ok()?;
    Some(Duration::from_millis(ms))
}
```

## Monitoring Rate Limits

### Metrics to Track

1. **Request counts per endpoint** (per minute)
2. **Rate limit hit rate** (% of requests hitting limits)
3. **Average wait time** when rate limited
4. **Cache hit rate** (reduce API calls)
5. **Aggregate throughput** (total req/10s)

### Logging Example

```rust
log::info!(
    "Rate limit status: place_order={}/10, cancel={}/600, aggregate={}/600",
    place_order_count,
    cancel_count,
    aggregate_count
);
```

## Testing Rate Limits

### Simulator

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(10, 1);

        // Should allow 10 requests
        for _ in 0..10 {
            assert!(limiter.check_and_add());
        }

        // 11th should fail
        assert!(!limiter.check_and_add());

        // Wait 1 second
        tokio::time::sleep(Duration::from_secs(1)).await;

        // Should allow again
        assert!(limiter.check_and_add());
    }
}
```

## Summary Table

| Category | Endpoint | Limit | Window | Weight | Per-IP |
|----------|----------|-------|--------|--------|--------|
| **Aggregate** | All | 600 | 10s | - | No |
| **Market Data** | status | 60 | 1s | 1 | No |
| | market_price | 60 | 1s | 1 | No |
| | market_liquidity | 60 | 1s | 1 | Yes (40/s) |
| | all_products | 12 | 1s | 5 | No |
| **Account** | subaccount_info | 40 | 10s | 10 | Yes |
| | fee_rates | 30 | 1s | 2 | No |
| | subaccount_orders | 30 | 1s | 2 | No |
| **Trading** | place_order (lev) | 10 | 1s | - | No |
| | place_order (spot) | 5 | 10s | - | No |
| | cancel_orders | 600 | 1s | - | No |
| | cancel_all | 2 | 1s | - | No |
| **Indexer** | candlesticks | 60 | 1s | - | No |
| **WebSocket** | Connections | 5 | - | - | Per wallet |

## Best Practices Summary

1. Implement client-side rate limiting
2. Use weighted tracking for aggregate limit
3. Cache data with appropriate TTLs
4. Prioritize critical requests (orders, cancels)
5. Use WebSocket for real-time data
6. Implement exponential backoff retries
7. Monitor and log rate limit hits
8. Batch requests when possible
9. Respect per-IP limits
10. Test rate limiters before production
