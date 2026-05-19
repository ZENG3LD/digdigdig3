# Angel One SmartAPI - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up**: Yes (Angel One trading account + SmartAPI registration)
- **API key required**: Yes (obtain from SmartAPI dashboard)
- **Credit card required**: No
- **Trading account required**: Yes (must have active Angel One trading account)
- **KYC required**: Yes (standard KYC for Indian trading accounts)

### Rate Limits

#### Order Management APIs
- **Place Order**: 20 orders per second
- **Modify Order**: 20 orders per second
- **Cancel Order**: 20 orders per second
- **GTT APIs (Create/Modify/Cancel)**: 20 per second (shared with order APIs)

**Note**: Rate limit increased from 10/sec to 20/sec in recent updates (2024-2026).

#### Query APIs
- **Individual Order Status**: 10 requests per second
- **Margin Calculator**: 10 requests per second
- **Other query endpoints**: Not publicly specified (reasonable usage expected)

#### Additional Limits
- **Per minute limits**: Yes (specific values not publicly documented)
- **Per hour limits**: Yes (specific values not publicly documented)
- **Burst allowed**: Not specified (likely enforced via rolling window)

### Data Access
- **Real-time data**: Yes (all market data is real-time)
- **Delayed data**: No (all data is real-time, no delay)
- **Historical data**: Yes (FREE for all segments)
  - NSE, BSE (equities)
  - NFO, BFO (derivatives)
  - MCX, CDS (commodities & currency)
  - NCDEX (commodities)
  - Depth: Up to 2000 days for daily bars, varies by interval
- **WebSocket**: Yes (WebSocket V2 included)
  - Max 1 connection limit not specified
  - Max 1000 token subscriptions per connection
  - All 4 modes available (LTP, Quote, Snap Quote, Depth 20)
- **Data types**: All data types available
  - Market quotes (LTP, OHLC, ticker)
  - Historical candles (all intervals)
  - Order book depth (5 levels in Snap Quote, 20 levels in Depth 20)
  - Real-time order updates

### Limitations
- **Symbols**: Unlimited (all symbols on supported exchanges)
- **Endpoints**: All endpoints available
- **Features**: Full access to all features
  - Trading (all order types)
  - Market data (real-time & historical)
  - Portfolio management
  - WebSocket streaming
  - GTT, OCO, AMO, Bracket Orders
  - Margin calculator

### Cost
- **API Usage**: **FREE** (₹0)
- **Historical Data**: **FREE** (₹0)
- **WebSocket Access**: **FREE** (₹0)
- **Account Maintenance**: Standard Angel One account charges (if any)

### Charges Apply For
- **Brokerage**: Standard trading charges apply
  - Equity delivery: Typically ₹20 per trade (flat)
  - Equity intraday: Typically ₹20 per trade (flat)
  - F&O: Typically ₹20 per order (flat)
  - Currency: Typically ₹20 per order (flat)
  - Commodities: Typically ₹20 per order (flat)
- **Exchange charges**: Standard NSE/BSE/MCX transaction charges
- **Taxes**: GST, STT, stamp duty as per regulations

**Note**: Brokerage charges are separate from API charges. API usage itself is completely free.

## Paid Tiers

**No separate paid tiers for SmartAPI.**

Angel One SmartAPI is completely free for all Angel One clients. There are no premium API tiers.

| Tier Name | Price | Rate Limit | Additional Features | WebSocket | Historical | Support |
|-----------|-------|------------|---------------------|-----------|------------|---------|
| Free (Only Tier) | ₹0 | 20/sec (orders), 10/sec (queries) | All features | Unlimited | Unlimited | Forum + Email |

### No Premium Features
All features are available in the free tier:
- Full trading capabilities
- Real-time market data
- Historical data (all segments)
- WebSocket V2 with Depth 20
- GTT, OCO, AMO, Bracket Orders
- Margin calculator
- Portfolio & position tracking

## Rate Limit Details

### How Measured
- **Window**: Per second (primary), with additional per-minute and per-hour limits
- **Rolling window**: Likely (standard API practice, not explicitly documented)
- **Fixed window**: Not specified
- **Granularity**: Requests counted per endpoint category

### Limit Scope
- **Per IP address**: Not specified (likely yes for abuse prevention)
- **Per API key**: Yes (limits apply per API key)
- **Per account**: Yes (limits tied to client code/account)
- **Shared across**: Order APIs share 20/sec limit (place/modify/cancel/GTT)

### Burst Handling
- **Burst allowed**: Not explicitly documented
- **Burst size**: Not specified
- **Burst window**: Not specified
- **Token bucket**: Not specified (implementation details not public)

### Response Headers
**Not publicly documented** - Angel One SmartAPI does not appear to return standard rate limit headers.

Standard headers like `X-RateLimit-Limit`, `X-RateLimit-Remaining`, `X-RateLimit-Reset` are **not documented** as being available.

**Client must track requests locally** to avoid hitting limits.

### Error Response (HTTP 429)
```json
{
  "status": false,
  "message": "Rate limit exceeded. Please try after some time.",
  "errorcode": "AB8003",
  "data": null
}
```

**Note**: Exact error code for rate limiting not explicitly documented. Error response format follows standard Angel One error structure.

### Handling Strategy
1. **Track requests locally**: Maintain request counter per second/minute
2. **Rate limiting client-side**: Implement request queue with rate limiter
3. **Exponential backoff**: Wait progressively longer on repeated errors
4. **Retry logic**: Retry with delay on rate limit errors
5. **Queue requests**: Use queue for order placement to stay under limits
6. **Batch when possible**: Use batch endpoints where available (e.g., margin calculator for basket)

## Quota/Credits System

**Not applicable** - Angel One SmartAPI does not use a quota or credits system.

All endpoints are available with rate limits only (no usage quotas or credit deductions per request).

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP**: Not publicly specified
- **Max connections per API key**: Not publicly specified
- **Max connections total**: Not specified (likely multiple concurrent connections allowed)

### Subscription Limits
- **Max subscriptions per connection**: **1000 tokens** (hard limit, documented)
- **Max symbols per subscription**: Not specified per subscription message
- **Total active subscriptions**: 1000 tokens across all subscriptions

**Critical**: If you exceed 1000 tokens, ticks won't be received for tokens beyond the limit.

### Message Rate Limits
- **Messages per second**: Not publicly specified
- **Server may throttle**: Likely yes (to prevent abuse)
- **Auto-disconnect on violation**: Not documented (likely yes)

### Connection Duration
- **Max lifetime**: Until midnight (session validity)
- **Auto-reconnect needed**: Yes (after midnight when session expires)
- **Idle timeout**: Not publicly specified
- **Reconnection**: SDKs provide built-in reconnection logic

## Monitoring Usage

### Dashboard
- **Usage dashboard**: https://smartapi.angelone.in (SmartAPI dashboard)
- **Real-time tracking**: Not publicly documented
- **Historical usage**: Logs visible in dashboard (details not specified)
- **API call logs**: Available in SmartAPI portal (exact features not detailed)

### API Endpoints
**No public API endpoints for usage monitoring.**

There are no documented REST endpoints to check:
- Current usage
- Remaining quota
- Rate limit status

**Users must track usage client-side.**

### Alerts
- **Email alerts**: Not documented
- **Webhook**: Not available
- **Dashboard notifications**: Likely available in SmartAPI portal (not detailed)

## Practical Rate Limit Scenarios

### High-Frequency Trading
- **20 orders/sec** = Maximum 1200 orders per minute
- **Suitable for**: Medium-frequency algo trading
- **Not suitable for**: Ultra-high-frequency trading (microsecond level)

### Market Data Polling
- **No specified limit** for quote/LTP APIs (use responsibly)
- **Better approach**: Use WebSocket for real-time data (avoid polling)

### Multi-Symbol Strategies
- **1000 token WebSocket limit**: Can track up to 1000 symbols simultaneously
- **For more symbols**: Rotate subscriptions or use multiple connections (if allowed)

### Order Management
- **20 orders/sec**: Sufficient for most retail algo trading
- **For higher needs**: Implement queuing and batching strategies

## Recent Changes (2024-2026)

### Rate Limit Increases
- **2024**: Order API rate limit increased from 10/sec to 20/sec
- **Announcement**: Communicated via SmartAPI forum
- **Impact**: 2x improvement for high-frequency order placement

### New Features (Free)
- **June 2025**: Margin Calculator API launched (10/sec limit)
- **2024**: WebSocket V2 Depth 20 feature (beta → stable)
- **2024**: Free historical data for all segments announced
- **2024**: Up to 8000 candles per request (increased limit)
- **2024**: Support for 120 indices across NSE, BSE, MCX

## Comparison with Other India Brokers

| Feature | Angel One | Zerodha Kite | Upstox |
|---------|-----------|--------------|--------|
| **API Cost** | Free | ₹2000/month | Free |
| **Order Rate Limit** | 20/sec | 10/sec | 10/sec |
| **Historical Data** | Free | Paid/Limited | Limited |
| **WebSocket** | Free | Included | Included |
| **Depth Levels** | 20 (Depth 20) | 5 | 5 |

**Angel One advantage**: Free API with higher rate limits and extended order book depth.

## Best Practices for Rate Limit Management

### 1. Client-Side Rate Limiting
```python
import time
from collections import deque

class RateLimiter:
    def __init__(self, max_calls, period):
        self.max_calls = max_calls
        self.period = period
        self.calls = deque()

    def __call__(self, func):
        def wrapper(*args, **kwargs):
            now = time.time()
            # Remove old calls
            while self.calls and self.calls[0] < now - self.period:
                self.calls.popleft()
            # Check limit
            if len(self.calls) >= self.max_calls:
                sleep_time = self.period - (now - self.calls[0])
                time.sleep(sleep_time)
                return wrapper(*args, **kwargs)
            # Make call
            self.calls.append(now)
            return func(*args, **kwargs)
        return wrapper

# Usage
@RateLimiter(max_calls=20, period=1.0)
def place_order(order_params):
    return smartApi.placeOrder(order_params)
```

### 2. Order Queue
```python
import queue
import threading

order_queue = queue.Queue()

def order_worker():
    while True:
        order_params = order_queue.get()
        try:
            result = place_order(order_params)  # Rate-limited
            print(f"Order placed: {result}")
        except Exception as e:
            print(f"Order failed: {e}")
        finally:
            order_queue.task_done()

# Start worker thread
threading.Thread(target=order_worker, daemon=True).start()

# Queue orders
order_queue.put(order_params_1)
order_queue.put(order_params_2)
```

### 3. WebSocket Instead of Polling
```python
# BAD: Polling (wastes rate limits)
while True:
    quote = smartApi.getLTP("NSE", "3045")
    time.sleep(1)  # Still 1 req/sec per symbol

# GOOD: WebSocket (no rate limit consumption)
sws = SmartWebSocketV2(AUTH_TOKEN, API_KEY, CLIENT_CODE, FEED_TOKEN)
sws.subscribe(correlation_id, mode=1, token_list)
# Receive continuous updates via callback
```

### 4. Batch Operations
```python
# Use margin calculator for multiple positions at once
positions_basket = [
    {"symbol": "SBIN-EQ", "qty": 100, "price": 500},
    {"symbol": "INFY-EQ", "qty": 50, "price": 1500},
    # ... more positions
]
margin_required = smartApi.marginCalculator(positions_basket)
# Single API call for entire basket
```

### 5. Error Handling
```python
import time

def safe_api_call(func, *args, max_retries=3, **kwargs):
    for attempt in range(max_retries):
        try:
            return func(*args, **kwargs)
        except Exception as e:
            if "rate limit" in str(e).lower():
                wait_time = 2 ** attempt  # Exponential backoff
                print(f"Rate limited. Waiting {wait_time}s...")
                time.sleep(wait_time)
            else:
                raise
    raise Exception("Max retries exceeded")
```

## Summary Table

| Aspect | Details |
|--------|---------|
| **API Cost** | FREE (₹0) |
| **Account Required** | Yes (Angel One trading account) |
| **Order Rate Limit** | 20 per second |
| **Query Rate Limit** | 10 per second (individual order status, margin calc) |
| **WebSocket Token Limit** | 1000 symbols |
| **Historical Data** | FREE, unlimited (all segments) |
| **Real-time Data** | FREE, unlimited (rate limits apply) |
| **Session Validity** | Until midnight |
| **Paid Tiers** | None (single free tier) |
| **Trading Charges** | Standard brokerage (separate from API) |
| **Support** | Forum, Email, GitHub |
| **Recent Improvements** | 2x order rate limit (2024), Depth 20 (2024) |

## Recommendations

### For Retail Algo Traders
- Angel One SmartAPI is excellent value (completely free)
- 20/sec order limit sufficient for most strategies
- Use WebSocket for market data (avoid polling)
- Implement client-side rate limiting

### For High-Frequency Traders
- 20/sec may be limiting for true HFT
- Consider if broker allows multiple API keys for higher throughput
- Optimize order placement logic to minimize API calls
- Use margin calculator to pre-validate orders

### For Market Data Applications
- Leverage free historical data (significant cost savings vs competitors)
- WebSocket V2 with Depth 20 provides excellent market depth
- 1000 token limit sufficient for most portfolios
- Use instrument master file for symbol lookups (no API calls)

### General Best Practices
1. Implement client-side rate limiting
2. Use WebSocket instead of polling
3. Handle rate limit errors gracefully
4. Monitor usage via dashboard
5. Batch operations when possible
6. Queue orders to smooth request rate
7. Use margin calculator for pre-trade checks
