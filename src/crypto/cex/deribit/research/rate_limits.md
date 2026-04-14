# Deribit Rate Limits

Complete specification of Deribit's credit-based rate limiting system.

## Overview

Deribit uses a **credit-based rate limiting system** instead of simple request-per-second limits. Each API request consumes a certain number of credits, and credits refill continuously over time.

**Key Concepts**:
- Each request costs a specific number of credits
- Credits refill at a constant rate (refill rate)
- Maximum credit pool is capped (burst capacity)
- Rate limits vary by account tier and trading volume
- Separate limits for different request types

---

## Credit System Mechanics

### How Credits Work

1. **Credit Pool**: Each sub-account has a maximum credit capacity (e.g., 50,000 credits)
2. **Credit Cost**: Each API call consumes credits (e.g., 500 credits per request)
3. **Refill Rate**: Credits regenerate continuously (e.g., 10,000 credits/second)
4. **Burst Capacity**: You can burst above sustained rate if you have accumulated credits

### Formula

```
Available Credits = min(Max Credits, Current Credits + (Time Since Last Request × Refill Rate))
```

**Example**:
- Max Credit Cap: 50,000
- Refill Rate: 10,000 credits/second
- Request Cost: 500 credits

If you start with 50,000 credits (full):
- Send 100 requests instantly → Consume 50,000 credits → Pool drops to 0
- Wait 1 second → Refill 10,000 credits → Pool now at 10,000
- Sustained rate: 10,000 / 500 = **20 requests/second**

### Benefits

1. **Burst Capacity**: Allows submitting multiple orders at once if pool is full
2. **Fairness**: Heavy users don't monopolize resources
3. **Predictability**: Clear limits based on credit consumption

---

## Rate Limit Tiers

Rate limits scale with your **trading volume tier**. Higher volume traders get:
- Larger maximum credit pools
- Faster refill rates
- More sustainable throughput

**Checking Your Limits**:
Call `private/get_account_summary` to see your current rate limits:

```json
{
  "limits": {
    "matching_engine": {
      "used": 2500,
      "burst": 50000,
      "rate": 10000
    },
    "non_matching_engine": {
      "used": 1200,
      "burst": 200000,
      "rate": 20000
    }
  }
}
```

**Fields**:
- `used`: Credits consumed recently
- `burst`: Maximum credit pool (burst capacity)
- `rate`: Refill rate (credits per second)

---

## Request Categories

Deribit splits rate limits into two categories:

### 1. Matching Engine (Order Operations)

Applies to order placement and cancellation:
- `private/buy`
- `private/sell`
- `private/edit`
- `private/cancel`
- `private/cancel_all`
- `private/cancel_all_by_currency`
- `private/cancel_by_label`

**Typical Limits** (example):
- Burst: 50,000 credits
- Refill Rate: 10,000 credits/second
- Sustained Rate: ~20 requests/second (if each costs 500 credits)

**Why Separate?**: Order operations affect market fairness, so stricter limits prevent quote stuffing.

---

### 2. Non-Matching Engine (Data Operations)

Applies to data queries and non-trading operations:
- `public/get_instruments`
- `public/get_order_book`
- `public/ticker`
- `public/get_last_trades_by_instrument`
- `private/get_account_summary`
- `private/get_positions`
- `private/get_open_orders`

**Typical Limits** (example):
- Burst: 200,000 credits
- Refill Rate: 20,000 credits/second
- Sustained Rate: Higher than matching engine

**Why Separate?**: Data queries don't affect trading fairness, so higher limits are acceptable.

---

## Credit Costs per Endpoint

**Standard Cost**: Most endpoints cost **500 credits** per request.

**Variable Costs**: Some endpoints may cost more or less depending on complexity:
- Simple queries (e.g., `public/ticker`): 500 credits
- Complex queries (e.g., `public/get_instruments` with large result sets): May cost more
- Batch operations (e.g., `private/cancel_all_by_currency`): May cost more

**Note**: Exact credit costs per endpoint are not publicly documented in detail. Use the `limits` field in `get_account_summary` to monitor usage.

---

## Public vs Authenticated Requests

### Public Requests (Unauthenticated)

- Rate-limited **per IP address**
- Do NOT consume sub-account credit pool
- Lower limits than authenticated requests
- Encourages users to authenticate for better access

**Best Practice**: Always authenticate for production use to benefit from higher limits.

---

### Authenticated Requests

- Rate-limited **per sub-account**
- Consume credits from sub-account pool
- Higher and more transparent limits
- Limits scale with trading volume tier

**Advantage**: Authenticated connections have significantly better rate limits.

---

## Environment Separation

**Production and testnet operate on separate rate-limit pools.**

- Testnet usage does NOT affect production limits
- Test environment has independent credit pools
- Separate accounts required for test and production

**Implication**: You can test heavily on testnet without worrying about production rate limits.

---

## WebSocket vs HTTP

### WebSocket Subscriptions

- **Subscriptions** do NOT consume credits after initial subscription
- Push-based updates (server sends data to client)
- No polling required (efficient)
- Max 500 channels per subscription call

**Example**: Subscribe to `ticker.BTC-PERPETUAL.100ms` once, receive updates continuously without additional credit cost.

---

### HTTP Polling

- Each poll consumes credits
- Not recommended for real-time data (use WebSocket instead)
- Suitable for occasional snapshots (e.g., initial orderbook state)

**Best Practice**: Use WebSocket subscriptions for real-time data, HTTP for one-time queries.

---

## Rate Limit Error Handling

### Error Code: 10028

When credits are exhausted:

```json
{
  "jsonrpc": "2.0",
  "id": 1234,
  "error": {
    "code": 10028,
    "message": "too_many_requests"
  }
}
```

**What Happens**:
- Request is rejected immediately
- WebSocket connection may be terminated (for severe violations)
- Credits continue to refill

---

### Recovery Strategy

**If you hit error 10028**:

1. **Wait for Refill**: Credits refill continuously
   - Wait ~50ms → Refill ~500 credits (if refill rate = 10,000/sec)
   - Enough for 1 request
2. **Send Critical Requests**: Use recovered credits for urgent operations (e.g., cancel orders)
3. **Implement Backoff**: Use exponential backoff for retries

**Example**:
```rust
if error.code == 10028 {
    // Wait for credits to refill
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send critical cancel-all request
    cancel_all_orders().await?;

    // Implement exponential backoff for other requests
    backoff_delay = min(backoff_delay * 2, max_backoff);
    tokio::time::sleep(backoff_delay).await;
}
```

---

## Monitoring Rate Limits

### Check Remaining Credits

Call `private/get_account_summary` periodically to monitor:

```rust
let summary = get_account_summary("BTC").await?;
let matching_used = summary.limits.matching_engine.used;
let matching_burst = summary.limits.matching_engine.burst;
let matching_available = matching_burst - matching_used;

if matching_available < 5000 {
    // Low on credits, slow down
    warn!("Low on matching engine credits: {}", matching_available);
}
```

---

### Track Request Rate

**Client-side tracking**:
```rust
struct RateLimiter {
    max_credits: u64,
    refill_rate: u64, // credits per second
    current_credits: u64,
    last_update: Instant,
}

impl RateLimiter {
    fn consume(&mut self, cost: u64) -> Result<(), RateLimitError> {
        self.refill();

        if self.current_credits >= cost {
            self.current_credits -= cost;
            Ok(())
        } else {
            Err(RateLimitError::InsufficientCredits)
        }
    }

    fn refill(&mut self) {
        let elapsed = self.last_update.elapsed().as_secs_f64();
        let refilled = (elapsed * self.refill_rate as f64) as u64;
        self.current_credits = (self.current_credits + refilled).min(self.max_credits);
        self.last_update = Instant::now();
    }
}
```

---

## Best Practices

### 1. Use WebSocket for Real-Time Data

**Avoid**:
```rust
// BAD: Polling orderbook every 100ms
loop {
    let orderbook = get_order_book("BTC-PERPETUAL").await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
}
```

**Recommended**:
```rust
// GOOD: Subscribe to orderbook updates
subscribe("book.BTC-PERPETUAL.100ms").await?;
// Receive updates via WebSocket notifications (no credit cost)
```

---

### 2. Batch Subscriptions

Subscribe to multiple channels in one call (up to 500 channels):

```json
{
  "method": "public/subscribe",
  "params": {
    "channels": [
      "ticker.BTC-PERPETUAL.100ms",
      "ticker.ETH-PERPETUAL.100ms",
      "book.BTC-PERPETUAL.100ms",
      "trades.BTC-PERPETUAL.raw"
    ]
  }
}
```

**Benefit**: One request consumes credits once, but you get multiple streams.

---

### 3. Minimize Polling

Use REST only for:
- Initial snapshots (e.g., current positions on startup)
- Infrequent data (e.g., account summary every 5 minutes)
- Historical data (e.g., past trades)

**Avoid REST for**:
- Real-time orderbook updates (use WebSocket `book.*`)
- Real-time ticker data (use WebSocket `ticker.*`)
- Real-time trades (use WebSocket `trades.*`)

---

### 4. Cancel Operations Efficiently

**Use batch cancel methods**:
- `private/cancel_all_by_currency` - Cancel all orders for a currency
- `private/cancel_all` - Cancel all orders
- `private/cancel_by_label` - Cancel by label

**Avoid**:
```rust
// BAD: Cancelling orders one by one
for order_id in order_ids {
    cancel(order_id).await?; // N requests, N × 500 credits
}
```

**Recommended**:
```rust
// GOOD: Cancel all at once
cancel_all_by_currency("BTC").await?; // 1 request, 500 credits
```

---

### 5. Implement Client-Side Rate Limiting

**Proactive throttling**:
```rust
let rate_limiter = RateLimiter::new(50000, 10000); // burst, refill_rate

async fn call_api(&mut self, endpoint: &str) -> Result<Response> {
    // Wait if insufficient credits
    while !self.rate_limiter.can_consume(500) {
        tokio::time::sleep(Duration::from_millis(10)).await;
    }

    self.rate_limiter.consume(500)?;
    self.http_client.post(endpoint).send().await
}
```

---

### 6. Monitor and Alert

Set up monitoring for:
- Rate limit errors (10028)
- Credit pool depletion
- Sustained high request rates

**Example**:
```rust
if error.code == 10028 {
    metrics.increment("deribit.rate_limit.exceeded");
    alert("Deribit rate limit exceeded");
}
```

---

### 7. Choose Aggregation Intervals Wisely

For WebSocket subscriptions, choose intervals that balance latency and load:
- `raw` - Every update (highest load, lowest latency)
- `100ms` - Aggregated every 100ms (good balance)
- `agg2` - Aggregated (lower load, higher latency)

**Example**:
- High-frequency trading: `book.BTC-PERPETUAL.raw`
- Regular trading: `book.BTC-PERPETUAL.100ms`
- Monitoring: `ticker.BTC-PERPETUAL.agg2`

---

## Connection Limits

In addition to credit-based rate limits:

### WebSocket Connections
- **Max 32 connections per IP address**
- **Max 16 sessions per API key**

**If you exceed**:
- New connections are rejected
- Close unused connections before opening new ones

**Best Practice**: Reuse WebSocket connections for multiple subscriptions (up to 500 channels per connection).

---

### HTTP Connections

No explicit connection limit, but:
- Respect credit-based rate limits
- Use HTTP/2 or connection pooling for efficiency

---

## Special Considerations

### 1. Web Platform Usage

**Important**: Using the Deribit web platform (browser) also consumes API credits from your rate limit pool.

- Browsing orderbook, positions, account pages triggers API calls
- These calls consume the same credit pool as your API usage
- Monitor total usage (API + web)

**Implication**: If you're running a bot, be aware that manual web usage can affect your bot's rate limits.

---

### 2. Subscriptions with Authentication

**Raw public feeds require authenticated connections** (as a safeguard against abuse).

Example:
```json
{
  "method": "private/subscribe",
  "params": {
    "channels": ["book.BTC-PERPETUAL.raw"]
  }
}
```

**Requirement**: Must authenticate WebSocket connection before subscribing to `raw` feeds.

---

### 3. Historical Data Queries

Queries like `public/get_last_trades_by_instrument_and_time` may return large result sets:
- May cost more credits
- May take longer to process
- Implement pagination if available

---

## Rate Limit Strategies

### Strategy 1: Token Bucket (Client-Side)

Implement a token bucket algorithm:
```rust
struct TokenBucket {
    capacity: u64,
    tokens: u64,
    refill_rate: u64,
    last_refill: Instant,
}

impl TokenBucket {
    fn try_consume(&mut self, cost: u64) -> bool {
        self.refill();
        if self.tokens >= cost {
            self.tokens -= cost;
            true
        } else {
            false
        }
    }
}
```

---

### Strategy 2: Request Queue with Throttling

Queue requests and process at sustainable rate:
```rust
let mut queue = VecDeque::new();
let mut last_request = Instant::now();

loop {
    if let Some(request) = queue.pop_front() {
        let elapsed = last_request.elapsed();
        let min_interval = Duration::from_millis(50); // 20 req/sec

        if elapsed < min_interval {
            tokio::time::sleep(min_interval - elapsed).await;
        }

        send_request(request).await?;
        last_request = Instant::now();
    }
}
```

---

### Strategy 3: Adaptive Rate Control

Monitor error rates and adjust:
```rust
let mut request_interval = Duration::from_millis(50); // Start at 20 req/sec

loop {
    match send_request().await {
        Err(e) if e.code == 10028 => {
            // Rate limit hit, slow down
            request_interval = request_interval * 2;
            warn!("Rate limit hit, slowing to {:?}", request_interval);
        },
        Ok(_) => {
            // Success, can gradually speed up
            request_interval = max(
                request_interval * 0.95,
                Duration::from_millis(50) // min 20 req/sec
            );
        },
        Err(e) => return Err(e),
    }

    tokio::time::sleep(request_interval).await;
}
```

---

## Testing Rate Limits

### On Testnet

1. Authenticate to testnet
2. Call `private/get_account_summary` to see your limits
3. Send requests at increasing rates until you hit 10028
4. Measure:
   - Burst capacity (how many requests can you send instantly)
   - Sustained rate (requests per second before hitting limit)
   - Refill rate (how quickly credits recover)

**Example Test**:
```rust
// Send 100 requests as fast as possible
let mut success_count = 0;
for i in 0..100 {
    match send_request().await {
        Ok(_) => success_count += 1,
        Err(e) if e.code == 10028 => {
            println!("Hit rate limit after {} requests", success_count);
            break;
        },
        Err(e) => return Err(e),
    }
}

// Wait and measure refill
tokio::time::sleep(Duration::from_secs(1)).await;

// Try again
match send_request().await {
    Ok(_) => println!("Credits refilled successfully"),
    Err(_) => println!("Credits not yet refilled"),
}
```

---

## Implementation Checklist

For V5 connector:

- [ ] Implement client-side rate limiter (token bucket or similar)
- [ ] Track credit usage for matching and non-matching engines separately
- [ ] Prefer WebSocket subscriptions over HTTP polling
- [ ] Batch WebSocket subscriptions (up to 500 channels)
- [ ] Handle error 10028 with exponential backoff
- [ ] Monitor credit pool via `get_account_summary`
- [ ] Set alerts for rate limit violations
- [ ] Use batch cancel methods (`cancel_all_by_currency`, etc.)
- [ ] Limit concurrent WebSocket connections (max 32 per IP, 16 per API key)
- [ ] Test rate limits on testnet before production
- [ ] Log rate limit metrics (requests/sec, credit usage)
- [ ] Implement adaptive rate control (adjust based on errors)

---

## Summary Table

| Aspect | Details |
|--------|---------|
| **Rate Limit Type** | Credit-based (not simple req/sec) |
| **Standard Credit Cost** | ~500 credits per request |
| **Refill Rate** | ~10,000 credits/sec (matching), ~20,000 (non-matching) - varies by tier |
| **Burst Capacity** | ~50,000 (matching), ~200,000 (non-matching) - varies by tier |
| **Sustained Rate** | ~20 req/sec (matching), higher (non-matching) |
| **Public Requests** | Rate-limited per IP, lower limits |
| **Authenticated Requests** | Rate-limited per sub-account, higher limits |
| **Error Code** | 10028 (`too_many_requests`) |
| **WebSocket Subscriptions** | No ongoing credit cost after initial subscribe |
| **Max WebSocket Connections** | 32 per IP, 16 per API key |
| **Environment Isolation** | Testnet and production have separate pools |
| **Check Limits** | `private/get_account_summary` → `limits` field |

---

## References

- Rate Limits Support Article: https://support.deribit.com/hc/en-us/articles/25944617523357-Rate-Limits
- API Usage Policy: https://support.deribit.com/hc/en-us/articles/25944617449373-API-Usage-Policy
- Deribit API Documentation: https://docs.deribit.com/
