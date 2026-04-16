# Crypto.com Exchange API v1 - Rate Limits

## Overview

Crypto.com Exchange API v1 enforces strict rate limits to ensure fair usage and system stability. Rate limits vary by endpoint category and are measured in requests per time window.

**Important:** Exceeding rate limits results in error code `10007` (THROTTLE_REACHED) and temporary blocking.

---

## REST API Rate Limits

### Trading Endpoints (Critical)

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| `private/create-order` | 15 req | 100ms | Order creation |
| `private/cancel-order` | 15 req | 100ms | Order cancellation |
| `private/cancel-all-orders` | 15 req | 100ms | Bulk cancellation |

**Critical Trading Limits:**
- 15 requests per 100 milliseconds = 150 requests/second
- Shared pool across create/cancel operations
- Highest priority for market makers

---

### Order Query Endpoints

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| `private/get-order-detail` | 30 req | 100ms | Single order details |

**Query Limits:**
- 30 requests per 100ms = 300 requests/second
- Higher limit due to read-only nature

---

### Historical Data Endpoints

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| `private/get-trades` | 1 req | 1 second | User trade history |
| `private/get-order-history` | 1 req | 1 second | Order history |

**Historical Limits:**
- 1 request per second
- Lower priority due to archival nature
- Use pagination for large datasets

---

### Other Private Endpoints

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| `private/user-balance` | 3 req | 100ms | Account balance |
| `private/get-positions` | 3 req | 100ms | Open positions |
| `private/get-open-orders` | 3 req | 100ms | Active orders |
| `private/get-accounts` | 3 req | 100ms | Account details |
| `private/get-fee-rate` | 3 req | 100ms | Fee rates |

**General Private Limits:**
- 3 requests per 100ms = 30 requests/second
- Applies to most account/position queries

---

### Public Market Data Endpoints

| Endpoint | Limit | Window | Notes |
|----------|-------|--------|-------|
| `public/get-instruments` | 100 req | 1 second | Instrument list |
| `public/get-book` | 100 req | 1 second | Order book |
| `public/get-tickers` | 100 req | 1 second | Ticker data |
| `public/get-trades` | 100 req | 1 second | Recent trades |
| `public/get-candlestick` | 100 req | 1 second | OHLCV data |
| `public/get-valuations` | 100 req | 1 second | Mark/index prices |

**Public Data Limits:**
- 100 requests per second
- Shared pool across all public endpoints
- No authentication required

---

## WebSocket Rate Limits

### User API WebSocket

| Category | Limit | Notes |
|----------|-------|-------|
| User API Requests | 150 req/second | All authenticated requests |
| Subscription Requests | 150 req/second | Subscribe/unsubscribe |

**Connection Limits:**
- Add **1-second sleep** after establishing connection
- Rate limits are **pro-rated** based on connection timestamp
- Multiple connections allowed (subject to total limits)

---

### Market Data WebSocket

| Category | Limit | Notes |
|----------|-------|-------|
| Market Data Requests | 100 req/second | All public subscriptions |
| Subscription Requests | 100 req/second | Subscribe/unsubscribe |

**Connection Limits:**
- Add **1-second sleep** after establishing connection
- Rate limits pro-rated by connection time
- Separate limit pool from User API

---

## Rate Limit Implementation

### Basic Rate Limiter (Rust)

```rust
use std::time::{Duration, Instant};
use std::collections::VecDeque;

pub struct RateLimiter {
    requests: VecDeque<Instant>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            requests: VecDeque::new(),
            max_requests,
            window,
        }
    }

    pub async fn acquire(&mut self) {
        let now = Instant::now();

        // Remove expired requests
        while let Some(&first) = self.requests.front() {
            if now.duration_since(first) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Wait if limit exceeded
        if self.requests.len() >= self.max_requests {
            if let Some(&first) = self.requests.front() {
                let wait_time = self.window - now.duration_since(first);
                tokio::time::sleep(wait_time).await;

                // Clean up again after waiting
                let now = Instant::now();
                while let Some(&first) = self.requests.front() {
                    if now.duration_since(first) > self.window {
                        self.requests.pop_front();
                    } else {
                        break;
                    }
                }
            }
        }

        self.requests.push_back(now);
    }
}
```

---

### Multi-Tier Rate Limiter

```rust
use std::collections::HashMap;

pub struct MultiTierRateLimiter {
    limiters: HashMap<String, RateLimiter>,
}

impl MultiTierRateLimiter {
    pub fn new() -> Self {
        let mut limiters = HashMap::new();

        // Trading endpoints: 15 req/100ms
        limiters.insert(
            "trading".to_string(),
            RateLimiter::new(15, Duration::from_millis(100)),
        );

        // Order detail: 30 req/100ms
        limiters.insert(
            "order_detail".to_string(),
            RateLimiter::new(30, Duration::from_millis(100)),
        );

        // Historical: 1 req/second
        limiters.insert(
            "historical".to_string(),
            RateLimiter::new(1, Duration::from_secs(1)),
        );

        // Other private: 3 req/100ms
        limiters.insert(
            "private".to_string(),
            RateLimiter::new(3, Duration::from_millis(100)),
        );

        // Public: 100 req/second
        limiters.insert(
            "public".to_string(),
            RateLimiter::new(100, Duration::from_secs(1)),
        );

        Self { limiters }
    }

    pub async fn acquire(&mut self, category: &str) {
        if let Some(limiter) = self.limiters.get_mut(category) {
            limiter.acquire().await;
        }
    }
}
```

---

### Usage Example

```rust
pub struct CryptoComConnector {
    rate_limiter: MultiTierRateLimiter,
}

impl CryptoComConnector {
    pub async fn create_order(&mut self, params: OrderParams) -> Result<Order, Error> {
        // Wait for rate limit
        self.rate_limiter.acquire("trading").await;

        // Make API request
        let response = self.http_client
            .post("https://api.crypto.com/exchange/v1/private/create-order")
            .json(&params)
            .send()
            .await?;

        // Process response
        Ok(response.json().await?)
    }

    pub async fn get_ticker(&mut self, symbol: &str) -> Result<Ticker, Error> {
        // Public endpoint
        self.rate_limiter.acquire("public").await;

        let response = self.http_client
            .post("https://api.crypto.com/exchange/v1/public/get-tickers")
            .json(&json!({
                "id": 1,
                "method": "public/get-tickers",
                "params": { "instrument_name": symbol },
                "nonce": generate_nonce()
            }))
            .send()
            .await?;

        Ok(response.json().await?)
    }
}
```

---

## Endpoint Categories

### Trading Category
```rust
const TRADING_ENDPOINTS: &[&str] = &[
    "private/create-order",
    "private/cancel-order",
    "private/cancel-all-orders",
    "private/amend-order",
];
```

### Order Detail Category
```rust
const ORDER_DETAIL_ENDPOINTS: &[&str] = &[
    "private/get-order-detail",
];
```

### Historical Category
```rust
const HISTORICAL_ENDPOINTS: &[&str] = &[
    "private/get-trades",
    "private/get-order-history",
];
```

### Private Category
```rust
const PRIVATE_ENDPOINTS: &[&str] = &[
    "private/user-balance",
    "private/get-positions",
    "private/get-open-orders",
    "private/get-accounts",
    "private/get-fee-rate",
    "private/get-transactions",
];
```

### Public Category
```rust
const PUBLIC_ENDPOINTS: &[&str] = &[
    "public/get-instruments",
    "public/get-book",
    "public/get-tickers",
    "public/get-trades",
    "public/get-candlestick",
    "public/get-valuations",
];
```

---

## Error Handling

### Rate Limit Error Response

```json
{
  "id": 1,
  "method": "private/create-order",
  "code": 10007,
  "message": "THROTTLE_REACHED"
}
```

**Error Code:** `10007`
**Message:** `THROTTLE_REACHED`

### Retry Strategy

```rust
pub async fn request_with_retry<T>(
    &mut self,
    endpoint: &str,
    params: Value,
    max_retries: u32,
) -> Result<T, Error>
where
    T: serde::de::DeserializeOwned,
{
    let mut retries = 0;

    loop {
        match self.request(endpoint, params.clone()).await {
            Ok(response) => return Ok(response),
            Err(Error::RateLimitExceeded) if retries < max_retries => {
                retries += 1;
                let backoff = Duration::from_millis(100 * 2u64.pow(retries));
                tokio::time::sleep(backoff).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## WebSocket Connection Management

### Connection with Delay

```rust
pub async fn connect_websocket(&self, url: &str) -> Result<WebSocket, Error> {
    let (ws_stream, _) = tokio_tungstenite::connect_async(url).await?;

    // CRITICAL: Add 1-second sleep after connection
    tokio::time::sleep(Duration::from_secs(1)).await;

    Ok(ws_stream)
}
```

### Subscription Rate Limiting

```rust
pub struct WebSocketClient {
    rate_limiter: RateLimiter,
}

impl WebSocketClient {
    pub async fn subscribe(&mut self, channels: Vec<String>) -> Result<(), Error> {
        for channel in channels {
            // Rate limit subscriptions
            self.rate_limiter.acquire().await;

            let subscribe_msg = json!({
                "id": generate_id(),
                "method": "subscribe",
                "params": {
                    "channels": [channel]
                },
                "nonce": generate_nonce()
            });

            self.send(subscribe_msg).await?;
        }

        Ok(())
    }
}
```

---

## Best Practices

### 1. Pre-emptive Rate Limiting
```rust
// Enforce limits before making requests
self.rate_limiter.acquire("trading").await;
let result = self.api_request(...).await?;
```

### 2. Connection Pooling
```rust
// Use connection pooling to avoid reconnection overhead
let client = reqwest::Client::builder()
    .pool_max_idle_per_host(10)
    .build()?;
```

### 3. Batch Operations
```rust
// Use cancel-all-orders instead of multiple cancel-order calls
self.cancel_all_orders(instrument).await?;
```

### 4. WebSocket Preference
```rust
// Use WebSocket for frequent updates instead of REST polling
// Subscribe to user.order instead of polling get-open-orders
```

### 5. Cache Public Data
```rust
// Cache instrument list (changes rarely)
pub struct InstrumentCache {
    data: HashMap<String, Instrument>,
    last_refresh: Instant,
    ttl: Duration,
}

impl InstrumentCache {
    pub async fn get(&mut self, api: &mut ApiClient) -> &HashMap<String, Instrument> {
        if self.last_refresh.elapsed() > self.ttl {
            self.data = api.get_instruments().await.unwrap();
            self.last_refresh = Instant::now();
        }
        &self.data
    }
}
```

---

## Monitoring Rate Limits

### Track Usage

```rust
pub struct RateLimitMetrics {
    pub requests_made: u64,
    pub requests_throttled: u64,
    pub last_throttle: Option<Instant>,
}

impl RateLimiter {
    pub fn metrics(&self) -> RateLimitMetrics {
        RateLimitMetrics {
            requests_made: self.total_requests,
            requests_throttled: self.throttle_count,
            last_throttle: self.last_throttle_time,
        }
    }
}
```

### Logging

```rust
pub async fn acquire(&mut self) {
    let wait_time = self.calculate_wait_time();

    if wait_time > Duration::ZERO {
        log::warn!(
            "Rate limit reached, waiting {:?} (category: {})",
            wait_time,
            self.category
        );
        tokio::time::sleep(wait_time).await;
    }

    self.record_request();
}
```

---

## Testing Rate Limits

### Unit Test

```rust
#[tokio::test]
async fn test_rate_limiter() {
    let mut limiter = RateLimiter::new(5, Duration::from_secs(1));

    let start = Instant::now();

    // First 5 should be immediate
    for _ in 0..5 {
        limiter.acquire().await;
    }
    assert!(start.elapsed() < Duration::from_millis(100));

    // 6th should wait
    limiter.acquire().await;
    assert!(start.elapsed() >= Duration::from_secs(1));
}
```

### Integration Test

```rust
#[tokio::test]
async fn test_trading_rate_limit() {
    let mut connector = CryptoComConnector::new();

    let start = Instant::now();

    // Create 15 orders (should be immediate)
    for i in 0..15 {
        connector.create_order(test_order(i)).await.unwrap();
    }
    assert!(start.elapsed() < Duration::from_millis(200));

    // 16th order should wait
    connector.create_order(test_order(15)).await.unwrap();
    assert!(start.elapsed() >= Duration::from_millis(100));
}
```

---

## Troubleshooting

### Issue: Frequent THROTTLE_REACHED Errors

**Causes:**
1. Not implementing rate limiting
2. Incorrect rate limit windows
3. Multiple instances using same API key
4. Bursty request patterns

**Solutions:**
```rust
// 1. Implement proper rate limiting
self.rate_limiter.acquire().await;

// 2. Use correct windows
RateLimiter::new(15, Duration::from_millis(100)) // Not 1 second!

// 3. Coordinate across instances
// Use distributed rate limiter (Redis-based)

// 4. Smooth out requests
tokio::time::sleep(Duration::from_millis(10)).await;
```

---

### Issue: WebSocket Connection Drops

**Cause:** Not waiting 1 second after connection

**Solution:**
```rust
let ws = connect(url).await?;
tokio::time::sleep(Duration::from_secs(1)).await; // CRITICAL
ws.subscribe(channels).await?;
```

---

## Summary Table

| Category | Limit | Window | Endpoints |
|----------|-------|--------|-----------|
| Trading | 15 | 100ms | create/cancel/amend order |
| Order Detail | 30 | 100ms | get-order-detail |
| Historical | 1 | 1s | get-trades, get-order-history |
| Private | 3 | 100ms | balance, positions, etc. |
| Public | 100 | 1s | market data |
| WS User | 150 | 1s | user subscriptions |
| WS Market | 100 | 1s | market subscriptions |

---

## Implementation Checklist

- [ ] Rate limiter implemented for all categories
- [ ] Multi-tier rate limiting based on endpoint
- [ ] WebSocket 1-second delay enforced
- [ ] Retry logic for THROTTLE_REACHED errors
- [ ] Exponential backoff for retries
- [ ] Request metrics tracked
- [ ] Rate limit errors logged
- [ ] Connection pooling configured
- [ ] Public data cached appropriately
- [ ] WebSocket preferred over REST polling
- [ ] Unit tests for rate limiters
- [ ] Integration tests for API limits
- [ ] Monitoring dashboard for rate limit metrics
