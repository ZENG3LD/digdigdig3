# dYdX v4 Rate Limits

## Overview

dYdX v4 has a multi-layered rate limiting system:

1. **Blockchain-Level Rate Limits** - Protocol-enforced limits on order placement
2. **Indexer API Rate Limits** - HTTP/WebSocket query limits (public endpoints)
3. **Withdrawal Rate Limits** - Governance-controlled withdrawal gating

## Blockchain-Level Rate Limits

These are **protocol-enforced** limits that apply to all traders and cannot be bypassed.

### Order Placement Rate Limits

**Short-Term Orders**:
- **200 order attempts per block** (per account)
- Includes placements and cancellations
- All subaccounts under the same main account share this limit

**Stateful Orders** (Long-Term/Conditional):
- **2 orders per block** (per account)
- **20 orders per 100 blocks** (per account)

### Important Notes

1. **Per Account, Not Per Subaccount**:
   - Subaccounts 0, 1, 2, etc. all share the same rate limit
   - Limit is tied to the main account (wallet address)

2. **Block Time**:
   - dYdX v4 block time: ~1-2 seconds
   - Short-term limit: ~200 orders per 1-2 seconds = ~100-200 orders/sec
   - Stateful limit: ~2 orders per 1-2 seconds = ~1-2 orders/sec

3. **Enforcement**:
   - Exceeded limits result in transaction rejection
   - No grace period or queue
   - Must wait for next block

### Querying Current Rate Limits

**Endpoint**: `https://<REST_NODE_ENDPOINT>/dydxprotocol/clob/block_rate`

Response:
```json
{
  "short_term_order_rate_limit": 200,
  "stateful_order_rate_limit": 2,
  "stateful_order_rate_limit_window": 100
}
```

**Note**: Rate limits can be updated via governance.

### Order Type Selection Impact

**Short-Term Orders** (orderFlags = 0):
- Higher rate limit (200/block)
- Best for high-frequency trading
- Live in memory only (~20 blocks = 30 seconds)
- Best-effort cancellation

**Stateful Orders** (orderFlags = 32 or 64):
- Lower rate limit (2/block)
- Best for longer-lived orders
- Persist on-chain
- Guaranteed cancellation

**Strategy**:
- Use short-term orders for market making and scalping
- Use stateful orders for limit orders that may take time to fill

## Indexer API Rate Limits

The Indexer provides read-only access to market data, account info, and historical data.

### HTTP API Rate Limits

**Status**: Not explicitly documented

From available information:
- Public indexer endpoint has rate limits
- Specific numbers (requests/second, requests/minute) are **not publicly documented**
- Rate limits likely vary by endpoint type:
  - Market data endpoints: Higher limits
  - Account-specific queries: Moderate limits
  - Historical data queries: Lower limits (more expensive)

**Rate Limit Headers** (typical for HTTP APIs):
- `X-RateLimit-Limit`: Total allowed requests
- `X-RateLimit-Remaining`: Remaining requests
- `X-RateLimit-Reset`: When limit resets

**Note**: Actual header names may differ. Check response headers when implementing.

### WebSocket Rate Limits

**Connection Limits**:
- Maximum concurrent connections: Not documented
- Maximum subscriptions per connection: Not documented

**Message Rate Limits**:
- Subscribe/unsubscribe message rate: Not documented
- Batching option available to reduce message frequency

**Example batched subscription**:
```json
{
  "type": "subscribe",
  "channel": "v4_trades",
  "id": "BTC-USD",
  "batched": true
}
```

**Effect**: Reduces update frequency, potentially staying within rate limits

### HTTP Error Response for Rate Limiting

**Status Code**: `429 Too Many Requests`

**Response Body**:
```json
{
  "errors": [
    {
      "msg": "Rate limit exceeded",
      "code": "RATE_LIMIT_EXCEEDED"
    }
  ]
}
```

### Strategies to Avoid Rate Limits

1. **Use WebSockets Instead of Polling**:
   - Subscribe to real-time updates
   - Reduces need for frequent HTTP requests

2. **Batch Requests**:
   - Query multiple markets in one request where possible
   - Use parent subaccount endpoints to get data for multiple subaccounts

3. **Cache Data**:
   - Cache market info (changes infrequently)
   - Cache ticker ↔ clobPairId mappings
   - Use local candle aggregation

4. **Run Your Own Indexer**:
   - No rate limits on self-hosted indexer
   - Requires infrastructure investment
   - Full control over query performance

5. **Respect 429 Responses**:
   - Implement exponential backoff
   - Track rate limit headers
   - Spread requests over time

### Example: Rate Limit Handling

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

struct RateLimiter {
    requests_per_second: u32,
    last_request: Instant,
}

impl RateLimiter {
    fn new(requests_per_second: u32) -> Self {
        Self {
            requests_per_second,
            last_request: Instant::now(),
        }
    }

    async fn wait_if_needed(&mut self) {
        let min_interval = Duration::from_millis(1000 / self.requests_per_second as u64);
        let elapsed = self.last_request.elapsed();

        if elapsed < min_interval {
            sleep(min_interval - elapsed).await;
        }

        self.last_request = Instant::now();
    }
}

// Usage
let mut limiter = RateLimiter::new(10); // 10 requests per second

for ticker in ["BTC-USD", "ETH-USD", "SOL-USD"] {
    limiter.wait_if_needed().await;
    let orderbook = get_orderbook(ticker).await?;
}
```

## Withdrawal Rate Limits

Withdrawals from dYdX v4 to other chains are rate-limited for security.

### Noble USDC Withdrawal Limits

**Hourly Limit**:
```
max(1% of TVL, $1,000,000) per hour
```

**Daily Limit**:
```
max(10% of TVL, $10,000,000) per day
```

**Where**:
- **TVL** = Total Value Locked in the protocol
- Limits apply to aggregate withdrawals across all users

### Example Calculation

If TVL = $50,000,000:
- Hourly limit: max(1% × $50M, $1M) = $1,000,000 (larger)
- Daily limit: max(10% × $50M, $10M) = $10,000,000 (larger)

If TVL = $500,000,000:
- Hourly limit: max(1% × $500M, $1M) = $5,000,000 (larger)
- Daily limit: max(10% × $500M, $10M) = $50,000,000 (larger)

### Withdrawal Gating

Withdrawals may be temporarily frozen (50-block freeze) when:

1. **Negatively Collateralized Subaccount**:
   - A subaccount is insolvent (negative equity)
   - Cannot be liquidated or deleveraged
   - Prevents further value extraction

2. **Chain Outage**:
   - Network downtime lasting 5+ minutes
   - Prevents withdrawals during potential attack

**Duration**: 50 blocks (~75-100 seconds)

### Governance Control

- Withdrawal rate limit parameters can be modified through governance
- Community can vote to increase/decrease limits
- Emergency changes possible in crisis situations

### Querying Withdrawal Status

No specific endpoint documented for checking current withdrawal limits or status.

**Workaround**:
- Monitor governance proposals for limit changes
- Track total withdrawal volume via blockchain explorer
- Implement retry logic for withdrawal transactions

## Indexer Throughput

### Expected Data Throughput

**On-chain Data State Changes**:
- Expected throughput: **10-50 events/second**
- Includes: Order placements, fills, position updates, transfers

**Off-chain Data State Changes**:
- Expected throughput: **500-1,000 events/second**
- Includes: Orderbook updates, price feeds, oracle updates

**Difference**: 10-100x more off-chain events than on-chain events

### Implications for API Design

1. **WebSocket Preferred for High-Frequency Data**:
   - Orderbook updates: Use WebSocket
   - Trade updates: Use WebSocket
   - Price feeds: Use WebSocket

2. **HTTP API for Periodic Queries**:
   - Account balances: Poll every few seconds
   - Position updates: Poll every few seconds
   - Historical data: Query as needed

3. **Data Lag**:
   - Indexer may lag behind blockchain by 0-2 seconds
   - During high load: Lag can increase
   - WebSocket typically faster than HTTP due to read replica lag

## Best Practices

### General Guidelines

1. **Minimize API Calls**:
   - Cache static data (market info)
   - Use WebSockets for real-time data
   - Batch requests where possible

2. **Implement Backoff**:
   - Exponential backoff on 429 errors
   - Track retry count
   - Maximum retry limit

3. **Monitor Rate Limits**:
   - Track response headers
   - Log rate limit hits
   - Adjust request rate dynamically

4. **Separate Read and Write Operations**:
   - Indexer for reads (market data, account info)
   - Node API for writes (orders, cancellations)
   - Different rate limit pools

### Order Management Best Practices

1. **Prefer Short-Term Orders for HFT**:
   - 200/block limit vs 2/block
   - Faster execution
   - Lower latency

2. **Batch Order Operations**:
   - Place multiple orders in same block
   - Cancel multiple orders together
   - Use client-side order ID tracking

3. **Monitor Block Height**:
   - Track current block
   - Calculate when rate limit resets
   - Schedule orders across blocks

4. **Handle Rate Limit Errors**:
   - Catch transaction rejections
   - Queue orders for next block
   - Implement local rate limiting

### Example: Block-Aware Rate Limiting

```rust
use std::collections::VecDeque;
use std::time::{Duration, Instant};

struct BlockRateLimiter {
    max_per_block: u32,
    current_block: u64,
    current_block_count: u32,
    pending_orders: VecDeque<Order>,
}

impl BlockRateLimiter {
    fn new(max_per_block: u32) -> Self {
        Self {
            max_per_block,
            current_block: 0,
            current_block_count: 0,
            pending_orders: VecDeque::new(),
        }
    }

    async fn update_block_height(&mut self, new_height: u64) {
        if new_height > self.current_block {
            self.current_block = new_height;
            self.current_block_count = 0;

            // Process pending orders
            while self.current_block_count < self.max_per_block {
                if let Some(order) = self.pending_orders.pop_front() {
                    self.place_order_now(order).await;
                    self.current_block_count += 1;
                } else {
                    break;
                }
            }
        }
    }

    async fn place_order(&mut self, order: Order) -> Result<(), ExchangeError> {
        if self.current_block_count < self.max_per_block {
            self.place_order_now(order).await?;
            self.current_block_count += 1;
            Ok(())
        } else {
            // Queue for next block
            self.pending_orders.push_back(order);
            Ok(())
        }
    }

    async fn place_order_now(&self, order: Order) -> Result<(), ExchangeError> {
        // Actual gRPC call to place order
        todo!()
    }
}
```

## Self-Hosted Indexer

### Benefits
- **No rate limits** on queries
- **Lower latency** (direct database access)
- **Full control** over infrastructure
- **Custom indexing** (add your own queries)

### Requirements
- Database server (PostgreSQL)
- Indexer service (runs continuously)
- Node connection (to sync blockchain data)
- Storage (grows over time with historical data)

### Resources
- GitHub: https://github.com/dydxprotocol/v4-chain
- Documentation: https://docs.dydx.exchange/infrastructure_providers-network/resources

### Trade-offs
- Infrastructure cost vs API rate limits
- Maintenance overhead
- Initial sync time

## Summary Table

| Limit Type | Value | Scope | Enforcement |
|------------|-------|-------|-------------|
| Short-term orders | 200/block | Per account | Protocol |
| Stateful orders | 2/block, 20/100 blocks | Per account | Protocol |
| Indexer HTTP | Not documented | Per IP/connection | Indexer |
| Indexer WebSocket | Not documented | Per connection | Indexer |
| USDC withdrawals | max(1% TVL, $1M)/hour | Global | Protocol |
| USDC withdrawals | max(10% TVL, $10M)/day | Global | Protocol |

## Rate Limit Queries

### Check Block Rate Limits
```bash
curl https://<NODE_REST_ENDPOINT>/dydxprotocol/clob/block_rate
```

### Monitor Indexer Headers
```bash
curl -i https://indexer.dydx.trade/v4/perpetualMarkets
# Check response headers for rate limit info
```

### Get Current Block Height
```bash
curl https://indexer.dydx.trade/v4/height
```

## Error Handling Strategy

```rust
async fn execute_with_retry<F, T>(
    mut f: F,
    max_retries: u32,
) -> Result<T, ExchangeError>
where
    F: FnMut() -> Result<T, ExchangeError>,
{
    let mut retries = 0;
    let mut backoff = Duration::from_millis(100);

    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(ExchangeError::RateLimitExceeded) if retries < max_retries => {
                retries += 1;
                tokio::time::sleep(backoff).await;
                backoff *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Monitoring and Observability

### Metrics to Track
- Requests per second (by endpoint)
- Rate limit hits (429 responses)
- Average response time
- Current block height
- Orders per block (current usage)
- Pending order queue size

### Logging
```rust
log::warn!(
    "Rate limit hit on {} - backing off for {:?}",
    endpoint,
    backoff_duration
);

log::info!(
    "Block {} - {} orders placed, {} remaining in limit",
    block_height,
    orders_placed,
    limit_remaining
);
```

## Future Considerations

- Rate limits subject to governance changes
- Monitor governance proposals for limit adjustments
- New rate limit tiers may be introduced
- Self-hosted indexer remains best option for high-frequency trading

## Resources

- **Indexer Source**: https://github.com/dydxprotocol/v4-chain/tree/main/indexer
- **Protocol Docs**: https://docs.dydx.xyz
- **Rate Limit Governance**: Check dYdX governance forum
