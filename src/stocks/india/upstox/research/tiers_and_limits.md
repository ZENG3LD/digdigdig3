# Upstox - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up:** Yes (Upstox Demat account required)
- **API key required:** Yes (create app at developer portal)
- **Credit card required:** No for API creation, Yes for trading operations (account opening)

### Rate Limits
- **Requests per second:** 50 (standard APIs), 4 (multi-order APIs)
- **Requests per minute:** 500 (standard APIs), 40 (multi-order APIs)
- **Requests per 30 minutes:** 2000 (standard APIs), 160 (multi-order APIs)
- **Burst allowed:** Not explicitly documented

### Data Access
- **Real-time data:** Yes (via API and WebSocket)
- **Delayed data:** No
- **Historical data:** Yes
  - Daily bars: From year 2000
  - Intraday bars: From January 2022
- **WebSocket:** Allowed
  - Standard users: 2 connections max
  - Upstox Plus: 5 connections max
- **Data types:** All market data, quotes, OHLC, option chain, depth

### Limitations
- **Symbols:** Unlimited (all NSE, BSE, MCX instruments)
- **Endpoints:** All available with appropriate subscription
- **Features:** Trading requires paid subscription

---

## Paid Tiers

### Subscription Model

| Tier | Price | Description | Trading | Data Access | Support |
|------|-------|-------------|---------|-------------|---------|
| Free API | Rs 0 | API creation only | No | Public historical only | Community |
| Interactive API | Rs 499/month (GST incl.) | Full trading API | Yes | All data | Email |
| Historical API | Rs 499/month (GST incl.) | Historical data access | No | All historical | Email |
| Upstox Plus | Additional fee | Premium features | Yes | Enhanced | Priority |

### Special Promotional Offer (Valid till March 31, 2026)
- **Flat Rs 10/order** pricing via API
- Applies to all API users during promotional period
- Standard brokerage charges may apply after March 31, 2026

### Tier Comparison

**Free API (No Subscription):**
- Create API keys for free
- Access public historical data endpoints
- Download instrument files
- No trading operations
- Community forum support

**Interactive API (Rs 499/month):**
- All trading operations (place, modify, cancel orders)
- Multi-order APIs (beta)
- Real-time market data via REST and WebSocket
- Portfolio and position tracking
- P&L and charges APIs
- GTT orders support
- Webhook notifications
- Email support

**Historical API (Rs 499/month):**
- Historical candle data (v2 and v3)
- Intraday candle data
- OHLC historical data
- Multiple timeframes (1min to 1 month)
- Data from year 2000 (daily) and 2022 (intraday)
- No trading operations

**Upstox Plus:**
- Enhanced WebSocket limits (5 connections vs 2)
- Full D30 mode (30 depth levels instead of 5)
- Different instrument subscription limits
- Priority support
- Additional features

---

## Rate Limit Details

### How Measured
- **Window:** Per second, per minute, per 30 minutes
- **Rolling window:** Yes (sliding window)
- **Fixed window:** No

### Limit Scope
- **Per IP address:** Not primary limiting factor
- **Per API key:** Yes
- **Per user account:** Yes (per-API, per-user basis)
- **Shared across:** All API calls from same user/app

### Rate Limit Categories

#### Standard APIs
- **Per second:** 50 requests
- **Per minute:** 500 requests
- **Per 30 minutes:** 2000 requests

**Applies to:**
- Market data (quotes, LTP, OHLC)
- Historical data
- Portfolio endpoints
- Single order operations
- User profile and account info

#### Multi-Order APIs (Beta)
- **Per second:** 4 requests
- **Per minute:** 40 requests
- **Per 30 minutes:** 160 requests

**Applies to:**
- POST /v2/order/multi/place (batch order placement)
- DELETE /v2/order/multi/cancel (cancel all orders)
- DELETE /v2/portfolio/positions (exit all positions)

### Burst Handling
- **Burst allowed:** Not explicitly documented
- **Burst size:** Not specified
- **Burst window:** Not specified
- **Token bucket:** Likely (based on sliding window)

### Response Headers
**Note:** Documentation does not specify custom rate limit headers. Standard HTTP 429 response on rate limit exceeded.

**Expected headers (common pattern):**
```
X-RateLimit-Limit: 500
X-RateLimit-Remaining: 245
X-RateLimit-Reset: 1706251200
Retry-After: 30
```

### Error Response (HTTP 429)
```json
{
  "status": "error",
  "errors": [
    {
      "errorCode": "429",
      "message": "Rate limit exceeded",
      "propertyPath": null,
      "invalidValue": null,
      "error_code": "TOO_MANY_REQUESTS",
      "property_path": null,
      "invalid_value": null
    }
  ]
}
```

### Handling Strategy
- **Exponential backoff:** Recommended
- **Retry logic:** Implement with delays
- **Queue requests:** Use request queuing to stay within limits
- **Distribute calls:** Spread requests across time windows
- **Monitor usage:** Track request counts client-side

**Example retry logic:**
```python
import time

def call_api_with_retry(func, max_retries=3):
    for attempt in range(max_retries):
        try:
            return func()
        except RateLimitError as e:
            if attempt == max_retries - 1:
                raise
            wait_time = 2 ** attempt  # Exponential backoff: 1s, 2s, 4s
            time.sleep(wait_time)
```

---

## WebSocket Specific Limits

### Connection Limits

| User Type | Max Connections | Notes |
|-----------|-----------------|-------|
| Standard | 2 | Per API key |
| Upstox Plus | 5 | Per API key |

### Subscription Limits (Market Data Feed)

**Standard Users:**

| Mode | Max Instruments | Notes |
|------|-----------------|-------|
| LTPC | 5000 | Latest trade price & close |
| Option Greeks | 3000 | Greeks data |
| Full | 2000 | 5 depth levels + full data |
| Full D30 | 0 | Not available for standard |
| **Combined** | **2000** | Total across all modes |

**Upstox Plus Users:**

| Mode | Max Instruments | Notes |
|------|-----------------|-------|
| LTPC | 5000 | Latest trade price & close |
| Option Greeks | 3000 | Greeks data |
| Full | 2000 | 5 depth levels + full data |
| Full D30 | 50 | 30 depth levels (Plus exclusive) |
| **Combined (Full D30)** | **1500** | Limit for Full D30 mode |

### Message Rate Limits
- **Messages per second:** Not specified
- **Server may throttle:** Yes (automatic based on load)
- **Auto-disconnect on violation:** Not documented

### Connection Duration
- **Max lifetime:** Not specified (24 hours typical, no hard limit documented)
- **Auto-reconnect needed:** Yes (implement client-side)
- **Idle timeout:** Connection maintained via ping/pong (no idle timeout)

---

## API Availability

### Operating Hours
- **Available:** 5:30 AM to 12:00 AM IST (18.5 hours)
- **Unavailable:** 12:00 AM to 5:30 AM IST (5.5 hours)
- **Error code during downtime:** UDAPI100074

### Maintenance Windows
- Daily maintenance: 12:00 AM to 5:30 AM IST
- No trading or data access during this period
- Plan token refresh and system maintenance accordingly

---

## Monitoring Usage

### Dashboard
- **Usage dashboard:** https://account.upstox.com/developer/apps
- **Real-time tracking:** Not explicitly available
- **Historical usage:** Available in app dashboard
- **Features:**
  - API key management
  - App configuration
  - Token generation
  - Webhook settings

### API Endpoints
**No dedicated usage/quota endpoints documented.**

Developers must track usage client-side:
- Count requests per time window
- Implement local rate limiting
- Log API calls and responses
- Monitor 429 errors

### Alerts
- **Email alerts:** Not documented
- **Webhook:** Not for usage alerts (only for order/portfolio updates)
- **Custom alerts:** Implement client-side monitoring

---

## Quota/Credits System

**Not Applicable** - Upstox does not use a credit/quota system.

Rate limiting is based on:
- Requests per second/minute/30 minutes
- Per-API, per-user basis
- No credit deduction per request

---

## Cost Breakdown

### Monthly Costs

**API Subscription:**
- Interactive API: Rs 499/month (GST included)
- Historical API: Rs 499/month (GST included)
- Both APIs: Contact Upstox for bundle pricing

**Trading Costs (During Promotional Period till March 31, 2026):**
- **Flat Rs 10 per order** (via API)

**Trading Costs (Post-Promotion - Standard Upstox Rates):**
- Equity Delivery: Rs 20 or 2.5% (whichever is lower)
- Equity Intraday: Rs 20 or 2.5% (whichever is lower)
- F&O: Rs 20 or 2.5% (whichever is lower)
- Currency: Rs 20 or 2.5% (whichever is lower)
- Commodity: Rs 20 or 2.5% (whichever is lower)

**Additional Charges:**
- STT (Securities Transaction Tax)
- GST on brokerage
- Transaction charges (NSE/BSE/MCX)
- SEBI turnover fees
- Stamp duty
- DP charges (for delivery trades)

### Free Features
- API key creation
- Public historical data access
- Instrument file downloads
- Community forum access
- Documentation and examples

---

## Upgrade Benefits

### Free → Interactive API (Rs 499/month)
**Unlocks:**
- All trading operations
- Real-time market data REST APIs
- WebSocket market data feed
- Portfolio and positions tracking
- Order management (place, modify, cancel)
- GTT orders
- Multi-order APIs (beta)
- Webhook notifications
- Trade charges and P&L APIs

### Free → Historical API (Rs 499/month)
**Unlocks:**
- Full historical data access
- Intraday candle data
- Multiple timeframe support
- Higher resolution data
- Expanded units and intervals (v3 APIs)

### Standard → Upstox Plus
**Unlocks:**
- Increased WebSocket connections (2 → 5)
- Full D30 mode (30 depth levels)
- Changed subscription limits
- Priority support
- Enhanced trading features

---

## Rate Limit Enforcement

### Enforcement Policy
- Rate limits enforced on **per-API, per-user basis**
- Exceeding thresholds may result in:
  - HTTP 429 responses
  - Temporary suspension of API access
  - Account review for persistent violations

### Violations
- **First violation:** HTTP 429, retry after delay
- **Repeated violations:** Temporary suspension
- **Severe abuse:** Account suspension, contact support

### Fair Use Policy
- Rate limits designed to prevent system overload
- Ensure equitable access for all users
- Comply with limits to maintain service quality
- Contact support for higher limits (enterprise use)

---

## Enterprise/Custom Plans

For applications requiring:
- Higher rate limits
- Dedicated infrastructure
- Custom features
- Multiple client support
- Business-level SLA

**Contact:** Upstox Business API team
- Email: support@upstox.com
- Website: https://upstox.com/uplink-for-business/
- Product: Uplink Business API

**Features may include:**
- Custom rate limits
- Dedicated support
- Extended token validity
- White-label options
- Regulatory compliance support

---

## Comparison: Standard vs Plus Users

| Feature | Standard | Upstox Plus |
|---------|----------|-------------|
| API Subscription | Rs 499/month | Additional fee |
| WebSocket Connections | 2 | 5 |
| LTPC Instruments | 5000 | 5000 |
| Option Greeks Instruments | 3000 | 3000 |
| Full Mode Instruments | 2000 | 2000 |
| Full D30 Mode | Not available | 50 instruments |
| Combined Limit | 2000 | 1500 (Full D30) |
| Support | Email | Priority |
| Rate Limits (REST) | 50/s, 500/min | 50/s, 500/min |
| Rate Limits (Multi) | 4/s, 40/min | 4/s, 40/min |

---

## Summary

- **Flexible pricing:** Free API creation, paid subscriptions for trading/full data
- **Reasonable limits:** 50 req/s, 500 req/min for standard APIs
- **WebSocket limits:** Connection and subscription caps based on user type
- **Promotional offer:** Rs 10/order till March 31, 2026
- **No credit system:** Simple request-based rate limiting
- **Daily downtime:** 12:00 AM - 5:30 AM IST for maintenance
- **Monitor client-side:** No API for usage tracking, implement your own
- **Fair enforcement:** Per-API, per-user rate limiting
