# KRX - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes (mandatory)
- API key required: Yes
- Credit card required: No
- Approval process: Yes (up to 1 business day)
- Cost: Free

### Rate Limits

#### Public Data Portal (data.go.kr)
- Requests per second: Not specified
- Requests per minute: Not specified
- Requests per hour: Not specified
- Requests per day: **100,000**
- Burst allowed: Yes (exact limit not documented)
- Window type: Fixed daily window (resets at 00:00 KST)

#### KRX Data Marketplace (data.krx.co.kr)
- Requests per second: Not publicly documented
- Requests per minute: Not publicly documented
- Requests per hour: Not publicly documented
- Requests per day: Not publicly documented
- Burst allowed: Unknown
- **Note:** Rate limits exist but are not transparently published

### Data Access

- Real-time data: **No** (delayed by 1+ business day)
- Delayed data: **Yes** (delay: +1 business day minimum)
- Historical data: **Yes** (depth: from listing date, varies by stock)
- WebSocket: **No** (not available)
- Intraday data: **No** (daily granularity only)
- Data types available:
  - Daily OHLCV (Open, High, Low, Close, Volume)
  - Stock ticker lists
  - Market capitalization
  - Trading value by investor type
  - Sector/industry information
  - Index data (KOSPI, KOSDAQ, KRX100, etc.)
  - Basic company information
  - Short selling data
  - Foreign ownership data

### Limitations

- Symbols: Unlimited (all listed stocks on KOSPI, KOSDAQ, KONEX)
- Endpoints: Most available, some may require service-specific approval
- Features: No real-time data, daily updates only
- Historical range per request: Typically 1-5 years (varies by endpoint)
- Data format: JSON, CSV, XML (depending on endpoint)
- Update timing: 1:00 PM KST on business day following reference date

## Paid Tiers

### KRX Data Marketplace - Commercial Packages

KRX offers commercial data packages through their Data Marketplace. Pricing and features are not publicly listed and require direct contact with KRX Data Sales Division.

**Known Commercial Offerings:**

| Tier | Price | Rate Limit | Additional Features | Real-Time | Historical | Support |
|------|-------|------------|---------------------|-----------|------------|---------|
| Free | $0 | Limited | Basic delayed data | No | Yes (daily) | Community |
| Commercial Basic | Contact KRX | Higher | Extended data sets | Depends | Yes | Email |
| Commercial Premium | Contact KRX | Custom | Full market data | Possible | Unlimited | Priority |
| Institutional Feed | Contact KRX | No limit | Direct exchange feed | Yes | Unlimited | Dedicated |

### Third-Party Provider Pricing

Since KRX doesn't offer real-time public API, third-party providers fill the gap:

#### ICE Data Services
- Pricing: Contact for quote
- Real-time: Yes (Level 1, Level 2)
- Historical: Yes (extensive)
- WebSocket: Yes
- API: Yes

#### Twelve Data
- Free: 8 API requests/minute
- Basic: $29/month - 800 requests/day
- Grow: $79/month - 20,000 requests/day
- Pro: $129/month - 66,000 requests/day
- Real-time: Available (with WebSocket)
- Historical: Available

#### TickData
- Pricing: Custom quotes based on data requirements
- Real-time: Yes
- Historical tick data: Yes
- Institutional grade

### Upgrade Benefits

**From Free to Commercial (KRX direct):**
- Increased rate limits (exact numbers not public)
- Possible real-time or reduced delay access
- Bulk data access methods
- Priority API support
- Custom data feeds
- Historical depth increases
- Additional data fields

**Third-Party Benefits:**
- Real-time streaming (WebSocket)
- Global data access (not just KRX)
- Better documentation
- Multiple API protocols
- SDK support
- Guaranteed SLAs

## Rate Limit Details

### How Measured

#### Public Data Portal
- Window: Daily (24-hour period)
- Rolling window: No
- Fixed window: Yes (resets at 00:00 KST)
- Scope: Per API key

#### Data Marketplace
- Window: Unknown (likely per-minute or per-day)
- Rolling window: Unknown
- Fixed window: Unknown
- **Documentation:** Not publicly available

### Limit Scope

- Per IP address: Possible (not confirmed)
- Per API key: **Yes** (primary limiting factor)
- Per account: Possible for commercial tiers
- Shared across: All endpoints using same API key

### Burst Handling

- Burst allowed: Likely yes (typical for most APIs)
- Burst size: Not documented
- Burst window: Not documented
- Token bucket: Unknown implementation
- **Recommendation:** Implement client-side rate limiting to be safe

### Response Headers

#### Public Data Portal (Observed)
```http
X-RateLimit-Limit: 100000
X-RateLimit-Remaining: 95432
X-RateLimit-Reset: 1706572800 (Unix timestamp)
```

**Note:** Not all endpoints return these headers consistently.

#### Data Marketplace
Rate limit headers **not observed** in public API responses. Rate limiting may be:
- Silent (requests blocked without explanation)
- HTTP 429 responses
- HTTP 503 Service Unavailable

### Error Response (HTTP 429)

**Expected format (standard):**
```json
{
  "error": "Rate limit exceeded",
  "message": "Too many requests",
  "limit": 100000,
  "remaining": 0,
  "reset": 1706572800,
  "retry_after": 3600
}
```

**Actual KRX behavior:** Not well-documented. May return:
- HTTP 429 with minimal message
- HTTP 503 during high load
- Connection timeouts

### Handling Strategy

```python
import time
import requests
from datetime import datetime, timedelta

class KRXRateLimiter:
    """Conservative rate limiter for KRX API"""

    def __init__(self, requests_per_minute=50):
        """
        Args:
            requests_per_minute: Conservative estimate (50/min = 3000/hour)
        """
        self.requests_per_minute = requests_per_minute
        self.min_interval = 60.0 / requests_per_minute
        self.last_request = datetime.min

    def wait_if_needed(self):
        """Enforce rate limit with buffer"""
        now = datetime.now()
        time_since_last = (now - self.last_request).total_seconds()

        if time_since_last < self.min_interval:
            sleep_time = self.min_interval - time_since_last
            time.sleep(sleep_time)

        self.last_request = datetime.now()

    def make_request(self, func, *args, **kwargs):
        """Wrapper with exponential backoff"""
        max_retries = 5
        base_delay = 1

        for attempt in range(max_retries):
            self.wait_if_needed()

            try:
                response = func(*args, **kwargs)

                if response.status_code == 429:
                    retry_after = int(response.headers.get('Retry-After', base_delay * (2 ** attempt)))
                    print(f"Rate limited. Waiting {retry_after} seconds...")
                    time.sleep(retry_after)
                    continue

                return response

            except requests.exceptions.RequestException as e:
                if attempt == max_retries - 1:
                    raise
                wait_time = base_delay * (2 ** attempt)
                time.sleep(wait_time)

        raise Exception("Max retries exceeded")
```

## Quota/Credits System

### Not Applicable

KRX does not use a credit/quota system. Rate limiting is based on:
- Request count per time window
- Daily/monthly caps (for commercial tiers)
- No per-endpoint cost differentiation observed

## WebSocket Specific Limits

### Not Applicable

KRX public API does not support WebSocket connections.

For third-party WebSocket providers:
- See provider-specific documentation
- Typical limits: 5-100 connections per account
- Subscription limits: 50-1000 symbols per connection

## Monitoring Usage

### Dashboard

#### Public Data Portal
- Usage dashboard: https://www.data.go.kr/ (after login)
- Real-time tracking: Yes (shows remaining quota)
- Historical usage: Yes (daily/monthly statistics)
- Alert system: Email alerts available

#### KRX Data Marketplace
- Usage dashboard: https://openapi.krx.co.kr/ (My Page section)
- Real-time tracking: Unknown
- Historical usage: Likely available in portal
- Alert system: Not documented

### API Endpoints

**No programmatic usage check endpoints documented.**

Possible approaches:
- Parse rate limit headers from responses
- Track usage client-side
- Contact KRX support for commercial usage monitoring

### Alerts

- Email alerts: Available through Public Data Portal
- Webhook: Not available
- SMS: Not available
- At X% usage: Configurable in portal (data.go.kr)

## Comparison with Other Providers

| Feature | KRX Free | KRX Commercial | ICE | Twelve Data |
|---------|----------|----------------|-----|-------------|
| Price | Free | Contact | Contact | $29-129/mo |
| Real-time | No | Maybe | Yes | Yes |
| Rate limit (day) | 100k | Custom | Unlimited* | 800-66k |
| WebSocket | No | No | Yes | Yes |
| Historical depth | Full | Full | Full | Full |
| Data delay | +1 day | Varies | <1s | <1s |
| Support | Community | Priority | Dedicated | Email |

*Subject to fair use policies

## Important Considerations

### Data Freshness Limitation

**CRITICAL:** Even with no rate limits, KRX public API data is delayed by a minimum of 1 business day.

- Reference date: 2026-01-20 (Monday)
- Data available: 2026-01-21 1:00 PM KST
- Delay: ~24+ hours

**Implication:** Rate limits are less critical than data delay for most use cases.

### Use Case Recommendations

| Use Case | Recommended Tier | Notes |
|----------|------------------|-------|
| Historical backtesting | Free tier | Sufficient |
| Research/Analysis | Free tier | 100k/day is generous |
| Daily portfolio tracking | Free tier | Works fine |
| Real-time trading | Third-party provider | KRX API unsuitable |
| High-frequency data | Third-party provider | Need WebSocket |
| Institutional trading | Direct KRX feed | Exchange membership required |
| Multi-market coverage | Third-party aggregator | Global data access |

### Rate Limit Best Practices

1. **Implement client-side limiting**
   - Don't rely on server enforcement
   - Use conservative estimates (50 req/min safe)

2. **Batch requests when possible**
   - Use date ranges instead of single-day queries
   - Fetch multiple symbols in one request (if supported)

3. **Cache aggressively**
   - Data only updates once daily
   - No need to re-fetch same day's data

4. **Use OTP download for bulk data**
   - More efficient than repeated API calls
   - Better for initial data loads

5. **Respect update schedule**
   - Don't query before 1:00 PM KST
   - No point querying multiple times per day

## Contact for Commercial Tiers

**KRX Data Sales Division:**
- Website: https://data.krx.co.kr/
- Email: Available through contact form on website
- Phone: Listed on KRX website (Korean language)
- Process: Submit inquiry → Quote → Contract → Access

**Typical Commercial Questions to Ask:**
- What rate limits apply?
- Is real-time or reduced-delay data available?
- What is the pricing structure?
- Are there volume discounts?
- What additional data fields are available?
- Is WebSocket/streaming available?
- What SLA guarantees exist?
- What is the contract term?
