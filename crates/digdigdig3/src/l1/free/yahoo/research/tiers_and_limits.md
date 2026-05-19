# yahoo - Tiers, Pricing, and Rate Limits

## Free Tier (Unofficial Direct Access)

### Access Level
- Requires sign-up: No
- API key required: No
- Credit card required: No

### Rate Limits
- Requests per second: No official limit (~5-10 safe)
- Requests per minute: No official limit (~100-200 safe)
- Requests per hour: ~2000 (community-observed)
- Requests per day: No official limit (IP-based throttling)
- Burst allowed: Yes (~10-20 requests rapid burst tolerated)

### Data Access
- Real-time data: Yes (15-20 second delay for some exchanges)
- Delayed data: Yes (some exchanges 15-minute delay)
- Historical data: Yes (depth: varies by symbol, typically decades)
- WebSocket: Allowed (No explicit connection limit, ~5-10 recommended)
- Data types: All public data (stocks, crypto, forex, commodities, indices, fundamentals, news)

### Limitations
- Symbols: Unlimited (all Yahoo Finance symbols)
- Endpoints: All available (no restrictions)
- Features: Full access to all unofficial endpoints
- Risk: IP blocking for excessive requests
- Stability: No SLA, endpoints may change without notice

## Paid Tiers (RapidAPI - Third-Party Proxy)

### RapidAPI Yahoo Finance1 API

| Tier Name | Price | Rate Limit | Requests/Month | Support | Notes |
|-----------|-------|------------|----------------|---------|-------|
| Free | $0 | 5 req/sec | 500 | Community | Good for testing |
| Basic | $10/mo | 5 req/sec | 10,000 | Email | Personal projects |
| Pro | $50/mo | 10 req/sec | 100,000 | Email | Small business |
| Ultra | $200/mo | 20 req/sec | 1,000,000 | Priority | Production use |
| Mega | Custom | Custom | Unlimited | Dedicated | Enterprise |

**Note:** Pricing may vary. Check RapidAPI for current rates: https://rapidapi.com/apidojo/api/yahoo-finance1

### Upgrade Benefits (RapidAPI)
- **New endpoints unlock:** No (same endpoints as free direct access)
- **New data available:** No (same data, just proxied)
- **Real-time vs delayed:** No change (still 15-20 sec delay where applicable)
- **Historical depth:** No change (same as direct access)
- **Main benefit:** Guaranteed rate limits, stable access, no IP blocks, official support

### RapidAPI vs Direct Access Comparison

| Feature | Direct (Free) | RapidAPI (Paid) |
|---------|---------------|-----------------|
| Rate limits | Unclear (~2000/hr) | Guaranteed (5-20 req/sec) |
| IP blocks | Possible | No blocks |
| Support | None | Email/Priority |
| Stability | Can break | More stable |
| Authentication | Cookie/crumb | Simple API key |
| Cost | Free | $10-200+/mo |
| Terms | Personal use only | Commercial allowed |
| Data | All public data | Same data |

## Rate Limit Details (Direct Access)

### How Measured
- Window: Per hour (rolling window)
- Rolling window: Yes (not fixed)
- Fixed window: No

### Limit Scope
- Per IP address: Yes (primary limiting factor)
- Per API key: N/A (no API keys)
- Per account: N/A (no accounts)
- Shared across: All requests from same IP share limit

### Burst Handling
- Burst allowed: Yes (short bursts tolerated)
- Burst size: ~10-20 requests
- Burst window: ~1-2 seconds
- Token bucket: Unknown (likely rate-based throttling)

### Response Headers
**No rate limit headers provided by Yahoo Finance!**

Direct access endpoints do NOT return rate limit headers like:
- ~~X-RateLimit-Limit~~
- ~~X-RateLimit-Remaining~~
- ~~X-RateLimit-Reset~~
- ~~Retry-After~~

You only know you hit the limit when you receive a 429 error.

### Error Response (HTTP 429)

**Plain Text Response:**
```
Too Many Requests
```

**HTTP Status:** 429 Too Many Requests

**No JSON, no details!** The response is just plain text.

**Sometimes includes:**
```
Edge: Too Many Requests
```

**Example Error Handling:**
```python
import requests

response = requests.get("https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL")

if response.status_code == 429:
    print("Rate limited!")
    print(response.text)  # "Too Many Requests\r\n"
    # Implement backoff
elif response.status_code == 200:
    data = response.json()
```

### Handling Strategy

**Recommended: Exponential Backoff**
```python
import time
import requests

def get_with_backoff(url, params=None, max_retries=5):
    for attempt in range(max_retries):
        response = requests.get(url, params=params)

        if response.status_code == 200:
            return response.json()
        elif response.status_code == 429:
            wait_time = (2 ** attempt) + random.uniform(0, 1)
            print(f"Rate limited, waiting {wait_time:.1f}s")
            time.sleep(wait_time)
        else:
            raise Exception(f"Error {response.status_code}: {response.text}")

    raise Exception("Max retries exceeded")
```

**Recommended: Request Throttling**
```python
import time
import requests

class ThrottledClient:
    def __init__(self, requests_per_second=2):
        self.min_interval = 1.0 / requests_per_second
        self.last_request_time = 0

    def get(self, url, params=None):
        # Wait if needed
        elapsed = time.time() - self.last_request_time
        if elapsed < self.min_interval:
            time.sleep(self.min_interval - elapsed)

        response = requests.get(url, params=params)
        self.last_request_time = time.time()

        return response

# Usage
client = ThrottledClient(requests_per_second=2)  # 2 requests/sec = safe rate
response = client.get("https://query1.finance.yahoo.com/v7/finance/quote?symbols=AAPL")
```

**Recommended: Queue with Rate Limiting**
```python
import time
import queue
import threading
import requests

class RateLimitedQueue:
    def __init__(self, requests_per_second=2):
        self.queue = queue.Queue()
        self.interval = 1.0 / requests_per_second
        self.results = {}
        self.worker = threading.Thread(target=self._process_queue, daemon=True)
        self.worker.start()

    def _process_queue(self):
        while True:
            if not self.queue.empty():
                request_id, url, params = self.queue.get()
                response = requests.get(url, params=params)
                self.results[request_id] = response
                time.sleep(self.interval)
            else:
                time.sleep(0.1)

    def add_request(self, request_id, url, params=None):
        self.queue.put((request_id, url, params))

    def get_result(self, request_id):
        while request_id not in self.results:
            time.sleep(0.1)
        return self.results.pop(request_id)
```

## Rate Limit Details (RapidAPI)

### How Measured
- Window: Per second (for req/sec limit) + monthly quota
- Rolling window: Yes (1-second rolling window)
- Fixed window: Monthly quota resets on billing date

### Limit Scope
- Per API key: Yes (each API key has own limits)
- Per IP address: No (rate limit tied to API key)
- Per account: Multiple API keys allowed

### Response Headers (RapidAPI)
```
X-RateLimit-Limit: 5
X-RateLimit-Remaining: 3
X-RateLimit-Requests-Limit: 10000
X-RateLimit-Requests-Remaining: 9847
```

### Error Response (RapidAPI 429)
```json
{
  "message": "You have exceeded the rate limit per second for your plan, BASIC, by the API provider"
}
```

## Quota/Credits System

**Not Applicable** - Yahoo Finance (direct or RapidAPI) does not use a credits/quota system. All rate limiting is request-based.

## WebSocket Specific Limits

### Connection Limits (Direct Access)
- Max connections per IP: Unknown (no official documentation)
- Max connections per API key: N/A (no API keys)
- Max connections total: Unknown
- **Community Recommendation:** 1-5 connections per IP to avoid blocks

### Subscription Limits
- Max subscriptions per connection: Unknown (tested up to 100+ symbols)
- Max symbols per subscription: Unlimited (can subscribe to array of any size)
- **Community Recommendation:** 50-100 symbols per connection for stability

### Message Rate Limits
- Messages per second: No official limit
- Server may throttle: Unknown (no documented throttling)
- Auto-disconnect on violation: No evidence of auto-disconnect
- **Community Recommendation:** Subscribe in batches, not all at once

### Connection Duration
- Max lifetime: Unlimited (can stay connected indefinitely)
- Auto-reconnect needed: No forced disconnects (but implement reconnection for reliability)
- Idle timeout: None observed (connection persists with ping/pong)

### WebSocket Rate Limit Best Practices

**Connection Management:**
```python
import websocket
import json
import time

class YahooWebSocketClient:
    def __init__(self, symbols, max_symbols_per_connection=50):
        self.symbols = symbols
        self.max_symbols_per_connection = max_symbols_per_connection
        self.connections = []

    def connect_all(self):
        # Split symbols across multiple connections if needed
        symbol_chunks = [
            self.symbols[i:i + self.max_symbols_per_connection]
            for i in range(0, len(self.symbols), self.max_symbols_per_connection)
        ]

        for chunk in symbol_chunks:
            ws = self._create_connection(chunk)
            self.connections.append(ws)
            time.sleep(1)  # Delay between connection creation

    def _create_connection(self, symbols):
        ws = websocket.WebSocketApp(
            "wss://streamer.finance.yahoo.com/?version=2",
            on_open=lambda ws: self._on_open(ws, symbols),
            on_message=self._on_message
        )
        # Start in separate thread
        threading.Thread(target=ws.run_forever, daemon=True).start()
        return ws

    def _on_open(self, ws, symbols):
        # Subscribe in batches
        batch_size = 10
        for i in range(0, len(symbols), batch_size):
            batch = symbols[i:i + batch_size]
            subscribe_msg = json.dumps({"subscribe": batch})
            ws.send(subscribe_msg)
            time.sleep(0.1)  # Small delay between batches

    def _on_message(self, ws, message):
        # Handle protobuf message
        pass
```

## Monitoring Usage

### Dashboard
- Usage dashboard: Not available (direct access)
- RapidAPI dashboard: Yes (for RapidAPI users) - https://rapidapi.com/developer/dashboard
- Real-time tracking: No (direct), Yes (RapidAPI)
- Historical usage: No (direct), Yes (RapidAPI)

### API Endpoints
**No usage monitoring endpoints available for direct access.**

RapidAPI provides usage stats through their dashboard only, not via API.

### Alerts
- Email alerts: No (direct access)
- RapidAPI alerts: Yes (at 80%, 100% quota usage)
- Webhook: No

## Community-Observed Rate Limits (Direct Access)

Based on community testing and reports:

| Metric | Observed Limit | Confidence | Source |
|--------|----------------|------------|--------|
| Requests/hour | ~2000 | High | GitHub issues, forums |
| Requests/minute | ~100-200 | Medium | User testing |
| Requests/second | ~5-10 burst | Medium | User testing |
| Daily requests | Unknown | Low | No consistent data |
| WebSocket connections/IP | ~5-10 | Low | Limited testing |
| WebSocket subscriptions | ~100+/conn | Medium | Library testing |

## Avoiding Rate Limits - Best Practices

### 1. Request Throttling
- Limit to 2 requests/second (safe rate)
- Add random jitter to avoid patterns
- Use sleep between requests

### 2. Caching
- Cache responses for at least 1 second
- Cache fundamentals for 1 hour+
- Cache historical data indefinitely

### 3. Batch Requests
- Use `symbols=AAPL,MSFT,GOOGL` for multiple quotes
- Use quoteSummary with multiple modules
- Minimize total request count

### 4. Use WebSocket for Real-Time
- Don't poll REST API repeatedly
- WebSocket has less strict limits
- One connection >> many REST requests

### 5. Respect 429 Errors
- Implement exponential backoff
- Don't retry immediately
- Wait at least 60 seconds after 429

### 6. Rotate IPs (if necessary)
- Use proxy rotation for heavy scraping
- Residential proxies work best
- Datacenter IPs more likely to be blocked

### 7. Mimic Browser Behavior
- Use realistic User-Agent
- Add Referer header
- Use cookie/session management
- Add random delays

### Example: Production-Ready Rate-Limited Client

```python
import time
import random
import requests
from datetime import datetime, timedelta

class YahooFinanceClient:
    def __init__(self, requests_per_second=2, enable_cache=True):
        self.min_interval = 1.0 / requests_per_second
        self.last_request_time = 0
        self.session = requests.Session()
        self.session.headers.update({
            "User-Agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36",
            "Referer": "https://finance.yahoo.com/"
        })
        self.cache = {} if enable_cache else None
        self.cache_ttl = 60  # 60 seconds

    def _throttle(self):
        elapsed = time.time() - self.last_request_time
        if elapsed < self.min_interval:
            jitter = random.uniform(0, 0.2)
            time.sleep(self.min_interval - elapsed + jitter)
        self.last_request_time = time.time()

    def _get_cached(self, cache_key):
        if not self.cache:
            return None
        if cache_key in self.cache:
            data, timestamp = self.cache[cache_key]
            if time.time() - timestamp < self.cache_ttl:
                return data
        return None

    def _set_cache(self, cache_key, data):
        if self.cache is not None:
            self.cache[cache_key] = (data, time.time())

    def get_quote(self, symbols):
        cache_key = f"quote_{symbols}"
        cached = self._get_cached(cache_key)
        if cached:
            return cached

        self._throttle()

        url = "https://query1.finance.yahoo.com/v7/finance/quote"
        params = {"symbols": symbols}

        for attempt in range(3):
            try:
                response = self.session.get(url, params=params, timeout=10)

                if response.status_code == 200:
                    data = response.json()
                    self._set_cache(cache_key, data)
                    return data
                elif response.status_code == 429:
                    wait = (2 ** attempt) * 30
                    print(f"Rate limited, waiting {wait}s")
                    time.sleep(wait)
                else:
                    raise Exception(f"Error {response.status_code}")

            except requests.exceptions.RequestException as e:
                print(f"Request failed: {e}")
                time.sleep(2 ** attempt)

        raise Exception("Max retries exceeded")

# Usage
client = YahooFinanceClient(requests_per_second=2, enable_cache=True)
quote = client.get_quote("AAPL,MSFT")
```

## Summary

| Aspect | Direct Access | RapidAPI |
|--------|---------------|----------|
| Cost | Free | $0-200+/mo |
| Rate Limit | ~2000/hr (uncertain) | 5-20 req/sec (guaranteed) |
| Stability | Can break | More stable |
| IP Blocks | Possible | No |
| Support | None | Email/Priority |
| Commercial Use | Prohibited | Allowed |
| Best For | Personal projects, testing | Production, commercial |

**Recommendation:**
- **Personal/Educational:** Direct access with proper rate limiting
- **Production/Commercial:** RapidAPI or similar paid proxy service
- **Heavy Usage:** Implement caching, throttling, and WebSocket where possible
