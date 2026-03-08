# HyperLiquid Rate Limits

HyperLiquid uses a sophisticated multi-tier rate limiting system combining IP-based, address-based, and volume-based limits.

---

## IP-Based Rate Limits (REST API)

### Weight System
Aggregate limit: **1200 weight per minute per IP address**

### Exchange Endpoint Weights

**Formula**: `1 + floor(batch_length / 40)`

| Request Type | Weight |
|--------------|--------|
| Single order/cancel | 1 |
| Batch of 1-39 orders | 1 |
| Batch of 40-79 orders | 2 |
| Batch of 80-119 orders | 3 |
| Modify order | 1 |
| Update leverage | 1 |
| Update isolated margin | 1 |
| USD transfer | 1 |
| Withdraw | 1 |

**Example**:
- 10 orders in batch = weight 1
- 79 orders in batch = weight 2
- 120 orders in batch = weight 4

---

### Info Endpoint Weights

#### Weight 2 Endpoints
- `l2Book`
- `allMids`
- `clearinghouseState`
- `orderStatus`
- `spotClearinghouseState`
- `exchangeStatus`

#### Weight 20 Endpoints (Most Common)
- `meta`
- `metaAndAssetCtxs`
- `spotMeta`
- `spotMetaAndAssetCtxs`
- `openOrders`
- `portfolio`
- `userFees`
- `userRole`
- `subAccounts`
- `referral`
- And most other documented info requests

#### Weight 60 Endpoints
- `userRole` (high cost due to blockchain lookup)

#### Variable Weight Endpoints
**Base weight + additional weight per response items**

Add weight per 20 items returned:
- `recentTrades`
- `historicalOrders`
- `userFills`
- `userFillsByTime`
- `fundingHistory`
- `userFunding`
- `nonUserFundingUpdates`
- `twapHistory`
- `userTwapSliceFills`
- `userTwapSliceFillsByTime`
- `delegatorHistory`
- `delegatorRewards`
- `validatorStats`

Add weight per 60 items returned:
- `candleSnapshot`

**Example**:
- `userFills` with 100 results = base weight + 5 additional = 25 total
- `candleSnapshot` with 300 candles = base weight + 5 additional = 25 total

#### Explorer API
- **Base weight**: 40 per request
- **blockList**: Additional 1 weight per block

---

### EVM JSON-RPC Limits
- **Endpoint**: `rpc.hyperliquid.xyz/evm`
- **Limit**: 100 requests per minute

---

## WebSocket Rate Limits

### Connection Limits
- **Maximum connections**: 100 per IP
- **Maximum subscriptions**: 1000 total across all connections
- **Maximum unique users**: 10 for user-specific subscriptions
- **Message rate**: 2000 messages per minute across all connections
- **Inflight post messages**: 100 simultaneous

### WebSocket Subscriptions Count
Each subscription counts separately:
```javascript
// Counts as 3 subscriptions
ws.send({ method: "subscribe", subscription: { type: "trades", coin: "BTC" } });
ws.send({ method: "subscribe", subscription: { type: "trades", coin: "ETH" } });
ws.send({ method: "subscribe", subscription: { type: "l2Book", coin: "BTC" } });
```

### User-Specific Subscription Limits
Maximum 10 unique user addresses across all subscriptions:
```javascript
// These count toward the 10 user limit
{ type: "openOrders", user: "0xUser1..." }
{ type: "userFills", user: "0xUser1..." }    // Same user, still counts as 1
{ type: "userFills", user: "0xUser2..." }    // Different user, counts as 2 total
```

---

## Address-Based Rate Limits

### Request Allowance Formula
**1 request per 1 USDC traded cumulatively since address inception**

### Initial Buffer
All addresses receive: **10,000 requests** as starting buffer

### Rate Limited Behavior
When rate limited: **1 request every 10 seconds**

### Cancel Request Exemption
Cancel requests have elevated limit:
```
cancel_limit = min(normal_limit + 100,000, normal_limit * 2)
```

**Example**:
- Normal limit: 50,000 requests
- Cancel limit: min(150,000, 100,000) = 100,000 requests

This allows managing orders even when hitting rate limits.

---

## Open Order Limits

### Base Limit
**1000 open orders** per address

### Volume-Based Increases
**+1 order per 5M USDC** trading volume

**Formula**:
```
max_open_orders = min(1000 + floor(cumulative_volume / 5_000_000), 5000)
```

### Maximum Cap
**5000 open orders** (hard limit)

### Example Progression

| Cumulative Volume | Max Open Orders |
|-------------------|-----------------|
| $0 - $4.9M | 1000 |
| $5M | 1001 |
| $50M | 1010 |
| $500M | 1100 |
| $5B | 2000 |
| $20B+ | 5000 (cap) |

### Restriction with 1000+ Open Orders
If address has 1000+ open orders:
- New orders rejected if:
  - Reduce-only orders
  - Trigger orders (stop loss, take profit)

This prevents excessive order book pollution.

---

## Congestion-Based Throttling

### Block Space Allocation
During high congestion:
```
address_limit = 2x * maker_share_percentage * block_space
```

**Maker share**: Percentage of maker volume contributed to exchange

**Example**:
- Address contributes 1% of maker volume
- During congestion: Limited to 2% of block space
- Fair allocation based on contribution

---

## Batching Impact

### Single Batch Request
**IP Rate Limit**: Counts based on batch size formula
```
weight = 1 + floor(batch_length / 40)
```

**Address Rate Limit**: Each item counts separately
```
address_requests_used = number_of_orders_or_cancels
```

**Example - Batch of 50 Orders**:
- IP weight consumed: 2
- Address requests consumed: 50

### Strategic Batching
- Batch to reduce IP weight consumption
- Monitor address-based request allowance separately
- Ideal batch size: 39 items (weight = 1, best efficiency)

---

## Querying Rate Limit Status

### Endpoint
**Method**: POST
**URL**: `/info`

**Request**:
```json
{
  "type": "userRateLimit",
  "user": "0x1234567890abcdef1234567890abcdef12345678"
}
```

**Response**:
```json
{
  "cumVlm": "12345678.90",
  "nRequestsUsed": 5432,
  "nRequestsCap": 12345678,
  "nRequestsSurplus": 10000
}
```

### Response Fields
- `cumVlm`: Cumulative trading volume (USDC)
- `nRequestsUsed`: Requests consumed
- `nRequestsCap`: Current request allowance (based on volume)
- `nRequestsSurplus`: Initial 10K buffer remaining

### Calculating Remaining Requests
```
remaining = nRequestsCap + nRequestsSurplus - nRequestsUsed
```

---

## Time-Based Restrictions

### Nonce Time Constraints
Nonces must be within: **(T - 2 days, T + 1 day)**
- T = current block timestamp (milliseconds)

### Expires After Field
Optional field for automatic request rejection:
```json
{
  "action": {
    "type": "order",
    "orders": [...],
    "expiresAfter": 1704067200000
  }
}
```

**Important**: If rejected due to expiration, request consumes **5x rate limit weight**

**Recommendation**: Avoid using unless necessary

---

## Public Infrastructure Limits

### Use Case Restrictions
Public infrastructure suitable for:
- Prototyping
- Educational projects
- Low-frequency applications

**Maximum recommended**: 100 requests per minute

### Production Recommendations
For production/high-frequency trading:
- Consider running own infrastructure
- Use dedicated API wallets per process
- Implement local rate limiting

---

## Rate Limit Best Practices

### 1. Request Budgeting
```rust
// Track weight consumption
let mut weight_used = 0;
let weight_limit = 1200;  // Per minute

// Before request
if weight_used + request_weight <= weight_limit {
    make_request();
    weight_used += request_weight;
} else {
    wait_until_next_minute();
}
```

### 2. Optimize Batch Sizes
```
Batch Size | Weight | Efficiency
-----------|--------|------------
1-39       | 1      | Best
40-79      | 2      | Good
80-119     | 3      | OK
120+       | 4+     | Consider splitting
```

**Optimal**: Keep batches ≤ 39 items for weight = 1

### 3. Separate Order Types
**Validator prioritizes differently**:
- IOC/GTC orders: Standard priority
- ALO (post-only): Lower priority

**Strategy**: Batch separately for better execution
```json
// Batch 1: IOC/GTC orders
{ "type": "order", "orders": [ioc_orders...] }

// Batch 2: ALO orders
{ "type": "order", "orders": [alo_orders...] }
```

### 4. Cancel Allowance Usage
Cancels have 2x limit - use for cleanup:
```rust
// Regular limit exhausted
if requests_remaining == 0 && cancel_allowance > 0 {
    // Can still cancel orders
    cancel_all_orders();
}
```

### 5. WebSocket for Market Data
**Avoid polling**: Use WebSocket subscriptions
```
❌ Poll l2Book every second (60 requests/min, weight 120)
✓ Subscribe to l2Book WS (0 REST requests)
```

### 6. Info Request Caching
Cache static/slow-changing data:
- `meta` (asset metadata)
- `spotMeta` (spot pairs)
- `userFees` (fee schedule)

**Update frequency**: Every 5-15 minutes

### 7. Multi-Process Architecture
**Problem**: Multiple processes share address limit

**Solution**: Separate API wallets
```
Process A: Trading Bot → API Wallet A → Master Account
Process B: Market Maker → API Wallet B → Master Account
Process C: Liquidation → API Wallet C → Master Account
```

Each API wallet has independent rate limits.

### 8. Monitor Rate Limit Status
Poll `userRateLimit` periodically:
```rust
// Check every 5 minutes
if should_check_rate_limit() {
    let status = get_user_rate_limit(address).await?;

    if status.remaining_requests() < 1000 {
        warn!("Low request allowance: {}", status.remaining_requests());
        reduce_trading_frequency();
    }
}
```

---

## Error Handling

### Rate Limit Errors
**HTTP 429**: Too Many Requests
```json
{
  "status": "error",
  "error": "Rate limit exceeded"
}
```

**Response Strategy**:
1. Implement exponential backoff
2. Reduce request frequency
3. Switch to WebSocket for data
4. Consider additional API wallets

### Retry Logic
```rust
async fn with_rate_limit_retry<T, F>(
    mut operation: F,
    max_retries: u32
) -> Result<T>
where
    F: FnMut() -> Future<Output = Result<T>>
{
    let mut retries = 0;

    loop {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if is_rate_limit_error(&e) && retries < max_retries => {
                retries += 1;
                let delay = Duration::from_secs(2_u64.pow(retries));
                tokio::time::sleep(delay).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

---

## Rate Limit Tracking Implementation

### Local Rate Limiter
```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    weight_limit: u32,
    window: Duration,
    requests: VecDeque<(Instant, u32)>,
}

impl RateLimiter {
    pub fn new(weight_limit: u32, window: Duration) -> Self {
        Self {
            weight_limit,
            window,
            requests: VecDeque::new(),
        }
    }

    pub fn check_and_add(&mut self, weight: u32) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;

        // Remove old requests
        while let Some((time, _)) = self.requests.front() {
            if *time < cutoff {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Calculate current weight
        let current_weight: u32 = self.requests
            .iter()
            .map(|(_, w)| w)
            .sum();

        if current_weight + weight <= self.weight_limit {
            self.requests.push_back((now, weight));
            true
        } else {
            false
        }
    }

    pub fn wait_time(&self) -> Option<Duration> {
        let now = Instant::now();
        let cutoff = now - self.window;

        if let Some((oldest, _)) = self.requests.front() {
            if *oldest < cutoff {
                None
            } else {
                Some(*oldest + self.window - now)
            }
        } else {
            None
        }
    }
}

// Usage
let mut limiter = RateLimiter::new(1200, Duration::from_secs(60));

if limiter.check_and_add(request_weight) {
    make_request().await?;
} else if let Some(wait) = limiter.wait_time() {
    tokio::time::sleep(wait).await;
    limiter.check_and_add(request_weight);
    make_request().await?;
}
```

---

## Summary Table

| Limit Type | Value | Notes |
|------------|-------|-------|
| **IP Weight** | 1200/min | Per IP address |
| **Single order** | 1 weight | Per order/cancel |
| **Batch orders** | 1 + ⌊len/40⌋ | Efficient at <40 |
| **WebSocket connections** | 100 | Per IP |
| **WebSocket subscriptions** | 1000 | Total across connections |
| **WS messages** | 2000/min | Across all connections |
| **Address requests** | 1 per $1 traded | +10K initial |
| **Open orders** | 1000 + ⌊volume/5M⌋ | Max 5000 |
| **Cancel allowance** | 2x normal | Up to +100K |
| **Rate limited delay** | 10 seconds | Per request |
| **EVM RPC** | 100/min | Separate limit |

---

## Integration Checklist

- [ ] Implement local weight tracking (1200/min)
- [ ] Calculate request weights correctly
- [ ] Use WebSocket for market data (reduce REST calls)
- [ ] Batch orders efficiently (≤39 items optimal)
- [ ] Cache static data (meta, spotMeta)
- [ ] Monitor address-based rate limit status
- [ ] Separate API wallets for multi-process setups
- [ ] Implement exponential backoff for rate limit errors
- [ ] Track open order count vs limit
- [ ] Use cancel allowance for cleanup operations
- [ ] Test rate limits on testnet first
- [ ] Set up monitoring/alerts for limit approaching
