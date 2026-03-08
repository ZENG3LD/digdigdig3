# Whale Alert - Tiers, Pricing, and Rate Limits

## Free Tier (Developer API v1 - Deprecated)

### Access Level
- Requires sign-up: Yes
- API key required: Yes
- Credit card required: No

### Rate Limits
- Requests per second: Not specified
- Requests per minute: 10
- Requests per hour: 600 (estimated, 10/min × 60)
- Requests per day: Not explicitly specified
- Burst allowed: Not documented

### Data Access
- Real-time data: No (deprecated API)
- Delayed data: Yes (transaction history)
- Historical data: Limited access
- WebSocket: No (not available in free tier)
- Data types: Basic transaction queries, status checks

### Limitations
- Symbols: Unlimited (all supported blockchains and currencies)
- Endpoints: Limited to Developer API v1 endpoints
- Features: No WebSocket, no Enterprise API access, basic transaction history only

---

## Paid Tiers

| Tier Name | Price | Rate Limit | Additional Features | WebSocket | Historical | Support |
|-----------|-------|------------|---------------------|-----------|------------|---------|
| Free (v1 API) | $0 | 10/min | Basic transaction queries | No | Limited | Community |
| Personal (v1 API) | Not specified | 60/min | Increased rate limit | No | Limited | Community |
| Custom Alerts | $29.95/mo | 100 alerts/hour | Real-time WebSocket alerts | Yes (2 conn) | No | Email |
| Quantitative | $699/mo | 1000 req/min | Live transaction stream, ML-ready | No | 30 days | Priority |
| Priority Alerts | $1,299/mo | 10,000 alerts/hour | Fastest alerts (1min advantage) | Yes (2 conn) | No | Priority |
| Historical | $1,990/year | Custom | Historical datasets for training | Custom | Full historical | Dedicated |

### Trial Period
- **7-Day Free Trial** available for Custom Alerts tier ($29.95/mo)
- No trial mentioned for other paid tiers

---

## Tier Details

### 1. Free Tier (Developer API v1) - DEPRECATED

**Price:** $0

**Rate Limits:**
- 10 requests per minute
- Fixed window (likely)

**Access:**
- REST API v1 only
- Basic transaction queries
- Status endpoint
- Single transaction lookup
- Multi-transaction queries with pagination

**Limitations:**
- No WebSocket access
- No Enterprise API features
- Deprecated (may be discontinued)
- No historical data API
- Limited support

**Best For:** Testing, personal projects, learning

---

### 2. Personal Tier (Developer API v1)

**Price:** Not publicly listed (likely discontinued or merged into other tiers)

**Rate Limits:**
- 60 requests per minute

**Access:**
- Same as Free tier but with higher rate limit

**Limitations:**
- Still on deprecated v1 API
- No WebSocket
- No Enterprise features

---

### 3. Custom Alerts (WebSocket API)

**Price:** $29.95/month
**Trial:** 7-day free trial

**Rate Limits:**
- Max connections: 2 per API key
- Max alerts: 100 per hour
- Message rate: Not specified

**Access:**
- WebSocket real-time alerts
- Filter by blockchain, symbol, transaction type
- Minimum value: $100,000 USD
- Social media alerts (Whale Alert posts)
- Custom alert parameters

**Features:**
- Real-time transaction monitoring
- Customizable filters
- Browser and mobile notifications (if using dashboard)
- ~100 crypto assets supported
- Address attribution data
- Owner identification

**Limitations:**
- Up to 100 alerts per hour
- No historical data access
- No REST API access (alerts only)
- 2 concurrent connections maximum

**Best For:** Traders, small teams, real-time monitoring

---

### 4. Quantitative (Enterprise REST API)

**Price:** $699/month

**Rate Limits:**
- 1,000 requests per minute
- Rolling or fixed window (not specified)

**Access:**
- Enterprise REST API (v2)
- Live stream of millions of transactions per day
- 30 days of historical data
- All supported blockchains
- Address attribution for exchanges
- Transaction filtering and streaming
- Block-level queries
- Address transaction history

**Features:**
- Ideal for AI/ML models
- Real-time transaction data
- Standardized format across all blockchains
- High-quality curated data
- Used by top quantitative traders
- Suitable for algorithmic trading

**Data Volume:**
- Millions of transactions per day
- 30-day historical depth
- 11+ blockchains
- All major cryptocurrencies

**Limitations:**
- No WebSocket alerts (use REST polling or separate Custom Alerts subscription)
- Historical data limited to 30 days (use Historical tier for deeper history)

**Best For:** Quantitative trading, algorithmic strategies, ML model training (recent data)

---

### 5. Priority Alerts (WebSocket API - Professional)

**Price:** $1,299/month

**Rate Limits:**
- Max connections: 2 per API key
- Max alerts: 10,000 per hour (technically unlimited according to docs)
- Message rate: Technically unlimited rate capacity

**Access:**
- Same WebSocket API as Custom Alerts
- **Speed advantage: Up to 1 minute faster delivery**
- First in line to receive new alerts
- All features of Custom Alerts tier

**Features:**
- Ultra-low latency alerts
- Customizable JSON integration
- Over 100 supported cryptocurrencies
- Real-time on-chain signals
- Integration support
- Secure and private data handling
- Professional-grade SLA

**Use Cases:**
- Professional traders
- Asset managers
- Institutional trading
- High-frequency monitoring
- Risk management
- Early event detection

**Proven Track Record:**
- Early warning for Bybit hack
- Early warning for FTX collapse
- Scientifically validated alerts

**Best For:** Professional traders, institutions, those needing fastest possible alerts

---

### 6. Historical Data

**Price:** $1,990 per year of data

**Rate Limits:**
- Custom (negotiated)
- Delivered as dataset, not API

**Access:**
- Custom historical datasets
- Transaction data with timestamps
- Asset price information
- Suitable for model training
- Bulk delivery (not real-time API)

**Features:**
- Historical transaction archives
- Price data included
- Custom date ranges
- Research-grade data quality
- Ideal for backtesting

**Limitations:**
- Not a real-time API
- Delivered as files/database dumps
- Custom delivery (contact sales)

**Best For:** ML model training, academic research, backtesting, historical analysis

---

## Rate Limit Details

### How Measured

**Developer API v1:**
- Window: Per minute
- Rolling window: Likely
- Fixed window: Possible

**Enterprise API (Quantitative):**
- Window: Per minute
- Rolling window: Not specified
- Limit: 1,000 requests/minute

**WebSocket (Custom Alerts):**
- Window: Per hour
- Limit: 100 alerts/hour
- Connection limit: 2 concurrent

**WebSocket (Priority Alerts):**
- Window: Per hour
- Limit: 10,000 alerts/hour (effectively unlimited)
- Connection limit: 2 concurrent

### Limit Scope
- Per IP address: No (per API key)
- Per API key: Yes (all limits are per API key)
- Per account: Yes (API key represents account)
- Shared across: Not specified (likely separate limits for REST vs WebSocket)

### Burst Handling
- Burst allowed: Not documented
- Burst size: Not specified
- Burst window: Not specified
- Token bucket: Likely (standard implementation)

### Response Headers

**REST API Rate Limit Headers:**

Not explicitly documented, but likely standard format:

```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 856
X-RateLimit-Reset: 1640000000
```

On HTTP 429:
```
Retry-After: 30
```

### Error Response (HTTP 429)

```json
{
  "error": "Rate limit exceeded",
  "error_code": 429,
  "limit": 1000,
  "remaining": 0,
  "reset": 1640000000,
  "retry_after": 30
}
```

OR (simpler format):

```json
{
  "error_code": 429,
  "error_message": "Rate limit exceeded. Please wait before retrying."
}
```

### Handling Strategy

**Recommended Implementation:**

1. **Pre-emptive Rate Limiting:**
   - Track requests locally
   - Implement token bucket or leaky bucket algorithm
   - Stay under limit (e.g., target 950/min for 1000/min limit)

2. **Backoff on 429:**
   - Exponential backoff: 1s, 2s, 4s, 8s, 16s, ...
   - Respect `Retry-After` header if present
   - Max retries: 3-5 attempts

3. **Request Queuing:**
   - Queue requests when approaching limit
   - Distribute requests evenly across time window
   - Priority queue for critical requests

4. **Multi-Key Load Balancing:**
   - Use multiple API keys (if allowed)
   - Round-robin or least-used key selection
   - Aggregate rate limits across keys

---

## Quota/Credits System

**Not Applicable** - Whale Alert uses time-based rate limits, not credit/quota systems.

Exception: Custom Alerts has "100 alerts per hour" which functions like a quota but resets hourly.

---

## WebSocket Specific Limits

### Custom Alerts API ($29.95/mo)

**Connection Limits:**
- Max connections per API key: 2
- Max connections total: 2 per API key
- Connection duration: Not specified (likely 24 hours or unlimited with keep-alive)

**Subscription Limits:**
- Max subscriptions per connection: Not specified (likely multiple allowed)
- Max symbols per subscription: Unlimited (filter arrays support multiple values)
- Max blockchains per subscription: Unlimited (filter arrays support multiple values)

**Message Rate Limits:**
- Alerts per hour: 100 maximum
- Server throttling: Yes (enforced via alert limit)
- Auto-disconnect on violation: Likely (behavior not documented)

**Alert Volume Control:**
- Minimum transaction value: $100,000 USD (enforced)
- Filtering required to stay under 100 alerts/hour limit
- Consider using higher `min_value_usd` during high-volume periods

### Priority Alerts API ($1,299/mo)

**Connection Limits:**
- Max connections per API key: 2
- Max connections total: 2 per API key
- Connection duration: Not specified

**Subscription Limits:**
- Max subscriptions per connection: Not specified (likely multiple)
- Max symbols per subscription: Unlimited
- Max blockchains per subscription: Unlimited

**Message Rate Limits:**
- Alerts per hour: 10,000 (technically unlimited according to docs)
- Server throttling: Minimal to none
- Auto-disconnect on violation: Unlikely given "unlimited" claim

**Latency Advantage:**
- Up to 1 minute faster than Custom Alerts
- First in line for new alerts

### Connection Duration
- Max lifetime: Not specified (likely 24 hours or connection-based)
- Auto-reconnect needed: Yes (implement reconnection logic)
- Idle timeout: Not specified
- Keep-alive: Standard WebSocket ping/pong recommended

---

## Monitoring Usage

### Dashboard
- Usage dashboard: https://developer.whale-alert.io/ (likely includes usage stats)
- Real-time tracking: Not confirmed
- Historical usage: Not confirmed

### API Endpoints
- Check quota: Not available
- Check limits: Not available
- Account info: Likely available in developer dashboard

### Alerts
- Email alerts: Not documented
- Usage warnings: Not documented
- Overage handling: Connection throttled or terminated

---

## Upgrade Benefits

### From Free to Personal (v1 API)
- Rate limit: 10/min → 60/min
- Access: Same endpoints, just faster

### From Free/Personal to Custom Alerts
- **New capability:** Real-time WebSocket alerts
- **New capability:** Social media alerts
- **New capability:** Custom filtering
- Rate model: Switch from requests/min to alerts/hour
- Data: Real-time transaction notifications

### From Custom Alerts to Priority Alerts
- **Speed:** Up to 1 minute faster delivery
- **Volume:** 100/hour → 10,000/hour (100x increase)
- **Reliability:** Technically unlimited rate capacity
- **Priority:** First in line for alerts
- **Use case:** Professional/institutional grade

### From Free/Alerts to Quantitative
- **New capability:** Enterprise REST API access
- **New capability:** Transaction streaming
- **New capability:** 30-day historical access
- **New capability:** Block-level queries
- **New capability:** Address history and attribution
- Rate limit: 1,000 requests/minute (much higher than v1 API)
- Data format: Standardized across all blockchains
- Use case: Algorithmic trading, ML models

### Adding Historical Data
- **New capability:** Deep historical archives
- **New capability:** Bulk data delivery
- Use case: Model training, backtesting, research
- Format: Custom datasets (not API)

---

## Comparison Matrix

| Feature | Free (v1) | Custom Alerts | Quantitative | Priority Alerts | Historical |
|---------|-----------|---------------|--------------|-----------------|------------|
| **Price** | $0 | $29.95/mo | $699/mo | $1,299/mo | $1,990/year |
| **REST API** | v1 (deprecated) | No | v2 (Enterprise) | No | No (bulk delivery) |
| **WebSocket** | No | Yes | No | Yes | No |
| **Rate Limit** | 10/min | 100/hour | 1000/min | 10k/hour | Custom |
| **Real-time** | No | Yes | Yes | Yes (fastest) | No |
| **Historical** | Limited | No | 30 days | No | Full archive |
| **Trial** | Free forever | 7 days | No | No | No |
| **Best For** | Testing | Traders | Quant/ML | Institutions | Research |

---

## Recommendations by Use Case

**Personal Learning/Testing:**
- Start with Free tier (Developer API v1)
- Explore basic transaction queries
- Understand data structure

**Active Trader:**
- Custom Alerts ($29.95/mo)
- Real-time notifications
- Filter for specific assets/exchanges
- 100 alerts/hour sufficient for most retail traders

**Quantitative Strategy:**
- Quantitative tier ($699/mo)
- Live data stream for ML models
- 30-day historical for recent pattern analysis
- Consider adding Historical tier for deeper backtesting

**Professional/Institutional:**
- Priority Alerts ($1,299/mo)
- Fastest possible data delivery
- Risk management and alpha generation
- 10k alerts/hour for comprehensive monitoring

**Academic Research:**
- Historical tier ($1,990/year of data)
- Bulk datasets for analysis
- No need for real-time API
- Cost-effective for non-trading applications

**Comprehensive Solution:**
- Combine Quantitative + Priority Alerts
- Real-time fastest alerts via WebSocket
- Programmatic access via REST API
- Total: $1,998/mo (both APIs)

---

## Notes

1. **No Free WebSocket:** Real-time alerts require paid subscription ($29.95/mo minimum)
2. **Developer API Deprecated:** Free and Personal tiers use deprecated v1 API
3. **7-Day Trial:** Only available for Custom Alerts tier
4. **Per-Year Pricing:** Historical data priced per year of data, not monthly subscription
5. **Speed Matters:** Priority Alerts deliver 1-minute advantage - critical for institutional trading
6. **Separate APIs:** REST (Quantitative) and WebSocket (Alerts) are separate products with separate pricing
7. **No Enterprise Trial:** Quantitative and Priority tiers don't mention trial periods
8. **Rate Limits Strict:** Custom Alerts limited to 100/hour - requires careful filtering to avoid hitting limit
9. **Institutional Pricing:** For higher limits or custom needs, contact Whale Alert sales
