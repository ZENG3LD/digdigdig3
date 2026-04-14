# Tiingo - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up**: Yes (free account at https://www.tiingo.com/)
- **API key required**: Yes (provided immediately upon registration)
- **Credit card required**: No

### Rate Limits

#### REST API Limits
- **Requests per minute**: 5
- **Requests per day**: 500
- **Requests per hour**: Not separately specified (derive from per-minute limit)
- **Symbols per hour**: 50
- **Burst allowed**: Not specified (likely minimal buffering)

#### WebSocket Limits
- **Connections**: Not explicitly limited (reasonable use expected)
- **Subscriptions per connection**: Unlimited (firehose provides all data)
- **Message rate**: Unlimited (receives full firehose at microsecond resolution)

### Data Access

#### Real-time Data
- **Real-time data**: Yes (via WebSocket and IEX REST endpoints)
- **Delayed data**: No delay for included data
- **WebSocket**: Allowed (1+ connections, full firehose access)

#### Historical Data
- **End-of-Day (EOD)**: Yes
  - **Depth**: 50+ years for US stocks
  - **Coverage**: 32,000+ US equities, 33,000+ ETFs/mutual funds
- **IEX Intraday**: Yes
  - **Depth**: Limited historical depth (not specified exactly)
  - **Granularity**: Minute-level or resampled
- **Crypto**: Yes
  - **Depth**: Historical depth varies by exchange/pair
  - **Exchanges**: 40+ exchanges, 2,100-4,100+ tickers
- **Forex**: Yes
  - **Depth**: Historical FX data available
  - **Pairs**: 140+ currency pairs
- **Fundamentals**: Yes (limited)
  - **Depth**: 5 years of history
  - **Coverage**: 5,500+ equities, 80+ indicators

#### Data Types
- End-of-Day stock prices (adjusted for splits/dividends)
- IEX real-time and intraday data
- Cryptocurrency prices (top-of-book and historical)
- Forex quotes (top-of-book and historical)
- Fundamentals (5 years: daily metrics, quarterly/annual statements)
- Financial news (curated feed)

### Limitations
- **Symbols**: No hard limit on symbol count (50 symbols/hour rate limit)
- **Endpoints**: All endpoints available (some features limited by tier)
- **Features**:
  - Fundamentals limited to 5 years (vs 15+ for paid tiers)
  - Lower rate limits (5/min vs up to 1,200/min for paid)
  - Daily request cap (500/day)
  - Symbol/hour limit (50/hour)

---

## Paid Tiers

Tiingo offers flexible pricing with three main tiers:

| Tier Name | Price | Rate Limit | WebSocket | Fundamentals History | Daily Cap | Support |
|-----------|-------|------------|-----------|---------------------|-----------|---------|
| **Free** | $0 | 5 req/min, 50 symbols/hr | Unlimited (firehose) | 5 years | 500/day | Community |
| **Launch** | $0 minimum (pay-as-you-go) | Higher limits (usage-based) | Unlimited | 15+ years | Usage-based | Email |
| **Grow** | $100/month minimum | Higher limits (up to 1,200/min) | Unlimited | 15+ years | No daily cap | Priority email |
| **Enterprise** | Custom pricing | Custom (very high) | Unlimited | 15+ years | No cap | Dedicated support |

### Tier Details

#### Free Tier ($0)
- **Rate Limit**: 5 requests/minute
- **Daily Limit**: 500 requests/day
- **Symbol Limit**: 50 symbols/hour
- **Fundamentals**: 5 years of history
- **WebSocket**: Full firehose access
- **Support**: Community forums, knowledge base
- **Use Case**: Personal projects, research, prototyping

#### Launch Tier ($0 minimum, pay-as-you-go)
- **Rate Limit**: Usage-based pricing (higher than free)
- **Daily Limit**: No hard cap (pay for what you use)
- **Fundamentals**: 15+ years of history
- **WebSocket**: Full firehose access
- **Support**: Email support
- **Use Case**: Growing applications, variable usage patterns
- **Pricing**: Flexible, pay only for requests made

#### Grow Tier ($100/month minimum)
- **Rate Limit**: Up to 1,200 requests/minute
- **Daily Limit**: No daily cap
- **Fundamentals**: 15+ years of history
- **WebSocket**: Full firehose access
- **Support**: Priority email support
- **Use Case**: Production applications, consistent high usage
- **Pricing**: Monthly minimum with additional usage charges

#### Enterprise Tier (Custom)
- **Rate Limit**: Custom (very high or unlimited)
- **Daily Limit**: No cap
- **Fundamentals**: 15+ years + custom data requests
- **WebSocket**: Full firehose + custom feeds if needed
- **Support**: Dedicated account manager, SLA
- **Additional Features**:
  - Custom data integrations
  - On-premise deployment options (if available)
  - Bulk data downloads
  - Redistribution rights (with appropriate licensing)
- **Use Case**: Institutional clients, data vendors, high-volume applications
- **Pricing**: Contact sales

### Upgrade Benefits

#### Free → Launch
- **Unlock**: Pay-as-you-go flexibility
- **Gain**: 10 more years of fundamentals (5yr → 15yr)
- **Remove**: Daily request cap (500/day → usage-based)
- **Support**: Community → Email

#### Free → Grow
- **Unlock**: 240x higher rate limit (5/min → 1,200/min)
- **Gain**: 10 more years of fundamentals (5yr → 15yr)
- **Remove**: Daily request cap (500/day → unlimited)
- **Remove**: Symbol/hour limit (50/hr → unlimited)
- **Support**: Community → Priority email

#### Grow → Enterprise
- **Unlock**: Custom rate limits (beyond 1,200/min)
- **Gain**: Custom data integrations, SLA guarantees
- **Gain**: Redistribution rights (if needed)
- **Support**: Priority email → Dedicated account manager

---

## Rate Limit Details

### How Measured
- **Window**: Per minute (60-second window)
- **Rolling window**: Likely yes (not explicitly documented)
- **Fixed window**: Possibly (implementation details not public)
- **Daily window**: Per day (midnight-to-midnight, likely UTC)

### Limit Scope
- **Per IP address**: Not specified (likely not the primary constraint)
- **Per API key**: Yes (rate limits tracked per API token)
- **Per account**: Yes (tied to account tier)
- **Shared across**: All REST endpoints share the same rate limit pool

### Burst Handling
- **Burst allowed**: Not explicitly specified (assume minimal buffering)
- **Burst size**: Not documented
- **Burst window**: Not documented
- **Token bucket**: Implementation details not public

### Response Headers

Tiingo includes rate limit information in HTTP response headers:

```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 3
X-RateLimit-Reset: 1234567890
```

**Header fields:**
- `X-RateLimit-Limit`: Total requests allowed in current window (e.g., 5 per minute)
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when rate limit resets

### Error Response (HTTP 429)

When rate limit exceeded:

**Status Code:** 429 Too Many Requests

**Headers:**
```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1234567890
Retry-After: 30
```

**Response Body:**
```json
{
  "detail": "Request was throttled. Expected available in 30 seconds."
}
```

**Fields:**
- `detail`: Human-readable error message
- May include seconds until reset

### Handling Strategy

#### Basic Rate Limiter (Python)

```python
import time
import requests

class RateLimiter:
    def __init__(self, requests_per_minute):
        self.rpm = requests_per_minute
        self.interval = 60.0 / requests_per_minute
        self.last_request = 0

    def wait_if_needed(self):
        elapsed = time.time() - self.last_request
        if elapsed < self.interval:
            time.sleep(self.interval - elapsed)
        self.last_request = time.time()

# Usage
limiter = RateLimiter(5)  # 5 requests per minute

for ticker in tickers:
    limiter.wait_if_needed()
    response = requests.get(f"https://api.tiingo.com/tiingo/daily/{ticker}")
```

#### Exponential Backoff (on 429 errors)

```python
import time
import requests

def get_with_backoff(url, headers, max_retries=5):
    for attempt in range(max_retries):
        response = requests.get(url, headers=headers)

        if response.status_code == 429:
            retry_after = int(response.headers.get('Retry-After', 60))
            wait_time = retry_after * (2 ** attempt)  # Exponential backoff
            print(f"Rate limited. Waiting {wait_time}s...")
            time.sleep(wait_time)
            continue

        return response

    raise Exception("Max retries exceeded")
```

#### Queue Requests (Advanced)

```python
import queue
import threading
import time
import requests

class TiingoQueue:
    def __init__(self, api_key, rpm=5):
        self.api_key = api_key
        self.rpm = rpm
        self.queue = queue.Queue()
        self.thread = threading.Thread(target=self._worker, daemon=True)
        self.thread.start()

    def _worker(self):
        interval = 60.0 / self.rpm
        while True:
            url, callback = self.queue.get()
            try:
                response = requests.get(
                    url,
                    headers={'Authorization': f'Token {self.api_key}'}
                )
                callback(response)
            except Exception as e:
                callback(e)
            finally:
                self.queue.task_done()
                time.sleep(interval)

    def get(self, url, callback):
        self.queue.put((url, callback))
```

---

## Quota/Credits System

**Not applicable** - Tiingo does not use a credits/quota system.

Pricing is based on:
- **Tier selection**: Choose tier (Free, Launch, Grow, Enterprise)
- **Rate limits**: Hard limits on requests per minute/day
- **Usage charges**: Pay-as-you-go on Launch tier (by request volume)
- **Monthly minimums**: $100/month for Grow tier

All endpoints have the same "cost" (one API request = one request toward rate limit).

---

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP**: Not explicitly documented
- **Max connections per API key**: Not explicitly documented
- **Max connections total**: Not documented (reasonable use expected)

**Note**: Tiingo emphasizes that all tiers (including Free) have full WebSocket access to the firehose.

### Subscription Limits
- **Max subscriptions per connection**: Unlimited
  - The firehose model provides all data for the endpoint (IEX, Forex, or Crypto)
  - ThresholdLevel parameter controls data volume (not a hard subscription limit)
- **Max symbols per subscription**: All symbols available in firehose
  - No explicit symbol limit on WebSocket
  - ThresholdLevel 5 = all top-of-book updates for all symbols

### Message Rate Limits
- **Messages per second**: No limit imposed by Tiingo
- **Server may throttle**: No (firehose delivers full stream)
- **Auto-disconnect on violation**: Not documented (reasonable use expected)

**Important**: The firehose can deliver **microsecond-resolution data** at extremely high message rates. Client systems must be built to handle this volume.

### Connection Duration
- **Max lifetime**: Not specified (likely 24 hours or until disconnect)
- **Auto-reconnect needed**: Yes (on any disconnect)
- **Idle timeout**: Not documented (heartbeat messages keep connection alive)

---

## Monitoring Usage

### Dashboard
- **Usage dashboard**: Likely available at https://www.tiingo.com/account/
- **Real-time tracking**: Not explicitly documented
- **Historical usage**: Likely available in account dashboard
- **Billing details**: Available for paid tiers

### API Endpoints

No documented API endpoints for checking usage/quota.

**Alternative**: Track usage client-side using rate limit headers:

```python
import requests

response = requests.get(
    "https://api.tiingo.com/tiingo/daily/AAPL",
    headers={"Authorization": "Token YOUR_API_KEY"}
)

limit = int(response.headers.get('X-RateLimit-Limit', 0))
remaining = int(response.headers.get('X-RateLimit-Remaining', 0))
reset_timestamp = int(response.headers.get('X-RateLimit-Reset', 0))

print(f"Used: {limit - remaining}/{limit}")
print(f"Resets at: {reset_timestamp}")
```

### Alerts
- **Email alerts**: Not explicitly documented
- **Webhook**: Not available
- **Manual monitoring**: Check response headers, track usage client-side

---

## Pricing Transparency

Tiingo emphasizes **pricing transparency**:
- Clear tier structure published on website
- Usage-limit tables clearly defined by plan
- No hidden fees or surprise charges
- Free tier with no credit card required
- Pay-as-you-go Launch tier for flexibility
- Monthly minimums clearly stated for Grow/Enterprise

**Pricing page**: https://www.tiingo.com/about/pricing

---

## Summary

### Free Tier
- **Rate Limit**: 5 req/min, 500 req/day, 50 symbols/hr
- **Cost**: $0
- **WebSocket**: Full firehose access (unlimited)
- **Fundamentals**: 5 years of history
- **Best For**: Personal projects, learning, prototyping

### Launch Tier
- **Rate Limit**: Higher than free (usage-based)
- **Cost**: $0 minimum (pay per request)
- **WebSocket**: Full firehose access
- **Fundamentals**: 15+ years
- **Best For**: Variable usage, growing applications

### Grow Tier
- **Rate Limit**: Up to 1,200 req/min, no daily cap
- **Cost**: $100/month minimum
- **WebSocket**: Full firehose access
- **Fundamentals**: 15+ years
- **Best For**: Production apps, consistent high usage

### Enterprise Tier
- **Rate Limit**: Custom (very high)
- **Cost**: Custom pricing
- **WebSocket**: Full firehose + custom feeds
- **Fundamentals**: 15+ years + custom data
- **Best For**: Institutional, data redistribution, very high volume

### Key Takeaways
- All tiers have **full WebSocket access** (microsecond firehose)
- Free tier is generous (5/min, 500/day, 50 symbols/hr)
- Rate limits scale dramatically with paid tiers (5/min → 1,200/min)
- No quota/credits system (simple rate limiting)
- Transparent pricing (no hidden fees)
- Fundamentals history increases with paid tiers (5yr → 15yr)
