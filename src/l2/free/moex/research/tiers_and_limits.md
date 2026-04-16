# MOEX - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- **Requires sign-up**: No (public access available)
- **API key required**: No
- **Credit card required**: No
- **MOEX Passport account**: Optional (not required for basic access)

### Rate Limits
**Note**: MOEX does not publicly document rate limits for ISS API.

**Estimated (based on typical practices)**:
- **Requests per second**: Unknown (likely 1-5)
- **Requests per minute**: Unknown (likely 60-300)
- **Requests per hour**: Unknown
- **Requests per day**: Unknown
- **Burst allowed**: Unknown

**Enforcement**:
- HTTP 429 (Too Many Requests) if exceeded
- No public documentation on exact limits
- Limits likely vary by endpoint type

**Recommendation**: Implement conservative request patterns (1 req/sec) and exponential backoff.

### Data Access
- **Real-time data**: No (delayed by 15 minutes)
- **Delayed data**: Yes (15-minute delay for market data)
- **Historical data**: Yes (unlimited depth, free access)
- **WebSocket**: Allowed (delayed data only)
- **Data types**: All market data types available (delayed)

**Available Data (Free Tier)**:
- Current market data (15-min delay)
- Historical prices (unlimited depth, from 1997+ for major indices)
- OHLC/Candles (all intervals)
- Trades (delayed)
- Quotes/Best bid-ask (delayed)
- Reference data (securities, boards, markets)
- Corporate information (financials, ratings, dividends)
- Indices and analytics
- News and events
- Statistical data

**Restrictions**:
- 15-minute data delay
- No real-time orderbook access
- No Level 2 market depth
- Unknown rate limits (implement conservative patterns)

### Limitations
- **Symbols**: Unlimited (all MOEX-traded securities)
- **Endpoints**: All public endpoints available
- **Features**: Real-time data and orderbook restricted
- **Data redistribution**: Prohibited
- **Commercial use**: Prohibited without license

## Paid Tiers

**Note**: MOEX does not publish public pricing tiers. Access is subscription-based with custom pricing.

### Subscription Types

| Tier Type | Price | Rate Limit | Data Access | WebSocket | Historical | Support |
|-----------|-------|------------|-------------|-----------|------------|---------|
| Free (Public) | $0 | Unknown (conservative) | Delayed 15min | Delayed | Unlimited | Community/Email |
| Real-time Subscriber | Contact MOEX | Unknown (higher) | Real-time | Real-time | Unlimited | Email/Phone |
| Local Distributor | Contact MOEX | Custom | Real-time + Redistribution | Real-time | Unlimited | Priority |
| Foreign Distributor | Contact MOEX | Custom | Real-time + Redistribution | Real-time | Unlimited | Priority |
| Institutional | Contact MOEX | Custom | Everything + Full Orderbook | Real-time | Unlimited | Dedicated |

### Pricing Structure (General Information)

**For Subscribers (No Redistribution)**:
- Real-time streaming data
- Daily trading results
- Archive data access
- Contact MOEX sales for pricing

**For Distributors (With Redistribution Rights)**:
- **Local (Russian) Distributors**:
  - Real-time streaming data (including orderbooks)
  - Real-time deal data (without orderbooks) - separate pricing
  - Delayed (15 min) streaming data for public display
  - Trading results for public display
  - Tariff for delayed data includes redistribution rights

- **Foreign (Non-Russian) Distributors**:
  - Similar categories to local distributors
  - Different pricing structure
  - Contact MOEX for international rates

### Upgrade Benefits

**Free → Real-time Subscriber**:
- **Data delay**: 15 minutes → 0 minutes (real-time)
- **Orderbook**: Not available → Limited access to best quotes
- **WebSocket**: Delayed streams → Real-time streams
- **Support**: Community → Email/Phone support
- **Rate limits**: Likely increased (not documented)

**Real-time Subscriber → Distributor**:
- **Redistribution**: Prohibited → Allowed
- **Commercial use**: Prohibited → Allowed
- **Public display**: Prohibited → Allowed
- **Orderbook depth**: Limited → Full (10x10 or 5x5)
- **Additional features**: Full Order Book product access

**Distributor → Institutional**:
- **Custom integration**: Standard → Custom solutions
- **Dedicated support**: Email/Phone → Account manager
- **Data products**: Standard feeds → Custom data products
- **Infrastructure**: Shared → Potentially dedicated
- **SLA**: Standard → Custom SLA

### How to Subscribe

1. **Real-time Subscription**:
   - Contact: https://www.moex.com/s1147 (Data Services)
   - Phone: +7 (495) 733-9507
   - Email: help@moex.com
   - Provide use case and requirements
   - Receive pricing quote
   - Sign subscription agreement
   - Account activated for real-time access

2. **Distributor License**:
   - Contact MOEX sales team
   - Provide business plan and redistribution intent
   - Receive custom pricing based on:
     - Data types required
     - Redistribution scope
     - Geographic region
     - Number of end-users
   - Sign distributor agreement
   - Receive redistribution rights

## Rate Limit Details

### How Measured
**Note**: Not publicly documented. Estimated based on standard practices:

- **Window**: Unknown (likely per minute or per second)
- **Rolling window**: Unknown (likely yes)
- **Fixed window**: Unknown (possibly no)

### Limit Scope
- **Per IP address**: Likely yes (common practice)
- **Per API key**: Not applicable (no API key system)
- **Per account**: Unknown (for authenticated users)
- **Shared across**: Unknown

### Burst Handling
- **Burst allowed**: Unknown
- **Burst size**: Unknown
- **Burst window**: Unknown
- **Token bucket**: Unknown

**Best practice**: Implement rate limiting on client side to avoid hitting server limits.

### Response Headers

**Not documented**. Typical rate limit headers (may or may not be present):
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1234567890
Retry-After: 30
```

**Check MOEX responses** to see if these headers are provided.

### Error Response (HTTP 429)

**Expected format** (not documented):
```json
{
  "error": "Rate limit exceeded",
  "message": "Too many requests. Please slow down.",
  "retry_after": 30
}
```

Or possibly XML:
```xml
<error>
  <message>Rate limit exceeded</message>
  <retry_after>30</retry_after>
</error>
```

### Handling Strategy

**Recommended implementation**:

1. **Conservative Rate Limiting**:
   ```rust
   // Limit to 1 request per second to be safe
   let rate_limiter = RateLimiter::new(1, Duration::from_secs(1));
   ```

2. **Exponential Backoff**:
   ```rust
   let mut delay = Duration::from_secs(1);
   loop {
       match make_request().await {
           Ok(response) => return Ok(response),
           Err(e) if e.status() == Some(429) => {
               sleep(delay).await;
               delay *= 2; // Exponential backoff
               if delay > Duration::from_secs(60) {
                   return Err(e); // Give up after 60s delay
               }
           }
           Err(e) => return Err(e),
       }
   }
   ```

3. **Retry Logic**:
   - Retry on 429, 500, 503
   - Don't retry on 400, 401, 403, 404
   - Implement jitter to avoid thundering herd

4. **Request Queuing**:
   - Queue requests internally
   - Process at controlled rate
   - Prioritize critical requests

## Quota/Credits System

**Not applicable**: MOEX does not use a credits/quota system. Access is based on:
- Free (delayed data, unlimited requests within rate limits)
- Subscription (real-time data, higher rate limits)

## WebSocket Specific Limits

### Connection Limits (Estimated)
- **Max connections per IP**: Unknown (likely 5-10 for free, higher for paid)
- **Max connections per API key**: Not applicable (credential-based)
- **Max connections per account**: Unknown (likely 10-50 for paid)

### Subscription Limits (Estimated)
- **Max subscriptions per connection**: Unknown (likely 100-500)
- **Max symbols per subscription**: Unknown (varies by topic)

### Message Rate Limits (Estimated)
- **Messages per second**: Server may throttle high-frequency subs
- **Server throttling**: Likely yes (automatic)
- **Auto-disconnect on violation**: Possible but not documented

### Connection Duration
- **Max lifetime**: Unlimited (with proper heart-beats)
- **Auto-reconnect needed**: On connection loss or heart-beat timeout
- **Idle timeout**: Unknown (heart-beats prevent idle disconnect)

**Best practice**: Implement automatic reconnection with exponential backoff.

## Monitoring Usage

### Dashboard
- **Usage dashboard**: Not available publicly
- **Real-time tracking**: Not available publicly
- **Historical usage**: Not available publicly

**For paid subscribers**: May have access to usage dashboard (contact MOEX).

### API Endpoints
**No public endpoints** for checking quota or limits.

Possible for paid subscribers:
- Check quota: Unknown
- Check limits: Unknown
- Response format: Unknown

### Alerts
- **Email alerts**: Not available publicly
- **Webhook**: Not available publicly
- **Usage warnings**: Unknown

## Data Redistribution Pricing

### For Local (Russian) Distributors

**Data Type Categories** (pricing not public):
1. **Real-time streaming data (with orderbooks)**
   - Full market depth
   - Real-time updates
   - Redistribution rights

2. **Real-time deal data (without orderbooks)**
   - Trades only
   - No orderbook depth
   - Lower pricing tier

3. **Delayed streaming data (15 min)**
   - Delayed by 15 minutes
   - Public display/dissemination allowed
   - Includes trading results redistribution
   - Most affordable tier

4. **Trading results**
   - End-of-day data
   - Daily summaries
   - Public display allowed

### For Foreign (Non-Russian) Distributors

Similar categories to local distributors with different pricing:
- International pricing rates
- Multi-currency support
- Cross-border data delivery
- Contact MOEX for specific rates

### Redistribution Rights

**Included with distributor license**:
- Right to resell data
- Right to display publicly
- Right to create data products
- Right to serve multiple clients

**Not included**:
- Unlimited redistribution (scope defined in contract)
- Derivative works (may require approval)
- Sub-licensing (check contract terms)

## Historical Data Access

### Free Tier
- **Depth**: Unlimited (data from 1997+ for major instruments)
- **Granularity**: All intervals available
- **Bulk downloads**: Archive files available via API
- **Restrictions**: None on historical data access

### Paid Tier
- **Depth**: Unlimited (same as free)
- **Granularity**: All intervals
- **Bulk downloads**: Full archive access
- **Additional**: Full Order Book historical data (separate product)

## Full Order Book Product

**Institutional product** for complete market reconstruction:

### Features
- **Zipped files** with all MOEX Market Data messages
- **Complete order book** at any point in time
- **Message-by-message** market events
- **Replay capability** for backtesting

### Pricing
- Contact MOEX sales
- Institutional pricing
- Based on data volume and usage

### Use Cases
- High-frequency backtesting
- Market microstructure research
- Order book analytics
- Algorithm development

## Connectivity Fees

**Note**: MOEX has separate connectivity fees for direct market access (trading), not covered here.

For **market data (ISS)**:
- No separate connectivity fees
- Included in subscription pricing
- Internet connection required (standard HTTPS/WebSocket)

## Free Data Summary

**What's Free**:
- Delayed market data (15 min)
- All historical data (unlimited depth)
- Reference data (securities, boards, markets)
- Corporate information (financials, ratings, dividends)
- Indices and analytics
- News and events
- Statistical data
- Archives (bulk downloads)

**What Requires Subscription**:
- Real-time market data (no delay)
- Real-time orderbook depth
- Full Order Book product
- Data redistribution rights
- Higher rate limits (possibly)

## Contact Information for Pricing

- **Data Services**: https://www.moex.com/s1147
- **Technical Support**: help@moex.com
- **Phone**: +7 (495) 733-9507
- **Sales**: Contact form on MOEX website
- **International**: Request international sales contact

## Summary

- **Free tier**: Delayed data (15 min), unlimited historical, unknown rate limits
- **Paid tier**: Real-time data, subscription-based, custom pricing
- **Distributors**: Redistribution rights, custom contracts
- **Rate limits**: Not documented, implement conservative patterns (1 req/sec)
- **WebSocket**: Connections and subscriptions limited (unknown exact numbers)
- **Historical data**: Free and unlimited for all users
- **Pricing**: Contact MOEX sales for quotes
- **No API key system**: Uses username/password authentication
- **No public usage dashboard**: Monitor client-side

**Recommendation for V5 Connector**:
- Implement client-side rate limiting (1 req/sec default)
- Exponential backoff on errors
- Support both free (delayed) and paid (real-time) modes
- Cache reference data aggressively
- Use WebSocket for real-time streams (more efficient than REST polling)
