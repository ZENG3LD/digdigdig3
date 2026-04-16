# AlphaVantage - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up**: Yes (email registration)
- **API key required**: Yes (provided upon sign-up)
- **Credit card required**: No

### Rate Limits
- **Requests per second**: 0.08 (5 requests per minute = 1 per 12 seconds)
- **Requests per minute**: 5
- **Requests per hour**: 300 (theoretical max if sustained)
- **Requests per day**: 25 (hard daily cap)
- **Burst allowed**: No - strict 5 per minute enforcement

### Data Access
- **Real-time data**: Delayed 15 minutes for US stocks (or end-of-day)
- **Delayed data**: Yes - 15 minute delay for US markets
- **Historical data**: Yes (up to 20+ years for most assets)
- **WebSocket**: Not available (no WebSocket at any tier)
- **Data types available**:
  - ✅ Daily/weekly/monthly time series (stocks, forex, crypto)
  - ✅ Global quotes (current price, delayed for US)
  - ✅ Forex daily/weekly/monthly
  - ✅ Crypto daily/weekly/monthly
  - ✅ Most technical indicators (except VWAP, MACD)
  - ✅ Fundamental data (company overview, financials, earnings)
  - ✅ Economic indicators (GDP, CPI, unemployment, etc.)
  - ✅ Commodities data
  - ✅ News sentiment (limited)
  - ❌ **Intraday data** (premium only)
  - ❌ **Adjusted intraday** (premium only)
  - ❌ **Real-time US stocks** (premium only)
  - ❌ **Real-time options** (premium only)
  - ❌ **VWAP indicator** (premium only)
  - ❌ **MACD indicator** (premium only)

### Limitations
- **Symbols**: Unlimited (can query any supported symbol)
- **Endpoints**: Some restricted (intraday, real-time US, options require premium)
- **Output size**: `compact` only (100 data points max per request)
- **Features restricted**:
  - No intraday intervals (1min, 5min, 15min, 30min, 60min)
  - No adjusted intraday data
  - No real-time US market data
  - No historical options data
  - No VWAP or MACD indicators
  - Limited news sentiment access
  - Cannot use `outputsize=full` effectively (daily limit too low)

### Daily Limit Calculation
```
25 requests/day ÷ 5 requests/minute = 5 minutes of continuous use
Or: 1 request every 58 minutes for 24 hours
```

**Practical usage**:
- Portfolio of 5 stocks: Update 5 times per day
- Portfolio of 25 stocks: Update once per day
- Single stock monitoring: Update every hour (24 requests/day)

---

## Premium Tiers

### Monthly Plans

| Tier | Rate Limit (req/min) | Monthly Price | Annual Price (save 2 months) |
|------|---------------------|---------------|----------------------------|
| **Plan 15** | 75 | $49.99 | $499 ($41.58/month) |
| **Plan 60** | 150 | $99.99 | $999 ($83.25/month) |
| **Plan 120** | 300 | $149.99 | $1,499 ($124.92/month) |
| **Plan 360** | 600 | $199.99 | $1,999 ($166.58/month) |
| **Plan 600** | 1,200 | $249.99 | $2,499 ($208.25/month) |
| **Enterprise** | Custom/Unlimited | Contact Sales | Contact Sales |

### Premium Features (All Tiers)

**All premium plans include**:
- ✅ **No daily limits** (only per-minute rate limits)
- ✅ **All premium features unlocked**:
  - Intraday data (1min, 5min, 15min, 30min, 60min)
  - Real-time US stock market data (NASDAQ-licensed)
  - 15-minute delayed US market data
  - Adjusted time series (splits, dividends)
  - Real-time US options data
  - Historical options data (15+ years)
  - Extended hours data (pre-market, after-hours)
  - VWAP technical indicator
  - MACD technical indicator
  - Full outputsize (complete historical datasets)
  - Bulk quotes API (up to 100 symbols)
  - Extended news sentiment access
- ✅ **Premium support** (priority email support)
- ✅ **Cancel anytime** - no lock-in
- ✅ **No hidden costs**
- ✅ **No overage charges** (requests blocked after limit, not charged extra)

### Tier Selection Guide

| Use Case | Recommended Tier | Rationale |
|----------|-----------------|-----------|
| Small portfolio (5-10 stocks) | Plan 15 (75/min) | 75 updates/min = 4,500/hour sufficient |
| Medium portfolio (50+ stocks) | Plan 60 (150/min) | 9,000 updates/hour for active monitoring |
| Large portfolio (100+ stocks) | Plan 120 (300/min) | 18,000 updates/hour |
| Trading application | Plan 360 (600/min) | 36,000 updates/hour for responsive UI |
| Multi-tenant platform | Plan 600 (1200/min) | 72,000 updates/hour for many users |
| Institutional/HFT | Enterprise | Unlimited or custom rate limits |

### Enterprise Options

**Need more than 1,200 requests per minute?**

Alpha Vantage supports:
- **Unlimited requests per minute**
- **Custom pricing**
- **Dedicated support**
- **SLA guarantees** (likely)
- **Custom data feeds**
- **Bulk historical data access**

**Contact**: Sales team via website for custom quote

---

## Rate Limit Details

### How Measured

- **Window**: Per minute (60-second rolling window)
- **Rolling window**: Yes - counts requests in last 60 seconds
- **Fixed window**: No - not reset at fixed intervals

### Limit Scope

- **Per IP address**: No
- **Per API key**: Yes - rate limits tied to API key
- **Per account**: Yes - same as API key
- **Shared across**: Not applicable (one key per account typically)

### Burst Handling

- **Burst allowed**: No - strict enforcement
- **Burst size**: N/A
- **Burst window**: N/A
- **Token bucket**: Not documented (likely simple rate counter)

**Recommendation**: Distribute requests evenly to avoid hitting limits.

### Example Rate Limit Scenario (Free Tier)

```
Time 00:00 - Request 1 ✅
Time 00:12 - Request 2 ✅
Time 00:24 - Request 3 ✅
Time 00:36 - Request 4 ✅
Time 00:48 - Request 5 ✅
Time 00:50 - Request 6 ❌ Rate limit error (5 requests in last 60 seconds)
Time 01:01 - Request 6 ✅ (Request 1 dropped out of 60-second window)
```

### Example Rate Limit Scenario (Plan 15 - 75/min)

```
Time 00:00 - Requests 1-75 (rapid burst) ✅
Time 00:05 - Request 76 ❌ Rate limit error (75 requests in last 60 seconds)
Time 01:00 - Requests 76-150 ✅ (all previous requests dropped out of window)
```

### Response Headers

**AlphaVantage does NOT provide rate limit headers.**

No headers like:
- ❌ `X-RateLimit-Limit`
- ❌ `X-RateLimit-Remaining`
- ❌ `X-RateLimit-Reset`
- ❌ `Retry-After` (even on 429 error)

**Client must track rate limits manually.**

### Error Response (Rate Limit Exceeded)

**HTTP Status**: 200 OK (not 429)

**Body** (per-minute limit):
```json
{
  "Note": "Thank you for using Alpha Vantage! Our standard API call frequency is 5 calls per minute. Please visit https://www.alphavantage.co/premium/ if you would like to target a higher API call frequency."
}
```

**Body** (daily limit - free tier):
```json
{
  "Note": "Thank you for using Alpha Vantage! You have reached the daily limit of 25 API requests. Please try again tomorrow or visit https://www.alphavantage.co/premium/ to upgrade."
}
```

**Important**: Error is in response body, NOT HTTP status code.

### Handling Strategy

#### Client-Side Rate Limiting

```python
import time
from collections import deque

class AlphaVantageRateLimiter:
    def __init__(self, requests_per_minute=5):
        self.requests_per_minute = requests_per_minute
        self.request_times = deque()

    def wait_if_needed(self):
        now = time.time()

        # Remove requests older than 60 seconds
        while self.request_times and self.request_times[0] < now - 60:
            self.request_times.popleft()

        # If at limit, wait until oldest request expires
        if len(self.request_times) >= self.requests_per_minute:
            sleep_time = 60 - (now - self.request_times[0]) + 0.1
            time.sleep(sleep_time)
            self.wait_if_needed()  # Recursive check

        # Record this request
        self.request_times.append(time.time())

# Usage
limiter = AlphaVantageRateLimiter(requests_per_minute=5)

for symbol in ['IBM', 'AAPL', 'MSFT', 'GOOGL', 'AMZN']:
    limiter.wait_if_needed()
    response = requests.get(f'https://www.alphavantage.co/query?function=GLOBAL_QUOTE&symbol={symbol}&apikey={API_KEY}')
    # Process response...
```

#### Exponential Backoff (on Error)

```python
import time
import requests

def make_request_with_retry(url, max_retries=3):
    for attempt in range(max_retries):
        response = requests.get(url)
        data = response.json()

        # Check if rate limit error
        if 'Note' in data and 'call frequency' in data['Note']:
            wait_time = 60 * (2 ** attempt)  # Exponential: 60s, 120s, 240s
            print(f"Rate limit hit. Waiting {wait_time} seconds...")
            time.sleep(wait_time)
            continue

        return data

    raise Exception("Max retries exceeded")
```

#### Queue System

For production applications:
1. **Request queue** with rate limiter
2. **Background worker** processing queue
3. **Response cache** to minimize API calls
4. **Prioritization** for critical requests

---

## Quota/Credits System

**Not applicable** - AlphaVantage uses rate limits, not credits/quota system.

- No credits per request
- No monthly quota allocation
- Simple time-based rate limiting only

---

## WebSocket Specific Limits

**Not applicable** - AlphaVantage does not support WebSocket.

---

## Monitoring Usage

### Dashboard

- **Usage dashboard**: Yes (account dashboard on alphavantage.co)
- **Real-time tracking**: Likely yes (check dashboard for current usage)
- **Historical usage**: Likely yes (view past usage patterns)

**Access**: Login to your account at alphavantage.co

### API Endpoints

**No API endpoints for checking usage** - must use web dashboard.

- ❌ No `GET /account/usage`
- ❌ No `GET /account/limits`
- ❌ No programmatic usage tracking

### Alerts

- **Email alerts**: Not documented (check dashboard settings)
- **Webhook**: No
- **Recommended**: Implement client-side monitoring and alerts

---

## Historical Rate Limit Changes

AlphaVantage has tightened free tier limits over time:

| Period | Free Tier Daily Limit | Notes |
|--------|----------------------|-------|
| Early years | 500 requests/day | Very generous |
| Mid period | 100 requests/day | Still reasonable |
| **Current (2026)** | **25 requests/day** | Significant restriction |

**Trend**: Free tier becoming more restrictive, pushing users toward premium.

**Per-minute limit**: Consistently 5 requests/minute for free tier.

---

## Cost Analysis

### Free Tier Value
- **Cost**: $0
- **Requests per month**: 750 (25/day × 30 days)
- **Cost per 1000 requests**: $0

**Good for**:
- Learning and experimentation
- Low-frequency portfolio tracking
- Academic projects
- Proof of concept

**Not good for**:
- Production applications
- Real-time monitoring
- Intraday data needs
- High-frequency updates

### Premium Tier Value

**Plan 15 ($49.99/month)**:
- **Requests per month**: ~3,240,000 (75/min × 60 min × 24 hr × 30 days)
- **Cost per 1000 requests**: $0.015
- **Unlock**: Intraday data, real-time US stocks, options

**Plan 600 ($249.99/month)**:
- **Requests per month**: ~51,840,000 (1200/min × 60 min × 24 hr × 30 days)
- **Cost per 1000 requests**: $0.0048
- **Best value** for high-volume applications

### Comparison with Alternatives

| Provider | Free Tier | Entry Premium | Real-time Data | Intraday | WebSocket |
|----------|-----------|---------------|----------------|----------|-----------|
| **AlphaVantage** | 25/day | $49.99/mo | Premium only | Premium only | No |
| **Polygon.io** | 5 calls/min | $29/mo | Yes (premium) | Yes (premium) | Yes |
| **Finnhub** | 60 calls/min | $0 | Delayed | Limited free | Yes |
| **IEX Cloud** | Closed 2026 | - | - | - | - |
| **Twelve Data** | 800/day | $8/mo | Premium only | Premium only | Yes |

**AlphaVantage strengths**:
- Comprehensive fundamental data
- Economic indicators
- 50+ technical indicators
- Multi-asset coverage (stocks, forex, crypto, commodities)
- 20+ years historical data
- NASDAQ-licensed (regulatory compliant)

**AlphaVantage weaknesses**:
- No WebSocket
- Restrictive free tier (25/day)
- Premium required for intraday
- No real-time streaming

---

## Recommendations

### For Free Tier Users

1. **Cache aggressively** - store responses, reuse data
2. **Batch requests** - use REALTIME_BULK_QUOTES when available
3. **Off-peak updates** - update portfolio once daily
4. **Limit symbols** - focus on 10-25 most important symbols
5. **Use outputsize=compact** - free tier restriction anyway
6. **Mix with other free sources** - combine with other APIs
7. **Consider upgrading** - if hitting limits regularly

### For Premium Tier Selection

1. **Calculate required requests**:
   - Symbols × Updates per hour × Hours per day = Requests per day
   - Divide by 60 to get req/min needed
2. **Add 50% buffer** for burst capacity
3. **Consider growth** - choose tier with headroom
4. **Start small** - upgrade as needed (no lock-in)

### For Cost Optimization

1. **Implement intelligent caching** (5-60 minute TTL based on data type)
2. **Use bulk endpoints** (REALTIME_BULK_QUOTES for 100 symbols)
3. **Prioritize requests** (real-time for watchlist, delayed for others)
4. **Deduplicate requests** (multiple clients requesting same symbol)
5. **Rate limit client-side** (avoid wasted requests on errors)

---

## Summary Table

| Aspect | Free Tier | Premium (Plan 15) | Premium (Plan 600) | Enterprise |
|--------|-----------|-------------------|-------------------|------------|
| **Per-minute limit** | 5 | 75 | 1,200 | Custom/Unlimited |
| **Daily limit** | 25 | Unlimited | Unlimited | Unlimited |
| **Price/month** | $0 | $49.99 | $249.99 | Contact Sales |
| **Intraday data** | No | Yes | Yes | Yes |
| **Real-time US** | No | Yes | Yes | Yes |
| **Options data** | No | Yes | Yes | Yes |
| **Outputsize full** | Limited | Yes | Yes | Yes |
| **Best for** | Learning | Small app | Production | Enterprise |
