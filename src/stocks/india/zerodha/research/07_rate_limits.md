# Zerodha Kite Connect - Tiers, Pricing, and Rate Limits

## Free Tier (Personal API)

### Access Level

- **Requires sign-up**: Yes (active Zerodha trading account required)
- **API key required**: Yes
- **Credit card required**: No
- **Prerequisites**:
  - Active Zerodha trading account
  - 2FA TOTP enabled on account
  - Register on Kite Connect Developer Portal

### Rate Limits

Personal API has the **same rate limits** as paid Connect API:

| Rate Limit Type | Value |
|-----------------|-------|
| **Requests per second** | 10 per API key |
| **Requests per minute** | 600 (10 req/sec × 60) |
| **Requests per hour** | Not explicitly limited (36,000 theoretical max) |
| **Requests per day** | Not explicitly limited |
| **Burst allowed** | No (hard limit at 10 req/sec) |

**Important**: Rate limits are enforced at the **API key level**, not per user or access token.

### Data Access

| Feature | Personal API | Connect API (₹500/mo) |
|---------|--------------|----------------------|
| **Real-time data** | ✅ Yes (REST quotes) | ✅ Yes (REST + WebSocket) |
| **Delayed data** | ❌ No | ❌ No |
| **Historical candle data** | ❌ No | ✅ Yes |
| **WebSocket streaming** | ❌ No | ✅ Yes |
| **Data types** | Quotes, positions, orders | All data types |

**Personal API Includes**:
- Place/modify/cancel orders
- Get order history
- Track positions and holdings
- Check margins and funds
- Real-time quotes (REST only, rate-limited)
- Portfolio management

**Personal API Does NOT Include**:
- Historical candle data (`/instruments/historical` endpoint)
- WebSocket streaming (real-time push updates)
- Bulk data access

### Limitations

- **Symbols**: Unlimited (all Indian exchanges supported)
- **Endpoints**: Historical data and WebSocket restricted
- **Features**: No WebSocket, no historical data
- **Order limits**: Same as paid tier (3,000/day, 200/min)

---

## Paid Tier (Connect API)

### Pricing

| Tier | Price | Description |
|------|-------|-------------|
| **Connect API** | ₹500/month | Full API access with WebSocket and historical data |

**Pricing History**:
- Previous pricing: ₹2,000/month (reduced in 2024)
- Current pricing: ₹500/month (as of 2026)

**No enterprise tiers** - single paid tier at ₹500/month

### What Unlocks at Paid Tier?

| Feature | Personal (Free) | Connect (₹500/mo) |
|---------|----------------|-------------------|
| Order management | ✅ Yes | ✅ Yes |
| Real-time quotes (REST) | ✅ Yes | ✅ Yes |
| Historical candles | ❌ No | ✅ Yes |
| WebSocket streaming | ❌ No | ✅ Yes |
| Market depth | ✅ Yes (REST) | ✅ Yes (REST + WS) |
| Portfolio management | ✅ Yes | ✅ Yes |
| Margins calculation | ✅ Yes | ✅ Yes |
| Rate limits | 10 req/sec | 10 req/sec |
| Order limits | 3,000/day | 3,000/day |

**Key Upgrades**:
1. **Historical candle data**: Access to years of OHLC data
2. **WebSocket streaming**: Real-time push updates for up to 3,000 instruments
3. **Reduced latency**: WebSocket provides faster updates than REST polling

---

## Rate Limit Details

### How Measured

- **Window**: Per second (rolling)
- **Rolling window**: Yes (continuous 1-second window)
- **Fixed window**: No

### Limit Scope

- **Per API key**: ✅ Yes (primary enforcement)
- **Per IP address**: ❌ No
- **Per account**: ❌ No (per API key, can have multiple keys)
- **Shared across**: All requests from the same API key

**Important**: If you create multiple apps (API keys), each has its own 10 req/sec limit.

### Rate Limit by Endpoint Category

| Endpoint Category | Rate Limit | Notes |
|------------------|------------|-------|
| **Quote endpoints** | 1 req/sec | `/quote`, `/quote/ohlc`, `/quote/ltp` |
| **Historical data** | 3 req/sec | `/instruments/historical` |
| **All other endpoints** | 10 req/sec | Orders, portfolio, margins, etc. |

**Critical**: Quote endpoints have stricter limits (1 req/sec vs 10 req/sec)

### Burst Handling

- **Burst allowed**: No
- **Burst size**: N/A
- **Token bucket**: Not explicitly documented (appears to be simple rate limiting)
- **Exceeding limit**: Immediate 429 error response

**Behavior**:
- Hard limit at specified rate
- No burst allowance
- Requests exceeding rate are rejected immediately
- No queuing of excess requests

### Response Headers

**Rate limit headers** (when available):

```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 7
X-RateLimit-Reset: 1706254380
```

**Important**: Not all endpoints return rate limit headers. Monitor your request rate client-side.

### Error Response (HTTP 429)

```json
{
  "status": "error",
  "message": "Too many requests",
  "error_type": "NetworkException",
  "data": null
}
```

**HTTP Status**: 429 Too Many Requests

**Error Type**: NetworkException

### Handling Strategy

**Best Practices**:

1. **Client-side rate limiting**:
   ```python
   import time
   from collections import deque

   class RateLimiter:
       def __init__(self, max_requests, time_window):
           self.max_requests = max_requests
           self.time_window = time_window
           self.requests = deque()

       def wait_if_needed(self):
           now = time.time()

           # Remove requests outside time window
           while self.requests and self.requests[0] < now - self.time_window:
               self.requests.popleft()

           if len(self.requests) >= self.max_requests:
               sleep_time = self.time_window - (now - self.requests[0])
               if sleep_time > 0:
                   time.sleep(sleep_time)
               self.requests.popleft()

           self.requests.append(time.time())

   # Usage
   limiter = RateLimiter(max_requests=10, time_window=1.0)  # 10 req/sec

   def make_api_call():
       limiter.wait_if_needed()
       # Make API call
   ```

2. **Exponential backoff** on 429:
   ```python
   import time

   def api_call_with_retry(func, max_retries=5):
       delay = 1  # Start with 1 second

       for attempt in range(max_retries):
           try:
               return func()
           except RateLimitError:
               if attempt == max_retries - 1:
                   raise
               print(f"Rate limited, retrying in {delay}s...")
               time.sleep(delay)
               delay *= 2  # Exponential backoff
   ```

3. **Use WebSocket instead of polling**:
   - REST polling consumes rate limit
   - WebSocket provides push updates (no polling needed)
   - Subscribe to up to 3,000 instruments on one connection

4. **Batch requests**:
   - Quote endpoints support multiple instruments
   - `/quote?i=NSE:INFY&i=NSE:RELIANCE` counts as 1 request
   - Use batch requests to optimize rate limit usage

5. **Cache responses**:
   - Cache instrument list (updated once daily)
   - Cache user profile (rarely changes)
   - Don't repeatedly fetch static data

---

## Order-Specific Limits

### Daily Order Limits

| Limit Type | Value | Scope |
|------------|-------|-------|
| **Total orders per day** | 3,000 | Per user/API key, all segments |
| **MIS orders per day** | 2,000 | Across all segments |
| **Cover Orders per day** | 2,000 | Across all segments |

**Important**: These limits are **regulatory requirements**, not API limits.

### Per-Minute Order Limits

| Limit Type | Value |
|------------|-------|
| **Orders per minute** | 200 |

**Enforcement**: Hard limit at 200 orders/minute

### Order Modification Limits

| Limit Type | Value |
|------------|-------|
| **Modifications per order** | 25 |

**Important**: Cannot modify an order more than 25 times.

---

## WebSocket Specific Limits

### Connection Limits

| Limit Type | Value |
|------------|-------|
| **Max connections per API key** | 3 |
| **Max connections per IP** | Not limited |
| **Max connections total** | 3 per API key |

**Example**:
- Can open 3 WebSocket connections simultaneously
- Each connection can subscribe to 3,000 instruments
- Total capacity: 9,000 instruments across 3 connections

### Subscription Limits

| Limit Type | Value |
|------------|-------|
| **Max subscriptions per connection** | 3,000 instruments |
| **Max symbols per subscription** | N/A (subscribe by token, not symbol) |
| **Total max subscriptions** | 9,000 (3 connections × 3,000) |

**Important**: Free Personal API does NOT support WebSocket (limit is 0).

### Message Rate Limits

- **Outgoing messages**: No documented limit (reasonable usage expected)
- **Incoming messages**: Unlimited (server pushes as needed)
- **Server throttling**: Not documented (server sends at market update rate)
- **Auto-disconnect on violation**: Connection may be closed on abuse

### Connection Duration

| Parameter | Value |
|-----------|-------|
| **Max lifetime** | Until market close or disconnect |
| **Auto-reconnect needed** | After token expiry (6 AM IST) |
| **Idle timeout** | None (server sends heartbeats) |

**Behavior**:
- Connection stays open during market hours
- Automatically disconnected after market close
- Must reconnect with new access_token after 6 AM expiry

---

## Monitoring Usage

### Dashboard

- **Usage dashboard**: Available on Kite Connect Developer Portal
- **Real-time tracking**: No (usage logs updated periodically)
- **Historical usage**: Available on dashboard
- **URL**: https://developers.kite.trade

**Dashboard Features**:
- View API key usage
- Monitor app statistics
- Check subscription status
- View billing information

### API Endpoints

**No dedicated usage tracking endpoints** in the API itself.

**Workaround**: Track client-side:
```python
class UsageTracker:
    def __init__(self):
        self.request_count = 0
        self.order_count = 0
        self.daily_orders = 0
        self.start_time = time.time()

    def track_request(self):
        self.request_count += 1

    def track_order(self):
        self.order_count += 1
        self.daily_orders += 1

    def reset_daily(self):
        self.daily_orders = 0

    def get_stats(self):
        elapsed = time.time() - self.start_time
        return {
            "total_requests": self.request_count,
            "requests_per_second": self.request_count / elapsed,
            "total_orders": self.order_count,
            "daily_orders": self.daily_orders,
            "orders_remaining": 3000 - self.daily_orders
        }
```

### Alerts

- **Email alerts**: No automatic alerts from Zerodha
- **Webhook**: Not available
- **Client-side alerts**: Implement your own monitoring

**Recommended**:
```python
def check_limits(tracker):
    stats = tracker.get_stats()

    if stats["daily_orders"] > 2500:
        print(f"WARNING: {stats['daily_orders']}/3000 daily orders used!")

    if stats["requests_per_second"] > 8:
        print("WARNING: Approaching rate limit!")
```

---

## Comparison: Free vs Paid

| Feature | Free (Personal) | Paid (Connect - ₹500/mo) |
|---------|----------------|--------------------------|
| **Pricing** | Free | ₹500/month |
| **Setup** | Requires trading account | Requires trading account |
| **REST API** | ✅ Yes | ✅ Yes |
| **WebSocket** | ❌ No | ✅ Yes (3 connections, 3k instruments each) |
| **Historical data** | ❌ No | ✅ Yes |
| **Order management** | ✅ Yes | ✅ Yes |
| **Portfolio** | ✅ Yes | ✅ Yes |
| **Margins** | ✅ Yes | ✅ Yes |
| **Real-time quotes** | ✅ Yes (REST, rate-limited) | ✅ Yes (REST + WebSocket) |
| **Rate limit** | 10 req/sec | 10 req/sec |
| **Quote endpoints** | 1 req/sec | 1 req/sec |
| **Historical endpoint** | N/A | 3 req/sec |
| **Order limits** | 3,000/day, 200/min | 3,000/day, 200/min |
| **Support** | Community forum | Community forum |

---

## Rate Limit Summary Table

| Endpoint/Feature | Rate Limit | Scope |
|------------------|------------|-------|
| **General API** | 10 req/sec | Per API key |
| **Quote endpoints** | 1 req/sec | Per API key |
| **Historical data** | 3 req/sec | Per API key |
| **Orders (placement)** | 200/min, 3,000/day | Per user/API key |
| **Order modifications** | 25 per order | Per order |
| **WebSocket connections** | 3 max | Per API key |
| **WebSocket subscriptions** | 3,000 per connection | Per connection |
| **MIS orders** | 2,000/day | Per user |
| **Cover Orders** | 2,000/day | Per user |

---

## Best Practices

1. **Implement client-side rate limiting**: Don't rely on server to enforce limits

2. **Use WebSocket for real-time data**: Reduces REST API calls, more efficient

3. **Batch quote requests**: Request multiple instruments in one call

4. **Cache static data**: Instrument list, user profile

5. **Monitor your usage**: Track requests, orders, and subscriptions client-side

6. **Implement exponential backoff**: On 429 errors, wait and retry

7. **Multiple API keys** (if needed): Each key has separate rate limits

8. **Order limit tracking**: Monitor daily order count to avoid hitting 3,000 limit

9. **Choose appropriate mode**: Use `ltp` WebSocket mode if you don't need full depth

10. **Test in low-volume periods**: Test your rate limit handling before production use

---

## Free Trial

**No free trial for Connect API** - must subscribe at ₹500/month.

**Alternative**: Use free Personal API for development, upgrade when WebSocket/historical data needed.

---

## Billing

- **Billing cycle**: Monthly
- **Payment method**: Debit/credit card or other Zerodha payment methods
- **Auto-renewal**: Yes (subscription-based)
- **Cancellation**: Can cancel anytime via developer portal
- **Pro-rated refunds**: Not specified (check terms on kite.trade)

**Subscription management**: https://developers.kite.trade
