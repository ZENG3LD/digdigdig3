# Gate.io API v4 Rate Limits

**Research Date**: 2026-01-21
**Source**: https://www.gate.com/announcements/article/31282

---

## Overview

Gate.io API v4 implements rate limits to prevent abuse and ensure fair access. Rate limits are applied **per UID** (User ID) for private endpoints and **per IP** for public endpoints.

---

## Rate Limit Rules

### Public Endpoints

**Applied by**: IP Address

**General Rule**: Most public endpoints have no explicit rate limit, but excessive requests may be throttled.

**Recommended**:
- Keep request frequency reasonable (< 10 requests/second per IP)
- Use WebSocket for real-time data instead of polling REST API

### Private Endpoints

**Applied by**: UID (User ID / Account ID)

Rate limits vary by endpoint type and trading activity.

---

## Spot Trading Rate Limits

### Order Placement and Modification

**Endpoints affected**:
- `POST /spot/orders` (create order)
- `PATCH /spot/orders/{order_id}` (modify order)
- `POST /spot/batch_orders` (batch create)
- `PATCH /spot/batch_orders` (batch modify)

**Rate Limit**: **10 requests per second** (10r/s)

**Applies to**: Combined total of:
- Single order placement
- Single order modification
- Batch order operations

**Note**: This is the **total** limit for all order placement/modification operations combined, not per endpoint.

### Order Cancellation

**Endpoints affected**:
- `DELETE /spot/orders/{order_id}` (cancel single order)
- `DELETE /spot/orders` (cancel all orders)

**Rate Limit**: No explicit limit

**Recommended**: Keep cancellations reasonable to avoid throttling

### Query Endpoints

**Endpoints affected**:
- `GET /spot/orders` (list orders)
- `GET /spot/orders/{order_id}` (get order)
- `GET /spot/accounts` (get balances)
- `GET /spot/my_trades` (get trades)

**Rate Limit**: No explicit limit

**Recommended**: Don't poll excessively, use WebSocket for real-time updates

---

## Futures Trading Rate Limits

### Order Placement and Modification

**Endpoints affected**:
- `POST /futures/{settle}/orders` (create order)
- `PUT /futures/{settle}/orders/{order_id}` (modify order)
- `POST /futures/{settle}/batch_orders` (batch create)
- `DELETE /futures/{settle}/orders/{order_id}` (cancel single)
- `DELETE /futures/{settle}/orders` (cancel all)

**Rate Limit**: **100 requests per second** (100r/s)

**Applies to**: Combined total of:
- Single order placement
- Single order modification
- Single order cancellation
- Batch order operations
- Bulk cancellation

**Note**: Futures have a **10x higher** rate limit than spot (100 r/s vs 10 r/s).

### Query Endpoints

**Endpoints affected**:
- `GET /futures/{settle}/orders` (list orders)
- `GET /futures/{settle}/positions` (get positions)
- `GET /futures/{settle}/accounts` (get account)

**Rate Limit**: No explicit limit

---

## Additional Restrictions for Low Fill Ratio

Gate.io applies **additional restrictions** for users with low fill ratios (order-to-fill ratio).

### Low Fill Ratio Penalty

**Affected endpoints**:
- `POST /spot/orders` (order placement)
- `PATCH /spot/orders/{order_id}` (order modification)

**Additional Limit**: **10 requests per 10 seconds** (1 r/s average)

**Applied to**: Users with consistently low fill ratios

**Purpose**: Prevent order spam and market manipulation

**How to avoid**:
- Place orders with realistic prices
- Don't place and immediately cancel orders repeatedly
- Maintain a reasonable fill ratio

---

## Rate Limit Headers

Gate.io does not return rate limit information in response headers (unlike Binance/Coinbase).

**Alternative**: Use endpoint `/account/rate_limit` to check your current rate limit status.

### Get Rate Limit Status

**Endpoint**: `GET /account/rate_limit`

**Auth Required**: Yes

**Response**:
```json
{
  "uid": 123456,
  "currency_pair_limits": {
    "BTC_USDT": {
      "limit": 10,
      "remaining": 8,
      "reset": 1729100692
    }
  },
  "global_limits": {
    "order_placement": {
      "limit": 10,
      "remaining": 5,
      "reset": 1729100692
    }
  }
}
```

**Note**: This endpoint may not be available in all API versions. Check documentation for current availability.

---

## Rate Limit Errors

### Error Response

When rate limit is exceeded:

**HTTP Status**: `429 Too Many Requests`

**Response Body**:
```json
{
  "label": "TOO_MANY_REQUESTS",
  "message": "Rate limit exceeded"
}
```

### Handling Rate Limits

**Best practices**:

1. **Implement exponential backoff**:
   ```rust
   let mut delay = 1000; // Start with 1 second
   loop {
       match make_request().await {
           Ok(response) => return Ok(response),
           Err(e) if e.status() == 429 => {
               tokio::time::sleep(Duration::from_millis(delay)).await;
               delay *= 2; // Exponential backoff
               if delay > 60000 {
                   return Err(e); // Give up after 1 minute
               }
           }
           Err(e) => return Err(e),
       }
   }
   ```

2. **Track request counts locally**:
   ```rust
   struct RateLimiter {
       requests: Vec<Instant>,
       limit: usize,
       window: Duration,
   }

   impl RateLimiter {
       fn check_and_add(&mut self) -> bool {
           let now = Instant::now();
           // Remove requests outside the window
           self.requests.retain(|&t| now.duration_since(t) < self.window);

           if self.requests.len() < self.limit {
               self.requests.push(now);
               true
           } else {
               false // Rate limit would be exceeded
           }
       }
   }
   ```

3. **Use WebSocket for real-time data**:
   - Don't poll REST API for order updates
   - Subscribe to WebSocket channels instead
   - Much more efficient and avoids rate limits

4. **Batch operations when possible**:
   - Use batch order endpoints for multiple orders
   - Reduces number of requests

---

## Comparison with Other Exchanges

| Exchange | Spot Order Limit | Futures Order Limit | Applied By |
|----------|------------------|---------------------|------------|
| **Gate.io** | 10 r/s | 100 r/s | UID |
| Binance | 50 r/10s | 300 r/10s | UID + IP |
| OKX | 60 r/2s | 300 r/2s | UID |
| Bybit | 50 r/s | 100 r/s | UID |
| KuCoin | 45 r/3s | 40 r/s | IP |

**Gate.io observations**:
- **Lower** spot order rate limit compared to competitors
- **Competitive** futures rate limit
- Applied **per UID** (not IP), making it easier for high-frequency traders with multiple IPs

---

## WebSocket Rate Limits

### Connection Limits

**Maximum connections**: Not explicitly documented, but reasonable limits apply

**Recommended**: Maintain single connection per account, use multiple subscriptions

### Subscription Limits

**Maximum subscriptions per connection**: Not explicitly documented

**Recommended**: Keep subscriptions reasonable (< 100 per connection)

### Message Rate Limits

**Incoming messages**: No explicit limit

**Outgoing messages** (commands): Limit not documented, but avoid excessive commands

**Ping/Pong**: Required every 10-30 seconds to keep connection alive

---

## Best Practices

### 1. Use WebSocket for Real-Time Data

**Instead of**:
```rust
// Polling REST API every second (BAD)
loop {
    let orders = get_open_orders().await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

**Do this**:
```rust
// Subscribe to WebSocket order updates (GOOD)
ws_subscribe("spot.orders", callback).await?;
```

### 2. Implement Request Queueing

```rust
use tokio::sync::Semaphore;

struct RequestQueue {
    semaphore: Semaphore,
}

impl RequestQueue {
    fn new(rate_limit: usize) -> Self {
        Self {
            semaphore: Semaphore::new(rate_limit),
        }
    }

    async fn execute<F, T>(&self, f: F) -> Result<T>
    where
        F: Future<Output = Result<T>>,
    {
        let _permit = self.semaphore.acquire().await?;
        let result = f.await?;

        // Release permit after 1 second (for 10 r/s limit)
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(1)).await;
            drop(_permit);
        });

        Ok(result)
    }
}
```

### 3. Prioritize Order Operations

```rust
enum RequestPriority {
    High,    // Order cancellation
    Medium,  // Order placement
    Low,     // Query operations
}

// Process high-priority requests first
```

### 4. Cache Public Data

```rust
struct MarketDataCache {
    tickers: HashMap<String, Ticker>,
    last_update: Instant,
    ttl: Duration,
}

impl MarketDataCache {
    async fn get_ticker(&mut self, symbol: &str) -> Result<Ticker> {
        let now = Instant::now();

        // Return cached data if still fresh
        if now.duration_since(self.last_update) < self.ttl {
            if let Some(ticker) = self.tickers.get(symbol) {
                return Ok(ticker.clone());
            }
        }

        // Fetch fresh data
        let ticker = fetch_ticker(symbol).await?;
        self.tickers.insert(symbol.to_string(), ticker.clone());
        self.last_update = now;

        Ok(ticker)
    }
}
```

### 5. Monitor and Log Rate Limits

```rust
struct RateLimitMonitor {
    requests_made: AtomicUsize,
    errors_429: AtomicUsize,
}

impl RateLimitMonitor {
    fn record_request(&self) {
        self.requests_made.fetch_add(1, Ordering::Relaxed);
    }

    fn record_rate_limit_error(&self) {
        self.errors_429.fetch_add(1, Ordering::Relaxed);
        warn!("Rate limit exceeded! Total 429 errors: {}",
              self.errors_429.load(Ordering::Relaxed));
    }

    fn get_stats(&self) -> (usize, usize) {
        (
            self.requests_made.load(Ordering::Relaxed),
            self.errors_429.load(Ordering::Relaxed),
        )
    }
}
```

---

## Rate Limit Summary

| Endpoint Type | Spot Limit | Futures Limit | Applied By |
|---------------|------------|---------------|------------|
| **Order Placement/Modification** | 10 r/s | 100 r/s | UID |
| **Order Cancellation** | No limit | 100 r/s | UID |
| **Query Endpoints** | No limit | No limit | - |
| **Public Endpoints** | No limit | No limit | IP |
| **Low Fill Ratio Penalty** | 1 r/s | - | UID |

### Key Takeaways

1. **Spot trading**: Conservative 10 r/s limit for orders
2. **Futures trading**: More generous 100 r/s limit
3. **Applied per UID**, not IP (good for distributed systems)
4. **No limits** on query endpoints (but don't abuse)
5. **Low fill ratio penalty** applies additional restrictions
6. **WebSocket preferred** for real-time data (no REST polling)
7. **No rate limit headers** in responses (must track locally)

---

## Implementation Checklist

- [ ] Implement rate limiter (10 r/s for spot, 100 r/s for futures)
- [ ] Add exponential backoff for 429 errors
- [ ] Track request counts locally
- [ ] Use WebSocket for real-time updates
- [ ] Cache public data (tickers, orderbook depth)
- [ ] Prioritize critical operations (cancellations > placements > queries)
- [ ] Log rate limit errors and monitor frequency
- [ ] Implement request queueing system
- [ ] Batch operations when possible
- [ ] Maintain reasonable fill ratio to avoid penalties

---

## Sources

- [Gate.io API Rate Limit Adjustment](https://www.gate.com/announcements/article/31282)
- [Gate.io API Rate Limit Rules](https://www.gate.com/announcements/article/33910)
- [Gate.io Spot Trading Rate Limit Update](https://www.coincarp.com/exchange/announcement/gate-io-40657/)
- [Gate.io Futures Rate Limit Policy](https://www.coincarp.com/exchange/announcement/gate-io-38255/)
- [Gate.io API Documentation](https://www.gate.com/docs/developers/apiv4/en/)

---

**Research completed**: 2026-01-21
**Implementation priority**: Rate limiting is critical for production use. Implement before deploying to avoid 429 errors.
