# Alpaca - Tiers, Pricing, and Rate Limits

## Free Tier

### Access Level
- Requires sign-up: **Yes** (email only, no verification for paper trading)
- API key required: **Yes** (auto-generated on signup)
- Credit card required: **No** (completely free for paper trading and basic market data)

### Rate Limits

**REST API:**
- Requests per second: **~3** (burst limit of 10/sec allowed)
- Requests per minute: **200** (primary limit)
- Requests per hour: **12,000** (200/min × 60)
- Requests per day: **288,000** (200/min × 1440)
- Burst allowed: **Yes** (10 requests/second burst)

**WebSocket:**
- Max connections: **1** per API key
- Max symbols: **30** (hard limit for free tier)
- Message rate: **Not specified** (server-side throttling possible)
- Connection lifetime: **No time limit** (can run indefinitely)

### Data Access

**Real-time data:**
- **Via WebSocket:** Yes (real-time from IEX exchange)
- **Via REST:** No (15-minute delayed, real-time via latest endpoints but limited)
- **Feed:** IEX only (~2.5% of US market volume)

**Delayed data:**
- **REST API:** Latest endpoints provide near-real-time (within seconds)
- **WebSocket:** Real-time (no delay)
- **Historical:** 15-minute delayed for free tier REST calls

**Historical data:**
- **Depth:** 7+ years (stocks), 6+ years (crypto)
- **Minute bars:** 5 years back (limited to 2016+)
- **Daily bars:** 7+ years
- **Tick data:** No (bars only)

**WebSocket:**
- **Allowed:** Yes
- **Limits:** 1 connection, 30 symbols max
- **Channels:** All channels available (trades, quotes, bars, dailyBars, statuses, lulds)

**Data types:**
- US Stocks: IEX exchange only
- US Options: Indicative feed only (delayed, not real-time)
- Crypto: Full access (same as paid tier)
- Extended hours: Yes (BOATS feed with 15-min delay via "overnight" feed)
- News: Yes (all news sources)
- Corporate actions: Yes (dividends, splits)

### Limitations

**Symbols:**
- REST: Unlimited symbols per request (limited by rate limits)
- WebSocket: **30 symbols max** per connection
- Options: Indicative quotes only (not real-time OPRA feed)

**Endpoints:**
- **Restricted:** None (all endpoints available)
- **Limited:** Options data is indicative (delayed), not real-time
- **Full access:** Stocks, crypto, news, corporate actions

**Features:**
- Paper trading: Unlimited (free forever)
- Live trading: Requires funded account (but still commission-free)
- Fractional shares: Yes
- Margin: No (cash account only for free tier paper trading)
- Options trading: Yes (but data is indicative)

**Exchange coverage:**
- Stocks: **IEX only** (~2.5% market volume)
- To get all exchanges (SIP feed), must upgrade to paid tier

## Paid Tiers

| Tier Name | Price | Rate Limit | Additional Features | WebSocket | Historical | Support |
|-----------|-------|------------|---------------------|-----------|------------|---------|
| **Free (IEX)** | $0 | 200/min | IEX exchange, indicative options, 30 symbols WebSocket | 1 conn, 30 symbols | 7+ years | Community forum |
| **Algo Trader Plus** | $99/mo | Unlimited | All US exchanges (SIP), real-time options (OPRA), unlimited symbols | 1 conn, unlimited symbols | 7+ years | Email support |

**Note:** Only two tiers exist for individual traders. Broker API has different tiers (see below).

### Algo Trader Plus Benefits ($99/mo)

**Rate Limits:**
- REST API: **Unlimited** (fair use applies, no hard cap)
- WebSocket: **Unlimited symbols** (no 30-symbol cap)
- Connections: **1** (same as free tier)

**Data Access:**
- **SIP Feed:** All US exchanges (100% market volume) instead of IEX only
- **Real-time Options:** Full OPRA feed with real Greeks and IV (vs indicative)
- **Extended hours:** Full BOATS feed (Blue Ocean ATS) real-time
- **Everything else:** Same as free tier (crypto, news, corporate actions, 7+ years history)

**Trading:**
- Same commission-free trading as free tier
- No additional trading benefits (trading is always free)

**Upgrade unlocks:**
- Real-time options data (Greeks, IV, chains)
- All US exchanges for stocks (vs IEX only)
- Unlimited symbols on WebSocket (vs 30 max)
- Unlimited REST API calls (vs 200/min)

### Broker API Tiers (for businesses)

These are separate tiers for businesses building trading apps:

| Tier Name | Price | Rate Limit | Options? | WebSocket Conns | Notes |
|-----------|-------|------------|----------|-----------------|-------|
| **Standard** | $0 | 1,000 RPM | No | 5 | Basic broker services |
| **StandardPlus3000** | $500/mo | 3,000 RPM | No | Not specified | Increased limits |
| **StandardPlus5000** | $1,000/mo | 5,000 RPM | Yes | Not specified | Includes options data |
| **StandardPlus10000** | $2,000/mo | 10,000 RPM | Yes | Not specified | Highest tier |

**RPM = Requests Per Minute**

### Alpaca Elite (Account Tier)

**Not a data subscription** - This is an account tier based on balance:

**Requirements:**
- Deposit: **$100,000+** in account

**Benefits:**
- Lower margin rate: **5%** (vs 6.5% standard)
- Free market data subscription: **Algo Trader Plus included** (saves $99/mo)
- White-glove support: Dedicated account manager

**Trading benefits:**
- Same commission-free trading
- Enhanced margin rates
- Priority support

## Rate Limit Details

### How Measured

**Window:**
- **Primary:** Per minute (rolling 60-second window)
- **Burst:** Per second (10 requests/sec allowed short-term)

**Window type:**
- **Rolling window:** Yes (not fixed minute boundaries)
- **Fixed window:** No (resets continuously as time passes)

### Limit Scope

**Per API key:**
- Rate limits: **Yes** (each key has independent 200/min or unlimited)
- WebSocket connections: **Yes** (1 per key)
- Symbol limits: **Yes** (30 symbols per WebSocket connection for free tier)

**Per IP address:**
- **Not specified** (likely not IP-based, key-based only)

**Per account:**
- **No** (can generate multiple keys, each with own limits)

**Shared across:**
- **Not shared** - Each API key has independent rate limit
- **Exception:** WebSocket symbol limit is per connection, not per key

### Burst Handling

**Burst allowed:** Yes
- **Burst size:** ~10 requests/second
- **Burst window:** 1 second
- **Token bucket:** Yes (can burst up to 10/sec, sustained rate 200/min)

**Example:**
- Can send 10 requests instantly
- Then limited to ~3 req/sec sustained (200/min average)
- Burst capacity refills as average drops below 200/min

### Response Headers

All REST API responses include:

```http
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 145
X-RateLimit-Reset: 1705599600
```

**Fields:**
- `X-RateLimit-Limit`: Maximum requests per window (200 for free, large number for unlimited)
- `X-RateLimit-Remaining`: Requests remaining in current window
- `X-RateLimit-Reset`: Unix timestamp when limit resets

**On 429 error:**
```http
HTTP/1.1 429 Too Many Requests
Retry-After: 30
X-RateLimit-Limit: 200
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1705599630
```

### Error Response (HTTP 429)

```json
{
  "code": 429,
  "message": "rate limit exceeded",
  "limit": 200,
  "remaining": 0,
  "reset": 1705599630
}
```

**Note:** Exact error format may vary (some endpoints return different structures)

### Handling Strategy

**Recommended approach:**

1. **Check headers:** Monitor `X-RateLimit-Remaining` before each request
2. **Exponential backoff:** On 429, wait `Retry-After` seconds, then double wait time on repeated failures
3. **Queue requests:** Implement request queue to stay under 200/min
4. **Use WebSocket:** For real-time data, WebSocket is better (no rate limit on messages)
5. **Batch requests:** Use multi-symbol endpoints (`/v2/stocks/bars?symbols=AAPL,MSFT,GOOGL`)
6. **Cache data:** Don't request same historical data repeatedly

**Exponential backoff example:**
```
1st 429: Wait 30s (from Retry-After)
2nd 429: Wait 60s (2× previous)
3rd 429: Wait 120s (2× previous)
4th 429: Wait 240s (2× previous, max ~5min)
```

## Quota/Credits System (if applicable)

**NOT USED** - Alpaca uses simple rate limits, not a credit/quota system.

All endpoints cost the same:
- 1 request = 1 count toward rate limit
- No endpoint-specific costs
- No monthly quota (only per-minute rate limit)

**Exception:** Historical data doesn't have different costs per timeframe (unlike some providers)

## WebSocket Specific Limits

### Connection Limits

**Free Tier:**
- Max connections per IP: **Not specified** (likely unlimited IPs)
- Max connections per API key: **1** (error 406 if exceeded)
- Max connections total: **1** per key

**Paid Tier:**
- Max connections per API key: **1** (same as free, but unlimited symbols)
- Error 406: "connection limit exceeded" if trying to open multiple connections

**Workaround:** Generate multiple API keys if need multiple connections (each key gets 1 connection)

### Subscription Limits

**Free Tier:**
- Max subscriptions per connection: **Unlimited channels**
- Max symbols per subscription: **30 symbols total** (across all channels)
- Example: 10 trades + 10 quotes + 10 bars = 30 symbols (limit reached)

**Paid Tier:**
- Max subscriptions per connection: **Unlimited channels**
- Max symbols per subscription: **Unlimited**
- Example: Can subscribe to all S&P 500 stocks at once

**Error 405:** "symbol limit exceeded" if free tier exceeds 30 symbols

### Message Rate Limits

**Inbound (client → server):**
- Messages per second: **Not specified** (reasonable use expected)
- Server may throttle: **Possible** (not documented)
- Auto-disconnect on violation: **Possible** (not documented)

**Outbound (server → client):**
- Messages per second: **Unlimited** (depends on market activity)
- Batching: Server batches messages in arrays for efficiency
- Throttling: **Not applied** (you receive all updates in real-time)

**Note:** If subscribing to 30 active stocks, expect hundreds of messages/second during market hours

### Connection Duration

**Max lifetime:**
- **Unlimited** (WebSocket can stay open 24/7)
- No forced disconnections

**Auto-reconnect needed:**
- **Not required** (but recommended to handle network issues)
- Implement reconnection logic for robustness

**Idle timeout:**
- **None documented** (connection stays open even if no messages sent)
- **Recommendation:** Send periodic pings to keep alive (standard WebSocket practice)

**Maintenance disconnections:**
- **Possible** during Alpaca system maintenance
- Always implement reconnection logic

## Monitoring Usage

### Dashboard

**Free Tier:**
- Usage dashboard: **Not available** (no dashboard for API usage stats)
- Real-time tracking: **No** (must check headers)
- Historical usage: **No**

**Paid Tier:**
- Usage dashboard: **Not documented** (may exist for Algo Trader Plus)
- Rate limit is unlimited, so less critical to monitor

**Recommendation:** Track usage in your own application using response headers

### API Endpoints

**No usage endpoints documented:**
- No `/account/usage` endpoint
- No `/account/limits` endpoint
- Must rely on `X-RateLimit-*` headers in responses

**Workaround:** Log headers from each response to track usage

### Alerts

**Email alerts:**
- **Not available** (no built-in usage alerts)

**Webhook:**
- **Not available**

**Recommendation:**
- Implement own monitoring
- Alert when `X-RateLimit-Remaining` drops below threshold
- Log 429 errors and investigate patterns

## Comparing Tiers - Quick Reference

| Feature | Free (IEX) | Algo Trader Plus ($99/mo) |
|---------|------------|---------------------------|
| **REST Rate Limit** | 200/min | Unlimited |
| **WebSocket Connections** | 1 | 1 |
| **WebSocket Symbols** | 30 max | Unlimited |
| **Stock Feed** | IEX only (~2.5% volume) | SIP (100% market, all exchanges) |
| **Options Data** | Indicative (delayed) | Real-time OPRA (Greeks, IV) |
| **Crypto Data** | Full access | Full access (same) |
| **Historical Data** | 7+ years | 7+ years (same) |
| **News** | Full access | Full access (same) |
| **Corporate Actions** | Full access | Full access (same) |
| **Extended Hours** | Overnight feed (15-min delay) | BOATS real-time |
| **Paper Trading** | Free forever | Free forever (same) |
| **Live Trading** | Commission-free | Commission-free (same) |
| **Support** | Community forum | Email support |

## Special Cases

### Paper Trading
- **Always free** (even with paid market data subscription)
- Uses same rate limits as live account tier
- Full trading features (margin, options, crypto)

### Live Trading
- **Commission-free** (both free and paid tiers)
- No per-trade fees
- SEC fees apply (regulatory, unavoidable)
- Payment for order flow (how Alpaca makes money)

### Crypto Trading
- **Same data access** (free and paid tiers identical for crypto)
- No upgrade needed for crypto
- Historical crypto data: **No auth required**

### Options Trading
- **Data:** Indicative (free) vs Real-time OPRA (paid)
- **Trading:** Allowed on both tiers (but need options account approval)
- **Levels:** Up to Level 3 options trading available

### Margin Trading
- **Interest rate:** 6.5% standard, 5% for Alpaca Elite ($100k+ balance)
- **Buying power:** Up to 4X intraday, 2X overnight
- **No data tier impact:** Margin rates same for free/paid data subscribers

## Upgrade Path

**From Free to Algo Trader Plus:**
1. Go to Dashboard → Subscriptions
2. Click "Upgrade" to Algo Trader Plus
3. Pay $99/mo (charged monthly)
4. Instant access (no waiting)
5. Same API keys work (no regeneration needed)

**From Algo Trader Plus to Free:**
1. Cancel subscription in Dashboard
2. Downgrade at end of billing period
3. WebSocket limited to 30 symbols
4. REST API limited to 200/min
5. Options data becomes indicative
6. Stock feed becomes IEX only

**Refunds:** Pro-rated (not documented, contact support)

## Fair Use Policy

**Unlimited tier:**
- Not truly infinite (fair use applies)
- Excessive abuse may be flagged
- Reasonable: Thousands of requests/minute for legitimate trading
- Unreasonable: DDOS-like behavior, scraping entire market every second

**Best practices:**
- Use WebSocket for real-time data (more efficient than polling)
- Cache historical data (don't re-request same data)
- Batch multi-symbol requests
- Don't hammer API for no reason

**No documented hard limits for unlimited tier** - trust-based system
