# Bitget API Rate Limits

Comprehensive documentation on Bitget API rate limiting mechanisms, headers, error handling, and best practices.

## 1. REST API Limits

### Overall IP Limit
- **6,000 requests per IP per minute** (overall cap across all endpoints)
- **Recovery time**: 5 minutes after rate limit is triggered
- **Calculation**: Each API interface's rate limit is calculated independently

### Endpoint-Specific Limits

#### Public Market Data Interfaces
- **20 requests per second per IP** (unified rate limit)
- Applies to: market info, contracts, depth, tickers, etc.

#### Default Rate
- **10 requests per second** when endpoint-specific limit is not specified

#### Trading Endpoints
- **Place batch orders**: 10 orders per currency pair = 1 request
- **Leverage/margin changes**: 5 calls per second (5c/1s)
- **Position queries**: 5 calls per second (5c/1s)

#### Specific Examples from Documentation
- Contract Config: 20 req/sec/IP
- Get Trade Rate: 10 times/1s (UID-based)
- Historical Funding Rate: 20 times/1s (IP-based)

### Limit Types
- **IP-based limits**: Most public endpoints (market data)
- **UID-based limits**: Some account/trading endpoints (identified by API key)
- **Independent calculation**: Spot and Futures endpoints calculated separately

### Weight System
Bitget uses a **time-based rate limiting system** rather than a traditional weight system:
- Base limit: ~50ms per request
- Approximately 3,000 requests per 5 minutes
- ~10 requests per second sustained rate
- Different endpoints may consume different amounts of quota

### Detailed Endpoint Limits

#### Spot Trading

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Place Order | 10/sec | UID |
| Batch Place Orders | 10/sec | UID |
| Cancel Order | 10/sec | UID |
| Batch Cancel Orders | 10/sec | UID |
| Modify Order | 10/sec | UID |
| Get Open Orders | 10/sec | UID |
| Get Order History | 10/sec | UID |
| Get Order Details | 10/sec | UID |
| Get Fills | 10/sec | UID |

**Maximum Orders:**
- **400 orders across all spot and margin trading pairs**
- Includes both active and pending orders
- Per user account

#### Futures Trading

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Place Order | 10/sec | UID |
| Batch Place Orders | 10/sec | UID |
| Cancel Order | 10/sec | UID |
| Batch Cancel Orders | 10/sec | UID |
| Modify Order | 10/sec | UID |
| Cancel All Orders | 10/sec | UID |
| Close All Positions | 10/sec | UID |
| Get Current Orders | 10/sec | UID |
| Get Order History | 10/sec | UID |
| Get Order Details | 10/sec | UID |
| Get Fills | 10/sec | UID |

**Maximum Orders:**
- **400 orders across all USDT, Coin-M, and USDC futures trading pairs**
- Per user account

#### Account Management Endpoints

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Set Leverage | 5/sec | UID |
| Set Margin | 5/sec | UID |
| Set Margin Mode | 5/sec | UID |
| Set Position Mode | 5/sec | UID |
| Get Account Info | 10/sec | UID |
| Get Account Assets | 10/sec | UID |
| Get Account Bills | 10/sec | UID |
| Transfer | 10/sec | UID |

#### Position Endpoints

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Get Single Position | 20/sec | IP |
| Get All Positions | 20/sec | IP |
| Get Historical Positions | 10/sec | UID |
| Calculate Max Open Size | 10/sec | UID |

#### Market Data Endpoints

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Get Server Time | 20/sec | IP |
| Get Symbols | 20/sec | IP |
| Get Ticker | 20/sec | IP |
| Get All Tickers | 20/sec | IP |
| Get Order Book | 20/sec | IP |
| Get Candles | 20/sec | IP |
| Get Recent Trades | 20/sec | IP |
| Get Funding Rate | 20/sec | IP |
| Get Index Price | 20/sec | IP |
| Get Mark Price | 20/sec | IP |

#### Wallet Endpoints

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Get Deposit Address | 10/sec | UID |
| Withdraw | 10/sec | UID |
| Get Withdrawal List | 10/sec | UID |
| Get Deposit List | 10/sec | UID |

#### Plan Orders (Stop Loss/Take Profit)

| Endpoint | Rate Limit | Unit |
|----------|-----------|------|
| Place Plan Order | 10/sec | UID |
| Place TP/SL | 10/sec | UID |
| Modify Plan Order | 10/sec | UID |
| Cancel Plan Order | 10/sec | UID |
| Get Current Plans | 10/sec | UID |
| Get Plan History | 10/sec | UID |

## 2. Response Headers

### Rate Limit Monitoring Header
```
x-mbx-used-remain-limit
```
- **Purpose**: Indicates remaining rate limit quota per second
- **Type**: Integer value
- **Usage**: Check this header to monitor how close you are to the limit
- **Note**: Header name follows Binance convention (MBX) for API compatibility

### Alternative Headers (may be present)
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 1695806875
```

**Note:** These headers are not consistently provided. Implement client-side rate limiting for reliability.

## 3. Error Handling

### HTTP Status Code
```
429 Too Many Requests
```
- Returned when access exceeds the frequency limit
- System automatically limits the request

### Error Codes

#### Code 30014 (Primary Rate Limit Error)
```json
{
  "code": 30014,
  "message": "request too frequent"
}
```
- **Type**: DDoSProtection
- **Meaning**: Standard rate limiting error for excessive requests

#### Code 40014 (Alternative Format)
```json
{
  "code": "40014",
  "msg": "Too many requests",
  "requestTime": 1695806875837,
  "data": null
}
```
- **Type**: DDoSProtection
- **Meaning**: Rate limit exceeded

#### Code 429 (HTTP Status)
```json
{
  "code": 429,
  "message": "Too many requests"
}
```
- **Type**: DDoSProtection
- **Meaning**: HTTP status code for rate limiting

#### Code 1001 (Throttling)
```json
{
  "code": 1001,
  "message": "The request is too frequent and has been throttled"
}
```
- **Type**: RateLimitExceeded
- **Meaning**: Request throttled due to frequency

### Response Format
All Bitget API responses are returned in **JSON format** after parameter security verification.

### Retry-After Header
- **Not explicitly documented** in Bitget API docs
- Recommended to implement exponential backoff instead
- Recovery time: 5 minutes after overall IP limit is triggered

### Recovery Times
- **Overall IP limit (6000/min):** 5 minutes
- **Endpoint-specific limits:** Typically 1 second (for per-second limits)

## 4. WebSocket Limits

### Connection Limits
- **100 connections per IP address** (maximum)

### Subscription Limits
- **240 subscription requests per hour per connection**
- **1,000 maximum channels per connection** (hard limit)
- **Recommended: <50 channels per connection** for optimal stability
- Connections with fewer subscriptions are more stable

### Message Rate Limits
- **10 messages per second** (includes all message types):
  - Ping messages
  - Subscribe/unsubscribe requests
  - Other JSON messages
- **Violation**: Connection will be disconnected if limit exceeded

### Heartbeat Requirements
- **Send "ping"** every 30 seconds
- **Expect "pong"** response
- **Reconnect** if no "pong" received
- **Connection timeout:** 2 minutes without ping

If the WebSocket server doesn't receive a ping for 2 minutes, it will disconnect the connection.

## 5. Best Practices

### Recovery Strategy After 429

#### Exponential Backoff
1. First retry: Wait 1 second
2. Second retry: Wait 2 seconds
3. Third retry: Wait 4 seconds
4. Continue doubling until success or max wait time

```rust
async fn place_order_with_retry(
    api: &BitgetApi,
    order: Order,
    max_retries: u32,
) -> Result<OrderResponse> {
    let mut retries = 0;

    loop {
        match api.place_order(order.clone()).await {
            Ok(response) => return Ok(response),
            Err(Error::RateLimitExceeded) if retries < max_retries => {
                retries += 1;
                let backoff = Duration::from_secs(2u64.pow(retries));
                eprintln!("Rate limited, waiting {:?}", backoff);
                sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

#### Monitoring
- Check `x-mbx-used-remain-limit` header in every response
- Track request rate on client side
- Implement request queuing to stay within limits

#### IP Limit Recovery
- After triggering the 6,000/minute limit: wait 5 minutes
- During recovery, all requests from that IP will be blocked

### Recommended Intervals

#### Sustained Rate
- **Conservative**: 8-9 requests per second (well below 10/sec limit)
- **Public endpoints**: 15-18 requests per second (below 20/sec limit)
- **Burst capacity**: Avoid bursts; maintain steady rate

#### Request Queuing
- Implement client-side rate limiter
- Queue requests and release at controlled intervals
- Prioritize critical requests (trading) over market data

### Use Batch Endpoints

When placing multiple orders, use batch endpoints to reduce request count:

```rust
// Instead of:
for order in orders {
    api.place_order(order).await?; // 10 requests
}

// Use:
api.batch_place_orders(orders).await?; // 1 request
```

### Cache Market Data

Cache frequently accessed data to reduce API calls:

```rust
use std::time::{Duration, Instant};

struct CachedTicker {
    data: TickerData,
    cached_at: Instant,
    ttl: Duration,
}

impl CachedTicker {
    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

struct MarketDataCache {
    tickers: HashMap<String, CachedTicker>,
}

impl MarketDataCache {
    async fn get_ticker(&mut self, symbol: &str, api: &BitgetApi) -> Result<TickerData> {
        if let Some(cached) = self.tickers.get(symbol) {
            if cached.is_valid() {
                return Ok(cached.data.clone());
            }
        }

        // Fetch from API
        let ticker = api.get_ticker(symbol).await?;
        self.tickers.insert(
            symbol.to_string(),
            CachedTicker {
                data: ticker.clone(),
                cached_at: Instant::now(),
                ttl: Duration::from_secs(1),
            },
        );

        Ok(ticker)
    }
}
```

### Use WebSocket for Real-Time Data

For streaming data, use WebSocket instead of polling REST endpoints:

```rust
// Instead of polling (20 API calls/sec):
loop {
    let ticker = api.get_ticker("BTCUSDT_SPBL").await?;
    sleep(Duration::from_millis(50)).await;
}

// Use WebSocket:
ws.subscribe_ticker("BTCUSDT").await?;
while let Some(update) = ws.next_message().await {
    // Process ticker update
}
```

### Security Recommendations
- Bind API keys to specific IP addresses
- Keep `SecretKey` and `Passphrase` confidential
- Monitor timestamp synchronization (must be within 30 seconds of server time)
- Use official SDKs when possible (Java, Python, Go, Node.js, PHP)

## 6. Implementation Notes

### Client-Side vs Server-Side Tracking

#### Client-Side Tracking (Recommended Primary Approach)
**Advantages:**
- Prevents hitting rate limits before 429 errors
- No dependency on server headers
- Full control over request timing
- Can implement sophisticated queuing

**Implementation:**
```rust
// Token bucket or sliding window algorithm
// Track requests per second/minute locally
// Queue and throttle outgoing requests
```

#### Server-Side Tracking (Supplementary)
**Use cases:**
- Verify client-side accuracy
- Adjust client limits dynamically
- Handle distributed systems

**Implementation:**
```rust
// Parse x-mbx-used-remain-limit header
// Adjust client-side limiter based on server feedback
// Log discrepancies for tuning
```

### Recommended Approach
**Hybrid Strategy:**
1. **Primary**: Client-side token bucket algorithm
   - Track 1-second windows for per-second limits
   - Track 1-minute windows for per-minute (6000) limit
   - Separate buckets for IP-based vs UID-based endpoints

2. **Secondary**: Server header monitoring
   - Parse `x-mbx-used-remain-limit` when available
   - Use as feedback to tune client-side limits
   - Log warnings when client/server diverge

3. **Tertiary**: Error-based backoff
   - Catch 429/30014/40014/1001 errors as last resort
   - Implement exponential backoff
   - Reduce client-side limits after hitting server limit

### Implementation Considerations

#### Separate Limiters Needed
- **IP-based limiter**: For public market data (20/sec)
- **UID-based limiter**: For account/trading endpoints (varies)
- **Global IP limiter**: For overall 6000/minute cap

#### Request Categorization
```rust
enum LimitType {
    PublicMarketData,  // 20/sec IP-based
    Trading,           // 10/sec UID-based
    Account,           // 10/sec UID-based
    Leverage,          // 5/sec UID-based
}
```

#### Token Bucket Parameters
```rust
// Public market data
capacity: 20 tokens
refill_rate: 20 tokens per second
max_burst: 20

// Overall IP limit
capacity: 6000 tokens
refill_rate: 100 tokens per second (6000/60)
max_burst: 6000

// Recovery after 429
backoff: exponential, max 5 minutes
```

### Rate Limiting Algorithms

#### 1. Token Bucket Algorithm

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct RateLimiter {
    capacity: u32,
    tokens: u32,
    refill_rate: u32, // tokens per second
    last_refill: Instant,
}

impl RateLimiter {
    fn new(capacity: u32, refill_rate: u32) -> Self {
        Self {
            capacity,
            tokens: capacity,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        let tokens_to_add = (elapsed * self.refill_rate as f64) as u32;

        if tokens_to_add > 0 {
            self.tokens = (self.tokens + tokens_to_add).min(self.capacity);
            self.last_refill = now;
        }
    }

    async fn acquire(&mut self, tokens: u32) -> Result<()> {
        loop {
            self.refill();
            if self.tokens >= tokens {
                self.tokens -= tokens;
                return Ok(());
            }
            // Wait before retry
            sleep(Duration::from_millis(100)).await;
        }
    }
}

// Usage
let mut trading_limiter = RateLimiter::new(10, 10); // 10/sec
trading_limiter.acquire(1).await?;
// Make API call
```

#### 2. Sliding Window

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

struct SlidingWindowLimiter {
    window: Duration,
    max_requests: usize,
    requests: VecDeque<Instant>,
}

impl SlidingWindowLimiter {
    fn new(window: Duration, max_requests: usize) -> Self {
        Self {
            window,
            max_requests,
            requests: VecDeque::new(),
        }
    }

    fn clean_old_requests(&mut self) {
        let now = Instant::now();
        while let Some(&front) = self.requests.front() {
            if now.duration_since(front) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }
    }

    async fn acquire(&mut self) -> Result<()> {
        loop {
            self.clean_old_requests();
            if self.requests.len() < self.max_requests {
                self.requests.push_back(Instant::now());
                return Ok(());
            }

            // Calculate wait time
            if let Some(&oldest) = self.requests.front() {
                let wait_until = oldest + self.window;
                let now = Instant::now();
                if wait_until > now {
                    sleep(wait_until - now).await;
                }
            }
        }
    }
}

// Usage
let mut limiter = SlidingWindowLimiter::new(
    Duration::from_secs(1),
    10 // 10 requests per second
);
limiter.acquire().await?;
// Make API call
```

#### 3. Multi-Tier Rate Limiter

```rust
use std::collections::HashMap;

struct MultiTierLimiter {
    limiters: HashMap<String, RateLimiter>,
}

impl MultiTierLimiter {
    fn new() -> Self {
        let mut limiters = HashMap::new();

        // IP-level limiter (6000/min)
        limiters.insert(
            "ip".to_string(),
            RateLimiter::new(6000, 100), // 100/sec average
        );

        // Trading endpoints (10/sec)
        limiters.insert(
            "trading".to_string(),
            RateLimiter::new(10, 10),
        );

        // Account management (10/sec)
        limiters.insert(
            "account".to_string(),
            RateLimiter::new(10, 10),
        );

        // Leverage/margin (5/sec)
        limiters.insert(
            "leverage".to_string(),
            RateLimiter::new(5, 5),
        );

        // Market data (20/sec)
        limiters.insert(
            "market".to_string(),
            RateLimiter::new(20, 20),
        );

        Self { limiters }
    }

    async fn acquire(&mut self, category: &str) -> Result<()> {
        // Always check IP limit
        if let Some(ip_limiter) = self.limiters.get_mut("ip") {
            ip_limiter.acquire(1).await?;
        }

        // Check category-specific limit
        if let Some(limiter) = self.limiters.get_mut(category) {
            limiter.acquire(1).await?;
        }

        Ok(())
    }
}

// Usage
let mut limiter = MultiTierLimiter::new();
limiter.acquire("trading").await?;
api.place_order(...).await?;
```

### Rate Limit Monitoring

```rust
struct RateLimitMonitor {
    requests_made: HashMap<String, u64>,
    start_time: Instant,
}

impl RateLimitMonitor {
    fn record_request(&mut self, endpoint: &str) {
        *self.requests_made.entry(endpoint.to_string()).or_insert(0) += 1;
    }

    fn get_stats(&self) -> Vec<(String, u64, f64)> {
        let elapsed = self.start_time.elapsed().as_secs_f64();
        self.requests_made
            .iter()
            .map(|(endpoint, count)| {
                let rate = *count as f64 / elapsed;
                (endpoint.clone(), *count, rate)
            })
            .collect()
    }

    fn print_stats(&self) {
        println!("Rate Limit Usage:");
        for (endpoint, count, rate) in self.get_stats() {
            println!("  {}: {} requests ({:.2}/sec)", endpoint, count, rate);
        }
    }
}
```

### Testing Recommendations
1. Test rate limits in sandbox/testnet if available
2. Gradually increase request rate to find true limits
3. Monitor `x-mbx-used-remain-limit` during testing
4. Log all 429 errors with request patterns
5. Validate that different endpoints have independent limits

### Edge Cases
- **Shared IP (VPN/proxy)**: May hit limits faster due to other users
- **Clock skew**: Ensure system time is synchronized (within 30sec)
- **Burst patterns**: Avoid sending all requests at start of second
- **Connection pooling**: Reuse connections to reduce overhead

### Maintenance Windows

During backend maintenance, APIs may return specific error codes:
- **40725**: System under maintenance
- **45001**: Backend maintenance
- **40015**: Timestamp-related errors

**Regular Maintenance Times:** Check Bitget announcements for scheduled maintenance.

## Summary Table

| Category | Limit | Unit | Recovery |
|----------|-------|------|----------|
| Overall IP | 6000/min | IP | 5 min |
| Public Market Data | 20/sec | IP | Immediate (per-second) |
| Default | 10/sec | Varies | Immediate |
| Trading | 10/sec | UID | Immediate |
| Account | 10/sec | UID | Immediate |
| Leverage/Margin | 5/sec | UID | Immediate |
| Position | 20/sec | IP | Immediate |
| Max Spot Orders | 400 total | UID | - |
| Max Futures Orders | 400 total | UID | - |
| WS Connections | 100 | IP | - |
| WS Channels | 1000 max (50 recommended) | Connection | - |
| WS Messages | 10/sec | Connection | Disconnection |
| WS Subscriptions | 240/hour | Connection | 1 hour window |

## Sources

- [Bitget API Rate Limits: A Comprehensive Overview](https://www.bitget.com/wiki/bitget-api-rate-limits)
- [Bitget API Docs - Mix/Margin](https://bitgetlimited.github.io/apidoc/en/mix/)
- [Bitget WebSocket API Documentation](https://www.bitget.com/api-doc/common/websocket-intro)
- [CCXT Bitget Implementation](https://github.com/ccxt/ccxt/blob/master/python/ccxt/bitget.py)
- [Bitget WS Rate Limit Issue #24458](https://github.com/ccxt/ccxt/issues/24458)
- [HTTP 429 Best Practices](https://blog.postman.com/http-error-429/)
- [Bitget API Request Interaction](https://www.bitget.com/api-doc/common/signature-samaple/interaction)

---

**Last Updated**: 2026-01-20
**API Version**: V5
**Status**: Active
