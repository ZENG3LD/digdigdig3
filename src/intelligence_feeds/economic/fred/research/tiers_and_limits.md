# FRED - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes (free FRED account at https://fredaccount.stlouisfed.org/)
- API key required: Yes (obtained after free registration)
- Credit card required: No - completely free service

### Rate Limits
- Requests per second: ~2 (120/60)
- Requests per minute: 120
- Requests per hour: Not explicitly limited (theoretical max: 7,200)
- Requests per day: Not explicitly limited (theoretical max: 172,800)
- Burst allowed: Not documented - appears to be fixed 120/min window

### Data Access
- Real-time data: Yes (data appears shortly after official release times, typically within minutes to hours)
- Delayed data: No delay beyond official government release schedules
- Historical data: Yes (extensive depth - some series from 1700s, most from 1900s onward)
- WebSocket: No - not available
- Data types: Full access to all 840,000+ economic time series

### Limitations
- Symbols: Unlimited - access to all series in FRED database
- Endpoints: All 30 endpoints available
- Features: Full API access with no feature restrictions

**There are NO limitations on data access in the free tier - everything is available.**

## Paid Tiers

**FRED API has NO paid tiers.**

| Tier Name | Price | Rate Limit | Additional Features | WebSocket | Historical | Support |
|-----------|-------|------------|---------------------|-----------|------------|---------|
| Free | $0 | 120/min | All features | No | Unlimited | Documentation only |

### Why No Paid Tiers?

FRED is a public service provided by the Federal Reserve Bank of St. Louis as part of its educational and research mission. The data is funded by taxpayers and intended for public use.

### Upgrade Benefits

N/A - no upgrades available or needed.

### Commercial Use Restrictions

While the API is free, **commercial use is restricted**:

1. **Non-commercial use**: Freely allowed for:
   - Educational purposes
   - Personal use
   - Academic research
   - Non-profit organizations

2. **Commercial use**: Requires special permission
   - Must contact Federal Reserve Bank of St. Louis
   - May require licensing agreement
   - Third-party data in FRED may have additional restrictions

3. **Prohibited uses** (from 2024 Terms of Use update):
   - Training AI/ML models (including LLMs)
   - Caching/storing/archiving FRED data
   - Redistributing FRED data to third parties
   - Wholesale downloading
   - Replicating FRED website functionality

## Rate Limit Details

### How Measured
- Window: Per minute
- Rolling window: Not documented (likely rolling)
- Fixed window: Not documented (implementation unclear)
- Measurement: 120 requests in any 60-second period

### Limit Scope
- Per IP address: No
- Per API key: Yes - each API key has independent 120/min limit
- Per account: No - multiple keys allowed
- Shared across: Each API key is independent

### Burst Handling
- Burst allowed: Not documented
- Burst size: Unknown - assume none
- Burst window: N/A
- Token bucket: Unknown implementation

**Recommendation**: Assume strict 120 requests per 60-second window with no burst allowance.

### Response Headers

FRED **does NOT provide rate limit headers** in API responses.

No headers like:
- X-RateLimit-Limit
- X-RateLimit-Remaining
- X-RateLimit-Reset
- Retry-After (even on 429 errors)

**You must implement client-side rate limiting.**

### Error Response (HTTP 429)

When rate limit is exceeded:

**JSON format:**
```json
{
  "error_code": 429,
  "error_message": "Too Many Requests. Rate limit exceeded."
}
```

**XML format:**
```xml
<?xml version="1.0" encoding="utf-8" ?>
<error code="429" message="Too Many Requests. Rate limit exceeded."/>
```

**Note**: The exact error message is not officially documented - this is typical HTTP 429 behavior.

### Handling Strategy

**Exponential backoff: Recommended**
```python
import time

def fetch_with_backoff(url, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url)
        if response.status_code == 429:
            wait_time = (2 ** attempt) * 1  # 1s, 2s, 4s
            time.sleep(wait_time)
            continue
        return response
    raise Exception("Rate limit exceeded after retries")
```

**Rate limiter: Recommended**
```python
import time
from collections import deque

class FREDRateLimiter:
    def __init__(self, requests_per_minute=120):
        self.rpm = requests_per_minute
        self.requests = deque()

    def acquire(self):
        now = time.time()
        # Remove requests older than 60 seconds
        while self.requests and self.requests[0] < now - 60:
            self.requests.popleft()

        # If at limit, wait
        if len(self.requests) >= self.rpm:
            sleep_time = 60 - (now - self.requests[0])
            if sleep_time > 0:
                time.sleep(sleep_time + 0.1)  # Small buffer

        self.requests.append(time.time())

# Usage
limiter = FREDRateLimiter(118)  # Leave 2 req buffer
limiter.acquire()
response = requests.get(url)
```

**Queue requests:**
```python
import queue
import threading
import time

class FREDRequestQueue:
    def __init__(self, rpm=118):
        self.queue = queue.Queue()
        self.rpm = rpm
        self.interval = 60.0 / rpm
        self.worker = threading.Thread(target=self._process_queue, daemon=True)
        self.worker.start()

    def _process_queue(self):
        while True:
            request_func = self.queue.get()
            request_func()
            time.sleep(self.interval)

    def submit(self, func):
        self.queue.put(func)
```

## Quota/Credits System (if applicable)

**FRED does NOT use a quota or credits system.**

- No monthly quotas
- No credit costs per request
- No overage charges (service is free)
- Unlimited requests subject only to 120/min rate limit

## WebSocket Specific Limits

N/A - WebSocket not supported.

## Monitoring Usage

### Dashboard
- Usage dashboard: No official dashboard available
- Real-time tracking: No
- Historical usage: No

**You must track your own usage client-side.**

### API Endpoints
- Check quota: No endpoint available
- Check limits: No endpoint available
- Response format: N/A

**There is no way to programmatically check your usage or remaining quota.**

### Alerts
- Email alerts: No
- Webhook: No
- No built-in alerting system

## Workarounds for Rate Limits

### Multiple API Keys

Since each API key has independent 120/min limit:

1. **Request multiple keys** for the same application
2. **Distribute requests** across keys using round-robin
3. **Effective rate**: N keys × 120 req/min

**Example with 3 keys:**
```python
class MultiKeyFRED:
    def __init__(self, api_keys):
        self.keys = api_keys
        self.current = 0
        self.limiters = {key: FREDRateLimiter() for key in api_keys}

    def get_key(self):
        key = self.keys[self.current]
        self.current = (self.current + 1) % len(self.keys)
        return key

    def request(self, endpoint, params):
        key = self.get_key()
        self.limiters[key].acquire()
        params['api_key'] = key
        return requests.get(endpoint, params=params)
```

### Caching Strategy

Since FRED data doesn't change frequently:

1. **Cache responses** with appropriate TTL:
   - Daily series: Cache for 24 hours
   - Monthly series: Cache for 7-30 days
   - Quarterly series: Cache for 30-90 days
   - Annual series: Cache for 180-365 days

2. **Use /fred/series/updates** to invalidate cache intelligently

3. **Store historical data** locally (but note Terms of Use restrictions on archiving)

**Example cache:**
```python
import time
from functools import lru_cache

class FREDCache:
    def __init__(self, ttl_seconds=3600):
        self.cache = {}
        self.ttl = ttl_seconds

    def get(self, key):
        if key in self.cache:
            data, timestamp = self.cache[key]
            if time.time() - timestamp < self.ttl:
                return data
        return None

    def set(self, key, data):
        self.cache[key] = (data, time.time())
```

### Batch Requests Efficiently

- Use limit=100000 for /series/observations to get max data in one request
- Fetch multiple series in parallel (up to 120/min)
- Use /category/series to discover all series in category at once

## Best Practices

1. **Conservative rate limiting**: Use 118 req/min, not 120 (leave buffer)
2. **Implement client-side tracking**: FRED provides no rate limit info
3. **Cache aggressively**: Economic data changes infrequently
4. **Use multiple keys for high-volume**: If you need >120 req/min
5. **Batch operations**: Fetch large date ranges in single requests
6. **Monitor errors**: Track 429 responses to adjust rate limiting
7. **Respect Terms of Use**: Don't bulk download, cache excessively, or train AI models

## Requesting Rate Limit Increase

According to documentation: "If you have a reason that you need to exceed the limit, you can contact them."

**Contact method**: Not explicitly provided - likely through:
- FRED contact form on website
- Email to Federal Reserve Bank of St. Louis

**Likelihood of approval**: Unknown - use multiple API keys instead for higher throughput.

## Summary

| Feature | Status |
|---------|--------|
| Free tier | Yes - 100% free |
| Paid tiers | No |
| Rate limit | 120 req/min per API key |
| Rate limit headers | No |
| Usage dashboard | No |
| Burst allowance | Unknown/No |
| Multiple keys | Yes - independent limits |
| Commercial use | Restricted - requires permission |
| AI training | Prohibited |
| Data caching | Prohibited (per ToS) |
| Wholesale download | Prohibited |
