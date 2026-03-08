# JQuants - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: Yes
- API key required: Yes (refresh token + ID token)
- Credit card required: No

### Rate Limits
- Requests per second: ~0.08 (5 per minute)
- Requests per minute: **5**
- Requests per hour: 300 (5/min sustained)
- Requests per day: 7,200 (5/min sustained)
- Burst allowed: Not documented (likely strict 5/min)

### Data Access
- Real-time data: No
- Delayed data: Yes (**12-week delay** on all data)
- Historical data: Yes (2 years with 12-week delay)
- WebSocket: No (not available on any tier)
- Data types available:
  - Listed Issue Master (2 years, delayed 12 weeks)
  - Stock Prices/OHLC (2 years, delayed 12 weeks)
  - Financial Data (2 years, delayed 12 weeks)
  - Earnings Calendar (recent data only)
  - Trading Calendar (2 years, delayed 12 weeks)

### Limitations
- Symbols: Unlimited (all TSE listed stocks)
- Endpoints: Most basic endpoints available, but:
  - No morning/afternoon session prices
  - No trading by investor type
  - No TOPIX/indices data
  - No derivatives (futures/options)
  - No margin trading data
  - No short selling data
  - No breakdown trading data
  - No minute bars or tick data
- Features: 12-week delay makes it unsuitable for current analysis
- Account expiry: Free plan expires after **1 year** and auto-cancels (must re-register)

## Paid Tiers

| Tier Name | Price (JPY/month) | Price (USD/month est.) | Rate Limit | Historical Depth | Additional Features | Support |
|-----------|-------------------|------------------------|------------|------------------|---------------------|---------|
| Free | ¥0 | $0 | 5 req/min | 2 years (12w delay) | Basic endpoints only | Community/Docs |
| Light | ¥1,650 | ~$11 | 60 req/min | 5 years | No delay on basic data | Email |
| Standard | ¥3,300 | ~$22 | 120 req/min | 10 years | + Indices, Trading by Type, Margin, Short Selling | Email |
| Premium | ¥16,500 | ~$110 | 500 req/min | All periods (from 2008) | + Derivatives, Breakdown, Morning/Afternoon, Dividends | Priority Email |

**Currency conversion**: Approximate based on 1 USD = 150 JPY (rates vary)

**Billing**: Monthly subscription, auto-renewal

### Tier-Specific Endpoint Access

#### Free Tier Endpoints (12-week delay)
- ✅ `/v1/listed/info` - Listed issue master
- ✅ `/v1/prices/daily_quotes` - Daily OHLC (basic)
- ✅ `/v1/fins/statements` - Financial statements
- ✅ `/v1/fins/announcement` - Earnings calendar
- ✅ `/v1/markets/trading_calendar` - Trading calendar

#### Light Tier Adds (no delay)
- ✅ Same endpoints as Free, but **no 12-week delay**
- ✅ 5 years of historical data

#### Standard Tier Adds
- ✅ 10 years of historical data
- ✅ `/v1/indices` - TOPIX and other indices
- ✅ `/v1/markets/trading_by_type` - Trading by investor type
- ✅ `/v1/markets/margin` - Margin trading outstanding
- ✅ `/v1/markets/short_selling` - Short sale data
- ✅ `/v1/option/index_option` - Index options
- ✅ `/v1/listed/info` - Margin code/name fields unlocked

#### Premium Tier Adds
- ✅ All historical data (from May 2008 onwards)
- ✅ `/v1/derivatives/futures` - Futures prices
- ✅ `/v1/derivatives/options` - Options prices
- ✅ `/v1/markets/breakdown` - Detail breakdown trading
- ✅ `/v1/prices/daily_quotes` - Morning/afternoon session fields
- ✅ `/v1/fins/dividend` - Cash dividend data
- ✅ `/v1/fins/statements` - Full BS/PL/CF data

### Add-on Plans

**Minute Bars & Tick Data Add-on** (January 2026 release)

| Add-on | Price | Rate Limit | Description |
|--------|-------|------------|-------------|
| Stock Prices (minute-OHLC, Tick) | TBD | 60 req/min | Minute-level and tick-level historical data |

- **Independent rate limit**: 60 req/min separate from base plan
- **Availability**: Works with any base plan (Free, Light, Standard, Premium)
- **Access**: Via `/v2/prices/bars/minute` and `/v2/prices/ticks` endpoints
- **Data delay**: TBD (likely matches base plan delay for Free tier)

## Rate Limit Details

### How Measured
- Window: Per minute
- Rolling window: Not documented (assumed fixed 1-minute windows)
- Fixed window: Likely (resets every 60 seconds)

### Limit Scope
- Per IP address: No
- Per API key: **Yes**
- Per account: Yes (same as per API key)
- Shared across: All endpoints (except add-on APIs which have separate 60/min limit)

### Burst Handling
- Burst allowed: Not documented (assume no burst)
- Burst size: N/A
- Burst window: N/A
- Token bucket: Not documented

### Response Headers

JQuants does **NOT** provide rate limit headers in responses:
- ❌ No `X-RateLimit-Limit`
- ❌ No `X-RateLimit-Remaining`
- ❌ No `X-RateLimit-Reset`
- ❌ No `Retry-After` (on 429 error)

**Implication**: Client must track rate limits locally.

### Error Response (HTTP 429)

**Status Code**: 429 Too Many Requests

**Body**: (exact format not documented, likely JSON or plain text)
```json
{
  "error": "Rate limit exceeded"
}
```

### Escalating Penalties

**Warning from docs**:
> "If you significantly exceed the rate limit and continue making requests, access may be completely blocked for approximately **5 minutes**."

**Strategy**:
- Respect rate limits strictly
- Implement exponential backoff on 429
- Stop requests immediately on 429 to avoid 5-minute block

### Handling Strategy

**Recommended approach**:

```python
import time
from collections import deque

class RateLimiter:
    def __init__(self, requests_per_minute):
        self.limit = requests_per_minute
        self.requests = deque()

    def wait_if_needed(self):
        now = time.time()

        # Remove requests older than 1 minute
        while self.requests and self.requests[0] < now - 60:
            self.requests.popleft()

        # If at limit, wait
        if len(self.requests) >= self.limit:
            sleep_time = 60 - (now - self.requests[0]) + 0.1  # +0.1s safety
            time.sleep(sleep_time)
            self.requests.popleft()

        self.requests.append(time.time())

# Usage
limiter = RateLimiter(5)  # Free tier: 5 req/min

for _ in range(100):
    limiter.wait_if_needed()
    response = requests.get(url, headers=headers)

    if response.status_code == 429:
        print("Rate limited! Waiting 5 minutes...")
        time.sleep(300)  # Wait 5 minutes to avoid block
```

**Exponential backoff**:
```python
import time

def request_with_retry(url, headers, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url, headers=headers)

        if response.status_code == 429:
            # Exponential backoff: 60s, 120s, 240s
            wait_time = 60 * (2 ** attempt)
            print(f"Rate limited. Waiting {wait_time}s...")
            time.sleep(wait_time)
            continue

        return response

    raise Exception("Max retries exceeded")
```

## Quota/Credits System (if applicable)

JQuants does **NOT** use a credits/quota system. It uses:
- **Fixed rate limits** (requests per minute)
- **No per-endpoint costs**
- **No monthly quota**

All endpoints cost the same (1 request = 1 request against your limit).

## Monitoring Usage

### Dashboard
- Usage dashboard: Available in user portal at https://jpx-jquants.com/en
- Real-time tracking: Not documented (likely shows usage stats)
- Historical usage: Not documented

### API Endpoints
- Check quota: **No dedicated endpoint**
- Check limits: **No API to check remaining requests**
- Response format: N/A

**Implication**: Cannot query current usage via API; must track client-side.

### Alerts
- Email alerts: Not documented
- Webhook: No
- Account suspension: Free plan auto-expires after 1 year

## Upgrade Benefits

### Free → Light (¥1,650/month)
- **Data delay**: 12 weeks → **No delay**
- **Rate limit**: 5/min → **60/min** (12x increase)
- **Historical depth**: 2 years → **5 years**
- **New endpoints**: None (same endpoints, just faster and current data)
- **Use case**: Current data for research and backtesting

### Light → Standard (¥3,300/month)
- **Rate limit**: 60/min → **120/min** (2x increase)
- **Historical depth**: 5 years → **10 years**
- **New endpoints**:
  - TOPIX and indices
  - Trading by investor type
  - Margin trading data
  - Short selling data
  - Index options
- **Use case**: Institutional-grade analysis, market structure research

### Standard → Premium (¥16,500/month)
- **Rate limit**: 120/min → **500/min** (4.2x increase)
- **Historical depth**: 10 years → **All available** (from May 2008)
- **New endpoints**:
  - Futures and options (derivatives)
  - Breakdown trading data
  - Morning/afternoon session prices
  - Cash dividend data
  - Full financial statement data (BS/PL/CF)
- **Use case**: Professional quants, algo trading, comprehensive market analysis

## Best Practices for Rate Limits

1. **Know your tier**: Hard-code rate limit in your client
   ```rust
   const RATE_LIMIT: u32 = 5; // Free tier
   const RATE_LIMIT: u32 = 60; // Light tier
   const RATE_LIMIT: u32 = 120; // Standard tier
   const RATE_LIMIT: u32 = 500; // Premium tier
   ```

2. **Implement client-side limiting**: Don't rely on 429 responses
   - Use token bucket or sliding window
   - Add 10% safety margin (e.g., treat 60/min as 54/min)

3. **Batch requests efficiently**:
   - Use date-based queries to get all symbols at once
   - Avoid per-symbol iteration
   ```python
   # Bad: 1 request per symbol (wastes rate limit)
   for code in ["7203", "6758", "9984"]:
       data = get_daily_quotes(code=code)

   # Good: 1 request for all symbols on a date
   data = get_daily_quotes(date="2024-01-15")  # Returns all symbols
   ```

4. **Respect update schedules**:
   - Don't poll before data updates (see update timing below)
   - Daily prices update at 16:30 JST; no need to poll before

5. **Use CSV bulk downloads** (when available):
   - For historical data backfills, use CSV downloads (free for rate limit)
   - Save API requests for incremental daily updates

6. **Handle 429 gracefully**:
   - Stop all requests immediately
   - Wait at least 60 seconds (or 300 seconds to avoid 5-min block)
   - Log and alert on rate limit hits

## Data Update Timing (Avoid Wasted Requests)

| Data Type | Update Time (JST) | Recommendation |
|-----------|-------------------|----------------|
| Daily stock prices | 16:30 | Don't poll before 16:30 |
| Morning session | 12:00 | Don't poll before 12:00 |
| Financial statements | 18:00 / 24:30 | Poll twice: after 18:00 and 00:30 |
| Indices | 16:30 | Don't poll before 16:30 |
| Futures/Options | 27:00 (3:00 AM) | Don't poll before 03:00 |
| Trading by investor type | Thu 18:00 | Poll once per week on Thursday evening |
| Margin trading | Tue 16:30 | Poll once per week on Tuesday |
| Earnings calendar | ~19:00 | Poll once daily in evening |

**Efficiency tip**: Schedule polling based on update times to maximize data freshness while minimizing wasted requests.

## CSV Bulk Downloads (January 2026 Feature)

### Availability
- **Plans**: Likely available on all paid tiers (TBD)
- **Access method**: Via user portal or SFTP (J-Quants Pro)
- **Rate limit impact**: **None** (downloads don't count against API rate limit)

### Use Cases
- Historical data backfills
- Initial database population
- Bulk analysis without API calls

### Recommendation
- Use CSV for historical data (e.g., load last 5 years)
- Use API for daily incremental updates
- This maximizes efficiency and stays within rate limits

## Add-on Rate Limits (Minute/Tick Data)

### Separate Limit Pool
- **Base plan limit**: 5-500/min (depends on tier)
- **Add-on limit**: **60/min** (separate)
- **Total effective**: Base limit + 60/min for add-on endpoints

**Example (Free + Add-on)**:
- Base endpoints: 5/min
- Minute/tick endpoints: 60/min additional
- Can make 5 regular requests + 60 minute bar requests in same minute

**Example (Premium + Add-on)**:
- Base endpoints: 500/min
- Minute/tick endpoints: 60/min additional
- Minute bars are separate pool, so don't consume your 500/min quota

### Add-on Endpoints
- `/v2/prices/bars/minute` - 60/min limit
- `/v2/prices/ticks` - 60/min limit

## J-Quants Pro (Enterprise)

For corporate users, J-Quants Pro offers:
- **Delivery methods**: REST API + CSV/SFTP + Snowflake integration
- **Rate limits**: Custom (contact sales)
- **Pricing**: Custom (contact sales)
- **Data**: Richer datasets, additional breakdowns
- **Support**: Dedicated support

**Comparison**:
| Feature | J-Quants API | J-Quants Pro |
|---------|--------------|--------------|
| Target | Individuals | Corporations |
| Pricing | Fixed tiers | Custom |
| Delivery | REST API only | API + SFTP + Snowflake |
| Rate limits | 5-500/min | Custom |
| Bulk data | CSV downloads (2026) | SFTP bulk delivery |

## Recommendations for V5 Connector

1. **Default to Free tier settings**: Assume 5/min unless configured otherwise
2. **Make tier configurable**: Allow user to specify tier in config
3. **Implement robust rate limiting**: Client-side token bucket with safety margin
4. **Batch efficiently**: Prefer date-based queries over per-symbol iteration
5. **Handle 429 with backoff**: Stop requests on 429, wait 60-300s
6. **Log rate limit hits**: Alert user when approaching limits
7. **Respect update schedules**: Don't poll before data is available
8. **Consider CSV for backfills**: Use API for daily updates only
