# Bitquery - Tiers, Pricing, and Rate Limits

## Free Tier

### Developer Plan

#### Access Level
- **Requires sign-up**: Yes (https://account.bitquery.io/auth/signup)
- **API key required**: Yes (OAuth token)
- **Credit card required**: No
- **Email verification**: Yes (confirmation link sent)
- **Approval process**: No - instant access after signup

#### Rate Limits
- **Requests per second**: Not specified (burst allowed)
- **Requests per minute**: 10
- **Requests per hour**: Not specified (assumed 600 based on 10/min)
- **Requests per day**: Not specified
- **Burst allowed**: Yes (but limited by points)

#### WebSocket Limits
- **Simultaneous streams**: 2 (max 2 subscriptions running concurrently)
- **Connections**: Not limited (but streams are)
- **Messages per second**: Not specified
- **Subscription duration**: No time limit (but points consumed per minute)

#### Data Access
- **Real-time data**: Yes (via subscriptions)
- **Delayed data**: No delay - all data is real-time or historical (no artificial delays)
- **Historical data**: Yes (depth: Full history from blockchain genesis)
- **WebSocket**: Allowed (2 simultaneous streams)
- **Data types**: All cubes available (Blocks, Transactions, Transfers, DEXTrades, Events, Calls, etc.)

#### Points Quota
- **Monthly allocation**: 10,000 points (first month trial)
- **Realtime queries**: 5 points per cube per query
- **Archive queries**: Variable (based on complexity and data volume)
- **Subscriptions**: 40 points per minute per stream
- **Overage**: Blocked when quota exhausted

#### Query Limits
- **Rows per request**: 10 (max limit parameter)
- **Query timeout**: Standard timeout (complex queries may timeout)
- **Default limit**: 10,000 records if no limit specified (but free tier restricted to 10)

#### Limitations
- **Symbols**: Unlimited (all blockchains and tokens)
- **Endpoints**: All cubes available (no restrictions)
- **Features**:
  - Limited rows per query (10)
  - Limited rate (10 req/min)
  - Limited streams (2 simultaneous)
  - Limited points (10,000/month)
  - Community support only

#### Support
- **Public Telegram**: Yes
- **Community forum**: Yes (https://community.bitquery.io/)
- **Email support**: No (paid plans only)
- **Response time**: Best effort (community-driven)

---

## Paid Tiers

### Commercial Plan

| Feature | Free (Developer) | Commercial |
|---------|------------------|------------|
| **Price** | $0/month | Custom (contact sales) |
| **Monthly Points** | 10K trial | Custom allocation |
| **Rows per Request** | 10 | Unlimited |
| **Rate Limit** | 10 req/min | No throttling (scalable) |
| **WebSocket Streams** | 2 simultaneous | Unlimited |
| **Blockchain Access** | All 40+ chains | All 40+ chains |
| **Historical Data** | Full (from genesis) | Full (from genesis) |
| **Real-time Streams** | Yes (limited) | Yes (unlimited) |
| **Support** | Public Telegram | 24/7 Engineering + Priority Slack/Telegram |
| **SLA** | None | Custom SLA |
| **Data Interfaces** | GraphQL only | GraphQL + SQL + Kafka + Cloud exports |
| **Onboarding** | Self-service | Dedicated onboarding |
| **Billing** | Free | Pay-as-you-go (usage-based) |

### Additional Enterprise Features
- **Cloud Integrations**: AWS, GCP, Azure, Snowflake, BigQuery, Databricks
- **Data Formats**: JSON, Protocol Buffers, Parquet, custom formats
- **Kafka Streaming**: Real-time data pipelines
- **SQL Access**: Direct SQL queries on blockchain data
- **Scheduled Exports**: Automated data exports
- **Webhook Integration**: Trigger external systems on events
- **Multiple Users**: Team access with separate tokens

---

## Datashares & Exports (Enterprise)

### Availability
- **Platforms**: Snowflake, BigQuery, S3, Azure
- **Pricing**: Custom (contact sales)
- **Data Type**: Real-time + historical blockchain data
- **Format**: Structured tables for AI agents, MCP servers, analytics

### Use Cases
- Data warehousing
- AI/ML model training
- Business intelligence (BI) tools
- Custom analytics pipelines
- Multi-chain aggregation

---

## Rate Limit Details

### How Measured
- **Window**: Per minute (rolling window likely)
- **Rolling window**: Not explicitly stated (assumed yes)
- **Fixed window**: Not confirmed

### Limit Scope
- **Per IP address**: No (limits are per API key)
- **Per API key**: Yes (10 req/min on free tier)
- **Per account**: Yes (shared across all API keys in account)
- **Shared across**: All API keys in same account

### Burst Handling
- **Burst allowed**: Yes (within reason)
- **Burst size**: Not specified (limited by points quota)
- **Burst window**: Not specified
- **Token bucket**: Not documented (likely uses sliding window)

### Response Headers
**Not documented** - Bitquery doesn't appear to return standard rate limit headers like:
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 5
X-RateLimit-Reset: 1234567890
```

### Error Response (HTTP 429)
```json
{
  "errors": [
    {
      "message": "Too Many Sessions: 429",
      "extensions": {
        "code": "RATE_LIMIT_EXCEEDED"
      }
    }
  ]
}
```

**Additional error** (when points exhausted):
```json
{
  "errors": [
    {
      "message": "Points quota exceeded. Please upgrade your plan.",
      "extensions": {
        "code": "QUOTA_EXCEEDED",
        "remaining_points": 0
      }
    }
  ]
}
```

### Handling Strategy
```python
import time
from requests.exceptions import HTTPError

def query_with_backoff(query, token, max_retries=3):
    for attempt in range(max_retries):
        try:
            response = requests.post(
                'https://streaming.bitquery.io/graphql',
                headers={'Authorization': f'Bearer {token}'},
                json={'query': query}
            )
            response.raise_for_status()
            return response.json()
        except HTTPError as e:
            if e.response.status_code == 429:
                wait_time = 2 ** attempt  # Exponential backoff: 1s, 2s, 4s
                print(f"Rate limited. Waiting {wait_time}s...")
                time.sleep(wait_time)
            elif e.response.status_code == 424:
                print("Temporary issue. Retrying in 5s...")
                time.sleep(5)
            else:
                raise
    raise Exception("Max retries exceeded")
```

---

## Points System (Credit-based Quota)

### How it Works

Bitquery uses a **points-based quota system** instead of simple rate limits.

- **Monthly quota**: 10,000 points (free tier)
- **Each query costs points**: Variable based on complexity and data volume
- **Overage**: Requests blocked when quota exhausted (no extra charges on free tier)

### Points Calculation Factors

#### 1. Dataset Type
- **Realtime**: **5 points per cube** (flat rate, regardless of query complexity or data returned)
- **Archive**: **Variable** (based on resource usage)
- **Combined**: Higher cost (not recommended)

**Example**:
```graphql
# Realtime query: 5 points (1 cube = Blocks)
{ EVM(network: eth, dataset: realtime) { Blocks { Block { Number } } } }

# Realtime query with 2 cubes: 10 points (Blocks + DEXTrades)
{ EVM(network: eth, dataset: realtime) {
    Blocks { Block { Number } }
    DEXTrades { Trade { Buy { Amount } } }
} }
```

#### 2. Query Complexity (Archive Dataset)
- **Simple queries**: Low points (e.g., 1-10 points)
- **Complex queries**: High points (e.g., 50-500+ points)
- **Factors**:
  - Number of records returned (limit parameter)
  - Time range (since/till filters)
  - Number of filters
  - Number of addresses queried
  - Metrics/aggregations used

**Example**:
```graphql
# Low cost: Limited rows, recent time range
{ EVM(network: eth, dataset: archive) {
    Blocks(
      limit: {count: 10}
      where: {Block: {Time: {since: "2024-01-01"}}}
    ) { Block { Number } }
} }

# High cost: Large time range, many rows, complex filters
{ EVM(network: eth, dataset: archive) {
    DEXTrades(
      limit: {count: 10000}
      where: {Block: {Time: {since: "2020-01-01", till: "2024-01-01"}}}
    ) { Trade { Buy { Amount Price } Sell { Amount } } }
} }
```

#### 3. Data Volume
- **Rows returned**: More rows = more points
- **Optimizing**: Use narrower time ranges or reduce `limit`

**Cost reduction strategies**:
- Reduce `limit: {count: X}`
- Narrow time range: `{Block: {Time: {since: "recent_date"}}}`
- Filter by specific addresses: `{Transfer: {Sender: {is: "0x..."}}}`

#### 4. Number of Addresses/Symbols
- **More addresses queried** = higher cost
- **Example**: Querying 100 addresses costs more than 1 address

### Subscription Points Cost

**Subscriptions (WebSocket)**:
- **Cost**: **40 points per minute per stream**
- **Calculation**: `points = streams × duration_minutes × 40`

**Examples**:
- 1 stream for 10 minutes = 400 points
- 2 streams for 10 minutes = 800 points
- 1 stream for 1 hour = 2,400 points
- 5 streams for 1 hour = 12,000 points

**Free tier math**:
- 10,000 points / (2 streams × 40 points/min) = 125 minutes total
- ~2 hours of continuous dual-stream usage per month

### Commercial Plan (New Pricing)
- **Subscriptions**: Charged per simultaneous streams (flat rate, not points)
- **Queries**: Points-based or unlimited (depending on contract)
- **Custom allocation**: Negotiate based on usage patterns

---

## Credit Costs (Archive Dataset)

**Note**: Exact point costs are not publicly documented. Based on usage patterns:

| Query Type | Estimated Cost | Notes |
|------------|----------------|-------|
| Simple block query (10 rows) | 1-5 points | Recent blocks, minimal fields |
| Token transfers (100 rows) | 10-50 points | Depends on time range |
| DEX trades (1000 rows) | 50-200 points | High complexity |
| Large historical query (10k rows) | 500-5000+ points | Very expensive |
| Realtime query (any size) | 5 points per cube | Flat rate |
| WebSocket subscription | 40 points/min/stream | Per-minute billing |

### Monitoring Points Usage

Users can see points consumption:
- **During query execution**: IDE shows points used
- **Account dashboard**: https://account.bitquery.io
  - Real-time tracking
  - Monthly statistics
  - Remaining quota
  - Usage history

---

## WebSocket Specific Limits

### Connection Limits
- **Max connections per IP**: Not specified (assumed reasonable)
- **Max connections per API key**: Not specified
- **Max connections total**: Unlimited (but streams are limited)

### Subscription Limits
- **Free tier**: 2 simultaneous streams
- **Commercial**: Unlimited simultaneous streams
- **Max subscriptions per connection**: Multiple allowed (each counts as separate stream)
- **Max symbols per subscription**: Not limited (but affects performance)

**Important**: Each **cube** in a subscription counts as a separate stream.

**Example**:
```graphql
# 1 subscription, 1 stream (only Blocks cube)
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks { Block { Number } }
  }
}

# 1 subscription, 2 streams (Blocks + DEXTrades cubes)
subscription {
  EVM(network: eth, dataset: realtime) {
    Blocks { Block { Number } }
    DEXTrades { Trade { Buy { Amount } } }
  }
}
```

On free tier (2 stream limit), the second example would consume entire quota.

### Message Rate Limits
- **Messages per second**: Not documented
- **Server may throttle**: Yes (based on plan and load)
- **Auto-disconnect on violation**: Not documented (likely yes for abuse)

### Connection Duration
- **Max lifetime**: Unlimited (if keepalives maintained)
- **Auto-reconnect needed**: Yes (implement client-side)
- **Idle timeout**: Not specified (no automatic disconnect mentioned)
- **Keepalive requirement**: Must handle `pong`/`ka` messages

---

## Monitoring Usage

### Dashboard
- **Usage dashboard**: https://account.bitquery.io
- **Real-time tracking**: Yes (points consumption shown during queries in IDE)
- **Historical usage**: Yes (monthly statistics)
- **Breakdown**: By query type, time period

### API Endpoints
**Not available** - No programmatic API to check quota/usage.

Must use account dashboard for monitoring.

### Alerts
- **Email alerts**: Not documented (likely available on commercial plan)
- **Webhook**: Not documented
- **Dashboard warnings**: Yes (when approaching quota limits)

---

## Upgrade Process

### How to Upgrade
1. **Contact Sales**: Email or use contact form on bitquery.io
2. **Discuss Requirements**: Usage patterns, data needs
3. **Custom Quote**: Pricing based on anticipated usage
4. **Sign Agreement**: Contract with SLA
5. **Billing Setup**: Credit card or invoice billing

### When to Upgrade

Upgrade if:
- Hitting 10 req/min rate limit frequently
- Exhausting 10,000 points/month quota
- Need more than 2 simultaneous WebSocket streams
- Need more than 10 rows per request
- Require enterprise features (SQL, Kafka, cloud exports)
- Need dedicated support with SLA

### Commercial Plan Benefits
- **No throttling**: Scalable API calls
- **Unlimited streams**: As many subscriptions as needed
- **Custom points**: Negotiate allocation based on usage
- **Enterprise features**: SQL, Kafka, cloud integrations
- **Priority support**: 24/7 engineering team access
- **Custom SLA**: Guaranteed uptime and response times
- **Dedicated onboarding**: Setup assistance

---

## Cost Optimization Strategies

### 1. Minimize Points Usage
- Use `limit: {count: X}` to reduce data returned
- Narrow time ranges: `{Block: {Time: {since: "recent_date"}}}`
- Filter by specific entities: `{Transfer: {Currency: {SmartContract: {is: "0x..."}}}}`
- Use realtime dataset for subscriptions (flat 5 points/cube)

### 2. Batch Queries
- Combine multiple data requests into single query
- Use GraphQL fragments for reusable query parts

### 3. Cache Results
- Store frequently accessed data locally
- Refresh only when needed (not on every request)

### 4. Use Subscriptions Wisely
- Free tier: 2 streams = ~2 hours/month continuous use
- Commercial: Unlimited, but still billed per stream
- Close subscriptions when not actively needed

### 5. Query Optimization
- Request only needed fields (GraphQL benefit)
- Avoid overly broad queries (e.g., all DEX trades for entire year)
- Use pagination for large datasets

---

## Comparison: Free vs Commercial

| Metric | Free (Developer) | Commercial |
|--------|------------------|------------|
| **Monthly Cost** | $0 | Custom (likely $500-$5000+/month) |
| **Points** | 10K trial | Custom (10M+ typical) |
| **Rate Limit** | 10 req/min | No limit (scalable) |
| **Rows/Query** | 10 | Unlimited |
| **WebSocket Streams** | 2 | Unlimited |
| **Support** | Community (Telegram) | 24/7 Engineering + Slack |
| **SLA** | None | Custom SLA |
| **Data Export** | GraphQL only | GraphQL + SQL + Kafka + Cloud |
| **Use Case** | Testing, learning, small projects | Production, commercial apps, analytics |

---

## Fair Use Policy

While not explicitly documented, expect:
- **No abuse**: Don't hammer API excessively
- **No scraping**: Don't download entire blockchain datasets
- **Respect limits**: Stay within quota and rate limits
- **Commercial use**: Must upgrade to paid plan

Violating fair use may result in:
- Account suspension
- IP blocking
- API key revocation

---

## Additional Notes

1. **Trial period**: First month includes 10K points - evaluate usage patterns
2. **No automatic upgrade**: Free tier doesn't auto-charge if quota exceeded
3. **Points don't roll over**: Unused points expire monthly (on free tier)
4. **Commercial pricing**: Negotiate based on projected usage (not fixed tiers)
5. **Academic/Research**: May qualify for discounts (contact sales)
6. **Multiple accounts**: One free account per email (terms likely prohibit multi-accounting)
