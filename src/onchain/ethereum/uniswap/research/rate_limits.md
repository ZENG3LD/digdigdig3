# Uniswap Rate Limits and Usage Restrictions

## Overview

Uniswap APIs have different rate limits depending on the service:
1. **Trading API** - 12 requests/second (default)
2. **The Graph Subgraph** - Variable limits based on billing
3. **JSON-RPC Nodes** - Provider-dependent limits
4. **Smart Contracts** - Network gas limits only

---

## 1. Trading API Rate Limits

### Default Rate Limit

**12 requests per second per API key**

```
Rate Limit: 12 req/s
Window: 1 second (sliding)
Scope: Per API key
```

### Headers

**Rate Limit Information:**
```http
X-RateLimit-Limit: 12
X-RateLimit-Remaining: 11
X-RateLimit-Reset: 1735680001
```

### Exceeding Rate Limit

**429 Too Many Requests:**
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Rate limit exceeded. Try again in 1 second.",
  "retryAfter": 1
}
```

**Response Headers:**
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 1
X-RateLimit-Limit: 12
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1735680001
```

### Increasing Rate Limits

**Contact Support:**
> "If a higher rate is needed, please reach out to your Uniswap Labs contact."

**Process:**
1. Contact Uniswap Labs support
2. Explain use case and required throughput
3. Negotiate custom rate limit
4. Agreement may include volume-based or per-swap fees

### Beta Environment

Separate rate limits for testing:
```
Production API Key: 12 req/s
Beta API Key: 12 req/s (separate quota)
```

Request beta access from your account manager.

---

## 2. Usage Restrictions

### Prohibited Activities

**From Uniswap Terms:**

> "Under no circumstances do they allow you to reoffer the API substantially unchanged (e.g. pass through our API to your end customers)."

**Forbidden:**
- ❌ Reselling API access
- ❌ White-labeling API responses
- ❌ Acting as a proxy service
- ❌ Redistributing raw API data

**Allowed:**
- ✅ Building applications on top of the API
- ✅ Adding your own logic/features
- ✅ Aggregating with other data sources
- ✅ Internal company use

### Commercial Licensing

**For High-Volume or Commercial Use:**

Uniswap offers contracts with:
- Volume-based fees
- Per-swap fees
- Custom rate limits
- SLA guarantees
- Priority support

Contact Uniswap Labs for enterprise agreements.

### Fee Changes (2026 Update)

**UNIfication Governance (Jan 1, 2026):**

> "Labs' fees on the interface, wallet, and API were set to zero."

**Current State:**
- Trading API fees: **$0** (as of Jan 2026)
- No per-swap charges (governance decision)
- API key still required for authentication

This may change with future governance votes.

---

## 3. The Graph Subgraph Rate Limits

### Billing-Based Limits

The Graph uses a **query-per-second (QPS)** model with billing.

**Rate Limit Factors:**
1. API key tier/plan
2. Monthly spending limit
3. Query complexity
4. Network congestion

### Spending Limits

**Set Monthly Budget:**
```
Default: No limit (pay-as-you-go)
Optional: Set maximum USD/month
```

Configure in [The Graph Studio](https://thegraph.com/studio/apikeys/):
```
API Key Settings:
  - Monthly Spending Limit: $100 USD
  - Auto-disable if exceeded: Yes
```

### Query Complexity Limits

**GraphQL Complexity Scoring:**
- Simple field: 1 point
- Nested entity: +5 points
- Array field: +10 points
- Nested array: +50 points

**Max Complexity Per Query:**
```
Standard: ~1000 points
Enterprise: Negotiable
```

**Example - High Complexity:**
```graphql
{
  pools(first: 1000) {  # +1000 points (large array)
    swaps(first: 100) {  # +100 points per pool = 100k points
      transaction {
        mints { ... }   # Additional nesting
      }
    }
  }
}
# Total: ~100k+ points (likely rejected)
```

### Pagination Limits

**Maximum Results Per Query:**
```
first: 1000  # Max
skip: 5000   # Max
```

**Recommended Approach:**
```graphql
# Query 1
{ pools(first: 1000, skip: 0) { id } }

# Query 2
{ pools(first: 1000, skip: 1000) { id } }

# Continue until results < 1000
```

### Rate Limit Responses

**429 Too Many Requests:**
```json
{
  "errors": [
    {
      "message": "Rate limit exceeded"
    }
  ]
}
```

**Budget Exceeded:**
```json
{
  "errors": [
    {
      "message": "Monthly spending limit exceeded. Increase your limit or wait until next billing cycle."
    }
  ]
}
```

---

## 4. JSON-RPC Node Rate Limits

### Provider-Specific Limits

Rate limits vary by Ethereum node provider:

#### Infura

**Free Tier:**
```
Requests: 100,000 per day
Rate: ~1 req/s average
Burst: Up to 10 req/s
```

**Paid Plans:**
- Developer: 1M requests/day
- Team: 10M requests/day
- Growth: 100M requests/day

#### Alchemy

**Free Tier:**
```
Compute Units: 300M per month
Requests: ~50M per month (varies by method)
Rate: No hard limit, CU-based throttling
```

**Paid Plans:**
- Growth: 1.5B CU/month
- Scale: Custom

#### Chainstack

**Elastic Nodes:**
```
Requests: Unlimited (pay per request)
Rate: No hard limit
Pricing: $0.0001 per request
```

**Dedicated Nodes:**
```
Requests: Unlimited
Rate: Hardware-limited (very high)
Pricing: Fixed monthly fee
```

#### Public Nodes (Cloudflare, etc.)

**Strict Limits:**
```
Rate: 1-5 req/s
Reliability: Low (no SLA)
Recommended: Development only
```

### WebSocket Limits

**Subscription Limits:**
```
Infura: 100 concurrent subscriptions
Alchemy: 500 concurrent subscriptions
Chainstack: Unlimited (dedicated nodes)
```

**Reconnection Handling:**
```rust
use tokio_tungstenite::connect_async;
use futures_util::StreamExt;

async fn maintain_websocket_connection() {
    loop {
        match connect_async(ws_url).await {
            Ok((ws_stream, _)) => {
                // Handle messages
                while let Some(msg) = ws_stream.next().await {
                    // Process message
                }
                // Connection closed, reconnect
            }
            Err(e) => {
                eprintln!("WebSocket error: {}", e);
                tokio::time::sleep(Duration::from_secs(5)).await;
            }
        }
    }
}
```

---

## 5. Smart Contract Gas Limits

### Block Gas Limit

**Ethereum Mainnet:**
```
Block Gas Limit: ~30,000,000 gas
Block Time: ~12 seconds
Max Throughput: ~2.5M gas/second
```

### Typical Transaction Gas Costs

| Operation | Gas Cost | USD (50 gwei, $2000 ETH) |
|-----------|----------|--------------------------|
| Simple swap (V3) | ~120,000 | ~$12 |
| Multi-hop swap | ~200,000 | ~$20 |
| Add liquidity | ~150,000 | ~$15 |
| Remove liquidity | ~100,000 | ~$10 |
| Collect fees | ~80,000 | ~$8 |

### No API Rate Limits

Smart contract calls via RPC are limited by:
1. **Gas limits** (network capacity)
2. **RPC provider limits** (see section 4)
3. **Nonce ordering** (sequential per address)

**No Uniswap-imposed limits** on contract interactions.

---

## 6. Best Practices for Rate Limit Management

### 6.1 Request Queuing

**Implement Token Bucket:**
```rust
use tokio::sync::Semaphore;
use std::sync::Arc;

struct RateLimiter {
    semaphore: Arc<Semaphore>,
    refill_task: tokio::task::JoinHandle<()>,
}

impl RateLimiter {
    fn new(rate: usize) -> Self {
        let semaphore = Arc::new(Semaphore::new(rate));
        let sem_clone = semaphore.clone();

        let refill_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                // Add permits back (up to max)
                sem_clone.add_permits(rate - sem_clone.available_permits());
            }
        });

        RateLimiter { semaphore, refill_task }
    }

    async fn acquire(&self) {
        self.semaphore.acquire().await.unwrap().forget();
    }
}

// Usage
let limiter = RateLimiter::new(12);  // 12 req/s
limiter.acquire().await;
make_api_request().await?;
```

### 6.2 Exponential Backoff

**Handle 429 Responses:**
```rust
async fn request_with_retry<T>(
    url: &str,
    max_retries: u32,
) -> Result<T> {
    let mut attempt = 0;
    loop {
        match make_request(url).await {
            Ok(response) if response.status() == 429 => {
                if attempt >= max_retries {
                    return Err(Error::RateLimitExceeded);
                }

                let wait_time = 2u64.pow(attempt);
                tokio::time::sleep(Duration::from_secs(wait_time)).await;
                attempt += 1;
            }
            Ok(response) => return parse_response(response).await,
            Err(e) => return Err(e),
        }
    }
}
```

### 6.3 Caching

**Cache Responses:**
```rust
use moka::future::Cache;
use std::time::Duration;

let cache = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(Duration::from_secs(60))
    .build();

async fn get_token_info(address: Address) -> Result<TokenInfo> {
    if let Some(info) = cache.get(&address) {
        return Ok(info);
    }

    let info = fetch_token_info_from_api(address).await?;
    cache.insert(address, info.clone()).await;
    Ok(info)
}
```

### 6.4 Batch Requests

**Use Multicall for Contract Reads:**
```rust
// Instead of:
let balance1 = token1.balanceOf(addr).call().await?;
let balance2 = token2.balanceOf(addr).call().await?;
let balance3 = token3.balanceOf(addr).call().await?;

// Use multicall:
let results = Multicall::new()
    .add_call(token1.balanceOf(addr))
    .add_call(token2.balanceOf(addr))
    .add_call(token3.balanceOf(addr))
    .call_array()
    .await?;
```

**Use GraphQL Queries Efficiently:**
```graphql
# Instead of 3 separate queries:
# { pool1: pool(id: "0x...") { ... } }
# { pool2: pool(id: "0x...") { ... } }
# { pool3: pool(id: "0x...") { ... } }

# Single query with aliases:
{
  pool1: pool(id: "0xabc...") { ...poolFields }
  pool2: pool(id: "0xdef...") { ...poolFields }
  pool3: pool(id: "0x123...") { ...poolFields }
}
```

### 6.5 Monitor Usage

**Track Request Counts:**
```rust
use std::sync::atomic::{AtomicU64, Ordering};

struct ApiMetrics {
    requests_made: AtomicU64,
    rate_limits_hit: AtomicU64,
    errors: AtomicU64,
}

impl ApiMetrics {
    fn record_request(&self) {
        self.requests_made.fetch_add(1, Ordering::Relaxed);
    }

    fn record_rate_limit(&self) {
        self.rate_limits_hit.fetch_add(1, Ordering::Relaxed);
    }

    fn report(&self) {
        println!("Requests: {}", self.requests_made.load(Ordering::Relaxed));
        println!("Rate limits: {}", self.rate_limits_hit.load(Ordering::Relaxed));
    }
}
```

---

## 7. Rate Limit Summary Table

| API/Service | Rate Limit | Scope | Upgrade Available |
|-------------|------------|-------|-------------------|
| Trading API | 12 req/s | Per API key | Yes (contact support) |
| Trading API (Beta) | 12 req/s | Per API key | Separate quota |
| The Graph Subgraph | Variable | Per API key + budget | Yes (increase budget) |
| Infura Free | ~1 req/s avg | Per API key | Yes (paid plans) |
| Alchemy Free | CU-based | Per API key | Yes (paid plans) |
| Chainstack | Unlimited* | Per node | - |
| Smart Contracts | Gas-limited | Network-wide | No (blockchain limit) |

*Chainstack elastic nodes have per-request pricing; dedicated nodes are hardware-limited.

---

## 8. Error Handling Examples

### 8.1 Rust Implementation

```rust
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Rate limit exceeded. Retry after {0}s")]
    RateLimitExceeded(u64),

    #[error("Monthly budget exceeded")]
    BudgetExceeded,

    #[error("HTTP error: {0}")]
    HttpError(StatusCode),
}

async fn handle_api_error(response: reqwest::Response) -> Result<(), ApiError> {
    match response.status() {
        StatusCode::TOO_MANY_REQUESTS => {
            let retry_after = response
                .headers()
                .get("Retry-After")
                .and_then(|h| h.to_str().ok())
                .and_then(|s| s.parse().ok())
                .unwrap_or(1);

            Err(ApiError::RateLimitExceeded(retry_after))
        }
        StatusCode::PAYMENT_REQUIRED => {
            Err(ApiError::BudgetExceeded)
        }
        status if !status.is_success() => {
            Err(ApiError::HttpError(status))
        }
        _ => Ok(()),
    }
}
```

### 8.2 Request with Retry

```rust
async fn make_api_call_with_retry<T>(
    client: &reqwest::Client,
    url: &str,
    max_retries: u32,
) -> Result<T>
where
    T: for<'de> serde::Deserialize<'de>,
{
    let mut attempts = 0;

    loop {
        let response = client.get(url).send().await?;

        match response.status() {
            StatusCode::OK => {
                return response.json::<T>().await.map_err(Into::into);
            }
            StatusCode::TOO_MANY_REQUESTS if attempts < max_retries => {
                let retry_after = response
                    .headers()
                    .get("Retry-After")
                    .and_then(|h| h.to_str().ok())
                    .and_then(|s| s.parse::<u64>().ok())
                    .unwrap_or(2u64.pow(attempts));

                println!("Rate limited. Waiting {}s...", retry_after);
                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                attempts += 1;
            }
            status => {
                return Err(ApiError::HttpError(status).into());
            }
        }
    }
}
```

---

## 9. Optimization Tips

### 9.1 Prefer WebSockets for Real-Time Data

**Instead of polling:**
```rust
// ❌ Inefficient - polls every second
loop {
    let price = get_pool_price(pool_address).await?;
    tokio::time::sleep(Duration::from_secs(1)).await;
}
```

**Use WebSocket subscriptions:**
```rust
// ✅ Efficient - push-based updates
let mut stream = subscribe_to_swaps(pool_address).await?;
while let Some(swap_event) = stream.next().await {
    process_swap(swap_event);
}
```

### 9.2 Aggregate Queries

**Subgraph optimization:**
```graphql
# ❌ Bad - makes 3 separate requests
query { pool1: pool(id: "0x...") { liquidity } }
query { pool2: pool(id: "0x...") { liquidity } }
query { pool3: pool(id: "0x...") { liquidity } }

# ✅ Good - single request
query {
  pools(where: { id_in: ["0x...", "0x...", "0x..."] }) {
    id
    liquidity
  }
}
```

### 9.3 Use Static Data

**Load once at startup:**
```rust
// Token metadata rarely changes
lazy_static! {
    static ref TOKEN_METADATA: HashMap<Address, TokenInfo> = {
        load_token_metadata_from_file("tokens.json")
    };
}
```

---

## Summary

**Key Takeaways:**

1. **Trading API**: 12 req/s default, contact support for more
2. **The Graph**: Budget-based, set monthly limits
3. **RPC Nodes**: Provider-specific (use paid tier for production)
4. **Smart Contracts**: No API limits, only gas costs
5. **Implement**: Rate limiting, caching, retries, batching
6. **Monitor**: Track usage to avoid surprises
7. **Optimize**: WebSockets > polling, batch > individual requests

**Production Checklist:**
- [ ] Implement request rate limiter
- [ ] Add exponential backoff for 429 errors
- [ ] Cache static data (token metadata, pool addresses)
- [ ] Use WebSocket subscriptions for real-time data
- [ ] Batch contract calls with Multicall
- [ ] Monitor API usage metrics
- [ ] Set budget limits on The Graph
- [ ] Use paid RPC provider with SLA
