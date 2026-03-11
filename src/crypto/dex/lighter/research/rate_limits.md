# Lighter Exchange Rate Limits

## Overview

Lighter enforces rate limits on both REST API and WebSocket connections to ensure fair resource allocation and prevent abuse. Limits are applied **per IP address** and **per L1 wallet address**.

---

## Account Tiers

### Standard Account

**Access**: Free
**Trading Fees**: 0% maker, 0% taker
**REST API Limit**: **60 weighted requests per minute**
**Equivalent**: ~1 request per second

### Premium Account

**Access**: Paid tier
**Trading Fees**: 0.002% maker (0.2 bps), 0.02% taker (2 bps)
**REST API Limit**: **24,000 weighted requests per minute**
**Equivalent**: ~400 requests per second

### Switching Tiers

**Endpoint**: `POST /changeAccountTier`

**Note**: Can switch between Standard and Premium at any time.

---

## REST API Rate Limits

### Mainnet API (mainnet.zklighter.elliot.ai)

**Time Window**: 60 seconds (rolling)

**Limits**:
- Standard: 60 weighted requests per minute
- Premium: 24,000 weighted requests per minute

**Enforcement**: Per user (IP + L1 address)

---

### Endpoint Weights

Different endpoints consume different amounts of rate limit quota.

#### Low Weight (6 per request)

- `POST /api/v1/sendTx`
- `POST /api/v1/sendTxBatch`
- `GET /api/v1/nextNonce`

**Usage**: Transaction submission and nonce queries

---

#### Medium Weight (50 per request)

- `GET /api/v1/publicPools`
- `GET /api/v1/txFromL1TxHash`

**Usage**: Public pool data, L1 transaction lookups

---

#### High Weight (100 per request)

- `GET /api/v1/accountInactiveOrders`
- `GET /api/v1/deposit/latest`

**Usage**: Historical order data, deposit queries

---

#### Very High Weight (150 per request)

- `GET /api/v1/apikeys`

**Usage**: API key management

---

#### Ultra High Weight (500 per request)

- `GET /api/v1/transferFeeInfo`

**Usage**: Transfer fee information

---

#### Heavy Weight (600 per request)

- `GET /api/v1/trades`
- `GET /api/v1/recentTrades`

**Usage**: Trade history queries

---

#### Maximum Weight (3000 per request)

**Account Management Endpoints** (12 total):
- `GET /api/v1/account`
- `GET /api/v1/accountsByL1Address`
- `GET /api/v1/pnl`
- `GET /api/v1/accountTxs`
- `GET /api/v1/withdraw/history`
- Other account-related endpoints

**Usage**: Account data, transactions, withdrawals

---

#### Default Weight (300 per request)

**All Other Endpoints**:
- `GET /api/v1/orderBooks`
- `GET /api/v1/orderBookDetails`
- `GET /api/v1/orderBookOrders`
- `GET /api/v1/candlesticks`
- `GET /api/v1/fundings`
- `GET /api/v1/exchangeStats`
- `GET /api/v1/block`
- `GET /api/v1/blocks`
- `GET /api/v1/currentHeight`
- `GET /api/v1/tx`
- `GET /api/v1/txs`
- `GET /api/v1/blockTxs`
- `GET /`
- `GET /info`

**Usage**: Market data, blocks, general queries

---

### Rate Limit Calculation Examples

#### Standard Account (60 per minute)

**Scenario**: Fetching market data
- 1x `GET /orderBooks` (300 weight) → **Exceeds limit!**

**Solution**: Use Premium account or reduce frequency

**Practical Use**:
- 10x `GET /nextNonce` (6 weight each) = 60 weight → ✓ Allowed
- 1x `GET /orderBooks` (300 weight) → ✗ Exceeds limit

**Conclusion**: Standard accounts are limited to lightweight operations only.

---

#### Premium Account (24,000 per minute)

**Scenario 1**: High-frequency trading
- 100x `POST /sendTx` (6 weight each) = 600 weight
- 50x `GET /orderBookDetails` (300 weight each) = 15,000 weight
- **Total**: 15,600 weight → ✓ Allowed

**Scenario 2**: Market data monitoring
- 60x `GET /candlesticks` (300 weight each) = 18,000 weight
- 10x `GET /recentTrades` (600 weight each) = 6,000 weight
- **Total**: 24,000 weight → ✓ At limit

**Scenario 3**: Account management
- 8x `GET /account` (3000 weight each) = 24,000 weight → ✓ At limit
- Cannot make additional requests this minute

---

## Transaction Type Limits (Standard Accounts Only)

Standard accounts have additional per-transaction-type limits.

**Time Window**: 60 seconds (rolling minute) OR per 10 seconds for some types

### Transaction Type Rate Limits

| Transaction Type | Type Code | Limit | Time Window |
|------------------|-----------|-------|-------------|
| Default (most types) | Various | 40 requests | per minute |
| L2Withdraw | 13 | 2 requests | per minute |
| L2UpdateLeverage | 20 | 1 request | per minute |
| L2CreateSubAccount | 9 | 2 requests | per minute |
| L2CreatePublicPool | 10 | 2 requests | per minute |
| L2ChangePubKey | 8 | 2 requests | per 10 seconds |
| L2Transfer | 12 | 1 request | per minute |

**Note**: Premium accounts do not have transaction type limits, only the standard REST API weight limits.

---

### Transaction Type Codes Reference

| Type Code | Transaction Type | Description |
|-----------|------------------|-------------|
| 8 | L2ChangePubKey | Change API public key |
| 9 | L2CreateSubAccount | Create sub-account |
| 10 | L2CreatePublicPool | Create public liquidity pool |
| 11 | L2UpdatePublicPool | Update public pool |
| 12 | L2Transfer | Transfer between accounts |
| 13 | L2Withdraw | Withdraw to L1 |
| 14 | L2CreateOrder | Create order |
| 15 | L2CancelOrder | Cancel order |
| 16 | L2CancelAllOrders | Cancel all orders |
| 17 | L2ModifyOrder | Modify order |
| 18 | L2MintShares | Mint pool shares |
| 19 | L2BurnShares | Burn pool shares |
| 20 | L2UpdateLeverage | Update leverage |
| 28 | L2CreateGroupedOrders | Create grouped orders |
| 29 | L2UpdateMargin | Update margin |

---

## Explorer API Rate Limits

**Base URL**: `https://explorer.elliot.ai`

**Limit**: **15 weighted requests per minute** (all user types)

**Time Window**: 60 seconds (rolling)

### Explorer Endpoint Weights

| Endpoint | Weight |
|----------|--------|
| `GET /api/search` | 3 |
| `GET /accounts/{param}/positions` | 2 |
| `GET /accounts/{param}/logs` | 2 |
| All other endpoints | 1 |

**Example**:
- 5x `GET /api/search` (3 weight each) = 15 weight → ✓ At limit
- 7x `GET /accounts/{id}/positions` (2 weight each) = 14 weight → ✓ Allowed

---

## WebSocket Rate Limits

### Connection Limits (Per IP)

| Limit Type | Value |
|------------|-------|
| Max Connections | 100 |
| Max Subscriptions per Connection | 100 |
| Max Total Subscriptions | 1,000 |
| Max Connections Per Minute | 60 |
| Max Unique Accounts | 10 |

---

### Message Limits (Per IP)

| Limit Type | Value | Notes |
|------------|-------|-------|
| Max Messages Per Minute | 200 | Excludes sendTx/sendTxBatch |
| Max Inflight Messages | 50 | Messages awaiting response |

**Note**: `sendTx` and `sendTxBatch` WebSocket messages are excluded from the 200 messages/minute limit but still count toward REST API rate limits.

---

### WebSocket Connection Best Practices

1. **Reuse Connections**: Open one connection and subscribe to multiple channels
2. **Limit Subscriptions**: Stay under 100 subscriptions per connection
3. **Avoid Reconnection Loops**: Implement exponential backoff
4. **Monitor Account Limits**: Max 10 unique accounts per IP
5. **Batch Subscriptions**: Subscribe to channels in batches during initial connection

---

## Rate Limit Headers

### Response Headers

Lighter API **may** include rate limit information in response headers (verify in actual responses):

**Possible Headers**:
- `X-RateLimit-Limit` - Total requests allowed in window
- `X-RateLimit-Remaining` - Remaining requests in current window
- `X-RateLimit-Reset` - Unix timestamp when limit resets

**Note**: Header names and presence are not confirmed in documentation. Implement detection and fallback.

---

## Error Responses

### HTTP 429 Too Many Requests

**When**: Rate limit exceeded

**Response**:
```json
{
  "code": 429,
  "message": "Too Many Requests"
}
```

**Action**: Wait before retrying (implement backoff)

---

### WebSocket Disconnection

**When**: Excessive messages on WebSocket

**Behavior**: Connection will be closed by server

**Action**: Implement exponential backoff before reconnecting

---

## Rate Limit Strategies

### For Standard Accounts

**Limitations**:
- Only 60 weighted requests/minute
- Cannot fetch most market data (300+ weight)
- Limited to transaction operations only

**Recommended Actions**:
1. Use WebSocket for market data instead of REST
2. Minimize REST API calls to essential operations only
3. Cache data aggressively
4. Consider upgrading to Premium

**Use Cases**:
- Low-frequency trading (few orders per minute)
- Testing and development
- Personal use with manual trading

---

### For Premium Accounts

**Capabilities**:
- 24,000 weighted requests/minute
- Full access to all endpoints
- High-frequency trading possible

**Optimization Strategies**:

#### 1. Request Batching
```
Instead of:
  10 separate GET /orderBookDetails calls (300 weight each) = 3000 weight

Use:
  1 GET /orderBookDetails?market_id=255 call (300 weight) = 300 weight
```

#### 2. Caching
- Cache market metadata (decimals, limits) - rarely changes
- Cache orderbook snapshots - update via WebSocket
- Cache account data - poll only when needed

#### 3. WebSocket Preference
- Use WebSocket for real-time data (orderbook, trades, account updates)
- Use REST only for operations (create order, cancel order)
- Reduce REST calls by 90%+

#### 4. Intelligent Polling
- Market data: Update every 1-5 seconds (not every 100ms)
- Account data: Update every 5-10 seconds
- Static data: Update every hour

---

## Rate Limit Tracking

### Client-Side Implementation

Track rate limit usage locally to avoid hitting limits.

**Algorithm**:
1. Maintain sliding window of requests (last 60 seconds)
2. Track weight of each request
3. Sum weights in current window
4. Check before making request
5. Delay if approaching limit

**Example (Rust)**:
```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

pub struct RateLimiter {
    requests: VecDeque<(Instant, u32)>,  // (timestamp, weight)
    limit: u32,
    window: Duration,
}

impl RateLimiter {
    pub fn new(limit: u32, window_seconds: u64) -> Self {
        Self {
            requests: VecDeque::new(),
            limit,
            window: Duration::from_secs(window_seconds),
        }
    }

    pub fn check_and_add(&mut self, weight: u32) -> Result<(), String> {
        let now = Instant::now();

        // Remove expired requests
        while let Some((timestamp, _)) = self.requests.front() {
            if now.duration_since(*timestamp) > self.window {
                self.requests.pop_front();
            } else {
                break;
            }
        }

        // Calculate current usage
        let current_usage: u32 = self.requests.iter().map(|(_, w)| w).sum();

        // Check if request would exceed limit
        if current_usage + weight > self.limit {
            let oldest = self.requests.front().unwrap().0;
            let wait_time = self.window - now.duration_since(oldest);
            return Err(format!("Rate limit exceeded. Wait {:?}", wait_time));
        }

        // Add request
        self.requests.push_back((now, weight));
        Ok(())
    }

    pub fn get_remaining(&self) -> u32 {
        let now = Instant::now();
        let current_usage: u32 = self.requests
            .iter()
            .filter(|(timestamp, _)| now.duration_since(*timestamp) <= self.window)
            .map(|(_, w)| w)
            .sum();

        self.limit.saturating_sub(current_usage)
    }
}

// Usage
let mut limiter = RateLimiter::new(24000, 60);  // Premium account

match limiter.check_and_add(300) {  // orderBooks request
    Ok(_) => {
        // Make API call
    },
    Err(e) => {
        // Wait before retrying
        println!("Rate limit: {}", e);
    }
}
```

---

## Retry Strategy

### Exponential Backoff

When hitting rate limits, implement exponential backoff:

**Algorithm**:
1. Initial wait: 1 second
2. On subsequent failures: wait * 2
3. Max wait: 60 seconds
4. Add jitter: ±20%

**Example (Rust)**:
```rust
use tokio::time::{sleep, Duration};
use rand::Rng;

pub async fn retry_with_backoff<F, T, E>(
    mut f: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    let mut wait_seconds = 1;
    let mut rng = rand::thread_rng();

    for attempt in 0..max_retries {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if attempt == max_retries - 1 => return Err(e),
            Err(_) => {
                // Add jitter: ±20%
                let jitter = rng.gen_range(-0.2..0.2);
                let actual_wait = wait_seconds as f64 * (1.0 + jitter);

                sleep(Duration::from_secs_f64(actual_wait)).await;

                wait_seconds = (wait_seconds * 2).min(60);
            }
        }
    }

    unreachable!()
}
```

---

## Burst vs Sustained Rates

### Burst Allowance

Lighter uses a **rolling 60-second window**, which allows short bursts.

**Example** (Premium, 24,000 limit):
- Seconds 0-10: 12,000 weight → ✓ Allowed
- Seconds 10-20: 12,000 weight → ✓ Allowed
- **Total in 20s**: 24,000 weight → ✓ Allowed

However:
- Seconds 0-10: 24,000 weight → ✓ Allowed (burst)
- Seconds 10-20: 1,000 weight → ✗ Denied (still in window)

**Implication**: You can burst up to the limit, but then must wait for the window to slide.

---

### Sustained Rate

For continuous operation, target **80% of limit** to account for variations:

**Premium Account**:
- Limit: 24,000 per minute
- Target: 19,200 per minute
- Safe rate: ~320 requests/minute (at 300 weight each)

**Standard Account**:
- Limit: 60 per minute
- Target: 48 per minute
- Safe rate: ~8 requests/minute (at 6 weight each)

---

## Monitoring and Alerts

### Metrics to Track

1. **Request Rate**: Requests per minute
2. **Weight Usage**: Total weight consumed per minute
3. **429 Errors**: Count of rate limit rejections
4. **Retry Rate**: How often requests are retried
5. **Queue Depth**: Number of requests waiting due to rate limits

### Alerts

Set up alerts for:
- 429 error rate > 1% of requests
- Weight usage > 90% of limit
- Sustained retry rate > 5%

---

## Best Practices Summary

### Do's

1. **Use Premium Account** for production trading
2. **Prefer WebSockets** for real-time data
3. **Cache aggressively** - market metadata, symbols
4. **Track rate limits** client-side
5. **Implement backoff** on 429 errors
6. **Batch requests** when possible (use market_id=255)
7. **Monitor usage** and set alerts
8. **Use connection pooling** for REST API

### Don'ts

1. **Don't poll market data** at high frequency via REST
2. **Don't ignore 429 errors** - implement proper handling
3. **Don't open multiple WebSocket connections** unnecessarily
4. **Don't use Standard account** for production (too limited)
5. **Don't retry immediately** after 429 - use backoff
6. **Don't make redundant requests** - cache when possible

---

## Comparison with Other Exchanges

| Exchange | Standard Tier | Premium Tier | Burst Allowance |
|----------|---------------|--------------|-----------------|
| Lighter | 60/min | 24,000/min | Yes (rolling window) |
| Binance | 1,200/min | 6,000/min | Yes (weight-based) |
| Bybit | 120/min | 600/min | Limited |
| OKX | 20/s | 40/s | No (strict) |

**Lighter Advantages**:
- Very high premium limits (24,000/min)
- Weighted system allows optimization
- Rolling window enables bursts

**Lighter Disadvantages**:
- Standard tier extremely limited (60/min)
- No free tier suitable for production

---

## Rate Limit Testing

### Test Scenarios

1. **Burst Test**: Submit maximum requests in minimum time
2. **Sustained Test**: Maintain constant rate for 5 minutes
3. **Recovery Test**: Hit limit, wait, verify recovery
4. **WebSocket Test**: Max connections, subscriptions, messages

### Tools

- **Apache Bench**: `ab -n 1000 -c 10 https://mainnet.zklighter.elliot.ai/api/v1/orderBooks`
- **Custom Script**: Implement precise weight tracking
- **Monitoring**: Track actual vs expected limit behavior

---

## Implementation Checklist

- [ ] Implement rate limiter with weight tracking
- [ ] Add endpoint weight constants
- [ ] Implement exponential backoff on 429
- [ ] Parse rate limit headers (if present)
- [ ] Cache market metadata
- [ ] Prefer WebSocket for real-time data
- [ ] Monitor 429 error rate
- [ ] Set up alerts for high usage
- [ ] Test burst and sustained rates
- [ ] Document limits for users
- [ ] Handle transaction type limits (Standard accounts)

---

## References

- [Lighter API Rate Limits](https://apidocs.lighter.xyz/docs/rate-limits)
- [Account Types](https://apidocs.lighter.xyz/docs/account-types)
- [WebSocket Reference](https://apidocs.lighter.xyz/docs/websocket-reference)
