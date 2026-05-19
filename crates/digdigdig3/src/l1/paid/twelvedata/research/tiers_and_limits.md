# Twelvedata - Tiers, Pricing, and Rate Limits

## Free Tier (Basic Plan)

### Access Level
- Requires sign-up: **Yes** (email registration)
- API key required: **Yes** (automatically generated on signup)
- Credit card required: **No** (completely free)

### Rate Limits
- Requests per second: ~0.13 (8 per minute / 60 seconds)
- **Requests per minute: 8**
- Requests per hour: 480 (8/min × 60 min)
- **Requests per day: 800** (hard daily cap)
- Burst allowed: Not documented (likely no burst allowance)

### Data Access
- Real-time data: **Delayed/Limited** (not true real-time)
- Delayed data: **Yes** (delays vary by asset type)
- Historical data: **Yes** (limited depth - typically 1-2 years for intraday)
- WebSocket: **No** (Pro plan minimum required)
- Data types available:
  - ✓ Time series (OHLCV)
  - ✓ Quote data
  - ✓ Price data
  - ✓ EOD data
  - ✓ Reference data (stocks, forex, crypto lists)
  - ✓ Basic technical indicators
  - ✗ Fundamentals (requires Grow+ plan)
  - ✗ Market movers (requires Pro+ plan)
  - ✗ Extended hours data (requires Pro+ plan)

### Limitations
- Symbols: **Unlimited** (can query any supported symbol)
- Endpoints: **Some restricted** (fundamentals, market movers, exchange schedule unavailable)
- Features:
  - Max 5000 data points per request
  - Max 30 default outputsize (can increase to 5000)
  - No WebSocket streaming
  - No pre/post-market data
  - No analyst ratings
  - No financial statements
  - Community support only

## Paid Tiers

| Tier Name | Price (Monthly) | Calls/Min | Calls/Day | WebSocket Credits | Historical Depth | Support | Key Features |
|-----------|-----------------|-----------|-----------|-------------------|------------------|---------|--------------|
| **Basic** | **$0** | **8** | **800** | **0** | 1-2 years | Community | Basic data only |
| **Grow** | $29 | 55 | ~79,000 | 0 | 5+ years | Email | + Fundamentals |
| **Grow** | $49 | 144 | ~207,000 | 0 | 5+ years | Email | + More credits |
| **Grow** | $79 | 377 | ~543,000 | 0 | 5+ years | Email | + Even more |
| **Pro** | $99 | 610 | ~878,000 | 8 | Unlimited | Priority | + Real-time, WebSocket |
| **Pro** | $149 | 987 | ~1.42M | 16 | Unlimited | Priority | + More WS credits |
| **Pro** | $249 | 1,597 | ~2.3M | 32 | Unlimited | Priority | + Even more |
| **Ultra** | $329 | 2,584 | ~3.72M | 2,500 | Unlimited | Dedicated | + Premium features |
| **Ultra** | $499 | 4,181 | ~6.02M | 4,000 | Unlimited | Dedicated | + More capacity |
| **Ultra** | $999 | 8,361 | ~12.04M | 8,000 | Unlimited | Dedicated | + High volume |
| **Ultra** | $1,999 | 16,721 | ~24.08M | 16,000 | Unlimited | Dedicated | + Enterprise-grade |
| **Enterprise** | Contact Sales | Custom | Custom | Custom | Unlimited | Dedicated | Fully customized |

### Tier Progression Summary

**Basic → Grow**: Unlocks fundamentals (financials, earnings, dividends)
**Grow → Pro**: Unlocks real-time data, WebSocket, market movers, extended hours
**Pro → Ultra**: Unlocks exchange schedules, FIGI/Composite FIGI, massive credit increase
**Ultra → Enterprise**: Custom contracts, SLAs, dedicated support, on-premise options

### Upgrade Benefits

#### Grow Plan Unlocks
- **Fundamentals**: Company profiles, financial statements, earnings, dividends
- **Cross-listings**: Find all exchanges where security trades (40 credits/request)
- **Historical depth**: 5+ years vs 1-2 years on Basic
- **Higher rate limits**: 55-377 calls/min vs 8/min
- **Email support**: vs community-only

#### Pro Plan Unlocks
- **Real-time data**: True real-time prices (vs delayed on lower tiers)
- **WebSocket streaming**: Low-latency real-time feeds (~170ms)
- **Market movers**: Top gainers/losers across asset classes (100 credits)
- **Extended hours**: US pre/post-market data (4 AM - 8 PM ET)
- **Even higher limits**: 610-1,597 calls/min
- **Priority support**: Faster response times

#### Ultra Plan Unlocks
- **Exchange schedules**: Detailed trading hours, session times (100 credits)
- **FIGI support**: Financial Instrument Global Identifier
- **Composite FIGI**: Multi-exchange instrument identifiers
- **Massive credits**: 2,584-16,721 calls/min
- **WebSocket credits**: 2,500-16,000 WS credits
- **Dedicated support**: Personal account manager
- **Production SLAs**: Uptime guarantees

#### Enterprise Plan Features
- **Custom rate limits**: Tailored to your needs
- **Custom data feeds**: Specialized data integrations
- **On-premise options**: Self-hosted solutions possible
- **Volume discounts**: Negotiated pricing
- **White-label**: Brand as your own
- **Direct exchange feeds**: Optional direct market data

## Rate Limit Details

### How Measured
- **Window**: Per minute (primary), per day (secondary)
- **Rolling window**: Not explicitly documented (likely fixed 1-minute windows)
- **Fixed window**: Appears to be fixed 1-minute windows based on tier limits

### Limit Scope
- **Per API key**: Yes (primary limit)
- Per IP address: Not documented (likely not a separate limit)
- Per account: Yes (all keys under account share quota on some tiers)
- Shared across: All endpoints share the same rate limit pool

### Burst Handling
- Burst allowed: **Not documented** (likely no burst)
- Burst size: N/A
- Burst window: N/A
- Token bucket: Likely not implemented (simple rate limiting)

**Assumption**: Fixed rate limit per minute with no burst allowance. If you exhaust your minute quota early, you must wait for the next minute window.

### Response Headers

On **all successful responses**:
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1234567890
```

**Header descriptions:**
- `X-RateLimit-Limit`: Total requests allowed in current window (per minute)
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp (seconds) when limit resets

### Error Response (HTTP 429)

```json
{
  "code": 429,
  "message": "Rate limit exceeded. Your plan allows 8 API calls per minute",
  "status": "error"
}
```

**Additional headers on 429**:
```
Retry-After: 30
X-RateLimit-Limit: 8
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1706300460
```

### Handling Strategy

**Recommended approach**:

1. **Exponential backoff**:
   - 1st retry: wait 2s
   - 2nd retry: wait 4s
   - 3rd retry: wait 8s
   - 4th retry: wait 16s
   - Max retries: 5

2. **Honor Retry-After header**: Use value from header if present

3. **Proactive rate limiting**:
   - Track `X-RateLimit-Remaining` header
   - Slow down requests when approaching limit
   - Queue requests if needed

4. **Circuit breaker pattern**: Stop sending requests after consecutive 429s

**Example Rust implementation**:
```rust
async fn fetch_with_retry<T>(
    request: Request,
    max_retries: u32,
) -> Result<T, Error> {
    let mut retries = 0;
    let mut backoff = Duration::from_secs(2);

    loop {
        match send_request(&request).await {
            Ok(response) => {
                if response.status() == 429 {
                    if retries >= max_retries {
                        return Err(Error::RateLimitExceeded);
                    }

                    // Check Retry-After header
                    let wait_time = response
                        .headers()
                        .get("Retry-After")
                        .and_then(|h| h.to_str().ok())
                        .and_then(|s| s.parse::<u64>().ok())
                        .map(Duration::from_secs)
                        .unwrap_or(backoff);

                    tokio::time::sleep(wait_time).await;
                    retries += 1;
                    backoff *= 2; // Exponential backoff
                } else {
                    return response.json::<T>().await;
                }
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Quota/Credits System

Twelvedata uses a **credit-based system** where different endpoints consume different amounts of credits.

### How it Works
- **Monthly quota**: Varies by tier (e.g., Basic = 800/day = ~24,000/month)
- **Credit costs**: 1 request = 1-100+ credits (varies by endpoint)
- **Overage**: **Blocked** (requests fail with 429, no auto-charges)
- **Reset**: Daily at midnight (for daily limits), per-minute for rate limits

### Credit Costs

| Endpoint Type | Credits per Request | Notes |
|---------------|---------------------|-------|
| **Basic Market Data** | | |
| Price | 1 | Latest price only |
| Quote | 1 | Full quote data |
| Time Series | 1 per symbol | OHLCV bars |
| EOD | 1 per symbol | End of day |
| Exchange Rate | 1 | Currency conversion rate |
| **Advanced Market Data** | | |
| Cross Rate Time Series | 5 per symbol | Exotic pairs calculation |
| Market Movers | 100 per request | Top gainers/losers |
| Exchange Schedule | 100 per request | Trading hours detail |
| Cross Listings | 40 per request | Multi-exchange lookup |
| **Fundamentals** | | |
| Logo | 1 | Company logo |
| Profile | 10 | Company details |
| Statistics | High demand | Varies, likely 10-20 |
| Earnings | 20 | Historical earnings |
| Dividends | 20 | Historical dividends |
| Income Statement | High demand | Varies, likely 50+ |
| Balance Sheet | High demand | Varies, likely 50+ |
| Cash Flow | High demand | Varies, likely 50+ |
| **Technical Indicators** | | |
| Basic indicators (SMA, EMA) | Varies | Likely 1-2 per symbol |
| High-demand (BBANDS, RSI, MACD) | Varies | Likely 2-5 per symbol |
| **Batch Requests** | | |
| Batch (up to 100 symbols) | 1 per 100 symbols | Significant savings |
| **Reference Data** | | |
| Stocks list | 1 | Full catalog |
| Forex pairs | 1 | Full catalog |
| Crypto list | 1 | Full catalog |
| Symbol search | 1 | Per search query |

**Important**: "High demand" and "Varies" indicate costs may change or depend on data volume. Check documentation for current costs.

### Credit Optimization Strategies

1. **Use batch requests**: 100 symbols = 1 credit vs 100 credits individually
2. **Cache reference data**: Stocks/forex/crypto lists update only daily
3. **Request only needed outputsize**: Don't fetch 5000 bars if you need 100
4. **Use appropriate intervals**: Daily data is same cost as 1-minute but less volume
5. **Avoid redundant fundamentals**: Financial statements are expensive, cache them

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP**: Not documented
- **Max connections per API key**: **3 concurrent connections**
- Max connections total: 3 (per API key, not IP)

### Subscription Limits
- Max subscriptions per connection: **Not documented** (likely 100-120 symbols)
- Max symbols per subscription: Likely matches batch limit (120 symbols)

### Message Rate Limits
- Messages per second: **Not documented**
- Server may throttle: **Not documented**
- Auto-disconnect on violation: **Not documented** (likely yes)

### Connection Duration
- Max lifetime: **Unlimited** (with proper heartbeat)
- Auto-reconnect needed: Not required if heartbeat maintained
- **Idle timeout**: Connection closed if no heartbeat for extended period (exact time not documented)
- **Heartbeat requirement**: Every 10 seconds recommended

### WebSocket Credit Consumption

**Separate from REST API credits:**

| Tier | WebSocket Credits | REST Calls/Min | Notes |
|------|-------------------|----------------|-------|
| Basic | 0 (No access) | 8 | WebSocket not available |
| Grow | 0 (No access) | 55-377 | WebSocket not available |
| Pro | 8-32 | 610-1,597 | WebSocket included |
| Ultra | 2,500-16,000 | 2,584-16,721 | High WebSocket capacity |
| Enterprise | Custom | Custom | Unlimited WS possible |

**Credit consumption rate**: Not explicitly documented, but likely based on:
- Number of concurrent connections
- Number of subscribed symbols
- Connection duration
- Update frequency

**Best practice**: Close idle connections explicitly to avoid wasting WebSocket credits.

## Monitoring Usage

### Dashboard
- **Usage dashboard**: https://twelvedata.com/account
- Real-time tracking: **Yes** (live credit consumption)
- Historical usage: **Yes** (past usage statistics)
- Features:
  - Current credit usage (REST + WebSocket)
  - Rate limit status
  - Quota remaining
  - Usage graphs
  - Billing information
  - Plan comparison

### API Endpoints

**Not explicitly documented**, but likely available:
```
GET /account/usage
GET /account/limits
```

Expected response format:
```json
{
  "plan": "Pro",
  "rate_limit": {
    "calls_per_minute": 610,
    "remaining_this_minute": 543,
    "reset_at": 1706300460
  },
  "daily_quota": {
    "limit": 878000,
    "used": 12543,
    "remaining": 865457,
    "reset_at": 1706313600
  },
  "websocket_credits": {
    "total": 8,
    "used": 2,
    "remaining": 6
  }
}
```

**Note**: Actual endpoint and format not confirmed in documentation.

### Monitoring via Response Headers

**On every REST API response:**
```
X-RateLimit-Limit: 610
X-RateLimit-Remaining: 543
X-RateLimit-Reset: 1706300460
```

**Best practice**: Log these headers to track usage patterns and predict quota exhaustion.

### Alerts
- Email alerts: **Not documented** (check dashboard settings)
- Webhook: **Not documented**
- Dashboard notifications: Likely available at 80-90% quota usage

## Tier Recommendations by Use Case

| Use Case | Recommended Tier | Reasoning |
|----------|------------------|-----------|
| **Learning/Testing** | Basic (Free) | Sufficient for development and testing |
| **Historical Analysis** | Grow ($29-79) | Access to fundamentals, longer history |
| **Real-time Monitoring** | Pro ($99+) | WebSocket streaming, real-time data |
| **Trading Application** | Pro/Ultra ($249-999) | High rate limits, real-time, reliability |
| **Institutional/Enterprise** | Ultra/Enterprise ($1999+) | SLAs, dedicated support, massive capacity |
| **Multi-asset Dashboard** | Pro ($149+) | WebSocket for multiple symbols, real-time |
| **Fundamental Research** | Grow ($49+) | Financials, earnings, dividends |
| **Charting Platform** | Pro ($99+) | Time series, indicators, real-time |

## Special Discounts

- **Students**: 20% discount (verification required)
- **Startups**: 20% discount (criteria not specified)
- **Non-profits**: Contact sales for special pricing
- **Annual billing**: Potential discount (not explicitly advertised)

## Fair Use Policy

Not explicitly documented, but typical restrictions likely include:
- No automated scraping of entire symbol catalogs
- No reselling/redistribution of raw data
- No excessive WebSocket reconnections
- No sharing API keys across organizations
- No circumventing rate limits via multiple accounts

## Comparison: Twelvedata vs Competitors

| Feature | Twelvedata Basic | Alpha Vantage Free | Polygon Free | Yahoo Finance |
|---------|------------------|-------------------|--------------|---------------|
| Price | $0 | $0 | $0 | $0 |
| Calls/min | 8 | 5 | 5 | Unlimited* |
| Calls/day | 800 | ~500 | ~Unlimited | Unlimited* |
| WebSocket | No | No | No | No |
| Fundamentals | No | Limited | No | Yes |
| Reliability | High | Medium | High | Low* |
| Support | Community | Community | Community | None |

*Yahoo Finance is free but unsupported and may break without notice.

## Rate Limit Recovery Time

| Tier | Recovery (if exhausted) |
|------|-------------------------|
| Basic | 1 minute (for per-minute), 24 hours (for daily quota) |
| Grow+ | 1 minute (daily quota much higher, unlikely to exhaust) |
| Pro+ | 1 minute (very high daily quota) |
| Ultra+ | Effectively unlimited (daily quota in millions) |

## Upgrade/Downgrade Policy

- **Upgrade**: Immediate (prorated charge)
- **Downgrade**: End of billing period (no prorated refund)
- **Cancellation**: End of billing period
- **Trial**: 8 API + 8 WebSocket credits for testing before purchase
