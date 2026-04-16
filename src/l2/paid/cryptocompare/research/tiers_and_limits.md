# CryptoCompare - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes (account at cryptocompare.com)
- API key required: Yes (free to create)
- Credit card required: No

### Rate Limits
- Requests per second: 50
- Requests per minute: 1,000
- Requests per hour: 150,000
- Requests per day: Not specified (based on hourly limit: ~3.6M theoretical)
- Burst allowed: Yes (short bursts above per-second limit tolerated)

### Data Access
- Real-time data: Yes (10-second cache on most price endpoints)
- Delayed data: No (data is real-time with cache)
- Historical data: Yes (depth: varies by data type)
  - Daily bars: Full history (years)
  - Hourly bars: Full history
  - Minute bars: 7 days
- WebSocket: Allowed (limits: 1-5 connections, not officially documented)
- Data types available:
  - Current prices (all exchanges + CCCAGG aggregate)
  - Historical OHLCV (daily, hourly, 7-day minute)
  - Top lists (by volume, market cap)
  - Exchange metadata
  - Coin lists
  - News (limited)
  - Social stats (limited)

### Limitations
- Symbols: Unlimited (access to all 5,700+ coins)
- Endpoints: Most available, some restricted:
  - News API: Limited results
  - Social API: Basic data only
  - Orderbook: Not available
  - Tick data: Not available
  - Extended minute history (>7 days): Not available
- Features restricted:
  - No Level 2 orderbook data
  - No raw tick/trade data download
  - Limited historical minute data
  - Attribution required (must credit CryptoCompare)
  - Community support only

## Paid Tiers

| Tier Name | Price | Rate Limit | Historical Minute | WebSocket | Orderbook | Support |
|-----------|-------|------------|-------------------|-----------|-----------|---------|
| Free | $0/mo | 50/s, 1k/min, 150k/hr | 7 days | Limited | No | Community |
| Starter | ~$80/mo | 300/min (estimated) | 30 days | 5+ conn | No | Email |
| Professional | ~$200/mo | Higher | 1 year | Unlimited | Yes | Priority |
| Enterprise | Custom | Custom (40k/s burst) | Unlimited | Unlimited | Yes | Dedicated |

**Note:** Exact pricing and tier names may vary. CryptoCompare doesn't publish detailed pricing publicly. Contact sales for current pricing.

### Upgrade Benefits

#### Starter Tier (~$80/mo)
- Higher rate limits (300+ req/min estimated)
- Extended minute data (30 days vs 7 days)
- More WebSocket connections (5+)
- Email support
- No attribution requirement
- Commercial use allowed

#### Professional Tier (~$200/mo)
- Higher rate limits (1000+ req/min estimated)
- 1 year of minute data
- Unlimited WebSocket connections
- Level 2 orderbook data (Channel 16)
- Priority support
- Advanced features
- Tick data access (possibly)

#### Enterprise Tier (Custom pricing)
- Custom rate limits (up to 40,000 req/sec burst documented)
- Unlimited historical data
- Raw tick/trade data
- Custom data feeds
- Dedicated support
- SLA guarantees
- White-label options
- Custom integrations
- On-premise deployment (possibly)

### What Unlocks at Each Tier?

**Starter → Professional:**
- Orderbook data (WebSocket Channel 16)
- Extended minute history (30 days → 1 year)
- Higher rate limits
- Priority support

**Professional → Enterprise:**
- Raw tick data
- Unlimited history
- Custom rate limits
- SLA guarantees
- Dedicated account manager

## Rate Limit Details

### How Measured
- Window: Per second, minute, and hour (three separate limits)
- Rolling window: Yes (60-second sliding window for minute limit, 3600-second for hour)
- Fixed window: No (rolling/sliding windows)

### Limit Scope
- Per IP address: No (limits are per API key)
- Per API key: Yes (primary limiting factor)
- Per account: No (each API key has independent limits)
- Shared across: All endpoints for a given API key

### Three-Tier Limiting
CryptoCompare uses three simultaneous rate limits:

1. **Per-Second Limit:** 50 requests/second
   - Prevents bursts that could overload servers
   - Short-term limiting

2. **Per-Minute Limit:** 1,000 requests/minute
   - Medium-term limiting
   - Most commonly hit limit

3. **Per-Hour Limit:** 150,000 requests/hour
   - Long-term limiting
   - ~41.67 requests/second sustained

**All three must be satisfied.** If any limit is exceeded, request is rejected.

### Burst Handling
- Burst allowed: Yes (short bursts above per-second limit tolerated)
- Burst size: ~10-20 requests (not officially documented)
- Burst window: 1-2 seconds
- Token bucket: Likely used internally, but not exposed

**Example:** You can send 60 requests in 1 second occasionally, but consistent 60/s will hit limit.

### Response Headers
CryptoCompare does NOT return standard HTTP rate limit headers.

**No headers like:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 850
X-RateLimit-Reset: 1706280060
```

### Checking Rate Limits
Use dedicated endpoint:
```bash
GET https://min-api.cryptocompare.com/stats/rate/limit?api_key=YOUR_API_KEY
```

**Response:**
```json
{
  "Response": "Success",
  "Message": "",
  "Data": {
    "calls_made": {
      "second": 12,
      "minute": 345,
      "hour": 8921
    },
    "calls_left": {
      "second": 38,
      "minute": 655,
      "hour": 141079
    }
  }
}
```

**Fields:**
- `calls_made.second`: Requests in current second window
- `calls_made.minute`: Requests in current 60-second window
- `calls_made.hour`: Requests in current 3600-second window
- `calls_left.*`: Remaining requests before hitting limit

### Error Response (HTTP 429 or Type 99)

When rate limit exceeded, you get:

**HTTP Status:** 200 (not 429) - CryptoCompare returns 200 with error in JSON

**Response:**
```json
{
  "Response": "Error",
  "Message": "You are over your rate limit please upgrade your account!",
  "HasWarning": false,
  "Type": 99,
  "RateLimit": {
    "calls_made": {
      "second": 51,
      "minute": 1005,
      "hour": 150100
    },
    "calls_left": {
      "second": 0,
      "minute": 0,
      "hour": 0
    }
  },
  "Data": {}
}
```

**Note:** Type 99 specifically indicates rate limit error.

### Handling Strategy

#### Recommended Approach
```javascript
async function fetchWithRateLimit(url) {
  const maxRetries = 3;
  let retryCount = 0;

  while (retryCount < maxRetries) {
    const response = await fetch(url);
    const data = await response.json();

    if (data.Response === 'Error' && data.Type === 99) {
      // Rate limit exceeded
      const waitTime = Math.min(1000 * Math.pow(2, retryCount), 60000); // Exponential backoff, max 60s
      console.log(`Rate limit hit. Waiting ${waitTime}ms...`);
      await sleep(waitTime);
      retryCount++;
      continue;
    }

    if (data.Response === 'Error') {
      throw new Error(data.Message);
    }

    return data;
  }

  throw new Error('Max retries exceeded');
}
```

#### Proactive Rate Limiting
```javascript
class RateLimiter {
  constructor(maxPerSecond, maxPerMinute, maxPerHour) {
    this.limits = {
      second: { max: maxPerSecond, window: 1000, calls: [] },
      minute: { max: maxPerMinute, window: 60000, calls: [] },
      hour: { max: maxPerHour, window: 3600000, calls: [] }
    };
  }

  async acquire() {
    const now = Date.now();

    for (const [period, limit] of Object.entries(this.limits)) {
      // Remove old calls outside window
      limit.calls = limit.calls.filter(t => now - t < limit.window);

      // Check if limit exceeded
      if (limit.calls.length >= limit.max) {
        const oldestCall = limit.calls[0];
        const waitTime = limit.window - (now - oldestCall);
        await sleep(waitTime);
        return this.acquire(); // Retry
      }
    }

    // Record call
    for (const limit of Object.values(this.limits)) {
      limit.calls.push(now);
    }
  }
}

// Usage
const limiter = new RateLimiter(50, 1000, 150000);

async function fetchPrice(fsym, tsym) {
  await limiter.acquire(); // Wait if needed
  return fetch(`https://min-api.cryptocompare.com/data/price?fsym=${fsym}&tsyms=${tsym}&api_key=${API_KEY}`);
}
```

#### Queue-Based Approach
```python
import time
from collections import deque

class RateLimiter:
    def __init__(self, max_per_second=50, max_per_minute=1000, max_per_hour=150000):
        self.limits = {
            'second': {'max': max_per_second, 'window': 1, 'calls': deque()},
            'minute': {'max': max_per_minute, 'window': 60, 'calls': deque()},
            'hour': {'max': max_per_hour, 'window': 3600, 'calls': deque()},
        }

    def wait_if_needed(self):
        now = time.time()
        max_wait = 0

        for period, limit in self.limits.items():
            # Remove old calls
            while limit['calls'] and now - limit['calls'][0] > limit['window']:
                limit['calls'].popleft()

            # Check if limit exceeded
            if len(limit['calls']) >= limit['max']:
                oldest = limit['calls'][0]
                wait_time = limit['window'] - (now - oldest)
                max_wait = max(max_wait, wait_time)

        if max_wait > 0:
            time.sleep(max_wait)

        # Record call
        now = time.time()
        for limit in self.limits.values():
            limit['calls'].append(now)
```

## Quota/Credits System

CryptoCompare does NOT use a credits/quota system. It uses pure rate limiting (requests per time period).

**Some providers use credits where different endpoints cost different amounts. CryptoCompare does not.**

## WebSocket Specific Limits

### Connection Limits
- Max connections per IP: Not documented (likely 5-10 for free tier)
- Max connections per API key: Varies by tier
  - Free: 1-5 connections (not officially documented)
  - Professional: Unlimited
  - Enterprise: Unlimited
- Max connections total: Per API key limit

### Subscription Limits
- Max subscriptions per connection: ~300 recommended (no hard limit documented)
- Max symbols per subscription: Not applicable (one subscription = one symbol pair)
- Practical limit: ~50-100 active subscriptions per connection for stable performance

### Message Rate Limits
- Messages per second: No specific limit on incoming messages
- Server may throttle: Yes (if too many subscriptions, updates may slow)
- Auto-disconnect on violation: Possible (not documented)
- Outgoing messages (subscriptions): No specific limit, but avoid spamming

### Connection Duration
- Max lifetime: 24 hours (recommended reconnect after this)
- Auto-reconnect needed: Yes (connections may drop after 24h or server maintenance)
- Idle timeout: ~60 seconds (no heartbeat may cause disconnect)
- Recommendation: Send WebSocket ping every 30-45 seconds

## Monitoring Usage

### Dashboard
- Usage dashboard: https://www.cryptocompare.com/cryptopian/api-keys
- Real-time tracking: No (dashboard shows quota usage, not real-time)
- Historical usage: Yes (can view past usage stats)

### API Endpoints
- Check quota: `GET /stats/rate/limit?api_key=YOUR_API_KEY`
- Check hourly: `GET /stats/rate/hour/limit?api_key=YOUR_API_KEY`
- Response format:
```json
{
  "Response": "Success",
  "Data": {
    "calls_made": {"second": 10, "minute": 150, "hour": 5000},
    "calls_left": {"second": 40, "minute": 850, "hour": 145000}
  }
}
```

### Alerts
- Email alerts: No automatic alerts from CryptoCompare
- Webhook: Not supported
- Recommendation: Implement your own monitoring
  - Poll `/stats/rate/limit` periodically
  - Alert when `calls_left` drops below threshold (e.g., 10% remaining)

### Monitoring Implementation
```javascript
async function monitorRateLimit(apiKey) {
  const response = await fetch(`https://min-api.cryptocompare.com/stats/rate/limit?api_key=${apiKey}`);
  const data = await response.json();

  if (data.Response === 'Success') {
    const limits = data.Data;

    // Check each limit
    for (const [period, values] of Object.entries(limits.calls_left)) {
      const totalLimit = limits.calls_made[period] + values;
      const percentRemaining = (values / totalLimit) * 100;

      if (percentRemaining < 10) {
        console.warn(`Rate limit warning: ${period} - ${percentRemaining.toFixed(1)}% remaining`);
        // Send alert (email, Slack, etc.)
      }
    }
  }
}

// Poll every 5 minutes
setInterval(() => monitorRateLimit(API_KEY), 5 * 60 * 1000);
```

## Comparison with Other Providers

| Provider | Free Tier Limit | Pricing | Notable |
|----------|----------------|---------|---------|
| CryptoCompare | 50/s, 1k/min, 150k/hr | ~$80-200/mo | Good free tier, attribution required |
| CoinGecko | 10-30/min | $129+/mo | Lower free tier |
| CoinMarketCap | 333/day (free) | $79+/mo | Very limited free |
| Binance API | 1200/min (direct) | Free | Exchange-specific, no aggregation |

## Best Practices

### Optimize API Usage
1. **Cache responses:** Price data cached for 10s server-side, cache client-side too
2. **Batch requests:** Use `pricemulti` instead of multiple `price` calls
3. **Use WebSocket:** For real-time data, use WebSocket instead of polling REST
4. **Request only what you need:** Don't request all symbols if you only need a few
5. **Monitor usage:** Track your rate limit usage proactively

### Avoid Rate Limits
1. **Implement client-side rate limiter** (see examples above)
2. **Use exponential backoff** on errors
3. **Don't retry immediately** on rate limit errors
4. **Spread requests** over time instead of bursts
5. **Consider upgrading tier** if consistently hitting limits

### When to Upgrade

Upgrade if:
- Hitting rate limits regularly (>5 times per day)
- Need more than 7 days of minute data
- Need orderbook data
- Commercial application (attribution removal)
- Need SLA guarantees
- Need dedicated support

## Summary Table

| Feature | Free | Starter | Professional | Enterprise |
|---------|------|---------|--------------|------------|
| **Rate Limit** | 50/s, 1k/min, 150k/hr | ~300/min | ~1k/min | Custom (40k/s burst) |
| **Minute History** | 7 days | 30 days | 1 year | Unlimited |
| **WebSocket Conn** | 1-5 | 5+ | Unlimited | Unlimited |
| **Orderbook** | No | No | Yes | Yes |
| **Tick Data** | No | No | Possible | Yes |
| **Support** | Community | Email | Priority | Dedicated |
| **Attribution** | Required | Not required | Not required | Not required |
| **SLA** | No | No | Possible | Yes |
| **Price** | $0 | ~$80/mo | ~$200/mo | Custom |

**Free tier is generous for hobbyists and development. Upgrade when you need commercial features or hit rate limits regularly.**
