# Dhan - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes (Dhan trading account required)
- API key required: Yes (free to generate)
- Credit card required: No
- KYC required: Yes (standard Dhan account opening)
- Minimum balance: No minimum for API access

### Trading APIs - Completely FREE

**Always Free Features**:
- Order placement, modification, cancellation
- Portfolio management (holdings, positions)
- Funds information
- Trade history
- Order book access
- Super Orders (bracket + trailing SL)
- Forever Orders (GTT)
- EDIS integration
- Postback/Webhooks
- Live order updates via WebSocket

**No monthly charges** for Trading APIs - available to all Dhan users.

### Data APIs - Conditional FREE

**Free Access Criteria**:
- Complete 25+ trades in previous 30 days
- Automatic qualification check
- No manual activation needed

**If FREE (25+ trades)**:
- All Data APIs included at no cost
- Same rate limits as paid tier
- Full historical data access
- Real-time market feeds
- Option chain data

**If NOT Free (<25 trades)**:
- Cost: Rs. 499 + taxes per month
- Billed monthly
- Can activate/deactivate anytime

### Rate Limits (Free Tier)

**Order APIs**:
- Requests per second: 25
- Requests per minute: 250
- Requests per hour: 1,000
- Requests per day: 7,000
- Burst allowed: No explicit burst
- **Additional**: Max 25 modifications per order

**Data APIs**:
- Requests per second: 5
- Requests per minute: Unlimited
- Requests per hour: Unlimited
- Requests per day: 100,000

**Quote APIs**:
- Requests per second: 1
- Requests per minute: Unlimited
- Requests per hour: Unlimited
- Requests per day: Unlimited

**Non-Trading APIs**:
- Requests per second: 20
- Requests per minute: Unlimited
- Requests per hour: Unlimited
- Requests per day: Unlimited

### Data Access
- Real-time data: Yes (tick-by-tick)
- Delayed data: No delay (real-time only)
- Historical data: Yes
  - Daily: From instrument inception (10+ years for many stocks)
  - Intraday: Last 5 years (1m, 5m, 15m, 25m, 60m)
- WebSocket: Yes
  - 5 connections allowed
  - 5,000 instruments per connection
  - 25,000 total instruments possible
- Data types:
  - Live quotes (LTP, OHLC, Volume)
  - Market depth (5-level, 20-level, 200-level)
  - Historical OHLC
  - Option chains with Greeks
  - Open Interest (for derivatives)
  - Order updates

### Limitations
- Symbols: Unlimited (all NSE, BSE, MCX instruments)
- Endpoints: All available (no restrictions)
- Features: Full access to all features
- Static IP: Required from January 2026 for Order APIs
- Token validity: 24 hours (must regenerate daily)

## Paid Tiers

**Dhan has NO traditional paid tiers**. The model is:

| Tier | Price | Who Qualifies | Trading APIs | Data APIs |
|------|-------|---------------|--------------|-----------|
| Active Trader | FREE | 25+ trades/month | FREE | FREE |
| Less Active Trader | Rs. 499/month | <25 trades/month | FREE | PAID |

### Data API Subscription (Rs. 499 + taxes/month)

**Only applies to Data APIs if trading volume <25 trades/month**.

**What's Included**:
- All historical data APIs
- Market quote APIs
- Live market feed (WebSocket)
- Option chain data
- Full market depth
- Same rate limits as free tier

**What's Still Free**:
- All Trading APIs (always free regardless of subscription)
- Portfolio/Holdings queries
- Order management
- Funds information

### No "Enterprise" or "Pro" Tiers
Dhan does NOT offer:
- Higher rate limit tiers
- Premium support tiers
- Volume-based pricing
- White-label solutions (via API)

All users get same rate limits and features.

## Rate Limit Details

### How Measured
- Window: Per second (for most APIs)
- Rolling window: Yes
- Fixed window: No (rolling window implementation)
- Granularity: Second-level for Order/Data APIs

### Limit Scope
- Per IP address: No
- Per API key: Yes (limits are per account/API key)
- Per account: Yes (same as API key)
- Shared across: All requests from same API key

### Burst Handling
- Burst allowed: No explicit burst allowance
- Burst size: N/A
- Burst window: N/A
- Token bucket: Likely (standard implementation)
- Behavior: Requests rejected immediately if limit exceeded

### Response Headers
**Dhan does NOT return rate limit headers** in responses.

No headers like:
- `X-RateLimit-Limit`
- `X-RateLimit-Remaining`
- `X-RateLimit-Reset`
- `Retry-After`

**Implication**: Client must track rate limits locally.

### Error Response (HTTP 429 - Rate Limit Exceeded)

**Not confirmed** - Dhan documentation doesn't specify exact 429 response format.

**Expected**:
```json
{
  "errorType": "RateLimitError",
  "errorCode": "RL4001",
  "errorMessage": "Rate limit exceeded. Please retry after some time."
}
```

### Handling Strategy
1. **Client-side rate limiting**:
   - Track requests per second locally
   - Implement queue with rate limiter
   - Use token bucket algorithm

2. **Exponential backoff**:
   - On any error, wait before retry
   - Increase wait time exponentially (1s, 2s, 4s, 8s...)
   - Max backoff: 60 seconds

3. **Request queuing**:
   - Queue all API requests
   - Process queue at max allowed rate
   - Priority queuing (order placement > data fetch)

4. **Connection pooling** (for WebSocket):
   - Use multiple connections (max 5)
   - Distribute subscriptions across connections
   - Balance load to avoid single connection bottleneck

## Quota/Credits System

**NOT APPLICABLE** - Dhan does not use a credit/quota system.

Rate limits are simple request-per-time-period limits, not credit-based.

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP**: Not specified (likely unlimited)
- **Max connections per API key**: 5 (for Live Market Feed)
- **Max connections total**: 5

### Subscription Limits

**Live Market Feed**:
- Max subscriptions per connection: 5,000 instruments
- Max symbols per subscription: 5,000 (cumulative)
- Total across all connections: 25,000 instruments (5 × 5,000)

**20-Level Market Depth**:
- Max subscriptions per connection: 50 instruments
- Max symbols per subscription: 50 (cumulative)

**200-Level Market Depth**:
- Max subscriptions per connection: 1 instrument only
- Max symbols per subscription: 1

### Message Rate Limits
- **Client → Server**: No explicit limit on subscription messages
- **Server → Client**: No throttling (server pushes at market speed)
- Messages per second: Controlled by server (client receives at market rate)
- Auto-disconnect on violation: No (no client-side message limit)

### Connection Duration
- Max lifetime: Unlimited (no forced disconnect)
- Auto-reconnect needed: No (connection stays alive)
- Idle timeout: None
- Recommended: Monitor health, reconnect if stale data detected

## Monitoring Usage

### Dashboard
- Usage dashboard: Available in Dhan web portal (web.dhan.co)
- Real-time tracking: Limited (no live API usage meter)
- Historical usage: Not publicly available
- Trading volume tracking: Yes (for qualifying for free Data APIs)

### API Endpoints
**No dedicated API endpoints for usage monitoring**.

Dhan does NOT provide:
- `GET /account/usage`
- `GET /account/limits`
- Usage statistics API

**Workaround**: Track locally in your application.

### Alerts
- Email alerts: No automatic alerts for API usage
- Webhook: No usage webhooks
- Threshold alerts: No built-in alerts

**Recommendation**: Implement client-side monitoring.

## Rate Limit Comparison (Industry Context)

| Broker | Order API (req/sec) | Data API (req/sec) | Daily Limit | WebSocket Connections |
|--------|---------------------|-----------------------|-------------|----------------------|
| **Dhan** | **25** | **5** | **7,000 (orders), 100,000 (data)** | **5** |
| Zerodha | 10 | 3 | 3,000 | 3 |
| Upstox | 10 | 3 | N/A | 3 |
| Angel One | 10 | 5 | N/A | 1 |

**Dhan offers the most liberal rate limits in Indian broking industry.**

## Cost Comparison (Monthly)

| Broker | Trading APIs | Data APIs | Historical Data |
|--------|--------------|-----------|-----------------|
| **Dhan** | **FREE** | **FREE (if 25+ trades) or Rs. 499** | **Included** |
| Zerodha | Rs. 2,000 | Included | Included |
| Upstox | FREE | Rs. 300 | Extra charges |
| Angel One | FREE | Rs. 500 | Limited |

**Dhan is the most cost-effective for active traders (25+ trades/month).**

## Optimization Strategies

### To Maximize Free Tier
1. **Maintain 25+ trades/month**:
   - Ensures free Data API access
   - No monthly charges
   - Full feature access

2. **Use WebSocket for live data**:
   - More efficient than polling
   - Subscribe to 5,000 instruments per connection
   - Use 5 connections = 25,000 instruments

3. **Batch requests**:
   - Market quotes: Up to 1,000 instruments per request
   - Reduces API call count
   - Stays within rate limits

4. **Client-side caching**:
   - Cache historical data locally
   - Avoid repeated requests
   - Refresh only when needed

### Order API Optimization
1. **Queue order requests**:
   - Max 25/second
   - Implement local queue
   - Process sequentially at 20/sec (safety margin)

2. **Minimize modifications**:
   - Max 25 modifications per order
   - Plan order parameters carefully
   - Cancel and replace instead of excessive modifications

3. **Use Super Orders**:
   - Single API call for entry + target + SL
   - Reduces API call count vs manual management
   - Trailing SL built-in

### Data API Optimization
1. **Historical data caching**:
   - 90-day limit for intraday data queries
   - Fetch once, store locally
   - Incremental updates only

2. **WebSocket over REST**:
   - Real-time data via WebSocket (no rate limit)
   - REST quotes only for snapshots
   - Reduces REST API usage

3. **Option chain optimization**:
   - Rate limit: 1 req/3 seconds
   - Cache option chain data
   - Refresh only when needed (OI updates slowly)

## Special Considerations for 2026

### Static IP Requirement (Jan 2026)
- **Impact**: Must have static IP for order placement
- **Workaround**: Use cloud instances with Elastic IP
- **Cost**: Cloud provider charges for static IP (AWS: ~$3-5/month)
- **Sandbox**: Static IP NOT required for testing

### Token Management
- **24-hour validity**: Automate daily token generation
- **SEBI compliance**: Cannot extend beyond 24 hours
- **Strategy**: Scheduled job at 6 AM IST to regenerate

### Scaling Considerations
With 7,000 order API calls/day:
- ~7 orders/minute sustained
- 25/second burst capacity
- Sufficient for most retail algo strategies
- May need multiple accounts for HFT

## Fair Usage Policy

While not explicitly documented, implied fair usage:
- Don't abuse rate limits (no hammering)
- Don't create artificial trading volume for free Data API access
- Don't share API keys across multiple users
- Use sandbox for testing (not production for testing)

Violation may result in:
- API key suspension
- Account termination
- Reduced rate limits (at Dhan's discretion)

## Rate Limit Error Handling (Best Practices)

### Python Example
```python
import time
from functools import wraps

def rate_limit_handler(max_retries=3, backoff_factor=2):
    def decorator(func):
        @wraps(func)
        def wrapper(*args, **kwargs):
            for attempt in range(max_retries):
                try:
                    return func(*args, **kwargs)
                except RateLimitError:
                    if attempt < max_retries - 1:
                        wait_time = backoff_factor ** attempt
                        time.sleep(wait_time)
                    else:
                        raise
        return wrapper
    return decorator

@rate_limit_handler()
def place_order(dhan_client, order_params):
    return dhan_client.place_order(order_params)
```

### Rust Example
```rust
use std::time::Duration;
use tokio::time::sleep;

async fn retry_with_backoff<F, T, E>(
    mut f: F,
    max_retries: u32,
    backoff_factor: u64,
) -> Result<T, E>
where
    F: FnMut() -> Result<T, E>,
{
    for attempt in 0..max_retries {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                let wait_time = backoff_factor.pow(attempt);
                sleep(Duration::from_secs(wait_time)).await;
            }
            Err(e) => return Err(e),
        }
    }
    unreachable!()
}
```

## Conclusion

Dhan's pricing and rate limits are **industry-leading**:
- **Most generous rate limits** (25 orders/sec, 20,000 non-trading/day)
- **Completely free Trading APIs** (no monthly charges)
- **Free Data APIs** for active traders (25+ trades/month)
- **Affordable Data API** (Rs. 499/month for inactive traders)
- **No tiered pricing** (everyone gets same limits)

**Best for**:
- Active retail algo traders
- Strategy developers
- Quant traders
- FinTech platforms building on top

**Limitations**:
- Static IP requirement (from Jan 2026) may be inconvenient
- No higher rate limit tiers (stuck at 25/sec for orders)
- 24-hour token validity requires daily regeneration
