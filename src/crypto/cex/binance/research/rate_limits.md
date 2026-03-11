# Binance API Rate Limits

**Last Updated:** 2026-01-20
**API Version:** Spot API v3, Futures API v1

## Table of Contents

1. [Overview](#overview)
2. [REST API Limits](#rest-api-limits)
3. [Response Headers](#response-headers)
4. [Error Handling](#error-handling)
5. [WebSocket Limits](#websocket-limits)
6. [Endpoint Weights](#endpoint-weights)
7. [Best Practices](#best-practices)
8. [Implementation Notes](#implementation-notes)

---

## Overview

Binance uses a **weight-based rate limiting system** where each API endpoint consumes a specific amount of "weight" from your rate limit allowance. This is more sophisticated than simple request counting.

### Key Concepts

- **IP-Based Limits**: All rate limits are based on IP address, NOT API keys
- **Weight System**: Each endpoint has a weight value (1-250+) based on computational cost
- **Multiple Limit Types**: REQUEST_WEIGHT, RAW_REQUEST, and ORDERS are tracked separately
- **Shared Pool**: All API keys from the same IP share the same rate limit pool

---

## REST API Limits

### Spot API

#### REQUEST_WEIGHT Limits

**Primary Limit:**
- **6,000 weight per minute** (per IP address)
- This is the main limit that most endpoints count against

**Individual Endpoint Limits:**
- Some endpoints have independent limits
- IP-based endpoints: **12,000 per minute** (per endpoint)
- UID-based endpoints: **180,000 per minute** (per endpoint)

#### Order Rate Limits

**Per Account (across all API keys):**
- **50 orders per 10 seconds**
- **160,000 orders per 24 hours**

**Unfilled Order Count:**
- Tracks open/unfilled orders per account
- Filled orders don't count toward this limit
- Check status via `GET /api/v3/rateLimit/order`

### Futures API (USDS-Margined)

#### REQUEST_WEIGHT Limits

**Same as Spot:**
- **6,000 weight per minute** (per IP address)

#### Order Rate Limits

**Same as Spot:**
- **50 orders per 10 seconds** (per account)
- **160,000 orders per 24 hours** (per account)

### Rate Limit Types

Binance tracks three types of limits (found in `/api/v3/exchangeInfo` or `/fapi/v1/exchangeInfo`):

1. **REQUEST_WEIGHT**: Weighted request limits (main limit)
2. **RAW_REQUEST**: Raw request count limits
3. **ORDERS**: Order-specific limits

---

## Response Headers

Binance includes rate limit tracking headers in every API response.

### X-MBX-USED-WEIGHT Headers

**Format:** `X-MBX-USED-WEIGHT-(intervalNum)(intervalLetter)`

**Examples:**
- `X-MBX-USED-WEIGHT-1M`: Current weight used in the last 1 minute
- Shows cumulative weight consumption for your IP

**Example Response:**
```http
HTTP/1.1 200 OK
X-MBX-USED-WEIGHT-1M: 245
Content-Type: application/json
...
```

### X-MBX-ORDER-COUNT Headers

**Format:** `X-MBX-ORDER-COUNT-(intervalNum)(intervalLetter)`

**Examples:**
- `X-MBX-ORDER-COUNT-1S`: Orders placed in last 1 second
- `X-MBX-ORDER-COUNT-1M`: Orders placed in last 1 minute
- `X-MBX-ORDER-COUNT-1D`: Orders placed in last 1 day

**Example Response:**
```http
HTTP/1.1 200 OK
X-MBX-ORDER-COUNT-1S: 2
X-MBX-ORDER-COUNT-1M: 15
X-MBX-ORDER-COUNT-1D: 347
Content-Type: application/json
...
```

**Important Notes:**
- Only **successful** orders include these headers
- Rejected/unsuccessful orders may not include order count headers
- These track against your 50/10s and 160,000/24h limits

### Interval Codes

- `S` = Seconds
- `M` = Minutes
- `D` = Days

---

## Error Handling

### HTTP Status Codes

#### 429 - Rate Limit Exceeded

**When:** You've exceeded any rate limit (weight, raw requests, or orders)

**Response:**
```json
{
  "code": -1003,
  "msg": "Too many requests; current limit is 6000 requests per 1 MINUTE."
}
```

**Headers:**
- `Retry-After`: Number of seconds to wait before retrying

**Action Required:**
- **Back off immediately**
- Wait for the time specified in `Retry-After` header
- Do NOT continue sending requests

#### 418 - IP Auto-Banned

**When:** IP continues sending requests after receiving 429s

**Response:**
```json
{
  "code": -1003,
  "msg": "Way too many requests; IP banned until 1640123456789. Please use the websocket for live updates to avoid bans."
}
```

**Headers:**
- `Retry-After`: Number of seconds until ban expires

**Ban Durations (escalating):**
- First offense: **2 minutes**
- Repeat offenses: **up to 3 days**
- Ban durations scale with repeated violations

**Critical:**
- **IP bans are tracked** and persist across repeated violations
- Bans affect ALL users on the same IP
- Recovery requires waiting out the full ban duration

### Best Response to Rate Limits

1. **Immediately stop** sending requests
2. **Respect** `Retry-After` header values
3. **Implement exponential backoff** for additional safety
4. **Switch to WebSocket streams** for real-time data
5. **Never ignore 429** responses

---

## WebSocket Limits

### WebSocket API (websocket-api)

**Connection Limits:**
- **300 connection attempts per 5 minutes** (per IP)

**REQUEST_WEIGHT Limits:**
- **6,000 weight per minute** (same as REST)
- Connecting to WebSocket API costs **2 weight**

**Order Limits:**
- **50 orders per 10 seconds** (per account)
- **160,000 orders per 24 hours** (per account)

**Response:**
```json
{
  "id": "request-id",
  "status": 200,
  "rateLimits": [
    {
      "rateLimitType": "REQUEST_WEIGHT",
      "interval": "MINUTE",
      "intervalNum": 1,
      "limit": 6000,
      "count": 245
    }
  ],
  "result": {...}
}
```

**Rate Limit Control:**
- `returnRateLimits` parameter: Can disable rate limit info in responses
- Available in connection URL or individual requests

### WebSocket Streams (stream.binance.com)

**Connection Limits:**
- **300 connection attempts per 5 minutes** (per IP)

**Streams Per Connection:**
- **Maximum 1,024 streams** per single connection
- Exceeding this requires opening additional connections

**Message Rate Limits:**
- **5 incoming messages per second** (Spot streams)
- **10 incoming messages per second** (Futures streams)
- Includes: PING frames, PONG frames, JSON control messages (subscribe/unsubscribe)

**Connection Duration:**
- **24 hours maximum** per connection
- Expect automatic disconnect at 24-hour mark
- Must reconnect after 24 hours

**Ping/Pong Requirements:**
- Server sends PING every **20 seconds**
- Client must respond with PONG within **60 seconds**
- No response = automatic disconnect

**Enforcement:**
- Connections exceeding limits are disconnected immediately
- IPs repeatedly disconnected may be **banned**

---

## Endpoint Weights

### Market Data Endpoints

| Endpoint | Weight | Notes |
|----------|--------|-------|
| **Order Book** (`GET /api/v3/depth`) | | |
| - Limit 1-100 | 5 | |
| - Limit 101-500 | 25 | |
| - Limit 501-1000 | 50 | |
| - Limit 1001-5000 | 250 | Heavy operation |
| **Recent Trades** (`GET /api/v3/trades`) | 25 | |
| **Historical Trades** (`GET /api/v3/historicalTrades`) | 25 | |
| **Aggregate Trades** (`GET /api/v3/aggTrades`) | 4 | |
| **Klines/Candlesticks** (`GET /api/v3/klines`) | 2 | |
| **UIKlines** (`GET /api/v3/uiKlines`) | 2 | |
| **Current Average Price** (`GET /api/v3/avgPrice`) | 2 | |
| **24hr Ticker** (`GET /api/v3/ticker/24hr`) | | |
| - Single symbol | 2 | |
| - 1-20 symbols | 2 | |
| - 21-100 symbols | 40 | |
| - 101+ symbols or all | 80 | Heavy operation |
| **Trading Day Ticker** (`GET /api/v3/ticker/tradingDay`) | 4 per symbol | Capped at 200 for 50+ symbols |
| **Symbol Price Ticker** (`GET /api/v3/ticker/price`) | | |
| - Single symbol | 2 | |
| - All symbols/multiple | 4 | |
| **Order Book Ticker** (`GET /api/v3/ticker/bookTicker`) | | |
| - Single symbol | 2 | |
| - All symbols/multiple | 4 | |
| **Rolling Window Stats** (`GET /api/v3/ticker`) | 4 per symbol | Capped at 200 for 50+ symbols |

### Trading Endpoints

| Endpoint | Weight | Notes |
|----------|--------|-------|
| **New Order** (`POST /api/v3/order`) | 1 | |
| **Test New Order** (`POST /api/v3/order/test`) | 1 or 20 | 20 with `computeCommissionRates` |
| **Cancel Order** (`DELETE /api/v3/order`) | 1 | |
| **Cancel All Open Orders** (`DELETE /api/v3/openOrders`) | 1 | |
| **Cancel & Replace** (`POST /api/v3/order/cancelReplace`) | 1 | |
| **Order Amend Keep Priority** (`PUT /api/v3/order/amend/keepPriority`) | 4 | |
| **New OCO** (`POST /api/v3/order/oco`) | 1 | Deprecated |
| **New Order List - OCO** (`POST /api/v3/orderList/oco`) | 1 | |
| **New Order List - OTO** (`POST /api/v3/orderList/oto`) | 1 | |
| **New Order List - OTOCO** (`POST /api/v3/orderList/otoco`) | 1 | |
| **New Order List - OPO** (`POST /api/v3/orderList/opo`) | 1 | |
| **New Order List - OPOCO** (`POST /api/v3/orderList/opoco`) | 1 | |
| **Cancel Order List** (`DELETE /api/v3/orderList`) | 1 | |
| **New SOR Order** (`POST /api/v3/sor/order`) | 1 | Smart Order Routing |
| **Test New SOR Order** (`POST /api/v3/sor/order/test`) | 1 or 20 | 20 with `computeCommissionRates` |

### Account Endpoints

| Endpoint | Weight | Notes |
|----------|--------|-------|
| **Account Information** (`GET /api/v3/account`) | 20 | |
| **Query Order** (`GET /api/v3/order`) | 4 | |
| **Current Open Orders** (`GET /api/v3/openOrders`) | | |
| - Single symbol | 6 | |
| - No symbol (all) | 80 | Heavy operation |
| **All Orders** (`GET /api/v3/allOrders`) | 20 | |
| **Query Order List** (`GET /api/v3/orderList`) | 4 | |
| **Query All Order Lists** (`GET /api/v3/allOrderList`) | 20 | |
| **Query Open Order Lists** (`GET /api/v3/openOrderList`) | 6 | |
| **Account Trade List** (`GET /api/v3/myTrades`) | | |
| - Without orderId | 20 | |
| - With orderId | 5 | |
| **Query Unfilled Order Count** (`GET /api/v3/rateLimit/order`) | 40 | |
| **Query Prevented Matches** (`GET /api/v3/myPreventedMatches`) | 2-20 | Varies by query type |
| **Query Allocations** (`GET /api/v3/myAllocations`) | 20 | |
| **Query Commission Rates** (`GET /api/v3/account/commission`) | 20 | |
| **Query Order Amendments** (`GET /api/v3/order/amendments`) | 4 | |
| **Query Relevant Filters** (`GET /api/v3/account/filters`) | 40 | |

### Futures-Specific Endpoints

| Endpoint | Weight | Notes |
|----------|--------|-------|
| **Auto Cancel All** (`POST /fapi/v1/countdownCancelAll`) | 10 | Countdown timer |
| **Cancel Order** (Futures WS API) | 1 | |

### Weight Patterns

**Light Operations (1-4 weight):**
- Single-symbol queries
- Basic order operations
- Lightweight market data (klines, price ticker)

**Medium Operations (5-25 weight):**
- Order book queries (small depth)
- Historical trades
- Account-specific queries

**Heavy Operations (40-250 weight):**
- Multi-symbol queries without symbol parameter
- Deep order book (1000-5000 levels)
- Bulk account operations

---

## Best Practices

### 1. Prefer WebSocket Streams Over REST

**Official Recommendation:**
> "It is strongly recommended to use websocket stream for getting data as much as possible, which can not only ensure the timeliness of the message, but also reduce the access restriction pressure caused by the request."

**Use WebSocket for:**
- Real-time price updates
- Order book updates
- Trade streams
- Account updates (via User Data Stream)

**Use REST for:**
- Initial data loading
- Historical data queries
- Order placement/cancellation
- Account management

### 2. Monitor Response Headers

**Always track:**
- `X-MBX-USED-WEIGHT-1M`: Know your current weight usage
- `X-MBX-ORDER-COUNT-*`: Track order rate consumption

**Example tracking logic:**
```rust
let used_weight = response.headers()
    .get("X-MBX-USED-WEIGHT-1M")
    .and_then(|v| v.to_str().ok())
    .and_then(|v| v.parse::<u32>().ok())
    .unwrap_or(0);

// Warn at 80% capacity (4800 of 6000)
if used_weight > 4800 {
    warn!("Rate limit at {}%", (used_weight * 100) / 6000);
}
```

### 3. Implement Intelligent Backoff

**On 429 Response:**
```rust
if response.status() == 429 {
    let retry_after = response.headers()
        .get("Retry-After")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(60); // Default 60s if header missing

    tokio::time::sleep(Duration::from_secs(retry_after)).await;
}
```

**Exponential Backoff:**
- Start with 1 second delay
- Double on each consecutive failure
- Cap at 60 seconds
- Reset after successful request

### 4. Batch Operations When Possible

**Bad (80+ weight):**
```rust
// Don't query all symbols separately
for symbol in symbols {
    get_ticker(symbol).await; // 2 weight each = 200 weight for 100 symbols
}
```

**Good (4 weight):**
```rust
// Query all at once
get_all_tickers().await; // 4 weight total
```

### 5. Use Symbol Parameters

**Bad (80 weight):**
```rust
GET /api/v3/openOrders // No symbol parameter
```

**Good (6 weight):**
```rust
GET /api/v3/openOrders?symbol=BTCUSDT
```

### 6. Optimize Order Book Depth

**Consider your needs:**
- 100 levels = 5 weight
- 500 levels = 25 weight (5x cost)
- 5000 levels = 250 weight (50x cost!)

**Most applications only need 100-500 levels**

### 7. Cache Aggressively

**Cache static/slow-changing data:**
- Exchange info (`/api/v3/exchangeInfo`)
- Symbol filters and rules
- Trading pairs list

**Update frequency:**
- Exchange info: Once per hour or on demand
- Price data: Use WebSocket streams
- Account data: Use User Data Stream

### 8. Implement Circuit Breaker

**Pattern:**
```rust
struct RateLimitCircuitBreaker {
    failures: AtomicU32,
    last_failure: Mutex<Instant>,
}

impl RateLimitCircuitBreaker {
    fn should_block(&self) -> bool {
        let failures = self.failures.load(Ordering::Relaxed);
        if failures >= 3 {
            // Block for 60 seconds after 3 consecutive 429s
            let elapsed = self.last_failure.lock().unwrap().elapsed();
            elapsed < Duration::from_secs(60)
        } else {
            false
        }
    }
}
```

### 9. Distribute Load Across IPs

**If possible:**
- Use multiple IPs for different services
- Separate market data fetching from trading
- Consider proxy rotation for market data (NOT for trading)

**Warning:**
- Don't use proxies for authenticated/trading requests
- Binance may detect and ban proxy patterns

### 10. Handle 418 Bans Gracefully

**On IP ban:**
1. **Stop all requests immediately**
2. **Wait for full Retry-After duration**
3. **Log ban event for analysis**
4. **Alert operations team**
5. **Review code for rate limit violations**

**Prevention is key:**
- Never ignore 429 responses
- Always implement backoff
- Monitor weight usage proactively

---

## Implementation Notes

### Tracking the Weight System

Binance's weight system is more complex than simple request counting. Here's how to implement proper tracking:

#### 1. Weight Budget Tracker

```rust
use std::sync::Arc;
use tokio::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug)]
struct WeightBudget {
    limit: u32,           // 6000 for spot/futures
    used: u32,
    window_start: Instant,
    window_duration: Duration,
}

impl WeightBudget {
    fn new(limit: u32, window_minutes: u64) -> Self {
        Self {
            limit,
            used: 0,
            window_start: Instant::now(),
            window_duration: Duration::from_secs(window_minutes * 60),
        }
    }

    fn reset_if_expired(&mut self) {
        if self.window_start.elapsed() >= self.window_duration {
            self.used = 0;
            self.window_start = Instant::now();
        }
    }

    fn can_afford(&mut self, weight: u32) -> bool {
        self.reset_if_expired();
        self.used + weight <= self.limit
    }

    fn consume(&mut self, weight: u32) {
        self.used += weight;
    }

    fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }

    fn update_from_header(&mut self, used_weight: u32) {
        // Use server's authoritative count
        self.used = used_weight;
    }
}
```

#### 2. Per-Endpoint Weight Mapping

```rust
use std::collections::HashMap;

fn endpoint_weights() -> HashMap<&'static str, u32> {
    let mut weights = HashMap::new();

    // Market Data
    weights.insert("GET /api/v3/depth", 5);      // Default, varies by limit
    weights.insert("GET /api/v3/trades", 25);
    weights.insert("GET /api/v3/klines", 2);
    weights.insert("GET /api/v3/ticker/24hr", 2); // Single symbol
    weights.insert("GET /api/v3/ticker/price", 2);

    // Trading
    weights.insert("POST /api/v3/order", 1);
    weights.insert("DELETE /api/v3/order", 1);

    // Account
    weights.insert("GET /api/v3/account", 20);
    weights.insert("GET /api/v3/openOrders", 6);  // With symbol
    weights.insert("GET /api/v3/myTrades", 20);

    weights
}

fn calculate_weight(endpoint: &str, params: &HashMap<String, String>) -> u32 {
    match endpoint {
        "GET /api/v3/depth" => {
            let limit = params.get("limit")
                .and_then(|l| l.parse::<u32>().ok())
                .unwrap_or(100);

            match limit {
                0..=100 => 5,
                101..=500 => 25,
                501..=1000 => 50,
                _ => 250,
            }
        },
        "GET /api/v3/ticker/24hr" => {
            if params.contains_key("symbol") {
                2
            } else if let Some(symbols) = params.get("symbols") {
                let count = symbols.split(',').count();
                match count {
                    1..=20 => 2,
                    21..=100 => 40,
                    _ => 80,
                }
            } else {
                80  // All symbols
            }
        },
        "GET /api/v3/openOrders" => {
            if params.contains_key("symbol") {
                6
            } else {
                80  // All symbols
            }
        },
        "GET /api/v3/myTrades" => {
            if params.contains_key("orderId") {
                5
            } else {
                20
            }
        },
        _ => {
            // Fallback to static mapping
            endpoint_weights().get(endpoint).copied().unwrap_or(1)
        }
    }
}
```

#### 3. Rate Limiter with Weight Awareness

```rust
use tokio::sync::Semaphore;

pub struct BinanceRateLimiter {
    weight_budget: Arc<Mutex<WeightBudget>>,
    order_limiter_10s: Arc<Semaphore>,
    order_limiter_1d: Arc<Mutex<OrderCounter>>,
}

impl BinanceRateLimiter {
    pub fn new() -> Self {
        Self {
            weight_budget: Arc::new(Mutex::new(
                WeightBudget::new(6000, 1)
            )),
            order_limiter_10s: Arc::new(Semaphore::new(50)),
            order_limiter_1d: Arc::new(Mutex::new(
                OrderCounter::new(160_000, Duration::from_secs(86400))
            )),
        }
    }

    pub async fn acquire_weight(&self, weight: u32) -> Result<(), String> {
        let mut budget = self.weight_budget.lock().await;

        // Wait if we don't have budget
        while !budget.can_afford(weight) {
            drop(budget);  // Release lock while waiting
            tokio::time::sleep(Duration::from_millis(100)).await;
            budget = self.weight_budget.lock().await;
            budget.reset_if_expired();
        }

        budget.consume(weight);
        Ok(())
    }

    pub async fn update_from_response(&self, headers: &HeaderMap) {
        if let Some(used_weight) = headers.get("X-MBX-USED-WEIGHT-1M") {
            if let Ok(weight_str) = used_weight.to_str() {
                if let Ok(weight) = weight_str.parse::<u32>() {
                    let mut budget = self.weight_budget.lock().await;
                    budget.update_from_header(weight);
                }
            }
        }
    }
}
```

#### 4. Integration with HTTP Client

```rust
pub struct BinanceClient {
    http_client: reqwest::Client,
    rate_limiter: BinanceRateLimiter,
}

impl BinanceClient {
    pub async fn request(
        &self,
        endpoint: &str,
        params: HashMap<String, String>,
    ) -> Result<Response, ExchangeError> {
        // Calculate weight before request
        let weight = calculate_weight(endpoint, &params);

        // Acquire weight budget
        self.rate_limiter.acquire_weight(weight).await?;

        // Make request
        let response = self.http_client
            .get(&format!("{}{}", BASE_URL, endpoint))
            .query(&params)
            .send()
            .await?;

        // Update from server response
        self.rate_limiter.update_from_response(response.headers()).await;

        // Handle rate limit errors
        match response.status() {
            StatusCode::TOO_MANY_REQUESTS => {
                let retry_after = response.headers()
                    .get("Retry-After")
                    .and_then(|v| v.to_str().ok())
                    .and_then(|v| v.parse::<u64>().ok())
                    .unwrap_or(60);

                Err(ExchangeError::RateLimitExceeded {
                    retry_after
                })
            },
            StatusCode::IM_A_TEAPOT => {
                Err(ExchangeError::IpBanned {
                    retry_after: retry_after_from_headers(response.headers())
                })
            },
            _ => Ok(response),
        }
    }
}
```

### Differences: Spot vs Futures

Both Spot and Futures APIs use the **same rate limiting system**:

- Same REQUEST_WEIGHT limit: **6,000/minute**
- Same order limits: **50/10s, 160,000/24h**
- Same response headers
- Same error codes

**Key difference:**
- Futures WebSocket streams allow **10 messages/second** vs Spot's **5 messages/second**

### Special Considerations

#### 1. WebSocket API vs WebSocket Streams

**WebSocket API** (websocket-api):
- Uses same weight system as REST (6,000/minute)
- Connection costs 2 weight
- Includes weight tracking in responses
- Subject to order limits

**WebSocket Streams** (stream.binance.com):
- No weight system
- Limited by message rate (5-10/second)
- Limited by streams per connection (1,024)
- 24-hour connection duration

#### 2. Order Count vs Weight

These are **independent** limits:
- You can hit order limit without hitting weight limit
- You can hit weight limit without hitting order limit
- Both are enforced simultaneously

#### 3. IP Sharing

**Critical for production:**
- All services on same IP share limits
- Multiple trading bots on one IP = shared 6,000 weight pool
- Consider separate IPs for:
  - Market data services
  - Trading engines
  - Analytics/backtesting

#### 4. Retry-After Header

**Always present on:**
- 429 responses (rate limit exceeded)
- 418 responses (IP banned)

**Format:** Integer seconds to wait

**Must be respected** to avoid escalating bans

---

## Sources

- [Binance Spot API Limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits)
- [Binance WebSocket API Rate Limits](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/rate-limits)
- [Binance Futures General Info](https://developers.binance.com/docs/derivatives/usds-margined-futures/general-info)
- [Binance Market Data Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/market-data-endpoints)
- [Binance Account Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/account-endpoints)
- [Binance Trading Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Binance WebSocket Streams](https://developers.binance.com/docs/binance-spot-api-docs/web-socket-streams)
- [Rate Limits on Binance Futures](https://www.binance.com/en/support/faq/detail/281596e222414cdd9051664ea621cdc3)
- [Binance API GitHub Documentation](https://github.com/binance/binance-spot-api-docs/blob/master/rest-api.md)
- [What Are Binance WebSocket Limits?](https://academy.binance.com/en/articles/what-are-binance-websocket-limits)
