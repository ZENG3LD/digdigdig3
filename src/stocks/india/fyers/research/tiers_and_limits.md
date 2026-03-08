# Fyers - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up:** Yes (Fyers trading account)
- **API key required:** Yes (create app in dashboard)
- **Credit card required:** No (free API access)
- **Trading account required:** Yes (active Fyers account)
- **2FA TOTP required:** Yes (mandatory for API access)

### Rate Limits
- **Requests per second:** 10
- **Requests per minute:** 200
- **Requests per hour:** Not specified (covered by daily limit)
- **Requests per day:** 100,000 (increased 10x from previous 10,000)
- **Burst allowed:** Not specified

**Rate Limit Changes (V3 Update - 2026):**
- Previous daily limit: 10,000 requests/day
- New daily limit: 100,000 requests/day
- Represents a **10x increase** in capacity

### Data Access
- **Real-time data:** Yes (tick-by-tick, <1 second latency)
- **Delayed data:** No (all data is real-time)
- **Historical data:** Yes (availability varies by symbol/timeframe)
  - Intraday: 1min, 5min, 15min, etc.
  - Daily: Multiple years
  - Options: Limited (may be daily only)
- **WebSocket:** Allowed
  - Connections: Not officially limited per user
  - Subscriptions: Up to 5,000 symbols (V3, with latest SDK)
  - Practical limit: 200 symbols per connection (widely reported)
  - Legacy limit: 50 symbols (older versions)
- **Data types:** All data types available
  - Market data (quotes, depth, trades)
  - Account data (profile, funds, holdings)
  - Order data (orderbook, tradebook, positions)
  - Historical OHLC candles

### Limitations
- **Symbols:** Unlimited (all NSE, BSE, MCX, NCDEX symbols)
- **Endpoints:** All endpoints available
- **Features:** All features available (no restrictions)
- **Order execution:** No limits (subject to exchange rules)
- **WebSocket subscriptions:** 5,000 symbols (V3) / 200 symbols (practical) / 50 symbols (legacy)

### What's Included in Free Tier
- REST API access (all endpoints)
- WebSocket streaming (Data, Order, TBT)
- Historical data
- Real-time market data
- Order placement/management
- Portfolio tracking
- E-DIS integration
- Symbol master data
- Market status
- No subscription fee

---

## Paid Tiers

### Fyers API Bridge

**Price:** Rs 500/month or Rs 3,500/year

**What is API Bridge:**
- Third-party platform integration
- Connect Fyers account to external platforms (TradingView, AmiBroker, etc.)
- Not required for direct API usage

**Note:** Direct API usage is **completely free**. API Bridge is only for third-party platform integrations.

### No Tiered API Plans

Fyers API does **not** have tiered pricing (Free/Starter/Pro/Enterprise).

**All users get:**
- Same rate limits
- Same data access
- Same features
- Same real-time data
- Same historical data
- Same WebSocket capabilities

**Only Cost:** Maintaining active Fyers trading account (brokerage fees apply for trading).

---

## Comparison with Other Brokers

| Feature | Fyers | Typical Broker |
|---------|-------|----------------|
| API Access | Free | Rs 2,000-3,000/month |
| Real-time Data | Free | Often paid add-on |
| WebSocket | Free | Often limited/paid |
| Rate Limits | 100k/day | Varies |
| Historical Data | Free | Often limited depth |

**Fyers Advantage:** Completely free API access is rare among Indian brokers.

---

## Rate Limit Details

### How Measured
- **Window:** Per second / minute / day
- **Rolling window:** Yes (continuous tracking)
- **Fixed window:** No

### Rate Limit Breakdown

| Limit Type | Value | Window Type |
|------------|-------|-------------|
| Per Second | 10 requests | Rolling 1-second window |
| Per Minute | 200 requests | Rolling 1-minute window |
| Per Day | 100,000 requests | Rolling 24-hour window |

### Limit Scope
- **Per IP address:** No
- **Per API key:** No
- **Per account:** Yes (trading account)
- **Shared across:** All apps created under same account

**Note:** Creating multiple apps does **not** increase rate limits. Limits apply to the trading account, not individual apps.

### Burst Handling
- **Burst allowed:** Not officially specified
- **Burst size:** Unknown
- **Burst window:** Unknown
- **Token bucket:** Likely (standard implementation)

**Recommendation:** Stay within per-second limit to avoid sudden rate limit errors.

### Response Headers

**Rate Limit Headers (if present):**
```
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 195
X-RateLimit-Reset: 1640000000
```

**Note:** Header presence not guaranteed in all responses. Monitor 429 errors instead.

### Error Response (HTTP 429)

**Status Code:** 429 Too Many Requests

**Response Body:**
```json
{
  "s": "error",
  "code": 429,
  "message": "request limit reached"
}
```

**Known Issue:** Some users report receiving two JSON objects in response:
```json
{"code": 429, "message": "request limit reached", "s": "error"}
{"code": 200, "s": "ok"}
```

### Handling Strategy

**Recommended Approach:**
1. **Implement Rate Limiter** in your code
   - Track requests per second/minute/day
   - Queue requests to stay within limits
2. **Exponential Backoff** on 429 errors
   - Wait 1s, 2s, 4s, 8s, max 60s
   - Retry after backoff period
3. **Monitor Usage** via dashboard
4. **Use WebSocket** for real-time data (doesn't count against REST limits)
5. **Batch Requests** when possible (basket orders, multiple symbols in quotes)

**Python Example - Simple Rate Limiter:**
```python
import time
from collections import deque

class RateLimiter:
    def __init__(self, max_per_second=10, max_per_minute=200):
        self.max_per_second = max_per_second
        self.max_per_minute = max_per_minute
        self.second_requests = deque()
        self.minute_requests = deque()

    def wait_if_needed(self):
        now = time.time()

        # Clean old requests
        while self.second_requests and self.second_requests[0] < now - 1:
            self.second_requests.popleft()
        while self.minute_requests and self.minute_requests[0] < now - 60:
            self.minute_requests.popleft()

        # Check limits
        if len(self.second_requests) >= self.max_per_second:
            sleep_time = 1 - (now - self.second_requests[0])
            if sleep_time > 0:
                time.sleep(sleep_time)

        if len(self.minute_requests) >= self.max_per_minute:
            sleep_time = 60 - (now - self.minute_requests[0])
            if sleep_time > 0:
                time.sleep(sleep_time)

        # Record request
        now = time.time()
        self.second_requests.append(now)
        self.minute_requests.append(now)

# Usage
limiter = RateLimiter()

def make_api_call():
    limiter.wait_if_needed()
    # Make API request here
    response = fyers.get_profile()
    return response
```

---

## Quota/Credits System

### Does Fyers Use Credits?
**No** - Fyers uses traditional rate limits, not a credit/quota system.

**Rate Limit Model:**
- Fixed number of requests per time period
- All endpoints count equally (1 request = 1 count)
- No weighted costs per endpoint

---

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP:** Not officially documented
- **Max connections per API key:** Not officially documented
- **Max connections total:** Not officially documented
- **Recommended:** 1-2 connections (Data + Order WebSocket)

**Multiple Connections:**
Users report successfully running multiple WebSocket connections, but official limits not documented.

### Subscription Limits

**Data WebSocket:**

| Version | Symbol Limit | Notes |
|---------|--------------|-------|
| V3 (Latest) | 5,000 symbols | Announced in V3 update, requires latest SDK |
| Practical | 200 symbols | Widely reported working limit |
| Legacy | 50 symbols | Older API versions, some users still see this limit |

**Error When Exceeded:**
```json
{
  "s": "error",
  "code": -351,
  "message": "You have provided symbols greater than 50"
}
```

**Recommendation:**
- Test with your specific SDK version
- Start with 50 symbols, increase gradually
- Monitor for error code -351
- Use multiple connections if needed (>50 symbols total)

**Order WebSocket:**
- No subscription limit (auto-receives all account updates)

**TBT WebSocket:**
- Limits not officially documented
- Channel-based subscriptions

### Message Rate Limits
- **Messages per second:** Not officially documented
- **Server may throttle:** Yes (if excessive subscribe/unsubscribe)
- **Auto-disconnect on violation:** Possible

**Best Practice:** Don't spam subscribe/unsubscribe messages. Set up subscriptions once.

### Connection Duration
- **Max lifetime:** Unlimited (persistent connection)
- **Auto-reconnect needed:** Yes (on network issues/server disconnect)
- **Idle timeout:** None (as long as ping/pong active)

**Auto-Reconnect:**
```python
# Python SDK - auto-reconnect enabled
data_socket = data_ws.FyersDataSocket(
    access_token=access_token,
    reconnect=True  # Automatically reconnects on disconnect
)
```

---

## Monitoring Usage

### Dashboard
- **Usage dashboard:** https://myapi.fyers.in/dashboard/
- **Real-time tracking:** Not available in dashboard
- **Historical usage:** Not available in dashboard

**Dashboard Features:**
- View created apps
- Manage API credentials
- Create/delete apps
- No usage analytics/graphs

### API Endpoints
- **Check quota:** No dedicated endpoint
- **Check limits:** No dedicated endpoint
- **Response headers:** May include rate limit info (not guaranteed)

### Monitoring Strategy

**Manual Tracking:**
1. Implement request counter in your code
2. Log API calls with timestamps
3. Track against known limits (10/sec, 200/min, 100k/day)
4. Alert when approaching limits

**Example Tracker:**
```python
class UsageTracker:
    def __init__(self):
        self.daily_count = 0
        self.daily_reset = time.time() + 86400

    def track_request(self):
        if time.time() > self.daily_reset:
            self.daily_count = 0
            self.daily_reset = time.time() + 86400

        self.daily_count += 1

        if self.daily_count >= 90000:  # 90% of 100k
            print("WARNING: Approaching daily limit")

        return self.daily_count
```

### Alerts
- **Email alerts:** Not provided by Fyers
- **Webhook:** Not provided by Fyers
- **Custom implementation:** Required (implement in your code)

---

## Rate Limit Increase Requests

### Can Limits Be Increased?
- **Not publicly documented**
- **Likely no** - Limits seem fixed for all users
- **Contact support** if needed: support@fyers.in

### Workarounds for High-Volume Users
1. **Use WebSocket** for real-time data (no REST calls)
2. **Batch requests** (quotes for multiple symbols, basket orders)
3. **Cache data** (don't re-fetch static data)
4. **Optimize polling** (reduce unnecessary calls)
5. **Distribute across time** (don't burst all at once)

---

## Cost Breakdown

### Free API Usage
- **Monthly cost:** Rs 0
- **Annual cost:** Rs 0
- **Per request cost:** Rs 0
- **WebSocket cost:** Rs 0
- **Historical data cost:** Rs 0

### Fyers Account Costs
- **Brokerage:** Pay-per-trade (Rs 20/order or 0.03%, whichever lower)
- **AMC:** Varies (check Fyers pricing)
- **API access:** FREE

### API Bridge (Optional)
- **Monthly:** Rs 500
- **Annual:** Rs 3,500
- **Purpose:** Third-party platform integration only

### Total Cost of Ownership (API Only)
- **Setup:** Rs 0 (if already have Fyers account)
- **Monthly:** Rs 0 (direct API usage)
- **Scaling:** Rs 0 (no additional costs for higher usage)

**Comparison:**
- Many brokers charge Rs 2,000-3,000/month for API access
- Fyers API is completely free (major competitive advantage)

---

## Best Practices for Rate Limit Management

1. **Implement Rate Limiter** in your code (essential)
2. **Use WebSocket for real-time data** (bypasses REST limits)
3. **Batch API calls** when possible
4. **Cache static data** (symbol master, market hours)
5. **Monitor daily usage** and implement alerts
6. **Handle 429 errors gracefully** with exponential backoff
7. **Don't poll unnecessarily** (use WebSocket instead)
8. **Optimize historical data fetches** (fetch once, cache)
9. **Use Lite mode WebSocket** when only LTP needed
10. **Distribute requests evenly** (avoid bursts)

---

## Rate Limit Comparison (Indian Brokers)

| Broker | API Cost | Daily Limit | Real-time Data | WebSocket |
|--------|----------|-------------|----------------|-----------|
| Fyers | Free | 100,000 | Free | Free, 5,000 symbols |
| Zerodha | Rs 2,000/mo | Variable | Paid | 3,000 symbols |
| Upstox | Free | Not specified | Free | Limited |
| Angel One | Free | Not specified | Free | Available |
| ICICI Direct | Paid | Not specified | Paid | Limited |

**Note:** Fyers offers one of the most generous free API tiers among Indian brokers.

---

## Notes

1. **API is completely free** for Fyers account holders
2. **Rate limits apply per account**, not per app
3. **WebSocket subscriptions** have conflicting documentation (50/200/5,000 symbols)
4. **Test symbol limits** with your SDK version before production
5. **Daily limit increased 10x** in V3 update (10k → 100k)
6. **No usage dashboard** available (track manually)
7. **429 errors** indicate rate limit exceeded
8. **Use WebSocket** to minimize REST API calls
9. **API Bridge** is separate paid service for third-party integrations
10. **Contact support** for high-volume requirements or limit increase requests
