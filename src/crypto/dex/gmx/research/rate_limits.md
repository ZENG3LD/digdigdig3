# GMX Rate Limits

## Overview

GMX provides **public REST API endpoints** without documented rate limits. However, as a decentralized exchange operating on blockchain infrastructure, rate limiting considerations differ from centralized exchanges.

## REST API Rate Limits

### No Published Rate Limits

GMX's public API documentation **does not specify explicit rate limits** for their REST endpoints:
- `/{chain}-api.gmxinfra.io/*`

### Observed Behavior

Based on the infrastructure:
- **No authentication required** for read-only endpoints
- **No API keys** to rate limit against
- Endpoints served via gmxinfra.io infrastructure
- Fallback URLs suggest high availability focus

### Recommended Best Practices

Even without published limits, implement conservative rate limiting:

**Recommended Limits:**
- **Market Data (Tickers, Prices):** 10 requests/second
- **Candlestick Data:** 5 requests/second
- **Market Info:** 2 requests/second
- **Historical Data:** 1 request/second

**Reasoning:**
- Avoids overwhelming infrastructure
- Prevents potential IP-based throttling
- Good citizenship in decentralized ecosystem
- Fallback URL rotation provides redundancy

### Implementation Strategy

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct RateLimiter {
    last_request: Instant,
    min_interval: Duration,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        let min_interval = Duration::from_millis(1000 / requests_per_second as u64);
        Self {
            last_request: Instant::now() - min_interval,
            min_interval,
        }
    }

    async fn wait(&mut self) {
        let elapsed = self.last_request.elapsed();
        if elapsed < self.min_interval {
            sleep(self.min_interval - elapsed).await;
        }
        self.last_request = Instant::now();
    }
}

// Usage:
let mut limiter = RateLimiter::new(10); // 10 req/s

for symbol in symbols {
    limiter.wait().await;
    let price = fetch_ticker(symbol).await?;
}
```

## Blockchain Transaction Limits

### Network-Level Constraints

GMX trading operates on blockchain networks with their own limitations:

**Arbitrum:**
- Block time: ~250ms (4 blocks/second)
- Max gas per block: 32,000,000
- Transactions per block: Varies by gas usage
- Practical limit: ~100-200 txs/second network-wide

**Avalanche:**
- Block time: ~2 seconds
- Higher throughput than Ethereum
- Practical limit: ~4,500 txs/second network-wide

### Account-Level Constraints

**Nonce Management:**
- Each account has a sequential nonce
- Transactions must be submitted in order
- Concurrent transactions possible but require nonce coordination

**Practical Trading Limits:**
- 1 transaction every 2-3 seconds per account (conservative)
- 5-10 transactions per block (with proper nonce management)
- Limited by gas estimation and confirmation time

### Gas Price Competition

During high network congestion:
- Higher gas prices needed for fast execution
- Transaction costs increase
- Effective rate limit based on economic factors

## Keeper Execution Limits

### Order Execution Delays

GMX uses an asynchronous execution model:

**Create Order:**
- User submits transaction immediately
- Limited only by blockchain capacity
- No GMX-specific rate limit

**Order Execution:**
- Executed by off-chain keepers
- Typical delay: 5-30 seconds
- During high activity: Up to 1-2 minutes

**Practical Throughput:**
- Can create orders as fast as blockchain allows
- Execution throughput limited by keeper capacity
- Not a traditional "rate limit" but affects trading speed

### Keeper Capacity

**Observable Limits:**
- Keepers process orders in queue order
- High-volume periods see longer execution times
- No hard limit on pending orders
- Economic cost (execution fees) natural throttle

## Oracle Price Updates

### Price Feed Frequency

**Oracle Keepers Update Prices:**
- Frequency: Every 1-5 seconds (chain dependent)
- More frequent during volatility
- Signed prices cached briefly

**Implications for Trading:**
- Order execution uses recent oracle prices
- Stale prices (>60 seconds) typically rejected
- No need to spam requests; prices update automatically

## Fallback URL Strategy

GMX provides multiple endpoint URLs for redundancy:

**Primary → Fallback → Fallback2**

### Rotation Strategy

```rust
struct GmxClient {
    urls: Vec<String>,
    current_index: usize,
    failures: HashMap<String, u32>,
}

impl GmxClient {
    fn new(chain: &str) -> Self {
        let urls = vec![
            format!("https://{}-api.gmxinfra.io", chain),
            format!("https://{}-api-fallback.gmxinfra.io", chain),
            format!("https://{}-api-fallback2.gmxinfra.io", chain),
        ];
        Self {
            urls,
            current_index: 0,
            failures: HashMap::new(),
        }
    }

    async fn request(&mut self, endpoint: &str) -> Result<Response> {
        for attempt in 0..3 {
            let url = &self.urls[self.current_index];
            let full_url = format!("{}{}", url, endpoint);

            match reqwest::get(&full_url).await {
                Ok(resp) if resp.status().is_success() => {
                    self.failures.insert(url.clone(), 0);
                    return Ok(resp);
                }
                Ok(resp) if resp.status() == 429 => {
                    // Rate limited - rotate immediately
                    self.rotate_url();
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
                _ => {
                    *self.failures.entry(url.clone()).or_insert(0) += 1;
                    self.rotate_url();
                }
            }
        }
        Err(Error::AllEndpointsFailed)
    }

    fn rotate_url(&mut self) {
        self.current_index = (self.current_index + 1) % self.urls.len();
    }
}
```

### Fallback Triggers

Rotate to fallback URL on:
- **429 Too Many Requests** (if implemented)
- **5xx Server Errors**
- **Connection timeouts** (>10 seconds)
- **3+ consecutive failures**

## Subsquid GraphQL Rate Limits

### GraphQL Endpoint Limits

GMX uses Subsquid for GraphQL queries:
- `https://gmx.squids.live/gmx-synthetics-arbitrum:prod/api/graphql`

**Subsquid Rate Limits:**
- Not publicly documented for GMX's deployment
- Hosted Subsquid instances typically allow high throughput
- Recommend: **5-10 queries per second**

### Query Complexity Limits

GraphQL queries have complexity scoring:

**High-Complexity Queries:**
- Large date ranges
- Deep nested queries
- Many joined entities

**Recommendations:**
- Limit result sets to 100-1000 records per query
- Use pagination for large datasets
- Avoid deeply nested queries (>3 levels)

**Example - Paginated Query:**
```graphql
query GetPositions($account: String!, $limit: Int!, $offset: Int!) {
  positions(
    where: { account: $account }
    limit: $limit
    offset: $offset
    orderBy: createdAt_DESC
  ) {
    id
    market
    sizeInUsd
    collateralAmount
  }
}
```

## RPC Node Rate Limits

### Public RPC Endpoints

When interacting with smart contracts, you use blockchain RPC nodes:

**Arbitrum Public RPC:**
- URL: `https://arb1.arbitrum.io/rpc`
- Rate limit: **~50 requests/second** (unofficial)
- May throttle heavy users

**Avalanche Public RPC:**
- URL: `https://api.avax.network/ext/bc/C/rpc`
- Rate limit: **~10 requests/second** (unofficial)
- Shared infrastructure

### Private RPC Recommendations

For production use, consider private RPC providers:

**Alchemy:**
- Free tier: 300M compute units/month
- ~3,000 requests/second on paid plans
- Recommended for high-frequency trading

**Infura:**
- Free tier: 100K requests/day
- Paid plans: Higher limits

**QuickNode:**
- Dedicated nodes available
- Customizable rate limits

**Implementation:**
```rust
let provider = Provider::<Http>::try_from(
    std::env::var("RPC_URL").unwrap_or_else(|_|
        "https://arb1.arbitrum.io/rpc".to_string()
    )
)?;
```

## Error Handling

### Rate Limit Detection

GMX may not return standard 429 errors. Detect rate limiting by:

**HTTP Status Codes:**
- `429 Too Many Requests` - Explicit rate limit
- `503 Service Unavailable` - Overloaded
- `504 Gateway Timeout` - Slow response

**Response Patterns:**
- Empty responses
- Repeated timeouts
- Increased latency (>5 seconds)

### Retry Strategy

```rust
async fn request_with_retry<T>(
    request_fn: impl Fn() -> Future<Output = Result<T>>,
    max_retries: u32,
) -> Result<T> {
    let mut retries = 0;
    let mut backoff = Duration::from_millis(100);

    loop {
        match request_fn().await {
            Ok(result) => return Ok(result),
            Err(e) if retries >= max_retries => return Err(e),
            Err(e) if is_rate_limit_error(&e) => {
                // Exponential backoff for rate limits
                tokio::time::sleep(backoff).await;
                backoff *= 2;
                backoff = backoff.min(Duration::from_secs(30));
                retries += 1;
            }
            Err(e) => return Err(e),
        }
    }
}

fn is_rate_limit_error(error: &Error) -> bool {
    matches!(error,
        Error::Http(status) if *status == 429 || *status == 503
    )
}
```

### Circuit Breaker Pattern

```rust
struct CircuitBreaker {
    failure_threshold: u32,
    failures: u32,
    last_failure: Option<Instant>,
    reset_timeout: Duration,
}

impl CircuitBreaker {
    fn new(threshold: u32, timeout: Duration) -> Self {
        Self {
            failure_threshold: threshold,
            failures: 0,
            last_failure: None,
            reset_timeout: timeout,
        }
    }

    fn is_open(&self) -> bool {
        if self.failures < self.failure_threshold {
            return false;
        }

        if let Some(last) = self.last_failure {
            last.elapsed() < self.reset_timeout
        } else {
            false
        }
    }

    fn record_success(&mut self) {
        self.failures = 0;
        self.last_failure = None;
    }

    fn record_failure(&mut self) {
        self.failures += 1;
        self.last_failure = Some(Instant::now());
    }

    async fn call<T>(
        &mut self,
        f: impl Future<Output = Result<T>>,
    ) -> Result<T> {
        if self.is_open() {
            return Err(Error::CircuitBreakerOpen);
        }

        match f.await {
            Ok(result) => {
                self.record_success();
                Ok(result)
            }
            Err(e) => {
                self.record_failure();
                Err(e)
            }
        }
    }
}
```

## Caching Strategy

Reduce API calls through intelligent caching:

### What to Cache

**Static Data (Cache: 1 hour - 24 hours):**
- Markets list
- Token addresses
- Contract addresses

**Semi-Static Data (Cache: 1-5 minutes):**
- Market info (liquidity, max OI)
- Token metadata

**Dynamic Data (Cache: 5-30 seconds):**
- Prices (tickers)
- Open interest
- Funding rates

**Don't Cache:**
- Signed prices (for transactions)
- Account positions
- Pending orders
- Real-time candlesticks

### Implementation

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

struct CachedValue<T> {
    value: T,
    cached_at: Instant,
    ttl: Duration,
}

impl<T> CachedValue<T> {
    fn is_stale(&self) -> bool {
        self.cached_at.elapsed() > self.ttl
    }
}

struct ApiCache {
    markets: Arc<RwLock<Option<CachedValue<Vec<Market>>>>>,
    tickers: Arc<RwLock<HashMap<String, CachedValue<Ticker>>>>,
}

impl ApiCache {
    async fn get_markets(&self) -> Option<Vec<Market>> {
        let cache = self.markets.read().await;
        cache.as_ref()
            .filter(|c| !c.is_stale())
            .map(|c| c.value.clone())
    }

    async fn set_markets(&self, markets: Vec<Market>) {
        let mut cache = self.markets.write().await;
        *cache = Some(CachedValue {
            value: markets,
            cached_at: Instant::now(),
            ttl: Duration::from_secs(3600), // 1 hour
        });
    }
}
```

## Websocket / Real-Time Considerations

### No Native WebSocket API

GMX does not provide a native WebSocket API for real-time updates.

**Alternatives:**

1. **Polling REST API**
   - Poll tickers every 1-5 seconds
   - Use If-Modified-Since headers (if supported)

2. **GraphQL Subscriptions**
   - Check if Subsquid supports subscriptions
   - May provide real-time position/order updates

3. **Blockchain Events**
   - Subscribe to contract events via RPC
   - Real-time order execution notifications
   - Requires WebSocket RPC connection

**Event Subscription Example:**
```rust
let ws = Provider::<Ws>::connect("wss://arb1.arbitrum.io/ws").await?;
let event_filter = Filter::new()
    .address(exchange_router)
    .event("OrderExecuted(bytes32,uint256)");

let mut stream = ws.subscribe_logs(&event_filter).await?;

while let Some(log) = stream.next().await {
    let order_key = decode_order_key(&log)?;
    println!("Order executed: {}", order_key);
}
```

## Summary: Recommended Rate Limits

| Operation | Recommended Limit | Notes |
|-----------|------------------|-------|
| REST API - Tickers | 10 req/s | Conservative estimate |
| REST API - Candlesticks | 5 req/s | Higher data volume |
| REST API - Market Info | 2 req/s | Complex responses |
| GraphQL Queries | 5-10 req/s | Subsquid infrastructure |
| Smart Contract Reads | 20 req/s | RPC dependent |
| Smart Contract Writes | 1 tx/2s | Blockchain + nonce limits |
| Public RPC (Arbitrum) | 20-50 req/s | Unofficial limit |
| Public RPC (Avalanche) | 10 req/s | Unofficial limit |

## Implementation Checklist

### Rate Limiting Module (`rate_limits.rs`)

- [ ] Implement token bucket rate limiter
- [ ] Per-endpoint rate limit configuration
- [ ] Fallback URL rotation logic
- [ ] Circuit breaker for endpoint failures
- [ ] Exponential backoff retry strategy
- [ ] Rate limit error detection
- [ ] Cache layer for static/semi-static data
- [ ] TTL-based cache invalidation
- [ ] Concurrent request throttling
- [ ] Request queue with priority

### Monitoring

- [ ] Track requests per second by endpoint
- [ ] Log rate limit errors
- [ ] Monitor endpoint health (success rate)
- [ ] Alert on repeated failures
- [ ] Track fallback URL usage

## Sources

Since GMX does not publish official rate limit documentation, this document is based on:
- Best practices for public APIs
- Blockchain network constraints
- Observed infrastructure patterns
- General decentralized exchange considerations

**References:**
- [GMX REST API Documentation](https://docs.gmx.io/docs/api/rest/)
- [Arbitrum Network Specifications](https://docs.arbitrum.io/)
- [Avalanche Network Specifications](https://docs.avax.network/)
- [Subsquid Documentation](https://docs.sqd.dev/)
