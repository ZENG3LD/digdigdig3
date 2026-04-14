# HTX API Rate Limits

Complete rate limiting documentation for HTX (formerly Huobi) exchange API.

## Overview

HTX implements rate limiting to ensure API stability and fair usage. Rate limits are applied:

- **Per UID** (User ID) across all API keys
- Different limits for different endpoint categories
- Separate limits for REST and WebSocket
- Response headers indicate remaining quota

## Rate Limit Policy

### Key Principles

1. **UID-based limiting**: Limits apply to the entire user account, not individual API keys
2. **Endpoint-specific**: Different endpoints have different limits
3. **Time windows**: Limits reset at fixed intervals (1 second, 3 seconds, etc.)
4. **Penalty for excess**: Exceeding limits may result in temporary bans

## REST API Rate Limits

### Spot Trading Limits

#### Public Endpoints (Market Data)

**IP-based limits** (per IP address):

| Category | Limit | Window | Notes |
|----------|-------|--------|-------|
| Market data | 800 requests | 1 second | Klines, depth, trades, tickers |
| Reference data | 100 requests | 1 second | Symbols, currencies, timestamps |
| System status | 50 requests | 1 second | Market status, server info |

**Examples:**
- `GET /market/history/kline` - 800/sec per IP
- `GET /market/depth` - 800/sec per IP
- `GET /market/tickers` - 800/sec per IP
- `GET /v2/settings/common/symbols` - 100/sec per IP

#### Private Endpoints (Authenticated)

**UID-based limits** (per user account):

| Category | Limit | Window | Notes |
|----------|-------|--------|-------|
| Account queries | 100 requests | 1 second | Balance, ledger, history |
| Order placement | 100 requests | 1 second | Place, cancel orders |
| Order queries | 50 requests | 1 second | Order status, history |
| Trade queries | 50 requests | 1 second | Match results |
| Wallet operations | 20 requests | 1 second | Deposit, withdrawal |

**Detailed breakdown:**

**Account endpoints:**
- `GET /v1/account/accounts` - 100/sec per UID
- `GET /v1/account/accounts/{id}/balance` - 100/sec per UID
- `GET /v2/account/ledger` - 50/sec per UID
- `GET /v2/account/asset-valuation` - 100/sec per UID

**Trading endpoints:**
- `POST /v1/order/orders/place` - 100/sec per UID
- `POST /v1/order/batch-orders` - 50/sec per UID
- `POST /v1/order/orders/{id}/submitcancel` - 100/sec per UID
- `POST /v1/order/orders/batchcancel` - 50/sec per UID
- `GET /v1/order/openOrders` - 50/sec per UID
- `GET /v1/order/orders` - 50/sec per UID
- `GET /v1/order/matchresults` - 50/sec per UID

**Wallet endpoints:**
- `GET /v2/account/deposit/address` - 20/sec per UID
- `POST /v1/dw/withdraw/api/create` - 20/sec per UID
- `GET /v1/query/deposit-withdraw` - 20/sec per UID

### Futures/Contracts Limits

#### Coin-Margined Futures

**Public endpoints:**
- IP-based: 120 requests per 3 seconds (~40/sec)

**Private endpoints:**
- UID-based: 72 requests per 3 seconds (~24/sec)
  - Trading: 36 requests per 3 seconds (~12/sec)
  - Query: 36 requests per 3 seconds (~12/sec)

#### USDT-Margined Contracts

**Public endpoints:**
- IP-based: 240 requests per 3 seconds (~80/sec)

**Private endpoints:**
- UID-based: 144 requests per 3 seconds (~48/sec)
  - Trading: 72 requests per 3 seconds (~24/sec)
  - Query: 72 requests per 3 seconds (~24/sec)

## WebSocket Rate Limits

### Connection Limits

| WebSocket Type | Max Connections | Per |
|----------------|-----------------|-----|
| Market data (v1) | 100 | IP address |
| Account/Orders (v2) | 10 | API key |
| MBP feed | 50 | IP address |

**Important:** Maximum 10 concurrent WebSocket v2 connections per API key.

### Subscription Limits

| Action | Limit | Notes |
|--------|-------|-------|
| `sub` (subscribe) | Unlimited | Server pushes data |
| `req` (request) | 50 per connection | One-time requests |
| `unsub` (unsubscribe) | Unlimited | Remove subscriptions |

**Examples:**
```json
// Subscribe - no limit
{"sub": "market.btcusdt.kline.1min"}

// Request - max 50/connection
{"req": "market.btcusdt.kline.1min"}

// Unsubscribe - no limit
{"unsub": "market.btcusdt.kline.1min"}
```

### Message Rate

No explicit limit on incoming message frequency from server. Handle burst updates appropriately.

## Rate Limit Headers

### Response Headers

Every API response includes rate limit headers:

```
X-HB-RateLimit-Requests-Remain: 95
X-HB-RateLimit-Requests-Expire: 1629384060000
```

**Header Descriptions:**

| Header | Type | Description |
|--------|------|-------------|
| `X-HB-RateLimit-Requests-Remain` | integer | Remaining requests in current window |
| `X-HB-RateLimit-Requests-Expire` | long | Window expiration timestamp (ms) |

**Example:**
```
X-HB-RateLimit-Requests-Remain: 95
X-HB-RateLimit-Requests-Expire: 1629384001000
```

Means:
- 95 requests remaining
- Window resets at timestamp 1629384001000 (Unix ms)

### Monitoring Rate Limits

```rust
use reqwest::Response;

async fn check_rate_limit(response: &Response) {
    if let Some(remain) = response.headers().get("X-HB-RateLimit-Requests-Remain") {
        let remaining: i32 = remain.to_str().unwrap().parse().unwrap();
        println!("Remaining requests: {}", remaining);

        if remaining < 10 {
            println!("WARNING: Approaching rate limit!");
        }
    }

    if let Some(expire) = response.headers().get("X-HB-RateLimit-Requests-Expire") {
        let expiry_ms: i64 = expire.to_str().unwrap().parse().unwrap();
        println!("Rate limit resets at: {}", expiry_ms);
    }
}
```

## Rate Limit Errors

### Error Response

When rate limit is exceeded:

```json
{
  "status": "error",
  "err-code": "gateway-internal-error",
  "err-msg": "request too frequent",
  "data": null
}
```

Or:

```json
{
  "status": "error",
  "err-code": "api-request-too-frequent",
  "err-msg": "Request too frequent. Please try again later.",
  "data": null
}
```

### HTTP Status Codes

| Status | Meaning | Action |
|--------|---------|--------|
| `429` | Too Many Requests | Back off and retry |
| `503` | Service Unavailable | Server overload, retry with backoff |

## Best Practices

### 1. Implement Rate Limiting

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

pub struct RateLimiter {
    max_requests: u32,
    window: Duration,
    requests: Vec<Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: u32, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            requests: Vec::new(),
        }
    }

    pub async fn wait(&mut self) {
        let now = Instant::now();

        // Remove expired requests
        self.requests.retain(|&req_time| {
            now.duration_since(req_time) < self.window
        });

        // Wait if limit reached
        if self.requests.len() >= self.max_requests as usize {
            let oldest = self.requests[0];
            let wait_duration = self.window - now.duration_since(oldest);
            sleep(wait_duration).await;

            // Clean up again after waiting
            let now = Instant::now();
            self.requests.retain(|&req_time| {
                now.duration_since(req_time) < self.window
            });
        }

        self.requests.push(now);
    }
}

// Usage
let mut limiter = RateLimiter::new(100, Duration::from_secs(1));

for _ in 0..150 {
    limiter.wait().await;
    // Make API call
}
```

### 2. Exponential Backoff

```rust
use std::time::Duration;
use tokio::time::sleep;

pub async fn retry_with_backoff<F, T, E>(
    mut func: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut retries = 0;

    loop {
        match func() {
            Ok(result) => return Ok(result),
            Err(e) if retries >= max_retries => return Err(e),
            Err(_) => {
                let wait_ms = 2u64.pow(retries) * 100; // 100ms, 200ms, 400ms, ...
                sleep(Duration::from_millis(wait_ms)).await;
                retries += 1;
            }
        }
    }
}

// Usage
let result = retry_with_backoff(
    || make_api_call(),
    5
).await?;
```

### 3. Request Queueing

```rust
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};

pub struct RequestQueue {
    tx: mpsc::Sender<String>,
}

impl RequestQueue {
    pub fn new(rate_per_sec: u64) -> Self {
        let (tx, mut rx) = mpsc::channel(1000);

        tokio::spawn(async move {
            let mut ticker = interval(Duration::from_millis(1000 / rate_per_sec));

            while let Some(request) = rx.recv().await {
                ticker.tick().await;
                // Process request
                println!("Processing: {}", request);
            }
        });

        Self { tx }
    }

    pub async fn send(&self, request: String) -> Result<(), String> {
        self.tx.send(request).await.map_err(|e| e.to_string())
    }
}

// Usage
let queue = RequestQueue::new(100); // 100 req/sec

for i in 0..200 {
    queue.send(format!("Request {}", i)).await?;
}
```

### 4. Monitor Response Headers

```rust
pub struct RateLimitTracker {
    remaining: i32,
    reset_time: i64,
}

impl RateLimitTracker {
    pub fn update_from_response(&mut self, response: &Response) {
        if let Some(remain) = response.headers().get("X-HB-RateLimit-Requests-Remain") {
            self.remaining = remain.to_str().unwrap().parse().unwrap_or(0);
        }

        if let Some(expire) = response.headers().get("X-HB-RateLimit-Requests-Expire") {
            self.reset_time = expire.to_str().unwrap().parse().unwrap_or(0);
        }
    }

    pub fn should_throttle(&self) -> bool {
        self.remaining < 10 // Throttle when < 10 requests remain
    }

    pub fn time_until_reset(&self) -> Duration {
        let now = chrono::Utc::now().timestamp_millis();
        let wait_ms = (self.reset_time - now).max(0) as u64;
        Duration::from_millis(wait_ms)
    }
}
```

### 5. Use WebSocket for Real-time Data

Instead of polling REST endpoints, use WebSocket subscriptions:

**Bad (polling):**
```rust
// Polls every 100ms - wastes rate limit
loop {
    let ticker = get_ticker("btcusdt").await?;
    sleep(Duration::from_millis(100)).await;
}
```

**Good (WebSocket):**
```rust
// Subscribe once, receive updates
let ws = connect_websocket().await?;
ws.subscribe("market.btcusdt.ticker").await?;

while let Some(msg) = ws.next().await {
    // Process ticker update
}
```

### 6. Batch Operations

Use batch endpoints when available:

**Bad:**
```rust
for order in orders {
    cancel_order(order.id).await?; // Multiple API calls
}
```

**Good:**
```rust
cancel_batch_orders(order_ids).await?; // Single API call
```

## Penalty System

### Warning Levels

HTX may apply penalties for excessive rate limit violations:

| Violation | Penalty | Duration |
|-----------|---------|----------|
| Occasional | None | - |
| Frequent (< 10/day) | Warning | - |
| Excessive (> 10/day) | Temporary ban | 1-2 hours |
| Severe abuse | Account suspension | Variable |

### Avoiding Penalties

1. Monitor rate limit headers
2. Implement client-side rate limiting
3. Use exponential backoff on errors
4. Prefer WebSocket for real-time data
5. Batch operations when possible
6. Cache frequently accessed data

## Special Considerations

### Sub-Users

Rate limits are shared across parent and all sub-users under the same UID.

**Example:**
- Parent account: 50 req/sec
- Sub-user 1: 30 req/sec
- Sub-user 2: 20 req/sec
- **Total: 100 req/sec** (hits limit)

### API Key Rotation

Rate limits are per UID, not per API key. Rotating API keys **does not** increase limits.

### IP Whitelisting

IP whitelisting does not affect rate limits. Public endpoints are limited per IP regardless of whitelisting.

### Trading vs Query Limits

Some endpoints have separate limits for trading and query operations:

**Futures (coin-margined):**
- Trading: 36 req/3sec
- Query: 36 req/3sec
- **Total: 72 req/3sec**

If you only trade (no queries), you get full 36 req/3sec for trading operations.

## Rate Limit by Endpoint Type

### Quick Reference

| Endpoint Category | Limit | Window | Basis |
|-------------------|-------|--------|-------|
| Public market data | 800 | 1 sec | IP |
| Public reference data | 100 | 1 sec | IP |
| Account queries | 100 | 1 sec | UID |
| Order placement | 100 | 1 sec | UID |
| Order queries | 50 | 1 sec | UID |
| Trade queries | 50 | 1 sec | UID |
| Wallet operations | 20 | 1 sec | UID |
| WebSocket connections | 10 | - | API key |
| WebSocket requests | 50 | - | Connection |

## Implementation Example

### Complete Rate Limiter

```rust
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};

pub enum EndpointCategory {
    PublicMarketData,
    PublicReference,
    AccountQuery,
    OrderPlacement,
    OrderQuery,
    TradeQuery,
    Wallet,
}

impl EndpointCategory {
    fn limit(&self) -> (u32, Duration) {
        match self {
            Self::PublicMarketData => (800, Duration::from_secs(1)),
            Self::PublicReference => (100, Duration::from_secs(1)),
            Self::AccountQuery => (100, Duration::from_secs(1)),
            Self::OrderPlacement => (100, Duration::from_secs(1)),
            Self::OrderQuery => (50, Duration::from_secs(1)),
            Self::TradeQuery => (50, Duration::from_secs(1)),
            Self::Wallet => (20, Duration::from_secs(1)),
        }
    }
}

pub struct HTXRateLimiter {
    limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
}

impl HTXRateLimiter {
    pub fn new() -> Self {
        Self {
            limiters: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn wait(&self, category: EndpointCategory) {
        let key = format!("{:?}", category);
        let mut limiters = self.limiters.write().await;

        let limiter = limiters.entry(key).or_insert_with(|| {
            let (max, window) = category.limit();
            RateLimiter::new(max, window)
        });

        limiter.wait().await;
    }
}

// Usage
let limiter = HTXRateLimiter::new();

limiter.wait(EndpointCategory::OrderPlacement).await;
place_order().await?;

limiter.wait(EndpointCategory::PublicMarketData).await;
get_ticker().await?;
```

## Summary

- **UID-based**: Limits apply per user account across all API keys
- **Monitor headers**: Track `X-HB-RateLimit-Requests-Remain`
- **Different limits**: Public (IP-based) vs Private (UID-based)
- **WebSocket preferred**: For real-time data (no polling)
- **Implement client-side limiting**: Don't rely on server enforcement
- **Use backoff**: Exponential backoff on errors
- **Batch operations**: Reduce API call count

Key limits to remember:
- Public market data: **800/sec per IP**
- Private trading: **100/sec per UID**
- Order queries: **50/sec per UID**
- WebSocket connections: **10 per API key**
